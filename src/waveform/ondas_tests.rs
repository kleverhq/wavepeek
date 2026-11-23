use std::io::Write;
use std::path::Path;
use std::process::Command;

use tempfile::NamedTempFile;

use super::{OndasBackend, scope_type_alias, var_type_alias};
use crate::waveform::{
    EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES, STABLE_SCOPE_KIND_ALIASES,
    STABLE_SIGNAL_KIND_ALIASES, SampledSignal, ScopeEntry, Waveform,
    duplicate_preserving_projection,
};

const TEST_VCD: &str = "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var reg 8 \" data $end\n$var parameter 8 # cfg $end\n$scope module cpu $end\n$var wire 1 $ valid $end\n$upscope $end\n$scope function helper $end\n$var wire 1 & helper_flag $end\n$upscope $end\n$scope module mem $end\n$var wire 1 % ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nb00000000 \"\nb10101010 #\n0$\n0&\n0%\n#5\n1!\n1$\n1&\n#10\nb00001111 \"\n1%\n";

const RICH_VALUE_VCD: &str = "$date\n  2026-03-12\n$end\n$version\n  wavepeek-rich-value\n$end\n$timescale 1ns $end\n$scope module top $end\n$var real 1 ! temp $end\n$var string 1 \" msg $end\n$upscope $end\n$enddefinitions $end\n#0\nr1.5 !\nsgo \"\n";

const DERIVED_SPLIT_VCD: &str = "$date\n  2026-07-26\n$end\n$version\n  wavepeek-derived-split\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! split [0] $end\n$var wire 1 \" split [1] $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n#10\n1\"\n#15\n0!\n#20\n0\"\n";

const RECURSIVE_TEST_VCD: &str = "$date\n  2026-02-28\n$end\n$version\n  wavepeek-recursive-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$scope module cpu $end\n$var wire 1 \" valid $end\n$scope module core $end\n$var wire 1 # execute $end\n$upscope $end\n$upscope $end\n$scope module mem $end\n$var wire 1 $ ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n0$\n#5\n1!\n1\"\n1#\n1$\n";

const DELAYED_VALUE_VCD: &str = "$date\n  2026-03-03\n$end\n$version\n  wavepeek-delayed-value\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! delayed $end\n$upscope $end\n$enddefinitions $end\n#0\n#5\n1!\n";

#[test]
fn expression_metadata_overrides_kind_defaults_without_changing_width() {
    use crate::expr::{ExprTypeKind, IntegerLikeKind};
    use ondas::{Encoding, LogicDomain, Signedness};

    let bits = Encoding::Bits { width: 7 };
    let ty = super::expression_value_type(
        bits,
        "int",
        Some(Signedness::Unsigned),
        Some(LogicDomain::TwoState),
        false,
    )
    .unwrap();
    assert_eq!(ty.kind, ExprTypeKind::IntegerLike(IntegerLikeKind::Int));
    assert_eq!(ty.width, 7);
    assert!(!ty.is_signed);
    assert!(!ty.is_four_state);

    for kind in [
        "byte",
        "shortint",
        "short-int",
        "short_int",
        "int",
        "longint",
        "long-int",
        "long_int",
        "integer",
    ] {
        let ty = super::expression_value_type(bits, kind, None, None, false).unwrap();
        assert_eq!(ty.width, 7, "{kind}");
        assert!(ty.is_signed, "{kind}");
        assert!(ty.is_four_state, "unknown domain: {kind}");
    }
    for kind in ["wire", "bit", "boolean", "vendor:unknown", "time"] {
        let ty = super::expression_value_type(bits, kind, None, None, false).unwrap();
        assert!(!ty.is_signed, "{kind}");
        assert!(ty.is_four_state, "{kind}");
    }
    let enumeration = super::expression_value_type(
        bits,
        "wire",
        Some(Signedness::Signed),
        Some(LogicDomain::NineState),
        true,
    )
    .unwrap();
    assert_eq!(enumeration.kind, ExprTypeKind::EnumCore);
    assert_eq!(enumeration.width, 7);
    assert!(enumeration.is_signed && enumeration.is_four_state);
    for (encoding, expected) in [
        (Encoding::Real, ExprTypeKind::Real),
        (Encoding::String, ExprTypeKind::String),
        (Encoding::Event, ExprTypeKind::Event),
    ] {
        let ty = super::expression_value_type(encoding, "integer", None, None, false).unwrap();
        assert_eq!(ty.kind, expected);
        assert!(!ty.is_four_state && !ty.is_signed);
    }
}

#[test]
fn unknown_vector_metadata_uses_language_defaults_and_preserves_missing() {
    use crate::expr::{
        ExprValuePayload, ExpressionHost, bind_logical_expr_ast, eval_logical_expr_at,
        parse_logical_expr_ast,
    };
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! data $end\n$upscope $end\n$enddefinitions $end\n#0\n#1\nbxxxxxxxx !\n#2\nb11111111 !\n",
        "unknown-metadata.vcd",
    );
    let raw = ondas::open(fixture.path()).unwrap();
    let variable = raw.hierarchy().variable("top.data").unwrap();
    assert_eq!(variable.signedness(), None);
    assert_eq!(variable.logic_domain(), None);
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let resolved = waveform.resolve_expr_signal("top.data").unwrap();
    assert_eq!(
        waveform.sample_expr_value(&resolved, 0).unwrap(),
        crate::expr::SampledValue::Integral {
            bits: None,
            label: None
        }
    );
    assert_eq!(
        waveform.sample_expr_value(&resolved, 1).unwrap(),
        crate::expr::SampledValue::Integral {
            bits: Some("xxxxxxxx".into()),
            label: None
        }
    );
    let host = crate::waveform::expr_host::WaveformExprHost::new(waveform);
    for (source, expected) in [
        ("top.data < 0", "0"),
        ("signed'(top.data) < 0", "1"),
        ("top.data === 8'hxx", "1"),
    ] {
        let expr = bind_logical_expr_ast(&parse_logical_expr_ast(source).unwrap(), &host).unwrap();
        let time = if source.contains("===") { 1 } else { 2 };
        assert_eq!(
            eval_logical_expr_at(&expr, &host, time).unwrap().payload,
            ExprValuePayload::Integral {
                bits: expected.into(),
                label: None
            }
        );
    }
    let handle = host.resolve_signal("top.data").unwrap();
    assert!(host.signal_type(handle).unwrap().is_four_state);
    assert_eq!(variable.signedness(), None);
}

#[test]
fn global_matching_preserves_scope_order_and_split_width() {
    let fixture = write_fixture(TEST_VCD, "global-matching.vcd");
    let waveform = Waveform::open(fixture.path()).unwrap();
    let matched = waveform.matching_signals(|_, name| name != "data");
    assert_eq!(
        matched
            .iter()
            .map(|entry| entry.path.to_string())
            .collect::<Vec<_>>(),
        [
            "top.cfg",
            "top.clk",
            "top.cpu.valid",
            "top.helper.helper_flag",
            "top.mem.ready"
        ]
    );
    waveform.signals_in_scope("top").unwrap();
    assert_eq!(waveform.matching_signals(|_, name| name != "data"), matched);

    let fixture = write_fixture(DERIVED_SPLIT_VCD, "global-split.vcd");
    let waveform = Waveform::open(fixture.path()).unwrap();
    let entries = waveform.matching_signals(|path, _| path == "top.split");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].width, Some(2));

    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! memory[0] [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000001 !\n",
        "global-array.vcd",
    );
    let waveform = Waveform::open(fixture.path()).unwrap();
    let entries = waveform.matching_signals(|_, name| name == "memory[0]");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path.to_string(), "top.memory[0]");
    assert_eq!(entries[0].width, Some(8));
    waveform.signals_in_scope("top").unwrap();
    assert_eq!(
        waveform.matching_signals(|_, name| name == "memory[0]"),
        entries
    );
}

#[test]
fn open_and_read_metadata_from_vcd() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let metadata = waveform.metadata().expect("metadata should be available");

    assert_eq!(metadata.time_unit, "1ns");
    assert_eq!(metadata.time_start, "0ns");
    assert_eq!(metadata.time_end, "10ns");
}

