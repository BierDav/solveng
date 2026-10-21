// Names as the user's bugs showed them: units next to variables, quoted and long names, parameters, renaming.

use solveng_ide::*;

fn u16_at(doc: &str, needle: &str) -> u32 {
    doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32
}

fn text_at(doc: &str, start: u32, end: u32) -> String {
    let u: Vec<u16> = doc.encode_utf16().collect();
    String::from_utf16(&u[start as usize..end as usize]).unwrap()
}

fn kind_at(doc: &str, needle: &str) -> Option<u8> {
    let at = u16_at(doc, needle);
    document_tokens(doc, &Context::default()).into_iter().find(|t| t.start <= at && at < t.start + t.length).map(|t| t.kind)
}

/// The document with `edits` applied.
fn applied(doc: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|e| std::cmp::Reverse(e.start));
    let mut u: Vec<u16> = doc.encode_utf16().collect();
    for e in edits {
        assert!(e.path.is_none());
        u.splice(e.start as usize..e.end as usize, e.text.encode_utf16());
    }
    String::from_utf16(&u).unwrap()
}

const FULL: &str = "#show: worksheet\n";

/// A solveng code block of `lines`.
fn code(lines: &str) -> String {
    format!("```solveng\n{lines}\n```\n")
}

#[test]
fn a_star_after_a_number_multiplies_by_a_variable() {
    let doc = format!("{FULL}$ A := 4 A $\n{}", code("z := 5*A\ny := 5 A*s\nw := 5·A"));
    assert_eq!(kind_at(&doc, "A\ny"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(&doc, "A\n```"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(&doc, "A*s"), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "s\nw"), Some(TokenKind::UNIT));
    // So go to definition finds the A defined in math.
    let def = definition(&doc, &Context::default(), u16_at(&doc, "A\ny")).unwrap();
    assert_eq!(def.start, u16_at(&doc, "A := 4"));
}

#[test]
fn quoted_math_names_are_variables() {
    let doc = format!("{FULL}$ \"tews\" := 23 $\n$ x := 2 \"tews\" mA $\n{}", code("k := tews * 2"));
    assert_eq!(kind_at(&doc, "tews\" := 23"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(&doc, "mA"), Some(TokenKind::UNIT));
    let d = definition(&doc, &Context::default(), u16_at(&doc, "tews * 2")).unwrap();
    assert_eq!(text_at(&doc, d.start, d.end), "\"tews\"");
    assert_eq!(usages(&doc, &Context::default(), d.start).len(), 3);
}

#[test]
fn long_subscripts_belong_to_the_name() {
    let doc = format!("{FULL}$ \"test\"_(\"test\" Delta) := 3 $\n{}$ R_\"load\" := 1 $ {}", code("y := test_test\"\"Delta * 2"), code("q := R_load"));
    let d = definition(&doc, &Context::default(), u16_at(&doc, "test\"\"Delta *")).unwrap();
    assert_eq!(text_at(&doc, d.start, d.end), "\"test\"_(\"test\" Delta)");
    let d = definition(&doc, &Context::default(), u16_at(&doc, "R_load\n")).unwrap();
    assert_eq!(text_at(&doc, d.start, d.end), "R_\"load\"");
}

#[test]
fn parameters_belong_to_their_function() {
    let doc = format!("{FULL}{}#worksheet(\"g(alpha) := 2 alpha\")\n", code("x := 3\nf(x) := 2 x + x^2\ny := f(x)"));
    let param = u16_at(&doc, "x) := 2");
    let uses: Vec<String> = usages(&doc, &Context::default(), param).iter().map(|l| text_at(&doc, l.start, l.end)).collect();
    assert_eq!(uses.len(), 3);
    let body = u16_at(&doc, "x + x^2");
    assert_eq!(definition(&doc, &Context::default(), body).unwrap().start, param);
    // The x passed to f is the variable.
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "x)\n```")).unwrap().start, u16_at(&doc, "x := 3"));
    let renamed = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "alpha)"), "beta"));
    assert!(renamed.contains("#worksheet(\"g(beta) := 2 beta\")"), "{renamed}");
}

#[test]
fn renames_in_each_spelling() {
    let doc = format!("{FULL}$ R_\"load\" := 1 $\n{}#worksheet(\"p := R_load + q\")\n$ s := @R_\"load\" $\n", code("q := R_load * 2"));
    let at = u16_at(&doc, "R_load *");
    let target = rename_target(&doc, &Context::default(), at).unwrap();
    assert_eq!(target.name, "R_load");
    let out = applied(&doc, rename(&doc, &Context::default(), at, "tews"));
    assert_eq!(out, format!("{FULL}$ \"tews\" := 1 $\n{}#worksheet(\"p := tews + q\")\n$ s := @\"tews\" $\n", code("q := tews * 2")));
    // Typed as math, a glued name goes into a string escaped.
    let out = applied(&doc, rename(&doc, &Context::default(), at, "Delta\"\"r_\"in\""));
    assert!(out.contains("$ Δ\"\"r_\"in\" := 1 $") && out.contains("q := Δr_in * 2") && out.contains("#worksheet(\"p := Δr_in + q\")"), "{out}");
    let out = applied(&doc, rename(&doc, &Context::default(), at, "Delta\"\"r"));
    assert!(out.contains("#worksheet(\"p := Delta\\\"\\\"r + q\")"), "{out}");
    assert!(rename(&doc, &Context::default(), at, "2x").is_empty());
}

