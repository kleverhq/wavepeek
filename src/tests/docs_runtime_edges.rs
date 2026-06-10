use std::collections::BTreeMap;

use super::*;

fn summary(id: &str, title: &str, description: &str) -> TopicSummary {
    TopicSummary {
        id: id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        section: "commands".to_string(),
        see_also: vec!["reference/schema".to_string()],
    }
}

fn record(id: &str, title: &str, body: &str) -> TopicRecord {
    TopicRecord {
        topic: summary(id, title, "compact description"),
        raw_markdown: format!("---\nid: {id}\n---\n{body}"),
        body: body.to_string(),
        headings: extract_headings(body),
        source_relpath: format!("{id}.md"),
    }
}

#[test]
fn derives_search_edges_and_catalog_sorting() {
    let topic = summary("commands/change", "Change", "Find changes");
    let topic_clone = topic.clone();
    assert_eq!(topic, topic_clone);
    assert!(format!("{topic:?}").contains("commands/change"));
    assert!(
        serde_json::to_string(&topic)
            .unwrap()
            .contains("Find changes")
    );

    let rec = record(
        "commands/change",
        "Change",
        "# Change\n\n## Details\nbody token",
    );
    let rec_clone = rec.clone();
    assert_eq!(rec, rec_clone);
    assert!(format!("{rec:?}").contains("Details"));

    let id_exact = search_match(&rec, "commands/change", &[]).expect("id exact");
    assert_eq!(id_exact.match_kind, MatchKind::IdExact);
    let title_exact = search_match(&rec, "change", &["change".to_string()]).expect("title exact");
    assert_eq!(title_exact.match_kind, MatchKind::TitleExact);
    let heading = search_match(&rec, "details", &["details".to_string()]).expect("heading");
    assert_eq!(heading.match_kind, MatchKind::Heading);
    let body = search_match(&rec, "token", &["token".to_string()]).expect("body");
    assert_eq!(body.match_kind, MatchKind::Body);
    assert!(search_match(&rec, "missing", &["missing".to_string()]).is_none());

    let match_clone = body.clone();
    assert_eq!(body, match_clone);
    assert!(format!("{body:?}").contains("Body"));
    assert!(
        serde_json::to_string(&body)
            .unwrap()
            .contains("matched_tokens")
    );

    let manifest = ExportManifest {
        kind: EXPORT_KIND.to_string(),
        export_format_version: EXPORT_FORMAT_VERSION,
        cli_name: "wavepeek".to_string(),
        cli_version: "0.test".to_string(),
        topics: vec![topic.clone()],
    };
    let manifest_clone = manifest.clone();
    assert_eq!(manifest, manifest_clone);
    assert!(format!("{manifest:?}").contains("wavepeek-docs-export"));
    assert!(
        serde_json::to_string(&manifest)
            .unwrap()
            .contains("cli_version")
    );

    let export = ExportSummary {
        out_dir: "out".to_string(),
        topics: vec![topic.clone()],
    };
    assert_eq!(export.clone(), export);
    assert!(format!("{export:?}").contains("out"));
    assert!(serde_json::to_string(&export).unwrap().contains("topics"));

    let mut topics = BTreeMap::new();
    topics.insert(
        "workflows/a".to_string(),
        record("workflows/a", "Workflow", "# Workflow"),
    );
    topics.insert("intro/a".to_string(), record("intro/a", "Intro", "# Intro"));
    let catalog = DocsCatalog { topics };
    let sorted = catalog.logical_topic_summaries();
    assert_eq!(sorted[0].section, "commands");
    assert!(format!("{catalog:?}").contains("workflows/a"));
    assert_eq!(topic_section_rank("future"), 5);

    let front = FrontMatter {
        id: "x".to_string(),
        title: "X".to_string(),
        description: "S".to_string(),
        section: "intro".to_string(),
        see_also: Vec::new(),
    };
    assert!(format!("{front:?}").contains("intro"));
}

#[test]
fn filesystem_error_handlers_are_exercised() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, "not a directory").expect("blocker file");
    assert!(export_catalog(&blocker.join("out"), false).is_err());

    let manifest = ExportManifest {
        kind: EXPORT_KIND.to_string(),
        export_format_version: EXPORT_FORMAT_VERSION,
        cli_name: "wavepeek".to_string(),
        cli_version: "test".to_string(),
        topics: Vec::new(),
    };
    let catalog = DocsCatalog {
        topics: BTreeMap::new(),
    };
    assert!(write_export_tree(&blocker, &catalog, &manifest).is_err());

    let tree = tmp.path().join("tree");
    std::fs::create_dir(&tree).expect("tree dir");
    let mut topics = BTreeMap::new();
    topics.insert(
        "blocked/topic".to_string(),
        TopicRecord {
            topic: summary("blocked/topic", "Blocked", "blocked"),
            raw_markdown: "# Blocked\n".to_string(),
            body: "# Blocked\n".to_string(),
            headings: vec!["Blocked".to_string()],
            source_relpath: "blocked/topic.md".to_string(),
        },
    );
    let catalog = DocsCatalog { topics };
    std::fs::write(tree.join("blocked"), "file blocks directory").expect("blocking file");
    assert!(write_export_tree(&tree, &catalog, &manifest).is_err());

    let manifest_block = tmp.path().join("manifest-block");
    std::fs::create_dir(&manifest_block).expect("manifest tree");
    std::fs::create_dir(manifest_block.join("manifest.json")).expect("manifest directory");
    let catalog = DocsCatalog {
        topics: BTreeMap::new(),
    };
    assert!(write_export_tree(&manifest_block, &catalog, &manifest).is_err());
}
