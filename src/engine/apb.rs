use std::collections::{HashMap, HashSet};
use std::fs;

use regex::Regex;
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::cli::extract::ApbArgs;
use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{SharedWaveform, open_shared_waveform};
use crate::engine::extract::{
    self, ExtractGenericRow, ExtractPlan, ExtractRowSink, ExtractRunArgs, ExtractRunStats,
    ExtractSource,
};
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;

const DEFAULT_PROFILE: &str = "apb4";
const DEFAULT_PREADY_MODE: &str = "mapped";
const DEFAULT_NAME: &str = "apb";
const SOURCE_KIND: &str = "extract.apb.source";
const HELP: &str = "wavepeek extract apb";
const ISSUE: &str = "E";

const ORDERED_SIGNALS: &[&str] = &[
    "pclk", "presetn", "psel", "penable", "pwrite", "pready", "paddr", "pprot", "pnse", "pauser",
    "pwdata", "pstrb", "pwuser", "prdata", "pslverr", "pruser", "pbuser",
];
const REQUIRED_SIGNALS: &[&str] = &["pclk", "psel", "penable", "pwrite"];
const COMPLETION_ONLY_SIGNALS: &[&str] = &["prdata", "pslverr", "pruser", "pbuser"];
const WRITE_ONLY_SIGNALS: &[&str] = &["pwdata", "pwuser"];
const READ_ONLY_SIGNALS: &[&str] = &["prdata", "pruser"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApbSignalMapping {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApbEventPayload {
    pub standard: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApbEvent {
    pub time: String,
    pub sample_time: String,
    pub profile: String,
    pub event: String,
    pub direction: String,
    pub payload: Vec<ApbEventPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApbContext {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub pready_mode: String,
    pub include_wait: bool,
    pub mappings: Vec<ApbSignalMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApbData {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub pready_mode: String,
    pub include_wait: bool,
    pub mappings: Vec<ApbSignalMapping>,
    pub events: Vec<ApbEvent>,
}

impl ApbData {
    pub(crate) fn context(&self) -> ApbContext {
        ApbContext {
            name: self.name.clone(),
            profile: self.profile.clone(),
            issue: self.issue.clone(),
            pready_mode: self.pready_mode.clone(),
            include_wait: self.include_wait,
            mappings: self.mappings.clone(),
        }
    }
}

#[derive(Debug)]
struct ApbOutcome {
    context: ApbContext,
    diagnostics: Vec<Diagnostic>,
    stats: ExtractRunStats,
}

trait ApbEventSink {
    fn start(&mut self, _context: &ApbContext) -> Result<(), WavepeekError> {
        Ok(())
    }

    fn emit(&mut self, event: ApbEvent) -> Result<(), WavepeekError>;
}

#[derive(Default)]
struct CollectingApbSink {
    events: Vec<ApbEvent>,
}

impl ApbEventSink for CollectingApbSink {
    fn emit(&mut self, event: ApbEvent) -> Result<(), WavepeekError> {
        self.events.push(event);
        Ok(())
    }
}

struct JsonlApbSink<'a, W: std::io::Write> {
    writer: &'a mut crate::output::JsonlWriter<W>,
}

impl<W: std::io::Write> ApbEventSink for JsonlApbSink<'_, W> {
    fn start(&mut self, context: &ApbContext) -> Result<(), WavepeekError> {
        self.writer.begin_context(context)
    }

    fn emit(&mut self, event: ApbEvent) -> Result<(), WavepeekError> {
        self.writer.item(&event)
    }
}

struct GenericToApbSink<'a, S: ApbEventSink + ?Sized> {
    context: &'a ApbContext,
    payload_standards: &'a HashMap<String, Vec<String>>,
    sink: &'a mut S,
}

impl<S: ApbEventSink + ?Sized> ExtractRowSink for GenericToApbSink<'_, S> {
    fn start(&mut self) -> Result<(), WavepeekError> {
        self.sink.start(self.context)
    }

    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError> {
        let standards = self
            .payload_standards
            .get(row.source.as_str())
            .ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "missing APB payload metadata for event '{}'",
                    row.source
                ))
            })?;
        if standards.len() != row.payload.len() {
            return Err(WavepeekError::Internal(format!(
                "APB payload metadata length mismatch for event '{}'",
                row.source
            )));
        }

        let direction = standards
            .iter()
            .zip(row.payload.iter())
            .find(|(standard, _)| standard.as_str() == "pwrite")
            .map(|(_, payload)| direction_from_pwrite(payload.value.as_str()))
            .ok_or_else(|| {
                WavepeekError::Internal("APB event has no pwrite payload".to_string())
            })?;
        let event_kind = row.source;
        let payload = row
            .payload
            .into_iter()
            .zip(standards.iter())
            .filter(|(_, standard)| payload_allowed(event_kind.as_str(), direction, standard))
            .map(|(payload, standard)| ApbEventPayload {
                standard: standard.clone(),
                path: payload.path,
                value: payload.value,
            })
            .collect();

        self.sink.emit(ApbEvent {
            time: row.time,
            sample_time: row.sample_time,
            profile: self.context.profile.clone(),
            event: event_kind,
            direction: direction.as_str().to_string(),
            payload,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApbDirection {
    Read,
    Write,
    Unknown,
}

impl ApbDirection {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Unknown => "unknown",
        }
    }
}

