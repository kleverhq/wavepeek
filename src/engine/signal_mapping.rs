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
