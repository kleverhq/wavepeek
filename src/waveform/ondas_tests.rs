use std::io::Write;
use std::path::Path;
use std::process::Command;

use tempfile::NamedTempFile;

use super::{OndasBackend, scope_type_alias, var_type_alias};
use crate::waveform::{
    ChangeCandidateCollectionMode, EXCLUDED_SCOPE_KIND_ALIASES, EXCLUDED_SIGNAL_KIND_ALIASES,
    STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES, SampledSignal, ScopeEntry, Waveform,
    classify_edge, duplicate_preserving_projection,
};

const TEST_VCD: &str = "$date\n  today\n$end\n$version\n  wavepeek-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var reg 8 \" data $end\n$var parameter 8 # cfg $end\n$scope module cpu $end\n$var wire 1 $ valid $end\n$upscope $end\n$scope function helper $end\n$var wire 1 & helper_flag $end\n$upscope $end\n$scope module mem $end\n$var wire 1 % ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nb00000000 \"\nb10101010 #\n0$\n0&\n0%\n#5\n1!\n1$\n1&\n#10\nb00001111 \"\n1%\n";

const RICH_VALUE_VCD: &str = "$date\n  2026-03-12\n$end\n$version\n  wavepeek-rich-value\n$end\n$timescale 1ns $end\n$scope module top $end\n$var real 1 ! temp $end\n$var string 1 \" msg $end\n$upscope $end\n$enddefinitions $end\n#0\nr1.5 !\nsgo \"\n";

const DERIVED_SPLIT_VCD: &str = "$date\n  2026-07-26\n$end\n$version\n  wavepeek-derived-split\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! split [0] $end\n$var wire 1 \" split [1] $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n#5\n1!\n#10\n1\"\n#15\n0!\n#20\n0\"\n";

const RECURSIVE_TEST_VCD: &str = "$date\n  2026-02-28\n$end\n$version\n  wavepeek-recursive-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$scope module cpu $end\n$var wire 1 \" valid $end\n$scope module core $end\n$var wire 1 # execute $end\n$upscope $end\n$upscope $end\n$scope module mem $end\n$var wire 1 $ ready $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n0$\n#5\n1!\n1\"\n1#\n1$\n";

const DELAYED_VALUE_VCD: &str = "$date\n  2026-03-03\n$end\n$version\n  wavepeek-delayed-value\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! delayed $end\n$upscope $end\n$enddefinitions $end\n#0\n#5\n1!\n";

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
    let paths = ["top.\\a.b", "top.\\\\a.b"].map(str::to_owned);
    assert_eq!(
        listing
            .entries
            .iter()
            .map(|entry| entry.path.clone())
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
    assert_eq!(entries[0].name, "\\plain");
    assert_eq!(entries[0].path, "top.\\plain");
    let values = waveform
        .sample_signals_at_time(&["top.\\plain".into()], 0)
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
        "top.\\foo.bar",
        "top.\\foo/bar",
        "top.\\block.dot.value",
        "top.foo.bar",
    ]
    .map(str::to_owned);
    assert_eq!(
        listing
            .entries
            .iter()
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>(),
        paths
    );
    assert!(listing.omitted_ambiguous_paths.is_empty());
    let bounded = waveform
        .signals_in_scope_recursive_report("top", Some(1))
        .unwrap();
    assert_eq!(bounded.entries, listing.entries);
    assert_eq!(
        waveform.signals_in_scope("top.\\block.dot").unwrap()[0].name,
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
            .map(|scope| scope.path.as_str())
            .collect::<Vec<_>>(),
        ["top", "top.a", "top.a.z", "top.a$"]
    );
    let signals = waveform
        .signals_in_scope_recursive_report("top", None)
        .unwrap();
    assert_eq!(
        signals
            .entries
            .iter()
            .map(|signal| signal.path.as_str())
            .collect::<Vec<_>>(),
        ["top.root", "top.a.base", "top.a.z.deep", "top.a$.tail"]
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
                path: "top".to_string(),
                depth: 0,
                kind: "module".to_string()
            },
            ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string()
            },
            ScopeEntry {
                path: "top.helper".to_string(),
                depth: 1,
                kind: "function".to_string()
            },
            ScopeEntry {
                path: "top.mem".to_string(),
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
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    let unbounded_paths = unbounded
        .iter()
        .map(|entry| entry.path.clone())
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
    assert_eq!(signals[0].path, "top.cfg");
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
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    let depth_1_paths = depth_1
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    let depth_2_paths = depth_2
        .iter()
        .map(|entry| entry.path.clone())
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
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    let unbounded_paths = unbounded
        .iter()
        .map(|entry| entry.path.clone())
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
        .map(|entry| entry.path.clone())
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
fn resolved_signal_ids_are_stable_across_resolution_paths() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.clk".to_string(), "top.clk".to_string()])
        .expect("signal should resolve");
    let expr_resolved = waveform
        .resolve_expr_signal("top.clk")
        .expect("expression signal should resolve");

    assert_eq!(resolved[0].id, resolved[1].id);
    assert_eq!(resolved[0].id, expr_resolved.id);
}

