use clap::Args;

#[derive(Debug, Args)]
pub struct SchemaArgs {
    /// Strict JSON envelope output
    #[arg(long)]
    pub json: bool,
}
