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

const FULL: &str = "#import \"@preview/solveng:1.0.0\": *\n#import units: *\n#show: math\n";

#[test]
fn a_bare_letter_after_a_number_is_the_variable() {
    let doc = format!("{FULL}$ A := 4 #A $\n$ z := 5 A $\n$ y := 5 #A dot s $");
    assert_eq!(kind_at(&doc, "A $\n$ y"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(&doc, "A dot"), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "s $"), Some(TokenKind::VARIABLE));
    // So go to definition finds the A defined in math.
    let def = definition(&doc, &Context::default(), u16_at(&doc, "A $\n$ y")).unwrap();
    assert_eq!(def.start, u16_at(&doc, "A := 4"));
}

#[test]
fn quoted_math_names_are_variables() {
    let doc = format!("{FULL}$ \"tews\" := 23 $\n$ x := 2 \"tews\" mA $\n$ k := \"tews\" dot 2 $\n#clear(\"tews\")");
    assert_eq!(kind_at(&doc, "tews\" := 23"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(&doc, "mA"), Some(TokenKind::UNIT));
    let d = definition(&doc, &Context::default(), u16_at(&doc, "tews\" dot")).unwrap();
    assert_eq!(text_at(&doc, d.start, d.end), "\"tews\"");
    assert_eq!(usages(&doc, &Context::default(), d.start).len(), 4);
}

#[test]
fn quoted_unit_names_are_variables() {
    let doc = format!("{FULL}$ \"km\" := 23 $\n$ x := \"km\" + 3 km $\n$ y := 2 \"km\" $");
    assert_eq!(kind_at(&doc, "km\" := 23"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(&doc, "km\" +"), Some(TokenKind::VARIABLE));
    // The unquoted name is the unit binding; quoted text is the math variable.
    assert_eq!(kind_at(&doc, "km $\n"), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "km\" $"), Some(TokenKind::VARIABLE));
    let at = u16_at(&doc, "km\" $");
    let d = definition(&doc, &Context::default(), at).unwrap();
    assert_eq!(text_at(&doc, d.start, d.end), "\"km\"");
    let uses: Vec<String> = usages(&doc, &Context::default(), at).iter().map(|l| text_at(&doc, l.start, l.end)).collect();
    assert_eq!(uses, ["\"km\"", "\"km\"", "\"km\""]);
    let h = hover(&doc, &Context::default(), at).unwrap().markdown;
    assert!(h.contains("km") && !h.contains("kilometre"), "{h}");
    // the unit isn't one of its uses
    assert!(definition(&doc, &Context::default(), u16_at(&doc, "km $\n")).is_none_or(|d| d.start == u16_at(&doc, "units: *")));
}

#[test]
fn long_subscripts_belong_to_the_name() {
    let doc = format!("{FULL}$ \"test\"_(\"test\" Delta) := 3 $\n$ y := \"test\"_(\"test\" Delta) dot 2 $\n$ R_\"load\" := 1 $ $ q := R_\"load\" $\n#clear(\"R_load\")");
    let d = definition(&doc, &Context::default(), u16_at(&doc, "\"test\"_(\"test\" Delta) dot")).unwrap();
    assert_eq!(text_at(&doc, d.start, d.end), "\"test\"_(\"test\" Delta)");
    let d = definition(&doc, &Context::default(), u16_at(&doc, "R_load\")")).unwrap();
    assert_eq!(text_at(&doc, d.start, d.end), "R_\"load\"");
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "R_\"load\" $\n")).unwrap().start, d.start);
}

