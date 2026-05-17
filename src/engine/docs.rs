use crate::cli::docs::{
    DocsArgs, DocsCommand, DocsExportArgs, DocsSearchArgs, DocsShowArgs, DocsTopicsArgs,
};
use crate::docs;
use crate::engine::{
    CommandData, CommandName, CommandResult, DocsSearchData, DocsSearchMatchData, DocsTopicsData,
    HumanRenderOptions,
};
use crate::error::WavepeekError;

pub fn run(args: DocsArgs) -> Result<CommandResult, WavepeekError> {
    match args.command {
        None => orientation_index(),
        Some(DocsCommand::Topics(args)) => topics(args),
        Some(DocsCommand::Show(args)) => show(args),
        Some(DocsCommand::Search(args)) => search(args),
        Some(DocsCommand::Export(args)) => export(args),
    }
}

fn orientation_index() -> Result<CommandResult, WavepeekError> {
    Ok(text_result(
        CommandName::Docs,
        "wavepeek local docs\n\nStart here when you need more than command syntax.\n\nTry:\n  wavepeek docs topics\n  wavepeek docs show intro\n  wavepeek docs search transitions\n  wavepeek docs export /tmp/wavepeek-docs\n".to_string(),
    ))
}

fn topics(args: DocsTopicsArgs) -> Result<CommandResult, WavepeekError> {
    let topics = docs::list_topics()?;
    if args.json {
        return Ok(CommandResult {
            command: CommandName::DocsTopics,
            json: true,
            human_options: HumanRenderOptions::default(),
            data: CommandData::DocsTopics(DocsTopicsData { topics }),
            warnings: Vec::new(),
        });
    }

    let rendered = topics
        .iter()
        .map(|topic| format!("{} — {}", topic.id, topic.summary))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(text_result(CommandName::DocsTopics, rendered))
}

fn show(args: DocsShowArgs) -> Result<CommandResult, WavepeekError> {
    let topic = docs::lookup_topic(&args.topic)?.ok_or_else(|| unknown_topic_error(&args.topic))?;
    let rendered = if args.summary {
        topic.summary.summary.clone()
    } else {
        topic.body.clone()
    };

    Ok(text_result(CommandName::DocsShow, rendered))
}

