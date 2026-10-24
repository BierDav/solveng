// Operations in math: Typst's own operators, solveng's imported from `operations`, and explicit references.

use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }

#[test]
fn standard_signal_operations_are_offered_without_configuration() {
    let doc = "#graph($ tr $)";
    let result = complete(doc, &Context::default(), at(doc, "tr $") + 2).unwrap();
    assert!(result.items.iter().any(|i| i.label == "tri"));
}

#[test]
fn solveng_operations_complete_with_their_import() {
    // with solveng imported, its operation comes with `#import operations: …`
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: worksheet\n$a := tr$";
    let result = complete(doc, &Context::default(), at(doc, "tr$") + 2).unwrap();
    let tri = result.items.iter().find(|i| i.label == "tri").unwrap();
    assert_eq!(tri.insert, "tri($0)");
    assert!(tri.edits.iter().any(|e| e.text.contains("#import operations: tri")), "{:?}", tri.edits);
    // already imported: no edits
    let doc = "#import operations: tri\n#show: worksheet\n$a := tr$";
    let result = complete(doc, &Context::default(), at(doc, "tr$") + 2).unwrap();
    assert!(result.items.iter().find(|i| i.label == "tri").unwrap().edits.is_empty());
    // nowhere to import it to: the explicit reference
    let doc = "#show: worksheet\n$a := tr$";
    let result = complete(doc, &Context::default(), at(doc, "tr$") + 2).unwrap();
    assert_eq!(result.items.iter().find(|i| i.label == "tri").unwrap().insert, "op(\"tri\")($0)");
}

#[test]
fn typst_operator_bindings_complete_as_calculation_aliases() {
    let doc = "#let bump = math.op(\"tri\")\n#graph($ bump(x) $)";
    let ctx = Context::default();
    assert!(definition(doc, &ctx, at(doc, "bump(x)")).is_none());
    for constructor in ["math.op(\"tri\")", "operation(\"tri\")"] {
        let doc = format!("#let bump = {constructor}\n#graph($ bu $)");
        let result = complete(&doc, &ctx, at(&doc, "bu $") + 2).unwrap();
        assert!(result.items.iter().any(|i| i.label == "bump" && i.insert == "bump($0)"), "{constructor}");
    }
}

#[test]
fn default_sum_completion_keeps_math_attachment_syntax() {
    let doc = "#show: worksheet\n$a := su$";
    let result = complete(doc, &Context::default(), at(doc, "su$") + 2).unwrap();
    let sum = result.items.iter().find(|i| i.label == "sum").unwrap();
    assert!(sum.insert.starts_with("sum_("), "{}", sum.insert);
}

#[test]
fn operations_highlight_by_their_binding() {
    let doc = "#import operations: tri\n#show: worksheet\n#let triangle = math.op(\"tri\")\n#let pulse = operation(\"rect\")\n$y := tri(x)$\n$z := triangle(x)$\n$v := pulse(x)$\n$u := sin(x)$\n$w := \"tri\"(x)$\n$q := op(\"tri\")(x)$";
    let tokens = document_tokens(doc, &Context::default());
    let kind = |needle: &str| tokens.iter().find(|t| t.start == at(doc, needle)).map(|t| t.kind);
    assert_eq!(kind("tri(x)"), Some(TokenKind::BUILTIN_FUNCTION));
    assert_eq!(kind("triangle(x)"), Some(TokenKind::BUILTIN_FUNCTION));
    assert_eq!(kind("pulse(x)"), Some(TokenKind::BUILTIN_FUNCTION));
    // Typst's own math operators need no binding
    assert_eq!(kind("sin(x)"), Some(TokenKind::BUILTIN_FUNCTION));
    // quoted text is a worksheet name, never an operation
    assert_eq!(kind("\"tri\"(x)"), Some(TokenKind::FUNCTION_CALL));
    // the explicit reference's name
    assert_eq!(tokens.iter().find(|t| t.start == at(doc, "tri\")(x)") ).map(|t| t.kind), Some(TokenKind::BUILTIN_FUNCTION));
    let completed = complete(doc, &Context::default(), at(doc, "tri\")(x)") + 2).unwrap();
    assert!(completed.items.iter().any(|i| i.label == "tri" && i.insert == "tri"));
    assert_eq!(completed.to, Some(at(doc, "tri\")(x)") + 3));
    // without the import solveng's operation is a user call
    let bare = doc.replace("#import operations: tri\n", "");
    let ts = document_tokens(&bare, &Context::default());
    assert_eq!(ts.iter().find(|t| t.start == at(&bare, "tri(x)")).unwrap().kind, TokenKind::FUNCTION_CALL, "{ts:?}");
    // math that isn't calculated has no operations (the import still names one)
    let inactive = doc.replace("#show: worksheet", "#show: worksheet.with(evaluate: false)");
    let math = at(&inactive, "$");
    assert!(document_tokens(&inactive, &Context::default()).iter().all(|t| t.start < math || t.kind != TokenKind::BUILTIN_FUNCTION));
}

#[test]
fn bindings_follow_typst_scopes() {
    let scoped = "#show: worksheet\n#let pulse = math.op(\"rect\")\n#[\n#import operations: tri\n$a := tri(0)$ $b := pulse(0)$\n]\n$c := tri(0)$\n#let pulse = 3\n$d := pulse(0)$";
    let ts = document_tokens(scoped, &Context::default());
    for (needle, expected) in [("tri(0)$ $b", TokenKind::BUILTIN_FUNCTION), ("pulse(0)$\n]", TokenKind::BUILTIN_FUNCTION), ("tri(0)$\n#let", TokenKind::FUNCTION_CALL)] {
        assert_eq!(ts.iter().find(|t| t.start == at(scoped, needle)).unwrap().kind, expected, "{needle}");
    }
    // shadowed by a plain value
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

#[test]
fn every_operation_of_the_operations_module_is_an_imported_operation() {
    // read from the package, so new operations (Re, conj, …) need no editor change
    let doc = "#import operations: Re, conj\n#show: worksheet\n$a := Re(z) + conj(z)$";
    let tokens = document_tokens(doc, &Context::default());
    for needle in ["Re(z)", "conj(z)"] {
        assert_eq!(tokens.iter().find(|t| t.start == at(doc, needle)).map(|t| t.kind), Some(TokenKind::BUILTIN_FUNCTION), "{needle}");
    }
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: worksheet\n$a := con$";
    let item = complete(doc, &Context::default(), at(doc, "con$") + 3).unwrap().items.into_iter().find(|i| i.label == "conj").unwrap();
    assert!(item.edits.iter().any(|e| e.text.contains("#import operations: conj")));
}
