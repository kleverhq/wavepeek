#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import sys
from collections.abc import Mapping, Sequence
from typing import Any

from common import (
    COMPARE_SCHEMA_VERSION,
    DEFAULT_TIMING_THRESHOLD_PCT,
    DEFAULT_TIMING_THRESHOLD_SECONDS,
    REPO_ROOT,
    BenchGateError,
    CompareResult,
    enforce_clean_worktree,
    ensure_empty_dir,
    make_default_output_dir,
    read_json,
    relative_to,
    resolve_ref,
    run_command,
    utc_now,
    write_json,
)


def e2e_artifact_stems(run_dir: pathlib.Path) -> set[str]:
    hyperfine = {path.name.removesuffix(".hyperfine.json") for path in run_dir.glob("*.hyperfine.json")}
    wavepeek = {path.name.removesuffix(".wavepeek.json") for path in run_dir.glob("*.wavepeek.json")}
    if hyperfine != wavepeek:
        missing_wavepeek = sorted(hyperfine - wavepeek)
        missing_hyperfine = sorted(wavepeek - hyperfine)
        details: list[str] = []
        if missing_wavepeek:
            details.append("missing wavepeek JSON for " + ", ".join(missing_wavepeek))
        if missing_hyperfine:
            details.append("missing hyperfine JSON for " + ", ".join(missing_hyperfine))
        raise BenchGateError(f"incomplete e2e artifact set in {run_dir}: {'; '.join(details)}")
    if not hyperfine:
        raise BenchGateError(f"empty e2e artifact set in {run_dir}")
    return hyperfine


def assert_matching_e2e_artifacts(golden: pathlib.Path, revised: pathlib.Path) -> None:
    golden_stems = e2e_artifact_stems(golden)
    revised_stems = e2e_artifact_stems(revised)
    if golden_stems != revised_stems:
        golden_only = sorted(golden_stems - revised_stems)
        revised_only = sorted(revised_stems - golden_stems)
        details: list[str] = []
        if golden_only:
            details.append("golden-only " + ", ".join(golden_only))
        if revised_only:
            details.append("revised-only " + ", ".join(revised_only))
        raise BenchGateError(
            f"e2e artifact sets differ between {golden} and {revised}: {'; '.join(details)}"
        )


def suite_dir(capture_dir: pathlib.Path, suite_name: str) -> pathlib.Path:
    manifest_path = capture_dir / "manifest.json"
    if manifest_path.is_file():
        try:
            manifest = read_json(manifest_path)
        except BenchGateError:
            return capture_dir / suite_name
        suites = manifest.get("suites")
        if isinstance(suites, Mapping):
            suite = suites.get(suite_name)
            if isinstance(suite, Mapping) and isinstance(suite.get("path"), str):
                return (capture_dir / str(suite["path"])).resolve()
    return capture_dir / suite_name


def assert_e2e_subset(*, superset: pathlib.Path, subset: pathlib.Path) -> None:
    superset_stems = e2e_artifact_stems(superset)
    subset_stems = e2e_artifact_stems(subset)
    extra = sorted(subset_stems - superset_stems)
    if extra:
        raise BenchGateError(
            f"cross-format FSDB artifact set has entries missing from FST capture {superset}: "
            + ", ".join(extra)
        )


def timing_failure_names(compare_result: Mapping[str, Any]) -> list[str]:
    failures = compare_result.get("timing_failures")
    if not isinstance(failures, list):
        return []
    names: list[str] = []
    for item in failures:
        if isinstance(item, Mapping) and isinstance(item.get("test_name"), str):
            names.append(str(item["test_name"]))
    return names


def compare_failed_only_on_timing(compare_result: Mapping[str, Any]) -> bool:
    if not timing_failure_names(compare_result):
        return False
    for field in (
        "functional_mismatches",
        "functional_artifact_errors",
        "functional_timeout_warnings",
    ):
        value = compare_result.get(field)
        if isinstance(value, list) and value:
            return False
    return True


