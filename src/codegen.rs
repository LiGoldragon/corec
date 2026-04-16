/// cc codegen — emits Rust source from parsed .aski domains.
///
/// Enums → Rust enums with derive macros.
/// Structs → Rust structs with derive macros.
/// Names stay PascalCase (they already are).
/// Fields get snake_case conversion for Rust convention.

use crate::parse::{Module, Domain, EnumDef, StructDef, StructField};

pub fn emit_module(module: &Module) -> String {
    let mut out = String::new();


    for domain in &module.domains {
        match domain {
            Domain::Enum(def) => emit_enum(&mut out, def),
            Domain::Struct(def) => emit_struct(&mut out, def),
        }
        out.push('\n');
    }

    out
}

fn emit_enum(out: &mut String, def: &EnumDef) {
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    out.push_str(&format!("pub enum {} {{\n", def.name));
    for variant in &def.variants {
        out.push_str(&format!("    {},\n", variant));
    }
    out.push_str("}\n");
}

fn emit_struct(out: &mut String, def: &StructDef) {
    out.push_str("#[derive(Debug, Clone, PartialEq)]\n");
    out.push_str(&format!("pub struct {} {{\n", def.name));
    for field in &def.fields {
        match field {
            StructField::Typed { name, typ } => {
                let rust_name = to_snake_case(name);
                let rust_type = map_type(typ);
                out.push_str(&format!("    pub {}: {},\n", rust_name, rust_type));
            }
            StructField::SelfTyped { name } => {
                let rust_name = to_snake_case(name);
                out.push_str(&format!("    pub {}: {},\n", rust_name, name));
            }
        }
    }
    out.push_str("}\n");
}

fn map_type(aski_type: &str) -> &str {
    match aski_type {
        "U8" => "u8",
        "U16" => "u16",
        "U32" => "u32",
        "U64" => "u64",
        "I8" => "i8",
        "I16" => "i16",
        "I32" => "i32",
        "I64" => "i64",
        "F32" => "f32",
        "F64" => "f64",
        "Bool" => "bool",
        "String" => "String",
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
    result
}
