#!/usr/bin/env python3

from __future__ import annotations

import argparse
import pathlib
import sys
from dataclasses import dataclass


@dataclass(frozen=True)
class CoverageTotals:
    regions: float
    functions: float
    lines: float

    @property
    def average(self) -> float:
        return (self.regions + self.functions + self.lines) / 3.0

    @property
    def minimum(self) -> float:
        return min(self.regions, self.functions, self.lines)


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate cargo-llvm-cov summary output against source-coverage thresholds."
    )
    parser.add_argument(
        "--summary",
        type=pathlib.Path,
        default=pathlib.Path("tmp/coverage/coverage-src-summary.txt"),
        help="path to the cargo llvm-cov summary-only text output",
    )
    parser.add_argument(
        "--min-regions",
        type=float,
        default=90.0,
        help="minimum required region coverage percentage",
    )
    parser.add_argument(
        "--min-functions",
        type=float,
        default=90.0,
        help="minimum required function coverage percentage",
    )
    parser.add_argument(
        "--min-lines",
        type=float,
        default=90.0,
        help="minimum required line coverage percentage",
    )
    parser.add_argument(
        "--job-summary-path",
        type=pathlib.Path,
        help="optional GitHub Actions job-summary markdown output path",
    )
    return parser.parse_args()


def parse_totals(summary_path: pathlib.Path) -> CoverageTotals:
    if not summary_path.exists():
        fail(f"error: coverage: summary file not found: {summary_path}")

    total_line: str | None = None
    for line in summary_path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("TOTAL"):
            total_line = stripped

    if total_line is None:
        fail(f"error: coverage: TOTAL row not found in {summary_path}")

    fields = total_line.split()
    if len(fields) < 10:
        fail(f"error: coverage: malformed TOTAL row in {summary_path}: {total_line}")

    try:
        regions = parse_percent(fields[3])
        functions = parse_percent(fields[6])
        lines = parse_percent(fields[9])
    except ValueError as error:
        fail(f"error: coverage: malformed TOTAL row in {summary_path}: {error}")

    return CoverageTotals(regions=regions, functions=functions, lines=lines)


def parse_percent(token: str) -> float:
    if not token.endswith("%"):
        raise ValueError(f"expected percentage token, got {token!r}")
    return float(token[:-1])


def write_job_summary(path: pathlib.Path, totals: CoverageTotals, args: argparse.Namespace) -> None:
    summary = (
        "## Source coverage gate\n\n"
        f"- Regions: `{totals.regions:.2f}%` (min `{args.min_regions:.2f}%`)\n"
        f"- Functions: `{totals.functions:.2f}%` (min `{args.min_functions:.2f}%`)\n"
        f"- Lines: `{totals.lines:.2f}%` (min `{args.min_lines:.2f}%`)\n"
        f"- Average: `{totals.average:.2f}%`\n"
        f"- Minimum metric: `{totals.minimum:.2f}%`\n"
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(summary)


def check_thresholds(totals: CoverageTotals, args: argparse.Namespace) -> None:
    failures: list[str] = []
    if totals.regions < args.min_regions:
        failures.append(f"regions {totals.regions:.2f}% < {args.min_regions:.2f}%")
    if totals.functions < args.min_functions:
        failures.append(f"functions {totals.functions:.2f}% < {args.min_functions:.2f}%")
    if totals.lines < args.min_lines:
        failures.append(f"lines {totals.lines:.2f}% < {args.min_lines:.2f}%")

    if failures:
        fail("error: coverage: source coverage gate failed: " + "; ".join(failures))


def main() -> None:
    args = parse_args()
    totals = parse_totals(args.summary)
    check_thresholds(totals, args)

    if args.job_summary_path is not None:
        write_job_summary(args.job_summary_path, totals, args)

    print(
        "coverage ok: "
        f"regions={totals.regions:.2f}% "
        f"functions={totals.functions:.2f}% "
        f"lines={totals.lines:.2f}% "
        f"average={totals.average:.2f}% "
        f"minimum={totals.minimum:.2f}%"
    )


if __name__ == "__main__":
    main()
