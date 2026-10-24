// Names imported from solveng's `units`, `constants` and `functions` modules: colours at the import, navigation,
// renaming to an alias, hovers, import fixes, and the nudge from `op("…")` toward them.

use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }
fn nth(doc: &str, needle: &str, n: usize) -> u32 { doc[..doc.match_indices(needle).nth(n).unwrap().0].encode_utf16().count() as u32 }

fn apply(doc: &str, mut edits: Vec<(u32, u32, String)>) -> String {
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut text: Vec<u16> = doc.encode_utf16().collect();
    for (s, e, t) in edits { text.splice(s as usize..e as usize, t.encode_utf16()); }
    String::from_utf16(&text).unwrap()
}

fn applied_fix(doc: &str, fix: &Conversion) -> String {
    let mut edits = vec![(fix.start, fix.end, fix.text.clone())];
    edits.extend(fix.edits.iter().map(|e| (e.start, e.end, e.text.clone())));
    apply(doc, edits)
}

const SOLVENG: &str = "#import \"@preview/solveng:1.0.0\": *\n";

#[test]
fn imported_names_are_coloured_and_renamed_module_names_are_not() {
    let doc = format!("{SOLVENG}#import units: km, s as test\n#import constants: pi\n#import functions: tri\n#import modifiers: solve as löse\n#show: worksheet\n$x := 2 km = #test$ $y := pi tri(x)$ $x^2 = 4 -> löse(x)$");
    let tokens = document_tokens(&doc, &Context::default());
    let kind = |offset: u32| tokens.iter().find(|t| t.start == offset).map(|t| t.kind);
    assert_eq!(kind(at(&doc, "km,")), Some(TokenKind::UNIT));
    assert_eq!(kind(at(&doc, "test\n")), Some(TokenKind::UNIT));
    assert_eq!(kind(at(&doc, "s as")), Some(TokenKind::FUNCTION_CALL));
    assert_eq!(kind(at(&doc, "pi\n")), Some(TokenKind::CONSTANT));
    assert_eq!(kind(at(&doc, "löse\n")), Some(TokenKind::KEYWORD));
    assert_eq!(kind(at(&doc, "solve as")), Some(TokenKind::FUNCTION_CALL));
    assert_eq!(kind(at(&doc, "löse(x)")), Some(TokenKind::KEYWORD));
    assert_eq!(kind(at(&doc, "tri\n")), Some(TokenKind::BUILTIN_FUNCTION));
    assert_eq!(kind(at(&doc, "km =")), Some(TokenKind::UNIT));
    assert_eq!(kind(at(&doc, "test$")), Some(TokenKind::UNIT));
    assert_eq!(kind(at(&doc, "pi tri")), Some(TokenKind::CONSTANT));
    assert_eq!(kind(at(&doc, "tri(x)")), Some(TokenKind::BUILTIN_FUNCTION));
}

#[test]
fn uses_go_to_their_import() {
    let doc = format!("{SOLVENG}#import units: km, s as test\n#show: worksheet\n$x := 2 km$ $y := 3 #test$");
    let ctx = Context::default();
    assert_eq!(definition(&doc, &ctx, at(&doc, "km$")).unwrap().start, at(&doc, "km,"));
    assert_eq!(definition(&doc, &ctx, at(&doc, "test$")).unwrap().start, at(&doc, "test\n"));
    assert_eq!(usages(&doc, &ctx, at(&doc, "km,")).len(), 2);
}

#[test]
fn renaming_makes_or_drops_an_alias() {
    let ctx = Context::default();
    let rename_at = |doc: &str, offset: u32, name: &str| apply(doc, rename(doc, &ctx, offset, name).into_iter().map(|e| (e.start, e.end, e.text)).collect());
    let doc = format!("{SOLVENG}#import units: km, s\n#show: worksheet\n$a := 1 #s$ $b := 2 #s$");
    let renamed = rename_at(&doc, nth(&doc, "s$", 0), "test");
    assert!(renamed.contains("#import units: km, s as test\n") && renamed.contains("$a := 1 test$ $b := 2 test$"), "{renamed}");
    // another alias replaces the alias; the module's own name drops it
    let again = rename_at(&renamed, at(&renamed, "test\n"), "sec");
    assert!(again.contains("s as sec\n") && again.contains("1 sec$"), "{again}");
    let back = rename_at(&again, at(&again, "sec$"), "s");
    assert_eq!(back, doc);
    assert_eq!(rename_target(&doc, &ctx, nth(&doc, "s$", 1)).unwrap().name, "s");
    assert!(rename(&doc, &ctx, nth(&doc, "s$", 0), "not an ident").is_empty());
}

#[test]
fn renaming_in_the_import_list_is_marked_for_editors() {
    let ctx = Context::default();
    let doc = format!("{SOLVENG}#import constants: pi, e as euler\n#show: worksheet\n$x := 2 pi euler$");
    assert!(rename_target(&doc, &ctx, at(&doc, "pi,")).unwrap().import_alias);
    assert!(rename_target(&doc, &ctx, at(&doc, "euler\n")).unwrap().import_alias);
    assert!(rename_target(&doc, &ctx, at(&doc, "e as") ).unwrap().import_alias);
    assert!(!rename_target(&doc, &ctx, at(&doc, "pi euler")).unwrap().import_alias);
}

