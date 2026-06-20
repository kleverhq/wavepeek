#!/usr/bin/env python3

from __future__ import annotations

import argparse
import dataclasses
import pathlib
import sys
from collections.abc import Mapping, Sequence
from typing import Any

from common import (
    CAPTURE_SCHEMA_VERSION,
    FSDB_SKIP_STATUS,
    REPO_ROOT,
    BenchGateError,
    CaptureResult,
    CommandResult,
    FsdbPlan,
    clone_checkout,
    command_to_manifest,
    enforce_clean_source_for_head_refs,
    ensure_empty_dir,
    make_default_output_dir,
    read_json,
    relative_to,
    resolve_ref,
    run_command,
    tool_version,
    utc_now,
    write_json,
)


@dataclasses.dataclass
class CaptureSession:
    checkout: pathlib.Path
    capture_dir: pathlib.Path
    source_ref: str
    source_sha: str
    fsdb_mode: str
    fsdb_plan: FsdbPlan
    environment_note: str
    logs_dir: pathlib.Path
    commands: list[CommandResult]
    suites: dict[str, dict[str, Any]]
    fsdb_catalog: pathlib.Path | None = None
    fsdb_skipped_tests: list[str] = dataclasses.field(default_factory=list)


def support_files_missing(checkout: pathlib.Path) -> list[str]:
    required = [
        "tools/fsdb/check_fsdb_env.py",
        "tools/fsdb/generate_bench_catalog.py",
        "tools/fsdb/prepare_fsdb_fixtures.sh",
        "tools/fsdb/check_fsdb_bench_artifacts.py",
        "bench/e2e/tests_fsdb.json",
    ]
    return [path for path in required if not (checkout / path).is_file()]


def assess_fsdb(
    checkout: pathlib.Path,
    *,
    log_path: pathlib.Path,
    mode: str,
) -> FsdbPlan:
    if mode == "never":
        return FsdbPlan(capture=False, status="skipped", reason="FSDB disabled by --fsdb=never")

    missing = support_files_missing(checkout)
    if missing:
        reason = "FSDB support files missing: " + ", ".join(missing)
        if mode == "always":
            raise BenchGateError(reason)
        return FsdbPlan(capture=False, status="unsupported", reason=reason)

    result = run_command(
        "check-fsdb-env",
        ["python3", "-B", "tools/fsdb/check_fsdb_env.py"],
        cwd=checkout,
        log_path=log_path,
        check=False,
    )
    if result.returncode == 0:
        return FsdbPlan(capture=True, status="available")
    if result.returncode == FSDB_SKIP_STATUS and mode == "auto":
        return FsdbPlan(
            capture=False,
            status="skipped",
            reason="Verdi FSDB Reader SDK unavailable",
        )
    raise BenchGateError(f"FSDB environment check failed; see {log_path}")


def resolve_gate_fsdb_plan(
    baseline: FsdbPlan,
    revised: FsdbPlan,
    *,
    mode: str,
) -> FsdbPlan:
    if mode == "never":
        return FsdbPlan(capture=False, status="skipped", reason="FSDB disabled by --fsdb=never")
    if baseline.capture and revised.capture:
        return FsdbPlan(capture=True, status="available")
    if mode == "always":
        reasons = [reason for reason in (baseline.reason, revised.reason) if reason]
        raise BenchGateError("FSDB requested with --fsdb=always but cannot be captured: " + "; ".join(reasons))
    if baseline.status == "skipped" or revised.status == "skipped":
        return FsdbPlan(
            capture=False,
            status="skipped",
            reason="Verdi FSDB Reader SDK unavailable",
        )
    if baseline.status == "unsupported" and revised.status == "unsupported":
        return FsdbPlan(
            capture=False,
            status="unsupported",
            reason="FSDB support files missing in both refs",
        )
    raise BenchGateError(
        "FSDB support is asymmetric between baseline and revised refs while Verdi appears available; "
        "use --fsdb=never only if this release intentionally skips FSDB performance review"
    )


