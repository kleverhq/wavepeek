use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;

use regex::Regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::cli::extract::AxiStreamArgs;
use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{SharedWaveform, open_shared_waveform};
use crate::engine::extract::{
    self, ExtractGenericRow, ExtractPlan, ExtractRowSink, ExtractRunArgs, ExtractRunStats,
    ExtractSource,
};
use crate::engine::signal_mapping::candidate_matching_standards;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;

const DEFAULT_PROFILE: &str = "axi4-stream";
const DEFAULT_TREADY_MODE: &str = "mapped";
const DEFAULT_NAME: &str = "axistream";
const SOURCE_KIND: &str = "extract.axistream.source";
const HELP: &str = "wavepeek extract axistream";
const CONTEXT_SIGNALS: &[&str] = &["aclk", "aresetn"];
const PAYLOAD_SIGNALS: &[&str] = &["tdata", "tstrb", "tkeep", "tlast", "tid", "tdest", "tuser"];
const STANDARD_SIGNALS: &[&str] = &[
    "aclk", "aresetn", "tvalid", "tready", "tdata", "tstrb", "tkeep", "tlast", "tid", "tdest",
    "tuser",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiStreamSignalMapping {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiStreamTransferPayload {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiStreamTransfer {
    pub time: String,
    pub sample_time: String,
    pub profile: String,
    pub payload: Vec<AxiStreamTransferPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiStreamContext {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub tready_mode: String,
    pub mappings: Vec<AxiStreamSignalMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiStreamData {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub tready_mode: String,
    pub mappings: Vec<AxiStreamSignalMapping>,
    pub transfers: Vec<AxiStreamTransfer>,
}

impl AxiStreamData {
    pub(crate) fn context(&self) -> AxiStreamContext {
        AxiStreamContext {
            name: self.name.clone(),
            profile: self.profile.clone(),
            issue: self.issue.clone(),
            tready_mode: self.tready_mode.clone(),
            mappings: self.mappings.clone(),
        }
    }
}

#[derive(Debug)]
struct AxiStreamOutcome {
    context: AxiStreamContext,
    diagnostics: Vec<Diagnostic>,
    stats: ExtractRunStats,
}

trait AxiStreamTransferSink {
    fn start(&mut self, _context: &AxiStreamContext) -> Result<(), WavepeekError> {
        Ok(())
    }

    fn emit(&mut self, transfer: AxiStreamTransfer) -> Result<(), WavepeekError>;
}

#[derive(Default)]
struct CollectingAxiStreamSink {
    transfers: Vec<AxiStreamTransfer>,
}

impl AxiStreamTransferSink for CollectingAxiStreamSink {
    fn emit(&mut self, transfer: AxiStreamTransfer) -> Result<(), WavepeekError> {
        self.transfers.push(transfer);
        Ok(())
    }
}

struct JsonlAxiStreamSink<'a, W: std::io::Write> {
    writer: &'a mut crate::output::JsonlWriter<W>,
}

impl<W: std::io::Write> AxiStreamTransferSink for JsonlAxiStreamSink<'_, W> {
    fn start(&mut self, context: &AxiStreamContext) -> Result<(), WavepeekError> {
        self.writer.begin_context(context)
    }

    fn emit(&mut self, transfer: AxiStreamTransfer) -> Result<(), WavepeekError> {
        self.writer.item(&transfer)
    }
}

struct GenericToAxiStreamSink<'a, S: AxiStreamTransferSink + ?Sized> {
    context: &'a AxiStreamContext,
    payload_standards: &'a [String],
    sink: &'a mut S,
}

impl<S: AxiStreamTransferSink + ?Sized> ExtractRowSink for GenericToAxiStreamSink<'_, S> {
    fn start(&mut self) -> Result<(), WavepeekError> {
        self.sink.start(self.context)
    }

    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError> {
        if self.payload_standards.len() != row.payload.len() {
            return Err(WavepeekError::Internal(
                "AXI-Stream payload metadata length mismatch".to_string(),
            ));
        }
        let payload = row
            .payload
            .into_iter()
            .zip(self.payload_standards.iter())
            .map(|(payload, standard)| AxiStreamTransferPayload {
                standard: standard.clone(),
                display: payload.display,
                path: payload.path,
                value: payload.value,
            })
            .collect();
        self.sink.emit(AxiStreamTransfer {
            time: row.time,
            sample_time: row.sample_time,
            profile: self.context.profile.clone(),
            payload,
        })
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
    tready_mode: Option<String>,
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
struct AxiStreamConfig {
    profile: AxiStreamProfile,
    tready_mode: TreadyMode,
    name: String,
    includes: Vec<String>,
    maps: Vec<(String, String)>,
}

#[derive(Debug)]
struct BuiltAxiStreamPlan {
    context: AxiStreamContext,
    plan: ExtractPlan,
    waveform: SharedWaveform,
    debug: DebugTrace,
    payload_standards: Vec<String>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct SignalCandidate {
    display: String,
    name: String,
    path: String,
}

#[derive(Debug, Clone, Copy)]
struct AxiStreamProfile {
    spec: &'static AxiStreamProfileSpec,
}

#[derive(Debug)]
pub(crate) struct AxiStreamProfileSpec {
    pub(crate) name: &'static str,
    pub(crate) issue: &'static str,
}

const AXI4_STREAM_PROFILE: AxiStreamProfileSpec = AxiStreamProfileSpec {
    name: "axi4-stream",
    issue: "B",
};
const AXI5_STREAM_PROFILE: AxiStreamProfileSpec = AxiStreamProfileSpec {
    name: "axi5-stream",
    issue: "B",
};

pub(crate) fn profile_specs() -> &'static [AxiStreamProfileSpec] {
    &[AXI4_STREAM_PROFILE, AXI5_STREAM_PROFILE]
}

pub(crate) const fn standard_signals() -> &'static [&'static str] {
    STANDARD_SIGNALS
}

pub(crate) const fn payload_signals() -> &'static [&'static str] {
    PAYLOAD_SIGNALS
}

pub fn run(args: AxiStreamArgs) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let signals_abs = args.abs;
    let mut sink = CollectingAxiStreamSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::ExtractAxiStream,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        data: CommandData::ExtractAxiStream(AxiStreamData {
            name: outcome.context.name,
            profile: outcome.context.profile,
            issue: outcome.context.issue,
            tready_mode: outcome.context.tready_mode,
            mappings: outcome.context.mappings,
            transfers: sink.transfers,
        }),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: AxiStreamArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    let outcome = {
        let mut sink = JsonlAxiStreamSink { writer };
        run_with_sink(args, &mut sink)?
    };

    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end(outcome.stats.truncated)
}

