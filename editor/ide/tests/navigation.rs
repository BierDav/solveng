// Per-element activity from the compiler's report, definitions, usages and unknown-unit warnings.

use solveng_ide::*;

fn u16_at(doc: &str, needle: &str) -> u32 {
    doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32
}

fn text_at(doc: &str, start: u32, end: u32) -> String {
    let u: Vec<u16> = doc.encode_utf16().collect();
    String::from_utf16(&u[start as usize..end as usize]).unwrap()
}

/// A report of `(element text, active, evaluate)` in `main.typ`, at their positions in `doc`.
fn report(doc: &str, elements: &[(&str, bool, bool)]) -> Report {
    Report {
        blocks: 1,
        elements: elements
            .iter()
            .map(|(text, active, evaluate)| ReportElement {
                path: "main.typ".into(),
                start: u16_at(doc, text) as usize,
                end: (u16_at(doc, text) + text.encode_utf16().count() as u32) as usize,
                text: (*text).into(),
                kind: if text.starts_with('$') { "math".into() } else { "code".into() },
                active: *active,
                evaluate: *evaluate,
            })
            .collect(),
        ..Report::default()
    }
}

/// Whether math tokens cover the text `needle`.
fn coloured(doc: &str, context: &Context, needle: &str) -> bool {
    let at = u16_at(doc, needle);
    document_tokens(doc, context).iter().any(|t| t.start <= at && at < t.start + t.length)
}

#[test]
fn decides_per_element_from_the_report() {
    let doc = "$a := 1$\n#show: worksheet\n$b := 2$\n#worksheet(evaluate: false)[\n$d := 4$\n]";
    let compiled = report(doc, &[("$b := 2$", true, true), ("$d := 4$", false, false)]);
    let context = Context::new("").with_report(&compiled, "main.typ", true);
    // Before the worksheet, not calculated; then as the compiler saw each.
    assert!(!coloured(doc, &context, "a := 1"));
    assert!(coloured(doc, &context, "b := 2"));
    assert!(!coloured(doc, &context, "d := 4"));

    // Written since the compile: like the matched element before it.
    let edited = "$a := 1$\n#show: worksheet\n$b := 2$ $x := 9$\n#worksheet(evaluate: false)[\n$d := 4$ $y := 8$\n]";
    assert!(coloured(edited, &context, "x := 9"));
    assert!(!coloured(edited, &context, "y := 8"));
}

#[test]
fn files_without_reported_elements_keep_the_settings() {
    let doc = "#show: worksheet\n$b := 2$\n";
    let compiled = report("$q := 1$", &[("$q := 1$", true, true)]);
    // The main file was compiled: nothing of it is in a worksheet.
    assert!(!coloured(doc, &Context::new("").with_report(&compiled, "main2.typ", true), "b := 2"));
    // Another file falls back to its settings, as does a solveng without reports.
    assert!(coloured(doc, &Context::new("").with_report(&compiled, "chapter.typ", false), "b := 2"));
    assert!(coloured(doc, &Context::new("").with_report(&Report::default(), "main.typ", true), "b := 2"));
}

#[test]
fn tagged_code_is_shown_but_defines_only_when_calculated() {
    let doc = "```solveng\nk := 1\n```\n#show: worksheet\n$b := 2$\n$b + k =$\n";
    let compiled = report(doc, &[("$b := 2$", true, true), ("$b + k =$", true, true)]);
    let context = Context::new("").with_report(&compiled, "main.typ", true);
    assert!(coloured(doc, &context, "k := 1"));
    let at = u16_at(doc, "$b + k =$") + 1;
    let items = complete(doc, &context, at).unwrap().items;
    assert!(items.iter().any(|i| i.label == "b"));
    assert!(!items.iter().any(|i| i.label == "k"));
}

/// The UTF-16 offset of the `n`th (from 0) `needle` in `doc`.
fn nth(doc: &str, needle: &str, n: usize) -> u32 {
    let byte = doc.match_indices(needle).nth(n).unwrap().0;
    doc[..byte].encode_utf16().count() as u32
}

