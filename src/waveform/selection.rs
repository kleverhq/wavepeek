use crate::error::WavepeekError;

// Commas also need quoting because CLI selection flags accept comma-separated lists.
pub(crate) fn format_path(path: &ondas::HierarchyPath) -> String {
    if !path.components().any(|component| component.contains(',')) {
        return path.to_string();
    }
    path.components()
        .map(|component| {
            if component.contains(',') {
                serde_json::Value::String(component.to_owned()).to_string()
            } else {
                ondas::HierarchyPath::from_components([component]).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BitRange {
    pub msb: u32,
    pub lsb: u32,
}

pub(crate) fn parse_trailing_range(token: &str) -> Result<Option<(&str, BitRange)>, WavepeekError> {
    let (mut quoted, mut escaped, mut verilog, mut open) = (false, false, false, None);
    for (index, ch) in token.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if quoted {
            if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else if verilog {
            if ch.is_whitespace() {
                verilog = false;
            }
        } else {
            match ch {
                '"' => quoted = true,
                '\\' => verilog = true,
                '[' => open = Some(index),
                _ => {}
            }
        }
    }
    let Some(open) = open else {
        return Ok(None);
    };
    let contents = &token[open + 1..];
    if !contents.contains(':') {
        return Ok(None);
    }
    if !token.ends_with(']') {
        return Err(invalid_projection(token, "range must end with ']'"));
    }

    let base = &token[..open];
    if base.is_empty() {
        return Err(invalid_projection(
            token,
            "base signal path must not be empty",
        ));
    }
    if base.ends_with(']') {
        return Err(invalid_projection(
            token,
            "chained and multidimensional projections are not supported",
        ));
    }

    let range = &contents[..contents.len() - 1];
    let Some((msb, lsb)) = range.split_once(':') else {
        return Ok(None);
    };
    if msb.is_empty() || lsb.is_empty() || lsb.contains(':') {
        return Err(invalid_projection(
            token,
            "expected one static range '[msb:lsb]'",
        ));
    }
    let msb = parse_bound(token, msb)?;
    let lsb = parse_bound(token, lsb)?;
    if msb < lsb {
        return Err(invalid_projection(
            token,
            "msb must be greater than or equal to lsb",
        ));
    }

    Ok(Some((base, BitRange { msb, lsb })))
}

fn parse_bound(token: &str, bound: &str) -> Result<u32, WavepeekError> {
    if !bound.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid_projection(
            token,
            "bounds must be non-negative decimal integers",
        ));
    }
    bound
        .parse::<u32>()
        .map_err(|_| invalid_projection(token, "bounds exceed the supported integer range"))
}

fn invalid_projection(token: &str, detail: &str) -> WavepeekError {
    WavepeekError::Signal(format!("invalid flat projection '{token}': {detail}"))
}

#[cfg(test)]
mod tests {
    use super::{BitRange, parse_trailing_range};

    #[test]
    fn parses_only_one_static_trailing_range() {
        assert_eq!(parse_trailing_range("top.data").unwrap(), None);
        assert_eq!(parse_trailing_range("top.mem[0]").unwrap(), None);
        assert_eq!(
            parse_trailing_range("top.data[7:4]").unwrap(),
            Some(("top.data", BitRange { msb: 7, lsb: 4 }))
        );
        assert_eq!(
            parse_trailing_range("top.data[0:0]").unwrap(),
            Some(("top.data", BitRange { msb: 0, lsb: 0 }))
        );

        for invalid in [
            "top.data[4:7]",
            "top.data[-1:0]",
            "top.data[WIDTH:0]",
            "top.data[7:]",
            "top.data[7:4:1]",
            "top.data[7:4",
            "top.data[7:4][3:2]",
        ] {
            assert!(
                parse_trailing_range(invalid).is_err(),
                "{invalid} should fail"
            );
        }
    }
}