def run_e2e_best_confirm(
    *,
    name: str,
    golden: pathlib.Path,
    revised: pathlib.Path,
    compare_dir: pathlib.Path,
    tooling_root: pathlib.Path,
    threshold_pct: float,
    threshold_seconds: float,
    test_names: Sequence[str],
) -> dict[str, Any]:
    result_json_path = compare_dir / f"{name}.best-confirm.result.json"
    args = [
        "python3",
        "-B",
        str(tooling_root / "bench/e2e/perf.py"),
        "confirm",
        "--revised",
        str(revised),
        "--golden",
        str(golden),
        "--max-negative-delta-pct",
        str(threshold_pct),
        "--max-negative-delta-seconds",
        str(threshold_seconds),
        "--result-json",
        str(result_json_path),
        "--verbose",
    ]
    for test_name in test_names:
        args.extend(["--test", test_name])

    log_path = compare_dir / f"{name}.best-confirm.log"
    result = run_command(
        f"confirm-{name}-best",
        args,
        cwd=tooling_root,
        log_path=log_path,
        check=False,
    )
    status = "passed" if result.returncode == 0 else "failed"
    confirm: dict[str, Any] = {
        "status": status,
        "returncode": result.returncode,
        "metric": "best",
        "test_count": len(test_names),
        "threshold_pct": threshold_pct,
        "threshold_seconds": threshold_seconds,
        "log_path": relative_to(log_path, compare_dir),
        "result_json_path": relative_to(result_json_path, compare_dir),
    }
    if result_json_path.is_file():
        try:
            payload = read_json(result_json_path)
        except BenchGateError as error:
            confirm["result_json_error"] = str(error)
        else:
            failures = payload.get("failures")
            if isinstance(failures, list):
                confirm["failure_count"] = len(failures)
    return confirm


def run_e2e_compare(
    *,
    name: str,
    golden: pathlib.Path,
    revised: pathlib.Path,
    compare_dir: pathlib.Path,
    tooling_root: pathlib.Path,
    threshold_pct: float,
    threshold_seconds: float,
    functional_only: bool,
    allow_golden_extra: bool = False,
) -> dict[str, Any]:
    if functional_only and allow_golden_extra:
        assert_e2e_subset(superset=golden, subset=revised)
    elif functional_only:
        assert_matching_e2e_artifacts(golden, revised)
    else:
        assert_matching_e2e_artifacts(golden, revised)

    result_json_path = compare_dir / f"{name}.result.json"
    args = [
        "python3",
        "-B",
        str(tooling_root / "bench/e2e/perf.py"),
        "compare",
        "--revised",
        str(revised),
        "--golden",
        str(golden),
        "--result-json",
        str(result_json_path),
        "--verbose",
    ]
    if functional_only:
        args.append("--functional-only")
        if allow_golden_extra:
            args.append("--allow-golden-extra")
    else:
        args.extend(
            [
                "--max-negative-delta-pct",
                str(threshold_pct),
                "--max-negative-delta-seconds",
                str(threshold_seconds),
            ]
        )

    log_path = compare_dir / f"{name}.log"
    result = run_command(
        f"compare-{name}",
        args,
        cwd=tooling_root,
        log_path=log_path,
        check=False,
    )
    status = "passed" if result.returncode == 0 else "failed"
    suite: dict[str, Any] = {
        "status": status,
        "returncode": result.returncode,
        "threshold_pct": None if functional_only else threshold_pct,
        "threshold_seconds": None if functional_only else threshold_seconds,
        "timing_metric": None if functional_only else "median",
        "functional_only": functional_only,
        "allow_golden_extra": allow_golden_extra,
        "log_path": relative_to(log_path, compare_dir),
        "result_json_path": relative_to(result_json_path, compare_dir),
    }

    compare_result: dict[str, Any] = {}
    if result_json_path.is_file():
        try:
            compare_result = read_json(result_json_path)
        except BenchGateError as error:
            suite["result_json_error"] = str(error)

    if not functional_only and result.returncode != 0 and compare_failed_only_on_timing(compare_result):
        failed_tests = timing_failure_names(compare_result)
        confirm = run_e2e_best_confirm(
            name=name,
            golden=golden,
            revised=revised,
            compare_dir=compare_dir,
            tooling_root=tooling_root,
            threshold_pct=threshold_pct,
            threshold_seconds=threshold_seconds,
            test_names=failed_tests,
        )
        suite["median_compare_status"] = "failed"
        suite["median_compare_returncode"] = result.returncode
        suite["timing_confirm"] = confirm
        if confirm.get("status") == "passed":
            suite["status"] = "passed"
            suite["returncode"] = 0
    return suite


