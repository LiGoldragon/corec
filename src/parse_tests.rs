#[cfg(test)]
mod tests {
    use crate::lex::Lexer;
    use crate::parse::*;

    fn parse(source: &str) -> Module {
        let tokens = Lexer::new(source).lex().unwrap();
        Parser::new(tokens).parse_file().unwrap()
    }

    #[test]
    fn module_and_enum() {
        let m = parse("(Name NameDomain Operator)\n(NameDomain Type Variant Field)");
        assert_eq!(m.name, "Name");
        assert_eq!(m.exports, vec!["NameDomain", "Operator"]);
        assert_eq!(m.domains.len(), 1);
        match &m.domains[0] {
            Domain::Enum(e) => {
                assert_eq!(e.name, "NameDomain");
                assert_eq!(e.variants.len(), 3);
                assert!(matches!(&e.variants[0], EnumVariant::Bare(n) if n == "Type"));
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn struct_typed_fields() {
        let m = parse("(Span Span)\n{Span (Start U32) (End U32)}");
        match &m.domains[0] {
            Domain::Struct(s) => {
                assert_eq!(s.name, "Span");
                assert_eq!(s.fields.len(), 2);
                assert!(matches!(&s.fields[0], StructField::Typed { name, .. } if name == "Start"));
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn type_application() {
        let m = parse("(T F)\n{F (Items [Vec Item]) (Count U32)}");
        match &m.domains[0] {
            Domain::Struct(s) => match &s.fields[0] {
                StructField::Typed { typ, .. } => match typ {
                    TypeExpr::Application { constructor, args } => {
                        assert_eq!(constructor, "Vec");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("expected application"),
                }
                _ => panic!("expected typed field"),
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn nested_type_application() {
        let m = parse("(T F)\n{F (Tail [Option [Box Expr]])}");
        match &m.domains[0] {
            Domain::Struct(s) => match &s.fields[0] {
                StructField::Typed { typ, .. } => match typ {
                    TypeExpr::Application { constructor, args } => {
                        assert_eq!(constructor, "Option");
                        assert!(matches!(&args[0], TypeExpr::Application { constructor, .. } if constructor == "Box"));
                    }
                    _ => panic!("expected application"),
                }
                _ => panic!("expected typed field"),
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn data_carrying_variant() {
        let m = parse("(T F)\n(F (Some String) None)");
        match &m.domains[0] {
            Domain::Enum(e) => {
                assert!(matches!(&e.variants[0], EnumVariant::Data { name, .. } if name == "Some"));
                assert!(matches!(&e.variants[1], EnumVariant::Bare(n) if n == "None"));
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn struct_variant() {
        let m = parse("(T F)\n(F {Bar (X U32) (Y U32)} Baz)");
        match &m.domains[0] {
            Domain::Enum(e) => {
                assert!(matches!(&e.variants[0], EnumVariant::Struct(s) if s.name == "Bar"));
                assert!(matches!(&e.variants[1], EnumVariant::Bare(n) if n == "Baz"));
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn self_typed_field() {
        let m = parse("(T F)\n{F (Count U32) Name}");
        match &m.domains[0] {
            Domain::Struct(s) => {
                assert!(matches!(&s.fields[0], StructField::Typed { name, .. } if name == "Count"));
                assert!(matches!(&s.fields[1], StructField::SelfTyped { name } if name == "Name"));
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn full_dialect_aski() {
        let source = std::fs::read_to_string(
            std::env::var("CARGO_MANIFEST_DIR").unwrap() + "/../aski-core/core/dialect.aski"
        );
        if let Ok(source) = source {
            let m = parse(&source);
            assert_eq!(m.name, "Dialect");
            assert!(m.domains.len() >= 10);
        }
        // Skip if file not found (CI without sibling repos)
    }
}
