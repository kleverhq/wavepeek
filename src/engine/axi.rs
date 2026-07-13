use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;

use regex::Regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::cli::extract::AxiArgs;
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

const DEFAULT_PROFILE: &str = "axi4";
const DEFAULT_NAME: &str = "axi";
const SOURCE_KIND: &str = "extract.axi.source";
const HELP: &str = "wavepeek extract axi";
const COMMON_SIGNALS: &[&str] = &["aclk", "aresetn"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiSignalMapping {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiTransferPayload {
    pub standard: String,
    #[serde(skip_serializing)]
    pub display: String,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiTransfer {
    pub time: String,
    pub sample_time: String,
    pub profile: String,
    pub channel: String,
    pub payload: Vec<AxiTransferPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiContext {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub mappings: Vec<AxiSignalMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxiData {
    pub name: String,
    pub profile: String,
    pub issue: String,
    pub mappings: Vec<AxiSignalMapping>,
    pub transfers: Vec<AxiTransfer>,
}

impl AxiData {
    pub(crate) fn context(&self) -> AxiContext {
        AxiContext {
            name: self.name.clone(),
            profile: self.profile.clone(),
            issue: self.issue.clone(),
            mappings: self.mappings.clone(),
        }
    }
}

#[derive(Debug)]
struct AxiOutcome {
    context: AxiContext,
    diagnostics: Vec<Diagnostic>,
    stats: ExtractRunStats,
}

trait AxiTransferSink {
    fn start(&mut self, _context: &AxiContext) -> Result<(), WavepeekError> {
        Ok(())
    }

    fn emit(&mut self, transfer: AxiTransfer) -> Result<(), WavepeekError>;
}

#[derive(Default)]
struct CollectingAxiSink {
    transfers: Vec<AxiTransfer>,
}

impl AxiTransferSink for CollectingAxiSink {
    fn emit(&mut self, transfer: AxiTransfer) -> Result<(), WavepeekError> {
        self.transfers.push(transfer);
        Ok(())
    }
}

struct JsonlAxiSink<'a, W: std::io::Write> {
    writer: &'a mut crate::output::JsonlWriter<W>,
}

impl<W: std::io::Write> AxiTransferSink for JsonlAxiSink<'_, W> {
    fn start(&mut self, context: &AxiContext) -> Result<(), WavepeekError> {
        self.writer.begin_context(context)
    }

    fn emit(&mut self, transfer: AxiTransfer) -> Result<(), WavepeekError> {
        self.writer.item(&transfer)
    }
}

struct GenericToAxiSink<'a, S: AxiTransferSink + ?Sized> {
    context: &'a AxiContext,
    payload_standards: &'a HashMap<String, Vec<String>>,
    sink: &'a mut S,
}

impl<S: AxiTransferSink + ?Sized> ExtractRowSink for GenericToAxiSink<'_, S> {
    fn start(&mut self) -> Result<(), WavepeekError> {
        self.sink.start(self.context)
    }

    fn emit(&mut self, row: ExtractGenericRow) -> Result<(), WavepeekError> {
        let standards = self
            .payload_standards
            .get(row.source.as_str())
            .ok_or_else(|| {
                WavepeekError::Internal(format!(
                    "missing AXI payload metadata for channel '{}'",
                    row.source
                ))
            })?;
        if standards.len() != row.payload.len() {
            return Err(WavepeekError::Internal(format!(
                "AXI payload metadata length mismatch for channel '{}'",
                row.source
            )));
        }
        let payload = row
            .payload
            .into_iter()
            .zip(standards.iter())
            .map(|(payload, standard)| AxiTransferPayload {
                standard: standard.clone(),
                display: payload.display,
                path: payload.path,
                value: payload.value,
            })
            .collect();
        self.sink.emit(AxiTransfer {
            time: row.time,
            sample_time: row.sample_time,
            profile: self.context.profile.clone(),
            channel: row.source,
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
struct AxiConfig {
    profile: AxiProfile,
    name: String,
    includes: Vec<String>,
    maps: Vec<(String, String)>,
}

#[derive(Debug)]
struct BuiltAxiPlan {
    context: AxiContext,
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
struct AxiProfile {
    spec: &'static AxiProfileSpec,
}

#[derive(Debug)]
pub(crate) struct AxiProfileSpec {
    pub(crate) name: &'static str,
    pub(crate) issue: &'static str,
    pub(crate) channels: &'static [AxiChannelSpec],
}

#[derive(Debug)]
pub(crate) struct AxiChannelSpec {
    pub(crate) name: &'static str,
    pub(crate) valid: &'static str,
    pub(crate) ready: &'static str,
    pub(crate) signals: &'static [&'static str],
}

// AXI3 signal names are based on Arm IHI 0022H.c Tables A2-2 through A2-6.
const AXI3_AW: &[&str] = &[
    "awid", "awaddr", "awlen", "awsize", "awburst", "awlock", "awcache", "awprot", "awvalid",
    "awready",
];
const AXI3_W: &[&str] = &["wid", "wdata", "wstrb", "wlast", "wvalid", "wready"];
const AXI3_B: &[&str] = &["bid", "bresp", "bvalid", "bready"];
const AXI3_AR: &[&str] = &[
    "arid", "araddr", "arlen", "arsize", "arburst", "arlock", "arcache", "arprot", "arvalid",
    "arready",
];
const AXI3_R: &[&str] = &["rid", "rdata", "rresp", "rlast", "rvalid", "rready"];
const AXI3_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: AXI3_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: AXI3_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: AXI3_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: AXI3_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: AXI3_R,
    },
];

// AXI4 signal names are based on Arm IHI 0022H.c Tables A2-2 through A2-6.
const AXI4_AW: &[&str] = &[
    "awid", "awaddr", "awlen", "awsize", "awburst", "awlock", "awcache", "awprot", "awqos",
    "awregion", "awuser", "awvalid", "awready",
];
const AXI4_W: &[&str] = &["wdata", "wstrb", "wlast", "wuser", "wvalid", "wready"];
const AXI4_B: &[&str] = &["bid", "bresp", "buser", "bvalid", "bready"];
const AXI4_AR: &[&str] = &[
    "arid", "araddr", "arlen", "arsize", "arburst", "arlock", "arcache", "arprot", "arqos",
    "arregion", "aruser", "arvalid", "arready",
];
const AXI4_R: &[&str] = &[
    "rid", "rdata", "rresp", "rlast", "ruser", "rvalid", "rready",
];
const AXI4_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: AXI4_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: AXI4_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: AXI4_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: AXI4_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: AXI4_R,
    },
];

// AXI4-Lite signal names are based on Arm IHI 0022H.c Table B1-1.
const AXI4_LITE_AW: &[&str] = &["awaddr", "awprot", "awvalid", "awready"];
const AXI4_LITE_W: &[&str] = &["wdata", "wstrb", "wvalid", "wready"];
const AXI4_LITE_B: &[&str] = &["bresp", "bvalid", "bready"];
const AXI4_LITE_AR: &[&str] = &["araddr", "arprot", "arvalid", "arready"];
const AXI4_LITE_R: &[&str] = &["rdata", "rresp", "rvalid", "rready"];
const AXI4_LITE_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: AXI4_LITE_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: AXI4_LITE_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: AXI4_LITE_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: AXI4_LITE_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: AXI4_LITE_R,
    },
];

