// The tests Typewright's Kotlin core had, against the Rust port.

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

fn tokens_of(code: &str) -> Vec<String> {
    tokens(code).iter().map(|t| format!("{}:{}", slice(code, t.start, t.length), t.kind)).collect()
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

#[test]
fn highlights_values_units_and_variables() {
    assert_eq!(
        tokens_of("t := Q0 / (2 I) = [h] // half"),
        vec![
            t("t", TokenKind::VARIABLE), t(":=", TokenKind::OPERATOR), t("Q0", TokenKind::VARIABLE), t("2", TokenKind::NUMBER),
            t("I", TokenKind::VARIABLE), t("=", TokenKind::EVALUATE), t("h", TokenKind::UNIT), t("// half", TokenKind::COMMENT),
        ]
    );
    // the clause: keywords, a unit, the names solve takes
    assert_eq!(
        tokens_of("x^2 = 4 -> solve(x), digits(3)"),
        vec![
            t("x", TokenKind::VARIABLE), t("2", TokenKind::NUMBER), t("=", TokenKind::OPERATOR), t("4", TokenKind::NUMBER),
            t("->", TokenKind::EVALUATE), t("solve", TokenKind::KEYWORD), t("x", TokenKind::VARIABLE), t("digits", TokenKind::KEYWORD),
            t("3", TokenKind::NUMBER),
        ]
    );
    // = followed by a value is a step, not a result
    assert_eq!(tokens_of("V := 2 A = 2000 mA")[4], t("=", TokenKind::OPERATOR));
    assert_eq!(tokens_of("Q0/I = mA, sci")[2], t("=", TokenKind::EVALUATE));
    assert_eq!(
        tokens_of("g = 9.81 m/s^2; x = 3 k\"\"Omega"),
        vec![
            t("g", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("9.81", TokenKind::NUMBER), t("m", TokenKind::UNIT),
            t("s", TokenKind::UNIT), t("2", TokenKind::UNIT), t(";", TokenKind::OPERATOR), t("x", TokenKind::VARIABLE),
            t("=", TokenKind::OPERATOR), t("3", TokenKind::NUMBER), t("k\"\"Omega", TokenKind::UNIT),
        ]
    );
}

#[test]
fn highlights_brackets_and_functions() {
    assert_eq!(
        tokens_of("P = [5 V] I sqrt(R) 2 pi f"),
        vec![
            t("P", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("5", TokenKind::NUMBER), t("V", TokenKind::UNIT),
            t("I", TokenKind::VARIABLE), t("sqrt", TokenKind::FUNCTION), t("R", TokenKind::VARIABLE), t("2", TokenKind::NUMBER),
            t("pi", TokenKind::KEYWORD), t("f", TokenKind::VARIABLE),
        ]
    );
    // A letter after a number is a unit only if solveng knows it.
    assert_eq!(
        tokens_of("y = 2 R"),
        vec![t("y", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("2", TokenKind::NUMBER), t("R", TokenKind::VARIABLE)]
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
fn finds_bodies_of_solveng_code() {
    assert_eq!(body("```solveng\nx := 1\n```"), Some((10, 18)));
    // inline code, other languages and code without one are never solveng
    assert_eq!(body("```solveng x := 1```"), None);
    assert_eq!(body("```py\nx = 1\n```"), None);
    assert_eq!(body("`x = 1`"), None);
    assert_eq!(body("```\nx = 1\n```"), None);
}

#[test]
fn reads_whether_math_is_evaluated() {
    assert!(!evaluates("$x = 1$", &ctx("")));
    assert!(evaluates("#show: worksheet", &ctx("")));
    // evaluate: false leaves its content's math alone, but not nested worksheets' or what's passed to one
    let doc = "#worksheet[$a := 1$ #worksheet(evaluate: false)[$b := 2$ #worksheet[$c := 3$] #worksheet($d := 4$)] $e := 5$]";
    assert!(!math_tokens(doc, &ctx("")).is_empty());
    let coloured = |needle: &str| {
        let at = doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32;
        math_tokens(doc, &ctx("")).iter().any(|t| t.start <= at && at < t.start + t.length)
    };
    assert!(coloured("a := 1") && coloured("c := 3") && coloured("d := 4") && coloured("e := 5"));
    assert!(!coloured("b := 2"));
    // also for the rest of the block after a show rule
    let shown = "#show: worksheet.with(evaluate: false)\n$x := 1$";
    assert!(math_tokens(shown, &ctx("")).is_empty());
}

#[test]
fn reads_code_passed_to_a_worksheet() {
    let doc = "#worksheet(```\nt := 2\n```)";
    assert!(regions(doc, &ctx("")).iter().any(|r| r.kind == "code"));
}

#[test]
fn completes_definitions_from_the_whole_document() {
    let doc = "#show: worksheet\n#worksheet(\"R := 7.2 kΩ\", \"f(x) := x^2 + 1\")\nThe load draws $I := 30 mA$ and $Q_0/2 = I t$.\n```solveng\nP := [5 V] I\nU := |\n```";
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
    let units = labels("```solveng\nI := 30 m|\n```");
    assert!(units.contains(&"mA".into()) && units.contains(&"kΩ".into()) && units.contains(&"h".into()));
    assert!(labels("```solveng\nP := [5 |] I\n```").contains(&"A".into()));
    assert!(labels("```solveng\ng := 9.81 m/|\n```").contains(&"s".into()));
    assert!(labels("#worksheet-config(units: (bar: \"100 kPa\"))\n```solveng\np := 3 |\n```").contains(&"bar".into()));
    assert!(!labels("```solveng\nU := 2 I |\n```").contains(&"mA".into()));
    // The word being typed is replaced.
    assert_eq!(complete_at("```solveng\nI := 30 mA|\n```").unwrap().from, "```solveng\nI := 30 ".len() as u32);
}

#[test]
fn completes_clauses() {
    let after = labels("```solveng\nP := U I = |\n```");
    assert!(after.contains(&"mW".into()) && after.contains(&"digits".into()) && after.contains(&"solve".into()));
    assert!(labels("```solveng\nt = h, d|\n```").contains(&"digits".into()));
    // inside solve(…) the names
    let doc = "```solveng\nI := 2 A\nx^2 = 4 -> solve(|)\n```";
    assert!(labels(doc).iter().any(|l| name(l) == "I") && !labels(doc).contains(&"mA".into()));
    let c = complete_at("#show: worksheet\n$t := 3 h$ $t = |$").unwrap();
    assert!(c.exclusive && c.items.iter().any(|i| i.label == "min") && c.items.iter().any(|i| i.insert == "digits($0)"));
}

#[test]
fn only_in_solveng_code() {
    assert!(complete_at("```py\nx = |\n```").is_none());
    assert!(complete_at("text `x = |`").is_none());
    assert!(complete_at("#show: worksheet\ntext `x = |`").is_none());
    assert!(complete_at("inline ```solveng = |```").is_none());
    assert_eq!(complete_at("```solveng\nx := 1 // a |\n```").unwrap().items.len(), 0);
}

#[test]
fn completes_language_tags() {
    let c = complete_at("text\n```sol|").unwrap();
    assert_eq!(c.items.iter().map(|i| i.label.as_str()).collect::<Vec<_>>(), vec!["solveng"]);
    assert_eq!(complete_at("```|```").unwrap().items[0].label, "solveng");
    assert!(labels("```p|\n```").contains(&"python".into()));
    assert!(complete_at("text ` `|").is_none());
}

#[test]
fn completes_in_math() {
    let units = complete_at("#show: worksheet\n$t_\"half\" := 3 h$ and $I := 30 \"m|\"$").unwrap();
    assert!(units.exclusive);
    assert!(units.items.iter().any(|i| i.label == "mA"));
    let names = complete_at("#show: worksheet\n$t_\"half\" := 3 h$ $x := t|$").unwrap();
    assert!(!names.exclusive);
    assert_eq!(names.items.iter().find(|i| name(&i.label) == "t_half").unwrap().insert, "t_\"half\"");
    // Without a worksheet math is Typst's.
    assert!(complete_at("$x = 3 \"m|\"$").is_none());
}

#[test]
fn colors_math_like_code() {
    let doc = "#show: worksheet\n$I := 30 mA dot R_\"load\" + sqrt(x) = 1 A$";
    let found: Vec<String> = math_tokens(doc, &ctx("")).iter().map(|x| format!("{}:{}", slice(doc, x.start, x.length), x.kind)).collect();
    assert_eq!(
        found,
        vec![
            t("I", TokenKind::VARIABLE), t(":=", TokenKind::OPERATOR), t("30", TokenKind::NUMBER), t("mA", TokenKind::UNIT),
            t("dot", TokenKind::OPERATOR), t("R", TokenKind::VARIABLE), t("\"load\"", TokenKind::VARIABLE), t("sqrt", TokenKind::FUNCTION),
            t("x", TokenKind::VARIABLE), t("=", TokenKind::EVALUATE), t("1", TokenKind::NUMBER), t("A", TokenKind::UNIT),
        ]
    );
    // a letter alone after = is a variable: t = A compares
    let doc = "#show: worksheet\n$t = A$";
    let found: Vec<String> = math_tokens(doc, &ctx("")).iter().map(|x| format!("{}:{}", slice(doc, x.start, x.length), x.kind)).collect();
    assert_eq!(found, vec![t("t", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("A", TokenKind::VARIABLE)]);
    // solveng's names of units are units anywhere in math, unit("…") too
    let doc = "#show: worksheet\n$x := U \"as\" kΩ + unit(\"bar\")$";
    let found: Vec<String> = math_tokens(doc, &ctx("")).iter().map(|x| format!("{}:{}", slice(doc, x.start, x.length), x.kind)).collect();
    assert!(found.contains(&t("kΩ", TokenKind::UNIT)) && found.contains(&t("\"bar\"", TokenKind::UNIT)));
    assert!(math_tokens("#worksheet(evaluate: false)[$x = 1$]", &ctx("")).is_empty());
}

fn concealed(code: &str) -> String {
    let u = units16(code);
    let mut out: Vec<u16> = Vec::new();
    let mut i = 0;
    for c in conceals_of(code) {
        out.extend_from_slice(&u[i..c.start as usize]);
        let mut buf = [0u16; 2];
        out.extend_from_slice(char::from_u32(c.character).unwrap().encode_utf16(&mut buf));
        i = (c.start + c.length) as usize;
    }
    out.extend_from_slice(&u[i..]);
    String::from_utf16(&out).unwrap()
}

#[test]
fn conceals_names_units_and_operators() {
    assert_eq!(concealed("Delta_T = Q0 * rho -> k\"\"Omega").replace('→', "->").replace('\0', ""), "Δ_T = Q0 * ρ -> kΩ");
    // A word starting like a letter is a name of its own; glued, each part shows as it renders.
    assert_eq!(concealed("Deltas = DeltaT + Delta\"\"s + Delta\"\"Omega_R").replace('\0', ""), "Deltas = DeltaT + Δs + ΔΩ_R");
    assert_eq!(concealed("g = 9.81 m/s^2 -> uA // a * b"), "g = 9.81 m/s² → µA // a * b");
    assert_eq!(concealed("y = sqrt(pi x) <= alpha1"), "y = √(π x) ≤ α1");
    let doc = "```solveng\nx = 2 Omega\n```";
    let c = conceals(doc, &ctx(""));
    assert_eq!(
        c,
        vec![Conceal { start: 17, length: 5, character: 'Ω' as u32, element_start: 0, element_end: units16(doc).len() as u32 }]
    );
}

fn convert(doc: &str) -> Vec<String> {
    let text = doc.replacen('|', "", 1);
    let at = units16(&doc[..doc.find('|').unwrap()]).len() as u32;
    let u = units16(&text);
    conversions(&text, &ctx(""), at)
        .into_iter()
        .map(|c| {
            String::from_utf16(&u[..c.start as usize]).unwrap() + &c.text + &String::from_utf16(&u[c.end as usize..]).unwrap()
        })
        .collect()
}

const WS: &str = "#import \"@preview/solveng:1.0.0\": *\n#show: worksheet\n";

#[test]
fn converts_math_to_code_and_back() {
    let i = WS;
    assert_eq!(convert(&format!("{i}$R := |7.2 kΩ$")), vec![format!("{i}```solveng\nR := 7.2 kΩ\n```")]);
    assert_eq!(convert(&format!("{i}$t := Q_0/I = 1 h, digits(2)|$")), vec![format!("{i}```solveng\nt := Q_0/I = [h], digits(2)\n```")]);
    assert_eq!(convert(&format!("{i}$I := frac(3450 mAh, 30 h) dot 2|$")), vec![format!("{i}```solveng\nI := (3450 mAh)/(30 h) * 2\n```")]);
    assert_eq!(convert(&format!("{i}$x| := A unit(\"A\")$")), vec![format!("{i}```solveng\nx := A [A]\n```")]);
    assert_eq!(convert(&format!("{i}$x + y = 3, x - y = 1 -> solve(x, y)|$")), vec![format!("{i}```solveng\nx + y = 3; x - y = 1 -> solve(x, y)\n```")]);
    assert!(convert(&format!("{i}$a => b|$")).is_empty());
    assert_eq!(
        convert(&format!("{i}```solveng\nR := U/I = [mA] // load\n|x^2 = 4 -> solve(x)\n```")),
        vec![format!("{i}$R := U/I = mA$ // load\n\n$x^2 = 4 -> solve(x)$")]
    );
    assert_eq!(convert(&format!("{i}```solveng\nx| := A [A]; p := 3 [bar]\n```")), vec![format!("{i}$x := A unit(\"A\"), p := 3 unit(\"bar\")$")]);
}

#[test]
fn reads_the_files_around() {
    assert_eq!(
        references("#import \"defaults.typ\": *\n#import \"@preview/solveng:1.0.0\": *\n#include \"ch/1.typ\""),
        vec!["defaults.typ", "ch/1.typ"]
    );
    // A chapter of a main file that sets everything up.
    let main = ctx("#show: worksheet\n#worksheet-config(units: (bar: \"100 kPa\"))\n$U := 5 V$");
    assert!(evaluates("text", &main));
    let chapter = "```solveng\nP := U |\n```";
    let c = complete(&chapter.replace('|', ""), &main, chapter.find('|').unwrap() as u32).unwrap();
    assert_eq!(c.items.iter().find(|i| name(&i.label) == "U").unwrap().detail, "= 5 V");
    assert!(complete("```solveng\np := 3 \n```", &main, 18).unwrap().items.iter().any(|i| i.label == "bar"));
    assert!(!math_tokens("$x = 1$", &main).is_empty());
}

#[test]
fn trusts_the_compilers_report() {
    let element = |path: &str, evaluate: bool| {
        format!(r#"{{"path":"{path}","start":0,"end":7,"text":"$a = \"1\"$","kind":"math","active":true,"evaluate":{evaluate}}}"#)
    };
    let report = format!(
        r#"{{"blocks":1,"elements":[{},{},{},{}],"defined":[],"units":[]}}"#,
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
fn conceals_the_strings_of_calls() {
    let doc = "#graph(\"alpha3 = cos(x)\", xrange: \"2 uA\")\n#worksheet(\"Delta_R := 2 * x\")";
    let c = conceals(doc, &ctx(""));
    let shown: Vec<(String, char)> = c
        .iter()
        .map(|x| (String::from_utf16(&units16(doc)[x.start as usize..(x.start + x.length) as usize]).unwrap(), char::from_u32(x.character).unwrap()))
        .collect();
    assert_eq!(shown, [("alpha".into(), 'α'), ("u".into(), 'µ'), ("Delta".into(), 'Δ')]);
    // Glued and subscripted letters in a string, its quotes escaped.
    let glued = "#worksheet(\"Delta\\\"\\\"tau := Delta_tau\")";
    let letters: Vec<char> = conceals(glued, &ctx("")).iter().filter_map(|x| char::from_u32(x.character)).filter(|&c| c != '\0').collect();
    assert_eq!(letters, ['Δ', 'τ', 'Δ', 'τ']);
    // Each string shows as typed while the caret is in it, quotes included.
    let first = &c[0];
    assert_eq!((first.element_start, first.element_end), (7, 24));
}

