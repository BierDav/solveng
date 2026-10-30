// Units are Typst bindings: visible where Typst's scopes say, referenced with `#` when a single letter, never
// written as quoted text, which names math variables.

use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }
fn kind_at(doc: &str, at: u32) -> Option<u8> {
    document_tokens(doc, &Context::default()).iter().find(|t| t.start <= at && at < t.start + t.length).map(|t| t.kind)
}
fn kind(doc: &str, needle: &str) -> Option<u8> { kind_at(doc, at(doc, needle)) }
/// The kind of the last `word` in the document.
fn kind_last(doc: &str, word: &str) -> Option<u8> { kind_at(doc, doc[..doc.rfind(word).unwrap()].encode_utf16().count() as u32) }

#[test]
fn unit_bindings_follow_typst_scopes() {
    let doc = "#show: math\n#[\n#import units: km\n#let minute = unit($60 #s$)\n$x := 2 km + 1 minute$\n]\n$y := 2 km + 1 minute$";
    assert_eq!(kind(doc, "km + 1 minute$\n]"), Some(TokenKind::UNIT));
    assert_eq!(kind(doc, "minute$\n]"), Some(TokenKind::UNIT));
    // outside the block they're no bindings: Typst can't find them
    assert_eq!(kind_last(doc, "km"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_last(doc, "minute"), Some(TokenKind::VARIABLE));
    // a later binding of another value shadows a unit
    let shadowed = "#import units: km\n#show: math\n$x := 2 km$\n#let km = 3\n$y := 2 km$";
    assert_eq!(kind(shadowed, "km$\n#let"), Some(TokenKind::UNIT));
    assert_eq!(kind_last(shadowed, "km"), Some(TokenKind::VARIABLE));
}

#[test]
fn constructors_aliases_and_module_fields_make_units() {
    let doc = "#import units: km, s\n#let time = base-unit(\"time\")\n#let minute = unit($60 #s$)\n#let distance = km\n#let metre = units.m\n#let cusom = \"iwef\"\n#show: math\n$x := 1 time + 2 minute + 3 distance + 4 metre + 5 cusom$";
    for needle in ["time +", "minute +", "distance +", "metre +"] {
        assert_eq!(kind(doc, needle), Some(TokenKind::UNIT), "{needle}");
    }
    // a plain string is no unit: Typst's value, which solveng reads as it is
    assert_eq!(kind(doc, "cusom$"), Some(TokenKind::VARIABLE));
    assert!(hover(doc, &Context::default(), at(doc, "cusom$")).is_none());
    // what they mean, where math uses them
    let md = |needle: &str| hover(doc, &Context::default(), at(doc, needle)).unwrap().markdown;
    assert!(md("distance +").contains("kilometre"), "{}", md("distance +"));
    assert!(md("metre +").contains("metre"), "{}", md("metre +"));
    // the expression of a unit is math solveng reads
    assert_eq!(kind(doc, "60"), Some(TokenKind::NUMBER));
    assert_eq!(kind(doc, "s$)"), Some(TokenKind::UNIT));
}

#[test]
fn quoted_text_names_variables_never_bindings() {
    let doc = "#import units: km\n#show: math\n$\"km\" := 23$\n$x := 2 km + \"km\"$";
    let context = Context::default();
    assert_eq!(kind(doc, "km + "), Some(TokenKind::UNIT));
    assert_eq!(kind(doc, "\"km\"$"), Some(TokenKind::VARIABLE));
    // the variable's usages leave the unit out
    let usages: Vec<u32> = usages(doc, &context, at(doc, "\"km\" :=") + 1).iter().map(|l| l.start).collect();
    assert_eq!(usages, [at(doc, "\"km\" :="), at(doc, "\"km\"$")]);
    // the binding is navigated where it's imported, not as a variable
    let bare = at(doc, "km + ");
    assert_eq!(definition(doc, &context, bare).unwrap().start, at(doc, "km\n"));
    assert!(rename(doc, &context, bare, "other").iter().all(|e| e.start != at(doc, "\"km\" :=")));
}

#[test]
fn quoted_parameter_types_do_not_resolve_typst_bindings() {
    let doc = "#show: math\n#let second = base-unit(\"time\")\n$f(\"xaef\": second) := \"xaef\"$\n$g(\"xaef\": \"second\") := \"xaef\"$";
    assert_eq!(kind(doc, "second)"), Some(TokenKind::UNIT));
    assert_eq!(kind(doc, "\"second\""), Some(TokenKind::VARIABLE));
}

fn applied_completion(marked: &str, label: &str) -> String {
    let caret = marked.find('|').unwrap();
    let offset = marked[..caret].encode_utf16().count() as u32;
    let doc = marked.replacen('|', "", 1);
    let result = complete(&doc, &Context::default(), offset).unwrap();
    let item = result.items.iter().find(|item| item.label == label).unwrap();
    let mut text = doc.encode_utf16().collect::<Vec<_>>();
    text.splice(result.from as usize..result.to.unwrap_or(offset) as usize, item.insert.encode_utf16());
    String::from_utf16(&text).unwrap()
}

#[test]
fn single_letter_units_complete_with_hash_bindings() {
    for typed in ["34s|", "34#s|", "34 #s|", "34 |"] {
        let doc = format!("#show: math\n#let s = base-unit(\"time\")\n$z := {typed}$");
        let result = applied_completion(&doc, "s");
        assert!(result.ends_with("34#s$") || result.ends_with("34 #s$"), "{result}");
    }
    // greek letters too
    let doc = "#show: math\n#let α = unit($2 #s$)\n#let s = base-unit(\"time\")\n$z := 34 |$";
    assert!(applied_completion(doc, "α").ends_with("34 #α$"));
}

#[test]
fn single_letter_bindings_need_a_hash_with_a_fix_and_gray_hash() {
    for doc in ["#show: math\n#let s = base-unit(\"time\")\n😅 $z := 34s$", "#import constants: e\n#show: math\n😅 $z := 34e$"] {
        let name = if doc.contains("constants") { "e" } else { "s" };
        let offset = at(doc, &format!("{name}$"));
        let fixes = unit_fixes(doc, &Context::default(), offset);
        assert!(fixes.iter().any(|f| (f.start, f.end, f.text.as_str()) == (offset, offset, "#")), "{doc}: {fixes:?}");
        let warning = diagnostics(doc, &Context::default()).into_iter().find(|p| p.message.contains("bare single letters")).unwrap();
        assert_eq!((warning.start, warning.end), (offset, offset + 1));
        assert!(warning.message.contains(if name == "e" { "constant binding" } else { "unit binding" }), "{}", warning.message);
        let fixed = doc.replace(&format!("{name}$"), &format!("#{name}$"));
        assert_eq!(kind(&fixed, &format!("#{name}$")), Some(TokenKind::UNIT_REFERENCE));
        assert!(unit_fixes(&fixed, &Context::default(), at(&fixed, &format!("{name}$"))).is_empty());
        assert!(!diagnostics(&fixed, &Context::default()).iter().any(|p| p.message.contains("bare single letters")));
    }
    // a math variable of that name is meant: no warning
    let variable = "#show: math\n#let s = base-unit(\"time\")\n$s := 5$\n$z := 34s$";
    assert!(unit_fixes(variable, &Context::default(), at(variable, "s$")).is_empty());
    assert!(!diagnostics(variable, &Context::default()).iter().any(|p| p.message.contains("bare single letters")));
    // nor without a binding of that name
    let unbound = "#show: math\n$z := 34s$";
    assert!(diagnostics(unbound, &Context::default()).is_empty());
}

#[test]
fn common_units_rank_before_uncommon_prefixes() {
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: math\n$x := 1 |$";
    let caret = doc.find('|').unwrap();
    let source = doc.replace('|', "");
    let result = complete(&source, &Context::default(), source[..caret].encode_utf16().count() as u32).unwrap();
    let rank = |label| result.items.iter().position(|item| item.label == label).unwrap();
    assert!(rank("A") < rank("mA"));
    assert!(rank("mA") < rank("GA"));
    assert!(rank("s") < rank("ps"));
    // and the document's own bindings first
    let own = doc.replace("#show", "#let minute = unit($60 #s$)\n#show");
    let caret = own.find('|').unwrap();
    let source = own.replace('|', "");
    let result = complete(&source, &Context::default(), source[..caret].encode_utf16().count() as u32).unwrap();
    assert_eq!(result.items.iter().find(|i| i.kind == ItemKind::UNIT).unwrap().label, "minute");
}

#[test]
fn units_are_found_by_their_names() {
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: math\n$R := 5 |$";
    let caret = doc.find('|').unwrap();
    let source = doc.replace('|', "");
    let items = complete(&source, &Context::default(), source[..caret].encode_utf16().count() as u32).unwrap().items;
    for (typed, label) in [("ohm", "Ω"), ("kiloohm", "kΩ"), ("volt", "V"), ("Celsius", "degC")] {
        assert!(items.iter().any(|i| i.label == label && i.filter.as_deref().unwrap_or(&i.label).to_lowercase().contains(&typed.to_lowercase())), "{label} for {typed}");
    }
}

#[test]
fn code_binding_names_need_hashes_in_math() {
    for name in ["fluid_ounce", "fluid_ounce_uk", "fluid-ounce", "fl_oz2"] {
        let prefix = format!("#show: math\n#let {name} = unit($28.413 ml$, display: $\"fl oz\"$)\n");
        let doc = format!("{prefix}$1 #units.l -> {name}$");
        let offset = at(&doc, &format!("{name}$"));
        let warning = diagnostics(&doc, &Context::default()).into_iter().find(|p| p.message.contains(&format!("use '#{name}'"))).unwrap_or_else(|| panic!("no warning: {doc}"));
        assert_eq!((warning.start, warning.end), (offset, offset + name.len() as u32));
        assert_eq!(kind_at(&doc, offset), Some(TokenKind::VARIABLE), "{doc}");
        assert!(unit_fixes(&doc, &Context::default(), offset).iter().any(|f| f.start == offset && f.text == "#"));
        let fixed = format!("{prefix}$1 #units.l -> #{name}$");
        assert_eq!(kind(&fixed, &format!("{name}$")), Some(TokenKind::UNIT));
        assert!(!diagnostics(&fixed, &Context::default()).iter().any(|p| p.message.contains("math splits")));
        for typed in ["|", "fl|", "fluid_|", "#fl|"] {
            let input = format!("{prefix}$1 #units.l -> {typed}$");
            assert!(applied_completion(&input, name).ends_with(&format!("-> #{name}$")), "{input}");
        }
    }
    let doc = "#show: math\n#let fluid_ounce = unit($28.413 ml$)\n$fluid_ounce := 2$\n$x := fluid_ounce$";
    assert!(!diagnostics(doc, &Context::default()).iter().any(|p| p.message.contains("math splits")));
}
