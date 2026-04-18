use crate::cli::docs::{
    DocsArgs, DocsCommand, DocsExportArgs, DocsSearchArgs, DocsShowArgs, DocsSkillArgs,
    DocsTopicsArgs,
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
        Some(DocsCommand::Skill(args)) => skill(args),
    }
}

fn orientation_index() -> Result<CommandResult, WavepeekError> {
    Ok(text_result(
        CommandName::Docs,
        "wavepeek local docs\n\nStart here when you need more than command syntax.\n\nTry:\n  wavepeek docs topics\n  wavepeek docs show intro\n  wavepeek docs search transitions\n  wavepeek docs skill\n  wavepeek docs export /tmp/wavepeek-docs\n".to_string(),
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

    let rendered = if args.summary {
        topics
            .iter()
            .map(|topic| format!("{}  {}", topic.id, topic.summary))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        topics
            .iter()
            .map(|topic| format!("{}  {} — {}", topic.id, topic.title, topic.summary))
            .collect::<Vec<_>>()
            .join("\n")
    };

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
    let matches = docs::search_topics(&raw_query, args.full_text)?;

    if args.json {
        return Ok(CommandResult {
            command: CommandName::DocsSearch,
            json: true,
            human_options: HumanRenderOptions::default(),
            data: CommandData::DocsSearch(DocsSearchData {
                query: normalized_query,
                full_text: args.full_text,
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

fn skill(_args: DocsSkillArgs) -> Result<CommandResult, WavepeekError> {
    Ok(text_result(
        CommandName::DocsSkill,
        docs::packaged_skill_markdown().to_string(),
    ))
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
