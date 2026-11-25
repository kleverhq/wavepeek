use ondas::{HierarchyPath, PathFormatError, PathStyle, SignalSelector};

use crate::error::WavepeekError;
use crate::waveform::{ResolvedSignal, Waveform};

#[derive(Debug, Clone)]
pub(super) struct OutputPaths {
    style: PathStyle,
    scope: Option<HierarchyPath>,
    absolute: bool,
}

impl OutputPaths {
    pub(super) fn new(
        style: PathStyle,
        scope: Option<&str>,
        absolute: bool,
    ) -> Result<Self, WavepeekError> {
        let scope = scope
            .map(HierarchyPath::parse)
            .transpose()
            .map_err(|error| WavepeekError::Scope(error.to_string()))?;
        Ok(Self {
            style,
            scope,
            absolute,
        })
    }

    pub(super) fn context_scope(
        &self,
        waveform: &Waveform<'_>,
        machine: bool,
    ) -> Result<Option<String>, WavepeekError> {
        if !machine {
            return Ok(None);
        }
        self.scope
            .as_ref()
            .map(|path| self.exact(&waveform.canonical_scope(path)?))
            .transpose()
    }

    pub(super) fn exact(&self, path: &HierarchyPath) -> Result<String, WavepeekError> {
        path.format(self.style).map_err(path_style_error)
    }

    pub(super) fn listed(&self, path: HierarchyPath) -> Result<(String, String), WavepeekError> {
        self.selector(&SignalSelector::new(path, None))
    }

    pub(super) fn signal(
        &self,
        selected: &ResolvedSignal,
    ) -> Result<(String, String), WavepeekError> {
        self.selector(&selected.selector)
    }

    pub(super) fn selector(
        &self,
        selector: &SignalSelector,
    ) -> Result<(String, String), WavepeekError> {
        let format =
            |selector: &SignalSelector| selector.format(self.style).map_err(path_style_error);
        let base = selector.path();
        let relative = self
            .scope
            .as_ref()
            .filter(|scope| {
                !scope.is_empty()
                    && scope.len() < base.len()
                    && scope.components().eq(base.components().take(scope.len()))
            })
            .map(|scope| {
                let suffix = HierarchyPath::from_components(base.components().skip(scope.len()));
                SignalSelector::new(suffix, selector.projection())
            });
        let display = format(relative.as_ref().unwrap_or(selector))?;
        let path = if self.absolute && relative.is_some() {
            format(selector)?
        } else {
            display.clone()
        };
        Ok((path, display))
    }
}

fn path_style_error(error: PathFormatError) -> WavepeekError {
    WavepeekError::Args(format!("--path-style sv: {error}"))
}
