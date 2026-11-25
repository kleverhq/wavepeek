use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::ops::ControlFlow;
use std::sync::Arc;

use regex::Regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use super::output_paths::OutputPaths;
use crate::cli::extract::AhbArgs;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::expr_runtime::{
    SharedWaveform, bind_waveform_event_expr, eval_with_waveform_errors, event_candidate_handles,
    event_expr_matches, expr_diagnostic, open_shared_waveform, signal_operand,
};
use crate::engine::extract::{max_entries, parse_bound_time};
use crate::engine::signal_mapping;
use crate::engine::signal_mapping::{
    MappedSignal, SignalCandidate, collect_include_candidates, resolve_explicit_mapping,
};
use crate::engine::time::{format_raw_timestamp, parse_dump_time_context};
use crate::engine::value_format::format_verilog_literal;
use crate::engine::{
    CommandData, CommandName, CommandResult, DECODING_PROTOCOL_EVENTS, HumanRenderOptions,
    ReportProgress, ResultSummary,
};
use crate::error::WavepeekError;
use crate::expr::{BoundEventExpr, EventEvalFrame, ExpressionHost, SampledValue, SignalHandle};
use crate::waveform::expr_host::RuntimeExpressionHost;
use crate::waveform::expr_query::{ExpressionQuery, ExpressionQueryTick};
use crate::waveform::{ResolvedSignal, SampledSignalState};

const DEFAULT_PROFILE: &str = "ahb-lite";
const DEFAULT_NAME: &str = "ahb";
const SOURCE_KIND: &str = "extract.ahb.source";
const HELP: &str = "wavepeek extract ahb";
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
pub(crate) const ADDRESS_PAYLOAD: &[&str] = &[
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
pub(crate) const IDLE_PAYLOAD: &[&str] = &["htrans", "haddr", "hmastlock"];
pub(crate) const WRITE_DATA_PAYLOAD: &[&str] = &["hwdata", "hwstrb", "hwuser"];
pub(crate) const READ_DATA_PAYLOAD: &[&str] = &["hrdata"];
pub(crate) const SUCCESS_READ_PAYLOAD: &[&str] = &["hruser"];
pub(crate) const SUCCESS_PAYLOAD: &[&str] = &["hbuser", "hexokay"];

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

#[derive(Debug)]
struct ResolvedMapping {
    mapping: AhbSignalMapping,
    resolved: ResolvedSignal,
    handle: SignalHandle,
}

#[derive(Debug)]
struct SamplePlan {
    mappings: Vec<ResolvedMapping>,
    by_standard: HashMap<String, usize>,
}

impl SamplePlan {
    fn new(mappings: Vec<ResolvedMapping>) -> Self {
        let by_standard = mappings
            .iter()
            .enumerate()
            .map(|(index, mapping)| (mapping.mapping.standard.clone(), index))
            .collect();
        Self {
            mappings,
            by_standard,
        }
    }
}

struct BoundClock {
    source: String,
    event: BoundEventExpr,
    query: ExpressionQuery,
}

struct BuiltAhb<'report> {
    waveform: SharedWaveform<'report>,
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
    summary: ResultSummary,
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
        self.writer.data(&event)
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
        let declarations = values
            .iter()
            .enumerate()
            .map(|(index, (_, bits))| {
                format!("$var wire {} s{index} value{index} $end\n", bits.len())
            })
            .collect::<String>();
        let source = format!("{declarations}$enddefinitions $end\n#0\n");
        let wave = ondas::open_bytes("frame.vcd", source.into_bytes().into()).unwrap();

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
                    signal: wave.hierarchy().signal(&format!("value{index}")).unwrap(),
                    width: bits.len() as u32,
                    selector: ondas::SignalSelector::new(
                        ondas::HierarchyPath::from_components(["top", *standard]),
                        None,
                    ),
                },
                handle: SignalHandle(index as u32),
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

