// Which equations are calculated when the source repeats them or changed since the compile: the report decides
// what it saw, worksheets written in the source decide where they end.

use solveng_ide::*;

/// UTF-16 offset of the `nth` (0-based) occurrence of `needle`.
fn nth(doc: &str, needle: &str, nth: usize) -> u32 {
    let (at, _) = doc.match_indices(needle).nth(nth).unwrap_or_else(|| panic!("{needle} #{nth} in {doc}"));
    doc[..at].encode_utf16().count() as u32
}

/// A report of `(element text, occurrence in compiled, active, evaluate)` in `main.typ`, at their positions in
/// `compiled`, the source as it was compiled.
fn report(compiled: &str, elements: &[(&str, usize, bool, bool)]) -> Context {
    let elements = elements
        .iter()
        .map(|&(text, occurrence, active, evaluate)| {
            let start = nth(compiled, text, occurrence) as usize;
            ReportElement {
                path: "main.typ".into(),
                start,
                end: start + text.encode_utf16().count(),
                text: text.into(),
                kind: "math".into(),
                active,
                evaluate,
            }
        })
        .collect();
    let report = Report { blocks: 1, elements, ..Report::default() };
    Context::new("").with_report(&report, "main.typ", true)
}

/// Whether solveng colours the `occurrence`th `needle`.
fn coloured(doc: &str, context: &Context, needle: &str, occurrence: usize) -> bool {
    let at = nth(doc, needle, occurrence) + 1;
    document_tokens(doc, context).iter().any(|t| t.start <= at && at < t.start + t.length)
}

/// Which copies of `$y := x$` solveng colours.
fn copies(doc: &str, context: &Context) -> Vec<bool> {
    (0..doc.matches("$y := x$").count()).map(|i| coloured(doc, context, "$y := x$", i)).collect()
}

const Y: &str = "$y := x$";

#[test]
fn a_copy_after_a_worksheet_block_is_not_calculated() {
    let doc = "#worksheet[\n$y := x$\n]\n\n$y := x$";
    assert_eq!(copies(doc, &Context::default()), [true, false]);
    assert_eq!(copies(doc, &report(doc, &[(Y, 0, true, true)])), [true, false]);
}

#[test]
fn a_copy_before_a_show_rule_is_not_calculated() {
    let doc = "#import \"@solveng/solveng:1.0.0\": *\n\n$y := x$\n\n#show: worksheet\n$y := x$";
    assert_eq!(copies(doc, &Context::default()), [false, true]);
    assert_eq!(copies(doc, &report(doc, &[(Y, 1, true, true)])), [false, true]);
}

#[test]
fn copies_around_worksheet_blocks_follow_the_blocks() {
    let doc = "$y := x$\n#worksheet[$y := x$ $y := x$]\n$y := x$\n#worksheet[$y := x$]\n$y := x$";
    let context = report(doc, &[(Y, 1, true, true), (Y, 2, true, true), (Y, 4, true, true)]);
    assert_eq!(copies(doc, &Context::default()), [false, true, true, false, true, false]);
    assert_eq!(copies(doc, &context), [false, true, true, false, true, false]);
}

#[test]
fn equations_written_since_the_compile_follow_the_worksheet_they_are_in() {
    let compiled = "#worksheet[\n$a := 1$\n]\n";
    let context = report(compiled, &[("$a := 1$", 0, true, true)]);
    let edited = "$z := 0$\n#worksheet[\n$a := 1$ $b := 2$\n]\n$c := 3$\n";
    assert!(!coloured(edited, &context, "$z := 0$", 0));
    assert!(coloured(edited, &context, "$a := 1$", 0));
    assert!(coloured(edited, &context, "$b := 2$", 0));
    assert!(!coloured(edited, &context, "$c := 3$", 0));
}