#[test]
fn vcd_backslash_components_do_not_alias_separator_components() {
    let source = "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! \\a.b $end\n$scope module \\\\a $end\n$var wire 1 \" b $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n";
    let fixture = write_fixture(source, "backslash-components.vcd");
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let listing = waveform
        .signals_in_scope_recursive_report("top", None)
        .unwrap();
    let paths = [r#"top."a.b""#, r#"top."\\a".b"#].map(str::to_owned);
    assert_eq!(
        listing
            .entries
            .iter()
            .map(|entry| entry.path.to_string())
            .collect::<Vec<_>>(),
        paths
    );
    assert!(listing.omitted_ambiguous_paths.is_empty());
    let values = waveform.sample_signals_at_time(&paths, 0).unwrap();
    assert_eq!(
        values
            .iter()
            .map(|value| value.bits.as_str())
            .collect::<Vec<_>>(),
        ["0", "1"]
    );
}

#[test]
fn vcd_preserves_escaped_simple_declaration_spelling() {
    let source = "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! \\plain $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n";
    let fixture = write_fixture(source, "escaped-simple.vcd");
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let entries = waveform.signals_in_scope("top").unwrap();
    assert_eq!(entries[0].name, "plain");
    assert_eq!(entries[0].path.to_string(), "top.plain");
    let values = waveform
        .sample_signals_at_time(&["top.plain".into()], 0)
        .unwrap();
    assert_eq!(values[0].bits, "0");
}

#[test]
fn vcd_escaped_components_remain_distinct_from_nested_paths() {
    let source = "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! \\foo.bar $end\n$var wire 1 \" \\foo/bar $end\n$scope module foo $end\n$var wire 1 # bar $end\n$upscope $end\n$scope module \\block.dot $end\n$var wire 1 $ value $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n1#\n1$\n";
    let fixture = write_fixture(source, "escaped-components.vcd");
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let listing = waveform
        .signals_in_scope_recursive_report("top", None)
        .unwrap();
    let paths = [
        r#"top."foo.bar""#,
        r#"top."foo/bar""#,
        r#"top."block.dot".value"#,
        "top.foo.bar",
    ]
    .map(str::to_owned);
    assert_eq!(
        listing
            .entries
            .iter()
            .map(|entry| entry.path.to_string())
            .collect::<Vec<_>>(),
        paths
    );
    assert!(listing.omitted_ambiguous_paths.is_empty());
    let bounded = waveform
        .signals_in_scope_recursive_report("top", Some(1))
        .unwrap();
    assert_eq!(bounded.entries, listing.entries);
    assert_eq!(
        waveform.signals_in_scope(r#"top."block.dot""#).unwrap()[0].name,
        "value"
    );
    let values = waveform.sample_signals_at_time(&paths, 0).unwrap();
    assert_eq!(
        values
            .iter()
            .map(|value| value.bits.as_str())
            .collect::<Vec<_>>(),
        ["0", "1", "1", "1"]
    );
}

#[test]
fn scope_and_signal_preorder_keeps_descendants_before_prefix_siblings() {
    let source = "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! root $end\n$scope module a $end\n$var wire 1 \" base $end\n$scope module z $end\n$var wire 1 # deep $end\n$upscope $end\n$upscope $end\n$scope module a$ $end\n$var wire 1 $ tail $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n0$\n";
    let fixture = write_fixture(source, "preorder.vcd");
    let waveform = Waveform::open(fixture.path()).unwrap();
    let scopes = waveform.scopes_depth_first(None).unwrap();
    assert_eq!(
        scopes
            .iter()
            .map(|scope| scope.path.to_string())
            .collect::<Vec<_>>(),
        ["top", "top.a", "top.a.z", r#"top."a$""#]
    );
    let signals = waveform
        .signals_in_scope_recursive_report("top", None)
        .unwrap();
    assert_eq!(
        signals
            .entries
            .iter()
            .map(|signal| signal.path.to_string())
            .collect::<Vec<_>>(),
        ["top.root", "top.a.base", "top.a.z.deep", r#"top."a$".tail"#]
    );
}

#[test]
fn scopes_use_deterministic_depth_first_lexicographic_order_with_kind() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let scopes = waveform
        .scopes_depth_first(Some(5))
        .expect("scope traversal should succeed");

    assert_eq!(
        scopes,
        vec![
            ScopeEntry {
                path: ondas::HierarchyPath::parse("top").unwrap(),
                depth: 0,
                kind: "module".to_string()
            },
            ScopeEntry {
                path: ondas::HierarchyPath::parse("top.cpu").unwrap(),
                depth: 1,
                kind: "module".to_string()
            },
            ScopeEntry {
                path: ondas::HierarchyPath::parse("top.helper").unwrap(),
                depth: 1,
                kind: "function".to_string()
            },
            ScopeEntry {
                path: ondas::HierarchyPath::parse("top.mem").unwrap(),
                depth: 1,
                kind: "module".to_string()
            },
        ]
    );
}

#[test]
fn scopes_depth_first_none_includes_all_nested_depths() {
    let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let bounded = waveform
        .scopes_depth_first(Some(1))
        .expect("bounded scope traversal should succeed");
    let unbounded = waveform
        .scopes_depth_first(None)
        .expect("unbounded scope traversal should succeed");

    let bounded_paths = bounded
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();
    let unbounded_paths = unbounded
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();

    assert_eq!(bounded_paths, vec!["top", "top.cpu", "top.mem"]);
    assert_eq!(
        unbounded_paths,
        vec!["top", "top.cpu", "top.cpu.core", "top.mem"]
    );
}

#[test]
fn signals_in_scope_are_sorted_and_preserve_parser_var_type_aliases() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let signals = waveform
        .signals_in_scope("top")
        .expect("scope lookup should succeed");

    assert_eq!(signals.len(), 3);
    assert_eq!(signals[0].name, "cfg");
    assert_eq!(signals[0].path.to_string(), "top.cfg");
    assert_eq!(signals[0].kind, "parameter");
    assert_eq!(signals[1].name, "clk");
    assert_eq!(signals[1].kind, "wire");
    assert_eq!(signals[2].name, "data");
    assert_eq!(signals[2].kind, "reg");
}

#[test]
fn missing_scope_returns_scope_category_error() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let error = waveform
        .signals_in_scope("top.nope")
        .expect_err("unknown scope should fail");

    assert_eq!(
        error.to_string(),
        "fatal: scope: scope 'top.nope' not found in dump"
    );
    assert_eq!(error.exit_code(), 1);
}

#[test]
fn recursive_signals_in_scope_respect_depth_boundaries() {
    let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let depth_0 = waveform
        .signals_in_scope_recursive("top", Some(0))
        .expect("depth-0 lookup should succeed");
    let depth_1 = waveform
        .signals_in_scope_recursive("top", Some(1))
        .expect("depth-1 lookup should succeed");
    let depth_2 = waveform
        .signals_in_scope_recursive("top", Some(2))
        .expect("depth-2 lookup should succeed");

    let depth_0_paths = depth_0
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();
    let depth_1_paths = depth_1
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();
    let depth_2_paths = depth_2
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();

    assert_eq!(depth_0_paths, vec!["top.clk"]);
    assert_eq!(
        depth_1_paths,
        vec!["top.clk", "top.cpu.valid", "top.mem.ready"]
    );
    assert_eq!(
        depth_2_paths,
        vec![
            "top.clk",
            "top.cpu.valid",
            "top.cpu.core.execute",
            "top.mem.ready"
        ]
    );
}

#[test]
fn recursive_signals_in_scope_none_depth_includes_all_nested_levels() {
    let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let bounded = waveform
        .signals_in_scope_recursive("top", Some(1))
        .expect("bounded lookup should succeed");
    let unbounded = waveform
        .signals_in_scope_recursive("top", None)
        .expect("unbounded lookup should succeed");

    let bounded_paths = bounded
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();
    let unbounded_paths = unbounded
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        bounded_paths,
        vec!["top.clk", "top.cpu.valid", "top.mem.ready"]
    );
    assert_eq!(
        unbounded_paths,
        vec![
            "top.clk",
            "top.cpu.valid",
            "top.cpu.core.execute",
            "top.mem.ready"
        ]
    );
}

