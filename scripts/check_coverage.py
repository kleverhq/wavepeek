#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import pathlib
import sys
from dataclasses import dataclass


@dataclass(frozen=True)
class MetricCounts:
    count: int
    covered: int

    def __add__(self, other: "MetricCounts") -> "MetricCounts":
        return MetricCounts(count=self.count + other.count, covered=self.covered + other.covered)

    @property
    def percent(self) -> float:
        if self.count == 0:
            return 100.0
        return (self.covered / self.count) * 100.0


@dataclass(frozen=True)
class CoverageTotals:
    regions: MetricCounts
    functions: MetricCounts
    lines: MetricCounts

    @property
    def average(self) -> float:
        return (self.regions.percent + self.functions.percent + self.lines.percent) / 3.0

    @property
    def minimum(self) -> float:
        return min(self.regions.percent, self.functions.percent, self.lines.percent)


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate cargo-llvm-cov JSON summary output against source-coverage thresholds."
    )
    parser.add_argument(
        "--summary-json",
        type=pathlib.Path,
        default=pathlib.Path("tmp/coverage/coverage-src-summary.json"),
        help="path to the cargo llvm-cov --summary-only --json output",
    )
    parser.add_argument(
        "--repo-root",
        type=pathlib.Path,
        default=pathlib.Path("."),
        help="repository root used to scope coverage paths",
    )
    parser.add_argument(
        "--scope-prefix",
        default="src",
        help="top-level repository subdirectory to include in coverage aggregation",
    )
    parser.add_argument(
        "--exclude-dir",
        action="append",
        default=["tests"],
        help="path component to exclude within the scoped tree (repeatable)",
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
        "--markdown-output",
        type=pathlib.Path,
        help="optional path to write a markdown coverage summary",
    )
    return parser.parse_args()


def load_export(summary_path: pathlib.Path) -> dict:
    if not summary_path.exists():
        fail(f"error: coverage: summary file not found: {summary_path}")

    try:
        payload = json.loads(summary_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"error: coverage: summary JSON is invalid: {error}")

    if not isinstance(payload, dict):
        fail(f"error: coverage: expected top-level JSON object in {summary_path}")

    data = payload.get("data")
    if not isinstance(data, list) or not data:
        fail(f"error: coverage: expected non-empty 'data' array in {summary_path}")

    return payload


def aggregate_totals(payload: dict, *, repo_root: pathlib.Path, scope_prefix: str, exclude_dirs: set[str]) -> CoverageTotals:
    scoped_root = repo_root / scope_prefix
    region_totals = MetricCounts(count=0, covered=0)
    function_totals = MetricCounts(count=0, covered=0)
    line_totals = MetricCounts(count=0, covered=0)
    matched_files = 0

    for export_block in payload["data"]:
        if not isinstance(export_block, dict):
            fail("error: coverage: expected export block objects under 'data'")
        files = export_block.get("files")
        if not isinstance(files, list):
            fail("error: coverage: expected each export block to contain a 'files' array")

        for file_entry in files:
            if not isinstance(file_entry, dict):
                fail("error: coverage: expected file entries to be JSON objects")
            filename = file_entry.get("filename")
            summary = file_entry.get("summary")
            if not isinstance(filename, str) or not isinstance(summary, dict):
                fail("error: coverage: each file entry must contain string 'filename' and object 'summary'")

            file_path = normalize_path(pathlib.Path(filename), repo_root)
            try:
                relative_path = file_path.relative_to(repo_root)
            except ValueError:
                continue

            if not is_scoped_path(relative_path, scoped_root.relative_to(repo_root), exclude_dirs):
                continue

            region_totals += parse_metric(summary, "regions")
            function_totals += parse_metric(summary, "functions")
            line_totals += parse_metric(summary, "lines")
            matched_files += 1

    if matched_files == 0:
        fail(
            "error: coverage: no files matched scope "
            f"'{scope_prefix}/**' after excluding directories {sorted(exclude_dirs)}"
        )

    return CoverageTotals(regions=region_totals, functions=function_totals, lines=line_totals)


def normalize_path(path: pathlib.Path, repo_root: pathlib.Path) -> pathlib.Path:
    if path.is_absolute():
        return path.resolve(strict=False)
    return (repo_root / path).resolve(strict=False)


def is_scoped_path(relative_path: pathlib.Path, scope_root: pathlib.Path, exclude_dirs: set[str]) -> bool:
    parts = relative_path.parts
    scope_parts = scope_root.parts
    if parts[: len(scope_parts)] != scope_parts:
        return False
    return not any(part in exclude_dirs for part in parts)


def parse_metric(summary: dict, name: str) -> MetricCounts:
    metric = summary.get(name)
    if not isinstance(metric, dict):
        fail(f"error: coverage: expected metric object for {name!r}")

    count = metric.get("count")
    covered = metric.get("covered")
    if not isinstance(count, int) or not isinstance(covered, int):
        fail(f"error: coverage: metric {name!r} must contain integer 'count' and 'covered'")
    if count < 0 or covered < 0 or covered > count:
        fail(f"error: coverage: metric {name!r} has invalid counts: count={count}, covered={covered}")

    return MetricCounts(count=count, covered=covered)


def write_markdown_summary(path: pathlib.Path, totals: CoverageTotals, args: argparse.Namespace) -> None:
    excluded = ", ".join(f"`**/{name}/**`" for name in sorted(set(args.exclude_dir)))
    summary = (
        "## Source coverage gate\n\n"
        f"- Scope: `{args.scope_prefix}/**`"
        + (f" excluding {excluded}\n" if excluded else "\n")
        + f"- Regions: `{totals.regions.percent:.2f}%` (min `{args.min_regions:.2f}%`)\n"
        + f"- Functions: `{totals.functions.percent:.2f}%` (min `{args.min_functions:.2f}%`)\n"
        + f"- Lines: `{totals.lines.percent:.2f}%` (min `{args.min_lines:.2f}%`)\n"
        + f"- Average: `{totals.average:.2f}%`\n"
        + f"- Minimum metric: `{totals.minimum:.2f}%`\n"
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(summary, encoding="utf-8")


def check_thresholds(totals: CoverageTotals, args: argparse.Namespace) -> None:
    failures: list[str] = []
    if totals.regions.percent < args.min_regions:
        failures.append(f"regions {totals.regions.percent:.2f}% < {args.min_regions:.2f}%")
    if totals.functions.percent < args.min_functions:
        failures.append(f"functions {totals.functions.percent:.2f}% < {args.min_functions:.2f}%")
    if totals.lines.percent < args.min_lines:
        failures.append(f"lines {totals.lines.percent:.2f}% < {args.min_lines:.2f}%")

    if failures:
        fail("error: coverage: source coverage gate failed: " + "; ".join(failures))


def main() -> None:
    args = parse_args()
    repo_root = args.repo_root.resolve(strict=False)
    payload = load_export(args.summary_json)
    totals = aggregate_totals(
        payload,
        repo_root=repo_root,
        scope_prefix=args.scope_prefix,
        exclude_dirs=set(args.exclude_dir),
    )

    if args.markdown_output is not None:
        write_markdown_summary(args.markdown_output, totals, args)

    check_thresholds(totals, args)

    print(
        "coverage ok: "
        f"scope={args.scope_prefix}/** "
        f"regions={totals.regions.percent:.2f}% "
        f"functions={totals.functions.percent:.2f}% "
        f"lines={totals.lines.percent:.2f}% "
        f"average={totals.average:.2f}% "
        f"minimum={totals.minimum:.2f}%"
    )


if __name__ == "__main__":
    main()
