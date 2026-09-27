use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn assert_debug_stderr_is_well_formed(stderr: &[u8]) {
    let stderr = String::from_utf8(stderr.to_vec()).expect("stderr should be utf8");
    let lines = stderr.lines().collect::<Vec<_>>();
    assert!(!lines.is_empty(), "DEBUG=1 should emit debug events");
    for line in lines {
        let event: Value = serde_json::from_str(line).expect("debug line should be json");
        assert_eq!(event["kind"], "debug");
        assert!(event["message"].is_string());
        assert!(event["timestamp_ns"].is_u64());
        assert!(event["details"].is_object());
    }
}

fn run_change_json(waves: &str, extra_args: &[&str]) -> Value {
    let mut args = vec!["change", "--waves", waves];
    if !extra_args.contains(&"--on") {
        args.extend_from_slice(&["--on", "*"]);
    }
    if !extra_args.contains(&"--sample-mode") {
        args.extend_from_slice(&["--sample-mode", "native"]);
    }
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
    if !extra_args.contains(&"--on") {
        args.extend_from_slice(&["--on", "*"]);
    }
    if !extra_args.contains(&"--sample-mode") {
        args.extend_from_slice(&["--sample-mode", "native"]);
    }
    args.extend_from_slice(extra_args);
    args.push("--json");

    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args(args)
        .output()
        .expect("change should execute");
    assert!(output.status.success());
    assert_debug_stderr_is_well_formed(&output.stderr);
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

#[test]
fn change_vcd_and_fst_payloads_match_for_explicit_wildcard_native_trigger() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    for (row_mode, row_values) in [
        ("dense", "full"),
        ("dense", "delta"),
        ("sparse", "full"),
        ("sparse", "delta"),
    ] {
        let args = [
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data[7:4],top.data[5:2],top.data",
            "--row-mode",
            row_mode,
            "--row-values",
            row_values,
        ];
        let vcd_json = run_change_json(vcd_fixture.as_str(), &args);
        let fst_json = run_change_json(fst_fixture.as_str(), &args);

        assert_eq!(vcd_json["data"], fst_json["data"]);
        assert_eq!(vcd_json["diagnostics"], fst_json["diagnostics"]);
    }
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
            "--on",
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
            "--on",
            "posedge clk",
        ],
        vec![
            "--from",
            "0ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--on",
            "data",
        ],
    ] {
        let vcd_json = run_change_json(vcd_fixture.as_str(), args.as_slice());
        let fst_json = run_change_json(fst_fixture.as_str(), args.as_slice());
        assert_eq!(vcd_json["data"], fst_json["data"]);
        assert_eq!(vcd_json["diagnostics"], fst_json["diagnostics"]);
    }
}

#[test]
fn change_vcd_and_fst_payloads_match_for_typed_iff_trigger() {
    let vcd_fixture = fixture_path("m2_core.vcd");
    let vcd_fixture = vcd_fixture.to_string_lossy().into_owned();
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    let args = [
        "--scope",
        "top",
        "--signals",
        "data,clk",
        "--on",
        "posedge clk iff data == 8'h00",
    ];
    let vcd_json = run_change_json(vcd_fixture.as_str(), &args);
    let fst_json = run_change_json(fst_fixture.as_str(), &args);

    assert_eq!(vcd_json["data"], fst_json["data"]);
    assert_eq!(vcd_json["diagnostics"], fst_json["diagnostics"]);
}

#[test]
fn change_fst_stream_candidate_path_matches_random_access_for_all_row_modes() {
    let fst_fixture = fixture_path("m2_core.fst");
    let fst_fixture = fst_fixture.to_string_lossy().into_owned();

    for engine in ["baseline", "fused"] {
        for (row_mode, row_values) in [
            ("dense", "full"),
            ("dense", "delta"),
            ("sparse", "full"),
            ("sparse", "delta"),
        ] {
            let args = [
                "--from",
                "1ns",
                "--to",
                "10ns",
                "--signals",
                "top.clk,top.data[7:4],top.data[5:2],top.data",
                "--row-mode",
                row_mode,
                "--row-values",
                row_values,
            ];
            let random =
                run_change_json_with_tune_modes(fst_fixture.as_str(), &args, engine, "random");
            let stream =
                run_change_json_with_tune_modes(fst_fixture.as_str(), &args, engine, "stream");

            assert_eq!(random["data"], stream["data"]);
            assert_eq!(random["diagnostics"], stream["diagnostics"]);
        }
    }
}