pub fn run(args: AhbArgs, report: ReportProgress<'_>) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let signals_abs = args.abs;
    let summary_only = args.summary;
    let mut sink = CollectingAhbSink::default();
    let outcome = run_with_sink(args, &mut sink, report)?;

    Ok(CommandResult {
        command: CommandName::ExtractAhb,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        scope: None,
        summary_only,
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
        summary: Some(outcome.summary),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: AhbArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    writer.suppress_data(args.summary);
    let outcome = {
        let mut sink = JsonlAhbSink { writer };
        run_with_sink(args, &mut sink, &|_| {})?
    };
    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end_summary(&outcome.summary)
}

fn run_with_sink<S: AhbEventSink + ?Sized>(
    args: AhbArgs,
    sink: &mut S,
    report: ReportProgress<'_>,
) -> Result<AhbOutcome, WavepeekError> {
    let max = max_entries(&args.max)?;
    let mut diagnostics = Vec::new();
    let BuiltAhb {
        waveform,
        mut context,
        resolved_mappings,
        bound_clock,
        diagnostics: build_diagnostics,
        debug,
    } = build_ahb(&args, report)?;
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
    let mut started = false;
    let mut emitted = 0usize;
    let mut truncated = false;
    report(DECODING_PROTOCOL_EVENTS);
    waveform.borrow_mut().query_expressions(
        &bound_clock.query,
        dump_start_raw..=to_raw,
        false,
        |host, tick| {
            let ExpressionQueryTick::Candidate(timestamp) = tick else {
                return Ok(ControlFlow::Continue(()));
            };
            let collect_events = timestamp >= from_raw;
            if collect_events && !started {
                context.initial_data_phase = walker.state.initial_context();
                sink.start(&context)?;
                started = true;
            }
            let Some((time, sample_time, samples)) = sample_rising_edge(
                host,
                &sample_plan,
                &bound_clock,
                timestamp,
                dump_start_raw,
                dump_end_raw,
                dump_time.dump_tick,
            )?
            else {
                return Ok(ControlFlow::Continue(()));
            };
            let events = walker.process_edge(
                time.as_str(),
                sample_time.as_str(),
                &samples,
                collect_events,
            );
            for event in events {
                if max.is_some_and(|limit| emitted == limit) {
                    truncated = true;
                    return Ok(ControlFlow::Break(()));
                }
                sink.emit(event)?;
                emitted += 1;
            }
            Ok(ControlFlow::Continue(()))
        },
    )?;
    // A quiet window still exposes the state established by warmup.
    if !started {
        context.initial_data_phase = walker.state.initial_context();
        sink.start(&context)?;
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
        })
    });

    Ok(AhbOutcome {
        context,
        diagnostics,
        summary: ResultSummary::from_run(emitted, max, truncated),
    })
}

fn build_ahb<'report>(
    args: &AhbArgs,
    report: ReportProgress<'report>,
) -> Result<BuiltAhb<'report>, WavepeekError> {
    let config = config_from_args(args)?;
    let includes = compile_include_regexes(&config.includes)?;
    let debug = DebugTrace::for_command(CommandName::ExtractAhb);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = open_shared_waveform(args.waves.as_path(), report)?;
    {
        let waveform_ref = waveform.borrow();
        debug.event("backend.open.done", || {
            serde_json::json!({
                "backend": waveform_ref.backend_name(),
                "format": waveform_ref.format_name(),
            })
        });
    }
    if includes.is_empty()
        && let Some(scope) = args.scope.as_deref()
    {
        waveform.borrow().validate_scope(scope)?;
    }
    let candidates =
        collect_include_candidates(&waveform, args.scope.as_deref(), &includes, report)?;
    let explicit = explicit_mappings(
        &waveform,
        args.scope.as_deref(),
        config.profile,
        &config.maps,
    )?;
    let (mappings_by_standard, diagnostics) = auto_mappings(config.profile, candidates, explicit)?;
    require_mappings(&mappings_by_standard)?;

    let clock = mappings_by_standard
        .get("hclk")
        .expect("required mappings checked");
    let clock_name = signal_operand(&clock.path);
    let source = format!("posedge {clock_name}");
    let (host, event) =
        bind_waveform_event_expr(waveform.clone(), args.scope.as_deref(), source.as_str())?;
    let absolute_paths = args.abs || args.json || args.jsonl;
    let paths = OutputPaths::new(
        args.path_output.style(),
        args.scope.as_deref(),
        absolute_paths,
    )?;
    let resolved_mappings = config
        .profile
        .signals()
        .iter()
        .filter_map(|standard| {
            mappings_by_standard
                .get(*standard)
                .map(|mapping| (standard, mapping))
        })
        .map(|(standard, mapped)| {
            let resolved = waveform.borrow().resolve_typed_signal_with_diagnostic(
                &mapped.path,
                &mapped.selector,
                None,
            )?;
            let handle = host
                .resolve_signal(&mapped.path)
                .map_err(|diagnostic| expr_diagnostic(&mapped.path, diagnostic))?;
            let (path, display) = paths.selector(&mapped.selector)?;
            Ok(ResolvedMapping {
                mapping: AhbSignalMapping {
                    standard: standard.to_string(),
                    path,
                    display,
                },
                resolved,
                handle,
            })
        })
        .collect::<Result<Vec<_>, WavepeekError>>()?;
    let ordered_mappings = resolved_mappings
        .iter()
        .map(|mapping| mapping.mapping.clone())
        .collect();
    let operands = resolved_mappings
        .iter()
        .map(|mapping| mapping.handle)
        .collect::<Vec<_>>();
    let query = ExpressionQuery::new(&host, &event_candidate_handles(&event), &operands)?;

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
            event,
            query,
        },
        diagnostics,
        debug,
    })
}

