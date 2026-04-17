#[cfg(test)]
mod tests {
    use crate::primitive::Primitives;

    #[test]
    fn primitives_load() {
        let p = Primitives::load();
        assert!(!p.all.is_empty(), "primitives should not be empty");
    }

    #[test]
    fn scalar_types_registered() {
        let p = Primitives::load();
        for name in &["U8", "U16", "U32", "U64", "I8", "I16", "I32", "I64",
                       "F32", "F64", "Bool", "String", "Char"] {
            assert!(p.is_primitive(name), "{} should be primitive", name);
            assert_eq!(p.arity(name), None, "{} should have arity 0 (None)", name);
        }
    }

    #[test]
    fn generic_types_registered() {
        let p = Primitives::load();
        assert!(p.is_primitive("Vec"));
        assert_eq!(p.arity("Vec"), Some(1));
        assert!(p.is_primitive("Option"));
        assert_eq!(p.arity("Option"), Some(1));
        assert!(p.is_primitive("Box"));
        assert_eq!(p.arity("Box"), Some(1));
        assert!(p.is_primitive("Result"));
        assert_eq!(p.arity("Result"), Some(2));
    }

    #[test]
    fn omit_bounds_types() {
        let p = Primitives::load();
        assert!(p.needs_omit_bounds("Vec"));
        assert!(p.needs_omit_bounds("Option"));
        assert!(p.needs_omit_bounds("Box"));
        assert!(!p.needs_omit_bounds("Result"));
        assert!(!p.needs_omit_bounds("U32"));
    }

    #[test]
    fn rust_name_mapping() {
        let p = Primitives::load();
        assert_eq!(p.map_to_rust("U32"), "u32");
        assert_eq!(p.map_to_rust("I64"), "i64");
        assert_eq!(p.map_to_rust("F64"), "f64");
        assert_eq!(p.map_to_rust("Bool"), "bool");
        assert_eq!(p.map_to_rust("Char"), "char");
        assert_eq!(p.map_to_rust("String"), "String");
        // Non-primitive passes through
        assert_eq!(p.map_to_rust("Element"), "Element");
    }

    #[test]
    fn non_primitive_not_found() {
        let p = Primitives::load();
        assert!(!p.is_primitive("Element"));
        assert!(!p.is_primitive("MyType"));
        assert_eq!(p.arity("Element"), None);
    }
}