#[test]
fn docs_of_the_whole_function_name() {
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: worksheet-config.with(digits: 4)\n#show: worksheet-config\n";
    for needle in ["worksheet-config.with", "worksheet-config\n"] {
        let h = hover(doc, &Context::default(), u16_at(doc, needle) + 2).unwrap();
        assert!(h.markdown.starts_with("`worksheet-config`"), "{}", h.markdown);
    }
}

#[test]
fn a_graph_plots_against_its_parameter() {
    let doc = "#worksheet(\"k := 2\")\n#graph(\"q(..) = 12 A\", \"k A\")\n#graph(\"P(..) = U^2/R\")\n#graph(\"cos(w)\", \"w\")\n";
    // A is a variable in a curve, the parameter q takes, and what the unnamed curve is plotted against.
    assert_eq!(kind_at(doc, "A\", "), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "A\")"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "k A"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(doc, "q(.."), Some(TokenKind::FUNCTION));
    // Both of .. show, though the graph can't plot against two.
    assert_eq!(kind_at(doc, "U^2"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "R\")"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "w)"), Some(TokenKind::PARAMETER));
    // A renames in its curve.
    let out = applied(doc, rename(doc, &Context::default(), u16_at(doc, "A\", "), "I"));
    assert!(out.contains("#graph(\"q(..) = 12 I\", \"k A\")"), "{out}");
    assert!(hover(doc, &Context::default(), u16_at(doc, "A\", ")).unwrap().markdown.contains("parameter"));
}

#[test]
fn parameters_stand_out_and_resolve() {
    let doc = format!("{FULL}{}$ g(..) := U^2 unit(\"V\") $\n#graph(\"U(I: mA) = I * [2 Ω]\")\n", code("k := 2\nf(x: A, ..) := x * k + m"));
    for p in ["x: A", "x * k", "m\n", "U^2", "I: mA", "I * "] {
        assert_eq!(kind_at(&doc, p), Some(TokenKind::PARAMETER), "{p}");
    }
    assert_eq!(kind_at(&doc, "k + m"), Some(TokenKind::VARIABLE));
    // .. shows what it takes.
    let h = hover(&doc, &Context::default(), u16_at(&doc, "f(x")).unwrap();
    assert!(h.markdown.starts_with("```solveng\nf(x: A, m) "), "{}", h.markdown);
    let h = hover(&doc, &Context::default(), u16_at(&doc, "g(..")).unwrap();
    assert!(h.markdown.starts_with("```solveng\ng(U) "), "{}", h.markdown);
    // m renames with the function.
    let out = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "m\n"), "n"));
    assert!(out.contains("f(x: A, ..) := x * k + n"), "{out}");
}

#[test]
fn functions_in_math() {
    let doc = format!("{FULL}#worksheet-config(units: (ü: auto))\n$ q(x) := x unit(\"ü\") $\n{}", code("y := q(3)\nx := 5"));
    let d = definition(&doc, &Context::default(), u16_at(&doc, "q(3)")).unwrap();
    assert_eq!(d.start, u16_at(&doc, "q(x)"));
    // x is the parameter, not the variable defined below.
    let body = u16_at(&doc, "x unit");
    assert_eq!(definition(&doc, &Context::default(), body).unwrap().start, u16_at(&doc, "x) :="));
    let out = applied(&doc, rename(&doc, &Context::default(), body, "t"));
    assert!(out.contains("$ q(t) := t unit(\"ü\") $") && out.contains("x := 5"), "{out}");
}

#[test]
fn custom_units_are_found_and_renamed() {
    let doc = "#show: worksheet\n#worksheet-config(units: (ü: auto, mü: \"10ü\", \"kq\": \"2 ü\"))\n$ a := 3 ü $\n$ b := 2 \"kü\" $\n```solveng\nc := 4 ü\nd := a + c = [ü]\n```\n#lambdify(\"a\", \"a\", unit: \"ü\")\n";
    let at = u16_at(doc, "ü $");
    let d = definition(doc, &Context::default(), at).unwrap();
    assert_eq!(d.start, u16_at(doc, "ü: auto"));
    // The key, the values of other units, prefixed and quoted uses, code and call strings; not mü, a unit of its own.
    let found: Vec<String> = usages(doc, &Context::default(), at).iter().map(|l| text_at(doc, l.start, l.end)).collect();
    let ctx: Vec<String> = usages(doc, &Context::default(), at).iter().map(|l| text_at(doc, l.start.saturating_sub(4), l.end + 2)).collect();
    assert_eq!(found.len(), 8, "{ctx:?}");
    assert!(found.iter().all(|f| f == "ü"));
    let out = applied(doc, rename(doc, &Context::default(), at, "sheep"));
    assert!(out.contains("units: (sheep: auto, mü: \"10sheep\", \"kq\": \"2 sheep\")"), "{out}");
    assert!(out.contains("$ a := 3 \"sheep\" $") && out.contains("\"ksheep\"") && out.contains("c := 4 sheep") && out.contains("[sheep]") && out.contains("unit: \"sheep\""), "{out}");
    let target = rename_target(doc, &Context::default(), u16_at(doc, "ü\")\n")).unwrap();
    assert_eq!(target.name, "ü");
}