fn direction_from_pwrite(value: &str) -> ApbDirection {
    let Some((_, digits)) = value.split_once("'h") else {
        return ApbDirection::Unknown;
    };
    if digits.chars().any(|digit| matches!(digit, 'x' | 'z')) {
        return ApbDirection::Unknown;
    }
    let significant = digits.trim_start_matches('0');
    if significant.is_empty() {
        ApbDirection::Read
    } else if significant == "1" {
        ApbDirection::Write
    } else {
        ApbDirection::Unknown
    }
}

fn payload_allowed(event: &str, direction: ApbDirection, standard: &str) -> bool {
    if event != "access-complete" && COMPLETION_ONLY_SIGNALS.contains(&standard) {
        return false;
    }
    match direction {
        ApbDirection::Read => !WRITE_ONLY_SIGNALS.contains(&standard),
        ApbDirection::Write => !READ_ONLY_SIGNALS.contains(&standard),
        ApbDirection::Unknown => true,
    }
}

#[derive(Debug, Deserialize)]
struct SourceFile {
    #[serde(rename = "$schema")]
    schema: String,
    kind: String,
    #[serde(default, deserialize_with = "optional_string")]
    profile: Option<String>,
    #[serde(default, deserialize_with = "optional_string")]
    pready_mode: Option<String>,
    #[serde(default, deserialize_with = "optional_bool")]
    include_wait: Option<bool>,
    #[serde(default, deserialize_with = "optional_string")]
    name: Option<String>,
    #[serde(default)]
    includes: Vec<String>,
    #[serde(default, deserialize_with = "source_maps")]
    maps: Vec<(String, String)>,
}

fn optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(value) => Ok(Some(value)),
        serde_json::Value::Null => Err(D::Error::custom("expected string, got null")),
        _ => Err(D::Error::custom("expected string")),
    }
}

fn optional_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Bool(value) => Ok(Some(value)),
        serde_json::Value::Null => Err(D::Error::custom("expected boolean, got null")),
        _ => Err(D::Error::custom("expected boolean")),
    }
}

fn source_maps<'de, D>(deserializer: D) -> Result<Vec<(String, String)>, D::Error>
where
    D: Deserializer<'de>,
{
    struct SourceMapsVisitor;

    impl<'de> Visitor<'de> for SourceMapsVisitor {
        type Value = Vec<(String, String)>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object of APB standard-to-waveform signal mappings")
        }

        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut maps = Vec::with_capacity(access.size_hint().unwrap_or(0));
            let mut seen = HashSet::new();
            while let Some((standard, waves)) = access.next_entry::<String, String>()? {
                if !seen.insert(standard.clone()) {
                    return Err(A::Error::custom(format!(
                        "duplicate APB mapping key '{standard}'"
                    )));
                }
                maps.push((standard, waves));
            }
            Ok(maps)
        }
    }

    deserializer.deserialize_map(SourceMapsVisitor)
}

#[derive(Debug)]
struct ApbConfig {
    profile: ApbProfile,
    pready_mode: PreadyMode,
    include_wait: bool,
    name: String,
    includes: Vec<String>,
    maps: Vec<(String, String)>,
}

#[derive(Debug)]
struct BuiltApbPlan {
    context: ApbContext,
    plan: ExtractPlan,
    waveform: SharedWaveform,
    debug: DebugTrace,
    payload_standards: HashMap<String, Vec<String>>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct SignalCandidate {
    display: String,
    name: String,
    path: String,
}

#[derive(Debug, Clone, Copy)]
struct ApbProfile {
    spec: &'static ApbProfileSpec,
}

#[derive(Debug)]
pub(crate) struct ApbProfileSpec {
    pub(crate) name: &'static str,
    pub(crate) issue: &'static str,
    pub(crate) signals: &'static [&'static str],
}

