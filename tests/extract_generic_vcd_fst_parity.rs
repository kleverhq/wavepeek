use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn run_extract_json(waves: &str, extra_args: &[&str]) -> Value {
    let mut args = vec!["extract", "generic", "--waves", waves];
    args.extend_from_slice(extra_args);
    args.push("--json");

    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("extract generic should execute");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

#[test]
fn extract_generic_vcd_and_fst_payloads_match() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let args = [
        "--scope",
        "top",
        "--on",
        "posedge clk",
        "--when",
        "1",
        "--payload",
        "data",
        "--max",
        "5",
    ];
    let vcd_json = run_extract_json(vcd_fixture.as_str(), &args);
    let fst_json = run_extract_json(fst_fixture.as_str(), &args);

    assert_eq!(vcd_json["data"], fst_json["data"]);
    assert_eq!(vcd_json["diagnostics"], fst_json["diagnostics"]);
}
