use solveng_ide::*;

fn completion(marked: &str, label: &str) -> (String, Completions, Item) {
    let at = marked.find('|').unwrap();
    let offset = marked[..at].encode_utf16().count() as u32;
    let document = marked.replacen('|', "", 1);
    let result = complete(&document, &Context::default(), offset).unwrap_or_else(|| panic!("no completion: {marked}"));
    let matches: Vec<_> = result.items.iter().filter(|i| i.label == label).collect();
    assert_eq!(matches.len(), 1, "{marked}: {:?}", result.items.iter().map(|i| &i.label).collect::<Vec<_>>());
    let item = matches[0].clone();
    let mut text: Vec<u16> = document.encode_utf16().collect();
    text.splice(result.from as usize..result.to.unwrap_or(offset) as usize, item.insert.replace("$0", "").encode_utf16());
    (String::from_utf16(&text).unwrap(), result, item)
}

#[test]
fn completion_accepts_every_partial_greek_spelling_and_uses_readable_insertions() {
    for prefix in ["D", "De", "Del", "Delta", "Delta2", "Delta2t", "Delta2ta", "Delta2tau", "Δ", "Δ2", "Δ2t", "Δ2ta", "Δ2τ", "Delta\"\"2\"\"ta"] {
        let math = format!("#show: worksheet\n$\"Delta2tau\" := 3$\n$x := {prefix}|$");
        let (applied, _, item) = completion(&math, "Δ2τ");
        assert_eq!(item.insert, "\"Δ2τ\"", "{prefix}");
        assert!(applied.ends_with("x := \"Δ2τ\"$"), "{applied}");
    }
}

#[test]
fn quoted_prefixes_replace_the_whole_carrier_without_duplicate_quotes() {
    for prefix in ["\"Del|\"", "\"Delta2ta|u\"", "\"Δ2t|\"", "\"Del|"] {
        let document = format!("#show: worksheet\n$\"Delta2tau\" := 1$\n$x := {prefix}$");
        let (applied, _, _) = completion(&document, "Δ2τ");
        assert!(applied.ends_with("$x := \"Δ2τ\"$"), "{applied}");
    }
    let document = "#show: worksheet\n$Delta\"\"tau := 1$\n$x := \"Delta\\\"\\\"ta|\"$";
    let (applied, _, _) = completion(document, "Δτ");
    assert!(applied.ends_with("$x := \"Δτ\"$"), "{applied}");
}

#[test]
fn numeric_subscript_functions_complete_without_trailing_name_joins() {
    for name in ["g_2", "g_12", "g_2x"] {
        let expected = if name == "g_2x" { "g_\"2x\"" } else { name };
        for call in [format!("${name}|$"), format!("#graph(${name}|$)")] {
            let doc = format!("#show: worksheet\n${expected}(t) := t$\n{call}");
            let (applied, _, item) = completion(&doc, &format!("{name}(t)"));
            assert_eq!(item.insert, format!("{expected}($0)"));
            assert!(applied.ends_with(&call.replace(&format!("{name}|"), &format!("{expected}()"))), "{applied}");
        }
    }
    let doc = "#show: worksheet\n$g_2(t) := t$\n#graph($g_2|(3)$)";
    let (applied, _, item) = completion(doc, "g_2(t)");
    assert_eq!(item.insert, "g_2");
    assert!(applied.ends_with("#graph($g_2(3)$)"), "{applied}");
}

#[test]
fn call_strings_complete_escaped_carriers_and_keep_utf16_ranges() {
    for prefix in ["Delta2ta|", "Δ2t|", "\\\"Delta2ta|\\\""] {
        let document = format!("#show: worksheet\n$\"😅\" := 1$ $\"Delta2tau\" := 2$\n#clear(\"{prefix}\")");
        let (applied, _, item) = completion(&document, "Δ2τ");
        assert_eq!(item.insert, "Δ2τ");
        assert!(applied.ends_with("#clear(\"Δ2τ\")"), "{applied}");
    }
}

#[test]
fn aliases_share_navigation_and_rename_in_every_context() {
    let document = "#show: worksheet\n$\"Delta2tau\" := 3$\n$x := \"Δ2τ\" + \"Delta2tau\"$\n$y := \"Δ2τ\"$\n#worksheet($z := \"Delta2tau\"$)\n#clear(\"Delta2tau\")";
    let at = document.find("Δ2τ\" +").unwrap();
    let offset = document[..at].encode_utf16().count() as u32;
    let target = definition(document, &Context::default(), offset).unwrap();
    let text: Vec<u16> = document.encode_utf16().collect();
    assert_eq!(String::from_utf16(&text[target.start as usize..target.end as usize]).unwrap(), "\"Delta2tau\"");
    assert_eq!(usages(document, &Context::default(), offset).len(), 6);
    let edits = rename(document, &Context::default(), offset, "😅2tau");
    assert_eq!(edits.len(), 6);
    // math quotes the name, the string of a call takes it plain
    assert!(edits.iter().any(|e| e.text == "\"😅2τ\"") && edits.iter().any(|e| e.text == "😅2τ"), "{edits:?}");
}