const APB3_SIGNALS: &[&str] = &[
    "pclk", "presetn", "psel", "penable", "pwrite", "pready", "paddr", "pwdata", "prdata",
    "pslverr",
];
const APB4_SIGNALS: &[&str] = &[
    "pclk", "presetn", "psel", "penable", "pwrite", "pready", "paddr", "pprot", "pwdata", "pstrb",
    "prdata", "pslverr",
];
const APB5_SIGNALS: &[&str] = ORDERED_SIGNALS;

const APB3_PROFILE: ApbProfileSpec = ApbProfileSpec {
    name: "apb3",
    issue: ISSUE,
    signals: APB3_SIGNALS,
};
const APB4_PROFILE: ApbProfileSpec = ApbProfileSpec {
    name: "apb4",
    issue: ISSUE,
    signals: APB4_SIGNALS,
};
const APB5_PROFILE: ApbProfileSpec = ApbProfileSpec {
    name: "apb5",
    issue: ISSUE,
    signals: APB5_SIGNALS,
};

pub(crate) fn profile_specs() -> &'static [ApbProfileSpec] {
    &[APB3_PROFILE, APB4_PROFILE, APB5_PROFILE]
}

pub(crate) fn standard_signals(
    profile: &ApbProfileSpec,
) -> impl Iterator<Item = &'static str> + '_ {
    ORDERED_SIGNALS
        .iter()
        .copied()
        .filter(|standard| profile.signals.contains(standard))
}

pub(crate) fn event_payload_signals<'a>(
    profile: &'a ApbProfileSpec,
    event: &'a str,
    direction: &'a str,
) -> impl Iterator<Item = &'static str> + 'a {
    standard_signals(profile).filter(move |standard| {
        !matches!(
            *standard,
            "pclk" | "presetn" | "psel" | "penable" | "pready"
        ) && (event == "access-complete" || !COMPLETION_ONLY_SIGNALS.contains(standard))
            && (direction != "read" || !WRITE_ONLY_SIGNALS.contains(standard))
            && (direction != "write" || !READ_ONLY_SIGNALS.contains(standard))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreadyMode {
    Mapped,
    ImplicitHigh,
}

impl PreadyMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Mapped => "mapped",
            Self::ImplicitHigh => "implicit-high",
        }
    }
}

pub fn run(args: ApbArgs) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let signals_abs = args.abs;
    let mut sink = CollectingApbSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::ExtractApb,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        data: CommandData::ExtractApb(ApbData {
            name: outcome.context.name,
            profile: outcome.context.profile,
            issue: outcome.context.issue,
            pready_mode: outcome.context.pready_mode,
            include_wait: outcome.context.include_wait,
            mappings: outcome.context.mappings,
            events: sink.events,
        }),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: ApbArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    let outcome = {
        let mut sink = JsonlApbSink { writer };
        run_with_sink(args, &mut sink)?
    };

    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end(outcome.stats.truncated)
}

fn run_with_sink<S: ApbEventSink + ?Sized>(
    args: ApbArgs,
    sink: &mut S,
) -> Result<ApbOutcome, WavepeekError> {
    let BuiltApbPlan {
        context,
        plan,
        waveform,
        debug,
        payload_standards,
        diagnostics: build_diagnostics,
    } = build_apb_plan(&args)?;
    let mut generic_sink = GenericToApbSink {
        context: &context,
        payload_standards: &payload_standards,
        sink,
    };
    let outcome = extract::run_plan_with_waveform_sink(
        ExtractRunArgs {
            command: CommandName::ExtractApb,
            help_command: HELP,
            waves: args.waves,
            from: args.from,
            to: args.to,
            scope: args.scope,
            max: args.max,
        },
        plan,
        waveform,
        debug,
        &mut generic_sink,
    )?;

    let mut diagnostics = build_diagnostics;
    diagnostics.extend(outcome.diagnostics);
    Ok(ApbOutcome {
        context,
        diagnostics,
        stats: outcome.stats,
    })
}

