// Explicit operation references in math, `op("tri")`: completed, coloured and shown as their name.

use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 { doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32 }

const PREFIX: &str = "#show: math\n";

fn apply(marked: &str, label: &str) -> String {
    let caret = marked.find('|').unwrap();
    let offset = marked[..caret].encode_utf16().count() as u32;
    let doc = marked.replacen('|', "", 1);
    let result = complete(&doc, &Context::default(), offset).expect("reference completion");
    let item = result.items.iter().find(|i| i.label == label).expect("reference item");
    let mut chars: Vec<_> = doc.encode_utf16().collect();
    chars.splice(result.from as usize..result.to.unwrap_or(offset) as usize, item.insert.encode_utf16());
    String::from_utf16(&chars).unwrap()
}

#[test]
fn inline_reference_completion_preserves_quotes_and_arguments() {
    for (source, label, expected) in [
        ("$y := op(\"tr|\")(x)$", "tri", "$y := op(\"tri\")(x)$"),
        ("$y := op( \"tr|\" )(x)$", "tri", "$y := op( \"tri\" )(x)$"),
        ("$y := op(\"si|\")(x)$", "sin", "$y := op(\"sin\")(x)$"),
        ("$y := op(\"tr|", "tri", "$y := op(\"tri"),
    ] {
        assert_eq!(apply(&format!("{PREFIX}{source}"), label), format!("{PREFIX}{expected}"));
    }
}

#[test]
fn inline_reference_tokens_color_only_string_contents_without_overlap() {
    let doc = "#show: math\n😅 $y := op(\"tri\")(x)$\n$z := 2 op(\"rect\")(x) + op( \"sgn\" )(x)$";
    for tokens in [document_tokens(doc, &Context::default()), math_tokens(doc, &Context::default())] {
        for name in ["tri", "rect", "sgn"] {
            let start = at(doc, &format!("\"{name}\"")) + 1;
            let end = start + name.encode_utf16().count() as u32;
            let hits: Vec<_> = tokens.iter().filter(|t| t.start < end + 1 && start - 1 < t.start + t.length).collect();
            assert_eq!(hits, vec![&Token { start, length: end - start, kind: TokenKind::BUILTIN_FUNCTION }], "{name}: {tokens:?}");
        }
        for pair in tokens.windows(2) {
            assert!(pair[0].start + pair[0].length <= pair[1].start, "overlap: {pair:?}");
        }
    }
}

#[test]
fn inline_operations_share_the_underlying_operation_hover() {
    let doc = "#show: math\n$y := op(\"tri\")(x)$";
    assert!(hover(doc, &Context::default(), at(doc, "tri")).unwrap().markdown.contains("Triangular pulse"));
}

#[test]
fn reference_literals_do_not_resolve_math_variables() {
    let doc = "#show: math\n$\"tri\" := 7$\n$y := op(\"tri\")(x)$";
    let offset = at(doc, "op(\"tri") + 4;
    assert!(definition(doc, &Context::default(), offset).is_none());
    assert!(hover(doc, &Context::default(), offset).unwrap().markdown.contains("Triangular pulse"));
    // the quoted name is the variable
    let quoted = at(doc, "\"tri\" :=") + 1;
    assert_eq!(definition(doc, &Context::default(), quoted).map(|l| l.start), Some(quoted - 1));
}

fn collapsed(doc: &str) -> String {
    collapsed_in(doc, &Context::default())
}

fn collapsed_in(doc: &str, context: &Context) -> String {
    let mut text: Vec<_> = doc.encode_utf16().collect();
    let found = conceals(doc, context);
    for pair in found.windows(2) {
        assert!(pair[0].start + pair[0].length <= pair[1].start, "overlapping conceals: {pair:?}");
    }
    for conceal in found.iter().rev() {
        let replacement = if conceal.character == 0 { String::new() } else { char::from_u32(conceal.character).unwrap().to_string() };
        text.splice(conceal.start as usize..(conceal.start + conceal.length) as usize, replacement.encode_utf16());
    }
    String::from_utf16(&text).unwrap()
}

#[test]
fn inline_references_collapse_to_their_name_and_share_an_editing_range() {
    for (source, expected) in [
        ("$y := op(\"tri\")(x)$", "$y := tri(x)$"),
        ("$y := op( \"sin\" )(x)$", "$y := sin(x)$"),
        ("$y := op(\"tri\")(op(\"rect\")(x))$", "$y := tri(rect(x))$"),
    ] {
        let doc = format!("{PREFIX}😅 {source}");
        assert_eq!(collapsed(&doc), format!("{PREFIX}😅 {expected}"));
        for conceal in conceals(&doc, &Context::default()) {
            assert!(conceal.element_start <= conceal.start && conceal.start + conceal.length <= conceal.element_end);
        }
    }
}

#[test]
fn declarations_and_incomplete_references_stay_expanded() {
    for doc in [
        "#show: math\n#let tri = math.op(\"tri\")\n#let pulse = operation(\"rect\")",
        "#show: math\n$y := op(\"tr",
        "#show: math\n$y := op(\"tri\"",
    ] {
        assert_eq!(collapsed(doc), doc);
    }
}

#[test]
fn reference_display_does_not_depend_on_evaluation() {
    for document in [
        "$y := op(\"tri\")(x) + 1 $",
        "#math(evaluate: false)[$y := op(\"tri\")(x) + 1 $]",
    ] {
        assert_eq!(collapsed(document), document.replace("op(\"tri\")", "tri"));
    }
}

#[test]
fn reference_display_stays_stable_with_stale_or_missing_reports() {
    let equation = "$y := op(\"tri\")(x) + op(\"rect\")(x) $";
    let report = Report { blocks: 1, elements: vec![ReportElement {
        path: "main.typ".into(), text: equation.into(), kind: "math".into(), active: true, evaluate: true,
        start: PREFIX.len(), end: PREFIX.len() + equation.len(),
    }], ..Report::default() };
    let stale = Context::default().with_report(&report, "main.typ", true);
    let empty = Context::default().with_report(&Report { blocks: 1, ..Report::default() }, "main.typ", true);
    for equation in [equation.to_string(), equation.replace("(x)", "(x + 1)"), equation.replace("$y", "$longer"),
                     equation.replace("(x)", "(x +)"), equation.replace(" $", " + $"), equation.replace(" + op", " + (op")] {
        let document = format!("{PREFIX}😅 {equation}");
        let expected = document.replace("op(\"tri\")", "tri").replace("op(\"rect\")", "rect");
        for context in [&Context::default(), &stale, &empty] {
            assert_eq!(collapsed_in(&document, context), expected, "{document}");
        }
    }
}