#[test]
fn previous_sample_time_returns_strict_predecessor() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");

    assert_eq!(waveform.previous_sample_time(0), None);
    assert_eq!(waveform.previous_sample_time(5), Some(4));
    assert_eq!(waveform.previous_sample_time(7), Some(6));
    assert_eq!(waveform.previous_sample_time(10), Some(9));
}

#[test]
fn collect_change_times_facade_returns_unique_backend_timestamps() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.clk".to_string(), "top.data".to_string()])
        .expect("signals should resolve");

    assert_eq!(
        waveform
            .collect_change_times(&resolved, 0, 10)
            .expect("change times should collect"),
        vec![0, 5, 10]
    );
    assert_eq!(
        waveform
            .collect_change_times_with_mode(
                &resolved,
                0,
                10,
                ChangeCandidateCollectionMode::Random,
            )
            .expect("random access change times should collect"),
        vec![0, 5, 10]
    );
    assert!(
        waveform
            .collect_change_times(&[], 0, 10)
            .expect("empty resolved set should collect")
            .is_empty()
    );
    assert!(
        waveform
            .collect_change_times(&resolved, 20, 30)
            .expect("empty window should collect")
            .is_empty()
    );
    assert!(
        waveform
            .collect_change_times_with_mode(
                &resolved,
                0,
                10,
                ChangeCandidateCollectionMode::Stream,
            )
            .expect_err("forced stream on VCD should fail")
            .to_string()
            .contains("requires FST input")
    );
}

#[test]
fn streaming_candidate_decision_facade_rejects_vcd_inputs() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");

    assert!(!waveform.should_use_streaming_candidate_collection(
        2,
        0,
        10,
        ChangeCandidateCollectionMode::Random,
    ));
    assert!(!waveform.should_use_streaming_candidate_collection(
        2,
        0,
        10,
        ChangeCandidateCollectionMode::Stream,
    ));
    assert!(!waveform.should_use_streaming_candidate_collection(
        2,
        0,
        10,
        ChangeCandidateCollectionMode::Auto,
    ));
}

#[test]
fn streaming_candidate_collection_rejects_derived_signals() {
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
    assert_eq!(
        waveform.collect_change_times(&resolved, 5, 10).unwrap(),
        vec![5, 10]
    );
    assert!(!waveform.ensure_indexed_signals_loaded(&[resolved[0].id]));
    assert!(
        waveform
            .collect_change_times_with_mode(&resolved, 5, 10, ChangeCandidateCollectionMode::Stream)
            .is_err()
    );
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
fn fst_scope_only_does_not_build_full_index() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let backend = OndasBackend::open(&path).unwrap();
    let scopes = backend.scopes_depth_first(None).unwrap();
    assert!(backend.index.get().is_none());
    assert_eq!(scopes, backend.index().scopes);
}

#[test]
fn fst_single_exact_signal_does_not_build_full_index() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let mut backend = OndasBackend::open(&path).unwrap();
    let paths = ["top.clk".to_string()];
    let selected = backend.resolve_signals(&paths).unwrap();
    assert!(backend.index.get().is_none());
    let sampled = backend.sample_resolved_optional(&selected, 10).unwrap();
    assert!(backend.index.get().is_none());

    let mut full = OndasBackend::open(&path).unwrap();
    full.scopes_depth_first(None).unwrap();
    let expected = full.sample_resolved_optional(&full.resolve_signals(&paths).unwrap(), 10);
    assert_eq!(sampled, expected.unwrap());

    let mut selected = selected;
    selected.extend(backend.resolve_signals(&["top.data".to_string()]).unwrap());
    assert!(backend.index.get().is_none());
    let both = backend.sample_resolved_optional(&selected, 10).unwrap();
    assert!(backend.index.get().is_none());
    backend.signals_in_scope("top").unwrap();
    assert!(backend.index.get().is_some());
    assert_eq!(
        backend.sample_resolved_optional(&selected, 10).unwrap(),
        both
    );
}

