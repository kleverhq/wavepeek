use clap::{Args, ValueEnum};
use ondas::PathStyle;

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub(crate) enum PathStyleArg {
    #[default]
    Logical,
    Sv,
}

#[derive(Debug, Clone, Copy, Default, Args)]
pub(crate) struct PathOutputArgs {
    /// Path syntax for output. Both styles are accepted as input
    #[arg(
        long,
        value_enum,
        default_value_t = PathStyleArg::default(),
        help_heading = "Output options"
    )]
    path_style: PathStyleArg,
}

impl PathOutputArgs {
    pub(crate) fn style(self) -> PathStyle {
        match self.path_style {
            PathStyleArg::Logical => PathStyle::Logical,
            PathStyleArg::Sv => PathStyle::Sv,
        }
    }
}
