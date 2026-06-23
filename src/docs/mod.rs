use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use include_dir::{Dir, File, include_dir};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::WavepeekError;

static TOPICS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/docs/public");
pub const PACKAGED_SKILL_MARKDOWN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/skills/wavepeek.md"
));

const EXPORT_KIND: &str = "wavepeek-docs-export";
pub const EXPORT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopicSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub section: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicRecord {
    pub topic: TopicSummary,
    pub raw_markdown: String,
    pub body: String,
    pub headings: Vec<String>,
    pub source_relpath: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchKind {
    IdExact,
    IdPrefix,
    TitleExact,
    TitleOrDescription,
    Heading,
    Body,
}

impl MatchKind {
    const fn rank(self) -> usize {
        match self {
            Self::IdExact => 0,
            Self::IdPrefix => 1,
            Self::TitleExact => 2,
            Self::TitleOrDescription => 3,
            Self::Heading => 4,
            Self::Body => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchMatch {
    pub topic: TopicSummary,
    pub match_kind: MatchKind,
    pub matched_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsCatalog {
    topics: BTreeMap<String, TopicRecord>,
}

impl DocsCatalog {
    fn topic_summaries(&self) -> Vec<TopicSummary> {
        self.topics
            .values()
            .map(|record| record.topic.clone())
            .collect()
    }

    fn logical_topic_summaries(&self) -> Vec<TopicSummary> {
        let mut summaries = self.topic_summaries();
        summaries.sort_by(|left, right| {
            topic_section_rank(&left.section)
                .cmp(&topic_section_rank(&right.section))
                .then_with(|| left.id.cmp(&right.id))
        });
        summaries
    }
}

fn topic_section_rank(section: &str) -> usize {
    match section {
        "intro" => 0,
        "commands" => 1,
        "workflows" => 2,
        "troubleshooting" => 3,
        "reference" => 4,
        _ => 5,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportSummary {
    pub out_dir: String,
    pub topics: Vec<TopicSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportManifest {
    pub kind: String,
    pub export_format_version: u32,
    pub cli_name: String,
    pub cli_version: String,
    pub topics: Vec<TopicSummary>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrontMatter {
    id: String,
    title: String,
    description: String,
    section: String,
    #[serde(default)]
    see_also: Vec<String>,
}

static CATALOG: OnceLock<Result<DocsCatalog, String>> = OnceLock::new();

pub fn embedded_catalog() -> Result<&'static DocsCatalog, WavepeekError> {
    match CATALOG.get_or_init(load_catalog) {
        Ok(catalog) => Ok(catalog),
        Err(message) => Err(WavepeekError::Internal(message.clone())),
    }
}

pub fn lookup_topic(id: &str) -> Result<Option<&'static TopicRecord>, WavepeekError> {
    Ok(embedded_catalog()?.topics.get(id))
}

pub fn list_topics() -> Result<Vec<TopicSummary>, WavepeekError> {
    Ok(embedded_catalog()?.logical_topic_summaries())
}

pub fn normalize_search_query(query: &str) -> Result<String, WavepeekError> {
    normalize_query(query)
}

pub fn search_topics(query: &str) -> Result<Vec<SearchMatch>, WavepeekError> {
    let catalog = embedded_catalog()?;
    let normalized_query = normalize_query(query)?;
    let tokens = tokenize(&normalized_query);

    let mut matches = catalog
        .topics
        .values()
        .filter_map(|record| search_match(record, &normalized_query, &tokens))
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        let left_exact = left.match_kind == MatchKind::IdExact;
        let right_exact = right.match_kind == MatchKind::IdExact;

        right_exact
            .cmp(&left_exact)
            .then_with(|| right.matched_tokens.cmp(&left.matched_tokens))
            .then_with(|| left.match_kind.rank().cmp(&right.match_kind.rank()))
            .then_with(|| left.topic.id.cmp(&right.topic.id))
    });

    Ok(matches)
}

pub fn suggest_topics(input: &str, limit: usize) -> Vec<TopicSummary> {
    let Ok(catalog) = embedded_catalog() else {
        return Vec::new();
    };
    let normalized = input.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut suggestions = catalog
        .topics
        .values()
        .filter_map(|record| {
            let id = record.topic.id.to_ascii_lowercase();
            let title = record.topic.title.to_ascii_lowercase();
            let description = record.topic.description.to_ascii_lowercase();

            let score = if id.starts_with(&normalized) {
                0
            } else if id.contains(&normalized) {
                1
            } else if title.contains(&normalized) {
                2
            } else if description.contains(&normalized) {
                3
            } else {
                return None;
            };

            Some((score, record.topic.clone()))
        })
        .collect::<Vec<_>>();

    suggestions.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.id.cmp(&right.1.id))
    });
    suggestions
        .into_iter()
        .take(limit)
        .map(|(_, summary)| summary)
        .collect()
}

pub fn export_catalog(out_dir: &Path, force: bool) -> Result<ExportSummary, WavepeekError> {
    let catalog = embedded_catalog()?;
    validate_export_target(out_dir, force)?;

    let topics = catalog.logical_topic_summaries();
    let manifest = ExportManifest {
        kind: EXPORT_KIND.to_string(),
        export_format_version: EXPORT_FORMAT_VERSION,
        cli_name: env!("CARGO_PKG_NAME").to_string(),
        cli_version: env!("CARGO_PKG_VERSION").to_string(),
        topics: topics.clone(),
    };

    let parent = out_dir.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        WavepeekError::File(format!(
            "failed to create export parent directory '{}': {error}",
            parent.display()
        ))
    })?;

    let temp_dir = unique_sibling_path(parent, out_dir, ".wavepeek-docs-tmp");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|error| {
            WavepeekError::File(format!(
                "failed to clean temporary export directory '{}': {error}",
                temp_dir.display()
            ))
        })?;
    }

    write_export_tree(&temp_dir, catalog, &manifest)?;

    if out_dir.exists() {
        let backup_dir = unique_sibling_path(parent, out_dir, ".wavepeek-docs-backup");
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir).map_err(|error| {
                WavepeekError::File(format!(
                    "failed to clean backup export directory '{}': {error}",
                    backup_dir.display()
                ))
            })?;
        }

        fs::rename(out_dir, &backup_dir).map_err(|error| {
            WavepeekError::File(format!(
                "failed to stage managed export root '{}': {error}",
                out_dir.display()
            ))
        })?;

        if let Err(error) = fs::rename(&temp_dir, out_dir) {
            let _ = fs::rename(&backup_dir, out_dir);
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(WavepeekError::File(format!(
                "failed to install exported docs into '{}': {error}",
                out_dir.display()
            )));
        }

        fs::remove_dir_all(&backup_dir).map_err(|error| {
            WavepeekError::File(format!(
                "failed to remove backup export directory '{}': {error}",
                backup_dir.display()
            ))
        })?;
    } else {
        fs::rename(&temp_dir, out_dir).map_err(|error| {
            let _ = fs::remove_dir_all(&temp_dir);
            WavepeekError::File(format!(
                "failed to install exported docs into '{}': {error}",
                out_dir.display()
            ))
        })?;
    }

    Ok(ExportSummary {
        out_dir: out_dir.display().to_string(),
        topics,
    })
}

