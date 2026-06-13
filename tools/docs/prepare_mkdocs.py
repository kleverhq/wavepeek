#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import os
import pathlib
import re
import shutil
import sys
import tempfile
from dataclasses import dataclass
from typing import Any

import yaml

EXPORT_KIND = "wavepeek-docs-export"
EXPORT_FORMAT_VERSION = 1
CLI_NAME = "wavepeek"
SECTION_LABELS = {
    "intro": "Introduction",
    "commands": "Commands",
    "workflows": "Workflows",
    "troubleshooting": "Troubleshooting",
    "reference": "Reference",
}
SECTION_ORDER = ["intro", "commands", "workflows", "troubleshooting", "reference"]
SAFE_TOPIC_SEGMENT = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


class PrepareError(Exception):
    pass


@dataclass(frozen=True)
class Topic:
    id: str
    title: str
    description: str
    section: str
    see_also: list[str]
    source_relpath: pathlib.PurePosixPath
    page_relpath: pathlib.PurePosixPath


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prepare a generated MkDocs source tree from wavepeek docs export output."
    )
    parser.add_argument("export_dir", type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    parser.add_argument("--config-output", required=True, type=pathlib.Path)
    parser.add_argument("--version")
    parser.add_argument("--force", action="store_true")
    return parser.parse_args(argv)


def fail(message: str) -> None:
    raise PrepareError(message)


def safe_topic_id(value: Any, *, field: str = "id") -> str:
    if not isinstance(value, str) or not value.strip():
        fail(f"topic {field} must be a non-empty string")
    topic_id = value.strip()
    if topic_id.startswith("/") or "\\" in topic_id:
        fail(f"unsafe topic {field}: {value!r}")
    parts = topic_id.split("/")
    if any(part in {"", ".", ".."} for part in parts):
        fail(f"unsafe topic {field}: {value!r}")
    for part in parts:
        if not SAFE_TOPIC_SEGMENT.fullmatch(part):
            fail(f"unsafe topic {field}: {value!r}")
    return topic_id


def safe_child_path(root: pathlib.Path, relpath: pathlib.PurePosixPath) -> pathlib.Path:
    root_resolved = root.resolve()
    candidate = (root_resolved / pathlib.Path(*relpath.parts)).resolve()
    try:
        candidate.relative_to(root_resolved)
    except ValueError:
        fail(f"path escapes root: {candidate}")
    return candidate


def string_field(entry: dict[str, Any], name: str, *, topic_id: str) -> str:
    value = entry.get(name)
    if not isinstance(value, str) or not value.strip():
        fail(f"topic {topic_id!r} field {name!r} must be a non-empty string")
    return value


def see_also_field(entry: dict[str, Any], *, topic_id: str) -> list[str]:
    value = entry.get("see_also", [])
    if not isinstance(value, list):
        fail(f"topic {topic_id!r} field 'see_also' must be a list when present")
    return [safe_topic_id(item, field="see_also") for item in value]


def topic_relpaths(topic_id: str) -> tuple[pathlib.PurePosixPath, pathlib.PurePosixPath]:
    source_relpath = pathlib.PurePosixPath(f"{topic_id}.md")
    page_relpath = pathlib.PurePosixPath("index.md") if topic_id == "intro" else source_relpath
    return source_relpath, page_relpath


def load_manifest(export_dir: pathlib.Path, version: str | None) -> tuple[str, list[Topic]]:
    manifest_path = export_dir / "manifest.json"
    if not manifest_path.exists():
        fail(f"missing export manifest: {manifest_path}")
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        fail(f"export manifest is not valid JSON: {error}")
    if not isinstance(manifest, dict):
        fail("export manifest root must be an object")
    if manifest.get("kind") != EXPORT_KIND:
        fail(f"export manifest kind must be {EXPORT_KIND!r}")
    if manifest.get("export_format_version") != EXPORT_FORMAT_VERSION:
        fail(
            "export manifest export_format_version must be "
            f"{EXPORT_FORMAT_VERSION}"
        )
    if manifest.get("cli_name") != CLI_NAME:
        fail(f"export manifest cli_name must be {CLI_NAME!r}")
    cli_version = manifest.get("cli_version")
    if not isinstance(cli_version, str) or not cli_version:
        fail("export manifest cli_version must be a non-empty string")
    if version is not None and cli_version != version:
        fail(f"export manifest cli_version {cli_version!r} does not match {version!r}")
    raw_topics = manifest.get("topics")
    if not isinstance(raw_topics, list):
        fail("export manifest topics must be a list")

    topics: list[Topic] = []
    seen_ids: set[str] = set()
    for index, raw_topic in enumerate(raw_topics):
        if not isinstance(raw_topic, dict):
            fail(f"manifest topic at index {index} must be an object")
        topic_id = safe_topic_id(raw_topic.get("id"))
        if topic_id in seen_ids:
            fail(f"duplicate topic id {topic_id!r}")
        seen_ids.add(topic_id)
        if "summary" in raw_topic:
            fail(f"manifest topic {topic_id!r} uses unsupported legacy field 'summary'")
        title = string_field(raw_topic, "title", topic_id=topic_id)
        description = string_field(raw_topic, "description", topic_id=topic_id)
        section = string_field(raw_topic, "section", topic_id=topic_id)
        see_also = see_also_field(raw_topic, topic_id=topic_id)
        source_relpath, page_relpath = topic_relpaths(topic_id)
        topics.append(
            Topic(
                id=topic_id,
                title=title,
                description=description,
                section=section,
                see_also=see_also,
                source_relpath=source_relpath,
                page_relpath=page_relpath,
            )
        )

    topic_ids = {topic.id for topic in topics}
    for topic in topics:
        for target in topic.see_also:
            if target not in topic_ids:
                fail(f"topic {topic.id!r} references unknown see_also target {target!r}")

    return cli_version, topics


def split_front_matter(markdown: str, source: pathlib.Path) -> tuple[dict[str, Any], str]:
    if not markdown.startswith("---\n"):
        fail(f"topic {source} is missing YAML front matter")
    end = markdown.find("\n---", 4)
    if end == -1:
        fail(f"topic {source} has unterminated YAML front matter")
    front_text = markdown[4:end]
    body_start = end + len("\n---")
    if markdown.startswith("\r\n", body_start):
        body_start += 2
    elif markdown.startswith("\n", body_start):
        body_start += 1
    body = markdown[body_start:]
    try:
        front = yaml.safe_load(front_text) or {}
    except yaml.YAMLError as error:
        fail(f"topic {source} front matter is invalid YAML: {error}")
    if not isinstance(front, dict):
        fail(f"topic {source} front matter must be a mapping")
    return front, body


def normalize_markdown(source: pathlib.Path, topic: Topic) -> str:
    markdown = source.read_text(encoding="utf-8")
    front, body = split_front_matter(markdown, source)
    if "summary" in front:
        fail(f"topic {source} uses unsupported legacy field 'summary'")
    description = front.get("description")
    if not isinstance(description, str) or not description.strip():
        fail(f"topic {source} front matter must define description")

    normalized: dict[str, Any] = {}
    for key in ("id", "title"):
        if key in front:
            normalized[key] = front[key]
    normalized["description"] = description
    if "section" in front:
        normalized["section"] = front["section"]
    if "see_also" in front:
        normalized["see_also"] = front["see_also"]

    for key in front:
        if key not in {"id", "title", "description", "section", "see_also"}:
            normalized[key] = front[key]

    front_text = yaml.safe_dump(
        normalized,
        sort_keys=False,
        allow_unicode=True,
        default_flow_style=False,
    )
    return f"---\n{front_text}---\n{body}"


def build_nav(topics: list[Topic]) -> list[dict[str, Any]]:
    by_section: dict[str, list[Topic]] = {}
    for topic in topics:
        by_section.setdefault(topic.section, []).append(topic)

    nav: list[dict[str, Any]] = []
    for section in SECTION_ORDER:
        section_topics = by_section.pop(section, [])
        if section == "intro":
            for topic in section_topics:
                nav.append({SECTION_LABELS[section]: topic.page_relpath.as_posix()})
            continue
        if section_topics:
            nav.append(
                {
                    SECTION_LABELS[section]: [
                        {topic.title: topic.page_relpath.as_posix()}
                        for topic in section_topics
                    ]
                }
            )

    for section in sorted(by_section):
        section_topics = by_section[section]
        label = section.replace("-", " ").replace("_", " ").title()
        nav.append(
            {label: [{topic.title: topic.page_relpath.as_posix()} for topic in section_topics]}
        )
    return nav


def write_generated_config(
    config_output: pathlib.Path,
    output: pathlib.Path,
    nav: list[dict[str, Any]],
) -> None:
    config_parent = config_output.parent.resolve()
    root_config = pathlib.Path("mkdocs.yml").resolve()
    inherit_path = pathlib.Path(os.path.relpath(root_config, config_parent)).as_posix()
    docs_dir = pathlib.Path(os.path.relpath(output.resolve(), config_parent)).as_posix()
    site_dir = pathlib.Path(
        os.path.relpath((config_parent / "mkdocs-site").resolve(), config_parent)
    ).as_posix()
    generated = {
        "INHERIT": inherit_path,
        "docs_dir": docs_dir,
        "site_dir": site_dir,
        "nav": nav,
    }
    config_output.parent.mkdir(parents=True, exist_ok=True)
    text = yaml.safe_dump(generated, sort_keys=False, allow_unicode=True)
    config_output.write_text(text, encoding="utf-8")


def prepare_tree(
    export_dir: pathlib.Path,
    output: pathlib.Path,
    config_output: pathlib.Path,
    version: str | None,
    *,
    force: bool,
) -> tuple[str, list[Topic]]:
    export_dir = export_dir.resolve()
    output = output.resolve()
    config_output = config_output.resolve()

    if not export_dir.is_dir():
        fail(f"export directory does not exist: {export_dir}")
    if output.exists() and not force:
        fail(f"output directory already exists: {output}; rerun with --force")
    if config_output.exists() and not force:
        fail(f"config output already exists: {config_output}; rerun with --force")

    cli_version, topics = load_manifest(export_dir, version)
    nav = build_nav(topics)

    output.parent.mkdir(parents=True, exist_ok=True)
    temp_dir = pathlib.Path(
        tempfile.mkdtemp(prefix=f".{output.name}-", dir=output.parent)
    ).resolve()
    try:
        for topic in topics:
            source = safe_child_path(export_dir, topic.source_relpath)
            if not source.is_file():
                fail(f"missing exported topic file: {source}")
            destination = safe_child_path(temp_dir, topic.page_relpath)
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(normalize_markdown(source, topic), encoding="utf-8")

        if output.exists():
            shutil.rmtree(output)
        temp_dir.replace(output)
        write_generated_config(config_output, output, nav)
    except Exception:
        if temp_dir.exists():
            shutil.rmtree(temp_dir)
        raise

    return cli_version, topics


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        cli_version, topics = prepare_tree(
            args.export_dir,
            args.output,
            args.config_output,
            args.version,
            force=args.force,
        )
    except PrepareError as error:
        print(f"error: docs-site: {error}", file=sys.stderr)
        return 1

    print(
        f"prepared {len(topics)} topic(s) for wavepeek {cli_version} at {args.output}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
