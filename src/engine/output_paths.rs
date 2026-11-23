use ondas::{HierarchyPath, PathFormatError, PathStyle};

use crate::error::WavepeekError;
use crate::waveform::{ResolvedSignal, SelectorPath, Waveform, format_path};

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

    pub(super) fn context_scope(&self, machine: bool) -> Result<Option<String>, WavepeekError> {
        if !machine {
            return Ok(None);
        }
        self.scope
            .as_ref()
            .map(|path| match self.style {
                PathStyle::Logical => Ok(format_path(path)),
                PathStyle::Sv => path.format(self.style).map_err(path_style_error),
            })
            .transpose()
    }

    pub(super) fn exact(&self, logical: String) -> Result<String, WavepeekError> {
        if self.style == PathStyle::Logical {
            return Ok(logical);
        }
        parse_resolved_path(&logical)?
            .format(self.style)
            .map_err(path_style_error)
    }

    pub(super) fn listed(&self, logical: &str) -> Result<(String, String), WavepeekError> {
        self.render(&SelectorPath::new(
            parse_resolved_path(logical)?,
            None,
            false,
        ))
    }

    pub(super) fn signal(
        &self,
        selected: &ResolvedSignal,
    ) -> Result<(String, String), WavepeekError> {
        self.render(&selected.selector)
    }

    fn render(&self, selector: &SelectorPath) -> Result<(String, String), WavepeekError> {
        let format = |path: &SelectorPath| match self.style {
            PathStyle::Logical => Ok(path.logical()),
            PathStyle::Sv => path.sv().map_err(path_style_error),
        };
        let relative = self
            .scope
            .as_ref()
            .and_then(|scope| selector.relative_to(scope));
        let display = format(relative.as_ref().unwrap_or(selector))?;
        let path = if self.absolute && relative.is_some() {
            format(selector)?
        } else {
            display.clone()
        };
        Ok((path, display))
    }

    pub(super) fn style_mappings<'a>(
        &self,
        waveform: &Waveform,
        mappings: impl IntoIterator<Item = (&'a mut String, &'a mut String)>,
    ) -> Result<(), WavepeekError> {
        if self.style == PathStyle::Sv {
            for (path, display) in mappings {
                let selector = waveform.resolve_selector_path(path)?;
                self.style_mapping(&selector, path, display)?;
            }
        }
        Ok(())
    }

    pub(super) fn style_mapping(
        &self,
        selector: &SelectorPath,
        path: &mut String,
        display: &mut String,
    ) -> Result<(), WavepeekError> {
        if self.style == PathStyle::Sv {
            (*path, *display) = self.render(selector)?;
        }
        Ok(())
    }
}

fn parse_resolved_path(logical: &str) -> Result<HierarchyPath, WavepeekError> {
    HierarchyPath::parse(logical).map_err(|error| {
        WavepeekError::Internal(format!("invalid resolved path '{logical}': {error}"))
    })
}

fn path_style_error(error: PathFormatError) -> WavepeekError {
    WavepeekError::Args(format!("--path-style sv: {error}"))
}