#[test]
fn functions_normalize_and_completion_keeps_existing_parentheses() {
    let document = "#show: worksheet\n$\"Delta2tau\"(x) := 2 x$\n$y := Delta2t|(3)$";
    let (applied, _, item) = completion(document, "Δ2τ(x)");
    assert_eq!(item.insert, "\"Δ2τ\"");
    assert!(applied.contains("y := \"Δ2τ\"(3)"), "{applied}");
}

#[test]
fn subscript_carriers_complete_without_leaving_parentheses_or_quotes() {
    for prefix in ["x_ta|", "x_\"ta|\"", "x_(\"ta|\")", "\"x_tau|\""] {
        let document = format!("#show: worksheet\n$x_tau := 1$\n$y := {prefix}$");
        let (applied, _, item) = completion(&document, "x_τ");
        assert_eq!(item.insert, "x_τ");
        assert!(applied.ends_with("$y := x_τ$"), "{applied}");
    }
}

#[test]
fn exact_names_rank_before_alternate_greek_spellings() {
    let document = "#show: worksheet\n$\"Deltatau\" := 1$\n$Delta\"\"tau := 2$\n$x := \"Deltatau|\"$";
    let (_, result, _) = completion(document, "Deltatau");
    assert_eq!(result.items[0].label, "Deltatau");
}

#[test]
fn function_parameters_match_aliases_and_show_normalized_signatures() {
    let document = "#show: worksheet\n$f(\"Delta2tau\") := \"Delta2tau\" + \"Δ2τ\"$\n$y := f|$";
    let (_, _, item) = completion(document, "f(Δ2τ)");
    assert_eq!(item.insert, "f($0)");
    let text = document.replacen('|', "", 1);
    let at = text.find("Δ2τ\"$").unwrap();
    let target = definition(&text, &Context::default(), text[..at].encode_utf16().count() as u32).unwrap();
    let chars: Vec<u16> = text.encode_utf16().collect();
    assert_eq!(String::from_utf16(&chars[target.start as usize..target.end as usize]).unwrap(), "\"Delta2tau\"");
}

#[test]
fn solved_function_unknowns_are_local_and_do_not_change_the_signature() {
    for statement in [
        "$\"circle\"(x, r) := x^2 + b^2 = r^2 -> solve(b)$",
        "#worksheet($\"circle\"(x, r) := x^2 + b^2 = r^2 -> solve(b)$)",
    ] {
        let document = format!("#show: worksheet\n$b := 99$\n{statement}\n$z := b$");
        let at = document.find("b^2").unwrap();
        let offset = document[..at].encode_utf16().count() as u32;
        let target = definition(&document, &Context::default(), offset).unwrap();
        let text: Vec<u16> = document.encode_utf16().collect();
        assert_eq!(String::from_utf16(&text[target.start as usize..target.end as usize]).unwrap(), "b");
        let solve_at = document.find("solve(b)").unwrap() + "solve(".len();
        assert_eq!(target.start, document[..solve_at].encode_utf16().count() as u32);
        assert_eq!(usages(&document, &Context::default(), offset).len(), 2);
        let marked = format!("{document}\n$\"cir|\"$");
        let (_, _, item) = completion(&marked, "circle(x, r)");
        assert!(!item.insert.contains('b'));
    }
}

#[test]
fn acute_accents_are_part_of_completion_and_navigation_names() {
    for statement in ["$x´ := 3$", "#worksheet($x´ := 3$)"] {
        let document = format!("#show: worksheet\n{statement}\n$y := x´|$");
        let (applied, _, item) = completion(&document, "x´");
        // math quotes it, as for every name Typst's math wouldn't read as one
        assert_eq!(item.insert, "\"x´\"");
        assert!(applied.contains("y := \"x´\"$"), "{applied}");
        let text = document.replace('|', "");
        let at = text.rfind("x´").unwrap();
        let offset = text[..at].encode_utf16().count() as u32;
        let target = definition(&text, &Context::default(), offset).unwrap();
        let chars: Vec<u16> = text.encode_utf16().collect();
        assert_eq!(String::from_utf16(&chars[target.start as usize..target.end as usize]).unwrap(), "x´");
        assert_eq!(usages(&text, &Context::default(), offset).len(), 2);
    }
}

