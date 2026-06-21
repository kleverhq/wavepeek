#!/usr/bin/env python3

from __future__ import annotations

import dataclasses
import datetime as dt
import json
import os
import pathlib
import subprocess
import sys
from collections.abc import Mapping, Sequence
from typing import Any

REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
DEFAULT_OUTPUT_ROOT = pathlib.Path("tmp/bench-gate")
DEFAULT_TIMING_THRESHOLD_PCT = 5.0
DEFAULT_TIMING_THRESHOLD_SECONDS = 0.005
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


def worktree_status(source_root: pathlib.Path) -> str:
    return run_output(
        ["git", "status", "--short", "--untracked-files=all"],
        cwd=source_root,
    ).stdout.strip()


def enforce_clean_worktree(source_root: pathlib.Path, *, reason: str) -> None:
    status = worktree_status(source_root)
    if status:
        raise BenchGateError(f"source worktree is dirty; {reason}")


def enforce_clean_source_for_head_refs(
    source_root: pathlib.Path,
    refs: Sequence[str],
) -> None:
    if "HEAD" not in refs:
        return
    enforce_clean_worktree(
        source_root,
        reason="HEAD clones only committed state; commit or stash changes before benchmarking HEAD",
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
        return os.path.relpath(path, base)


def command_to_manifest(result: CommandResult, base: pathlib.Path) -> dict[str, Any]:
    return {
        "name": result.name,
        "args": result.args,
        "cwd": result.cwd,
        "returncode": result.returncode,
        "log_path": relative_to(pathlib.Path(result.log_path), base) if result.log_path else None,
    }


def tool_version(args: Sequence[str], *, cwd: pathlib.Path) -> str | None:
    result = run_output(args, cwd=cwd, check=False)
    if result.returncode != 0:
        return None
    return (result.stdout or result.stderr).strip()
