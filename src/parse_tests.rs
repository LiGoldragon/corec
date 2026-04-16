#[cfg(test)]
mod tests {
    use crate::lex;
    use crate::parse::*;

    fn parse(source: &str) -> Module {
        let tokens = lex::lex(source).unwrap();
        parse_file(tokens).unwrap()
    }

    #[test]
    fn parse_module_and_enum() {
        let module = parse("(Name NameDomain Operator)\n(NameDomain Type Variant Field)");
        assert_eq!(module.name, "Name");
        assert_eq!(module.exports, vec!["NameDomain", "Operator"]);
        assert_eq!(module.domains.len(), 1);
        match &module.domains[0] {
            Domain::Enum(e) => {
                assert_eq!(e.name, "NameDomain");
                assert_eq!(e.variants.len(), 3);
                assert!(matches!(&e.variants[0], EnumVariant::Bare(n) if n == "Type"));
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn parse_struct_typed_fields() {
        let module = parse("(Span Span)\n{Span (Start U32) (End U32)}");
        assert_eq!(module.domains.len(), 1);
        match &module.domains[0] {
            Domain::Struct(s) => {
                assert_eq!(s.name, "Span");
                assert_eq!(s.fields.len(), 2);
                match &s.fields[0] {
                    StructField::Typed { name, typ } => {
                        assert_eq!(name, "Start");
                        assert!(matches!(typ, TypeExpr::Simple(t) if t == "U32"));
                    }
                    _ => panic!("expected typed field"),
                }
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn parse_full_name_file() {
        let source = r#"
;; name.aski ��� name classification

(Name NameDomain Operator)

(NameDomain
  Type Variant Field Trait Method
  Module Literal TypeParam)

(Operator
  Add Sub Mul Mod
  Eq NotEq Lt Gt LtEq GtEq
  And Or)
"#;
        let module = parse(source);
        assert_eq!(module.name, "Name");
        assert_eq!(module.domains.len(), 2);
        match &module.domains[0] {
            Domain::Enum(e) => {
                assert_eq!(e.name, "NameDomain");
                assert_eq!(e.variants.len(), 8);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn parse_full_span_file() {
        let module = parse("(Span Span)\n{Span (Start U32) (End U32)}");
        assert_eq!(module.domains.len(), 1);
        match &module.domains[0] {
            Domain::Struct(s) => {
                assert_eq!(s.name, "Span");
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn parse_full_scope_file() {
        let source = r#"
(Scope ScopeKind Visibility)

(ScopeKind
  Root Module
  Enum Struct Newtype Trait TraitImpl
  Method Block MatchArm Loop Iteration)

(Visibility Exported Local)
"#;
        let module = parse(source);
        assert_eq!(module.name, "Scope");
        assert_eq!(module.domains.len(), 2);
        match &module.domains[0] {
            Domain::Enum(e) => {
                assert_eq!(e.name, "ScopeKind");
                assert_eq!(e.variants.len(), 12);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn parse_type_application() {
        let module = parse("(Test Foo)\n{Foo (Items [Vec Item]) (Count U32)}");
        match &module.domains[0] {
            Domain::Struct(s) => {
                assert_eq!(s.fields.len(), 2);
                match &s.fields[0] {
                    StructField::Typed { name, typ } => {
                        assert_eq!(name, "Items");
                        match typ {
                            TypeExpr::Application { constructor, args } => {
                                assert_eq!(constructor, "Vec");
                                assert_eq!(args.len(), 1);
                            }
                            _ => panic!("expected application"),
                        }
                    }
                    _ => panic!("expected typed field"),
                }
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn parse_nested_type_application() {
        let module = parse("(Test Foo)\n{Foo (Tail [Option [Box Expr]])}");
        match &module.domains[0] {
            Domain::Struct(s) => {
                match &s.fields[0] {
                    StructField::Typed { typ, .. } => {
                        match typ {
                            TypeExpr::Application { constructor, args } => {
                                assert_eq!(constructor, "Option");
                                assert!(matches!(&args[0], TypeExpr::Application { constructor, .. } if constructor == "Box"));
                            }
                            _ => panic!("expected application"),
                        }
                    }
                    _ => panic!("expected typed field"),
                }
            }
            _ => panic!("expected struct"),
        }
    }

    #[test]
    fn parse_data_carrying_variant() {
        let module = parse("(Test Foo)\n(Foo (Some String) None)");
        match &module.domains[0] {
            Domain::Enum(e) => {
                assert_eq!(e.variants.len(), 2);
                assert!(matches!(&e.variants[0], EnumVariant::Data { name, .. } if name == "Some"));
                assert!(matches!(&e.variants[1], EnumVariant::Bare(n) if n == "None"));
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn parse_struct_variant() {
        let module = parse("(Test Foo)\n(Foo {Bar (X U32) (Y U32)} Baz)");
        match &module.domains[0] {
            Domain::Enum(e) => {
                assert_eq!(e.variants.len(), 2);
                assert!(matches!(&e.variants[0], EnumVariant::Struct(s) if s.name == "Bar"));
                assert!(matches!(&e.variants[1], EnumVariant::Bare(n) if n == "Baz"));
            }
            _ => panic!("expected enum"),
        }
    }
}