def render_compare_summary(manifest: Mapping[str, Any]) -> str:
    lines = [
        "# Wavepeek Benchmark Comparison",
        "",
        f"Golden: `{manifest.get('golden_dir', '<unknown>')}`",
        f"Revised: `{manifest.get('revised_dir', '<unknown>')}`",
        "",
        "## Results",
        "",
    ]
    suites = manifest.get("suites", {})
    if isinstance(suites, Mapping):
        for name in sorted(suites):
            suite = suites[name]
            if not isinstance(suite, Mapping):
                continue
            status = suite.get("status", "unknown")
            text = f"- `{name}`: {status}"
            if suite.get("functional_only"):
                text += " (functional-only)"
            elif suite.get("threshold_pct") is not None:
                threshold_seconds = suite.get("threshold_seconds")
                timing_metric = suite.get("timing_metric") or "timing"
                if threshold_seconds is not None:
                    text += f" ({timing_metric} threshold max({suite['threshold_pct']}%, {threshold_seconds}s))"
                else:
                    text += f" ({timing_metric} threshold {suite['threshold_pct']}%)"
            timing_confirm = suite.get("timing_confirm")
            if isinstance(timing_confirm, Mapping):
                if timing_confirm.get("status") == "passed":
                    text += " (median failed, best-sample confirm passed)"
                else:
                    text += " (best-sample confirm failed)"
            if suite.get("reason"):
                text += f" — {suite['reason']}"
            if suite.get("log_path"):
                text += f" (log: `{suite['log_path']}`)"
            if isinstance(timing_confirm, Mapping) and timing_confirm.get("log_path"):
                text += f" (confirm: `{timing_confirm['log_path']}`)"
            lines.append(text)
    lines.append("")
    return "\n".join(lines)


