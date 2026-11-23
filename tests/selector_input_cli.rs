use std::path::Path;

use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

const SELECTORS: &str = r#"$timescale 1ns $end
$scope module top $end
$var wire 1 $ clk $end
$var wire 8 # \leaf.name $end
$var wire 8 % \comma,name $end
$scope module \a.b $end
$var wire 8 ! data [7:0] $end
$upscope $end
$scope module \a\b $end
$var wire 8 " data [7:0] $end
$upscope $end
$upscope $end
$enddefinitions $end
#0
0$
b10100101 !
b00111100 "
b11110000 #
b11001010 %
#5
1$
b01011010 !
b10100101 "
b00001111 #
b10100101 %
"#;

fn write_fixture(text: &str, suffix: &str) -> NamedTempFile {
    let file = NamedTempFile::with_suffix(suffix).unwrap();
    std::fs::write(file.path(), text).unwrap();
    file
}

fn run_json(waves: &Path, args: &[&str]) -> Value {
    let output = wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(waves)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn value_preserves_raw_escaped_selectors_lists_and_emitted_roundtrips() {
    let waves = write_fixture(SELECTORS, ".vcd");
    let selection =
        " top.\\a.b .data[3:0] , top.\\leaf.name  , top.\\comma,name  , top.\\a.b .data[3:0] ";
    let result = run_json(
        waves.path(),
        &["value", "--signals", selection, "--at", "0ns,5ns,0ns"],
    );
    let rows = result["data"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0], rows[2]);
    for (row, expected) in rows.iter().zip([
        ["4'h5", "8'hf0", "8'hca", "4'h5"],
        ["4'ha", "8'h0f", "8'ha5", "4'ha"],
        ["4'h5", "8'hf0", "8'hca", "4'h5"],
    ]) {
        for (signal, value) in row["signals"].as_array().unwrap().iter().zip(expected) {
            assert_eq!(signal["value"], value);
        }
        assert_eq!(row["signals"][0]["path"], r#"top."a.b".data[3:0]"#);
        assert_eq!(row["signals"][1]["path"], r#"top."leaf.name""#);
        assert_eq!(row["signals"][2]["path"], r#"top."comma,name""#);
        assert_eq!(row["signals"][0], row["signals"][3]);
    }
    let quoted = run_json(
        waves.path(),
        &[
            "value",
            "--signals",
            r#"top."a.b".data[3:0]"#,
            "--at",
            "0ns",
        ],
    );
    assert_eq!(quoted["data"][0]["signals"][0], rows[0]["signals"][0]);

    let forced = run_json(
        waves.path(),
        &[
            "value",
            "--signals",
            r#"top."a\\b"."data"[3:0]"#,
            "--at",
            "0ns",
        ],
    );
    let emitted = forced["data"][0]["signals"][0]["path"].as_str().unwrap();
    assert_eq!(emitted, r#"top."a\\b".data[3:0]"#);
    let repeated = run_json(
        waves.path(),
        &["value", "--signals", emitted, "--at", "0ns"],
    );
    assert_eq!(forced["data"], repeated["data"]);
    assert_eq!(repeated["data"][0]["signals"][0]["value"], "4'hc");
}

#[test]
fn change_and_generic_queries_keep_escaped_slices_and_single_payload_entries() {
    let waves = write_fixture(SELECTORS, ".vcd");
    let selectors = [r"top.\a.b .data[3:0]", r#"top."a.b".data[3:0]"#];
    let mut previous = None;
    for selector in selectors {
        let result = run_json(
            waves.path(),
            &[
                "change",
                "--signals",
                selector,
                "--on",
                "*",
                "--sample-mode",
                "native",
            ],
        );
        let rows = result["data"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["time"], "5ns");
        assert_eq!(rows[0]["signals"][0]["value"], "4'ha");
        if let Some(previous) = previous {
            assert_eq!(result["data"], previous);
        }
        previous = Some(result["data"].clone());
        let extracted = run_json(
            waves.path(),
            &[
                "extract",
                "generic",
                "--on",
                "posedge top.clk",
                "--when",
                "1",
                "--payload",
                selector,
            ],
        );
        assert_eq!(extracted["data"].as_array().unwrap().len(), 1);
        assert_eq!(extracted["data"][0]["sample_time"], "4ns");
        assert_eq!(extracted["data"][0]["payload"][0]["value"], "4'h5");
        assert_eq!(
            extracted["data"][0]["payload"][0]["path"],
            r#"top."a.b".data[3:0]"#
        );
    }
    let source = write_fixture(&json!({
        "kind": "extract.generic.sources",
        "sources": [{"name": "capture", "on": "posedge top.clk", "when": "1", "payload": [" top.comma,name ", " top.\\leaf.name  ", "top.comma,name"]}]
    }).to_string(), ".json");
    let result = run_json(
        waves.path(),
        &[
            "extract",
            "generic",
            "--source",
            source.path().to_str().unwrap(),
        ],
    );
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
    assert_eq!(
        result["data"][0]["payload"],
        json!([
            {"path": "top.\"comma,name\"", "value": "8'hca"},
            {"path": "top.\"leaf.name\"", "value": "8'hf0"},
            {"path": "top.\"comma,name\"", "value": "8'hca"},
        ])
    );
}

#[test]
fn scoped_protocol_maps_format_resolved_escaped_leaves() {
    let source = std::fs::read_to_string(fixture_path("extract_apb4.vcd")).unwrap();
    let waves = write_fixture(&source.replace("uart_apb_p_clk_i", r"\clock.name"), ".vcd");
    let include = "^uart_apb_(presetn_i|psel_o|penable_o|pwrite_o|pready_i|p_addr_o|pprot_o|pwdata_o|pstrb_o|prdata_i|pslverr_i)$";
    let mut previous_json = None;
    let mut previous_human = None;
    for mapping in ["pclk=\\clock.name  ", "pclk=\"clock.name\""] {
        let args = [
            "extract",
            "apb",
            "--scope",
            "top",
            "--profile",
            "apb4",
            "--include",
            include,
            "--map",
            mapping,
        ];
        let result = run_json(waves.path(), &args);
        assert!(!result["data"].as_array().unwrap().is_empty());
        assert_eq!(
            result["context"]["mappings"]["pclk"]["path"],
            r#"top."clock.name""#
        );
        if let Some(previous) = previous_json {
            assert_eq!(result, previous);
        }
        previous_json = Some(result);
        let output = wavepeek_cmd()
            .args(args)
            .arg("--waves")
            .arg(waves.path())
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("pclk = \"clock.name\""));
        if let Some((stdout, stderr)) = previous_human {
            assert_eq!((&output.stdout, &output.stderr), (&stdout, &stderr));
        }
        previous_human = Some((output.stdout, output.stderr));
    }
}

#[test]
fn path_style_preserves_comma_lists_and_indexed_projection_identity() {
    let fixture = SELECTORS
        .replace(
            "$scope module \\a.b $end",
            "$var wire 8 & mem[0] [7:0] $end\n\
         $var wire 4 ' \\mem[0][3:0] $end\n\
         $scope module \\a.b $end",
        )
        .replace("0$\n", "0$\nb10110101 &\nb1111 '\n");
    let waves = write_fixture(&fixture, ".vcd");
    let selection = concat!(
        r#"top."mem"[0][3:0],top.\comma,name ,top."mem[0][3:0]","#,
        r#"top.mem[0],top."mem[0][3:0]"[1:0]"#,
    );

    for (style, expected) in [
        (
            "logical",
            [
                r#"top."mem"[0][3:0]"#,
                r#"top."comma,name""#,
                r#"top."mem[0][3:0]""#,
                "top.mem[0]",
                r#"top."mem[0][3:0]"[1:0]"#,
            ],
        ),
        (
            "sv",
            [
                r"top.\mem [0][3:0]",
                r"top.\comma,name ",
                r"top.\mem[0][3:0] ",
                "top.mem[0]",
                r"top.\mem[0][3:0] [1:0]",
            ],
        ),
    ] {
        let result = run_json(
            waves.path(),
            &[
                "value",
                "--at",
                "0ns",
                "--signals",
                selection,
                "--path-style",
                style,
            ],
        );
        let rows = result["data"][0]["signals"].as_array().unwrap();
        assert_eq!(rows.len(), expected.len());
        for ((row, path), value) in rows
            .iter()
            .zip(expected)
            .zip(["4'h5", "8'hca", "4'hf", "8'hb5", "2'h3"])
        {
            assert_eq!(*row, json!({"path": path, "value": value}));
        }
        let repeated = run_json(
            waves.path(),
            &[
                "value",
                "--at",
                "0ns",
                "--signals",
                &expected.join(","),
                "--path-style",
                style,
            ],
        );
        assert_eq!(result, repeated);

        let result = run_json(
            waves.path(),
            &[
                "extract",
                "generic",
                "--scope",
                "top",
                "--on",
                "posedge clk",
                "--when",
                "1",
                "--payload",
                selection,
                "--path-style",
                style,
            ],
        );
        for (index, row) in result["data"][0]["payload"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            assert_eq!(row["path"], rows[index]["path"]);
            assert_eq!(row["value"], rows[index]["value"]);
            assert_eq!(
                row["relative_path"],
                expected[index].strip_prefix("top.").unwrap()
            );
        }
        let change = run_json(
            waves.path(),
            &[
                "change",
                "--on",
                "posedge top.clk",
                "--signals",
                selection,
                "--sample-mode",
                "pre-edge",
                "--path-style",
                style,
            ],
        );
        assert_eq!(change["data"][0]["signals"].as_array().unwrap(), rows);
    }
}

#[test]
fn apb_styles_mappings_and_human_payload_without_changing_machine_rows() {
    let source = std::fs::read_to_string(fixture_path("extract_apb4.vcd")).unwrap();
    let waves = write_fixture(&source.replace("uart_apb_pwdata_o", r"\write.data"), ".vcd");
    let mut args = vec![
        "extract",
        "apb",
        "--scope",
        "top",
        "--profile",
        "apb4",
        "--include",
        "^uart_apb_(p_clk_i|presetn_i|psel_o|penable_o|pwrite_o|pready_i|p_addr_o|pprot_o|pstrb_o|prdata_i|pslverr_i)$",
        "--map",
        r#"pwdata="write.data""#,
        "--abs",
    ];
    let logical = run_json(waves.path(), &args);
    args.extend(["--path-style", "sv"]);
    let sv = run_json(waves.path(), &args);
    assert_eq!(
        logical["context"]["mappings"]["pwdata"]["path"],
        r#"top."write.data""#
    );
    assert_eq!(
        sv["context"]["mappings"]["pwdata"]["path"],
        r"top.\write.data "
    );
    assert_eq!(sv["data"], logical["data"]);
    assert!(
        sv["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["payload"]["pwdata"].is_string())
    );
    let stream = wavepeek_cmd()
        .args(&args)
        .arg("--jsonl")
        .arg("--waves")
        .arg(waves.path())
        .output()
        .unwrap();
    assert!(stream.status.success());
    let records = String::from_utf8(stream.stdout).unwrap();
    let records = records
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(records[0]["context"], sv["context"]);
    let rows = records
        .iter()
        .filter(|record| record["type"] == "data")
        .map(|record| &record["data"]);
    assert!(rows.eq(sv["data"].as_array().unwrap()));
    let output = wavepeek_cmd()
        .args(args)
        .arg("--waves")
        .arg(waves.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("pwdata = top.\\write.data \n"));
    assert!(stdout.contains("top.\\write.data =8'h"));
}

#[test]
fn indexed_paths_from_output_can_be_used_in_event_and_logical_expressions() {
    let waves = write_fixture(
        "$timescale 1ns $end\n\
         $scope module top $end\n$var wire 1 ! clk $end\n\
         $scope module logic[0] $end\n$var wire 1 \" q $end\n$var wire 2 # bus [1:0] $end\n\
         $upscope $end\n\
         $scope module logic[-1] $end\n$var wire 1 \" q $end\n$var wire 2 # bus [1:0] $end\n\
         $upscope $end\n$upscope $end\n$enddefinitions $end\n\
         #0\n0!\n0\"\nb10 #\n#5\n1!\n1\"\n",
        ".vcd",
    );
    let sampled = run_json(
        waves.path(),
        &[
            "value",
            "--at",
            "0ns",
            "--signals",
            "top.logic[0].q,top.logic[-1].q",
            "--path-style",
            "sv",
        ],
    );
    let signals = sampled["data"][0]["signals"].as_array().unwrap();
    assert_eq!(signals.len(), 2);
    for (signal, index) in signals.iter().zip([0, -1]) {
        let sv = signal["path"].as_str().unwrap();
        assert_eq!(sv, format!(r"top.\logic [{index}].q"));
        let logical = format!("top.logic[{index}].q");
        let quoted = format!(r#"top."logic"[{index}].q"#);
        for path in [logical.as_str(), quoted.as_str(), sv] {
            let change = run_json(
                waves.path(),
                &[
                    "change",
                    "--on",
                    &format!("posedge {path}"),
                    "--sample-mode",
                    "native",
                    "--signals",
                    path,
                    "--path-style",
                    "sv",
                ],
            );
            assert_eq!(change["data"][0]["time"], "5ns");
            assert_eq!(
                change["data"][0]["signals"][0],
                json!({"path": sv, "value": "1'h1"})
            );
            let extracted = run_json(
                waves.path(),
                &[
                    "extract",
                    "generic",
                    "--on",
                    "posedge top.clk",
                    "--when",
                    &format!(
                        "{}.bus[0] == 1'b0 && {path} == 0",
                        path.strip_suffix(".q").unwrap()
                    ),
                    "--payload",
                    path,
                ],
            );
            assert_eq!(extracted["data"][0]["payload"][0]["value"], "1'h0");
        }
    }
}

#[test]
fn escaped_event_paths_support_triggered_postfix() {
    let waves = write_fixture(
        r#"$timescale 1ns $end
$scope module top $end
$var wire 1 ! clk $end
$var event 1 # \ev.pulse $end
$upscope $end
$enddefinitions $end
#0
0!
#5
1#
#6
1!
"#,
        ".vcd",
    );
    for predicate in [
        r#"top."ev.pulse".triggered()"#,
        r"top.\ev.pulse .triggered()",
    ] {
        let result = run_json(
            waves.path(),
            &[
                "extract",
                "generic",
                "--on",
                "posedge top.clk",
                "--when",
                predicate,
                "--payload",
                "top.clk",
            ],
        );
        assert_eq!(result["data"].as_array().unwrap().len(), 1);
        assert_eq!(result["data"][0]["time"], "6ns");
        assert_eq!(result["data"][0]["sample_time"], "5ns");
    }
}

#[test]
fn scoped_axistream_uses_full_mappings_for_clock_predicate_and_payload() {
    let waves = write_fixture(
        r#"$timescale 1ns $end
$scope module top $end
$var wire 1 ! clock $end
$var wire 1 " valid $end
$var wire 8 # data [7:0] $end
$scope module top $end
$var wire 1 % clock $end
$var wire 1 & valid $end
$var wire 8 $ data [7:0] $end
$upscope $end
$upscope $end
$enddefinitions $end
#0
0!
0"
b10101010 #
0%
1&
b10100101 $
#5
1%
#10
1!
"#,
        ".vcd",
    );
    for style in ["logical", "sv"] {
        let result = run_json(
            waves.path(),
            &[
                "extract",
                "axistream",
                "--scope",
                "top",
                "--tready-mode",
                "implicit-high",
                "--map",
                r#"aclk=top.top."clock"[0:0]"#,
                "--map",
                r#"tvalid=top.top."valid"[0:0]"#,
                "--map",
                r#"tdata=top.top."data"[3:0]"#,
                "--path-style",
                style,
            ],
        );
        assert_eq!(
            result["context"]["mappings"]["tdata"]["path"],
            "top.top.data[3:0]"
        );
        assert_eq!(result["data"].as_array().unwrap().len(), 1);
        assert_eq!(result["data"][0]["time"], "5ns");
        assert_eq!(result["data"][0]["payload"]["tdata"], "4'h5");
    }
}

#[test]
fn axistream_path_style_preserves_automatic_real_predicate_mapping() {
    let waves = write_fixture(
        "$timescale 1ns $end\n$scope module top $end\n\
         $var wire 1 ! aclk $end\n$var real 1 \" tvalid $end\n\
         $var wire 1 # tready $end\n$var wire 8 % tdata [7:0] $end\n\
         $upscope $end\n$enddefinitions $end\n#0\n0!\nr1 \"\n1#\nb10101010 %\n#5\n1!\n",
        ".vcd",
    );
    let mut args = vec![
        "extract",
        "axistream",
        "--include",
        "^(aclk|tvalid|tready|tdata)$",
    ];
    let logical = run_json(waves.path(), &args);
    assert_eq!(
        logical["context"]["mappings"]["tvalid"]["path"],
        "top.tvalid"
    );
    assert_eq!(logical["data"][0]["payload"]["tdata"], "8'haa");
    args.extend(["--path-style", "sv"]);
    assert_eq!(run_json(waves.path(), &args), logical);
}
