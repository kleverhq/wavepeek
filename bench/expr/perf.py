#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
from datetime import datetime, timezone
import hashlib
import json
import math
import pathlib
import shutil
import statistics
import subprocess
from typing import Any, NoReturn


SCRIPT_DIR = pathlib.Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[1]
DEFAULT_CATALOG_PATH = SCRIPT_DIR / "suites.json"
DEFAULT_RUNS_DIR = SCRIPT_DIR / "runs"
DEFAULT_CRITERION_ROOT = REPO_ROOT / "target" / "criterion"
SUMMARY_NAME = "summary.json"
README_NAME = "README.md"
EMOJI_THRESHOLD_PCT = 3.0


def fail(message: str) -> NoReturn:
    raise SystemExit(f"error: perf: {message}")


def normalize_path(path_value: str) -> pathlib.Path:
    return pathlib.Path(path_value).expanduser().resolve()


def normalize_repo_relative(path: pathlib.Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def require_nonempty_str(value: Any, field: str) -> str:
    if not isinstance(value, str) or not value:
        fail(f"`{field}` must be a non-empty string")
    return value


def require_nonempty_list(value: Any, field: str) -> list[Any]:
    if not isinstance(value, list) or not value:
        fail(f"`{field}` must be a non-empty list")
    return list(value)


def ensure_existing_dir(path: pathlib.Path, label: str) -> None:
    if not path.exists() or not path.is_dir():
        fail(f"{label} directory does not exist: {path}")


def ensure_empty_dir_ready(path: pathlib.Path) -> None:
    if not path.exists():
        path.mkdir(parents=True, exist_ok=False)
        return
    if not path.is_dir():
        fail(f"run path exists but is not a directory: {path}")
    if any(path.iterdir()):
        fail(
            "run directory must be empty before capture: "
            f"{path} (remove it or pass --missing-only to resume)"
        )


def resolve_run_dir(
    run_dir_arg: str | None,
    out_dir_arg: str,
    *,
    missing_only: bool,
) -> pathlib.Path:
    if run_dir_arg is not None:
        run_dir = normalize_path(run_dir_arg)
        if missing_only:
            ensure_existing_dir(run_dir, "run")
            if not any(run_dir.iterdir()):
                fail(
                    "run directory must already contain resumable artifacts when "
                    f"--missing-only is used: {run_dir}"
                )
            return run_dir

        ensure_empty_dir_ready(run_dir)
        return run_dir

    if missing_only:
        fail("--missing-only requires --run-dir")

    out_dir = normalize_path(out_dir_arg)
    out_dir.mkdir(parents=True, exist_ok=True)

    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d_%H-%M-%SZ")
    for attempt in range(1000):
        suffix = "" if attempt == 0 else f"-{attempt}"
        candidate = out_dir / f"{stamp}{suffix}"
        try:
            candidate.mkdir(parents=True, exist_ok=False)
            return candidate
        except FileExistsError:
            continue

    fail(f"failed to create run directory in {out_dir}")
    raise AssertionError("unreachable")


def load_catalog(path: pathlib.Path) -> list[dict[str, Any]]:
    if not path.exists() or not path.is_file():
        fail(f"catalog file does not exist: {path}")

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid JSON in catalog {path}: {error}")
    except OSError as error:
        fail(f"failed to read catalog {path}: {error}")

    if not isinstance(payload, dict):
        fail(f"catalog {path} must contain a JSON object")

    raw_suites = require_nonempty_list(payload.get("suites"), "suites")

    suites: list[dict[str, Any]] = []
    seen_ids: set[str] = set()
    seen_targets: set[str] = set()
    for index, raw_suite in enumerate(raw_suites):
        if not isinstance(raw_suite, dict):
            fail(f"suites[{index}] must be an object")

        suite_id = require_nonempty_str(raw_suite.get("id"), f"suites[{index}].id")
        bench_target = require_nonempty_str(
            raw_suite.get("bench_target"),
            f"suites[{index}].bench_target",
        )
        description = require_nonempty_str(
            raw_suite.get("description"),
            f"suites[{index}].description",
        )
        raw_scenarios = require_nonempty_list(
            raw_suite.get("scenarios"),
            f"suites[{index}].scenarios",
        )

        if suite_id in seen_ids:
            fail(f"duplicate suite id '{suite_id}' in {path}")
        if bench_target in seen_targets:
            fail(f"duplicate bench target '{bench_target}' in {path}")

        scenarios: list[str] = []
        seen_scenarios: set[str] = set()
        for scenario_index, raw_scenario in enumerate(raw_scenarios):
            scenario = require_nonempty_str(
                raw_scenario,
                f"suites[{index}].scenarios[{scenario_index}]",
            )
            if scenario in seen_scenarios:
                fail(
                    f"duplicate scenario '{scenario}' in suite '{suite_id}' "
                    f"within {path}"
                )
            seen_scenarios.add(scenario)
            scenarios.append(scenario)

        suites.append(
            {
                "id": suite_id,
                "bench_target": bench_target,
                "description": description,
                "scenarios": scenarios,
            }
        )
        seen_ids.add(suite_id)
        seen_targets.add(bench_target)

    return suites


def catalog_fingerprint(suites: list[dict[str, Any]]) -> str:
    normalized = json.dumps(
        {"suites": suites},
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    )
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def select_suites(
    suites: list[dict[str, Any]],
    requested_ids: list[str],
) -> list[dict[str, Any]]:
    if not requested_ids:
        return list(suites)

    selected_ids: set[str] = set()
    for suite_id in requested_ids:
        if suite_id in selected_ids:
            fail(f"duplicate --suite selection '{suite_id}'")
        selected_ids.add(suite_id)

    known_ids = {str(suite["id"]) for suite in suites}
    unknown = sorted(selected_ids - known_ids)
    if unknown:
        fail("unknown suite selection: " + ", ".join(unknown))

    return [suite for suite in suites if str(suite["id"]) in selected_ids]


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


def criterion_benchmark_candidates(suite_id: str, scenario: str) -> tuple[str, ...]:
    prefixed = f"{suite_id}__{scenario}"
    if prefixed == scenario:
        return (scenario,)
    return (prefixed, scenario)


def collect_suite_raw_csv_paths(
    criterion_root: pathlib.Path,
    baseline_name: str,
    suite: dict[str, Any],
) -> dict[str, tuple[str, pathlib.Path]]:
    ensure_existing_dir(criterion_root, "criterion root")

    suite_id = str(suite["id"])
    expected = {
        scenario: set(criterion_benchmark_candidates(suite_id, scenario))
        for scenario in suite["scenarios"]
    }
    all_expected_ids = {candidate for candidates in expected.values() for candidate in candidates}
    selected: dict[str, tuple[str, pathlib.Path]] = {}
    extra_prefixed: list[str] = []

    for path in sorted(criterion_root.rglob("raw.csv")):
        if path.parent.name != baseline_name:
            continue
        benchmark_id = path.parent.parent.name
        if benchmark_id.startswith(f"{suite_id}__") and benchmark_id not in all_expected_ids:
            extra_prefixed.append(benchmark_id)
            continue
        for scenario, candidate_ids in expected.items():
            if benchmark_id not in candidate_ids:
                continue
            if scenario in selected:
                fail(
                    f"duplicate raw.csv for suite '{suite_id}' scenario '{scenario}' "
                    f"and baseline '{baseline_name}'"
                )
            selected[scenario] = (benchmark_id, path)

    if not selected:
        fail(
            f"requested baseline '{baseline_name}' for suite '{suite_id}' "
            f"was not found under {criterion_root}"
        )

    missing = [scenario for scenario in suite["scenarios"] if scenario not in selected]
    if missing or extra_prefixed:
        details: list[str] = []
        if missing:
            details.append("missing scenarios: " + ", ".join(missing))
        if extra_prefixed:
            details.append("unexpected scenarios: " + ", ".join(sorted(extra_prefixed)))
        fail(
            f"suite capture mismatch for '{suite_id}': " + "; ".join(details)
        )

    return selected


def capture_suite_results(
    criterion_root: pathlib.Path,
    baseline_name: str,
    run_dir: pathlib.Path,
    suite: dict[str, Any],
) -> dict[str, Any]:
    suite_id = str(suite["id"])
    run_dir.mkdir(parents=True, exist_ok=True)
    selected = collect_suite_raw_csv_paths(criterion_root, baseline_name, suite)
    scenarios: list[dict[str, Any]] = []

    for scenario in suite["scenarios"]:
        criterion_benchmark_id, source_path = selected[scenario]
        samples = parse_raw_csv(source_path)
        exported_name = f"{suite_id}__{scenario}.raw.csv"
        shutil.copyfile(source_path, run_dir / exported_name)
        scenarios.append(
            {
                "scenario": scenario,
                "criterion_benchmark_id": criterion_benchmark_id,
                "raw_csv": exported_name,
                "sample_count": len(samples),
                "mean_ns_per_iter": statistics.fmean(samples),
                "median_ns_per_iter": statistics.median(samples),
            }
        )

    return {
        "id": suite_id,
        "bench_target": str(suite["bench_target"]),
        "description": str(suite["description"]),
        "scenarios": scenarios,
    }


def cargo_lock_criterion_version() -> str:
    lock_path = REPO_ROOT / "Cargo.lock"
    try:
        payload = lock_path.read_text(encoding="utf-8")
    except OSError as error:
        fail(f"failed to read Cargo.lock: {error}")

    marker = '[[package]]\nname = "criterion"\nversion = "'
    start = payload.find(marker)
    if start < 0:
        fail("criterion package version was not found in Cargo.lock")
    version_start = start + len(marker)
    version_end = payload.find('"', version_start)
    if version_end < 0:
        fail("criterion package version was not found in Cargo.lock")
    return payload[version_start:version_end]


def tool_version(command: list[str]) -> str:
    try:
        output = subprocess.check_output(command, cwd=REPO_ROOT, text=True)
    except (OSError, subprocess.CalledProcessError) as error:
        fail(f"failed to run {' '.join(command)}: {error}")
    return output.strip()


def git_source_commit() -> str:
    return tool_version(["git", "rev-parse", "HEAD"])


def git_worktree_state() -> str:
    try:
        output = subprocess.check_output(
            ["git", "status", "--short"],
            cwd=REPO_ROOT,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        fail(f"failed to inspect git worktree state: {error}")
    return "clean" if not output.strip() else "dirty"


def now_utc() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def build_summary(
    run_dir: pathlib.Path,
    catalog_path: pathlib.Path,
    catalog_fp: str,
    selected_suites: list[dict[str, Any]],
    suite_results: dict[str, dict[str, Any]],
    *,
    cargo_version: str,
    rustc_version: str,
    criterion_version: str,
    source_commit: str,
    worktree_state: str,
    environment_note: str,
) -> dict[str, Any]:
    ordered_results = [
        suite_results[str(suite["id"])]
        for suite in selected_suites
        if str(suite["id"]) in suite_results
    ]
    return {
        "generated_at_utc": now_utc(),
        "run_name": run_dir.name,
        "catalog_path": normalize_repo_relative(catalog_path),
        "catalog_fingerprint": catalog_fp,
        "selected_suite_ids": [str(suite["id"]) for suite in selected_suites],
        "cargo_version": cargo_version,
        "rustc_version": rustc_version,
        "criterion_version": criterion_version,
        "source_commit": source_commit,
        "worktree_state": worktree_state,
        "environment_note": environment_note,
        "suites": ordered_results,
    }


def write_summary(run_dir: pathlib.Path, payload: dict[str, Any]) -> pathlib.Path:
    summary_path = run_dir / SUMMARY_NAME
    summary_path.write_text(
        json.dumps(payload, ensure_ascii=True, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return summary_path


def load_summary(run_dir: pathlib.Path) -> dict[str, Any]:
    summary_path = run_dir / SUMMARY_NAME
    if not summary_path.exists() or not summary_path.is_file():
        fail(f"missing {SUMMARY_NAME} in {run_dir}")

    try:
        payload = json.loads(summary_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid JSON in {summary_path}: {error}")
    except OSError as error:
        fail(f"failed to read {summary_path}: {error}")

    if not isinstance(payload, dict):
        fail(f"{summary_path} must contain an object")

    for field in (
        "generated_at_utc",
        "run_name",
        "catalog_path",
        "catalog_fingerprint",
        "cargo_version",
        "rustc_version",
        "criterion_version",
        "source_commit",
        "worktree_state",
        "environment_note",
    ):
        require_nonempty_str(payload.get(field), field)

    raw_selected_suite_ids = require_nonempty_list(
        payload.get("selected_suite_ids"),
        "selected_suite_ids",
    )
    selected_suite_ids: list[str] = []
    seen_selected_suite_ids: set[str] = set()
    for index, raw_suite_id in enumerate(raw_selected_suite_ids):
        suite_id = require_nonempty_str(raw_suite_id, f"selected_suite_ids[{index}]")
        if suite_id in seen_selected_suite_ids:
            fail(f"duplicate selected suite id '{suite_id}' in {summary_path}")
        seen_selected_suite_ids.add(suite_id)
        selected_suite_ids.append(suite_id)
    payload["selected_suite_ids"] = selected_suite_ids

    raw_suites = payload.get("suites")
    if not isinstance(raw_suites, list):
        fail(f"{summary_path} field `suites` must be a list")

    suites: list[dict[str, Any]] = []
    seen_suite_ids: set[str] = set()
    seen_suite_scenarios: set[tuple[str, str]] = set()
    for suite_index, raw_suite in enumerate(raw_suites):
        if not isinstance(raw_suite, dict):
            fail(f"{summary_path} suites[{suite_index}] must be an object")

        suite_id = require_nonempty_str(raw_suite.get("id"), f"suites[{suite_index}].id")
        if suite_id in seen_suite_ids:
            fail(f"duplicate suite '{suite_id}' in {summary_path}")
        if suite_id not in seen_selected_suite_ids:
            fail(
                f"suite '{suite_id}' in {summary_path} is not listed in selected_suite_ids"
            )

        bench_target = require_nonempty_str(
            raw_suite.get("bench_target"),
            f"suites[{suite_index}].bench_target",
        )
        description = require_nonempty_str(
            raw_suite.get("description"),
            f"suites[{suite_index}].description",
        )
        raw_scenarios = raw_suite.get("scenarios")
        if not isinstance(raw_scenarios, list):
            fail(f"{summary_path} suites[{suite_index}].scenarios must be a list")

        scenarios: list[dict[str, Any]] = []
        for scenario_index, raw_scenario in enumerate(raw_scenarios):
            if not isinstance(raw_scenario, dict):
                fail(
                    f"{summary_path} suites[{suite_index}].scenarios[{scenario_index}] "
                    "must be an object"
                )

            scenario = require_nonempty_str(
                raw_scenario.get("scenario"),
                f"suites[{suite_index}].scenarios[{scenario_index}].scenario",
            )
            suite_scenario = (suite_id, scenario)
            if suite_scenario in seen_suite_scenarios:
                fail(
                    f"duplicate suite/scenario '{suite_id}/{scenario}' in {summary_path}"
                )
            seen_suite_scenarios.add(suite_scenario)

            criterion_benchmark_id = require_nonempty_str(
                raw_scenario.get("criterion_benchmark_id"),
                (
                    f"suites[{suite_index}].scenarios[{scenario_index}]"
                    ".criterion_benchmark_id"
                ),
            )
            raw_csv = require_nonempty_str(
                raw_scenario.get("raw_csv"),
                f"suites[{suite_index}].scenarios[{scenario_index}].raw_csv",
            )

            raw_sample_count = raw_scenario.get("sample_count")
            if not isinstance(raw_sample_count, (int, float, str)):
                fail(
                    f"{summary_path} suites[{suite_index}].scenarios[{scenario_index}]"
                    ".sample_count must be an integer"
                )
            try:
                sample_count = int(raw_sample_count)
            except (TypeError, ValueError):
                fail(
                    f"{summary_path} suites[{suite_index}].scenarios[{scenario_index}]"
                    ".sample_count must be an integer"
                )
            if sample_count <= 0:
                fail(
                    f"{summary_path} suites[{suite_index}].scenarios[{scenario_index}]"
                    ".sample_count must be positive"
                )

            raw_mean_ns_per_iter = raw_scenario.get("mean_ns_per_iter")
            raw_median_ns_per_iter = raw_scenario.get("median_ns_per_iter")
            if not isinstance(raw_mean_ns_per_iter, (int, float, str)) or not isinstance(
                raw_median_ns_per_iter,
                (int, float, str),
            ):
                fail(
                    f"{summary_path} suites[{suite_index}].scenarios[{scenario_index}] "
                    "must have numeric mean_ns_per_iter and median_ns_per_iter"
                )
            try:
                mean_ns_per_iter = float(raw_mean_ns_per_iter)
                median_ns_per_iter = float(raw_median_ns_per_iter)
            except (TypeError, ValueError):
                fail(
                    f"{summary_path} suites[{suite_index}].scenarios[{scenario_index}] "
                    "must have numeric mean_ns_per_iter and median_ns_per_iter"
                )
            if not math.isfinite(mean_ns_per_iter) or not math.isfinite(median_ns_per_iter):
                fail(
                    f"{summary_path} suite '{suite_id}' scenario '{scenario}' has non-finite "
                    "mean/median"
                )
            if mean_ns_per_iter <= 0 or median_ns_per_iter <= 0:
                fail(
                    f"{summary_path} suite '{suite_id}' scenario '{scenario}' has non-positive "
                    "mean/median"
                )

            scenarios.append(
                {
                    "scenario": scenario,
                    "criterion_benchmark_id": criterion_benchmark_id,
                    "raw_csv": raw_csv,
                    "sample_count": sample_count,
                    "mean_ns_per_iter": mean_ns_per_iter,
                    "median_ns_per_iter": median_ns_per_iter,
                }
            )

        suites.append(
            {
                "id": suite_id,
                "bench_target": bench_target,
                "description": description,
                "scenarios": scenarios,
            }
        )
        seen_suite_ids.add(suite_id)

    payload["suites"] = suites
    return payload


def validate_existing_suite_artifacts(run_dir: pathlib.Path, summary: dict[str, Any]) -> None:
    for suite in summary["suites"]:
        for scenario in suite["scenarios"]:
            artifact_path = run_dir / str(scenario["raw_csv"])
            if not artifact_path.exists() or not artifact_path.is_file():
                fail(
                    "resumable run is missing exported raw csv artifact: "
                    f"{artifact_path}"
                )


def validate_resume_summary(
    summary: dict[str, Any],
    catalog_path: pathlib.Path,
    catalog_fp: str,
    selected_suite_ids: list[str],
) -> None:
    if str(summary["catalog_fingerprint"]) != catalog_fp:
        fail(
            "catalog fingerprint mismatch for --missing-only resume: "
            f"summary={summary['catalog_fingerprint']}, requested={catalog_fp}"
        )

    if list(summary["selected_suite_ids"]) != selected_suite_ids:
        fail(
            "selected suite mismatch for --missing-only resume: "
            f"summary={summary['selected_suite_ids']}, requested={selected_suite_ids}"
        )


def suite_row_map(summary: dict[str, Any]) -> dict[tuple[str, str], dict[str, Any]]:
    rows: dict[tuple[str, str], dict[str, Any]] = {}
    for suite in summary["suites"]:
        suite_id = str(suite["id"])
        for row in suite["scenarios"]:
            rows[(suite_id, str(row["scenario"]))] = row
    return rows


def ensure_complete_selected_suites(summary: dict[str, Any]) -> None:
    selected_suite_ids = list(summary["selected_suite_ids"])
    actual_suite_ids = [str(suite["id"]) for suite in summary["suites"]]
    if actual_suite_ids != selected_suite_ids:
        missing = [suite_id for suite_id in selected_suite_ids if suite_id not in actual_suite_ids]
        extra = [suite_id for suite_id in actual_suite_ids if suite_id not in selected_suite_ids]
        details: list[str] = []
        if missing:
            details.append("missing suites: " + ", ".join(missing))
        if extra:
            details.append("unexpected suites: " + ", ".join(extra))
        fail("suite mismatch in summary: " + "; ".join(details))


def validate_compare_identity(revised: dict[str, Any], golden: dict[str, Any]) -> None:
    if str(revised["catalog_fingerprint"]) != str(golden["catalog_fingerprint"]):
        fail(
            "catalog fingerprint mismatch between summaries: "
            f"revised={revised['catalog_fingerprint']}, golden={golden['catalog_fingerprint']}"
        )

    if list(revised["selected_suite_ids"]) != list(golden["selected_suite_ids"]):
        fail(
            "selected suite mismatch between summaries: "
            f"revised={revised['selected_suite_ids']}, golden={golden['selected_suite_ids']}"
        )

    ensure_complete_selected_suites(revised)
    ensure_complete_selected_suites(golden)

    revised_suite_ids = [str(suite["id"]) for suite in revised["suites"]]
    golden_suite_ids = [str(suite["id"]) for suite in golden["suites"]]
    if revised_suite_ids != golden_suite_ids:
        fail(
            "suite mismatch between summaries: "
            f"revised={revised_suite_ids}, golden={golden_suite_ids}"
        )

    golden_by_id = {str(suite["id"]): suite for suite in golden["suites"]}
    for revised_suite in revised["suites"]:
        suite_id = str(revised_suite["id"])
        golden_suite = golden_by_id[suite_id]
        if str(revised_suite["bench_target"]) != str(golden_suite["bench_target"]):
            fail(
                f"bench target mismatch for suite '{suite_id}': "
                f"revised={revised_suite['bench_target']}, golden={golden_suite['bench_target']}"
            )

        revised_scenarios = {str(row["scenario"]): row for row in revised_suite["scenarios"]}
        golden_scenarios = {str(row["scenario"]): row for row in golden_suite["scenarios"]}
        if set(revised_scenarios) != set(golden_scenarios):
            missing = sorted(set(golden_scenarios) - set(revised_scenarios))
            extra = sorted(set(revised_scenarios) - set(golden_scenarios))
            details: list[str] = []
            if missing:
                details.append("missing scenarios: " + ", ".join(missing))
            if extra:
                details.append("unexpected scenarios: " + ", ".join(extra))
            fail(f"scenario mismatch for suite '{suite_id}': " + "; ".join(details))

        for scenario_name, revised_row in revised_scenarios.items():
            golden_row = golden_scenarios[scenario_name]
            if str(revised_row["criterion_benchmark_id"]) != str(
                golden_row["criterion_benchmark_id"]
            ):
                fail(
                    "criterion benchmark id mismatch for suite/scenario "
                    f"'{suite_id}/{scenario_name}': revised={revised_row['criterion_benchmark_id']}, "
                    f"golden={golden_row['criterion_benchmark_id']}"
                )


def delta_pct(revised: float, golden: float) -> float:
    return ((golden - revised) / golden) * 100.0


def speed_factor(revised: float, golden: float) -> tuple[float, str]:
    if revised == golden:
        return 1.0, "same"
    if revised < golden:
        if revised == 0:
            return float("inf"), "faster"
        return golden / revised, "faster"
    if golden == 0:
        return float("inf"), "slower"
    return revised / golden, "slower"


def format_speed_factor(revised: float, golden: float) -> str:
    factor, direction = speed_factor(revised, golden)
    factor_text = "infx" if math.isinf(factor) else f"{factor:.2f}x"
    if direction == "same":
        return factor_text
    return f"{factor_text} {direction}"


def format_metric(value: float, baseline: dict[str, Any] | None, metric: str) -> str:
    base = f"{value:.6f}"
    if baseline is None:
        return base

    baseline_value = float(baseline[metric])
    delta = delta_pct(value, baseline_value)
    speed = format_speed_factor(value, baseline_value)
    emoji = ""
    if delta >= EMOJI_THRESHOLD_PCT:
        emoji = " 🟢"
    elif delta <= -EMOJI_THRESHOLD_PCT:
        emoji = " 🔴"
    return f"{base} ({delta:+.2f}%, {speed}){emoji}"


def render_report(
    run_dir: pathlib.Path,
    summary: dict[str, Any],
    compare_summary: dict[str, Any] | None,
    compare_dir: pathlib.Path | None,
) -> str:
    lines = [
        f"# Expression Bench Run: {run_dir.name}",
        "",
        f"- Generated at (UTC): {now_utc()}",
        f"- Catalog: `{summary['catalog_path']}`",
        f"- Catalog fingerprint: `{summary['catalog_fingerprint']}`",
        f"- Selected suites: `{', '.join(summary['selected_suite_ids'])}`",
        f"- cargo -V: `{summary['cargo_version']}`",
        f"- rustc -V: `{summary['rustc_version']}`",
        f"- criterion crate version: `{summary['criterion_version']}`",
        f"- Source commit: `{summary['source_commit']}`",
        f"- Worktree state: `{summary['worktree_state']}`",
        f"- Environment note: {summary['environment_note']}",
    ]

    compare_rows: dict[tuple[str, str], dict[str, Any]] = {}
    if compare_summary is not None and compare_dir is not None:
        compare_rows = suite_row_map(compare_summary)
        lines.extend(
            [
                f"- Compare baseline: `{compare_dir}`",
                "- Delta formula: `((golden - revised) / golden) * 100`",
                "- Speed factor: `golden/revised` when faster, `revised/golden` when slower",
            ]
        )
    lines.append("")

    if not summary["suites"]:
        lines.append("No suite data captured yet.")
        lines.append("")
        return "\n".join(lines)

    for suite in summary["suites"]:
        suite_id = str(suite["id"])
        lines.extend([f"## {suite_id}", ""])
        if not suite["scenarios"]:
            lines.append("No captured scenarios yet.")
            lines.append("")
            continue
        lines.extend(
            [
                "| scenario | mean ns/iter | median ns/iter |",
                "| --- | --- | --- |",
            ]
        )
        for row in suite["scenarios"]:
            baseline = compare_rows.get((suite_id, str(row["scenario"])))
            lines.append(
                "| "
                + " | ".join(
                    [
                        str(row["scenario"]),
                        format_metric(float(row["mean_ns_per_iter"]), baseline, "mean_ns_per_iter"),
                        format_metric(
                            float(row["median_ns_per_iter"]),
                            baseline,
                            "median_ns_per_iter",
                        ),
                    ]
                )
                + " |"
            )
        lines.append("")

    return "\n".join(lines)


def write_report(
    run_dir: pathlib.Path,
    summary: dict[str, Any],
    compare_summary: dict[str, Any] | None,
    compare_dir: pathlib.Path | None,
) -> pathlib.Path:
    report_path = run_dir / README_NAME
    report_path.write_text(
        render_report(run_dir, summary, compare_summary, compare_dir) + "\n",
        encoding="utf-8",
    )
    return report_path


def run_suite_benchmark(suite: dict[str, Any], baseline_name: str) -> None:
    command = [
        "cargo",
        "bench",
        "--bench",
        str(suite["bench_target"]),
        "--",
        "--save-baseline",
        baseline_name,
        "--noplot",
    ]
    result = subprocess.run(command, cwd=REPO_ROOT, check=False)
    if result.returncode != 0:
        fail(
            f"cargo bench failed for suite '{suite['id']}' "
            f"(bench target {suite['bench_target']})"
        )


def cmd_list(args: argparse.Namespace) -> int:
    catalog_path = normalize_path(getattr(args, "catalog", str(DEFAULT_CATALOG_PATH)))
    suites = select_suites(load_catalog(catalog_path), list(getattr(args, "suite", [])))
    for suite in suites:
        print(
            f"{suite['id']}\t{suite['bench_target']}\t{len(suite['scenarios'])} scenarios"
        )
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    catalog_path = normalize_path(getattr(args, "catalog", str(DEFAULT_CATALOG_PATH)))
    catalog = load_catalog(catalog_path)
    selected_suites = select_suites(catalog, list(getattr(args, "suite", [])))
    if not selected_suites:
        fail("no suites selected")

    selected_suite_ids = [str(suite["id"]) for suite in selected_suites]
    run_dir = resolve_run_dir(
        getattr(args, "run_dir", None),
        getattr(args, "out_dir", str(DEFAULT_RUNS_DIR)),
        missing_only=bool(getattr(args, "missing_only", False)),
    )
    criterion_root = normalize_path(getattr(args, "criterion_root", str(DEFAULT_CRITERION_ROOT)))
    catalog_fp = catalog_fingerprint(catalog)

    cargo_version = tool_version(["cargo", "-V"])
    rustc_version = tool_version(["rustc", "-V"])
    criterion_version = cargo_lock_criterion_version()
    source_commit = git_source_commit()
    worktree_state = git_worktree_state()
    environment_note = str(getattr(args, "environment_note", "wavepeek devcontainer/CI image"))
    compare_dir = normalize_path(args.compare) if getattr(args, "compare", None) else None
    compare_summary: dict[str, Any] | None = None

    suite_results: dict[str, dict[str, Any]] = {}
    if bool(getattr(args, "missing_only", False)):
        existing_summary = load_summary(run_dir)
        validate_resume_summary(existing_summary, catalog_path, catalog_fp, selected_suite_ids)
        validate_existing_suite_artifacts(run_dir, existing_summary)
        suite_results = {str(suite["id"]): suite for suite in existing_summary["suites"]}
    else:
        partial_summary = build_summary(
            run_dir,
            catalog_path,
            catalog_fp,
            selected_suites,
            suite_results,
            cargo_version=cargo_version,
            rustc_version=rustc_version,
            criterion_version=criterion_version,
            source_commit=source_commit,
            worktree_state=worktree_state,
            environment_note=environment_note,
        )
        write_summary(run_dir, partial_summary)
        write_report(run_dir, partial_summary, None, None)

    baseline_name = run_dir.name
    for suite in selected_suites:
        suite_id = str(suite["id"])
        if suite_id in suite_results:
            continue
        run_suite_benchmark(suite, baseline_name)
        suite_results[suite_id] = capture_suite_results(criterion_root, baseline_name, run_dir, suite)
        partial_summary = build_summary(
            run_dir,
            catalog_path,
            catalog_fp,
            selected_suites,
            suite_results,
            cargo_version=cargo_version,
            rustc_version=rustc_version,
            criterion_version=criterion_version,
            source_commit=source_commit,
            worktree_state=worktree_state,
            environment_note=environment_note,
        )
        write_summary(run_dir, partial_summary)
        write_report(run_dir, partial_summary, None, None)

    final_summary = build_summary(
        run_dir,
        catalog_path,
        catalog_fp,
        selected_suites,
        suite_results,
        cargo_version=cargo_version,
        rustc_version=rustc_version,
        criterion_version=criterion_version,
        source_commit=source_commit,
        worktree_state=worktree_state,
        environment_note=environment_note,
    )
    write_summary(run_dir, final_summary)

    if compare_dir is not None:
        ensure_existing_dir(compare_dir, "compare")
        compare_summary = load_summary(compare_dir)
        validate_compare_identity(final_summary, compare_summary)
    write_report(run_dir, final_summary, compare_summary, compare_dir)

    print(
        f"ok: run: captured {len(selected_suites)} suites into {run_dir} "
        f"(catalog fingerprint {catalog_fp})"
    )
    return 0


def cmd_report(args: argparse.Namespace) -> int:
    run_dir = normalize_path(args.run_dir)
    ensure_existing_dir(run_dir, "report")
    summary = load_summary(run_dir)
    compare_dir = normalize_path(args.compare) if getattr(args, "compare", None) else None
    compare_summary: dict[str, Any] | None = None
    if compare_dir is not None:
        ensure_existing_dir(compare_dir, "report compare")
        compare_summary = load_summary(compare_dir)
        validate_compare_identity(summary, compare_summary)
    report_path = write_report(run_dir, summary, compare_summary, compare_dir)
    print(f"info: report updated at {report_path}")
    return 0


def cmd_compare(args: argparse.Namespace) -> int:
    threshold = float(args.max_negative_delta_pct)
    if not math.isfinite(threshold) or threshold < 0:
        fail("--max-negative-delta-pct must be a finite non-negative number")

    revised_dir = normalize_path(args.revised)
    golden_dir = normalize_path(args.golden)
    ensure_existing_dir(revised_dir, "revised")
    ensure_existing_dir(golden_dir, "golden")

    revised = load_summary(revised_dir)
    golden = load_summary(golden_dir)
    validate_compare_identity(revised, golden)

    summary_path_revised = revised_dir / SUMMARY_NAME
    summary_path_golden = golden_dir / SUMMARY_NAME
    try:
        revised_payload = json.loads(summary_path_revised.read_text(encoding="utf-8"))
        golden_payload = json.loads(summary_path_golden.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid JSON while loading metadata keys: {error}")
    except OSError as error:
        fail(f"failed to read summary payload for metadata checks: {error}")

    for key in getattr(args, "require_matching_metadata", []):
        if key not in revised_payload or key not in golden_payload:
            fail(
                "required metadata key missing from summary: "
                f"{key} (revised={revised_dir}, golden={golden_dir})"
            )
        if revised_payload[key] != golden_payload[key]:
            fail(
                "required metadata mismatch: "
                f"{key} (revised={revised_payload[key]!r}, golden={golden_payload[key]!r})"
            )

    golden_rows = suite_row_map(golden)
    failures: list[str] = []
    for suite in revised["suites"]:
        suite_id = str(suite["id"])
        for row in suite["scenarios"]:
            scenario = str(row["scenario"])
            golden_row = golden_rows[(suite_id, scenario)]
            revised_mean = float(row["mean_ns_per_iter"])
            revised_median = float(row["median_ns_per_iter"])
            golden_mean = float(golden_row["mean_ns_per_iter"])
            golden_median = float(golden_row["median_ns_per_iter"])
            mean_delta = delta_pct(revised_mean, golden_mean)
            median_delta = delta_pct(revised_median, golden_median)
            if mean_delta < -threshold or median_delta < -threshold:
                failures.append(
                    f"{suite_id}/{scenario}: mean_delta={mean_delta:+.2f}% "
                    f"(revised={revised_mean:.6f}, golden={golden_mean:.6f}), "
                    f"median_delta={median_delta:+.2f}% "
                    f"(revised={revised_median:.6f}, golden={golden_median:.6f})"
                )

    if failures:
        for row in failures:
            print(row)
        fail(
            "one or more suite scenarios exceeded allowed negative delta "
            f"({threshold:.2f}%)"
        )

    print(
        f"ok: no suite scenario exceeded {threshold:.2f}% "
        "negative delta in mean or median"
    )
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Expression microbenchmark harness")
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list", help="list benchmark suites")
    list_parser.add_argument(
        "--catalog",
        default=str(DEFAULT_CATALOG_PATH),
        help="path to benchmark catalog JSON",
    )
    list_parser.add_argument(
        "--suite",
        action="append",
        default=[],
        help="repeatable suite id filter",
    )

    run_parser = subparsers.add_parser("run", help="run selected benchmark suites")
    run_parser.add_argument(
        "--catalog",
        default=str(DEFAULT_CATALOG_PATH),
        help="path to benchmark catalog JSON",
    )
    run_parser.add_argument(
        "--suite",
        action="append",
        default=[],
        help="repeatable suite id filter",
    )
    run_parser.add_argument("--run-dir", default=None, help="existing or new run directory")
    run_parser.add_argument(
        "--out-dir",
        default=str(DEFAULT_RUNS_DIR),
        help="parent directory for timestamped runs when --run-dir is omitted",
    )
    run_parser.add_argument(
        "--compare",
        default=None,
        help="baseline run directory for README delta annotations",
    )
    run_parser.add_argument(
        "--missing-only",
        action="store_true",
        help="resume an existing run directory and capture only missing suites",
    )
    run_parser.add_argument(
        "--criterion-root",
        default=str(DEFAULT_CRITERION_ROOT),
        help="path to target/criterion",
    )
    run_parser.add_argument(
        "--environment-note",
        default="wavepeek devcontainer/CI image",
        help="provenance note written into summary and README",
    )

    report_parser = subparsers.add_parser("report", help="regenerate README.md for a run")
    report_parser.add_argument("--run-dir", required=True, help="run directory")
    report_parser.add_argument(
        "--compare",
        default=None,
        help="baseline run directory for README delta annotations",
    )

    compare_parser = subparsers.add_parser("compare", help="compare revised and golden runs")
    compare_parser.add_argument("--revised", required=True, help="revised run directory")
    compare_parser.add_argument("--golden", required=True, help="golden run directory")
    compare_parser.add_argument(
        "--max-negative-delta-pct",
        required=True,
        type=float,
        help="fail when mean or median delta goes below the negative threshold",
    )
    compare_parser.add_argument(
        "--require-matching-metadata",
        nargs="+",
        default=[],
        help="extra summary.json metadata keys that must match between runs",
    )

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "list":
        return cmd_list(args)
    if args.command == "run":
        return cmd_run(args)
    if args.command == "report":
        return cmd_report(args)
    if args.command == "compare":
        return cmd_compare(args)

    fail(f"unsupported command: {args.command}")
    raise AssertionError("unreachable")


if __name__ == "__main__":
    raise SystemExit(main())
