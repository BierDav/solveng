use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }

#[test]
fn math_operators_resolve_typst_bindings_in_all_symbol_spellings() {
    for spelling in ["ΛΩ", "Lambda\"\"Omega", "LambdaΩ", "ΛOmega"] {
        let doc = format!("#let ΛΩ = math.op(\"tri\")\n#graph($ {spelling}(x) $)");
        let ctx = Context::default();
        let offset = at(&doc, &format!("{spelling}(x)"));
        assert_eq!(definition(&doc, &ctx, offset).unwrap().start, 5, "{spelling}");
        assert_eq!(usages(&doc, &ctx, offset).len(), 2);
        let edits = rename(&doc, &ctx, offset, "dreieick");
        assert_eq!(edits.len(), 2);
        assert!(edits.iter().all(|e| e.text == "dreieick"));
        let compiled = operator_edits(&doc);
        assert_eq!(compiled.len(), 2);
        assert!(compiled.iter().all(|e| e.name == "ΛΩ"));
        assert_eq!(compiled.iter().filter(|e| e.math).count(), 1);
    }
}

#[test]
fn operator_aliases_do_not_expose_the_operation_name() {
    let doc = "#let dreieick = math.op(\"tri\")\n#graph($ dreieick(x) $)\n#graph($ tri(x) $)\n#graph($ Lambda(x) $)";
    let ctx = Context::default();
    assert_eq!(usages(doc, &ctx, 6).len(), 2);
    assert!(definition(doc, &ctx, at(doc, "tri(x)")).is_none());
    assert!(definition(doc, &ctx, at(doc, "Lambda(x)")).is_none());
    assert_eq!(operator_edits(doc).iter().filter(|e| e.math).count(), 1);
}

#[test]
fn quoted_calculation_names_are_separate_from_typst_bindings() {
    let doc = "#let ΛΩ = math.op(\"tri\")\n#show: worksheet\n$\"ΛΩ\" := 3$\n$y := \"ΛΩ\"$\n#graph($Lambda\"\"Omega(x)$)";
    let ctx = Context::default();
    assert_eq!(usages(doc, &ctx, 5).len(), 2);
    assert_eq!(usages(doc, &ctx, at(doc, "\"ΛΩ\" :=")).len(), 2);
    assert!(rename(doc, &ctx, 5, "2x").is_empty());
}

#[test]
fn undeclared_signal_functions_are_not_offered_in_math() {
    let doc = "#graph($ tr $)";
    let result = complete(doc, &Context::default(), at(doc, "tr $") + 2).unwrap();
    assert!(!result.items.iter().any(|i| i.label == "tri"));
    let doc = "#let dreieick = math.op(\"tri\")\n#graph($ dr $)";
    let result = complete(doc, &Context::default(), at(doc, "dr $") + 2).unwrap();
    assert!(result.items.iter().any(|i| i.label == "dreieick(x)" && i.insert == "dreieick($0)"));
}

#[test]
fn operators_inside_closures_follow_typst_parameter_shadowing() {
    let doc = "#let ΛΩ = math.op(\"tri\")\n#let f(x) = $Lambda\"\"Omega(x)$\n#let g(ΛΩ) = $Lambda\"\"Omega(x)$\n#graph($Lambda\"\"Omega(x)$)";
    let edits = operator_edits(doc);
    assert_eq!(edits.iter().filter(|e| e.math).count(), 2);
}

#[test]
fn symbol_operator_completion_inserts_a_typst_reference() {
    let doc = "#let ΛΩ = math.op(\"tri\")\n#graph($ Lambda\"\"Om $)";
    let result = complete(doc, &Context::default(), at(doc, "Om $") + 2).unwrap();
    assert!(result.items.iter().any(|i| i.label == "ΛΩ(x)" && i.insert == "ΛΩ($0)"), "{result:?}");
}
