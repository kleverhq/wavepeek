use super::*;

#[test]
fn derives_private_state_and_mode_helpers() {
    let estimate = AutoDispatchWorkEstimate {
        fused_work: 7,
        edge_work: 11,
    };
    assert_eq!(estimate, estimate.clone());
    assert!(format!("{estimate:?}").contains("fused_work"));

    let signal = ChangeSignalValue {
        display: "sig".to_string(),
        path: "top.sig".to_string(),
        value: "1'b1".to_string(),
    };
    assert_eq!(signal.clone(), signal);
    assert!(serde_json::to_string(&signal).unwrap().contains("top.sig"));
    assert!(format!("{signal:?}").contains("display"));

    let snapshot = ChangeSnapshot {
        time: "5ns".to_string(),
        signals: vec![signal.clone()],
    };
    assert_eq!(snapshot.clone(), snapshot);
    assert!(serde_json::to_string(&snapshot).unwrap().contains("5ns"));
    assert!(format!("{snapshot:?}").contains("signals"));

    let requested = RequestedSignal {
        display: "sig".to_string(),
        path: "top.sig".to_string(),
    };
    assert_eq!(requested.clone(), requested);
    assert!(format!("{requested:?}").contains("top.sig"));

    for mode in [
        ChangeEngineMode::Baseline,
        ChangeEngineMode::Fused,
        ChangeEngineMode::EdgeFast,
    ] {
        assert_eq!(mode, mode.clone());
        assert!(format!("{mode:?}").len() > 3);
    }

    let output = ChangeRunOutput {
        snapshots: vec![snapshot],
        truncated: true,
    };
    assert!(format!("{output:?}").contains("truncated"));

    let rolling = RollingSignalState {
        offset: None,
        bits: Some("10".to_string()),
    };
    assert_eq!(rolling.clone().bits, rolling.bits);
    assert!(format!("{rolling:?}").contains("offset"));

    let cached = CachedEventSamples {
        current: SampledValue::Integral {
            bits: Some("1".to_string()),
            label: None,
        },
        previous: SampledValue::Integral {
            bits: Some("0".to_string()),
            label: None,
        },
    };
    assert!(matches!(
        cached.clone().current,
        SampledValue::Integral { .. }
    ));
}
