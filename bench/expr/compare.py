#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import math
import pathlib
from typing import Any, NoReturn


def fail(message: str) -> NoReturn:
    raise SystemExit(f"error: compare: {message}")


def normalize_path(path_value: str) -> pathlib.Path:
    return pathlib.Path(path_value).expanduser().resolve()


def ensure_existing_dir(path: pathlib.Path, label: str) -> None:
    if not path.exists() or not path.is_dir():
        fail(f"{label} directory does not exist: {path}")


def load_summary(run_dir: pathlib.Path) -> dict[str, dict[str, float]]:
    summary_path = run_dir / "summary.json"
    if not summary_path.exists() or not summary_path.is_file():
        fail(f"missing summary.json in {run_dir}")

    try:
        payload = json.loads(summary_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"invalid JSON in {summary_path}: {error}")
    except OSError as error:
        fail(f"failed to read {summary_path}: {error}")

    if not isinstance(payload, dict):
        fail(f"{summary_path} must be an object")

    raw_scenarios = payload.get("scenarios")
    if not isinstance(raw_scenarios, list) or not raw_scenarios:
        fail(f"{summary_path} field `scenarios` must be a non-empty list")

    by_name: dict[str, dict[str, float]] = {}
    for index, row in enumerate(raw_scenarios):
        if not isinstance(row, dict):
            fail(f"{summary_path} scenarios[{index}] must be an object")

        scenario = row.get("scenario")
        if not isinstance(scenario, str) or not scenario:
            fail(f"{summary_path} scenarios[{index}].scenario must be a non-empty string")
        if scenario in by_name:
            fail(f"{summary_path} has duplicate scenario '{scenario}'")

        mean_raw = row.get("mean_ns_per_iter")
        median_raw = row.get("median_ns_per_iter")
        if mean_raw is None or median_raw is None:
            fail(
                f"{summary_path} scenario '{scenario}' is missing "
                "mean_ns_per_iter or median_ns_per_iter"
            )
        try:
            mean_value = float(mean_raw)
            median_value = float(median_raw)
        except (TypeError, ValueError):
            fail(
                f"{summary_path} scenario '{scenario}' must have numeric "
                "mean_ns_per_iter and median_ns_per_iter"
            )
        if not math.isfinite(mean_value) or not math.isfinite(median_value):
            fail(f"{summary_path} scenario '{scenario}' has non-finite mean/median")
        if mean_value <= 0 or median_value <= 0:
            fail(
                f"{summary_path} scenario '{scenario}' mean/median must be positive numbers"
            )

        by_name[scenario] = {
            "mean": mean_value,
            "median": median_value,
        }

    return by_name


def delta_pct(revised: float, golden: float) -> float:
    return ((golden - revised) / golden) * 100.0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Compare exported expression benchmark summaries"
    )
    parser.add_argument("--revised", required=True, help="revised run directory")
    parser.add_argument("--golden", required=True, help="golden run directory")
    parser.add_argument(
        "--max-negative-delta-pct",
        required=True,
        type=float,
        help="fail when mean or median delta goes below negative threshold",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    threshold = float(args.max_negative_delta_pct)
    if threshold < 0:
        fail("--max-negative-delta-pct must be non-negative")

    revised_dir = normalize_path(args.revised)
    golden_dir = normalize_path(args.golden)
    ensure_existing_dir(revised_dir, "revised")
    ensure_existing_dir(golden_dir, "golden")

    revised = load_summary(revised_dir)
    golden = load_summary(golden_dir)

    revised_set = set(revised)
    golden_set = set(golden)
    if revised_set != golden_set:
        missing = sorted(golden_set - revised_set)
        extra = sorted(revised_set - golden_set)
        details: list[str] = []
        if missing:
            details.append(f"missing scenarios: {', '.join(missing)}")
        if extra:
            details.append(f"unexpected scenarios: {', '.join(extra)}")
        fail("scenario set mismatch between summaries: " + "; ".join(details))

    failures: list[str] = []
    for scenario in sorted(revised):
        revised_mean = revised[scenario]["mean"]
        revised_median = revised[scenario]["median"]
        golden_mean = golden[scenario]["mean"]
        golden_median = golden[scenario]["median"]

        mean_delta = delta_pct(revised_mean, golden_mean)
        median_delta = delta_pct(revised_median, golden_median)

        if mean_delta < -threshold or median_delta < -threshold:
            failures.append(
                f"{scenario}: mean_delta={mean_delta:+.2f}% "
                f"(revised={revised_mean:.6f}, golden={golden_mean:.6f}), "
                f"median_delta={median_delta:+.2f}% "
                f"(revised={revised_median:.6f}, golden={golden_median:.6f})"
            )

    if failures:
        for row in failures:
            print(row)
        fail(
            "one or more scenarios exceeded allowed negative delta "
            f"({threshold:.2f}%)"
        )

    print(
        f"ok: no matched scenario exceeded {threshold:.2f}% "
        "negative delta in mean or median"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