#[test]
fn renaming_a_name_a_wildcard_gives_imports_it_under_the_new_name() {
    let ctx = Context::default();
    let rename_at = |doc: &str, offset: u32, name: &str| apply(doc, rename(doc, &ctx, offset, name).into_iter().map(|e| (e.start, e.end, e.text)).collect());
    let doc = format!("{SOLVENG}#[\n  #import constants: *\n  #show: worksheet\n  $x := 2 pi + pi$\n]");
    let target = rename_target(&doc, &ctx, nth(&doc, "pi", 0)).unwrap();
    assert_eq!((target.name.as_str(), target.import_alias), ("pi", false));
    assert_eq!(usages(&doc, &ctx, nth(&doc, "pi", 1)).len(), 2);
    let renamed = rename_at(&doc, nth(&doc, "pi", 1), "tau");
    assert_eq!(renamed, format!("{SOLVENG}#[\n  #import constants: *\n  #import constants: pi as tau\n  #show: worksheet\n  $x := 2 tau + tau$\n]"));
    // the new import is a listed name, renamed back like any other
    let back = rename_at(&renamed, at(&renamed, "tau +"), "pi");
    assert!(back.contains("#import constants: pi\n") && back.contains("2 pi + pi"), "{back}");
    assert!(rename(&doc, &ctx, nth(&doc, "pi", 0), "pi").iter().all(|e| e.text == "pi"));
}

#[test]
fn modules_and_imported_names_hover_with_their_docs() {
    let doc = format!("{SOLVENG}#import units: km, s as test\n#import constants: pi\n#import functions: tri\n#show: worksheet\n$x := 1 #test$ $y := tri(2) km$ $z := 2 pi$");
    let ctx = Context::default();
    let md = |offset: u32| hover(&doc, &ctx, offset).unwrap().markdown;
    for module in ["units", "constants", "functions"] {
        let text = md(at(&doc, &format!("{module}:")) + 1);
        assert!(text.starts_with(&format!("`{module}` module of solveng")), "{text}");
    }
    let test = md(at(&doc, "test$"));
    assert!(test.contains("shorthand for `s`") && test.contains("second") && test.contains("See `units`"), "{test}");
    let km = md(at(&doc, "km$"));
    assert!(km.contains("kilometre") && km.contains("shorthand from solveng's `units`") && km.contains("`km = 10³ m`"), "{km}");
    let tri = md(at(&doc, "tri(2)"));
    assert!(tri.contains("shorthand from solveng's `functions`") && tri.contains("### Parameters"), "{tri}");
    let pi = md(at(&doc, "pi$"));
    assert!(pi.contains("shorthand from solveng's `constants`") && pi.contains("identity `pi`"), "{pi}");
}

#[test]
fn the_documents_own_bindings_hover_where_math_uses_them() {
    let doc = "#import units: s\n#let minute = unit($60 #s$)\n#let time = base-unit(\"time\")\n#let i = constant(\"imaginary\", $sqrt(-1)$)\n#show: worksheet\n$t := 2 minute + 3 time$ $z := 2 #i$";
    let md = |needle: &str| hover(doc, &Context::default(), at(doc, needle)).map(|h| h.markdown).unwrap_or_default();
    assert_eq!(md("minute +"), "`minute`: unit: `60 #s`");
    assert!(md("time$").contains("base unit of the dimension `time`"), "{}", md("time$"));
    assert_eq!(md("i$"), "`i`: constant: `sqrt(-1)`, identity `imaginary`");
    // where they're made, Typst's language server answers
    assert!(hover(doc, &Context::default(), at(doc, "minute =")).is_none());
}

#[test]
fn local_bindings_shadow_imports() {
    let doc = "#import units: km, m\n#show: worksheet\n$a := 2 km$\n#[\n#let km = unit($1000 #m$)\n$b := 3 km$\n]";
    let md = |needle: &str| hover(doc, &Context::default(), at(doc, needle)).unwrap().markdown;
    assert!(md("km$\n#[").contains("kilometre"));
    assert_eq!(md("km$\n]"), "`km`: unit: `1000 #m`");
}

