//! Expression values borrowed from one Ondas query tick.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{ControlFlow, RangeInclusive};
use std::path::Path;

use ondas::{QueryContext, Sample, SampleRef, Selection, Time, TimeRange};

use crate::error::WavepeekError;
use crate::expr::{
    DiagnosticLayer, ExprDiagnostic, ExprType, ExpressionHost, SampledValue, SignalHandle, Span,
};

use super::ExprResolvedSignal;
use super::expr_host::{RuntimeExpressionHost, WaveformExprHost};
use super::ondas_backend::expression_sample;
use super::ondas_error::read_error;

pub(crate) enum ExpressionQueryTick {
    Initial(u64),
    Candidate(u64),
}

/// Ordered runtime operands and driver positions, frozen after binding.
pub(crate) struct ExpressionQuery {
    pub(super) signals: Vec<ExprResolvedSignal>,
    indices: HashMap<SignalHandle, usize>,
    drivers: Vec<usize>,
}

impl ExpressionQuery {
    pub(crate) fn new(
        host: &WaveformExprHost<'_>,
        drivers: &[SignalHandle],
        operands: &[SignalHandle],
    ) -> Result<Self, WavepeekError> {
        let mut query = Self {
            signals: Vec::new(),
            indices: HashMap::new(),
            drivers: Vec::new(),
        };
        for &handle in drivers.iter().chain(operands) {
            if !query.indices.contains_key(&handle) {
                let signal = host.resolved_signal_for_handle(handle)?;
                query.indices.insert(handle, query.signals.len());
                query.signals.push(signal);
            }
        }
        query.drivers = drivers.iter().map(|handle| query.indices[handle]).collect();
        Ok(query)
    }

    pub(super) fn run(
        &self,
        selection: &mut Selection<'_>,
        range: RangeInclusive<u64>,
        initialize: bool,
        path: &Path,
        mut visitor: impl FnMut(
            &dyn RuntimeExpressionHost,
            ExpressionQueryTick,
        ) -> Result<ControlFlow<()>, WavepeekError>,
    ) -> Result<(), WavepeekError> {
        if initialize {
            let time = *range.start();
            let samples = selection
                .samples(Time::from_ticks(time))
                .map_err(|error| read_error(path, "read samples from", error))?;
            let host = QueryExprHost::new(self, QuerySamples::Initial(time, &samples), path);
            if visitor(&host, ExpressionQueryTick::Initial(time))?.is_break() {
                return Ok(());
            }
        }
        let outcome = selection
            .query(
                TimeRange::closed(
                    Time::from_ticks(*range.start()),
                    Time::from_ticks(*range.end()),
                ),
                &self.drivers,
                |context| {
                    let host = QueryExprHost::new(self, QuerySamples::Candidate(context), path);
                    // Application failures stay typed in the Break payload.
                    Ok(
                        match visitor(
                            &host,
                            ExpressionQueryTick::Candidate(context.time().ticks()),
                        ) {
                            Ok(ControlFlow::Continue(())) => ControlFlow::Continue(()),
                            result => ControlFlow::Break(result.map(|_| ())),
                        },
                    )
                },
            )
            .map_err(|error| read_error(path, "query samples from", error))?;
        match outcome {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(result) => result,
        }
    }
}

enum QuerySamples<'a> {
    Initial(u64, &'a [Sample]),
    Candidate(&'a QueryContext<'a>),
}

#[derive(Clone)]
enum ExpressionSample {
    Value(SampledValue),
    Event(bool),
}

struct QueryExprHost<'a> {
    query: &'a ExpressionQuery,
    samples: QuerySamples<'a>,
    path: &'a Path,
    cache: RefCell<HashMap<(usize, u64), ExpressionSample>>,
    read_error: RefCell<Option<WavepeekError>>,
}

impl<'a> QueryExprHost<'a> {
    fn new(query: &'a ExpressionQuery, samples: QuerySamples<'a>, path: &'a Path) -> Self {
        Self {
            query,
            samples,
            path,
            cache: RefCell::new(HashMap::new()),
            read_error: RefCell::new(None),
        }
    }

    fn index(&self, handle: SignalHandle) -> Result<usize, ExprDiagnostic> {
        self.query.indices.get(&handle).copied().ok_or_else(|| {
            diagnostic(
                "HOST-UNKNOWN-SIGNAL",
                format!("unknown signal handle {}", handle.0),
            )
        })
    }

