use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::OndasBackend;
use crate::expr::{ExprTypeKind, SampledValue};
use crate::waveform::{
    ChangeCandidateCollectionMode, STABLE_SCOPE_KIND_ALIASES, STABLE_SIGNAL_KIND_ALIASES,
};

#[test]
fn fsdb_nine_state_samples_and_enum_keys_remain_raw() {
    let path = PathBuf::from(std::env::var_os("ONDAS_FIXTURES_DIR").unwrap())
        .join("fsdb/fsdb0020-nine-state-ranges/waveform.fsdb");
    let mut backend = OndasBackend::open(&path).unwrap();
    let mut resolved = backend
        .resolve_expr_signal("RICH_LOGIC.RESOLVED_BIT")
        .unwrap();
    assert_eq!(resolved.expr_type.width, 1);
    assert!(resolved.expr_type.is_four_state);
    resolved.expr_type.kind = ExprTypeKind::EnumCore;
    resolved.expr_type.enum_labels = Some(vec![
        crate::expr::EnumLabelInfo {
            name: "ONE".into(),
            bits: "1".into(),
        },
        crate::expr::EnumLabelInfo {
            name: "WEAK_HIGH".into(),
            bits: "h".into(),
        },
    ]);
    assert_eq!(
        backend.sample_expr_value(&resolved, 70).unwrap(),
        SampledValue::Integral {
            bits: Some("h".into()),
            label: Some("WEAK_HIGH".into()),
        }
    );
    let payload = backend.resolve_signals(&[resolved.path.clone()]).unwrap();
    assert_eq!(
        backend.sample_resolved_optional(&payload, 70).unwrap()[0]
            .bits
            .as_deref(),
        Some("1")
    );
    assert_eq!(
        backend.sample_expr_value(&resolved, 70).unwrap(),
        SampledValue::Integral {
            bits: Some("h".into()),
            label: Some("WEAK_HIGH".into()),
        }
    );
    assert_eq!(
        resolved.expr_type.enum_labels.as_ref().unwrap()[1].bits,
        "h"
    );
}