// AXI5 signal names are based on Arm IHI 0022L Tables B1.1 through B1.7 and
// the ready/valid interface-class presence rules in Table B2.2.
const AXI5_AW: &[&str] = &[
    "awid",
    "awaddr",
    "awregion",
    "awlen",
    "awsize",
    "awburst",
    "awlock",
    "awcache",
    "awprot",
    "awnse",
    "awpas",
    "awinst",
    "awpriv",
    "awqos",
    "awuser",
    "awdomain",
    "awsnoop",
    "awstashnid",
    "awstashniden",
    "awstashlpid",
    "awstashlpiden",
    "awtrace",
    "awloop",
    "awmmuvalid",
    "awmmusecsid",
    "awmmusid",
    "awmmussidv",
    "awmmussid",
    "awmmuatst",
    "awmmuflow",
    "awmmupasunknown",
    "awmmupm",
    "awpbha",
    "awmecid",
    "awnsaid",
    "awsubsysid",
    "awatop",
    "awmpam",
    "awidunq",
    "awcmo",
    "awtagop",
    "awact",
    "awactv",
    "awvalid",
    "awready",
];
const AXI5_W: &[&str] = &[
    "wdata",
    "wstrb",
    "wtag",
    "wtagupdate",
    "wlast",
    "wuser",
    "wpoison",
    "wtrace",
    "wvalid",
    "wready",
];
const AXI5_B: &[&str] = &[
    "bid",
    "bidunq",
    "bresp",
    "bcomp",
    "bpersist",
    "btagmatch",
    "buser",
    "btrace",
    "bloop",
    "bbusy",
    "bvalid",
    "bready",
];
const AXI5_AR: &[&str] = &[
    "arid",
    "araddr",
    "arregion",
    "arlen",
    "arsize",
    "arburst",
    "arlock",
    "arcache",
    "arprot",
    "arnse",
    "arpas",
    "arinst",
    "arpriv",
    "arqos",
    "aruser",
    "ardomain",
    "arsnoop",
    "artrace",
    "arloop",
    "armmuvalid",
    "armmusecsid",
    "armmusid",
    "armmussidv",
    "armmussid",
    "armmuatst",
    "armmuflow",
    "armmupasunknown",
    "armmupm",
    "arpbha",
    "armecid",
    "arnsaid",
    "arsubsysid",
    "armpam",
    "archunken",
    "aridunq",
    "artagop",
    "aract",
    "aractv",
    "arvalid",
    "arready",
];
const AXI5_R: &[&str] = &[
    "rid",
    "ridunq",
    "rdata",
    "rtag",
    "rresp",
    "rlast",
    "ruser",
    "rpoison",
    "rtrace",
    "rloop",
    "rchunkv",
    "rchunknum",
    "rchunkstrb",
    "rbusy",
    "rvalid",
    "rready",
];
const AXI5_AC: &[&str] = &["acaddr", "acvmidext", "actrace", "acvalid", "acready"];
const AXI5_CR: &[&str] = &["crtrace", "crvalid", "crready"];
const AXI5_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: AXI5_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: AXI5_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: AXI5_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: AXI5_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: AXI5_R,
    },
    AxiChannelSpec {
        name: "ac",
        valid: "acvalid",
        ready: "acready",
        signals: AXI5_AC,
    },
    AxiChannelSpec {
        name: "cr",
        valid: "crvalid",
        ready: "crready",
        signals: AXI5_CR,
    },
];

// AXI5-Lite signal names are based on Arm IHI 0022L Table B2.2.
const AXI5_LITE_AW: &[&str] = &[
    "awid",
    "awaddr",
    "awsize",
    "awprot",
    "awuser",
    "awtrace",
    "awsubsysid",
    "awidunq",
    "awvalid",
    "awready",
];
const AXI5_LITE_W: &[&str] = &[
    "wdata", "wstrb", "wuser", "wpoison", "wtrace", "wvalid", "wready",
];
const AXI5_LITE_B: &[&str] = &[
    "bid", "bidunq", "bresp", "buser", "btrace", "bvalid", "bready",
];
const AXI5_LITE_AR: &[&str] = &[
    "arid",
    "araddr",
    "arsize",
    "arprot",
    "aruser",
    "artrace",
    "arsubsysid",
    "aridunq",
    "arvalid",
    "arready",
];
const AXI5_LITE_R: &[&str] = &[
    "rid", "ridunq", "rdata", "rresp", "ruser", "rpoison", "rtrace", "rvalid", "rready",
];
const AXI5_LITE_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: AXI5_LITE_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: AXI5_LITE_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: AXI5_LITE_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: AXI5_LITE_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: AXI5_LITE_R,
    },
];

// ACE signal names are based on Arm IHI 0022H.c Tables D2-1 through D2-6.
const ACE_AW: &[&str] = &[
    "awid", "awaddr", "awlen", "awsize", "awburst", "awlock", "awcache", "awprot", "awqos",
    "awregion", "awuser", "awdomain", "awsnoop", "awbar", "awunique", "awvalid", "awready",
];
const ACE_W: &[&str] = AXI4_W;
const ACE_B: &[&str] = AXI4_B;
const ACE_AR: &[&str] = &[
    "arid", "araddr", "arlen", "arsize", "arburst", "arlock", "arcache", "arprot", "arqos",
    "arregion", "aruser", "ardomain", "arsnoop", "arbar", "arvalid", "arready",
];
const ACE_R: &[&str] = AXI4_R;
const ACE_AC: &[&str] = &["acaddr", "acsnoop", "acprot", "acvalid", "acready"];
const ACE_CR: &[&str] = &["crresp", "crvalid", "crready"];
const ACE_CD: &[&str] = &["cddata", "cdlast", "cdvalid", "cdready"];
const ACE_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: ACE_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: ACE_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: ACE_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: ACE_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: ACE_R,
    },
    AxiChannelSpec {
        name: "ac",
        valid: "acvalid",
        ready: "acready",
        signals: ACE_AC,
    },
    AxiChannelSpec {
        name: "cr",
        valid: "crvalid",
        ready: "crready",
        signals: ACE_CR,
    },
    AxiChannelSpec {
        name: "cd",
        valid: "cdvalid",
        ready: "cdready",
        signals: ACE_CD,
    },
];

// ACE-Lite uses the AXI4 channels with the address additions permitted by
// Arm IHI 0022H.c Section D11.1. AWUNIQUE is a legal optional payload.
const ACE_LITE_AW: &[&str] = ACE_AW;
const ACE_LITE_W: &[&str] = AXI4_W;
const ACE_LITE_B: &[&str] = AXI4_B;
const ACE_LITE_AR: &[&str] = ACE_AR;
const ACE_LITE_R: &[&str] = AXI4_R;
const ACE_LITE_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: ACE_LITE_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: ACE_LITE_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: ACE_LITE_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: ACE_LITE_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: ACE_LITE_R,
    },
];

// ACE5 signal names are based on Arm IHI 0022H.c Tables F1-2 through F1-9.
// AxBAR is not present because ACE5 does not support barrier transactions.
const ACE5_AW: &[&str] = &[
    "awid",
    "awaddr",
    "awlen",
    "awsize",
    "awburst",
    "awlock",
    "awcache",
    "awprot",
    "awqos",
    "awregion",
    "awuser",
    "awdomain",
    "awsnoop",
    "awunique",
    "awtrace",
    "awloop",
    "awmmusecsid",
    "awmmusid",
    "awmmussidv",
    "awmmussid",
    "awmmuatst",
    "awnsaid",
    "awmpam",
    "awidunq",
    "awvalid",
    "awready",
];
const ACE5_W: &[&str] = &[
    "wdata", "wstrb", "wlast", "wuser", "wpoison", "wtrace", "wvalid", "wready",
];
const ACE5_B: &[&str] = &[
    "bid", "bresp", "buser", "btrace", "bloop", "bidunq", "bvalid", "bready",
];
const ACE5_AR: &[&str] = &[
    "arid",
    "araddr",
    "arlen",
    "arsize",
    "arburst",
    "arlock",
    "arcache",
    "arprot",
    "arqos",
    "arregion",
    "aruser",
    "ardomain",
    "arsnoop",
    "arvmidext",
    "artrace",
    "arloop",
    "armmusecsid",
    "armmusid",
    "armmussidv",
    "armmussid",
    "armmuatst",
    "arnsaid",
    "armpam",
    "aridunq",
    "arvalid",
    "arready",
];
const ACE5_R: &[&str] = &[
    "rid", "rdata", "rresp", "rlast", "ruser", "rpoison", "rtrace", "rloop", "ridunq", "rvalid",
    "rready",
];
const ACE5_AC: &[&str] = &[
    "acaddr",
    "acsnoop",
    "acprot",
    "acvmidext",
    "actrace",
    "acvalid",
    "acready",
];
const ACE5_CR: &[&str] = &["crresp", "crtrace", "crnsaid", "crvalid", "crready"];
const ACE5_CD: &[&str] = &[
    "cddata", "cdlast", "cdpoison", "cdtrace", "cdvalid", "cdready",
];
const ACE5_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: ACE5_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: ACE5_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: ACE5_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: ACE5_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: ACE5_R,
    },
    AxiChannelSpec {
        name: "ac",
        valid: "acvalid",
        ready: "acready",
        signals: ACE5_AC,
    },
    AxiChannelSpec {
        name: "cr",
        valid: "crvalid",
        ready: "crready",
        signals: ACE5_CR,
    },
    AxiChannelSpec {
        name: "cd",
        valid: "cdvalid",
        ready: "cdready",
        signals: ACE5_CD,
    },
];