#[test]
fn recursive_signals_in_scope_are_deterministic_depth_first() {
    let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let first = waveform
        .signals_in_scope_recursive("top", Some(2))
        .expect("first recursive lookup should succeed");
    let second = waveform
        .signals_in_scope_recursive("top", Some(2))
        .expect("second recursive lookup should succeed");

    assert_eq!(first, second);
    let ordered_paths = first
        .iter()
        .map(|entry| entry.path.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        ordered_paths,
        vec![
            "top.clk",
            "top.cpu.valid",
            "top.cpu.core.execute",
            "top.mem.ready"
        ]
    );
}

#[test]
fn open_missing_file_maps_to_file_error() {
    let error = Waveform::open(Path::new("/tmp/this-file-does-not-exist.vcd"))
        .expect_err("missing file should fail");

    assert!(error.to_string().starts_with("fatal: file: cannot open"));
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn parse_failures_map_to_file_error() {
    let fixture = write_fixture("not-a-waveform", "invalid.wave");

    let error = Waveform::open(fixture.path()).expect_err("invalid file should fail");

    assert!(error.to_string().starts_with("fatal: file: cannot parse"));
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn sample_signals_at_time_preserves_order_and_duplicates() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let sampled = waveform
        .sample_signals_at_time(
            &[
                "top.clk".to_string(),
                "top.clk".to_string(),
                "top.data".to_string(),
            ],
            10,
        )
        .expect("sampling should succeed");

    assert_eq!(
        sampled,
        vec![
            SampledSignal {
                path: "top.clk".to_string(),
                width: 1,
                bits: "1".to_string()
            },
            SampledSignal {
                path: "top.clk".to_string(),
                width: 1,
                bits: "1".to_string()
            },
            SampledSignal {
                path: "top.data".to_string(),
                width: 8,
                bits: "00001111".to_string()
            },
        ]
    );
}

#[test]
fn sample_signals_at_time_stays_non_bit_vector_for_rich_values() {
    let fixture = write_fixture(RICH_VALUE_VCD, "rich-sample.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let error = waveform
        .sample_signals_at_time(&["top.temp".to_string()], 0)
        .expect_err("rich real sampling should stay on the legacy CLI rejection path");

    assert_eq!(
        error.to_string(),
        "fatal: signal: signal 'top.temp' has unsupported non-bit-vector encoding"
    );
}

#[test]
fn duplicate_projection_deduplicates_paths_and_tracks_requested_order() {
    let (unique_paths, projection) = duplicate_preserving_projection(&[
        "top.clk".to_string(),
        "top.data".to_string(),
        "top.clk".to_string(),
        "top.cpu.valid".to_string(),
        "top.data".to_string(),
    ]);

    assert_eq!(
        unique_paths,
        vec![
            "top.clk".to_string(),
            "top.data".to_string(),
            "top.cpu.valid".to_string()
        ]
    );
    assert_eq!(projection, vec![0, 1, 0, 2, 1]);
}

#[test]
fn resolved_signal_handles_are_stable_across_resolution_paths() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.clk".to_string(), "top.clk".to_string()])
        .expect("signal should resolve");
    let expr_resolved = waveform
        .resolve_expr_signal("top.clk")
        .expect("expression signal should resolve");

    assert_eq!(resolved[0].signal, resolved[1].signal);
    assert_eq!(resolved[0].signal, expr_resolved.signal);
}

#[test]
fn composition_point_samples_preserve_part_updates() {
    let fixture = write_fixture(DERIVED_SPLIT_VCD, "derived-split.vcd");
    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.split".into()])
        .expect("split vector resolves");
    assert_eq!(resolved[0].width, 2);
    assert_eq!(
        waveform.sample_resolved_optional(&resolved, 5).unwrap()[0]
            .bits
            .as_deref(),
        Some("01")
    );
    assert_eq!(
        waveform.sample_resolved_optional(&resolved, 10).unwrap()[0]
            .bits
            .as_deref(),
        Some("11")
    );
}

#[test]
fn inline_indices_without_range_metadata_remain_exact_declarations() {
    let vcd = DERIVED_SPLIT_VCD.replace("split [", "split[");
    let fixture = write_fixture(&vcd, "inline-split.vcd");
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let signals = waveform.signals_in_scope("top").unwrap();
    assert_eq!(
        signals
            .iter()
            .map(|entry| entry.path.to_string())
            .collect::<Vec<_>>(),
        ["top.split[0]", "top.split[1]"]
    );
    assert!(waveform.resolve_signals(&["top.split".into()]).is_err());
    for entry in signals {
        let values = waveform
            .sample_signals_at_time(&[entry.path.to_string()], 10)
            .unwrap();
        assert_eq!(values[0].bits, "1");
    }
}

#[test]
fn fst_queries_share_file_and_byte_loading() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let mut file = OndasBackend::open(&path).expect("FST opens");
    let mut bytes = OndasBackend::open_bytes(&path, std::fs::read(&path).unwrap().into())
        .expect("FST bytes open");
    assert_eq!(file.metadata().unwrap(), bytes.metadata().unwrap());
    assert_eq!(
        file.scopes_depth_first(None).unwrap(),
        bytes.scopes_depth_first(None).unwrap()
    );
    let paths = ["top.clk".into()];
    let file_signals = file.resolve_signals(&paths).unwrap();
    let byte_signals = bytes.resolve_signals(&paths).unwrap();
    assert_eq!(
        file.sample_resolved_optional(&file_signals, 10).unwrap(),
        bytes.sample_resolved_optional(&byte_signals, 10).unwrap()
    );
}

#[test]
fn fst_point_handles_remain_valid_after_listing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let mut backend = OndasBackend::open(&path).unwrap();
    let paths = ["top.clk".to_string()];
    let selected = backend.resolve_signals(&paths).unwrap();
    let sampled = backend.sample_resolved_optional(&selected, 10).unwrap();

    let mut full = OndasBackend::open(&path).unwrap();
    full.scopes_depth_first(None).unwrap();
    let expected = full.sample_resolved_optional(&full.resolve_signals(&paths).unwrap(), 10);
    assert_eq!(sampled, expected.unwrap());

    let mut selected = selected;
    selected.extend(backend.resolve_signals(&["top.data".to_string()]).unwrap());
    let both = backend.sample_resolved_optional(&selected, 10).unwrap();
    backend.signals_in_scope("top").unwrap();
    assert_eq!(
        backend.sample_resolved_optional(&selected, 10).unwrap(),
        both
    );
}

#[test]
fn fst_batch_handles_remain_valid_after_listing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let mut backend = OndasBackend::open(&path).unwrap();
    let paths = ["top.clk".to_string(), "top.data".to_string()];
    let resolved = backend.resolve_signals(&paths).unwrap();
    let sampled = backend.sample_resolved_optional(&resolved, 10).unwrap();
    backend.signals_in_scope("top").unwrap();
    assert_eq!(
        backend.sample_resolved_optional(&resolved, 10).unwrap(),
        sampled
    );

    let mut full = OndasBackend::open(&path).unwrap();
    full.signals_in_scope("top").unwrap();
    let expected = full.sample_resolved_optional(&full.resolve_signals(&paths).unwrap(), 10);
    assert_eq!(sampled, expected.unwrap());
}

#[test]
fn fst_batch_selects_same_leaf_under_distinct_parents() {
    let source = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$scope module left $end\n$var wire 1 ! bit $end\n$upscope $end\n$scope module right $end\n$var wire 1 \" bit $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n",
        "siblings.vcd",
    );
    let dir = tempfile::tempdir().unwrap();
    let fst = dir.path().join("siblings.fst");
    assert!(
        Command::new("vcd2fst")
            .arg(source.path())
            .arg(&fst)
            .status()
            .unwrap()
            .success()
    );

    let paths = ["top.left.bit".into(), "top.right.bit".into()];
    let mut batch = OndasBackend::open(&fst).unwrap();
    let selected = batch.resolve_signals(&paths).unwrap();
    assert_ne!(selected[0].signal, selected[1].signal);
    let actual = batch.sample_resolved_optional(&selected, 0).unwrap();

    let mut full = OndasBackend::open(&fst).unwrap();
    full.signals_in_scope("top").unwrap();
    let expected = full.sample_resolved_optional(&full.resolve_signals(&paths).unwrap(), 0);
    assert_eq!(actual, expected.unwrap());
}

