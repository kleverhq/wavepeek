#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import json
import math
import pathlib
import re
import shutil
import statistics
import subprocess
from typing import Any, NoReturn


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]


def fail(message: str) -> NoReturn:
    raise SystemExit(f"error: capture: {message}")


def normalize_path(path_value: str) -> pathlib.Path:
    return pathlib.Path(path_value).expanduser().resolve()


def ensure_existing_dir(path: pathlib.Path, label: str) -> None:
    if not path.exists() or not path.is_dir():
        fail(f"{label} directory does not exist: {path}")


def normalize_repo_relative(path: pathlib.Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def load_scenario_set(path: pathlib.Path) -> tuple[str, str, tuple[str, ...]]:
    if not path.exists() or not path.is_file():
        fail(f"scenario-set file does not exist: {path}")

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid JSON in scenario-set file {path}: {error}")
    except OSError as error:
        fail(f"failed to read scenario-set file {path}: {error}")

    if not isinstance(payload, dict):
        fail(f"scenario-set file {path} must contain a JSON object")

    scenario_set_id = payload.get("id")
    if not isinstance(scenario_set_id, str) or not scenario_set_id:
        fail(f"scenario-set file {path} field `id` must be a non-empty string")

    bench_target = payload.get("bench_target")
    if not isinstance(bench_target, str) or not bench_target:
        fail(
            f"scenario-set file {path} field `bench_target` must be a non-empty string"
        )

    scenarios_raw = payload.get("scenarios")
    if not isinstance(scenarios_raw, list) or not scenarios_raw:
        fail(
            f"scenario-set file {path} field `scenarios` must be a non-empty list"
        )

    scenarios: list[str] = []
    seen: set[str] = set()
    for index, item in enumerate(scenarios_raw):
        if not isinstance(item, str) or not item:
            fail(
                f"scenario-set file {path} scenarios[{index}] must be a non-empty string"
            )
        if item in seen:
            fail(f"scenario-set file {path} has duplicate scenario '{item}'")
        seen.add(item)
        scenarios.append(item)

    return scenario_set_id, bench_target, tuple(scenarios)


def parse_raw_csv(path: pathlib.Path) -> list[float]:
    try:
        with path.open("r", encoding="utf-8", newline="") as handle:
            reader = csv.DictReader(handle)
            if reader.fieldnames is None:
                fail(f"{path} has no CSV header")
            required = {"sample_measured_value", "iteration_count"}
            missing = sorted(required.difference(reader.fieldnames))
            if missing:
                fail(f"{path} missing CSV columns: {', '.join(missing)}")

            values: list[float] = []
            for row in reader:
                measured = float(row["sample_measured_value"])
                iterations = int(row["iteration_count"])
                if iterations <= 0:
                    fail(f"{path} contains non-positive iteration_count")
                if not math.isfinite(measured):
                    fail(f"{path} contains non-finite sample_measured_value")
                values.append(measured / iterations)
    except OSError as error:
        fail(f"failed to read {path}: {error}")
    except ValueError as error:
        fail(f"invalid numeric value in {path}: {error}")

    if not values:
        fail(f"{path} contains no samples")
    if not all(math.isfinite(value) and value > 0 for value in values):
        fail(f"{path} contains non-finite or non-positive per-iteration values")
    return values


def collect_raw_csv_paths(
    criterion_root: pathlib.Path,
    baseline_name: str,
    required_scenarios: tuple[str, ...],
) -> dict[str, pathlib.Path]:
    selected: dict[str, pathlib.Path] = {}

    for path in sorted(criterion_root.rglob("raw.csv")):
        if path.parent.name != baseline_name:
            continue
        scenario = path.parent.parent.name
        if not scenario:
            fail(f"cannot infer scenario name from {path}")
        if scenario in selected:
            fail(f"duplicate raw.csv for scenario '{scenario}' and baseline '{baseline_name}'")
        selected[scenario] = path

    if not selected:
        fail(f"requested baseline '{baseline_name}' not found under {criterion_root}")

    expected = set(required_scenarios)
    actual = set(selected.keys())
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing or extra:
        details: list[str] = []
        if missing:
            details.append(f"missing scenarios: {', '.join(missing)}")
        if extra:
            details.append(f"unexpected scenarios: {', '.join(extra)}")
        fail("scenario set mismatch for baseline export: " + "; ".join(details))

    return selected


def cargo_lock_criterion_version() -> str:
    lock_path = REPO_ROOT / "Cargo.lock"
    try:
        payload = lock_path.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"failed to read Cargo.lock: {error}")

    match = re.search(
        r'\[\[package\]\]\nname = "criterion"\nversion = "([^"]+)"',
        payload,
    )
    if match is None:
        fail("criterion package version was not found in Cargo.lock")
    return match.group(1)


def tool_version(command: list[str]) -> str:
    try:
        output = subprocess.check_output(command, cwd=REPO_ROOT, text=True)
    except (OSError, subprocess.CalledProcessError) as error:
        fail(f"failed to run {' '.join(command)}: {error}")
    return output.strip()