#[test]
fn fst_batch_value_selection_keeps_full_index_lazy() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let mut backend = OndasBackend::open(&path).unwrap();
    let paths = ["top.clk".to_string(), "top.data".to_string()];
    backend.prepare_value_signals(&paths);
    let resolved = backend.resolve_signals(&paths).unwrap();
    assert!(backend.index.get().is_none());
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
    batch.prepare_value_signals(&paths);
    let selected = batch.resolve_signals(&paths).unwrap();
    assert!(batch.index.get().is_none());
    assert_ne!(selected[0].id, selected[1].id);
    let actual = batch.sample_resolved_optional(&selected, 0).unwrap();

    let mut full = OndasBackend::open(&fst).unwrap();
    full.signals_in_scope("top").unwrap();
    let expected = full.sample_resolved_optional(&full.resolve_signals(&paths).unwrap(), 0);
    assert_eq!(actual, expected.unwrap());
}

#[test]
fn fst_cached_trace_handles_reverse_and_repeated_times() {
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
    backend
        .preload_resolved_value_changes(&resolved, 0, 10)
        .unwrap();
    for (time, expected) in [(5, "1"), (5, "1"), (4, "0"), (10, "0"), (0, "0"), (5, "1")] {
        let sampled = backend.sample_resolved_optional(&resolved, time).unwrap();
        assert_eq!(sampled[0].bits.as_deref(), Some(expected), "at tick {time}");
    }
}

#[test]
fn fst_array_path_uses_full_index_spelling() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! memory[0] [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000001 !\n",
        "array.vcd",
    );
    let wave = ondas::open(fixture.path()).unwrap();
    let variable = wave.hierarchy().variables().next().unwrap();
    let direct = super::public_fst_variable_path(&variable);
    let full = super::HierarchyIndex::new(wave.hierarchy(), ondas::Format::Fst, false);
    assert_eq!(direct.as_deref(), Some("top.memory.[0]"));
    assert!(full.by_path.contains_key(direct.as_deref().unwrap()));
}

#[test]
fn fst_batch_array_alias_remains_ambiguous() {
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
    let path = "top.memory.[0]";
    let single = OndasBackend::open(&fst)
        .unwrap()
        .resolve_signals(&[path.into()])
        .unwrap_err();
    let batch = OndasBackend::open(&fst).unwrap();
    batch.prepare_value_signals(&[path.into(), path.into()]);
    assert_eq!(
        batch
            .resolve_signals(&[path.into()])
            .unwrap_err()
            .to_string(),
        single.to_string()
    );
}

#[test]
fn fst_expression_signal_does_not_build_full_index() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/generated/m2_core.fst");
    let mut backend = OndasBackend::open(&path).unwrap();
    let signal = backend.resolve_expr_signal("top.clk").unwrap();
    assert!(backend.index.get().is_none());
    let sampled = backend.sample_expr_value(&signal, 10).unwrap();
    assert!(backend.index.get().is_none());

    let mut full = OndasBackend::open(&path).unwrap();
    full.signals_in_scope("top").unwrap();
    let expected = full.sample_expr_value(&full.resolve_expr_signal("top.clk").unwrap(), 10);
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
fn indexed_signal_offset_at_compares_data_position_only() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.data".to_string()])
        .expect("signal should resolve");
    let clock = waveform.resolve_signals(&["top.clk".into()]).unwrap();
    waveform.ensure_indexed_signals_loaded(&[resolved[0].id, clock[0].id]);

    let offset_at_0 = waveform
        .indexed_signal_offset_at(resolved[0].id, 0)
        .expect("Wellen backend supports indexed offsets")
        .expect("offset at #0 should exist");
    let offset_at_5 = waveform
        .indexed_signal_offset_at(resolved[0].id, 1)
        .expect("Wellen backend supports indexed offsets")
        .expect("offset at #5 should exist");
    let offset_at_10 = waveform
        .indexed_signal_offset_at(resolved[0].id, 2)
        .expect("Wellen backend supports indexed offsets")
        .expect("offset at #10 should exist");

    assert_eq!(offset_at_0, offset_at_5);
    assert_ne!(offset_at_5, offset_at_10);
}