#[test]
fn fst_point_samples_handle_reverse_and_repeated_times() {
    let source = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! bit $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n#5\n0!\n#5\n1!\n#10\n0!\n",
        "same-tick.vcd",
    );
    let dir = tempfile::tempdir().unwrap();
    let fst = dir.path().join("same-tick.fst");
    assert!(
        Command::new("vcd2fst")
            .arg(source.path())
            .arg(&fst)
            .status()
            .unwrap()
            .success()
    );

    let mut backend = OndasBackend::open(&fst).unwrap();
    let resolved = backend.resolve_signals(&["top.bit".into()]).unwrap();
    for (time, expected) in [(5, "1"), (5, "1"), (4, "0"), (10, "0"), (0, "0"), (5, "1")] {
        let sampled = backend.sample_resolved_optional(&resolved, time).unwrap();
        assert_eq!(sampled[0].bits.as_deref(), Some(expected), "at tick {time}");
    }
}

#[test]
fn indexed_names_use_public_exact_path_spelling() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! memory[0] [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000001 !\n",
        "array.vcd",
    );
    let wave = ondas::open(fixture.path()).unwrap();
    let variable = wave.hierarchy().variables().next().unwrap();
    let direct = variable.path().to_string();
    assert_eq!(direct, "top.memory[0]");
    let backend = OndasBackend::open(fixture.path()).unwrap();
    assert_eq!(
        backend.signals_in_scope("top").unwrap()[0].path.to_string(),
        direct
    );
}

#[test]
fn fst_indexed_declaration_and_nested_bracket_name_remain_distinct() {
    let source = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! memory[0] [7:0] $end\n$scope module memory $end\n$var wire 8 \" [0] [7:0] $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000001 !\nb00000010 \"\n",
        "collision.vcd",
    );
    let dir = tempfile::tempdir().unwrap();
    let fst = dir.path().join("collision.fst");
    assert!(
        Command::new("vcd2fst")
            .arg(source.path())
            .arg(&fst)
            .status()
            .unwrap()
            .success()
    );
    let mut backend = OndasBackend::open(&fst).unwrap();
    let paths = ["top.memory[0]".into(), r#"top.memory."[0]""#.into()];
    let signals = backend.resolve_signals(&paths).unwrap();
    let values = backend.sample_resolved_optional(&signals, 0).unwrap();
    assert_eq!(values[0].bits.as_deref(), Some("00000001"));
    assert_eq!(values[1].bits.as_deref(), Some("00000010"));
}

#[test]
fn fst_expression_handles_remain_valid_after_listing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let mut backend = OndasBackend::open(&path).unwrap();
    let signal = backend.resolve_expr_signal("top.clk", None).unwrap();
    let sampled = backend.sample_expr_value(&signal, 10).unwrap();

    let mut full = OndasBackend::open(&path).unwrap();
    full.signals_in_scope("top").unwrap();
    let expected = full.sample_expr_value(&full.resolve_expr_signal("top.clk", None).unwrap(), 10);
    assert_eq!(sampled, expected.unwrap());
}

#[test]
fn sample_signals_at_time_uses_latest_change_before_timestamp() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let sampled = waveform
        .sample_signals_at_time(&["top.data".to_string()], 7)
        .expect("sampling should succeed");

    assert_eq!(sampled[0].width, 8);
    assert_eq!(sampled[0].bits, "00000000");
}

#[test]
fn sample_signals_at_time_returns_signal_error_for_missing_path() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let error = waveform
        .sample_signals_at_time(&["top.nope".to_string()], 10)
        .expect_err("missing signal should fail");

    assert_eq!(
        error.to_string(),
        "fatal: signal: signal 'top.nope' not found in dump"
    );
    assert_eq!(error.exit_code(), 1);
    assert!(matches!(
        error,
        crate::error::WavepeekError::SignalNotFound(_)
    ));
}

