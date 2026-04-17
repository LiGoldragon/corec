/// corec lexer — tokenizer for .core domain definitions.
///
/// Zero dependencies. Methods on Lexer struct.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct Spanned {
    pub token: Token,
    pub start: usize,
    pub end: usize,
}

pub struct Lexer<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer { bytes: source.as_bytes(), pos: 0 }
    }

    pub fn lex(mut self) -> Result<Vec<Spanned>, String> {
        let mut tokens = Vec::new();
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                b';' if self.peek_next() == Some(b';') => self.skip_comment(),
                b'(' => { tokens.push(self.single(Token::LParen)); }
                b')' => { tokens.push(self.single(Token::RParen)); }
                b'[' => { tokens.push(self.single(Token::LBracket)); }
                b']' => { tokens.push(self.single(Token::RBracket)); }
                b'{' => { tokens.push(self.single(Token::LBrace)); }
                b'}' => { tokens.push(self.single(Token::RBrace)); }
                b'A'..=b'Z' | b'a'..=b'z' => { tokens.push(self.ident()); }
                b => return Err(format!("unexpected byte '{}' at position {}", b as char, self.pos)),
            }
        }
        Ok(tokens)
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    fn skip_comment(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
            self.pos += 1;
        }
    }

    fn single(&mut self, token: Token) -> Spanned {
        let start = self.pos;
        self.pos += 1;
        Spanned { token, start, end: self.pos }
    }

    fn ident(&mut self) -> Spanned {
        let start = self.pos;
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_alphanumeric() || self.bytes[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let name = String::from_utf8_lossy(&self.bytes[start..self.pos]).to_string();
        Spanned { token: Token::Ident(name), start, end: self.pos }
    }
}
