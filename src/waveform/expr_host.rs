use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::error::WavepeekError;
use crate::expr::{
    DiagnosticLayer, ExprDiagnostic, ExprType, ExpressionHost, SampledValue, SignalHandle, Span,
};
use crate::waveform::{ExprResolvedSignal, Waveform};

#[derive(Debug)]
pub(crate) struct WaveformExprHost {
    waveform: RefCell<Waveform>,
    handles_by_name: RefCell<HashMap<String, SignalHandle>>,
    signals_by_handle: RefCell<HashMap<SignalHandle, Rc<ExprResolvedSignal>>>,
    next_handle: Cell<u32>,
}

impl WaveformExprHost {
    pub(crate) fn open(path: &Path) -> Result<Self, WavepeekError> {
        let waveform = Waveform::open(path)?;
        Ok(Self::new(waveform))
    }

    pub(crate) fn new(waveform: Waveform) -> Self {
        Self {
            waveform: RefCell::new(waveform),
            handles_by_name: RefCell::new(HashMap::new()),
            signals_by_handle: RefCell::new(HashMap::new()),
            next_handle: Cell::new(1),
        }
    }

    fn resolved_signal(
        &self,
        handle: SignalHandle,
    ) -> Result<Rc<ExprResolvedSignal>, ExprDiagnostic> {
        self.signals_by_handle
            .borrow()
            .get(&handle)
            .cloned()
            .ok_or_else(|| ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-SIGNAL",
                message: format!("unknown signal handle {}", handle.0),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })
    }
}

impl ExpressionHost for WaveformExprHost {
    fn resolve_signal(&self, name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        if let Some(handle) = self.handles_by_name.borrow().get(name).copied() {
            return Ok(handle);
        }

        let resolved = self
            .waveform
            .borrow()
            .resolve_expr_signal(name)
            .map_err(|error| ExprDiagnostic {
                layer: DiagnosticLayer::Semantic,
                code: "HOST-UNKNOWN-SIGNAL",
                message: error.to_string(),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })?;

        let handle = SignalHandle(self.next_handle.get());
        self.next_handle.set(handle.0 + 1);
        self.handles_by_name
            .borrow_mut()
            .insert(name.to_string(), handle);
        self.signals_by_handle
            .borrow_mut()
            .insert(handle, Rc::new(resolved));
        Ok(handle)
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        Ok(self.resolved_signal(handle)?.expr_type.clone())
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        let resolved = self.resolved_signal(handle)?;
        self.waveform
            .borrow_mut()
            .sample_expr_value(&resolved, timestamp)
            .map_err(|error| ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "HOST-SAMPLE-ERROR",
                message: error.to_string(),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        let resolved = self.resolved_signal(handle)?;
        self.waveform
            .borrow_mut()
            .expr_event_occurred(&resolved, timestamp)
            .map_err(|error| ExprDiagnostic {
                layer: DiagnosticLayer::Runtime,
                code: "HOST-EVENT-ERROR",
                message: error.to_string(),
                primary_span: Span::new(0, 0),
                notes: vec![],
            })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::NamedTempFile;

    use super::WaveformExprHost;
    use crate::expr::{
        ExprValuePayload, bind_logical_expr_ast, eval_logical_expr_at, parse_logical_expr_ast,
    };

    const RICH_VCD: &str = "$date\n  today\n$end\n$version\n  wavepeek-c4\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! clk $end\n$var event 1 \" ev $end\n$var real 1 # temp $end\n$var string 1 $ msg $end\n$var enum 2 % state $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\nr0.5 #\nsidle $\nb00 %\n#10\n1!\n1\"\nr1.5 #\nsgo $\nb01 %\n#20\n0!\nr1.5 #\nshold $\nb10 %\n";

    #[test]
    fn waveform_expr_host_supports_rich_vcd_values() {
        let fixture = write_fixture(RICH_VCD, "rich-c4.vcd");
        let host = WaveformExprHost::open(fixture.path()).expect("fixture should open");
        let ast =
            parse_logical_expr_ast("top.ev.triggered && (top.temp > 1.0) && (top.msg == \"go\")")
                .expect("expression should parse");
        let bound = bind_logical_expr_ast(&ast, &host).expect("expression should bind");
        let value = eval_logical_expr_at(&bound, &host, 10).expect("expression should evaluate");

        assert!(matches!(
            value.payload,
            ExprValuePayload::Integral { ref bits, .. } if bits == "1"
        ));
    }

    #[test]
    fn waveform_expr_host_supports_recovered_bit_vector_cast_on_vcd_and_fst() {
        for filename in ["m2_core.vcd", "m2_core.fst"] {
            let host =
                WaveformExprHost::open(&fixture_path(filename)).expect("fixture should open");
            let ast =
                parse_logical_expr_ast("type(top.data)'(3)").expect("expression should parse");
            let bound = bind_logical_expr_ast(&ast, &host).expect("expression should bind");
            let value = eval_logical_expr_at(&bound, &host, 0).expect("expression should evaluate");
            assert!(matches!(
                value.payload,
                ExprValuePayload::Integral { ref bits, .. } if bits == "00000011"
            ));
        }
    }

    #[test]
    fn waveform_expr_host_reports_missing_enum_metadata_on_vcd() {
        let fixture = write_fixture(RICH_VCD, "rich-c4.vcd");
        let host = WaveformExprHost::open(fixture.path()).expect("fixture should open");
        let ast = parse_logical_expr_ast("type(top.state)::BUSY").expect("expression should parse");
        let error = bind_logical_expr_ast(&ast, &host).expect_err("bind should fail");

        assert_eq!(error.code, "C4-SEMANTIC-METADATA");
    }

    fn fixture_path(filename: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("hand")
            .join(filename)
    }

    fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
        let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
        fs::write(fixture.path(), contents).expect("fixture should write");
        fixture
    }
}
