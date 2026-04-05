#!/usr/bin/env python3

"""Print repository line-count statistics with stable category totals."""

from __future__ import annotations

import os
import pathlib
import re


REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent

SKIP_DIR_NAMES = {
    ".git",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "__pycache__",
    "node_modules",
    "target",
    "venv",
}

EXCLUDED_CONTENT_DIR_NAMES = {
    "fixture",
    "fixtures",
    "generated",
    "runs",
    "snapshots",
}

GENERATED_FILE_MARKERS = (
    ".generated.",
    ".gen.",
)

RUST_TEST_ATTRIBUTE_RE = re.compile(
    r"^\s*#\[(?:[A-Za-z_][A-Za-z0-9_]*::)*test(?:\([^]]*\))?\]\s*$",
    re.MULTILINE,
)


def is_excluded_path(path: pathlib.Path, *, allow_fixture_dirs: bool = False) -> bool:
    relative_path = path.relative_to(REPO_ROOT)
    for part in relative_path.parts[:-1]:
        if part in SKIP_DIR_NAMES:
            return True
        if part in EXCLUDED_CONTENT_DIR_NAMES:
            if allow_fixture_dirs and part in {"fixture", "fixtures"}:
                continue
            return True
    name = path.name.lower()
    return any(marker in name for marker in GENERATED_FILE_MARKERS)


def iter_files(base_dir: pathlib.Path, suffixes: tuple[str, ...]) -> list[pathlib.Path]:
    files: list[pathlib.Path] = []
    for root, dir_names, file_names in os.walk(base_dir):
        root_path = pathlib.Path(root)
        dir_names[:] = [
            name
            for name in dir_names
            if name not in SKIP_DIR_NAMES and name not in EXCLUDED_CONTENT_DIR_NAMES
        ]
        for file_name in file_names:
            path = root_path / file_name
            if path.suffix not in suffixes:
                continue
            if is_excluded_path(path):
                continue
            files.append(path)
    files.sort()
    return files


def iter_test_fixture_json_files() -> list[pathlib.Path]:
    base_dir = REPO_ROOT / "tests" / "fixtures"
    files: list[pathlib.Path] = []
    for root, dir_names, file_names in os.walk(base_dir):
        root_path = pathlib.Path(root)
        dir_names[:] = [
            name
            for name in dir_names
            if name not in SKIP_DIR_NAMES and name != "generated"
        ]
        for file_name in file_names:
            path = root_path / file_name
            if path.suffix != ".json":
                continue
            if is_excluded_path(path, allow_fixture_dirs=True):
                continue
            files.append(path)
    files.sort()
    return files


def count_lines(path: pathlib.Path) -> int:
    text = path.read_text(encoding="utf-8")
    if not text:
        return 0
    return text.count("\n") + (0 if text.endswith("\n") else 1)


def total_lines(paths: list[pathlib.Path]) -> int:
    return sum(count_lines(path) for path in paths)


def is_test_file(path: pathlib.Path) -> bool:
    name = path.name
    return name.startswith("test_") or name.endswith("_test.py") or name.endswith("_test.rs")


def main() -> None:
    src_rust_files = iter_files(REPO_ROOT / "src", (".rs",))
    tests_rust_files = iter_files(REPO_ROOT / "tests", (".rs",))

    collateral_files = iter_files(REPO_ROOT / "bench", (".py", ".rs"))
    collateral_files.extend(iter_files(REPO_ROOT / "scripts", (".py", ".rs")))
    collateral_files.sort()
    collateral_test_files = [path for path in collateral_files if is_test_file(path)]
    collateral_code_files = [path for path in collateral_files if not is_test_file(path)]

    markdown_files = [
        path
        for path in iter_files(REPO_ROOT, (".md",))
        if path.relative_to(REPO_ROOT).parts[:2] != ("docs", "exec-plans")
    ]
    exec_plan_files = iter_files(REPO_ROOT / "docs" / "exec-plans", (".md",))
    test_fixture_json_files = iter_test_fixture_json_files()

    tests_rust_count = sum(
        len(RUST_TEST_ATTRIBUTE_RE.findall(path.read_text(encoding="utf-8")))
        for path in tests_rust_files
    )

    src_rust_lines = total_lines(src_rust_files)
    tests_rust_lines = total_lines(tests_rust_files)
    collateral_code_lines = total_lines(collateral_code_files)
    collateral_test_lines = total_lines(collateral_test_files)
    markdown_lines = total_lines(markdown_files)
    exec_plan_lines = total_lines(exec_plan_files)
    test_fixture_json_lines = total_lines(test_fixture_json_files)
    total_code_lines = (
        src_rust_lines
        + tests_rust_lines
        + collateral_code_lines
        + collateral_test_lines
    )

    print(f"Source Rust code: {src_rust_lines:,} lines")
    print(f"Rust tests: {tests_rust_lines:,} lines, {tests_rust_count:,} tests")
    print(f"Collateral code (bench, scripts): {collateral_code_lines:,} lines")
    print(f"Collateral tests (bench, scripts): {collateral_test_lines:,} lines")
    print(f"Test fixtures (JSON): {test_fixture_json_lines:,} lines")
    print(f"Markdown docs (excluding exec-plans): {markdown_lines:,} lines")
    print(f"Exec plans: {exec_plan_lines:,} lines")
    print()
    print(f"Total code: {total_code_lines:,} lines")
    print(f"Total documentation: {markdown_lines:,} lines")


if __name__ == "__main__":
    main()