fn search(args: DocsSearchArgs) -> Result<CommandResult, WavepeekError> {
    let raw_query = args.query.join(" ");
    let normalized_query = docs::normalize_search_query(&raw_query)?;
    let matches = docs::search_topics(&raw_query)?;

    if args.json {
        return Ok(CommandResult {
            command: CommandName::DocsSearch,
            json: true,
            human_options: HumanRenderOptions::default(),
            data: CommandData::DocsSearch(DocsSearchData {
                query: normalized_query,
                matches: matches
                    .into_iter()
                    .map(|entry| DocsSearchMatchData {
                        topic: entry.topic,
                        match_kind: entry.match_kind,
                        matched_tokens: entry.matched_tokens,
                    })
                    .collect(),
            }),
            warnings: Vec::new(),
        });
    }

    let rendered = matches
        .iter()
        .map(|entry| {
            format!(
                "{}  {} — {} [{}]",
                entry.topic.id,
                entry.topic.title,
                entry.topic.summary,
                match_label(entry.match_kind)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(text_result(CommandName::DocsSearch, rendered))
}

fn export(args: DocsExportArgs) -> Result<CommandResult, WavepeekError> {
    let summary = docs::export_catalog(args.out_dir.as_path(), args.force)?;
    let rendered = format!(
        "exported {} topic(s) to {}",
        summary.topics.len(),
        summary.out_dir
    );

    Ok(text_result(CommandName::DocsExport, rendered))
}

fn text_result(command: CommandName, text: String) -> CommandResult {
    CommandResult {
        command,
        json: false,
        human_options: HumanRenderOptions::default(),
        data: CommandData::Text(text),
        warnings: Vec::new(),
    }
}

fn unknown_topic_error(topic: &str) -> WavepeekError {
    let suggestions = docs::suggest_topics(topic, 3);
    if suggestions.is_empty() {
        return WavepeekError::Args(format!("unknown docs topic '{topic}'"));
    }

    let suggestions = suggestions
        .iter()
        .map(|suggestion| suggestion.id.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    WavepeekError::Args(format!(
        "unknown docs topic '{topic}'. Did you mean: {suggestions}?"
    ))
}

fn match_label(kind: crate::docs::MatchKind) -> &'static str {
    match kind {
        crate::docs::MatchKind::IdExact => "matched id",
        crate::docs::MatchKind::IdPrefix => "matched id prefix",
        crate::docs::MatchKind::TitleExact => "matched title",
        crate::docs::MatchKind::TitleOrSummary => "matched title or summary",
        crate::docs::MatchKind::Heading => "matched heading",
        crate::docs::MatchKind::Body => "matched body",
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{match_label, run, unknown_topic_error};
    use crate::cli::docs::{
        DocsArgs, DocsCommand, DocsExportArgs, DocsSearchArgs, DocsShowArgs, DocsTopicsArgs,
    };
    use crate::docs::MatchKind;
    use crate::engine::{CommandData, CommandName};

    #[test]
    fn run_without_subcommand_returns_orientation_text() {
        let result = run(DocsArgs { command: None }).expect("docs root should succeed");

        assert_eq!(result.command, CommandName::Docs);
        assert!(matches!(
            result.data,
            CommandData::Text(ref text) if text.contains("wavepeek docs topics")
        ));
    }

    #[test]
    fn topics_support_human_and_json_modes() {
        let human = run(DocsArgs {
            command: Some(DocsCommand::Topics(DocsTopicsArgs { json: false })),
        })
        .expect("human topics should succeed");
        assert_eq!(human.command, CommandName::DocsTopics);
        assert!(matches!(
            human.data,
            CommandData::Text(ref text) if text.contains("intro")
        ));

        let json = run(DocsArgs {
            command: Some(DocsCommand::Topics(DocsTopicsArgs { json: true })),
        })
        .expect("json topics should succeed");
        assert!(matches!(
            json.data,
            CommandData::DocsTopics(ref data) if !data.topics.is_empty()
        ));
    }

    #[test]
    fn show_uses_summary_flag_and_unknown_topic_suggestions() {
        let summary = run(DocsArgs {
            command: Some(DocsCommand::Show(DocsShowArgs {
                topic: "intro".to_string(),
                summary: true,
            })),
        })
        .expect("summary show should succeed");
        assert!(matches!(
            summary.data,
            CommandData::Text(ref text) if !text.contains("# ") && text.contains("wavepeek")
        ));

        let body = run(DocsArgs {
            command: Some(DocsCommand::Show(DocsShowArgs {
                topic: "intro".to_string(),
                summary: false,
            })),
        })
        .expect("body show should succeed");
        assert!(matches!(
            body.data,
            CommandData::Text(ref text) if text.starts_with("# ")
        ));

        let error = run(DocsArgs {
            command: Some(DocsCommand::Show(DocsShowArgs {
                topic: "commands/cha".to_string(),
                summary: false,
            })),
        })
        .expect_err("unknown topic should fail");
        assert!(error.to_string().contains("Did you mean: commands/change?"));
    }

    #[test]
    fn search_supports_human_and_json_modes() {
        let human = run(DocsArgs {
            command: Some(DocsCommand::Search(DocsSearchArgs {
                query: vec!["change".to_string()],
                json: false,
            })),
        })
        .expect("human search should succeed");
        assert!(matches!(
            human.data,
            CommandData::Text(ref text) if text.contains("[matched")
        ));

        let json = run(DocsArgs {
            command: Some(DocsCommand::Search(DocsSearchArgs {
                query: vec!["Change".to_string(), "command".to_string()],
                json: true,
            })),
        })
        .expect("json search should succeed");
        assert!(matches!(
            json.data,
            CommandData::DocsSearch(ref data)
                if data.query == "change command" && !data.matches.is_empty()
        ));
    }

    #[test]
    fn export_reports_destination_path() {
        let temp = tempdir().expect("tempdir should exist");
        let out_dir = temp.path().join("docs-export");
        let result = run(DocsArgs {
            command: Some(DocsCommand::Export(DocsExportArgs {
                out_dir: out_dir.clone(),
                force: false,
            })),
        })
        .expect("export should succeed");

        assert_eq!(result.command, CommandName::DocsExport);
        assert!(matches!(
            result.data,
            CommandData::Text(ref text)
                if text.contains("exported") && text.contains(out_dir.display().to_string().as_str())
        ));
    }

    #[test]
    fn unknown_topic_error_handles_empty_suggestions() {
        let error = unknown_topic_error("zzzzzzzzzzzz");
        assert_eq!(
            error.to_string(),
            "error: args: unknown docs topic 'zzzzzzzzzzzz'"
        );
    }

    #[test]
    fn match_label_exercises_all_search_kinds() {
        assert_eq!(match_label(MatchKind::IdExact), "matched id");
        assert_eq!(match_label(MatchKind::IdPrefix), "matched id prefix");
        assert_eq!(match_label(MatchKind::TitleExact), "matched title");
        assert_eq!(
            match_label(MatchKind::TitleOrSummary),
            "matched title or summary"
        );
        assert_eq!(match_label(MatchKind::Heading), "matched heading");
        assert_eq!(match_label(MatchKind::Body), "matched body");
    }
}