pub fn packaged_skill_markdown() -> &'static str {
    PACKAGED_SKILL_MARKDOWN
}

fn load_catalog() -> Result<DocsCatalog, String> {
    let mut files = Vec::new();
    collect_markdown_files(&TOPICS_DIR, &mut files);

    let mut topics = BTreeMap::new();
    for file in files {
        let record = parse_topic_file(file)?;
        let topic_id = record.topic.id.clone();
        if topics.insert(topic_id.clone(), record).is_some() {
            return Err(format!("duplicate docs topic id '{topic_id}'"));
        }
    }

    for (topic_id, record) in &topics {
        for target in &record.topic.see_also {
            if !topics.contains_key(target) {
                return Err(format!(
                    "docs topic '{topic_id}' references unknown see_also target '{target}'"
                ));
            }
        }
    }

    Ok(DocsCatalog { topics })
}

fn collect_markdown_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a File<'a>>) {
    for file in dir.files() {
        if file
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("md")
            && file.path().file_name().and_then(|name| name.to_str()) != Some("AGENTS.md")
        {
            out.push(file);
        }
    }
    for child in dir.dirs() {
        collect_markdown_files(child, out);
    }
}

fn parse_topic_file(file: &File<'_>) -> Result<TopicRecord, String> {
    let raw_markdown = file
        .contents_utf8()
        .ok_or_else(|| format!("docs topic '{}' is not valid UTF-8", file.path().display()))?
        .to_string();
    let (front_matter, body) = split_front_matter(&raw_markdown, file.path())?;
    let metadata: FrontMatter = serde_yaml::from_str(front_matter).map_err(|error| {
        format!(
            "failed to parse YAML front matter for docs topic '{}': {error}",
            file.path().display()
        )
    })?;

    let first_heading = body
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| format!("docs topic '{}' has an empty body", file.path().display()))?;
    let expected_h1 = format!("# {}", metadata.title);
    if first_heading.trim_end_matches('\r') != expected_h1 {
        return Err(format!(
            "docs topic '{}' must start with '{expected_h1}'",
            file.path().display()
        ));
    }

    let headings = extract_headings(body);
    let body = body.to_string();
    let expected_relpath = canonical_source_relpath(&metadata.id);
    let actual_relpath = file.path().to_string_lossy().replace('\\', "/");
    if actual_relpath != expected_relpath {
        return Err(format!(
            "docs topic '{}' declares id '{}' but lives at '{}'; expected '{}'",
            file.path().display(),
            metadata.id,
            actual_relpath,
            expected_relpath
        ));
    }

    Ok(TopicRecord {
        topic: TopicSummary {
            id: metadata.id,
            title: metadata.title,
            description: metadata.description,
            section: metadata.section,
            see_also: metadata.see_also,
        },
        raw_markdown,
        body,
        headings,
        source_relpath: expected_relpath,
    })
}

fn split_front_matter<'a>(raw: &'a str, path: &Path) -> Result<(&'a str, &'a str), String> {
    let matcher = Regex::new(r"(?s)\A---\r?\n(.*?)\r?\n---\r?\n")
        .map_err(|error| format!("failed to build docs front matter regex: {error}"))?;
    let captures = matcher.captures(raw).ok_or_else(|| {
        format!(
            "docs topic '{}' must start with YAML front matter delimited by ---",
            path.display()
        )
    })?;
    let envelope = captures.get(0).expect("full match should exist");
    let front_matter = captures.get(1).expect("front matter capture should exist");
    Ok((front_matter.as_str(), &raw[envelope.end()..]))
}

