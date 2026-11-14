use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn run_value(waves: &str, times: &str) -> Value {
    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            waves,
            "--scope",
            "top",
            "--signals",
            "data[7:4],data[0:0],data,data[7:4]",
            "--at",
            times,
            "--json",
        ])
        .output()
        .expect("value should execute");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

#[test]
fn value_vcd_and_fst_projected_payloads_match() {
    let vcd = fixture_path("m2_core.vcd");
    let fst = fixture_path("m2_core.fst");
    let vcd = run_value(vcd.to_str().unwrap(), "10ns,0ns,10ns,5ns");
    let fst = run_value(fst.to_str().unwrap(), "10ns,0ns,10ns,5ns");

    assert_eq!(vcd["data"], fst["data"]);
    assert_eq!(vcd["diagnostics"], fst["diagnostics"]);
    assert_eq!(
        vcd["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["time"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["10ns", "0ns", "10ns", "5ns"]
    );
    assert_eq!(vcd["data"][0], vcd["data"][2]);
    for row in vcd["data"].as_array().unwrap() {
        assert_eq!(row["signals"][0], row["signals"][3]);
    }
}

#[test]
fn value_single_time_and_repeated_batch_match_across_formats() {
    for waves in [fixture_path("m2_core.vcd"), fixture_path("m2_core.fst")] {
        let point = run_value(waves.to_str().unwrap(), "10ns");
        let batch = run_value(waves.to_str().unwrap(), "10ns,10ns");
        assert_eq!(point["data"].as_array().unwrap().len(), 1);
        assert_eq!(batch["data"].as_array().unwrap().len(), 2);
        assert_eq!(batch["data"][0], point["data"][0]);
        assert_eq!(batch["data"][1], point["data"][0]);
    }
}