#[test]
fn parameters_belong_to_their_function() {
    let doc = format!("{FULL}$ x := 3 $\n$ f(x) := 2 x + x^2 $\n$ y := f(x) $\n#math($g(alpha) := 2 alpha$)\n");
    let param = u16_at(&doc, "x) := 2");
    let uses: Vec<String> = usages(&doc, &Context::default(), param).iter().map(|l| text_at(&doc, l.start, l.end)).collect();
    assert_eq!(uses.len(), 3);
    let body = u16_at(&doc, "x + x^2");
    assert_eq!(definition(&doc, &Context::default(), body).unwrap().start, param);
    // The x passed to f is the variable.
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "x) $")).unwrap().start, u16_at(&doc, "x := 3"));
    let renamed = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "alpha)"), "beta"));
    assert!(renamed.contains("#math($g(β) := 2 β$)"), "{renamed}");
}

#[test]
fn renames_in_each_spelling() {
    let doc = format!("{FULL}$ R_\"load\" := 1 $\n$ q := R_\"load\" dot 2 $\n#math($p := R_\"load\" + q$)\n#clear(\"R_load\")\n");
    let at = u16_at(&doc, "R_\"load\" dot");
    let target = rename_target(&doc, &Context::default(), at).unwrap();
    assert_eq!(target.name, "R_\"load\"");
    let out = applied(&doc, rename(&doc, &Context::default(), at, "tews"));
    assert_eq!(out, format!("{FULL}$ \"tews\" := 1 $\n$ q := \"tews\" dot 2 $\n#math($p := \"tews\" + q$)\n#clear(\"tews\")\n"));
    // Rename emits the canonical name in each syntax: quoted in math, plain in a call's string.
    let out = applied(&doc, rename(&doc, &Context::default(), at, "Delta\"\"r_\"in\""));
    assert!(out.contains("$ \"Δr\"_\"in\" := 1 $") && out.contains("#clear(\"Δr_in\")"), "{out}");
    let out = applied(&doc, rename(&doc, &Context::default(), at, "R_load2"));
    assert!(out.contains("$ R_\"load2\" := 1 $") && out.contains("#clear(\"R_load2\")"), "{out}");
    assert!(rename(&doc, &Context::default(), at, "2x").iter().all(|e| e.text.contains("2\"\"x") || e.text.contains("2\\\"\\\"x")));
}

#[test]
fn docs_of_the_whole_function_name() {
    let doc = "#import \"@preview/solveng:1.0.0\": *\n#show: math.with(digits: 4)\n#show: math\n";
    for needle in ["math.with", "math\n"] {
        let h = hover(doc, &Context::default(), u16_at(doc, needle) + 2).unwrap();
        assert!(h.markdown.starts_with("`math`"), "{}", h.markdown);
    }
    // Like the completion's docs, every parameter shows.
    let doc = "#math(evaluate: true)[]";
    let h = hover(doc, &Context::default(), 2).unwrap();
    assert!(h.markdown.contains("### Parameters\n\n- `..body` (content)"), "{}", h.markdown);
    assert!(h.markdown.contains("- `hidden` (auto, bool) = `auto`: Hide calculated math"), "{}", h.markdown);
}

#[test]
fn a_graph_plots_against_its_parameter() {
    let doc = "#math($k := 2$)\n#graph($q(..) = 12 A \\ k A$)\n#graph($P(..) = U^2/R$)\n#graph($cos(w) \\ w$)\n";
    // A is a variable in a curve, the parameter q takes, and what the unnamed curve is plotted against.
    assert_eq!(kind_at(doc, "A \\"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "A$)"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "k A"), Some(TokenKind::VARIABLE));
    assert_eq!(kind_at(doc, "q(.."), Some(TokenKind::FUNCTION_CALL));
    // Both of .. show, though the graph can't plot against two.
    assert_eq!(kind_at(doc, "U^2"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "R$)"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(doc, "w)"), Some(TokenKind::PARAMETER));
    // A renames in its curve.
    let out = applied(doc, rename(doc, &Context::default(), u16_at(doc, "A \\"), "I"));
    assert!(out.contains("#graph($q(..) = 12 I \\ k A$)"), "{out}");
    assert!(hover(doc, &Context::default(), u16_at(doc, "A \\")).unwrap().markdown.contains("parameter"));
}

#[test]
fn parameters_stand_out_and_resolve() {
    let doc = format!("{FULL}$ k := 2 $\n$ f(x: #A, ..) := x dot k + m $\n$ g(..) := U^2 #V $\n#graph($U(I: mA) = I dot 2 kΩ$)\n");
    for p in ["x: #A", "x dot k", "m $", "U^2", "I: mA", "I dot"] {
        assert_eq!(kind_at(&doc, p), Some(TokenKind::PARAMETER), "{p}");
    }
    assert_eq!(kind_at(&doc, "k + m"), Some(TokenKind::VARIABLE));
    // .. shows what it takes.
    let h = hover(&doc, &Context::default(), u16_at(&doc, "f(x")).unwrap();
    assert!(h.markdown.starts_with("```text\nf(x: A, m) "), "{}", h.markdown);
    let h = hover(&doc, &Context::default(), u16_at(&doc, "g(..")).unwrap();
    assert!(h.markdown.starts_with("```text\ng(U) "), "{}", h.markdown);
    // m renames with the function.
    let out = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "m $"), "n"));
    assert!(out.contains("f(x: #A, ..) := x dot k + n"), "{out}");
}