fn extract_headings(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let hashes = trimmed
                .chars()
                .take_while(|character| *character == '#')
                .count();
            if !(1..=6).contains(&hashes) {
                return None;
            }
            let title = trimmed.get(hashes..)?.trim_start();
            if title.is_empty() {
                None
            } else {
                Some(title.trim_end_matches('\r').to_string())
            }
        })
        .collect()
}

fn canonical_source_relpath(id: &str) -> String {
    format!("{id}.md")
}

fn normalize_query(query: &str) -> Result<String, WavepeekError> {
    let normalized_parts = query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if normalized_parts.is_empty() {
        return Err(WavepeekError::Args(
            "query must contain at least one non-whitespace token".to_string(),
        ));
    }
    let normalized = normalized_parts.join(" ");
    Ok(normalized)
}

fn tokenize(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for token in query.split_whitespace() {
        if token.is_empty() || tokens.iter().any(|existing| existing == token) {
            continue;
        }
        tokens.push(token.to_string());
    }
    tokens
}

fn search_match(
    record: &TopicRecord,
    normalized_query: &str,
    tokens: &[String],
) -> Option<SearchMatch> {
    let normalized_id = record.topic.id.to_ascii_lowercase();
    let normalized_title = record.topic.title.to_ascii_lowercase();
    let normalized_description = record.topic.description.to_ascii_lowercase();
    let normalized_headings = record
        .headings
        .iter()
        .map(|heading| heading.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let normalized_body = record.body.to_ascii_lowercase();

    if normalized_id == normalized_query {
        return Some(SearchMatch {
            topic: record.topic.clone(),
            match_kind: MatchKind::IdExact,
            matched_tokens: tokens.len().max(1),
        });
    }

    let mut matched_tokens = 0;
    let mut best_kind = None;

    let title_exact = normalized_title == normalized_query;
    if title_exact {
        best_kind = Some(
            best_kind.map_or(MatchKind::TitleExact, |current: MatchKind| {
                if MatchKind::TitleExact.rank() < current.rank() {
                    MatchKind::TitleExact
                } else {
                    current
                }
            }),
        );
    }

    for token in tokens {
        let token_kind = if id_matches_token(&normalized_id, token) {
            Some(MatchKind::IdPrefix)
        } else if normalized_title.contains(token) || normalized_description.contains(token) {
            Some(MatchKind::TitleOrDescription)
        } else if normalized_headings
            .iter()
            .any(|heading| heading.contains(token))
        {
            Some(MatchKind::Heading)
        } else if normalized_body.contains(token) {
            Some(MatchKind::Body)
        } else {
            None
        };

        if let Some(kind) = token_kind {
            matched_tokens += 1;
            best_kind = Some(best_kind.map_or(kind, |current: MatchKind| {
                if kind.rank() < current.rank() {
                    kind
                } else {
                    current
                }
            }));
        }
    }

    let match_kind = best_kind.map(|current| {
        if title_exact && current != MatchKind::IdExact {
            MatchKind::TitleExact
        } else {
            current
        }
    })?;

    Some(SearchMatch {
        topic: record.topic.clone(),
        match_kind,
        matched_tokens,
    })
}

fn id_matches_token(normalized_id: &str, token: &str) -> bool {
    normalized_id.starts_with(token)
        || normalized_id.split('/').any(|segment| {
            segment.starts_with(token)
                || segment
                    .split('-')
                    .any(|chunk| !chunk.is_empty() && chunk.starts_with(token))
        })
}

fn validate_export_target(out_dir: &Path, force: bool) -> Result<(), WavepeekError> {
    if !out_dir.exists() {
        return Ok(());
    }

    let metadata = fs::metadata(out_dir).map_err(|error| {
        WavepeekError::File(format!(
            "failed to inspect export target '{}': {error}",
            out_dir.display()
        ))
    })?;
    if !metadata.is_dir() {
        return Err(WavepeekError::Args(format!(
            "export target '{}' must be a directory path",
            out_dir.display()
        )));
    }

    let is_empty = fs::read_dir(out_dir)
        .map_err(|error| {
            WavepeekError::File(format!(
                "failed to inspect export target '{}': {error}",
                out_dir.display()
            ))
        })?
        .next()
        .is_none();
    if is_empty {
        return Ok(());
    }
    if !force {
        return Err(WavepeekError::Args(format!(
            "export target '{}' is not empty; rerun with --force only for an empty or managed export root",
            out_dir.display()
        )));
    }

    let manifest_path = out_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Err(WavepeekError::Args(format!(
            "refusing to replace unmanaged directory '{}'; --force only applies to a managed export root",
            out_dir.display()
        )));
    }

    let manifest_bytes = fs::read_to_string(&manifest_path).map_err(|error| {
        WavepeekError::File(format!(
            "failed to read export manifest '{}': {error}",
            manifest_path.display()
        ))
    })?;
    let manifest_value: serde_json::Value =
        serde_json::from_str(&manifest_bytes).map_err(|error| {
            WavepeekError::Args(format!(
                "managed export root '{}' has an invalid manifest: {error}",
                out_dir.display()
            ))
        })?;

    if manifest_value["kind"] != EXPORT_KIND {
        return Err(WavepeekError::Args(format!(
            "refusing to replace unmanaged directory '{}'; --force only applies to a managed export root",
            out_dir.display()
        )));
    }

    let Some(version) = manifest_value["export_format_version"].as_u64() else {
        return Err(WavepeekError::Args(format!(
            "managed export root '{}' is missing a recognized export_format_version",
            out_dir.display()
        )));
    };
    if version != EXPORT_FORMAT_VERSION as u64 {
        return Err(WavepeekError::Args(format!(
            "managed export root '{}' has unrecognized export manifest version {version}",
            out_dir.display()
        )));
    }

    Ok(())
}

