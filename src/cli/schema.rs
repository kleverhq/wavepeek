use clap::Args;

#[derive(Debug, Args)]
pub struct SchemaArgs {
    /// Print the JSONL stream record schema instead of the JSON envelope schema
    #[arg(long, conflicts_with = "input", help_heading = "Selection options")]
    pub stream: bool,
    /// Print the JSON input document schema instead of the JSON envelope schema
    #[arg(long, conflicts_with = "stream", help_heading = "Selection options")]
    pub input: bool,
}
