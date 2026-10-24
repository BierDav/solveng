// Completions follow what the compile sees at the caret: scopes, clearing, the bindings in scope and imports.

use solveng_ide::*;

fn complete_marked(marked: &str) -> Vec<Item> {
    let caret = marked.find('|').unwrap();
    let doc = marked.replacen('|', "", 1);
    let offset = doc[..caret].encode_utf16().count() as u32;
    complete(&doc, &Context::default(), offset).map(|c| c.items).unwrap_or_default()
}

fn names(marked: &str) -> Vec<String> {
    complete_marked(marked).into_iter().filter(|i| matches!(i.kind, ItemKind::VARIABLE | ItemKind::FUNCTION))
        .map(|i| i.label.split([':', '(']).next().unwrap().to_string()).collect()
}

fn units(marked: &str) -> Vec<(String, String)> {
    complete_marked(marked).into_iter().filter(|i| i.kind == ItemKind::UNIT).map(|i| (i.label, i.insert)).collect()
}

const EXERCISES: &str = "#show: worksheet.with(clear-at: <exercise>)\n= A <exercise>\n$y := x$\n= B <exercise>\n$\"yt\" := x$\n";

#[test]
fn cleared_names_are_not_offered() {
    let found = names(&format!("{EXERCISES}$z := |$"));
    assert!(found.contains(&"yt".into()) && !found.contains(&"y".into()), "{found:?}");
    // a heading without the label doesn't clear
    let found = names(&EXERCISES.replace("= B <exercise>", "= B").replace("\"yt\" := x$\n", "\"yt\" := x$\n$z := |$"));
    assert!(found.contains(&"y".into()), "{found:?}");
    // nor does a name defined later count
    let found = names("#show: worksheet\n$a := |$\n$b := 2$");
    assert!(!found.contains(&"b".into()), "{found:?}");
    // clear("…") and clear(all: true)
    let found = names("#show: worksheet\n$a := 1$\n$b := 2$\n#clear(\"a\")\n$c := |$");
    assert!(found.contains(&"b".into()) && !found.contains(&"a".into()), "{found:?}");
    let found = names("#show: worksheet\n$a := 1$\n#clear(all: true)\n$c := |$");
    assert!(!found.contains(&"a".into()), "{found:?}");
}

#[test]
fn worksheet_scopes_decide_what_is_offered() {
    let outer = "#show: worksheet\n$a := 1$\n";
    // (a block holding only an equation is the equation to Typst, so these hold text too)
    assert!(!names(&format!("{outer}#worksheet[Text $b := |$]")).contains(&"a".into()));
    assert!(names(&format!("{outer}#worksheet(scope: \"inherit\")[Text $b := |$]")).contains(&"a".into()));
    // what an inherit worksheet defines ends with it; a shared one keeps it
    assert!(!names(&format!("{outer}#worksheet(scope: \"inherit\")[Text $i := 1$]\n$b := |$")).contains(&"i".into()));
    assert!(names(&format!("{outer}#worksheet(scope: \"shared\")[Text $i := 1$]\n$b := |$")).contains(&"i".into()));
    // equations passed to a worksheet share by default
    assert!(names(&format!("{outer}#worksheet($i := 1$)\n$b := |$")).contains(&"i".into()));
}

#[test]
fn substitute_offers_what_the_expression_names() {
    // yt is the expression; y was cleared and x has no formula
    assert_eq!(names(&format!("{EXERCISES}$\"yt\" -> substitute(|)$")), ["yt"]);
    let doc = "#show: worksheet\n$U := R I$\n$R := 2$\n$P := U^2/R -> substitute(|)$";
    let found = names(doc);
    assert!(found.contains(&"U".into()) && found.contains(&"R".into()) && !found.contains(&"P".into()) && !found.contains(&"I".into()), "{found:?}");
    // through a listed variable its formula's variables, transitively
    let chain = "#show: worksheet\n$a := 1$\n$\"yt\" := a + x$\n$\"yy\" := 2 \"yt\"$\n$\"yy\" -> substitute(|)$";
    assert_eq!(names(chain), ["yy"]);
    assert_eq!(names(&chain.replace("substitute(|)", "substitute(\"yy\", |)")), ["yt"]);
    assert_eq!(names(&chain.replace("substitute(|)", "substitute(\"yy\", \"yt\", |)")), ["a"]);
    // parameters have no formula
    assert!(names("#show: worksheet\n$a := 1$\n$f(x) := x + a -> substitute(|)$").iter().all(|n| n != "x"));
    // math spells names as math needs them
    let items = complete_marked(&format!("{EXERCISES}$\"yt\" -> substitute(|)$"));
    assert_eq!(items[0].insert, "\"yt\"");
}

#[test]
fn units_follow_the_bindings_in_scope_at_the_caret() {
    let doc = "#let minute = unit($60 #s$)\n#show: worksheet\n$x := 1 |$\n#[#let hour = unit($60 minute$)\n$y := 1 $]";
    let outer = units(doc);
    assert!(outer.iter().any(|(l, i)| l == "minute" && i == "minute") && !outer.iter().any(|(l, _)| l == "hour"), "{outer:?}");
    // a binding counts in its block, after it's made
    let inner = units(&doc.replace("$x := 1 |$", "$x := 1$").replace("$y := 1 $", "$y := 1 |$"));
    assert!(inner.iter().any(|(l, _)| l == "hour") && inner.iter().any(|(l, _)| l == "minute"), "{inner:?}");
    assert!(!units("#show: worksheet\n$x := 1 |$\n#let minute = unit($60 #s$)").iter().any(|(l, _)| l == "minute"));
    // a later binding of the same name that isn't a unit hides it
    let hidden = units("#let minute = unit($60 #s$)\n#let minute = 3\n#show: worksheet\n$x := 1 |$");
    assert!(!hidden.iter().any(|(l, _)| l == "minute"), "{hidden:?}");
}

