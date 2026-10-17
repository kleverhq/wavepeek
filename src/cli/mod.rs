pub mod change;
pub mod extract;
pub mod info;
pub mod limits;
pub mod property;
pub mod sampling;
pub mod scope;
pub mod signal;
pub mod skill;
pub mod value;

use std::ffi::{OsStr, OsString};
use std::io::Write;

use clap::error::ErrorKind;
use clap::parser::ValueSource;
use clap::{Arg, ArgAction, Args, CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::engine::{self, Command as EngineCommand};
use crate::error::WavepeekError;
use crate::output::{self, JsonlWriter};
use crate::output_mode::OutputMode;

#[derive(Debug, Parser)]
#[command(
    name = "wavepeek",
    disable_version_flag = true,
    disable_help_subcommand = true,
    about = "wavepeek queries saved RTL waveform dumps.",
    long_about = r#"wavepeek queries saved RTL waveform dumps.

Behavior:
- Each waveform command opens one waveform dump, runs one query, writes its output, and exits.
- Every build supports VCD and FST. FSDB requires Linux x86_64, Cargo feature `fsdb`, and the Synopsys Verdi FSDB Reader SDK.
- SHM directories require Linux x86_64, Cargo feature `shm`, and a compatible Xcelium reader selected by XCELIUM_HOME.
- Waveform commands write text by default. Use `--json` for one JSON value or `--jsonl` for a stream of JSON records.
- Time values use an integer and an explicit unit, for example `250ps`, `10ns`, or `2us`. Supported units are `zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, and `s`.

Examples:
  wavepeek info --waves dump.fst
  wavepeek scope --waves dump.fst --tree --max-depth 2
  wavepeek help extract axi

Notes:
- Count and traversal limits keep output bounded by default. Use `unlimited` only when the full result is needed.
- Parsed times use the dump's `time_unit`. The `--from` and `--to` boundaries are inclusive.
- Text failures use `fatal: <category>: <message>`. JSON and JSONL use typed fatal records.
- Names ending in `.md` refer to files in the packaged skill. Run `wavepeek skill ./wavepeek-skill` to extract it, then open the files under `./wavepeek-skill/references/`.
- See machine-output.md for machine output and timeunits.md for time syntax."#,
    help_template = "{about-with-newline}\nUsage: {usage}\n\nWaveform commands:\n  info      Show metadata for one waveform dump.\n  scope     List scopes in a waveform hierarchy.\n  signal    List signals in one waveform scope.\n  value     Read selected signal values at one or more times.\n  change    Read signal values at selected events over a time range.\n  property  Evaluate a Boolean expression at selected events.\n  extract   Extract event rows from waveform signals.\n\nHelper commands:\n  skill     Extract the packaged agent skill into a directory.\n  help      Show detailed help for a command path.\n\nOptions:\n{options}{after-help}"
)]
pub struct Cli {
    /// Print semver version
    #[arg(short = 'V', action = ArgAction::SetTrue)]
    version_semver: bool,
    /// Print full version
    #[arg(long = "version", action = ArgAction::SetTrue)]
    version_full: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(flatten, next_help_heading = "Waveform commands")]
    Waveform(WaveformCommand),
    #[command(flatten, next_help_heading = "Helper commands")]
    Helper(HelperCommand),
}

#[derive(Debug, Subcommand)]
enum WaveformCommand {
    #[command(
        about = "Show metadata for one waveform dump.",
        long_about = r#"Show metadata for one waveform dump.

Behavior:
- Reports the dump time unit, start time, and end time.
- Text output uses a simple field list. JSON and JSONL use the standard machine output records.

Example:
  wavepeek info --waves dump.fst

Notes:
- Run `info` before choosing values for `--at`, `--from`, or `--to`.
- See explore-dump.md for the usual inspection flow and timeunits.md for time rules.
- See machine-output.md for JSON and JSONL records."#
    )]
    Info(info::InfoArgs),
    #[command(
        about = "List scopes in a waveform hierarchy.",
        long_about = r#"List scopes in a waveform hierarchy.

Behavior:
- Matches `--filter` against full scope paths and reports each scope's name, depth, and kind.
- Uses stable pre-order depth-first traversal with lexicographic child ordering.
- Reports normalized scope kinds for modules and other recorded hierarchy objects.
- `--tree` prints an indented hierarchy and includes the ancestors of matching scopes.
- Truncation produces a coded diagnostic. An empty text result prints a short message unless `--summary` is set.

Examples:
  wavepeek scope --waves dump.fst --tree --max-depth 2 --max 30
  wavepeek scope --waves dump.fst --filter '.*(cpu|axi|uart).*'

Notes:
- Use `scope` before commands that need an exact scope path.
- See explore-dump.md for a hierarchy walkthrough and paths.md for path rules.
- See machine-output.md for JSON and JSONL records."#
    )]
    Scope(scope::ScopeArgs),
    #[command(
        about = "List signals in one waveform scope.",
        long_about = r#"List signals in one waveform scope.

Behavior:
- Matches `--filter` against signal names and reports each signal's name, kind, and available metadata such as width.
- Lists direct signals by default. `--recursive` also visits child scopes in stable depth-first order.
- Reports normalized signal kinds for wires and other recorded objects.
- Omits ambiguous FSDB paths instead of choosing a backing record and produces a coded diagnostic.
- Truncation produces a coded diagnostic. An empty text result prints a short message unless `--summary` is set.

Examples:
  wavepeek signal --waves dump.fst \
    --scope tb.dut.cpu \
    --filter '.*(clk|reset|state).*'

  wavepeek signal --waves dump.fst \
    --scope tb.dut --recursive --max-depth 2 --abs

Notes:
- Use `--abs` to print canonical paths that can be copied into later commands.
- See explore-dump.md for signal discovery and paths.md for path rules.
- See machine-output.md for JSON and JSONL records."#
    )]
    Signal(signal::SignalArgs),
    #[command(
        about = "Read selected signal values at one or more times.",
        long_about = r#"Read selected signal values at one or more times.

Behavior:
- `--at` and `--signals` accept comma-separated values, repeated options, or both. The command prints one row per `--at` value and preserves request order and duplicates.
- Uses canonical signal paths by default. With `--scope`, names may be relative or canonical paths inside that scope, and both forms may be mixed.
- A trailing `[msb:lsb]` selects bits from a flat integral signal. Use `[n:n]` for one bit; `[n]` remains part of an ordinary waveform path.
- Exact waveform paths take precedence over projection syntax.
- Text values use Verilog literals such as `8'h0f`, including `x` and `z` digits.
- The command fails if a signal cannot be resolved or a requested time is finer than the dump resolution.

Examples:
  wavepeek value --waves dump.fst \
    --scope tb.dut.cpu --at 120ns --signals state,pc

  wavepeek value --waves dump.fst \
    --at 100ns,110ns --signals 'top.status[7:4]'

Notes:
- Time values need explicit units and must align with dump precision.
- To reproduce a `change` or `property` row, pass its `sample_time` to `--at`. In `pre-edge` mode, `time` is the trigger time and `sample_time` is the value sample time.
- See inspect-values.md for examples, paths.md for signal names, and timeunits.md for time rules.
- See machine-output.md for JSON and JSONL records."#
    )]
    Value(value::ValueArgs),
    #[command(
        about = "Read signal values at selected events over a time range.",
        long_about = r#"Read signal values at selected events over a time range.

