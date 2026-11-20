// CLI lists and single source/map entries share the same lexical boundaries.
pub(super) fn signal_list(inputs: &[String]) -> Vec<String> {
    let mut tokens = Vec::new();
    for input in inputs {
        walk_signal_tokens(input, true, |token| tokens.push(token.to_owned()));
    }
    tokens
}

pub(crate) fn signal_token(input: &str) -> String {
    let mut token = String::new();
    walk_signal_tokens(input, false, |part| token = part.to_owned());
    token
}

fn walk_signal_tokens(input: &str, split_commas: bool, mut emit: impl FnMut(&str)) {
    let (mut quoted, mut escaped, mut verilog) = (false, false, false);
    let mut start = None;
    let mut end = 0;
    for (index, ch) in input.char_indices() {
        if split_commas && ch == ',' && !quoted && !escaped && !verilog {
            emit(&input[start.unwrap_or(end)..end]);
            start = None;
            end = index + ch.len_utf8();
            continue;
        }
        // The first whitespace after a Verilog identifier is part of its syntax.
        if quoted || escaped || verilog || !ch.is_whitespace() {
            start.get_or_insert(index);
            end = index + ch.len_utf8();
        }
        if escaped {
            escaped = false;
        } else if quoted {
            match ch {
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
        } else if verilog {
            if ch.is_whitespace() {
                verilog = false;
            }
        } else {
            match ch {
                '"' => quoted = true,
                '\\' => verilog = true,
                _ => {}
            }
        }
    }
    emit(&input[start.unwrap_or(end)..end]);
}

#[cfg(test)]
mod tests {
    use super::{signal_list, signal_token};

    #[test]
    fn lists_preserve_empty_entries_order_and_escaped_commas() {
        assert_eq!(signal_list(&[" a,,b, ".into()]), ["a", "", "b", ""]);
        assert_eq!(
            signal_list(&[" top.\\a,b  , top.\"c,d\", top.\\a,b ".into()]),
            ["top.\\a,b ", "top.\"c,d\"", "top.\\a,b "]
        );
    }

    #[test]
    fn single_tokens_keep_raw_syntax_and_required_terminators() {
        for (input, expected) in [
            ("  top.data[3:0]  ", "top.data[3:0]"),
            ("  top.\\a.b .data[3:0]  ", "top.\\a.b .data[3:0]"),
            (" top.\\leaf  ", "top.\\leaf "),
            (" top.\\leaf\u{2003}  ", "top.\\leaf\u{2003}"),
            (" top.a,b ", "top.a,b"),
            (" top.\\unterminated", "top.\\unterminated"),
            (" top.\"unclosed  ", "top.\"unclosed  "),
            (" data [3:0] ", "data [3:0]"),
        ] {
            assert_eq!(signal_token(input), expected);
            assert_eq!(signal_token(expected), expected);
        }
    }
}