#[test]
fn functions_in_math() {
    let doc = format!("{FULL}#let sheep = base-unit(\"sheep\")\n$ q(x) := x sheep $\n$ y := q(3) $\n$ x := 5 $");
    let d = definition(&doc, &Context::default(), u16_at(&doc, "q(3)")).unwrap();
    assert_eq!(d.start, u16_at(&doc, "q(x)"));
    // x is the parameter, not the variable defined below.
    let body = u16_at(&doc, "x sheep");
    assert_eq!(definition(&doc, &Context::default(), body).unwrap().start, u16_at(&doc, "x) :="));
    let out = applied(&doc, rename(&doc, &Context::default(), body, "t"));
    assert!(out.contains("$ q(t) := t sheep $") && out.contains("x := 5"), "{out}");
}

#[test]
fn quoted_functions_in_math() {
    let doc = format!("{FULL}$ \"pytagoras\"(a, b, c) := a^2 + b^2 = c^2 $\n#math($d := \"pytagoras\"(1, 2, 3)$)\n#clear(\"pytagoras\")\n");
    let out = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "a^2"), "side"));
    assert!(out.contains("$ \"pytagoras\"(\"side\", b, c) := \"side\"^2 + b^2 = c^2 $"), "{out}");
    // A longer function name stays quoted in math, and is written plain in the string.
    let out = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "pytagoras\"("), "hyp"));
    assert!(out.contains("$ \"hyp\"(a, b, c)") && out.contains("#math($d := \"hyp\"(1, 2, 3)$)") && out.contains("#clear(\"hyp\")"), "{out}");
}

#[test]
fn local_unit_bindings_can_be_renamed() {
    let doc = format!("{FULL}#let sheep = base-unit(\"sheep\")\n#let flock = unit($12 sheep$)\n$ a := 3 sheep $\n$ b := 2 flock = sheep $\n");
    let ctx = Context::default();
    for needle in ["sheep $", "flock = sheep", "sheep $\n"] {
        let at = u16_at(&doc, needle);
        assert_eq!(kind_at(&doc, needle), Some(TokenKind::UNIT), "{needle}");
        assert!(definition(&doc, &ctx, at).is_none(), "{needle}");
        assert!(!usages(&doc, &ctx, at).is_empty(), "{needle}");
        assert!(rename_target(&doc, &ctx, at).is_some(), "{needle}");
        assert!(!rename(&doc, &ctx, at, "goat").is_empty(), "{needle}");
    }
    // the variables next to them still rename
    assert_eq!(rename(&doc, &ctx, u16_at(&doc, "a :="), "c").len(), 1);
}

