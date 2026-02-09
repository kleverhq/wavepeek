pub mod at;
pub mod changes;
pub mod info;
pub mod schema;
pub mod signals;
pub mod tree;
pub mod when;

use crate::cli;
use crate::error::WavepeekError;

#[derive(Debug)]
pub enum Command {
    Schema(cli::schema::SchemaArgs),
    Info(cli::info::InfoArgs),
    Tree(cli::tree::TreeArgs),
    Signals(cli::signals::SignalsArgs),
    At(cli::at::AtArgs),
    Changes(cli::changes::ChangesArgs),
    When(cli::when::WhenArgs),
}

pub fn run(command: Command) -> Result<(), WavepeekError> {
    match command {
        Command::Schema(args) => schema::run(args),
        Command::Info(args) => info::run(args),
        Command::Tree(args) => tree::run(args),
        Command::Signals(args) => signals::run(args),
        Command::At(args) => at::run(args),
        Command::Changes(args) => changes::run(args),
        Command::When(args) => when::run(args),
    }
}
