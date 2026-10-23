// Completions follow what the compile sees at the caret: scopes, clearing, units in effect and shorthands.

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
    // in code
    assert_eq!(names("```solveng\nU := R I\nP := U^2 -> substitute(|)\n```"), ["U"]);
}

#[test]
fn units_follow_the_scope_at_the_caret() {
    let doc = "#show: worksheet.with(units: (km: \"s\", kmold: \"10^3m\"))\n$x := 1 |$\n#worksheet(builtin-units: false)[Text $y := 1 $]";
    let outer = units(doc);
    assert!(outer.iter().any(|(l, _)| l == "kmold") && outer.iter().any(|(l, _)| l == "min"));
    let inner = units(&doc.replace("$x := 1 |$", "$x := 1$").replace("$y := 1 $", "$y := 1 |$"));
    assert!(inner.is_empty(), "{inner:?}");
    // an isolated worksheet starts over with the built-in units, without the outer ones
    let fresh = units(&doc.replace("$x := 1 |$", "$x := 1$").replace("builtin-units: false", "digits: 3").replace("$y := 1 $", "$y := 1 |$"));
    assert!(fresh.iter().any(|(l, _)| l == "min") && !fresh.iter().any(|(l, _)| l == "kmold"), "{fresh:?}");
    let isolated = "#show: worksheet.with(units: (km: \"s\", kmold: \"10^3m\"))\n#worksheet(units: (ü: auto))[Text $y := 1 |$]";
    let found = units(isolated);
    assert!(found.iter().any(|(l, _)| l == "ü") && !found.iter().any(|(l, _)| l == "kmold"), "{found:?}");
}

#[test]
fn shorthands_are_preferred_over_quoted_units() {
    let found = units("#import units: km\n#let sec = unit(\"s\")\n#let s = unit(\"s\")\n#show: worksheet\n$x := 1 |$");
    assert_eq!(found[0].1, "km");
    // a shorthand stands for its unit: no quoted copy; single letters stay quoted
    assert!(!found.iter().any(|(_, i)| i == "\"km\""), "{found:?}");
    assert!(found.iter().any(|(l, i)| l == "s" && i == "\"s\"") && found.iter().any(|(l, i)| l == "sec" && i == "sec"));
    // others are added to the import in scope
    let items = complete_marked("#import units: km\n#show: worksheet\n$x := 1 |$");
    let ma = items.iter().find(|i| i.label == "mA").unwrap();
    assert_eq!((ma.insert.as_str(), ma.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>()), ("mA", vec![", mA"]));
    // without solveng's import nothing can be imported along
    assert!(units("#show: worksheet\n$x := 1 |$").iter().any(|(l, i)| l == "mA" && i == "\"mA\""));
    // with it, a new import follows it
    let items = complete_marked("#import \"@preview/solveng:1.0.0\": worksheet\n#show: worksheet\n$x := 1 |$");
    let ma = items.iter().find(|i| i.label == "mA").unwrap();
    assert_eq!(ma.edits.iter().map(|e| e.text.as_str()).collect::<Vec<_>>(), [", units", "\n#import units: mA"]);
    // a renamed import
    assert!(units("#import units: km as kilo\n#show: worksheet\n$x := 1 |$").iter().any(|(l, i)| l == "kilo" && i == "kilo"));
    // shorthands are lexical: not outside the block importing them
    let found = units("#show: worksheet\n#[#import units: km\n]\n$x := 1 |$");
    assert!(found.iter().any(|(_, i)| i == "\"km\""), "{found:?}");
}

#[test]
fn operations_need_their_shorthand_in_math() {
    let insert = |doc: &str| complete_marked(doc).into_iter().find(|i| i.label == "tri").map(|i| i.insert);
    assert_eq!(insert("#show: worksheet\n$y := tr|$").as_deref(), Some("op(\"tri\")($0)"));
    assert_eq!(insert("#import functions: tri\n#show: worksheet\n$y := tr|$").as_deref(), Some("tri($0)"));
    // Typst's own math operators need nothing; code needs no shorthands at all
    assert!(complete_marked("#show: worksheet\n$y := si|$").iter().any(|i| i.label == "sin" && i.insert.starts_with("sin(")));
    assert_eq!(insert("```solveng\ny := tr|\n```").as_deref(), Some("tri($0)"));
}

#[test]
fn reported_documents_highlight_their_units() {
    let doc = "#import \"@solveng/solveng:1.0.0\": *\n#import units: km, m\n#show: worksheet.with(units: (km: \"s\", kmold: \"10^3m\"))\n$1#km -> min$\n$1unit(\"km\") -> min$\n$1\"kmold\" -> \"m\"$\n$2\"kmold\" -> m$\n#worksheet(builtin-units: false)[\n  $2#km -> min$\n  $2unit(\"km\") -> min$\n]";
    let tokens = document_tokens(doc, &Context::default());
    let kind = |needle: &str, nth: usize| {
        let at = doc[..doc.match_indices(needle).nth(nth).unwrap().0].encode_utf16().count() as u32;
        tokens.iter().find(|t| t.start == at).map(|t| t.kind)
    };
    assert_eq!(kind("#km ->", 0), Some(TokenKind::UNIT_REFERENCE));
    assert_eq!(kind("km ->", 0), Some(TokenKind::UNIT));
    assert_eq!(kind("km\")", 0), Some(TokenKind::UNIT));
    assert_eq!(kind("\"kmold\"", 0), Some(TokenKind::UNIT));
    assert_eq!(kind("\"m\"$", 0), Some(TokenKind::UNIT));
    // a bare letter is a variable, even imported: `#m` reaches the shorthand
    assert_eq!(kind("m$", 0), Some(TokenKind::VARIABLE));
    assert_eq!(kind("min", 0), Some(TokenKind::UNIT));
    // nothing is a unit without the built-in ones
    assert_ne!(kind("km ->", 1), Some(TokenKind::UNIT));
    assert_ne!(kind("km\")", 1), Some(TokenKind::UNIT));
    assert_ne!(kind("min", 2), Some(TokenKind::UNIT));
}
