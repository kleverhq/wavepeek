use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

mod common;
use common::wavepeek_cmd;

const EXACT_VCD: &str = r#"$timescale 1ns $end
$var wire 8 0 \root.name $end
$scope module top $end
$var wire 8 ! \literal[7:0] $end
$var wire 8 " \literal $end
$var wire 8 ! alias $end
$var wire 8 # \a.b.c $end
$var wire 8 $ \comma,name[3:0] $end
$var wire 1 % clk $end
$var wire 8 9 \comma,alone $end
$scope module \a.b $end
$var wire 8 & c $end
$upscope $end
$upscope $end
$enddefinitions $end
#0
b00110011 0
b10100101 !
b11110000 "
b00010001 #
b11000011 $
0%
b00110110 9
b00100010 &
#5
1%
#10
b01011010 !
0%
"#;

struct Fixture {
    _dir: tempfile::TempDir,
    paths: [PathBuf; 2],
}

impl Fixture {
    fn new(source: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let vcd = dir.path().join("identity.vcd");
        let fst = dir.path().join("identity.fst");
        std::fs::write(&vcd, source).unwrap();
        let output = Command::new("vcd2fst")
            .arg(&vcd)
            .arg(&fst)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self {
            _dir: dir,
            paths: [vcd, fst],
        }
    }
}

