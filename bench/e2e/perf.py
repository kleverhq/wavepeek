#!/usr/bin/env python3

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
import math
import os
import pathlib
import re
import shlex
import shutil
import subprocess
import sys
from typing import Any, NoReturn


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]
TESTS_PATH = SCRIPT_DIR / "tests.json"
DEFAULT_RUNS_DIR = SCRIPT_DIR / "runs"
README_NAME = "README.md"
WAVEPEEK_BIN_ENV = "WAVEPEEK_BIN"
EMOJI_THRESHOLD_PCT = 3.0
METRICS = ("mean", "stddev", "median", "min", "max")
HYPERFINE_SUFFIX = ".hyperfine.json"
WAVEPEEK_SUFFIX = ".wavepeek.json"
FUNCTIONAL_MATCH_MARKER = "✅"
FUNCTIONAL_MISMATCH_MARKER = "⚠️"
FUNCTIONAL_MISSING_MARKER = "?"
FUNCTIONAL_TIMEOUT_MARKER = "⏱T"
DEFAULT_WAVEPEEK_TIMEOUT_SECONDS = 300


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


def load_tests() -> list[dict[str, Any]]:
    if not TESTS_PATH.exists():
        fail(f"error: tests: missing file {TESTS_PATH}")

    try:
        payload = json.loads(TESTS_PATH.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: tests: invalid JSON in {TESTS_PATH}: {error}")

    if not isinstance(payload, dict):
        fail(f"error: tests: root of {TESTS_PATH} must be object")

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


def resolve_wavepeek_bin() -> str:
    value = os.environ.get(WAVEPEEK_BIN_ENV, "wavepeek")
    if pathlib.Path(value).is_absolute() or "/" in value:
        path = normalize_path(value)
        if not path.exists():
            fail(f"error: run: `{WAVEPEEK_BIN_ENV}` points to missing file: {path}")
        return str(path)

    if shutil.which(value) is None:
        fail(f"error: run: `{WAVEPEEK_BIN_ENV}` value `{value}` is not in PATH")
    return value


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


def test_has_complete_artifacts(run_dir: pathlib.Path, test_name: str) -> bool:
    return hyperfine_result_path(run_dir, test_name).is_file() and wavepeek_result_path(
        run_dir, test_name
    ).is_file()


def partition_missing_only_tests(
    selected: list[dict[str, Any]],
    run_dir: pathlib.Path,
) -> tuple[list[dict[str, Any]], list[str]]:
    runnable: list[dict[str, Any]] = []
    skipped: list[str] = []
    for test in selected:
        test_name = str(test["name"])
        if test_has_complete_artifacts(run_dir, test_name):
            skipped.append(test_name)
        else:
            runnable.append(test)
    return runnable, skipped


def run_test(
    test: dict[str, Any],
    run_dir: pathlib.Path,
    wavepeek_bin: str,
    timeout_seconds: int,
) -> None:
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
    result = subprocess.run(hyperfine_cmd, check=False, cwd=REPO_ROOT)
    if result.returncode != 0:
        fail(f"error: run: hyperfine failed for `{test['name']}`")


def validate_functional_payload(payload: Any, source: str) -> dict[str, Any]:
    if not isinstance(payload, dict):
        raise ValueError(f"{source} must be object")
    if "data" not in payload:
        raise ValueError(f"{source} missing key `data`")
    if "warnings" not in payload:
        raise ValueError(f"{source} missing key `warnings`")
    if not isinstance(payload["data"], dict) and not isinstance(payload["data"], list):
        raise ValueError(f"{source} field `data` must be object or list")
    if not isinstance(payload["warnings"], list):
        raise ValueError(f"{source} field `warnings` must be list")
    return {"data": payload["data"], "warnings": payload["warnings"]}


def run_functional_capture(
    test: dict[str, Any],
    wavepeek_bin: str,
    caller: str,
    timeout_seconds: int,
) -> dict[str, Any]:
    command_args = build_functional_command(resolve_test_command(test, wavepeek_bin))
    result = subprocess.run(
        command_args,
        check=False,
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        timeout=timeout_seconds,
    )
    if result.returncode != 0:
        details = result.stderr.strip()
        suffix = f": {details}" if details else ""
        fail(
            f"error: {caller}: functional capture failed for `{test['name']}` "
            f"(exit {result.returncode}){suffix}"
        )

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        fail(f"error: {caller}: invalid JSON output for `{test['name']}`: {error}")

    try:
        return validate_functional_payload(payload, f"functional output for `{test['name']}`")
    except ValueError as error:
        fail(f"error: {caller}: {error}")
    raise AssertionError("unreachable")


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


def load_hyperfine_results(run_dir: pathlib.Path) -> dict[str, dict[str, Any]]:
    result_map: dict[str, dict[str, Any]] = {}
    for path in sorted(run_dir.glob(f"*{HYPERFINE_SUFFIX}")):
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
    if revised_payload.get("data") != golden_payload.get("data"):
        return ["data"]
    return []


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
) -> str:
    lines = [
        f"# CLI E2E Bench Run: {run_dir.name}",
        "",
        f"- Generated at (UTC): {datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')}",
        f"- Run directory: `{run_dir}`",
        f"- Hyperfine JSON files: {len(results)}",
        f"- Wavepeek JSON files: {len(functional_results)}",
    ]
    if compare_dir is not None:
        lines.extend(
            [
                f"- Compare baseline: `{compare_dir}`",
                "- Delta formula: `((golden - revised) / golden) * 100`",
                "- Speed factor: `golden/revised` when faster, `revised/golden` when slower",
                f"- Emoji threshold: abs(delta) >= {EMOJI_THRESHOLD_PCT:.2f}% (`🟢` faster, `🔴` slower)",
                "- Functional status: `✅` match, `✅E` match with empty data, `⚠️D` data mismatch, `⏱T` timeout artifact, `?` missing counterpart",
            ]
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
    markdown = render_report(
        run_dir,
        results,
        tests_by_name,
        compare_results,
        compare_dir,
        functional_results,
        compare_functional_results,
    )
    report_path = run_dir / README_NAME
    report_path.write_text(markdown, encoding="utf-8")
    return report_path


def preview_command(test: dict[str, Any]) -> str:
    try:
        parts = [
            str(token).format(wavepeek_bin="${WAVEPEEK_BIN}") for token in test["command"]
        ]
    except KeyError as error:
        fail(f"error: tests: missing placeholder {error!s} in test `{test['name']}`")
    return shlex.join(parts)


def cmd_list(args: argparse.Namespace) -> int:
    tests = select_tests(load_tests(), args.filter)
    for test in tests:
        print(
            f"{test['name']}\t{test['category']}\truns={test['runs']}\twarmup={test['warmup']}\t{preview_command(test)}"
        )
    print(f"total={len(tests)}")
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    tests = load_tests()
    selected = select_tests(tests, args.filter)
    if not selected:
        fail("error: run: no tests matched the provided filter")

    timeout_seconds = int(args.wavepeek_timeout_seconds)
    if timeout_seconds < 1:
        fail("error: run: --wavepeek-timeout-seconds must be >= 1")

    compare_dir = normalize_path(args.compare) if args.compare else None
    if compare_dir is not None:
        ensure_existing_dir(compare_dir, "run")

    run_dir = resolve_run_dir(args.run_dir, args.out_dir)
    print(f"info: run directory: {run_dir}")

    selected_to_run = selected
    if args.missing_only:
        selected_to_run, skipped = partition_missing_only_tests(selected, run_dir)
        for test_name in skipped:
            print(
                f"info: skip `{test_name}` (missing-only: artifacts already exist)"
            )

    if selected_to_run:
        ensure_hyperfine()
        wavepeek_bin = resolve_wavepeek_bin()
        for index, test in enumerate(selected_to_run, start=1):
            print(
                f"[{index}/{len(selected_to_run)}] {test['name']} "
                f"(runs={test['runs']}, warmup={test['warmup']})"
            )
            run_test(test, run_dir, wavepeek_bin, timeout_seconds)
            try:
                functional_payload = run_functional_capture(
                    test,
                    wavepeek_bin,
                    "run",
                    timeout_seconds,
                )
            except subprocess.TimeoutExpired:
                print(
                    f"warning: run: functional capture timed out for `{test['name']}` "
                    f"after {timeout_seconds}s; writing empty wavepeek artifact"
                )
                write_wavepeek_timeout_artifact(run_dir, str(test["name"]))
                continue
            write_wavepeek_artifact(run_dir, str(test["name"]), functional_payload)
    else:
        print("info: no tests to run after --missing-only filter")

    tests_by_name = {str(test["name"]): test for test in tests}
    report_path = write_report(run_dir, tests_by_name, compare_dir)
    print(f"info: run artifacts written to {run_dir}")
    print(f"info: report updated at {report_path}")
    return 0


def cmd_report(args: argparse.Namespace) -> int:
    run_dir = normalize_path(args.run_dir)
    ensure_existing_dir(run_dir, "report")

    compare_dir = normalize_path(args.compare) if args.compare else None
    if compare_dir is not None:
        ensure_existing_dir(compare_dir, "report")

    tests = load_tests()
    tests_by_name = {str(test["name"]): test for test in tests}
    report_path = write_report(run_dir, tests_by_name, compare_dir)
    print(f"info: report updated at {report_path}")
    return 0


def cmd_compare(args: argparse.Namespace) -> int:
    revised_dir = normalize_path(args.revised)
    golden_dir = normalize_path(args.golden)
    ensure_existing_dir(revised_dir, "compare")
    ensure_existing_dir(golden_dir, "compare")

    threshold = float(args.max_negative_delta_pct)
    if threshold < 0:
        fail("error: compare: --max-negative-delta-pct must be non-negative")

    revised = load_hyperfine_results(revised_dir)
    golden = load_hyperfine_results(golden_dir)
    if not revised:
        fail(f"error: compare: no hyperfine JSON files found in {revised_dir}")

    revised_names = set(revised)
    golden_names = set(golden)
    matched = sorted(revised_names & golden_names)
    revised_only = sorted(revised_names - golden_names)
    golden_only = sorted(golden_names - revised_names)

    timing_failures: list[str] = []
    functional_mismatches: list[str] = []
    functional_artifact_errors: list[str] = []
    functional_timeout_warnings: list[str] = []

    for test_name in matched:
        revised_row = revised[test_name]
        golden_row = golden[test_name]

        delta = delta_pct(float(revised_row["mean"]), float(golden_row["mean"]))
        if delta is not None and delta < -threshold:
            speed = format_speed_factor(
                float(revised_row["mean"]),
                float(golden_row["mean"]),
            )
            timing_failures.append(
                f"{test_name}: mean revised={float(revised_row['mean']):.6f}s, "
                f"golden={float(golden_row['mean']):.6f}s, "
                f"delta={delta:+.2f}%, speed={speed}"
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
    if functional_timeout_warnings:
        print("warning: compare: timeout functional artifacts detected:", file=sys.stderr)
        for issue in functional_timeout_warnings:
            print(f"  - {issue}", file=sys.stderr)

    if timing_failures:
        print(
            "error: compare: mean regression exceeds allowed negative delta "
            f"({threshold:.2f}%):",
            file=sys.stderr,
        )
        for issue in timing_failures:
            print(f"  - {issue}", file=sys.stderr)

    if functional_mismatches:
        print("error: compare: functional mismatch detected:", file=sys.stderr)
        for issue in functional_mismatches:
            print(f"  - {issue}", file=sys.stderr)

    if functional_artifact_errors:
        print("error: compare: functional artifact errors detected:", file=sys.stderr)
        for issue in functional_artifact_errors:
            print(f"  - {issue}", file=sys.stderr)

    if timing_failures or functional_mismatches or functional_artifact_errors:
        return 1

    print("info: compare: all checks passed")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="CLI E2E benchmark harness")
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list", help="list available benchmark tests")
    list_parser.add_argument("--filter", default=None, help="regex filter by test name")

    run_parser = subparsers.add_parser("run", help="run selected benchmarks")
    run_parser.add_argument("--filter", default=None, help="regex filter by test name")
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
        "--wavepeek-timeout-seconds",
        type=int,
        default=DEFAULT_WAVEPEEK_TIMEOUT_SECONDS,
        help="max seconds per wavepeek invocation before timeout cap",
    )

    report_parser = subparsers.add_parser("report", help="generate/update README.md for a run")
    report_parser.add_argument("--run-dir", required=True, help="run directory")
    report_parser.add_argument("--compare", default=None, help="baseline run directory for README deltas")

    compare_parser = subparsers.add_parser("compare", help="check revised run against golden")
    compare_parser.add_argument("--revised", required=True, help="revised run directory")
    compare_parser.add_argument("--golden", required=True, help="golden run directory")
    compare_parser.add_argument(
        "--max-negative-delta-pct",
        required=True,
        type=float,
        help="fail when mean delta goes below negative threshold",
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

    fail(f"error: unsupported command: {args.command}")
    raise AssertionError("unreachable")


if __name__ == "__main__":
    raise SystemExit(main())
