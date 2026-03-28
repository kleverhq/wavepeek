use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn run_property_json(waves: &str, extra_args: &[&str]) -> Value {
    let mut args = vec!["property", "--waves", waves];
    args.extend_from_slice(extra_args);
    args.push("--json");

    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("property should execute");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

#[test]
fn property_vcd_and_fst_payloads_match_for_match_capture() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let args = [
        "--scope",
        "top",
        "--on",
        "posedge clk",
        "--eval",
        "data == 8'h00",
        "--capture",
        "match",
    ];
    let vcd_json = run_property_json(vcd_fixture.as_str(), &args);
    let fst_json = run_property_json(fst_fixture.as_str(), &args);

    assert_eq!(vcd_json["data"], fst_json["data"]);
    assert_eq!(vcd_json["warnings"], fst_json["warnings"]);
}

#[test]
fn property_vcd_and_fst_payloads_match_for_switch_capture() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let args = [
        "--scope",
        "top",
        "--eval",
        "data == 8'h0f",
        "--capture",
        "switch",
    ];
    let vcd_json = run_property_json(vcd_fixture.as_str(), &args);
    let fst_json = run_property_json(fst_fixture.as_str(), &args);

    assert_eq!(vcd_json["data"], fst_json["data"]);
    assert_eq!(vcd_json["warnings"], fst_json["warnings"]);
}
