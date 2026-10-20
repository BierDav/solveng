// What the Rust crate adds: solveng's calls, options, the document's units and the compiler's values.

use solveng_ide::*;

fn u16len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

fn slice(s: &str, start: u32, len: u32) -> String {
    let u: Vec<u16> = s.encode_utf16().collect();
    String::from_utf16(&u[start as usize..(start + len) as usize]).unwrap()
}

fn complete_at(doc: &str, context: &Context) -> Option<Completions> {
    complete(&doc.replacen('|', "", 1), context, u16len(&doc[..doc.find('|').unwrap()]))
}

fn labels(doc: &str) -> Vec<String> {
    complete_at(doc, &Context::default()).map(|c| c.items.into_iter().map(|i| i.label).collect()).unwrap_or_default()
}

fn found(doc: &str) -> Vec<String> {
    document_tokens(doc, &Context::default()).iter().map(|t| format!("{}:{}", slice(doc, t.start, t.length), t.kind)).collect()
}

#[test]
fn reads_the_package_signatures() {
    let solve = params::functions().iter().find(|f| f.name == "solve").unwrap();
    let steps = solve.named.iter().find(|p| p.name == "steps").unwrap();
    assert_eq!(steps.values, vec!["full", "formula", "result", "value"]);
    assert!(steps.doc.starts_with("What is shown"));
    assert_eq!(solve.positional[0].name, "eqs");
    let graph = params::functions().iter().find(|f| f.name == "graph").unwrap();
    // Options documented only at solve-config.
    assert!(!graph.named.iter().find(|p| p.name == "digits").unwrap().doc.is_empty());
    let config = params::functions().iter().find(|f| f.name == "solve-config").unwrap();
    assert_eq!(config.named.iter().find(|p| p.name == "integration").unwrap().values, vec!["full", "code", "math", "none"]);
    let defined = params::functions().iter().find(|f| f.name == "defined").unwrap();
    assert_eq!(defined.named.iter().find(|p| p.name == "style").unwrap().values, vec!["grid", "column", "inline", "list"]);
}

#[test]
fn highlights_the_strings_of_calls() {
    let doc = "#def(\"R = 7.2 kOhm\") #solve(\"Q_0/2 = I t\", unit: \"h\", steps: \"value\") #graph(\"P = U^2/R\", x: \"U\", mark: \"extrema\")";
    let t = found(doc);
    for want in [
        format!("R:{}", TokenKind::VARIABLE),
        format!("7.2:{}", TokenKind::NUMBER),
        format!("kOhm:{}", TokenKind::UNIT),
        format!("h:{}", TokenKind::UNIT),
        format!("value:{}", TokenKind::KEYWORD),
        format!("U:{}", TokenKind::VARIABLE),
        format!("extrema:{}", TokenKind::KEYWORD),
    ] {
        assert!(t.contains(&want), "{want} in {t:?}");
    }
    // Strings of other functions stay Typst's.
    assert!(found("#text(\"R = 3 V\")").is_empty());
    // A `$` in a string doesn't start math.
    assert!(found("#solve(\"x = 3 $\") $y$").iter().all(|t| !t.starts_with('y')));
}

#[test]
fn completes_in_the_strings_of_calls() {
    let names = labels("#def(\"R = 7.2 kOhm\")\n#solve(\"U = |\")");
    assert!(names.contains(&"R".into()) && names.contains(&"sqrt".into()));
    assert!(labels("#def(\"I = 30 m|\")").contains(&"mA".into()));
    let c = complete_at("#solve(\"x\", unit: \"k|\")", &Context::default()).unwrap();
    assert!(c.exclusive && c.items.iter().any(|i| i.label == "kOhm"));
    assert!(labels("#graph(\"y = x^2\", xunit: \"|\")").contains(&"V".into()));
    assert!(labels("#def(\"U = 5 V\")\n#graph(\"P = U^2/R\", x: \"|\")").contains(&"U".into()));
    assert_eq!(labels("#solve(\"x\", steps: \"r|\")"), vec!["full", "formula", "result", "value"]);
    assert!(labels("#graph(\"y\", mark: \"|\")").contains(&"extrema".into()));
    assert!(labels("#lambdify(\"U^2/R\", \"U\", unit: \"m|\")").contains(&"mW".into()));
    // Unclosed strings are what's being typed.
    assert!(labels("#def(\"I = 30 m|").contains(&"mA".into()));
}

