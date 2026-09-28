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
fn event_payload_remains_available_when_cached_between_occurrences() {
    let fixture = fixture_path("change_property_events.vcd");
    let fixture = fixture.to_str().unwrap();
    for (mode, sample_times) in [("native", ["10ns", "25ns"]), ("pre-edge", ["9ns", "24ns"])] {
        let result = run_change_json(
            fixture,
            &[
                "--signals",
                "top.tick",
                "--on",
                "posedge top.armed",
                "--sample-mode",
                mode,
            ],
        );
        let rows = result["data"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        for (row, sample_time) in rows.iter().zip(sample_times) {
            assert_eq!(row["sample_time"], sample_time);
            assert_eq!(row["signals"][0]["value"], "0'h0");
        }
    }
}

#[test]
fn split_vector_edge_after_nonzero_dump_start() {
    let fixture = tempfile::Builder::new().suffix(".vcd").tempfile().unwrap();
    std::fs::write(
        fixture.path(),
        "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! sig [0] $end\n$var wire 1 \" sig [1] $end\n$upscope $end\n$enddefinitions $end\n#10\n0!\n0\"\n#15\n1!\n",
    )
    .unwrap();
    let result = run_change_json(
        fixture.path().to_str().unwrap(),
        &["--signals", "top.sig", "--on", "posedge top.sig"],
    );
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
    assert_eq!(result["data"][0]["time"], "15ns");
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