#[test]
fn a_copy_inserted_before_the_worksheet_does_not_take_its_report() {
    // The report's position is stale: the new copy at the top is now nearer to it than the calculated one.
    let compiled = "#worksheet[$y := x$]\n$y := x$";
    let context = report(compiled, &[(Y, 0, true, true)]);
    let edited = "$y := x$\n#worksheet[$y := x$]\n$y := x$";
    assert_eq!(copies(edited, &context), [false, true, false]);
}

#[test]
fn a_show_rule_ends_with_the_block_it_is_in() {
    let doc = "#block[\n#show: worksheet\n$y := x$\n]\n$y := x$";
    assert_eq!(copies(doc, &Context::default()), [true, false]);
    assert_eq!(copies(doc, &report(doc, &[(Y, 0, true, true)])), [true, false]);
}

#[test]
fn evaluate_false_turns_math_off_inside_only() {
    let doc = "#show: worksheet\n$y := x$\n#worksheet(evaluate: false)[$y := x$]\n$y := x$";
    let context = report(doc, &[(Y, 0, true, true), (Y, 1, false, false), (Y, 2, true, true)]);
    assert_eq!(copies(doc, &Context::default()), [true, false, true]);
    assert_eq!(copies(doc, &context), [true, false, true]);
    // Written since: inside the block not, after it again.
    let edited = "#show: worksheet\n$y := x$\n#worksheet(evaluate: false)[$y := x$ $p := 1$]\n$y := x$ $q := 2$";
    assert!(!coloured(edited, &context, "$p := 1$", 0));
    assert!(coloured(edited, &context, "$q := 2$", 0));
}

#[test]
fn templates_solveng_cannot_read_are_trusted_where_the_report_says() {
    // `template` applies a worksheet the editor can't see: only the report tells.
    let compiled = "#import \"t.typ\": template\n#show: template\n$a := 1$\n";
    let context = report(compiled, &[("$a := 1$", 0, true, true)]);
    assert!(coloured(compiled, &context, "$a := 1$", 0));
    // Written after a calculated one in the same block: calculated like it.
    let edited = "#import \"t.typ\": template\n#show: template\n$a := 1$\n$b := 2$\n";
    assert!(coloured(edited, &context, "$b := 2$", 0));
    // But not past the end of the block the calculated one is in.
    let compiled = "#template[$a := 1$]\n";
    let context = report(compiled, &[("$a := 1$", 0, true, true)]);
    let edited = "#template[$a := 1$ $b := 2$]\n$c := 3$\n";
    assert!(coloured(edited, &context, "$b := 2$", 0));
    assert!(!coloured(edited, &context, "$c := 3$", 0));
}

#[test]
fn an_uncalculated_copy_in_a_worksheet_stays_uncalculated() {
    // Not shown, like a branch of `#if false`: the report says it wasn't calculated.
    let doc = "#show: worksheet\n#if false [$y := x$]\n$y := x$";
    let context = report(doc, &[(Y, 0, false, true), (Y, 1, true, true)]);
    assert_eq!(copies(doc, &context), [false, true]);
}

#[test]
fn included_files_are_calculated_where_a_worksheet_includes_them() {
    let main = "#show: worksheet\n#include \"part.typ\"\n";
    let part = "$y := x$\n";
    let files = vec![ContextFile { path: "main.typ".into(), text: main.into() }, ContextFile { path: "part.typ".into(), text: part.into() }];
    let start = 0;
    let report = Report {
        blocks: 1,
        elements: vec![ReportElement { path: "part.typ".into(), start, end: start + Y.len(), text: Y.into(), kind: "math".into(), active: true, evaluate: true }],
        ..Report::default()
    };
    let context = Context::new("").with_files(files.clone()).with_report(&report, "part.typ", false);
    let edited = "$y := x$\n$w := 1$\n";
    assert!(coloured(edited, &context, Y, 0));
    assert!(coloured(edited, &context, "$w := 1$", 0));
    // Before any compile, the including worksheet decides.
    let context = Context::new("").with_files(files).with_report(&Report::default(), "part.typ", false);
    assert!(coloured("$w := 1$\n", &context, "$w := 1$", 0));
}
