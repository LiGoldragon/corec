/// corec codegen — domain definitions → Rust source with rkyv derives.
///
/// All methods on Codegen struct.

use crate::parse::{Module, Domain, EnumDef, EnumVariant, StructDef, StructField, TypeExpr};

pub struct Codegen {
    out: String,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen { out: String::new() }
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
                if Self::needs_omit_bounds(payload) {
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
                if Self::needs_omit_bounds(typ) {
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
            TypeExpr::Simple(name) => Self::map_primitive(name).to_string(),
            TypeExpr::Application { constructor, args } => {
                let args_rust: Vec<String> = args.iter()
                    .map(|a| self.type_to_rust(a))
                    .collect();
                format!("{}<{}>", constructor, args_rust.join(", "))
            }
        }
    }

    fn needs_omit_bounds(typ: &TypeExpr) -> bool {
        match typ {
            TypeExpr::Simple(_) => false,
            TypeExpr::Application { constructor, args } => {
                constructor == "Box"
                    || constructor == "Vec"
                    || constructor == "Option"
                    || args.iter().any(|a| Self::needs_omit_bounds(a))
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

    fn map_primitive(name: &str) -> &str {
        match name {
            "U8" => "u8", "U16" => "u16", "U32" => "u32", "U64" => "u64",
            "I8" => "i8", "I16" => "i16", "I32" => "i32", "I64" => "i64",
            "F32" => "f32", "F64" => "f64",
            "Bool" => "bool", "String" => "String",
            other => other,
        }
    }

    fn to_snake(pascal: &str) -> String {
        let mut result = String::new();
        for (i, ch) in pascal.chars().enumerate() {
            if ch.is_uppercase() && i > 0 { result.push('_'); }
            result.push(ch.to_lowercase().next().unwrap());
        }
        match result.as_str() {
            "type" => "typ".into(),
            "trait" => "trait_name".into(),
            "self" => "self_".into(),
            "match" => "match_".into(),
            "loop" => "loop_".into(),
            "return" => "return_".into(),
            "move" => "move_".into(),
            "ref" => "ref_".into(),
            "mut" => "mut_".into(),
            _ => result,
        }
    }
}