fn write_export_tree(
    temp_dir: &Path,
    catalog: &DocsCatalog,
    manifest: &ExportManifest,
) -> Result<(), WavepeekError> {
    fs::create_dir_all(temp_dir).map_err(|error| {
        WavepeekError::File(format!(
            "failed to create temporary export directory '{}': {error}",
            temp_dir.display()
        ))
    })?;

    for record in catalog.topics.values() {
        let destination = temp_dir.join(&record.source_relpath);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                WavepeekError::File(format!(
                    "failed to create export directory '{}': {error}",
                    parent.display()
                ))
            })?;
        }
        fs::write(&destination, &record.raw_markdown).map_err(|error| {
            WavepeekError::File(format!(
                "failed to write exported topic '{}': {error}",
                destination.display()
            ))
        })?;
    }

    let manifest_json = serde_json::to_string_pretty(manifest).map_err(|error| {
        WavepeekError::Internal(format!("failed to serialize export manifest: {error}"))
    })?;
    fs::write(temp_dir.join("manifest.json"), format!("{manifest_json}\n")).map_err(|error| {
        WavepeekError::File(format!(
            "failed to write export manifest '{}': {error}",
            temp_dir.join("manifest.json").display()
        ))
    })?;

    Ok(())
}

fn unique_sibling_path(parent: &Path, out_dir: &Path, prefix: &str) -> PathBuf {
    let stem = out_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("wavepeek-docs");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    parent.join(format!("{prefix}-{stem}-{nonce}"))
}

#[cfg(test)]
#[path = "../tests/docs_runtime_edges.rs"]
mod docs_runtime_edges;

#[cfg(test)]
mod docs_inline_derive_tests {
    use super::*;

