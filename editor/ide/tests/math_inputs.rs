// What math gives solveng: expressions, unit expressions of options and constructors, `#` before single letters.

use solveng_ide::*;

fn offset(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }

fn kind(doc: &str, needle: &str) -> Option<u8> {
    let at = offset(doc, needle);
    document_tokens(doc, &Context::default()).iter().find(|t| t.start == at).map(|t| t.kind)
}

#[test]
fn expression_strings_have_no_editor_language() {
    for name in ["math", "value", "graph", "compile"] {
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
fn unit_expressions_of_options_and_constructors_are_math() {
    let doc = "#import units: cm, m, s as sec, W, V\n#let inch = unit($2.54 cm$)\n#show: math.with(preferred-units: ($m / #sec$, $inch$))\n#compile($U^2/R$, \"U\", unit: $#W$, units: (U: $#V$))\n$x := 1 $";
    for (needle, want) in [("cm$)", TokenKind::UNIT), ("2.54", TokenKind::NUMBER), ("sec$", TokenKind::UNIT), ("inch$)", TokenKind::UNIT), ("W$,", TokenKind::UNIT), ("V$)", TokenKind::UNIT)] {
        assert_eq!(kind(doc, needle), Some(want), "{needle}");
    }
    // they're regions solveng reads
    let regions = regions(doc, &Context::default());
    assert!(regions.iter().any(|r| r.kind == "math" && r.start == offset(doc, "2.54")), "{regions:?}");
    // the binding made there completes as a unit, with its expression
    let at = doc.encode_utf16().count() as u32 - 1;
    let items = complete(doc, &Context::default(), at).unwrap().items;
    let inch = items.iter().find(|i| i.label == "inch").unwrap();
    assert_eq!((inch.kind, inch.insert.as_str(), inch.detail.as_str()), (ItemKind::UNIT, "inch", "unit = 2.54 cm"));
}

fn applied(doc: &str, at: u32, label: &str) -> String {
    let result = complete(doc, &Context::default(), at).unwrap();
    let item = result.items.iter().find(|i| i.label == label).unwrap();
    let mut text: Vec<u16> = doc.encode_utf16().collect();
    text.splice(result.from as usize..result.to.unwrap_or(at) as usize, item.insert.encode_utf16());
    String::from_utf16(&text).unwrap()
}

#[test]
fn single_letter_completions_use_hash_bindings() {
    for (suffix, expected) in [("1 s", "1 #s"), ("1 #s", "1 #s"), ("1 #", "1 #s")] {
        let doc = format!("#import units: s\n#show: math\n$x := {suffix}$");
        let at = doc.encode_utf16().count() as u32 - 1;
        assert!(applied(&doc, at, "s").ends_with(&format!("{expected}$")), "{suffix}");
    }
    // longer names need none, also when one is typed
    for (suffix, expected) in [("1 m", "1 ms"), ("1 #m", "1 #ms")] {
        let doc = format!("#import units: ms\n#show: math\n$x := {suffix}$");
        let at = doc.encode_utf16().count() as u32 - 1;
        assert!(applied(&doc, at, "ms").ends_with(&format!("{expected}$")), "{suffix}");
    }
    // not imported yet: the import comes along
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: math\n$x := 1 $";
    let result = complete(doc, &Context::default(), doc.len() as u32 - 1).unwrap();
    let item = result.items.iter().find(|i| i.label == "s").unwrap();
    assert_eq!(item.insert, "#s");
    assert!(item.edits.iter().any(|e| e.text.contains("#import units: s")));
}

#[test]
fn hash_conceals_only_solveng_bindings_and_uses_utf16() {
    let doc = "#import units: s as sec\n#import constants: e\n#let s = base-unit(\"time\")\n#let triangle = math.op(\"tri\")\n#let other = 2\n#show: math\n😅 $1#s + 2#sec + #triangle(0) + #e + #other$";
    let conceals = conceals(doc, &Context::default());
    for name in ["#s +", "#sec +", "#triangle(", "#e +"] {
        let at = offset(doc, name);
        assert!(conceals.iter().any(|c| c.start == at && c.length == 1 && c.character == 0 && c.element_start == at), "{name}: {conceals:?}");
    }
    assert!(!conceals.iter().any(|c| c.start == offset(doc, "#other$")));
    // and the hash is gray
    assert_eq!(kind(doc, "#s +"), Some(TokenKind::UNIT_REFERENCE));
    assert_eq!(kind(doc, "#e +"), Some(TokenKind::UNIT_REFERENCE));
}

#[test]
fn quoted_text_is_a_name_and_bindings_are_units() {
    let doc = "#import units: s\n#show: math\n$\"s\" := 2$\n$x := 1\"s\"$\n$y := 1#s$";
    assert_eq!(kind(doc, "\"s\"$"), Some(TokenKind::VARIABLE));
    let unit = doc.rfind("s$").unwrap();
    assert_eq!(kind(doc, &doc[unit..]), Some(TokenKind::UNIT));
    // the quoted name is the variable
    let use_ = offset(doc, "\"s\"$");
    assert_eq!(definition(doc, &Context::default(), use_ + 1).map(|l| l.start), Some(offset(doc, "\"s\" :=")));
}
