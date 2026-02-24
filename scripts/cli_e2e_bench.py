#!/usr/bin/env python3

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import json
import pathlib
import re
import shlex
import shutil
import subprocess
import sys


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_OUTPUT_DIR = pathlib.Path("bench-runs")
DEFAULT_WAVEPEEK_BINARY = pathlib.Path("target/release/wavepeek")
HYPERFINE_SUFFIX = ".hyperfine.json"
WAVEPEEK_SUFFIX = ".wavepeek.json"
TIME_KEYS = ("mean", "stddev", "median", "min", "max")


@dataclass(frozen=True)
class DumpSpec:
    size: str
    waves_path: str


@dataclass(frozen=True)
class TestCase:
    name: str
    command: str
    wavepeek_args: tuple[str, ...]
    dims: dict[str, str]


@dataclass(frozen=True)
class HyperfineMetrics:
    command: str
    mean: float
    stddev: float
    median: float
    min: float
    max: float

    def as_dict(self) -> dict[str, float]:
        return {
            "mean": self.mean,
            "stddev": self.stddev,
            "median": self.median,
            "min": self.min,
            "max": self.max,
        }


DUMP_MATRIX: tuple[DumpSpec, ...] = (
    DumpSpec(size="small", waves_path="/benchmarks/dumps/small.fst"),
    DumpSpec(size="medium", waves_path="/benchmarks/dumps/medium.fst"),
    DumpSpec(size="large", waves_path="/benchmarks/dumps/large.fst"),
)
AT_SIGNAL_COUNTS: tuple[int, ...] = (1, 10, 100, 1000)
AT_TIME_POINTS_PCT: tuple[int, ...] = (5, 50, 95)
CHANGE_SIGNAL_COUNTS: tuple[int, ...] = (1, 10, 100)
CHANGE_WINDOW_POSITIONS_PCT: tuple[int, ...] = (5, 50, 95)
CHANGE_WINDOW_SIZES: tuple[int, ...] = (2000, 4000, 8000)
CHANGE_TRIGGERS: tuple[tuple[str, str], ...] = (
    ("posedge_clk", "posedge clk"),
    ("any", "*"),
    ("signal", "signal"),
)


def build_test_catalog() -> list[TestCase]:
    tests: list[TestCase] = []

    for dump in DUMP_MATRIX:
        tests.append(
            TestCase(
                name=f"info_size_{dump.size}",
                command="info",
                wavepeek_args=("info", "--waves", dump.waves_path),
                dims={"size": dump.size},
            )
        )

    for dump in DUMP_MATRIX:
        for signal_count in AT_SIGNAL_COUNTS:
            for time_pct in AT_TIME_POINTS_PCT:
                tests.append(
                    TestCase(
                        name=(
                            f"at_size_{dump.size}"
                            f"_signals_{signal_count}"
                            f"_time_{time_pct}"
                        ),
                        command="at",
                        wavepeek_args=(
                            "at",
                            "--waves",
                            dump.waves_path,
                            "--scope",
                            "top",
                            "--signals",
                            f"signal_set_{signal_count}",
                            "--time",
                            f"{time_pct}%",
                        ),
                        dims={
                            "size": dump.size,
                            "signals": str(signal_count),
                            "time_pct": str(time_pct),
                        },
                    )
                )

    for dump in DUMP_MATRIX:
        for signal_count in CHANGE_SIGNAL_COUNTS:
            for window_position_pct in CHANGE_WINDOW_POSITIONS_PCT:
                for window_size in CHANGE_WINDOW_SIZES:
                    for trigger_slug, trigger_value in CHANGE_TRIGGERS:
                        tests.append(
                            TestCase(
                                name=(
                                    f"change_size_{dump.size}"
                                    f"_signals_{signal_count}"
                                    f"_window_pos_{window_position_pct}"
                                    f"_window_size_{window_size}"
                                    f"_trigger_{trigger_slug}"
                                ),
                                command="change",
                                wavepeek_args=(
                                    "change",
                                    "--waves",
                                    dump.waves_path,
                                    "--scope",
                                    "top",
                                    "--signals",
                                    f"signal_set_{signal_count}",
                                    "--window-position",
                                    f"{window_position_pct}%",
                                    "--window-size",
                                    str(window_size),
                                    "--when",
                                    trigger_value,
                                ),
                                dims={
                                    "size": dump.size,
                                    "signals": str(signal_count),
                                    "window_position_pct": str(window_position_pct),
                                    "window_size": str(window_size),
                                    "trigger": trigger_value,
                                },
                            )
                        )

    return tests


