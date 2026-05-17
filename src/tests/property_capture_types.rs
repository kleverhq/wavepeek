use super::*;

#[test]
fn derives_public_capture_types() {
    for kind in [
        PropertyResultKind::Match,
        PropertyResultKind::Assert,
        PropertyResultKind::Deassert,
    ] {
        assert_eq!(kind, kind.clone());
        assert!(format!("{kind:?}").len() > 3);
        assert!(
            serde_json::to_string(&kind)
                .unwrap()
                .contains(&kind.to_string())
        );
    }

    let row = PropertyCaptureRow {
        time: "12ns".to_string(),
        kind: PropertyResultKind::Match,
    };
    assert_eq!(row.clone(), row);
    assert!(format!("{row:?}").contains("12ns"));
    assert!(serde_json::to_string(&row).unwrap().contains("match"));
}