#[test]
fn unbound_module_names_get_an_import_fix() {
    let doc = format!("{SOLVENG}#show: worksheet\n$x := 2 km$ $y := 2 pi$ $z := tri(1)$");
    let ctx = Context::default();
    for (needle, title, import) in [("km$", "Import km from solveng's units", "#import units: km\n"), ("pi$", "Import pi from solveng's constants", "#import constants: pi\n"), ("tri(", "Import tri from solveng's functions", "#import functions: tri\n")] {
        let fixes = unit_fixes(&doc, &ctx, at(&doc, needle) + 1);
        let fix = fixes.iter().find(|f| f.title == title).unwrap_or_else(|| panic!("{title}: {fixes:?}"));
        let fixed = applied_fix(&doc, fix);
        assert!(fixed.contains(import) && fixed.ends_with("$z := tri(1)$"), "{fixed}");
    }
    // added to the module's import in scope
    let some = format!("{SOLVENG}#import units: m\n#show: worksheet\n$x := 2 km$");
    let fix = unit_fixes(&some, &ctx, at(&some, "km$")).remove(0);
    assert!(applied_fix(&some, &fix).contains("#import units: m, km\n"));
    // nothing to fix where the name is bound, outside calculated math or without solveng's import
    let bound = format!("{SOLVENG}#import units: km\n#show: worksheet\n$x := 2 km$");
    assert!(unit_fixes(&bound, &ctx, at(&bound, "km$")).is_empty());
    let plain = format!("{SOLVENG}$x := 2 km$");
    assert!(unit_fixes(&plain, &ctx, at(&plain, "km$")).is_empty());
    let bare = "#show: worksheet\n$x := 2 km$";
    assert!(unit_fixes(bare, &ctx, at(bare, "km$")).is_empty());
}

#[test]
fn names_typst_math_owns_warn_and_get_their_import() {
    let ctx = Context::default();
    // Without the import, Typst reads these names itself, so it reports no unknown variable: solveng says it.
    let doc = format!("{SOLVENG}#show: worksheet\n$y := at(vec(3, 5), 0)$");
    let problems = diagnostics(&doc, &ctx);
    assert_eq!(problems.len(), 1, "{problems:?}");
    let p = &problems[0];
    assert_eq!(p.severity, "warning");
    assert_eq!((p.start, p.end), (at(&doc, "at("), at(&doc, "at(") + 2));
    assert!(p.message.contains("Typst's own") && p.message.contains("#import functions: at"), "{}", p.message);
    // the import fix is offered where the name is
    let fix = unit_fixes(&doc, &ctx, at(&doc, "at(") + 1).into_iter().find(|f| f.title == "Import at from solveng's functions").unwrap();
    assert!(applied_fix(&doc, &fix).contains("#import functions: at\n"));
    // other names Typst's math owns warn too
    let more = format!("{SOLVENG}#show: worksheet\n$f := floor(2.5) + Re(z) + rect(t)$");
    let warned: Vec<_> = diagnostics(&more, &ctx).into_iter().filter(|p| p.message.contains("Typst's own")).collect();
    assert_eq!(warned.len(), 3, "{warned:?}");
    // imported, by name or through `*`: nothing to say, nothing to fix
    for import in ["#import functions: at", "#import functions: *"] {
        let imported = format!("{SOLVENG}{import}\n#show: worksheet\n$y := at(vec(3, 5), 0)$");
        assert!(diagnostics(&imported, &ctx).is_empty(), "{import}");
        assert!(unit_fixes(&imported, &ctx, at(&imported, "at(") + 1).iter().all(|f| !f.title.starts_with("Import")));
    }
    // Typst's symbol alone, outside calculated math, or bound by the document: no warning
    for doc in [
        format!("{SOLVENG}#show: worksheet\n$x := at$"),
        format!("{SOLVENG}$y := at(vec(3, 5), 0)$"),
        format!("{SOLVENG}#show: worksheet\n#let at = math.op(\"tri\")\n$y := at(1)$"),
    ] {
        assert!(diagnostics(&doc, &ctx).iter().all(|p| !p.message.contains("Typst's own")), "{doc}");
    }
}

#[test]
fn explicit_operations_get_a_nudge_and_a_fix() {
    let doc = format!("{SOLVENG}#show: worksheet\n$y := op(\"tri\")(1) + op(\"sin\")(2)$ $z := op(\"tri\")(3)$");
    let ctx = Context::default();
    let warned: Vec<String> = diagnostics(&doc, &ctx).into_iter().filter(|p| p.message.contains("shorthand"))
        .map(|p| String::from_utf16(&doc.encode_utf16().collect::<Vec<_>>()[p.start as usize..p.end as usize]).unwrap()).collect();
    // Typst's own operators have no shorthand in solveng's functions
    assert_eq!(warned, ["op(\"tri\")", "op(\"tri\")"]);
    let fix = unit_fixes(&doc, &ctx, at(&doc, "op(\"tri\")") + 4).into_iter().find(|f| f.title.contains("shorthand")).unwrap();
    let fixed = applied_fix(&doc, &fix);
    assert!(fixed.contains("#import functions: tri\n") && fixed.contains("$y := tri(1) +"), "{fixed}");
    // no nudge where the name means something else, nor without solveng's import to add it to
    let taken = doc.replace("#show: worksheet", "#let tri = 3\n#show: worksheet");
    assert!(!diagnostics(&taken, &ctx).iter().any(|p| p.message.contains("shorthand")));
    let bare = doc.replace(SOLVENG, "");
    assert!(!diagnostics(&bare, &ctx).iter().any(|p| p.message.contains("shorthand")));
}