Behavior:
- `--on` selects events, and `--signals` selects the values printed for each event. `--signals` accepts comma-separated values, repeated options, or both.
- Signal names may end in `[msb:lsb]`. Sparse rows, delta rows, and wildcard comparisons use the projected values. Exact waveform paths take precedence, and `[n]` remains path syntax.
- `--row-mode dense` prints every sampled event. `sparse` prints only samples that changed from the previous selected sample.
- `--row-values full` prints every requested value. `delta` prints changed values, except that its first row is always full.
- Pre-edge sampling reads values before edge-only triggers while keeping the trigger timestamp as the row time. Events without an earlier representable sample are skipped.
- Native sampling reads values at the event timestamp and is required for wildcard triggers such as `--on '*'`.
- Range boundaries are inclusive. In sparse mode, `--from` provides the comparison baseline and does not force a row.
- Truncation produces a coded diagnostic. An empty text result prints a short message unless `--summary` is set.

Examples:
  wavepeek change --waves dump.fst \
    --scope tb.dut.cpu --on 'posedge clk' \
    --signals state,req,ack

  wavepeek change --waves dump.fst \
    --from 100ns --to 160ns --on '*' --sample-mode native \
    --signals top.req,top.ack

Notes:
- JSON and JSONL rows contain `time` for the selected event and `sample_time` for the sampled values. Text shows `sample@<time>` only when they differ.
- See inspect-values.md for common queries, sampling.md for sampling modes, and event-expressions.md for `--on` syntax.
- See machine-output.md for JSON and JSONL records."#
    )]
    Change(change::ChangeArgs),
    #[command(
        about = "Evaluate a Boolean expression at selected events.",
        long_about = r#"Evaluate a Boolean expression at selected events.

Behavior:
- `--on` selects events, and `--eval` defines the expression checked at each event.
- `--capture match` prints every selected event where the expression is true.
- `--capture switch` prints both result transitions. `assert` prints false-to-true transitions, and `deassert` prints true-to-false transitions.
- Pre-edge sampling checks the expression before edge-only triggers while keeping the trigger timestamp as the row time.
- Native sampling checks at the event timestamp and is required for wildcard triggers such as `--on '*'`.
- Truncation produces a coded diagnostic. An empty text result prints a short message unless `--summary` is set.

Examples:
  wavepeek property --waves dump.fst \
    --scope tb.dut --on 'posedge clk' \
    --eval 'req && !ack'

  wavepeek property --waves dump.fst \
    --on '*' --sample-mode native \
    --eval "top.error != 8'h00" --capture assert

Notes:
- This is a sampled Boolean check, not a SystemVerilog temporal assertion.
- JSON and JSONL rows contain `time` for the selected event and `sample_time` for the expression sample. Text shows `sample@<time>` only when they differ.
- See evaluate-properties.md for common queries, boolean-expressions.md for `--eval`, and sampling.md for sampling modes.
- See machine-output.md for JSON and JSONL records."#
    )]
    Property(property::PropertyArgs),
    #[command(
        subcommand,
        disable_help_subcommand = true,
        about = "Extract event rows from waveform signals.",
        long_about = r#"Extract event rows from waveform signals.