def fsdb_catalog_test_is_supported(test: Mapping[str, Any]) -> bool:
    command = test.get("command")
    if not isinstance(command, list):
        return True
    # Converted RTL FSDB fixtures expose packed arrays as names such as
    # `foo[0] [31:0]`, while the FST/VCD-derived benchmark catalog uses
    # `foo.[0]`. Current wavepeek releases cannot resolve those VCD-style
    # scalar element names in FSDB dumps, so gate FSDB capture omits those
    # scenarios and records the skip list in the capture manifest.
    return not any(isinstance(token, str) and ".[" in token for token in command)


def write_runnable_fsdb_catalog(
    *,
    checkout: pathlib.Path,
    capture_dir: pathlib.Path,
) -> tuple[pathlib.Path, list[str]]:
    source = checkout / "bench/e2e/tests_fsdb.json"
    payload = read_json(source)
    tests = payload.get("tests")
    if not isinstance(tests, list):
        raise BenchGateError(f"FSDB benchmark catalog must contain a tests array: {source}")

    kept: list[Any] = []
    skipped: list[str] = []
    for test in tests:
        if not isinstance(test, dict):
            kept.append(test)
            continue
        if fsdb_catalog_test_is_supported(test):
            kept.append(test)
        else:
            skipped.append(str(test.get("name", "<unnamed>")))

    if not kept:
        raise BenchGateError(f"FSDB benchmark catalog has no runnable tests after filtering: {source}")

    generated = dict(payload)
    generated["tests"] = kept
    output = capture_dir / "generated" / "tests_fsdb.runnable.json"
    write_json(output, generated)
    return output, skipped


def capture_environment(checkout: pathlib.Path) -> dict[str, Any]:
    return {
        "cargo_version": tool_version(["cargo", "--version"], cwd=checkout),
        "rustc_version": tool_version(["rustc", "--version"], cwd=checkout),
        "hyperfine_version": tool_version(["hyperfine", "--version"], cwd=checkout),
        "uname": tool_version(["uname", "-a"], cwd=checkout),
    }


def render_capture_readme(manifest: Mapping[str, Any]) -> str:
    lines = [
        f"# Wavepeek Benchmark Capture: {manifest.get('source_ref', '<unknown>')}",
        "",
        f"Source SHA: `{manifest.get('source_sha', '<unknown>')}`",
        f"Generated: `{manifest.get('generated_at_utc', '<unknown>')}`",
        "",
        "## Suites",
        "",
    ]
    suites = manifest.get("suites", {})
    if isinstance(suites, Mapping):
        for name in sorted(suites):
            suite = suites[name]
            if not isinstance(suite, Mapping):
                continue
            status = suite.get("status", "unknown")
            reason = suite.get("reason")
            path = suite.get("path")
            text = f"- `{name}`: {status}"
            if path:
                text += f" (`{path}`)"
            if reason:
                text += f" — {reason}"
            lines.append(text)
    lines.append("")
    return "\n".join(lines)


def init_capture_session(
    *,
    checkout: pathlib.Path,
    capture_dir: pathlib.Path,
    source_ref: str,
    source_sha: str,
    fsdb_mode: str,
    fsdb_plan: FsdbPlan,
    environment_note: str,
) -> CaptureSession:
    ensure_empty_dir(capture_dir)
    logs_dir = capture_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    return CaptureSession(
        checkout=checkout,
        capture_dir=capture_dir,
        source_ref=source_ref,
        source_sha=source_sha,
        fsdb_mode=fsdb_mode,
        fsdb_plan=fsdb_plan,
        environment_note=environment_note,
        logs_dir=logs_dir,
        commands=[],
        suites={},
    )


def build_release(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "build-release",
            ["cargo", "build", "--release"],
            cwd=session.checkout,
            log_path=session.logs_dir / "build-release.log",
        )
    )