#[test]
fn sample_signals_at_time_errors_when_signal_has_no_prior_value() {
    let fixture = write_fixture(DELAYED_VALUE_VCD, "delayed.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let error = waveform
        .sample_signals_at_time(&["top.delayed".to_string()], 0)
        .expect_err("missing prior value should fail");

    assert_eq!(
        error.to_string(),
        "fatal: signal: signal 'top.delayed' has no value at or before requested time"
    );
}
#[test]
fn stable_kind_aliases_cover_full_inventory() {
    let scope_cases = STABLE_SCOPE_KIND_ALIASES
        .iter()
        .map(|kind| (*kind, *kind))
        .chain(
            EXCLUDED_SCOPE_KIND_ALIASES
                .iter()
                .map(|kind| (*kind, "unknown")),
        );
    for (scope_type, expected_alias) in scope_cases {
        let alias = scope_type_alias(scope_type);
        assert_eq!(
            alias, expected_alias,
            "unexpected scope alias for {scope_type:?}"
        );
        assert!(
            STABLE_SCOPE_KIND_ALIASES.contains(&alias.as_str()),
            "scope alias {alias:?} for {scope_type:?} escaped the stable inventory"
        );
    }
    for alias in EXCLUDED_SCOPE_KIND_ALIASES {
        assert!(
            !STABLE_SCOPE_KIND_ALIASES.contains(alias),
            "excluded scope alias {alias:?} leaked into the stable inventory"
        );
    }

    let signal_cases = STABLE_SIGNAL_KIND_ALIASES
        .iter()
        .map(|kind| (*kind, *kind))
        .chain([
            ("sparsearray", "sparse_array"),
            ("realparameter", "real_parameter"),
            ("realtime", "real_time"),
            ("shortint", "short_int"),
            ("longint", "long_int"),
            ("shortreal", "short_real"),
            ("std_logic", "logic"),
            ("std_ulogic", "logic"),
            ("std_logic_vector", "bit_vector"),
            ("std_ulogic_vector", "bit_vector"),
            ("std-logic", "logic"),
            ("std-logic-vector", "bit_vector"),
            ("short-int", "short_int"),
            ("long-int", "long_int"),
            ("stdlogic", "logic"),
            ("stdulogic", "logic"),
            ("stdlogicvector", "bit_vector"),
            ("stdulogicvector", "bit_vector"),
            ("bitvector", "bit_vector"),
        ]);
    for (var_type, expected_alias) in signal_cases {
        let alias = var_type_alias(var_type);
        assert_eq!(
            alias, expected_alias,
            "unexpected signal alias for {var_type:?}"
        );
        assert!(
            STABLE_SIGNAL_KIND_ALIASES.contains(&alias.as_str()),
            "signal alias {alias:?} for {var_type:?} escaped the stable inventory"
        );
    }
    for alias in EXCLUDED_SIGNAL_KIND_ALIASES {
        assert!(
            !STABLE_SIGNAL_KIND_ALIASES.contains(alias),
            "excluded signal alias {alias:?} leaked into the stable inventory"
        );
    }
}

const TYPE_SURFACE_VCD: &str = concat!(
    "$date\n  2026-05-17\n$end\n",
    "$version\n  wavepeek-type-surface\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var byte 8 ! bytev $end\n",
    "$var shortint 16 \" shortv $end\n",
    "$var int 32 # intv $end\n",
    "$var longint 64 $ longv $end\n",
    "$var integer 32 % integerv $end\n",
    "$var time 64 & timeval $end\n",
    "$var real 1 ' realv $end\n",
    "$var string 1 ( strv $end\n",
    "$var event 1 ) ev $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "b00000001 !\n",
    "b0000000000000010 \"\n",
    "b00000000000000000000000000000011 #\n",
    "b0000000000000000000000000000000000000000000000000000000000000100 $\n",
    "b00000000000000000000000000000101 %\n",
    "b0000000000000000000000000000000000000000000000000000000000000110 &\n",
    "r2.5 '\n",
    "shello (\n",
    "#5\n",
    "1)\n"
);

#[test]
fn expr_resolution_and_sampling_exercise_real_string_event_paths() {
    const EXPR_VCD: &str = concat!(
        "$date\n  today\n$end\n",
        "$version\n  wavepeek-test\n$end\n",
        "$timescale 1ns $end\n",
        "$scope module top $end\n",
        "$var wire 1 ! sig $end\n",
        "$var event 1 \" ev $end\n",
        "$var real 1 # temp $end\n",
        "$var string 1 $ msg $end\n",
        "$var event 1 % second_event $end\n",
        "$upscope $end\n",
        "$enddefinitions $end\n",
        "#0\n",
        "0!\n",
        "r0.0 #\n",
        "shello $\n",
        "#5\n",
        "1!\n",
        "1\"\n",
        "r2.5 #\n",
        "sworld $\n",
        "1%\n"
    );

    let fixture = write_fixture(EXPR_VCD, "expr-sample.vcd");
    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");

    let real = waveform
        .resolve_expr_signal("top.temp")
        .expect("real signal should resolve");
    assert!(matches!(
        real.expr_type.kind,
        crate::expr::ExprTypeKind::Real
    ));
    assert_eq!(
        waveform
            .sample_expr_value(&real, 5)
            .expect("real value should sample"),
        crate::expr::SampledValue::Real { value: Some(2.5) }
    );

    let string = waveform
        .resolve_expr_signal("top.msg")
        .expect("string signal should resolve");
    assert!(matches!(
        string.expr_type.kind,
        crate::expr::ExprTypeKind::String
    ));
    assert_eq!(
        waveform
            .sample_expr_value(&string, 5)
            .expect("string value should sample"),
        crate::expr::SampledValue::String {
            value: Some("world".to_string())
        }
    );

    let event = waveform
        .resolve_expr_signal("top.ev")
        .expect("event signal should resolve");
    assert!(matches!(
        event.expr_type.kind,
        crate::expr::ExprTypeKind::Event
    ));
    assert!(
        waveform
            .expr_event_occurred(&event, 5)
            .expect("event should occur")
    );
    assert!(
        !waveform
            .expr_event_occurred(&event, 4)
            .expect("non-sampled event timestamp should be false")
    );
    assert!(
        waveform
            .sample_expr_value(&event, 5)
            .expect_err("events cannot be sampled as values")
            .to_string()
            .contains("is a raw event and cannot be sampled as a value")
    );

    let second_event = waveform
        .resolve_expr_signal("top.second_event")
        .expect("second event should resolve");
    assert!(matches!(
        second_event.expr_type.kind,
        crate::expr::ExprTypeKind::Event
    ));
    assert!(
        waveform
            .expr_event_occurred(&second_event, 5)
            .expect("second event should occur")
    );

    let signal = waveform
        .resolve_expr_signal("top.sig")
        .expect("bit-vector signal should resolve");
    assert!(
        waveform
            .expr_event_occurred(&signal, 5)
            .expect_err("non-events cannot be queried as events")
            .to_string()
            .contains("is not a raw event")
    );
}

#[test]
fn scope_kind_aliases_normalize_interface_references() {
    for kind in [
        "interface",
        "modport",
        "modport-ref",
        "modport_ref",
        "interface-port-ref",
        "interface_port_ref",
    ] {
        assert_eq!(scope_type_alias(kind), "interface", "{kind}");
    }
    assert_eq!(scope_type_alias("vendor-scope"), "unknown");
}

#[test]
fn fsdb_kind_aliases_stay_inside_stable_contract() {
    for kind in [
        "module",
        "task",
        "function",
        "begin",
        "fork",
        "generate",
        "struct",
        "union",
        "class",
        "interface",
        "package",
        "program",
        "unknown",
    ] {
        let alias = scope_type_alias(kind);
        assert!(STABLE_SCOPE_KIND_ALIASES.contains(&alias.as_str()));
        assert!(!EXCLUDED_SCOPE_KIND_ALIASES.contains(&alias.as_str()));
    }
    for kind in [
        "event",
        "integer",
        "parameter",
        "real",
        "reg",
        "supply0",
        "supply1",
        "time",
        "tri",
        "triand",
        "trior",
        "trireg",
        "tri0",
        "tri1",
        "wand",
        "wire",
        "wor",
        "string",
        "port",
        "sparsearray",
        "realtime",
        "realparameter",
        "bit",
        "logic",
        "int",
        "shortint",
        "longint",
        "byte",
        "enum",
        "shortreal",
        "boolean",
        "bitvector",
        "unknown",
    ] {
        let alias = var_type_alias(kind);
        assert!(STABLE_SIGNAL_KIND_ALIASES.contains(&alias.as_str()));
        assert!(!EXCLUDED_SIGNAL_KIND_ALIASES.contains(&alias.as_str()));
    }
}

#[test]
fn metadata_normalizes_raw_integer_tags() {
    let fixture = write_fixture(
        "$timescale 100ps $end\n$scope module top $end\n$var wire 1 ! bit $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#42\n1!\n",
        "scaled-time.vcd",
    );
    let waveform = Waveform::open(fixture.path()).unwrap();
    let metadata = waveform.metadata().unwrap();
    assert_eq!(metadata.time_unit, "100ps");
    assert_eq!(metadata.time_start, "0ps");
    assert_eq!(metadata.time_end, "4200ps");
}

#[test]
fn metadata_rejects_raw_time_overflow() {
    let contents = format!(
        "$timescale 100ns $end\n$scope module top $end\n$var wire 1 ! bit $end\n$upscope $end\n$enddefinitions $end\n#{}\n0!\n",
        u64::MAX,
    );
    let fixture = write_fixture(&contents, "overflow.vcd");
    let waveform = Waveform::open(fixture.path()).unwrap();
    let error = waveform.metadata().unwrap_err();
    assert_eq!(error.exit_code(), 2);
    assert!(error.to_string().contains("time value overflow"));
}

#[test]
fn helper_functions_exercise_invalid_paths_and_timescales() {
    const EXPR_VCD: &str = concat!(
        "$date\n  today\n$end\n",
        "$version\n  wavepeek-test\n$end\n",
        "$timescale 1ns $end\n",
        "$scope module top $end\n",
        "$var real 1 ! temp $end\n",
        "$upscope $end\n",
        "$enddefinitions $end\n",
        "#0\n",
        "r1.0 !\n"
    );

    let fixture = write_fixture(EXPR_VCD, "expr-helpers.vcd");
    let backend = OndasBackend::open(fixture.path()).expect("fixture should open");
    for path in ["", "top."] {
        assert_eq!(
            backend
                .resolve_expr_signal(path, None)
                .unwrap_err()
                .into_error()
                .exit_code(),
            1
        );
    }
    assert!(
        backend
            .resolve_signals(&["top.temp".into()])
            .unwrap_err()
            .to_string()
            .contains("unsupported non-bit-vector encoding")
    );
    assert_eq!(backend.metadata().unwrap().time_unit, "1ns");
    let bad_scale = write_fixture(&EXPR_VCD.replace("1ns", "1unknown"), "bad-scale.vcd");
    assert_eq!(
        OndasBackend::open(bad_scale.path())
            .unwrap_err()
            .exit_code(),
        2
    );
    let missing_scale = write_fixture(
        &EXPR_VCD.replace("$timescale 1ns $end\n", ""),
        "missing-scale.vcd",
    );
    assert!(
        OndasBackend::open(missing_scale.path())
            .unwrap()
            .metadata()
            .unwrap_err()
            .to_string()
            .contains("missing timescale")
    );
    let types = TYPE_SURFACE_VCD.replace(
        "$upscope",
        "$var bit 1 * bitv $end\n$var logic 1 + logicv $end\n$upscope",
    );
    let types = write_fixture(&types, "types.vcd");
    let types = Waveform::open(types.path()).unwrap();
    assert!(
        types
            .resolve_expr_signal("top.bitv")
            .unwrap()
            .expr_type
            .is_four_state
    );
    assert!(
        types
            .resolve_expr_signal("top.logicv")
            .unwrap()
            .expr_type
            .is_four_state
    );
    assert!(
        types
            .resolve_expr_signal("top.integerv")
            .unwrap()
            .expr_type
            .is_signed
    );
    assert!(
        !types
            .resolve_expr_signal("top.logicv")
            .unwrap()
            .expr_type
            .is_signed
    );
}

#[test]
fn waveform_helper_tables_exercise_decode_timescale_and_extra_var_types() {
    let typed_fixture = write_fixture(TYPE_SURFACE_VCD, "typed-vars.vcd");
    let mut typed = Waveform::open(typed_fixture.path()).expect("typed fixture should open");
    let event = typed.resolve_signals(&["top.ev".into()]).unwrap();
    assert_eq!(
        typed.sample_resolved_optional(&event, 5).unwrap()[0].bits,
        Some(String::new())
    );
    for path in ["top.strv", "top.realv"] {
        assert!(
            typed
                .resolve_signals(&[path.into()])
                .unwrap_err()
                .to_string()
                .contains("unsupported non-bit-vector encoding")
        );
    }
    for (unit, expected) in [
        (ondas::TimeUnit::Zeptosecond, "zs"),
        (ondas::TimeUnit::Attosecond, "as"),
        (ondas::TimeUnit::Femtosecond, "fs"),
        (ondas::TimeUnit::Picosecond, "ps"),
        (ondas::TimeUnit::Nanosecond, "ns"),
        (ondas::TimeUnit::Microsecond, "us"),
        (ondas::TimeUnit::Millisecond, "ms"),
        (ondas::TimeUnit::Second, "s"),
    ] {
        assert_eq!(super::time_unit(unit).unwrap(), expected);
    }
    for (path, expected_width, expected_four_state, expected_signed) in [
        ("top.bytev", 8, true, true),
        ("top.shortv", 16, true, true),
        ("top.intv", 32, true, true),
        ("top.longv", 64, true, true),
        ("top.integerv", 32, true, true),
        ("top.timeval", 64, true, false),
    ] {
        let ty = typed
            .resolve_expr_signal(path)
            .expect("expr type")
            .expr_type;
        assert_eq!(ty.width, expected_width, "{path}");
        assert_eq!(ty.is_four_state, expected_four_state, "{path}");
        assert_eq!(ty.is_signed, expected_signed, "{path}");
    }
}

#[test]
fn waveform_sampling_and_scope_error_helpers_exercise_public_error_paths() {
    let fixture = write_fixture(RECURSIVE_TEST_VCD, "recursive-errors.vcd");
    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let error = waveform
        .signals_in_scope_recursive("top.nope", Some(1))
        .expect_err("missing recursive scope should fail");
    assert!(
        error
            .to_string()
            .contains("scope 'top.nope' not found in dump")
    );

    let delayed_fixture = write_fixture(DELAYED_VALUE_VCD, "delayed-public.vcd");
    let mut delayed = Waveform::open(delayed_fixture.path()).expect("fixture should open");
    let empty: Vec<super::ResolvedSignal> = Vec::new();
    assert!(
        delayed
            .sample_resolved_optional(&empty, 0)
            .expect("empty resolved set should short-circuit")
            .is_empty()
    );

    let late_only_fixture = write_fixture(
        concat!(
            "$date\n  today\n$end\n",
            "$version\n  wavepeek-late\n$end\n",
            "$timescale 1ns $end\n",
            "$scope module top $end\n",
            "$var wire 1 ! late $end\n",
            "$upscope $end\n",
            "$enddefinitions $end\n",
            "#5\n",
            "1!\n"
        ),
        "late-only.vcd",
    );
    let mut late_only = Waveform::open(late_only_fixture.path()).expect("fixture should open");
    let late_resolved = late_only
        .resolve_signals(&["top.late".to_string()])
        .expect("signal should resolve");
    assert!(
        late_only
            .sample_resolved_optional(&late_resolved, 0)
            .expect_err("sampling before the first dump timestamp should fail")
            .to_string()
            .contains("before first dump timestamp")
    );

    let delayed_expr = delayed
        .resolve_expr_signal("top.delayed")
        .expect("expr signal should resolve");
    assert_eq!(
        delayed
            .sample_expr_value(&delayed_expr, 0)
            .expect("pre-value integral sample should succeed"),
        crate::expr::SampledValue::Integral {
            bits: None,
            label: None,
        }
    );

    let rich_delayed_fixture = write_fixture(
        concat!(
            "$date\n  today\n$end\n",
            "$version\n  wavepeek-rich-delayed\n$end\n",
            "$timescale 1ns $end\n",
            "$scope module top $end\n",
            "$var real 1 ! temp $end\n",
            "$var string 1 \" msg $end\n",
            "$upscope $end\n",
            "$enddefinitions $end\n",
            "#5\n",
            "r3.25 !\n",
            "slate \"\n"
        ),
        "rich-delayed.vcd",
    );
    let mut rich_delayed =
        Waveform::open(rich_delayed_fixture.path()).expect("fixture should open");
    let real = rich_delayed
        .resolve_expr_signal("top.temp")
        .expect("real signal should resolve");
    let string = rich_delayed
        .resolve_expr_signal("top.msg")
        .expect("string signal should resolve");
    assert_eq!(
        rich_delayed
            .sample_expr_value(&real, 0)
            .expect("pre-value real sample should succeed"),
        crate::expr::SampledValue::Real { value: None }
    );
    assert_eq!(
        rich_delayed
            .sample_expr_value(&string, 0)
            .expect("pre-value string sample should succeed"),
        crate::expr::SampledValue::String { value: None }
    );
}

#[test]
fn unpacked_array_names_preserve_public_scope_components() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! memory[0] [7:0] $end\n$var wire 8 \" memory[1] [7:0] $end\n$var wire 8 # matrix[2][3] [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000001 !\nb00000010 \"\nb00000011 #\n",
        "arrays.vcd",
    );
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    for (path, expected) in [
        ("top.memory[0]", "00000001"),
        ("top.memory[1]", "00000010"),
        (r#"top."matrix[2][3]""#, "00000011"),
    ] {
        let signal = waveform.resolve_expr_signal(path).unwrap();
        assert_eq!(
            waveform.sample_expr_value(&signal, 0).unwrap(),
            crate::expr::SampledValue::Integral {
                bits: Some(expected.into()),
                label: None
            }
        );
    }
    let entries = waveform.signals_in_scope("top").unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        [r#""matrix[2][3]""#, "memory[0]", "memory[1]"]
    );
}

