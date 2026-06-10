from __future__ import annotations

import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest

import yaml

MODULE_PATH = pathlib.Path(__file__).with_name("prepare_mkdocs.py")
SPEC = importlib.util.spec_from_file_location("prepare_mkdocs", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
prepare_mkdocs = importlib.util.module_from_spec(SPEC)
sys.modules["prepare_mkdocs"] = prepare_mkdocs
SPEC.loader.exec_module(prepare_mkdocs)


class PrepareMkdocsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temp.name)
        self.export = self.root / "export"
        self.output = self.root / "mkdocs-src"
        self.config = self.root / "mkdocs.yml"
        self.export.mkdir()
        self.write_topic(
            "intro",
            "Introduction",
            "intro",
            description="Start here.",
            see_also=["commands/change"],
        )
        self.write_topic(
            "commands/change",
            "Change command",
            "commands",
            description="Find changes.",
        )
        self.write_manifest(
            [
                {
                    "id": "intro",
                    "title": "Introduction",
                    "description": "Start here.",
                    "section": "intro",
                    "see_also": ["commands/change"],
                },
                {
                    "id": "commands/change",
                    "title": "Change command",
                    "description": "Find changes.",
                    "section": "commands",
                },
            ]
        )

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_manifest(self, topics: list[dict[str, object]], **overrides: object) -> None:
        manifest = {
            "kind": "wavepeek-docs-export",
            "export_format_version": 1,
            "cli_name": "wavepeek",
            "cli_version": "0.5.0",
            "topics": topics,
        }
        manifest.update(overrides)
        (self.export / "manifest.json").write_text(
            json.dumps(manifest), encoding="utf-8"
        )

    def write_topic(
        self,
        topic_id: str,
        title: str,
        section: str,
        *,
        description: str | None = None,
        see_also: list[str] | None = None,
    ) -> None:
        front: dict[str, object] = {
            "id": topic_id,
            "title": title,
            "section": section,
        }
        if description is not None:
            front["description"] = description
        if see_also is not None:
            front["see_also"] = see_also
        relpath = pathlib.Path(*topic_id.split("/")).with_suffix(".md")
        path = self.export / relpath
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            "---\n"
            + yaml.safe_dump(front, sort_keys=False)
            + "---\n"
            + f"# {title}\n\nBody.\n",
            encoding="utf-8",
        )

    def prepare(self, *, force: bool = True, version: str = "0.5.0"):
        return prepare_mkdocs.prepare_tree(
            self.export, self.output, self.config, version, force=force
        )

    def test_prepares_tree_nav_and_description_front_matter(self) -> None:
        self.prepare()

        self.assertTrue((self.output / "index.md").is_file())
        self.assertTrue((self.output / "commands" / "change.md").is_file())
        self.assertFalse((self.output / "manifest.json").exists())

        intro = (self.output / "index.md").read_text(encoding="utf-8")
        self.assertIn("description: Start here.", intro)
        self.assertNotIn("summary:", intro)

        config = yaml.safe_load(self.config.read_text(encoding="utf-8"))
        self.assertEqual(config["docs_dir"], "mkdocs-src")
        self.assertEqual(config["site_dir"], "mkdocs-site")
        self.assertIn({"Introduction": "index.md"}, config["nav"])
        self.assertIn(
            {"Commands": [{"Change command": "commands/change.md"}]},
            config["nav"],
        )

    def test_rejects_legacy_summary_metadata(self) -> None:
        self.write_manifest(
            [
                {
                    "id": "intro",
                    "title": "Introduction",
                    "summary": "Legacy intro.",
                    "section": "intro",
                    "see_also": ["commands/change"],
                }
            ]
        )

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "unsupported legacy field"):
            self.prepare()

        self.write_manifest(
            [
                {
                    "id": "intro",
                    "title": "Introduction",
                    "description": "Start here.",
                    "section": "intro",
                    "see_also": ["commands/change"],
                },
                {
                    "id": "commands/change",
                    "title": "Change command",
                    "description": "Find changes.",
                    "section": "commands",
                },
            ]
        )
        change = self.export / "commands" / "change.md"
        change.write_text(
            change.read_text(encoding="utf-8").replace(
                "description: Find changes.", "summary: Legacy summary."
            ),
            encoding="utf-8",
        )

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "unsupported legacy field"):
            self.prepare()

    def test_force_is_required_to_replace_outputs(self) -> None:
        self.prepare()

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "rerun with --force"):
            self.prepare(force=False)

    def test_rejects_unsupported_manifest_version(self) -> None:
        self.write_manifest([], export_format_version=999)

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "export_format_version"):
            self.prepare()

    def test_rejects_wrong_cli_name_and_version(self) -> None:
        self.write_manifest([], cli_name="other")
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "cli_name"):
            self.prepare()

        self.write_manifest([], cli_version="0.6.0")
        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "does not match"):
            self.prepare()

    def test_rejects_missing_required_topic_field(self) -> None:
        self.write_manifest(
            [{"id": "intro", "title": "Introduction", "section": "intro"}]
        )

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "description"):
            self.prepare()

    def test_rejects_unsafe_topic_ids(self) -> None:
        self.write_manifest(
            [
                {
                    "id": "../escape",
                    "title": "Escape",
                    "description": "No.",
                    "section": "commands",
                }
            ]
        )

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "unsafe topic"):
            self.prepare()

    def test_rejects_invalid_see_also_values(self) -> None:
        self.write_manifest(
            [
                {
                    "id": "intro",
                    "title": "Introduction",
                    "description": "Start.",
                    "section": "intro",
                    "see_also": ["../escape"],
                }
            ]
        )

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "unsafe topic"):
            self.prepare()

    def test_rejects_missing_exported_source_file(self) -> None:
        (self.export / "commands" / "change.md").unlink()

        with self.assertRaisesRegex(prepare_mkdocs.PrepareError, "missing exported topic"):
            self.prepare()


if __name__ == "__main__":
    unittest.main()