    #[test]
    fn inline_derive_calls_are_attributed_to_docs_module() {
        let topic = TopicSummary {
            id: "commands/demo".to_string(),
            title: "Demo".to_string(),
            description: "Demo description".to_string(),
            section: "commands".to_string(),
            see_also: Vec::new(),
        };
        assert_eq!(topic.clone(), topic);
        assert!(format!("{topic:?}").contains("Demo"));
        assert!(serde_json::to_string(&topic).unwrap().contains("Demo"));
        let front: FrontMatter = serde_yaml::from_str(
            "id: commands/demo\ntitle: Demo\ndescription: Demo description\nsection: commands\n",
        )
        .unwrap();
        assert!(format!("{front:?}").contains("commands/demo"));

        let legacy_error = serde_yaml::from_str::<FrontMatter>(
            "id: commands/legacy\ntitle: Legacy\ndescription: Current\nsummary: Legacy summary\nsection: commands\n",
        )
        .expect_err("legacy summary metadata should be rejected");
        assert!(legacy_error.to_string().contains("summary"));
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use include_dir::{Dir, include_dir};
    use tempfile::tempdir;

    use super::{
        DocsCatalog, EXPORT_FORMAT_VERSION, EXPORT_KIND, ExportManifest, MatchKind, SearchMatch,
        TOPICS_DIR, TopicRecord, TopicSummary, canonical_source_relpath, collect_markdown_files,
        embedded_catalog, export_catalog, extract_headings, id_matches_token, list_topics,
        lookup_topic, normalize_query, normalize_search_query, packaged_skill_markdown,
        parse_topic_file, search_match, search_topics, split_front_matter, suggest_topics,
        tokenize, topic_section_rank, unique_sibling_path, validate_export_target,
        write_export_tree,
    };

    static DOC_FIXTURES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/tests/fixtures/docs_embed");

    #[test]
    fn public_docs_api_exercises_catalog_search_and_lookup_wrappers() {
        let catalog = embedded_catalog().expect("embedded catalog should load");
        assert!(catalog.topics.len() >= 20);

        let topics = list_topics().expect("topics should list");
        assert!(topics.len() >= 20);
        assert!(topics.iter().any(|topic| topic.id == "commands/change"));

        assert_eq!(
            normalize_search_query("  commands   info  ").expect("query should normalize"),
            "commands info"
        );

        let change = lookup_topic("commands/change")
            .expect("lookup should succeed")
            .expect("topic should exist");
        assert_eq!(change.topic.id, "commands/change");

        let matches = search_topics("commands info").expect("search should succeed");
        assert!(
            matches
                .iter()
                .any(|entry| entry.topic.id == "commands/info")
        );

        let suggestions = suggest_topics("value", 3);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|topic| topic.id == "commands/value"));
    }

    #[test]
    fn embedded_topics_load_in_lexicographic_order() {
        let matches = search_topics("commands/change").expect("catalog should load");
        assert_eq!(matches[0].topic.id, "commands/change");
        assert_eq!(matches[0].match_kind, MatchKind::IdExact);
    }

    #[test]
    fn lookup_topic_preserves_raw_markdown_and_body_split() {
        let topic = lookup_topic("commands/change")
            .expect("lookup should succeed")
            .expect("topic should exist");

        assert!(topic.raw_markdown.starts_with("---\nid: commands/change\n"));
        assert!(topic.body.starts_with("# Change command\n"));
    }

    #[test]
    fn search_ranks_exact_title_above_weaker_matches() {
        let matches = search_topics("find first change").expect("search should succeed");

        assert_eq!(matches[0].topic.id, "workflows/find-first-change");
        assert_eq!(matches[0].match_kind, MatchKind::TitleExact);

        let heading_idx = matches
            .iter()
            .position(|entry| entry.topic.id == "troubleshooting/empty-results")
            .expect("troubleshooting/empty-results should match");
        assert_eq!(matches[heading_idx].match_kind, MatchKind::Heading);

        let title_or_description_idx = matches
            .iter()
            .position(|entry| entry.topic.id == "reference/expression-language")
            .expect("reference/expression-language should match");
        assert_eq!(
            matches[title_or_description_idx].match_kind,
            MatchKind::TitleOrDescription
        );

        let body_idx = matches
            .iter()
            .position(|entry| entry.match_kind == MatchKind::Body)
            .expect("query should produce at least one body match");

        let id_prefix_idx = matches
            .iter()
            .position(|entry| entry.topic.id == "commands/change")
            .expect("commands/change should match");
        assert_eq!(matches[id_prefix_idx].match_kind, MatchKind::IdPrefix);

        assert!(heading_idx > 0);
        assert!(title_or_description_idx > heading_idx);
        assert!(body_idx > heading_idx);
        assert!(id_prefix_idx > heading_idx);
    }

    #[test]
    fn suggestions_prefer_id_prefix_matches() {
        let suggestions = suggest_topics("commands/cha", 3);

        assert_eq!(suggestions[0].id, "commands/change");
    }

    #[test]
    fn search_matches_topic_id_tokens_by_default() {
        let matches = search_topics("empty-results").expect("search should succeed");

        assert_eq!(matches[0].topic.id, "troubleshooting/empty-results");
        assert_eq!(matches[0].match_kind, MatchKind::IdPrefix);
        assert_eq!(matches[0].matched_tokens, 1);
    }

    #[test]
    fn search_counts_distinct_query_tokens_only_once() {
        let matches = search_topics("change change").expect("search should succeed");

        assert_eq!(matches[0].matched_tokens, 1);
    }

    #[test]
    fn search_preserves_exact_title_match_kind() {
        let matches = search_topics("Change command").expect("search should succeed");

        let change_match = matches
            .iter()
            .find(|entry| entry.topic.id == "commands/change")
            .expect("commands/change should match exact title query");
        assert_eq!(change_match.match_kind, MatchKind::TitleExact);
    }

    #[test]
    fn all_topic_paths_match_canonical_ids() {
        let topic = lookup_topic("commands/change")
            .expect("lookup should succeed")
            .expect("topic should exist");

        assert_eq!(topic.source_relpath, "commands/change.md");
    }

    #[test]
    fn all_see_also_targets_resolve_to_known_topics() {
        let topic = lookup_topic("commands/change")
            .expect("lookup should succeed")
            .expect("topic should exist");

        for target in &topic.topic.see_also {
            assert!(
                lookup_topic(target)
                    .expect("lookup should succeed")
                    .is_some(),
                "see_also target {target} should resolve"
            );
        }
    }

    #[test]
    fn export_writes_manifest_and_topics_without_skill_file() {
        let temp = tempdir().expect("tempdir should be created");
        let out_dir = temp.path().join("wavepeek-docs");

        let summary = export_catalog(&out_dir, false).expect("export should succeed");

        assert_eq!(summary.topics.len(), 22);
        assert!(out_dir.join("commands").join("change.md").exists());
        assert!(out_dir.join("manifest.json").exists());
        assert!(!out_dir.join("wavepeek.md").exists());
        assert_eq!(EXPORT_FORMAT_VERSION, 1);
        assert!(packaged_skill_markdown().starts_with("---\nname: wavepeek\n"));
    }

    #[test]
    fn normalize_query_rejects_whitespace_only_input() {
        let error = normalize_query("   \n\t  ").expect_err("empty query should fail");
        assert_eq!(
            error.to_string(),
            "fatal: args: query must contain at least one non-whitespace token"
        );
    }

    #[test]
    fn id_and_heading_helpers_exercise_segmented_forms() {
        assert!(id_matches_token("commands/change-help", "cha"));
        assert!(id_matches_token("commands/change-help", "help"));
        assert!(!id_matches_token("commands/change-help", "scope"));
        assert_eq!(
            canonical_source_relpath("commands/change"),
            "commands/change.md"
        );
        assert_eq!(
            extract_headings("# Title\ntext\n## Child\r\n### Grandchild\n####\n"),
            vec!["Title", "Child", "Grandchild"]
        );
        assert_eq!(
            extract_headings("####### Too deep\n#\n## Good\r\n"),
            vec!["Good"]
        );
    }

    #[test]
    fn split_front_matter_requires_valid_envelope() {
        let error = split_front_matter("# no front matter\n", Path::new("broken.md"))
            .expect_err("missing envelope should fail");
        assert!(error.contains("must start with YAML front matter"));

        let (front, body) = split_front_matter(
            "---\r\nid: test/topic\r\n---\r\n# Title\r\nbody\r\n",
            Path::new("crlf.md"),
        )
        .expect("CRLF front matter should split");
        assert_eq!(front, "id: test/topic");
        assert_eq!(body, "# Title\r\nbody\r\n");
    }

    #[test]
    fn validate_export_target_rejects_non_directory_and_unmanaged_roots() {
        let temp = tempdir().expect("tempdir should be created");
        let file_path = temp.path().join("not-a-dir");
        std::fs::write(&file_path, "x").expect("file should be written");
        let error = validate_export_target(&file_path, false).expect_err("file target should fail");
        assert!(error.to_string().contains("must be a directory path"));

        let nonempty = temp.path().join("nonempty");
        std::fs::create_dir(&nonempty).expect("directory should be created");
        std::fs::write(nonempty.join("note.txt"), "x").expect("note should write");
        let error = validate_export_target(&nonempty, false)
            .expect_err("nonempty unforced target should fail");
        assert!(
            error
                .to_string()
                .contains("is not empty; rerun with --force")
        );

        let unmanaged = temp.path().join("unmanaged");
        std::fs::create_dir(&unmanaged).expect("directory should be created");
        std::fs::write(unmanaged.join("topic.md"), "# topic\n").expect("topic should write");
        let error = validate_export_target(&unmanaged, true)
            .expect_err("unmanaged forced replacement should fail");
        assert!(
            error
                .to_string()
                .contains("--force only applies to a managed export root")
        );

        let managed = temp.path().join("managed");
        std::fs::create_dir(&managed).expect("managed dir should create");
        std::fs::write(
            managed.join("manifest.json"),
            format!(
                "{{\"kind\":\"{}\",\"export_format_version\":{}}}",
                EXPORT_KIND, EXPORT_FORMAT_VERSION
            ),
        )
        .expect("managed manifest should write");
        std::fs::write(managed.join("stale.txt"), "x").expect("managed payload should write");
        validate_export_target(&managed, true).expect("managed forced target should pass");
    }

    #[test]
    fn validate_export_target_rejects_invalid_manifests() {
        let temp = tempdir().expect("tempdir should be created");

        let invalid_json = temp.path().join("invalid-json");
        std::fs::create_dir(&invalid_json).expect("directory should be created");
        std::fs::write(invalid_json.join("manifest.json"), "{not json}")
            .expect("manifest should be written");
        let error = validate_export_target(&invalid_json, true)
            .expect_err("invalid manifest json should fail");
        assert!(error.to_string().contains("has an invalid manifest"));

        let wrong_kind = temp.path().join("wrong-kind");
        std::fs::create_dir(&wrong_kind).expect("directory should be created");
        std::fs::write(
            wrong_kind.join("manifest.json"),
            format!(
                "{{\"kind\":\"other\",\"export_format_version\":{}}}",
                EXPORT_FORMAT_VERSION
            ),
        )
        .expect("manifest should be written");
        let error =
            validate_export_target(&wrong_kind, true).expect_err("wrong manifest kind should fail");
        assert!(
            error
                .to_string()
                .contains("--force only applies to a managed export root")
        );

        let missing_version = temp.path().join("missing-version");
        std::fs::create_dir(&missing_version).expect("directory should be created");
        std::fs::write(
            missing_version.join("manifest.json"),
            format!("{{\"kind\":\"{}\"}}", EXPORT_KIND),
        )
        .expect("manifest should be written");
        let error = validate_export_target(&missing_version, true)
            .expect_err("missing version should fail");
        assert!(
            error
                .to_string()
                .contains("missing a recognized export_format_version")
        );

        let wrong_version = temp.path().join("wrong-version");
        std::fs::create_dir(&wrong_version).expect("directory should be created");
        std::fs::write(
            wrong_version.join("manifest.json"),
            format!(
                "{{\"kind\":\"{}\",\"export_format_version\":999}}",
                EXPORT_KIND
            ),
        )
        .expect("manifest should be written");
        let error =
            validate_export_target(&wrong_version, true).expect_err("wrong version should fail");
        assert!(
            error
                .to_string()
                .contains("has unrecognized export manifest version 999")
        );
    }

    #[test]
    fn docs_catalog_helpers_sort_sections_and_preserve_topics() {
        let catalog = DocsCatalog {
            topics: BTreeMap::from([
                (
                    "reference/zeta".to_string(),
                    fake_topic("reference/zeta", "Zeta", "z", "reference", &[]),
                ),
                (
                    "misc/topic".to_string(),
                    fake_topic("misc/topic", "Misc", "m", "misc", &[]),
                ),
                (
                    "commands/alpha".to_string(),
                    fake_topic("commands/alpha", "Alpha", "a", "commands", &[]),
                ),
            ]),
        };

        let ids = catalog
            .logical_topic_summaries()
            .into_iter()
            .map(|topic| topic.id)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["commands/alpha", "reference/zeta", "misc/topic"]);
        assert_eq!(topic_section_rank("intro"), 0);
        assert_eq!(topic_section_rank("mystery"), 5);
        assert_eq!(catalog.topic_summaries().len(), 3);
    }

    #[test]
    fn search_helpers_exercise_tokenization_and_match_kinds() {
        assert_eq!(tokenize("alpha alpha beta"), vec!["alpha", "beta"]);

        let record = fake_topic(
            "commands/change-help",
            "Change command",
            "Use this to review deltas",
            "commands",
            &["Details"],
        );
        let SearchMatch {
            match_kind,
            matched_tokens,
            ..
        } = search_match(&record, "details", &["details".to_string()])
            .expect("heading token should match");
        assert_eq!(match_kind, MatchKind::Heading);
        assert_eq!(matched_tokens, 1);

        let SearchMatch { match_kind, .. } =
            search_match(&record, "bodytoken", &["bodytoken".to_string()])
                .expect("body token should match");
        assert_eq!(match_kind, MatchKind::Body);

        assert!(search_match(&record, "nomatch", &["nomatch".to_string()]).is_none());
    }

    #[test]
    fn export_target_helpers_accept_empty_dirs_and_generate_unique_names() {
        let temp = tempdir().expect("tempdir should be created");
        let empty = temp.path().join("empty");
        std::fs::create_dir(&empty).expect("empty dir should be created");
        validate_export_target(&empty, false).expect("empty dir should be accepted");

        let sibling_one = unique_sibling_path(temp.path(), &empty, ".prefix");
        let sibling_two = unique_sibling_path(temp.path(), &empty, ".prefix");
        assert_ne!(sibling_one, sibling_two);
        assert!(
            sibling_one
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("empty")
        );
    }

    #[test]
    fn embedded_loader_helpers_exercise_parse_and_export_details() {
        let mut files = Vec::new();
        collect_markdown_files(&TOPICS_DIR, &mut files);
        assert!(
            files
                .iter()
                .any(|file| file.path() == Path::new("commands/change.md"))
        );
        assert!(
            files
                .iter()
                .all(|file| file.path().file_name().unwrap() != "AGENTS.md")
        );

        let change = TOPICS_DIR
            .get_file("commands/change.md")
            .expect("embedded file should exist");
        let parsed = parse_topic_file(change).expect("embedded topic should parse");
        assert_eq!(parsed.topic.id, "commands/change");
        assert_eq!(parsed.headings[0], "Change command");
        assert_eq!(parsed.source_relpath, "commands/change.md");

        let nested = DOC_FIXTURES
            .get_file("nested/topic.md")
            .expect("fixture topic should exist");
        let nested = parse_topic_file(nested).expect("fixture topic should parse");
        assert_eq!(nested.topic.id, "nested/topic");
        assert_eq!(nested.headings, vec!["Nested topic", "Child"]);

        let mismatch = DOC_FIXTURES
            .get_file("mismatch.md")
            .expect("mismatch fixture should exist");
        assert!(
            parse_topic_file(mismatch)
                .expect_err("mismatched id/path should fail")
                .contains("declares id 'commands/change'")
        );

        let empty_body = DOC_FIXTURES
            .get_file("empty-body.md")
            .expect("empty-body fixture should exist");
        assert!(
            parse_topic_file(empty_body)
                .expect_err("empty body should fail")
                .contains("has an empty body")
        );

        let bad_heading = DOC_FIXTURES
            .get_file("bad-heading.md")
            .expect("bad-heading fixture should exist");
        assert!(
            parse_topic_file(bad_heading)
                .expect_err("bad heading should fail")
                .contains("must start with '# Expected title'")
        );

        let bad_yaml = DOC_FIXTURES
            .get_file("bad-yaml.md")
            .expect("bad-yaml fixture should exist");
        assert!(
            parse_topic_file(bad_yaml)
                .expect_err("bad yaml should fail")
                .contains("failed to parse YAML front matter")
        );

        let temp = tempdir().expect("tempdir should be created");
        let export_dir = temp.path().join("export-tree");
        let catalog = DocsCatalog {
            topics: BTreeMap::from([(nested.topic.id.clone(), nested.clone())]),
        };
        let manifest = ExportManifest {
            kind: EXPORT_KIND.to_string(),
            export_format_version: EXPORT_FORMAT_VERSION,
            cli_name: "wavepeek".to_string(),
            cli_version: "test".to_string(),
            topics: catalog.logical_topic_summaries(),
        };
        write_export_tree(&export_dir, &catalog, &manifest).expect("export tree should write");
        assert!(export_dir.join("nested/topic.md").exists());
        let manifest_json = std::fs::read_to_string(export_dir.join("manifest.json"))
            .expect("manifest should be readable");
        assert!(manifest_json.contains("\"kind\": \"wavepeek-docs-export\""));
    }

    #[test]
    fn suggestion_sorting_and_error_helpers_exercise_match_paths() {
        assert!(suggest_topics("   ", 10).is_empty());
        let suggestions = suggest_topics("command", 10);
        assert!(suggestions.len() > 1);
        assert_eq!(suggestions[0].id, "commands/change");
        assert_eq!(suggest_topics("command", 0).len(), 0);
        assert!(
            suggest_topics("First change", 5)
                .iter()
                .any(|topic| topic.title.contains("Find first change"))
        );
        assert!(
            suggest_topics("exact JSON contract", 10)
                .iter()
                .any(|topic| topic.description.contains("exact JSON contract"))
        );
        assert!(suggest_topics("no such docs phrase", 10).is_empty());

        assert_eq!(MatchKind::IdExact.rank(), 0);
        assert_eq!(MatchKind::Body.rank(), 5);

        let summary_record = fake_topic(
            "misc/topic",
            "Totally unrelated",
            "Contains command keyword in description",
            "misc",
            &[],
        );
        let summary_match = search_match(&summary_record, "command", &["command".to_string()])
            .expect("description token should match");
        assert_eq!(summary_match.match_kind, MatchKind::TitleOrDescription);

        let exact_title = search_match(
            &fake_topic("misc/exact", "Command", "s", "misc", &[]),
            "command",
            &["command".to_string()],
        )
        .expect("exact title should match");
        assert_eq!(exact_title.match_kind, MatchKind::TitleExact);

        let exact_id = search_match(
            &fake_topic("commands/change", "Rename me", "s", "commands", &[]),
            "commands/change",
            &["commands/change".to_string()],
        )
        .expect("exact id should match");
        assert_eq!(exact_id.match_kind, MatchKind::IdExact);

        let title_beats_body = search_match(
            &fake_topic(
                "misc/title-body",
                "Command",
                "description",
                "misc",
                &["Command details"],
            ),
            "command details",
            &["command".to_string(), "details".to_string()],
        )
        .expect("multiple tokens should match");
        assert_eq!(title_beats_body.match_kind, MatchKind::TitleOrDescription);
        assert_eq!(title_beats_body.matched_tokens, 2);
    }

    #[test]
    fn export_catalog_replaces_existing_managed_root() {
        let temp = tempdir().expect("tempdir should be created");
        let out_dir = temp.path().join("wavepeek-docs");

        export_catalog(&out_dir, false).expect("initial export should succeed");
        std::fs::write(out_dir.join("stale.txt"), "obsolete").expect("stale file should write");

        let summary = export_catalog(&out_dir, true).expect("managed replacement should succeed");
        assert_eq!(summary.out_dir, out_dir.display().to_string());
        assert!(!out_dir.join("stale.txt").exists());
        assert!(out_dir.join("manifest.json").exists());
        assert!(out_dir.join("commands/change.md").exists());
    }

    #[test]
    fn export_helpers_report_filesystem_collisions() {
        let nested = fake_topic("nested/topic", "Nested", "description", "misc", &["Nested"]);
        let catalog = DocsCatalog {
            topics: BTreeMap::from([(nested.topic.id.clone(), nested.clone())]),
        };
        let manifest = ExportManifest {
            kind: EXPORT_KIND.to_string(),
            export_format_version: EXPORT_FORMAT_VERSION,
            cli_name: "wavepeek".to_string(),
            cli_version: "test".to_string(),
            topics: catalog.logical_topic_summaries(),
        };

        let temp = tempdir().expect("tempdir should be created");
        let temp_file = temp.path().join("occupied");
        std::fs::write(&temp_file, "file").expect("blocking file should be written");
        let error = write_export_tree(&temp_file, &catalog, &manifest)
            .expect_err("file-backed export root should fail");
        assert!(
            error
                .to_string()
                .contains("failed to create temporary export directory")
        );

        let blocked_parent = temp.path().join("blocked-parent");
        std::fs::create_dir(&blocked_parent).expect("directory should be created");
        std::fs::write(blocked_parent.join("nested"), "file")
            .expect("blocking parent should be written");
        let error = write_export_tree(&blocked_parent, &catalog, &manifest)
            .expect_err("file-backed topic parent should fail");
        assert!(
            error
                .to_string()
                .contains("failed to create export directory")
        );

        let blocked_topic = temp.path().join("blocked-topic");
        std::fs::create_dir_all(blocked_topic.join("nested/topic.md"))
            .expect("topic collision directory should be created");
        let error = write_export_tree(&blocked_topic, &catalog, &manifest)
            .expect_err("directory-backed topic output should fail");
        assert!(error.to_string().contains("failed to write exported topic"));

        let blocked_manifest = temp.path().join("blocked-manifest");
        std::fs::create_dir(&blocked_manifest).expect("manifest dir should be created");
        std::fs::create_dir(blocked_manifest.join("manifest.json"))
            .expect("manifest collision directory should be created");
        let error = write_export_tree(&blocked_manifest, &catalog, &manifest)
            .expect_err("directory-backed manifest path should fail");
        assert!(
            error
                .to_string()
                .contains("failed to write export manifest")
        );

        let unreadable_manifest = temp.path().join("manifest-as-dir");
        std::fs::create_dir(&unreadable_manifest).expect("directory should be created");
        std::fs::create_dir(unreadable_manifest.join("manifest.json"))
            .expect("manifest directory should be created");
        let error = validate_export_target(&unreadable_manifest, true)
            .expect_err("directory-backed manifest should fail to read");
        assert!(error.to_string().contains("failed to read export manifest"));

        let parent_file = temp.path().join("parent-file");
        std::fs::write(&parent_file, "file").expect("parent file should be written");
        let error = export_catalog(&parent_file.join("child"), false)
            .expect_err("file-backed export parent should fail");
        assert!(
            error
                .to_string()
                .contains("failed to create export parent directory")
        );
    }

    fn fake_topic(
        id: &str,
        title: &str,
        description: &str,
        section: &str,
        headings: &[&str],
    ) -> TopicRecord {
        TopicRecord {
            topic: TopicSummary {
                id: id.to_string(),
                title: title.to_string(),
                description: description.to_string(),
                section: section.to_string(),
                see_also: vec![],
            },
            raw_markdown: format!("---\nid: {id}\n---\n# {title}\n{description}\n"),
            body: format!("# {title}\n{description}\nbodytoken details\n"),
            headings: headings
                .iter()
                .map(|heading| (*heading).to_string())
                .collect(),
            source_relpath: canonical_source_relpath(id),
        }
    }
}
