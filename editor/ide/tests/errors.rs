// The errors the last compile reported, shown where they are in the document as it is now.

use solveng_ide::*;

fn u16at(doc: &str, text: &str) -> u32 {
    doc[..doc.find(text).unwrap()].encode_utf16().count() as u32
}

/// The report's error for the element with source `element` (empty: passed to `worksheet`) in `doc` as compiled.
fn error(doc: &str, element: &str, source: &str, message: &str) -> ReportError {
    let start = if element.is_empty() { 0 } else { u16at(doc, element) as usize };
    let path = if element.is_empty() { "" } else { "main.typ" };
    ReportError { path: path.into(), start, end: start + element.encode_utf16().count(), text: element.into(), source: source.into(), message: message.into() }
}

fn context(errors: Vec<ReportError>) -> Context {
    Context::default().with_report(&Report { errors, ..Report::default() }, "main.typ", true)
}

/// The document's errors: their text and message.
fn errors(doc: &str, context: &Context) -> Vec<(String, String)> {
    let u: Vec<u16> = doc.encode_utf16().collect();
    diagnostics(doc, context)
        .into_iter()
        .filter(|p| p.severity == "error")
        .map(|p| (String::from_utf16(&u[p.start as usize..p.end as usize]).unwrap(), p.message))
        .collect()
}

const DOC: &str = "#import units: V, s, Ω\n#show: worksheet\n$U := 5 #V$\n$t := U + 2 #s =>$\n$R := 2 #Ω$ $x := R + U =>$\n#worksheet($z := q + 1 =>$)\n";

fn reported() -> Context {
    context(vec![
        error(DOC, "$t := U + 2 #s =>$", "", "adding different units: s and V"),
        error(DOC, "$x := R + U =>$", "", "adding different units: Ω and V"),
        error(DOC, "", "z := q + 1 =>", "'q' in 'z' isn't defined"),
    ])
}

#[test]
fn shows_reported_errors_at_their_statements() {
    assert_eq!(
        errors(DOC, &reported()),
        vec![
            ("t := U + 2 #s =>".to_string(), "adding different units: s and V".to_string()),
            ("x := R + U =>".to_string(), "adding different units: Ω and V".to_string()),
            ("z := q + 1 =>".to_string(), "'q' in 'z' isn't defined".to_string()),
        ]
    );
    // inspections stay warnings
    assert!(diagnostics(DOC, &reported()).iter().all(|p| p.severity == "error"));
}

#[test]
fn follows_edits_and_drops_errors_of_edited_elements() {
    // text written before moves the errors along
    let moved = DOC.replace("$U := 5 #V$", "Some words first. $U := 5 #V$");
    assert_eq!(errors(&moved, &reported()).len(), 3);
    // an edited equation has no error until the next compile tells
    let edited = DOC.replace("$t := U + 2 #s =>$", "$t := U + 2 #V =>$");
    let found = errors(&edited, &reported());
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|(text, _)| !text.starts_with("t :=")));
    // another file's errors aren't this one's
    let mut other = reported();
    for e in &mut other.errors {
        if !e.path.is_empty() {
            e.path = "other.typ".into();
        }
    }
    assert_eq!(errors(DOC, &other).len(), 1);
}

#[test]
fn places_errors_of_twin_elements_by_position() {
    let doc = "#import units: V, s\n#show: worksheet\n$a := 1 #V + 1 #s =>$\ntext\n$a := 1 #V + 1 #s =>$\n";
    let second = doc.rfind("$a").unwrap();
    let start = doc[..second].encode_utf16().count();
    let err = ReportError { path: "main.typ".into(), start, end: start + 20, text: "$a := 1 #V + 1 #s =>$".into(), source: String::new(), message: "units".into() };
    let found: Vec<Problem> = diagnostics(doc, &context(vec![err])).into_iter().filter(|p| p.severity == "error").collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].start as usize, start + 1);
}

#[test]
fn reads_errors_from_the_report_json() {
    let report = Report::parse(r#"{"blocks": 1, "errors": [{"path": "main.typ", "start": 17, "end": 27, "text": "$U := 5 X$", "message": "unknown unit X"}]}"#);
    assert_eq!(report.errors.len(), 1);
    let doc = "#show: worksheet\n$U := 5 X$";
    let context = Context::default().with_report(&report, "main.typ", true);
    assert_eq!(errors(doc, &context), vec![("U := 5 X".to_string(), "unknown unit X".to_string())]);
}

#[test]
fn places_errors_in_math_arguments_without_coloring_unrelated_math() {
    for function in ["value", "worksheet"] {
        let doc = format!("$q + 1$\n#{function}($q + 1$)");
        let c = context(vec![error(&doc, "", "q + 1", "undefined q")]);
        let problems: Vec<_> = diagnostics(&doc, &c).into_iter().filter(|p| p.severity == "error").collect();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].start, doc.rfind("q + 1").unwrap() as u32);
        assert_eq!(errors(&doc, &c), vec![("q + 1".into(), "undefined q".into())]);
        assert!(errors(&doc.replace(&format!("#{function}($q + 1$)"), &format!("#{function}($q + 2$)")), &c).is_empty());
    }
}

#[test]
fn name_validation_errors_target_only_the_invalid_name() {
    for (name, invalid) in [(r#""e-"10"#, '-'), (r#""a+b""#, '+'), (r#""😅/"2"#, '/')] {
        let message = format!("'{invalid}' isn't allowed in a variable name; use spaces or operators between expressions");
        let expression = format!("y(x) := 1{name}");
        let element = format!("${expression}$");
        let doc = format!("#show: worksheet\n😅\n{element}");
        let c = context(vec![error(&doc, &element, "", &message)]);
        assert_eq!(errors(&doc, &c), vec![(name.into(), message.clone())], "{doc}");
        // passed to a worksheet: found by its statement
        let doc = format!("😅 #worksheet(${expression}$)");
        let c = context(vec![error(&doc, "", &expression, &message)]);
        assert_eq!(errors(&doc, &c), vec![(name.to_string(), message)]);
    }
}
