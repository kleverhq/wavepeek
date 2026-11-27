#!/usr/bin/env python3

from __future__ import annotations

import argparse
import dataclasses
import pathlib
import sys
from collections.abc import Mapping, Sequence
from typing import Any

from capture import (
    assess_fsdb,
    build_release,
    build_release_fsdb,
    build_release_shm,
    finalize_capture,
    init_capture_session,
    prepare_fsdb,
    prepare_shm,
    resolve_gate_fsdb_plan,
    run_e2e_fsdb_many,
    run_e2e_fst_many,
    run_e2e_shm_many,
    run_e2e_vcd_many,
    prepare_vcd,
    source_supports_feature,
)
from common import (
    DEFAULT_TIMING_THRESHOLD_PCT,
    DEFAULT_TIMING_THRESHOLD_SECONDS,
    GATE_SCHEMA_VERSION,
    REPO_ROOT,
    BenchGateError,
    FsdbPlan,
    clone_checkout,
    enforce_clean_worktree,
    ensure_empty_dir,
    make_default_output_dir,
    resolve_ref,
    utc_now,
    write_json,
)
from compare import compare_captures


def gate_compare_summary(compare_manifest: Mapping[str, Any]) -> dict[str, Any]:
    suites = compare_manifest.get("suites", {})
    suite_summaries: dict[str, Any] = {}
    totals = {
        "comparable_count": 0,
        "skipped_uncomparable_count": 0,
        "failed_uncomparable_count": 0,
        "uncomparable_count": 0,
        "integrity_error_count": 0,
    }
    if isinstance(suites, Mapping):
        for name, raw_suite in suites.items():
            if not isinstance(raw_suite, Mapping):
                continue
            summary: dict[str, Any] = {"status": raw_suite.get("status", "unknown")}
            for field in (
                "comparable_count",
                "skipped_uncomparable_count",
                "failed_uncomparable_count",
                "uncomparable_count",
                "baseline_unsupported",
                "revised_failures",
                "both_side_failures",
                "integrity_errors",
                "reason",
                "input_identity",
                "measured_count",
            ):
                if field in raw_suite:
                    summary[field] = raw_suite[field]
            suite_summaries[str(name)] = summary
            for field in (
                "comparable_count",
                "skipped_uncomparable_count",
                "failed_uncomparable_count",
                "uncomparable_count",
            ):
                value = raw_suite.get(field)
                if isinstance(value, int):
                    totals[field] += value
            integrity_errors = raw_suite.get("integrity_errors")
            if isinstance(integrity_errors, list):
                totals["integrity_error_count"] += len(integrity_errors)
    return {"suites": suite_summaries, "totals": totals}


def render_gate_summary(manifest: Mapping[str, Any]) -> str:
    compare_section = manifest.get("compare", {})
    compare_status = compare_section.get("status", "unknown") if isinstance(compare_section, Mapping) else "unknown"
    totals = compare_section.get("totals", {}) if isinstance(compare_section, Mapping) else {}
    lines = [
        "# Wavepeek Manual Performance Gate",
        "",
        f"Baseline ref: `{manifest.get('baseline_ref', '<unknown>')}`",
        f"Baseline SHA: `{manifest.get('baseline_sha', '<unknown>')}`",
        f"Revised ref: `{manifest.get('revised_ref', '<unknown>')}`",
        f"Revised SHA: `{manifest.get('revised_sha', '<unknown>')}`",
        f"Comparison status: **{compare_status}**",
        "",
    ]
    if isinstance(totals, Mapping):
        lines.extend(
            [
                f"Comparable tests: `{totals.get('comparable_count', 0)}`",
                f"Skipped uncomparable tests: `{totals.get('skipped_uncomparable_count', 0)}`",
                f"Failed uncomparable tests: `{totals.get('failed_uncomparable_count', 0)}`",
                f"Integrity errors: `{totals.get('integrity_error_count', 0)}`",
                "",
            ]
        )
    if isinstance(compare_section, Mapping):
        if compare_section.get("reason"):
            lines.extend([str(compare_section["reason"]), ""])
        for name, suite in compare_section.get("suites", {}).items():
            detail = f" — {suite['reason']}" if suite.get("reason") else ""
            lines.append(f"- `{name}` {suite['status']}{detail}")
        lines.append("")
    lines.extend(
        [
            "See `baseline/`, `revised/`, and `compare/` for captured artifacts and logs.",
            "Each `e2e-*/manifest.json` records the catalog and input checksums for its run.",
            "",
        ]
    )
    return "\n".join(lines)


