use std::path::{Path, PathBuf};

use serde_json::Value;

mod common;
use common::wavepeek_cmd;

fn query(input: &Path, args: &[&str], status: i32) -> Value {
    let output = wavepeek_cmd()
        .args(args)
        .args(["--json", "--waves"])
        .arg(input)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(status), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn systemc_digital_history_uses_the_selected_shm_directory() {
    let input = PathBuf::from(std::env::var_os("ONDAS_FIXTURES_DIR").expect("run inside ./dev"))
        .join("vwdb/vwdb0085-systemc-lwtr4tlm2/waveform.vwdb");
    let outer = query(&input, &["info"], 0);
    assert_eq!(outer["data"][0]["time_end"], "290000ps");
    let sample = ["value", "--signals", "sc_main.tb.clk", "--at", "5ns"];
    let diagnostic = query(&input, &sample, 1);
    assert_eq!(diagnostic["code"], "WPK-F0004");

    let digital = input.join("waves_ida.shm");
    let inner = query(&digital, &["info"], 0);
    assert_eq!(inner["data"][0]["time_end"], "290000ps");
    let point = query(&digital, &sample, 0);
    assert_eq!(point["data"][0]["signals"][0]["value"], "1'h0");
}