#[test]
fn indexed_signal_offset_at_returns_none_when_signal_is_not_loaded() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.data".to_string()])
        .expect("signal should resolve");

    assert_eq!(
        waveform.indexed_signal_offset_at(resolved[0].id, 0),
        Some(None)
    );
}

#[test]
fn decode_indexed_signal_at_matches_sample_resolved_optional() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.clk".to_string(), "top.data".to_string()])
        .expect("signals should resolve");
    let signal_ids = resolved.iter().map(|signal| signal.id).collect::<Vec<_>>();
    waveform.ensure_indexed_signals_loaded(&signal_ids);

    let at_10 = waveform
        .sample_resolved_optional(&resolved, 10)
        .expect("batch sampling should succeed");
    let decoded = resolved
        .iter()
        .map(|signal| {
            waveform
                .decode_indexed_signal_at(signal, 2)
                .map(|sample| sample.expect("Wellen backend supports indexed decoding"))
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("point decode should succeed");

    assert_eq!(decoded, at_10);
}

#[test]
fn decode_indexed_signal_at_returns_none_when_no_prior_value_exists() {
    let fixture = write_fixture(DELAYED_VALUE_VCD, "delayed.vcd");

    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.delayed".to_string()])
        .expect("signal should resolve");
    waveform.ensure_indexed_signals_loaded(&[resolved[0].id]);

    let sample_before_first_value = waveform
        .decode_indexed_signal_at(&resolved[0], 0)
        .expect("decode should succeed")
        .expect("Wellen backend supports indexed decoding");
    let sample_after_first_value = waveform
        .decode_indexed_signal_at(&resolved[0], 1)
        .expect("decode should succeed")
        .expect("Wellen backend supports indexed decoding");

    assert_eq!(sample_before_first_value.bits, None);
    assert_eq!(sample_after_first_value.bits.as_deref(), Some("1"));
}

#[test]
fn decode_indexed_signal_at_requires_loaded_signal_data() {
    let fixture = write_fixture(TEST_VCD, "sample.vcd");

    let waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_signals(&["top.data".to_string()])
        .expect("signal should resolve");

    let error = waveform
        .decode_indexed_signal_at(&resolved[0], 0)
        .expect_err("decode must fail before load");
    assert_eq!(
        error.to_string(),
        "fatal: internal: signal 'top.data' could not be loaded from waveform backend"
    );
}

#[test]
fn edge_classification_sv2023_matrix() {
    for (previous, current) in [("0", "1"), ("0", "x"), ("0", "z"), ("x", "1"), ("z", "1")] {
        let edge = classify_edge(previous, current);
        assert!(edge.posedge, "expected posedge for {previous}->{current}");
    }

    for (previous, current) in [("1", "0"), ("1", "x"), ("1", "z"), ("x", "0"), ("z", "0")] {
        let edge = classify_edge(previous, current);
        assert!(edge.negedge, "expected negedge for {previous}->{current}");
    }

    for previous in ["0", "1", "x", "z"] {
        for current in ["0", "1", "x", "z"] {
            let edge = classify_edge(previous, current);
            assert_eq!(edge.edge(), edge.posedge || edge.negedge);
        }
    }
}

#[test]
fn edge_classification_ninestate_maps_to_x() {
    assert!(classify_edge("h", "1").posedge);
    assert!(classify_edge("1", "l").negedge);
    assert!(classify_edge("u", "0").negedge);
    assert!(classify_edge("0", "w").posedge);
    assert!(classify_edge("-", "1").posedge);
}

#[test]
fn edge_detection_uses_lsb_only() {
    let msb_only = classify_edge("0001", "1001");
    assert!(!msb_only.edge());

    let lsb_flip = classify_edge("1000", "1001");
    assert!(lsb_flip.posedge);
}

#[test]
fn edge_classification_treats_empty_samples_as_no_edge() {
    assert!(!classify_edge("", "1").edge());
    assert!(!classify_edge("0", "").edge());
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
fn candidate_collection_and_time_helpers_exercise_split_paths() {
    const EXPR_VCD: &str = concat!(
        "$date\n  today\n$end\n",
        "$version\n  wavepeek-test\n$end\n",
        "$timescale 1ns $end\n",
        "$scope module top $end\n",
        "$var wire 1 ! sig $end\n",
        "$var event 1 \" ev $end\n",
        "$upscope $end\n",
        "$enddefinitions $end\n",
        "#0\n",
        "0!\n",
        "#5\n",
        "1!\n",
        "1\"\n"
    );

    let fixture = write_fixture(EXPR_VCD, "expr-candidates.vcd");
    let mut waveform = Waveform::open(fixture.path()).expect("fixture should open");
    let resolved = waveform
        .resolve_expr_signals(&["top.sig".to_string(), "top.ev".to_string()])
        .expect("signals should resolve");
    let candidates = waveform
        .collect_expr_candidate_times_with_mode(
            &resolved,
            0,
            5,
            ChangeCandidateCollectionMode::Random,
        )
        .expect("candidate collection should succeed");
    assert_eq!(candidates, vec![0, 5]);

    let empty: Vec<super::ResolvedSignal> = Vec::new();
    assert!(
        waveform
            .collect_change_times_with_mode(&empty, 0, 5, ChangeCandidateCollectionMode::Random)
            .expect("empty signal list should short-circuit")
            .is_empty()
    );

    assert!(
        waveform
            .collect_change_times_with_mode(
                &[super::ResolvedSignal {
                    path: "top.sig".to_string(),
                    id: resolved[0].id,
                    width: 1,
                }],
                0,
                5,
                ChangeCandidateCollectionMode::Stream,
            )
            .expect_err("forcing stream mode on VCD should fail")
            .to_string()
            .contains("forced stream candidate collection")
    );
    assert!(!waveform.should_use_streaming_candidate_collection(
        1,
        0,
        5,
        ChangeCandidateCollectionMode::Auto,
    ));

    assert!(
        waveform
            .collect_expr_candidate_times_with_mode(
                &resolved,
                9,
                1,
                ChangeCandidateCollectionMode::Random
            )
            .unwrap()
            .is_empty()
    );
    assert!(
        waveform
            .collect_expr_candidate_times_with_mode(
                &resolved,
                6,
                9,
                ChangeCandidateCollectionMode::Random
            )
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        waveform
            .collect_expr_candidate_times_with_mode(
                &resolved,
                1,
                9,
                ChangeCandidateCollectionMode::Random
            )
            .unwrap(),
        vec![5]
    );
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
        assert!(
            backend
                .resolve_expr_signal(path)
                .unwrap_err()
                .to_string()
                .contains("not found in dump")
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
        !types
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
        ("top.bytev", 8, false, true),
        ("top.shortv", 16, false, true),
        ("top.intv", 32, false, true),
        ("top.longv", 64, false, true),
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
fn packed_fst_ranges_are_not_unpacked_array_indices() {
    for (name, width, expected) in [
        ("q_err[3:0]", 4, Some(("q_err", 3, 0))),
        ("q_err[3]", 1, Some(("q_err", 3, 3))),
        ("memory[0][7:0]", 8, Some(("memory[0]", 7, 0))),
        ("word[-2:-9]", 8, Some(("word", -2, -9))),
        ("memory[0]", 8, None),
        ("q_err[3:0]", 8, None),
        ("literal[name]", 4, None),
    ] {
        assert_eq!(
            super::packed_name_range(name, width).map(|(name, range)| (
                name,
                range.msb(),
                range.lsb()
            )),
            expected
        );
    }
    assert_eq!(super::array_name_parts("q_err[3:0]"), ["q_err[3:0]"]);
    assert_eq!(super::array_name_parts("literal[name]"), ["literal[name]"]);
}

#[test]
fn unpacked_array_names_preserve_public_scope_components() {
    let fixture = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! memory[0] [7:0] $end\n$var wire 8 \" memory[1] [7:0] $end\n$var wire 8 # matrix[2][3] [7:0] $end\n$upscope $end\n$enddefinitions $end\n#0\nb00000001 !\nb00000010 \"\nb00000011 #\n",
        "arrays.vcd",
    );
    let mut waveform = Waveform::open(fixture.path()).unwrap();
    for (path, expected) in [
        ("top.memory.[0]", "00000001"),
        ("top.memory.[1]", "00000010"),
        ("top.matrix.[2].[3]", "00000011"),
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
    let entries = waveform.signals_in_scope("top.memory").unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        ["[0]", "[1]"]
    );
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
