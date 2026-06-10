use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::docs::{MatchKind, TopicSummary};
use crate::engine::property::{PropertyCaptureRow, PropertyResultKind};
use crate::engine::{
    CommandData, CommandName, CommandResult, DocsSearchData, DocsSearchMatchData, DocsTopicsData,
    HumanRenderOptions,
};

use super::*;

fn topic(id: &str) -> TopicSummary {
    TopicSummary {
        id: id.to_string(),
        title: id.to_string(),
        description: format!("description for {id}"),
        section: "commands".to_string(),
        see_also: Vec::new(),
    }
}

#[test]
fn exercises_docs_topics_human_json_and_diagnostic_rendering() {
    let topics = vec![topic("commands/change"), topic("commands/value")];
    let data = CommandData::DocsTopics(DocsTopicsData {
        topics: topics.clone(),
    });
    let rendered = render_human(&data, HumanRenderOptions::default());
    assert!(rendered.contains("commands/change — description for commands/change"));
    assert!(rendered.contains("commands/value — description for commands/value"));

    let search = CommandData::DocsSearch(DocsSearchData {
        query: "change".to_string(),
        matches: vec![DocsSearchMatchData {
            topic: topics[0].clone(),
            match_kind: MatchKind::TitleOrDescription,
            matched_tokens: 1,
        }],
    });
    assert!(render_human(&search, HumanRenderOptions::default()).contains("commands/change"));

    let json = render_json(CommandResult {
        command: CommandName::DocsTopics,
        json: true,
        human_options: HumanRenderOptions::default(),
        data,
        diagnostics: vec![Diagnostic::warning(
            WarningDiagnosticCode::EmptyResult,
            "careful",
        )],
    })
    .expect("docs topics json should render");
    assert!(json.contains("docs topics"));
    assert!(json.contains("careful"));

    let properties = CommandData::Property(vec![
        PropertyCaptureRow {
            time: "0ns".to_string(),
            kind: PropertyResultKind::Assert,
        },
        PropertyCaptureRow {
            time: "1ns".to_string(),
            kind: PropertyResultKind::Deassert,
        },
    ]);
    assert!(render_human(&properties, HumanRenderOptions::default()).contains("@1ns deassert"));

    let envelope = OutputEnvelope::with_diagnostics(
        "docs search",
        search,
        vec![Diagnostic::warning(
            WarningDiagnosticCode::EmptyResult,
            "heads up",
        )],
    );
    let debug = format!("{envelope:?}");
    assert!(debug.contains("docs search"));
    assert!(
        serde_json::to_string(&envelope)
            .expect("envelope serializes")
            .contains("heads up")
    );
}
