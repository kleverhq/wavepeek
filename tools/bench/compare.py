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


HYPERFINE_SUFFIX = ".hyperfine.json"
WAVEPEEK_SUFFIX = ".wavepeek.json"
FAILURE_SUFFIX = ".failure.json"


CROSS_FORMAT_FUNCTIONAL_IGNORES: tuple[tuple[str, str], ...] = (
    (
        "scope_clustered_all_depth13_json",
        "FST exposes generated memory pseudo-scopes such as `.Memory`; "
        "FSDB does not expose equivalent hierarchy nodes.",
    ),
    (
        "scope_scr1_all_depth7_json",
        "FST exposes selected unpacked-array and vector aggregates as "
        "scope-like nodes; FSDB exposes the same objects through signal "
        "elements instead.",
    ),
    (
        "signal_scr1_top_recursive_all_json",
        "FST and FSDB encode unpacked-array elements, vector ranges, and "
        "width-1 scalar ranges differently in signal path strings.",
    ),
    (
        "signal_scr1_top_recursive_depth2_json",
        "FST and FSDB count array-element hierarchy levels differently at "
        "this depth boundary, so FSDB reports extra element signals.",
    ),
    (
        "signal_scr1_top_recursive_filter_valid_json",
        "FSDB includes an explicit `[0:0]` range suffix for this width-1 "
        "signal while FST reports the scalar name without the suffix.",
    ),
)

SHM_FUNCTIONAL_IGNORES: tuple[tuple[str, str], ...] = (
    (
        "info_scr1:time_start:1ps:0ps",
        "The converted SHM starts at zero; the source FST starts at 1ps.",
    ),
    (
        "info_scr1_isr_sample:time_start:1ps:0ps",
        "The converted SHM starts at zero; the source FST starts at 1ps.",
    ),
    (
        "info_scr1_riscv_compliance:time_start:1ps:0ps",
        "The converted SHM starts at zero; the source FST starts at 1ps.",
    ),
    (
        "scope_scr1_all_depth7_json",
        "SHM exposes indexed instance names as literal components.",
    ),
    (
        "signal_scr1_top_recursive_all_json",
        "SHM exposes converted declarations with generic kinds and literal indexed names.",
    ),
    (
        "signal_scr1_top_recursive_depth2_json",
        "SHM exposes converted declarations with generic kinds and literal indexed names.",
    ),
    (
        "signal_scr1_top_recursive_filter_valid_json:kind",
        "SHM exposes converted declarations with generic kinds.",
    ),
)


def e2e_artifact_stems(run_dir: pathlib.Path, *, functional_only: bool = False) -> set[str]:
    hyperfine = {path.name.removesuffix(HYPERFINE_SUFFIX) for path in run_dir.glob(f"*{HYPERFINE_SUFFIX}")}
    wavepeek = {path.name.removesuffix(WAVEPEEK_SUFFIX) for path in run_dir.glob(f"*{WAVEPEEK_SUFFIX}")}
    failures = {path.name.removesuffix(FAILURE_SUFFIX) for path in run_dir.glob(f"*{FAILURE_SUFFIX}")}
    success = set(wavepeek) if functional_only else hyperfine & wavepeek
    details: list[str] = []

    missing_wavepeek = sorted(hyperfine - wavepeek - failures)
    missing_hyperfine = [] if functional_only else sorted(wavepeek - hyperfine - failures)
    if missing_wavepeek:
        details.append("missing wavepeek JSON for " + ", ".join(missing_wavepeek))
    if missing_hyperfine:
        details.append("missing hyperfine JSON for " + ", ".join(missing_hyperfine))

    conflicting = sorted(failures & (hyperfine | wavepeek))
    if conflicting:
        details.append("both normal and failure JSON for " + ", ".join(conflicting))
    if details:
        raise BenchGateError(f"incomplete e2e artifact set in {run_dir}: {'; '.join(details)}")

    outcomes = success | failures
    if not outcomes:
        raise BenchGateError(f"empty e2e artifact set in {run_dir}")
    return outcomes


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


def suite_entry(capture_dir: pathlib.Path, suite_name: str) -> Mapping[str, Any]:
    manifest_path = capture_dir / "manifest.json"
    if manifest_path.is_file():
        try:
            manifest = read_json(manifest_path)
        except BenchGateError:
            return {}
        suites = manifest.get("suites")
        if isinstance(suites, Mapping):
            suite = suites.get(suite_name)
            if isinstance(suite, Mapping):
                return suite
    return {}


def suite_dir(capture_dir: pathlib.Path, suite_name: str) -> pathlib.Path:
    path = suite_entry(capture_dir, suite_name).get("path")
    if isinstance(path, str):
        return (capture_dir / path).resolve()
    return capture_dir / suite_name


