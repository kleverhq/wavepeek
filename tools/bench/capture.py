#!/usr/bin/env python3

from __future__ import annotations

import argparse
import dataclasses
import pathlib
import sys
import tomllib
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
    enforce_clean_worktree,
    ensure_empty_dir,
    make_default_output_dir,
    read_json,
    relative_to,
    resolve_ref,
    run_command,
    run_output,
    tool_version,
    utc_now,
    write_json,
)


@dataclasses.dataclass
class CaptureSession:
    tooling_root: pathlib.Path
    tooling_sha: str
    binary_checkout: pathlib.Path
    capture_dir: pathlib.Path
    source_ref: str
    source_sha: str
    fsdb_mode: str
    fsdb_plan: FsdbPlan
    environment_note: str
    binary_label: str
    logs_dir: pathlib.Path
    commands: list[CommandResult]
    suites: dict[str, dict[str, Any]]
    strict: bool = False
    record_inputs: bool = False


def support_files_missing(tooling_root: pathlib.Path) -> list[str]:
    required = [
        "tools/fsdb/check_fsdb_env.py",
        "bench/e2e/generate_catalog.py",
        "bench/e2e/tests.json",
        "tools/fsdb/prepare_fsdb_fixtures.sh",
        "tools/fsdb/check_fsdb_bench_artifacts.py",
    ]
    return [path for path in required if not (tooling_root / path).is_file()]


def source_supports_feature(binary_checkout: pathlib.Path, feature: str) -> bool:
    cargo_toml = binary_checkout / "Cargo.toml"
    try:
        payload = tomllib.loads(cargo_toml.read_text(encoding="utf-8"))
    except (FileNotFoundError, tomllib.TOMLDecodeError) as error:
        raise BenchGateError(f"cannot read Cargo.toml for {feature.upper()} support check: {cargo_toml}: {error}") from error
    features = payload.get("features")
    return isinstance(features, dict) and feature in features


def assess_fsdb(
    binary_checkout: pathlib.Path,
    *,
    tooling_root: pathlib.Path,
    log_path: pathlib.Path,
    mode: str,
) -> FsdbPlan:
    if mode == "never":
        return FsdbPlan(capture=False, status="skipped", reason="FSDB disabled by --fsdb=never")

    missing = support_files_missing(tooling_root)
    if missing:
        reason = "current FSDB support files missing: " + ", ".join(missing)
        if mode == "always":
            raise BenchGateError(reason)
        return FsdbPlan(capture=False, status="unsupported", reason=reason)

    if not source_supports_feature(binary_checkout, "fsdb"):
        reason = f"binary source ref does not define the fsdb feature: {binary_checkout}"
        if mode == "always":
            raise BenchGateError(reason)
        return FsdbPlan(capture=False, status="unsupported", reason=reason)

    result = run_command(
        "check-fsdb-env",
        ["python3", "-B", "tools/fsdb/check_fsdb_env.py"],
        cwd=tooling_root,
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
        reasons = [reason for reason in (baseline.reason, revised.reason) if reason]
        detail = "; ".join(reasons) if reasons else "baseline and revised refs are unsupported"
        return FsdbPlan(
            capture=False,
            status="unsupported",
            reason="FSDB unsupported for both refs: " + detail,
        )
    raise BenchGateError(
        "FSDB support is asymmetric between baseline and revised refs while Verdi appears available; "
        "use --fsdb=never only if this release intentionally skips FSDB performance review"
    )


def capture_environment(*, tooling_root: pathlib.Path, binary_checkout: pathlib.Path) -> dict[str, Any]:
    return {
        "binary_build": {
            "cargo_version": tool_version(["cargo", "--version"], cwd=binary_checkout),
            "rustc_version": tool_version(["rustc", "--version"], cwd=binary_checkout),
        },
        "tooling": {
            "hyperfine_version": tool_version(["hyperfine", "--version"], cwd=tooling_root),
            "uname": tool_version(["uname", "-a"], cwd=tooling_root),
        },
    }


def render_capture_readme(manifest: Mapping[str, Any]) -> str:
    lines = [
        f"# Wavepeek Benchmark Capture: {manifest.get('binary_ref', manifest.get('source_ref', '<unknown>'))}",
        "",
        f"Binary SHA: `{manifest.get('binary_sha', manifest.get('source_sha', '<unknown>'))}`",
        f"Tooling SHA: `{manifest.get('tooling_sha', '<unknown>')}`",
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
    tooling_root: pathlib.Path,
    tooling_sha: str,
    binary_checkout: pathlib.Path,
    capture_dir: pathlib.Path,
    source_ref: str,
    source_sha: str,
    fsdb_mode: str,
    fsdb_plan: FsdbPlan,
    environment_note: str,
    binary_label: str = "subject",
    strict: bool = False,
) -> CaptureSession:
    ensure_empty_dir(capture_dir)
    logs_dir = capture_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    return CaptureSession(
        tooling_root=tooling_root,
        tooling_sha=tooling_sha,
        binary_checkout=binary_checkout,
        capture_dir=capture_dir,
        source_ref=source_ref,
        source_sha=source_sha,
        fsdb_mode=fsdb_mode,
        fsdb_plan=fsdb_plan,
        environment_note=environment_note,
        binary_label=binary_label,
        logs_dir=logs_dir,
        commands=[],
        suites={},
        strict=strict,
    )


def build_release(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "build-release",
            ["cargo", "build", "--release", "--locked"],
            cwd=session.binary_checkout,
            log_path=session.logs_dir / "build-release.log",
        )
    )
    session.commands.append(
        run_command(
            "prepare-waveform-fixtures",
            ["just", "prepare-waveform-fixtures"],
            cwd=session.tooling_root,
            log_path=session.logs_dir / "prepare-waveform-fixtures.log",
        )
    )


