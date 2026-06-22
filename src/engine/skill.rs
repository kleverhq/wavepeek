use crate::cli::skill::SkillArgs;
use crate::docs;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;

pub fn run(_args: SkillArgs) -> Result<CommandResult, WavepeekError> {
    Ok(CommandResult {
        command: CommandName::Skill,
        output_mode: crate::output_mode::OutputMode::Human,
        human_options: HumanRenderOptions::default(),
        data: CommandData::Text(docs::packaged_skill_markdown().to_string()),
        diagnostics: Vec::new(),
    })
}