#[test]
fn repeated_equations_do_not_take_a_later_worksheets_report() {
    let equation = "$y := x$";
    for prefix in ["", "😅\n#import \"@solveng/solveng:1.0.0\": *\n"] {
        let doc = format!("{prefix}{equation}\n#show: worksheet\n{equation}");
        let start = nth(&doc, equation, 1) as usize;
        let compiled = Report { blocks: 1, elements: vec![ReportElement {
            path: "main.typ".into(), start, end: start + equation.len(), text: equation.into(),
            kind: "math".into(), active: true, evaluate: true,
        }], ..Report::default() };
        let context = Context::default().with_report(&compiled, "main.typ", true);
        for document in [&doc, &format!("// inserted\n{doc}")] {
            let tokens = document_tokens(document, &context);
            for (occurrence, expected) in [(0, false), (1, true)] {
                let at = nth(document, equation, occurrence) + 1;
                assert_eq!(tokens.iter().any(|t| t.start <= at && at < t.start + t.length), expected, "{document}, occurrence {occurrence}");
            }
        }
    }
}

#[test]
fn identical_equations_keep_distinct_activity_across_nested_worksheets() {
    let equation = "$y := x$";
    let doc = format!("{equation}\n#show: worksheet\n{equation}\n#worksheet(evaluate: false)[{equation}]\n{equation}");
    let compiled = Report { blocks: 1, elements: [(1, true), (2, false), (3, true)].into_iter().map(|(i, active)| {
        let start = nth(&doc, equation, i) as usize;
        ReportElement { path: "main.typ".into(), start, end: start + equation.len(), text: equation.into(),
            kind: "math".into(), active, evaluate: active }
    }).collect(), ..Report::default() };
    let context = Context::default().with_report(&compiled, "main.typ", true);
    let tokens = document_tokens(&doc, &context);
    for (i, expected) in [false, true, false, true].into_iter().enumerate() {
        let at = nth(&doc, equation, i) + 1;
        assert_eq!(tokens.iter().any(|t| t.start <= at && at < t.start + t.length), expected, "occurrence {i}");
    }
}

/// The texts of `usages` in `doc` (path None), with their positions.
fn used(doc: &str, context: &Context, offset: u32) -> Vec<u32> {
    usages(doc, context, offset).into_iter().map(|l| {
        assert_eq!(l.path, None);
        l.start
    }).collect()
}

#[test]
fn keeps_worksheets_apart() {
    let doc = "#worksheet[$x := 1$ $y := x$]\n#worksheet[$x := 2$ $z := x$]\n";
    let c = Context::new("");
    assert_eq!(used(doc, &c, nth(doc, "x", 0)), [nth(doc, "x", 0), nth(doc, "x", 1)]);
    assert_eq!(definition(doc, &c, nth(doc, "x", 3)).unwrap().start, nth(doc, "x", 2));
    // Usages before the definition don't count.
    let doc = "#show: worksheet\n$a := x$\n$x := 1$\n";
    assert_eq!(used(doc, &c, nth(doc, "x", 1)), [nth(doc, "x", 1)]);
    assert_eq!(definition(doc, &c, nth(doc, "x", 0)), None);
}

#[test]
fn clearing_splits_scopes() {
    let c = Context::new("");
    let doc = "#show: worksheet.with(clear-at: heading)\n= One\n$x := 1$ $a := x$\n= Two\n$b := x$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 2)), None);
    assert_eq!(used(doc, &c, nth(doc, "x", 0)).len(), 2);
    // Only headings of the level.
    let doc = "#show: worksheet.with(clear-at: heading.where(level: 1))\n= One\n$x := 1$\n== Sub\n$a := x$\n= Two\n$b := x$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 1)).unwrap().start, nth(doc, "x", 0));
    assert_eq!(definition(doc, &c, nth(doc, "x", 2)), None);
    // `#clear(all: true)` starts over once, `#clear(…)` forgets the names given.
    let doc = "#show: worksheet\n$x := 1$\n#clear(all: true)\n$b := x$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 1)), None);
    let doc = "#show: worksheet\n$x := 1$ $y := 2$ $f(t) := t$\n#clear(\"x\", $f$)\n$b := x + y + f(1)$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 2)), None);
    assert_eq!(definition(doc, &c, nth(doc, "y", 1)).unwrap().start, nth(doc, "y", 0));
    assert_eq!(definition(doc, &c, nth(doc, "f", 2)), None);
    // the name in the call is a use of what it forgets
    assert_eq!(definition(doc, &c, nth(doc, "x", 1)).unwrap().start, nth(doc, "x", 0));
}

