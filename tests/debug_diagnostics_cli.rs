use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn run_debug_json(args: &[String]) -> Value {
    let output = wavepeek_cmd()
        .env("DEBUG", "1")
        .args(args)
        .output()
        .expect("command should execute");
    assert!(
        output.status.success(),
        "command should succeed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout should be valid json")
}

#[test]
fn debug_mode_emits_performance_diagnostics_for_all_waveform_commands() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();
    let cases = [
        ("info", vec!["info", "--waves", fixture.as_str(), "--json"]),
        (
            "scope",
            vec!["scope", "--waves", fixture.as_str(), "--json"],
        ),
        (
            "signal",
            vec![
                "signal",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--json",
            ],
        ),
        (
            "value",
            vec![
                "value",
                "--waves",
                fixture.as_str(),
                "--at",
                "10ns",
                "--signals",
                "top.clk",
                "--json",
            ],
        ),
        (
            "change",
            vec![
                "change",
                "--waves",
                fixture.as_str(),
                "--signals",
                "top.clk",
                "--json",
            ],
        ),
        (
            "property",
            vec![
                "property",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--on",
                "clk",
                "--eval",
                "clk",
                "--capture",
                "match",
                "--json",
            ],
        ),
    ];

    for (command_name, args) in cases {
        let args = args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>();
        let value = run_debug_json(args.as_slice());
        assert_eq!(value["command"], command_name);
        let diagnostics = value["diagnostics"]
            .as_array()
            .expect("diagnostics should be an array");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic["kind"] == "debug"
                && diagnostic["code"] == "WPK-D1001"
                && diagnostic["details"]["event"] == "context"
                && diagnostic["details"]["command"] == command_name
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic["kind"] == "debug"
                && diagnostic["code"] == "WPK-D1002"
                && diagnostic["details"]["event"] == "phase"
                && diagnostic["details"]["phase"] == "backend.open"
                && diagnostic["details"]["duration_ns"].is_u64()
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic["kind"] == "debug"
                && diagnostic["code"] == "WPK-D1003"
                && diagnostic["details"]["event"] == "summary"
                && diagnostic["details"]["total_duration_ns"].is_u64()
        }));

        let serialized = serde_json::to_string(diagnostics).expect("diagnostics should serialize");
        assert!(!serialized.contains("1'h"));
    }
}
