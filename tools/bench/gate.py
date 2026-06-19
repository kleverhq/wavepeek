#!/usr/bin/env python3

from __future__ import annotations

import argparse
import dataclasses
import datetime as dt
import json
import os
import pathlib
import shutil
import subprocess
import sys
from collections.abc import Mapping, Sequence
from typing import Any

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
DEFAULT_OUTPUT_ROOT = pathlib.Path("tmp/bench-gate")
DEFAULT_E2E_THRESHOLD_PCT = 5.0
DEFAULT_EXPR_THRESHOLD_PCT = 15.0
FSDB_SKIP_STATUS = 77
CAPTURE_SCHEMA_VERSION = 1
COMPARE_SCHEMA_VERSION = 1
GATE_SCHEMA_VERSION = 1


class BenchGateError(Exception):
    pass


@dataclasses.dataclass(frozen=True)
class CommandResult:
    name: str
    args: list[str]
    cwd: str
    returncode: int
    log_path: str | None = None


@dataclasses.dataclass(frozen=True)
class FsdbPlan:
    capture: bool
    status: str
    reason: str | None = None


@dataclasses.dataclass(frozen=True)
class CaptureResult:
    capture_dir: pathlib.Path
    manifest: dict[str, Any]


@dataclasses.dataclass(frozen=True)
class CompareResult:
    compare_dir: pathlib.Path
    manifest: dict[str, Any]
    exit_code: int


def utc_now() -> dt.datetime:
    return dt.datetime.now(dt.UTC).replace(microsecond=0)


def utc_stamp() -> str:
    return utc_now().strftime("%Y%m%dT%H%M%SZ")


def json_default(value: Any) -> Any:
    if isinstance(value, pathlib.Path):
        return str(value)
    raise TypeError(f"object of type {type(value).__name__} is not JSON serializable")


def write_json(path: pathlib.Path, payload: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True, default=json_default)
        + "\n",
        encoding="utf-8",
    )


def read_json(path: pathlib.Path) -> dict[str, Any]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise BenchGateError(f"missing JSON file: {path}") from error
    except json.JSONDecodeError as error:
        raise BenchGateError(f"invalid JSON file {path}: {error}") from error
    if not isinstance(payload, dict):
        raise BenchGateError(f"JSON file must contain an object: {path}")
    return payload


