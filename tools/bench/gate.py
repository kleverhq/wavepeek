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
    finalize_capture,
    init_capture_session,
    prepare_fsdb,
    resolve_gate_fsdb_plan,
    write_fsdb_capture_catalog,
    run_e2e_fsdb_many,
    run_e2e_fst_many,
)
from common import (
    DEFAULT_TIMING_THRESHOLD_PCT,
    DEFAULT_TIMING_THRESHOLD_SECONDS,
    GATE_SCHEMA_VERSION,
    REPO_ROOT,
    BenchGateError,
    clone_checkout,
    enforce_clean_worktree,
    ensure_empty_dir,
    make_default_output_dir,
    resolve_ref,
    utc_now,
    write_json,
)
from compare import compare_captures


def render_gate_summary(manifest: Mapping[str, Any]) -> str:
    compare_status = manifest.get("compare", {}).get("status", "unknown") if isinstance(manifest.get("compare"), Mapping) else "unknown"
    lines = [
        "# Wavepeek Manual Performance Gate",
        "",
        f"Baseline ref: `{manifest.get('baseline_ref', '<unknown>')}`",
        f"Baseline SHA: `{manifest.get('baseline_sha', '<unknown>')}`",
        f"Revised ref: `{manifest.get('revised_ref', '<unknown>')}`",
        f"Revised SHA: `{manifest.get('revised_sha', '<unknown>')}`",
        f"Comparison status: **{compare_status}**",
        "",
        "See `baseline/`, `revised/`, and `compare/` for captured artifacts and logs.",
        "",
    ]
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
    checkouts_dir = out_dir / "checkouts"
    checkouts_dir.mkdir(parents=True, exist_ok=True)
    baseline_checkout = checkouts_dir / "baseline"
    revised_checkout = checkouts_dir / "revised"
    clone_checkout(source_root, baseline_sha, baseline_checkout)
    clone_checkout(source_root, revised_sha, revised_checkout)

    preflight_dir = out_dir / "logs"
    baseline_fsdb = assess_fsdb(
        baseline_checkout,
        tooling_root=source_root,
        log_path=preflight_dir / "baseline-fsdb-check-env.log",
        mode=args.fsdb,
    )
    revised_fsdb = assess_fsdb(
        revised_checkout,
        tooling_root=source_root,
        log_path=preflight_dir / "revised-fsdb-check-env.log",
        mode=args.fsdb,
    )
    gate_fsdb_plan = resolve_gate_fsdb_plan(baseline_fsdb, revised_fsdb, mode=args.fsdb)

    baseline = init_capture_session(
        tooling_root=source_root,
        tooling_sha=tooling_sha,
        binary_checkout=baseline_checkout,
        capture_dir=out_dir / "baseline",
        source_ref=args.baseline_ref,
        source_sha=baseline_sha,
        fsdb_mode=args.fsdb,
        fsdb_plan=gate_fsdb_plan,
        environment_note=args.environment_note,
        binary_label="baseline",
    )
    revised = init_capture_session(
        tooling_root=source_root,
        tooling_sha=tooling_sha,
        binary_checkout=revised_checkout,
        capture_dir=out_dir / "revised",
        source_ref=args.revised_ref,
        source_sha=revised_sha,
        fsdb_mode=args.fsdb,
        fsdb_plan=gate_fsdb_plan,
        environment_note=args.environment_note,
        binary_label="revised",
    )

    # Keep preparation work out of the measured section. Build both refs first,
    # then prepare current FSDB fixtures when needed, then run each measured
    # suite for baseline and revised in same-format pairs before comparing artifacts.
    build_release(baseline)
    build_release(revised)
    if gate_fsdb_plan.capture:
        build_release_fsdb(baseline)
        build_release_fsdb(revised)
        prepare_fsdb(baseline)
        write_fsdb_capture_catalog(revised)

    run_e2e_fst_many(
        [baseline, revised],
        run_dir=out_dir / "e2e-fst",
        log_path=out_dir / "logs" / "bench-e2e-fst.log",
    )
    if gate_fsdb_plan.capture:
        run_e2e_fsdb_many(
            [baseline, revised],
            run_dir=out_dir / "e2e-fsdb",
            log_path=out_dir / "logs" / "bench-e2e-fsdb.log",
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
    )

    status = "passed" if compare.exit_code == 0 else "failed"
    manifest: dict[str, Any] = {
        "schema_version": GATE_SCHEMA_VERSION,
        "kind": "wavepeek-bench-gate",
        "generated_at_utc": utc_now().isoformat().replace("+00:00", "Z"),
        "baseline_ref": args.baseline_ref,
        "baseline_sha": baseline_sha,
        "revised_ref": args.revised_ref,
        "revised_sha": revised_sha,
        "source_root": str(source_root),
        "tooling_sha": tooling_sha,
        "fsdb_mode": args.fsdb,
        "fsdb_plan": dataclasses.asdict(gate_fsdb_plan),
        "timing_threshold_pct": args.max_negative_delta_pct,
        "timing_threshold_seconds": args.max_negative_delta_seconds,
        "timing_metric": "median",
        "compare": {"status": status, "path": "compare"},
        "execution_order": [
            "build baseline release",
            "build revised release",
            "build baseline fsdb release if enabled",
            "build revised fsdb release if enabled",
            "prepare current fsdb fixtures if enabled",
            "write revised fsdb runnable catalog if enabled",
            "run FST e2e with labeled round-robin binaries",
            "run FSDB e2e with labeled round-robin binaries if enabled",
            "compare artifacts",
            "confirm same-format timing outliers with best samples if needed",
        ],
    }
    write_json(out_dir / "manifest.json", manifest)
    (out_dir / "summary.md").write_text(render_gate_summary(manifest), encoding="utf-8")
    print(f"gate written to {out_dir}")
    return compare.exit_code


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run the manual wavepeek performance gate")
    parser.add_argument("--baseline-ref", required=True)
    parser.add_argument("--revised-ref", default="HEAD")
    parser.add_argument("--source-root", type=pathlib.Path, default=REPO_ROOT)
    parser.add_argument("--out-dir", type=pathlib.Path)
    parser.add_argument("--fsdb", choices=("auto", "always", "never"), default="auto")
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
