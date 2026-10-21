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
        tokens_of("t = Q0 / (2 I) -> h // half"),
        vec![
            t("t", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("Q0", TokenKind::VARIABLE), t("2", TokenKind::NUMBER),
            t("I", TokenKind::VARIABLE), t("->", TokenKind::OPERATOR), t("h", TokenKind::UNIT), t("// half", TokenKind::COMMENT),
        ]
    );
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
fn highlights_brackets_functions_and_inlining() {
    assert_eq!(
        tokens_of("P = [5 V] I sqrt(@R) 2 pi f"),
        vec![
            t("P", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("5", TokenKind::NUMBER), t("V", TokenKind::UNIT),
            t("I", TokenKind::VARIABLE), t("sqrt", TokenKind::FUNCTION), t("@R", TokenKind::ANNOTATION), t("2", TokenKind::NUMBER),
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
    assert_eq!(body("```solveng x = 1```", Integration::NONE), Some((10, 16)));
    assert_eq!(body("```py x = 1```", Integration::CODE), None);
    assert_eq!(body("`x = 1`", Integration::MATH), None);
    assert_eq!(body("`x = 1`", Integration::CODE), Some((1, 6)));
    assert_eq!(body("```\nx = 1\n```", Integration::FULL), Some((3, 10)));
}

#[test]
fn reads_the_integration() {
    assert_eq!(integration("$x = 1$", &ctx("")), Integration::NONE);
    assert_eq!(integration("#show: solve-block", &ctx("")), Integration::MATH);
    assert_eq!(integration("#show: solve-config.with(integration: \"full\")\n#show: solve-block", &ctx("")), Integration::FULL);
    assert_eq!(integration("#solve-config(integration: \"code\")\n#solve-block[]", &ctx("")), Integration::CODE);
    assert_eq!(integration("#solve-config(integration: \"none\")\n#solve-block[]", &ctx("")), Integration::NONE);
}

#[test]
fn completes_definitions_from_the_whole_document() {
    let doc = "#show: solve-block\n#def(\"R = 7.2 kΩ\", \"f(x) = x^2 + 1\")\nThe load draws $I = 30 \"mA\"$ and $Q_0/2 = I t$.\n```solveng\nP = [5 V] I\nU = |\n```";
    let c = complete_at(doc).unwrap();
    let get = |l: &str| c.items.iter().find(|i| name(&i.label) == l).unwrap();
    assert_eq!(get("I").detail, "= 30 mA");
    assert_eq!(get("R").detail, "= 7.2 kΩ");
    assert_eq!(get("f(x)").insert, "f($0)");
    assert_eq!(get("sqrt").kind, ItemKind::FUNCTION);
    assert!(c.items.iter().any(|i| name(&i.label) == "P"));
    // Equations that aren't definitions define nothing by name.
    assert!(!c.items.iter().any(|i| name(&i.label) == "Q_0"));
}

#[test]
fn completes_units_where_a_value_is_written() {
    let units = labels("```solveng\nI = 30 m|\n```");
    assert!(units.contains(&"mA".into()) && units.contains(&"kΩ".into()) && units.contains(&"h".into()));
    assert!(labels("```solveng\nP = [5 |] I\n```").contains(&"A".into()));
    assert!(labels("```solveng\nP = U I -> |\n```").contains(&"mW".into()));
    assert!(labels("```solveng\ng = 9.81 m/|\n```").contains(&"s".into()));
    assert!(labels("#solve-config(units: (bar: \"100 kPa\"))\n```solveng\np = 3 |\n```").contains(&"bar".into()));
    assert!(!labels("```solveng\nU = 2 I |\n```").contains(&"mA".into()));
    // The word being typed is replaced.
    assert_eq!(complete_at("```solveng\nI = 30 mA|\n```").unwrap().from, "```solveng\nI = 30 ".len() as u32);
}

#[test]
fn only_in_solveng_code() {
    assert!(complete_at("```py\nx = |\n```").is_none());
    assert!(complete_at("text `x = |`").is_none());
    assert!(complete_at("#show: solve-config.with(integration: \"code\")\n#show: solve-block\ntext `x = |`").is_some());
    assert!(complete_at("#show: solve-block\ntext `x = |`").is_none());
    assert!(complete_at("inline ```solveng = |```").is_some());
    assert_eq!(complete_at("```solveng\nx = 1 // a |\n```").unwrap().items.len(), 0);
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
    let units = complete_at("#show: solve-block\n$t_\"half\" = 3 \"h\"$ and $I = 30 \"m|\"$").unwrap();
    assert!(units.exclusive);
    assert!(units.items.iter().any(|i| i.label == "mA"));
    let names = complete_at("#show: solve-block\n$t_\"half\" = 3 \"h\"$ $x = t|$").unwrap();
    assert!(!names.exclusive);
    assert_eq!(names.items.iter().find(|i| name(&i.label) == "t_half").unwrap().insert, "t_\"half\"");
    // Without a solve-block math is Typst's.
    assert!(complete_at("$x = 3 \"m|\"$").is_none());
}

#[test]
fn colors_math_like_code() {
    let doc = "#show: solve-block\n$I = 30 \"mA\" dot R_\"load\" + sqrt(x)$";
    let found: Vec<String> = math_tokens(doc, &ctx("")).iter().map(|x| format!("{}:{}", slice(doc, x.start, x.length), x.kind)).collect();
    assert_eq!(
        found,
        vec![
            t("I", TokenKind::VARIABLE), t("=", TokenKind::OPERATOR), t("30", TokenKind::NUMBER), t("\"mA\"", TokenKind::UNIT),
            t("dot", TokenKind::OPERATOR), t("R", TokenKind::VARIABLE), t("\"load\"", TokenKind::VARIABLE), t("sqrt", TokenKind::FUNCTION),
            t("x", TokenKind::VARIABLE),
        ]
    );
    assert!(math_tokens("#show: solve-config.with(integration: \"code\")\n#show: solve-block\n$x = 1$", &ctx("")).is_empty());
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
    assert_eq!(concealed("Delta_T = Q0 * rho -> k\"\"Omega").replace('→', "->").replace('\0', ""), "Δ_T = Q₀ · ρ -> kΩ");
    // A word starting like a letter is a name of its own; glued, each part shows as it renders.
    assert_eq!(concealed("Deltas = DeltaT + Delta\"\"s + Delta\"\"Omega_R").replace('\0', ""), "Deltas = DeltaT + Δs + ΔΩ_R");
    assert_eq!(concealed("g = 9.81 m/s^2 -> uA // a * b"), "g = 9.81 m/s² → µA // a * b");
    assert_eq!(concealed("y = sqrt(pi x) <= alpha1"), "y = √(π x) ≤ α₁");
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

const IMP: &str = "#import \"@preview/solveng:0.2.0\": *\n";

#[test]
fn converts_def_to_math_and_back() {
    let i = IMP;
    assert_eq!(convert(&format!("{i}#d|ef(\"R = 7.2 kΩ\", \"t_half = Q_0/(2 I)\")")), vec![format!("{i}$R = 7.2 \"kΩ\"$ $t_\"half\" = Q_0/(2 I)$")]);
    assert_eq!(convert(&format!("{i}$R = |7.2 \"k\"Omega$")), vec![format!("{i}#def(\"R = 7.2 kΩ\")")]);
    assert_eq!(convert(&format!("{i}$I = frac(3450 \"mAh\", 30 \"h\") dot 2|$")), vec![format!("{i}#def(\"I = (3450 mAh)/(30 h) * 2\")")]);
    assert_eq!(convert(&format!("{i}$x| = A \"A\"$")), vec![format!("{i}#def(\"x = A [A]\")")]);
    // Options and names math can't write stay calls.
    assert!(convert(&format!("{i}#def(\"tol = 3\")|")).is_empty());
    assert!(convert(&format!("{i}#def(\"x = 3\", output: true)|")).is_empty());
    assert!(convert(&format!("{i}$a => b|$")).is_empty());
}

#[test]
fn converts_solve_to_code_and_back() {
    let i = IMP;
    assert_eq!(convert(&format!("{i}#sol|ve(\"R = U/I\", unit: \"mA\")")), vec![format!("{i}```solveng\nR = U/I -> mA\n```")]);
    assert_eq!(
        convert(&format!("{i}a #solve(\"Q0/I\", unit: \"h\", inline: true, steps: \"value\")| b")),
        vec![format!("{i}a ```solveng = Q0/I -> h``` b")]
    );
    assert_eq!(convert(&format!("{i}#solve(\"x + y = 3\", \"x - y = 1\")|")), vec![format!("{i}```solveng\nx + y = 3; x - y = 1\n```")]);
    assert!(convert(&format!("{i}#solve(\"R = U/I\", digits: 2)|")).is_empty());
    assert_eq!(
        convert(&format!("{i}```solveng\nR = U/I -> mA // load\n|x + y = 3; x - y = 1\n```")),
        vec![format!("{i}#solve(\"R = U/I\", unit: \"mA\") // load\n#solve(\"x + y = 3\", \"x - y = 1\")")]
    );
    assert_eq!(convert(&format!("{i}a ```solveng = t| -> h``` b")), vec![format!("{i}a #solve(\"t\", unit: \"h\", inline: true, steps: \"value\") b")]);
}

#[test]
fn reads_the_files_around() {
    assert_eq!(
        references("#import \"defaults.typ\": *\n#import \"@preview/solveng:0.2.0\": *\n#include \"ch/1.typ\""),
        vec!["defaults.typ", "ch/1.typ"]
    );
    // A chapter of a main file that sets everything up.
    let main = ctx("#show: solve-config.with(integration: \"full\")\n#show: solve-block\n#solve-config(units: (bar: \"100 kPa\"))\n$U = 5 \"V\"$");
    assert_eq!(integration("text", &main), Integration::FULL);
    assert_eq!(integration("#solve-config(integration: \"code\")", &main), Integration::CODE);
    let chapter = "text `P = U |`";
    let c = complete(&chapter.replace('|', ""), &main, chapter.find('|').unwrap() as u32).unwrap();
    assert_eq!(c.items.iter().find(|i| name(&i.label) == "U").unwrap().detail, "= 5 V");
    assert!(complete("`p = 3 `", &main, 7).unwrap().items.iter().any(|i| i.label == "bar"));
    assert!(!math_tokens("$x = 1$", &main).is_empty());
}

#[test]
fn trusts_the_compilers_report() {
    let element = |path: &str, integration: &str| {
        format!(r#"{{"path":"{path}","start":0,"end":7,"text":"$a = \"1\"$","kind":"math","active":true,"integration":"{integration}"}}"#)
    };
    let report = format!(
        r#"{{"blocks":1,"elements":[{},{},{}],"defined":[],"units":[]}}"#,
        element("main.typ", "math"),
        element("ch/1.typ", "code"),
        element("ch/1.typ", "math")
    );
    assert_eq!(compiled_integration(&report, "main.typ", true), Integration::MATH as i32);
    assert_eq!(compiled_integration(&report, "ch/1.typ", false), Integration::FULL as i32);
    assert_eq!(compiled_integration(&report, "other.typ", false), -1);
    assert_eq!(compiled_integration(r#"{"blocks":1,"elements":[]}"#, "main.typ", true), Integration::NONE as i32);
    // Without reports (an older solveng) the settings decide.
    assert_eq!(compiled_integration(r#"{"blocks":0,"elements":[]}"#, "main.typ", true), -1);
    let context = ctx(&with_compiled("#show: solve-block", Integration::CODE as i32));
    assert_eq!(integration("text $x = 1$", &context), Integration::CODE);
    assert!(math_tokens("$x = 1$", &context).is_empty());
    assert_eq!(with_compiled("ctx", -1), "ctx");
    // The same through a parsed report.
    let c = Context::new("").with_report(&Report::parse(&report), "ch/1.typ", false);
    assert_eq!(integration("", &c), Integration::FULL);
}

#[test]
fn conceals_the_strings_of_calls() {
    let doc = "#graph(\"alpha3 = cos(x)\", xrange: \"2 uA\")\n#def(\"Delta_R = 2 * x\")";
    let c = conceals(doc, &ctx(""));
    let shown: Vec<(String, char)> = c
        .iter()
        .map(|x| (String::from_utf16(&units16(doc)[x.start as usize..(x.start + x.length) as usize]).unwrap(), char::from_u32(x.character).unwrap()))
        .collect();
    assert_eq!(shown, [("alpha".into(), 'α'), ("3".into(), '₃'), ("u".into(), 'µ'), ("Delta".into(), 'Δ'), ("*".into(), '·')]);
    // Each string shows as typed while the caret is in it, quotes included.
    let first = &c[0];
    assert_eq!((first.element_start, first.element_end), (7, 24));
}