fn build_apb_plan(args: &ApbArgs) -> Result<BuiltApbPlan, WavepeekError> {
    let config = config_from_args(args)?;
    let include_regexes = compile_include_regexes(&config.includes)?;
    let debug = DebugTrace::for_command(CommandName::ExtractApb);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = open_shared_waveform(args.waves.as_path())?;
    {
        let waveform_ref = waveform.borrow();
        debug.event("backend.open.done", || {
            serde_json::json!({
                "backend": waveform_ref.backend_name(),
                "format": waveform_ref.format_name(),
            })
        });
    }

    let candidates =
        collect_include_candidates(&waveform, args.scope.as_deref(), &include_regexes)?;
    let explicit = explicit_mappings(
        &waveform,
        args.scope.as_deref(),
        config.profile,
        &config.maps,
    )?;
    let (mappings_by_standard, diagnostics) = auto_mappings(config.profile, candidates, explicit)?;
    validate_required_mappings(
        config.profile,
        config.pready_mode,
        config.include_wait,
        &mappings_by_standard,
    )?;
    let sources = build_extract_sources(
        config.profile,
        config.pready_mode,
        config.include_wait,
        args.scope.as_deref(),
        &mappings_by_standard,
    )?;

    let ordered_mappings = ordered_standard_names(config.profile)
        .into_iter()
        .filter_map(|standard| mappings_by_standard.get(standard).cloned())
        .collect::<Vec<_>>();
    let payload_standards = sources
        .iter()
        .map(|source| (source.event.to_string(), source.payload_standards.clone()))
        .collect();
    let extract_sources = sources
        .into_iter()
        .enumerate()
        .map(|(index, source)| {
            ExtractSource::new(
                index,
                source.event,
                source.on,
                source.when,
                source.payload_waves,
            )
        })
        .collect();

    Ok(BuiltApbPlan {
        context: ApbContext {
            name: config.name,
            profile: config.profile.name().to_string(),
            issue: config.profile.issue().to_string(),
            pready_mode: config.pready_mode.as_str().to_string(),
            include_wait: config.include_wait,
            mappings: ordered_mappings,
        },
        plan: ExtractPlan::new(extract_sources),
        waveform,
        debug,
        payload_standards,
        diagnostics,
    })
}

fn config_from_args(args: &ApbArgs) -> Result<ApbConfig, WavepeekError> {
    if let Some(path) = args.source.as_ref() {
        if args.name.is_some()
            || !args.maps.is_empty()
            || !args.includes.is_empty()
            || args.include_wait
        {
            return Err(WavepeekError::Args(
                "--source cannot be combined with --profile, --pready-mode, --include-wait, --name, --map, or --include. See 'wavepeek extract apb --help'.".to_string(),
            ));
        }
        return config_from_source(path);
    }

    let config = ApbConfig {
        profile: parse_profile(args.profile.as_str())?,
        pready_mode: parse_pready_mode(args.pready_mode.as_str())?,
        include_wait: args.include_wait,
        name: args
            .name
            .clone()
            .unwrap_or_else(|| DEFAULT_NAME.to_string()),
        includes: args.includes.clone(),
        maps: parse_cli_maps(&args.maps)?,
    };
    validate_mode(config.pready_mode, config.include_wait)?;
    Ok(config)
}

