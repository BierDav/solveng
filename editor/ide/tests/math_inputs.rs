use solveng_ide::*;

fn offset(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }

#[test]
fn expression_strings_have_no_editor_language() {
    for name in ["worksheet", "value", "graph", "lambdify"] {
        let doc = format!("#{name}(\"x := 2 s\")");
        let at = offset(&doc, "2 s") + 2;
        assert!(document_tokens(&doc, &Context::default()).is_empty(), "{doc}");
        assert!(complete(&doc, &Context::default(), at).is_none(), "{doc}");
        assert!(conceals(&doc, &Context::default()).is_empty(), "{doc}");
        assert!(format(&doc, &Context::default()).is_empty(), "{doc}");
        assert!(regions(&doc, &Context::default()).iter().all(|r| r.kind != "string"), "{doc}");
    }
}

#[test]
fn math_unit_dictionaries_keep_nested_calls_and_all_entries() {
    let doc = "#import units: cm, s as sec\n#show: worksheet.with(units: (\n  inch: $2.54cm$,\n  speed: $frac(2 unit(\"m\"), 4#sec)$,\n  // Last entry follows parentheses and a comment.\n  foot: $30.48cm$,\n))\n$x := 1 $";
    let at = offset(doc, "$x := 1 ") + "$x := 1 ".len() as u32;
    let items = complete(doc, &Context::default(), at).unwrap().items;
    for name in ["inch", "speed", "foot"] {
        assert!(items.iter().any(|i| i.label == name && i.insert == format!("unit(\"{name}\")")), "{name}");
    }
    let cm = offset(doc, "cm$,");
    assert!(document_tokens(doc, &Context::default()).iter().any(|t| t.start == cm && t.kind == TokenKind::UNIT));
}

#[test]
fn single_letter_completions_use_hash_bindings() {
    for suffix in ["1 s", "1 #s", "1 #"] {
        let doc = format!("#import units: s\n#show: worksheet\n$x := {suffix}$");
        let at = doc.encode_utf16().count() as u32 - 1;
        let result = complete(&doc, &Context::default(), at).unwrap();
        assert_eq!(result.items.iter().find(|i| i.label == "s").unwrap().insert, "#s");
    }
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: worksheet\n$x := 1 $";
    let result = complete(doc, &Context::default(), doc.len() as u32 - 1).unwrap();
    let item = result.items.iter().find(|i| i.label == "s").unwrap();
    assert_eq!(item.insert, "#s");
    assert!(item.edits.iter().any(|e| e.text.contains("#import units: s")));
}

#[test]
fn hash_conceals_only_resolved_shortcuts_and_uses_utf16() {
    let doc = "#import units: s as sec\n#let s = unit(\"s\")\n#let triangle = math.op(\"tri\")\n#let other = 2\n#show: worksheet\n😅 $1#s + 2#sec + #triangle(0) + #other$";
    let conceals = conceals(doc, &Context::default());
    for name in ["#s +", "#sec +", "#triangle("] {
        let at = offset(doc, name);
        assert!(conceals.iter().any(|c| c.start == at && c.length == 1 && c.character == 0 && c.element_start == at), "{name}: {conceals:?}");
    }
    assert!(!conceals.iter().any(|c| c.start == offset(doc, "#other$")));
}

#[test]
fn quoted_units_are_names_and_constructors_keep_unit_styling() {
    let doc = "#show: worksheet\n$x := 1\"s\"$\n$y := 1unit(\"s\")$";
    let tokens = document_tokens(doc, &Context::default());
    assert!(tokens.iter().any(|t| t.start == offset(doc, "\"s\"$") && t.kind == TokenKind::VARIABLE));
    assert!(tokens.iter().any(|t| t.start == offset(doc, "s\")$") && t.kind == TokenKind::UNIT));
}
