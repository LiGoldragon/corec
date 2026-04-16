/// cc parser — reads .aski declarations into domain definitions.
///
/// Handles the three domain forms:
/// - Enum: (Name Variant1 Variant2 ...)
/// - Struct: {Name (Field Type) ... / SelfTypedField ...}
/// - Module: first () in file — (ModuleName Export1 Export2 ...)
///
/// No bodies, no expressions, no traits, no generics.

use crate::lex::{Token, Spanned};

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub exports: Vec<String>,
    pub domains: Vec<Domain>,
}

#[derive(Debug)]
pub enum Domain {
    Enum(EnumDef),
    Struct(StructDef),
}

#[derive(Debug)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug)]
pub enum StructField {
    Typed { name: String, typ: String },
    SelfTyped { name: String },
}

struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Spanned>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos).map(|t| &t.token);
        if tok.is_some() { self.pos += 1; }
        tok
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::PascalIdent(s)) => Ok(s.clone()),
            other => Err(format!("expected identifier, got {:?}", other)),
        }
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.advance() {
            Some(t) if t == expected => Ok(()),
            other => Err(format!("expected {:?}, got {:?}", expected, other)),
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

pub fn parse_file(tokens: Vec<Spanned>) -> Result<Module, String> {
    let mut parser = Parser::new(tokens);
    let mut module = None;
    let mut domains = Vec::new();

    while !parser.at_end() {
        match parser.peek() {
            Some(Token::LParen) => {
                let def = parse_paren(&mut parser)?;
                if module.is_none() {
                    // first () is the module declaration
                    module = Some((def.name.clone(), def.variants.clone()));
                } else {
                    domains.push(Domain::Enum(def));
                }
            }
            Some(Token::LBrace) => {
                let def = parse_brace(&mut parser)?;
                domains.push(Domain::Struct(def));
            }
            other => {
                return Err(format!("expected ( or {{ at root, got {:?}", other));
            }
        }
    }

    let (name, exports) = module.ok_or("no module declaration found")?;
    Ok(Module { name, exports, domains })
}

fn parse_paren(parser: &mut Parser) -> Result<EnumDef, String> {
    parser.expect(&Token::LParen)?;
    let name = parser.expect_ident()?;
    let mut variants = Vec::new();

    while parser.peek() != Some(&Token::RParen) {
        if parser.at_end() {
            return Err("unexpected EOF inside ()".into());
        }
        let variant = parser.expect_ident()?;
        variants.push(variant);
    }

    parser.expect(&Token::RParen)?;
    Ok(EnumDef { name, variants })
}

fn parse_brace(parser: &mut Parser) -> Result<StructDef, String> {
    parser.expect(&Token::LBrace)?;
    let name = parser.expect_ident()?;
    let mut fields = Vec::new();

    while parser.peek() != Some(&Token::RBrace) {
        if parser.at_end() {
            return Err("unexpected EOF inside {}".into());
        }

        match parser.peek() {
            // typed field: (FieldName Type)
            Some(Token::LParen) => {
                parser.expect(&Token::LParen)?;
                let field_name = parser.expect_ident()?;
                let typ = parser.expect_ident()?;
                parser.expect(&Token::RParen)?;
                fields.push(StructField::Typed { name: field_name, typ });
            }
            // self-typed field: FieldName
            Some(Token::PascalIdent(_)) => {
                let field_name = parser.expect_ident()?;
                fields.push(StructField::SelfTyped { name: field_name });
            }
            other => {
                return Err(format!("expected field or ), got {:?}", other));
            }
        }
    }

    parser.expect(&Token::RBrace)?;
    Ok(StructDef { name, fields })
}