fn config_from_source(path: &std::path::Path) -> Result<ApbConfig, WavepeekError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        WavepeekError::File(format!(
            "failed to read APB extract source file '{}': {error}",
            path.display()
        ))
    })?;
    let input: SourceFile = serde_json::from_str(&contents).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid APB extract source JSON '{}': {error}",
            path.display()
        ))
    })?;

    if input.schema != INPUT_SCHEMA_URL {
        return Err(WavepeekError::Args(format!(
            "APB extract source file '{}' uses unsupported $schema {}; expected {}",
            path.display(),
            input.schema,
            INPUT_SCHEMA_URL
        )));
    }
    if input.kind != SOURCE_KIND {
        return Err(WavepeekError::Args(format!(
            "APB extract source file '{}' has kind {}; expected {}",
            path.display(),
            input.kind,
            SOURCE_KIND
        )));
    }

    let maps = input
        .maps
        .into_iter()
        .map(|(standard, waves)| normalize_map_pair(standard.as_str(), waves.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    require_unique_map_standards(&maps)?;
    let config = ApbConfig {
        profile: parse_profile(input.profile.as_deref().unwrap_or(DEFAULT_PROFILE))?,
        pready_mode: parse_pready_mode(
            input.pready_mode.as_deref().unwrap_or(DEFAULT_PREADY_MODE),
        )?,
        include_wait: input.include_wait.unwrap_or(false),
        name: input.name.unwrap_or_else(|| DEFAULT_NAME.to_string()),
        includes: input.includes,
        maps,
    };
    validate_mode(config.pready_mode, config.include_wait)?;
    Ok(config)
}

fn validate_mode(pready_mode: PreadyMode, include_wait: bool) -> Result<(), WavepeekError> {
    if pready_mode == PreadyMode::ImplicitHigh && include_wait {
        return Err(WavepeekError::Args(
            "--include-wait cannot be used with --pready-mode implicit-high. See 'wavepeek extract apb --help'.".to_string(),
        ));
    }
    Ok(())
}

fn parse_cli_maps(values: &[String]) -> Result<Vec<(String, String)>, WavepeekError> {
    let maps = values
        .iter()
        .map(|value| {
            let (standard, waves) = value.split_once('=').ok_or_else(|| {
                WavepeekError::Args(format!(
                    "invalid --map '{}': expected STD_NAME=WAVES_NAME. See 'wavepeek extract apb --help'.",
                    value
                ))
            })?;
            normalize_map_pair(standard, waves)
        })
        .collect::<Result<Vec<_>, _>>()?;
    require_unique_map_standards(&maps)?;
    Ok(maps)
}

fn normalize_map_pair(standard: &str, waves: &str) -> Result<(String, String), WavepeekError> {
    let standard = normalize_standard_name(standard)?;
    let waves = waves.trim();
    if waves.is_empty() {
        return Err(WavepeekError::Args(
            "APB mapped waveform signal names must not be empty. See 'wavepeek extract apb --help'."
                .to_string(),
        ));
    }
    Ok((standard, waves.to_string()))
}

fn normalize_standard_name(standard: &str) -> Result<String, WavepeekError> {
    let standard = standard.trim().to_ascii_lowercase();
    if standard.is_empty() {
        Err(WavepeekError::Args(
            "APB standard signal names must not be empty. See 'wavepeek extract apb --help'."
                .to_string(),
        ))
    } else {
        Ok(standard)
    }
}

fn require_unique_map_standards(maps: &[(String, String)]) -> Result<(), WavepeekError> {
    let mut seen = HashSet::new();
    for (standard, _) in maps {
        if !seen.insert(standard.as_str()) {
            return Err(WavepeekError::Args(format!(
                "duplicate APB mapping for standard signal '{standard}'. See 'wavepeek extract apb --help'."
            )));
        }
    }
    Ok(())
}

fn parse_profile(profile: &str) -> Result<ApbProfile, WavepeekError> {
    let normalized = profile.trim().to_ascii_lowercase();
    let spec = match normalized.as_str() {
        "apb3" => &APB3_PROFILE,
        "apb4" => &APB4_PROFILE,
        "apb5" => &APB5_PROFILE,
        _ => {
            return Err(WavepeekError::Args(format!(
                "unsupported APB profile '{profile}'; expected apb3, apb4, or apb5. See 'wavepeek extract apb --help'."
            )));
        }
    };
    Ok(ApbProfile { spec })
}

fn parse_pready_mode(mode: &str) -> Result<PreadyMode, WavepeekError> {
    match mode.trim().to_ascii_lowercase().as_str() {
        "mapped" => Ok(PreadyMode::Mapped),
        "implicit-high" | "implicit_high" => Ok(PreadyMode::ImplicitHigh),
        _ => Err(WavepeekError::Args(format!(
            "unsupported APB PREADY mode '{mode}'; expected mapped or implicit-high. See 'wavepeek extract apb --help'."
        ))),
    }
}

fn compile_include_regexes(includes: &[String]) -> Result<Vec<Regex>, WavepeekError> {
    includes
        .iter()
        .map(|include| {
            Regex::new(include).map_err(|error| {
                WavepeekError::Args(format!(
                    "invalid APB include regex '{}': {error}. See 'wavepeek extract apb --help'.",
                    include
                ))
            })
        })
        .collect()
}

fn collect_include_candidates(
    waveform: &SharedWaveform,
    scope: Option<&str>,
    includes: &[Regex],
) -> Result<Vec<SignalCandidate>, WavepeekError> {
    if includes.is_empty() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    let mut seen_paths = HashSet::new();
    if let Some(scope) = scope {
        for entry in waveform.borrow().signals_in_scope(scope)? {
            if includes
                .iter()
                .any(|include| include.is_match(entry.name.as_str()))
                && seen_paths.insert(entry.path.clone())
            {
                candidates.push(SignalCandidate {
                    display: entry.name.clone(),
                    name: entry.name,
                    path: entry.path,
                });
            }
        }
        return Ok(candidates);
    }

    let scopes = waveform.borrow().scopes_depth_first(None)?;
    for scope in scopes {
        for entry in waveform.borrow().signals_in_scope(scope.path.as_str())? {
            if includes.iter().any(|include| {
                include.is_match(entry.path.as_str()) || include.is_match(entry.name.as_str())
            }) && seen_paths.insert(entry.path.clone())
            {
                candidates.push(SignalCandidate {
                    display: entry.path.clone(),
                    name: entry.name,
                    path: entry.path,
                });
            }
        }
    }
    Ok(candidates)
}

fn explicit_mappings(
    waveform: &SharedWaveform,
    scope: Option<&str>,
    profile: ApbProfile,
    maps: &[(String, String)],
) -> Result<HashMap<String, ApbSignalMapping>, WavepeekError> {
    let mut result = HashMap::new();
    for (standard, waves) in maps {
        if !profile.contains_standard(standard) {
            return Err(WavepeekError::Args(format!(
                "APB profile {} has no standard signal '{}'. See 'wavepeek extract apb --help'.",
                profile.name(),
                standard
            )));
        }
        if scope.is_some() && waves.contains('.') {
            return Err(WavepeekError::Args(format!(
                "APB mapping for '{standard}' must use a scope-relative signal when --scope is set. See 'wavepeek extract apb --help'."
            )));
        }
        let query_path = match scope {
            Some(scope) => format!("{scope}.{waves}"),
            None => waves.clone(),
        };
        let mut resolved = waveform.borrow().resolve_signals(&[query_path])?;
        let resolved = resolved.remove(0);
        result.insert(
            standard.clone(),
            ApbSignalMapping {
                standard: standard.clone(),
                display: match scope {
                    Some(_) => waves.clone(),
                    None => resolved.path.clone(),
                },
                path: resolved.path,
            },
        );
    }
    Ok(result)
}

fn auto_mappings(
    profile: ApbProfile,
    candidates: Vec<SignalCandidate>,
    mut explicit: HashMap<String, ApbSignalMapping>,
) -> Result<(HashMap<String, ApbSignalMapping>, Vec<Diagnostic>), WavepeekError> {
    let mut diagnostics = Vec::new();
    let mut auto: HashMap<String, Vec<SignalCandidate>> = HashMap::new();
    let standards = ordered_standard_names(profile);
    let explicit_paths = explicit
        .values()
        .map(|mapping| mapping.path.as_str())
        .collect::<HashSet<_>>();

    for candidate in candidates {
        if explicit_paths.contains(candidate.path.as_str()) {
            continue;
        }
        let matched = candidate_matching_standards(candidate.name.as_str(), standards.as_slice());
        if matched.len() > 1 {
            return Err(WavepeekError::Args(format!(
                "ambiguous APB auto-mapping for '{}': matched {}. Add explicit --map entries. See 'wavepeek extract apb --help'.",
                candidate.display,
                matched.join(", ")
            )));
        }
        if matched.is_empty() {
            diagnostics.push(Diagnostic::warning(
                WarningDiagnosticCode::UnmatchedExtractCandidate,
                format!(
                    "ignored APB include candidate '{}' because it did not match any {} standard signal",
                    candidate.display,
                    profile.name()
                ),
            ));
            continue;
        }
        let standard = matched[0];
        if explicit.contains_key(standard) {
            continue;
        }
        auto.entry(standard.to_string())
            .or_default()
            .push(candidate);
    }

    for standard in standards {
        let Some(mut candidates) = auto.remove(standard) else {
            continue;
        };
        candidates.sort_by(|left, right| left.path.cmp(&right.path));
        candidates.dedup_by(|left, right| left.path == right.path);
        if candidates.len() > 1 {
            let paths = candidates
                .iter()
                .map(|candidate| candidate.display.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(WavepeekError::Args(format!(
                "ambiguous APB auto-mapping for '{standard}': {paths}. Add --map {standard}=<signal>. See 'wavepeek extract apb --help'."
            )));
        }
        let candidate = candidates.remove(0);
        explicit.insert(
            standard.to_string(),
            ApbSignalMapping {
                standard: standard.to_string(),
                display: candidate.display,
                path: candidate.path,
            },
        );
    }

    Ok((explicit, diagnostics))
}

fn validate_required_mappings(
    profile: ApbProfile,
    pready_mode: PreadyMode,
    include_wait: bool,
    mappings: &HashMap<String, ApbSignalMapping>,
) -> Result<(), WavepeekError> {
    validate_mode(pready_mode, include_wait)?;
    for standard in REQUIRED_SIGNALS {
        if !mappings.contains_key(*standard) {
            return Err(WavepeekError::Args(format!(
                "APB extraction requires {standard}. See 'wavepeek extract apb --help'."
            )));
        }
    }
    match pready_mode {
        PreadyMode::Mapped if !mappings.contains_key("pready") => {
            return Err(WavepeekError::Args(
                "APB mapped PREADY mode requires pready. See 'wavepeek extract apb --help'."
                    .to_string(),
            ));
        }
        PreadyMode::ImplicitHigh if mappings.contains_key("pready") => {
            return Err(WavepeekError::Args(
                "APB implicit-high PREADY mode forbids a pready mapping. See 'wavepeek extract apb --help'."
                    .to_string(),
            ));
        }
        _ => {}
    }
    debug_assert!(
        REQUIRED_SIGNALS
            .iter()
            .all(|standard| profile.contains_standard(standard))
    );
    Ok(())
}

#[derive(Debug)]
struct BuiltApbSource {
    event: &'static str,
    on: String,
    when: String,
    payload_waves: Vec<String>,
    payload_standards: Vec<String>,
}

fn build_extract_sources(
    profile: ApbProfile,
    pready_mode: PreadyMode,
    include_wait: bool,
    scope: Option<&str>,
    mappings: &HashMap<String, ApbSignalMapping>,
) -> Result<Vec<BuiltApbSource>, WavepeekError> {
    let pclk = mappings
        .get("pclk")
        .ok_or_else(|| WavepeekError::Internal("validated APB plan has no pclk".to_string()))?;
    let psel = mappings
        .get("psel")
        .ok_or_else(|| WavepeekError::Internal("validated APB plan has no psel".to_string()))?;
    let penable = mappings
        .get("penable")
        .ok_or_else(|| WavepeekError::Internal("validated APB plan has no penable".to_string()))?;
    let pready = mappings.get("pready");
    let reset = mappings.get("presetn");
    let on = format!("posedge {}", expr_name(pclk, scope));

    let mut event_predicates = vec![(
        "setup",
        format!(
            "{} && !{}",
            expr_name(psel, scope),
            expr_name(penable, scope)
        ),
    )];
    if pready_mode == PreadyMode::Mapped && include_wait {
        let pready = pready.expect("mapped mode was validated with pready");
        event_predicates.push((
            "access-wait",
            format!(
                "{} && {} && !{}",
                expr_name(psel, scope),
                expr_name(penable, scope),
                expr_name(pready, scope)
            ),
        ));
    }
    event_predicates.push((
        "access-complete",
        match pready_mode {
            PreadyMode::Mapped => format!(
                "{} && {} && {}",
                expr_name(psel, scope),
                expr_name(penable, scope),
                expr_name(
                    pready.expect("mapped mode was validated with pready"),
                    scope
                )
            ),
            PreadyMode::ImplicitHigh => format!(
                "{} && {}",
                expr_name(psel, scope),
                expr_name(penable, scope)
            ),
        },
    ));

    event_predicates
        .into_iter()
        .map(|(event, predicate)| {
            let when = match reset {
                Some(reset) => format!("{} && {predicate}", expr_name(reset, scope)),
                None => predicate,
            };
            let payload = ordered_standard_names(profile)
                .into_iter()
                .filter(|standard| {
                    !matches!(
                        *standard,
                        "pclk" | "presetn" | "psel" | "penable" | "pready"
                    ) && (event == "access-complete" || !COMPLETION_ONLY_SIGNALS.contains(standard))
                })
                .filter_map(|standard| mappings.get(standard).map(|mapping| (standard, mapping)))
                .collect::<Vec<_>>();
            if !payload.iter().any(|(standard, _)| *standard == "pwrite") {
                return Err(WavepeekError::Internal(format!(
                    "APB {event} source has no pwrite payload"
                )));
            }
            Ok(BuiltApbSource {
                event,
                on: on.clone(),
                when,
                payload_waves: payload
                    .iter()
                    .map(|(_, mapping)| expr_name(mapping, scope).to_string())
                    .collect(),
                payload_standards: payload
                    .iter()
                    .map(|(standard, _)| (*standard).to_string())
                    .collect(),
            })
        })
        .collect()
}

fn expr_name<'a>(mapping: &'a ApbSignalMapping, scope: Option<&str>) -> &'a str {
    if scope.is_some() {
        mapping.display.as_str()
    } else {
        mapping.path.as_str()
    }
}

