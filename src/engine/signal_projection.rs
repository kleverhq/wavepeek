use crate::engine::scoped_signal_path;
use crate::error::WavepeekError;
use crate::waveform::selection::parse_trailing_range;
use crate::waveform::{ResolvedSignal, SampledSignalState, Waveform};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectedSignal {
    pub source: ResolvedSignal,
    pub path: String,
    projected: bool,
}

impl ProjectedSignal {
    #[cfg(test)]
    pub(crate) fn unprojected(source: ResolvedSignal) -> Self {
        Self {
            path: source.path.clone(),
            source,
            projected: false,
        }
    }

    pub(crate) fn width(&self) -> u32 {
        self.source.width
    }

    pub(crate) fn is_projected(&self) -> bool {
        self.projected
    }

    pub(crate) fn project_sample(&self, sample: SampledSignalState) -> SampledSignalState {
        SampledSignalState {
            path: self.path.clone(),
            width: self.width(),
            bits: sample.bits,
        }
    }
}

pub(crate) fn resolve_projected_signal(
    waveform: &Waveform,
    token: &str,
    scope: Option<&str>,
) -> Result<ProjectedSignal, WavepeekError> {
    let canonical_path = scoped_signal_path(token, scope);
    match waveform.resolve_signals(std::slice::from_ref(&canonical_path)) {
        Ok(mut resolved) => {
            let source = resolved.remove(0);
            return Ok(ProjectedSignal {
                path: source.path.clone(),
                source,
                projected: false,
            });
        }
        Err(WavepeekError::SignalNotFound(_)) => {}
        Err(_)
            if ondas::HierarchyPath::parse(&canonical_path).is_err()
                && parse_trailing_range(token)?.is_some() => {}
        Err(error) => return Err(error),
    }

    let Some((base_token, range)) = parse_trailing_range(token)? else {
        let source = waveform
            .resolve_signals_with_diagnostics(
                std::slice::from_ref(&canonical_path),
                std::slice::from_ref(&token.to_string()),
                scope,
            )?
            .remove(0);
        return Ok(ProjectedSignal {
            path: source.path.clone(),
            source,
            projected: false,
        });
    };

    let base_path = scoped_signal_path(base_token, scope);
    let source = waveform
        .resolve_signals_with_diagnostics(
            std::slice::from_ref(&base_path),
            std::slice::from_ref(&base_token.to_string()),
            scope,
        )?
        .remove(0);
    if range.msb >= source.width {
        return Err(WavepeekError::Signal(format!(
            "projection '[{}:{}]' is outside signal '{}' width {}",
            range.msb, range.lsb, source.path, source.width
        )));
    }

    let source = waveform.project_signal(&source, range.msb, range.lsb)?;
    Ok(ProjectedSignal {
        path: source.path.clone(),
        source,
        projected: true,
    })
}