def gate_command(args: argparse.Namespace) -> int:
    source_root = args.source_root.resolve()
    baseline_sha = resolve_ref(source_root, args.baseline_ref)
    revised_sha = resolve_ref(source_root, args.revised_ref)
    tooling_sha = resolve_ref(source_root, "HEAD")
    enforce_clean_worktree(
        source_root,
        reason="current benchmark tooling must be committed before running the gate",
    )

    out_dir = args.out_dir or make_default_output_dir(
        "gates",
        f"{baseline_sha[:12]}..{revised_sha[:12]}",
    )
    out_dir = out_dir.resolve()
    ensure_empty_dir(out_dir)
    profile = getattr(args, "release_profile", None)
    manifest = {
        "schema_version": GATE_SCHEMA_VERSION,
        "kind": "wavepeek-bench-gate",
        "generated_at_utc": utc_now().isoformat().replace("+00:00", "Z"),
        "baseline_ref": args.baseline_ref,
        "baseline_sha": baseline_sha,
        "revised_ref": args.revised_ref,
        "revised_sha": revised_sha,
        "source_root": str(source_root),
        "tooling_sha": tooling_sha,
        "release_profile": profile,
        "timing_threshold_pct": args.max_negative_delta_pct,
        "timing_threshold_seconds": args.max_negative_delta_seconds,
        "timing_metric": "median",
    }
    try:
        exit_code = run_gate(args, manifest, out_dir)
    except BenchGateError as error:
        manifest["compare"] = {"status": "failed", "reason": str(error)}
        exit_code = 1
        print(f"error: bench-gate: {error}", file=sys.stderr)
    write_json(out_dir / "manifest.json", manifest)
    (out_dir / "summary.md").write_text(render_gate_summary(manifest), encoding="utf-8")
    print(f"gate written to {out_dir}")
    return exit_code


def first_baseline(baseline: pathlib.Path, revised: pathlib.Path, feature: str) -> bool:
    if not source_supports_feature(revised, feature):
        raise BenchGateError(f"revised ref lacks the required {feature} feature")
    return not source_supports_feature(baseline, feature)