def build_release_fsdb(session: CaptureSession) -> None:
    if not session.fsdb_plan.capture:
        return
    session.commands.append(
        run_command(
            "build-release-fsdb",
            ["cargo", "build", "--release", "--locked", "--features", "fsdb"],
            cwd=session.binary_checkout,
            log_path=session.logs_dir / "build-release-fsdb.log",
            env={"CARGO_TARGET_DIR": "target/fsdb"},
        )
    )


def prepare_fsdb(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "fsdb-prepare-fixtures",
            ["just", "prepare-and-check-fsdb-ondas-fixtures"],
            cwd=session.tooling_root,
            log_path=session.logs_dir / "fsdb-prepare-fixtures.log",
        )
    )


def build_release_reader(
    session: CaptureSession, feature: str, command_prefix: Sequence[str] = (),
) -> None:
    session.commands.append(
        run_command(
            f"build-release-{feature}",
            [*command_prefix, "cargo", "build", "--release", "--locked", "--features", feature],
            cwd=session.binary_checkout,
            log_path=session.logs_dir / f"build-release-{feature}.log",
            env={"CARGO_TARGET_DIR": f"target/{feature}"},
        )
    )


def prepare_wlf(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "wlf-prepare-fixtures",
            ["just", "prepare-wlf-fixtures"],
            cwd=session.tooling_root,
            log_path=session.logs_dir / "wlf-prepare-fixtures.log",
        )
    )


def build_release_shm(session: CaptureSession) -> None:
    build_release_reader(session, "shm", ("bash", str(session.tooling_root / "tools/shm/with_shm_env.sh")))


def prepare_shm(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "shm-prepare-fixtures",
            ["just", "prepare-shm-fixtures"],
            cwd=session.tooling_root,
            log_path=session.logs_dir / "shm-prepare-fixtures.log",
        )
    )


def prepare_vwdb(session: CaptureSession) -> pathlib.Path:
    result = run_output(["just", "prepare-vwdb-bench-fixtures"], cwd=session.tooling_root)
    return pathlib.Path(result.stdout.strip()) / "extract_global_include/ida.db/ida.vwdb"


