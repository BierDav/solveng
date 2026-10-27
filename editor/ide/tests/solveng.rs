// The tests Typewright's Kotlin core had, against the Rust port, for calculations in math.

use solveng_ide::*;

/// A completion's name: variables are labelled with their unit (`V: A`).
fn name(label: &str) -> &str {
    label.split(": ").next().unwrap_or(label)
}

fn ctx(text: &str) -> Context {
    Context::new(text)
}

fn units16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn slice(s: &str, start: u32, len: u32) -> String {
    let u = units16(s);
    String::from_utf16(&u[start as usize..(start + len) as usize]).unwrap()
}

/// The tokens of the last line of `doc`, an equation.
fn tokens_of(doc: &str) -> Vec<String> {
    let from = units16(&doc[..doc.rfind('\n').map_or(0, |i| i + 1)]).len() as u32;
    document_tokens(doc, &ctx("")).iter().filter(|t| t.start >= from).map(|t| format!("{}:{}", slice(doc, t.start, t.length), t.kind)).collect()
}

/// Completions at the `|` in `doc`.
fn complete_at(doc: &str) -> Option<Completions> {
    let at = units16(&doc[..doc.find('|').unwrap()]).len() as u32;
    complete(&doc.replacen('|', "", 1), &ctx(""), at)
}

fn labels(doc: &str) -> Vec<String> {
    complete_at(doc).unwrap().items.into_iter().map(|i| i.label).collect()
}

fn t(word: &str, kind: u8) -> String {
    format!("{word}:{kind}")
}

const SOLVENG: &str = "#import \"@preview/solveng:1.0.0\": *\n";