fn ordered_standard_names(profile: ApbProfile) -> Vec<&'static str> {
    ORDERED_SIGNALS
        .iter()
        .copied()
        .filter(|standard| profile.contains_standard(standard))
        .collect()
}

fn candidate_matching_standards(
    candidate_name: &str,
    standards: &[&'static str],
) -> Vec<&'static str> {
    let tokens = candidate_core_tokens(candidate_name);
    let suffix_matches = standards
        .iter()
        .filter_map(|standard| {
            standard_suffix_start(tokens.as_slice(), standard).map(|start| (*standard, start))
        })
        .collect::<Vec<_>>();

    let [(suffix_standard, suffix_start)] = suffix_matches.as_slice() else {
        return suffix_matches
            .into_iter()
            .map(|(standard, _)| standard)
            .collect();
    };

    standards
        .iter()
        .filter(|standard| {
            *standard == suffix_standard
                || (0..*suffix_start).any(|start| {
                    standard_matches_range(tokens.as_slice(), start, *suffix_start, standard)
                })
        })
        .copied()
        .collect()
}

fn candidate_core_tokens(name: &str) -> Vec<String> {
    let mut tokens = tokenize_candidate(name);
    while tokens
        .last()
        .is_some_and(|token| is_candidate_suffix_affix(token))
    {
        tokens.pop();
    }
    tokens
}