#[test]
fn fsdb_recursive_listing_uses_structural_depth_for_escaped_scopes() {
    let source = "$timescale 1ns $end\n$scope module top $end\n$scope module block $end\n$var wire 1 ! inside $end\n$upscope $end\n$scope module \\block.dot $end\n$var wire 1 \" outside $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n";
    let fixture = GeneratedFsdbFixture::from_contents(source);
    let backend = OndasBackend::open(fixture.path()).unwrap();
    let listing = backend
        .signals_in_scope_recursive_report("top.block", None)
        .unwrap();
    assert_eq!(
        listing
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        ["top.block.inside"]
    );
    let sibling = backend.signals_in_scope(r#"top."\\block.dot""#).unwrap();
    assert_eq!(sibling[0].path, r#"top."\\block.dot".outside"#);
    let bounded = backend
        .signals_in_scope_recursive_report("top", Some(1))
        .unwrap();
    assert_eq!(
        bounded
            .entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        [r#"top."\\block.dot".outside"#, "top.block.inside"]
    );
}

#[test]
fn fsdb_scope_escape_markers_are_not_public_path_components() {
    let source: std::sync::Arc<[u8]> = b"$timescale 1ns $end\n$scope module top $end\n$scope module \\block[0] $end\n$var wire 1 ! flag $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n".as_slice().into();
    let wave = ondas::open_bytes("scope.vcd", source).unwrap();
    let index = super::HierarchyIndex::new(wave.hierarchy(), false);
    assert!(
        index
            .scopes
            .iter()
            .any(|scope| scope.path == r#"top."block[0]""#)
    );
    assert!(
        index
            .by_path
            .contains_key(&ondas::HierarchyPath::parse(r#"top."block[0]".flag"#).unwrap())
    );
}

const HIERARCHY_VCD: &str = "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$scope module mem $end\n$var wire 1 # ready $end\n$upscope $end\n$scope module cpu $end\n$var reg 1 \" valid $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n";

#[test]
fn fsdb_hierarchy_sorts_scopes_and_filters_max_depth() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$scope module z $end\n$var wire 1 ! bit $end\n$upscope $end\n$scope module a $end\n$var wire 1 \" bit $end\n$upscope $end\n$upscope $end\n$scope module alpha $end\n$var wire 1 # bit $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n",
    );
    let backend = OndasBackend::open(fixture.path()).unwrap();
    let scopes = backend.scopes_depth_first(None).unwrap();
    let expected = [("alpha", 0), ("top", 0), ("top.a", 1), ("top.z", 1)]
        .into_iter()
        .map(|(path, depth)| crate::waveform::ScopeEntry {
            path: path.into(),
            depth,
            kind: "module".into(),
        })
        .collect::<Vec<_>>();
    assert_eq!(scopes, expected);
    assert_eq!(backend.scopes_depth_first(Some(0)).unwrap(), expected[..2]);
}

#[test]
fn fsdb_hierarchy_lists_direct_and_recursive_signals() {
    let fixture = GeneratedFsdbFixture::from_contents(HIERARCHY_VCD);
    let backend = OndasBackend::open(fixture.path()).unwrap();
    let paths = |entries: Vec<crate::waveform::SignalEntry>| {
        entries
            .into_iter()
            .map(|entry| entry.path)
            .collect::<Vec<_>>()
    };
    assert_eq!(paths(backend.signals_in_scope("top").unwrap()), ["top.clk"]);
    assert_eq!(
        paths(
            backend
                .signals_in_scope_recursive_report("top", None)
                .unwrap()
                .entries
        ),
        ["top.clk", "top.cpu.valid", "top.mem.ready"]
    );
    assert_eq!(
        paths(
            backend
                .signals_in_scope_recursive_report("top", Some(0))
                .unwrap()
                .entries
        ),
        ["top.clk"]
    );
}

#[test]
fn fsdb_exact_value_avoids_second_hierarchy_index() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let paths = ["top.clk".to_string()];
    let resolved = backend.resolve_signals(&paths).unwrap();
    assert!(backend.index.get().is_none());
    let values = backend.sample_resolved_optional(&resolved, 5).unwrap();
    assert!(backend.index.get().is_none());
    let expr = backend.resolve_expr_signal("top.clk").unwrap();
    assert!(backend.index.get().is_none());
    assert_eq!(
        backend.sample_expr_value(&expr, 5).unwrap(),
        SampledValue::Integral {
            bits: Some("1".into()),
            label: None,
        }
    );

    backend.signals_in_scope("top").unwrap();
    assert!(backend.index.get().is_some());
    assert_eq!(
        backend.sample_resolved_optional(&resolved, 5).unwrap(),
        values
    );
}

#[test]
fn fsdb_batch_value_selection_keeps_full_index_lazy() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 \" data $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n#5\n1!\n0\"\n",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let paths = ["top.clk".to_string(), "top.data".to_string()];
    backend.prepare_value_signals(&paths);
    let resolved = backend.resolve_signals(&paths).unwrap();
    assert!(backend.index.get().is_none());
    let values = backend.sample_resolved_optional(&resolved, 5).unwrap();
    assert!(backend.index.get().is_none());

    let mut full = OndasBackend::open(fixture.path()).unwrap();
    full.signals_in_scope("top").unwrap();
    let expected = full.sample_resolved_optional(&full.resolve_signals(&paths).unwrap(), 5);
    assert_eq!(values, expected.unwrap());
}

#[test]
fn fsdb_repeated_scope_declarations_merge_members() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$upscope $end\n$scope module top $end\n$var event 1 \" ev $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n1\"\n",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    assert_eq!(backend.scopes_depth_first(None).unwrap().len(), 1);
    let entries = backend.signals_in_scope("top").unwrap();
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        ["top.clk", "top.ev"]
    );
    let clock = backend.resolve_expr_signal("top.clk").unwrap();
    let event = backend.resolve_expr_signal("top.ev").unwrap();
    assert_ne!(clock.id, event.id);
    assert_eq!(
        backend.sample_expr_value(&clock, 5).unwrap(),
        SampledValue::Integral {
            bits: Some("1".into()),
            label: None
        }
    );
    assert!(backend.expr_event_occurred(&event, 5).unwrap());
}

