// Shorthands from solveng's `units` and `functions` modules: colours at the import, navigation, renaming to an
// alias, hovers, and the nudges toward them.

use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }
fn nth(doc: &str, needle: &str, n: usize) -> u32 { doc[..doc.match_indices(needle).nth(n).unwrap().0].encode_utf16().count() as u32 }

fn apply(doc: &str, mut edits: Vec<(u32, u32, String)>) -> String {
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut text: Vec<u16> = doc.encode_utf16().collect();
    for (s, e, t) in edits { text.splice(s as usize..e as usize, t.encode_utf16()); }
    String::from_utf16(&text).unwrap()
}

const SOLVENG: &str = "#import \"@preview/solveng:1.0.0\": *\n";

#[test]
fn imported_names_are_coloured_and_renamed_module_names_are_not() {
    let doc = format!("{SOLVENG}#import units: km, s as test\n#import functions: tri\n#show: worksheet\n$1unit(\"min\") -> #test$");
    let tokens = document_tokens(&doc, &Context::default());
    let kind = |offset: u32| tokens.iter().find(|t| t.start == offset).map(|t| t.kind);
    assert_eq!(kind(at(&doc, "km,")), Some(TokenKind::UNIT));
    assert_eq!(kind(at(&doc, "test\n")), Some(TokenKind::UNIT));
    assert_eq!(kind(at(&doc, "s as")), Some(TokenKind::FUNCTION_CALL));
    assert_eq!(kind(at(&doc, "tri\n")), Some(TokenKind::BUILTIN_FUNCTION));
    assert_eq!(kind(at(&doc, "test$")), Some(TokenKind::UNIT));
}

#[test]
fn uses_go_to_their_import() {
    let doc = format!("{SOLVENG}#import units: km, s as test\n#show: worksheet\n$x := 2 km$ $1unit(\"min\") -> #test$");
    let ctx = Context::default();
    assert_eq!(definition(&doc, &ctx, at(&doc, "km$")).unwrap().start, at(&doc, "km,"));
    assert_eq!(definition(&doc, &ctx, at(&doc, "test$")).unwrap().start, at(&doc, "test\n"));
    assert_eq!(usages(&doc, &ctx, at(&doc, "km,")).len(), 2);
}

#[test]
fn renaming_makes_or_drops_an_alias() {
    let ctx = Context::default();
    let rename_at = |doc: &str, offset: u32, name: &str| apply(doc, rename(doc, &ctx, offset, name).into_iter().map(|e| (e.start, e.end, e.text)).collect());
    let doc = format!("{SOLVENG}#import units: km, s\n#show: worksheet\n$1unit(\"min\") -> #s$ $2unit(\"h\") -> #s$");
    let renamed = rename_at(&doc, nth(&doc, "s$", 0), "test");
    assert!(renamed.contains("#import units: km, s as test\n") && renamed.contains("-> #test$ $2unit(\"h\") -> #test$"), "{renamed}");
    // another alias replaces the alias; the module's own name drops it
    let again = rename_at(&renamed, at(&renamed, "test\n"), "sec");
    assert!(again.contains("s as sec\n") && again.contains("-> #sec$"), "{again}");
    let back = rename_at(&again, at(&again, "sec$"), "s");
    assert_eq!(back, doc);
    assert_eq!(rename_target(&doc, &ctx, nth(&doc, "s$", 1)).unwrap().name, "s");
    assert!(rename(&doc, &ctx, nth(&doc, "s$", 0), "not an ident").is_empty());
}

#[test]
fn modules_and_shorthands_hover_with_their_docs() {
    let doc = format!("{SOLVENG}#import units: km, s as test\n#import functions: tri\n#show: worksheet\n$1unit(\"min\") -> #test$ $y := tri(2) km$");
    let ctx = Context::default();
    let md = |offset: u32| hover(&doc, &ctx, offset).unwrap().markdown;
    let units = md(at(&doc, "units:") + 1);
    assert!(units.contains("`units` module") && units.contains("lexical"), "{units}");
    let functions = md(at(&doc, "functions:") + 1);
    assert!(functions.contains("`functions` module") && functions.contains("op(\"tri\")"), "{functions}");
    let test = md(at(&doc, "test$"));
    assert!(test.contains("shorthand for `s`") && test.contains("second") && test.contains("See `units`"), "{test}");
    let km = md(at(&doc, "km$"));
    assert!(km.contains("kilometre") && km.contains("shorthand from solveng's `units`"), "{km}");
    let tri = md(at(&doc, "tri(2)"));
    assert!(tri.contains("shorthand from solveng's `functions`") && tri.contains("### Parameters"), "{tri}");
}

#[test]
fn strings_with_a_shorthand_get_a_nudge_and_a_fix() {
    let doc = format!("{SOLVENG}#show: worksheet\n$x := 2 unit(\"km\") + 3 unit(\"km\")$ $y := op(\"tri\")(1)$ $z := 1 unit(\"s\")$");
    let ctx = Context::default();
    let warned: Vec<String> = diagnostics(&doc, &ctx).into_iter().filter(|p| p.message.contains("shorthand"))
        .map(|p| String::from_utf16(&doc.encode_utf16().collect::<Vec<_>>()[p.start as usize..p.end as usize]).unwrap()).collect();
    // single letters stay quoted
    assert_eq!(warned, ["unit(\"km\")", "unit(\"km\")", "op(\"tri\")", "unit(\"s\")"]);
    let fix = unit_fixes(&doc, &ctx, at(&doc, "unit(\"km\")") + 6).into_iter().find(|f| f.title.contains("shorthand")).unwrap();
    let mut edits = vec![(fix.start, fix.end, fix.text.clone())];
    edits.extend(fix.edits.iter().map(|e| (e.start, e.end, e.text.clone())));
    let fixed = apply(&doc, edits);
    assert!(fixed.contains("#import units: km\n") && fixed.contains("$x := 2 km +"), "{fixed}");
    // no nudge where the name means something else or the unit is redefined
    let taken = doc.replace("#show: worksheet", "#let km = 3\n#show: worksheet");
    assert!(!diagnostics(&taken, &ctx).iter().any(|p| p.message.contains("shorthand km")));
    let redefined = doc.replace("#show: worksheet", "#show: worksheet.with(units: (km: $s$))");
    assert!(!diagnostics(&redefined, &ctx).iter().any(|p| p.message.contains("shorthand km")));
    // nor without solveng's import to add it to
    let bare = doc.replace(SOLVENG, "");
    assert!(!diagnostics(&bare, &ctx).iter().any(|p| p.message.contains("shorthand")));
}

#[test]
fn prefixed_uses_follow_the_units_in_effect() {
    // the shorthand is outside the worksheet defining m: its km is kilometre, no use of that m
    let doc = "#let km = unit(\"km\")\n#show: worksheet.with(builtin-units: false, units: (\n  m: $1$,\n  km_old: $10^3m$,\n))\n$x := 2 unit(\"km_old\") + 3 unit(\"km\")$";
    let ctx = Context::default();
    let mut found: Vec<u32> = usages(doc, &ctx, at(doc, "m: $1$")).iter().map(|l| l.start).collect();
    found.sort();
    // its key, the m of 10^3m and the m of "km" inside the worksheet; never inside km_old or the outer km
    assert_eq!(found, [at(doc, "m: $1$"), at(doc, "m$,\n))"), nth(doc, "m\")$", 0)], "{found:?}");
    assert!(usages(doc, &ctx, at(doc, "km_old")).iter().all(|l| l.end - l.start == 6));
}
