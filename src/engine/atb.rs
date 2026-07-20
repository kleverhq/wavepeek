use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;

use regex::Regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::cli::extract::AtbArgs;
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

const DEFAULT_PROFILE: &str = "atb-c";
const DEFAULT_NAME: &str = "atb";
const SOURCE_KIND: &str = "extract.atb.source";
const HELP: &str = "wavepeek extract atb";
const ISSUE: &str = "C";
const COMMON_SIGNALS: &[&str] = &[
    "atclk", "atresetn", "atvalid", "atready", "atbytes", "atdata", "atid", "afvalid", "afready",
];
const ATB_A_SIGNALS: &[&str] = COMMON_SIGNALS;
const ATB_B_SIGNALS: &[&str] = &[
    "atclk", "atresetn", "atvalid", "atready", "atbytes", "atdata", "atid", "afvalid", "afready",
    "syncreq",
];
const ATB_C_SIGNALS: &[&str] = ATB_B_SIGNALS;
const TRANSFER_PAYLOAD_SIGNALS: &[&str] = &["atbytes", "atdata", "atid"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtbSignalMapping {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtbEventPayload {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AtbEventKind {
    Transfer,
    Flush,
    SyncRequest,
}

impl AtbEventKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Transfer => "transfer",
            Self::Flush => "flush",
            Self::SyncRequest => "sync-request",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtbEvent {
    pub time: String,
    pub sample_time: String,
    pub profile: String,
    pub event: AtbEventKind,
    pub payload: Vec<AtbEventPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtbContext {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub mappings: Vec<AtbSignalMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AtbData {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub mappings: Vec<AtbSignalMapping>,
    pub events: Vec<AtbEvent>,
}

impl AtbData {
    pub(crate) fn context(&self) -> AtbContext {
        AtbContext {
            name: self.name.clone(),
            profile: self.profile.clone(),
            issue: self.issue.clone(),
            mappings: self.mappings.clone(),
        }
    }
}

#[derive(Debug)]
struct AtbOutcome {
    context: AtbContext,
    diagnostics: Vec<Diagnostic>,
    stats: ExtractRunStats,
}

trait AtbEventSink {
    fn start(&mut self, _context: &AtbContext) -> Result<(), WavepeekError> {
        Ok(())
    }

    fn emit(&mut self, event: AtbEvent) -> Result<(), WavepeekError>;
}

#[derive(Default)]
struct CollectingAtbSink {
    events: Vec<AtbEvent>,
}

impl AtbEventSink for CollectingAtbSink {
    fn emit(&mut self, event: AtbEvent) -> Result<(), WavepeekError> {
        self.events.push(event);
        Ok(())
    }
}

struct JsonlAtbSink<'a, W: std::io::Write> {
    writer: &'a mut crate::output::JsonlWriter<W>,
}

impl<W: std::io::Write> AtbEventSink for JsonlAtbSink<'_, W> {
    fn start(&mut self, context: &AtbContext) -> Result<(), WavepeekError> {
        self.writer.begin_context(context)
    }

    fn emit(&mut self, event: AtbEvent) -> Result<(), WavepeekError> {
        self.writer.item(&event)
    }
}

struct GenericToAtbSink<'a, S: AtbEventSink + ?Sized> {
    context: &'a AtbContext,
    payload_standards: &'a HashMap<String, Vec<String>>,
    sink: &'a mut S,
}

impl<S: AtbEventSink + ?Sized> ExtractRowSink for GenericToAtbSink<'_, S> {
    fn start(&mut self) -> Result<(), WavepeekError> {
        self.sink.start(self.context)
    }

    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError> {
        let event = event_kind(row.source.as_str())?;
        let standards = self
            .payload_standards
            .get(row.source.as_str())
            .ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "missing ATB payload metadata for event '{}'",
                    row.source
                ))
            })?;
        if standards.len() != row.payload.len() {
            return Err(WavepeekError::Internal(format!(
                "ATB payload metadata length mismatch for event '{}'",
                row.source
            )));
        }
        let payload = row
            .payload
            .into_iter()
            .zip(standards.iter())
            .map(|(payload, standard)| AtbEventPayload {
                standard: standard.clone(),
                display: payload.display,
                path: payload.path,
                value: payload.value,
            })
            .collect();
        self.sink.emit(AtbEvent {
            time: row.time,
            sample_time: row.sample_time,
            profile: self.context.profile.clone(),
            event,
            payload,
        })
    }
}

