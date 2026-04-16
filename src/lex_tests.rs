#[cfg(test)]
mod tests {
    use crate::lex::*;

    #[test]
    fn lex_enum() {
        let source = "(Element Fire Earth Air Water)";
        let tokens = lex(source).unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::PascalIdent("Element".into()));
        assert_eq!(tokens[2].token, Token::PascalIdent("Fire".into()));
        assert_eq!(tokens[5].token, Token::PascalIdent("Water".into()));
        assert_eq!(tokens[6].token, Token::RParen);
    }

    #[test]
    fn lex_struct() {
        let source = "{Span (Start U32) (End U32)}";
        let tokens = lex(source).unwrap();
        assert_eq!(tokens[0].token, Token::LBrace);
        assert_eq!(tokens[1].token, Token::PascalIdent("Span".into()));
        assert_eq!(tokens[2].token, Token::LParen);
        assert_eq!(tokens[3].token, Token::PascalIdent("Start".into()));
        assert_eq!(tokens[4].token, Token::PascalIdent("U32".into()));
        assert_eq!(tokens[5].token, Token::RParen);
    }

    #[test]
    fn lex_comment() {
        let source = ";; this is a comment\n(Span Span)";
        let tokens = lex(source).unwrap();
        assert_eq!(tokens[0].token, Token::LParen);
        assert_eq!(tokens[1].token, Token::PascalIdent("Span".into()));
    }

    #[test]
    fn lex_multiline() {
        let source = "(ScopeKind\n  Root Module\n  Enum Struct)";
        let tokens = lex(source).unwrap();
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[3].token, Token::PascalIdent("Module".into()));
    }
}
