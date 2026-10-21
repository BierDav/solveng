use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }

#[test]
fn standard_signal_functions_are_offered_without_configuration() {
    let doc = "#graph($ tr $)";
    let result = complete(doc, &Context::default(), at(doc, "tr $") + 2).unwrap();
    assert!(result.items.iter().any(|i| i.label == "tri"));
}

#[test]
fn typst_operator_bindings_do_not_become_calculation_aliases() {
    let doc = "#let bump = math.op(\"tri\")\n#graph($ bump(x) $)";
    let ctx = Context::default();
    assert!(definition(doc, &ctx, at(doc, "bump(x)")).is_none());
    let doc = "#let bump = math.op(\"tri\")\n#graph($ bu $)";
    let result = complete(doc, &ctx, at(doc, "bu $") + 2).unwrap();
    assert!(!result.items.iter().any(|i| i.label == "bump(x)"));
}

#[test]
fn default_sum_completion_keeps_math_attachment_syntax() {
    let doc = "#show: worksheet\n$a := su$";
    let result = complete(doc, &Context::default(), at(doc, "su$") + 2).unwrap();
    let sum = result.items.iter().find(|i| i.label == "sum").unwrap();
    assert!(sum.insert.starts_with("sum_("), "{}", sum.insert);
}
