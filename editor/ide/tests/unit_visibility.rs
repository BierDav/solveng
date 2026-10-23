use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }
fn kind(doc: &str, needle: &str) -> u8 {
    let at = at(doc, needle);
    document_tokens(doc, &Context::default()).iter().find(|t| t.start <= at && at < t.start + t.length).unwrap().kind
}

#[test]
fn hidden_unit_names_are_variables_without_hiding_prefixes() {
    let doc = "#show: worksheet.with(units: (s: none, mys: \"s\"))\n$x := 2s$\n$y := 2\"s\"$\n$z := 2\"ms\"$\n$t := 2\"mys\"$";
    assert_eq!(kind(doc, "s$"), TokenKind::VARIABLE);
    assert_eq!(kind(doc, "\"s\"$"), TokenKind::VARIABLE);
    assert_eq!(kind(doc, "\"ms\"$"), TokenKind::UNIT);
    assert_eq!(kind(doc, "\"mys\"$"), TokenKind::UNIT);
}

#[test]
fn builtin_units_off_leaves_only_the_documents_own() {
    let doc = "#show: worksheet.with(builtin-units: false, units: (s: auto))\n$x := 2s$\n$y := 2\"m\"$\n$z := 2\"ms\"$\n$w := 2\"s\" -> \"min\"$\n$v := 2unit(\"km\")$";
    assert_eq!(kind(doc, "s$"), TokenKind::VARIABLE);
    assert_eq!(kind(doc, "\"m\"$"), TokenKind::VARIABLE);
    // prefixes still apply to its own units
    assert_eq!(kind(doc, "\"ms\"$"), TokenKind::UNIT);
    assert_eq!(kind(doc, "\"s\" ->"), TokenKind::UNIT);
    assert_eq!(kind(doc, "\"min\"$"), TokenKind::VARIABLE);
    // explicit references don't reach built-in units either
    assert_eq!(kind(doc, "km\")$"), TokenKind::VARIABLE);
    let c = complete(&doc.replace("2unit(\"km\")$", "2 $"), &Context::default(), at(doc, "2unit") + 2).unwrap();
    let labels: Vec<_> = c.items.iter().filter(|i| i.kind == ItemKind::UNIT).map(|i| i.label.as_str()).collect();
    assert_eq!(labels, ["s"]);
    let scoped = "#show: worksheet\n#worksheet(units: (s: none))[Text $x := 2s$]\n$y := 3s$";
    assert_eq!(kind(scoped, "s$]"), TokenKind::VARIABLE);
    assert_eq!(kind(scoped, "s$"), TokenKind::VARIABLE);
    let tokens = document_tokens(scoped, &Context::default());
    let end = scoped.rfind("s$").unwrap() as u32;
    assert_eq!(tokens.iter().find(|t| t.start == end).unwrap().kind, TokenKind::VARIABLE);
}

#[test]
fn typst_bindings_take_precedence_only_for_unquoted_names() {
    let doc = "#show: worksheet.with(units: (to: \"10km\"))\n#let to = unit(\"m\")\n$\"km\" := 23to$\n$x := 2\"to\"$\n$y := 2\"km\"$";
    let context = Context::default();
    assert_eq!(kind(doc, "to$"), TokenKind::UNIT);
    assert_eq!(kind(doc, "\"to\"$"), TokenKind::UNIT);
    assert_eq!(kind(doc, "\"km\"$"), TokenKind::VARIABLE);
    // Typst's language server owns the binding and its refactoring.
    let bare = at(doc, "to$");
    assert!(definition(doc, &context, bare).is_none());
    assert!(hover(doc, &context, bare).unwrap().markdown.contains("unit shortcut"));
    assert!(rename_target(doc, &context, bare).is_none());
    assert!(rename(doc, &context, bare, "other").is_empty());
    let unit = at(doc, "\"to\"$") + 1;
    assert_eq!(definition(doc, &context, unit).unwrap().start, at(doc, "to:"));
    assert!(rename(doc, &context, unit, "other").iter().all(|e| e.start != bare));
}