fn tokenize_candidate(name: &str) -> Vec<String> {
    let base = name.rsplit('.').next().unwrap_or(name);
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in base.chars() {
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn is_candidate_suffix_affix(token: &str) -> bool {
    matches!(
        token,
        "i" | "o" | "in" | "out" | "input" | "output" | "d" | "q" | "r" | "reg"
    )
}

fn standard_suffix_start(tokens: &[String], standard: &str) -> Option<usize> {
    (0..tokens.len()).find(|start| standard_matches_range(tokens, *start, tokens.len(), standard))
}

fn standard_matches_range(tokens: &[String], start: usize, end: usize, standard: &str) -> bool {
    tokens[start..end]
        .iter()
        .flat_map(|token| token.chars())
        .eq(standard.chars())
}

impl ApbProfile {
    fn name(self) -> &'static str {
        self.spec.name
    }

    fn issue(self) -> &'static str {
        self.spec.issue
    }

    fn contains_standard(self, standard: &str) -> bool {
        self.spec.signals.contains(&standard)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApbDirection, candidate_matching_standards, direction_from_pwrite, event_payload_signals,
        parse_pready_mode, parse_profile, payload_allowed, profile_specs,
    };

    #[test]
    fn profiles_expose_issue_e_and_expected_signal_boundaries() {
        assert_eq!(profile_specs().len(), 3);
        for profile in profile_specs() {
            assert_eq!(profile.issue, "E");
            assert!(profile.signals.contains(&"pwrite"));
        }
        assert!(!profile_specs()[0].signals.contains(&"pprot"));
        assert!(profile_specs()[1].signals.contains(&"pprot"));
        assert!(profile_specs()[2].signals.contains(&"pnse"));
    }

    #[test]
    fn parsing_is_case_insensitive_and_accepts_cli_mode_alias() {
        assert_eq!(parse_profile("APB3").unwrap().name(), "apb3");
        assert_eq!(parse_profile("apb5").unwrap().name(), "apb5");
        assert_eq!(
            parse_pready_mode("IMPLICIT_HIGH").unwrap().as_str(),
            "implicit-high"
        );
        assert!(parse_profile("apb2").is_err());
        assert!(parse_pready_mode("default-high").is_err());
    }

    #[test]
    fn auto_mapping_uses_complete_normalized_suffixes() {
        let standards = &["paddr", "pready", "psel"];
        for name in ["paddr", "p_addr", "apb_paddr_i", "apb_p_addr_i"] {
            assert_eq!(candidate_matching_standards(name, standards), ["paddr"]);
        }
        assert!(candidate_matching_standards("preadychk", standards).is_empty());
        assert!(candidate_matching_standards("psel0", standards).is_empty());
        assert!(candidate_matching_standards("pselx", standards).is_empty());
    }

    #[test]
    fn direction_and_event_filtering_follow_sampled_pwrite() {
        assert_eq!(direction_from_pwrite("1'h0"), ApbDirection::Read);
        assert_eq!(direction_from_pwrite("1'h1"), ApbDirection::Write);
        assert_eq!(direction_from_pwrite("1'hx"), ApbDirection::Unknown);
        assert!(!payload_allowed("setup", ApbDirection::Read, "prdata"));
        assert!(!payload_allowed("setup", ApbDirection::Write, "pslverr"));
        assert!(!payload_allowed(
            "access-complete",
            ApbDirection::Read,
            "pwdata"
        ));
        assert!(!payload_allowed(
            "access-complete",
            ApbDirection::Write,
            "prdata"
        ));
        assert!(payload_allowed(
            "access-complete",
            ApbDirection::Unknown,
            "prdata"
        ));
    }

    #[test]
    fn schema_payload_helper_applies_profile_event_and_direction() {
        let apb5 = &profile_specs()[2];
        let setup_read = event_payload_signals(apb5, "setup", "read").collect::<Vec<_>>();
        assert!(setup_read.contains(&"pwrite"));
        assert!(!setup_read.contains(&"pwdata"));
        assert!(!setup_read.contains(&"pslverr"));
        let complete_unknown =
            event_payload_signals(apb5, "access-complete", "unknown").collect::<Vec<_>>();
        assert!(complete_unknown.contains(&"pwdata"));
        assert!(complete_unknown.contains(&"prdata"));
        assert!(complete_unknown.contains(&"pslverr"));
    }
}
