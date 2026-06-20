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
    REPO_ROOT,
    BenchGateError,
    CompareResult,
    ensure_empty_dir,
    make_default_output_dir,
    relative_to,
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


def assert_e2e_subset(*, superset: pathlib.Path, subset: pathlib.Path) -> None:
    superset_stems = e2e_artifact_stems(superset)
    subset_stems = e2e_artifact_stems(subset)
    extra = sorted(subset_stems - superset_stems)
    if extra:
        raise BenchGateError(
            f"cross-format FSDB artifact set has entries missing from FST capture {superset}: "
            + ", ".join(extra)
        )


def run_e2e_compare(
    *,
    name: str,
    golden: pathlib.Path,
    revised: pathlib.Path,
    compare_dir: pathlib.Path,
    threshold_pct: float,
    functional_only: bool,
    allow_golden_extra: bool = False,
) -> dict[str, Any]:
    if functional_only and allow_golden_extra:
        assert_e2e_subset(superset=golden, subset=revised)
    elif functional_only:
        assert_matching_e2e_artifacts(golden, revised)
    else:
        assert_matching_e2e_artifacts(golden, revised)

    args = [
        "python3",
        "-B",
        str(REPO_ROOT / "bench/e2e/perf.py"),
        "compare",
        "--revised",
        str(revised),
        "--golden",
        str(golden),
        "--verbose",
    ]
    if functional_only:
        args.append("--functional-only")
        if allow_golden_extra:
            args.append("--allow-golden-extra")
    else:
        args.extend(["--max-negative-delta-pct", str(threshold_pct)])

    log_path = compare_dir / f"{name}.log"
    result = run_command(
        f"compare-{name}",
        args,
        cwd=REPO_ROOT,
        log_path=log_path,
        check=False,
    )
    status = "passed" if result.returncode == 0 else "failed"
    return {
        "status": status,
        "returncode": result.returncode,
        "threshold_pct": None if functional_only else threshold_pct,
        "functional_only": functional_only,
        "allow_golden_extra": allow_golden_extra,
        "log_path": relative_to(log_path, compare_dir),
    }


def run_expr_compare(
    *,
    golden: pathlib.Path,
    revised: pathlib.Path,
    compare_dir: pathlib.Path,
    threshold_pct: float,
) -> dict[str, Any]:
    log_path = compare_dir / "expr.log"
    result = run_command(
        "compare-expr",
        [
            "python3",
            "-B",
            str(REPO_ROOT / "bench/expr/perf.py"),
            "compare",
            "--revised",
            str(revised),
            "--golden",
            str(golden),
            "--max-negative-delta-pct",
            str(threshold_pct),
        ],
        cwd=REPO_ROOT,
        log_path=log_path,
        check=False,
    )
    status = "passed" if result.returncode == 0 else "failed"
    return {
        "status": status,
        "returncode": result.returncode,
        "threshold_pct": threshold_pct,
        "functional_only": False,
        "log_path": relative_to(log_path, compare_dir),
    }


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
                text += f" (threshold {suite['threshold_pct']}%)"
            if suite.get("reason"):
                text += f" — {suite['reason']}"
            if suite.get("log_path"):
                text += f" (log: `{suite['log_path']}`)"
            lines.append(text)
    lines.append("")
    return "\n".join(lines)


def compare_captures(
    *,
    golden_dir: pathlib.Path,
    revised_dir: pathlib.Path,
    compare_dir: pathlib.Path,
    timing_threshold_pct: float = DEFAULT_TIMING_THRESHOLD_PCT,
) -> CompareResult:
    if not golden_dir.is_dir():
        raise BenchGateError(f"golden capture directory does not exist: {golden_dir}")
    if not revised_dir.is_dir():
        raise BenchGateError(f"revised capture directory does not exist: {revised_dir}")
    ensure_empty_dir(compare_dir)

    suites: dict[str, dict[str, Any]] = {}
    failures: list[str] = []

    required = {
        "e2e-fst": lambda: run_e2e_compare(
            name="e2e-fst",
            golden=golden_dir / "e2e-fst",
            revised=revised_dir / "e2e-fst",
            compare_dir=compare_dir,
            threshold_pct=timing_threshold_pct,
            functional_only=False,
        ),
        "expr": lambda: run_expr_compare(
            golden=golden_dir / "expr",
            revised=revised_dir / "expr",
            compare_dir=compare_dir,
            threshold_pct=timing_threshold_pct,
        ),
    }
    for name, runner in required.items():
        golden = golden_dir / name
        revised = revised_dir / name
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

    golden_fsdb = golden_dir / "e2e-fsdb"
    revised_fsdb = revised_dir / "e2e-fsdb"
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
                    threshold_pct=timing_threshold_pct,
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
        fst = capture_dir / "e2e-fst"
        fsdb = capture_dir / "e2e-fsdb"
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
                threshold_pct=timing_threshold_pct,
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
        "timing_threshold_pct": timing_threshold_pct,
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
    compare_dir = args.out_dir or make_default_output_dir("compares", "compare")
    result = compare_captures(
        golden_dir=args.golden.resolve(),
        revised_dir=args.revised.resolve(),
        compare_dir=compare_dir.resolve(),
        timing_threshold_pct=args.max_negative_delta_pct,
    )
    print(f"comparison written to {result.compare_dir}")
    return result.exit_code


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Compare two wavepeek benchmark capture directories")
    parser.add_argument("--golden", type=pathlib.Path, required=True)
    parser.add_argument("--revised", type=pathlib.Path, required=True)
    parser.add_argument("--out-dir", type=pathlib.Path)
    parser.add_argument(
        "--max-negative-delta-pct",
        type=float,
        default=DEFAULT_TIMING_THRESHOLD_PCT,
        help="maximum allowed negative delta for same-format FST, same-format FSDB, and expression timing",
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
