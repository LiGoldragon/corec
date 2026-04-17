/// corec parser — .core tokens → domain definitions.
///
/// Handles: module, enum (bare/data/struct variants),
/// struct (typed/self-typed fields), type applications.
/// All methods on Parser struct.

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
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub enum EnumVariant {
    Bare(String),
    Data { name: String, payload: TypeExpr },
    Struct(StructDef),
}

#[derive(Debug)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug)]
pub enum StructField {
    Typed { name: String, typ: TypeExpr },
    SelfTyped { name: String },
}

#[derive(Debug)]
pub enum TypeExpr {
    Simple(String),
    Application { constructor: String, args: Vec<TypeExpr> },
}

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse_file(mut self) -> Result<Module, String> {
        let mut module = None;
        let mut domains = Vec::new();

        while !self.at_end() {
            match self.peek() {
                Some(Token::LParen) => {
                    let def = self.parse_enum()?;
                    if module.is_none() {
                        let exports = def.variants.iter().filter_map(|v| {
                            if let EnumVariant::Bare(name) = v { Some(name.clone()) } else { None }
                        }).collect();
                        module = Some((def.name.clone(), exports));
                    } else {
                        domains.push(Domain::Enum(def));
                    }
                }
                Some(Token::LBrace) => {
                    domains.push(Domain::Struct(self.parse_struct()?));
                }
                other => return Err(format!("expected ( or {{ at root, got {:?}", other)),
            }
        }

        let (name, exports) = module.ok_or("no module declaration found")?;
        Ok(Module { name, exports, domains })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos).map(|t| &t.token);
        if tok.is_some() { self.pos += 1; }
        tok
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.advance() {
            Some(t) if t == expected => Ok(()),
            other => Err(format!("expected {:?}, got {:?}", expected, other)),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            other => Err(format!("expected identifier, got {:?}", other)),
        }
    }

    fn parse_enum(&mut self) -> Result<EnumDef, String> {
        self.expect(&Token::LParen)?;
        let name = self.expect_ident()?;
        let mut variants = Vec::new();
        while self.peek() != Some(&Token::RParen) {
            if self.at_end() { return Err("unexpected EOF inside ()".into()); }
            variants.push(self.parse_variant()?);
        }
        self.expect(&Token::RParen)?;
        Ok(EnumDef { name, variants })
    }

    fn parse_variant(&mut self) -> Result<EnumVariant, String> {
        match self.peek() {
            Some(Token::LParen) => {
                self.expect(&Token::LParen)?;
                let name = self.expect_ident()?;
                if self.peek() == Some(&Token::RParen) {
                    self.expect(&Token::RParen)?;
                    Ok(EnumVariant::Bare(name))
                } else {
                    let payload = self.parse_type_expr()?;
                    self.expect(&Token::RParen)?;
                    Ok(EnumVariant::Data { name, payload })
                }
            }
            Some(Token::LBrace) => Ok(EnumVariant::Struct(self.parse_struct()?)),
            Some(Token::Ident(_)) => Ok(EnumVariant::Bare(self.expect_ident()?)),
            other => Err(format!("expected variant, got {:?}", other)),
        }
    }

    fn parse_struct(&mut self) -> Result<StructDef, String> {
        self.expect(&Token::LBrace)?;
        let name = self.expect_ident()?;
        let mut fields = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            if self.at_end() { return Err("unexpected EOF inside {}".into()); }
            fields.push(self.parse_field()?);
        }
        self.expect(&Token::RBrace)?;
        Ok(StructDef { name, fields })
    }

    fn parse_field(&mut self) -> Result<StructField, String> {
        match self.peek() {
            Some(Token::LParen) => {
                self.expect(&Token::LParen)?;
                let name = self.expect_ident()?;
                let typ = self.parse_type_expr()?;
                self.expect(&Token::RParen)?;
                Ok(StructField::Typed { name, typ })
            }
            Some(Token::Ident(_)) => Ok(StructField::SelfTyped { name: self.expect_ident()? }),
            other => Err(format!("expected field, got {:?}", other)),
        }
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, String> {
        match self.peek() {
            Some(Token::LBracket) => {
                self.expect(&Token::LBracket)?;
                let constructor = self.expect_ident()?;
                let mut args = Vec::new();
                while self.peek() != Some(&Token::RBracket) {
                    if self.at_end() { return Err("unexpected EOF inside []".into()); }
                    args.push(self.parse_type_expr()?);
                }
                self.expect(&Token::RBracket)?;
                Ok(TypeExpr::Application { constructor, args })
            }
            Some(Token::Ident(_)) => Ok(TypeExpr::Simple(self.expect_ident()?)),
            other => Err(format!("expected type, got {:?}", other)),
        }
    }
}
