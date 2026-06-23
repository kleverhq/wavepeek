#!/usr/bin/env python3

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import math
import pathlib
import re
import shlex
import shutil
import subprocess
import sys
from collections.abc import Mapping, Sequence
from typing import Any, NamedTuple, NoReturn


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]
TESTS_PATH = SCRIPT_DIR / "tests.json"
DEFAULT_RUNS_DIR = SCRIPT_DIR / "runs"
README_NAME = "README.md"
BINARY_LABEL_RE = re.compile(r"^[A-Za-z0-9_.-]+$")
EMOJI_THRESHOLD_PCT = 3.0
METRICS = ("mean", "stddev", "median", "min", "max")
COMPARE_TIMING_METRIC = "median"
HYPERFINE_SUFFIX = ".hyperfine.json"
WAVEPEEK_SUFFIX = ".wavepeek.json"
FAILURE_SUFFIX = ".failure.json"
FAILURE_KIND = "wavepeek-e2e-bench-failure"
FAILURE_SUMMARY_LIMIT = 4096
FUNCTIONAL_MATCH_MARKER = "✅"
FUNCTIONAL_MISMATCH_MARKER = "⚠️"
FUNCTIONAL_MISSING_MARKER = "?"
FUNCTIONAL_TIMEOUT_MARKER = "⏱T"
DEFAULT_WAVEPEEK_TIMEOUT_SECONDS = 300
DIAGNOSTIC_CODE_RE = re.compile(r"^WPK-[WE][0-9]{4}$")


class BinarySpec(NamedTuple):
    label: str
    path: str


class RunOutcome(NamedTuple):
    status: str
    test_name: str
    failure: dict[str, Any] | None
    errors: list[str]


def fail(message: str) -> NoReturn:
    raise SystemExit(message)


def as_int(value: Any, field: str) -> int:
    try:
        return int(value)
    except (TypeError, ValueError):
        fail(f"error: tests: `{field}` must be integer")


def require_nonempty_str(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"error: tests: `{field}` must be non-empty string")
    return value


def require_object(value: Any, field: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"error: tests: `{field}` must be object")
    return dict(value)


def require_nonempty_list(value: Any, field: str) -> list[Any]:
    if not isinstance(value, list) or not value:
        fail(f"error: tests: `{field}` must be a non-empty list")
    return list(value)


def normalize_path(path_value: str) -> pathlib.Path:
    return pathlib.Path(path_value).expanduser().resolve()


def ensure_existing_dir(path: pathlib.Path, label: str) -> None:
    if not path.exists() or not path.is_dir():
        fail(f"error: {label}: directory does not exist: {path}")