Behavior:
- `generic` selects clocked events with an event expression and Boolean predicate, then samples an ordered payload.
- Protocol extractors map standard interface signals and emit protocol-specific channel or phase events.

Examples:
  wavepeek extract generic --waves dump.fst \
    --scope top.fifo --on 'posedge clk' \
    --when 'valid && ready' --payload data

  wavepeek extract axi --waves dump.fst \
    --scope top.axi --profile axi4 \
    --map aclk=clk --include '^m_axi_'

Notes:
- Extractors report sampled events. They do not perform full protocol checking or reconstruct high-level transactions.
- See extract-transfers.md for generic extraction and extract-axi.md, extract-axis.md, extract-ahb.md, extract-apb.md, or extract-atb.md for AMBA examples."#
    )]
    Extract(extract::ExtractCommand),
}

#[derive(Debug, Subcommand)]
enum HelperCommand {
    #[command(
        about = "Extract the packaged agent skill into a directory.",
        long_about = r#"Extract the packaged agent skill into a directory.

Behavior:
- Writes the skill package that matches the installed `wavepeek` version.
- Requires a new or empty destination directory.

Example:
  wavepeek skill ./wavepeek-skill

Notes:
- Follow your agent harness's instructions to install the extracted directory.
- See quickstart.md for extraction and installation."#
    )]
    Skill(skill::SkillArgs),
    #[command(
        about = "Show detailed help for a command path.",
        long_about = r#"Show detailed help for a command path.

Behavior:
- With no command path, prints top-level help.
- With a command path, prints help for that command or nested subcommand.

Examples:
  wavepeek help
  wavepeek help value
  wavepeek help extract axi

Notes:
- See commands.md for a summary of the command groups."#,
        help_template = "{about-with-newline}\nUsage: wavepeek help [COMMAND]...\n\nArguments:\n  [COMMAND]...  Command path to describe (for example, extract axi)\n"
    )]
    Help(HelpArgs),
}

#[derive(Debug, Args)]
pub(super) struct HelpArgs {
    /// Command path to describe (for example, extract axi)
    #[arg(value_name = "COMMAND")]
    pub(super) command: Vec<String>,
}

struct OptionalFeatureHelp {
    name: &'static str,
    enabled: bool,
    disabled_hint: &'static str,
}

const OPTIONAL_FEATURES: &[OptionalFeatureHelp] = &[
    OptionalFeatureHelp {
        name: "FSDB",
        enabled: cfg!(feature = "fsdb"),
        disabled_hint: "FSDB support is currently Linux x86_64 only; reinstall with Cargo flag `--features fsdb` and provide the Synopsys Verdi FSDB Reader SDK",
    },
    OptionalFeatureHelp {
        name: "SHM",
        enabled: cfg!(feature = "shm"),
        disabled_hint: "SHM requires Linux x86_64, Cargo flag `--features shm`, and a compatible XCELIUM_HOME",
    },
];

const BROWSER_LONG_ABOUT: &str = r#"wavepeek queries saved RTL waveform dumps.

Browser behavior:
- Waveform commands read the active demo or local VCD or FST file.
- Parsing and queries run locally in the browser worker.
- FSDB, SHM, `skill`, and extraction `--source <FILE>` options are not available.
- Output and time rules match the native `wavepeek` CLI."#;
const BROWSER_HELP_TEMPLATE: &str = "{about-with-newline}\nUsage: {usage}\n\nWaveform commands:\n  info      Show metadata for one waveform dump.\n  scope     List scopes in a waveform hierarchy.\n  signal    List signals in one waveform scope.\n  value     Read selected signal values at one or more times.\n  change    Read signal values at selected events over a time range.\n  property  Evaluate a Boolean expression at selected events.\n  extract   Extract event rows from waveform signals.\n\nHelper commands:\n  help      Show detailed help for a command path.\n\nOptions:\n{options}{after-help}";
const BROWSER_NEXT_STEPS: &str =
    "Browser limits: FSDB/SHM input, skill, and extraction --source files are unavailable.";

fn root_after_long_help() -> String {
    let mut help = String::from("Optional features:\n");
    for feature in OPTIONAL_FEATURES {
        if feature.enabled {
            help.push_str("- ");
            help.push_str(feature.name);
            help.push_str(" - enabled\n");
        } else {
            help.push_str("- ");
            help.push_str(feature.name);
            help.push_str(" - disabled (");
            help.push_str(feature.disabled_hint);
            help.push_str(")\n");
        }
    }
    help
}

#[derive(Clone, Copy)]
struct OutputSelection {
    mode: OutputMode,
    json: usize,
    jsonl: usize,
}

pub(crate) struct CliFailure {
    pub(crate) error: WavepeekError,
    pub(crate) reported: bool,
}

impl From<WavepeekError> for CliFailure {
    fn from(error: WavepeekError) -> Self {
        Self {
            error,
            reported: false,
        }
    }
}