#[test]
fn redefinitions_split_usages() {
    let c = Context::new("");
    let doc = "#show: worksheet\n$x := 1$ $a := x$\n$x := 2$ $b := x$\n";
    assert_eq!(used(doc, &c, nth(doc, "x", 0)), [nth(doc, "x", 0), nth(doc, "x", 1)]);
    assert_eq!(definition(doc, &c, nth(doc, "x", 3)).unwrap().start, nth(doc, "x", 2));
    // = states something about a name: a use of its definition, as is showing it
    let doc = "#show: worksheet\n$x := 1$\n$x = 1$ $x =$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 1)).unwrap().start, nth(doc, "x", 0));
    assert_eq!(definition(doc, &c, nth(doc, "x", 2)).unwrap().start, nth(doc, "x", 0));
    // The right side still means the old value.
    let doc = "```solveng\nx := 1\nx := x + 1\ny := x\n```\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 2)).unwrap().start, nth(doc, "x", 0));
    assert_eq!(definition(doc, &c, nth(doc, "x", 3)).unwrap().start, nth(doc, "x", 1));
    // what's solved for is defined in the clause
    let doc = "#show: worksheet\n$x^2 = 4 -> solve(x)$ $y := x$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 2)).unwrap().start, nth(doc, "x", 1));
}

#[test]
fn inherits_only_when_asked() {
    let c = Context::new("");
    let doc = "#show: worksheet\n$x := 1$\n#worksheet(scope: \"inherit\")[ $a := x$ ]\n#worksheet[ $b := x$ ]\n#worksheet[ $x := 9$ ]\n$c := x$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 1)).unwrap().start, nth(doc, "x", 0));
    assert_eq!(definition(doc, &c, nth(doc, "x", 2)), None);
    // After a worksheet the outer definitions come back.
    assert_eq!(definition(doc, &c, nth(doc, "x", 4)).unwrap().start, nth(doc, "x", 0));
    // Equations share the scope by default; isolated ones neither see nor keep.
    let doc = "#show: worksheet\n$x := 1$\n#worksheet($a := x$)\n#worksheet($x := 9$, $b := x$, scope: \"isolated\")\n$c := x$\n";
    assert_eq!(definition(doc, &c, nth(doc, "x", 1)).unwrap().start, nth(doc, "x", 0));
    assert_eq!(definition(doc, &c, nth(doc, "x", 3)).unwrap().start, nth(doc, "x", 2));
    assert_eq!(definition(doc, &c, nth(doc, "x", 4)).unwrap().start, nth(doc, "x", 0));
}

/// A chapter at `chapters/one.typ`, compiled from `main` at `main.typ`.
fn chapter_context(main: &str) -> Context {
    Context::new("")
        .with_report(&Report::default(), "chapters/one.typ", false)
        .with_files(vec![ContextFile { path: "main.typ".into(), text: main.into() }])
        .with_main("main.typ")
}

#[test]
fn follows_includes_in_compile_order() {
    let chapter = "$I := U / 2 Ω$\n";
    let main = "#show: worksheet\n$U := 5 V$\n#include \"chapters/one.typ\"\n$W := U$\n";
    let c = chapter_context(main);
    let def = definition(chapter, &c, nth(chapter, "U", 0)).unwrap();
    assert_eq!(def.path.as_deref(), Some("main.typ"));
    assert_eq!(def.start, nth(main, "U", 0));
    let found = usages(chapter, &c, nth(chapter, "U", 0));
    assert_eq!(found.iter().map(|l| (l.path.clone(), l.start)).collect::<Vec<_>>(), [
        (Some("main.typ".to_string()), nth(main, "U", 0)),
        (None, nth(chapter, "U", 0)),
        (Some("main.typ".to_string()), nth(main, "U", 1)),
    ]);
    // Included before the worksheet, or after it, the chapter doesn't see main's definitions.
    let before = "#include \"chapters/one.typ\"\n#show: worksheet\n$U := 5 V$\n";
    assert_eq!(definition(chapter, &chapter_context(before), nth(chapter, "U", 0)), None);
    let outside = "#worksheet[\n$U := 5 V$\n]\n#include \"chapters/one.typ\"\n";
    assert_eq!(definition(chapter, &chapter_context(outside), nth(chapter, "U", 0)), None);
    // Usages from main don't reach a chapter that isn't in the worksheet either.
    assert!(usages(chapter, &chapter_context(outside), nth(chapter, "I", 0)).is_empty());
    // Without a main file the document stands alone.
    let alone = Context::new("").with_files(vec![ContextFile { path: "main.typ".into(), text: main.into() }]);
    assert_eq!(definition(chapter, &alone, nth(chapter, "U", 0)), None);
}

