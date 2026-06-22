use clap::Args;

#[derive(Debug, Args)]
pub struct SchemaArgs {
    /// Print the JSONL stream record schema instead of the JSON envelope schema
    #[arg(long, help_heading = "Selection options")]
    pub stream: bool,
}