def build_release_fsdb(session: CaptureSession) -> None:
    if not session.fsdb_plan.capture:
        return
    session.commands.append(
        run_command(
            "build-release-fsdb",
            ["cargo", "build", "--release", "--features", "fsdb"],
            cwd=session.checkout,
            log_path=session.logs_dir / "build-release-fsdb.log",
            env={"CARGO_TARGET_DIR": "target/fsdb"},
        )
    )


def prepare_fsdb(session: CaptureSession) -> None:
    if not session.fsdb_plan.capture:
        return
    session.commands.append(
        run_command(
            "fsdb-check-env-required",
            ["python3", "-B", "tools/fsdb/check_fsdb_env.py", "--require"],
            cwd=session.checkout,
            log_path=session.logs_dir / "fsdb-check-env-required.log",
        )
    )
    session.commands.append(
        run_command(
            "fsdb-check-catalog",
            ["python3", "-B", "tools/fsdb/generate_bench_catalog.py", "--check"],
            cwd=session.checkout,
            log_path=session.logs_dir / "fsdb-check-catalog.log",
        )
    )
    session.commands.append(
        run_command(
            "fsdb-prepare-fixtures",
            ["bash", "tools/fsdb/prepare_fsdb_fixtures.sh"],
            cwd=session.checkout,
            log_path=session.logs_dir / "fsdb-prepare-fixtures.log",
        )
    )
    session.fsdb_catalog, session.fsdb_skipped_tests = write_runnable_fsdb_catalog(
        checkout=session.checkout,
        capture_dir=session.capture_dir,
    )
    session.commands.append(
        run_command(
            "fsdb-check-artifacts",
            ["python3", "-B", "tools/fsdb/check_fsdb_bench_artifacts.py", str(session.fsdb_catalog)],
            cwd=session.checkout,
            log_path=session.logs_dir / "fsdb-check-artifacts.log",
        )
    )


def run_e2e_fst(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "bench-e2e-fst",
            [
                "python3",
                "-B",
                "bench/e2e/perf.py",
                "run",
                "--run-dir",
                str(session.capture_dir / "e2e-fst"),
            ],
            cwd=session.checkout,
            log_path=session.logs_dir / "bench-e2e-fst.log",
            env={"WAVEPEEK_BIN": str(session.checkout / "target/release/wavepeek")},
        )
    )
    session.suites["e2e-fst"] = {"status": "passed", "path": "e2e-fst"}


def run_e2e_fsdb(session: CaptureSession) -> None:
    if not session.fsdb_plan.capture:
        return
    if session.fsdb_catalog is None:
        raise BenchGateError("FSDB runnable catalog missing; call prepare_fsdb before run_e2e_fsdb")
    session.commands.append(
        run_command(
            "bench-e2e-fsdb",
            [
                "python3",
                "-B",
                "bench/e2e/perf.py",
                "run",
                "--tests",
                str(session.fsdb_catalog),
                "--run-dir",
                str(session.capture_dir / "e2e-fsdb"),
            ],
            cwd=session.checkout,
            log_path=session.logs_dir / "bench-e2e-fsdb.log",
            env={"WAVEPEEK_BIN": str(session.checkout / "target/fsdb/release/wavepeek")},
        )
    )
    session.suites["e2e-fsdb"] = {
        "status": "passed",
        "path": "e2e-fsdb",
        "filtered_catalog_path": relative_to(session.fsdb_catalog, session.capture_dir),
        "skipped_tests": session.fsdb_skipped_tests,
        "skipped_test_count": len(session.fsdb_skipped_tests),
    }


def run_expr(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "bench-expr",
            [
                "python3",
                "-B",
                "bench/expr/perf.py",
                "run",
                "--run-dir",
                str(session.capture_dir / "expr"),
                "--environment-note",
                session.environment_note,
            ],
            cwd=session.checkout,
            log_path=session.logs_dir / "bench-expr.log",
        )
    )
    session.suites["expr"] = {"status": "passed", "path": "expr"}