// ACE5-Lite family signal names are based on Arm IHI 0022L Table B2.2.
const ACE5_LITE_AW: &[&str] = &[
    "awid",
    "awaddr",
    "awregion",
    "awlen",
    "awsize",
    "awburst",
    "awlock",
    "awcache",
    "awprot",
    "awnse",
    "awqos",
    "awuser",
    "awdomain",
    "awsnoop",
    "awstashnid",
    "awstashniden",
    "awstashlpid",
    "awstashlpiden",
    "awtrace",
    "awloop",
    "awmmuvalid",
    "awmmusecsid",
    "awmmusid",
    "awmmussidv",
    "awmmussid",
    "awmmuatst",
    "awmmuflow",
    "awpbha",
    "awmecid",
    "awnsaid",
    "awsubsysid",
    "awatop",
    "awmpam",
    "awidunq",
    "awcmo",
    "awtagop",
    "awvalid",
    "awready",
];
const ACE5_LITE_W: &[&str] = AXI5_W;
const ACE5_LITE_B: &[&str] = AXI5_B;
const ACE5_LITE_AR: &[&str] = &[
    "arid",
    "araddr",
    "arregion",
    "arlen",
    "arsize",
    "arburst",
    "arlock",
    "arcache",
    "arprot",
    "arnse",
    "arqos",
    "aruser",
    "ardomain",
    "arsnoop",
    "artrace",
    "arloop",
    "armmuvalid",
    "armmusecsid",
    "armmusid",
    "armmussidv",
    "armmussid",
    "armmuatst",
    "armmuflow",
    "arpbha",
    "armecid",
    "arnsaid",
    "arsubsysid",
    "armpam",
    "archunken",
    "aridunq",
    "artagop",
    "arvalid",
    "arready",
];
const ACE5_LITE_R: &[&str] = AXI5_R;
const ACE5_LITE_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: ACE5_LITE_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: ACE5_LITE_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: ACE5_LITE_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: ACE5_LITE_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: ACE5_LITE_R,
    },
];

const ACE5_LITE_DVM_AW: &[&str] = &[
    "awid",
    "awaddr",
    "awregion",
    "awlen",
    "awsize",
    "awburst",
    "awlock",
    "awcache",
    "awprot",
    "awnse",
    "awqos",
    "awuser",
    "awdomain",
    "awsnoop",
    "awstashnid",
    "awstashniden",
    "awstashlpid",
    "awstashlpiden",
    "awtrace",
    "awloop",
    "awpbha",
    "awmecid",
    "awnsaid",
    "awsubsysid",
    "awatop",
    "awmpam",
    "awidunq",
    "awcmo",
    "awtagop",
    "awvalid",
    "awready",
];
const ACE5_LITE_DVM_B: &[&str] = &[
    "bid", "bidunq", "bresp", "bcomp", "bpersist", "buser", "btrace", "bloop", "bbusy", "bvalid",
    "bready",
];
const ACE5_LITE_DVM_AR: &[&str] = &[
    "arid",
    "araddr",
    "arregion",
    "arlen",
    "arsize",
    "arburst",
    "arlock",
    "arcache",
    "arprot",
    "arnse",
    "arqos",
    "aruser",
    "ardomain",
    "arsnoop",
    "artrace",
    "arloop",
    "arpbha",
    "armecid",
    "arnsaid",
    "arsubsysid",
    "armpam",
    "archunken",
    "aridunq",
    "artagop",
    "arvalid",
    "arready",
];
const ACE5_LITE_DVM_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: ACE5_LITE_DVM_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: ACE5_LITE_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: ACE5_LITE_DVM_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: ACE5_LITE_DVM_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: ACE5_LITE_R,
    },
    AxiChannelSpec {
        name: "ac",
        valid: "acvalid",
        ready: "acready",
        signals: AXI5_AC,
    },
    AxiChannelSpec {
        name: "cr",
        valid: "crvalid",
        ready: "crready",
        signals: AXI5_CR,
    },
];

const ACE5_LITE_ACP_AW: &[&str] = &[
    "awid",
    "awaddr",
    "awlen",
    "awcache",
    "awprot",
    "awuser",
    "awdomain",
    "awsnoop",
    "awstashnid",
    "awstashniden",
    "awstashlpid",
    "awstashlpiden",
    "awtrace",
    "awmpam",
    "awidunq",
    "awvalid",
    "awready",
];
const ACE5_LITE_ACP_W: &[&str] = &[
    "wdata", "wstrb", "wlast", "wuser", "wpoison", "wtrace", "wvalid", "wready",
];
const ACE5_LITE_ACP_B: &[&str] = &[
    "bid", "bidunq", "bresp", "buser", "btrace", "bvalid", "bready",
];
const ACE5_LITE_ACP_AR: &[&str] = &[
    "arid",
    "araddr",
    "arlen",
    "arcache",
    "arprot",
    "aruser",
    "ardomain",
    "arsnoop",
    "artrace",
    "armpam",
    "archunken",
    "aridunq",
    "arvalid",
    "arready",
];
const ACE5_LITE_ACP_R: &[&str] = &[
    "rid",
    "ridunq",
    "rdata",
    "rresp",
    "rlast",
    "ruser",
    "rpoison",
    "rtrace",
    "rchunkv",
    "rchunknum",
    "rchunkstrb",
    "rvalid",
    "rready",
];
const ACE5_LITE_ACP_CHANNELS: &[AxiChannelSpec] = &[
    AxiChannelSpec {
        name: "aw",
        valid: "awvalid",
        ready: "awready",
        signals: ACE5_LITE_ACP_AW,
    },
    AxiChannelSpec {
        name: "w",
        valid: "wvalid",
        ready: "wready",
        signals: ACE5_LITE_ACP_W,
    },
    AxiChannelSpec {
        name: "b",
        valid: "bvalid",
        ready: "bready",
        signals: ACE5_LITE_ACP_B,
    },
    AxiChannelSpec {
        name: "ar",
        valid: "arvalid",
        ready: "arready",
        signals: ACE5_LITE_ACP_AR,
    },
    AxiChannelSpec {
        name: "r",
        valid: "rvalid",
        ready: "rready",
        signals: ACE5_LITE_ACP_R,
    },
];

