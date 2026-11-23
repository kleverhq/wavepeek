use std::path::Path;

use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::wavepeek_cmd;

const LOGICAL_SCOPE: &str = r#"top."u_reg[0]""#;
const SV_SCOPE: &str = "top.\\u_reg[0] ";
const VCD_LITERAL: &str = r#"$scope module \u_reg[0] $end
$var reg 1 ! q $end
$upscope $end
"#;
const VCD_INDEXED: &str = "$scope module u_reg[0] $end\n$var reg 1 ~ q $end\n$upscope $end\n";

fn vcd(indexed: &str, samples: &str) -> NamedTempFile {
    let file = NamedTempFile::with_suffix(".vcd").unwrap();
    std::fs::write(
        file.path(),
        format!(
            "$timescale 1ns $end\n$scope module top $end\n{VCD_LITERAL}{indexed}\
             $upscope $end\n$enddefinitions $end\n#0\n1!\n{samples}#1\n"
        ),
    )
    .unwrap();
    file
}

fn run(path: &Path, args: &[&str]) -> String {
    let result = wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    String::from_utf8(result.stdout).unwrap()
}

fn check_instance_paths(path: &Path, has_indexed: bool) {
    for (style, scope, input) in [
        ("logical", LOGICAL_SCOPE, SV_SCOPE),
        ("sv", SV_SCOPE, LOGICAL_SCOPE),
    ] {
        let output = run(path, &["scope", "--path-style", style]);
        assert!(
            output
                .lines()
                .any(|line| line == format!("1 {scope} kind=module"))
        );
        let listed: Value =
            serde_json::from_str(&run(path, &["scope", "--path-style", style, "--json"])).unwrap();
        let scope_row = listed["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["path"] == scope)
            .unwrap();
        assert_eq!(scope_row["depth"], 1);

        let signal = format!("{scope}.q");
        let selection = format!("{input}.q");
        let output = run(
            path,
            &[
                "value",
                "--at",
                "0ns",
                "--signals",
                &selection,
                "--path-style",
                style,
            ],
        );
        assert_eq!(output, format!("@0ns {signal}=1'h1\n"));

        let list = if has_indexed {
            format!("top.u_reg[0].q,{selection}")
        } else {
            selection
        };
        let output = run(
            path,
            &[
                "value",
                "--at",
                "0ns",
                "--signals",
                &list,
                "--path-style",
                style,
                "--jsonl",
            ],
        );
        let data: Value = serde_json::from_str(output.lines().nth(1).unwrap()).unwrap();
        let mut expected = vec![json!({"path": signal, "value": "1'h1"})];
        if has_indexed {
            expected.insert(0, json!({"path": "top.u_reg[0].q", "value": "1'h0"}));
        }
        assert_eq!(data["data"]["signals"], json!(expected));
        for item in expected {
            let selector = item["path"].as_str().unwrap();
            let output = run(
                path,
                &["value", "--at", "0ns", "--signals", selector, "--json"],
            );
            let sampled: Value = serde_json::from_str(&output).unwrap();
            assert_eq!(sampled["data"][0]["signals"][0]["value"], item["value"]);
        }

        let scope = scope_row["path"].as_str().unwrap();
        let output = run(
            path,
            &["signal", "--scope", scope, "--path-style", style, "--json"],
        );
        let result: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(result["context"]["scope"], scope);
        assert!(
            result["data"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["path"] == signal && row["relative_path"] == "q")
        );
    }
    if !has_indexed {
        let output = wavepeek_cmd()
            .args([
                "value",
                "--at",
                "0ns",
                "--signals",
                "top.u_reg[0].q",
                "--waves",
            ])
            .arg(path)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("not found"));
    }
}

#[test]
fn vcd_escaped_instance_paths_are_reusable() {
    check_instance_paths(vcd("", "").path(), false);
    check_instance_paths(vcd(VCD_INDEXED, "0~\n").path(), true);
}

#[test]
fn sv_output_uses_escape_metadata_from_the_dump() {
    let waves = vcd(
        "$scope module \\plain $end\n$var wire 1 ~ \\q $end\n$upscope $end\n",
        "0~\n",
    );
    let json: Value = serde_json::from_str(&run(
        waves.path(),
        &[
            "signal",
            "--scope",
            "top.plain",
            "--path-style",
            "sv",
            "--json",
        ],
    ))
    .unwrap();
    assert_eq!(json["context"]["scope"], r"top.\plain ");
    assert_eq!(json["data"][0]["path"], r"top.\plain .\q ");
    assert_eq!(json["data"][0]["name"], r"\q ");
    assert_eq!(json["data"][0]["relative_path"], r"\q ");
}