def run_e2e_many(
    *,
    format_name: str,
    sessions: Sequence[CaptureSession],
    run_dir: pathlib.Path,
    log_path: pathlib.Path,
    binary_path: str,
    command_prefix: Sequence[str] = (),
    recording: pathlib.Path | None = None,
) -> None:
    if not sessions:
        return
    first = sessions[0]
    name = f"e2e-{format_name}"
    args = [
        *command_prefix,
        "python3",
        "-B",
        "bench/e2e/perf.py",
        "run",
        "--run-dir",
        str(run_dir),
    ]
    if format_name != "fst":
        generate = ["just", "generate-bench-e2e-catalog", format_name]
        if recording is not None:
            generate.append(str(recording))
        catalog = run_output(generate, cwd=first.tooling_root)
        args.extend(["--tests", catalog.stdout.strip()])
    if first.strict:
        args.append("--strict")
    elif first.record_inputs:
        args.append("--record-inputs")
    for session in sessions:
        args.extend(
            [
                "--binary",
                f"{session.binary_label}={session.binary_checkout / binary_path}",
            ]
        )
    result = run_command(
        f"bench-{name}",
        args,
        cwd=first.tooling_root,
        log_path=log_path,
        check=not first.strict,
    )
    completed = set()
    run_manifest = run_dir / "manifest.json"
    if result.returncode and run_manifest.is_file():
        report = read_json(run_manifest)
        count = report["test_count"]
        completed = {
            binary["label"]
            for binary in report["binaries"]
            if count == binary["wavepeek_json_count"] == binary["hyperfine_json_count"]
            and not binary["failure_count"]
        }
        if len(completed) == len(sessions):
            completed.clear()
    for session in sessions:
        session.commands.append(result)
        failed = result.returncode and session.binary_label not in completed
        session.suites[name] = {
            "status": "failed" if failed else "passed",
            "path": relative_to(run_dir / session.binary_label, session.capture_dir),
        }
        if failed:
            session.suites[name]["reason"] = f"benchmark scenario or input failed; see {log_path}"


def prepare_vcd(session: CaptureSession) -> None:
    session.commands.append(
        run_command(
            "vcd-prepare-fixtures",
            ["just", "prepare-vcd-ondas-fixtures"],
            cwd=session.tooling_root,
            log_path=session.logs_dir / "vcd-prepare-fixtures.log",
        )
    )


def run_e2e_vcd_many(
    sessions: Sequence[CaptureSession],
    *,
    run_dir: pathlib.Path,
    log_path: pathlib.Path,
) -> None:
    run_e2e_many(
        format_name="vcd",
        sessions=sessions,
        run_dir=run_dir,
        log_path=log_path,
        binary_path="target/release/wavepeek",
    )


def run_e2e_vcd(session: CaptureSession) -> None:
    run_e2e_vcd_many(
        [session],
        run_dir=session.capture_dir / "e2e-vcd",
        log_path=session.logs_dir / "bench-e2e-vcd.log",
    )


def run_e2e_fst_many(
    sessions: Sequence[CaptureSession],
    *,
    run_dir: pathlib.Path,
    log_path: pathlib.Path,
) -> None:
    run_e2e_many(
        format_name="fst",
        sessions=sessions,
        run_dir=run_dir,
        log_path=log_path,
        binary_path="target/release/wavepeek",
    )


def run_e2e_fst(session: CaptureSession) -> None:
    run_e2e_fst_many(
        [session],
        run_dir=session.capture_dir / "e2e-fst",
        log_path=session.logs_dir / "bench-e2e-fst.log",
    )


def run_e2e_fsdb_many(
    sessions: Sequence[CaptureSession],
    *,
    run_dir: pathlib.Path,
    log_path: pathlib.Path,
) -> None:
    active_sessions = [session for session in sessions if session.fsdb_plan.capture]
    if not active_sessions:
        return
    run_e2e_many(
        format_name="fsdb",
        sessions=active_sessions,
        run_dir=run_dir,
        log_path=log_path,
        binary_path="target/fsdb/release/wavepeek",
    )


def run_e2e_shm_many(
    sessions: Sequence[CaptureSession],
    *,
    run_dir: pathlib.Path,
    log_path: pathlib.Path,
) -> None:
    root = sessions[0].tooling_root
    run_e2e_many(
        format_name="shm",
        sessions=sessions,
        run_dir=run_dir,
        log_path=log_path,
        binary_path="target/shm/release/wavepeek",
        command_prefix=("bash", str(root / "tools/shm/with_shm_env.sh")),
    )


def run_e2e_wlf_many(
    sessions: Sequence[CaptureSession],
    *,
    run_dir: pathlib.Path,
    log_path: pathlib.Path,
) -> None:
    run_e2e_many(
        format_name="wlf",
        sessions=sessions,
        run_dir=run_dir,
        log_path=log_path,
        binary_path="target/wlf/release/wavepeek",
    )


