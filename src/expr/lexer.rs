#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Literal(String),
    Operator(String),
    LeftParen,
    RightParen,
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < source.len() {
        let ch = source[index..].chars().next().unwrap_or('\0');
        let ch_len = ch.len_utf8();

        if ch.is_whitespace() {
            index += ch_len;
            continue;
        }

        match ch {
            '(' => {
                tokens.push(Token::LeftParen);
                index += ch_len;
                continue;
            }
            ')' => {
                tokens.push(Token::RightParen);
                index += ch_len;
                continue;
            }
            ',' | '*' => {
                tokens.push(Token::Operator(ch.to_string()));
                index += ch_len;
                continue;
            }
            _ => {}
        }

        let mut end = index;
        for (offset, current) in source[index..].char_indices() {
            if current.is_whitespace() || matches!(current, '(' | ')' | ',' | '*') {
                break;
            }
            end = index + offset + current.len_utf8();
        }

        if end == index {
            tokens.push(Token::Literal(ch.to_string()));
            index += ch_len;
            continue;
        }

        let word = &source[index..end];
        match word {
            "or" | "iff" | "posedge" | "negedge" | "edge" => {
                tokens.push(Token::Operator(word.to_string()))
            }
            _ => tokens.push(Token::Identifier(word.to_string())),
        }
        index = end;
    }

    tokens
}
