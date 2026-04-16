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
                assert_eq!(e.variants, vec!["Type", "Variant", "Field"]);
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
                        assert_eq!(typ, "U32");
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
;; name.aski — name classification

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
        match &module.domains[1] {
            Domain::Enum(e) => {
                assert_eq!(e.name, "Operator");
                assert_eq!(e.variants.len(), 12);
            }
            _ => panic!("expected enum"),
        }
    }

    #[test]
    fn parse_full_span_file() {
        let source = r#"
;; span.aski — source position

(Span Span)

{Span (Start U32) (End U32)}
"#;
        let module = parse(source);
        assert_eq!(module.name, "Span");
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
;; scope.aski — scope classification

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
}
