use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(
    about = "Browse the embedded documentation packaged with this build.",
    long_about = "Browse the embedded documentation packaged with this build.\n\nUse this command family when you need command guidance, workflows, troubleshooting, reference topics, or agent guidance that complements command reference help.",
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
        about = "Search embedded documentation topics.",
        long_about = r#"Search embedded documentation topics.

Behavior:
- Query is plain text, not a regular expression; it is normalized case-insensitively and split into whitespace tokens.
- Scope searches topic IDs, titles, summaries, Markdown headings, and Markdown bodies.
- `--json` emits the standard machine-readable envelope for ranked search results (contract: see `wavepeek schema`)."#
    )]
    Search(DocsSearchArgs),
    #[command(
        about = "Export all embedded Markdown documentation to disk.",
        long_about = r#"Export all embedded Markdown documentation to disk.

Behavior:
- Writes one Markdown file per embedded topic using stable topic-path layout.
- Preserves YAML front matter exactly as authored in the packaged source.
- Writes a managed-root manifest that makes future `--force` replacement safe."#
    )]
    Export(DocsExportArgs),
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
    /// Plain-text query split into whitespace tokens
    #[arg(value_name = "QUERY", num_args = 1.., required = true)]
    pub query: Vec<String>,
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
