#[cfg(test)]
mod tests {
    use crate::lex::*;

    #[test]
    fn lex_enum() {
        let tokens = Lexer::new("(Element Fire Earth Air Water)").lex().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Ident("Element".into()));
        assert_eq!(tokens[5].token, Token::Ident("Water".into()));
        assert_eq!(tokens[6].token, Token::RParen);
    }

    #[test]
    fn lex_struct() {
        let tokens = Lexer::new("{Span (Start U32) (End U32)}").lex().unwrap();
        assert_eq!(tokens[0].token, Token::LBrace);
        assert_eq!(tokens[1].token, Token::Ident("Span".into()));
        assert_eq!(tokens[2].token, Token::LParen);
    }

    #[test]
    fn lex_comment() {
        let tokens = Lexer::new(";; comment\n(Span Span)").lex().unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::Ident("Span".into()));
    }

    #[test]
    fn lex_brackets() {
        let tokens = Lexer::new("[Vec Item]").lex().unwrap();
        assert_eq!(tokens[0].token, Token::LBracket);
        assert_eq!(tokens[1].token, Token::Ident("Vec".into()));
        assert_eq!(tokens[2].token, Token::Ident("Item".into()));
        assert_eq!(tokens[3].token, Token::RBracket);
    }
}