#[test]
fn quoted_parameter_types_do_not_resolve_typst_bindings() {
    let doc = "#show: worksheet\n#let second = unit(\"s\")\n$f(\"xaef\": second) := \"xaef\"$\n$g(\"xaef\": \"second\") := \"xaef\"$";
    assert_eq!(kind(doc, "second)"), TokenKind::UNIT);
    assert_eq!(kind(doc, "\"second\""), TokenKind::VARIABLE);
    let with_unit = doc.replace("#show: worksheet", "#show: worksheet.with(units: (second: \"s\"))");
    assert_eq!(kind(&with_unit, "second)"), TokenKind::UNIT);
    assert_eq!(kind(&with_unit, "\"second\""), TokenKind::UNIT);
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
fn math_unit_completion_prefers_quotes_and_replaces_an_existing_hash() {
    for typed in ["34s|", "34#s|", "34 #s|"] {
        let doc = format!("#show: worksheet.with(builtin-units: false, units: (s: \"s\"))\n#let s = unit(\"s\")\n$z := {typed}$");
        let result = applied_completion(&doc, "s");
        assert!(result.ends_with("34#s$") || result.ends_with("34 #s$"), "{result}");
    }
}

#[test]
fn explicit_single_letter_unit_references_have_a_fix_and_gray_hash() {
    let doc = "#show: worksheet.with(builtin-units: false, units: (s: auto))\n#let s = unit(\"s\")\n😅 $z := 34s$";
    let offset = at(doc, "s$");
    let fixes = unit_fixes(doc, &Context::default(), offset);
    assert!(fixes.iter().any(|f| (f.start, f.end, f.text.as_str()) == (offset, offset, "#")), "{fixes:?}");
    assert!(diagnostics(doc, &Context::default()).iter().any(|p| p.start == offset && p.message.contains("#s")));
    let fixed = doc.replace("34s", "34#s");
    assert_eq!(kind(&fixed, "#s$"), TokenKind::UNIT_REFERENCE);
    assert!(unit_fixes(&fixed, &Context::default(), at(&fixed, "s$")).is_empty());
    assert!(!diagnostics(&fixed, &Context::default()).iter().any(|p| p.message.contains("bare single letters")));
    let number = doc.replace("unit(\"s\")", "5");
    // no binding to reference: only quoting the unit is left
    assert!(unit_fixes(&number, &Context::default(), at(&number, "s$")).iter().all(|f| f.text == "\"s\""));
    let variable = doc.replace("😅 $z", "$s := 5$\n😅 $z");
    assert!(unit_fixes(&variable, &Context::default(), at(&variable, "s$")).is_empty());
}

#[test]
fn greek_unit_names_share_lookup_completion_and_concealing() {
    let doc = "#show: worksheet.with(units: (alpha: \"m\"))\n#let α = unit(\"alpha\")\n$x := 34\"alpha\"$\n$y := 34\"α\"$\n$z := 34#α$";
    assert_eq!(kind(doc, "\"alpha\"$"), TokenKind::UNIT);
    assert_eq!(kind(doc, "\"α\"$"), TokenKind::UNIT);
    assert_eq!(kind(doc, "#α$"), TokenKind::UNIT_REFERENCE);
    let target = definition(doc, &Context::default(), at(doc, "alpha\"$")).unwrap();
    assert_eq!(target.start, at(doc, "alpha:"));
    for needle in ["alpha\")", "alpha\"$"] {
        let offset = at(doc, needle);
        assert!(conceals(doc, &Context::default()).iter().any(|c| c.start == offset && c.length == 5 && c.character == 'α' as u32), "{needle}");
    }
    let completed = applied_completion("#show: worksheet.with(units: (alpha: \"m\"))\n$x := 34alp|$", "α");
    assert!(completed.ends_with("34\"α\"$"), "{completed}");
}

#[test]
fn uppercase_greek_units_share_lookup_completion_and_concealing() {
    for (name, symbol) in [("Alpha", 'Α'), ("Beta", 'Β'), ("Upsilon", 'Υ'), ("Omicron", 'Ο')] {
        let doc = format!("#show: worksheet.with(units: ({name}: 1))\n#let custom = unit(\"{name}\")\n$t := 5custom =$\n$x := 5\"{symbol}\"$");
        assert_eq!(kind(&doc, &format!("\"{symbol}\"$")), TokenKind::UNIT);
        let offset = at(&doc, &format!("{name}\")"));
        assert!(conceals(&doc, &Context::default()).iter().any(|c| c.start == offset && c.length == name.len() as u32 && c.character == symbol as u32));
        let marked = format!("#show: worksheet.with(units: ({name}: 1))\n$t := 5\"{name}|\"$");
        assert!(applied_completion(&marked, &symbol.to_string()).ends_with(&format!("5\"{symbol}\"$")));
    }
}

#[test]
fn unit_call_completion_uses_local_overlays_without_validating_future_uses() {
    let early = "#let α = unit(\"|\")\n#show: worksheet.with(units: (alpha: \"m\"))\n$x := 34#α$";
    let caret = early.find('|').unwrap();
    let doc = early.replacen('|', "", 1);
    let result = complete(&doc, &Context::default(), early[..caret].encode_utf16().count() as u32).unwrap();
    assert!(!result.items.iter().any(|item| item.label == "α"));
    assert!(result.items.iter().any(|item| item.label == "s"));
    let local = "#show: worksheet.with(units: (alpha: \"m\"))\n#let α = unit(\"alp|\")";
    assert!(applied_completion(local, "α").ends_with("unit(\"α\")"));
    let future = "#let α = unit(\"alpha\")\n#show: worksheet.with(units: (alpha: \"m\"))\n$x := 34#α$";
    assert!(!diagnostics(future, &Context::default()).iter().any(|p| p.message.contains("unknown unit")));
}

#[test]
fn constructor_completion_does_not_leak_nested_overlays() {
    let marked = "#worksheet(units: (alpha: \"m\"))[\n#worksheet(units: (beta: \"s\"))[Text]\n#let α = unit(\"|\")\n]";
    let caret = marked.find('|').unwrap();
    let doc = marked.replacen('|', "", 1);
    let result = complete(&doc, &Context::default(), marked[..caret].encode_utf16().count() as u32).unwrap();
    assert!(result.items.iter().any(|item| item.label == "α"));
    assert!(!result.items.iter().any(|item| item.label == "β"));
}

#[test]
fn quoted_unit_fix_is_available_without_a_typst_binding() {
    let document = "#show: worksheet.with(builtin-units: false, units: (alpha: \"m\"))\n$x := 34alpha$";
    let fix = unit_fixes(document, &Context::default(), at(document, "alpha$")).remove(0);
    assert_eq!(fix.text, "\"α\"");
    assert_eq!(fix.end - fix.start, 5);
}

#[test]
fn single_letter_units_require_quotes_outside_annotations() {
    let doc = "#show: worksheet\n$f(x: A) := x$\n$f(1A) =$\n$f(1\"A\") =$";
    assert_eq!(kind(doc, "A) :="), TokenKind::UNIT);
    assert_eq!(kind(doc, "A) ="), TokenKind::VARIABLE);
    assert_eq!(kind(doc, "\"A\""), TokenKind::UNIT);
    let fix = unit_fixes(doc, &Context::default(), at(doc, "A) =")).remove(0);
    assert_eq!(fix.text, "\"A\"");
    let defined = doc.replace("$f(1A)", "$A := 2\"A\"$\n$f(1A)");
    assert!(unit_fixes(&defined, &Context::default(), at(&defined, "A) =")).is_empty());
}

#[test]
fn common_builtin_units_rank_before_uncommon_prefixes() {
    let doc = "#show: worksheet\n$x := 1 |$";
    let caret = doc.find('|').unwrap();
    let source = doc.replace('|', "");
    let result = complete(&source, &Context::default(), caret as u32).unwrap();
    let rank = |label| result.items.iter().position(|item| item.label == label).unwrap();
    assert!(rank("A") < rank("mA"));
    assert!(rank("mA") < rank("GA"));
    assert!(rank("s") < rank("ps"));
}

#[test]
fn conversions_preserve_single_letter_variables_and_units() {
    assert_eq!(from_math("f(1A) =").as_deref(), Some("f(1*A) ="));
    assert_eq!(from_math("f(1\"A\") =").as_deref(), Some("f(1A) ="));
    assert_eq!(to_math("f(1 A) =").as_deref(), Some("f(1 \"A\") ="));
}

#[test]
fn bare_math_cast_has_a_quote_fix() {
    let doc = "#show: worksheet\n😅 $x := U as \"A\"$";
    let offset = at(doc, "as ");
    let fixes = unit_fixes(doc, &Context::default(), offset);
    let fix = fixes.iter().find(|fix| fix.text == "\"as\"").unwrap();
    assert_eq!((fix.start, fix.end), (offset, offset + 2));
    let warning = diagnostics(doc, &Context::default()).into_iter().find(|problem| problem.message.contains("to cast")).unwrap();
    assert_eq!((warning.start, warning.end), (offset, offset + 2));
    let fixed = doc.replace(" as ", " \"as\" ");
    assert!(!diagnostics(&fixed, &Context::default()).iter().any(|problem| problem.message.contains("to cast")));
    assert!(unit_fixes(&fixed, &Context::default(), at(&fixed, "as\"")).is_empty());
    for source in ["$x := U as A$", "#show: worksheet\n```solveng\nx := U as A\n```", "#show: worksheet\nText as text", "#show: worksheet\n$x := \" as \"$"] {
        let fixes = unit_fixes(source, &Context::default(), at(source, "as"));
        assert!(!fixes.iter().any(|fix| fix.text == "\"as\""), "{source}");
    }
}

#[test]
fn overlay_declarations_are_generated_in_the_lexical_scope() {
    for doc in [
        "#show: worksheet.with(units: (ing: \"km\", i: \"km\"))\n$t := 1ing$",
        "#worksheet(units: (ing: \"km\", i: \"km\"))[$t := 1ing$]",
    ] {
        let fixes = unit_fixes(doc, &Context::default(), at(doc, "ing:"));
        let fix = fixes.iter().find(|f| f.title == "Declare unit overlay bindings").expect("declaration action");
        assert!(fix.text.contains("#let ing = unit(\"ing\")"));
        assert!(fix.text.contains("#let i = unit(\"i\")"));
        if doc.starts_with("#worksheet(") { assert_eq!(fix.start, at(doc, "[$") + 1); }
        let mut applied = doc.to_string();
        applied.insert_str(fix.start as usize, &fix.text);
        assert!(unit_fixes(&applied, &Context::default(), at(&applied, "ing:")).iter().all(|f| f.title != "Declare unit overlay bindings"));
    }
}


#[test]
fn shortcut_declarations_color_only_the_string_and_explain_the_detection() {
    let doc = "#show: worksheet.with(units: (iwef: \"km\"))\n#let cusom = unit(\"iwef\")\n$t := 1cusom$";
    let tokens = document_tokens(doc, &Context::default());
    let begin = at(doc, "#let");
    let end = at(doc, "$t");
    let declaration: Vec<_> = tokens.iter().filter(|t| begin <= t.start && t.start < end).collect();
    assert_eq!(declaration.len(), 1, "{declaration:?}");
    assert_eq!(declaration[0].start, at(doc, "iwef\""));
    assert_eq!(declaration[0].length, 4);
    assert_eq!(declaration[0].kind, TokenKind::UNIT);
    for needle in ["cusom =", "iwef\"", "cusom$"] {
        let h = hover(doc, &Context::default(), at(doc, needle)).unwrap();
        assert!(h.markdown.contains("`cusom` unit shortcut for `iwef`"), "{}", h.markdown);
        assert!(h.markdown.contains("km"), "{}", h.markdown);
    }
}

#[test]
fn plain_strings_are_not_detected_as_unit_shortcuts() {
    let doc = "#show: worksheet.with(units: (iwef: \"km\"))\n#let cusom = \"iwef\"\n$t := 1cusom$";
    let from = at(doc, "#let");
    let to = at(doc, "$t");
    assert!(document_tokens(doc, &Context::default()).iter().all(|t| t.start < from || t.start >= to));
    assert!(hover(doc, &Context::default(), at(doc, "cusom$")).is_none());
}