def suite_inputs(run_dir: pathlib.Path) -> dict[str, Any] | None:
    manifest_path = run_dir.parent / "manifest.json"
    if not manifest_path.is_file():
        return None
    manifest = read_json(manifest_path)
    if manifest.get("kind") != "wavepeek-e2e-bench-run":
        return None
    inputs = manifest.get("inputs")
    if inputs is not None and (
        not isinstance(inputs, dict)
        or not inputs.get("tests")
        or not inputs.get("files")
        or not inputs.get("catalog_sha256")
    ):
        raise BenchGateError(f"invalid input identity in {manifest_path}")
    return inputs


def compare_suite_inputs(
    golden: pathlib.Path, revised: pathlib.Path, *, required: bool
) -> str:
    golden_inputs = suite_inputs(golden)
    revised_inputs = suite_inputs(revised)
    if golden_inputs is None or revised_inputs is None:
        if required:
            raise BenchGateError("input checksums missing from one or both runs")
        return "unverified"
    if golden_inputs != revised_inputs:
        raise BenchGateError("benchmark inputs or catalog differ between revisions")
    assert_recorded_coverage(golden, golden_inputs, require_success=required)
    assert_recorded_coverage(revised, revised_inputs, require_success=required)
    return "shared-run" if golden.parent == revised.parent else "matched"


def assert_recorded_coverage(
    run_dir: pathlib.Path, inputs: Mapping[str, Any], *, require_success: bool = True
) -> None:
    if set(inputs["tests"]) != e2e_artifact_stems(run_dir) or (
        require_success and any(run_dir.glob(f"*{FAILURE_SUFFIX}"))
    ):
        raise BenchGateError(f"incomplete benchmark coverage in {run_dir}")


def assert_e2e_subset(*, superset: pathlib.Path, subset: pathlib.Path) -> None:
    superset_stems = e2e_artifact_stems(superset, functional_only=True)
    subset_stems = e2e_artifact_stems(subset, functional_only=True)
    extra = sorted(subset_stems - superset_stems)
    if extra:
        raise BenchGateError(
            f"cross-format artifact set has entries missing from FST capture {superset}: "
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
        "revised_failures",
        "integrity_errors",
    ):
        value = compare_result.get(field)
        if isinstance(value, list) and value:
            return False
    return True


COMPARE_RESULT_SUMMARY_FIELDS = (
    "matched_count",
    "comparable_count",
    "skipped_uncomparable_count",
    "failed_uncomparable_count",
    "uncomparable_count",
    "revised_only",
    "golden_only",
    "baseline_unsupported",
    "revised_failures",
    "both_side_failures",
    "integrity_errors",
    "timing_failures",
    "functional_timeout_warnings",
    "functional_timeouts",
    "functional_mismatches",
    "functional_artifact_errors",
)