#[test]
fn highlights_values_units_and_variables() {
    let ws = "#import units: h, mA, m, s, kΩ\n#import modifiers: solve, digits, sci\n#show: worksheet\n";
    let found = tokens_of(&format!("{ws}$t := Q0 / (2 I) = #h$"));
    assert_eq!(
        found,
        vec![
            t("t", TokenKind::VARIABLE), t(":=", TokenKind::OPERATOR), t("Q0", TokenKind::VARIABLE), t("2", TokenKind::NUMBER),
            t("I", TokenKind::VARIABLE), t("=", TokenKind::EVALUATE), t("#", TokenKind::UNIT_REFERENCE), t("h", TokenKind::UNIT),
        ]
    );
    // the clause: modifiers, the names solve takes
    assert_eq!(
        tokens_of(&format!("{ws}$x^2 = 4 -> solve(x), digits(3)$")),
        vec![
            t("x", TokenKind::VARIABLE), t("2", TokenKind::NUMBER), t("=", TokenKind::OPERATOR), t("4", TokenKind::NUMBER),
            t("->", TokenKind::EVALUATE), t("solve", TokenKind::KEYWORD), t("x", TokenKind::VARIABLE), t("digits", TokenKind::KEYWORD),
            t("3", TokenKind::NUMBER),
        ]
    );
    // = followed by a value is a step, not a result; a unit binding after it a result
    assert!(tokens_of(&format!("{ws}$V := 2 #s = 2000 mA$")).contains(&t("=", TokenKind::OPERATOR)));
    assert!(tokens_of(&format!("{ws}$Q0/I = mA, sci$")).contains(&t("=", TokenKind::EVALUATE)));
    assert_eq!(
        tokens_of(&format!("{ws}$g = 9.81 #m/#s^2, x = 3 kΩ$")),
        vec![
            t("g", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("9.81", TokenKind::NUMBER), t("#", TokenKind::UNIT_REFERENCE),
            t("m", TokenKind::UNIT), t("#", TokenKind::UNIT_REFERENCE), t("s", TokenKind::UNIT), t("2", TokenKind::NUMBER),
            t("x", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("3", TokenKind::NUMBER), t("kΩ", TokenKind::UNIT),
        ]
    );
}

#[test]
fn highlights_bindings_and_functions() {
    let ws = "#import units: V\n#import constants: pi\n#show: worksheet\n";
    assert_eq!(
        tokens_of(&format!("{ws}$P = 5 #V I sqrt(R) 2 pi f$")),
        vec![
            t("P", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("5", TokenKind::NUMBER), t("#", TokenKind::UNIT_REFERENCE),
            t("V", TokenKind::UNIT), t("I", TokenKind::VARIABLE), t("sqrt", TokenKind::BUILTIN_FUNCTION), t("R", TokenKind::VARIABLE),
            t("2", TokenKind::NUMBER), t("pi", TokenKind::CONSTANT), t("f", TokenKind::VARIABLE),
        ]
    );
    // A letter after a number is a variable: units are bindings.
    assert_eq!(
        tokens_of("#import units: V\n#show: worksheet\n$y = 2 V$"),
        vec![t("y", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("2", TokenKind::NUMBER), t("V", TokenKind::VARIABLE)]
    );
}

#[test]
fn knows_units_with_prefixes_and_spellings() {
    assert!(is_unit("mAh"));
    // A prefix goes with a spelled Greek letter glued, as in math.
    assert!(is_unit("k\"\"Omega") && is_unit("kΩ") && !is_unit("kOmega") && !is_unit("kOhm"));
    assert!(is_unit("uA"));
    assert!(is_unit("degC"));
    assert!(!is_unit("R"));
    assert!(!is_unit("Q0"));
}

#[test]
fn solveng_code_blocks_are_raw_text() {
    // Raw text is never calculated, whatever its language, nor is math inside it.
    for doc in ["#show: worksheet\n```solveng\nx := 1\n```", "#show: worksheet\n`$x := 1$`", "#show: worksheet\n```py\nx = 1\n```"] {
        assert!(document_tokens(doc, &ctx("")).is_empty(), "{doc}");
        assert!(regions(doc, &ctx("")).is_empty(), "{doc}");
        assert!(complete(doc, &ctx(""), units16(doc).len() as u32 - 5).is_none(), "{doc}");
    }
}

#[test]
fn reads_whether_math_is_evaluated() {
    assert!(!evaluates("$x = 1$", &ctx("")));
    assert!(evaluates("#show: worksheet", &ctx("")));
    // No worksheet around the math: nothing is evaluated, however the document is set up.
    let plain = "#import \"@solveng/solveng:1.0.0\": *\n\n$R := 5cm$\n";
    assert!(!evaluates(plain, &ctx("")));
    assert!(math_tokens(plain, &ctx("")).is_empty());
    assert!(document_tokens(plain, &ctx("")).is_empty());
    // Worksheet content defaults to evaluated; only its own `evaluate: false` turns that off. Math outside every
    // worksheet stays as written, also before a worksheet further down.
    let doc = "#worksheet[$a := 1$ #worksheet(evaluate: false)[$b := 2$ #worksheet[$c := 3$] #worksheet($d := 4$)] $e := 5$]";
    assert!(!math_tokens(doc, &ctx("")).is_empty());
    let coloured = |doc: &str, needle: &str| {
        let at = doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32;
        math_tokens(doc, &ctx("")).iter().any(|t| t.start <= at && at < t.start + t.length)
    };
    assert!(coloured(doc, "a := 1") && coloured(doc, "c := 3") && coloured(doc, "d := 4") && coloured(doc, "e := 5"));
    assert!(!coloured(doc, "b := 2"));
    let around = "$o := 1$\n#worksheet[$i := 2$]\n$p := 3$\n#show: worksheet\n$q := 4$\n";
    assert!(!coloured(around, "o := 1") && coloured(around, "i := 2"));
    assert!(!coloured(around, "p := 3") && coloured(around, "q := 4"));
    // also for the rest of the block after a show rule
    let shown = "#show: worksheet.with(evaluate: false)\n$x := 1$";
    assert!(math_tokens(shown, &ctx("")).is_empty());
}

#[test]
fn follows_worksheet_aliases() {
    let coloured = |doc: &str, needle: &str| {
        let at = doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32;
        math_tokens(doc, &ctx("")).iter().any(|t| t.start <= at && at < t.start + t.length)
    };
    // `#let setup = worksheet…` wraps like the worksheet itself, with its options.
    let doc = "#let setup = worksheet.with(evaluate: false)\n#show: setup\n$a := 1$";
    assert!(!coloured(doc, "a := 1"));
    let doc = "#let setup = worksheet.with(evaluate: false)\n#show: setup.with(evaluate: true)\n$a := 1$";
    assert!(coloured(doc, "a := 1"));
    let doc = "#let setup = worksheet\n#show: setup\n$a := 1$";
    assert!(coloured(doc, "a := 1"));
    // An alias imported from a file around, also renamed on the way.
    let files = vec![
        ContextFile { path: "defaults.typ".into(), text: "#let setup = worksheet.with(evaluate: false)\n#let plain = setup".into() },
        ContextFile { path: "more.typ".into(), text: "#import \"defaults.typ\": plain as wrap".into() },
    ];
    let context = Context::new("").with_files(files).with_report(&Report::default(), "main.typ", true);
    for (doc, want) in [("#import \"defaults.typ\": *\n#show: setup\n$a := 1$", false), ("#import \"more.typ\": *\n#show: wrap\n$a := 1$", false)] {
        let at = doc[..doc.find("a := 1").unwrap()].encode_utf16().count() as u32;
        assert_eq!(math_tokens(doc, &context).iter().any(|t| t.start <= at && at < t.start + t.length), want, "{doc}");
    }
}

#[test]
fn only_a_worksheet_that_includes_a_file_evaluates_it() {
    let chapter = "$I := U / 2$\n";
    let context = |main: &str| {
        Context::new("")
            .with_report(&Report::default(), "chapters/one.typ", false)
            .with_files(vec![ContextFile { path: "main.typ".into(), text: main.into() }])
            .with_main("main.typ")
    };
    let coloured = |main: &str| math_tokens(chapter, &context(main)).iter().any(|t| t.start == 1);
    // The wrapping worksheet in main counts for what it includes.
    assert!(coloured("#show: worksheet\n#include \"chapters/one.typ\"\n"));
    assert!(coloured("#worksheet[\n#include \"chapters/one.typ\"\n]\n"));
    // Before it, after it, or not included at all, the chapter stays as written.
    assert!(!coloured("#include \"chapters/one.typ\"\n#show: worksheet\n"));
    assert!(!coloured("#worksheet(evaluate: false)[\n#include \"chapters/one.typ\"\n]\n"));
    assert!(!coloured("#worksheet[\n$U := 5$\n]\n#include \"chapters/one.typ\"\n"));
    assert!(!coloured("#show: worksheet\n$U := 5$\n"));
}

#[test]
fn reads_only_equations_passed_to_a_worksheet() {
    // Code passed to `worksheet` isn't calculated: its arguments are solveng's, its raw text isn't.
    let doc = "#worksheet(```\nt := 2\n```)";
    let kinds: Vec<_> = regions(doc, &ctx("")).iter().map(|r| r.kind).collect();
    assert_eq!(kinds, ["call"]);
    assert!(document_tokens(doc, &ctx("")).is_empty());
    let doc = "#worksheet($t := 2$)";
    assert!(regions(doc, &ctx("")).iter().any(|r| r.kind == "math"));
}

#[test]
fn completes_definitions_from_the_whole_document() {
    let doc = "#import units: kΩ, mA\n#show: worksheet\n#worksheet($R := 7.2 kΩ$, $f(x) := x^2 + 1$)\nThe load draws $I := 30 mA$ and $Q_0/2 = I t$.\n$P := 5 I$\n$U := |$";
    let c = complete_at(doc).unwrap();
    let get = |l: &str| c.items.iter().find(|i| name(&i.label) == l).unwrap();
    assert_eq!(get("I").detail, "= 30 mA");
    assert_eq!(get("R").detail, "= 7.2 kΩ");
    assert_eq!(get("f(x)").insert, "f($0)");
    assert_eq!(get("sqrt").kind, ItemKind::FUNCTION);
    assert!(c.items.iter().any(|i| name(&i.label) == "P"));
    // Equations that aren't definitions define nothing by name.
    assert!(!c.items.iter().any(|i| name(&i.label) == "Q_0") && !c.items.iter().any(|i| name(&i.label) == "t"));
}

#[test]
fn completes_units_where_a_value_is_written() {
    let ws = format!("{SOLVENG}#import units: mA\n#let bar = unit($100 kPa$)\n#show: worksheet\n");
    let units = labels(&format!("{ws}$I := 30 m|$"));
    assert!(units.contains(&"mA".into()) && units.contains(&"kΩ".into()) && units.contains(&"h".into()) && units.contains(&"bar".into()));
    // after a # only units and constants, the # kept
    let c = complete_at(&format!("{ws}$I := 30 #|$")).unwrap();
    assert!(c.exclusive && c.items.iter().all(|i| i.kind == ItemKind::UNIT || i.kind == ItemKind::CONSTANT));
    assert_eq!(c.items.iter().find(|i| i.label == "s").unwrap().insert, "s");
    // a variable after a term isn't a value
    assert!(!labels(&format!("{ws}$U := 2 I |$")).contains(&"mA".into()));
    // The word being typed is replaced.
    let doc = format!("{ws}$I := 30 mA|$");
    assert_eq!(complete_at(&doc).unwrap().from, units16(&doc[..doc.find("mA|").unwrap()]).len() as u32);
}

#[test]
fn completes_clauses() {
    let ws = format!("{SOLVENG}#import units: h, min\n#show: worksheet\n");
    let after = labels(&format!("{ws}$P := U I = |$"));
    assert!(after.contains(&"mW".into()) && after.contains(&"digits".into()) && after.contains(&"solve".into()));
    assert!(labels(&format!("{ws}$t = #h, d|$")).contains(&"digits".into()));
    // inside solve(…) the names of the equation, not every defined one
    let doc = format!("{ws}$I := 2$\n$R := 3$\n$x^2 = 4 I -> solve(|)$");
    let solve = labels(&doc);
    assert!(solve.iter().any(|l| name(l) == "x") && solve.iter().any(|l| name(l) == "I") && !solve.iter().any(|l| name(l) == "R") && !solve.contains(&"mA".into()), "{solve:?}");
    assert_eq!(name(&solve[0]), "x", "undefined first");
    // after = the value is shown or compared: the clause, and what a comparison takes
    let c = complete_at(&format!("{ws}$t := 3 #h$ $t = |$")).unwrap();
    assert!(!c.exclusive && c.items.iter().any(|i| i.label == "min") && c.items.iter().any(|i| i.insert == "digits($0)"));
    assert!(c.items.iter().any(|i| name(&i.label) == "t"));
    assert_eq!(c.items.iter().find(|i| i.label == "h").unwrap().insert, "#h");
    let doc = format!("{ws}$y_\"a1\"(t) := t^2$\n$y_1(t) = y|$");
    let c = complete_at(&doc).unwrap();
    assert_eq!(c.items.iter().find(|i| i.label == "y_a1(t)").unwrap().insert, "y_\"a1\"($0)");
    // what's written after = decides: a comparison takes names, a clause modifiers and units
    let has = |c: &Completions, l: &str| c.items.iter().any(|i| name(&i.label) == l);
    let c = complete_at(&format!("{ws}$v := 3$\n$y_1(t) = v + |$")).unwrap();
    assert!(!c.exclusive && has(&c, "v") && !has(&c, "digits") && !has(&c, "min"));
    let c = complete_at(&format!("{ws}$v := 3$\n$v = #min, |$")).unwrap();
    assert!(c.exclusive && has(&c, "digits") && !has(&c, "v"));
    // after -> only the clause follows
    let c = complete_at(&format!("{ws}$y_\"a1\"(t) := t^2$\n$y_1(t) -> |$")).unwrap();
    assert!(c.exclusive && !c.items.iter().any(|i| i.label == "y_a1(t)") && c.items.iter().any(|i| i.label == "expand"));
}

#[test]
fn only_in_calculated_math() {
    assert!(complete_at("```py\nx = |\n```").is_none());
    assert!(complete_at("text `x = |`").is_none());
    assert!(complete_at("#show: worksheet\ntext `x = |`").is_none());
    assert!(complete_at("#show: worksheet\n```solveng\nx := |\n```").is_none());
    // no language tags: raw text is Typst's
    assert!(complete_at("text\n```sol|").is_none());
}

#[test]
fn completes_in_math() {
    let units = complete_at("#import units: mA, h\n#show: worksheet\n$t_\"half\" := 3 #h$ and $I := 30 m|$").unwrap();
    assert!(units.items.iter().any(|i| i.label == "mA"));
    let names = complete_at("#show: worksheet\n$t_\"half\" := 3$ $x := t|$").unwrap();
    assert!(!names.exclusive);
    assert_eq!(names.items.iter().find(|i| name(&i.label) == "t_half").unwrap().insert, "t_\"half\"");
    // Without a worksheet math is Typst's.
    assert!(complete_at("#import units: mA\n$x = 3 m|$").is_none());
}

#[test]
fn colors_math_with_its_bindings() {
    let doc = "#import units: mA, A\n#show: worksheet\n$I := 30 mA dot R_\"load\" + sqrt(x) = 1 #A$";
    assert_eq!(
        tokens_of(doc),
        vec![
            t("I", TokenKind::VARIABLE), t(":=", TokenKind::OPERATOR), t("30", TokenKind::NUMBER), t("mA", TokenKind::UNIT),
            t("dot", TokenKind::OPERATOR), t("R", TokenKind::VARIABLE), t("\"load\"", TokenKind::VARIABLE), t("sqrt", TokenKind::BUILTIN_FUNCTION),
            t("x", TokenKind::VARIABLE), t("=", TokenKind::EVALUATE), t("1", TokenKind::NUMBER), t("#", TokenKind::UNIT_REFERENCE), t("A", TokenKind::UNIT),
        ]
    );
    // a letter alone after = is a variable: t = A compares
    assert_eq!(tokens_of("#import units: A\n#show: worksheet\n$t = A$"), vec![t("t", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("A", TokenKind::VARIABLE)]);
    // quoted text is a variable even where it spells a unit; bindings are units
    let found = tokens_of("#import units: kΩ, A\n#show: worksheet\n$x := U #A + \"kΩ\" + kΩ$");
    assert!(found.contains(&t("\"kΩ\"", TokenKind::VARIABLE)) && found.contains(&t("kΩ", TokenKind::UNIT)) && found.contains(&t("A", TokenKind::UNIT)), "{found:?}");
    assert!(math_tokens("#worksheet(evaluate: false)[$x = 1$]", &ctx("")).is_empty());
}

#[test]
fn reads_the_files_around() {
    assert_eq!(
        references("#import \"defaults.typ\": *\n#import \"@preview/solveng:1.0.0\": *\n#include \"ch/1.typ\""),
        vec!["defaults.typ", "ch/1.typ"]
    );
    // A chapter of a main file that sets everything up.
    let main = ctx("#show: worksheet\n$U := 5$");
    assert!(evaluates("text", &main));
    let chapter = "$P := U |$";
    let c = complete(&chapter.replace('|', ""), &main, chapter.find('|').unwrap() as u32).unwrap();
    assert_eq!(c.items.iter().find(|i| name(&i.label) == "U").unwrap().detail, "= 5");
    assert!(!math_tokens("$x = 1$", &main).is_empty());
}

#[test]
fn trusts_the_compilers_report() {
    let element = |path: &str, evaluate: bool| {
        format!(r#"{{"path":"{path}","start":0,"end":7,"text":"$a = \"1\"$","kind":"math","active":true,"evaluate":{evaluate}}}"#)
    };
    let report = format!(
        r#"{{"blocks":1,"elements":[{},{},{},{}],"defined":[]}}"#,
        element("main.typ", true),
        element("ch/1.typ", false),
        element("ch/1.typ", true),
        element("ch/2.typ", false)
    );
    assert_eq!(compiled_evaluate(&report, "main.typ", true), 1);
    assert_eq!(compiled_evaluate(&report, "ch/1.typ", false), 1);
    assert_eq!(compiled_evaluate(&report, "ch/2.typ", false), 0);
    assert_eq!(compiled_evaluate(&report, "other.typ", false), -1);
    assert_eq!(compiled_evaluate(r#"{"blocks":1,"elements":[]}"#, "main.typ", true), 0);
    // Without reports (an older solveng) the settings decide.
    assert_eq!(compiled_evaluate(r#"{"blocks":0,"elements":[]}"#, "main.typ", true), -1);
    let context = ctx(&with_compiled("#show: worksheet", 0));
    assert!(!evaluates("text $x = 1$", &context));
    assert!(math_tokens("$x = 1$", &context).is_empty());
    assert_eq!(with_compiled("ctx", -1), "ctx");
    // The same through a parsed report.
    let c = Context::new("").with_report(&Report::parse(&report), "ch/1.typ", false);
    assert!(evaluates("", &c));
}

#[test]
fn conceals_the_names_in_strings_of_calls() {
    let doc = "#graph($alpha3 = cos(x)$, xrange: \"2 uA\")\n#clear(\"Delta_R\")";
    let c = conceals(doc, &ctx(""));
    let shown: Vec<(String, char)> = c
        .iter()
        .map(|x| (String::from_utf16(&units16(doc)[x.start as usize..(x.start + x.length) as usize]).unwrap(), char::from_u32(x.character).unwrap()))
        .collect();
    assert_eq!(shown, [("Delta".into(), 'Δ')]);
    // Glued letters in a string, its quotes escaped.
    let glued = "#clear(\"Delta\\\"\\\"tau\", \"Delta_tau\")";
    let letters: Vec<char> = conceals(glued, &ctx("")).iter().filter_map(|x| char::from_u32(x.character)).filter(|&c| c != '\0').collect();
    assert_eq!(letters, ['Δ', 'τ', 'Δ', 'τ']);
    // Each string shows as typed while the caret is in it, quotes included.
    let first = &c[0];
    assert_eq!((first.element_start, first.element_end), (doc.find("\"Delta").unwrap() as u32, doc.len() as u32 - 1));
}