def finalize_capture(session: CaptureSession) -> CaptureResult:
    if "e2e-fsdb" not in session.suites:
        session.suites["e2e-fsdb"] = {
            "status": session.fsdb_plan.status,
            "reason": session.fsdb_plan.reason,
        }

    manifest: dict[str, Any] = {
        "schema_version": CAPTURE_SCHEMA_VERSION,
        "kind": "wavepeek-bench-capture",
        "generated_at_utc": utc_now().isoformat().replace("+00:00", "Z"),
        "source_ref": session.source_ref,
        "source_sha": session.source_sha,
        "checkout_path": str(session.checkout),
        "fsdb_mode": session.fsdb_mode,
        "environment_note": session.environment_note,
        "environment": capture_environment(session.checkout),
        "suites": session.suites,
        "commands": [command_to_manifest(command, session.capture_dir) for command in session.commands],
    }
    write_json(session.capture_dir / "manifest.json", manifest)
    (session.capture_dir / "README.md").write_text(render_capture_readme(manifest), encoding="utf-8")
    return CaptureResult(capture_dir=session.capture_dir, manifest=manifest)


def capture_checkout(
    *,
    checkout: pathlib.Path,
    capture_dir: pathlib.Path,
    source_ref: str,
    source_sha: str,
    fsdb_mode: str,
    fsdb_plan: FsdbPlan | None = None,
    environment_note: str,
) -> CaptureResult:
    ensure_empty_dir(capture_dir)
    logs_dir = capture_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    effective_fsdb_plan = fsdb_plan or assess_fsdb(
        checkout,
        log_path=logs_dir / "fsdb-check-env.log",
        mode=fsdb_mode,
    )
    session = CaptureSession(
        checkout=checkout,
        capture_dir=capture_dir,
        source_ref=source_ref,
        source_sha=source_sha,
        fsdb_mode=fsdb_mode,
        fsdb_plan=effective_fsdb_plan,
        environment_note=environment_note,
        logs_dir=logs_dir,
        commands=[],
        suites={},
    )
    build_release(session)
    if effective_fsdb_plan.capture:
        build_release_fsdb(session)
        prepare_fsdb(session)
    run_e2e_fst(session)
    if effective_fsdb_plan.capture:
        run_e2e_fsdb(session)
    run_expr(session)
    return finalize_capture(session)


def capture_ref(args: argparse.Namespace) -> int:
    source_root = args.source_root.resolve()
    source_sha = resolve_ref(source_root, args.ref)
    enforce_clean_source_for_head_refs(source_root, [args.ref])
    out_dir = args.out_dir or make_default_output_dir("captures", source_sha[:12])
    out_dir = out_dir.resolve()
    ensure_empty_dir(out_dir)
    checkout = out_dir / "checkout"
    clone_checkout(source_root, source_sha, checkout)
    capture_checkout(
        checkout=checkout,
        capture_dir=out_dir / "run",
        source_ref=args.ref,
        source_sha=source_sha,
        fsdb_mode=args.fsdb,
        environment_note=args.environment_note,
    )
    print(f"capture written to {out_dir / 'run'}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Capture wavepeek benchmark artifacts for one source ref")
    parser.add_argument("--ref", default="HEAD", help="source ref to capture")
    parser.add_argument("--source-root", type=pathlib.Path, default=REPO_ROOT)
    parser.add_argument("--out-dir", type=pathlib.Path)
    parser.add_argument("--fsdb", choices=("auto", "always", "never"), default="auto")
    parser.add_argument(
        "--environment-note",
        default="wavepeek manual performance gate",
        help="note written into expression benchmark summaries",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(list(argv) if argv is not None else None)
    try:
        return capture_ref(args)
    except BenchGateError as error:
        print(f"error: bench-capture: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