def load_tests(tests_path: pathlib.Path) -> list[dict[str, Any]]:
    if not tests_path.exists():
        fail(f"error: tests: missing file {tests_path}")

    try:
        payload = json.loads(tests_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: tests: invalid JSON in {tests_path}: {error}")

    if not isinstance(payload, dict):
        fail(f"error: tests: root of {tests_path} must be object")

    tests = require_nonempty_list(payload.get("tests"), "tests")

    validated: list[dict[str, Any]] = []
    seen: set[str] = set()

    for index, raw in enumerate(tests):
        if not isinstance(raw, dict):
            fail(f"error: tests: `tests[{index}]` must be object")

        name = require_nonempty_str(raw.get("name"), f"tests[{index}].name")
        if name in seen:
            fail(f"error: tests: duplicate test name `{name}`")
        category = require_nonempty_str(raw.get("category"), f"tests[{index}].category")
        runs = raw.get("runs")
        warmup = raw.get("warmup")
        command = require_nonempty_list(raw.get("command"), f"tests[{index}].command")
        meta = require_object(raw.get("meta", {}), f"tests[{index}].meta")

        runs_value = as_int(runs, f"tests[{index}].runs")
        warmup_value = as_int(warmup, f"tests[{index}].warmup")
        if runs_value < 1:
            fail(f"error: tests: `{name}` has runs < 1")
        if warmup_value < 0:
            fail(f"error: tests: `{name}` has warmup < 0")

        command_tokens: list[str] = []
        for token_index, token in enumerate(command):
            if not isinstance(token, str) or not token:
                fail(
                    "error: tests: "
                    f"`tests[{index}].command[{token_index}]` must be non-empty string"
                )
            command_tokens.append(token)

        validated.append(
            {
                "name": name,
                "category": category,
                "runs": runs_value,
                "warmup": warmup_value,
                "command": command_tokens,
                "meta": meta,
            }
        )
        seen.add(name)

    return sorted(validated, key=lambda test: str(test["name"]))


def select_tests(tests: list[dict[str, Any]], pattern: str | None) -> list[dict[str, Any]]:
    if pattern is None:
        return list(tests)
    try:
        regex = re.compile(pattern)
    except re.error as error:
        fail(f"error: filter: invalid regex {pattern!r}: {error}")
    return [test for test in tests if regex.search(str(test["name"]))]


def resolve_run_dir(run_dir_arg: str | None, out_dir_arg: str) -> pathlib.Path:
    if run_dir_arg:
        run_dir = normalize_path(run_dir_arg)
        run_dir.mkdir(parents=True, exist_ok=True)
        return run_dir

    out_dir = normalize_path(out_dir_arg)
    out_dir.mkdir(parents=True, exist_ok=True)

    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d_%H-%M-%SZ")
    for attempt in range(1000):
        suffix = "" if attempt == 0 else f"-{attempt}"
        candidate = out_dir / f"{stamp}{suffix}"
        try:
            candidate.mkdir(parents=True, exist_ok=False)
            return candidate
        except FileExistsError:
            continue

    fail(f"error: run: failed to create run directory in {out_dir}")
    raise AssertionError("unreachable")


def parse_binary_specs(values: list[str] | None) -> list[BinarySpec]:
    if not values:
        fail("error: run: at least one --binary label=path argument is required")

    specs: list[BinarySpec] = []
    seen: set[str] = set()
    for raw in values:
        if "=" not in raw:
            fail(f"error: run: --binary must use label=path form: {raw}")
        label, path_text = raw.split("=", 1)
        if not label or not BINARY_LABEL_RE.fullmatch(label):
            fail(
                "error: run: --binary label must contain only letters, digits, dot, dash, or underscore"
            )
        if label in {".", "..", README_NAME, "manifest.json"}:
            fail(f"error: run: --binary label `{label}` is reserved")
        if label in seen:
            fail(f"error: run: duplicate --binary label `{label}`")
        path = normalize_path(path_text)
        if not path.exists() or not path.is_file():
            fail(f"error: run: --binary `{label}` points to missing file: {path}")
        specs.append(BinarySpec(label=label, path=str(path)))
        seen.add(label)
    return specs


def ensure_hyperfine() -> None:
    if shutil.which("hyperfine") is None:
        fail("error: run: `hyperfine` is not available in PATH")


def resolve_test_command(test: dict[str, Any], wavepeek_bin: str) -> list[str]:
    command_tokens: list[str] = []
    for token in test["command"]:
        try:
            command_tokens.append(str(token).format(wavepeek_bin=wavepeek_bin))
        except KeyError as error:
            fail(f"error: tests: missing placeholder {error!s} in test `{test['name']}`")
    return command_tokens


def build_functional_command(command_args: list[str]) -> list[str]:
    if "--json" in command_args:
        return list(command_args)
    return [*command_args, "--json"]


def build_timed_benchmark_command(
    command_args: list[str],
    timeout_seconds: int,
    test_name: str,
) -> str:
    wrapper = SCRIPT_DIR / "wavepeek_timeout_wrapper.py"
    wrapper_args = [
        sys.executable,
        str(wrapper),
        "--timeout-seconds",
        str(timeout_seconds),
        "--label",
        test_name,
        "--",
        *command_args,
    ]
    return shlex.join(wrapper_args)


def is_timeout_functional_payload(payload: Any) -> bool:
    return isinstance(payload, dict) and len(payload) == 0


def artifact_test_name(path: pathlib.Path, suffix: str) -> str:
    if not path.name.endswith(suffix):
        fail(f"error: report: artifact has unexpected suffix in {path}")
    test_name = path.name[: -len(suffix)]
    if not test_name:
        fail(f"error: report: artifact has empty test name: {path}")
    return test_name


def hyperfine_result_path(run_dir: pathlib.Path, test_name: str) -> pathlib.Path:
    return run_dir / f"{test_name}{HYPERFINE_SUFFIX}"


def wavepeek_result_path(run_dir: pathlib.Path, test_name: str) -> pathlib.Path:
    return run_dir / f"{test_name}{WAVEPEEK_SUFFIX}"


def failure_result_path(run_dir: pathlib.Path, test_name: str) -> pathlib.Path:
    return run_dir / f"{test_name}{FAILURE_SUFFIX}"


def test_has_complete_artifacts(run_dir: pathlib.Path, test_name: str) -> bool:
    return hyperfine_result_path(run_dir, test_name).is_file() and wavepeek_result_path(
        run_dir, test_name
    ).is_file()


def test_has_terminal_artifact(run_dir: pathlib.Path, test_name: str) -> bool:
    return test_has_complete_artifacts(run_dir, test_name) or failure_result_path(
        run_dir,
        test_name,
    ).is_file()


def remove_test_artifacts(run_dir: pathlib.Path, test_name: str) -> None:
    for path in (
        hyperfine_result_path(run_dir, test_name),
        wavepeek_result_path(run_dir, test_name),
        failure_result_path(run_dir, test_name),
    ):
        try:
            path.unlink()
        except FileNotFoundError:
            pass


def binary_run_dir(run_dir: pathlib.Path, binary: BinarySpec) -> pathlib.Path:
    return run_dir / binary.label


def write_run_manifest(
    *,
    run_dir: pathlib.Path,
    tests_path: pathlib.Path,
    binaries: list[BinarySpec],
    selected_tests: list[dict[str, Any]],
    schedule: str,
    timeout_seconds: int,
) -> None:
    binary_entries: list[dict[str, Any]] = []
    for binary in binaries:
        label_dir = binary_run_dir(run_dir, binary)
        hyperfine_count = len(list(label_dir.glob(f"*{HYPERFINE_SUFFIX}")))
        wavepeek_count = len(list(label_dir.glob(f"*{WAVEPEEK_SUFFIX}")))
        failure_results = load_failure_results(label_dir) if label_dir.is_dir() else {}
        failures_by_phase: dict[str, int] = {}
        for failure in failure_results.values():
            phase = str(failure.get("phase", "unknown"))
            failures_by_phase[phase] = failures_by_phase.get(phase, 0) + 1
        binary_entries.append(
            {
                "label": binary.label,
                "path": binary.path,
                "run_dir": binary.label,
                "hyperfine_json_count": hyperfine_count,
                "wavepeek_json_count": wavepeek_count,
                "failure_count": len(failure_results),
                "failures_by_phase": failures_by_phase,
            }
        )

    payload = {
        "kind": "wavepeek-e2e-bench-run",
        "schema_version": 1,
        "generated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "tests_path": str(tests_path),
        "schedule": schedule,
        "timeout_seconds": timeout_seconds,
        "test_count": len(selected_tests),
        "binaries": binary_entries,
    }
    (run_dir / "manifest.json").write_text(
        json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def write_run_index(run_dir: pathlib.Path, binaries: list[BinarySpec]) -> pathlib.Path:
    lines = [f"# CLI E2E Bench Run: {run_dir.name}", "", "## Binaries", ""]
    for binary in binaries:
        lines.append(f"- `{binary.label}`: `{binary.path}` (`{binary.label}/`)" )
    lines.append("")
    index_path = run_dir / README_NAME
    index_path.write_text("\n".join(lines), encoding="utf-8")
    return index_path


def partition_missing_only_tests(
    selected: list[dict[str, Any]],
    run_dir: pathlib.Path,
) -> tuple[list[dict[str, Any]], list[str]]:
    runnable: list[dict[str, Any]] = []
    skipped: list[str] = []
    for test in selected:
        test_name = str(test["name"])
        if test_has_terminal_artifact(run_dir, test_name):
            skipped.append(test_name)
        else:
            runnable.append(test)
    return runnable, skipped


def summarize_process_text(value: Any) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        text = value.decode("utf-8", errors="replace")
    else:
        text = str(value)
    if len(text) <= FAILURE_SUMMARY_LIMIT:
        return text
    return text[:FAILURE_SUMMARY_LIMIT] + "\n... truncated ..."


def make_failure_artifact(
    *,
    test_name: str,
    phase: str,
    command: Sequence[str],
    exit_code: int | None,
    timed_out: bool = False,
    stdout: Any = None,
    stderr: Any = None,
    message: str | None = None,
    binary_label: str | None = None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "kind": FAILURE_KIND,
        "schema_version": 1,
        "generated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "test_name": test_name,
        "phase": phase,
        "command": [str(arg) for arg in command],
        "exit_code": exit_code,
        "timed_out": timed_out,
        "stdout_summary": summarize_process_text(stdout),
        "stderr_summary": summarize_process_text(stderr),
    }
    if message:
        payload["message"] = message
    if binary_label:
        payload["binary_label"] = binary_label
    return payload


def write_failure_artifact(run_dir: pathlib.Path, test_name: str, payload: dict[str, Any]) -> None:
    output_path = failure_result_path(run_dir, test_name)
    output_path.write_text(
        json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def parse_failure_result_file(path: pathlib.Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: report: invalid JSON in {path}: {error}")

    if not isinstance(payload, dict):
        fail(f"error: report: expected object in {path}")
    if payload.get("kind") != FAILURE_KIND:
        fail(f"error: report: unexpected failure artifact kind in {path}")
    artifact_name = artifact_test_name(path, FAILURE_SUFFIX)
    test_name = payload.get("test_name")
    if test_name != artifact_name:
        fail(f"error: report: failure artifact test name mismatch in {path}")
    if payload.get("phase") not in {"preflight", "benchmark"}:
        fail(f"error: report: invalid failure artifact phase in {path}")
    if not isinstance(payload.get("command"), list):
        fail(f"error: report: failure artifact command must be list in {path}")
    return dict(payload)


def load_failure_results(run_dir: pathlib.Path) -> dict[str, dict[str, Any]]:
    result_map: dict[str, dict[str, Any]] = {}
    for path in sorted(run_dir.glob(f"*{FAILURE_SUFFIX}")):
        parsed = parse_failure_result_file(path)
        result_map[str(parsed["test_name"])] = parsed
    return result_map


def run_test(
    test: dict[str, Any],
    run_dir: pathlib.Path,
    wavepeek_bin: str,
    timeout_seconds: int,
    verbose: bool,
) -> dict[str, Any] | None:
    command_args = resolve_test_command(test, wavepeek_bin)
    benchmark_command = build_timed_benchmark_command(
        command_args,
        timeout_seconds,
        str(test["name"]),
    )
    output_path = hyperfine_result_path(run_dir, str(test["name"]))

    hyperfine_cmd = [
        "hyperfine",
        "-N",
        "--style",
        "basic",
        "--warmup",
        str(test["warmup"]),
        "--runs",
        str(test["runs"]),
        "--command-name",
        str(test["name"]),
        "--export-json",
        str(output_path),
        benchmark_command,
    ]
    if verbose:
        result = subprocess.run(hyperfine_cmd, check=False, cwd=REPO_ROOT)
    else:
        result = subprocess.run(
            hyperfine_cmd,
            check=False,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
        )
    if result.returncode != 0:
        try:
            output_path.unlink()
        except FileNotFoundError:
            pass
        message = f"hyperfine failed for `{test['name']}`"
        return make_failure_artifact(
            test_name=str(test["name"]),
            phase="benchmark",
            command=hyperfine_cmd,
            exit_code=result.returncode,
            stdout=getattr(result, "stdout", None),
            stderr=getattr(result, "stderr", None),
            message=message,
        )
    return None


def validate_functional_diagnostic(diagnostic: Any, source: str, index: int) -> None:
    label = f"{source} field `diagnostics[{index}]`"
    if not isinstance(diagnostic, dict):
        raise ValueError(f"{label} must be object")
    extra = sorted(set(diagnostic) - {"kind", "code", "message"})
    if extra:
        raise ValueError(f"{label} has unexpected key `{extra[0]}`")
    kind = diagnostic.get("kind")
    message = diagnostic.get("message")
    code = diagnostic.get("code")
    if kind not in {"info", "warning", "error"}:
        raise ValueError(f"{label} field `kind` must be info, warning, or error")
    if not isinstance(message, str):
        raise ValueError(f"{label} field `message` must be string")
    if kind == "info":
        if "code" in diagnostic:
            raise ValueError(f"{label} field `code` must be omitted for info")
        return
    if not isinstance(code, str):
        raise ValueError(f"{label} field `code` must be string")
    if not DIAGNOSTIC_CODE_RE.fullmatch(code):
        raise ValueError(f"{label} field `code` must match WPK-[WE]####")
    expected_prefix = "WPK-W" if kind == "warning" else "WPK-E"
    if not code.startswith(expected_prefix):
        raise ValueError(f"{label} field `code` must use {expected_prefix} for {kind}")


def validate_functional_payload(payload: Any, source: str) -> dict[str, Any]:
    if not isinstance(payload, dict):
        raise ValueError(f"{source} must be object")
    if "warnings" in payload:
        raise ValueError(f"{source} must not contain legacy key `warnings`")
    if "data" not in payload:
        raise ValueError(f"{source} missing key `data`")
    if "diagnostics" not in payload:
        raise ValueError(f"{source} missing key `diagnostics`")
    if not isinstance(payload["data"], dict) and not isinstance(payload["data"], list):
        raise ValueError(f"{source} field `data` must be object or list")
    if not isinstance(payload["diagnostics"], list):
        raise ValueError(f"{source} field `diagnostics` must be list")
    for index, diagnostic in enumerate(payload["diagnostics"]):
        validate_functional_diagnostic(diagnostic, source, index)
    return {"data": payload["data"], "diagnostics": payload["diagnostics"]}


def run_functional_capture(
    test: dict[str, Any],
    wavepeek_bin: str,
    caller: str,
    timeout_seconds: int,
    *,
    fail_on_error: bool = True,
) -> dict[str, Any] | tuple[dict[str, Any] | None, dict[str, Any] | None]:
    test_name = str(test["name"])
    command_args = build_functional_command(resolve_test_command(test, wavepeek_bin))
    try:
        result = subprocess.run(
            command_args,
            check=False,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        failure = make_failure_artifact(
            test_name=test_name,
            phase="preflight",
            command=command_args,
            exit_code=None,
            timed_out=True,
            stdout=getattr(error, "stdout", None),
            stderr=getattr(error, "stderr", None),
            message=f"functional capture timed out after {timeout_seconds}s",
        )
        if fail_on_error:
            raise
        return None, failure

    if result.returncode != 0:
        details = result.stderr.strip()
        suffix = f": {details}" if details else ""
        message = (
            f"functional capture failed for `{test_name}` "
            f"(exit {result.returncode}){suffix}"
        )
        failure = make_failure_artifact(
            test_name=test_name,
            phase="preflight",
            command=command_args,
            exit_code=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr,
            message=message,
        )
        if fail_on_error:
            fail(f"error: {caller}: {message}")
        return None, failure

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        message = f"invalid JSON output for `{test_name}`: {error}"
        failure = make_failure_artifact(
            test_name=test_name,
            phase="preflight",
            command=command_args,
            exit_code=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr,
            message=message,
        )
        if fail_on_error:
            fail(f"error: {caller}: {message}")
        return None, failure

    try:
        validated = validate_functional_payload(payload, f"functional output for `{test_name}`")
    except ValueError as error:
        failure = make_failure_artifact(
            test_name=test_name,
            phase="preflight",
            command=command_args,
            exit_code=result.returncode,
            stdout=result.stdout,
            stderr=result.stderr,
            message=str(error),
        )
        if fail_on_error:
            fail(f"error: {caller}: {error}")
        return None, failure

    if fail_on_error:
        return validated
    return validated, None


def write_wavepeek_artifact(run_dir: pathlib.Path, test_name: str, payload: dict[str, Any]) -> None:
    output_path = wavepeek_result_path(run_dir, test_name)
    output_path.write_text(
        json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def write_wavepeek_timeout_artifact(run_dir: pathlib.Path, test_name: str) -> None:
    output_path = wavepeek_result_path(run_dir, test_name)
    output_path.write_text("{}\n", encoding="utf-8")


def parse_hyperfine_result_file(path: pathlib.Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: report: invalid JSON in {path}: {error}")

    if not isinstance(payload, dict):
        fail(f"error: report: expected object in {path}")

    results = payload.get("results")
    if not isinstance(results, list) or not results or not isinstance(results[0], dict):
        fail(f"error: report: missing `results[0]` object in {path}")
    first = dict(results[0])

    row: dict[str, Any] = {
        "test_name": artifact_test_name(path, HYPERFINE_SUFFIX),
        "command": str(first.get("command", "")),
    }
    for metric in METRICS:
        raw = first.get(metric)
        if raw is None and metric == "stddev":
            row[metric] = 0.0
            continue
        if raw is None:
            fail(f"error: report: missing metric `{metric}` in {path}")
        try:
            row[metric] = float(raw)
        except (TypeError, ValueError):
            fail(f"error: report: invalid metric `{metric}` in {path}")
    return row


def parse_wavepeek_result_file(path: pathlib.Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: report: invalid JSON in {path}: {error}")

    if is_timeout_functional_payload(payload):
        return {}

    try:
        return validate_functional_payload(payload, f"functional artifact `{path}`")
    except ValueError as error:
        fail(f"error: report: {error}")
    raise AssertionError("unreachable")


def load_hyperfine_results(
    run_dir: pathlib.Path,
    test_names: Iterable[str] | None = None,
) -> dict[str, dict[str, Any]]:
    result_map: dict[str, dict[str, Any]] = {}
    if test_names is None:
        paths = sorted(run_dir.glob(f"*{HYPERFINE_SUFFIX}"))
    else:
        paths = [hyperfine_result_path(run_dir, name) for name in sorted(test_names)]
    for path in paths:
        parsed = parse_hyperfine_result_file(path)
        result_map[str(parsed["test_name"])] = parsed
    return result_map


def load_wavepeek_results(run_dir: pathlib.Path) -> dict[str, dict[str, Any]]:
    result_map: dict[str, dict[str, Any]] = {}
    for path in sorted(run_dir.glob(f"*{WAVEPEEK_SUFFIX}")):
        test_name = artifact_test_name(path, WAVEPEEK_SUFFIX)
        result_map[test_name] = parse_wavepeek_result_file(path)
    return result_map


def load_wavepeek_artifact_for_compare(
    run_dir: pathlib.Path,
    test_name: str,
    label: str,
) -> tuple[dict[str, Any] | None, str | None]:
    path = wavepeek_result_path(run_dir, test_name)
    if not path.exists() or not path.is_file():
        return None, f"{label}: missing artifact `{path}`"

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        return None, f"{label}: invalid JSON in `{path}`: {error}"
    except OSError as error:
        return None, f"{label}: failed to read `{path}`: {error}"

    if is_timeout_functional_payload(payload):
        return {}, None

    try:
        normalized = validate_functional_payload(payload, f"{label} artifact `{path}`")
    except ValueError as error:
        return None, str(error)
    return normalized, None


def delta_pct(revised: float, golden: float) -> float | None:
    if golden == 0:
        return None
    return ((golden - revised) / golden) * 100.0


def allowed_slowdown(golden_time: float, threshold_pct: float, threshold_seconds: float) -> float:
    return max(golden_time * threshold_pct / 100.0, threshold_seconds)


def timing_record(
    *,
    test_name: str,
    metric: str,
    revised_time: float,
    golden_time: float,
    threshold_pct: float,
    threshold_seconds: float,
) -> dict[str, Any]:
    actual_slowdown = revised_time - golden_time
    return {
        "test_name": test_name,
        "metric": metric,
        "revised_seconds": revised_time,
        "golden_seconds": golden_time,
        "delta_pct": delta_pct(revised_time, golden_time),
        "slowdown_seconds": actual_slowdown,
        "allowed_slowdown_seconds": allowed_slowdown(
            golden_time,
            threshold_pct,
            threshold_seconds,
        ),
        "speed": format_speed_factor(revised_time, golden_time),
    }


def format_timing_record(record: dict[str, Any]) -> str:
    delta = record.get("delta_pct")
    delta_text = "n/a" if delta is None else f"{float(delta):+.2f}%"
    return (
        f"{record['test_name']}: {record['metric']} "
        f"revised={float(record['revised_seconds']):.6f}s, "
        f"golden={float(record['golden_seconds']):.6f}s, "
        f"delta={delta_text}, "
        f"slowdown={float(record['slowdown_seconds']):.6f}s, "
        f"allowed={float(record['allowed_slowdown_seconds']):.6f}s, "
        f"speed={record['speed']}"
    )


def write_result_json(path_arg: str | None, payload: dict[str, Any]) -> None:
    if not path_arg:
        return
    path = normalize_path(path_arg)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def parse_ignored_functional_tests(values: Sequence[str] | None) -> dict[str, str]:
    ignored: dict[str, str] = {}
    for raw in values or []:
        if "=" not in raw:
            fail(
                "error: compare: --ignore-functional-test must use "
                "NAME=REASON"
            )
        name, reason = raw.split("=", 1)
        name = name.strip()
        reason = reason.strip()
        if not name or not reason:
            fail(
                "error: compare: --ignore-functional-test requires non-empty "
                "NAME and REASON"
            )
        if name in ignored:
            fail(f"error: compare: duplicate ignored functional test `{name}`")
        ignored[name] = reason
    return ignored


def ignored_functional_test_records(
    ignored: Mapping[str, str],
    revised_names: set[str],
    golden_names: set[str],
) -> list[dict[str, Any]]:
    return [
        {
            "test_name": name,
            "reason": ignored[name],
            "present_in_revised": name in revised_names,
            "present_in_golden": name in golden_names,
        }
        for name in sorted(ignored)
    ]


def parse_hyperfine_times_file(path: pathlib.Path, caller: str) -> list[float]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"error: {caller}: missing hyperfine JSON file: {path}")
    except json.JSONDecodeError as error:
        fail(f"error: {caller}: invalid JSON in {path}: {error}")

    if not isinstance(payload, dict):
        fail(f"error: {caller}: expected object in {path}")
    results = payload.get("results")
    if not isinstance(results, list) or not results or not isinstance(results[0], dict):
        fail(f"error: {caller}: missing `results[0]` object in {path}")
    raw_times = results[0].get("times")
    if not isinstance(raw_times, list) or not raw_times:
        fail(f"error: {caller}: missing non-empty `results[0].times` array in {path}")

    times: list[float] = []
    for index, raw in enumerate(raw_times):
        try:
            value = float(raw)
        except (TypeError, ValueError):
            fail(f"error: {caller}: invalid time at `results[0].times[{index}]` in {path}")
        if not math.isfinite(value) or value < 0:
            fail(f"error: {caller}: invalid time at `results[0].times[{index}]` in {path}")
        times.append(value)
    return times


def hyperfine_sample_times(run_dir: pathlib.Path, test_name: str, caller: str) -> list[float]:
    return parse_hyperfine_times_file(hyperfine_result_path(run_dir, test_name), caller)


def speed_factor(revised: float, golden: float) -> tuple[float, str]:
    if revised == golden:
        return 1.0, "same"
    if revised < golden:
        if revised == 0:
            return float("inf"), "faster"
        return golden / revised, "faster"
    if golden == 0:
        return float("inf"), "slower"
    return revised / golden, "slower"


def format_speed_factor(revised: float, golden: float) -> str:
    factor, direction = speed_factor(revised, golden)
    factor_text = "infx" if math.isinf(factor) else f"{factor:.2f}x"
    if direction == "same":
        return factor_text
    return f"{factor_text} {direction}"


def format_metric(value: float, baseline: dict[str, Any] | None, metric: str) -> str:
    base = f"{value:.6f}"
    if baseline is None:
        return base

    baseline_value = float(baseline[metric])
    delta = delta_pct(value, baseline_value)
    speed = format_speed_factor(value, baseline_value)
    if delta is None:
        return f"{base} (n/a, {speed})"

    emoji = ""
    if delta >= EMOJI_THRESHOLD_PCT:
        emoji = " 🟢"
    elif delta <= -EMOJI_THRESHOLD_PCT:
        emoji = " 🔴"
    return f"{base} ({delta:+.2f}%, {speed}){emoji}"


def escape_md(value: Any) -> str:
    return str(value).replace("\n", " ").replace("|", "\\|")


def format_meta(meta: dict[str, Any]) -> str:
    if not meta:
        return "-"
    pairs = []
    for key, value in meta.items():
        if isinstance(value, (dict, list)):
            value_text = json.dumps(value, ensure_ascii=True, separators=(",", ":"))
        else:
            value_text = str(value)
        pairs.append(f"{key}={value_text}")
    return " ".join(pairs)


def functional_diff_fields(revised_payload: dict[str, Any], golden_payload: dict[str, Any]) -> list[str]:
    if is_timeout_functional_payload(revised_payload) or is_timeout_functional_payload(
        golden_payload
    ):
        return []
    fields = []
    if revised_payload.get("data") != golden_payload.get("data"):
        fields.append("data")
    if revised_payload.get("diagnostics") != golden_payload.get("diagnostics"):
        fields.append("diagnostics")
    return fields


def is_empty_data(value: Any) -> bool:
    if isinstance(value, list) or isinstance(value, dict):
        return len(value) == 0
    return False


def report_functional_status(
    test_name: str,
    revised_functional: dict[str, dict[str, Any]],
    compare_functional: dict[str, dict[str, Any]],
) -> str:
    revised_payload = revised_functional.get(test_name)
    compare_payload = compare_functional.get(test_name)
    if revised_payload is None or compare_payload is None:
        return FUNCTIONAL_MISSING_MARKER
    if is_timeout_functional_payload(revised_payload) or is_timeout_functional_payload(
        compare_payload
    ):
        return FUNCTIONAL_TIMEOUT_MARKER
    if revised_payload.get("data") != compare_payload.get("data"):
        return f"{FUNCTIONAL_MISMATCH_MARKER}D"
    if revised_payload.get("diagnostics") != compare_payload.get("diagnostics"):
        return f"{FUNCTIONAL_MISMATCH_MARKER}X"
    if is_empty_data(revised_payload.get("data")):
        return f"{FUNCTIONAL_MATCH_MARKER}E"
    return FUNCTIONAL_MATCH_MARKER


def render_report(
    run_dir: pathlib.Path,
    results: dict[str, dict[str, Any]],
    tests_by_name: dict[str, dict[str, Any]],
    compare_results: dict[str, dict[str, Any]],
    compare_dir: pathlib.Path | None,
    functional_results: dict[str, dict[str, Any]],
    compare_functional_results: dict[str, dict[str, Any]],
    failure_results: dict[str, dict[str, Any]] | None = None,
) -> str:
    lines = [
        f"# CLI E2E Bench Run: {run_dir.name}",
        "",
        f"- Generated at (UTC): {datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}",
        f"- Hyperfine JSON files: {len(results)}",
        f"- Wavepeek JSON files: {len(functional_results)}",
        f"- Failure JSON files: {len(failure_results or {})}",
    ]
    if compare_dir is not None:
        lines.extend(
            [
                f"- Compare baseline: `{compare_dir}`",
                "- Delta formula: `((golden - revised) / golden) * 100`",
                "- Speed factor: `golden/revised` when faster, `revised/golden` when slower",
                f"- Emoji threshold: abs(delta) >= {EMOJI_THRESHOLD_PCT:.2f}% (`🟢` faster, `🔴` slower)",
                "- Functional status: `✅` match, `✅E` match with empty data, `⚠️D` data mismatch, `⚠️X` diagnostic mismatch, `⏱T` timeout artifact, `?` missing counterpart",
            ]
        )
    lines.append("")

    failures = failure_results or {}
    if failures:
        lines.extend(["## Explicit Failures", ""])
        lines.extend(["| test | phase | exit | timed out | message |", "| --- | --- | --- | --- | --- |"])
        for failure in sorted(failures.values(), key=lambda item: str(item.get("test_name", ""))):
            exit_code = failure.get("exit_code")
            lines.append(
                "| "
                + " | ".join(
                    [
                        escape_md(failure.get("test_name", "")),
                        escape_md(failure.get("phase", "")),
                        escape_md("-" if exit_code is None else exit_code),
                        escape_md(str(bool(failure.get("timed_out", False))).lower()),
                        escape_md(failure.get("message", "")),
                    ]
                )
                + " |"
            )
        lines.append("")

    if not results:
        lines.append("No hyperfine JSON artifacts found in this run directory.")
        lines.append("")
        return "\n".join(lines)

    grouped: dict[str, list[dict[str, Any]]] = {}
    for row in results.values():
        test = tests_by_name.get(str(row["test_name"]))
        category = str(test["category"]) if test is not None else "unknown"
        grouped.setdefault(category, []).append(row)

    for category in sorted(grouped):
        lines.extend([f"## {category}", ""])
        if compare_dir is not None:
            lines.extend(
                [
                    "| test | mean_s | functional | meta |",
                    "| --- | --- | --- | --- |",
                ]
            )
        else:
            lines.extend(
                [
                    "| test | mean_s | meta |",
                    "| --- | --- | --- |",
                ]
            )

        for row in sorted(
            grouped[category],
            key=lambda item: (-float(item["mean"]), str(item["test_name"])),
        ):
            test_name = str(row["test_name"])
            test = tests_by_name.get(test_name)
            baseline = compare_results.get(test_name)
            line = [
                escape_md(test_name),
                escape_md(format_metric(float(row["mean"]), baseline, "mean")),
            ]
            if compare_dir is not None:
                status = report_functional_status(
                    test_name,
                    functional_results,
                    compare_functional_results,
                )
                line.append(escape_md(status))
            line.append(escape_md(format_meta(dict(test.get("meta", {})) if test is not None else {})))
            lines.append("| " + " | ".join(line) + " |")
        lines.append("")

    return "\n".join(lines)


def write_report(
    run_dir: pathlib.Path,
    tests_by_name: dict[str, dict[str, Any]],
    compare_dir: pathlib.Path | None,
) -> pathlib.Path:
    results = load_hyperfine_results(run_dir)
    compare_results = load_hyperfine_results(compare_dir) if compare_dir is not None else {}
    functional_results = load_wavepeek_results(run_dir)
    compare_functional_results = (
        load_wavepeek_results(compare_dir) if compare_dir is not None else {}
    )
    failure_results = load_failure_results(run_dir)
    markdown = render_report(
        run_dir,
        results,
        tests_by_name,
        compare_results,
        compare_dir,
        functional_results,
        compare_functional_results,
        failure_results,
    )
    report_path = run_dir / README_NAME
    report_path.write_text(markdown, encoding="utf-8")
    return report_path


def preview_command(test: dict[str, Any]) -> str:
    try:
        parts = [str(token).format(wavepeek_bin="<binary>") for token in test["command"]]
    except KeyError as error:
        fail(f"error: tests: missing placeholder {error!s} in test `{test['name']}`")
    return shlex.join(parts)


def cmd_list(args: argparse.Namespace) -> int:
    tests_path = normalize_path(getattr(args, "tests", str(TESTS_PATH)))
    tests = select_tests(load_tests(tests_path), args.filter)
    for test in tests:
        print(str(test["name"]))
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    tests_path = normalize_path(getattr(args, "tests", str(TESTS_PATH)))
    tests = load_tests(tests_path)
    selected = select_tests(tests, args.filter)
    if not selected:
        fail("error: run: no tests matched the provided filter")
    verbose = bool(getattr(args, "verbose", False))
    fail_fast = bool(getattr(args, "fail_fast", False))

    timeout_seconds = int(args.wavepeek_timeout_seconds)
    if timeout_seconds < 1:
        fail("error: run: --wavepeek-timeout-seconds must be >= 1")

    compare_dir = normalize_path(args.compare) if args.compare else None
    if compare_dir is not None:
        ensure_existing_dir(compare_dir, "run")

    binaries = parse_binary_specs(args.binary)
    schedule = str(args.schedule)
    run_dir = resolve_run_dir(args.run_dir, args.out_dir)
    for binary in binaries:
        binary_run_dir(run_dir, binary).mkdir(parents=True, exist_ok=True)
    if verbose:
        print(f"info: run directory: {run_dir}")
        print("info: binaries: " + ", ".join(f"{b.label}={b.path}" for b in binaries))
        print(f"info: schedule: {schedule}")

    tests_by_name = {str(test["name"]): test for test in tests}
    runnable_by_label: dict[str, list[dict[str, Any]]] = {}
    if args.missing_only:
        for binary in binaries:
            runnable, skipped = partition_missing_only_tests(
                selected,
                binary_run_dir(run_dir, binary),
            )
            runnable_by_label[binary.label] = runnable
            if verbose:
                for test_name in skipped:
                    print(
                        f"info: skip `{test_name}` for `{binary.label}` "
                        "(missing-only: artifacts already exist)"
                    )
    else:
        runnable_by_label = {binary.label: list(selected) for binary in binaries}

    total_jobs = sum(len(jobs) for jobs in runnable_by_label.values())
    if total_jobs:
        ensure_hyperfine()
        completed = 0

        def run_one(binary: BinarySpec, test: dict[str, Any]) -> None:
            nonlocal completed
            completed += 1
            label_dir = binary_run_dir(run_dir, binary)
            if verbose:
                print(
                    f"[{completed}/{total_jobs}] {binary.label}/{test['name']} "
                    f"(runs={test['runs']}, warmup={test['warmup']})"
                )
            test_name = str(test["name"])
            remove_test_artifacts(label_dir, test_name)
            functional_result = run_functional_capture(
                test,
                binary.path,
                "run",
                timeout_seconds,
                fail_on_error=False,
            )
            if isinstance(functional_result, tuple):
                functional_payload, functional_failure = functional_result
            else:
                functional_payload = functional_result
                functional_failure = None
            if functional_failure is not None:
                functional_failure["binary_label"] = binary.label
                write_failure_artifact(label_dir, test_name, functional_failure)
                if fail_fast:
                    fail(
                        f"error: run: preflight failed for "
                        f"`{binary.label}/{test_name}`; see {failure_result_path(label_dir, test_name)}"
                    )
                if verbose:
                    print(
                        f"warning: run: preflight failed for "
                        f"`{binary.label}/{test_name}`; wrote failure artifact"
                    )
                return

            benchmark_failure = run_test(test, label_dir, binary.path, timeout_seconds, verbose)
            if isinstance(benchmark_failure, dict):
                benchmark_failure["binary_label"] = binary.label
                write_failure_artifact(label_dir, test_name, benchmark_failure)
                if fail_fast:
                    fail(
                        f"error: run: benchmark failed for "
                        f"`{binary.label}/{test_name}`; see {failure_result_path(label_dir, test_name)}"
                    )
                if verbose:
                    print(
                        f"warning: run: benchmark failed for "
                        f"`{binary.label}/{test_name}`; wrote failure artifact"
                    )
                return
            write_wavepeek_artifact(label_dir, test_name, functional_payload)

        if schedule == "round-robin":
            for test in selected:
                for binary in binaries:
                    if test in runnable_by_label[binary.label]:
                        run_one(binary, test)
        elif schedule == "grouped":
            for binary in binaries:
                for test in runnable_by_label[binary.label]:
                    run_one(binary, test)
        else:
            fail(f"error: run: unsupported schedule `{schedule}`")
    elif verbose:
        print("info: no tests to run after --missing-only filter")

    report_paths: list[pathlib.Path] = []
    for binary in binaries:
        report_paths.append(write_report(binary_run_dir(run_dir, binary), tests_by_name, compare_dir))
    write_run_manifest(
        run_dir=run_dir,
        tests_path=tests_path,
        binaries=binaries,
        selected_tests=selected,
        schedule=schedule,
        timeout_seconds=timeout_seconds,
    )
    index_path = write_run_index(run_dir, binaries)
    if verbose:
        print(f"info: run artifacts written to {run_dir}")
        for report_path in report_paths:
            print(f"info: report updated at {report_path}")
        print(f"info: run index updated at {index_path}")
    else:
        print("ok: run: completed successfully (use --verbose for detailed logs)")
    return 0


def cmd_report(args: argparse.Namespace) -> int:
    run_dir = normalize_path(args.run_dir)
    ensure_existing_dir(run_dir, "report")

    compare_dir = normalize_path(args.compare) if args.compare else None
    if compare_dir is not None:
        ensure_existing_dir(compare_dir, "report")

    tests_path = normalize_path(getattr(args, "tests", str(TESTS_PATH)))
    tests = load_tests(tests_path)
    tests_by_name = {str(test["name"]): test for test in tests}

    manifest_path = run_dir / "manifest.json"
    if manifest_path.is_file():
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as error:
            fail(f"error: report: invalid JSON in {manifest_path}: {error}")
        if isinstance(manifest, dict) and manifest.get("kind") == "wavepeek-e2e-bench-run":
            binaries = manifest.get("binaries")
            if not isinstance(binaries, list) or not binaries:
                fail(f"error: report: invalid labeled run manifest in {manifest_path}")
            specs: list[BinarySpec] = []
            for item in binaries:
                if not isinstance(item, dict) or not isinstance(item.get("label"), str):
                    fail(f"error: report: invalid binary entry in {manifest_path}")
                label = str(item["label"])
                path = str(item.get("path", ""))
                label_dir = run_dir / label
                ensure_existing_dir(label_dir, "report")
                report_path = write_report(label_dir, tests_by_name, compare_dir)
                print(f"info: report updated at {report_path}")
                specs.append(BinarySpec(label, path))
            index_path = write_run_index(run_dir, specs)
            print(f"info: run index updated at {index_path}")
            return 0

    report_path = write_report(run_dir, tests_by_name, compare_dir)
    print(f"info: report updated at {report_path}")
    return 0


def wavepeek_artifact_names(run_dir: pathlib.Path) -> set[str]:
    return {
        artifact_test_name(path, WAVEPEEK_SUFFIX)
        for path in sorted(run_dir.glob(f"*{WAVEPEEK_SUFFIX}"))
    }


def scan_artifact_outcomes(run_dir: pathlib.Path, *, functional_only: bool) -> dict[str, RunOutcome]:
    hyperfine_names = {
        artifact_test_name(path, HYPERFINE_SUFFIX)
        for path in sorted(run_dir.glob(f"*{HYPERFINE_SUFFIX}"))
    }
    wavepeek_names = wavepeek_artifact_names(run_dir)
    failure_names = {
        artifact_test_name(path, FAILURE_SUFFIX)
        for path in sorted(run_dir.glob(f"*{FAILURE_SUFFIX}"))
    }
    names = hyperfine_names | wavepeek_names | failure_names
    failures = load_failure_results(run_dir)
    outcomes: dict[str, RunOutcome] = {}
    for name in sorted(names):
        has_hyperfine = name in hyperfine_names
        has_wavepeek = name in wavepeek_names
        has_failure = name in failure_names
        errors: list[str] = []
        if functional_only:
            has_success = has_wavepeek
            if has_hyperfine and not has_wavepeek and not has_failure:
                errors.append(f"{name}: hyperfine artifact without wavepeek or failure artifact")
        else:
            has_success = has_hyperfine and has_wavepeek
            if has_hyperfine != has_wavepeek and not has_failure:
                missing = "wavepeek" if has_hyperfine else "hyperfine"
                errors.append(f"{name}: missing {missing} artifact")
        if has_failure and (has_hyperfine or has_wavepeek):
            errors.append(f"{name}: has normal artifacts and a failure artifact")
        if errors:
            outcomes[name] = RunOutcome("invalid", name, failures.get(name), errors)
        elif has_failure:
            outcomes[name] = RunOutcome("failure", name, failures.get(name), [])
        elif has_success:
            outcomes[name] = RunOutcome("success", name, None, [])
        else:
            outcomes[name] = RunOutcome("missing", name, None, [f"{name}: missing outcome artifacts"])
    return outcomes


def failure_summary_record(
    test_name: str,
    side: str,
    outcome: RunOutcome | None,
) -> dict[str, Any]:
    failure = outcome.failure if outcome is not None and outcome.failure is not None else {}
    return {
        "test_name": test_name,
        "side": side,
        "phase": failure.get("phase"),
        "exit_code": failure.get("exit_code"),
        "timed_out": bool(failure.get("timed_out", False)),
        "message": failure.get("message", ""),
    }


def classify_compare_outcomes(
    revised_dir: pathlib.Path,
    golden_dir: pathlib.Path,
    *,
    functional_only: bool,
    allow_golden_extra: bool = False,
) -> dict[str, Any]:
    revised_outcomes = scan_artifact_outcomes(revised_dir, functional_only=functional_only)
    golden_outcomes = scan_artifact_outcomes(golden_dir, functional_only=functional_only)
    revised_names = set(revised_outcomes)
    golden_names = set(golden_outcomes)
    names = revised_names | golden_names

    integrity_errors: list[str] = []
    baseline_unsupported: list[dict[str, Any]] = []
    revised_failures: list[dict[str, Any]] = []
    both_side_failures: list[dict[str, Any]] = []
    comparable: list[str] = []
    golden_extra: list[str] = []
    revised_only: list[str] = []
    golden_only: list[str] = []

    for name in sorted(names):
        revised = revised_outcomes.get(name)
        golden = golden_outcomes.get(name)
        if revised is not None and revised.status == "invalid":
            integrity_errors.extend(f"revised: {error}" for error in revised.errors)
        if golden is not None and golden.status == "invalid":
            integrity_errors.extend(f"golden: {error}" for error in golden.errors)

        revised_status = revised.status if revised is not None else "missing"
        golden_status = golden.status if golden is not None else "missing"
        if revised_status == "invalid" or golden_status == "invalid":
            continue
        if revised_status == "success" and golden_status == "success":
            comparable.append(name)
            continue
        if revised_status == "success" and golden_status == "failure":
            baseline_unsupported.append(failure_summary_record(name, "golden", golden))
            continue
        if revised_status == "failure" and golden_status == "failure":
            both_side_failures.append(
                {
                    "test_name": name,
                    "revised": failure_summary_record(name, "revised", revised),
                    "golden": failure_summary_record(name, "golden", golden),
                }
            )
            continue
        if revised_status == "failure" and golden_status == "success":
            revised_failures.append(failure_summary_record(name, "revised", revised))
            continue
        if revised_status == "missing" and golden_status != "missing":
            golden_only.append(name)
            if not allow_golden_extra:
                integrity_errors.append(f"{name}: missing outcome in revised run")
            else:
                golden_extra.append(name)
            continue
        if golden_status == "missing" and revised_status != "missing":
            revised_only.append(name)
            integrity_errors.append(f"{name}: missing outcome in golden run")
            continue

    return {
        "revised_outcomes": revised_outcomes,
        "golden_outcomes": golden_outcomes,
        "comparable": comparable,
        "baseline_unsupported": baseline_unsupported,
        "revised_failures": revised_failures,
        "both_side_failures": both_side_failures,
        "integrity_errors": integrity_errors,
        "revised_only": revised_only,
        "golden_only": golden_only,
        "golden_extra": golden_extra,
    }


def compare_functional_only(
    revised_dir: pathlib.Path,
    golden_dir: pathlib.Path,
    allow_golden_extra: bool,
    verbose: bool,
    result_json: str | None = None,
    ignored_tests: Mapping[str, str] | None = None,
) -> int:
    classification = classify_compare_outcomes(
        revised_dir,
        golden_dir,
        functional_only=True,
        allow_golden_extra=allow_golden_extra,
    )
    revised_outcomes: dict[str, RunOutcome] = classification["revised_outcomes"]
    golden_outcomes: dict[str, RunOutcome] = classification["golden_outcomes"]
    raw_revised_names = set(revised_outcomes)
    raw_golden_names = set(golden_outcomes)
    if not raw_revised_names:
        fail(f"error: compare: no wavepeek or failure JSON files found in {revised_dir}")
    if not raw_golden_names:
        fail(f"error: compare: no wavepeek or failure JSON files found in {golden_dir}")

    ignored = dict(ignored_tests or {})
    ignored_names = set(ignored)
    ignored_records = ignored_functional_test_records(
        ignored,
        raw_revised_names,
        raw_golden_names,
    )
    matched = list(classification["comparable"])
    revised_only = list(classification["revised_only"])
    golden_only = list(classification["golden_only"])
    baseline_unsupported = list(classification["baseline_unsupported"])
    revised_failures = list(classification["revised_failures"])
    both_side_failures = list(classification["both_side_failures"])
    integrity_errors = list(classification["integrity_errors"])

    functional_mismatches: list[str] = []
    functional_artifact_errors: list[str] = list(integrity_errors)
    functional_timeouts: list[str] = []

    if revised_only:
        functional_artifact_errors.append(
            "tests only in revised run: " + ", ".join(revised_only)
        )
    if golden_only and not allow_golden_extra:
        functional_artifact_errors.append(
            "tests only in golden run: " + ", ".join(golden_only)
        )

    for name in sorted(ignored):
        missing_sides: list[str] = []
        if name not in raw_revised_names:
            missing_sides.append("revised")
        if name not in raw_golden_names:
            missing_sides.append("golden")
        if missing_sides:
            functional_artifact_errors.append(
                f"ignored functional test `{name}` missing from "
                f"{', '.join(missing_sides)}"
            )
        elif name not in matched:
            functional_artifact_errors.append(
                f"ignored functional test `{name}` is not comparable on both sides"
            )

    for test_name in matched:
        revised_payload, revised_error = load_wavepeek_artifact_for_compare(
            revised_dir,
            test_name,
            "revised",
        )
        golden_payload, golden_error = load_wavepeek_artifact_for_compare(
            golden_dir,
            test_name,
            "golden",
        )

        if revised_error is not None:
            functional_artifact_errors.append(f"{test_name}: {revised_error}")
        if golden_error is not None:
            functional_artifact_errors.append(f"{test_name}: {golden_error}")
        if revised_payload is None or golden_payload is None:
            continue

        revised_timed_out = is_timeout_functional_payload(revised_payload)
        golden_timed_out = is_timeout_functional_payload(golden_payload)
        if revised_timed_out or golden_timed_out:
            timeout_sides: list[str] = []
            if revised_timed_out:
                timeout_sides.append("revised")
            if golden_timed_out:
                timeout_sides.append("golden")
            functional_timeouts.append(
                f"{test_name}: timeout artifact on {', '.join(timeout_sides)}"
            )
            continue

        diff_fields = functional_diff_fields(revised_payload, golden_payload)
        if test_name in ignored_names:
            diff_fields = [field for field in diff_fields if field != "data"]
        if diff_fields:
            functional_mismatches.append(
                f"{test_name}: mismatched fields {', '.join(diff_fields)}"
            )

    if verbose:
        if ignored_records:
            print("warning: compare: ignored functional tests:", file=sys.stderr)
            for record in ignored_records:
                print(
                    f"  - {record['test_name']}: {record['reason']}",
                    file=sys.stderr,
                )
        if golden_only and allow_golden_extra:
            print(
                "warning: compare: ignored tests only in golden run: "
                + ", ".join(golden_only),
                file=sys.stderr,
            )
        if baseline_unsupported:
            print("warning: compare: skipped tests unsupported by golden:", file=sys.stderr)
            for record in baseline_unsupported:
                print(f"  - {record['test_name']}: {record.get('message', '')}", file=sys.stderr)
        if both_side_failures:
            print("warning: compare: skipped tests failed on both sides:", file=sys.stderr)
            for record in both_side_failures:
                print(f"  - {record['test_name']}", file=sys.stderr)
        if revised_failures:
            print("error: compare: revised failures where golden passed:", file=sys.stderr)
            for record in revised_failures:
                print(f"  - {record['test_name']}: {record.get('message', '')}", file=sys.stderr)
        if functional_timeouts:
            print("error: compare: timeout functional artifacts detected:", file=sys.stderr)
            for issue in functional_timeouts:
                print(f"  - {issue}", file=sys.stderr)
        if functional_mismatches:
            print("error: compare: functional mismatch detected:", file=sys.stderr)
            for issue in functional_mismatches:
                print(f"  - {issue}", file=sys.stderr)
        if functional_artifact_errors:
            print("error: compare: functional artifact errors detected:", file=sys.stderr)
            for issue in functional_artifact_errors:
                print(f"  - {issue}", file=sys.stderr)

    status = (
        "failed"
        if revised_failures or functional_timeouts or functional_mismatches or functional_artifact_errors
        else "passed"
    )
    skipped_uncomparable_count = len(baseline_unsupported) + len(both_side_failures)
    write_result_json(
        result_json,
        {
            "kind": "wavepeek-e2e-compare-result",
            "schema_version": 1,
            "status": status,
            "functional_only": True,
            "allow_golden_extra": allow_golden_extra,
            "matched_count": len(matched),
            "comparable_count": len(matched),
            "skipped_uncomparable_count": skipped_uncomparable_count,
            "failed_uncomparable_count": len(revised_failures),
            "uncomparable_count": skipped_uncomparable_count + len(revised_failures),
            "revised_only": revised_only,
            "golden_only": golden_only,
            "baseline_unsupported": baseline_unsupported,
            "revised_failures": revised_failures,
            "both_side_failures": both_side_failures,
            "integrity_errors": integrity_errors,
            "ignored_functional_tests": ignored_records,
            "functional_timeouts": functional_timeouts,
            "functional_mismatches": functional_mismatches,
            "functional_artifact_errors": functional_artifact_errors,
            "timing_failures": [],
        },
    )

    if status == "failed":
        if not verbose:
            print(
                "error: compare: functional checks failed "
                "(use --verbose for detailed logs)",
                file=sys.stderr,
            )
        return 1

    if verbose:
        print("info: compare: functional checks passed")
    else:
        print("ok: compare: functional checks passed (use --verbose for detailed logs)")
    return 0


def cmd_compare(args: argparse.Namespace) -> int:
    revised_dir = normalize_path(args.revised)
    golden_dir = normalize_path(args.golden)
    ensure_existing_dir(revised_dir, "compare")
    ensure_existing_dir(golden_dir, "compare")

    functional_only = bool(getattr(args, "functional_only", False))
    allow_golden_extra = bool(getattr(args, "allow_golden_extra", False))
    verbose = bool(getattr(args, "verbose", False))

    ignored_tests = parse_ignored_functional_tests(
        getattr(args, "ignore_functional_test", None)
    )
    if allow_golden_extra and not functional_only:
        fail("error: compare: --allow-golden-extra requires --functional-only")
    if ignored_tests and not functional_only:
        fail("error: compare: --ignore-functional-test requires --functional-only")
    result_json = getattr(args, "result_json", None)

    if functional_only:
        return compare_functional_only(
            revised_dir,
            golden_dir,
            allow_golden_extra,
            verbose,
            result_json,
            ignored_tests,
        )

    if args.max_negative_delta_pct is None:
        fail("error: compare: --max-negative-delta-pct is required unless --functional-only is set")
    threshold = float(args.max_negative_delta_pct)
    if threshold < 0:
        fail("error: compare: --max-negative-delta-pct must be non-negative")
    threshold_seconds = float(getattr(args, "max_negative_delta_seconds", 0.0) or 0.0)
    if threshold_seconds < 0:
        fail("error: compare: --max-negative-delta-seconds must be non-negative")

    classification = classify_compare_outcomes(
        revised_dir,
        golden_dir,
        functional_only=False,
        allow_golden_extra=False,
    )
    revised_outcomes: dict[str, RunOutcome] = classification["revised_outcomes"]
    golden_outcomes: dict[str, RunOutcome] = classification["golden_outcomes"]
    if not revised_outcomes:
        fail(f"error: compare: no hyperfine, wavepeek, or failure JSON files found in {revised_dir}")
    if not golden_outcomes:
        fail(f"error: compare: no hyperfine, wavepeek, or failure JSON files found in {golden_dir}")

    matched = list(classification["comparable"])
    revised = load_hyperfine_results(revised_dir, matched)
    golden = load_hyperfine_results(golden_dir, matched)
    revised_only = list(classification["revised_only"])
    golden_only = list(classification["golden_only"])
    baseline_unsupported = list(classification["baseline_unsupported"])
    revised_failures = list(classification["revised_failures"])
    both_side_failures = list(classification["both_side_failures"])
    integrity_errors = list(classification["integrity_errors"])

    timing_failures: list[dict[str, Any]] = []
    functional_mismatches: list[str] = []
    functional_artifact_errors: list[str] = list(integrity_errors)
    functional_timeout_warnings: list[str] = []

    for test_name in matched:
        revised_row = revised[test_name]
        golden_row = golden[test_name]

        metric = COMPARE_TIMING_METRIC
        revised_time = float(revised_row[metric])
        golden_time = float(golden_row[metric])
        allowed = allowed_slowdown(golden_time, threshold, threshold_seconds)
        actual_slowdown = revised_time - golden_time
        if actual_slowdown > allowed:
            timing_failures.append(
                timing_record(
                    test_name=test_name,
                    metric=metric,
                    revised_time=revised_time,
                    golden_time=golden_time,
                    threshold_pct=threshold,
                    threshold_seconds=threshold_seconds,
                )
            )

        revised_payload, revised_error = load_wavepeek_artifact_for_compare(
            revised_dir,
            test_name,
            "revised",
        )
        golden_payload, golden_error = load_wavepeek_artifact_for_compare(
            golden_dir,
            test_name,
            "golden",
        )

        if revised_error is not None:
            functional_artifact_errors.append(f"{test_name}: {revised_error}")
        if golden_error is not None:
            functional_artifact_errors.append(f"{test_name}: {golden_error}")
        if revised_payload is None or golden_payload is None:
            continue

        revised_timed_out = is_timeout_functional_payload(revised_payload)
        golden_timed_out = is_timeout_functional_payload(golden_payload)
        if revised_timed_out or golden_timed_out:
            timeout_sides: list[str] = []
            if revised_timed_out:
                timeout_sides.append("revised")
            if golden_timed_out:
                timeout_sides.append("golden")
            functional_timeout_warnings.append(
                f"{test_name}: timeout artifact on {', '.join(timeout_sides)}"
            )

        diff_fields = functional_diff_fields(revised_payload, golden_payload)
        if diff_fields:
            functional_mismatches.append(
                f"{test_name}: mismatched fields {', '.join(diff_fields)}"
            )

    if verbose:
        if revised_only:
            print(
                "warning: compare: tests only in revised run: " + ", ".join(revised_only),
                file=sys.stderr,
            )
        if golden_only:
            print(
                "warning: compare: tests only in golden run: " + ", ".join(golden_only),
                file=sys.stderr,
            )
        if baseline_unsupported:
            print("warning: compare: skipped tests unsupported by golden:", file=sys.stderr)
            for record in baseline_unsupported:
                print(f"  - {record['test_name']}: {record.get('message', '')}", file=sys.stderr)
        if both_side_failures:
            print("warning: compare: skipped tests failed on both sides:", file=sys.stderr)
            for record in both_side_failures:
                print(f"  - {record['test_name']}", file=sys.stderr)
        if revised_failures:
            print("error: compare: revised failures where golden passed:", file=sys.stderr)
            for record in revised_failures:
                print(f"  - {record['test_name']}: {record.get('message', '')}", file=sys.stderr)
        if functional_timeout_warnings:
            print("warning: compare: timeout functional artifacts detected:", file=sys.stderr)
            for issue in functional_timeout_warnings:
                print(f"  - {issue}", file=sys.stderr)

        if timing_failures:
            print(
                "error: compare: median regression exceeds allowed slowdown "
                f"(max({threshold:.2f}%, {threshold_seconds:.6f}s)):",
                file=sys.stderr,
            )
            for issue in timing_failures:
                print(f"  - {format_timing_record(issue)}", file=sys.stderr)

        if functional_mismatches:
            print("error: compare: functional mismatch detected:", file=sys.stderr)
            for issue in functional_mismatches:
                print(f"  - {issue}", file=sys.stderr)

        if functional_artifact_errors:
            print("error: compare: functional artifact errors detected:", file=sys.stderr)
            for issue in functional_artifact_errors:
                print(f"  - {issue}", file=sys.stderr)

    status = (
        "failed"
        if revised_failures or timing_failures or functional_mismatches or functional_artifact_errors
        else "passed"
    )
    skipped_uncomparable_count = len(baseline_unsupported) + len(both_side_failures)
    write_result_json(
        result_json,
        {
            "kind": "wavepeek-e2e-compare-result",
            "schema_version": 1,
            "status": status,
            "functional_only": False,
            "allow_golden_extra": False,
            "matched_count": len(matched),
            "comparable_count": len(matched),
            "skipped_uncomparable_count": skipped_uncomparable_count,
            "failed_uncomparable_count": len(revised_failures),
            "uncomparable_count": skipped_uncomparable_count + len(revised_failures),
            "revised_only": revised_only,
            "golden_only": golden_only,
            "baseline_unsupported": baseline_unsupported,
            "revised_failures": revised_failures,
            "both_side_failures": both_side_failures,
            "integrity_errors": integrity_errors,
            "timing_metric": COMPARE_TIMING_METRIC,
            "threshold_pct": threshold,
            "threshold_seconds": threshold_seconds,
            "timing_failures": timing_failures,
            "functional_timeout_warnings": functional_timeout_warnings,
            "functional_mismatches": functional_mismatches,
            "functional_artifact_errors": functional_artifact_errors,
        },
    )

    if status == "failed":
        if not verbose:
            print(
                "error: compare: checks failed (use --verbose for detailed logs)",
                file=sys.stderr,
            )
        return 1

    if verbose:
        print("info: compare: all checks passed")
    else:
        print("ok: compare: all checks passed (use --verbose for detailed logs)")
    return 0


def cmd_confirm(args: argparse.Namespace) -> int:
    revised_dir = normalize_path(args.revised)
    golden_dir = normalize_path(args.golden)
    ensure_existing_dir(revised_dir, "confirm")
    ensure_existing_dir(golden_dir, "confirm")

    tests = list(getattr(args, "test", None) or [])
    if not tests:
        fail("error: confirm: at least one --test argument is required")

    threshold = float(args.max_negative_delta_pct)
    if threshold < 0:
        fail("error: confirm: --max-negative-delta-pct must be non-negative")
    threshold_seconds = float(getattr(args, "max_negative_delta_seconds", 0.0) or 0.0)
    if threshold_seconds < 0:
        fail("error: confirm: --max-negative-delta-seconds must be non-negative")

    verbose = bool(getattr(args, "verbose", False))
    records: list[dict[str, Any]] = []
    failures: list[dict[str, Any]] = []
    for test_name in tests:
        golden_times = hyperfine_sample_times(golden_dir, test_name, "confirm")
        revised_times = hyperfine_sample_times(revised_dir, test_name, "confirm")
        golden_best = min(golden_times)
        revised_best = min(revised_times)
        record = timing_record(
            test_name=test_name,
            metric="best",
            revised_time=revised_best,
            golden_time=golden_best,
            threshold_pct=threshold,
            threshold_seconds=threshold_seconds,
        )
        record["golden_sample_count"] = len(golden_times)
        record["revised_sample_count"] = len(revised_times)
        records.append(record)
        if float(record["slowdown_seconds"]) > float(record["allowed_slowdown_seconds"]):
            failures.append(record)

    status = "failed" if failures else "passed"
    write_result_json(
        getattr(args, "result_json", None),
        {
            "kind": "wavepeek-e2e-timing-confirm-result",
            "schema_version": 1,
            "status": status,
            "metric": "best",
            "threshold_pct": threshold,
            "threshold_seconds": threshold_seconds,
            "test_count": len(tests),
            "confirmed": records,
            "failures": failures,
        },
    )

    if verbose:
        if failures:
            print(
                "error: confirm: best-sample regression exceeds allowed slowdown "
                f"(max({threshold:.2f}%, {threshold_seconds:.6f}s)):",
                file=sys.stderr,
            )
            for record in failures:
                print(f"  - {format_timing_record(record)}", file=sys.stderr)
        else:
            print(
                "info: confirm: best-sample timing confirmation passed "
                f"for {len(records)} test(s)"
            )
            for record in records:
                print(f"  - {format_timing_record(record)}")

    if failures:
        if not verbose:
            print(
                "error: confirm: best-sample timing confirmation failed "
                "(use --verbose for detailed logs)",
                file=sys.stderr,
            )
        return 1

    if not verbose:
        print("ok: confirm: best-sample timing confirmation passed")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="CLI E2E benchmark harness")
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list", help="list available benchmark tests")
    list_parser.add_argument("--filter", default=None, help="regex filter by test name")
    list_parser.add_argument(
        "--tests",
        default=str(TESTS_PATH),
        help="path to benchmark tests catalog JSON",
    )

    run_parser = subparsers.add_parser("run", help="run selected benchmarks")
    run_parser.add_argument("--filter", default=None, help="regex filter by test name")
    run_parser.add_argument(
        "--binary",
        action="append",
        metavar="LABEL=PATH",
        help="binary to benchmark; repeat for multiple labeled binaries",
    )
    run_parser.add_argument(
        "--schedule",
        choices=("round-robin", "grouped"),
        default="round-robin",
        help="test scheduling across multiple binaries",
    )
    run_parser.add_argument("--run-dir", default=None, help="existing/new run directory")
    run_parser.add_argument(
        "--out-dir",
        default=str(DEFAULT_RUNS_DIR),
        help="parent for timestamped run directories",
    )
    run_parser.add_argument("--compare", default=None, help="baseline run directory for README deltas")
    run_parser.add_argument(
        "--missing-only",
        action="store_true",
        help="run only tests missing artifacts in run directory",
    )
    run_parser.add_argument(
        "--fail-fast",
        action="store_true",
        help="stop on the first per-test preflight or benchmark failure",
    )
    run_parser.add_argument(
        "--wavepeek-timeout-seconds",
        type=int,
        default=DEFAULT_WAVEPEEK_TIMEOUT_SECONDS,
        help="max seconds per wavepeek invocation before timeout cap",
    )
    run_parser.add_argument(
        "--tests",
        default=str(TESTS_PATH),
        help="path to benchmark tests catalog JSON",
    )
    run_parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="show detailed benchmark progress and diagnostics",
    )

    report_parser = subparsers.add_parser("report", help="generate/update README.md for a run")
    report_parser.add_argument("--run-dir", required=True, help="run directory")
    report_parser.add_argument("--compare", default=None, help="baseline run directory for README deltas")
    report_parser.add_argument(
        "--tests",
        default=str(TESTS_PATH),
        help="path to benchmark tests catalog JSON",
    )

    compare_parser = subparsers.add_parser("compare", help="check revised run against golden")
    compare_parser.add_argument("--revised", required=True, help="revised run directory")
    compare_parser.add_argument("--golden", required=True, help="golden run directory")
    compare_parser.add_argument(
        "--max-negative-delta-pct",
        required=False,
        type=float,
        help="relative median slowdown threshold",
    )
    compare_parser.add_argument(
        "--max-negative-delta-seconds",
        required=False,
        type=float,
        default=0.0,
        help="absolute median slowdown floor in seconds",
    )
    compare_parser.add_argument(
        "--functional-only",
        action="store_true",
        help="compare only wavepeek JSON artifacts and skip timing artifacts",
    )
    compare_parser.add_argument(
        "--allow-golden-extra",
        action="store_true",
        help="with --functional-only, allow extra artifacts in the golden directory",
    )
    compare_parser.add_argument(
        "--ignore-functional-test",
        action="append",
        help="with --functional-only, ignore one test as NAME=REASON",
    )
    compare_parser.add_argument(
        "--result-json",
        default=None,
        help="write machine-readable compare result JSON",
    )
    compare_parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="show detailed compare warnings and failures",
    )

    confirm_parser = subparsers.add_parser(
        "confirm",
        help="confirm failed timing tests with best hyperfine samples",
    )
    confirm_parser.add_argument("--revised", required=True, help="revised run directory")
    confirm_parser.add_argument("--golden", required=True, help="golden run directory")
    confirm_parser.add_argument(
        "--test",
        action="append",
        help="test name to confirm; repeat for multiple tests",
    )
    confirm_parser.add_argument(
        "--max-negative-delta-pct",
        required=True,
        type=float,
        help="relative best-sample slowdown threshold",
    )
    confirm_parser.add_argument(
        "--max-negative-delta-seconds",
        required=False,
        type=float,
        default=0.0,
        help="absolute best-sample slowdown floor in seconds",
    )
    confirm_parser.add_argument(
        "--result-json",
        default=None,
        help="write machine-readable confirmation result JSON",
    )
    confirm_parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="show detailed confirmation failures",
    )

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "list":
        return cmd_list(args)
    if args.command == "run":
        return cmd_run(args)
    if args.command == "report":
        return cmd_report(args)
    if args.command == "compare":
        return cmd_compare(args)
    if args.command == "confirm":
        return cmd_confirm(args)

    fail(f"error: unsupported command: {args.command}")
    raise AssertionError("unreachable")


if __name__ == "__main__":
    raise SystemExit(main())
