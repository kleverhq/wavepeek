use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use include_dir::{Dir, File, include_dir};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::WavepeekError;

static TOPICS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/docs/cli/topics");
pub const PACKAGED_SKILL_MARKDOWN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/docs/cli/wavepeek-skill.md"
));

const EXPORT_KIND: &str = "wavepeek-docs-export";
pub const EXPORT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopicSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub section: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub see_also: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicRecord {
    pub summary: TopicSummary,
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
    TitleOrSummary,
    Heading,
    Body,
}

impl MatchKind {
    const fn rank(self) -> usize {
        match self {
            Self::IdExact => 0,
            Self::IdPrefix => 1,
            Self::TitleExact => 2,
            Self::TitleOrSummary => 3,
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
            .map(|record| record.summary.clone())
            .collect()
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
struct FrontMatter {
    id: String,
    title: String,
    summary: String,
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
    Ok(embedded_catalog()?.topic_summaries())
}

pub fn normalize_search_query(query: &str) -> Result<String, WavepeekError> {
    normalize_query(query)
}

pub fn search_topics(query: &str, full_text: bool) -> Result<Vec<SearchMatch>, WavepeekError> {
    let catalog = embedded_catalog()?;
    let normalized_query = normalize_query(query)?;
    let tokens = tokenize(&normalized_query);

    let mut matches = catalog
        .topics
        .values()
        .filter_map(|record| search_match(record, &normalized_query, &tokens, full_text))
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
            let id = record.summary.id.to_ascii_lowercase();
            let title = record.summary.title.to_ascii_lowercase();
            let summary = record.summary.summary.to_ascii_lowercase();

            let score = if id.starts_with(&normalized) {
                0
            } else if id.contains(&normalized) {
                1
            } else if title.contains(&normalized) {
                2
            } else if summary.contains(&normalized) {
                3
            } else {
                return None;
            };

            Some((score, record.summary.clone()))
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

    let topics = catalog.topic_summaries();
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
        let topic_id = record.summary.id.clone();
        if topics.insert(topic_id.clone(), record).is_some() {
            return Err(format!("duplicate docs topic id '{topic_id}'"));
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
        summary: TopicSummary {
            id: metadata.id,
            title: metadata.title,
            summary: metadata.summary,
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
    full_text: bool,
) -> Option<SearchMatch> {
    let normalized_id = record.summary.id.to_ascii_lowercase();
    let normalized_title = record.summary.title.to_ascii_lowercase();
    let normalized_summary = record.summary.summary.to_ascii_lowercase();
    let normalized_headings = record
        .headings
        .iter()
        .map(|heading| heading.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let normalized_body = full_text.then(|| record.body.to_ascii_lowercase());

    if normalized_id == normalized_query {
        return Some(SearchMatch {
            topic: record.summary.clone(),
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
        } else if normalized_title.contains(token) || normalized_summary.contains(token) {
            Some(MatchKind::TitleOrSummary)
        } else if normalized_headings
            .iter()
            .any(|heading| heading.contains(token))
        {
            Some(MatchKind::Heading)
        } else if normalized_body
            .as_ref()
            .is_some_and(|body| body.contains(token))
        {
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
        topic: record.summary.clone(),
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
mod tests {
    use tempfile::tempdir;

    use super::{
        EXPORT_FORMAT_VERSION, MatchKind, export_catalog, lookup_topic, packaged_skill_markdown,
        search_topics, suggest_topics,
    };

    #[test]
    fn embedded_topics_load_in_lexicographic_order() {
        let matches = search_topics("commands/change", false).expect("catalog should load");
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
        let matches = search_topics("find first change", false).expect("search should succeed");

        assert_eq!(matches[0].topic.id, "workflows/find-first-change");
        assert_eq!(matches[0].match_kind, MatchKind::TitleExact);
        assert_eq!(matches[1].topic.id, "commands/change");
        assert_eq!(matches[1].match_kind, MatchKind::IdPrefix);
        assert_eq!(matches[2].topic.id, "troubleshooting/empty-results");
        assert_eq!(matches[2].match_kind, MatchKind::Heading);
    }

    #[test]
    fn suggestions_prefer_id_prefix_matches() {
        let suggestions = suggest_topics("commands/cha", 3);

        assert_eq!(suggestions[0].id, "commands/change");
    }

    #[test]
    fn search_matches_topic_id_tokens_by_default() {
        let matches = search_topics("empty-results", false).expect("search should succeed");

        assert_eq!(matches[0].topic.id, "troubleshooting/empty-results");
        assert_eq!(matches[0].match_kind, MatchKind::IdPrefix);
        assert_eq!(matches[0].matched_tokens, 1);
    }

    #[test]
    fn search_counts_distinct_query_tokens_only_once() {
        let matches = search_topics("change change", false).expect("search should succeed");

        assert_eq!(matches[0].matched_tokens, 1);
    }

    #[test]
    fn search_preserves_exact_title_match_kind() {
        let matches = search_topics("Change command", false).expect("search should succeed");

        assert_eq!(matches[0].topic.id, "commands/change");
        assert_eq!(matches[0].match_kind, MatchKind::TitleExact);
    }

    #[test]
    fn all_topic_paths_match_canonical_ids() {
        let topic = lookup_topic("commands/change")
            .expect("lookup should succeed")
            .expect("topic should exist");

        assert_eq!(topic.source_relpath, "commands/change.md");
    }

    #[test]
    fn export_writes_manifest_and_topics_without_skill_file() {
        let temp = tempdir().expect("tempdir should be created");
        let out_dir = temp.path().join("wavepeek-docs");

        let summary = export_catalog(&out_dir, false).expect("export should succeed");

        assert_eq!(summary.topics.len(), 9);
        assert!(out_dir.join("commands").join("change.md").exists());
        assert!(out_dir.join("manifest.json").exists());
        assert!(!out_dir.join("wavepeek-skill.md").exists());
        assert_eq!(EXPORT_FORMAT_VERSION, 1);
        assert!(packaged_skill_markdown().starts_with("---\nname: wavepeek\n"));
    }
}