#[test]
fn parameters_with_units() {
    let doc = format!("{FULL}$ q(x: #A) := x dot 2 kΩ $\n$ p(I: mA, R) := I R $\n$ y := q(3) $");
    assert_eq!(kind_at(&doc, "A) :="), Some(TokenKind::UNIT));
    assert_eq!(kind_at(&doc, "mA"), Some(TokenKind::UNIT));
    // The unit isn't a parameter; x and I are.
    let uses: Vec<String> = usages(&doc, &Context::default(), u16_at(&doc, "x: #A")).iter().map(|l| text_at(&doc, l.start, l.end)).collect();
    assert_eq!(uses, ["x", "x"]);
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "I R $")).unwrap().start, u16_at(&doc, "I: "));
    assert_eq!(definition(&doc, &Context::default(), u16_at(&doc, "q(3)")).unwrap().start, u16_at(&doc, "q(x"));
}

#[test]
fn variables_show_their_unit() {
    let doc = format!("{FULL}#let flock = base-unit(\"flock\")\n$ V := 4 #A $\n$ I := 0.004 #A = mA $\n$ W := 3 flock $\n$ U := V I $\n$ y := U $");
    let at = |w: &str| u16_at(&doc, w);
    // Inferred from the value, units of solveng's module and the document's bindings alike.
    assert!(hover(&doc, &Context::default(), at("V I")).unwrap().markdown.starts_with("```text\nV: A "));
    assert!(hover(&doc, &Context::default(), at("I $")).unwrap().markdown.starts_with("```text\nI: A "));
    assert!(hover(&doc, &Context::default(), at("W := ")).unwrap().markdown.starts_with("```text\nW: flock "));
    assert_eq!(definition(&doc, &Context::default(), at("I $")).unwrap().start, at("I := "));
    assert_eq!(kind_at(&doc, "mA"), Some(TokenKind::UNIT));
    // A formula's unit is the compiler's to tell.
    assert!(hover(&doc, &Context::default(), at("U $")).unwrap().markdown.starts_with("```text\nU "));
}

#[test]
fn implicit_parameters_as_set() {
    let lines = "$ f() := x^2 + k $\n$ g(y) := y + n $";
    let with = |mode: &str| format!("{FULL}#math(implicit-parameters: \"{mode}\", evaluate: false)[]\n{lines}");
    // "no", the default: only .. takes the undefined rest.
    let doc = format!("{FULL}{lines}");
    assert_eq!(kind_at(&doc, "x^2"), Some(TokenKind::VARIABLE));
    let doc = with("no");
    assert_eq!(kind_at(&doc, "x^2"), Some(TokenKind::VARIABLE));
    // "empty": f() is f(..), g(y) takes only y.
    let doc = with("empty");
    assert_eq!(kind_at(&doc, "x^2"), Some(TokenKind::PARAMETER));
    assert_eq!(kind_at(&doc, "n $"), Some(TokenKind::VARIABLE));
    assert!(hover(&doc, &Context::default(), u16_at(&doc, "f()")).unwrap().markdown.starts_with("```text\nf(x, k)"));
    // "allow": every function.
    let doc = with("allow");
    assert_eq!(kind_at(&doc, "n $"), Some(TokenKind::PARAMETER));
    assert!(hover(&doc, &Context::default(), u16_at(&doc, "g(y)")).unwrap().markdown.starts_with("```text\ng(y, n)"));
}

#[test]
fn forgiving_rename_keeps_references_linked_in_every_syntax() {
    let doc = "#show: math\n$\"foo\" := 3$\n#math($x := \"foo\"$)\n#graph($\"foo\" * x$)";
    for typed in ["2x", "123", "Delta2tau", "2foo", "2x_tau", "😅2tau", "é2tau"] {
        let edits = rename(doc, &Context::default(), u16_at(doc, "foo\" *"), typed);
        assert_eq!(edits.len(), 3, "{typed}");
        let out = applied(doc, edits);
        let at = out.find('$').unwrap() as u32 + 1;
        assert_eq!(usages(&out, &Context::default(), at).len(), 3, "{typed}: {out} {:?} {:?}", usages(&out, &Context::default(), at), document_tokens(&out, &Context::default()));
    }
}

