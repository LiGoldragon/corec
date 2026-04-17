#[cfg(test)]
mod tests {
    use crate::codegen::Codegen;
    use crate::parse::{Module, Domain, EnumDef, EnumVariant, StructDef, StructField, TypeExpr};

    fn module(name: &str, domains: Vec<Domain>) -> Module {
        Module { name: name.into(), exports: vec![], domains }
    }

    fn gen(domains: Vec<Domain>) -> String {
        Codegen::new().emit_module(&module("Test", domains))
    }

    // ── to_snake keyword escaping ───────────────────────────

    #[test]
    fn to_snake_rust_keywords() {
        // Struct with fields named after Rust keywords
        let s = StructDef {
            name: "S".into(),
            fields: vec![
                StructField::Typed { name: "Const".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "Fn".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "Let".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "For".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "If".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "Async".into(), typ: TypeExpr::Simple("U32".into()) },
            ],
        };
        let out = gen(vec![Domain::Struct(s)]);
        assert!(out.contains("pub const_: u32"), "Const should become const_: {}", out);
        assert!(out.contains("pub fn_: u32"), "Fn should become fn_: {}", out);
        assert!(out.contains("pub let_: u32"), "Let should become let_: {}", out);
        assert!(out.contains("pub for_: u32"), "For should become for_: {}", out);
        assert!(out.contains("pub if_: u32"), "If should become if_: {}", out);
        assert!(out.contains("pub async_: u32"), "Async should become async_: {}", out);
    }

    #[test]
    fn to_snake_special_cases() {
        let s = StructDef {
            name: "S".into(),
            fields: vec![
                StructField::Typed { name: "Type".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "Trait".into(), typ: TypeExpr::Simple("U32".into()) },
            ],
        };
        let out = gen(vec![Domain::Struct(s)]);
        assert!(out.contains("pub typ: u32"), "Type should become typ: {}", out);
        assert!(out.contains("pub trait_name: u32"), "Trait should become trait_name: {}", out);
    }

    #[test]
    fn to_snake_normal_pascal_case() {
        let s = StructDef {
            name: "S".into(),
            fields: vec![
                StructField::Typed { name: "StartPos".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "Name".into(), typ: TypeExpr::Simple("String".into()) },
            ],
        };
        let out = gen(vec![Domain::Struct(s)]);
        assert!(out.contains("pub start_pos: u32"), "StartPos should become start_pos: {}", out);
        assert!(out.contains("pub name: String"), "Name should become name: {}", out);
    }

    // ── escape_variant ──────────────────────────────────────

    #[test]
    fn escape_variant_self_and_type() {
        let e = EnumDef {
            name: "E".into(),
            variants: vec![
                EnumVariant::Bare("Self".into()),
                EnumVariant::Bare("Type".into()),
                EnumVariant::Bare("Normal".into()),
            ],
        };
        let out = gen(vec![Domain::Enum(e)]);
        assert!(out.contains("Self_,"), "Self should become Self_: {}", out);
        assert!(out.contains("Type_,"), "Type should become Type_: {}", out);
        assert!(out.contains("Normal,"), "Normal unchanged: {}", out);
    }

    // ── omit_bounds ─────────────────────────────────────────

    #[test]
    fn omit_bounds_on_vec_and_option() {
        let s = StructDef {
            name: "S".into(),
            fields: vec![
                StructField::Typed {
                    name: "Items".into(),
                    typ: TypeExpr::Application {
                        constructor: "Vec".into(),
                        args: vec![TypeExpr::Simple("Item".into())],
                    },
                },
                StructField::Typed {
                    name: "Maybe".into(),
                    typ: TypeExpr::Application {
                        constructor: "Option".into(),
                        args: vec![TypeExpr::Simple("U32".into())],
                    },
                },
                StructField::Typed {
                    name: "Count".into(),
                    typ: TypeExpr::Simple("U32".into()),
                },
            ],
        };
        let out = gen(vec![Domain::Struct(s)]);
        assert!(out.contains("#[rkyv(omit_bounds)]\n    pub items: Vec<Item>"),
            "Vec should get omit_bounds: {}", out);
        assert!(out.contains("#[rkyv(omit_bounds)]\n    pub maybe: Option<u32>"),
            "Option should get omit_bounds: {}", out);
        assert!(!out.contains("omit_bounds")
            || out.matches("omit_bounds").count() == 2,
            "Count (u32) should NOT get omit_bounds");
    }

    // ── primitive mapping ───────────────────────────────────

    #[test]
    fn primitive_types_mapped() {
        let s = StructDef {
            name: "S".into(),
            fields: vec![
                StructField::Typed { name: "A".into(), typ: TypeExpr::Simple("U32".into()) },
                StructField::Typed { name: "B".into(), typ: TypeExpr::Simple("F64".into()) },
                StructField::Typed { name: "C".into(), typ: TypeExpr::Simple("Bool".into()) },
                StructField::Typed { name: "D".into(), typ: TypeExpr::Simple("Element".into()) },
            ],
        };
        let out = gen(vec![Domain::Struct(s)]);
        assert!(out.contains("pub a: u32"), "U32→u32: {}", out);
        assert!(out.contains("pub b: f64"), "F64→f64: {}", out);
        assert!(out.contains("pub c: bool"), "Bool→bool: {}", out);
        assert!(out.contains("pub d: Element"), "Element unchanged: {}", out);
    }

    // ── derives ─────────────────────────────────────────────

    #[test]
    fn bare_enum_gets_copy_derives() {
        let e = EnumDef {
            name: "Color".into(),
            variants: vec![
                EnumVariant::Bare("Red".into()),
                EnumVariant::Bare("Green".into()),
            ],
        };
        let out = gen(vec![Domain::Enum(e)]);
        assert!(out.contains("Copy"), "bare enum should get Copy: {}", out);
        assert!(out.contains("Hash"), "bare enum should get Hash: {}", out);
    }

    #[test]
    fn data_enum_gets_serialize_bounds() {
        let e = EnumDef {
            name: "Option".into(),
            variants: vec![
                EnumVariant::Data {
                    name: "Some".into(),
                    payload: TypeExpr::Simple("String".into()),
                },
                EnumVariant::Bare("None".into()),
            ],
        };
        let out = gen(vec![Domain::Enum(e)]);
        assert!(out.contains("serialize_bounds"), "data enum should get serialize_bounds: {}", out);
        assert!(!out.contains("Copy"), "data enum should NOT get Copy: {}", out);
    }
}
