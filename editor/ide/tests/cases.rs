use solveng_ide::{Context, complete, definition, operator_declaration};

fn offset(text: &str, needle: &str) -> u32 {
    text[..text.find(needle).unwrap()].encode_utf16().count() as u32
}

#[test]
fn case_functions_and_their_parameters_resolve_from_guards_and_graphs() {
    let doc = "#show: worksheet\n$sigma(t) := cases(0 & \"für\" t < 0, 1 & \"für\" t >= 0)$\n#graph($σ(x)$)";
    let context = Context::new("");
    let target = definition(doc, &context, offset(doc, "σ(x)")).unwrap();
    assert_eq!(target.start, offset(doc, "sigma"));
    for guard in ["t < 0", "t >= 0"] {
        let target = definition(doc, &context, offset(doc, guard)).unwrap();
        assert_eq!(target.start, offset(doc, "t)"));
    }
}

#[test]
fn cases_complete_with_the_syntax_for_the_current_mode() {
    for (doc, math) in [("#show: worksheet\n$x := cas", true), ("```solveng\nx := cas", false)] {
        let found = complete(doc, &Context::new(""), doc.encode_utf16().count() as u32).unwrap();
        let item = found.items.iter().find(|item| item.label == "cases").unwrap();
        assert!(item.insert.starts_with("cases("), "{}", item.insert);
        assert_eq!(item.insert.contains('&'), math, "{}", item.insert);
    }
    assert!(operator_declaration("cases").is_none());
}