def compare_captures(
    *,
    golden_dir: pathlib.Path,
    revised_dir: pathlib.Path,
    compare_dir: pathlib.Path,
    tooling_root: pathlib.Path = REPO_ROOT,
    timing_threshold_pct: float = DEFAULT_TIMING_THRESHOLD_PCT,
    timing_threshold_seconds: float = DEFAULT_TIMING_THRESHOLD_SECONDS,
) -> CompareResult:
    if not golden_dir.is_dir():
        raise BenchGateError(f"golden capture directory does not exist: {golden_dir}")
    if not revised_dir.is_dir():
        raise BenchGateError(f"revised capture directory does not exist: {revised_dir}")
    ensure_empty_dir(compare_dir)
    tooling_root = tooling_root.resolve()
    tooling_sha = resolve_ref(tooling_root, "HEAD")

    suites: dict[str, dict[str, Any]] = {}
    failures: list[str] = []

    required = {
        "e2e-fst": lambda: run_e2e_compare(
            name="e2e-fst",
            golden=suite_dir(golden_dir, "e2e-fst"),
            revised=suite_dir(revised_dir, "e2e-fst"),
            compare_dir=compare_dir,
            tooling_root=tooling_root,
            threshold_pct=timing_threshold_pct,
            threshold_seconds=timing_threshold_seconds,
            functional_only=False,
        ),
    }
    for name, runner in required.items():
        golden = suite_dir(golden_dir, name)
        revised = suite_dir(revised_dir, name)
        if not golden.is_dir() or not revised.is_dir():
            suites[name] = {"status": "failed", "reason": "required suite missing from one or both captures"}
            failures.append(name)
            continue
        try:
            suite_result = runner()
        except BenchGateError as error:
            suite_result = {"status": "failed", "reason": str(error)}
        suites[name] = suite_result
        if suite_result.get("status") != "passed":
            failures.append(name)

    golden_fsdb = suite_dir(golden_dir, "e2e-fsdb")
    revised_fsdb = suite_dir(revised_dir, "e2e-fsdb")
    if golden_fsdb.is_dir() or revised_fsdb.is_dir():
        if not golden_fsdb.is_dir() or not revised_fsdb.is_dir():
            suites["e2e-fsdb"] = {"status": "failed", "reason": "suite exists in only one capture"}
            failures.append("e2e-fsdb")
        else:
            try:
                suite_result = run_e2e_compare(
                    name="e2e-fsdb",
                    golden=golden_fsdb,
                    revised=revised_fsdb,
                    compare_dir=compare_dir,
                    tooling_root=tooling_root,
                    threshold_pct=timing_threshold_pct,
                    threshold_seconds=timing_threshold_seconds,
                    functional_only=False,
                )
            except BenchGateError as error:
                suite_result = {"status": "failed", "reason": str(error)}
            suites["e2e-fsdb"] = suite_result
            if suite_result.get("status") != "passed":
                failures.append("e2e-fsdb")
    else:
        suites["e2e-fsdb"] = {"status": "skipped", "reason": "optional suite missing from both captures"}

    cross_checks = {
        "cross-golden-fst-fsdb": golden_dir,
        "cross-revised-fst-fsdb": revised_dir,
    }
    for name, capture_dir in cross_checks.items():
        fst = suite_dir(capture_dir, "e2e-fst")
        fsdb = suite_dir(capture_dir, "e2e-fsdb")
        if not fsdb.is_dir():
            suites[name] = {"status": "skipped", "reason": "FSDB suite missing from capture"}
            continue
        if not fst.is_dir():
            suites[name] = {"status": "failed", "reason": "FST suite missing from capture"}
            failures.append(name)
            continue
        try:
            suite_result = run_e2e_compare(
                name=name,
                golden=fst,
                revised=fsdb,
                compare_dir=compare_dir,
                tooling_root=tooling_root,
                threshold_pct=timing_threshold_pct,
                threshold_seconds=timing_threshold_seconds,
                functional_only=True,
                allow_golden_extra=True,
            )
        except BenchGateError as error:
            suite_result = {"status": "failed", "reason": str(error)}
        suites[name] = suite_result
        if suite_result.get("status") != "passed":
            failures.append(name)

    status = "passed" if not failures else "failed"
    manifest: dict[str, Any] = {
        "schema_version": COMPARE_SCHEMA_VERSION,
        "kind": "wavepeek-bench-compare",
        "generated_at_utc": utc_now().isoformat().replace("+00:00", "Z"),
        "golden_dir": str(golden_dir),
        "revised_dir": str(revised_dir),
        "status": status,
        "tooling_root": str(tooling_root),
        "tooling_sha": tooling_sha,
        "timing_threshold_pct": timing_threshold_pct,
        "timing_threshold_seconds": timing_threshold_seconds,
        "timing_metric": "median",
        "suites": suites,
    }
    write_json(compare_dir / "manifest.json", manifest)
    (compare_dir / "README.md").write_text(render_compare_summary(manifest), encoding="utf-8")
    return CompareResult(
        compare_dir=compare_dir,
        manifest=manifest,
        exit_code=0 if status == "passed" else 1,
    )


def compare_command(args: argparse.Namespace) -> int:
    source_root = args.source_root.resolve()
    enforce_clean_worktree(
        source_root,
        reason="current benchmark tooling must be committed before compare",
    )
    compare_dir = args.out_dir or make_default_output_dir("compares", "compare")
    result = compare_captures(
        golden_dir=args.golden.resolve(),
        revised_dir=args.revised.resolve(),
        compare_dir=compare_dir.resolve(),
        tooling_root=source_root,
        timing_threshold_pct=args.max_negative_delta_pct,
        timing_threshold_seconds=args.max_negative_delta_seconds,
    )
    print(f"comparison written to {result.compare_dir}")
    return result.exit_code


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Compare two wavepeek benchmark capture directories")
    parser.add_argument("--golden", type=pathlib.Path, required=True)
    parser.add_argument("--revised", type=pathlib.Path, required=True)
    parser.add_argument("--out-dir", type=pathlib.Path)
    parser.add_argument("--source-root", type=pathlib.Path, default=REPO_ROOT)
    parser.add_argument(
        "--max-negative-delta-pct",
        type=float,
        default=DEFAULT_TIMING_THRESHOLD_PCT,
        help="relative median slowdown threshold for same-format FST and FSDB timing",
    )
    parser.add_argument(
        "--max-negative-delta-seconds",
        type=float,
        default=DEFAULT_TIMING_THRESHOLD_SECONDS,
        help="absolute median slowdown floor in seconds for same-format FST and FSDB timing",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(list(argv) if argv is not None else None)
    try:
        return compare_command(args)
    except BenchGateError as error:
        print(f"error: bench-compare: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