def write_summary(
    output_dir: pathlib.Path,
    bench_target: str,
    scenario_set_id: str,
    scenario_set_path: str,
    baseline_name: str,
    source_commit: str,
    worktree_state: str,
    environment_note: str,
    cargo_version: str,
    rustc_version: str,
    criterion_version: str,
    scenarios: list[dict[str, Any]],
) -> None:
    payload = {
        "baseline_name": baseline_name,
        "bench_target": bench_target,
        "cargo_version": cargo_version,
        "criterion_version": criterion_version,
        "environment_note": environment_note,
        "run_name": output_dir.name,
        "rustc_version": rustc_version,
        "scenario_set_id": scenario_set_id,
        "scenario_set_path": scenario_set_path,
        "scenarios": scenarios,
        "source_commit": source_commit,
        "worktree_state": worktree_state,
    }
    summary_path = output_dir / "summary.json"
    summary_path.write_text(
        json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def write_readme(
    output_dir: pathlib.Path,
    bench_target: str,
    scenario_set_id: str,
    scenario_set_path: str,
    baseline_name: str,
    source_commit: str,
    worktree_state: str,
    environment_note: str,
    cargo_version: str,
    rustc_version: str,
    criterion_version: str,
    scenarios: list[dict[str, Any]],
) -> None:
    command = f"cargo bench --bench {bench_target} -- --save-baseline {baseline_name} --noplot"
    lines = [
        f"# Expression run: {output_dir.name}",
        "",
        f"- Benchmark command: `{command}`",
        f"- Bench target: `{bench_target}`",
        f"- Scenario set: `{scenario_set_id}` ({scenario_set_path})",
        f"- cargo -V: `{cargo_version}`",
        f"- rustc -V: `{rustc_version}`",
        f"- criterion crate version: `{criterion_version}`",
        f"- Source commit: `{source_commit}`",
        f"- Worktree state: `{worktree_state}`",
        f"- Environment note: {environment_note}",
        "",
        "| scenario | samples | mean ns/iter | median ns/iter | raw csv |",
        "| --- | ---: | ---: | ---: | --- |",
    ]

    for row in scenarios:
        lines.append(
            "| "
            + " | ".join(
                [
                    str(row["scenario"]),
                    str(row["sample_count"]),
                    f"{float(row['mean_ns_per_iter']):.6f}",
                    f"{float(row['median_ns_per_iter']):.6f}",
                    str(row["raw_csv"]),
                ]
            )
            + " |"
        )

    readme_path = output_dir / "README.md"
    readme_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Export stable Criterion raw.csv artifacts for expression microbench runs"
    )
    parser.add_argument("--criterion-root", required=True, help="path to target/criterion")
    parser.add_argument(
        "--baseline-name",
        required=True,
        help="saved Criterion baseline name to export",
    )
    parser.add_argument(
        "--bench-target",
        required=True,
        help="criterion bench target name used for this run",
    )
    parser.add_argument(
        "--scenario-set",
        required=True,
        help="path to scenario-set JSON manifest",
    )
    parser.add_argument("--output", required=True, help="output run directory")
    parser.add_argument("--source-commit", required=True, help="git commit used for capture")
    parser.add_argument(
        "--worktree-state",
        required=True,
        choices=("clean", "dirty"),
        help="whether capture source worktree was clean or dirty",
    )
    parser.add_argument(
        "--environment-note",
        required=True,
        help="short environment note for artifact provenance",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    criterion_root = normalize_path(args.criterion_root)
    scenario_set_path = normalize_path(args.scenario_set)
    output_dir = normalize_path(args.output)
    ensure_existing_dir(criterion_root, "criterion root")
    output_dir.mkdir(parents=True, exist_ok=True)

    scenario_set_id, manifest_bench_target, required_scenarios = load_scenario_set(
        scenario_set_path
    )
    if manifest_bench_target != args.bench_target:
        fail(
            "bench target mismatch between arguments and scenario set: "
            f"--bench-target={args.bench_target} vs manifest bench_target={manifest_bench_target}"
        )

    selected = collect_raw_csv_paths(
        criterion_root,
        args.baseline_name,
        required_scenarios,
    )
    scenarios: list[dict[str, Any]] = []

    for scenario in sorted(required_scenarios):
        source_path = selected[scenario]
        samples = parse_raw_csv(source_path)

        exported_name = f"{scenario}.raw.csv"
        exported_path = output_dir / exported_name
        shutil.copyfile(source_path, exported_path)

        scenarios.append(
            {
                "baseline_name": args.baseline_name,
                "mean_ns_per_iter": statistics.fmean(samples),
                "median_ns_per_iter": statistics.median(samples),
                "raw_csv": exported_name,
                "sample_count": len(samples),
                "scenario": scenario,
            }
        )

    cargo_version = tool_version(["cargo", "-V"])
    rustc_version = tool_version(["rustc", "-V"])
    criterion_version = cargo_lock_criterion_version()

    write_summary(
        output_dir,
        args.bench_target,
        scenario_set_id,
        normalize_repo_relative(scenario_set_path),
        args.baseline_name,
        args.source_commit,
        args.worktree_state,
        args.environment_note,
        cargo_version,
        rustc_version,
        criterion_version,
        scenarios,
    )
    write_readme(
        output_dir,
        args.bench_target,
        scenario_set_id,
        normalize_repo_relative(scenario_set_path),
        args.baseline_name,
        args.source_commit,
        args.worktree_state,
        args.environment_note,
        cargo_version,
        rustc_version,
        criterion_version,
        scenarios,
    )

    print(
        "ok: exported baseline "
        f"'{args.baseline_name}' for {len(scenarios)} scenarios into {output_dir} "
        f"(bench target {args.bench_target}, scenario set {scenario_set_id})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