def select_tests(tests: list[TestCase], pattern: str | None) -> list[TestCase]:
    if pattern is None:
        return sorted(tests, key=lambda item: item.name)
    regex = re.compile(pattern)
    return sorted([test for test in tests if regex.search(test.name)], key=lambda item: item.name)


def ensure_release_binary(binary_path: pathlib.Path) -> None:
    if binary_path.exists():
        return
    print(f"info: release binary not found at {binary_path}; building with cargo --release")
    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=REPO_ROOT,
        check=False,
    )
    if result.returncode != 0:
        raise SystemExit("error: run: failed to build release binary via `cargo build --release`")
    if not binary_path.exists():
        raise SystemExit(
            "error: run: release build completed but binary is still missing at "
            f"{binary_path}"
        )


def run_tests(
    tests: list[TestCase],
    run_dir: pathlib.Path,
    *,
    runs: int,
    warmup: int,
) -> None:
    if shutil.which("hyperfine") is None:
        raise SystemExit("error: run: `hyperfine` is not available in PATH")

    for index, test in enumerate(tests, start=1):
        print(f"[{index}/{len(tests)}] {test.name}")
        wavepeek_path = run_dir / f"{test.name}{WAVEPEEK_SUFFIX}"
        hyperfine_path = run_dir / f"{test.name}{HYPERFINE_SUFFIX}"

        run_single_test(test, wavepeek_path=wavepeek_path, hyperfine_path=hyperfine_path, runs=runs, warmup=warmup)


def run_single_test(
    test: TestCase,
    *,
    wavepeek_path: pathlib.Path,
    hyperfine_path: pathlib.Path,
    runs: int,
    warmup: int,
) -> None:
    dummy_command = ["echo", "dummy args", *test.wavepeek_args]
    command_string = shlex.join(dummy_command)

    wavepeek_result = subprocess.run(
        dummy_command,
        check=False,
        capture_output=True,
        text=True,
    )
    data_value, warning_value = parse_data_warning_from_stdout(wavepeek_result.stdout)
    wavepeek_payload = {
        "test_name": test.name,
        "command": test.command,
        "dims": test.dims,
        "invoked_utc": utc_now_iso(),
        "exit_code": wavepeek_result.returncode,
        "stdout": wavepeek_result.stdout,
        "stderr": wavepeek_result.stderr,
        "data": data_value,
        "warning": warning_value,
        "benchmark_command": command_string,
        "mode": "dummy",
    }
    write_json(wavepeek_path, wavepeek_payload)

    hyperfine_command = [
        "hyperfine",
        "--warmup",
        str(warmup),
        "--runs",
        str(runs),
        "--export-json",
        str(hyperfine_path),
        command_string,
    ]
    result = subprocess.run(hyperfine_command, check=False)
    if result.returncode != 0:
        raise SystemExit(
            "error: run: hyperfine failed for test "
            f"{test.name} with command `{command_string}`"
        )


def parse_data_warning_from_stdout(stdout: str) -> tuple[object, object]:
    trimmed = stdout.strip()
    if not trimmed:
        return None, None
    try:
        payload = json.loads(trimmed)
    except json.JSONDecodeError:
        return trimmed, None
    if isinstance(payload, dict):
        return payload.get("data"), payload.get("warning")
    return payload, None