    fn sample(&self, index: usize, timestamp: u64) -> Result<ExpressionSample, WavepeekError> {
        if let Some(sample) = self.cache.borrow().get(&(index, timestamp)) {
            return Ok(sample.clone());
        }
        let convert = |sample: SampleRef<'_>| match sample {
            SampleRef::Event { occurrences, .. } => Ok(ExpressionSample::Event(occurrences > 0)),
            sample => {
                expression_sample(&self.query.signals[index], sample).map(ExpressionSample::Value)
            }
        };
        let sample = match self.samples {
            QuerySamples::Initial(time, samples) => {
                if timestamp != time {
                    return Err(WavepeekError::Internal(
                        "initial expression values are available only at the window start".into(),
                    ));
                }
                convert(samples[index].as_ref())?
            }
            QuerySamples::Candidate(context) => {
                let mut sample = None;
                let outcome =
                    context
                        .visit_samples(Time::from_ticks(timestamp), &[index], |_, value| {
                            match convert(value) {
                                Ok(value) => {
                                    sample = Some(value);
                                    Ok(ControlFlow::Continue(()))
                                }
                                Err(error) => Ok(ControlFlow::Break(error)),
                            }
                        })
                        .map_err(|error| read_error(self.path, "read query samples from", error))?;
                if let ControlFlow::Break(error) = outcome {
                    return Err(error);
                }
                sample.ok_or_else(|| {
                    WavepeekError::Internal("query returned no expression sample".into())
                })?
            }
        };
        self.cache
            .borrow_mut()
            .insert((index, timestamp), sample.clone());
        Ok(sample)
    }

    fn read_diagnostic(&self, error: WavepeekError) -> ExprDiagnostic {
        let diagnostic = diagnostic("HOST-SAMPLE-ERROR", error.to_string());
        self.read_error.replace(Some(error));
        diagnostic
    }
}

impl RuntimeExpressionHost for QueryExprHost<'_> {
    fn take_waveform_error(&self) -> Option<WavepeekError> {
        self.read_error.take()
    }
}

impl ExpressionHost for QueryExprHost<'_> {
    fn resolve_signal(&self, _name: &str) -> Result<SignalHandle, ExprDiagnostic> {
        Err(diagnostic(
            "HOST-QUERY-BINDING",
            "query operands must be bound before reading".into(),
        ))
    }

    fn signal_type(&self, handle: SignalHandle) -> Result<ExprType, ExprDiagnostic> {
        Ok(self.query.signals[self.index(handle)?].expr_type.clone())
    }

    fn sample_value(
        &self,
        handle: SignalHandle,
        timestamp: u64,
    ) -> Result<SampledValue, ExprDiagnostic> {
        let index = self.index(handle)?;
        match self
            .sample(index, timestamp)
            .map_err(|error| self.read_diagnostic(error))?
        {
            ExpressionSample::Value(value) => Ok(value),
            ExpressionSample::Event(_) => Err(diagnostic(
                "HOST-TYPE-MISMATCH",
                "raw events cannot be sampled as values".into(),
            )),
        }
    }

    fn event_occurred(&self, handle: SignalHandle, timestamp: u64) -> Result<bool, ExprDiagnostic> {
        let index = self.index(handle)?;
        match self
            .sample(index, timestamp)
            .map_err(|error| self.read_diagnostic(error))?
        {
            ExpressionSample::Event(value) => Ok(value),
            ExpressionSample::Value(_) => Err(diagnostic(
                "HOST-TYPE-MISMATCH",
                "expected a raw event operand".into(),
            )),
        }
    }
}