fn run(path: &Path, command: &str, args: &[&str]) -> Value {
    let mut cmd = wavepeek_cmd();
    cmd.arg(command);
    let args = if command == "extract" {
        cmd.arg(args[0]);
        &args[1..]
    } else {
        args
    };
    let output = cmd
        .arg("--waves")
        .arg(path)
        .args(args)
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
fn listed_exact_paths_round_trip_through_values_scopes_and_expression_binding() {
    let fixture = Fixture::new(EXACT_VCD);
    for path in &fixture.paths {
        let listing = run(
            path,
            "signal",
            &["--scope", "top", "--recursive", "--max", "unlimited"],
        );
        assert_eq!(listing["diagnostics"], json!([]));
        let expected = [
            (r#"top."a.b.c""#, "8'h11"),
            ("top.alias", "8'ha5"),
            ("top.clk", "1'h0"),
            (r#"top."comma,alone""#, "8'h36"),
            (r#"top."comma,name[3:0]""#, "8'hc3"),
            ("top.literal", "8'hf0"),
            (r#"top."literal[7:0]""#, "8'ha5"),
            (r#"top."a.b".c"#, "8'h22"),
        ];
        assert_eq!(listing["data"].as_array().unwrap().len(), expected.len());
        for (name, value) in expected {
            assert!(
                listing["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|row| row["path"] == name),
                "{listing}"
            );
            let sampled = run(path, "value", &["--at", "0ns", "--signals", name]);
            assert_eq!(sampled["data"][0]["signals"][0]["path"], name);
            assert_eq!(sampled["data"][0]["signals"][0]["value"], value);
            let expr = format!(
                "$signal({}) == {value}",
                serde_json::to_string(name).unwrap()
            );
            let result = run(
                path,
                "property",
                &[
                    "--on",
                    "top.clk",
                    "--sample-mode",
                    "native",
                    "--eval",
                    &expr,
                    "--capture",
                    "match",
                    "--max",
                    "unlimited",
                ],
            );
            assert!(!result["data"].as_array().unwrap().is_empty(), "{result}");
        }
        let scoped = run(
            path,
            "value",
            &["--scope", r#"top."a.b""#, "--at", "0ns", "--signals", "c"],
        );
        assert_eq!(scoped["data"][0]["signals"][0]["path"], r#"top."a.b".c"#);
        assert_eq!(scoped["data"][0]["signals"][0]["relative_path"], "c");
        let scopes = run(path, "scope", &["--max-depth", "1", "--max", "unlimited"]);
        assert!(
            scopes["data"]
                .as_array()
                .unwrap()
                .iter()
                .any(|scope| scope["path"] == r#"top."a.b""# && scope["depth"] == 1)
        );
        let root = run(path, "signal", &["--scope", "", "--max", "unlimited"]);
        assert_eq!(root["data"][0]["path"], r#""root.name""#);
        let root_value = run(
            path,
            "value",
            &["--at", "0ns", "--signals", r#""root.name""#],
        );
        assert_eq!(root_value["data"][0]["signals"][0]["value"], "8'h33");
        let escaped = run(
            path,
            "value",
            &["--at", "0ns", "--scope", "", "--signals", "\\root.name "],
        );
        assert_eq!(escaped["data"][0]["signals"][0]["value"], "8'h33");
        let root_expr = run(
            path,
            "property",
            &[
                "--on",
                "top.clk",
                "--sample-mode",
                "native",
                "--eval",
                r#"$signal("\"root.name\"") == 8'h33"#,
                "--capture",
                "match",
            ],
        );
        assert!(!root_expr["data"].as_array().unwrap().is_empty());
    }
}

#[test]
fn quoted_components_preserve_commas_brackets_and_selection_order() {
    let fixture = Fixture::new(EXACT_VCD);
    for path in &fixture.paths {
        let result = run(
            path,
            "value",
            &[
                "--scope",
                "top",
                "--at",
                "0ns,0ns",
                "--signals",
                r#""literal[7:0]","comma,name[3:0]"[7:4],literal,"literal[7:0]""#,
            ],
        );
        assert_eq!(result["data"].as_array().unwrap().len(), 2);
        assert_eq!(
            result["data"][0]["signals"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row["value"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["8'ha5", "4'hc", "8'hf0", "8'ha5"]
        );
        let property = run(
            path,
            "property",
            &[
                "--scope",
                "top",
                "--on",
                r#"posedge clk iff top."a.b.c" == 8'h11"#,
                "--eval",
                r#"top."a.b".c == 8'h22 && top."literal[7:0]" == 8'ha5"#,
                "--capture",
                "match",
            ],
        );
        assert_eq!(property["data"].as_array().unwrap().len(), 1);
        let change = run(
            path,
            "change",
            &[
                "--on",
                r#"top."literal[7:0]""#,
                "--sample-mode",
                "native",
                "--signals",
                r#"top."literal[7:0]",top.literal"#,
                "--max",
                "unlimited",
            ],
        );
        assert!(!change["data"].as_array().unwrap().is_empty());
        let extracted = run(
            path,
            "extract",
            &[
                "generic",
                "--on",
                r#"posedge $signal("top.clk")"#,
                "--when",
                r#"top."a.b.c" == 8'h11"#,
                "--payload",
                r#"top."comma,name[3:0]"[7:4],top.literal"#,
            ],
        );
        assert_eq!(extracted["data"][0]["payload"][0]["value"], "4'hc");
    }
}

#[test]
fn source_ambiguity_is_format_neutral_and_does_not_choose_a_declaration() {
    let source = "$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! same $end\n$var wire 1 \" same $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n";
    let fixture = Fixture::new(source);
    for path in &fixture.paths {
        let listing = run(path, "signal", &["--scope", "top"]);
        assert!(listing["data"].as_array().unwrap().is_empty());
        assert_eq!(listing["diagnostics"][0]["code"], "WPK-W0005");
        let message = listing["diagnostics"][0]["message"].as_str().unwrap();
        assert!(message.contains("source declarations"));
        assert!(!message.contains("FSDB"));
        let output = wavepeek_cmd()
            .args(["value", "--waves"])
            .arg(path)
            .args(["--at", "0ns", "--signals", "top.same", "--json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let fatal: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(fatal.to_string().contains("ambiguous"));
    }
}

#[test]
fn includes_and_explicit_protocol_maps_resolve_quoted_scope_and_local_names() {
    let source = std::fs::read_to_string(common::fixture_path("extract_apb4.vcd"))
        .unwrap()
        .replace(
            "$scope module top $end",
            "$scope module top $end\n$scope module \\bus.port $end",
        )
        .replace("$enddefinitions", "$upscope $end\n$enddefinitions")
        .replace("uart_apb_p_clk_i", "\\uart_apb_p.clk_i");
    let fixture = Fixture::new(&source);
    for path in &fixture.paths {
        let scoped = run(
            path,
            "extract",
            &[
                "apb",
                "--scope",
                r#"top."bus.port""#,
                "--profile",
                "apb4",
                "--include",
                "^uart_apb_(presetn_i|psel_o|penable_o|pwrite_o|pready_i|p_addr_o|pprot_o|pwdata_o|pstrb_o|prdata_i|pslverr_i)$",
                "--map",
                r#"pclk="uart_apb_p.clk_i""#,
                "--max",
                "unlimited",
            ],
        );
        let included = run(
            path,
            "extract",
            &[
                "apb",
                "--profile",
                "apb4",
                "--include",
                r#"^top\."bus\.port"\.uart_apb_(presetn_i|psel_o|penable_o|pwrite_o|pready_i|p_addr_o|pprot_o|pwdata_o|pstrb_o|prdata_i|pslverr_i)$"#,
                "--map",
                r#"pclk=top."bus.port"."uart_apb_p.clk_i""#,
                "--max",
                "unlimited",
            ],
        );
        assert!(!scoped["data"].as_array().unwrap().is_empty());
        assert_eq!(included["data"], scoped["data"]);
    }
}

#[test]
fn malformed_explicit_path_operands_fail_before_opening_waveform_values() {
    let fixture = Fixture::new(EXACT_VCD);
    for expression in [
        r#"$signal(12)"#,
        r#"$signal("top.clk""#,
        r#"$signal("bad\q")"#,
    ] {
        let output = wavepeek_cmd()
            .args(["property", "--waves"])
            .arg(&fixture.paths[0])
            .args([
                "--on",
                "top.clk",
                "--sample-mode",
                "native",
                "--eval",
                expression,
                "--json",
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stdout).contains("EXPR-PARSE-PATH"));
    }
}

#[test]
fn literal_range_names_and_explicit_projections_remain_distinct_in_every_order() {
    let fixture = Fixture::new(
        "$timescale 1ns $end\n$scope module top $end\n$var wire 8 ! data $end\n$var wire 4 \" \\data[3:0] $end\n$var wire 1 # clk $end\n$upscope $end\n$enddefinitions $end\n#0\nb10100101 !\nb1111 \"\n0#\n#5\n1#\n#10\nb10100110 !\n0#\n",
    );
    for path in &fixture.paths {
        for (selections, expected) in [
            (r#"top."data[3:0]",top."data"[3:0]"#, ["4'hf", "4'h5"]),
            (r#"top."data"[3:0],top."data[3:0]""#, ["4'h5", "4'hf"]),
        ] {
            let values = run(path, "value", &["--at", "0ns,0ns", "--signals", selections]);
            for row in values["data"].as_array().unwrap() {
                assert_eq!(
                    row["signals"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|signal| signal["value"].as_str().unwrap())
                        .collect::<Vec<_>>(),
                    expected
                );
                for signal in row["signals"].as_array().unwrap() {
                    let reused = run(
                        path,
                        "value",
                        &["--at", "0ns", "--signals", signal["path"].as_str().unwrap()],
                    );
                    assert_eq!(reused["data"][0]["signals"][0]["value"], signal["value"]);
                }
            }
            let changes = run(
                path,
                "change",
                &[
                    "--on",
                    "*",
                    "--sample-mode",
                    "native",
                    "--signals",
                    selections,
                ],
            );
            let changed = if expected[0] == "4'h5" {
                ["4'h6", "4'hf"]
            } else {
                ["4'hf", "4'h6"]
            };
            assert_eq!(
                changes["data"][0]["signals"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|signal| signal["value"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                changed
            );
            let extracted = run(
                path,
                "extract",
                &[
                    "generic",
                    "--on",
                    "posedge top.clk",
                    "--when",
                    "1",
                    "--payload",
                    selections,
                ],
            );
            assert_eq!(
                extracted["data"][0]["payload"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|signal| signal["value"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }
}

#[test]
fn explicit_operands_preserve_reserved_names_through_selections_and_type_casts() {
    let fixture = Fixture::new(
        "$timescale 1ns $end\n$var wire 8 ! logic $end\n$var wire 8 \" bit $end\n$var wire 1 $ \\logic[0] $end\n$scope module top $end\n$var wire 1 # clk $end\n$upscope $end\n$enddefinitions $end\n#0\nb10100101 !\nb11110001 \"\n0$\n0#\n#5\n1#\n",
    );
    for path in &fixture.paths {
        let result = run(
            path,
            "property",
            &[
                "--on",
                "posedge top.clk",
                "--eval",
                r#"$signal("logic")[7:0] == 8'ha5 && $signal("logic")[0] && $signal("bit")[7:0] == 8'hf1 && $signal("bit")[0] && type($signal("logic"))'(8'ha5) == $signal("logic")"#,
                "--capture",
                "match",
            ],
        );
        assert_eq!(result["data"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn root_recursive_listing_keeps_scope_order_and_structural_depth() {
    let fixture = Fixture::new(
        "$timescale 1ns $end\n$var wire 1 ! root $end\n$scope module top $end\n$var wire 1 \" z $end\n$scope module a $end\n$var wire 1 # sig $end\n$upscope $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n1\"\n0#\n",
    );
    for path in &fixture.paths {
        let listing = run(
            path,
            "signal",
            &["--scope", "", "--recursive", "--max", "unlimited"],
        );
        assert_eq!(
            listing["data"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["path"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["root", "top.z", "top.a.sig"]
        );
        let limited = run(
            path,
            "signal",
            &["--scope", "", "--recursive", "--max", "2"],
        );
        assert_eq!(limited["data"].as_array().unwrap().len(), 2);
        assert_eq!(limited["data"][1]["path"], "top.z");
        let depth = run(
            path,
            "signal",
            &["--scope", "", "--recursive", "--max-depth", "1"],
        );
        assert_eq!(depth["data"].as_array().unwrap().len(), 2);
    }
}

#[test]
fn generated_protocol_expressions_preserve_quoted_handshake_reset_and_payload_names() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "apb",
            "extract_apb4.vcd",
            &[
                "--map",
                "pclk=uart_apb_p_clk_i",
                "--include",
                "^uart_apb_(presetn_i|psel_o|penable_o|pwrite_o|pready_i|p_addr_o|pprot_o|pwdata_o|pstrb_o|prdata_i|pslverr_i)$",
            ],
        ),
        ("atb", "extract_atb.vcd", &["--include", "^trace_"]),
        (
            "axi",
            "extract_axi_lite.vcd",
            &[
                "--profile",
                "axi4-lite",
                "--map",
                "aclk=clk",
                "--map",
                "aresetn=aresetn",
                "--include",
                "^axi_",
            ],
        ),
        (
            "axistream",
            "extract_axistream.vcd",
            &[
                "--map",
                "aclk=clk",
                "--map",
                "aresetn=rst_n",
                "--include",
                "^s_axis_",
            ],
        ),
        (
            "ahb",
            "extract_ahb_lite.vcd",
            &["--map", "hclk=clk", "--include", "^ahb_lite_"],
        ),
    ];
    for &(protocol, filename, options) in cases {
        let original = common::fixture_path(filename);
        let mut baseline_args = vec![protocol, "--scope", "top", "--max", "unlimited"];
        baseline_args.extend_from_slice(options);
        let baseline = run(&original, "extract", &baseline_args);
        assert!(
            !baseline["data"].as_array().unwrap().is_empty(),
            "{protocol}"
        );
        let mappings = baseline["context"]["mappings"].as_object().unwrap();
        let mut replacements = std::collections::HashMap::new();
        let mut mapped_args = vec![
            protocol.to_string(),
            "--max".to_string(),
            "unlimited".to_string(),
        ];
        if protocol == "axi" {
            mapped_args.extend(["--profile".to_string(), "axi4-lite".to_string()]);
        }
        for (standard, mapping) in mappings {
            let old = mapping["path"]
                .as_str()
                .unwrap()
                .strip_prefix("top.")
                .unwrap();
            let new = format!("{standard}.mapped");
            replacements.insert(old.to_string(), format!("\\{new}"));
            mapped_args.extend([
                "--map".to_string(),
                format!("{standard}={}", serde_json::Value::String(new)),
            ]);
        }
        let source = std::fs::read_to_string(&original)
            .unwrap()
            .lines()
            .map(|line| {
                if line.starts_with("$var ") {
                    let mut fields = line
                        .split_whitespace()
                        .map(str::to_string)
                        .collect::<Vec<_>>();
                    if let Some(new) = replacements.get(&fields[4]) {
                        fields[4] = new.clone();
                    }
                    fields.join(" ")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        for scoped in [true, false] {
            let source = if scoped {
                source.clone()
            } else {
                source
                    .replace("$scope module top $end\n", "")
                    .replace("$upscope $end\n", "")
            };
            let fixture = Fixture::new(&source);
            let mut args = mapped_args.clone();
            if scoped {
                args.extend(["--scope".to_string(), "top".to_string()]);
            }
            let args = args.iter().map(String::as_str).collect::<Vec<_>>();
            for path in &fixture.paths {
                let actual = run(path, "extract", &args);
                assert_eq!(
                    actual["data"], baseline["data"],
                    "{protocol}, scoped={scoped}"
                );
            }
        }
    }
}