def compare_runs(
    *,
    revised_dir: pathlib.Path,
    golden_dir: pathlib.Path,
    max_negative_delta_pct: float,
    metric: str,
    required_equal_fields: tuple[str, ...],
) -> list[str]:
    revised_hyperfine = load_hyperfine_map(revised_dir)
    golden_hyperfine = load_hyperfine_map(golden_dir)
    revised_wavepeek = load_wavepeek_map(revised_dir)
    golden_wavepeek = load_wavepeek_map(golden_dir)

    issues: list[str] = []
    revised_test_names = sorted(set(revised_hyperfine) | set(revised_wavepeek))
    for test_name in revised_test_names:
        revised_metrics = revised_hyperfine.get(test_name)
        golden_metrics = golden_hyperfine.get(test_name)
        if revised_metrics is None:
            issues.append(f"{test_name}: missing revised hyperfine artifact")
        elif golden_metrics is not None:
            revised_value = revised_metrics.as_dict()[metric]
            golden_value = golden_metrics.as_dict()[metric]
            delta_pct = compute_delta_pct(current=revised_value, golden=golden_value)
            if delta_pct is not None and delta_pct < -max_negative_delta_pct:
                issues.append(
                    f"{test_name}: regression {metric}={revised_value:.6f}s vs {golden_value:.6f}s ({delta_pct:+.2f}%)"
                )

        if not required_equal_fields:
            continue

        has_golden_counterpart = (
            test_name in golden_hyperfine or test_name in golden_wavepeek
        )
        if not has_golden_counterpart:
            continue

        revised_payload = revised_wavepeek.get(test_name)
        golden_payload = golden_wavepeek.get(test_name)
        if revised_payload is None or golden_payload is None:
            issues.append(
                f"{test_name}: missing wavepeek artifact for required field checks"
            )
            continue

        for field in required_equal_fields:
            if field not in revised_payload or field not in golden_payload:
                issues.append(f"{test_name}: required field '{field}' is missing")
                continue
            if revised_payload[field] != golden_payload[field]:
                issues.append(f"{test_name}: field '{field}' mismatch")

    return issues


def render_report(run_dir: pathlib.Path, compare_dir: pathlib.Path | None = None) -> str:
    hyperfine_map = load_hyperfine_map(run_dir)
    wavepeek_map = load_wavepeek_map(run_dir)
    compare_map = load_hyperfine_map(compare_dir) if compare_dir is not None else {}
    test_names = sorted(set(hyperfine_map) | set(wavepeek_map))

    lines: list[str] = []
    lines.append(f"# CLI E2E Benchmark Run: {run_dir.name}")
    lines.append("")
    lines.append(f"- Generated at (UTC): {utc_now_iso()}")
    lines.append(f"- Run directory: `{run_dir}`")
    lines.append(f"- Tests found: {len(test_names)}")
    if compare_dir is not None:
        lines.append(f"- Compare baseline: `{compare_dir}`")
    lines.append("")
    lines.append("## Benchmark Results")
    lines.append("")
    lines.append("| test | command | dimensions | mean_s | stddev_s | median_s | min_s | max_s |")
    lines.append("| --- | --- | --- | --- | --- | --- | --- | --- |")

    for test_name in test_names:
        metrics = hyperfine_map.get(test_name)
        wavepeek_payload = wavepeek_map.get(test_name)
        if metrics is None:
            command = infer_command(test_name, wavepeek_payload)
            dims_text = format_dimensions(wavepeek_payload)
            lines.append(f"| {escape_md(test_name)} | {escape_md(command)} | {escape_md(dims_text)} | - | - | - | - | - |")
            continue

        baseline = compare_map.get(test_name)
        baseline_values = baseline.as_dict() if baseline is not None else {}
        lines.append(
            "| "
            + " | ".join(
                [
                    escape_md(test_name),
                    escape_md(infer_command(test_name, wavepeek_payload)),
                    escape_md(format_dimensions(wavepeek_payload)),
                    format_metric(metrics.mean, baseline_values.get("mean")),
                    format_metric(metrics.stddev, baseline_values.get("stddev")),
                    format_metric(metrics.median, baseline_values.get("median")),
                    format_metric(metrics.min, baseline_values.get("min")),
                    format_metric(metrics.max, baseline_values.get("max")),
                ]
            )
            + " |"
        )

    lines.append("")
    lines.append("## Wavepeek Artifacts")
    lines.append("")
    lines.append("| test | exit_code | data | warning | file |")
    lines.append("| --- | --- | --- | --- | --- |")
    for test_name in test_names:
        payload = wavepeek_map.get(test_name)
        if payload is None:
            lines.append(f"| {escape_md(test_name)} | - | - | - | - |")
            continue
        exit_code = payload.get("exit_code", "-")
        data_text = compact_json(payload.get("data"))
        warning_text = compact_json(payload.get("warning"))
        file_name = f"{test_name}{WAVEPEEK_SUFFIX}"
        lines.append(
            f"| {escape_md(test_name)} | {escape_md(exit_code)} | {escape_md(data_text)} | {escape_md(warning_text)} | {escape_md(file_name)} |"
        )

    lines.append("")
    return "\n".join(lines)