#[test]
fn ignores_units_and_prose() {
    let doc = "#show: worksheet\n$U := 5 unit(\"V\")$, $P := U dot 2 unit(\"V\") = mV$ and U in prose\n";
    let found = used(doc, &Context::new(""), nth(doc, "U", 0));
    assert_eq!(found, [nth(doc, "U", 0), nth(doc, "U", 1)]);
}

#[test]
fn finds_math_names_with_subscripts() {
    let doc = "#show: worksheet\n$Q_0 := 3450 mAh$, $t_\"half\" := Q_0 / 2 A$\n```solveng\nx := t_half\n```\n";
    let found = usages(doc, &Context::new(""), u16_at(doc, "t_half"));
    assert_eq!(found.len(), 2);
    assert_eq!(text_at(doc, found[0].start, found[0].end), "t_\"half\"");
    let q = definition(doc, &Context::new(""), u16_at(doc, "Q_0 / 2")).unwrap();
    assert_eq!(text_at(doc, q.start, q.end), "Q_0");
}

#[test]
fn warns_of_unknown_units_where_calculated() {
    let doc = "$a := 3 unit(\"zorp\")$\n#show: worksheet\n$b := 3 unit(\"zorp\")$\n#lambdify($b$, \"b\", unit: \"blop\")\n```solveng\nc := b = [blip]\n```\n#worksheet(units: (zorp: $2 m$), evaluate: false)[]\n";
    let compiled = report(doc, &[("$b := 3 unit(\"zorp\")$", true, true)]);
    let context = Context::new("").with_report(&compiled, "main.typ", true);
    let found = diagnostics(doc, &context);
    let words: Vec<String> = found.iter().map(|p| text_at(doc, p.start, p.end)).collect();
    assert_eq!(words, ["blip", "blop"]);
    assert!(found.iter().all(|p| p.severity == "warning"));
    // Without the custom unit, only the calculated equation warns.
    let bare = doc.replace("#worksheet(units: (zorp: $2 m$), evaluate: false)[]\n", "");
    let compiled = report(&bare, &[("$b := 3 unit(\"zorp\")$", true, true)]);
    let found = diagnostics(&bare, &Context::new("").with_report(&compiled, "main.typ", true));
    assert_eq!(found.iter().filter(|p| text_at(&bare, p.start, p.end) == "zorp").count(), 1);
}

#[test]
fn backslashes_separate_definitions_and_result_markers() {
    for doc in [
        "#worksheet($ t := 1 \\ g := t + 2 \\ g = $)",
        "#worksheet(`t := 1 \\ g := t + 2 \\ g =`)",
        "#show: worksheet\n```solveng\nt := 1 \\ g := t + 2 \\ g =\n```",
    ] {
        let context = Context::default();
        let at = doc.rfind("g =").unwrap();
        let target = definition(doc, &context, doc[..at].encode_utf16().count() as u32).expect(doc);
        assert_eq!(text_at(doc, target.start, target.end), "g", "{doc}");
        assert_eq!(target.start, u16_at(doc, "g :="), "{doc}");
        let results = document_tokens(doc, &context).into_iter().filter(|t| t.kind == TokenKind::EVALUATE).count();
        assert_eq!(results, 1, "{doc}");
    }
}

