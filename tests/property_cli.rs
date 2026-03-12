use assert_cmd::prelude::*;
use predicates::prelude::*;

mod common;
use common::{fixture_path, wavepeek_cmd};

#[test]
fn property_accepts_capture_flag_but_is_unimplemented() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    for capture in ["match", "switch", "assert", "deassert"] {
        wavepeek_cmd()
            .args([
                "property",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--on",
                "posedge clk",
                "--eval",
                "1",
                "--capture",
                capture,
            ])
            .assert()
            .failure()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::starts_with("error: unimplemented:"))
            .stderr(predicate::str::contains("error: args:").not())
            .stderr(predicate::str::contains(
                "`property` command execution is not implemented yet",
            ));
    }
}

#[test]
fn property_defaults_capture_to_switch() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let default_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "1",
        ])
        .output()
        .expect("property with default capture should execute");
    let switch_output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--eval",
            "1",
            "--capture",
            "switch",
        ])
        .output()
        .expect("property with explicit switch capture should execute");

    assert!(!default_output.status.success());
    assert!(!switch_output.status.success());
    assert!(default_output.stdout.is_empty());
    assert!(switch_output.stdout.is_empty());
    assert_eq!(default_output.stderr, switch_output.stderr);
    assert!(
        String::from_utf8_lossy(&default_output.stderr)
            .contains("error: unimplemented: `property` command execution is not implemented yet")
    );
}

#[test]
fn property_rejects_legacy_when_surface_flags() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--clk",
            "top.clk",
            "--eval",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--clk'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.clk",
            "--cond",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--cond'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn property_requires_eval_flag() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args(["property", "--waves", fixture.as_str(), "--on", "*"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--eval <EVAL>"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn property_rejects_legacy_when_flag_name() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--when",
            "posedge top.clk",
            "--eval",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: args:"))
        .stderr(predicate::str::contains("unexpected argument '--when'"))
        .stderr(predicate::str::contains("See 'wavepeek property --help'."));
}

#[test]
fn property_invalid_on_expression_still_fails_as_unimplemented() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge (",
            "--eval",
            "1",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: unimplemented:"))
        .stderr(predicate::str::contains("error: args:").not())
        .stderr(predicate::str::contains(
            "`property` command execution is not implemented yet",
        ));
}

#[test]
fn property_rich_c4_surface_stays_unimplemented() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.clk iff top.ev.triggered",
            "--eval",
            "type(top.data)'(3) == 8'h03 ? real'(1) > 0.5 : \"go\" == \"go\"",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("error: unimplemented:"))
        .stderr(predicate::str::contains("error: args:").not())
        .stderr(predicate::str::contains(
            "`property` command execution is not implemented yet",
        ));
}
