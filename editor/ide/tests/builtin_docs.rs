use solveng_ide::{Context, complete, hover, params};

fn offset(doc: &str, text: &str) -> u32 {
    doc[..doc.find(text).unwrap()].encode_utf16().count() as u32
}

#[test]
fn every_builtin_has_hover_documentation_in_math() {
    for name in solveng_names::OPERATIONS {
        for doc in [
            format!("#import operations: *\n#show: math\n$f(t) := {name}(t)$"),
            format!("#import operations: *\n#graph(${name}(t)$)"),
            format!("#show: math\n$f(t) := op(\"{name}\")(t)$"),
        ] {
            let at = doc.find(&format!("{name}\"")).or_else(|| doc.find(&format!("{name}(t)"))).unwrap();
            let result = hover(&doc, &Context::default(), offset(&doc, &doc[at..]) + 1).unwrap_or_else(|| panic!("{doc}"));
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
        ("abs", "preserves x's unit"), ("ln", "unitless and not 0"),
        ("diff", "Differentiation variable"), ("integrate", "Lower bound"),
        ("sum", "Inclusive upper integer bound"), ("at", "starting at 0"),
    ] {
        let doc = format!("#import operations: *\n#graph(${name}(t)$)");
        let result = hover(&doc, &Context::default(), offset(&doc, &format!("{name}(t)")) + 1).unwrap_or_else(|| panic!("{doc}"));
        assert!(result.markdown.contains(expected), "{name}: {}", result.markdown);
    }
}

#[test]
fn builtin_completions_carry_documentation_and_keep_math_snippets() {
    for (prefix, name) in [("tr", "tri"), ("in", "integrate"), ("su", "sum"), ("as", "asin")] {
        for doc in [format!("#show: math\n$x := {prefix}$"), format!("#graph(${prefix}$)")] {
            let result = complete(&doc, &Context::default(), offset(&doc, &format!("{prefix}$")) + prefix.len() as u32).unwrap();
            let item = result.items.iter().find(|i| i.label == name).unwrap();
            assert!(item.documentation.as_ref().unwrap().contains("### Parameters"), "{name}");
            if name == "sum" { assert!(item.insert.starts_with("sum_("), "{}", item.insert); }
        }
    }
}

#[test]
fn mathscope_option_has_documentation_values_and_isolated_default() {
    let math = params::functions().iter().find(|f| f.name == "mathscope").unwrap();
    assert!(!math.named.iter().any(|p| p.name == "inherit" || p.name == "export"));
    // the unit table and the automatic lookups are gone
    assert!(!math.named.iter().any(|p| matches!(p.name.as_str(), "units" | "builtin-units" | "builtin-functions")));
    let scope = math.named.iter().find(|p| p.name == "scope").unwrap();
    assert_eq!(scope.default, "\"isolated\"");
    assert_eq!(scope.values, vec!["isolated", "inherit", "shared"]);
    assert!(scope.doc.contains("isolated"));
}

#[test]
fn attached_sum_notation_has_the_same_documentation() {
    let doc = "#import operations: tri\n#show: math\n$f(t) := sum_(k=0)^infinity tri(t-k)$";
    let result = hover(doc, &Context::default(), offset(doc, "sum_") + 1).unwrap();
    assert!(result.markdown.contains("Inclusive integer sum"));
    assert!(result.markdown.contains("### Parameters"));
}

#[test]
fn adjacent_coefficients_do_not_hide_symbol_documentation() {
    let imports = "#import constants: pi, e\n";
    for (expression, name, description) in [("2pi", "pi", "constant"), ("12infinity", "infinity", "infinite"), ("1.5#e^2", "e", "euler"), ("2.5 #e^(-10)", "e", "euler")] {
        for doc in [format!("{imports}#show: math\n${expression}$"), format!("{imports}#value(${expression}$)")] {
            let at = offset(&doc, expression) + offset(expression, name);
            for caret in [at, at + name.len() as u32] {
                let result = hover(&doc, &Context::default(), caret).unwrap_or_else(|| panic!("{doc} at {caret}"));
                assert_eq!((result.start, result.end), (at, at + name.len() as u32), "{doc}");
                assert!(result.markdown.contains(description), "{}", result.markdown);
            }
        }
    }
    // no e-notation: `1e10` is a coefficient and the name e10, which has no documentation
    for doc in ["#value($1e10$)", "#show: math\n$3e4$"] {
        let at = doc[..doc.rfind('e').unwrap()].encode_utf16().count() as u32;
        assert!(hover(doc, &Context::default(), at).is_none());
        assert!(hover(doc, &Context::default(), at + 1).is_none());
    }
}
