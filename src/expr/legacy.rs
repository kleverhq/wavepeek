use crate::error::WavepeekError;

use super::{EventExpr, EventKind, EventTerm};

pub(crate) fn parse_event_expr(source: &str) -> Result<EventExpr, WavepeekError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(WavepeekError::Args(
            "--on expression cannot be empty. See 'wavepeek change --help'.".to_string(),
        ));
    }

    let term_chunks = split_terms(source)?;
    let mut terms = Vec::with_capacity(term_chunks.len());
    for chunk in term_chunks {
        terms.push(parse_event_term(chunk.as_str())?);
    }

    Ok(EventExpr { terms })
}

fn split_terms(source: &str) -> Result<Vec<String>, WavepeekError> {
    let mut terms = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut index = 0usize;

    while index < source.len() {
        let ch = source[index..].chars().next().ok_or_else(|| {
            WavepeekError::Internal("failed to decode event expression while splitting".to_string())
        })?;
        let ch_len = ch.len_utf8();

        match ch {
            '(' => {
                depth += 1;
                index += ch_len;
                continue;
            }
            ')' => {
                depth = depth.saturating_sub(1);
                index += ch_len;
                continue;
            }
            ',' if depth == 0 => {
                let segment = source[start..index].trim();
                if segment.is_empty() {
                    return Err(invalid_event_expr_error(source));
                }
                terms.push(segment.to_string());
                start = index + ch_len;
                index = start;
                continue;
            }
            _ => {}
        }

        if depth == 0 && source[index..].starts_with("or") && token_boundary(source, index, "or") {
            let segment = source[start..index].trim();
            if segment.is_empty() {
                return Err(invalid_event_expr_error(source));
            }
            terms.push(segment.to_string());
            start = index + 2;
            index = start;
            continue;
        }

        index += ch_len;
    }

    let tail = source[start..].trim();
    if tail.is_empty() {
        return Err(invalid_event_expr_error(source));
    }
    terms.push(tail.to_string());

    Ok(terms)
}

fn parse_event_term(segment: &str) -> Result<EventTerm, WavepeekError> {
    let segment = segment.trim();
    let (event, rest) = parse_basic_event(segment)?;
    let rest = rest.trim();
    if rest.is_empty() {
        return Ok(EventTerm {
            event,
            iff_expr: None,
        });
    }

    if !rest.starts_with("iff") || !token_boundary(rest, 0, "iff") {
        return Err(invalid_event_expr_error(segment));
    }

    let iff_expr = rest[3..].trim().to_string();
    Ok(EventTerm {
        event,
        iff_expr: Some(iff_expr),
    })
}

fn parse_basic_event(segment: &str) -> Result<(EventKind, &str), WavepeekError> {
    let segment = segment.trim();

    if let Some(rest) = segment.strip_prefix('*') {
        if rest.trim().is_empty() || rest.trim_start().starts_with("iff") {
            return Ok((EventKind::AnyTracked, rest));
        }
        return Err(invalid_event_expr_error(segment));
    }

    if let Some((name, rest)) = consume_prefixed_name(segment, "posedge") {
        return Ok((EventKind::Posedge(name), rest));
    }
    if let Some((name, rest)) = consume_prefixed_name(segment, "negedge") {
        return Ok((EventKind::Negedge(name), rest));
    }
    if let Some((name, rest)) = consume_prefixed_name(segment, "edge") {
        return Ok((EventKind::Edge(name), rest));
    }

    let (name, rest) = consume_name(segment)?;
    Ok((EventKind::AnyChange(name), rest))
}

fn consume_prefixed_name<'a>(segment: &'a str, prefix: &str) -> Option<(String, &'a str)> {
    if !segment.starts_with(prefix) || !token_boundary(segment, 0, prefix) {
        return None;
    }

    let rest = segment[prefix.len()..].trim_start();
    if rest.is_empty() {
        return Some((String::new(), rest));
    }

    let (name, tail) = consume_name(rest).ok()?;
    Some((name, tail))
}

fn consume_name(segment: &str) -> Result<(String, &str), WavepeekError> {
    let mut end = 0usize;
    for (index, ch) in segment.char_indices() {
        if is_name_char(ch) {
            end = index + ch.len_utf8();
            continue;
        }
        break;
    }

    if end == 0 {
        return Err(invalid_event_expr_error(segment));
    }

    Ok((segment[..end].to_string(), &segment[end..]))
}

fn is_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '$' | '[' | ']' | ':')
}

fn token_boundary(input: &str, start: usize, token: &str) -> bool {
    let end = start + token.len();
    let previous_ok = if start == 0 {
        true
    } else {
        !is_name_char(input[..start].chars().next_back().unwrap_or(' '))
    };
    let next_ok = if end >= input.len() {
        true
    } else {
        !is_name_char(input[end..].chars().next().unwrap_or(' '))
    };

    previous_ok && next_ok
}

fn invalid_event_expr_error(source: &str) -> WavepeekError {
    WavepeekError::Args(format!(
        "invalid --on expression '{source}'. See 'wavepeek change --help'."
    ))
}

#[cfg(test)]
mod tests {
    use super::parse_event_expr;
    use crate::expr::{EventKind, EventTerm};

    #[test]
    fn event_expr_iff_binding_with_union() {
        let parsed =
            parse_event_expr("negedge clk iff rstn or bar").expect("event expression should parse");

        assert_eq!(
            parsed.terms,
            vec![
                EventTerm {
                    event: EventKind::Negedge("clk".to_string()),
                    iff_expr: Some("rstn".to_string())
                },
                EventTerm {
                    event: EventKind::AnyChange("bar".to_string()),
                    iff_expr: None
                }
            ]
        );
    }

    #[test]
    fn event_expr_iff_capture_parenthesized_logical_payload() {
        let parsed = parse_event_expr("posedge clk iff (a || b) or bar")
            .expect("event expression should parse");

        assert_eq!(
            parsed.terms,
            vec![
                EventTerm {
                    event: EventKind::Posedge("clk".to_string()),
                    iff_expr: Some("(a || b)".to_string())
                },
                EventTerm {
                    event: EventKind::AnyChange("bar".to_string()),
                    iff_expr: None
                }
            ]
        );
    }

    #[test]
    fn event_expr_accepts_comma_union() {
        let parsed = parse_event_expr("posedge clk1, posedge clk2")
            .expect("comma-union expression should parse");

        assert_eq!(parsed.terms.len(), 2);
        assert!(matches!(parsed.terms[0].event, EventKind::Posedge(_)));
        assert!(matches!(parsed.terms[1].event, EventKind::Posedge(_)));
    }
}