#[test]
fn worksheet_scope_decides_what_is_seen_and_kept() {
    let ctx = Context::default();
    // (scope, sees the outer x, keeps its own)
    for (scope, inherit, export) in [("isolated", false, false), ("inherit", true, false), ("shared", true, true)] {
        let doc = format!("#show: worksheet\n$x := 1$\n#worksheet(scope: \"{scope}\")[$before := x$ $x := 2$ $inside := x$]\n$after := x$");
        let before = definition(&doc, &ctx, nth(&doc, "x$", 0));
        assert_eq!(before.map(|d| d.start), inherit.then(|| nth(&doc, "$x", 0) + 1), "{scope}");
        assert_eq!(definition(&doc, &ctx, nth(&doc, "x$", 1)).unwrap().start, nth(&doc, "$x", 1) + 1);
        assert_eq!(definition(&doc, &ctx, nth(&doc, "x$", 2)).unwrap().start, nth(&doc, "$x", if export { 1 } else { 0 }) + 1, "{scope}");
    }
}

#[test]
fn worksheet_expressions_share_by_default() {
    let ctx = Context::default();
    for (options, export) in [("", true), (", scope: auto", true), (", scope: \"shared\"", true), (", scope: \"inherit\"", false), (", scope: \"isolated\"", false)] {
        let doc = format!("#show: worksheet\n$x := 1$\n#worksheet($x := 2${options})\n$after := x$");
        assert_eq!(definition(&doc, &ctx, u16_at(&doc, "x$")).unwrap().start, nth(&doc, "$x", if export { 1 } else { 0 }) + 1, "{options}");
    }
}

#[test]
fn shared_worksheets_keep_definitions_in_only_the_immediate_parent() {
    let ctx = Context::default();
    let doc = "#show: worksheet\n$x := 1$\n#worksheet[ #worksheet(scope: \"shared\")[$x := 2$] $inside := x$ ]\n$outside := x$\n#worksheet(scope: \"shared\")[#clear(\"x\") $y := 3$]\n$final := x+y$";
    assert_eq!(definition(doc, &ctx, nth(doc, "x$", 0)).unwrap().start, nth(doc, "$x", 1) + 1);
    assert_eq!(definition(doc, &ctx, nth(doc, "x$", 1)).unwrap().start, nth(doc, "$x", 0) + 1);
    assert_eq!(definition(doc, &ctx, u16_at(doc, "x+y")).unwrap().start, nth(doc, "$x", 0) + 1);
    assert_eq!(definition(doc, &ctx, u16_at(doc, "y$")).unwrap().start, u16_at(doc, "$y") + 1);
}

#[test]
fn clear_at_labels_restart_at_the_labelled_element() {
    let ctx = Context::default();
    let doc = "#show: worksheet.with(clear-at: <exercise>)\n= A <exercise>\n$y := 1$\n$a := y$\n= Notes\n$b := y$\n= B <exercise>\n$c := y$\n$y := 2$ <exercise>\n$d := y$";
    assert_eq!(definition(doc, &ctx, u16_at(doc, "y$\n= Notes")).unwrap().start, nth(doc, "$y", 0) + 1);
    // other headings don't clear
    assert_eq!(definition(doc, &ctx, u16_at(doc, "y$\n= B")).unwrap().start, nth(doc, "$y", 0) + 1);
    assert_eq!(definition(doc, &ctx, u16_at(doc, "y$\n$y := 2")), None);
    // a labelled equation clears before itself and then defines
    let last = doc.rfind("y$").unwrap();
    assert_eq!(definition(doc, &ctx, doc[..last].encode_utf16().count() as u32).unwrap().start, nth(doc, "$y", 1) + 1);
    // anything else isn't a place to clear at
    let other = doc.replace("clear-at: <exercise>", "clear-at: \"exercise\"");
    assert_eq!(definition(&other, &ctx, u16_at(&other, "y$\n$y := 2")).unwrap().start, nth(&other, "$y", 0) + 1);
}

#[test]
fn single_equation_blocks_follow_expression_defaults_in_navigation() {
    let ctx = Context::default();
    let doc = "#show: worksheet\n$x := 1$\n#worksheet[$x := x+1$]\n$after := x$";
    assert_eq!(definition(doc, &ctx, u16_at(doc, "x+1")).unwrap().start, nth(doc, "$x", 0) + 1);
    assert_eq!(definition(doc, &ctx, u16_at(doc, "x$")).unwrap().start, nth(doc, "$x", 1) + 1);
    let doc = "#show: worksheet\n$x := 1$\n#worksheet(scope: \"isolated\")[$x := 2$]\n$after := x$";
    assert_eq!(definition(doc, &ctx, u16_at(doc, "x$")).unwrap().start, nth(doc, "$x", 0) + 1);
}