fn event_kind(source: &str) -> Result<AtbEventKind, WavepeekError> {
    match source {
        "transfer" => Ok(AtbEventKind::Transfer),
        "flush" => Ok(AtbEventKind::Flush),
        "sync-request" => Ok(AtbEventKind::SyncRequest),
        _ => Err(WavepeekError::Internal(format!(
            "unknown ATB extraction source '{source}'"
        ))),
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
    name: Option<String>,
    #[serde(default)]
    includes: Vec<String>,
    #[serde(default)]
    maps: BTreeMap<String, String>,
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

#[derive(Debug)]
struct AtbConfig {
    profile: AtbProfile,
    name: String,
    includes: Vec<String>,
    maps: Vec<(String, String)>,
}

#[derive(Debug)]
struct BuiltAtbPlan {
    context: AtbContext,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AtbProfile {
    AtbA,
    AtbB,
    AtbC,
}

impl AtbProfile {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::AtbA => "atb-a",
            Self::AtbB => "atb-b",
            Self::AtbC => "atb-c",
        }
    }

    pub(crate) const fn signals(self) -> &'static [&'static str] {
        match self {
            Self::AtbA => ATB_A_SIGNALS,
            Self::AtbB => ATB_B_SIGNALS,
            Self::AtbC => ATB_C_SIGNALS,
        }
    }

    fn contains_standard(self, standard: &str) -> bool {
        self.signals().contains(&standard)
    }
}

pub(crate) const fn profile_specs() -> [AtbProfile; 3] {
    [AtbProfile::AtbA, AtbProfile::AtbB, AtbProfile::AtbC]
}

pub fn run(args: AtbArgs) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let signals_abs = args.abs;
    let mut sink = CollectingAtbSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::ExtractAtb,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        data: CommandData::ExtractAtb(AtbData {
            name: outcome.context.name,
            profile: outcome.context.profile,
            issue: outcome.context.issue,
            mappings: outcome.context.mappings,
            events: sink.events,
        }),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: AtbArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    let outcome = {
        let mut sink = JsonlAtbSink { writer };
        run_with_sink(args, &mut sink)?
    };

    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end(outcome.stats.truncated)
}

