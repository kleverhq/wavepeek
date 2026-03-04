use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn run_change_json(waves: &str, extra_args: &[&str]) -> Value {
    let mut args = vec!["change", "--waves", waves];
    args.extend_from_slice(extra_args);
    args.push("--json");

    let output = wavepeek_cmd()
        .args(args)
        .output()
        .expect("change should execute");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

fn run_change_json_with_tune_modes(
    waves: &str,
    extra_args: &[&str],
    engine_mode: &str,
    candidate_mode: &str,
) -> Value {
    let mut args = vec!["change", "--waves", waves];
    args.extend_from_slice(&[
        "--tune-engine",
        engine_mode,
        "--tune-candidates",
        candidate_mode,
    ]);
    args.extend_from_slice(extra_args);
    args.push("--json");

    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args(args)
        .output()
        .expect("change should execute");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

#[test]
fn change_vcd_and_fst_payloads_match_for_default_trigger() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let args = [
        "--from",
        "1ns",
        "--to",
        "10ns",
        "--signals",
        "top.clk,top.data",
    ];
    let vcd_json = run_change_json(vcd_fixture.as_str(), &args);
    let fst_json = run_change_json(fst_fixture.as_str(), &args);

    assert_eq!(vcd_json["data"], fst_json["data"]);
    assert_eq!(vcd_json["warnings"], fst_json["warnings"]);
}

#[test]
fn change_vcd_and_fst_payloads_match_for_named_and_edge_triggers() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    for args in [
        vec![
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data,clk",
            "--when",
            "data",
        ],
        vec![
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data",
            "--when",
            "posedge clk",
        ],
    ] {
        let vcd_json = run_change_json(vcd_fixture.as_str(), args.as_slice());
        let fst_json = run_change_json(fst_fixture.as_str(), args.as_slice());
        assert_eq!(vcd_json["data"], fst_json["data"]);
        assert_eq!(vcd_json["warnings"], fst_json["warnings"]);
    }
}

#[test]
fn change_fst_stream_candidate_path_matches_random_access_path() {
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let args = [
        "--from",
        "1ns",
        "--to",
        "10ns",
        "--signals",
        "top.clk,top.data",
    ];

    let random_access =
        run_change_json_with_tune_modes(fst_fixture.as_str(), &args, "baseline", "random");
    let forced_stream =
        run_change_json_with_tune_modes(fst_fixture.as_str(), &args, "baseline", "stream");

    assert_eq!(random_access["data"], forced_stream["data"]);
    assert_eq!(random_access["warnings"], forced_stream["warnings"]);
}

#[test]
fn change_fst_fused_stream_candidate_path_matches_fused_random_access_path() {
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let args = [
        "--from",
        "1ns",
        "--to",
        "10ns",
        "--signals",
        "top.clk,top.data",
    ];

    let fused_random =
        run_change_json_with_tune_modes(fst_fixture.as_str(), &args, "fused", "random");
    let fused_stream =
        run_change_json_with_tune_modes(fst_fixture.as_str(), &args, "fused", "stream");

    assert_eq!(fused_random["data"], fused_stream["data"]);
    assert_eq!(fused_random["warnings"], fused_stream["warnings"]);
}