#[test]
fn signal_limit_uses_logical_leaf_order_in_both_styles() {
    let waves = vcd(
        "$var wire 1 ~ a $end\n$var wire 1 % \\z.dot $end\n",
        "0~\n0%\n",
    );
    for (style, expected) in [("logical", r#"top."z.dot""#), ("sv", r"top.\z.dot ")] {
        let result: Value = serde_json::from_str(&run(
            waves.path(),
            &[
                "signal",
                "--scope",
                "top",
                "--max",
                "1",
                "--path-style",
                style,
                "--json",
            ],
        ))
        .unwrap();
        assert_eq!(result["data"][0]["path"], expected);
    }
}

#[test]
fn style_preserves_scope_filtering_and_lexical_order_before_limit() {
    let waves = vcd(
        "$scope module u[2] $end\n$var reg 1 ~ q $end\n$upscope $end\n\
         $scope module u[10] $end\n$var reg 1 % q $end\n$upscope $end\n",
        "0~\n0%\n",
    );
    for style in ["logical", "sv"] {
        let args = ["--path-style", style, "--max", "1", "--json"];
        let scope = run(
            waves.path(),
            &[&["scope", "--filter", r"^top\.u\["][..], &args].concat(),
        );
        let scope: Value = serde_json::from_str(&scope).unwrap();
        assert_eq!(scope["summary"]["total"], 2);
        assert_eq!(scope["data"][0]["path"], "top.u[10]");
        let signal = run(
            waves.path(),
            &[
                &["signal", "--scope", "top", "--recursive", "--filter", "^q$"][..],
                &args,
            ]
            .concat(),
        );
        let signal: Value = serde_json::from_str(&signal).unwrap();
        assert_eq!(signal["summary"]["total"], 3);
        assert_eq!(signal["data"][0]["path"], "top.u[10].q");
    }
}

#[test]
fn sv_formats_only_listed_rows_and_fails_before_jsonl_begin() {
    let waves = vcd(
        "$scope module zété $end\n$var reg 1 ~ q $end\n$upscope $end\n",
        "0~\n",
    );
    for prefix in [
        vec!["scope"],
        vec!["signal", "--scope", "top", "--recursive"],
    ] {
        let logical: Value = serde_json::from_str(&run(
            waves.path(),
            &[prefix.as_slice(), &["--json"]].concat(),
        ))
        .unwrap();
        assert!(
            logical["data"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["path"].as_str().unwrap().contains("zété"))
        );
        let options = ["--path-style", "sv", "--jsonl"];
        let limited = run(
            waves.path(),
            &[prefix.as_slice(), &options, &["--max", "1"]].concat(),
        );
        assert!(limited.contains("\"type\":\"end\""));
        let filter = if prefix[0] == "scope" {
            "^top$"
        } else {
            "^other$"
        };
        let filtered = run(
            waves.path(),
            &[prefix.as_slice(), &options, &["--filter", filter]].concat(),
        );
        assert!(filtered.contains("\"type\":\"end\""));

        let output = wavepeek_cmd()
            .args(prefix)
            .args(options)
            .arg("--waves")
            .arg(waves.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(fatal["type"], "fatal");
        assert_eq!(fatal["seq"], 0);
        assert!(String::from_utf8_lossy(&output.stdout).contains("--path-style sv"));
    }
}

#[test]
fn sv_relative_text_can_omit_an_unrepresentable_scope() {
    let waves = vcd(
        "$scope module zété $end\n$var reg 1 ~ q $end\n$upscope $end\n",
        "0~\n",
    );
    let selection = [
        "value",
        "--scope",
        "top.zété",
        "--signals",
        "q",
        "--at",
        "0ns",
        "--path-style",
        "sv",
    ];
    assert_eq!(run(waves.path(), &selection), "@0ns q=1'h0\n");
    let summary = run(
        waves.path(),
        &[
            "signal",
            "--scope",
            "top.zété",
            "--summary",
            "--path-style",
            "sv",
        ],
    );
    assert!(summary.contains("complete: true"));

    for absolute in ["--abs", "--jsonl"] {
        let output = wavepeek_cmd()
            .args(selection)
            .arg(absolute)
            .arg("--waves")
            .arg(waves.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        if absolute == "--jsonl" {
            let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(fatal["code"], "WPK-F0001");
            assert_eq!(fatal["seq"], 0);
        } else {
            assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be represented"));
        }
    }
}

#[cfg(feature = "shm")]
#[test]
fn shm_escaped_instance_paths_are_reusable() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shm/escaped_instances.shm");
    check_instance_paths(&path, true);
}
