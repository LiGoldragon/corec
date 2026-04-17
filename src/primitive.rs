/// Primitive types — built from source/primitive.aski.
///
/// Three categories parsed from the .aski enum definitions:
/// - Scalar: bare variants, arity 0 (U32, Bool, String, ...)
/// - OmitBounds: generic types that need rkyv omit_bounds (Vec, Option, Box)
/// - Generic: other generic types (Result)

use crate::parse::{Domain, EnumDef, EnumVariant};
use crate::lex::Lexer;
use crate::parse::Parser;

use std::collections::{HashMap, HashSet};

pub struct Primitives {
    /// aski name → Rust name (e.g. "U32" → "u32")
    pub rust_names: HashMap<String, String>,
    /// Names of types that need #[rkyv(omit_bounds)]
    pub omit_bounds: HashSet<String>,
    /// aski name → generic arity (only for arity > 0)
    pub arities: HashMap<String, usize>,
    /// All primitive names
    pub all: HashSet<String>,
}

impl Primitives {
    pub fn load() -> Self {
        let source = include_str!("../source/primitive.core");
        let tokens = Lexer::new(source).lex()
            .expect("failed to lex primitive.aski");
        let module = Parser::new(tokens).parse_file()
            .expect("failed to parse primitive.aski");

        let mut prims = Primitives {
            rust_names: HashMap::new(),
            omit_bounds: HashSet::new(),
            arities: HashMap::new(),
            all: HashSet::new(),
        };

        // The first () is parsed as the module. Its "exports"
        // are the scalar primitive names (arity 0).
        for name in &module.exports {
            prims.all.insert(name.clone());
            prims.rust_names.insert(name.clone(), Self::default_rust_name(name));
        }

        // Remaining domains define generic primitives.
        for domain in &module.domains {
            if let Domain::Enum(def) = domain {
                prims.load_enum(def);
            }
        }

        prims
    }

    fn load_enum(&mut self, def: &EnumDef) {
        let is_omit = def.name == "OmitBounds";

        for variant in &def.variants {
            match variant {
                EnumVariant::Bare(name) => {
                    self.all.insert(name.clone());
                    self.rust_names.insert(name.clone(), Self::default_rust_name(name));
                }
                EnumVariant::Data { name, .. } => {
                    // Arity = 1 (single payload)
                    self.all.insert(name.clone());
                    self.arities.insert(name.clone(), 1);
                    self.rust_names.insert(name.clone(), name.clone());
                    if is_omit {
                        self.omit_bounds.insert(name.clone());
                    }
                }
                EnumVariant::Struct(sdef) => {
                    // Arity = number of fields
                    let arity = sdef.fields.len();
                    self.all.insert(sdef.name.clone());
                    self.arities.insert(sdef.name.clone(), arity);
                    self.rust_names.insert(sdef.name.clone(), sdef.name.clone());
                    if is_omit {
                        self.omit_bounds.insert(sdef.name.clone());
                    }
                }
            }
        }
    }

    fn default_rust_name(name: &str) -> String {
        match name {
            "U8" => "u8", "U16" => "u16", "U32" => "u32", "U64" => "u64",
            "I8" => "i8", "I16" => "i16", "I32" => "i32", "I64" => "i64",
            "F32" => "f32", "F64" => "f64",
            "Bool" => "bool", "Char" => "char",
            other => return other.to_string(),
        }.to_string()
    }

    pub fn is_primitive(&self, name: &str) -> bool {
        self.all.contains(name)
    }

    pub fn map_to_rust<'a>(&'a self, name: &'a str) -> &'a str {
        self.rust_names.get(name).map(|s| s.as_str()).unwrap_or(name)
    }

    pub fn needs_omit_bounds(&self, name: &str) -> bool {
        self.omit_bounds.contains(name)
    }

    pub fn arity(&self, name: &str) -> Option<usize> {
        self.arities.get(name).copied()
    }
}