const USER_WORKSHEET: &str = r#"#import "@solveng/solveng:1.0.0": *

#import units: km

#worksheet(evaluate: true)[
  $ "pytagoras"(a, b, c) := a^2 + b^2 = c^2 $
  $ "circle"(x, r) := "pytagoras"(x,b,r) -> solve(b) $
  $ r` := 3 $
  #graph(
    $tri(2x)$,
    width: 4cm,
    height: 4cm,
    mark: "1",
    xrange: ("r","-r")
  )
]"#;

#[test]
fn user_worksheet_highlights_and_resolves_names() {
    for accent in ["´", "`"] {
        let document = USER_WORKSHEET.replace("r`", &format!("r{accent}"));
        let ctx = Context::default();
        let offset = |at: usize| document[..at].encode_utf16().count() as u32;
        let function = document.find("\"pytagoras\"").unwrap();
        let call = document.rfind("\"pytagoras\"").unwrap();
        assert_eq!(definition(&document, &ctx, offset(call + 2)).map(|d| d.start), Some(offset(function)), "{accent}");
        assert_eq!(usages(&document, &ctx, offset(function + 2)).len(), 2, "{accent}");
        assert_eq!(rename(&document, &ctx, offset(call + 2), "hypotenuse").len(), 2, "{accent}");
        let at = document.find(&format!("r{accent} :=")).unwrap();
        let tokens = document_tokens(&document, &ctx);
        assert!(tokens.iter().any(|t| t.start == offset(at) && t.length == 2 && t.kind == TokenKind::VARIABLE), "{accent}: {tokens:?}");
        assert!(rename_target(&document, &ctx, offset(at + 1)).is_some(), "{accent}");
    }
}

#[test]
fn builtin_operations_without_bindings_are_not_graph_parameters() {
    let document = "#graph($tri(x) + sin(x)$)";
    let ctx = Context::default();
    assert!(usages(document, &ctx, document.find("tri(x)").unwrap() as u32).is_empty());
    assert!(usages(document, &ctx, document.find("sin(x)").unwrap() as u32).is_empty());
}

#[test]
fn quoted_text_is_a_variable_never_a_unit() {
    for name in [r#""e-"10"#, r#""a+b""#, r#""bad/name"2"#, r#""q!""#] {
        let expression = format!("1{name}");
        let tokens = math_tokens_of(&expression);
        assert!(tokens.iter().any(|t| t.start == 1 && t.length == name.encode_utf16().count() as u32 && t.kind == TokenKind::VARIABLE), "{expression}: {tokens:?}");
        let document = format!("#show: worksheet\n$y(x) := {expression}$");
        assert!(diagnostics(&document, &Context::default()).is_empty(), "{document}");
    }
    // quoted text is a name, also where it spells a unit
    for unit in [r#""mA""#, r#""km""#, r#""k"Omega"#] {
        assert!(math_tokens_of(&format!("1{unit}")).iter().any(|t| t.start == 1 && t.kind == TokenKind::VARIABLE), "{unit}");
    }
}

#[test]
fn function_definitions_calls_and_operations_have_distinct_styles() {
    let code = "f(x) := sin(x) + cos(x)\ny := f(tri(1)) + rect(2)";
    let sources = [
        format!("#import operations: tri, rect\n#show: worksheet\n${}$", code.replace('\n', "$\n$")),
        format!("#import operations: *\n#worksheet(${}$)", code.replace('\n', "$, $")),
        format!("#let tri = math.op(\"tri\")\n#let rect = operation(\"rect\")\n#show: worksheet\n${}$", code.replace('\n', "$\n$")),
    ];
    for source in sources {
        let tokens = document_tokens(&source, &Context::default());
        // the code itself, not the import before it
        let code_start = source[..source.find("f(x)").unwrap()].encode_utf16().count() as u32;
        let source: Vec<u16> = source.encode_utf16().collect();
        let functions: Vec<_> = tokens.iter().filter(|t| t.start >= code_start).filter_map(|t| {
            let name = String::from_utf16(&source[t.start as usize..(t.start + t.length) as usize]).unwrap();
            ["f", "sin", "cos", "tri", "rect"].contains(&name.as_str()).then_some((name, t.kind))
        }).collect();
        assert_eq!(functions, vec![("f".into(), TokenKind::FUNCTION), ("sin".into(), TokenKind::BUILTIN_FUNCTION), ("cos".into(), TokenKind::BUILTIN_FUNCTION), ("f".into(), TokenKind::FUNCTION_CALL), ("tri".into(), TokenKind::BUILTIN_FUNCTION), ("rect".into(), TokenKind::BUILTIN_FUNCTION)]);
    }
}