const AXI3_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "axi3",
    issue: "H.c",
    channels: AXI3_CHANNELS,
};
const AXI4_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "axi4",
    issue: "H.c",
    channels: AXI4_CHANNELS,
};
const AXI4_LITE_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "axi4-lite",
    issue: "H.c",
    channels: AXI4_LITE_CHANNELS,
};
const AXI5_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "axi5",
    issue: "L",
    channels: AXI5_CHANNELS,
};
const AXI5_LITE_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "axi5-lite",
    issue: "L",
    channels: AXI5_LITE_CHANNELS,
};
const ACE_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "ace",
    issue: "H.c",
    channels: ACE_CHANNELS,
};
const ACE_LITE_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "ace-lite",
    issue: "H.c",
    channels: ACE_LITE_CHANNELS,
};
const ACE5_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "ace5",
    issue: "H.c",
    channels: ACE5_CHANNELS,
};
const ACE5_LITE_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "ace5-lite",
    issue: "L",
    channels: ACE5_LITE_CHANNELS,
};
const ACE5_LITE_DVM_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "ace5-lite-dvm",
    issue: "L",
    channels: ACE5_LITE_DVM_CHANNELS,
};
const ACE5_LITE_ACP_PROFILE: AxiProfileSpec = AxiProfileSpec {
    name: "ace5-lite-acp",
    issue: "L",
    channels: ACE5_LITE_ACP_CHANNELS,
};

pub(crate) fn profile_specs() -> &'static [AxiProfileSpec] {
    &[
        AXI3_PROFILE,
        AXI4_PROFILE,
        AXI4_LITE_PROFILE,
        AXI5_PROFILE,
        AXI5_LITE_PROFILE,
        ACE_PROFILE,
        ACE_LITE_PROFILE,
        ACE5_PROFILE,
        ACE5_LITE_PROFILE,
        ACE5_LITE_DVM_PROFILE,
        ACE5_LITE_ACP_PROFILE,
    ]
}

pub(crate) fn standard_signals(profile: &AxiProfileSpec) -> Vec<&'static str> {
    COMMON_SIGNALS
        .iter()
        .copied()
        .chain(
            profile
                .channels
                .iter()
                .flat_map(|channel| channel.signals.iter().copied()),
        )
        .collect()
}

pub(crate) fn channel_payload_signals(
    channel: &AxiChannelSpec,
) -> impl Iterator<Item = &'static str> + '_ {
    channel
        .signals
        .iter()
        .copied()
        .filter(|standard| *standard != channel.valid && *standard != channel.ready)
}

pub fn run(args: AxiArgs) -> Result<CommandResult, WavepeekError> {
    let output_mode = crate::output_mode::OutputMode::from_json_flags(args.json, args.jsonl);
    let signals_abs = args.abs;
    let mut sink = CollectingAxiSink::default();
    let outcome = run_with_sink(args, &mut sink)?;

    Ok(CommandResult {
        command: CommandName::ExtractAxi,
        output_mode,
        human_options: HumanRenderOptions {
            scope_tree: false,
            signals_abs,
        },
        data: CommandData::ExtractAxi(AxiData {
            name: outcome.context.name,
            profile: outcome.context.profile,
            issue: outcome.context.issue,
            mappings: outcome.context.mappings,
            transfers: sink.transfers,
        }),
        diagnostics: outcome.diagnostics,
    })
}

pub fn run_jsonl<W: std::io::Write>(
    args: AxiArgs,
    writer: &mut crate::output::JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    let outcome = {
        let mut sink = JsonlAxiSink { writer };
        run_with_sink(args, &mut sink)?
    };

    for diagnostic in &outcome.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end(outcome.stats.truncated)
}

fn run_with_sink<S: AxiTransferSink + ?Sized>(
    args: AxiArgs,
    sink: &mut S,
) -> Result<AxiOutcome, WavepeekError> {
    let BuiltAxiPlan {
        context,
        plan,
        waveform,
        debug,
        payload_standards,
        diagnostics: build_diagnostics,
    } = build_axi_plan(&args)?;
    let mut generic_sink = GenericToAxiSink {
        context: &context,
        payload_standards: &payload_standards,
        sink,
    };
    let outcome = extract::run_plan_with_waveform_sink(
        ExtractRunArgs {
            command: CommandName::ExtractAxi,
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
    Ok(AxiOutcome {
        context,
        diagnostics,
        stats: outcome.stats,
    })
}

fn build_axi_plan(args: &AxiArgs) -> Result<BuiltAxiPlan, WavepeekError> {
    let config = config_from_args(args)?;
    let include_regexes = compile_include_regexes(&config.includes)?;
    let debug = DebugTrace::for_command(CommandName::ExtractAxi);
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

    let ordered_mappings = ordered_standard_names(config.profile)
        .into_iter()
        .filter_map(|standard| mappings_by_standard.get(standard).cloned())
        .collect::<Vec<_>>();
    let payload_standards = sources
        .iter()
        .map(|source| (source.channel.to_string(), source.payload_standards.clone()))
        .collect();
    let extract_sources = sources
        .into_iter()
        .enumerate()
        .map(|(index, source)| {
            ExtractSource::new(
                index,
                source.channel,
                source.on,
                source.when,
                source.payload_waves,
            )
        })
        .collect();

    Ok(BuiltAxiPlan {
        context: AxiContext {
            name: config.name,
            profile: config.profile.name().to_string(),
            issue: config.profile.issue().to_string(),
            mappings: ordered_mappings,
        },
        plan: ExtractPlan::new(extract_sources),
        waveform,
        debug,
        payload_standards,
        diagnostics,
    })
}

fn config_from_args(args: &AxiArgs) -> Result<AxiConfig, WavepeekError> {
    if let Some(path) = args.source.as_ref() {
        if args.name.is_some() || !args.maps.is_empty() || !args.includes.is_empty() {
            return Err(WavepeekError::Args(
                "--source cannot be combined with --profile, --name, --map, or --include. See 'wavepeek extract axi --help'.".to_string(),
            ));
        }
        return config_from_source(path);
    }

    Ok(AxiConfig {
        profile: parse_profile(args.profile.as_str())?,
        name: args
            .name
            .clone()
            .unwrap_or_else(|| DEFAULT_NAME.to_string()),
        includes: args.includes.clone(),
        maps: parse_cli_maps(&args.maps)?,
    })
}

fn config_from_source(path: &std::path::Path) -> Result<AxiConfig, WavepeekError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        WavepeekError::File(format!(
            "failed to read AXI extract source file '{}': {error}",
            path.display()
        ))
    })?;
    let input: SourceFile = serde_json::from_str(&contents).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid AXI extract source JSON '{}': {error}",
            path.display()
        ))
    })?;

    if input.schema != INPUT_SCHEMA_URL {
        return Err(WavepeekError::Args(format!(
            "AXI extract source file '{}' uses unsupported $schema {}; expected {}",
            path.display(),
            input.schema,
            INPUT_SCHEMA_URL
        )));
    }
    if input.kind != SOURCE_KIND {
        return Err(WavepeekError::Args(format!(
            "AXI extract source file '{}' has kind {}; expected {}",
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
    Ok(AxiConfig {
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
                    "invalid --map '{}': expected STD_NAME=WAVES_NAME. See 'wavepeek extract axi --help'.",
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
            "AXI mapped waveform signal names must not be empty. See 'wavepeek extract axi --help'."
                .to_string(),
        ));
    }
    Ok((standard, waves.to_string()))
}

fn normalize_standard_name(standard: &str) -> Result<String, WavepeekError> {
    let standard = standard.trim().to_ascii_lowercase();
    if standard.is_empty() {
        Err(WavepeekError::Args(
            "AXI standard signal names must not be empty. See 'wavepeek extract axi --help'."
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
                "duplicate AXI mapping for standard signal '{standard}'. See 'wavepeek extract axi --help'."
            )));
        }
    }
    Ok(())
}