fn run_with_sink<S: AxiStreamTransferSink + ?Sized>(
    args: AxiStreamArgs,
    sink: &mut S,
) -> Result<AxiStreamOutcome, WavepeekError> {
    let BuiltAxiStreamPlan {
        context,
        plan,
        waveform,
        debug,
        payload_standards,
        diagnostics: build_diagnostics,
    } = build_axistream_plan(&args)?;
    let mut generic_sink = GenericToAxiStreamSink {
        context: &context,
        payload_standards: &payload_standards,
        sink,
    };
    let outcome = extract::run_plan_with_waveform_sink(
        ExtractRunArgs {
            command: CommandName::ExtractAxiStream,
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
    Ok(AxiStreamOutcome {
        context,
        diagnostics,
        stats: outcome.stats,
    })
}

fn build_axistream_plan(args: &AxiStreamArgs) -> Result<BuiltAxiStreamPlan, WavepeekError> {
    let config = config_from_args(args)?;
    let include_regexes = compile_include_regexes(&config.includes)?;
    let debug = DebugTrace::for_command(CommandName::ExtractAxiStream);
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
    let explicit = explicit_mappings(&waveform, args.scope.as_deref(), &config.maps)?;
    let (mappings_by_standard, diagnostics) = auto_mappings(config.profile, candidates, explicit)?;
    let source = build_extract_source(
        config.tready_mode,
        args.scope.as_deref(),
        &mappings_by_standard,
    )?;

    let ordered_mappings = STANDARD_SIGNALS
        .iter()
        .filter_map(|standard| mappings_by_standard.get(*standard).cloned())
        .collect();
    let payload_standards = source.payload_standards;
    let extract_source = ExtractSource::new(
        0,
        DEFAULT_NAME,
        source.on,
        source.when,
        source.payload_waves,
    );

    Ok(BuiltAxiStreamPlan {
        context: AxiStreamContext {
            name: config.name,
            profile: config.profile.name().to_string(),
            issue: config.profile.issue().to_string(),
            tready_mode: config.tready_mode.name().to_string(),
            mappings: ordered_mappings,
        },
        plan: ExtractPlan::new(vec![extract_source]),
        waveform,
        debug,
        payload_standards,
        diagnostics,
    })
}

fn config_from_args(args: &AxiStreamArgs) -> Result<AxiStreamConfig, WavepeekError> {
    if let Some(path) = args.source.as_ref() {
        if args.name.is_some() || !args.maps.is_empty() || !args.includes.is_empty() {
            return Err(WavepeekError::Args(
                "--source cannot be combined with --profile, --tready-mode, --name, --map, or --include. See 'wavepeek extract axistream --help'.".to_string(),
            ));
        }
        return config_from_source(path);
    }

    Ok(AxiStreamConfig {
        profile: parse_profile(args.profile.as_str())?,
        tready_mode: parse_tready_mode(args.tready_mode.as_str())?,
        name: args
            .name
            .clone()
            .unwrap_or_else(|| DEFAULT_NAME.to_string()),
        includes: args.includes.clone(),
        maps: parse_cli_maps(&args.maps)?,
    })
}

fn config_from_source(path: &std::path::Path) -> Result<AxiStreamConfig, WavepeekError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        WavepeekError::File(format!(
            "failed to read AXI-Stream extract source file '{}': {error}",
            path.display()
        ))
    })?;
    let input: SourceFile = serde_json::from_str(&contents).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid AXI-Stream extract source JSON '{}': {error}",
            path.display()
        ))
    })?;

    if input.schema != INPUT_SCHEMA_URL {
        return Err(WavepeekError::Args(format!(
            "AXI-Stream extract source file '{}' uses unsupported $schema {}; expected {}",
            path.display(),
            input.schema,
            INPUT_SCHEMA_URL
        )));
    }
    if input.kind != SOURCE_KIND {
        return Err(WavepeekError::Args(format!(
            "AXI-Stream extract source file '{}' has kind {}; expected {}",
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
    Ok(AxiStreamConfig {
        profile: parse_profile(input.profile.as_deref().unwrap_or(DEFAULT_PROFILE))?,
        tready_mode: parse_tready_mode(
            input.tready_mode.as_deref().unwrap_or(DEFAULT_TREADY_MODE),
        )?,
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
                    "invalid --map '{}': expected STD_NAME=WAVES_NAME. See 'wavepeek extract axistream --help'.",
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
            "AXI-Stream mapped waveform signal names must not be empty. See 'wavepeek extract axistream --help'."
                .to_string(),
        ));
    }
    Ok((standard, waves.to_string()))
}

fn normalize_standard_name(standard: &str) -> Result<String, WavepeekError> {
    let standard = standard.trim().to_ascii_lowercase();
    if standard.is_empty() {
        Err(WavepeekError::Args(
            "AXI-Stream standard signal names must not be empty. See 'wavepeek extract axistream --help'."
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
                "duplicate AXI-Stream mapping for standard signal '{standard}'. See 'wavepeek extract axistream --help'."
            )));
        }
    }
    Ok(())
}

