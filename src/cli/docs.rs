use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(
    about = "Browse embedded narrative docs",
    long_about = r#"Browse the embedded narrative docs packaged with this wavepeek build.

Behavior:
- `wavepeek docs` prints the local docs index instead of clap help output.
- `topics` lists stable topic IDs and packaged metadata.
- `show` prints one topic body or just its stored summary.
- `search` ranks matching topics deterministically and can emit JSON.
- `export` writes the authored Markdown corpus to a managed output directory.
- `skill` prints the packaged agent skill Markdown.

Use this command family when you need concepts, workflows, troubleshooting, or agent guidance that complements command reference help."#,
    after_help = "Next steps:\n  wavepeek docs --help\n  wavepeek docs topics\n  wavepeek docs show intro",
    after_long_help = "Examples:\n  wavepeek docs\n  wavepeek docs topics\n  wavepeek docs show <TOPIC>\n  wavepeek docs show intro\n  wavepeek docs search transitions\n  wavepeek docs export <OUT_DIR>\n  wavepeek docs skill"
)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: Option<DocsCommand>,
}

#[derive(Debug, Subcommand)]
pub enum DocsCommand {
    #[command(
        about = "List embedded documentation topics",
        long_about = r#"List the embedded documentation topics shipped with this wavepeek build.

Behavior:
- Results are ordered lexicographically by stable topic ID.
- Default text output prints topic ID, title, and summary.
- `--summary` keeps the output list-oriented but omits titles.
- `--json` emits the standard machine-readable envelope for docs topic metadata."#
    )]
    Topics(DocsTopicsArgs),
    #[command(
        about = "Show one embedded documentation topic",
        long_about = r#"Show one embedded documentation topic by stable topic ID.

Behavior:
- The topic argument is a slash-separated topic ID, not a filesystem path.
- Default output prints the raw Markdown body exactly as authored, excluding YAML front matter.
- `--summary` prints only the stored summary text.
- Unknown topic IDs fail with deterministic close-match suggestions when available."#,
        after_help = "Quick shape:\n  Usage: wavepeek docs show <TOPIC>\n\nNext steps:\n  wavepeek docs show <TOPIC>\n  wavepeek docs show --help\n  wavepeek docs topics\n  wavepeek docs search <QUERY>",
        after_long_help = "Examples:\n  wavepeek docs show intro\n  wavepeek docs show commands/change\n  wavepeek docs show commands/change --summary"
    )]
    Show(DocsShowArgs),
    #[command(
        about = "Search embedded documentation topics",
        long_about = r#"Search embedded documentation topics in deterministic ranked order.

Behavior:
- Matching is case-insensitive and tokenized on whitespace.
- Default scope searches topic IDs, titles, summaries, and Markdown headings.
- `--full-text` expands the search to full Markdown bodies.
- `--json` emits the standard machine-readable envelope for ranked search results."#
    )]
    Search(DocsSearchArgs),
    #[command(
        about = "Export authored Markdown topics to disk",
        long_about = r#"Export the authored Markdown topic corpus to disk.

Behavior:
- Writes one Markdown file per embedded topic using stable topic-path layout.
- Preserves YAML front matter exactly as authored in the packaged source.
- Writes a managed-root manifest that makes future `--force` replacement safe.
- Excludes the separately routed packaged skill Markdown."#
    )]
    Export(DocsExportArgs),
    #[command(
        about = "Print the packaged agent skill Markdown",
        long_about = r#"Print the packaged agent skill Markdown for wavepeek.

Behavior:
- Writes the shipped skill source directly to stdout.
- Keeps agent guidance version-matched to the installed binary.
- Does not export through `wavepeek docs export`; use this subcommand when you need the skill text."#
    )]
    Skill(DocsSkillArgs),
}

#[derive(Debug, Args)]
pub struct DocsTopicsArgs {
    /// Print only topic IDs and summaries
    #[arg(long, conflicts_with = "json")]
    pub summary: bool,
    /// Machine-readable JSON output
    #[arg(long, conflicts_with = "summary")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DocsShowArgs {
    /// Stable slash-separated topic ID
    #[arg(value_name = "TOPIC")]
    pub topic: String,
    /// Print only the stored summary text
    #[arg(long)]
    pub summary: bool,
}

#[derive(Debug, Args)]
pub struct DocsSearchArgs {
    /// One or more query tokens
    #[arg(value_name = "QUERY", num_args = 1.., required = true)]
    pub query: Vec<String>,
    /// Expand matching to full Markdown bodies
    #[arg(long)]
    pub full_text: bool,
    /// Machine-readable JSON output
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DocsExportArgs {
    /// Output directory for exported topic Markdown files
    #[arg(value_name = "OUT_DIR")]
    pub out_dir: PathBuf,
    /// Replace an empty or previously managed export root
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args, Default)]
pub struct DocsSkillArgs {}
