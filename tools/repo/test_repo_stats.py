from __future__ import annotations

import contextlib
import importlib.util
import io
import pathlib
import re
import sys
import tempfile
import unittest
from unittest import mock

MODULE_PATH = pathlib.Path(__file__).with_name("repo_stats.py")
SPEC = importlib.util.spec_from_file_location("repo_stats", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
repo_stats = importlib.util.module_from_spec(SPEC)
sys.modules["repo_stats"] = repo_stats
SPEC.loader.exec_module(repo_stats)


class RepoStatsTests(unittest.TestCase):
    def test_iter_files_excludes_disposable_work_directories(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = pathlib.Path(temp)
            kept = root / "docs" / "kept.md"
            ignored = [
                root / "tmp" / "ignored.md",
                root / ".worktrees" / "ignored.md",
            ]
            kept.parent.mkdir(parents=True)
            kept.write_text("kept\n", encoding="utf-8")
            for path in ignored:
                path.parent.mkdir(parents=True)
                path.write_text("ignored\n", encoding="utf-8")

            with mock.patch.object(repo_stats, "REPO_ROOT", root):
                files = repo_stats.iter_files(root, (".md",))

            self.assertEqual(files, [kept])

    def test_report_totals_include_code_docs_and_json_fixtures(self) -> None:
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            repo_stats.main()

        report = output.getvalue()
        self.assertNotIn("WIP tracker", report)
        values = {
            match.group("label"): int(match.group("lines").replace(",", ""))
            for match in re.finditer(
                r"^(?P<label>.+): (?P<lines>[\d,]+) lines(?:,.*)?$",
                report,
                re.MULTILINE,
            )
        }
        code_total = (
            values["Source Rust code"]
            + values["Rust tests"]
            + values["Collateral code (bench, tools)"]
            + values["Collateral tests (bench, tools)"]
        )
        self.assertEqual(
            values["Total code (source, tests, bench, tools)"], code_total
        )
        self.assertEqual(
            values["Total lines"],
            code_total + values["Test fixtures (JSON)"] + values["Markdown docs"],
        )


if __name__ == "__main__":
    unittest.main()
