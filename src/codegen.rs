/// corec codegen — emits Rust source from parsed .aski domains.
///
/// Enums → Rust enums with rkyv + standard derives.
/// Structs → Rust structs with rkyv + standard derives.
/// Type applications → Rust generics.
/// Names stay PascalCase. Fields get snake_case.

use crate::parse::{Module, Domain, EnumDef, EnumVariant, StructDef, StructField, TypeExpr};

const COPY_DERIVES: &str = "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]";

// Non-copy types get serialize/deserialize bounds to handle recursive types
// (Box<T> fields). The bounds are harmless on non-recursive types.
const DATA_DERIVES: &str = "#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source))]
#[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]";

struct Codegen {
    out: String,
}

impl Codegen {
    fn new() -> Self {
        Codegen { out: String::new() }
    }

    fn emit_module(&mut self, module: &Module) {
        for domain in &module.domains {
            match domain {
                Domain::Enum(def) => self.emit_enum(def),
                Domain::Struct(def) => self.emit_struct(def),
            }
            self.out.push('\n');
        }
    }

    fn emit_enum(&mut self, def: &EnumDef) {
        let has_data = def.variants.iter().any(|v| !matches!(v, EnumVariant::Bare(_)));
        if has_data {
            self.out.push_str(DATA_DERIVES);
        } else {
            self.out.push_str(COPY_DERIVES);
        }
        self.out.push_str(&format!("\npub enum {} {{\n", def.name));
        for variant in &def.variants {
            self.emit_variant(variant);
        }
        self.out.push_str("}\n");
    }

    fn emit_variant(&mut self, variant: &EnumVariant) {
        match variant {
            EnumVariant::Bare(name) => {
                self.out.push_str(&format!("    {},\n", name));
            }
            EnumVariant::Data { name, payload } => {
                if Self::needs_omit_bounds(payload) {
                    self.out.push_str(&format!("    {}(#[rkyv(omit_bounds)] {}),\n",
                        name, self.type_to_rust(payload)));
                } else {
                    self.out.push_str(&format!("    {}({}),\n",
                        name, self.type_to_rust(payload)));
                }
            }
            EnumVariant::Struct(def) => {
                self.out.push_str(&format!("    {} {{\n", def.name));
                for field in &def.fields {
                    self.emit_field(field, "        ", false);
                }
                self.out.push_str("    },\n");
            }
        }
    }

    fn emit_struct(&mut self, def: &StructDef) {
        self.out.push_str(DATA_DERIVES);
        self.out.push_str(&format!("\npub struct {} {{\n", def.name));
        for field in &def.fields {
            self.emit_field(field, "    ", true);
        }
        self.out.push_str("}\n");
    }

    fn emit_field(&mut self, field: &StructField, indent: &str, pub_prefix: bool) {
        let pub_str = if pub_prefix { "pub " } else { "" };
        match field {
            StructField::Typed { name, typ } => {
                if Self::needs_omit_bounds(typ) {
                    self.out.push_str(&format!(
                        "{}#[rkyv(omit_bounds)]\n{}{}{}: {},\n",
                        indent, indent, pub_str, to_snake_case(name), self.type_to_rust(typ)
                    ));
                } else {
                    self.out.push_str(&format!(
                        "{}{}{}: {},\n", indent, pub_str, to_snake_case(name), self.type_to_rust(typ)
                    ));
                }
            }
            StructField::SelfTyped { name } => {
                self.out.push_str(&format!(
                    "{}{}{}: {},\n", indent, pub_str, to_snake_case(name), name
                ));
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

    fn type_to_rust(&self, typ: &TypeExpr) -> String {
        match typ {
            TypeExpr::Simple(name) => map_primitive(name).to_string(),
            TypeExpr::Application { constructor, args } => {
                let args_rust: Vec<String> = args.iter()
                    .map(|a| self.type_to_rust(a))
                    .collect();
                format!("{}<{}>", constructor, args_rust.join(", "))
            }
        }
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

fn to_snake_case(pascal: &str) -> String {
    let mut result = String::new();
    for (i, ch) in pascal.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    // Escape Rust reserved words
    match result.as_str() {
        "type" => "typ".to_string(),
        "trait" => "trait_name".to_string(),
        "self" => "self_".to_string(),
        "match" => "match_".to_string(),
        "loop" => "loop_".to_string(),
        "return" => "return_".to_string(),
        "move" => "move_".to_string(),
        "ref" => "ref_".to_string(),
        "mut" => "mut_".to_string(),
        _ => result,
    }
}

pub fn emit_module(module: &Module) -> String {
    let mut codegen = Codegen::new();
    codegen.emit_module(module);
    codegen.out
}