#[test]
fn bindings_come_first_and_module_units_are_imported_along() {
    let found = units("#import units: km\n#let sec = unit($1 #s$)\n#let s = base-unit(\"time\")\n#show: worksheet\n$x := 1 |$");
    assert_eq!(found[0].1, "km");
    // single letters need their hash in math
    assert!(found.iter().any(|(l, i)| l == "s" && i == "#s") && found.iter().any(|(l, i)| l == "sec" && i == "sec"), "{found:?}");
    // a bound name isn't offered again from the module
    assert_eq!(found.iter().filter(|(l, _)| l == "km" || l == "s").count(), 2, "{found:?}");
    // others are added to the import in scope
    let items = complete_marked("#import units: km\n#show: worksheet\n$x := 1 |$");
    let ma = items.iter().find(|i| i.label == "mA").unwrap();
    assert_eq!((ma.insert.as_str(), ma.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>()), ("mA", vec![", mA"]));
    // without solveng's import nothing can be imported along, so the module's units aren't offered
    assert!(!units("#show: worksheet\n$x := 1 |$").iter().any(|(l, _)| l == "mA"));
    // with it, a new import follows it
    let items = complete_marked("#import \"@preview/solveng:1.0.0\": worksheet\n#show: worksheet\n$x := 1 |$");
    let ma = items.iter().find(|i| i.label == "mA").unwrap();
    assert_eq!(ma.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), [", units", "\n#import units: mA"]);
    // a renamed import
    assert!(units("#import units: km as kilo\n#show: worksheet\n$x := 1 |$").iter().any(|(l, i)| l == "kilo" && i == "kilo"));
    // imports are lexical: not outside the block importing them
    let found = units("#show: worksheet\n#[#import units: km\n]\n$x := 1 |$");
    assert!(!found.iter().any(|(l, _)| l == "km"), "{found:?}");
}

#[test]
fn operations_need_their_binding_in_math() {
    let insert = |doc: &str| complete_marked(doc).into_iter().find(|i| i.label == "tri").map(|i| i.insert);
    assert_eq!(insert("#show: worksheet\n$y := tr|$").as_deref(), Some("op(\"tri\")($0)"));
    assert_eq!(insert("#import functions: tri\n#show: worksheet\n$y := tr|$").as_deref(), Some("tri($0)"));
    // imported along where solveng is imported
    let items = complete_marked("#import \"@preview/solveng:1.0.0\": *\n#show: worksheet\n$y := tr|$");
    let tri = items.iter().find(|i| i.label == "tri").unwrap();
    assert_eq!((tri.insert.as_str(), tri.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>()), ("tri($0)", vec!["\n#import functions: tri"]));
    // Typst's own math operators need nothing
    assert!(complete_marked("#show: worksheet\n$y := si|$").iter().any(|i| i.label == "sin" && i.insert.starts_with("sin(")));
}

#[test]
fn constants_complete_from_their_bindings_and_module() {
    let items = complete_marked("#import \"@preview/solveng:1.0.0\": *\n#import constants: pi\n#let g = constant(\"gravity\", $9.81 #m / #s^2$)\n#show: worksheet\n$y := |$");
    let constant = |label: &str| items.iter().find(|i| i.label == label && i.kind == ItemKind::CONSTANT).unwrap_or_else(|| panic!("{label}: {items:?}"));
    assert_eq!(constant("pi").insert, "pi");
    assert_eq!(constant("g").insert, "#g");
    assert!(constant("g").detail.contains("gravity"));
    // the module's others are imported along
    let e = constant("e");
    assert_eq!((e.insert.as_str(), e.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>()), ("#e", vec![", e"]));
}

#[test]
fn documents_highlight_their_unit_bindings() {
    let doc = "#import units: km, m, min\n#show: worksheet\n$1#km -> min$\n$2 km -> #m$\n$3 m$\n#[#import units: mA\n]\n$4 mA$";
    let tokens = document_tokens(doc, &Context::default());
    let kind = |needle: &str, nth: usize| {
        let at = doc[..doc.match_indices(needle).nth(nth).unwrap().0].encode_utf16().count() as u32;
        tokens.iter().find(|t| t.start == at).map(|t| t.kind)
    };
    assert_eq!(kind("#km ->", 0), Some(TokenKind::UNIT_REFERENCE));
    assert_eq!(kind("km ->", 0), Some(TokenKind::UNIT));
    assert_eq!(kind("min$", 0), Some(TokenKind::UNIT));
    assert_eq!(kind("km ->", 1), Some(TokenKind::UNIT));
    assert_eq!(kind("m$\n$3", 0), Some(TokenKind::UNIT));
    // a bare letter is a variable, even imported: `#m` reaches the binding
    assert_eq!(kind("m$\n#[", 0), Some(TokenKind::VARIABLE));
    // an import ends with its block
    assert_ne!(kind("mA$", 0), Some(TokenKind::UNIT));
}
