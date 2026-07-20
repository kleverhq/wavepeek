use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::sync::Arc;

use regex::Regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::cli::extract::AhbArgs;
use crate::contract::schema::INPUT_SCHEMA_URL;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{
    SharedWaveform, bind_waveform_event_expr, candidate_sources_for_handles,
    event_candidate_handles, event_expr_matches, open_shared_waveform,
};
use crate::engine::extract::{initial_diagnostics, max_entries, parse_bound_time};
use crate::engine::time::{format_raw_timestamp, parse_dump_time_context};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::expr::{BoundEventExpr, EventEvalFrame};
use crate::waveform::{
    ChangeCandidateCollectionMode, ExprResolvedSignal, ResolvedSignal, SampledSignalState,
    expr_host::WaveformExprHost,
};

const DEFAULT_PROFILE: &str = "ahb-lite";
const DEFAULT_NAME: &str = "ahb";
const SOURCE_KIND: &str = "extract.ahb.source";
const HELP: &str = "wavepeek extract ahb";
const CANDIDATE_CHUNK_RAW: u64 = 1_000_000;
const MAX_CANDIDATE_CHUNK_RAW: u64 = 4_000_000;
const REQUIRED_SIGNALS: &[&str] = &["hclk", "htrans", "hready", "hwrite"];
const AHB_LITE_SIGNALS: &[&str] = &[
    "hclk",
    "hresetn",
    "htrans",
    "hready",
    "hwrite",
    "haddr",
    "hburst",
    "hmastlock",
    "hprot",
    "hsize",
    "hauser",
    "hwdata",
    "hwstrb",
    "hwuser",
    "hrdata",
    "hruser",
    "hbuser",
    "hresp",
];
const AHB5_SIGNALS: &[&str] = &[
    "hclk",
    "hresetn",
    "htrans",
    "hready",
    "hwrite",
    "haddr",
    "hburst",
    "hmastlock",
    "hprot",
    "hsize",
    "hauser",
    "hnonsec",
    "hexcl",
    "hmaster",
    "hwdata",
    "hwstrb",
    "hwuser",
    "hrdata",
    "hruser",
    "hbuser",
    "hresp",
    "hexokay",
];
const ADDRESS_PAYLOAD: &[&str] = &[
    "htrans",
    "hwrite",
    "haddr",
    "hburst",
    "hmastlock",
    "hprot",
    "hsize",
    "hauser",
    "hnonsec",
    "hexcl",
    "hmaster",
];
const IDLE_PAYLOAD: &[&str] = &["htrans", "haddr", "hmastlock"];
const WRITE_DATA_PAYLOAD: &[&str] = &["hwdata", "hwstrb", "hwuser"];
const READ_DATA_PAYLOAD: &[&str] = &["hrdata"];
const SUCCESS_READ_PAYLOAD: &[&str] = &["hruser"];
const SUCCESS_PAYLOAD: &[&str] = &["hbuser", "hexokay"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AhbSignalMapping {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AhbPayloadValue {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AhbAddressSnapshot {
    pub time: String,
    pub sample_time: String,
    pub transfer: String,
    pub direction: String,
    pub payload: Vec<AhbPayloadValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AhbInitialDataPhase {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<AhbAddressSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AhbEvent {
    pub time: String,
    pub sample_time: String,
    pub profile: String,
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub payload: Vec<AhbPayloadValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AhbContext {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub include_stall: bool,
    pub include_idle: bool,
    pub include_busy: bool,
    pub initial_data_phase: AhbInitialDataPhase,
    pub mappings: Vec<AhbSignalMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AhbData {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub include_stall: bool,
    pub include_idle: bool,
    pub include_busy: bool,
    pub initial_data_phase: AhbInitialDataPhase,
    pub mappings: Vec<AhbSignalMapping>,
    pub events: Vec<AhbEvent>,
}

impl AhbData {
    pub(crate) fn context(&self) -> AhbContext {
        AhbContext {
            name: self.name.clone(),
            profile: self.profile.clone(),
            issue: self.issue.clone(),
            include_stall: self.include_stall,
            include_idle: self.include_idle,
            include_busy: self.include_busy,
            initial_data_phase: self.initial_data_phase.clone(),
            mappings: self.mappings.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AhbProfileSpec {
    pub(crate) name: &'static str,
    pub(crate) issue: &'static str,
    pub(crate) signals: &'static [&'static str],
}

const AHB_LITE_PROFILE: AhbProfileSpec = AhbProfileSpec {
    name: "ahb-lite",
    issue: "C",
    signals: AHB_LITE_SIGNALS,
};
const AHB5_PROFILE: AhbProfileSpec = AhbProfileSpec {
    name: "ahb5",
    issue: "C",
    signals: AHB5_SIGNALS,
};

pub(crate) fn profile_specs() -> &'static [AhbProfileSpec] {
    &[AHB_LITE_PROFILE, AHB5_PROFILE]
}

#[derive(Debug, Clone, Copy)]
struct AhbProfile {
    spec: &'static AhbProfileSpec,
}

impl AhbProfile {
    fn name(self) -> &'static str {
        self.spec.name
    }

    fn issue(self) -> &'static str {
        self.spec.issue
    }

    fn signals(self) -> &'static [&'static str] {
        self.spec.signals
    }

    fn contains_standard(self, standard: &str) -> bool {
        self.signals().contains(&standard)
    }
}

#[derive(Debug, Deserialize)]
struct SourceFile {
    #[serde(rename = "$schema")]
    schema: String,
    kind: String,
    #[serde(default, deserialize_with = "optional_string")]
    profile: Option<String>,
    #[serde(default, deserialize_with = "optional_bool")]
    include_stall: Option<bool>,
    #[serde(default, deserialize_with = "optional_bool")]
    include_idle: Option<bool>,
    #[serde(default, deserialize_with = "optional_bool")]
    include_busy: Option<bool>,
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

#[derive(Debug)]
struct AhbConfig {
    profile: AhbProfile,
    name: String,
    include_stall: bool,
    include_idle: bool,
    include_busy: bool,
    includes: Vec<String>,
    maps: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct SignalCandidate {
    display: String,
    name: String,
    path: String,
}

#[derive(Debug)]
struct ResolvedMapping {
    mapping: AhbSignalMapping,
    resolved: ResolvedSignal,
}

#[derive(Debug)]
struct SamplePlan {
    mappings: Vec<ResolvedMapping>,
    resolved: Vec<ResolvedSignal>,
    by_standard: HashMap<String, usize>,
}

impl SamplePlan {
    fn new(mappings: Vec<ResolvedMapping>) -> Self {
        let resolved = mappings
            .iter()
            .map(|mapping| mapping.resolved.clone())
            .collect();
        let by_standard = mappings
            .iter()
            .enumerate()
            .map(|(index, mapping)| (mapping.mapping.standard.clone(), index))
            .collect();
        Self {
            mappings,
            resolved,
            by_standard,
        }
    }
}

#[derive(Debug)]
struct BoundClock {
    source: String,
    host: WaveformExprHost,
    event: BoundEventExpr,
    candidates: Vec<ExprResolvedSignal>,
}

#[derive(Debug)]
struct BuiltAhb {
    waveform: SharedWaveform,
    context: AhbContext,
    resolved_mappings: Vec<ResolvedMapping>,
    bound_clock: BoundClock,
    diagnostics: Vec<Diagnostic>,
    debug: DebugTrace,
}

#[derive(Debug)]
struct AhbOutcome {
    context: AhbContext,
    diagnostics: Vec<Diagnostic>,
    truncated: bool,
}

trait AhbEventSink {
    fn start(&mut self, _context: &AhbContext) -> Result<(), WavepeekError> {
        Ok(())
    }

    fn emit(&mut self, event: AhbEvent) -> Result<(), WavepeekError>;
}

#[derive(Default)]
struct CollectingAhbSink {
    events: Vec<AhbEvent>,
}

impl AhbEventSink for CollectingAhbSink {
    fn emit(&mut self, event: AhbEvent) -> Result<(), WavepeekError> {
        self.events.push(event);
        Ok(())
    }
}

struct JsonlAhbSink<'a, W: std::io::Write> {
    writer: &'a mut crate::output::JsonlWriter<W>,
}

impl<W: std::io::Write> AhbEventSink for JsonlAhbSink<'_, W> {
    fn start(&mut self, context: &AhbContext) -> Result<(), WavepeekError> {
        self.writer.begin_context(context)
    }

    fn emit(&mut self, event: AhbEvent) -> Result<(), WavepeekError> {
        self.writer.item(&event)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Read,
    Write,
    Unknown,
}

impl Direction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transfer {
    Idle,
    Busy,
    Nonseq,
    Seq,
    Unknown,
}

impl Transfer {
    const fn as_str(self) -> Option<&'static str> {
        match self {
            Self::Idle => Some("idle"),
            Self::Busy => Some("busy"),
            Self::Nonseq => Some("nonseq"),
            Self::Seq => Some("seq"),
            Self::Unknown => None,
        }
    }
}

#[derive(Debug, Clone)]
struct PendingPhase {
    direction: Direction,
    address: AhbAddressSnapshot,
}

#[derive(Debug, Clone)]
enum PipelineState {
    Empty,
    Pending(PendingPhase),
    Desynchronized,
}

impl PipelineState {
    fn initial_context(&self) -> AhbInitialDataPhase {
        match self {
            Self::Empty => AhbInitialDataPhase {
                state: "empty".to_string(),
                address: None,
            },
            Self::Pending(pending) => AhbInitialDataPhase {
                state: "pending".to_string(),
                address: Some(pending.address.clone()),
            },
            Self::Desynchronized => AhbInitialDataPhase {
                state: "desynchronized".to_string(),
                address: None,
            },
        }
    }
}

#[derive(Debug)]
struct EdgeSamples {
    plan: Arc<SamplePlan>,
    sampled: Vec<SampledSignalState>,
}

impl EdgeSamples {
    fn index(&self, standard: &str) -> Option<usize> {
        self.plan.by_standard.get(standard).copied()
    }

    fn bits(&self, standard: &str) -> Option<&str> {
        self.sampled.get(self.index(standard)?)?.bits.as_deref()
    }

    fn has(&self, standard: &str) -> bool {
        self.index(standard).is_some()
    }

    fn known_bit(&self, standard: &str) -> Option<bool> {
        match self.bits(standard)? {
            "0" => Some(false),
            "1" => Some(true),
            _ => None,
        }
    }

    fn transfer(&self) -> Transfer {
        match self.bits("htrans") {
            Some("00") => Transfer::Idle,
            Some("01") => Transfer::Busy,
            Some("10") => Transfer::Nonseq,
            Some("11") => Transfer::Seq,
            _ => Transfer::Unknown,
        }
    }

    fn direction(&self) -> Direction {
        match self.known_bit("hwrite") {
            Some(false) => Direction::Read,
            Some(true) => Direction::Write,
            None => Direction::Unknown,
        }
    }

    fn payload(&self, standards: &[&str]) -> Vec<AhbPayloadValue> {
        standards
            .iter()
            .filter_map(|standard| {
                let index = self.index(standard)?;
                Some(payload_value(
                    &self.plan.mappings[index].mapping,
                    &self.sampled[index],
                ))
            })
            .collect()
    }

    #[cfg(test)]
    fn for_test(values: &[(&str, &str)]) -> Self {
        use crate::waveform::SignalId;

        let mappings = values
            .iter()
            .enumerate()
            .map(|(index, (standard, bits))| ResolvedMapping {
                mapping: AhbSignalMapping {
                    standard: (*standard).to_string(),
                    display: (*standard).to_string(),
                    path: format!("top.{standard}"),
                },
                resolved: ResolvedSignal {
                    path: format!("top.{standard}"),
                    id: SignalId::from_test_index(index as u64),
                    width: bits.len() as u32,
                },
            })
            .collect::<Vec<_>>();
        let sampled = values
            .iter()
            .map(|(standard, bits)| SampledSignalState {
                path: format!("top.{standard}"),
                width: bits.len() as u32,
                bits: Some((*bits).to_string()),
            })
            .collect();
        Self {
            plan: Arc::new(SamplePlan::new(mappings)),
            sampled,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Inclusion {
    stall: bool,
    idle: bool,
    busy: bool,
}

struct Walker {
    profile: String,
    inclusion: Inclusion,
    state: PipelineState,
    in_reset: bool,
}

impl Walker {
    fn new(profile: &str, inclusion: Inclusion) -> Self {
        Self {
            profile: profile.to_string(),
            inclusion,
            state: PipelineState::Desynchronized,
            in_reset: false,
        }
    }

    fn process_edge(
        &mut self,
        time: &str,
        sample_time: &str,
        samples: &EdgeSamples,
        collect_events: bool,
    ) -> Vec<AhbEvent> {
        let mut events = Vec::new();
        if samples.has("hresetn") {
            match samples.bits("hresetn").expect("sample values checked") {
                "0" => {
                    self.state = PipelineState::Empty;
                    if !self.in_reset && collect_events {
                        events.push(self.boundary_event(time, sample_time, "reset"));
                    }
                    self.in_reset = true;
                    return events;
                }
                "1" => self.in_reset = false,
                _ => {
                    self.in_reset = false;
                    self.enter_desynchronized(time, sample_time, collect_events, &mut events);
                    return events;
                }
            }
        }

        let Some(ready) = samples.known_bit("hready") else {
            self.enter_desynchronized(time, sample_time, collect_events, &mut events);
            return events;
        };

        if !ready {
            if let PipelineState::Pending(pending) = &self.state
                && self.inclusion.stall
                && collect_events
            {
                events.push(self.data_event(
                    time,
                    sample_time,
                    "data-stall",
                    pending.direction,
                    samples,
                ));
            }
            return events;
        }

        let previous = std::mem::replace(&mut self.state, PipelineState::Empty);
        let was_desynchronized = matches!(previous, PipelineState::Desynchronized);
        if let PipelineState::Pending(pending) = previous
            && collect_events
        {
            events.push(self.data_event(
                time,
                sample_time,
                "data-complete",
                pending.direction,
                samples,
            ));
        }

        let transfer = samples.transfer();
        match transfer {
            Transfer::Nonseq | Transfer::Seq => {
                let direction = samples.direction();
                let address = AhbAddressSnapshot {
                    time: time.to_string(),
                    sample_time: sample_time.to_string(),
                    transfer: transfer.as_str().expect("known transfer").to_string(),
                    direction: direction.as_str().to_string(),
                    payload: samples.payload(ADDRESS_PAYLOAD),
                };
                self.state = PipelineState::Pending(PendingPhase {
                    direction,
                    address: address.clone(),
                });
                if collect_events {
                    events.push(AhbEvent {
                        time: time.to_string(),
                        sample_time: sample_time.to_string(),
                        profile: self.profile.clone(),
                        event: "address".to_string(),
                        transfer: Some(address.transfer),
                        direction: Some(address.direction),
                        payload: address.payload,
                    });
                }
            }
            Transfer::Idle => {
                self.state = PipelineState::Empty;
                if self.inclusion.idle && collect_events {
                    events.push(self.address_slot_event(
                        time,
                        sample_time,
                        transfer,
                        None,
                        samples.payload(IDLE_PAYLOAD),
                    ));
                }
            }
            Transfer::Busy => {
                self.state = PipelineState::Empty;
                if self.inclusion.busy && collect_events {
                    events.push(self.address_slot_event(
                        time,
                        sample_time,
                        transfer,
                        Some(samples.direction()),
                        samples.payload(ADDRESS_PAYLOAD),
                    ));
                }
            }
            Transfer::Unknown => {
                self.state = PipelineState::Desynchronized;
                if !was_desynchronized && collect_events {
                    events.push(self.boundary_event(time, sample_time, "desynchronized"));
                }
            }
        }
        events
    }

    fn enter_desynchronized(
        &mut self,
        time: &str,
        sample_time: &str,
        collect_events: bool,
        events: &mut Vec<AhbEvent>,
    ) {
        if !matches!(self.state, PipelineState::Desynchronized) && collect_events {
            events.push(self.boundary_event(time, sample_time, "desynchronized"));
        }
        self.state = PipelineState::Desynchronized;
    }

    fn address_slot_event(
        &self,
        time: &str,
        sample_time: &str,
        transfer: Transfer,
        direction: Option<Direction>,
        payload: Vec<AhbPayloadValue>,
    ) -> AhbEvent {
        AhbEvent {
            time: time.to_string(),
            sample_time: sample_time.to_string(),
            profile: self.profile.clone(),
            event: transfer.as_str().expect("known transfer").to_string(),
            transfer: transfer.as_str().map(str::to_string),
            direction: direction.map(|value| value.as_str().to_string()),
            payload,
        }
    }

    fn boundary_event(&self, time: &str, sample_time: &str, kind: &str) -> AhbEvent {
        AhbEvent {
            time: time.to_string(),
            sample_time: sample_time.to_string(),
            profile: self.profile.clone(),
            event: kind.to_string(),
            transfer: None,
            direction: None,
            payload: Vec::new(),
        }
    }

    fn data_event(
        &self,
        time: &str,
        sample_time: &str,
        kind: &str,
        direction: Direction,
        samples: &EdgeSamples,
    ) -> AhbEvent {
        let mut payload = samples.payload(&["hresp"]);
        match direction {
            Direction::Read if kind == "data-complete" => {
                payload.extend(samples.payload(READ_DATA_PAYLOAD));
            }
            Direction::Read => {}
            Direction::Write => payload.extend(samples.payload(WRITE_DATA_PAYLOAD)),
            Direction::Unknown => {
                payload.extend(samples.payload(WRITE_DATA_PAYLOAD));
                payload.extend(samples.payload(READ_DATA_PAYLOAD));
            }
        }

        let successful_completion =
            kind == "data-complete" && samples.known_bit("hresp").is_some_and(|value| !value);
        if successful_completion {
            if matches!(direction, Direction::Read | Direction::Unknown) {
                payload.extend(samples.payload(SUCCESS_READ_PAYLOAD));
            }
            payload.extend(samples.payload(SUCCESS_PAYLOAD));
        }

        AhbEvent {
            time: time.to_string(),
            sample_time: sample_time.to_string(),
            profile: self.profile.clone(),
            event: kind.to_string(),
            transfer: None,
            direction: Some(direction.as_str().to_string()),
            payload,
        }
    }
}

pub fn run(args: AhbArgs) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let signals_abs = args.abs;
    let mut sink = CollectingAhbSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::ExtractAhb,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        data: CommandData::ExtractAhb(AhbData {
            name: outcome.context.name,
            profile: outcome.context.profile,
            issue: outcome.context.issue,
            include_stall: outcome.context.include_stall,
            include_idle: outcome.context.include_idle,
            include_busy: outcome.context.include_busy,
            initial_data_phase: outcome.context.initial_data_phase,
            mappings: outcome.context.mappings,
            events: sink.events,
        }),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: AhbArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    let outcome = {
        let mut sink = JsonlAhbSink { writer };
        run_with_sink(args, &mut sink)?
    };
    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end(outcome.truncated)
}

fn run_with_sink<S: AhbEventSink + ?Sized>(
    args: AhbArgs,
    sink: &mut S,
) -> Result<AhbOutcome, WavepeekError> {
    let max = max_entries(&args.max)?;
    let mut diagnostics = initial_diagnostics(&args.max);
    let BuiltAhb {
        waveform,
        mut context,
        resolved_mappings,
        bound_clock,
        diagnostics: build_diagnostics,
        debug,
    } = build_ahb(&args)?;
    diagnostics.extend(build_diagnostics);

    let metadata = waveform.borrow().metadata()?;
    let dump_time = parse_dump_time_context(&metadata)?;
    let dump_start_raw =
        u64::try_from(dump_time.dump_start_zs / dump_time.dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump start timestamp exceeds supported range".into())
        })?;
    let dump_end_raw =
        u64::try_from(dump_time.dump_end_zs / dump_time.dump_tick_zs).map_err(|_| {
            WavepeekError::Internal("dump end timestamp exceeds supported range".into())
        })?;
    let from_raw = match args.from.as_deref() {
        Some(value) => parse_bound_time(value, "--from", dump_time, &metadata, HELP)?,
        None => dump_start_raw,
    };
    let to_raw = match args.to.as_deref() {
        Some(value) => parse_bound_time(value, "--to", dump_time, &metadata, HELP)?,
        None => dump_end_raw,
    };
    if from_raw > to_raw {
        return Err(WavepeekError::Args(format!(
            "--from must be less than or equal to --to. See '{HELP} --help'."
        )));
    }

    debug.event("ahb.bind.done", || {
        serde_json::json!({
            "mappings": resolved_mappings.len(),
            "from_raw": from_raw,
            "to_raw": to_raw,
        })
    });
    let sample_plan = Arc::new(SamplePlan::new(resolved_mappings));
    let inclusion = Inclusion {
        stall: context.include_stall,
        idle: context.include_idle,
        busy: context.include_busy,
    };
    let mut walker = Walker::new(context.profile.as_str(), inclusion);
    if from_raw > dump_start_raw {
        visit_candidate_chunks(
            &waveform,
            &bound_clock,
            &sample_plan,
            dump_start_raw,
            from_raw - 1,
            |timestamp| {
                if let Some((time, sample_time, samples)) = sample_rising_edge(
                    &waveform,
                    &sample_plan,
                    &bound_clock,
                    timestamp,
                    dump_start_raw,
                    dump_end_raw,
                    dump_time.dump_tick,
                )? {
                    walker.process_edge(time.as_str(), sample_time.as_str(), &samples, false);
                }
                Ok(std::ops::ControlFlow::Continue(()))
            },
        )?;
    }

    context.initial_data_phase = walker.state.initial_context();
    sink.start(&context)?;
    let mut emitted = 0usize;
    let mut truncated = false;
    visit_candidate_chunks(
        &waveform,
        &bound_clock,
        &sample_plan,
        from_raw,
        to_raw,
        |timestamp| {
            let Some((time, sample_time, samples)) = sample_rising_edge(
                &waveform,
                &sample_plan,
                &bound_clock,
                timestamp,
                dump_start_raw,
                dump_end_raw,
                dump_time.dump_tick,
            )?
            else {
                return Ok(std::ops::ControlFlow::Continue(()));
            };
            let events = walker.process_edge(time.as_str(), sample_time.as_str(), &samples, true);
            for event in events {
                if max.is_some_and(|limit| emitted == limit) {
                    truncated = true;
                    return Ok(std::ops::ControlFlow::Break(()));
                }
                sink.emit(event)?;
                emitted += 1;
            }
            Ok(std::ops::ControlFlow::Continue(()))
        },
    )?;

    if emitted == 0 {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::EmptyResult,
            "no AHB events found in selected time range",
        ));
    }
    if let Some(limit) = max
        && truncated
    {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            format!("truncated output to {limit} entries (use --max to increase limit)"),
        ));
    }
    debug.event("ahb.emit.done", || {
        serde_json::json!({
            "emitted": emitted,
            "truncated": truncated,
            "initial_state": context.initial_data_phase.state,
            "backend_stats": waveform.borrow().debug_stats(),
        })
    });

    Ok(AhbOutcome {
        context,
        diagnostics,
        truncated,
    })
}

fn build_ahb(args: &AhbArgs) -> Result<BuiltAhb, WavepeekError> {
    let config = config_from_args(args)?;
    let includes = compile_include_regexes(&config.includes)?;
    let debug = DebugTrace::for_command(CommandName::ExtractAhb);
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
    let candidates = collect_include_candidates(&waveform, args.scope.as_deref(), &includes)?;
    let explicit = explicit_mappings(
        &waveform,
        args.scope.as_deref(),
        config.profile,
        &config.maps,
    )?;
    let (mappings_by_standard, diagnostics) = auto_mappings(config.profile, candidates, explicit)?;
    require_mappings(&mappings_by_standard)?;

    let ordered_mappings = config
        .profile
        .signals()
        .iter()
        .filter_map(|standard| mappings_by_standard.get(*standard).cloned())
        .collect::<Vec<_>>();
    let canonical_paths = ordered_mappings
        .iter()
        .map(|mapping| mapping.path.clone())
        .collect::<Vec<_>>();
    let resolved = waveform
        .borrow()
        .resolve_signals(canonical_paths.as_slice())?;
    let expr_resolved = waveform
        .borrow()
        .resolve_expr_signals(canonical_paths.as_slice())?;
    waveform
        .borrow()
        .validate_expr_values_supported(expr_resolved.as_slice())?;
    let resolved_mappings = ordered_mappings
        .iter()
        .cloned()
        .zip(resolved)
        .map(|(mapping, resolved)| ResolvedMapping { mapping, resolved })
        .collect::<Vec<_>>();

    let clock = mappings_by_standard
        .get("hclk")
        .expect("required mappings checked");
    let clock_name = expr_name(clock, args.scope.as_deref());
    let source = format!("posedge {clock_name}");
    let (host, event) =
        bind_waveform_event_expr(waveform.clone(), args.scope.as_deref(), source.as_str())?;
    let candidates =
        candidate_sources_for_handles(&host, event_candidate_handles(&event).as_slice())?;

    Ok(BuiltAhb {
        waveform,
        context: AhbContext {
            name: config.name,
            profile: config.profile.name().to_string(),
            issue: config.profile.issue().to_string(),
            include_stall: config.include_stall,
            include_idle: config.include_idle,
            include_busy: config.include_busy,
            initial_data_phase: AhbInitialDataPhase {
                state: "desynchronized".to_string(),
                address: None,
            },
            mappings: ordered_mappings,
        },
        resolved_mappings,
        bound_clock: BoundClock {
            source,
            host,
            event,
            candidates,
        },
        diagnostics,
        debug,
    })
}

fn visit_candidate_chunks<F>(
    waveform: &SharedWaveform,
    clock: &BoundClock,
    plan: &SamplePlan,
    from_raw: u64,
    to_raw: u64,
    mut visit: F,
) -> Result<(), WavepeekError>
where
    F: FnMut(u64) -> Result<std::ops::ControlFlow<()>, WavepeekError>,
{
    if from_raw > to_raw {
        return Ok(());
    }
    let mut cursor = from_raw;
    let mut span = CANDIDATE_CHUNK_RAW;
    loop {
        let chunk_end = cursor.saturating_add(span - 1).min(to_raw);
        let candidates = waveform
            .borrow_mut()
            .collect_expr_candidate_times_with_mode(
                clock.candidates.as_slice(),
                cursor,
                chunk_end,
                ChangeCandidateCollectionMode::Random,
            )?;
        let empty = candidates.is_empty();
        if !empty {
            let preload_from = cursor.saturating_sub(1);
            waveform.borrow_mut().preload_expr_value_changes(
                clock.candidates.as_slice(),
                preload_from,
                chunk_end,
            )?;
            waveform.borrow_mut().preload_resolved_value_changes(
                plan.resolved.as_slice(),
                preload_from,
                chunk_end,
            )?;
        }
        for timestamp in candidates {
            if visit(timestamp)?.is_break() {
                return Ok(());
            }
        }
        if chunk_end == to_raw {
            return Ok(());
        }
        cursor = chunk_end + 1;
        span = if empty {
            span.saturating_mul(2)
                .clamp(CANDIDATE_CHUNK_RAW, MAX_CANDIDATE_CHUNK_RAW)
        } else {
            CANDIDATE_CHUNK_RAW
        };
    }
}

fn sample_rising_edge(
    waveform: &SharedWaveform,
    plan: &Arc<SamplePlan>,
    clock: &BoundClock,
    timestamp: u64,
    dump_start_raw: u64,
    dump_end_raw: u64,
    dump_tick: crate::engine::time::ParsedTime,
) -> Result<Option<(String, String, EdgeSamples)>, WavepeekError> {
    let previous_timestamp = waveform.borrow().previous_sample_time(timestamp);
    let frame = EventEvalFrame {
        timestamp,
        previous_timestamp,
        tracked_signals: &[],
    };
    if !event_expr_matches(clock.source.as_str(), &clock.event, &clock.host, &frame)? {
        return Ok(None);
    }
    let Some(sample_timestamp) = timestamp
        .checked_sub(1)
        .filter(|value| *value >= dump_start_raw && *value <= dump_end_raw)
    else {
        return Ok(None);
    };
    Ok(Some((
        format_raw_timestamp(timestamp, dump_tick)?,
        format_raw_timestamp(sample_timestamp, dump_tick)?,
        sample_edge(waveform, plan, sample_timestamp)?,
    )))
}

fn sample_edge(
    waveform: &SharedWaveform,
    plan: &Arc<SamplePlan>,
    sample_timestamp: u64,
) -> Result<EdgeSamples, WavepeekError> {
    let sampled = waveform
        .borrow_mut()
        .sample_resolved_optional(plan.resolved.as_slice(), sample_timestamp)?;
    for (mapping, sample) in plan.mappings.iter().zip(&sampled) {
        if sample.bits.is_none() {
            return Err(WavepeekError::Signal(format!(
                "signal '{}' has no value at or before requested time",
                mapping.mapping.path
            )));
        }
    }
    Ok(EdgeSamples {
        plan: Arc::clone(plan),
        sampled,
    })
}

fn payload_value(mapping: &AhbSignalMapping, sampled: &SampledSignalState) -> AhbPayloadValue {
    let bits = sampled.bits.as_deref().expect("sample values checked");
    AhbPayloadValue {
        standard: mapping.standard.clone(),
        display: mapping.display.clone(),
        path: mapping.path.clone(),
        value: format_verilog_literal(sampled.width, bits),
    }
}

fn config_from_args(args: &AhbArgs) -> Result<AhbConfig, WavepeekError> {
    if let Some(path) = args.source.as_ref() {
        if args.name.is_some()
            || !args.maps.is_empty()
            || !args.includes.is_empty()
            || args.include_stall
            || args.include_idle
            || args.include_busy
        {
            return Err(WavepeekError::Args(
                "--source cannot be combined with --profile, --name, --map, --include, --include-stall, --include-idle, or --include-busy. See 'wavepeek extract ahb --help'.".to_string(),
            ));
        }
        return config_from_source(path);
    }
    Ok(AhbConfig {
        profile: parse_profile(args.profile.as_str())?,
        name: args
            .name
            .clone()
            .unwrap_or_else(|| DEFAULT_NAME.to_string()),
        include_stall: args.include_stall,
        include_idle: args.include_idle,
        include_busy: args.include_busy,
        includes: args.includes.clone(),
        maps: parse_cli_maps(&args.maps)?,
    })
}

fn config_from_source(path: &std::path::Path) -> Result<AhbConfig, WavepeekError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        WavepeekError::File(format!(
            "failed to read AHB extract source file '{}': {error}",
            path.display()
        ))
    })?;
    let input: SourceFile = serde_json::from_str(&contents).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid AHB extract source JSON '{}': {error}",
            path.display()
        ))
    })?;
    if input.schema != INPUT_SCHEMA_URL {
        return Err(WavepeekError::Args(format!(
            "AHB extract source file '{}' uses unsupported $schema {}; expected {}",
            path.display(),
            input.schema,
            INPUT_SCHEMA_URL
        )));
    }
    if input.kind != SOURCE_KIND {
        return Err(WavepeekError::Args(format!(
            "AHB extract source file '{}' has kind {}; expected {}",
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
    Ok(AhbConfig {
        profile: parse_profile(input.profile.as_deref().unwrap_or(DEFAULT_PROFILE))?,
        name: input.name.unwrap_or_else(|| DEFAULT_NAME.to_string()),
        include_stall: input.include_stall.unwrap_or(false),
        include_idle: input.include_idle.unwrap_or(false),
        include_busy: input.include_busy.unwrap_or(false),
        includes: input.includes,
        maps,
    })
}

fn parse_profile(profile: &str) -> Result<AhbProfile, WavepeekError> {
    let normalized = profile.trim().to_ascii_lowercase();
    let spec = match normalized.as_str() {
        "ahb-lite" | "ahb_lite" => &AHB_LITE_PROFILE,
        "ahb5" => &AHB5_PROFILE,
        _ => {
            return Err(WavepeekError::Args(format!(
                "unsupported AHB profile '{profile}'; expected ahb-lite or ahb5. See 'wavepeek extract ahb --help'."
            )));
        }
    };
    Ok(AhbProfile { spec })
}

fn parse_cli_maps(values: &[String]) -> Result<Vec<(String, String)>, WavepeekError> {
    let maps = values
        .iter()
        .map(|value| {
            let (standard, waves) = value.split_once('=').ok_or_else(|| {
                WavepeekError::Args(format!(
                    "invalid --map '{}': expected STD_NAME=WAVES_NAME. See 'wavepeek extract ahb --help'.",
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
            "AHB mapped waveform signal names must not be empty. See 'wavepeek extract ahb --help'."
                .to_string(),
        ));
    }
    Ok((standard, waves.to_string()))
}

fn normalize_standard_name(standard: &str) -> Result<String, WavepeekError> {
    let standard = standard.trim().to_ascii_lowercase();
    if standard.is_empty() {
        Err(WavepeekError::Args(
            "AHB standard signal names must not be empty. See 'wavepeek extract ahb --help'."
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
                "duplicate AHB mapping for standard signal '{standard}'. See 'wavepeek extract ahb --help'."
            )));
        }
    }
    Ok(())
}

fn compile_include_regexes(includes: &[String]) -> Result<Vec<Regex>, WavepeekError> {
    includes
        .iter()
        .map(|include| {
            Regex::new(include).map_err(|error| {
                WavepeekError::Args(format!(
                    "invalid AHB include regex '{}': {error}. See 'wavepeek extract ahb --help'.",
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
    for scope in waveform.borrow().scopes_depth_first(None)? {
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
    profile: AhbProfile,
    maps: &[(String, String)],
) -> Result<HashMap<String, AhbSignalMapping>, WavepeekError> {
    let mut result = HashMap::new();
    for (standard, waves) in maps {
        if !profile.contains_standard(standard) {
            return Err(WavepeekError::Args(format!(
                "AHB profile {} has no standard signal '{}'. See 'wavepeek extract ahb --help'.",
                profile.name(),
                standard
            )));
        }
        if scope.is_some() && waves.contains('.') {
            return Err(WavepeekError::Args(format!(
                "AHB mapping for '{standard}' must use a scope-relative signal when --scope is set. See 'wavepeek extract ahb --help'."
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
            AhbSignalMapping {
                standard: standard.clone(),
                display: if scope.is_some() {
                    waves.clone()
                } else {
                    resolved.path.clone()
                },
                path: resolved.path,
            },
        );
    }
    Ok(result)
}

fn auto_mappings(
    profile: AhbProfile,
    candidates: Vec<SignalCandidate>,
    mut explicit: HashMap<String, AhbSignalMapping>,
) -> Result<(HashMap<String, AhbSignalMapping>, Vec<Diagnostic>), WavepeekError> {
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
        let matched = candidate_matching_standards(candidate.name.as_str(), profile.signals());
        if matched.len() > 1 {
            return Err(WavepeekError::Args(format!(
                "ambiguous AHB auto-mapping for '{}': matched {}. Add explicit --map entries. See 'wavepeek extract ahb --help'.",
                candidate.display,
                matched.join(", ")
            )));
        }
        if matched.is_empty() {
            diagnostics.push(Diagnostic::warning(
                WarningDiagnosticCode::UnmatchedExtractCandidate,
                format!(
                    "ignored AHB include candidate '{}' because it did not match any {} standard signal",
                    candidate.display,
                    profile.name()
                ),
            ));
            continue;
        }
        let standard = matched[0];
        if !explicit.contains_key(standard) {
            auto.entry(standard.to_string())
                .or_default()
                .push(candidate);
        }
    }
    for standard in profile.signals() {
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
                "ambiguous AHB auto-mapping for '{standard}': {paths}. Add --map {standard}=<signal>. See 'wavepeek extract ahb --help'."
            )));
        }
        let candidate = candidates.remove(0);
        explicit.insert(
            (*standard).to_string(),
            AhbSignalMapping {
                standard: (*standard).to_string(),
                display: candidate.display,
                path: candidate.path,
            },
        );
    }
    Ok((explicit, diagnostics))
}

fn require_mappings(mappings: &HashMap<String, AhbSignalMapping>) -> Result<(), WavepeekError> {
    let missing = REQUIRED_SIGNALS
        .iter()
        .filter(|standard| !mappings.contains_key(**standard))
        .copied()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(WavepeekError::Args(format!(
            "AHB extraction requires mappings for {}. See 'wavepeek extract ahb --help'.",
            missing.join(", ")
        )))
    }
}

fn expr_name<'a>(mapping: &'a AhbSignalMapping, scope: Option<&str>) -> &'a str {
    if scope.is_some() {
        mapping.display.as_str()
    } else {
        mapping.path.as_str()
    }
}

fn candidate_matching_standards(
    candidate_name: &str,
    standards: &[&'static str],
) -> Vec<&'static str> {
    if is_mapping_decoy(candidate_name) {
        return Vec::new();
    }
    let tokens = candidate_core_tokens(candidate_name);
    let suffix_matches = standards
        .iter()
        .filter(|standard| {
            (0..tokens.len()).any(|start| {
                tokens[start..]
                    .iter()
                    .flat_map(|token| token.chars())
                    .eq(standard.chars())
            })
        })
        .copied()
        .collect::<Vec<_>>();
    let [suffix_standard] = suffix_matches.as_slice() else {
        return suffix_matches;
    };
    let suffix_start = (0..tokens.len())
        .find(|start| {
            tokens[*start..]
                .iter()
                .flat_map(|token| token.chars())
                .eq(suffix_standard.chars())
        })
        .expect("suffix match has a start");
    standards
        .iter()
        .filter(|standard| {
            *standard == suffix_standard
                || (0..suffix_start).any(|start| {
                    tokens[start..suffix_start]
                        .iter()
                        .flat_map(|token| token.chars())
                        .eq(standard.chars())
                })
        })
        .copied()
        .collect()
}

fn is_mapping_decoy(name: &str) -> bool {
    let tokens = tokenize_candidate(name);
    let joined = tokens
        .iter()
        .flat_map(|token| token.chars())
        .collect::<String>();
    joined.contains("hreadyout")
        || tokens.iter().any(|token| {
            token == "chk"
                || token == "check"
                || token == "parity"
                || token.ends_with("chk")
                || token.ends_with("check")
                || token.ends_with("parity")
        })
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

#[cfg(test)]
mod tests {
    use super::{
        AHB_LITE_SIGNALS, AHB5_SIGNALS, AhbSignalMapping, Direction, EdgeSamples, Inclusion,
        PipelineState, SignalCandidate, Walker, auto_mappings, candidate_matching_standards,
        is_mapping_decoy, parse_cli_maps, parse_profile, profile_specs,
    };
    use std::collections::HashMap;

    fn edge(values: &[(&str, &str)]) -> EdgeSamples {
        EdgeSamples::for_test(values)
    }

    #[test]
    fn profiles_and_aliases_are_exact() {
        assert_eq!(parse_profile("AHB_LITE").unwrap().name(), "ahb-lite");
        assert_eq!(parse_profile("ahb5").unwrap().name(), "ahb5");
        assert!(parse_profile("ahb").is_err());
        assert_eq!(
            profile_specs()
                .iter()
                .map(|profile| (profile.name, profile.issue))
                .collect::<Vec<_>>(),
            [("ahb-lite", "C"), ("ahb5", "C")]
        );
        assert_eq!(profile_specs()[0].signals, AHB_LITE_SIGNALS);
        assert_eq!(profile_specs()[1].signals, AHB5_SIGNALS);
    }

    #[test]
    fn map_parser_normalizes_names_and_rejects_duplicates() {
        assert_eq!(
            parse_cli_maps(&[" HADDR = top.addr ".to_string()]).unwrap(),
            [("haddr".to_string(), "top.addr".to_string())]
        );
        assert!(parse_cli_maps(&["broken".to_string()]).is_err());
        assert!(parse_cli_maps(&["haddr=a".to_string(), "HADDR=b".to_string()]).is_err());
    }

    #[test]
    fn normalized_suffix_matching_accepts_names_and_rejects_decoys() {
        for name in ["haddr", "h_addr", "ahb_haddr_i", "ahb_h_addr_i"] {
            assert_eq!(
                candidate_matching_standards(name, AHB5_SIGNALS),
                ["haddr"],
                "{name}"
            );
        }
        for name in [
            "hreadyout",
            "ahb_h_ready_out",
            "haddrchk",
            "haddr_check",
            "haddr_parity",
        ] {
            assert!(is_mapping_decoy(name));
            assert!(candidate_matching_standards(name, AHB5_SIGNALS).is_empty());
        }
    }

    #[test]
    fn auto_mapping_honors_explicit_precedence_and_rejects_duplicate_candidates() {
        let profile = parse_profile("ahb-lite").unwrap();
        let explicit = HashMap::from([(
            "haddr".to_string(),
            AhbSignalMapping {
                standard: "haddr".to_string(),
                display: "manual".to_string(),
                path: "top.manual".to_string(),
            },
        )]);
        let candidates = vec![SignalCandidate {
            display: "ahb_haddr_i".to_string(),
            name: "ahb_haddr_i".to_string(),
            path: "top.ahb_haddr_i".to_string(),
        }];
        let (mapped, _) = auto_mappings(profile, candidates, explicit).unwrap();
        assert_eq!(mapped["haddr"].path, "top.manual");

        let ambiguous = vec![
            SignalCandidate {
                display: "a_haddr".to_string(),
                name: "a_haddr".to_string(),
                path: "top.a_haddr".to_string(),
            },
            SignalCandidate {
                display: "b_haddr".to_string(),
                name: "b_haddr".to_string(),
                path: "top.b_haddr".to_string(),
            },
        ];
        assert!(auto_mappings(profile, ambiguous, HashMap::new()).is_err());
    }

    #[test]
    fn walker_orders_completion_before_next_address() {
        let mut walker = Walker::new(
            "ahb-lite",
            Inclusion {
                stall: false,
                idle: false,
                busy: false,
            },
        );
        let first = walker.process_edge(
            "10ns",
            "9ns",
            &edge(&[("hready", "1"), ("htrans", "10"), ("hwrite", "0")]),
            true,
        );
        assert_eq!(
            first
                .iter()
                .map(|event| event.event.as_str())
                .collect::<Vec<_>>(),
            ["address"]
        );
        let next = walker.process_edge(
            "20ns",
            "19ns",
            &edge(&[("hready", "1"), ("htrans", "11"), ("hwrite", "1")]),
            true,
        );
        assert_eq!(
            next.iter()
                .map(|event| event.event.as_str())
                .collect::<Vec<_>>(),
            ["data-complete", "address"]
        );
        assert!(matches!(walker.state, PipelineState::Pending(_)));
    }

    #[test]
    fn walker_stalls_then_preserves_final_completion_on_idle() {
        let mut walker = Walker::new(
            "ahb-lite",
            Inclusion {
                stall: true,
                idle: false,
                busy: false,
            },
        );
        walker.process_edge(
            "10ns",
            "9ns",
            &edge(&[("hready", "1"), ("htrans", "10"), ("hwrite", "1")]),
            false,
        );
        let stalled = walker.process_edge(
            "20ns",
            "19ns",
            &edge(&[
                ("hready", "0"),
                ("htrans", "10"),
                ("hwrite", "1"),
                ("hresp", "1"),
            ]),
            true,
        );
        assert_eq!(stalled[0].event, "data-stall");
        assert_eq!(stalled[0].direction.as_deref(), Some("write"));
        let completed = walker.process_edge(
            "30ns",
            "29ns",
            &edge(&[
                ("hready", "1"),
                ("htrans", "00"),
                ("hwrite", "0"),
                ("hresp", "1"),
            ]),
            true,
        );
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].event, "data-complete");
        assert!(matches!(walker.state, PipelineState::Empty));
    }

    #[test]
    fn unknown_current_transfer_follows_provable_old_completion() {
        let mut walker = Walker::new(
            "ahb-lite",
            Inclusion {
                stall: false,
                idle: false,
                busy: false,
            },
        );
        walker.process_edge(
            "10ns",
            "9ns",
            &edge(&[("hready", "1"), ("htrans", "10"), ("hwrite", "0")]),
            false,
        );
        let events = walker.process_edge(
            "20ns",
            "19ns",
            &edge(&[("hready", "1"), ("htrans", "xx"), ("hwrite", "0")]),
            true,
        );
        assert_eq!(
            events
                .iter()
                .map(|event| event.event.as_str())
                .collect::<Vec<_>>(),
            ["data-complete", "desynchronized"]
        );
        assert!(matches!(walker.state, PipelineState::Desynchronized));
    }

    #[test]
    fn unknown_control_boundaries_do_not_repeat_and_known_ready_resynchronizes() {
        let mut walker = Walker::new(
            "ahb5",
            Inclusion {
                stall: false,
                idle: false,
                busy: false,
            },
        );
        walker.state = PipelineState::Pending(super::PendingPhase {
            direction: Direction::Read,
            address: super::AhbAddressSnapshot {
                time: "1ns".to_string(),
                sample_time: "0ns".to_string(),
                transfer: "nonseq".to_string(),
                direction: "read".to_string(),
                payload: vec![],
            },
        });
        let first = walker.process_edge(
            "10ns",
            "9ns",
            &edge(&[("hready", "x"), ("htrans", "10"), ("hwrite", "0")]),
            true,
        );
        assert_eq!(first[0].event, "desynchronized");
        let repeated = walker.process_edge(
            "20ns",
            "19ns",
            &edge(&[("hready", "x"), ("htrans", "10"), ("hwrite", "0")]),
            true,
        );
        assert!(repeated.is_empty());
        let recovered = walker.process_edge(
            "30ns",
            "29ns",
            &edge(&[("hready", "1"), ("htrans", "10"), ("hwrite", "x")]),
            true,
        );
        assert_eq!(recovered[0].event, "address");
        assert_eq!(recovered[0].direction.as_deref(), Some("unknown"));
    }

    #[test]
    fn reset_episode_emits_one_boundary_and_unknown_reset_desynchronizes() {
        let mut walker = Walker::new(
            "ahb-lite",
            Inclusion {
                stall: false,
                idle: false,
                busy: false,
            },
        );
        let reset = edge(&[
            ("hresetn", "0"),
            ("hready", "1"),
            ("htrans", "00"),
            ("hwrite", "0"),
        ]);
        assert_eq!(
            walker.process_edge("10ns", "9ns", &reset, true)[0].event,
            "reset"
        );
        assert!(walker.process_edge("20ns", "19ns", &reset, true).is_empty());
        let unknown = edge(&[
            ("hresetn", "x"),
            ("hready", "1"),
            ("htrans", "00"),
            ("hwrite", "0"),
        ]);
        assert_eq!(
            walker.process_edge("30ns", "29ns", &unknown, true)[0].event,
            "desynchronized"
        );
        assert_eq!(
            walker.process_edge("40ns", "39ns", &reset, true)[0].event,
            "reset"
        );
    }
}