fn run_with_sink<S: AtbEventSink + ?Sized>(
    args: AtbArgs,
    sink: &mut S,
) -> Result<AtbOutcome, WavepeekError> {
    let BuiltAtbPlan {
        context,
        plan,
        waveform,
        debug,
        payload_standards,
        diagnostics: build_diagnostics,
    } = build_atb_plan(&args)?;
    let mut generic_sink = GenericToAtbSink {
        context: &context,
        payload_standards: &payload_standards,
        sink,
    };
    let outcome = extract::run_plan_with_waveform_sink(
        ExtractRunArgs {
            command: CommandName::ExtractAtb,
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
    Ok(AtbOutcome {
        context,
        diagnostics,
        stats: outcome.stats,
    })
}

fn build_atb_plan(args: &AtbArgs) -> Result<BuiltAtbPlan, WavepeekError> {
    let config = config_from_args(args)?;
    let include_regexes = compile_include_regexes(&config.includes)?;
    let debug = DebugTrace::for_command(CommandName::ExtractAtb);
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
    let sources =
        build_extract_sources(config.profile, args.scope.as_deref(), &mappings_by_standard)?;

    let ordered_mappings = config
        .profile
        .signals()
        .iter()
        .filter_map(|standard| mappings_by_standard.get(*standard).cloned())
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

    Ok(BuiltAtbPlan {
        context: AtbContext {
            name: config.name,
            profile: config.profile.name().to_string(),
            issue: ISSUE.to_string(),
            mappings: ordered_mappings,
        },
        plan: ExtractPlan::new(extract_sources),
        waveform,
        debug,
        payload_standards,
        diagnostics,
    })
}

fn config_from_args(args: &AtbArgs) -> Result<AtbConfig, WavepeekError> {
    if let Some(path) = args.source.as_ref() {
        if args.name.is_some() || !args.maps.is_empty() || !args.includes.is_empty() {
            return Err(WavepeekError::Args(
                "--source cannot be combined with --profile, --name, --map, or --include. See 'wavepeek extract atb --help'.".to_string(),
            ));
        }
        return config_from_source(path);
    }

    Ok(AtbConfig {
        profile: parse_profile(args.profile.as_str())?,
        name: args
            .name
            .clone()
            .unwrap_or_else(|| DEFAULT_NAME.to_string()),
        includes: args.includes.clone(),
        maps: parse_cli_maps(&args.maps)?,
    })
}

fn config_from_source(path: &std::path::Path) -> Result<AtbConfig, WavepeekError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        WavepeekError::File(format!(
            "failed to read ATB extract source file '{}': {error}",
            path.display()
        ))
    })?;
    let input: SourceFile = serde_json::from_str(&contents).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid ATB extract source JSON '{}': {error}",
            path.display()
        ))
    })?;

    if input.schema != INPUT_SCHEMA_URL {
        return Err(WavepeekError::Args(format!(
            "ATB extract source file '{}' uses unsupported $schema {}; expected {}",
            path.display(),
            input.schema,
            INPUT_SCHEMA_URL
        )));
    }
    if input.kind != SOURCE_KIND {
        return Err(WavepeekError::Args(format!(
            "ATB extract source file '{}' has kind {}; expected {}",
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
    Ok(AtbConfig {
        profile: parse_profile(input.profile.as_deref().unwrap_or(DEFAULT_PROFILE))?,
        name: input.name.unwrap_or_else(|| DEFAULT_NAME.to_string()),
        includes: input.includes,
        maps,
    })
}

fn parse_cli_maps(values: &[String]) -> Result<Vec<(String, String)>, WavepeekError> {
    let maps = values
        .iter()
        .map(|value| {
            let (standard, waves) = value.split_once('=').ok_or_else(|| {
                WavepeekError::Args(format!(
                    "invalid --map '{}': expected STD_NAME=WAVES_NAME. See 'wavepeek extract atb --help'.",
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
            "ATB mapped waveform signal names must not be empty. See 'wavepeek extract atb --help'."
                .to_string(),
        ));
    }
    Ok((standard, waves.to_string()))
}

fn normalize_standard_name(standard: &str) -> Result<String, WavepeekError> {
    let standard = standard.trim().to_ascii_lowercase();
    if standard.is_empty() {
        Err(WavepeekError::Args(
            "ATB standard signal names must not be empty. See 'wavepeek extract atb --help'."
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
                "duplicate ATB mapping for standard signal '{standard}'. See 'wavepeek extract atb --help'."
            )));
        }
    }
    Ok(())
}

fn parse_profile(profile: &str) -> Result<AtbProfile, WavepeekError> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "atb-a" | "atb_a" | "atbv1.0" => Ok(AtbProfile::AtbA),
        "atb-b" | "atb_b" | "atbv1.1" => Ok(AtbProfile::AtbB),
        "atb-c" | "atb_c" => Ok(AtbProfile::AtbC),
        _ => Err(WavepeekError::Args(format!(
            "unsupported ATB profile '{profile}'; expected atb-a, atb-b, or atb-c. See 'wavepeek extract atb --help'."
        ))),
    }
}

fn compile_include_regexes(includes: &[String]) -> Result<Vec<Regex>, WavepeekError> {
    includes
        .iter()
        .map(|include| {
            Regex::new(include).map_err(|error| {
                WavepeekError::Args(format!(
                    "invalid ATB include regex '{}': {error}. See 'wavepeek extract atb --help'.",
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
    profile: AtbProfile,
    maps: &[(String, String)],
) -> Result<HashMap<String, AtbSignalMapping>, WavepeekError> {
    let mut result = HashMap::new();
    for (standard, waves) in maps {
        if !profile.contains_standard(standard) {
            return Err(WavepeekError::Args(format!(
                "ATB profile {} has no standard signal '{}'. See 'wavepeek extract atb --help'.",
                profile.name(),
                standard
            )));
        }
        if scope.is_some() && waves.contains('.') {
            return Err(WavepeekError::Args(format!(
                "ATB mapping for '{standard}' must use a scope-relative signal when --scope is set. See 'wavepeek extract atb --help'."
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
            AtbSignalMapping {
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
    profile: AtbProfile,
    candidates: Vec<SignalCandidate>,
    mut explicit: HashMap<String, AtbSignalMapping>,
) -> Result<(HashMap<String, AtbSignalMapping>, Vec<Diagnostic>), WavepeekError> {
    let mut diagnostics = Vec::new();
    let mut auto: HashMap<String, Vec<SignalCandidate>> = HashMap::new();
    let standards = profile.signals();
    let explicit_paths = explicit
        .values()
        .map(|mapping| mapping.path.as_str())
        .collect::<HashSet<_>>();

    for candidate in candidates {
        if explicit_paths.contains(candidate.path.as_str()) {
            continue;
        }
        let matched = candidate_matching_standards(candidate.name.as_str(), standards);
        if matched.len() > 1 {
            let standards = matched.join(", ");
            return Err(WavepeekError::Args(format!(
                "ambiguous ATB auto-mapping for '{}': matched {standards}. Add explicit --map entries. See 'wavepeek extract atb --help'.",
                candidate.display
            )));
        }
        if matched.is_empty() {
            diagnostics.push(Diagnostic::warning(
                WarningDiagnosticCode::UnmatchedExtractCandidate,
                format!(
                    "ignored ATB include candidate '{}' because it did not match any {} standard signal",
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
        let Some(mut candidates) = auto.remove(*standard) else {
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
                "ambiguous ATB auto-mapping for '{standard}': {paths}. Add --map {standard}=<signal>. See 'wavepeek extract atb --help'."
            )));
        }
        let candidate = candidates.remove(0);
        explicit.insert(
            (*standard).to_string(),
            AtbSignalMapping {
                standard: (*standard).to_string(),
                display: candidate.display,
                path: candidate.path,
            },
        );
    }

    Ok((explicit, diagnostics))
}

#[derive(Debug)]
struct BuiltAtbSource {
    event: &'static str,
    on: String,
    when: String,
    payload_waves: Vec<String>,
    payload_standards: Vec<String>,
}

fn build_extract_sources(
    profile: AtbProfile,
    scope: Option<&str>,
    mappings: &HashMap<String, AtbSignalMapping>,
) -> Result<Vec<BuiltAtbSource>, WavepeekError> {
    let atclk = mappings.get("atclk").ok_or_else(|| {
        WavepeekError::Args(
            "ATB mapping requires atclk. See 'wavepeek extract atb --help'.".to_string(),
        )
    })?;
    let reset = mappings.get("atresetn");
    let on = format!("posedge {}", expr_name(atclk, scope));
    let mut sources = Vec::new();

    let transfer_complete = require_complete_pair(mappings, "transfer", "atvalid", "atready")?;
    let mapped_transfer_payload = TRANSFER_PAYLOAD_SIGNALS
        .iter()
        .filter_map(|standard| mappings.get(*standard).map(|mapping| (*standard, mapping)))
        .collect::<Vec<_>>();
    if !mapped_transfer_payload.is_empty() && !transfer_complete {
        return Err(WavepeekError::Args(
            "ATB transfer payload mappings require both atvalid and atready. See 'wavepeek extract atb --help'."
                .to_string(),
        ));
    }
    if transfer_complete {
        let valid = mappings.get("atvalid").expect("complete transfer pair");
        let ready = mappings.get("atready").expect("complete transfer pair");
        sources.push(BuiltAtbSource {
            event: "transfer",
            on: on.clone(),
            when: gated_predicate(reset, scope, &[valid, ready]),
            payload_waves: mapped_transfer_payload
                .iter()
                .map(|(_, mapping)| expr_name(mapping, scope).to_string())
                .collect(),
            payload_standards: mapped_transfer_payload
                .iter()
                .map(|(standard, _)| (*standard).to_string())
                .collect(),
        });
    }

    let flush_complete = require_complete_pair(mappings, "flush", "afvalid", "afready")?;
    if flush_complete {
        let valid = mappings.get("afvalid").expect("complete flush pair");
        let ready = mappings.get("afready").expect("complete flush pair");
        sources.push(BuiltAtbSource {
            event: "flush",
            on: on.clone(),
            when: gated_predicate(reset, scope, &[valid, ready]),
            payload_waves: Vec::new(),
            payload_standards: Vec::new(),
        });
    }

    if !transfer_complete && !flush_complete {
        return Err(WavepeekError::Args(
            "ATB extraction requires at least one complete transfer or flush handshake channel. See 'wavepeek extract atb --help'."
                .to_string(),
        ));
    }

    if let Some(syncreq) = mappings.get("syncreq") {
        debug_assert!(matches!(profile, AtbProfile::AtbB | AtbProfile::AtbC));
        sources.push(BuiltAtbSource {
            event: "sync-request",
            on,
            when: gated_predicate(reset, scope, &[syncreq]),
            payload_waves: Vec::new(),
            payload_standards: Vec::new(),
        });
    }

    Ok(sources)
}

fn require_complete_pair(
    mappings: &HashMap<String, AtbSignalMapping>,
    channel: &str,
    valid: &str,
    ready: &str,
) -> Result<bool, WavepeekError> {
    let valid_mapped = mappings.contains_key(valid);
    let ready_mapped = mappings.contains_key(ready);
    if valid_mapped ^ ready_mapped {
        return Err(WavepeekError::Args(format!(
            "ATB {channel} channel must map both {valid} and {ready}; no implicit ready is used. See 'wavepeek extract atb --help'."
        )));
    }
    Ok(valid_mapped)
}

fn gated_predicate(
    reset: Option<&AtbSignalMapping>,
    scope: Option<&str>,
    signals: &[&AtbSignalMapping],
) -> String {
    reset
        .into_iter()
        .chain(signals.iter().copied())
        .map(|mapping| expr_name(mapping, scope))
        .collect::<Vec<_>>()
        .join(" && ")
}

fn expr_name<'a>(mapping: &'a AtbSignalMapping, scope: Option<&str>) -> &'a str {
    if scope.is_some() {
        mapping.display.as_str()
    } else {
        mapping.path.as_str()
    }
}

fn candidate_matching_standards<'a>(
    candidate_name: &str,
    standards: &'a [&'static str],
) -> Vec<&'a str> {
    let tokens = candidate_core_tokens(candidate_name);
    standards
        .iter()
        .filter(|standard| standard_suffix_start(tokens.as_slice(), standard).is_some())
        .copied()
        .collect()
}

#[cfg(test)]
fn candidate_matches_standard(candidate_name: &str, standard: &str) -> bool {
    let tokens = candidate_core_tokens(candidate_name);
    standard_suffix_start(tokens.as_slice(), standard).is_some()
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
    (0..tokens.len()).find(|start| {
        tokens[*start..]
            .iter()
            .flat_map(|token| token.chars())
            .eq(standard.chars())
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AtbProfile, candidate_matches_standard, event_kind, parse_cli_maps, parse_profile,
        profile_specs,
    };

    #[test]
    fn profile_names_and_aliases_are_case_insensitive() {
        for alias in ["atb-a", "ATB_A", "ATBV1.0"] {
            assert_eq!(parse_profile(alias).unwrap(), AtbProfile::AtbA);
        }
        for alias in ["atb-b", "ATB_B", "ATBV1.1"] {
            assert_eq!(parse_profile(alias).unwrap(), AtbProfile::AtbB);
        }
        for alias in ["atb-c", "ATB_C"] {
            assert_eq!(parse_profile(alias).unwrap(), AtbProfile::AtbC);
        }
        assert!(parse_profile("atbv1.2").is_err());
    }

    #[test]
    fn profile_signal_sets_match_issue_c_contract() {
        let profiles = profile_specs();
        assert_eq!(profiles.map(AtbProfile::name), ["atb-a", "atb-b", "atb-c"]);
        assert_eq!(
            AtbProfile::AtbA.signals(),
            [
                "atclk", "atresetn", "atvalid", "atready", "atbytes", "atdata", "atid", "afvalid",
                "afready"
            ]
        );
        assert_eq!(
            AtbProfile::AtbB.signals(),
            [
                "atclk", "atresetn", "atvalid", "atready", "atbytes", "atdata", "atid", "afvalid",
                "afready", "syncreq"
            ]
        );
        assert_eq!(AtbProfile::AtbC.signals(), AtbProfile::AtbB.signals());
    }

    #[test]
    fn auto_mapping_matches_complete_normalized_suffixes_only() {
        for candidate in [
            "atvalid",
            "ATVALID",
            "at_valid",
            "trace_atvalid_o",
            "trace_at_valid_o",
        ] {
            assert!(candidate_matches_standard(candidate, "atvalid"));
        }
        for candidate in ["atvalidchk", "at_ready_check", "atvalid_extra"] {
            assert!(!candidate_matches_standard(candidate, "atvalid"));
        }
    }

    #[test]
    fn cli_maps_normalize_and_reject_duplicates() {
        let maps = parse_cli_maps(&[" ATVALID = top.atvalid ".to_string()]).unwrap();
        assert_eq!(
            maps,
            vec![("atvalid".to_string(), "top.atvalid".to_string())]
        );
        assert!(
            parse_cli_maps(&["ATVALID=top.a".to_string(), "atvalid=top.b".to_string()]).is_err()
        );
    }

    #[test]
    fn event_sources_map_to_public_kinds() {
        assert_eq!(event_kind("transfer").unwrap().as_str(), "transfer");
        assert_eq!(event_kind("flush").unwrap().as_str(), "flush");
        assert_eq!(event_kind("sync-request").unwrap().as_str(), "sync-request");
        assert!(event_kind("wake-up").is_err());
    }
}