def run_gate(args: argparse.Namespace, manifest: dict[str, Any], out_dir: pathlib.Path) -> int:
    source_root = pathlib.Path(manifest["source_root"])
    profile = manifest["release_profile"]
    fsdb_mode = {"public": "never", "private": "always"}.get(profile, args.fsdb)
    vcd_mode = "always" if profile else args.vcd
    manifest.update(fsdb_mode=fsdb_mode, vcd_mode=vcd_mode)
    checkouts_dir = out_dir / "checkouts"
    checkouts_dir.mkdir(parents=True, exist_ok=True)
    baseline_checkout = checkouts_dir / "baseline"
    revised_checkout = checkouts_dir / "revised"
    clone_checkout(source_root, manifest["baseline_sha"], baseline_checkout)
    clone_checkout(source_root, manifest["revised_sha"], revised_checkout)

    preflight_dir = out_dir / "logs"
    new_fsdb = profile == "private" and first_baseline(baseline_checkout, revised_checkout, "fsdb")
    new_shm = profile == "private" and first_baseline(baseline_checkout, revised_checkout, "shm")
    baseline_fsdb = (
        FsdbPlan(capture=False, status="unsupported")
        if new_fsdb
        else assess_fsdb(
            baseline_checkout,
            tooling_root=source_root,
            log_path=preflight_dir / "baseline-fsdb-check-env.log",
            mode=fsdb_mode,
        )
    )
    revised_fsdb = assess_fsdb(
        revised_checkout,
        tooling_root=source_root,
        log_path=preflight_dir / "revised-fsdb-check-env.log",
        mode=fsdb_mode,
    )
    gate_fsdb_plan = (
        revised_fsdb if new_fsdb else resolve_gate_fsdb_plan(baseline_fsdb, revised_fsdb, mode=fsdb_mode)
    )
    manifest["fsdb_plan"] = dataclasses.asdict(gate_fsdb_plan)

    baseline = init_capture_session(
        tooling_root=source_root,
        tooling_sha=manifest["tooling_sha"],
        binary_checkout=baseline_checkout,
        capture_dir=out_dir / "baseline",
        source_ref=args.baseline_ref,
        source_sha=manifest["baseline_sha"],
        fsdb_mode=fsdb_mode,
        fsdb_plan=baseline_fsdb if new_fsdb else gate_fsdb_plan,
        environment_note=args.environment_note,
        binary_label="baseline",
        strict=bool(profile),
    )
    revised = init_capture_session(
        tooling_root=source_root,
        tooling_sha=manifest["tooling_sha"],
        binary_checkout=revised_checkout,
        capture_dir=out_dir / "revised",
        source_ref=args.revised_ref,
        source_sha=manifest["revised_sha"],
        fsdb_mode=fsdb_mode,
        fsdb_plan=gate_fsdb_plan,
        environment_note=args.environment_note,
        binary_label="revised",
        strict=bool(profile),
    )
    for feature, introduced in (("fsdb", new_fsdb), ("shm", new_shm)):
        if introduced:
            baseline.suites[f"e2e-{feature}"] = {
                "status": "first-baseline",
                "reason": f"baseline ref lacks the {feature} feature",
            }

    # Keep preparation work out of the measured section. Build both refs first,
    # then prepare waveforms, then run each measured suite for baseline and revised
    # in same-format pairs before comparing artifacts.
    build_release(baseline)
    build_release(revised)
    if vcd_mode == "always":
        prepare_vcd(baseline)
    if gate_fsdb_plan.capture:
        build_release_fsdb(baseline)
        build_release_fsdb(revised)
        prepare_fsdb(revised)
    shm_sessions = [revised] if new_shm else [baseline, revised]
    if profile == "private":
        for session in shm_sessions:
            build_release_shm(session)
        prepare_shm(revised)

    run_e2e_fst_many(
        [baseline, revised],
        run_dir=out_dir / "e2e-fst",
        log_path=out_dir / "logs" / "bench-e2e-fst.log",
    )
    if vcd_mode == "always":
        run_e2e_vcd_many(
            [baseline, revised],
            run_dir=out_dir / "e2e-vcd",
            log_path=out_dir / "logs" / "bench-e2e-vcd.log",
        )
    if gate_fsdb_plan.capture:
        run_e2e_fsdb_many(
            [baseline, revised],
            run_dir=out_dir / "e2e-fsdb",
            log_path=out_dir / "logs" / "bench-e2e-fsdb.log",
        )
    if profile == "private":
        run_e2e_shm_many(
            shm_sessions,
            run_dir=out_dir / "e2e-shm",
            log_path=out_dir / "logs" / "bench-e2e-shm.log",
        )

    finalize_capture(baseline)
    finalize_capture(revised)

    compare = compare_captures(
        golden_dir=out_dir / "baseline",
        revised_dir=out_dir / "revised",
        compare_dir=out_dir / "compare",
        tooling_root=source_root,
        timing_threshold_pct=args.max_negative_delta_pct,
        timing_threshold_seconds=args.max_negative_delta_seconds,
        release_profile=profile,
    )

    status = "passed" if compare.exit_code == 0 else "failed"
    compare_summary = gate_compare_summary(compare.manifest)
    manifest["compare"] = {"status": status, "path": "compare", **compare_summary}
    return compare.exit_code


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run the manual wavepeek performance gate")
    parser.add_argument("--baseline-ref", required=True)
    parser.add_argument("--revised-ref", default="HEAD")
    parser.add_argument("--source-root", type=pathlib.Path, default=REPO_ROOT)
    parser.add_argument("--out-dir", type=pathlib.Path)
    parser.add_argument("--fsdb", choices=("auto", "always", "never"), default="auto")
    parser.add_argument("--vcd", choices=("always", "never"), default="never")
    parser.add_argument(
        "--release-profile", choices=("public", "private"),
        help="require VCD and FST, plus FSDB and SHM in the private profile",
    )
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
    parser.add_argument(
        "--environment-note",
        default="wavepeek manual performance gate",
        help="note written into benchmark gate capture manifests",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(list(argv) if argv is not None else None)
    try:
        return gate_command(args)
    except BenchGateError as error:
        print(f"error: bench-gate: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