def load_hyperfine_map(run_dir: pathlib.Path | None) -> dict[str, HyperfineMetrics]:
    if run_dir is None:
        return {}
    metrics_map: dict[str, HyperfineMetrics] = {}
    for path in sorted(run_dir.glob(f"*{HYPERFINE_SUFFIX}")):
        test_name = path.name[: -len(HYPERFINE_SUFFIX)]
        metrics_map[test_name] = load_hyperfine_metrics(path)
    return metrics_map


def load_hyperfine_metrics(path: pathlib.Path) -> HyperfineMetrics:
    payload = json.loads(path.read_text(encoding="utf-8"))
    results = payload.get("results")
    if not isinstance(results, list) or not results:
        raise SystemExit(f"error: report: malformed hyperfine result file {path}")
    first = results[0]
    for key in TIME_KEYS:
        if key not in first:
            raise SystemExit(f"error: report: hyperfine result missing `{key}` in {path}")

    mean = require_float_metric(first, "mean", path)
    stddev = optional_float_metric(first, "stddev", path, default=0.0)
    median = require_float_metric(first, "median", path)
    minimum = require_float_metric(first, "min", path)
    maximum = require_float_metric(first, "max", path)

    return HyperfineMetrics(
        command=str(first.get("command", "")),
        mean=mean,
        stddev=stddev,
        median=median,
        min=minimum,
        max=maximum,
    )


def require_float_metric(payload: dict[str, object], key: str, source_path: pathlib.Path) -> float:
    value = payload.get(key)
    if value is None:
        raise SystemExit(f"error: report: hyperfine result `{key}` is null in {source_path}")
    if not isinstance(value, (int, float, str)):
        raise SystemExit(
            f"error: report: hyperfine result `{key}` has unsupported type in {source_path}: {type(value).__name__}"
        )
    try:
        return float(value)
    except (TypeError, ValueError) as error:
        raise SystemExit(
            f"error: report: hyperfine result `{key}` is not numeric in {source_path}: {error}"
        ) from error


def optional_float_metric(
    payload: dict[str, object],
    key: str,
    source_path: pathlib.Path,
    *,
    default: float,
) -> float:
    value = payload.get(key)
    if value is None:
        return default
    if not isinstance(value, (int, float, str)):
        raise SystemExit(
            f"error: report: hyperfine result `{key}` has unsupported type in {source_path}: {type(value).__name__}"
        )
    try:
        return float(value)
    except (TypeError, ValueError) as error:
        raise SystemExit(
            f"error: report: hyperfine result `{key}` is not numeric in {source_path}: {error}"
        ) from error


def load_wavepeek_map(run_dir: pathlib.Path) -> dict[str, dict[str, object]]:
    payload_map: dict[str, dict[str, object]] = {}
    for path in sorted(run_dir.glob(f"*{WAVEPEEK_SUFFIX}")):
        test_name = path.name[: -len(WAVEPEEK_SUFFIX)]
        payload_map[test_name] = json.loads(path.read_text(encoding="utf-8"))
    return payload_map


def compute_delta_pct(*, current: float, golden: float) -> float | None:
    if golden == 0:
        return None
    return ((golden - current) / golden) * 100.0


def format_metric(current: float, golden: float | None) -> str:
    base = f"{current:.6f}"
    if golden is None:
        return base
    delta_pct = compute_delta_pct(current=current, golden=golden)
    if delta_pct is None:
        return f"{base} (n/a)"
    return f"{base} ({delta_pct:+.2f}%)"