fn parse_profile(profile: &str) -> Result<AxiProfile, WavepeekError> {
    let normalized = profile.trim().to_ascii_lowercase();
    let spec = match normalized.as_str() {
        "axi3" => &AXI3_PROFILE,
        "axi4" => &AXI4_PROFILE,
        "axi4-lite" | "axi4_lite" => &AXI4_LITE_PROFILE,
        "axi5" => &AXI5_PROFILE,
        "axi5-lite" | "axi5_lite" => &AXI5_LITE_PROFILE,
        "ace" => &ACE_PROFILE,
        "ace-lite" | "ace_lite" => &ACE_LITE_PROFILE,
        "ace5" => &ACE5_PROFILE,
        "ace5-lite" | "ace5_lite" => &ACE5_LITE_PROFILE,
        "ace5-lite-dvm" | "ace5-litedvm" | "ace5_litedvm" | "ace5_lite_dvm" => {
            &ACE5_LITE_DVM_PROFILE
        }
        "ace5-lite-acp" | "ace5-liteacp" | "ace5_liteacp" | "ace5_lite_acp" => {
            &ACE5_LITE_ACP_PROFILE
        }
        _ => {
            return Err(WavepeekError::Args(format!(
                "unsupported AXI profile '{profile}'; expected axi3, axi4, axi4-lite, axi5, axi5-lite, ace, ace-lite, ace5, ace5-lite, ace5-lite-dvm, or ace5-lite-acp. See 'wavepeek extract axi --help'."
            )));
        }
    };
    Ok(AxiProfile { spec })
}

fn compile_include_regexes(includes: &[String]) -> Result<Vec<Regex>, WavepeekError> {
    includes
        .iter()
        .map(|include| {
            Regex::new(include).map_err(|error| {
                WavepeekError::Args(format!(
                    "invalid AXI include regex '{}': {error}. See 'wavepeek extract axi --help'.",
                    include
                ))
            })
        })
        .collect()
}

fn collect_include_candidates(
    waveform: &crate::engine::expr_runtime::SharedWaveform,
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
    waveform: &crate::engine::expr_runtime::SharedWaveform,
    scope: Option<&str>,
    profile: AxiProfile,
    maps: &[(String, String)],
) -> Result<HashMap<String, AxiSignalMapping>, WavepeekError> {
    let mut result = HashMap::new();
    for (standard, waves) in maps {
        if !profile.contains_standard(standard) {
            return Err(WavepeekError::Args(format!(
                "AXI profile {} has no standard signal '{}'. See 'wavepeek extract axi --help'.",
                profile.name(),
                standard
            )));
        }
        if scope.is_some() && waves.contains('.') {
            return Err(WavepeekError::Args(format!(
                "AXI mapping for '{standard}' must use a scope-relative signal when --scope is set. See 'wavepeek extract axi --help'."
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
            AxiSignalMapping {
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
    profile: AxiProfile,
    candidates: Vec<SignalCandidate>,
    mut explicit: HashMap<String, AxiSignalMapping>,
) -> Result<(HashMap<String, AxiSignalMapping>, Vec<Diagnostic>), WavepeekError> {
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
            let standards = matched.join(", ");
            return Err(WavepeekError::Args(format!(
                "ambiguous AXI auto-mapping for '{}': matched {standards}. Add explicit --map entries. See 'wavepeek extract axi --help'.",
                candidate.display
            )));
        }
        if matched.is_empty() {
            diagnostics.push(Diagnostic::warning(
                WarningDiagnosticCode::UnmatchedExtractCandidate,
                format!(
                    "ignored AXI include candidate '{}' because it did not match any {} standard signal",
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
                "ambiguous AXI auto-mapping for '{standard}': {paths}. Add --map {standard}=<signal>. See 'wavepeek extract axi --help'."
            )));
        }
        let candidate = candidates.remove(0);
        explicit.insert(
            standard.to_string(),
            AxiSignalMapping {
                standard: standard.to_string(),
                display: candidate.display,
                path: candidate.path,
            },
        );
    }

    Ok((explicit, diagnostics))
}

#[derive(Debug)]
struct BuiltAxiSource {
    channel: &'static str,
    on: String,
    when: String,
    payload_waves: Vec<String>,
    payload_standards: Vec<String>,
}

fn build_extract_sources(
    profile: AxiProfile,
    scope: Option<&str>,
    mappings: &HashMap<String, AxiSignalMapping>,
) -> Result<Vec<BuiltAxiSource>, WavepeekError> {
    let aclk = mappings.get("aclk").ok_or_else(|| {
        WavepeekError::Args(
            "AXI mapping requires aclk. See 'wavepeek extract axi --help'.".to_string(),
        )
    })?;
    let reset = mappings.get("aresetn");
    let mut sources = Vec::new();

    for channel in profile.channels() {
        let valid_mapped = mappings.contains_key(channel.valid);
        let ready_mapped = mappings.contains_key(channel.ready);
        if valid_mapped ^ ready_mapped {
            return Err(WavepeekError::Args(format!(
                "AXI channel '{}' must map both {} and {}; no implicit ready is used. See 'wavepeek extract axi --help'.",
                channel.name, channel.valid, channel.ready
            )));
        }

        let payload = channel
            .signals
            .iter()
            .filter(|standard| **standard != channel.valid && **standard != channel.ready)
            .filter_map(|standard| mappings.get(*standard).map(|mapping| (*standard, mapping)))
            .collect::<Vec<_>>();
        if !payload.is_empty() && (!valid_mapped || !ready_mapped) {
            return Err(WavepeekError::Args(format!(
                "AXI channel '{}' has mapped payload signals but no complete ready/valid pair. See 'wavepeek extract axi --help'.",
                channel.name
            )));
        }
        if !(valid_mapped && ready_mapped) {
            continue;
        }

        let valid = mappings.get(channel.valid).expect("valid checked");
        let ready = mappings.get(channel.ready).expect("ready checked");
        let on = format!("posedge {}", expr_name(aclk, scope));
        let when = match reset {
            Some(reset) => format!(
                "{} && {} && {}",
                expr_name(reset, scope),
                expr_name(valid, scope),
                expr_name(ready, scope)
            ),
            None => format!("{} && {}", expr_name(valid, scope), expr_name(ready, scope)),
        };
        sources.push(BuiltAxiSource {
            channel: channel.name,
            on,
            when,
            payload_waves: payload
                .iter()
                .map(|(_, mapping)| expr_name(mapping, scope).to_string())
                .collect(),
            payload_standards: payload
                .iter()
                .map(|(standard, _)| (*standard).to_string())
                .collect(),
        });
    }

    if sources.is_empty() {
        return Err(WavepeekError::Args(
            "AXI extraction requires at least one complete ready/valid channel. See 'wavepeek extract axi --help'."
                .to_string(),
        ));
    }

    Ok(sources)
}

fn expr_name<'a>(mapping: &'a AxiSignalMapping, scope: Option<&str>) -> &'a str {
    if scope.is_some() {
        mapping.display.as_str()
    } else {
        mapping.path.as_str()
    }
}

fn ordered_standard_names(profile: AxiProfile) -> Vec<&'static str> {
    COMMON_SIGNALS
        .iter()
        .copied()
        .chain(
            profile
                .channels()
                .iter()
                .flat_map(|channel| channel.signals.iter().copied()),
        )
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
    (0..tokens.len()).find(|start| standard_matches_range(tokens, *start, tokens.len(), standard))
}

fn standard_matches_range(tokens: &[String], start: usize, end: usize, standard: &str) -> bool {
    if COMMON_SIGNALS.contains(&standard) {
        return end == start + 1 && tokens[start] == standard;
    }
    tokens[start..end]
        .iter()
        .flat_map(|token| token.chars())
        .eq(standard.chars())
}

impl AxiProfile {
    fn name(self) -> &'static str {
        self.spec.name
    }

    fn issue(self) -> &'static str {
        self.spec.issue
    }

    fn channels(self) -> &'static [AxiChannelSpec] {
        self.spec.channels
    }

    fn contains_standard(self, standard: &str) -> bool {
        COMMON_SIGNALS.contains(&standard)
            || self
                .channels()
                .iter()
                .any(|channel| channel.signals.contains(&standard))
    }
}

#[cfg(test)]
mod tests {
    use super::{candidate_matches_standard, parse_cli_maps, parse_profile, profile_specs};

