use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use wavepeek::contract::schema::{
    OUTPUT_SCHEMA_URL, STREAM_SCHEMA_URL, catalog_json, output_schema_json, stream_schema_json,
};

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        Command::Write { out_dir } => write_schemas(&out_dir),
        Command::Validate { schema_dir } => validate_schemas(&schema_dir),
    }
}

fn write_schemas(out_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(out_dir).map_err(|error| {
        format!(
            "failed to create output directory {}: {error}",
            out_dir.display()
        )
    })?;

    write_file(out_dir, "output.json", output_schema_json())?;
    write_file(out_dir, "stream.json", stream_schema_json())?;
    write_file(out_dir, "catalog.json", catalog_json())?;
    Ok(())
}

fn validate_schemas(schema_dir: &Path) -> Result<(), String> {
    let output = read_json(&schema_dir.join("output.json"))?;
    let stream = read_json(&schema_dir.join("stream.json"))?;
    let output_validator = jsonschema::validator_for(&output)
        .map_err(|error| format!("output schema does not compile: {error}"))?;
    let stream_validator = jsonschema::validator_for(&stream)
        .map_err(|error| format!("stream schema does not compile: {error}"))?;

    let valid_output = json!({
        "$schema": OUTPUT_SCHEMA_URL,
        "command": "info",
        "data": {"time_unit": "1ns", "time_start": "0ns", "time_end": "10ns"},
        "diagnostics": [{"kind": "warning", "code": "WPK-W0002", "message": "truncated"}],
        "x-extension": true,
    });
    output_validator
        .validate(&valid_output)
        .map_err(|error| format!("valid output sample failed validation: {error}"))?;
    let mut mismatch_output = valid_output.clone();
    mismatch_output["command"] = json!("scope");
    expect_invalid(
        &output_validator,
        &mismatch_output,
        "output command/data mismatch must reject",
    )?;
    let mut invalid_info_code = valid_output.clone();
    invalid_info_code["diagnostics"] =
        json!([{"kind": "info", "code": "WPK-W0001", "message": "bad"}]);
    expect_invalid(
        &output_validator,
        &invalid_info_code,
        "info diagnostics with codes must reject",
    )?;
    let mut invalid_matched_tokens = valid_output;
    invalid_matched_tokens["command"] = json!("docs search");
    invalid_matched_tokens["data"] = json!({
        "query": "docs",
        "matches": [{
            "topic": {"id": "intro", "title": "Intro", "description": "Start", "section": "intro"},
            "match_kind": "body",
            "matched_tokens": 0,
        }],
    });
    expect_invalid(
        &output_validator,
        &invalid_matched_tokens,
        "docs search matched_tokens below one must reject",
    )?;

    for record in [
        json!({"type": "begin", "seq": 0, "command": "change", "$schema": STREAM_SCHEMA_URL, "x-extension": true}),
        json!({"type": "item", "seq": 1, "command": "change", "item": {"time": "10ns", "sample_time": "9ns", "signals": []}}),
        json!({"type": "diagnostic", "seq": 2, "command": "change", "diagnostic": {"kind": "warning", "code": "WPK-W0002", "message": "truncated"}}),
        json!({"type": "end", "seq": 3, "command": "change", "summary": {"status": "ok", "items": 1, "diagnostics": 1, "truncated": false}}),
    ] {
        stream_validator
            .validate(&record)
            .map_err(|error| format!("valid stream sample failed validation: {error}"))?;
    }
    let mismatch_stream =
        json!({"type": "item", "seq": 1, "command": "info", "item": {"time": "10ns", "sample_time": "9ns", "signals": []}});
    expect_invalid(
        &stream_validator,
        &mismatch_stream,
        "stream command/item mismatch must reject",
    )?;
    Ok(())
}

fn expect_invalid(
    validator: &jsonschema::Validator,
    value: &Value,
    message: &'static str,
) -> Result<(), String> {
    if validator.is_valid(value) {
        Err(message.to_string())
    } else {
        Ok(())
    }
}

enum Command {
    Write { out_dir: PathBuf },
    Validate { schema_dir: PathBuf },
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Command, String> {
    let mut out_dir = None;
    let mut validate_dir = None;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--out requires a directory argument".to_string())?;
                if out_dir.replace(PathBuf::from(value)).is_some() {
                    return Err("--out may be provided only once".to_string());
                }
            }
            "--validate" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--validate requires a directory argument".to_string())?;
                if validate_dir.replace(PathBuf::from(value)).is_some() {
                    return Err("--validate may be provided only once".to_string());
                }
            }
            "-h" | "--help" => {
                println!("Usage: wavepeek-schema-gen --out <directory>");
                println!("       wavepeek-schema-gen --validate <schema-directory>");
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument {arg:?}")),
        }
    }

    match (out_dir, validate_dir) {
        (Some(out_dir), None) => Ok(Command::Write { out_dir }),
        (None, Some(schema_dir)) => Ok(Command::Validate { schema_dir }),
        (Some(_), Some(_)) => Err("--out and --validate are mutually exclusive".to_string()),
        (None, None) => Err("missing required --out <directory> or --validate <schema-directory>".to_string()),
    }
}

fn read_json(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&contents).map_err(|error| format!("invalid JSON in {}: {error}", path.display()))
}

fn write_file(out_dir: &Path, name: &str, contents: String) -> Result<(), String> {
    let path = out_dir.join(name);
    fs::write(&path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}