fn parse_profile(profile: &str) -> Result<AxiStreamProfile, WavepeekError> {
    let normalized = profile.trim().to_ascii_lowercase();
    let spec = match normalized.as_str() {
        "axi4-stream" | "axi4_stream" => &AXI4_STREAM_PROFILE,
        "axi5-stream" | "axi5_stream" => &AXI5_STREAM_PROFILE,
        _ => {
            return Err(WavepeekError::Args(format!(
                "unsupported AXI-Stream profile '{profile}'; expected axi4-stream or axi5-stream. See 'wavepeek extract axistream --help'."
            )));
        }
    };
    Ok(AxiStreamProfile { spec })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreadyMode {
    Mapped,
    ImplicitHigh,
}

fn parse_tready_mode(mode: &str) -> Result<TreadyMode, WavepeekError> {
    match mode.trim().to_ascii_lowercase().as_str() {
        "mapped" => Ok(TreadyMode::Mapped),
        "implicit-high" | "implicit_high" => Ok(TreadyMode::ImplicitHigh),
        _ => Err(WavepeekError::Args(format!(
            "unsupported AXI-Stream TREADY mode '{mode}'; expected mapped or implicit-high. See 'wavepeek extract axistream --help'."
        ))),
    }
}

fn compile_include_regexes(includes: &[String]) -> Result<Vec<Regex>, WavepeekError> {
    includes
        .iter()
        .map(|include| {
            Regex::new(include).map_err(|error| {
                WavepeekError::Args(format!(
                    "invalid AXI-Stream include regex '{}': {error}. See 'wavepeek extract axistream --help'.",
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
    maps: &[(String, String)],
) -> Result<HashMap<String, AxiStreamSignalMapping>, WavepeekError> {
    let mut result = HashMap::new();
    for (standard, waves) in maps {
        if !STANDARD_SIGNALS.contains(&standard.as_str()) {
            return Err(WavepeekError::Args(format!(
                "AXI-Stream profile has no standard signal '{}'. See 'wavepeek extract axistream --help'.",
                standard
            )));
        }
        if scope.is_some() && waves.contains('.') {
            return Err(WavepeekError::Args(format!(
                "AXI-Stream mapping for '{standard}' must use a scope-relative signal when --scope is set. See 'wavepeek extract axistream --help'."
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
            AxiStreamSignalMapping {
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
    profile: AxiStreamProfile,
    candidates: Vec<SignalCandidate>,
    mut explicit: HashMap<String, AxiStreamSignalMapping>,
) -> Result<(HashMap<String, AxiStreamSignalMapping>, Vec<Diagnostic>), WavepeekError> {
    let mut diagnostics = Vec::new();
    let mut auto: HashMap<String, Vec<SignalCandidate>> = HashMap::new();
    let explicit_paths = explicit
        .values()
        .map(|mapping| mapping.path.as_str())
        .collect::<HashSet<_>>();

    for candidate in candidates {
        if explicit_paths.contains(candidate.path.as_str()) {
            continue;
        }
        let matched = candidate_matching_standards(
            candidate.name.as_str(),
            STANDARD_SIGNALS,
            CONTEXT_SIGNALS,
        );
        if matched.len() > 1 {
            let standards = matched.join(", ");
            return Err(WavepeekError::Args(format!(
                "ambiguous AXI-Stream auto-mapping for '{}': matched {standards}. Add explicit --map entries. See 'wavepeek extract axistream --help'.",
                candidate.display
            )));
        }
        if matched.is_empty() {
            diagnostics.push(Diagnostic::warning(
                WarningDiagnosticCode::UnmatchedExtractCandidate,
                format!(
                    "ignored AXI-Stream include candidate '{}' because it did not match any {} standard signal",
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

    for standard in STANDARD_SIGNALS {
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
                "ambiguous AXI-Stream auto-mapping for '{standard}': {paths}. Add --map {standard}=<signal>. See 'wavepeek extract axistream --help'."
            )));
        }
        let candidate = candidates.remove(0);
        explicit.insert(
            standard.to_string(),
            AxiStreamSignalMapping {
                standard: standard.to_string(),
                display: candidate.display,
                path: candidate.path,
            },
        );
    }

    Ok((explicit, diagnostics))
}

#[derive(Debug)]
struct BuiltAxiStreamSource {
    on: String,
    when: String,
    payload_waves: Vec<String>,
    payload_standards: Vec<String>,
}

fn build_extract_source(
    tready_mode: TreadyMode,
    scope: Option<&str>,
    mappings: &HashMap<String, AxiStreamSignalMapping>,
) -> Result<BuiltAxiStreamSource, WavepeekError> {
    let aclk = mappings.get("aclk").ok_or_else(|| {
        WavepeekError::Args(
            "AXI-Stream mapping requires aclk. See 'wavepeek extract axistream --help'."
                .to_string(),
        )
    })?;
    let valid = mappings.get("tvalid").ok_or_else(|| {
        WavepeekError::Args(
            "AXI-Stream mapping requires tvalid. See 'wavepeek extract axistream --help'."
                .to_string(),
        )
    })?;
    let ready = mappings.get("tready");
    match (tready_mode, ready) {
        (TreadyMode::Mapped, None) => {
            return Err(WavepeekError::Args(
                "AXI-Stream tready-mode mapped requires a tready mapping. See 'wavepeek extract axistream --help'."
                    .to_string(),
            ));
        }
        (TreadyMode::ImplicitHigh, Some(_)) => {
            return Err(WavepeekError::Args(
                "AXI-Stream tready-mode implicit-high forbids a tready mapping. See 'wavepeek extract axistream --help'."
                    .to_string(),
            ));
        }
        _ => {}
    }

    let reset = mappings.get("aresetn");
    let mut predicates = Vec::new();
    if let Some(reset) = reset {
        predicates.push(expr_name(reset, scope));
    }
    predicates.push(expr_name(valid, scope));
    if let Some(ready) = ready {
        predicates.push(expr_name(ready, scope));
    }

    let payload = PAYLOAD_SIGNALS
        .iter()
        .filter_map(|standard| mappings.get(*standard).map(|mapping| (*standard, mapping)))
        .collect::<Vec<_>>();

    Ok(BuiltAxiStreamSource {
        on: format!("posedge {}", expr_name(aclk, scope)),
        when: predicates.join(" && "),
        payload_waves: payload
            .iter()
            .map(|(_, mapping)| expr_name(mapping, scope).to_string())
            .collect(),
        payload_standards: payload
            .iter()
            .map(|(standard, _)| (*standard).to_string())
            .collect(),
    })
}

fn expr_name<'a>(mapping: &'a AxiStreamSignalMapping, scope: Option<&str>) -> &'a str {
    if scope.is_some() {
        mapping.display.as_str()
    } else {
        mapping.path.as_str()
    }
}

impl AxiStreamProfile {
    fn name(self) -> &'static str {
        self.spec.name
    }

    fn issue(self) -> &'static str {
        self.spec.issue
    }
}

impl TreadyMode {
    fn name(self) -> &'static str {
        match self {
            Self::Mapped => "mapped",
            Self::ImplicitHigh => "implicit-high",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        STANDARD_SIGNALS, TreadyMode, parse_cli_maps, parse_profile, parse_tready_mode,
        profile_specs,
    };
    use crate::engine::signal_mapping::candidate_matching_standards;

    #[test]
    fn profiles_accept_canonical_case_and_underscore_aliases() {
        for (input, canonical) in [
            ("axi4-stream", "axi4-stream"),
            ("AXI4_STREAM", "axi4-stream"),
            ("axi5-stream", "axi5-stream"),
            ("AXI5_STREAM", "axi5-stream"),
        ] {
            assert_eq!(parse_profile(input).unwrap().name(), canonical);
        }
        assert_eq!(
            profile_specs()
                .iter()
                .map(|profile| (profile.name, profile.issue))
                .collect::<Vec<_>>(),
            [("axi4-stream", "B"), ("axi5-stream", "B")]
        );
    }

    #[test]
    fn tready_modes_accept_canonical_and_underscore_alias() {
        assert_eq!(parse_tready_mode("mapped").unwrap(), TreadyMode::Mapped);
        assert_eq!(
            parse_tready_mode("IMPLICIT_HIGH").unwrap(),
            TreadyMode::ImplicitHigh
        );
    }

    #[test]
    fn cli_maps_normalize_and_reject_duplicates() {
        assert_eq!(
            parse_cli_maps(&[" TVALID = valid ".to_string()]).unwrap(),
            [("tvalid".to_string(), "valid".to_string())]
        );
        assert!(parse_cli_maps(&["tdata=a".to_string(), "TDATA=b".to_string()]).is_err());
    }

    #[test]
    fn standard_matcher_accepts_documented_forms_and_rejects_check_decoys() {
        for name in [
            "tvalid",
            "t_valid",
            "axis_tvalid_o",
            "axis_t_valid_o",
            "s_axis_tvalid",
            "m_axis_t_valid",
        ] {
            assert_eq!(
                candidate_matching_standards(name, STANDARD_SIGNALS, super::CONTEXT_SIGNALS),
                ["tvalid"],
                "{name}"
            );
        }
        for name in ["tvalidchk", "t_ready_chk", "axis_tdatachk", "twakeup"] {
            assert!(
                candidate_matching_standards(name, STANDARD_SIGNALS, super::CONTEXT_SIGNALS)
                    .is_empty(),
                "{name}"
            );
        }
    }

    #[test]
    fn signal_sets_are_exact_and_ordered() {
        assert_eq!(
            STANDARD_SIGNALS,
            [
                "aclk", "aresetn", "tvalid", "tready", "tdata", "tstrb", "tkeep", "tlast", "tid",
                "tdest", "tuser"
            ]
        );
    }
}
