/// cc lexer — minimal tokenizer for .aski declarations.
///
/// Only handles: comments (;;), parens, braces, PascalCase identifiers.
/// No expressions, no sigils, no operators. Just enough for
/// module/enum/struct declarations.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    LBrace,
    RBrace,
    PascalIdent(String),
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub start: usize,
    pub end: usize,
}

pub fn lex(source: &str) -> Result<Vec<Spanned>, String> {
    let mut tokens = Vec::new();
    let bytes = source.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        let b = bytes[pos];
        match b {
            // whitespace
            b' ' | b'\t' | b'\n' | b'\r' => { pos += 1; }

            // comment — ;; to end of line
            b';' if pos + 1 < bytes.len() && bytes[pos + 1] == b';' => {
                while pos < bytes.len() && bytes[pos] != b'\n' {
                    pos += 1;
                }
            }

            b'(' => {
                tokens.push(Spanned { token: Token::LParen, start: pos, end: pos + 1 });
                pos += 1;
            }
            b')' => {
                tokens.push(Spanned { token: Token::RParen, start: pos, end: pos + 1 });
                pos += 1;
            }
            b'{' => {
                tokens.push(Spanned { token: Token::LBrace, start: pos, end: pos + 1 });
                pos += 1;
            }
            b'}' => {
                tokens.push(Spanned { token: Token::RBrace, start: pos, end: pos + 1 });
                pos += 1;
            }

            // PascalCase identifier (starts with uppercase A-Z)
            b'A'..=b'Z' => {
                let start = pos;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    pos += 1;
                }
                let name = String::from_utf8_lossy(&bytes[start..pos]).to_string();
                tokens.push(Spanned { token: Token::PascalIdent(name), start, end: pos });
            }

            // camelCase identifier (starts with lowercase a-z) — skip for cc
            b'a'..=b'z' => {
                let start = pos;
                while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                    pos += 1;
                }
                let name = String::from_utf8_lossy(&bytes[start..pos]).to_string();
                tokens.push(Spanned { token: Token::PascalIdent(name), start, end: pos });
            }

            _ => {
                return Err(format!("unexpected byte '{}' at position {}", b as char, pos));
            }
        }
    }

    Ok(tokens)
}