pub(crate) fn run(report_machine_errors: bool) -> Result<(), CliFailure> {
    run_from(
        std::env::args_os().collect(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
        report_machine_errors,
        false,
    )
}

pub(crate) fn run_from(
    argv: Vec<OsString>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    report_machine_errors: bool,
    browser: bool,
) -> Result<(), CliFailure> {
    let (selection, argv) = extract_output_selection(argv);

    match execute(
        argv,
        selection,
        stdout,
        stderr,
        report_machine_errors,
        browser,
    ) {
        Ok(()) => Ok(()),
        Err(CliFailure {
            error: WavepeekError::BrokenPipe,
            ..
        }) if report_machine_errors => Ok(()),
        Err(failure) if failure.reported || !report_machine_errors => Err(failure),
        Err(failure) => report_failure(selection.mode, failure.error, stdout),
    }
}

fn execute(
    argv: Vec<OsString>,
    selection: OutputSelection,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    report_machine_errors: bool,
    browser: bool,
) -> Result<(), CliFailure> {
    if selection.mode != OutputMode::Human {
        if selection.json > 1 {
            return Err(WavepeekError::Args(
                "the argument '--json' cannot be used multiple times".to_string(),
            )
            .into());
        }
        if selection.jsonl > 1 {
            return Err(WavepeekError::Args(
                "the argument '--jsonl' cannot be used multiple times".to_string(),
            )
            .into());
        }
        if selection.json > 0 && selection.jsonl > 0 {
            return Err(WavepeekError::Args(
                "the argument '--json' cannot be used with '--jsonl'".to_string(),
            )
            .into());
        }
    }

    let parse_argv = if argv.len() == 1 && selection.mode == OutputMode::Human {
        vec![argv[0].clone(), "-h".into()]
    } else if argv.len() == 2
        && selection.mode == OutputMode::Human
        && matches!(argv[1].to_str(), Some("extract"))
    {
        vec![argv[0].clone(), argv[1].clone(), "-h".into()]
    } else {
        argv
    };

    let matches = match build_cli_command_for(browser).try_get_matches_from(parse_argv) {
        Ok(matches) => matches,
        Err(error) => return handle_parse_error(error, stdout).map_err(Into::into),
    };

    if change_tune_overrides_requested(&matches) && !is_debug_mode_enabled() {
        return Err(WavepeekError::Args(
            "internal tuning overrides (--tune-*) require DEBUG=1. Set DEBUG=1 only for local diagnostics or CI debugging."
                .to_string(),
        )
        .into());
    }

    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => return handle_parse_error(error, stdout).map_err(Into::into),
    };

    if cli.version_semver {
        output::write_to(stdout, env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }

    if cli.version_full {
        output::write_to(stdout, concat!("wavepeek v", env!("CARGO_PKG_VERSION")))?;
        return Ok(());
    }

    let Some(command) = cli.command else {
        return Err(WavepeekError::Args("a waveform command is required".to_string()).into());
    };

    match command {
        Command::Helper(HelperCommand::Help(args)) => {
            write_command_help(&args.command, browser, stdout)
        }
        Command::Waveform(WaveformCommand::Extract(extract::ExtractCommand::Help(args))) => {
            let mut command = vec!["extract".to_string()];
            command.extend(args.command);
            write_command_help(&command, browser, stdout)
        }
        command => dispatch(
            command,
            selection.mode,
            stdout,
            stderr,
            report_machine_errors,
        ),
    }
}

fn write_command_help(
    path: &[String],
    browser: bool,
    stdout: &mut dyn Write,
) -> Result<(), CliFailure> {
    if path.iter().any(|segment| segment == "--") {
        return Err(
            WavepeekError::Args("help command paths cannot contain '--'".to_string()).into(),
        );
    }

    let argv = std::iter::once(OsString::from("wavepeek"))
        .chain(path.iter().map(OsString::from))
        .chain(std::iter::once(OsString::from("--help")));
    match build_cli_command_for(browser).try_get_matches_from(argv) {
        Ok(_) => Err(WavepeekError::Args("invalid help command path".to_string()).into()),
        Err(error) => handle_parse_error(error, stdout).map_err(Into::into),
    }
}

fn report_failure(
    mode: OutputMode,
    error: WavepeekError,
    stdout: &mut dyn Write,
) -> Result<(), CliFailure> {
    let reported = match mode {
        OutputMode::Human => return Err(error.into()),
        OutputMode::Json => output::write_json_fatal_to(&error, stdout),
        OutputMode::Jsonl => output::write_jsonl_fatal_to(&error, stdout),
    };

    match reported {
        Ok(()) => Err(CliFailure {
            error,
            reported: true,
        }),
        Err(WavepeekError::BrokenPipe) => Ok(()),
        Err(write_error) => Err(write_error.into()),
    }
}

fn extract_output_selection(argv: Vec<OsString>) -> (OutputSelection, Vec<OsString>) {
    let command = build_cli_command();
    let mut json = 0;
    let mut jsonl = 0;
    let mut options_ended = false;
    let mut parse_argv = Vec::with_capacity(argv.len());

    for (index, arg) in argv.iter().enumerate() {
        if index == 0 {
            parse_argv.push(arg.clone());
            continue;
        }
        if arg == OsStr::new("--") {
            options_ended = true;
            parse_argv.push(arg.clone());
            continue;
        }
        if !options_ended && !is_known_option_value(&command, &argv, index) {
            if arg == OsStr::new("--json") {
                json += 1;
                continue;
            }
            if arg == OsStr::new("--jsonl") {
                jsonl += 1;
                continue;
            }
        }
        parse_argv.push(arg.clone());
    }

    let mode = OutputMode::from_json_flags(json > 0, jsonl > 0);
    (OutputSelection { mode, json, jsonl }, parse_argv)
}

fn is_known_option_value(command: &clap::Command, argv: &[OsString], index: usize) -> bool {
    let Some(previous) = index.checked_sub(1).and_then(|index| argv[index].to_str()) else {
        return false;
    };
    let Some(name) = previous.strip_prefix("--") else {
        return false;
    };
    let active_command = argv[1..index].iter().fold(command, |command, arg| {
        arg.to_str()
            .and_then(|name| command.find_subcommand(name))
            .unwrap_or(command)
    });
    !name.contains('=')
        && active_command.get_arguments().any(|arg| {
            arg.get_long() == Some(name)
                && arg.get_action().takes_values()
                && arg.is_allow_hyphen_values_set()
        })
}

fn is_debug_mode_enabled() -> bool {
    std::env::var("DEBUG")
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn change_tune_overrides_requested(matches: &clap::ArgMatches) -> bool {
    let Some(("change", change_matches)) = matches.subcommand() else {
        return false;
    };

    is_command_line_override(change_matches, "tune_engine")
        || is_command_line_override(change_matches, "tune_candidates")
        || is_command_line_override(change_matches, "tune_edge_fast_force")
}

fn is_command_line_override(matches: &clap::ArgMatches, arg: &str) -> bool {
    matches!(matches.value_source(arg), Some(ValueSource::CommandLine))
}

fn build_cli_command() -> clap::Command {
    build_cli_command_for(false)
}

fn build_cli_command_for(browser: bool) -> clap::Command {
    let mut command = Cli::command().after_long_help(root_after_long_help());
    for command_name in ["info", "scope", "signal", "value", "change", "property"] {
        if let Some(subcommand) = command.find_subcommand_mut(command_name) {
            *subcommand = with_other_help_options(subcommand.clone());
        }
    }
    if let Some(extract) = command.find_subcommand_mut("extract") {
        for name in ["ahb", "apb", "atb", "axi", "axistream", "generic"] {
            if let Some(subcommand) = extract.find_subcommand_mut(name) {
                let mut updated = with_other_help_options(subcommand.clone());
                if browser {
                    updated = updated.mut_arg("source", |arg| arg.hide(true));
                    if let Some(help) = updated.get_long_about() {
                        let lines = help
                            .to_string()
                            .lines()
                            .map(str::to_owned)
                            .collect::<Vec<_>>();
                        let help = lines
                            .iter()
                            .enumerate()
                            .filter_map(|(index, line)| {
                                let lowercase = line.to_ascii_lowercase();
                                let continued_by_source = line.trim_end().ends_with('\\')
                                    && lines.get(index + 1).is_some_and(|next| {
                                        next.to_ascii_lowercase().contains("--source")
                                    });
                                (!continued_by_source
                                    && !lowercase.contains("source file")
                                    && !lowercase.contains("source-file")
                                    && !lowercase.contains("--source"))
                                .then_some(line.as_str())
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        updated = updated.long_about(help);
                    }
                }
                *subcommand = updated;
            }
        }
    }
    if browser {
        if let Some(skill) = command.find_subcommand_mut("skill") {
            *skill = skill.clone().hide(true);
        }
        command = command
            .long_about(BROWSER_LONG_ABOUT)
            .help_template(BROWSER_HELP_TEMPLATE)
            .after_help(BROWSER_NEXT_STEPS)
            .after_long_help(BROWSER_NEXT_STEPS);
    }
    command
}

fn with_other_help_options(command: clap::Command) -> clap::Command {
    command
        .disable_help_flag(true)
        .arg(
            Arg::new("help_short")
                .short('h')
                .action(ArgAction::HelpShort)
                .help("Print help (see more with '--help')")
                .help_heading("Other options"),
        )
        .arg(
            Arg::new("help_long")
                .long("help")
                .action(ArgAction::HelpLong)
                .help("Print help (see a summary with '-h')")
                .help_heading("Other options"),
        )
}

fn handle_parse_error(error: clap::Error, stdout: &mut dyn Write) -> Result<(), WavepeekError> {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => stdout
            .write_all(error.render().to_string().as_bytes())
            .map_err(output::map_stdout_io_error),
        _ => Err(WavepeekError::Args(normalize_clap_error(&error))),
    }
}

fn normalize_clap_error(error: &clap::Error) -> String {
    let rendered = error.render().to_string();
    let detail = clap_error_detail(rendered.as_str());
    let hint = help_hint_for_rendered_clap_error(rendered.as_str());

    format!("{detail} {hint}")
}

fn clap_error_detail(rendered: &str) -> String {
    let lines: Vec<&str> = rendered.lines().collect();
    if let Some(start_index) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("error:"))
    {
        let mut chunks = Vec::new();

        for (index, line) in lines.iter().enumerate().skip(start_index) {
            let trimmed = line.trim();
            if index > start_index
                && (trimmed.starts_with("Usage:") || trimmed.starts_with("For more information"))
            {
                break;
            }

            if index == start_index {
                if let Some(rest) = trimmed.strip_prefix("error:") {
                    let rest = rest.trim();
                    if !rest.is_empty() {
                        chunks.push(rest.to_string());
                    }
                }
                continue;
            }

            if !trimmed.is_empty() {
                chunks.push(trimmed.to_string());
            }
        }

        if !chunks.is_empty() {
            return chunks.join(" ");
        }
    }

    for line in lines {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("error:") {
            return rest.trim().to_string();
        }
    }

    rendered
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| "invalid arguments".to_string())
}

fn help_hint_for_rendered_clap_error(rendered: &str) -> String {
    let usage_line = rendered
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("Usage:"));

    let Some(usage_line) = usage_line else {
        return "See 'wavepeek --help'.".to_string();
    };

    let usage = usage_line.trim_start_matches("Usage:").trim();
    let mut parts = usage.split_whitespace();
    let Some(command_name) = parts.next() else {
        return "See 'wavepeek --help'.".to_string();
    };
    if command_name != "wavepeek" {
        return "See 'wavepeek --help'.".to_string();
    }

    let mut path_tokens = Vec::new();
    for token in parts {
        if token.starts_with('[') || token.starts_with('<') || token.starts_with('-') {
            break;
        }
        path_tokens.push(token);
    }

    if path_tokens.is_empty() {
        return "See 'wavepeek --help'.".to_string();
    }

    format!("See 'wavepeek {} --help'.", path_tokens.join(" "))
}

