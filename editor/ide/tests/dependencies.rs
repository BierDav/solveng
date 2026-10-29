// What a hover over a defined name lists under "Depends on": the names its formula uses, transitively, a line each.

use solveng_ide::*;

fn u16len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

/// The hover at the last occurrence of `at` in `doc`.
fn markdown(doc: &str, context: &Context, at: &str) -> String {
    hover(doc, context, u16len(&doc[..doc.rfind(at).unwrap()])).unwrap().markdown
}

fn dependencies(doc: &str, context: &Context, at: &str) -> Option<String> {
    markdown(doc, context, at).split_once("### Depends on\n\n").map(|(_, d)| d.to_string())
}

fn code(lines: &[&str]) -> String {
    format!("```text\n{}\n```", lines.join("\n"))
}

#[test]
fn lists_what_a_definition_depends_on() {
    let doc = "#import units: A\n#show: worksheet\n$a := 2 #A$\n$b := a + 3 #A$\n$f(x: #A) := b + x$\n$f(1 #A) =>$";
    assert_eq!(dependencies(doc, &Context::default(), "f(").unwrap(), code(&["b = a + 3 A", "a: A = 2 A"]));
    // undefined names are marked, built-in operations left out
    let doc = "#show: worksheet\n$ Sigma\"\"r := sin(x) + y^2 $\n$ Σ\"\"r -> $";
    assert_eq!(dependencies(doc, &Context::default(), "Σ").unwrap(), code(&["x // undefined", "y // undefined"]));
    // a given value depends on nothing and says nothing about it
    assert_eq!(dependencies("#import units: A\n#show: worksheet\n$a := 2 #A$\n$a =>$", &Context::default(), "a =>"), None);
}

#[test]
fn shows_functions_with_their_signature_once() {
    // a function's line is its signature and body, without a second =: o(x: Ω) = x, not o = (x: Ω) = x
    let doc = "#import units: Omega\n#show: worksheet\n$o(x: Omega) := x$\n$y := o(2 Omega)$\n$y =>$";
    assert_eq!(dependencies(doc, &Context::default(), "y =>").unwrap(), code(&["o(x: Omega) = x"]));
    // with the compiler's values, what it returns too, and the value of what's computed
    let report = Report {
        defined: vec![
            ReportValue { name: "o".into(), kind: "function".into(), value: "(x: Ω): Ω = x".into() },
            ReportValue { name: "y".into(), kind: "computed".into(), value: "2 Ω".into() },
        ],
        ..Report::default()
    };
    let context = Context::default().with_report(&report, "", true);
    assert_eq!(dependencies(doc, &context, "y =>").unwrap(), code(&["o(x: Ω): Ω = x"]));
    assert!(markdown(doc, &context, "o(2").starts_with("```text\no(x: Ω): Ω = x\n```"));
}

#[test]
fn follows_equations_passed_to_worksheets_and_the_visible_definition() {
    for doc in [
        "#show: worksheet\n$a := 2$\n$b := a + a$\n$f(x) := b + x$\n$f(1) =>$",
        "#worksheet($a := 2$, $b := a + a$, $f(x) := b + x$, $f(1) =>$)",
        "#show: worksheet\n$\n  a := 2 \\\n  b := a + a \\\n  f(x) := b + x \\\n  f(1) =>\n$",
    ] {
        assert_eq!(dependencies(doc, &Context::default(), "f(").unwrap(), code(&["b = a + a", "a = 2"]), "{doc}");
    }
    // the definition in force where the name is used
    let doc = "#show: worksheet\n$x := 1$\n$y := 2$\n$a := x$\n$a =>$\n$a := y$";
    let at = doc.find("a =>").unwrap();
    let h = hover(doc, &Context::default(), u16len(&doc[..at])).unwrap().markdown;
    assert!(h.ends_with(&code(&["x = 1"])), "{h}");
}

#[test]
fn parameters_and_indices_have_no_dependencies() {
    let doc = "#import units: V\n#show: worksheet\n$U := 5 V$\n$f(x: V) := x + U$\n$s := sum_(k=0)^3 k U$";
    let h = markdown(doc, &Context::default(), "x +");
    assert!(!h.contains("Depends on"), "{h}");
    let h = markdown(doc, &Context::default(), "k U");
    assert!(!h.contains("Depends on"), "{h}");
}

#[test]
fn follows_included_files() {
    let main = "#show: worksheet\n$a := 2$\n#include \"part.typ\"\n$b =>$";
    let part = "$b := a + 1$";
    let element = |path: &str, source: &str, text: &str| {
        let start = source.find(text).unwrap();
        ReportElement { path: path.into(), start, end: start + text.len(), text: text.into(), kind: "math".into(), active: true, evaluate: true }
    };
    let report = Report {
        blocks: 1,
        elements: vec![element("main.typ", main, "$a := 2$"), element("main.typ", main, "$b =>$"), element("part.typ", part, part)],
        ..Report::default()
    };
    let files = vec![ContextFile { path: "main.typ".into(), text: main.into() }, ContextFile { path: "part.typ".into(), text: part.into() }];
    let context = Context::new("").with_files(files).with_report(&report, "main.typ", true).with_main("main.typ");
    assert_eq!(dependencies(main, &context, "b =>").unwrap(), code(&["a = 2"]));
}