def write_report(run_dir: pathlib.Path, compare_dir: pathlib.Path | None = None) -> pathlib.Path:
    report = render_report(run_dir, compare_dir)
    report_path = run_dir / "README.md"
    report_path.write_text(report, encoding="utf-8")
    return report_path


def resolve_run_directory(run_dir: pathlib.Path | None, out_dir: pathlib.Path) -> pathlib.Path:
    if run_dir is not None:
        run_dir.mkdir(parents=True, exist_ok=True)
        return run_dir
    out_dir.mkdir(parents=True, exist_ok=True)
    for attempt in range(1000):
        stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d_%H-%M-%S-%fZ")
        suffix = "" if attempt == 0 else f"-{attempt}"
        created = out_dir / f"{stamp}{suffix}"
        try:
            created.mkdir(parents=True, exist_ok=False)
            return created
        except FileExistsError:
            continue
    raise SystemExit(
        f"error: run: failed to create a unique run directory inside {out_dir}"
    )


def write_json(path: pathlib.Path, payload: object) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")


def format_dimensions(payload: dict[str, object] | None) -> str:
    if payload is None:
        return "-"
    dims = payload.get("dims")
    if not isinstance(dims, dict) or not dims:
        return "-"
    pairs = [f"{key}={dims[key]}" for key in sorted(dims)]
    return ", ".join(pairs)


def infer_command(test_name: str, payload: dict[str, object] | None) -> str:
    if payload is not None and isinstance(payload.get("command"), str):
        return str(payload["command"])
    return test_name.split("_", 1)[0]


def compact_json(value: object) -> str:
    text = json.dumps(value, ensure_ascii=True, separators=(",", ":"))
    if len(text) > 80:
        return text[:77] + "..."
    return text


def escape_md(value: object) -> str:
    text = str(value)
    text = text.replace("\n", " ")
    return text.replace("|", "\\|")


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def normalize_path(value: str) -> pathlib.Path:
    return pathlib.Path(value).expanduser().resolve()


def normalize_binary_path(value: str) -> pathlib.Path:
    path = pathlib.Path(value)
    if path.is_absolute():
        return path
    return (REPO_ROOT / path).resolve()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="CLI E2E benchmark harness for wavepeek")
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list", help="show available benchmark tests")
    list_parser.add_argument("--filter", type=str, default=None, help="regex filter for test names")

    run_parser = subparsers.add_parser("run", help="execute benchmarks and write artifacts")
    run_parser.add_argument("--run-dir", type=str, default=None, help="existing/new run directory")
    run_parser.add_argument("--out-dir", type=str, default=str(DEFAULT_OUTPUT_DIR), help="parent directory for timestamped runs")
    run_parser.add_argument("--filter", type=str, default=None, help="regex filter for test names")
    run_parser.add_argument("--compare", type=str, default=None, help="baseline run directory for README deltas")
    run_parser.add_argument("--runs", type=int, default=5, help="hyperfine measured runs per test")
    run_parser.add_argument("--warmup", type=int, default=1, help="hyperfine warmup runs per test")
    run_parser.add_argument(
        "--wavepeek-bin",
        type=str,
        default=str(DEFAULT_WAVEPEEK_BINARY),
        help="path to release wavepeek binary checked before running",
    )

    report_parser = subparsers.add_parser("report", help="regenerate README from run artifacts")
    report_parser.add_argument("--run-dir", required=True, type=str, help="run directory")
    report_parser.add_argument("--compare", type=str, default=None, help="baseline run directory for README deltas")

    compare_parser = subparsers.add_parser("compare", help="compare revised run against golden run")
    compare_parser.add_argument("--revised", required=True, type=str, help="revised run directory")
    compare_parser.add_argument("--golden", required=True, type=str, help="golden run directory")
    compare_parser.add_argument(
        "--max-negative-delta-pct",
        required=True,
        type=float,
        help="fail when delta is below negative threshold",
    )
    compare_parser.add_argument(
        "--metric",
        choices=("mean", "median", "min", "max", "stddev"),
        default="median",
        help="timing metric for regression checks",
    )
    compare_parser.add_argument(
        "--require-equal-field",
        choices=("data", "warning"),
        action="append",
        default=[],
        help="wavepeek JSON field that must match between revised and golden",
    )

    return parser