#[test]
fn fsdb_repeated_scope_declarations_preserve_sibling_subtrees() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$scope module wrapped $end\n$scope interface vif $end\n$upscope $end\n$upscope $end\n$scope interface bus[0] $end\n$var reg 1 ! value $end\n$upscope $end\n$upscope $end\n$scope module top $end\n$scope interface bus[0] $end\n$var reg 1 ! value $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n",
    );
    let backend = OndasBackend::open(fixture.path()).unwrap();
    assert_eq!(
        backend
            .scopes_depth_first(None)
            .unwrap()
            .iter()
            .map(|scope| scope.path.as_str())
            .collect::<Vec<_>>(),
        ["top", r#"top."bus[0]""#, "top.wrapped", "top.wrapped.vif"]
    );
    let entries = backend
        .signals_in_scope_recursive_report("top", None)
        .unwrap()
        .entries;
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        [r#"top."bus[0]".value"#]
    );
}

#[test]
fn fsdb_hierarchy_skips_exact_duplicate_signal_declarations() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var wire 1 ! clk $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n#5\n1!\n",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    assert_eq!(
        backend.signals_in_scope("top").unwrap(),
        vec![crate::waveform::SignalEntry {
            name: "clk".into(),
            path: "top.clk".into(),
            kind: "wire".into(),
            width: Some(1)
        }]
    );
    let signal = backend.resolve_expr_signal("top.clk").unwrap();
    assert_eq!(
        backend.sample_expr_value(&signal, 5).unwrap(),
        SampledValue::Integral {
            bits: Some("1".into()),
            label: None
        }
    );
}

#[test]
fn fsdb_hierarchy_preserves_escaped_range_looking_local_names() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 32 ! \\range.dot[31:0]  $end\n$upscope $end\n$enddefinitions $end\n#0\nb0 !\n",
    );
    let backend = OndasBackend::open(fixture.path()).unwrap();
    assert_eq!(
        backend.signals_in_scope("top").unwrap(),
        vec![crate::waveform::SignalEntry {
            name: r#""range.dot[31:0]""#.into(),
            path: r#"top."range.dot[31:0]""#.into(),
            kind: "wire".into(),
            width: Some(32)
        }]
    );
    assert!(backend.signals_in_scope("top.range").is_err());
}

#[test]
fn fsdb_hierarchy_preserves_scalar_array_element_suffixes() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! \\mem[3]  $end\n$var wire 1 \" \\flags[3]  $end\n$var wire 1 # \\flags[0]  $end\n$var wire 1 $ \\flags[0:0]  $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n0#\n0$\n",
    );
    let backend = OndasBackend::open(fixture.path()).unwrap();
    let expected = ["flags[0:0]", "flags[0]", "flags[3]", "mem[3]"]
        .into_iter()
        .map(|name| crate::waveform::SignalEntry {
            name: ondas::HierarchyPath::from_components([name]).to_string(),
            path: ondas::HierarchyPath::from_components(["top", name]).to_string(),
            kind: "wire".into(),
            width: Some(1),
        })
        .collect::<Vec<_>>();
    assert_eq!(backend.signals_in_scope("top").unwrap(), expected);
}

#[test]
fn fsdb_hierarchy_preserves_escaped_local_names_with_separators() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! \\dot.name  $end\n$var wire 1 \" \\slash/name  $end\n$var wire 32 # \\wide.dot[0]  $end\n$var wire 32 $ \\wide/slash[0]  $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\nb1 #\nb10 $\n",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let signal = backend.resolve_expr_signal(r#"top."wide.dot[0]""#).unwrap();
    assert!(backend.index.get().is_none());
    let expected = [
        ("dot.name", 1),
        ("wide.dot[0]", 32),
        ("wide/slash[0]", 32),
        ("slash/name", 1),
    ]
    .into_iter()
    .map(|(name, width)| crate::waveform::SignalEntry {
        name: ondas::HierarchyPath::from_components([name]).to_string(),
        path: ondas::HierarchyPath::from_components(["top", name]).to_string(),
        kind: "wire".into(),
        width: Some(width),
    })
    .collect::<Vec<_>>();
    assert_eq!(backend.signals_in_scope("top").unwrap(), expected);
    for path in ["top.dot", "top.slash", "top.wide"] {
        assert!(backend.signals_in_scope(path).is_err());
    }
    assert_eq!(
        backend.sample_expr_value(&signal, 0).unwrap(),
        SampledValue::Integral {
            bits: Some(format!("{:032b}", 1)),
            label: None
        }
    );
}