#[test]
fn numeric_renames_keep_definitions_and_completions() {
    let doc = "😅\n#show: math\n#math($\"foo\" := 3$, $z := \"foo\"$)\n#graph($\"foo\" * x$)";
    let ctx = Context::default();
    let edits = rename(doc, &ctx, u16_at(doc, "foo\" :="), "2x");
    assert_eq!(edits.len(), 3);
    let out = applied(doc, edits);
    let at = u16_at(&out, "2\"\"x :=");
    assert_eq!(usages(&out, &ctx, at).len(), 3, "{out}");
    let marked = format!("{out}\n$a := $");
    let completions = complete(&marked, &ctx, u16_at(&marked, "a := ") + 5).unwrap();
    assert!(completions.items.iter().any(|i| i.label == "2x" && i.insert == "2\"\"x"));
}

#[test]
fn numeric_rename_emits_math_and_escaped_strings_across_files() {
    let main = "#show: math\n$foo := 3$\n#include \"chapter.typ\"";
    let chapter = "#graph($foo * x$)\n#clear(\"foo\")";
    let ctx = Context::new("").with_report(&Report::default(), "chapter.typ", false)
        .with_files(vec![ContextFile { path: "main.typ".into(), text: main.into() }]).with_main("main.typ");
    let edits = rename(chapter, &ctx, u16_at(chapter, "foo"), "2x");
    assert_eq!(edits.len(), 3);
    assert!(edits.iter().any(|e| e.path.as_deref() == Some("main.typ") && e.text == "2\"\"x"));
    assert!(edits.iter().any(|e| e.path.is_none() && e.text == "2\"\"x"));
    assert!(edits.iter().any(|e| e.path.is_none() && e.text == "2\\\"\\\"x"), "{edits:?}");
    for invalid in ["^", "a+b", "2.5x", "a b", "x\u{200b}y", "\"unclosed"] {
        assert!(rename(chapter, &ctx, u16_at(chapter, "foo"), invalid).is_empty(), "{invalid}");
    }
}

#[test]
fn binding_renames_adjust_math_hashes() {
    for declaration in ["#let inch = unit($2.54 #units.cm$)", "#import units: cm as inch", "#import units: *\n#let inch = cm"] {
        let doc = format!("{FULL}{declaration}\n$1 inch -> inch$\n$2 #inch^2$\n#let copy = inch\n");
        for name in ["fluid_ounce", "fluid-ounce", "x"] {
            let renamed = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "inch ->"), name));
            assert!(renamed.contains(&format!("$1 #{name} -> #{name}$")), "{renamed}");
            assert!(renamed.contains(&format!("$2 #{name}^2$")), "{renamed}");
            assert!(renamed.contains(&format!("#let copy = {name}")), "{renamed}");
            let back = applied(&renamed, rename(&renamed, &Context::default(), u16_at(&renamed, &format!("{name} ->")), "inch"));
            assert!(back.contains("$1 inch -> inch$"), "{back}");
            assert!(back.contains("$2 inch^2$"), "{back}");
            assert!(back.contains("#let copy = inch"), "{back}");
        }
    }
}

#[test]
fn binding_renames_preserve_code_and_adjacent_math() {
    let doc = format!("{FULL}#let fluid_ounce = unit($28.413 #units.ml$)\n$x#fluid_ounce$\n$1#fluid_ounce$\n$frac(#fluid_ounce, 2)$\n#let copy = fluid_ounce\n#let show-unit() = fluid_ounce\n");
    let out = applied(&doc, rename(&doc, &Context::default(), u16_at(&doc, "fluid_ounce ="), "ounce"));
    assert!(out.contains("$x#ounce$"), "{out}");
    assert!(out.contains("$1ounce$"), "{out}");
    assert!(out.contains("$frac(ounce, 2)$"), "{out}");
    assert!(out.contains("#let copy = ounce"), "{out}");
    assert!(out.contains("#let show-unit() = ounce"), "{out}");
}
