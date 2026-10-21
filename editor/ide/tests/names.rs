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
        let code = format!("#show: worksheet\n```solveng\nDelta2tau := 3\nx := {prefix}|\n```");
        let (applied, _, item) = completion(&code, "Δ2τ");
        assert_eq!(item.insert, "Δ2τ", "{prefix}");
        assert!(applied.contains("x := Δ2τ\n"), "{applied}");
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
fn worksheet_strings_complete_escaped_carriers_and_keep_utf16_ranges() {
    for prefix in ["Delta2ta|", "Δ2t|", "\\\"Delta2ta|\\\""] {
        let document = format!("#worksheet(\"😅 := 1\nDelta2tau := 2\nx := {prefix}\")");
        let (applied, _, item) = completion(&document, "Δ2τ");
        assert_eq!(item.insert, "Δ2τ");
        assert!(applied.ends_with("x := Δ2τ\")"), "{applied}");
    }
}

#[test]
fn aliases_share_navigation_and_rename_in_every_context() {
    let document = "#show: worksheet\n$\"Delta2tau\" := 3$\n```solveng\nx := Δ2τ + Delta2tau\n```\n$y := \"Δ2τ\"$\n#worksheet(\"z := Delta2tau\")";
    let at = document.find("Δ2τ +").unwrap();
    let offset = document[..at].encode_utf16().count() as u32;
    let target = definition(document, &Context::default(), offset).unwrap();
    let text: Vec<u16> = document.encode_utf16().collect();
    assert_eq!(String::from_utf16(&text[target.start as usize..target.end as usize]).unwrap(), "\"Delta2tau\"");
    assert_eq!(usages(document, &Context::default(), offset).len(), 5);
    let edits = rename(document, &Context::default(), offset, "😅2tau");
    assert_eq!(edits.len(), 5);
    assert!(edits.iter().any(|e| e.text == "\"😅2τ\""));
}

#[test]
fn functions_normalize_and_completion_keeps_existing_parentheses() {
    let document = "#show: worksheet\n```solveng\nDelta2tau(x) := 2 x\ny := Delta2t|(3)\n```";
    let (applied, _, item) = completion(document, "Δ2τ(x)");
    assert_eq!(item.insert, "Δ2τ");
    assert!(applied.contains("y := Δ2τ(3)"), "{applied}");
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
    let document = "#show: worksheet\n```solveng\nDeltatau := 1\nDelta\"\"tau := 2\nx := Deltatau|\n```";
    let (_, result, _) = completion(document, "Deltatau");
    assert_eq!(result.items[0].label, "Deltatau");
}

#[test]
fn function_parameters_match_aliases_and_show_normalized_signatures() {
    let document = "#show: worksheet\n```solveng\nf(Delta2tau) := Delta2tau + Δ2τ\ny := f|\n```";
    let (_, _, item) = completion(document, "f(Δ2τ)");
    assert_eq!(item.insert, "f($0)");
    let text = document.replacen('|', "", 1);
    let at = text.find("Δ2τ\n").unwrap();
    let target = definition(&text, &Context::default(), text[..at].encode_utf16().count() as u32).unwrap();
    let chars: Vec<u16> = text.encode_utf16().collect();
    assert_eq!(String::from_utf16(&chars[target.start as usize..target.end as usize]).unwrap(), "Delta2tau");
}

#[test]
fn solved_function_unknowns_are_local_and_do_not_change_the_signature() {
    for statement in [
        "$\"circle\"(x, r) := x^2 + b^2 = r^2 -> solve(b)$",
        "```solveng\ncircle(x, r) := x^2 + b^2 = r^2 -> solve(b)\n```",
        "#worksheet(\"circle(x, r) := x^2 + b^2 = r^2 -> solve(b)\")",
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
    for statement in ["$x´ := 3$", "```solveng\nx´ := 3\n```"] {
        let document = format!("#show: worksheet\n{statement}\n```solveng\ny := x´|\n```");
        let (applied, _, item) = completion(&document, "x´");
        assert_eq!(item.insert, "x´");
        assert!(applied.contains("y := x´\n"));
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

//#show: worksheet-config.with(
//units: (über: auto, üm: "10^3 über"),
//)

#let tri = math.op("tri")

#worksheet(evaluate: true)[
  $ "pytagoras"(a, b, c) := a^2 + b^2 = c^2 $
  $ "circle"(x, r) := "pytagoras"(x,b,r) -> solve(b) $
  $ r` := 3 $
  #graph(
    "tri(2x)",
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
fn math_operators_link_graph_calls_to_the_typst_binding() {
    for name in ["tri", "rect", "custom"] {
        let document = format!("#let {name} = math.op(\"{name}\")\n#worksheet(evaluate: true)[\n  #graph($ {name}(2x) $)\n  $ {name}(x) $\n]");
        let ctx = Context::default();
        let at = document.find(&format!("{name}(2x)")).unwrap() as u32;
        let target = definition(&document, &ctx, at).unwrap();
        assert_eq!((target.start, target.end), (5, 5 + name.len() as u32));
        assert_eq!(usages(&document, &ctx, at).len(), 3, "{name}");
        assert_eq!(rename_target(&document, &ctx, at).unwrap().name, name);
        let edits = rename(&document, &ctx, at, "pulse");
        assert_eq!(edits.len(), 3, "{name}");
        assert!(edits.iter().any(|e| e.start == 5 && e.text == "pulse"));
        assert!(edits.iter().any(|e| e.start == at && e.text == "pulse"));
        assert!(edits.iter().all(|e| e.text == "pulse"));
        assert!(rename(&document, &ctx, at, "😅2tau").is_empty());
        assert!(rename(&document, &ctx, at, "2x").is_empty());
    }
}

#[test]
fn math_operator_bindings_follow_lexical_scope_and_shadowing() {
    let document = "#let tri = math.op(\"tri\")\n#worksheet[\n#graph($ tri(x) $)\n#let tri = math.op(\"triangle\")\n#graph($ tri(2x) $)\n]\n#worksheet[ #graph($ tri(3x) $) ]\n#let tri = 3\n#graph($ tri(4x) $)";
    let ctx = Context::default();
    let at = |s| document.find(s).unwrap() as u32;
    assert_eq!(definition(document, &ctx, at("tri(x)")).unwrap().start, 5);
    assert_eq!(definition(document, &ctx, at("tri(2x)")).unwrap().start, at("tri = math.op(\"triangle"));
    assert_eq!(definition(document, &ctx, at("tri(3x)")).unwrap().start, 5);
    assert!(definition(document, &ctx, at("tri(4x)")).is_none());
    assert_eq!(usages(document, &ctx, 5).len(), 3);
}

#[test]
fn builtin_functions_without_bindings_are_not_graph_parameters() {
    let document = "#graph(\"tri(x) + sin(x)\")";
    let ctx = Context::default();
    assert!(usages(document, &ctx, document.find("tri(x)").unwrap() as u32).is_empty());
    assert!(usages(document, &ctx, document.find("sin(x)").unwrap() as u32).is_empty());
}
