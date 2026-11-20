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
