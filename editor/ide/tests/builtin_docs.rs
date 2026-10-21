use solveng_ide::{Context, complete, hover, params};

fn offset(doc: &str, text: &str) -> u32 {
    doc[..doc.find(text).unwrap()].encode_utf16().count() as u32
}

#[test]
fn every_builtin_has_hover_documentation_in_math_code_and_strings() {
    for name in solveng_names::OPERATIONS {
        for doc in [
            format!("#show: worksheet\n$f(t) := {name}(t)$"),
            format!("```solveng\nf(t) := {name}(t)\n```"),
            format!("#graph(\"{name}(t)\")"),
        ] {
            let result = hover(&doc, &Context::default(), offset(&doc, &format!("{name}(t)")) + 1).unwrap();
            assert!(result.markdown.contains("### Parameters"), "{name}: {}", result.markdown);
            assert!(!result.markdown.ends_with("built-in function"), "{name}: {}", result.markdown);
        }
    }
    // Calculation operations must not be offered as callable Typst package functions.
    assert!(params::functions().iter().all(|f| !solveng_names::OPERATIONS.contains(&f.name.as_str())));
}

#[test]
fn builtin_docs_include_domains_units_boundaries_and_calculus_parameters() {
    for (name, expected) in [
        ("rect", "½ at x = ±½"), ("tri", "max(1 − |x|, 0)"),
        ("asin", "[−1, 1]"), ("arccos", "[−1, 1]"),
        ("abs", "preserves x's unit"), ("ln", "positive and unitless"),
        ("diff", "Differentiation variable"), ("integrate", "Lower bound"),
        ("sum", "Inclusive upper integer bound"), ("at", "starting at 0"),
    ] {
        let doc = format!("#graph(\"{name}(t)\")");
        let result = hover(&doc, &Context::default(), offset(&doc, name) + 1).unwrap();
        assert!(result.markdown.contains(expected), "{name}: {}", result.markdown);
    }
}

#[test]
fn builtin_completions_carry_documentation_and_keep_math_snippets() {
    for (prefix, name) in [("tr", "tri"), ("in", "integrate"), ("su", "sum"), ("as", "asin")] {
        for doc in [format!("#show: worksheet\n$x := {prefix}$"), format!("#graph(\"{prefix}\")")] {
            let result = complete(&doc, &Context::default(), offset(&doc, &format!("{prefix}{}", if doc.ends_with('$') { "$" } else { "\"" })) + prefix.len() as u32).unwrap();
            let item = result.items.iter().find(|i| i.label == name).unwrap();
            assert!(item.documentation.as_ref().unwrap().contains("### Parameters"), "{name}");
            if name == "sum" && doc.ends_with('$') { assert!(item.insert.starts_with("sum_(")); }
        }
    }
}

#[test]
fn worksheet_export_option_has_documentation_and_auto_default() {
    let worksheet = params::functions().iter().find(|f| f.name == "worksheet").unwrap();
    let export = worksheet.named.iter().find(|p| p.name == "export").unwrap();
    assert_eq!(export.default, "auto");
    assert!(export.types.contains("bool"));
    assert!(export.doc.contains("no for content blocks"));
}

#[test]
fn attached_sum_notation_has_the_same_documentation() {
    let doc = "#show: worksheet\n$f(t) := sum_(k=0)^infinity tri(t-k)$";
    let result = hover(doc, &Context::default(), offset(doc, "sum_") + 1).unwrap();
    assert!(result.markdown.contains("Inclusive integer sum"));
    assert!(result.markdown.contains("### Parameters"));
}
