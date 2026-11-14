use serde_json::Value;

mod common;
use common::wavepeek_cmd;

const BROKEN_VCD: &[u8] = b"$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$upscope $end\n$enddefinitions $end\n#0\nQ!\n";

#[test]
fn malformed_vcd_cause_survives_fsdb_suffix_and_output_mode() {
    let directory = tempfile::tempdir().unwrap();
    for suffix in ["vcd", "fsdb"] {
        let path = directory.path().join(format!("broken.{suffix}"));
        std::fs::write(&path, BROKEN_VCD).unwrap();
        for command in ["info", "scope", "value"] {
            for mode in [None, Some("--json"), Some("--jsonl")] {
                let mut cmd = wavepeek_cmd();
                cmd.args([command, "--waves"]).arg(&path);
                if command == "value" {
                    cmd.args(["--signals", "top.clk", "--at", "0ns"]);
                }
                if let Some(mode) = mode {
                    cmd.arg(mode);
                }
                let output = cmd.output().unwrap();
                assert_eq!(output.status.code(), Some(2), "{command} {suffix} {mode:?}");
                let message = if let Some(mode) = mode {
                    assert!(output.stderr.is_empty(), "{output:?}");
                    let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
                    assert_eq!(fatal["type"], "fatal");
                    assert_eq!(fatal["code"], "WPK-F0002");
                    assert!(fatal.get("command").is_none());
                    if mode == "--jsonl" {
                        assert_eq!(fatal["seq"], 0);
                    } else {
                        assert!(fatal.get("seq").is_none());
                    }
                    fatal["message"].as_str().unwrap().to_owned()
                } else {
                    assert!(output.stdout.is_empty());
                    let message = String::from_utf8(output.stderr).unwrap();
                    assert!(message.starts_with("fatal: file: "));
                    message
                };
                assert!(message.contains("malformed Vcd"), "{message}");
                assert!(message.contains(path.to_str().unwrap()));
                assert!(!message.contains("--features fsdb"), "{message}");
            }
        }
    }
}

#[test]
fn non_ascii_string_is_a_value_support_failure_in_machine_modes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("non_ascii.vcd");
    std::fs::write(
        &path,
        b"$timescale 1ns $end\n$var string 1 ! msg $end\n$enddefinitions $end\n#0\ns\xc3\xa9 !\n",
    )
    .unwrap();
    for mode in ["--json", "--jsonl"] {
        let output = wavepeek_cmd()
            .args(["scope", "--waves"])
            .arg(&path)
            .arg(mode)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
        let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(fatal["code"], "WPK-F0004");
        assert!(
            fatal["message"]
                .as_str()
                .unwrap()
                .contains("non-ASCII string")
        );
    }
}

#[test]
fn lazy_fst_expression_reads_preserve_signal_failures_and_late_jsonl_framing() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("non_ascii.vcd");
    let fst = directory.path().join("non_ascii.fst");
    std::fs::write(&source, b"$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var string 1 \" msg $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nsok \"\n#4\ns\xc3\xa9 \"\n#5\n1!\n#10\n0!\n").unwrap();
    let conversion = std::process::Command::new("vcd2fst")
        .arg(&source)
        .arg(&fst)
        .output()
        .unwrap();
    assert!(conversion.status.success(), "{conversion:?}");
    let cases: &[&[&str]] = &[
        &[
            "property",
            "--on",
            "posedge top.clk",
            "--eval",
            "top.msg == \"ok\"",
            "--sample-mode",
            "native",
        ],
        &[
            "property",
            "--on",
            "posedge top.clk iff (top.msg == \"ok\")",
            "--eval",
            "1",
            "--sample-mode",
            "native",
        ],
        &[
            "change",
            "--on",
            "posedge top.clk iff (top.msg == \"ok\")",
            "--signals",
            "top.clk",
            "--sample-mode",
            "native",
        ],
        &[
            "extract",
            "generic",
            "--on",
            "posedge top.clk",
            "--when",
            "top.msg == \"ok\"",
            "--payload",
            "top.clk",
        ],
    ];
    for args in cases {
        for mode in ["--json", "--jsonl"] {
            let output = wavepeek_cmd()
                .args(*args)
                .arg("--waves")
                .arg(&fst)
                .args(["--max", "1", mode])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1), "{args:?} {output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            let records: Vec<Value> = std::str::from_utf8(&output.stdout)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let fatal = records.last().unwrap();
            assert_eq!(fatal["type"], "fatal");
            assert_eq!(fatal["code"], "WPK-F0004", "{args:?} {fatal}");
            assert!(
                fatal["message"]
                    .as_str()
                    .unwrap()
                    .contains("non-ASCII string")
            );
            assert!(
                fatal["message"]
                    .as_str()
                    .unwrap()
                    .contains(fst.to_str().unwrap())
            );
            if mode == "--jsonl" {
                assert_eq!(records.len(), 2, "{args:?} {records:?}");
                assert_eq!(records[0]["type"], "begin");
                assert_eq!(records[0]["seq"], 0);
                assert_eq!(fatal["seq"], 1);
            } else {
                assert_eq!(records.len(), 1);
                assert!(fatal.get("seq").is_none());
            }
        }
    }
}
