use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(
    about = "Browse the embedded documentation packaged with this build.",
    long_about = "Browse the embedded documentation packaged with this build.\n\nUse this command family when you need concepts, workflows, troubleshooting, or agent guidance that complements command reference help.",
    arg_required_else_help = true
)]
pub struct DocsArgs {
    #[command(subcommand)]
    pub command: Option<DocsCommand>,
}

#[derive(Debug, Subcommand)]
pub enum DocsCommand {
    #[command(
        about = "List embedded documentation topics.",
        long_about = r#"List embedded documentation topics.

Behavior:
- Prints each stable topic ID and summary.
- `--json` emits the standard machine-readable envelope for docs topic metadata."#
    )]
    Topics(DocsTopicsArgs),
    #[command(
        about = "Print one embedded documentation topic to output.",
        long_about = "Print one embedded documentation topic to output."
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
    /// Machine-readable JSON output
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DocsShowArgs {
    /// Slash-separated topic ID (see 'wavepeek docs topics')
    #[arg(value_name = "TOPIC")]
    pub topic: String,
    /// Print only the summary text
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