def cmd_list(filter_pattern: str | None) -> int:
    tests = select_tests(build_test_catalog(), filter_pattern)
    for test in tests:
        dims_text = ", ".join(f"{key}={value}" for key, value in sorted(test.dims.items()))
        print(f"{test.name}\t{test.command}\t{dims_text}")
    print(f"total={len(tests)}")
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    if args.runs < 1:
        raise SystemExit("error: run: --runs must be >= 1")
    if args.warmup < 0:
        raise SystemExit("error: run: --warmup must be >= 0")

    tests = select_tests(build_test_catalog(), args.filter)
    if not tests:
        raise SystemExit("error: run: no tests matched the provided filter")

    binary_path = normalize_binary_path(args.wavepeek_bin)
    ensure_release_binary(binary_path)

    explicit_run_dir = normalize_path(args.run_dir) if args.run_dir is not None else None
    out_dir = normalize_path(args.out_dir)
    compare_dir = normalize_path(args.compare) if args.compare is not None else None
    if compare_dir is not None and (not compare_dir.exists() or not compare_dir.is_dir()):
        raise SystemExit(
            f"error: run: compare directory does not exist: {compare_dir}"
        )
    run_dir = resolve_run_directory(explicit_run_dir, out_dir)

    run_tests(tests, run_dir, runs=args.runs, warmup=args.warmup)
    report_path = write_report(run_dir, compare_dir)
    print(f"info: run artifacts written to {run_dir}")
    print(f"info: report updated at {report_path}")
    return 0


def cmd_report(run_dir_arg: str, compare_arg: str | None) -> int:
    run_dir = normalize_path(run_dir_arg)
    if not run_dir.exists() or not run_dir.is_dir():
        raise SystemExit(f"error: report: run directory does not exist: {run_dir}")
    compare_dir = normalize_path(compare_arg) if compare_arg is not None else None
    if compare_dir is not None and (not compare_dir.exists() or not compare_dir.is_dir()):
        raise SystemExit(
            f"error: report: compare directory does not exist: {compare_dir}"
        )
    report_path = write_report(run_dir, compare_dir)
    print(f"info: report updated at {report_path}")
    return 0


def cmd_compare(args: argparse.Namespace) -> int:
    revised_dir = normalize_path(args.revised)
    golden_dir = normalize_path(args.golden)
    for label, path in (("revised", revised_dir), ("golden", golden_dir)):
        if not path.exists() or not path.is_dir():
            raise SystemExit(f"error: compare: {label} run directory does not exist: {path}")
    if args.max_negative_delta_pct < 0:
        raise SystemExit(
            "error: compare: --max-negative-delta-pct must be non-negative"
        )

    revised_hyperfine = load_hyperfine_map(revised_dir)
    golden_hyperfine = load_hyperfine_map(golden_dir)
    missing_golden_tests = sorted(set(revised_hyperfine) - set(golden_hyperfine))
    if missing_golden_tests:
        print(
            "warning: compare: no golden counterpart for tests: "
            + ", ".join(missing_golden_tests),
            file=sys.stderr,
        )

    issues = compare_runs(
        revised_dir=revised_dir,
        golden_dir=golden_dir,
        max_negative_delta_pct=args.max_negative_delta_pct,
        metric=args.metric,
        required_equal_fields=tuple(args.require_equal_field),
    )
    if issues:
        print("error: compare: regression checks failed:", file=sys.stderr)
        for issue in issues:
            print(f"  - {issue}", file=sys.stderr)
        return 1
    print("info: compare: all checks passed")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)

    if args.command == "list":
        return cmd_list(args.filter)
    if args.command == "run":
        return cmd_run(args)
    if args.command == "report":
        return cmd_report(args.run_dir, args.compare)
    if args.command == "compare":
        return cmd_compare(args)
    raise SystemExit(f"error: unsupported command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