fn diagnostic(code: &'static str, message: String) -> ExprDiagnostic {
    ExprDiagnostic {
        layer: DiagnosticLayer::Runtime,
        code,
        message,
        primary_span: Span::new(0, 0),
        notes: vec![],
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ops::ControlFlow;
    use std::rc::Rc;

    use tempfile::NamedTempFile;

    use super::{ExpressionQuery, ExpressionQueryTick};
    use crate::error::WavepeekError;
    use crate::expr::{ExprTypeKind, ExpressionHost, SampledValue, SignalHandle};
    use crate::waveform::Waveform;
    use crate::waveform::expr_host::WaveformExprHost;

    fn fixture() -> (
        NamedTempFile,
        Rc<RefCell<Waveform<'static>>>,
        WaveformExprHost<'static>,
    ) {
        let file = NamedTempFile::with_suffix(".vcd").unwrap();
        std::fs::write(
            file.path(),
            concat!(
                "$timescale 1ns $end\n$scope module top $end\n",
                "$var wire 1 c clk $end\n$var wire 1 c alias $end\n",
                "$var event 1 e event_value $end\n$var real 1 r real_value $end\n",
                "$var string 1 s string_value $end\n$upscope $end\n$enddefinitions $end\n",
                "#0\n0c\n1e\nr0.5 r\nsok s\n#4\n1e\n#5\n1c\n",
                "#8\n1e\n#10\n0c\n#11\n1e\n#15\n1c\n",
            ),
        )
        .unwrap();
        let wave = Rc::new(RefCell::new(Waveform::open(file.path()).unwrap()));
        let host = WaveformExprHost::from_shared(wave.clone());
        (file, wave, host)
    }

    #[test]
    fn query_samples_preserve_alias_slots_and_absolute_previous_events() {
        let (_file, wave, binding) = fixture();
        let clk = binding.resolve_signal("top.clk").unwrap();
        let alias = binding.resolve_signal("top.alias").unwrap();
        let event = binding.resolve_signal("top.event_value").unwrap();
        let real = binding.resolve_signal("top.real_value").unwrap();
        let string = binding.resolve_signal("top.string_value").unwrap();
        let query = ExpressionQuery::new(&binding, &[clk], &[alias, event, real, string]).unwrap();
        assert_ne!(clk, alias);
        assert_eq!(query.signals[0].signal, query.signals[1].signal);
        assert_eq!(query.signals.len(), 5);
        let mut ticks = Vec::new();
        wave.borrow_mut()
            .query_expressions(&query, 0..=15, false, |host, tick| {
                let ExpressionQueryTick::Candidate(time) = tick else {
                    panic!("unexpected baseline")
                };
                assert_eq!(
                    host.sample_value(clk, time).unwrap(),
                    host.sample_value(alias, time).unwrap()
                );
                assert_eq!(
                    host.sample_value(real, time).unwrap(),
                    SampledValue::Real { value: Some(0.5) }
                );
                assert_eq!(
                    host.sample_value(string, time).unwrap(),
                    SampledValue::String {
                        value: Some("ok".into())
                    }
                );
                assert_eq!(host.event_occurred(event, time).unwrap(), time == 0);
                if time > 0 {
                    let prior = time - 1;
                    assert_eq!(host.event_occurred(event, prior).unwrap(), time == 5);
                    let before = host.sample_value(clk, prior).unwrap();
                    assert_ne!(before, host.sample_value(clk, time).unwrap());
                    assert_eq!(before, host.sample_value(clk, prior).unwrap());
                    assert!(host.sample_value(clk, time - 2).is_err());
                    assert!(host.take_waveform_error().is_some());
                }
                assert!(host.resolve_signal("top.clk").is_err());
                assert!(host.signal_type(SignalHandle(999)).is_err());
                assert!(host.sample_value(event, time).is_err());
                assert!(host.event_occurred(clk, time).is_err());
                ticks.push(time);
                Ok(ControlFlow::Continue(()))
            })
            .unwrap();
        assert_eq!(ticks, [0, 5, 10, 15]);
    }

    #[test]
    fn initial_and_candidate_break_stop_the_outer_query() {
        let (_file, wave, binding) = fixture();
        let clk = binding.resolve_signal("top.clk").unwrap();
        let query = ExpressionQuery::new(&binding, &[clk], &[]).unwrap();
        for initialize in [true, false] {
            let mut calls = 0;
            wave.borrow_mut()
                .query_expressions(&query, 1..=15, initialize, |host, tick| {
                    calls += 1;
                    if let ExpressionQueryTick::Initial(time) = tick {
                        assert_eq!(time, 1);
                        assert!(host.sample_value(clk, time - 1).is_err());
                        assert!(matches!(
                            host.take_waveform_error(),
                            Some(WavepeekError::Internal(_))
                        ));
                        assert!(host.sample_value(clk, time).is_ok());
                    }
                    Ok(ControlFlow::Break(()))
                })
                .unwrap();
            assert_eq!(calls, 1);
        }
        let mut calls = 0;
        let error = wave
            .borrow_mut()
            .query_expressions(&query, 1..=15, false, |_, _| {
                calls += 1;
                Err(WavepeekError::Signal("sentinel application failure".into()))
            })
            .unwrap_err();
        assert_eq!(calls, 1);
        assert!(
            matches!(error, WavepeekError::Signal(ref message) if message == "sentinel application failure")
        );
    }

    #[test]
    fn nested_conversion_break_preserves_typed_failure_and_stops_query() {
        let (_file, wave, binding) = fixture();
        let clk = binding.resolve_signal("top.clk").unwrap();
        let mut query = ExpressionQuery::new(&binding, &[clk], &[]).unwrap();
        // Force the converter to reject a sample inside the subset visitor.
        query.signals[0].expr_type.kind = ExprTypeKind::Event;
        let mut calls = 0;
        let error = wave
            .borrow_mut()
            .query_expressions(&query, 1..=15, false, |host, tick| {
                calls += 1;
                let ExpressionQueryTick::Candidate(time) = tick else {
                    unreachable!()
                };
                assert!(host.sample_value(clk, time).is_err());
                Err(host.take_waveform_error().unwrap())
            })
            .unwrap_err();
        assert_eq!(calls, 1);
        assert!(
            matches!(error, WavepeekError::Internal(ref message) if message.contains("raw event"))
        );
    }
}