#[test]
fn completes_options_of_calls() {
    let c = complete_at("#solve(\"x\", st|)", &Context::default()).unwrap();
    assert!(!c.exclusive);
    let steps = c.items.iter().find(|i| i.label == "steps").unwrap();
    assert_eq!(steps.insert, "steps: ");
    assert_eq!(steps.kind, ItemKind::PARAMETER);
    assert!(steps.documentation.as_deref().unwrap().contains("What is shown"));
    // Given ones aren't offered again.
    assert!(!labels("#solve(\"x\", unit: \"h\", |)").contains(&"unit".into()));
    assert_eq!(labels("#solve(\"x\", steps: |)"), vec!["\"full\"", "\"formula\"", "\"result\"", "\"value\""]);
    assert!(labels("#show: solve-config.with(integration: |)").contains(&"\"full\"".into()));
    assert!(labels("#defined(style: \"|\")").contains(&"grid".into()));
    assert!(labels("#solve(\"x\", unit: |)").contains(&"\"mA\"".into()));
}

#[test]
fn knows_the_documents_units() {
    let doc = "#solve-config(units: (bar: \"100 kPa\", mbar: \"0.001 bar\"))\n```solveng\np = 3 mbar -> bar\n```";
    assert!(found(doc).contains(&format!("mbar:{}", TokenKind::UNIT)));
    let c = complete_at("#solve-config(units: (bar: \"100 kPa\", mbar: \"0.001 bar\"))\n#def(\"p = 3 |\")", &Context::default()).unwrap();
    assert_eq!(c.items.iter().find(|i| i.label == "mbar").unwrap().detail, "= 0.001 bar (bar = 100 kPa)");
    // The compiler's units count too, e.g. from a package.
    let context = Context::new("").with_report(
        &Report::parse(r#"{"blocks":1,"units":[{"name":"rpm","kind":"","value":"1/min"}]}"#),
        "main.typ",
        true,
    );
    assert!(complete_at("#def(\"n = 3 |\")", &context).unwrap().items.iter().any(|i| i.label == "rpm" && i.detail == "= 1/min"));
}

#[test]
fn shows_the_compilers_values() {
    let report = Report::parse(
        r#"{"blocks":1,"defined":[{"name":"I","kind":"given","value":"30 mA"},{"name":"t","kind":"computed","value":"115 h"},{"name":"f","kind":"function","value":"(x) = x^2"},{"name":"k","kind":"given","value":"3"}]}"#,
    );
    let context = Context::new("").with_report(&report, "main.typ", true);
    let c = complete_at("#def(\"I = 30 mA\", \"t = 3450 mAh / I\")\n#solve(\"x = |\")", &context).unwrap();
    let get = |l: &str| c.items.iter().find(|i| i.label == l).unwrap().detail.clone();
    assert_eq!(get("I"), "= 30 mA");
    assert_eq!(get("t"), "= 3450 mAh / I = 115 h");
    // Defined where the text doesn't say.
    assert_eq!(get("k"), "= 3");
    assert_eq!(get("f(x)"), "= x^2");
}

#[test]
fn finds_unknown_units() {
    let problems = diagnostics("#solve(\"x = 3\", unit: \"kWh\") #solve(\"y\", unit: \"furlong\")\n```solveng\nP = [5 Vv] I -> mW\n```", &Context::default());
    let words: Vec<_> = problems.iter().map(|p| p.message.split('\'').nth(1).unwrap().to_string()).collect();
    assert_eq!(words, vec!["Vv", "furlong"]);
    assert!(diagnostics("#solve-config(units: (bar: \"100 kPa\"))\n#solve(\"p\", unit: \"kbar\")", &Context::default()).is_empty());
    assert!(diagnostics("#show: solve-block\n$R = 7.2 \"k\"Omega$ $I = 3 \"mA\"$", &Context::default()).is_empty());
}

#[test]
fn hovers_names_units_and_options() {
    let doc = "#def(\"R = 7.2 kOhm\")\n#solve(\"U = R I\", steps: \"value\")";
    let at = |w: &str| u16len(&doc[..doc.rfind(w).unwrap()]) + 1;
    assert_eq!(hover(doc, &Context::default(), at("R I")).unwrap().markdown, "`R` = 7.2 kOhm");
    assert!(hover(doc, &Context::default(), at("steps")).unwrap().markdown.starts_with("`steps` (auto, str)"));
    assert_eq!(hover(doc, &Context::default(), at("kOhm") + 1).unwrap().markdown, "`kOhm` kiloohm");
    assert!(hover("text R here", &Context::default(), 6).is_none());
}

#[test]
fn lists_regions() {
    let doc = "#show: solve-block\n$x = 1$ `y` #solve(\"z\")";
    let kinds: Vec<_> = regions(doc, &Context::default()).iter().map(|r| r.kind).collect();
    assert_eq!(kinds, vec!["math", "call", "string"]);
}

#[test]
fn calculates_math_given_to_def() {
    // Math passed to def is calculated without a solve-block.
    let doc = "#def($I = 30 \"mA\"$)";
    assert!(!math_tokens(doc, &Context::default()).is_empty());
}