#[test]
fn parameters_with_units() {
    let doc = format!("{FULL}{}$ p(I: mA, R) := I R $\n{}", code("q(x: A) := x * [2 Ω]"), code("y := q(3)"));
    assert_eq!(kind_at(&doc, "A) :="), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "mA"), Some(TokenKind::UNIT));
    // The unit isn't a parameter; x and I are.
    let uses: Vec<String> = usages(&doc, &Context::default(), u16_at(&doc, "x: A")).iter().map(|l| text_at(&doc, l.start, l.end)).collect();
    assert_eq!(uses, ["x", "x"]);
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "I R $")).unwrap().start, u16_at(&doc, "I: "));
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "q(3)")).unwrap().start, u16_at(&doc, "q(x"));
}

#[test]
fn variables_show_their_unit() {
    let doc = format!("{FULL}$ V := 4 A $\n$ I := 0.004 A = mA $\n{}", code("W := 3 [V]\nU := V I\ny := U"));
    let at = |w: &str| u16_at(&doc, w);
    // Inferred from the value.
    assert!(hover(&doc, &Context::default(), at("V I")).unwrap().markdown.starts_with("```solveng\nV: A "));
    assert!(hover(&doc, &Context::default(), at("I\ny")).unwrap().markdown.starts_with("```solveng\nI: A "));
    assert!(hover(&doc, &Context::default(), at("W := ")).is_some());
    assert_eq!(definition(&doc, &Context::default(), at("I\ny")).unwrap().start, at("I := "));
    assert_eq!(kind_at(&doc, "mA"), Some(TokenKind::UNIT));
    // A formula's unit is the compiler's to tell.
    assert!(hover(&doc, &Context::default(), at("U\n```")).unwrap().markdown.starts_with("```solveng\nU "));
}

#[test]
fn implicit_parameters_as_set() {
    let lines = code("f() := x^2 + k\ng(y) := y + n");
    let with = |mode: &str| format!("{FULL}#worksheet-config(implicit-parameters: \"{mode}\")\n{lines}");
    // "no", the default: only .. takes the undefined rest.
    let doc = format!("{FULL}{lines}");
    assert_eq!(kind_at(&doc, "x^2"), Some(TokenKind::VARIABLE));
    let doc = with("no");
    assert_eq!(kind_at(&doc, "x^2"), Some(TokenKind::VARIABLE));
    // "empty": f() is f(..), g(y) takes only y.
    let doc = with("empty");
    assert_eq!(kind_at(&doc, "x^2"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(&doc, "n\n"), Some(TokenKind::VARIABLE));
    assert!(hover(&doc, &Context::default(), u16_at(&doc, "f()")).unwrap().markdown.starts_with("```solveng\nf(x, k)"));
    // "allow": every function.
    let doc = with("allow");
    assert_eq!(kind_at(&doc, "n\n"), Some(TokenKind::PARAMETER));
    assert!(hover(&doc, &Context::default(), u16_at(&doc, "g(y)")).unwrap().markdown.starts_with("```solveng\ng(y, n)"));
}

#[test]
fn casts() {
    let doc = format!("{FULL}{}$ J := U \"as\" mA $\n", code("U := 5 V\nI := U as mA\nq(x: A) := x\ny := q(U as A)\nz := 3 as A"));
    assert_eq!(kind_at(&doc, "as mA"), Some(TokenKind::KEYWORD));
    assert_eq!(kind_at(&doc, "mA\nq"), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "A)\nz"), Some(TokenKind::UNIT));
    // after a number as is the unit, attosecond: 3 as A is 3 as·A
    assert_eq!(kind_at(&doc, "as A\n"), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "as\" "), Some(TokenKind::KEYWORD));
    assert_eq!(kind_at(&doc, "mA $"), Some(TokenKind::UNIT));
    // The cast value is the variable defined above.
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "U as A")).unwrap().start, u16_at(&doc, "U := 5"));
    // Units complete after as, and 1 for a plain number.
    let typed = format!("{FULL}{}", code("U := 5 V\nI := U as "));
    let c = complete(&typed, &Context::default(), u16_at(&typed, "as \n") + 3).unwrap();
    assert!(c.items.iter().all(|i| i.kind == ItemKind::UNIT || (i.kind == ItemKind::KEYWORD && i.label == "1")));
}

#[test]
fn as_after_a_number_is_the_unit() {
    let doc = format!("{FULL}{}$ e := 3 \"as\" $\n", code("U := 5 V\na := 3 as\nb := U as as"));
    assert_eq!(kind_at(&doc, "as\nb"), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "as as"), Some(TokenKind::KEYWORD));
    assert_eq!(kind_at(&doc, "as\n```"), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "as\" $"), Some(TokenKind::UNIT));
}