fn dispatch(
    command: Command,
    output_mode: OutputMode,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    report_machine_errors: bool,
) -> Result<(), CliFailure> {
    let mut engine_command = into_engine_command(command);
    if matches!(engine_command, EngineCommand::Skill(_)) && output_mode != OutputMode::Human {
        return Err(WavepeekError::Args(
            "--json and --jsonl are available only for waveform commands".to_string(),
        )
        .into());
    }
    engine_command.set_output_mode(output_mode);

    if output_mode == OutputMode::Jsonl {
        let mut writer = JsonlWriter::new(stdout, engine_command.name());
        return match engine::run_jsonl(engine_command, &mut writer) {
            Ok(()) => Ok(()),
            Err(WavepeekError::BrokenPipe) => Err(WavepeekError::BrokenPipe.into()),
            Err(error) if !report_machine_errors => Err(error.into()),
            Err(error) => match writer.fatal(&error) {
                Ok(()) => Err(CliFailure {
                    error,
                    reported: true,
                }),
                Err(write_error) => Err(write_error.into()),
            },
        };
    }

    let result = engine::run(engine_command)?;
    output::write_result_to(result, stdout, stderr).map_err(Into::into)
}

fn into_engine_command(command: Command) -> EngineCommand {
    match command {
        Command::Waveform(command) => match command {
            WaveformCommand::Info(args) => EngineCommand::Info(args),
            WaveformCommand::Scope(args) => EngineCommand::Scope(args),
            WaveformCommand::Signal(args) => EngineCommand::Signal(args),
            WaveformCommand::Value(args) => EngineCommand::Value(args),
            WaveformCommand::Change(args) => EngineCommand::Change(args),
            WaveformCommand::Property(args) => EngineCommand::Property(args),
            WaveformCommand::Extract(command) => match command {
                extract::ExtractCommand::Ahb(args) => EngineCommand::ExtractAhb(*args),
                extract::ExtractCommand::Apb(args) => EngineCommand::ExtractApb(*args),
                extract::ExtractCommand::Atb(args) => EngineCommand::ExtractAtb(*args),
                extract::ExtractCommand::Axi(args) => EngineCommand::ExtractAxi(*args),
                extract::ExtractCommand::AxiStream(args) => EngineCommand::ExtractAxiStream(*args),
                extract::ExtractCommand::Generic(args) => EngineCommand::ExtractGeneric(*args),
                extract::ExtractCommand::Help(_) => {
                    unreachable!("help commands are handled before dispatch")
                }
            },
        },
        Command::Helper(command) => match command {
            HelperCommand::Skill(args) => EngineCommand::Skill(args),
            HelperCommand::Help(_) => unreachable!("help commands are handled before dispatch"),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::{CommandFactory, Parser};

    use crate::cli::limits::LimitArg;

    use super::{
        Cli, EngineCommand, build_cli_command, build_cli_command_for,
        change_tune_overrides_requested, clap_error_detail, handle_parse_error,
        help_hint_for_rendered_clap_error, into_engine_command, normalize_clap_error,
    };

    #[test]
    fn browser_generic_help_removes_complete_source_example() {
        let mut command = build_cli_command_for(true);
        let generic = command
            .find_subcommand_mut("extract")
            .and_then(|extract| extract.find_subcommand_mut("generic"))
            .expect("generic extractor command");
        let mut rendered = Vec::new();
        generic
            .write_long_help(&mut rendered)
            .expect("browser generic help should render");
        let help = String::from_utf8(rendered).expect("help should be UTF-8");

        assert!(!help.contains("--source"));
        assert!(!help.contains("fifo-sources.json"));
        assert_eq!(
            help.matches("wavepeek extract generic --waves dump.fst \\")
                .count(),
            1,
            "browser help should retain only the CLI example"
        );
        assert!(help.contains("extract-transfers.md"));
    }

    #[test]
    fn info_dispatch_keeps_json_and_waves_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "info",
            "--waves",
            "fixtures/sample.vcd",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Info(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert!(args.json);
            }
            other => panic!("expected info command, got {other:?}"),
        }
    }

    #[test]
    fn scope_dispatch_keeps_bounded_query_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "scope",
            "--waves",
            "fixtures/sample.vcd",
            "--max",
            "12",
            "--max-depth",
            "3",
            "--filter",
            "^top\\..*",
            "--tree",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Scope(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, LimitArg::Numeric(12));
                assert_eq!(args.max_depth, LimitArg::Numeric(3));
                assert_eq!(args.filter, "^top\\..*");
                assert!(args.tree);
                assert!(args.json);
            }
            other => panic!("expected scope command, got {other:?}"),
        }
    }

    #[test]
    fn scope_dispatch_accepts_unlimited_limit_literals() {
        let cli = Cli::parse_from([
            "wavepeek",
            "scope",
            "--waves",
            "fixtures/sample.vcd",
            "--max",
            "unlimited",
            "--max-depth",
            "unlimited",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Scope(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, LimitArg::Unlimited);
                assert_eq!(args.max_depth, LimitArg::Unlimited);
            }
            other => panic!("expected scope command, got {other:?}"),
        }
    }

    #[test]
    fn signal_dispatch_keeps_recursive_and_max_depth_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "signal",
            "--waves",
            "fixtures/sample.vcd",
            "--scope",
            "top.cpu",
            "--recursive",
            "--max-depth",
            "3",
            "--max",
            "7",
            "--filter",
            ".*clk.*",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Signal(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.scope, "top.cpu");
                assert!(args.recursive);
                assert_eq!(args.max_depth, LimitArg::Numeric(3));
                assert_eq!(args.max, LimitArg::Numeric(7));
                assert_eq!(args.filter, ".*clk.*");
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected signal command, got {other:?}"),
        }
    }

    #[test]
    fn value_dispatch_keeps_scope_signals_abs_and_json_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "value",
            "--waves",
            "fixtures/sample.vcd",
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Value(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.at, vec!["10ns"]);
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.signals, vec!["clk", "data"]);
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected value command, got {other:?}"),
        }
    }

    #[test]
    fn change_dispatch_keeps_on_abs_and_limits() {
        let cli = Cli::parse_from([
            "wavepeek",
            "change",
            "--waves",
            "fixtures/sample.vcd",
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--on",
            "posedge clk",
            "--max",
            "12",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Change(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.from.as_deref(), Some("1ns"));
                assert_eq!(args.to.as_deref(), Some("10ns"));
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.signals, vec!["clk", "data"]);
                assert_eq!(args.on, "posedge clk");
                assert_eq!(args.sample_mode, crate::cli::sampling::SampleMode::PreEdge);
                assert_eq!(args.row_mode, crate::cli::change::RowMode::Dense);
                assert_eq!(args.row_values, crate::cli::change::RowValues::Full);
                assert_eq!(args.max, LimitArg::Numeric(12));
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected change command, got {other:?}"),
        }
    }

    #[test]
    fn property_dispatch_parses_capture_default() {
        let cli = Cli::parse_from([
            "wavepeek",
            "property",
            "--waves",
            "fixtures/sample.vcd",
            "--on",
            "posedge top.clk",
            "--eval",
            "1",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Property(args) => {
                assert_eq!(args.on, "posedge top.clk");
                assert_eq!(args.sample_mode, crate::cli::sampling::SampleMode::PreEdge);
                assert_eq!(args.eval, "1");
                assert_eq!(args.capture, crate::cli::property::CaptureMode::Switch);
                assert_eq!(args.max, LimitArg::Numeric(50));
            }
            other => panic!("expected property command, got {other:?}"),
        }
    }

    #[test]
    fn property_dispatch_accepts_unlimited_max() {
        let cli = Cli::parse_from([
            "wavepeek",
            "property",
            "--waves",
            "fixtures/sample.vcd",
            "--on",
            "posedge top.clk",
            "--eval",
            "1",
            "--max",
            "unlimited",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Property(args) => {
                assert_eq!(args.max, LimitArg::Unlimited);
            }
            other => panic!("expected property command, got {other:?}"),
        }
    }

    #[test]
    fn extract_ahb_dispatch_keeps_pipeline_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "extract",
            "ahb",
            "--waves",
            "fixtures/sample.vcd",
            "--profile",
            "ahb5",
            "--scope",
            "top",
            "--name",
            "dmem",
            "--map",
            "hclk=clk",
            "--include",
            "^dmem_",
            "--include-stall",
            "--include-idle",
            "--include-busy",
            "--max",
            "unlimited",
            "--abs",
            "--jsonl",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::ExtractAhb(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.profile.as_str(), "ahb5");
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.name.as_deref(), Some("dmem"));
                assert_eq!(args.maps, ["hclk=clk"]);
                assert_eq!(args.includes, ["^dmem_"]);
                assert!(args.include_stall);
                assert!(args.include_idle);
                assert!(args.include_busy);
                assert_eq!(args.max, LimitArg::Unlimited);
                assert!(args.abs);
                assert!(args.jsonl);
            }
            other => panic!("expected extract ahb command, got {other:?}"),
        }
    }

    #[test]
    fn extract_generic_dispatch_keeps_single_source_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "extract",
            "generic",
            "--waves",
            "fixtures/sample.vcd",
            "--scope",
            "top",
            "--name",
            "rx.beat",
            "--on",
            "posedge clk",
            "--when",
            "valid && ready",
            "--payload",
            "data,last",
            "--max",
            "unlimited",
            "--abs",
            "--jsonl",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::ExtractGeneric(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.name.as_deref(), Some("rx.beat"));
                assert_eq!(args.on.as_deref(), Some("posedge clk"));
                assert_eq!(args.when.as_deref(), Some("valid && ready"));
                assert_eq!(
                    args.payload,
                    Some(vec!["data".to_string(), "last".to_string()])
                );
                assert_eq!(args.max, LimitArg::Unlimited);
                assert!(args.abs);
                assert!(args.jsonl);
            }
            other => panic!("expected extract generic command, got {other:?}"),
        }
    }

    #[test]
    fn clap_errors_are_normalized_to_single_line_message() {
        let error = Cli::try_parse_from(["wavepeek", "info", "--unknown"])
            .expect_err("unknown argument should fail");

        let normalized = normalize_clap_error(&error);
        assert!(normalized.contains("unexpected argument '--unknown'"));
        assert!(normalized.contains("See 'wavepeek info --help'."));
        assert!(!normalized.contains("Usage:"));
    }

    #[test]
    fn clap_error_detail_preserves_missing_argument_names() {
        let rendered = "error: the following required arguments were not provided:\n  --waves <FILE>\n\nUsage: wavepeek info --waves <FILE>\n\nFor more information, try '--help'.\n";

        let normalized = clap_error_detail(rendered);
        assert!(normalized.contains("the following required arguments were not provided"));
        assert!(normalized.contains("--waves <FILE>"));
        assert!(!normalized.contains("Usage:"));
    }

    #[test]
    fn help_hint_uses_global_help_for_top_level_parse_failures() {
        let rendered = "error: unexpected argument '--wat' found\n\nUsage: wavepeek [OPTIONS] <COMMAND>\n\nFor more information, try '--help'.\n";
        let hint = help_hint_for_rendered_clap_error(rendered);
        assert_eq!(hint, "See 'wavepeek --help'.");
    }

    #[test]
    fn help_hint_uses_subcommand_help_for_subcommand_parse_failures() {
        let rendered = "error: unexpected argument '--wat' found\n\nUsage: wavepeek info --waves <FILE>\n\nFor more information, try '--help'.\n";
        let hint = help_hint_for_rendered_clap_error(rendered);
        assert_eq!(hint, "See 'wavepeek info --help'.");
    }

    #[test]
    fn cli_helper_functions_exercise_override_detection_and_fallback_paths() {
        let command = build_cli_command();
        let info = command
            .find_subcommand("info")
            .expect("info subcommand should exist")
            .clone();
        assert!(info.get_arguments().any(|arg| arg.get_id() == "help_short"));
        assert!(info.get_arguments().any(|arg| arg.get_id() == "help_long"));

        let matches = Cli::command()
            .try_get_matches_from([
                "wavepeek",
                "change",
                "--waves",
                "fixtures/sample.vcd",
                "--signals",
                "clk",
                "--on",
                "*",
                "--sample-mode",
                "native",
                "--tune-engine",
                "fused",
            ])
            .expect("change command should parse");
        assert!(change_tune_overrides_requested(&matches));

        let matches = Cli::command()
            .try_get_matches_from([
                "wavepeek",
                "change",
                "--waves",
                "fixtures/sample.vcd",
                "--signals",
                "clk",
                "--on",
                "*",
                "--sample-mode",
                "native",
            ])
            .expect("change command should parse");
        assert!(!change_tune_overrides_requested(&matches));

        let help_error = Cli::command()
            .try_get_matches_from(["wavepeek", "info", "--help"])
            .expect_err("--help should short-circuit through clap");
        handle_parse_error(help_error, &mut Vec::new())
            .expect("display-help errors should print cleanly");

        assert_eq!(
            clap_error_detail("plain fallback detail"),
            "plain fallback detail"
        );
        assert_eq!(clap_error_detail("\n\n   \n"), "invalid arguments");
        assert_eq!(
            help_hint_for_rendered_clap_error("Usage: nope info --waves <FILE>\n"),
            "See 'wavepeek --help'."
        );
    }
}
