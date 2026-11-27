from __future__ import annotations

import contextlib
import io
import json
import pathlib
import sys
import tempfile
import unittest
from unittest import mock

import generate_catalog


class GenerateCatalogTest(unittest.TestCase):
    def test_catalogs_keep_every_scenario_with_the_expected_fixture_paths(self) -> None:
        source = generate_catalog.DEFAULT_SOURCE
        base = json.loads(source.read_text(encoding="utf-8"))["tests"]
        for target in ("vcd", "fsdb", "shm", "wlf"):
            with self.subTest(target=target):
                actual = json.loads(generate_catalog.generate_catalog(source, target))["tests"]
                self.assertEqual(len(actual), len(base))
                for original, derived in zip(base, actual, strict=True):
                    old_path = original["meta"]["waves"]
                    path = str(pathlib.PurePosixPath(old_path).with_suffix(f".{target}"))
                    if target != "vcd":
                        path = path.replace("tests/fixtures/generated/", f"tests/fixtures/{target}/")
                    command = original["command"].copy()
                    self.assertEqual(command[command.index("--waves") + 1], old_path)
                    command[command.index("--waves") + 1] = path
                    self.assertEqual(
                        derived,
                        {**original, "command": command, "meta": {**original["meta"], "waves": path}},
                    )

    def test_run_snapshots_are_independent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "base.json"
            output_root = root / "tmp/bench-catalogs"
            paths = []
            with (
                mock.patch.object(generate_catalog, "OUTPUT_ROOT", output_root),
                mock.patch.object(sys, "argv", ["generate_catalog.py", "vcd", "--source", str(source)]),
            ):
                for name in ("before", "after"):
                    source.write_text(json.dumps({"tests": [f"{name}.fst"]}))
                    stdout = io.StringIO()
                    with contextlib.redirect_stdout(stdout):
                        generate_catalog.main()
                    paths.append(pathlib.Path(stdout.getvalue().strip()))

            self.assertNotEqual(paths[0], paths[1])
            for path, name in zip(paths, ("before", "after"), strict=True):
                self.assertTrue(path.is_relative_to(output_root))
                self.assertEqual(json.loads(path.read_text()), {"tests": [f"{name}.vcd"]})

    def test_invalid_sources_fail(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = pathlib.Path(directory) / "base.json"
            with self.assertRaisesRegex(SystemExit, "failed to read"):
                generate_catalog.generate_catalog(source, "shm")
            source.write_text('{"tests": ["sample.fst"]', encoding="utf-8")
            with self.assertRaisesRegex(SystemExit, "invalid JSON"):
                generate_catalog.generate_catalog(source, "shm")
            source.write_text('{"tests": []}', encoding="utf-8")
            with self.assertRaisesRegex(SystemExit, "no .fst suffixes found"):
                generate_catalog.generate_catalog(source, "shm")


if __name__ == "__main__":
    unittest.main()