    fn assert_profile(name: &str, expected: &[(&str, &[&str])]) {
        let profile = parse_profile(name).unwrap();
        let actual = profile
            .channels()
            .iter()
            .map(|channel| {
                let expected_valid = format!("{}valid", channel.name);
                let expected_ready = format!("{}ready", channel.name);
                assert_eq!(channel.valid, expected_valid);
                assert_eq!(channel.ready, expected_ready);
                (channel.name, channel.signals)
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn profile_names_are_case_insensitive() {
        assert_eq!(parse_profile("AXI4").unwrap().name(), "axi4");
        assert_eq!(parse_profile("axi4_lite").unwrap().name(), "axi4-lite");
        assert_eq!(parse_profile("ACE").unwrap().name(), "ace");
        assert_eq!(parse_profile("ACE_LITE").unwrap().name(), "ace-lite");
        assert_eq!(parse_profile("ace_lite").unwrap().name(), "ace-lite");
        assert_eq!(parse_profile("AXI5").unwrap().name(), "axi5");
        assert_eq!(parse_profile("AXI5_LITE").unwrap().name(), "axi5-lite");
        assert_eq!(parse_profile("axi5_lite").unwrap().name(), "axi5-lite");
        assert_eq!(parse_profile("ACE5").unwrap().name(), "ace5");
        assert_eq!(parse_profile("ACE5_LITE").unwrap().name(), "ace5-lite");
        for alias in [
            "ace5-lite-dvm",
            "ACE5-LITEDVM",
            "ace5_litedvm",
            "ACE5_LITE_DVM",
        ] {
            assert_eq!(parse_profile(alias).unwrap().name(), "ace5-lite-dvm");
        }
        for alias in [
            "ace5-lite-acp",
            "ACE5-LITEACP",
            "ace5_liteacp",
            "ACE5_LITE_ACP",
        ] {
            assert_eq!(parse_profile(alias).unwrap().name(), "ace5-lite-acp");
        }
        for unsupported in [
            "ace5_lite-dvm",
            "ace5-lite_dvm",
            "ace5_lite-acp",
            "ace5-lite_acp",
        ] {
            assert!(
                parse_profile(unsupported).is_err(),
                "accepted {unsupported}"
            );
        }
    }

    #[test]
    fn ace_family_profile_specs_match_contract() {
        assert_eq!(
            profile_specs()
                .iter()
                .map(|profile| profile.name)
                .collect::<Vec<_>>(),
            [
                "axi3",
                "axi4",
                "axi4-lite",
                "axi5",
                "axi5-lite",
                "ace",
                "ace-lite",
                "ace5",
                "ace5-lite",
                "ace5-lite-dvm",
                "ace5-lite-acp"
            ]
        );

        assert_profile(
            "ace",
            &[
                (
                    "aw",
                    &[
                        "awid", "awaddr", "awlen", "awsize", "awburst", "awlock", "awcache",
                        "awprot", "awqos", "awregion", "awuser", "awdomain", "awsnoop", "awbar",
                        "awunique", "awvalid", "awready",
                    ],
                ),
                (
                    "w",
                    &["wdata", "wstrb", "wlast", "wuser", "wvalid", "wready"],
                ),
                ("b", &["bid", "bresp", "buser", "bvalid", "bready"]),
                (
                    "ar",
                    &[
                        "arid", "araddr", "arlen", "arsize", "arburst", "arlock", "arcache",
                        "arprot", "arqos", "arregion", "aruser", "ardomain", "arsnoop", "arbar",
                        "arvalid", "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid", "rdata", "rresp", "rlast", "ruser", "rvalid", "rready",
                    ],
                ),
                ("ac", &["acaddr", "acsnoop", "acprot", "acvalid", "acready"]),
                ("cr", &["crresp", "crvalid", "crready"]),
                ("cd", &["cddata", "cdlast", "cdvalid", "cdready"]),
            ],
        );

        assert_profile(
            "ace-lite",
            &[
                (
                    "aw",
                    &[
                        "awid", "awaddr", "awlen", "awsize", "awburst", "awlock", "awcache",
                        "awprot", "awqos", "awregion", "awuser", "awdomain", "awsnoop", "awbar",
                        "awunique", "awvalid", "awready",
                    ],
                ),
                (
                    "w",
                    &["wdata", "wstrb", "wlast", "wuser", "wvalid", "wready"],
                ),
                ("b", &["bid", "bresp", "buser", "bvalid", "bready"]),
                (
                    "ar",
                    &[
                        "arid", "araddr", "arlen", "arsize", "arburst", "arlock", "arcache",
                        "arprot", "arqos", "arregion", "aruser", "ardomain", "arsnoop", "arbar",
                        "arvalid", "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid", "rdata", "rresp", "rlast", "ruser", "rvalid", "rready",
                    ],
                ),
            ],
        );

        assert_profile(
            "ace5",
            &[
                (
                    "aw",
                    &[
                        "awid",
                        "awaddr",
                        "awlen",
                        "awsize",
                        "awburst",
                        "awlock",
                        "awcache",
                        "awprot",
                        "awqos",
                        "awregion",
                        "awuser",
                        "awdomain",
                        "awsnoop",
                        "awunique",
                        "awtrace",
                        "awloop",
                        "awmmusecsid",
                        "awmmusid",
                        "awmmussidv",
                        "awmmussid",
                        "awmmuatst",
                        "awnsaid",
                        "awmpam",
                        "awidunq",
                        "awvalid",
                        "awready",
                    ],
                ),
                (
                    "w",
                    &[
                        "wdata", "wstrb", "wlast", "wuser", "wpoison", "wtrace", "wvalid", "wready",
                    ],
                ),
                (
                    "b",
                    &[
                        "bid", "bresp", "buser", "btrace", "bloop", "bidunq", "bvalid", "bready",
                    ],
                ),
                (
                    "ar",
                    &[
                        "arid",
                        "araddr",
                        "arlen",
                        "arsize",
                        "arburst",
                        "arlock",
                        "arcache",
                        "arprot",
                        "arqos",
                        "arregion",
                        "aruser",
                        "ardomain",
                        "arsnoop",
                        "arvmidext",
                        "artrace",
                        "arloop",
                        "armmusecsid",
                        "armmusid",
                        "armmussidv",
                        "armmussid",
                        "armmuatst",
                        "arnsaid",
                        "armpam",
                        "aridunq",
                        "arvalid",
                        "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid", "rdata", "rresp", "rlast", "ruser", "rpoison", "rtrace", "rloop",
                        "ridunq", "rvalid", "rready",
                    ],
                ),
                (
                    "ac",
                    &[
                        "acaddr",
                        "acsnoop",
                        "acprot",
                        "acvmidext",
                        "actrace",
                        "acvalid",
                        "acready",
                    ],
                ),
                (
                    "cr",
                    &["crresp", "crtrace", "crnsaid", "crvalid", "crready"],
                ),
                (
                    "cd",
                    &[
                        "cddata", "cdlast", "cdpoison", "cdtrace", "cdvalid", "cdready",
                    ],
                ),
            ],
        );
    }

    #[test]
    fn axi5_profile_specs_match_issue_l_contract() {
        assert_profile(
            "axi5",
            &[
                (
                    "aw",
                    &[
                        "awid",
                        "awaddr",
                        "awregion",
                        "awlen",
                        "awsize",
                        "awburst",
                        "awlock",
                        "awcache",
                        "awprot",
                        "awnse",
                        "awpas",
                        "awinst",
                        "awpriv",
                        "awqos",
                        "awuser",
                        "awdomain",
                        "awsnoop",
                        "awstashnid",
                        "awstashniden",
                        "awstashlpid",
                        "awstashlpiden",
                        "awtrace",
                        "awloop",
                        "awmmuvalid",
                        "awmmusecsid",
                        "awmmusid",
                        "awmmussidv",
                        "awmmussid",
                        "awmmuatst",
                        "awmmuflow",
                        "awmmupasunknown",
                        "awmmupm",
                        "awpbha",
                        "awmecid",
                        "awnsaid",
                        "awsubsysid",
                        "awatop",
                        "awmpam",
                        "awidunq",
                        "awcmo",
                        "awtagop",
                        "awact",
                        "awactv",
                        "awvalid",
                        "awready",
                    ],
                ),
                (
                    "w",
                    &[
                        "wdata",
                        "wstrb",
                        "wtag",
                        "wtagupdate",
                        "wlast",
                        "wuser",
                        "wpoison",
                        "wtrace",
                        "wvalid",
                        "wready",
                    ],
                ),
                (
                    "b",
                    &[
                        "bid",
                        "bidunq",
                        "bresp",
                        "bcomp",
                        "bpersist",
                        "btagmatch",
                        "buser",
                        "btrace",
                        "bloop",
                        "bbusy",
                        "bvalid",
                        "bready",
                    ],
                ),
                (
                    "ar",
                    &[
                        "arid",
                        "araddr",
                        "arregion",
                        "arlen",
                        "arsize",
                        "arburst",
                        "arlock",
                        "arcache",
                        "arprot",
                        "arnse",
                        "arpas",
                        "arinst",
                        "arpriv",
                        "arqos",
                        "aruser",
                        "ardomain",
                        "arsnoop",
                        "artrace",
                        "arloop",
                        "armmuvalid",
                        "armmusecsid",
                        "armmusid",
                        "armmussidv",
                        "armmussid",
                        "armmuatst",
                        "armmuflow",
                        "armmupasunknown",
                        "armmupm",
                        "arpbha",
                        "armecid",
                        "arnsaid",
                        "arsubsysid",
                        "armpam",
                        "archunken",
                        "aridunq",
                        "artagop",
                        "aract",
                        "aractv",
                        "arvalid",
                        "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid",
                        "ridunq",
                        "rdata",
                        "rtag",
                        "rresp",
                        "rlast",
                        "ruser",
                        "rpoison",
                        "rtrace",
                        "rloop",
                        "rchunkv",
                        "rchunknum",
                        "rchunkstrb",
                        "rbusy",
                        "rvalid",
                        "rready",
                    ],
                ),
                (
                    "ac",
                    &["acaddr", "acvmidext", "actrace", "acvalid", "acready"],
                ),
                ("cr", &["crtrace", "crvalid", "crready"]),
            ],
        );

        assert_profile(
            "axi5-lite",
            &[
                (
                    "aw",
                    &[
                        "awid",
                        "awaddr",
                        "awsize",
                        "awprot",
                        "awuser",
                        "awtrace",
                        "awsubsysid",
                        "awidunq",
                        "awvalid",
                        "awready",
                    ],
                ),
                (
                    "w",
                    &[
                        "wdata", "wstrb", "wuser", "wpoison", "wtrace", "wvalid", "wready",
                    ],
                ),
                (
                    "b",
                    &[
                        "bid", "bidunq", "bresp", "buser", "btrace", "bvalid", "bready",
                    ],
                ),
                (
                    "ar",
                    &[
                        "arid",
                        "araddr",
                        "arsize",
                        "arprot",
                        "aruser",
                        "artrace",
                        "arsubsysid",
                        "aridunq",
                        "arvalid",
                        "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid", "ridunq", "rdata", "rresp", "ruser", "rpoison", "rtrace", "rvalid",
                        "rready",
                    ],
                ),
            ],
        );
    }

    #[test]
    fn ace5_lite_family_profile_specs_match_issue_l_contract() {
        assert_profile(
            "ace5-lite",
            &[
                (
                    "aw",
                    &[
                        "awid",
                        "awaddr",
                        "awregion",
                        "awlen",
                        "awsize",
                        "awburst",
                        "awlock",
                        "awcache",
                        "awprot",
                        "awnse",
                        "awqos",
                        "awuser",
                        "awdomain",
                        "awsnoop",
                        "awstashnid",
                        "awstashniden",
                        "awstashlpid",
                        "awstashlpiden",
                        "awtrace",
                        "awloop",
                        "awmmuvalid",
                        "awmmusecsid",
                        "awmmusid",
                        "awmmussidv",
                        "awmmussid",
                        "awmmuatst",
                        "awmmuflow",
                        "awpbha",
                        "awmecid",
                        "awnsaid",
                        "awsubsysid",
                        "awatop",
                        "awmpam",
                        "awidunq",
                        "awcmo",
                        "awtagop",
                        "awvalid",
                        "awready",
                    ],
                ),
                (
                    "w",
                    &[
                        "wdata",
                        "wstrb",
                        "wtag",
                        "wtagupdate",
                        "wlast",
                        "wuser",
                        "wpoison",
                        "wtrace",
                        "wvalid",
                        "wready",
                    ],
                ),
                (
                    "b",
                    &[
                        "bid",
                        "bidunq",
                        "bresp",
                        "bcomp",
                        "bpersist",
                        "btagmatch",
                        "buser",
                        "btrace",
                        "bloop",
                        "bbusy",
                        "bvalid",
                        "bready",
                    ],
                ),
                (
                    "ar",
                    &[
                        "arid",
                        "araddr",
                        "arregion",
                        "arlen",
                        "arsize",
                        "arburst",
                        "arlock",
                        "arcache",
                        "arprot",
                        "arnse",
                        "arqos",
                        "aruser",
                        "ardomain",
                        "arsnoop",
                        "artrace",
                        "arloop",
                        "armmuvalid",
                        "armmusecsid",
                        "armmusid",
                        "armmussidv",
                        "armmussid",
                        "armmuatst",
                        "armmuflow",
                        "arpbha",
                        "armecid",
                        "arnsaid",
                        "arsubsysid",
                        "armpam",
                        "archunken",
                        "aridunq",
                        "artagop",
                        "arvalid",
                        "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid",
                        "ridunq",
                        "rdata",
                        "rtag",
                        "rresp",
                        "rlast",
                        "ruser",
                        "rpoison",
                        "rtrace",
                        "rloop",
                        "rchunkv",
                        "rchunknum",
                        "rchunkstrb",
                        "rbusy",
                        "rvalid",
                        "rready",
                    ],
                ),
            ],
        );

        assert_profile(
            "ace5-lite-dvm",
            &[
                (
                    "aw",
                    &[
                        "awid",
                        "awaddr",
                        "awregion",
                        "awlen",
                        "awsize",
                        "awburst",
                        "awlock",
                        "awcache",
                        "awprot",
                        "awnse",
                        "awqos",
                        "awuser",
                        "awdomain",
                        "awsnoop",
                        "awstashnid",
                        "awstashniden",
                        "awstashlpid",
                        "awstashlpiden",
                        "awtrace",
                        "awloop",
                        "awpbha",
                        "awmecid",
                        "awnsaid",
                        "awsubsysid",
                        "awatop",
                        "awmpam",
                        "awidunq",
                        "awcmo",
                        "awtagop",
                        "awvalid",
                        "awready",
                    ],
                ),
                (
                    "w",
                    &[
                        "wdata",
                        "wstrb",
                        "wtag",
                        "wtagupdate",
                        "wlast",
                        "wuser",
                        "wpoison",
                        "wtrace",
                        "wvalid",
                        "wready",
                    ],
                ),
                (
                    "b",
                    &[
                        "bid", "bidunq", "bresp", "bcomp", "bpersist", "buser", "btrace", "bloop",
                        "bbusy", "bvalid", "bready",
                    ],
                ),
                (
                    "ar",
                    &[
                        "arid",
                        "araddr",
                        "arregion",
                        "arlen",
                        "arsize",
                        "arburst",
                        "arlock",
                        "arcache",
                        "arprot",
                        "arnse",
                        "arqos",
                        "aruser",
                        "ardomain",
                        "arsnoop",
                        "artrace",
                        "arloop",
                        "arpbha",
                        "armecid",
                        "arnsaid",
                        "arsubsysid",
                        "armpam",
                        "archunken",
                        "aridunq",
                        "artagop",
                        "arvalid",
                        "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid",
                        "ridunq",
                        "rdata",
                        "rtag",
                        "rresp",
                        "rlast",
                        "ruser",
                        "rpoison",
                        "rtrace",
                        "rloop",
                        "rchunkv",
                        "rchunknum",
                        "rchunkstrb",
                        "rbusy",
                        "rvalid",
                        "rready",
                    ],
                ),
                (
                    "ac",
                    &["acaddr", "acvmidext", "actrace", "acvalid", "acready"],
                ),
                ("cr", &["crtrace", "crvalid", "crready"]),
            ],
        );

        assert_profile(
            "ace5-lite-acp",
            &[
                (
                    "aw",
                    &[
                        "awid",
                        "awaddr",
                        "awlen",
                        "awcache",
                        "awprot",
                        "awuser",
                        "awdomain",
                        "awsnoop",
                        "awstashnid",
                        "awstashniden",
                        "awstashlpid",
                        "awstashlpiden",
                        "awtrace",
                        "awmpam",
                        "awidunq",
                        "awvalid",
                        "awready",
                    ],
                ),
                (
                    "w",
                    &[
                        "wdata", "wstrb", "wlast", "wuser", "wpoison", "wtrace", "wvalid", "wready",
                    ],
                ),
                (
                    "b",
                    &[
                        "bid", "bidunq", "bresp", "buser", "btrace", "bvalid", "bready",
                    ],
                ),
                (
                    "ar",
                    &[
                        "arid",
                        "araddr",
                        "arlen",
                        "arcache",
                        "arprot",
                        "aruser",
                        "ardomain",
                        "arsnoop",
                        "artrace",
                        "armpam",
                        "archunken",
                        "aridunq",
                        "arvalid",
                        "arready",
                    ],
                ),
                (
                    "r",
                    &[
                        "rid",
                        "ridunq",
                        "rdata",
                        "rresp",
                        "rlast",
                        "ruser",
                        "rpoison",
                        "rtrace",
                        "rchunkv",
                        "rchunknum",
                        "rchunkstrb",
                        "rvalid",
                        "rready",
                    ],
                ),
            ],
        );
    }

    #[test]
    fn ace5_lite_profiles_reject_out_of_profile_names() {
        let lite = parse_profile("ace5-lite").unwrap();
        for standard in ["awactv", "awpending", "awvalidchk", "acvalid", "cdvalid"] {
            assert!(
                !lite.contains_standard(standard),
                "ACE5-Lite accepted {standard}"
            );
        }

        let dvm = parse_profile("ace5-lite-dvm").unwrap();
        assert!(dvm.contains_standard("artagop"));
        for standard in [
            "awmmuvalid",
            "armmuvalid",
            "btagmatch",
            "acsnoop",
            "acprot",
            "crresp",
            "cdvalid",
        ] {
            assert!(
                !dvm.contains_standard(standard),
                "ACE5-LiteDVM accepted {standard}"
            );
        }

        let acp = parse_profile("ace5-lite-acp").unwrap();
        for standard in [
            "awsize", "awburst", "wtag", "bcomp", "arsize", "rtag", "acvalid",
        ] {
            assert!(
                !acp.contains_standard(standard),
                "ACE5-LiteACP accepted {standard}"
            );
        }
    }

    #[test]
    fn axi5_profiles_reject_nonfunctional_and_cross_profile_names() {
        let axi5 = parse_profile("axi5").unwrap();
        for standard in [
            "awbar",
            "awunique",
            "arbar",
            "cdvalid",
            "awpending",
            "awakeup",
            "varqosaccept",
            "syscoreq",
            "broadcastatomic",
            "activatereq",
        ] {
            assert!(
                !axi5.contains_standard(standard),
                "AXI5 must reject {standard}"
            );
        }

        let axi5_lite = parse_profile("axi5-lite").unwrap();
        for standard in [
            "awlen",
            "awburst",
            "awcache",
            "wlast",
            "rlast",
            "arsnoop",
            "acvalid",
            "awpending",
        ] {
            assert!(
                !axi5_lite.contains_standard(standard),
                "AXI5-Lite must reject {standard}"
            );
        }
    }

    #[test]
    fn ace5_rejects_removed_barrier_standard_names() {
        let profile = parse_profile("ace5").unwrap();
        assert!(!profile.contains_standard("awbar"));
        assert!(!profile.contains_standard("arbar"));
    }

    #[test]
    fn map_parser_normalizes_standard_side() {
        let maps = parse_cli_maps(&["AWVALID=axi_awvalid_o".to_string()]).unwrap();
        assert_eq!(
            maps,
            vec![("awvalid".to_string(), "axi_awvalid_o".to_string())]
        );
        assert!(parse_cli_maps(&["awvalid".to_string()]).is_err());
        assert!(parse_cli_maps(&["awvalid=a".to_string(), "AWVALID=b".to_string()]).is_err());
    }

    #[test]
    fn token_matching_avoids_channel_prefix_collisions() {
        assert!(candidate_matches_standard("awvalid", "awvalid"));
        assert!(candidate_matches_standard("aw_valid", "awvalid"));
        assert!(candidate_matches_standard("axi_awvalid_o", "awvalid"));
        assert!(candidate_matches_standard("axi_aw_valid_o", "awvalid"));
        assert!(candidate_matches_standard("axi_aw_prot_o_r", "awprot"));
        assert!(candidate_matches_standard("ace_acvalid_o", "acvalid"));
        assert!(candidate_matches_standard("ace_ac_valid_o", "acvalid"));
        assert!(candidate_matches_standard("ace_cr_ready_i", "crready"));
        assert!(candidate_matches_standard("ace_cd_data_o", "cddata"));
        assert!(candidate_matches_standard(
            "axi5_aw_mmu_valid_o",
            "awmmuvalid"
        ));
        assert!(candidate_matches_standard("axi5_aw_actv_o", "awactv"));
        assert!(candidate_matches_standard("axi5_ar_mecid_o", "armecid"));
        assert!(candidate_matches_standard("axi5_r_chunknum_i", "rchunknum"));
        assert!(candidate_matches_standard("axi5_ac_addr_i", "acaddr"));
        assert!(candidate_matches_standard("aclk", "aclk"));
        assert!(!candidate_matches_standard("wvalid", "awvalid"));
        assert!(!candidate_matches_standard("axi_wvalid_o", "awvalid"));
        assert!(!candidate_matches_standard("rready", "arready"));
        assert!(!candidate_matches_standard("ace_ac_valid_o", "awvalid"));
        assert!(!candidate_matches_standard("ace_ac_valid_o", "aclk"));
        assert!(!candidate_matches_standard("ar_esetn", "aresetn"));
    }

    #[test]
    fn check_signal_suffixes_do_not_match_functional_names() {
        for (candidate, functional) in [
            ("ace5_ac_valid_chk_i", "acvalid"),
            ("ace5_ac_ready_chk_o", "acready"),
            ("ace5_cr_valid_chk_o", "crvalid"),
            ("ace5_cr_ready_chk_i", "crready"),
            ("ace5_cd_valid_chk_o", "cdvalid"),
            ("ace5_cd_ready_chk_i", "cdready"),
            ("ace5_cd_data_chk_o", "cddata"),
            ("ace5_acvalidchk_i", "acvalid"),
            ("axi5_aw_valid_chk_o", "awvalid"),
        ] {
            assert!(
                !candidate_matches_standard(candidate, functional),
                "{candidate} must not match functional {functional}"
            );
        }
    }

    #[test]
    fn split_unique_id_names_match_only_complete_standard_names() {
        for (candidate, complete, shorter) in [
            ("ace5_aw_id_unq_o", "awidunq", "awid"),
            ("ace5_ar_id_unq_o", "aridunq", "arid"),
            ("ace5_b_id_unq_i", "bidunq", "bid"),
            ("ace5_r_id_unq_i", "ridunq", "rid"),
        ] {
            assert!(
                candidate_matches_standard(candidate, complete),
                "{candidate} must match {complete}"
            );
            assert!(
                !candidate_matches_standard(candidate, shorter),
                "{candidate} must not be truncated to {shorter}"
            );
        }
    }
}
