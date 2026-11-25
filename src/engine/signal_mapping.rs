use std::collections::HashSet;

use ondas::{HierarchyPath, SignalSelector};
use regex::Regex;

use super::ReportProgress;
use crate::engine::expr_runtime::SharedWaveform;
use crate::error::WavepeekError;

#[derive(Debug, Clone)]
pub(super) struct MappedSignal {
    pub(super) path: String,
    pub(super) selector: SignalSelector,
}

impl MappedSignal {
    pub(super) fn whole(path: HierarchyPath) -> Self {
        Self {
            path: path.to_string(),
            selector: SignalSelector::new(path, None),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct SignalCandidate {
    pub(super) display: String,
    pub(super) name: String,
    pub(super) selection: MappedSignal,
}

pub(super) fn collect_include_candidates(
    waveform: &SharedWaveform<'_>,
    scope: Option<&str>,
    includes: &[Regex],
    report: ReportProgress<'_>,
) -> Result<Vec<SignalCandidate>, WavepeekError> {
    if includes.is_empty() {
        return Ok(Vec::new());
    }

    report("finding protocol signals");
    let waveform = waveform.borrow();
    let matches = |text: &str| includes.iter().any(|include| include.is_match(text));
    let entries = if let Some(scope) = scope {
        waveform
            .signals_in_scope(scope)?
            .into_iter()
            .filter(|entry| matches(&entry.name))
            .collect()
    } else {
        waveform.matching_signals(|path, name| matches(path) || matches(name))
    };
    let mut seen = HashSet::new();
    Ok(entries
        .into_iter()
        .filter_map(|entry| {
            let selection = MappedSignal::whole(entry.path);
            seen.insert(selection.path.clone())
                .then(|| SignalCandidate {
                    display: if scope.is_some() {
                        entry.name.clone()
                    } else {
                        selection.path.clone()
                    },
                    name: entry.name,
                    selection,
                })
        })
        .collect())
}

pub(super) fn resolve_explicit_mapping(
    waveform: &SharedWaveform<'_>,
    input: &str,
    scope: Option<&str>,
    standard: &str,
    protocol: &str,
    help: &str,
) -> Result<MappedSignal, WavepeekError> {
    let selector = SignalSelector::parse(input)
        .map_err(|error| WavepeekError::Signal(ondas::LookupError::from(error).to_string()))?;
    if scope.is_some() && selector.path().len() != 1 {
        return Err(WavepeekError::Args(format!(
            "{protocol} mapping for '{standard}' must use a scope-relative signal when --scope is set. See '{help} --help'."
        )));
    }
    let resolved = waveform
        .borrow()
        .resolve_typed_signal_with_diagnostic(input, &selector, scope)?;
    Ok(MappedSignal {
        path: resolved.path,
        selector: resolved.selector,
    })
}

pub(crate) fn candidate_matching_standards<'a>(
    candidate_name: &str,
    standards: &[&'a str],
    atomic_standards: &[&str],
) -> Vec<&'a str> {
    let tokens = candidate_core_tokens(candidate_name);
    let suffix_matches = standards
        .iter()
        .filter_map(|standard| {
            standard_suffix_start(tokens.as_slice(), standard, atomic_standards)
                .map(|start| (*standard, start))
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
                    standard_matches_range(
                        tokens.as_slice(),
                        start,
                        *suffix_start,
                        standard,
                        atomic_standards,
                    )
                })
        })
        .copied()
        .collect()
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

fn standard_suffix_start(
    tokens: &[String],
    standard: &str,
    atomic_standards: &[&str],
) -> Option<usize> {
    (0..tokens.len()).find(|start| {
        standard_matches_range(tokens, *start, tokens.len(), standard, atomic_standards)
    })
}

fn standard_matches_range(
    tokens: &[String],
    start: usize,
    end: usize,
    standard: &str,
    atomic_standards: &[&str],
) -> bool {
    if atomic_standards.contains(&standard) {
        return end == start + 1 && tokens[start] == standard;
    }
    tokens[start..end]
        .iter()
        .flat_map(|token| token.chars())
        .eq(standard.chars())
}

#[cfg(test)]
mod tests {
    use super::candidate_matching_standards;

    #[test]
    fn matches_compact_and_split_suffixes_without_matching_decoys() {
        let standards = ["aclk", "tvalid", "tready", "tdata"];
        let atomic = ["aclk"];

        for name in ["tvalid", "axis_tvalid_o", "axis_t_valid_o", "s_axis_tvalid"] {
            assert_eq!(
                candidate_matching_standards(name, &standards, &atomic),
                ["tvalid"],
                "{name}"
            );
        }
        assert!(candidate_matching_standards("axis_tvalidchk", &standards, &atomic).is_empty());
        assert!(candidate_matching_standards("axis_aclk_gate", &standards, &atomic).is_empty());
    }
}