def run_e2e_fsdb(session: CaptureSession) -> None:
    if not session.fsdb_plan.capture:
        return
    run_e2e_fsdb_many(
        [session],
        run_dir=session.capture_dir / "e2e-fsdb",
        log_path=session.logs_dir / "bench-e2e-fsdb.log",
    )


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
        "binary_ref": session.source_ref,
        "binary_sha": session.source_sha,
        "binary_checkout_path": str(session.binary_checkout),
        "tooling_sha": session.tooling_sha,
        "tooling_root": str(session.tooling_root),
        "fsdb_mode": session.fsdb_mode,
        "environment_note": session.environment_note,
        "environment": capture_environment(
            tooling_root=session.tooling_root,
            binary_checkout=session.binary_checkout,
        ),
        "suites": session.suites,
        "commands": [command_to_manifest(command, session.capture_dir) for command in session.commands],
    }
    write_json(session.capture_dir / "manifest.json", manifest)
    (session.capture_dir / "README.md").write_text(render_capture_readme(manifest), encoding="utf-8")
    return CaptureResult(capture_dir=session.capture_dir, manifest=manifest)


def capture_checkout(
    *,
    tooling_root: pathlib.Path,
    tooling_sha: str,
    binary_checkout: pathlib.Path,
    capture_dir: pathlib.Path,
    source_ref: str,
    source_sha: str,
    fsdb_mode: str,
    fsdb_plan: FsdbPlan | None = None,
    vcd_mode: str = "never",
    shm_mode: str = "never",
    wlf_mode: str = "never",
    environment_note: str,
) -> CaptureResult:
    ensure_empty_dir(capture_dir)
    logs_dir = capture_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    effective_fsdb_plan = fsdb_plan or assess_fsdb(
        binary_checkout,
        tooling_root=tooling_root,
        log_path=logs_dir / "fsdb-check-env.log",
        mode=fsdb_mode,
    )
    session = CaptureSession(
        tooling_root=tooling_root,
        tooling_sha=tooling_sha,
        binary_checkout=binary_checkout,
        capture_dir=capture_dir,
        source_ref=source_ref,
        source_sha=source_sha,
        fsdb_mode=fsdb_mode,
        fsdb_plan=effective_fsdb_plan,
        environment_note=environment_note,
        binary_label="subject",
        logs_dir=logs_dir,
        commands=[],
        suites={},
        record_inputs=True,
    )
    build_release(session)
    if vcd_mode == "always":
        prepare_vcd(session)
    if effective_fsdb_plan.capture:
        build_release_fsdb(session)
        prepare_fsdb(session)
    if shm_mode == "always":
        build_release_shm(session)
        prepare_shm(session)
    if wlf_mode == "always":
        build_release_reader(session, "wlf")
        prepare_wlf(session)
    run_e2e_fst(session)
    if vcd_mode == "always":
        run_e2e_vcd(session)
    if effective_fsdb_plan.capture:
        run_e2e_fsdb(session)
    if shm_mode == "always":
        run_e2e_shm_many(
            [session],
            run_dir=capture_dir / "e2e-shm",
            log_path=logs_dir / "bench-e2e-shm.log",
        )
    if wlf_mode == "always":
        run_e2e_wlf_many(
            [session],
            run_dir=capture_dir / "e2e-wlf",
            log_path=logs_dir / "bench-e2e-wlf.log",
        )
    return finalize_capture(session)


def capture_ref(args: argparse.Namespace) -> int:
    source_root = args.source_root.resolve()
    source_sha = resolve_ref(source_root, args.ref)
    tooling_sha = resolve_ref(source_root, "HEAD")
    enforce_clean_worktree(
        source_root,
        reason="current benchmark tooling must be committed before capture",
    )
    out_dir = args.out_dir or make_default_output_dir("captures", source_sha[:12])
    out_dir = out_dir.resolve()
    ensure_empty_dir(out_dir)
    checkout = out_dir / "checkout"
    clone_checkout(source_root, source_sha, checkout)
    capture_checkout(
        tooling_root=source_root,
        tooling_sha=tooling_sha,
        binary_checkout=checkout,
        capture_dir=out_dir / "run",
        source_ref=args.ref,
        source_sha=source_sha,
        fsdb_mode=args.fsdb,
        vcd_mode=args.vcd,
        shm_mode=args.shm,
        wlf_mode=args.wlf,
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
    parser.add_argument("--vcd", choices=("always", "never"), default="never")
    parser.add_argument("--shm", choices=("always", "never"), default="never")
    parser.add_argument("--wlf", choices=("always", "never"), default="never")
    parser.add_argument(
        "--environment-note",
        default="wavepeek manual performance gate",
        help="note written into benchmark capture manifests",
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
