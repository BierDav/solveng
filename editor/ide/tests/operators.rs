use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }

#[test]
fn standard_signal_functions_are_offered_without_configuration() {
    let doc = "#graph($ tr $)";
    let result = complete(doc, &Context::default(), at(doc, "tr $") + 2).unwrap();
    assert!(result.items.iter().any(|i| i.label == "tri"));
}

#[test]
fn typst_operator_bindings_complete_as_calculation_aliases() {
    let doc = "#let bump = math.op(\"tri\")\n#graph($ bump(x) $)";
    let ctx = Context::default();
    assert!(definition(doc, &ctx, at(doc, "bump(x)")).is_none());
    let doc = "#let bump = math.op(\"tri\")\n#graph($ bu $)";
    let result = complete(doc, &ctx, at(doc, "bu $") + 2).unwrap();
    assert!(result.items.iter().any(|i| i.label == "bump" && i.insert == "bump($0)"));
}

#[test]
fn default_sum_completion_keeps_math_attachment_syntax() {
    let doc = "#show: worksheet\n$a := su$";
    let result = complete(doc, &Context::default(), at(doc, "su$") + 2).unwrap();
    let sum = result.items.iter().find(|i| i.label == "sum").unwrap();
    assert!(sum.insert.starts_with("sum_("), "{}", sum.insert);
}

#[test]
fn explicit_operators_complete_and_highlight_with_builtins_disabled() {
    let doc = "#show: worksheet.with(builtin-functions: false)\n#let triangle = math.op(\"tri\")\n$y := tri(x)$\n$z := triangle(x)$\n$w := \"tri\"(x)$";
    let tokens = document_tokens(doc, &Context::default());
    let kind = |needle: &str| tokens.iter().find(|t| t.start == at(doc, needle)).map(|t| t.kind);
    assert_eq!(kind("tri\""), Some(TokenKind::BUILTIN_FUNCTION));
    assert_eq!(kind("tri(x)"), Some(TokenKind::FUNCTION_CALL));
    let native = doc.replace("tri(x)", "sin(x)");
    assert_eq!(document_tokens(&native, &Context::default()).iter().find(|t| t.start == at(&native, "sin(x)")).unwrap().kind, TokenKind::BUILTIN_FUNCTION);
    assert_eq!(kind("triangle(x)"), Some(TokenKind::BUILTIN_FUNCTION));
    assert_eq!(kind("\"tri\"(x)"), Some(TokenKind::FUNCTION_CALL));
    let completed = complete(doc, &Context::default(), at(doc, "tri\"") + 2).unwrap();
    assert!(completed.items.iter().any(|i| i.label == "tri" && i.insert == "tri"));
    assert_eq!(completed.to, Some(at(doc, "tri\"") + 3));
    let inactive = doc.replace("#show: worksheet.with(builtin-functions: false)", "#show: worksheet.with(evaluate: false)");
    assert!(document_tokens(&inactive, &Context::default()).iter().all(|t| t.kind != TokenKind::BUILTIN_FUNCTION));
    let scoped = "#show: worksheet\n#let pulse = math.op(\"rect\")\n#worksheet(builtin-functions: false)[$a := tri(0)$ $b := pulse(0)$]\n$c := tri(0)$\n#let pulse = 3\n$d := pulse(0)$";
    let ts = document_tokens(scoped, &Context::default());
    for (needle, expected) in [("tri(0)$ $b", TokenKind::FUNCTION_CALL), ("pulse(0)$]", TokenKind::BUILTIN_FUNCTION), ("tri(0)$\n#let", TokenKind::BUILTIN_FUNCTION)] {
        assert_eq!(ts.iter().find(|t| t.start == at(scoped, needle)).unwrap().kind, expected, "{needle}");
    }
    let shadowed = scoped.rfind("pulse(0)").unwrap() as u32;
    assert_eq!(ts.iter().find(|t| t.start == shadowed).unwrap().kind, TokenKind::VARIABLE);
}

#[test]
fn single_letter_operator_bindings_need_an_explicit_hash() {
    let doc = "#show: worksheet\n#let b = math.op(\"tri\")\n$a := b$";
    let result = complete(doc, &Context::default(), at(doc, "b$") + 1).unwrap();
    assert_eq!(result.items.iter().find(|i| i.label == "b").unwrap().insert, "#b($0)");
    let doc = doc.replace("a := b", "a := #b");
    let result = complete(&doc, &Context::default(), at(&doc, "b$") + 1).unwrap();
    assert_eq!(result.items.iter().find(|i| i.label == "b").unwrap().insert, "b($0)");
    assert!(document_tokens(&doc, &Context::default()).iter().any(|t| t.start == at(&doc, "b$") && t.kind == TokenKind::BUILTIN_FUNCTION));
}