#[test]
fn event_payload_remains_available_between_occurrences() {
    let fixture = write_fixture(
        include_str!("../../tests/fixtures/hand/change_property_events.vcd"),
        "event-payload.vcd",
    );
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let resolved = waveform.resolve_signals(&["top.tick".into()]).unwrap();
    assert_eq!(
        waveform.sample_resolved_optional(&resolved, 9).unwrap()[0]
            .bits
            .as_deref(),
        Some("")
    );
    assert_eq!(
        waveform.sample_resolved_optional(&resolved, 9).unwrap()[0]
            .bits
            .as_deref(),
        Some("")
    );
    let event = waveform.resolve_expr_signal("top.tick").unwrap();
    assert!(!waveform.expr_event_occurred(&event, 9).unwrap());
    assert!(waveform.expr_event_occurred(&event, 10).unwrap());
}

#[test]
fn gapped_split_vectors_do_not_synthesize_huge_widths() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! a [0] $end\n$var wire 1 \" a [2147483647] $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n",
        "gapped-split.vcd",
    );
    let waveform = Waveform::open(fixture.path()).unwrap();
    let error = waveform.resolve_signals(&["top.a".into()]).unwrap_err();
    assert!(error.to_string().contains("ambiguous"));
}

#[test]
fn split_vector_expression_before_dump_and_mixed_event_samples() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! sig [0] $end\n$var wire 1 \" sig [1] $end\n$var event 1 # ev $end\n$upscope $end\n$enddefinitions $end\n#10\n0!\n0\"\n1#\n#15\n1!\n",
        "late-split-event.vcd",
    );
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let sig = waveform.resolve_expr_signal("top.sig").unwrap();
    assert_eq!(
        waveform.sample_expr_value(&sig, 9).unwrap(),
        crate::expr::SampledValue::Integral {
            bits: None,
            label: None,
        }
    );
    let resolved = waveform
        .resolve_signals(&["top.sig".into(), "top.ev".into()])
        .unwrap();
    let values = waveform.sample_resolved_optional(&resolved, 10).unwrap();
    assert_eq!(values[0].bits.as_deref(), Some("00"));
    assert_eq!(values[1].bits.as_deref(), Some(""));
}