def copy_compare_result_summary(suite: dict[str, Any], compare_result: Mapping[str, Any]) -> None:
    for field in COMPARE_RESULT_SUMMARY_FIELDS:
        if field in compare_result:
            suite[field] = compare_result[field]


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
    ignored_functional_tests: Sequence[tuple[str, str]] = (),
) -> dict[str, Any]:
    if functional_only and allow_golden_extra:
        assert_e2e_subset(superset=golden, subset=revised)
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
    ignored_records = [
        {"test_name": rule.split(":", 1)[0], "rule": rule, "reason": reason}
        for rule, reason in ignored_functional_tests
    ]
    if functional_only:
        args.append("--functional-only")
        if allow_golden_extra:
            args.append("--allow-golden-extra")
        for test_name, reason in ignored_functional_tests:
            args.extend(["--ignore-functional-test", f"{test_name}={reason}"])
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
    if ignored_records:
        suite["ignored_functional_tests"] = ignored_records

    compare_result: dict[str, Any] = {}
    if result_json_path.is_file():
        try:
            compare_result = read_json(result_json_path)
        except BenchGateError as error:
            suite["result_json_error"] = str(error)
        else:
            copy_compare_result_summary(suite, compare_result)
            if "ignored_functional_tests" in compare_result:
                suite["ignored_functional_tests"] = compare_result["ignored_functional_tests"]

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
                ignored = suite.get("ignored_functional_tests")
                if isinstance(ignored, list) and ignored:
                    text += f"; ignored {len(ignored)} functional metadata test(s)"
            elif suite.get("threshold_pct") is not None:
                threshold_seconds = suite.get("threshold_seconds")
                timing_metric = suite.get("timing_metric") or "timing"
                if threshold_seconds is not None:
                    text += f" ({timing_metric} threshold max({suite['threshold_pct']}%, {threshold_seconds}s))"
                else:
                    text += f" ({timing_metric} threshold {suite['threshold_pct']}%)"
            timing_confirm = suite.get("timing_confirm")
            comparable_count = suite.get("comparable_count")
            skipped_count = suite.get("skipped_uncomparable_count")
            failed_uncomparable_count = suite.get("failed_uncomparable_count")
            if comparable_count is not None:
                text += f"; comparable {comparable_count}"
            if skipped_count:
                text += f"; skipped {skipped_count} uncomparable"
            if failed_uncomparable_count:
                text += f"; failed {failed_uncomparable_count} uncomparable"
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
    release_profile: str | None = None,
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

    required = {"e2e-fst"}
    if release_profile:
        required.add("e2e-vcd")
    if release_profile == "private":
        required.update(("e2e-fsdb", "e2e-shm"))
    names = ("e2e-fst", "e2e-vcd", "e2e-fsdb", "e2e-shm")
    for name in names:
        golden = suite_dir(golden_dir, name)
        revised = suite_dir(revised_dir, name)
        base_entry = suite_entry(golden_dir, name)
        revised_entry = suite_entry(revised_dir, name)
        try:
            failed = [
                f"{label}: {entry.get('reason', 'capture failed')}"
                for label, entry in (("baseline", base_entry), ("revised", revised_entry))
                if entry.get("status") == "failed"
            ]
            if failed:
                raise BenchGateError("; ".join(failed))
            if base_entry.get("status") == "first-baseline" and revised.is_dir():
                inputs = suite_inputs(revised)
                if inputs is None:
                    raise BenchGateError("input checksums missing from the first baseline capture")
                assert_recorded_coverage(revised, inputs)
                # Use the E2E parser to validate both kinds of saved artifacts.
                validation = run_e2e_compare(
                    name=f"{name}-first-baseline-validation",
                    golden=revised,
                    revised=revised,
                    compare_dir=compare_dir,
                    tooling_root=tooling_root,
                    threshold_pct=timing_threshold_pct,
                    threshold_seconds=timing_threshold_seconds,
                    functional_only=False,
                )
                if validation["status"] != "passed":
                    raise BenchGateError(f"invalid first-baseline results; see {validation['log_path']}")
                suite_result = {
                    "status": "first-baseline",
                    "reason": base_entry["reason"],
                    "comparable_count": 0,
                    "measured_count": len(inputs["tests"]),
                }
            elif not golden.is_dir() and not revised.is_dir() and name not in required:
                suite_result = {
                    "status": "skipped",
                    "reason": revised_entry.get("reason") or "optional suite missing from both captures",
                }
            elif not golden.is_dir() or not revised.is_dir():
                raise BenchGateError("suite missing from one or both captures")
            else:
                input_status = compare_suite_inputs(golden, revised, required=bool(release_profile))
                suite_result = run_e2e_compare(
                    name=name,
                    golden=golden,
                    revised=revised,
                    compare_dir=compare_dir,
                    tooling_root=tooling_root,
                    threshold_pct=timing_threshold_pct,
                    threshold_seconds=timing_threshold_seconds,
                    functional_only=False,
                )
                suite_result["input_identity"] = input_status
        except BenchGateError as error:
            suite_result = {"status": "failed", "reason": str(error)}
        suites[name] = suite_result
        if suite_result.get("status") == "failed":
            failures.append(name)

    cross_ignores = {"fsdb": CROSS_FORMAT_FUNCTIONAL_IGNORES, "shm": SHM_FUNCTIONAL_IGNORES}
    cross_checks = (
        (f"cross-{label}-fst-{fmt}", capture_dir, fmt)
        for label, capture_dir in (("golden", golden_dir), ("revised", revised_dir))
        for fmt in ("vcd", "fsdb", "shm")
    )
    for name, capture_dir, fmt in cross_checks:
        fst = suite_dir(capture_dir, "e2e-fst")
        other = suite_dir(capture_dir, f"e2e-{fmt}")
        if not other.is_dir():
            suites[name] = {"status": "skipped", "reason": f"{fmt.upper()} suite missing from capture"}
            continue
        if not fst.is_dir():
            suites[name] = {"status": "failed", "reason": "FST suite missing from capture"}
            failures.append(name)
            continue
        try:
            suite_result = run_e2e_compare(
                name=name,
                golden=fst,
                revised=other,
                compare_dir=compare_dir,
                tooling_root=tooling_root,
                threshold_pct=timing_threshold_pct,
                threshold_seconds=timing_threshold_seconds,
                functional_only=True,
                allow_golden_extra=release_profile is None and fmt == "fsdb",
                ignored_functional_tests=cross_ignores.get(fmt, ()),
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
        help="relative median slowdown threshold for same-format timing",
    )
    parser.add_argument(
        "--max-negative-delta-seconds",
        type=float,
        default=DEFAULT_TIMING_THRESHOLD_SECONDS,
        help="absolute median slowdown floor in seconds for same-format timing",
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
