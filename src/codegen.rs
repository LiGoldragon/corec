/// corec codegen — domain definitions → Rust source with rkyv derives.
///
/// All methods on Codegen struct.

use crate::parse::{Module, Domain, EnumDef, EnumVariant, StructDef, StructField, TypeExpr};
use crate::primitive::Primitives;

pub struct Codegen {
    out: String,
    primitives: Primitives,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            out: String::new(),
            primitives: Primitives::load(),
        }
    }

    pub fn emit_module(mut self, module: &Module) -> String {
        for domain in &module.domains {
            match domain {
                Domain::Enum(def) => self.emit_enum(def),
                Domain::Struct(def) => self.emit_struct(def),
            }
            self.out.push('\n');
        }
        self.out
    }

    fn emit_enum(&mut self, def: &EnumDef) {
        let has_data = def.variants.iter().any(|v| !matches!(v, EnumVariant::Bare(_)));
        self.emit_derives(has_data);
        self.out.push_str(&format!("pub enum {} {{\n", def.name));
        for variant in &def.variants {
            self.emit_variant(variant);
        }
        self.out.push_str("}\n");
    }

    fn emit_variant(&mut self, variant: &EnumVariant) {
        match variant {
            EnumVariant::Bare(name) => {
                self.out.push_str(&format!("    {},\n", Self::escape_variant(name)));
            }
            EnumVariant::Data { name, payload } => {
                if self.check_omit_bounds(payload) {
                    self.out.push_str(&format!("    {}(#[rkyv(omit_bounds)] {}),\n",
                        Self::escape_variant(name), self.type_to_rust(payload)));
                } else {
                    self.out.push_str(&format!("    {}({}),\n",
                        Self::escape_variant(name), self.type_to_rust(payload)));
                }
            }
            EnumVariant::Struct(def) => {
                self.out.push_str(&format!("    {} {{\n", Self::escape_variant(&def.name)));
                for field in &def.fields {
                    self.emit_field(field, "        ", false);
                }
                self.out.push_str("    },\n");
            }
        }
    }

    fn emit_struct(&mut self, def: &StructDef) {
        self.emit_derives(true);
        self.out.push_str(&format!("pub struct {} {{\n", def.name));
        for field in &def.fields {
            self.emit_field(field, "    ", true);
        }
        self.out.push_str("}\n");
    }

    fn emit_field(&mut self, field: &StructField, indent: &str, public: bool) {
        let vis = if public { "pub " } else { "" };
        match field {
            StructField::Typed { name, typ } => {
                if self.check_omit_bounds(typ) {
                    self.out.push_str(&format!(
                        "{}#[rkyv(omit_bounds)]\n{}{}{}: {},\n",
                        indent, indent, vis, Self::to_snake(name), self.type_to_rust(typ)
                    ));
                } else {
                    self.out.push_str(&format!(
                        "{}{}{}: {},\n",
                        indent, vis, Self::to_snake(name), self.type_to_rust(typ)
                    ));
                }
            }
            StructField::SelfTyped { name } => {
                self.out.push_str(&format!(
                    "{}{}{}: {},\n", indent, vis, Self::to_snake(name), name
                ));
            }
        }
    }

    fn emit_derives(&mut self, has_data: bool) {
        if has_data {
            self.out.push_str(concat!(
                "#[derive(Debug, Clone, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]\n",
                "#[rkyv(serialize_bounds(",
                    "__S: rkyv::ser::Writer + rkyv::ser::Allocator, ",
                    "__S::Error: rkyv::rancor::Source))]\n",
                "#[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]\n",
            ));
        } else {
            self.out.push_str(concat!(
                "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ",
                "rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]\n",
            ));
        }
    }

    fn type_to_rust(&self, typ: &TypeExpr) -> String {
        match typ {
            TypeExpr::Simple(name) => self.primitives.map_to_rust(name).to_string(),
            TypeExpr::Application { constructor, args } => {
                let args_rust: Vec<String> = args.iter()
                    .map(|a| self.type_to_rust(a))
                    .collect();
                format!("{}<{}>", constructor, args_rust.join(", "))
            }
        }
    }

    fn check_omit_bounds(&self, typ: &TypeExpr) -> bool {
        match typ {
            TypeExpr::Simple(_) => false,
            TypeExpr::Application { constructor, args } => {
                self.primitives.needs_omit_bounds(constructor)
                    || args.iter().any(|a| self.check_omit_bounds(a))
            }
        }
    }

    fn escape_variant(name: &str) -> String {
        match name {
            "Self" => "Self_".into(),
            "Type" => "Type_".into(),
            _ => name.to_string(),
        }
    }

    fn to_snake(pascal: &str) -> String {
        let mut result = String::new();
        for (i, ch) in pascal.chars().enumerate() {
            if ch.is_uppercase() && i > 0 { result.push('_'); }
            result.push(ch.to_lowercase().next().unwrap());
        }
        if Self::is_rust_keyword(&result) {
            result.push('_');
        }
        // Special cases where appending _ isn't idiomatic
        match result.as_str() {
            "type_" => "typ".into(),
            "trait_" => "trait_name".into(),
            _ => result,
        }
    }

    fn is_rust_keyword(s: &str) -> bool {
        matches!(s,
            "as" | "async" | "await" | "break" | "const" | "continue"
            | "crate" | "dyn" | "else" | "enum" | "extern" | "false"
            | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop"
            | "match" | "mod" | "move" | "mut" | "pub" | "ref"
            | "return" | "self" | "static" | "struct" | "super"
            | "trait" | "true" | "type" | "unsafe" | "use" | "where"
            | "while" | "yield" | "do" | "abstract" | "become"
            | "box" | "final" | "macro" | "override" | "priv"
            | "try" | "typeof" | "unsized" | "virtual"
        )
    }
}
