use crate::expr::{ExprStorage, ExprType, ExprTypeKind};

use super::*;

fn bit_ty(width: u32) -> ExprType {
    ExprType {
        kind: ExprTypeKind::BitVector,
        storage: ExprStorage::PackedVector,
        width,
        is_four_state: true,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn event_ty() -> ExprType {
    ExprType {
        kind: ExprTypeKind::Event,
        storage: ExprStorage::Scalar,
        width: 0,
        is_four_state: false,
        is_signed: false,
        enum_type_id: None,
        enum_labels: None,
    }
}

fn write_small_vcd() -> tempfile::NamedTempFile {
    let file = tempfile::Builder::new()
        .suffix(".vcd")
        .tempfile()
        .expect("temp vcd");
    std::fs::write(
        file.path(),
        concat!(
            "$date\n  test\n$end\n",
            "$version\n  test\n$end\n",
            "$timescale 1ns $end\n",
            "$scope module top $end\n",
            "$var wire 1 ! a $end\n",
            "$upscope $end\n",
            "$enddefinitions $end\n",
            "#0\n0!\n#1\n1!\n",
        ),
    )
    .expect("write vcd");
    file
}

#[test]
fn invalid_signal_refs_exercise_backend_error_handlers() {
    let file = write_small_vcd();
    let mut waveform = Waveform::open(file.path()).expect("fixture should open");
    let bogus_ref = SignalRef::from_index(9999).expect("bogus signal ref should construct");
    let resolved = ResolvedSignal {
        path: "top.missing".to_string(),
        signal_ref: bogus_ref,
        width: 1,
    };
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = waveform.collect_change_times_with_mode(
                &[resolved],
                0,
                1,
                ChangeCandidateCollectionMode::Random,
            );
        }))
        .is_err()
    );

    let bogus_value = ExprResolvedSignal {
        path: "top.missing".to_string(),
        signal_ref: bogus_ref,
        expr_type: bit_ty(1),
    };
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = waveform.sample_expr_value(&bogus_value, 0);
        }))
        .is_err()
    );

    let bogus_event = ExprResolvedSignal {
        path: "top.ev".to_string(),
        signal_ref: bogus_ref,
        expr_type: event_ty(),
    };
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = waveform.expr_event_occurred(&bogus_event, 0);
        }))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = waveform.collect_expr_candidate_times_with_mode(
                &[bogus_event],
                0,
                1,
                ChangeCandidateCollectionMode::Random,
            );
        }))
        .is_err()
    );
}

#[test]
fn derive_surfaces_for_data_transfer_types() {
    let metadata = WaveformMetadata {
        time_unit: "1ns".to_string(),
        time_start: "0ns".to_string(),
        time_end: "10ns".to_string(),
    };
    assert_eq!(metadata.clone(), metadata);
    assert!(format!("{metadata:?}").contains("time_unit"));

    let scope = ScopeEntry {
        path: "top.cpu".to_string(),
        depth: 1,
        kind: "module".to_string(),
    };
    assert_eq!(scope.clone(), scope);
    assert!(format!("{scope:?}").contains("top.cpu"));

    let signal = SignalEntry {
        name: "data".to_string(),
        path: "top.data".to_string(),
        kind: "wire".to_string(),
        width: Some(8),
    };
    assert_eq!(signal.clone(), signal);
    assert!(format!("{signal:?}").contains("data"));

    let sampled = SampledSignal {
        path: "top.data".to_string(),
        width: 8,
        bits: "10101010".to_string(),
    };
    assert_eq!(sampled.clone(), sampled);
    assert!(format!("{sampled:?}").contains("10101010"));

    let state = SampledSignalState {
        path: "top.data".to_string(),
        width: 8,
        bits: None,
    };
    assert_eq!(state.clone(), state);
    assert!(format!("{state:?}").contains("None"));

    let resolved = ResolvedSignal {
        path: "top.data".to_string(),
        signal_ref: SignalRef::from_index(0).expect("signal ref"),
        width: 8,
    };
    assert_eq!(resolved.clone(), resolved);
    assert!(format!("{resolved:?}").contains("top.data"));

    let expr = ExprResolvedSignal {
        path: "top.data".to_string(),
        signal_ref: SignalRef::from_index(1).expect("signal ref"),
        expr_type: bit_ty(8),
    };
    assert_eq!(expr.clone(), expr);
    assert!(format!("{expr:?}").contains("expr_type"));

    for mode in [
        ChangeCandidateCollectionMode::Auto,
        ChangeCandidateCollectionMode::Random,
        ChangeCandidateCollectionMode::Stream,
    ] {
        assert_eq!(mode, mode.clone());
        assert!(format!("{mode:?}").len() > 3);
    }

    let offset = SignalOffsetData::new(10, 2);
    assert_eq!(offset, SignalOffsetData::new(10, 2));
    assert!(format!("{offset:?}").contains("start"));

    let edge = EdgeClassification {
        posedge: true,
        negedge: false,
    };
    assert_eq!(edge, edge.clone());
    assert!(edge.edge());
    assert!(format!("{edge:?}").contains("posedge"));
    assert!(!classify_edge("", "1").edge());
    assert!(!classify_edge("0", "").edge());
    assert!(classify_edge("0", "h").posedge);
    let mut previous = vec![Some("0".to_string()), None];
    assert!(should_emit_delta_and_update_baseline(
        &mut previous,
        &[Some("1".to_string()), Some("x".to_string())]
    ));
    assert_eq!(previous[1].as_deref(), Some("x"));
    assert!(
        normalize_time(
            u64::MAX,
            Timescale {
                factor: 2,
                unit: TimescaleUnit::NanoSeconds
            }
        )
        .is_err()
    );
}