#[test]
fn fsdb_ambiguous_exact_paths_are_quarantined() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var reg 1 ! opcode $end\n$var reg 1 \" opcode $end\n$var reg 1 \" opcode $end\n$var wire 1 # safe $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    for depth in [Some(0), None] {
        let listing = backend
            .signals_in_scope_recursive_report("top", depth)
            .unwrap();
        assert_eq!(
            listing
                .entries
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            ["top.safe"]
        );
        assert_eq!(listing.omitted_ambiguous_paths, ["top.opcode"]);
    }
    let expected = "fatal: signal: signal 'top.opcode' is ambiguous in dump: 2 source declarations; no candidate was selected";
    let direct = backend.resolve_signals(&["top.opcode".into()]).unwrap_err();
    assert!(matches!(direct, crate::error::WavepeekError::Signal(_)));
    assert_eq!(direct.to_string(), expected);
    assert_eq!(
        backend
            .resolve_expr_signal("top.opcode")
            .unwrap_err()
            .to_string(),
        expected
    );
    let safe = backend.resolve_expr_signal("top.safe").unwrap();
    assert_eq!(
        backend.sample_expr_value(&safe, 0).unwrap(),
        SampledValue::Integral {
            bits: Some("0".into()),
            label: None
        }
    );
}

#[test]
fn fsdb_ambiguous_candidate_descriptions_are_order_independent() {
    let declarations = [
        "$var wire 8 ! opcode [7:0] $end\n$var reg 1 \" opcode $end\n",
        "$var reg 1 \" opcode $end\n$var wire 8 ! opcode [7:0] $end\n",
    ];
    let expected = "fatal: signal: signal 'top.opcode' is ambiguous in dump: 2 source declarations; no candidate was selected";
    for declarations in declarations {
        let source = format!(
            "$timescale 1ns $end\n$scope module top $end\n{declarations}$upscope $end\n$enddefinitions $end\n#0\nb0 !\n1\"\n"
        );
        let fixture = GeneratedFsdbFixture::from_contents(&source);
        let backend = OndasBackend::open(fixture.path()).unwrap();
        let error = backend.resolve_signals(&["top.opcode".into()]).unwrap_err();
        assert_eq!(error.to_string(), expected);
        assert_eq!(error.exit_code(), 1);
    }
}