#[test]
fn escaped_unpacked_array_keeps_escape_on_base_component() {
    let source = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! \\x[0] [7:0] $end\n$var wire 8 \" \\matrix[2][3] [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000001 !\nb00000010 \"\n",
        "escaped-array.vcd",
    );
    let dir = tempfile::tempdir().unwrap();
    let fst = dir.path().join("escaped-array.fst");
    assert!(
        Command::new("vcd2fst")
            .arg(source.path())
            .arg(&fst)
            .status()
            .unwrap()
            .success()
    );
    let paths = [
        (r#"top."x[0]""#, "00000001"),
        (r#"top."matrix[2][3]""#, "00000010"),
    ];
    for path in [source.path(), fst.as_path()] {
        let mut waveform = Waveform::open(path).unwrap();
        for (signal_path, expected) in paths {
            let signal = waveform.resolve_expr_signal(signal_path).unwrap();
            assert_eq!(
                waveform.sample_expr_value(&signal, 0).unwrap(),
                crate::expr::SampledValue::Integral {
                    bits: Some(expected.into()),
                    label: None,
                }
            );
        }
        assert_eq!(waveform.scopes_depth_first(None).unwrap().len(), 1);
    }
}

fn write_fixture(contents: &str, filename: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .suffix(filename)
        .tempfile()
        .expect("tempfile should be created");
    file.write_all(contents.as_bytes())
        .expect("fixture should be written");
    file
}

#[test]
fn common_composition_and_slices_preserve_partial_and_all_missing() {
    use std::cell::RefCell;
    use std::ops::ControlFlow;
    use std::rc::Rc;

    use crate::expr::ExpressionHost;
    use crate::waveform::expr_host::WaveformExprHost;
    use crate::waveform::expr_query::{ExpressionQuery, ExpressionQueryTick};

    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 4 ! bus [7:4] $end\n$var wire 4 \" bus [3:0] $end\n$var wire 4 # asc [0:3] $end\n$var wire 4 $ asc [4:7] $end\n$upscope $end\n$enddefinitions $end\n#0\n#5\nb0110 \"\nb1100 #\n#10\nb1010 !\nb0011 $\n#15\nbxxxx !\n",
        "composition-missing.vcd",
    );
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    let bus = waveform
        .resolve_signals(&["top.bus".into()])
        .unwrap()
        .remove(0);
    let high = waveform
        .resolve_signals(&["top.bus[7:4]".into()])
        .unwrap()
        .remove(0);
    let cross = waveform
        .resolve_signals(&["top.bus[5:2]".into()])
        .unwrap()
        .remove(0);
    let asc = waveform
        .resolve_signals(&["top.asc".into()])
        .unwrap()
        .remove(0);
    let resolved = [bus, high, cross, asc];
    for (time, expected) in [
        (0, [None, None, None, None]),
        (
            5,
            [
                Some("xxxx0110"),
                Some("xxxx"),
                Some("xx01"),
                Some("1100xxxx"),
            ],
        ),
        (
            10,
            [
                Some("10100110"),
                Some("1010"),
                Some("1001"),
                Some("11000011"),
            ],
        ),
    ] {
        let samples = waveform.sample_resolved_optional(&resolved, time).unwrap();
        assert_eq!(
            samples
                .iter()
                .map(|sample| sample.bits.as_deref())
                .collect::<Vec<_>>(),
            expected
        );
    }
    let expr = waveform.resolve_expr_signal("top.bus").unwrap();
    assert_eq!(
        waveform.sample_expr_value(&expr, 0).unwrap(),
        crate::expr::SampledValue::Integral {
            bits: None,
            label: None
        }
    );
    let waveform = Rc::new(RefCell::new(waveform));
    let binding = WaveformExprHost::from_shared(waveform.clone());
    let bus = binding.resolve_signal("top.bus").unwrap();
    let query = ExpressionQuery::new(&binding, &[bus], &[]).unwrap();
    let mut samples = Vec::new();
    waveform
        .borrow_mut()
        .query_expressions(&query, 5..=15, false, |host, tick| {
            let ExpressionQueryTick::Candidate(time) = tick else {
                panic!("unexpected initial state")
            };
            samples.push((time, host.sample_value(bus, time).unwrap()));
            Ok(ControlFlow::Continue(()))
        })
        .unwrap();
    assert_eq!(
        samples,
        [(5, "xxxx0110"), (10, "10100110"), (15, "xxxx0110")].map(|(time, bits)| (
            time,
            crate::expr::SampledValue::Integral {
                bits: Some(bits.into()),
                label: None,
            }
        ))
    );
}