def run_output(
    args: Sequence[str],
    *,
    cwd: pathlib.Path = REPO_ROOT,
    env: Mapping[str, str] | None = None,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    command_env = os.environ.copy()
    if env:
        command_env.update(env)
    result = subprocess.run(
        list(args),
        cwd=str(cwd),
        env=command_env,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and result.returncode != 0:
        message = (result.stderr or result.stdout).strip()
        command = " ".join(args)
        raise BenchGateError(f"command failed ({result.returncode}): {command}: {message}")
    return result


def run_command(
    name: str,
    args: Sequence[str],
    *,
    cwd: pathlib.Path,
    log_path: pathlib.Path,
    env: Mapping[str, str] | None = None,
    check: bool = True,
) -> CommandResult:
    log_path.parent.mkdir(parents=True, exist_ok=True)
    command_env = os.environ.copy()
    if env:
        command_env.update(env)

    with log_path.open("w", encoding="utf-8") as log_file:
        log_file.write(f"# name: {name}\n")
        log_file.write(f"# cwd: {cwd}\n")
        log_file.write("$ " + " ".join(args) + "\n\n")
        log_file.flush()
        process = subprocess.Popen(
            list(args),
            cwd=str(cwd),
            env=command_env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        assert process.stdout is not None
        for line in process.stdout:
            sys.stdout.write(line)
            log_file.write(line)
        returncode = process.wait()
        log_file.write(f"\n# exit: {returncode}\n")

    result = CommandResult(
        name=name,
        args=list(args),
        cwd=str(cwd),
        returncode=returncode,
        log_path=str(log_path),
    )
    if check and returncode != 0:
        raise BenchGateError(f"{name} failed with exit code {returncode}; see {log_path}")
    return result


def ensure_empty_dir(path: pathlib.Path) -> pathlib.Path:
    if path.exists() and not path.is_dir():
        raise BenchGateError(f"output path exists and is not a directory: {path}")
    if path.exists() and any(path.iterdir()):
        raise BenchGateError(f"output directory already exists and is not empty: {path}")
    path.mkdir(parents=True, exist_ok=True)
    return path


def make_default_output_dir(kind: str, label: str) -> pathlib.Path:
    safe_label = "".join(c if c.isalnum() or c in ".-_" else "-" for c in label).strip("-")
    if not safe_label:
        safe_label = "run"
    return REPO_ROOT / DEFAULT_OUTPUT_ROOT / kind / f"{utc_stamp()}-{safe_label}"


def resolve_ref(source_root: pathlib.Path, ref: str) -> str:
    result = run_output(["git", "rev-parse", "--verify", f"{ref}^{{commit}}"], cwd=source_root)
    return result.stdout.strip()


def current_head(source_root: pathlib.Path) -> str:
    return resolve_ref(source_root, "HEAD")


def worktree_status(source_root: pathlib.Path) -> str:
    return run_output(
        ["git", "status", "--short", "--untracked-files=all"],
        cwd=source_root,
    ).stdout.strip()


def enforce_clean_source_for_head_refs(
    source_root: pathlib.Path,
    refs: Sequence[tuple[str, str]],
    *,
    allow_dirty_source: bool,
) -> None:
    if allow_dirty_source:
        return
    head = current_head(source_root)
    needs_clean = any(ref == "HEAD" or sha == head for ref, sha in refs)
    if not needs_clean:
        return
    status = worktree_status(source_root)
    if status:
        raise BenchGateError(
            "source worktree is dirty but HEAD/current refs clone only committed state; "
            "commit or stash changes, or pass --allow-dirty-source to benchmark committed refs intentionally"
        )


def clone_checkout(source_root: pathlib.Path, sha: str, checkout_dir: pathlib.Path) -> None:
    checkout_dir.parent.mkdir(parents=True, exist_ok=True)
    if checkout_dir.exists():
        if any(checkout_dir.iterdir()):
            raise BenchGateError(f"checkout directory already exists and is not empty: {checkout_dir}")
        checkout_dir.rmdir()
    run_output(["git", "clone", "--no-checkout", str(source_root), str(checkout_dir)], cwd=source_root)
    run_output(["git", "checkout", "--detach", sha], cwd=checkout_dir)


def relative_to(path: pathlib.Path, base: pathlib.Path) -> str:
    try:
        return str(path.relative_to(base))
    except ValueError:
        return str(path)


def command_to_manifest(result: CommandResult, base: pathlib.Path) -> dict[str, Any]:
    return {
        "name": result.name,
        "args": result.args,
        "cwd": result.cwd,
        "returncode": result.returncode,
        "log_path": relative_to(pathlib.Path(result.log_path), base) if result.log_path else None,
    }


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


def tool_version(args: Sequence[str], *, cwd: pathlib.Path) -> str | None:
    result = run_output(args, cwd=cwd, check=False)
    if result.returncode != 0:
        return None
    return (result.stdout or result.stderr).strip()


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
            if suite.get("reason"):
                text += f" — {suite['reason']}"
            if suite.get("log_path"):
                text += f" (log: `{suite['log_path']}`)"
            lines.append(text)
    lines.append("")
    return "\n".join(lines)


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


def run_fsdb_capture(
    checkout: pathlib.Path,
    capture_dir: pathlib.Path,
    logs_dir: pathlib.Path,
    commands: list[CommandResult],
) -> None:
    commands.append(
        run_command(
            "fsdb-check-env-required",
            ["python3", "-B", "tools/fsdb/check_fsdb_env.py", "--require"],
            cwd=checkout,
            log_path=logs_dir / "fsdb-check-env-required.log",
        )
    )
    commands.append(
        run_command(
            "fsdb-check-catalog",
            ["python3", "-B", "tools/fsdb/generate_bench_catalog.py", "--check"],
            cwd=checkout,
            log_path=logs_dir / "fsdb-check-catalog.log",
        )
    )
    commands.append(
        run_command(
            "fsdb-prepare-fixtures",
            ["bash", "tools/fsdb/prepare_fsdb_fixtures.sh"],
            cwd=checkout,
            log_path=logs_dir / "fsdb-prepare-fixtures.log",
        )
    )
    commands.append(
        run_command(
            "fsdb-check-artifacts",
            ["python3", "-B", "tools/fsdb/check_fsdb_bench_artifacts.py", "bench/e2e/tests_fsdb.json"],
            cwd=checkout,
            log_path=logs_dir / "fsdb-check-artifacts.log",
        )
    )
    fsdb_env = {"CARGO_TARGET_DIR": "target/fsdb"}
    commands.append(
        run_command(
            "build-release-fsdb",
            ["cargo", "build", "--release", "--features", "fsdb"],
            cwd=checkout,
            log_path=logs_dir / "build-release-fsdb.log",
            env=fsdb_env,
        )
    )
    commands.append(
        run_command(
            "bench-e2e-fsdb",
            [
                "python3",
                "-B",
                "bench/e2e/perf.py",
                "run",
                "--tests",
                "bench/e2e/tests_fsdb.json",
                "--run-dir",
                str(capture_dir / "e2e-fsdb"),
            ],
            cwd=checkout,
            log_path=logs_dir / "bench-e2e-fsdb.log",
            env={"WAVEPEEK_BIN": str(checkout / "target/fsdb/release/wavepeek")},
        )
    )


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
    commands: list[CommandResult] = []
    suites: dict[str, dict[str, Any]] = {}
    effective_fsdb_plan = fsdb_plan or assess_fsdb(
        checkout,
        log_path=logs_dir / "fsdb-check-env.log",
        mode=fsdb_mode,
    )

    commands.append(
        run_command(
            "build-release",
            ["cargo", "build", "--release"],
            cwd=checkout,
            log_path=logs_dir / "build-release.log",
        )
    )
    commands.append(
        run_command(
            "bench-e2e-fst",
            [
                "python3",
                "-B",
                "bench/e2e/perf.py",
                "run",
                "--run-dir",
                str(capture_dir / "e2e-fst"),
            ],
            cwd=checkout,
            log_path=logs_dir / "bench-e2e-fst.log",
            env={"WAVEPEEK_BIN": str(checkout / "target/release/wavepeek")},
        )
    )
    suites["e2e-fst"] = {"status": "passed", "path": "e2e-fst"}

    commands.append(
        run_command(
            "bench-expr",
            [
                "python3",
                "-B",
                "bench/expr/perf.py",
                "run",
                "--run-dir",
                str(capture_dir / "expr"),
                "--environment-note",
                environment_note,
            ],
            cwd=checkout,
            log_path=logs_dir / "bench-expr.log",
        )
    )
    suites["expr"] = {"status": "passed", "path": "expr"}

    if effective_fsdb_plan.capture:
        run_fsdb_capture(checkout, capture_dir, logs_dir, commands)
        suites["e2e-fsdb"] = {"status": "passed", "path": "e2e-fsdb"}
    else:
        suites["e2e-fsdb"] = {
            "status": effective_fsdb_plan.status,
            "reason": effective_fsdb_plan.reason,
        }

    manifest: dict[str, Any] = {
        "schema_version": CAPTURE_SCHEMA_VERSION,
        "kind": "wavepeek-bench-capture",
        "generated_at_utc": utc_now().isoformat().replace("+00:00", "Z"),
        "source_ref": source_ref,
        "source_sha": source_sha,
        "checkout_path": str(checkout),
        "fsdb_mode": fsdb_mode,
        "environment_note": environment_note,
        "environment": capture_environment(checkout),
        "suites": suites,
        "commands": [command_to_manifest(command, capture_dir) for command in commands],
    }
    write_json(capture_dir / "manifest.json", manifest)
    (capture_dir / "README.md").write_text(render_capture_readme(manifest), encoding="utf-8")
    return CaptureResult(capture_dir=capture_dir, manifest=manifest)


def capture_ref(args: argparse.Namespace) -> int:
    source_root = args.source_root.resolve()
    source_sha = resolve_ref(source_root, args.ref)
    enforce_clean_source_for_head_refs(
        source_root,
        [(args.ref, source_sha)],
        allow_dirty_source=args.allow_dirty_source,
    )
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


def compare_suite(
    *,
    name: str,
    golden: pathlib.Path,
    revised: pathlib.Path,
    compare_dir: pathlib.Path,
    threshold_pct: float,
) -> dict[str, Any]:
    log_path = compare_dir / f"{name}.log"
    extra_args: list[str] = []
    if name.startswith("e2e-"):
        assert_matching_e2e_artifacts(golden, revised)
        script = REPO_ROOT / "bench/e2e/perf.py"
        extra_args.append("--verbose")
    elif name == "expr":
        script = REPO_ROOT / "bench/expr/perf.py"
    else:
        raise BenchGateError(f"unknown benchmark suite: {name}")

    compare_args = [
        "python3",
        "-B",
        str(script),
        "compare",
        "--revised",
        str(revised),
        "--golden",
        str(golden),
        "--max-negative-delta-pct",
        str(threshold_pct),
        *extra_args,
    ]
    result = run_command(
        f"compare-{name}",
        compare_args,
        cwd=REPO_ROOT,
        log_path=log_path,
        check=False,
    )
    status = "passed" if result.returncode == 0 else "failed"
    return {
        "status": status,
        "returncode": result.returncode,
        "threshold_pct": threshold_pct,
        "log_path": relative_to(log_path, compare_dir),
    }


def compare_captures(
    *,
    golden_dir: pathlib.Path,
    revised_dir: pathlib.Path,
    compare_dir: pathlib.Path,
    e2e_threshold_pct: float,
    fsdb_threshold_pct: float,
    expr_threshold_pct: float,
) -> CompareResult:
    if not golden_dir.is_dir():
        raise BenchGateError(f"golden capture directory does not exist: {golden_dir}")
    if not revised_dir.is_dir():
        raise BenchGateError(f"revised capture directory does not exist: {revised_dir}")
    ensure_empty_dir(compare_dir)
    suites: dict[str, dict[str, Any]] = {}

    suite_config = {
        "e2e-fst": {"threshold": e2e_threshold_pct, "required": True},
        "expr": {"threshold": expr_threshold_pct, "required": True},
        "e2e-fsdb": {"threshold": fsdb_threshold_pct, "required": False},
    }
    failures: list[str] = []
    for name, config in suite_config.items():
        threshold = float(config["threshold"])
        required = bool(config["required"])
        golden = golden_dir / name
        revised = revised_dir / name
        golden_exists = golden.is_dir()
        revised_exists = revised.is_dir()
        if not golden_exists and not revised_exists:
            status = "failed" if required else "skipped"
            reason = "required suite missing from both captures" if required else "optional suite missing from both captures"
            suites[name] = {"status": status, "reason": reason}
            if required:
                failures.append(name)
            continue
        if golden_exists != revised_exists:
            suites[name] = {
                "status": "failed",
                "reason": "suite exists in only one capture",
            }
            failures.append(name)
            continue
        try:
            suite_result = compare_suite(
                name=name,
                golden=golden,
                revised=revised,
                compare_dir=compare_dir,
                threshold_pct=threshold,
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
        e2e_threshold_pct=args.e2e_threshold_pct,
        fsdb_threshold_pct=args.fsdb_threshold_pct,
        expr_threshold_pct=args.expr_threshold_pct,
    )
    print(f"comparison written to {result.compare_dir}")
    return result.exit_code


def gate_command(args: argparse.Namespace) -> int:
    source_root = args.source_root.resolve()
    baseline_sha = resolve_ref(source_root, args.baseline_ref)
    revised_sha = resolve_ref(source_root, args.revised_ref)
    enforce_clean_source_for_head_refs(
        source_root,
        [(args.baseline_ref, baseline_sha), (args.revised_ref, revised_sha)],
        allow_dirty_source=args.allow_dirty_source,
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
        log_path=preflight_dir / "baseline-fsdb-check-env.log",
        mode=args.fsdb,
    )
    revised_fsdb = assess_fsdb(
        revised_checkout,
        log_path=preflight_dir / "revised-fsdb-check-env.log",
        mode=args.fsdb,
    )
    gate_fsdb_plan = resolve_gate_fsdb_plan(baseline_fsdb, revised_fsdb, mode=args.fsdb)

    capture_checkout(
        checkout=baseline_checkout,
        capture_dir=out_dir / "baseline",
        source_ref=args.baseline_ref,
        source_sha=baseline_sha,
        fsdb_mode=args.fsdb,
        fsdb_plan=gate_fsdb_plan,
        environment_note=args.environment_note,
    )
    capture_checkout(
        checkout=revised_checkout,
        capture_dir=out_dir / "revised",
        source_ref=args.revised_ref,
        source_sha=revised_sha,
        fsdb_mode=args.fsdb,
        fsdb_plan=gate_fsdb_plan,
        environment_note=args.environment_note,
    )
    compare = compare_captures(
        golden_dir=out_dir / "baseline",
        revised_dir=out_dir / "revised",
        compare_dir=out_dir / "compare",
        e2e_threshold_pct=args.e2e_threshold_pct,
        fsdb_threshold_pct=args.fsdb_threshold_pct,
        expr_threshold_pct=args.expr_threshold_pct,
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
        "fsdb_mode": args.fsdb,
        "fsdb_plan": dataclasses.asdict(gate_fsdb_plan),
        "compare": {"status": status, "path": "compare"},
    }
    write_json(out_dir / "manifest.json", manifest)
    (out_dir / "summary.md").write_text(render_gate_summary(manifest), encoding="utf-8")
    print(f"gate written to {out_dir}")
    return compare.exit_code


def add_common_threshold_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument(
        "--e2e-threshold-pct",
        type=float,
        default=DEFAULT_E2E_THRESHOLD_PCT,
        help="maximum allowed negative delta for FST e2e benchmarks",
    )
    parser.add_argument(
        "--fsdb-threshold-pct",
        type=float,
        default=DEFAULT_E2E_THRESHOLD_PCT,
        help="maximum allowed negative delta for FSDB e2e benchmarks",
    )
    parser.add_argument(
        "--expr-threshold-pct",
        type=float,
        default=DEFAULT_EXPR_THRESHOLD_PCT,
        help="maximum allowed negative delta for expression benchmarks",
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Manual wavepeek performance gate helper")
    subcommands = parser.add_subparsers(dest="command", required=True)

    capture = subcommands.add_parser("capture", help="capture benchmarks for one source ref")
    capture.add_argument("--ref", default="HEAD", help="source ref to capture")
    capture.add_argument("--source-root", type=pathlib.Path, default=REPO_ROOT)
    capture.add_argument("--out-dir", type=pathlib.Path)
    capture.add_argument("--fsdb", choices=("auto", "always", "never"), default="auto")
    capture.add_argument("--allow-dirty-source", action="store_true")
    capture.add_argument(
        "--environment-note",
        default="wavepeek manual performance gate",
        help="note written into expression benchmark summaries",
    )
    capture.set_defaults(func=capture_ref)

    compare = subcommands.add_parser("compare", help="compare two benchmark capture directories")
    compare.add_argument("--golden", type=pathlib.Path, required=True)
    compare.add_argument("--revised", type=pathlib.Path, required=True)
    compare.add_argument("--out-dir", type=pathlib.Path)
    add_common_threshold_args(compare)
    compare.set_defaults(func=compare_command)

    gate = subcommands.add_parser("gate", help="capture and compare baseline and revised refs")
    gate.add_argument("--baseline-ref", required=True)
    gate.add_argument("--revised-ref", default="HEAD")
    gate.add_argument("--source-root", type=pathlib.Path, default=REPO_ROOT)
    gate.add_argument("--out-dir", type=pathlib.Path)
    gate.add_argument("--fsdb", choices=("auto", "always", "never"), default="auto")
    gate.add_argument("--allow-dirty-source", action="store_true")
    gate.add_argument(
        "--environment-note",
        default="wavepeek manual performance gate",
        help="note written into expression benchmark summaries",
    )
    add_common_threshold_args(gate)
    gate.set_defaults(func=gate_command)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(list(argv) if argv is not None else None)
    try:
        return int(args.func(args))
    except BenchGateError as error:
        print(f"error: bench-gate: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