#[test]
fn fsdb_scope_slashes_and_dots_remain_distinct_components() {
    let source = "$timescale 1ns $end\n$scope module \\top/a  $end\n$var wire 1 ! first $end\n$upscope $end\n$scope module \\top.a  $end\n$var wire 1 \" second $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n";
    let fixture = GeneratedFsdbFixture::from_contents(source);
    let mut backend = OndasBackend::open(fixture.path()).unwrap();

    let paths = [r#""\\top/a".first"#.into(), r#""\\top.a".second"#.into()];
    assert_eq!(
        backend.signals_in_scope(r#""\\top/a""#).unwrap()[0].path,
        paths[0]
    );
    assert_eq!(
        backend.signals_in_scope(r#""\\top.a""#).unwrap()[0].path,
        paths[1]
    );
    let signals = backend.resolve_signals(&paths).unwrap();
    let values = backend.sample_resolved_optional(&signals, 0).unwrap();
    assert_eq!(values[0].bits.as_deref(), Some("0"));
    assert_eq!(values[1].bits.as_deref(), Some("1"));
}

#[test]
fn fsdb_conflicting_scope_kinds_remain_file_errors() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! a $end\n$upscope $end\n$scope task top $end\n$var wire 1 \" b $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n0\"\n",
    );
    let error = OndasBackend::open(fixture.path()).unwrap_err();
    assert!(matches!(error, crate::error::WavepeekError::File(_)));
    assert_eq!(error.fatal_code(), Some("WPK-F0002"));
    assert_eq!(error.exit_code(), 2);
    assert!(
        error
            .to_string()
            .contains("conflicting FSDB scope top: kind \"module\" vs \"task\"")
    );
}

#[test]
fn fsdb_paths_colliding_across_owning_scopes_are_ambiguous() {
    let fixture = GeneratedFsdbFixture::from_contents(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! \\child.opcode  $end\n$scope module child $end\n$var wire 1 \" opcode $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n",
    );
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let paths = [r#"top."child.opcode""#.into(), "top.child.opcode".into()];
    for path in &paths {
        assert_eq!(
            backend
                .matching_signals(|candidate, _| candidate == path)
                .len(),
            1
        );
    }
    let resolved = backend.resolve_signals(&paths).unwrap();
    let samples = backend.sample_resolved_optional(&resolved, 0).unwrap();
    assert_eq!(samples[0].bits.as_deref(), Some("0"));
    assert_eq!(samples[1].bits.as_deref(), Some("1"));
    assert!(
        backend
            .signals_in_scope_recursive_report("top", None)
            .unwrap()
            .omitted_ambiguous_paths
            .is_empty()
    );
}

#[test]
fn fsdb_hierarchy_reports_missing_scope_and_signal_errors() {
    let fixture = GeneratedFsdbFixture::from_contents(HIERARCHY_VCD);
    let backend = OndasBackend::open(fixture.path()).unwrap();
    assert_eq!(
        backend.signals_in_scope("missing").unwrap_err().to_string(),
        "fatal: scope: scope 'missing' not found in dump"
    );
    let error = backend
        .resolve_signals(&["top.missing".into()])
        .unwrap_err();
    assert!(matches!(
        error,
        crate::error::WavepeekError::SignalNotFound(_)
    ));
    assert_eq!(
        error.to_string(),
        "fatal: signal: signal 'top.missing' not found in dump"
    );
}

#[test]
fn fsdb_reader_metadata_smoke() {
    let reader = OndasBackend::open(&cpu_fsdb_path()).expect("FSDB open failed");
    assert_eq!(reader.format_name(), "fsdb");
    assert!(!reader.metadata().unwrap().time_unit.is_empty());
    let span = reader.inner.borrow().metadata().time_span().unwrap();
    assert!(span.last() >= span.first());
}

#[test]
fn fsdb_hierarchy_excludes_hidden_subtrees() {
    let backend = OndasBackend::open(&cpu_fsdb_path()).unwrap();
    let roots = backend.scopes_depth_first(Some(0)).unwrap();
    assert_eq!(
        roots
            .iter()
            .map(|scope| scope.path.as_str())
            .collect::<Vec<_>>(),
        ["system"]
    );
    let scopes = backend.scopes_depth_first(None).unwrap();
    assert!(
        scopes
            .iter()
            .any(|scope| scope.path.ends_with(".mvsim_flag") && scope.kind == "struct")
    );
    for hidden in ["$attribute_root", "$_fsdbgate_"] {
        let prefix = format!("{hidden}.");
        assert!(
            !scopes
                .iter()
                .any(|scope| scope.path == hidden || scope.path.starts_with(&prefix))
        );
        assert!(backend.signals_in_scope(hidden).is_err());
    }
}

#[test]
fn fsdb_memory_elements_remain_exact_declarations() {
    let mut backend = OndasBackend::open(&cpu_fsdb_path()).unwrap();
    let scope = "system.i_cpu.i_CCU.i_maprom";
    let path = format!("{scope}.\"maprom[0]\"");
    let resolved = backend
        .resolve_signals(std::slice::from_ref(&path))
        .unwrap();
    assert_eq!(resolved[0].width, 8);
    assert!(backend.index.get().is_none());
    assert!(
        !backend
            .scopes_depth_first(None)
            .unwrap()
            .iter()
            .any(|entry| entry.path == format!("{scope}.maprom"))
    );
    assert!(
        backend
            .signals_in_scope(scope)
            .unwrap()
            .iter()
            .any(|entry| entry.path == path)
    );
    assert!(backend.sample_resolved_optional(&resolved, 0).is_ok());
}

#[test]
fn fsdb_hierarchy_datatype_enum_overrides_signal_kind() {
    let mut backend = OndasBackend::open(&cpu_fsdb_path()).unwrap();
    let entries = backend.signals_in_scope("system.mvsim_flag").unwrap();
    let entry = entries
        .iter()
        .find(|entry| entry.name == "assertControlType")
        .expect("SDK datatype-backed enum must retain its public declaration");
    assert_eq!(entry.kind, "enum");
    assert_eq!(entry.width, Some(2));
    let direct = backend
        .resolve_signals(&["system.mvsim_flag.assertControlType".into()])
        .unwrap();
    assert_eq!(direct[0].width, 2);
    let signal = backend
        .resolve_expr_signal("system.mvsim_flag.assertControlType")
        .unwrap();
    let SampledValue::Integral { bits, .. } = backend.sample_expr_value(&signal, 0).unwrap() else {
        panic!("enum must retain integral value encoding");
    };
    assert_eq!(bits.as_deref(), Some("00"));
}

#[test]
fn fsdb_hierarchy_datatype_enum_metadata_drives_expression_type() {
    let backend = OndasBackend::open(&cpu_fsdb_path()).unwrap();
    let resolved = backend
        .resolve_expr_signal("system.mvsim_flag.assertControlType")
        .unwrap();
    assert!(backend.index.get().is_none());
    let entries = backend.signals_in_scope("system.mvsim_flag").unwrap();
    let entry = entries
        .iter()
        .find(|entry| entry.name == "assertControlType")
        .unwrap();
    assert_eq!(entry.kind, "enum");
    assert_eq!(entry.width, Some(2));
    assert_eq!(resolved.expr_type.kind, ExprTypeKind::EnumCore);
    assert_eq!(resolved.expr_type.width, 2);
    assert_eq!(resolved.expr_type.enum_type_id.as_deref(), Some("Unknown"));
    let labels = [
        ("NO_CONTROL_ASSERT", "00"),
        ("SUSPEND_ASSERT", "01"),
        ("CONTINUE_ASSERT", "10"),
        ("KILL_ASSERT", "11"),
    ]
    .into_iter()
    .map(|(name, bits)| crate::expr::EnumLabelInfo {
        name: name.into(),
        bits: bits.into(),
    })
    .collect();
    assert_eq!(resolved.expr_type.enum_labels, Some(labels));
}

#[test]
fn fsdb_reader_hierarchy_smoke() {
    let reader = OndasBackend::open(&cpu_fsdb_path()).expect("FSDB open failed");
    let first = reader.scopes_depth_first(None).unwrap();
    assert!(!first.is_empty(), "bundled FSDB should expose scopes");
    assert_eq!(first, reader.scopes_depth_first(None).unwrap());
    for scope in &first {
        assert!(!scope.path.is_empty());
        assert!(!scope.path.contains('/'));
        assert!(STABLE_SCOPE_KIND_ALIASES.contains(&scope.kind.as_str()));
    }
    let signals = first
        .iter()
        .find_map(|scope| {
            let signals = reader
                .signals_in_scope_recursive_report(&scope.path, None)
                .unwrap()
                .entries;
            (!signals.is_empty()).then_some(signals)
        })
        .expect("bundled FSDB should expose signals");
    for signal in signals {
        assert!(!signal.path.is_empty());
        assert!(!signal.path.contains('/'));
        assert!(STABLE_SIGNAL_KIND_ALIASES.contains(&signal.kind.as_str()));
        if let Some(width) = signal.width {
            assert!(width > 0 || signal.kind == "event");
        }
    }
}

#[test]
fn fsdb_signal_session_reads_value_changes() {
    let fixture = GeneratedFsdbFixture::from_vcd("change_property_events.vcd");
    let mut reader = OndasBackend::open(fixture.path()).unwrap();
    let signal = reader.resolve_signals(&["top.armed".into()]).unwrap();
    reader
        .preload_resolved_value_changes(&signal, 1, 20)
        .unwrap();
    assert_eq!(reader.traces.len(), 1);
    assert_eq!(signal[0].width, 1);
    let trace = &reader.traces[&signal[0].id].trace;
    let ondas::ValueRef::Bits(initial) = trace.initial().unwrap().value() else {
        panic!("expected integral initial value")
    };
    assert_eq!(initial.to_string(), "0");
    let changes = trace
        .changes()
        .iter()
        .map(|change| {
            let ondas::ValueRef::Bits(bits) = change.value() else {
                panic!("expected integral change")
            };
            (change.time().ticks(), bits.to_string())
        })
        .collect::<Vec<_>>();
    assert_eq!(changes, vec![(10, "1".into()), (15, "0".into())]);
}

#[test]
fn fsdb_expr_event_occurred_rejects_non_event_signal() {
    let fixture = GeneratedFsdbFixture::from_vcd("change_property_events.vcd");
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let tick = backend.resolve_expr_signal("top.tick").unwrap();
    assert!(matches!(tick.expr_type.kind, ExprTypeKind::Event));
    assert!(backend.expr_event_occurred(&tick, 10).unwrap());
    let armed = backend.resolve_expr_signal("top.armed").unwrap();
    assert!(!matches!(armed.expr_type.kind, ExprTypeKind::Event));
    assert!(
        backend
            .expr_event_occurred(&armed, 10)
            .unwrap_err()
            .to_string()
            .contains("signal 'top.armed' is not a raw event")
    );
}

#[test]
fn fsdb_candidate_window_does_not_preload_sampled_payload() {
    let fixture = GeneratedFsdbFixture::from_vcd("change_property_events.vcd");
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let tick = backend.resolve_expr_signal("top.tick").unwrap();
    let armed = backend.resolve_expr_signal("top.armed").unwrap();
    let times = backend
        .collect_expr_candidate_times_with_mode(&[tick], 1, 20, ChangeCandidateCollectionMode::Auto)
        .unwrap();
    assert!(!times.is_empty());
    assert_eq!(backend.traces.len(), 1);
    assert_eq!(
        backend.sample_expr_value(&armed, 10).unwrap(),
        SampledValue::Integral {
            bits: Some("1".into()),
            label: None,
        }
    );
    assert_eq!(backend.traces.len(), 1);
}

#[test]
fn fsdb_timeline_cache_serves_expr_samples() {
    let fixture = GeneratedFsdbFixture::from_vcd("change_property_events.vcd");
    let mut backend = OndasBackend::open(fixture.path()).unwrap();
    let armed = backend.resolve_expr_signal("top.armed").unwrap();
    backend
        .preload_expr_value_changes(std::slice::from_ref(&armed), 1, 20)
        .unwrap();
    assert_eq!(backend.traces.len(), 1);
    for (time, bits) in [(1, "0"), (12, "1"), (20, "0")] {
        assert!(backend.cached_value(armed.id, time).is_some());
        assert_eq!(
            backend.sample_expr_value(&armed, time).unwrap(),
            SampledValue::Integral {
                bits: Some(bits.into()),
                label: None
            }
        );
    }
    assert_eq!(backend.traces.len(), 1);
}

struct GeneratedFsdbFixture {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl GeneratedFsdbFixture {
    fn from_vcd(name: &str) -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/hand")
            .join(name);
        Self::from_contents(&std::fs::read_to_string(source).unwrap())
    }

    fn from_contents(contents: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("fixture.vcd");
        std::fs::write(&source, contents).unwrap();
        let path = dir.path().join("fixture.fsdb");
        let output = Command::new("vcd2fsdb")
            .current_dir(dir.path())
            .arg(&source)
            .arg("-o")
            .arg(&path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("vcd2fsdb should be available");
        assert!(
            output.status.success(),
            "conversion failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Self { _dir: dir, path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

fn cpu_fsdb_path() -> PathBuf {
    PathBuf::from(std::env::var_os("VERDI_HOME").expect("VERDI_HOME is required"))
        .join("share/VIA/demo/waveform/cpu.fsdb")
}