#[test]
fn listing_include_and_direct_lookup_share_exact_declarations_and_alias_history() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$var wire 1 ! \\root.name $end\n$scope module top $end\n$var wire 1 ! alias $end\n$var wire 1 \" \\literal[7:0] $end\n$var wire 1 # \\literal $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n",
        "include-exact.vcd",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let entries = backend.matching_signals(|_, _| true);
    assert_eq!(entries.len(), 4);
    for entry in entries {
        let canonical = entry.path.to_string();
        let found = backend.matching_signals(|path, _| path == canonical);
        assert_eq!(found, std::slice::from_ref(&entry));
        let signals = backend
            .resolve_signals(std::slice::from_ref(&canonical))
            .unwrap();
        let expr = backend.resolve_expr_signal(&canonical, None).unwrap();
        assert_eq!(signals[0].signal, expr.signal);
        assert_eq!(
            backend.sample_resolved_optional(&signals, 0).unwrap()[0]
                .bits
                .as_deref(),
            Some(if canonical == r#"top."literal[7:0]""# {
                "1"
            } else {
                "0"
            })
        );
    }
    let aliases = backend
        .resolve_signals(&[r#""root.name""#.into(), "top.alias".into()])
        .unwrap();
    assert_eq!(aliases[0].signal, aliases[1].signal);
}

#[test]
fn repeated_scope_names_keep_their_declarations_in_their_own_scope() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$scope module child $end\n$var wire 1 ! a $end\n$upscope $end\n$scope module child $end\n$var wire 1 \" b $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n",
        "duplicate-scope.vcd",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let listing = backend
        .signals_in_scope_recursive_report("top", None)
        .unwrap();
    assert_eq!(
        listing
            .entries
            .iter()
            .map(|entry| entry.path.to_string())
            .collect::<Vec<_>>(),
        ["top.child.a", "top.child.b"]
    );
    let resolved = backend
        .resolve_signals(
            &listing
                .entries
                .iter()
                .map(|entry| entry.path.to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap();
    assert_eq!(
        backend
            .sample_resolved_optional(&resolved, 0)
            .unwrap()
            .iter()
            .map(|sample| sample.bits.as_deref())
            .collect::<Vec<_>>(),
        [Some("0"), Some("1")]
    );
}

#[test]
fn batch_samples_preserve_aliases_projections_and_composition_missing() {
    let fixture = write_fixture(
        concat!(
            "$timescale 1ns $end\n$scope module top $end\n",
            "$var wire 4 ! bus [7:4] $end\n$var wire 4 \" bus [3:0] $end\n",
            "$var wire 4 \" alias $end\n$var wire 8 # literal $end\n",
            "$var wire 4 $ \\literal[3:0] $end\n$upscope $end\n$enddefinitions $end\n",
            "#0\nb10100110 #\nb0001 $\n#5\nb0110 \"\n#10\nb1010 !\n#15\nbxxxx !\n",
        ),
        "batch-composition.vcd",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let paths = [
        "top.bus",
        "top.bus[5:2]",
        "top.alias",
        "top.bus",
        "top.bus[7:4]",
        r#"top."literal[3:0]""#,
        "top.literal[3:0]",
    ]
    .map(String::from);
    let resolved = backend.resolve_signals(&paths).unwrap();
    let times = [10, 0, 5, 10, 15, 4, 16];
    let rows = backend
        .sample_resolved_optional_at_times(&resolved, &times)
        .unwrap();
    for (row, time) in rows.iter().zip(times) {
        assert_eq!(
            *row,
            backend.sample_resolved_optional(&resolved, time).unwrap()
        );
        assert_eq!(row[0], row[3]);
        assert_eq!(row[5].bits.as_deref(), Some("0001"));
        assert_eq!(row[6].bits.as_deref(), Some("0110"));
    }
    assert_eq!(rows[0][0].bits.as_deref(), Some("10100110"));
    assert_eq!(rows[0][2].bits.as_deref(), Some("0110"));
    assert_eq!(rows[2][2].bits.as_deref(), Some("0110"));
    assert_eq!(rows[2][2].path, "top.alias");
    assert_eq!(rows[1][0].bits, None);
    assert_eq!(rows[1][2].bits, None);
    assert_eq!(rows[2][0].bits.as_deref(), Some("xxxx0110"));
    assert_eq!(rows[2][1].bits.as_deref(), Some("xx01"));
    assert_eq!(rows[2][4].bits.as_deref(), Some("xxxx"));
    assert_eq!(rows[4][0].bits.as_deref(), Some("xxxx0110"));
    assert!(
        backend
            .sample_resolved_optional_at_times(&resolved, &[])
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        backend
            .sample_resolved_optional_at_times(&[], &[5, 0, 5])
            .unwrap(),
        vec![vec![], vec![], vec![]]
    );
}

#[test]
fn point_and_batch_samples_preserve_window_and_event_boundaries() {
    let fixture = write_fixture(
        concat!(
            "$timescale 1ns $end\n$scope module top $end\n",
            "$var wire 1 ! state $end\n$var wire 1 \" delayed $end\n",
            "$var wire 1 # other $end\n$var event 1 $ tick $end\n",
            "$var wire 1 % never $end\n$upscope $end\n$enddefinitions $end\n",
            "#0\n0!\n0#\n#5\n1!\nx\"\n1$\n#10\n0!\n1\"\n1$\n",
            "#20\n1!\n1#\n1$\n#25\n",
        ),
        "saved-window.vcd",
    );
    let dir = tempfile::tempdir().unwrap();
    let fst = dir.path().join("saved-window.fst");
    assert!(
        Command::new("vcd2fst")
            .arg(fixture.path())
            .arg(&fst)
            .status()
            .unwrap()
            .success()
    );
    for path in [fixture.path(), fst.as_path()] {
        let mut point = OndasBackend::open(path).unwrap();
        let mut direct = OndasBackend::open(path).unwrap();
        let paths = ["top.state", "top.delayed", "top.tick", "top.never"].map(String::from);
        let resolved = point.resolve_signals(&paths).unwrap();
        let direct_resolved = direct.resolve_signals(&paths).unwrap();
        let event = point.resolve_expr_signal("top.tick", None).unwrap();
        let direct_event = direct.resolve_expr_signal("top.tick", None).unwrap();
        let times = [12, 6, 9, 10, 11, 6, 4, 5, 13, 20, 25, 30];
        let batch = direct
            .sample_resolved_optional_at_times(&direct_resolved, &times)
            .unwrap();
        for (time, batch_row) in times.into_iter().zip(batch) {
            let expected = direct
                .sample_resolved_optional(&direct_resolved, time)
                .unwrap();
            assert_eq!(batch_row, expected);
            assert_eq!(
                point.sample_resolved_optional(&resolved, time).unwrap(),
                expected,
                "{} at {time}",
                path.display()
            );
            assert_eq!(
                point.expr_event_occurred(&event, time).unwrap(),
                direct.expr_event_occurred(&direct_event, time).unwrap()
            );
            assert_eq!(
                point.expr_event_occurred(&event, time).unwrap(),
                [5, 10, 20].contains(&time)
            );
        }
        let quiet = point.sample_resolved_optional(&resolved, 6).unwrap();
        assert_eq!(quiet[0].bits.as_deref(), Some("1"));
        assert_eq!(quiet[1].bits.as_deref(), Some("x"));
        assert_eq!(
            point.sample_resolved_optional(&resolved, 30).unwrap()[0]
                .bits
                .as_deref(),
            Some("1")
        );
        assert_eq!(quiet[3].bits, None);
        let other = point.resolve_signals(&["top.other".into()]).unwrap();
        point.sample_resolved_optional(&other, 9).unwrap();
    }
}

#[test]
fn selectors_keep_scopes_indexed_bases_and_literal_identity() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! mem[0] [7:0] $end\n$var wire 8 \" data [7:0] $end\n$var wire 4 # \\data[3:0] $end\n$scope module \\a.b $end\n$var wire 1 $ flag $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\nb10100101 !\nb11110000 \"\nb1010 #\n1$\n",
        "selectors.vcd",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    for (selector, scope, path, bits) in [
        ("mem[0][7:4]", Some("top"), "top.mem[0][7:4]", "1010"),
        ("top.mem[0][7:4]", Some("top"), "top.mem[0][7:4]", "1010"),
        (r#""data[3:0]""#, Some("top"), r#"top."data[3:0]""#, "1010"),
        ("data[3:0]", Some("top"), "top.data[3:0]", "0000"),
        (r#""data"[3:0]"#, Some("top"), "top.data[3:0]", "0000"),
        (r#"flag"#, Some(r#"top."a.b""#), r#"top."a.b".flag"#, "1"),
        (r#"top.\a.b .flag"#, Some("top"), r#"top."a.b".flag"#, "1"),
    ] {
        let selected = backend.resolve_signal(selector, scope).unwrap();
        assert_eq!(selected.path, path);
        let roundtrip = backend.resolve_signal(&selected.path, None).unwrap();
        assert_eq!(selected.signal, roundtrip.signal);
        let samples = backend
            .sample_resolved_optional(&[selected, roundtrip], 0)
            .unwrap();
        assert!(
            samples
                .iter()
                .all(|sample| sample.bits.as_deref() == Some(bits))
        );
    }
}

#[test]
fn selected_alias_types_and_explicit_full_width_are_independent_of_handle_identity() {
    use crate::expr::{ExprStorage, ExprTypeKind, IntegerLikeKind};
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var integer 8 ! signed $end\n$var wire 8 ! unsigned $end\n$var enum 8 ! enumeration $end\n$upscope $end\n$enddefinitions $end\n#0\nb11111111 !\n",
        "alias-types.vcd",
    );
    let backend = OndasBackend::open(fixture.path()).unwrap();
    let signed = backend.resolve_expr_signal("signed", Some("top")).unwrap();
    let unsigned = backend
        .resolve_expr_signal("unsigned", Some("top"))
        .unwrap();
    let enumeration = backend
        .resolve_expr_signal("enumeration", Some("top"))
        .unwrap();
    assert_eq!(signed.signal, unsigned.signal);
    assert_eq!(signed.signal, enumeration.signal);
    assert_eq!(
        signed.expr_type.kind,
        ExprTypeKind::IntegerLike(IntegerLikeKind::Integer)
    );
    assert!(signed.expr_type.is_signed);
    assert!(!unsigned.expr_type.is_signed);
    assert_eq!(enumeration.expr_type.kind, ExprTypeKind::EnumCore);
    for name in ["signed", "unsigned", "enumeration"] {
        let projected = backend
            .resolve_expr_signal(&format!("{name}[7:0]"), Some("top"))
            .unwrap();
        assert_eq!(projected.signal, signed.signal);
        assert_eq!(projected.expr_type.kind, ExprTypeKind::BitVector);
        assert_eq!(projected.expr_type.storage, ExprStorage::PackedVector);
        assert!(!projected.expr_type.is_signed);
        assert!(projected.expr_type.enum_type_id.is_none());
        assert!(projected.expr_type.enum_labels.is_none());
    }
}