fn sample_rising_edge(
    host: &dyn RuntimeExpressionHost,
    plan: &Arc<SamplePlan>,
    clock: &BoundClock,
    timestamp: u64,
    dump_start_raw: u64,
    dump_end_raw: u64,
    dump_tick: crate::engine::time::ParsedTime,
) -> Result<Option<(String, String, EdgeSamples)>, WavepeekError> {
    let previous_timestamp = timestamp
        .checked_sub(1)
        .filter(|time| *time >= dump_start_raw);
    let frame = EventEvalFrame {
        timestamp,
        previous_timestamp,
        tracked_signals: &[],
    };
    if !event_expr_matches(clock.source.as_str(), &clock.event, host, &frame)? {
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
        sample_edge(host, plan, sample_timestamp)?,
    )))
}

fn sample_edge(
    host: &dyn RuntimeExpressionHost,
    plan: &Arc<SamplePlan>,
    sample_timestamp: u64,
) -> Result<EdgeSamples, WavepeekError> {
    let sampled = plan
        .mappings
        .iter()
        .map(|mapping| {
            let path = &mapping.resolved.path;
            let bits = if mapping.resolved.width == 0 {
                eval_with_waveform_errors(path, host, || {
                    host.event_occurred(mapping.handle, sample_timestamp)
                })?;
                Some(String::new())
            } else {
                match eval_with_waveform_errors(path, host, || {
                    host.sample_value(mapping.handle, sample_timestamp)
                })? {
                    SampledValue::Integral { bits, .. } => bits,
                    _ => {
                        return Err(WavepeekError::Signal(format!(
                            "signal '{path}' has unsupported non-bit-vector encoding"
                        )));
                    }
                }
            };
            let bits = bits.ok_or_else(|| {
                WavepeekError::Signal(format!(
                    "signal '{path}' has no value at or before requested time"
                ))
            })?;
            Ok(SampledSignalState {
                path: path.clone(),
                width: mapping.resolved.width,
                bits: Some(bits.chars().map(crate::logic::four_state).collect()),
            })
        })
        .collect::<Result<Vec<_>, WavepeekError>>()?;
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
    let waves = crate::cli::signal_token(waves);
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

fn explicit_mappings(
    waveform: &SharedWaveform<'_>,
    scope: Option<&str>,
    profile: AhbProfile,
    maps: &[(String, String)],
) -> Result<HashMap<String, MappedSignal>, WavepeekError> {
    let mut result = HashMap::new();
    for (standard, waves) in maps {
        if !profile.contains_standard(standard) {
            return Err(WavepeekError::Args(format!(
                "AHB profile {} has no standard signal '{}'. See 'wavepeek extract ahb --help'.",
                profile.name(),
                standard
            )));
        }
        result.insert(
            standard.clone(),
            resolve_explicit_mapping(waveform, waves, scope, standard, "AHB", HELP)?,
        );
    }
    Ok(result)
}

fn auto_mappings(
    profile: AhbProfile,
    candidates: Vec<SignalCandidate>,
    mut explicit: HashMap<String, MappedSignal>,
) -> Result<(HashMap<String, MappedSignal>, Vec<Diagnostic>), WavepeekError> {
    let mut diagnostics = Vec::new();
    let mut auto: HashMap<String, Vec<SignalCandidate>> = HashMap::new();
    let explicit_paths = explicit
        .values()
        .map(|mapping| mapping.path.as_str())
        .collect::<HashSet<_>>();
    for candidate in candidates {
        if explicit_paths.contains(candidate.selection.path.as_str()) {
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
        candidates.sort_by(|left, right| left.selection.path.cmp(&right.selection.path));
        candidates.dedup_by(|left, right| left.selection.path == right.selection.path);
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
        explicit.insert(standard.to_string(), candidate.selection);
    }
    Ok((explicit, diagnostics))
}

fn require_mappings(mappings: &HashMap<String, MappedSignal>) -> Result<(), WavepeekError> {
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

fn candidate_matching_standards(
    candidate_name: &str,
    standards: &[&'static str],
) -> Vec<&'static str> {
    if is_mapping_decoy(candidate_name) {
        Vec::new()
    } else {
        signal_mapping::candidate_matching_standards(candidate_name, standards, &[])
    }
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

#[cfg(test)]
mod tests {
    use super::{
        AHB_LITE_PROFILE, AHB_LITE_SIGNALS, AHB5_PROFILE, AHB5_SIGNALS, Direction, EdgeSamples,
        Inclusion, MappedSignal, PipelineState, SignalCandidate, Walker, auto_mappings,
        candidate_matching_standards, is_mapping_decoy, parse_cli_maps, parse_profile,
    };
    use std::collections::HashMap;

    fn edge(values: &[(&str, &str)]) -> EdgeSamples {
        EdgeSamples::for_test(values)
    }

    #[test]
    fn sink_failures_stop_output_and_preserve_the_error_type() {
        struct FailingSink {
            fail_start: bool,
            starts: Vec<String>,
            emitted: usize,
        }
        impl super::AhbEventSink for FailingSink {
            fn start(
                &mut self,
                context: &super::AhbContext,
            ) -> Result<(), crate::error::WavepeekError> {
                self.starts.push(context.initial_data_phase.state.clone());
                if self.fail_start {
                    return Err(crate::error::WavepeekError::File("start sentinel".into()));
                }
                Ok(())
            }
            fn emit(&mut self, _: super::AhbEvent) -> Result<(), crate::error::WavepeekError> {
                self.emitted += 1;
                Err(crate::error::WavepeekError::Signal("emit sentinel".into()))
            }
        }
        let fixture = tempfile::NamedTempFile::with_suffix(".vcd").unwrap();
        std::fs::write(fixture.path(), "$timescale 1ns $end\n$scope module top $end\n$var wire 1 c clk $end\n$var wire 2 t trans $end\n$var wire 1 y ready $end\n$var wire 1 w write $end\n$upscope $end\n$enddefinitions $end\n#0\n0c\nb10 t\n1y\n0w\n#5\n1c\n#10\n0c\n#15\n1c\n#20\n").unwrap();
        for (fail_start, from, to, initial) in [
            (true, None, None, "desynchronized"),
            (true, Some("6ns"), Some("9ns"), "pending"),
            (false, None, None, "desynchronized"),
        ] {
            let mut sink = FailingSink {
                fail_start,
                starts: Vec::new(),
                emitted: 0,
            };
            let error = super::run_with_sink(
                crate::cli::extract::AhbArgs {
                    path_output: Default::default(),
                    waves: fixture.path().into(),
                    profile: crate::cli::extract::AhbProfileArg::AhbLite,
                    source: None,
                    name: None,
                    from: from.map(str::to_owned),
                    to: to.map(str::to_owned),
                    scope: Some("top".into()),
                    maps: ["hclk=clk", "htrans=trans", "hready=ready", "hwrite=write"]
                        .map(str::to_owned)
                        .to_vec(),
                    includes: vec![],
                    include_stall: false,
                    include_idle: false,
                    include_busy: false,
                    max: crate::cli::limits::LimitArg::Unlimited,
                    summary: false,
                    abs: false,
                    json: false,
                    jsonl: true,
                },
                &mut sink,
                &|_| {},
            )
            .unwrap_err();
            assert_eq!(sink.starts, [initial]);
            assert_eq!(sink.emitted, usize::from(!fail_start));
            if fail_start {
                assert!(
                    matches!(error, crate::error::WavepeekError::File(ref message) if message == "start sentinel")
                );
            } else {
                assert!(
                    matches!(error, crate::error::WavepeekError::Signal(ref message) if message == "emit sentinel")
                );
            }
        }
    }

    #[test]
    fn sample_failure_preserves_the_typed_waveform_error() {
        use crate::expr::{
            DiagnosticLayer, ExprDiagnostic, ExprType, SampledValue, SignalHandle, Span,
        };
        use std::cell::Cell;
        struct FailedRead(Cell<bool>);
        impl crate::waveform::expr_host::RuntimeExpressionHost for FailedRead {
            fn take_waveform_error(&self) -> Option<crate::error::WavepeekError> {
                self.0
                    .replace(false)
                    .then(|| crate::error::WavepeekError::File("reader sentinel".into()))
            }
        }
        impl crate::expr::ExpressionHost for FailedRead {
            fn resolve_signal(&self, _: &str) -> Result<SignalHandle, ExprDiagnostic> {
                unreachable!()
            }
            fn signal_type(&self, _: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
                unreachable!()
            }
            fn event_occurred(&self, _: SignalHandle, _: u64) -> Result<bool, ExprDiagnostic> {
                unreachable!()
            }
            fn sample_value(
                &self,
                _: SignalHandle,
                _: u64,
            ) -> Result<SampledValue, ExprDiagnostic> {
                self.0.set(true);
                Err(ExprDiagnostic {
                    layer: DiagnosticLayer::Runtime,
                    code: "HOST-SAMPLE-ERROR",
                    message: "reader failed".into(),
                    primary_span: Span::new(0, 0),
                    notes: vec![],
                })
            }
        }
        let samples = edge(&[("hready", "1")]);
        let error =
            super::sample_edge(&FailedRead(Cell::new(false)), &samples.plan, 4).unwrap_err();
        assert!(
            matches!(error, crate::error::WavepeekError::File(ref message) if message == "reader sentinel")
        );
    }

    #[test]
    fn profiles_and_aliases_are_exact() {
        assert_eq!(parse_profile("AHB_LITE").unwrap().name(), "ahb-lite");
        assert_eq!(parse_profile("ahb5").unwrap().name(), "ahb5");
        assert!(parse_profile("ahb").is_err());
        assert_eq!(
            [AHB_LITE_PROFILE, AHB5_PROFILE]
                .iter()
                .map(|profile| (profile.name, profile.issue))
                .collect::<Vec<_>>(),
            [("ahb-lite", "C"), ("ahb5", "C")]
        );
        assert_eq!(AHB_LITE_PROFILE.signals, AHB_LITE_SIGNALS);
        assert_eq!(AHB5_PROFILE.signals, AHB5_SIGNALS);
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
            MappedSignal::whole(ondas::HierarchyPath::parse("top.manual").unwrap()),
        )]);
        let candidates = vec![SignalCandidate {
            display: "ahb_haddr_i".to_string(),
            name: "ahb_haddr_i".to_string(),
            selection: MappedSignal::whole(ondas::HierarchyPath::parse("top.ahb_haddr_i").unwrap()),
        }];
        let (mapped, _) = auto_mappings(profile, candidates, explicit).unwrap();
        assert_eq!(mapped["haddr"].path, "top.manual");

        let ambiguous = vec![
            SignalCandidate {
                display: "a_haddr".to_string(),
                name: "a_haddr".to_string(),
                selection: MappedSignal::whole(ondas::HierarchyPath::parse("top.a_haddr").unwrap()),
            },
            SignalCandidate {
                display: "b_haddr".to_string(),
                name: "b_haddr".to_string(),
                selection: MappedSignal::whole(ondas::HierarchyPath::parse("top.b_haddr").unwrap()),
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
