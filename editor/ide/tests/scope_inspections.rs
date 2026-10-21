use solveng_ide::{Context, diagnostics};

#[test]
fn warns_when_an_auto_export_block_collapses_to_an_expression() {
    for call in [
        "#worksheet[$x := 1$]",
        "#worksheet(export: auto)[$x := 1$]",
        "#solveng.worksheet(inherit: false)[/*comment*/$x := 1$]",
        "#worksheet[```solveng\nx := 1\n```]",
    ] {
        let doc = format!("😀 text\n{call}");
        let problems = diagnostics(&doc, &Context::default());
        let problem = problems.iter().find(|p| p.message.contains("export: auto")).unwrap_or_else(|| panic!("{doc}: {problems:?}"));
        assert_eq!(problem.severity, "warning");
        assert!(problem.message.contains("export: false"));
        let utf16: Vec<_> = doc.encode_utf16().collect();
        let source = String::from_utf16(&utf16[problem.start as usize..problem.end as usize]).unwrap();
        assert!(source.starts_with('[') && source.ends_with(']'), "{source}");
    }
}

#[test]
fn explicit_export_and_actual_content_blocks_do_not_warn() {
    for doc in [
        "#worksheet(export: true)[$x := 1$]",
        "#worksheet(export: false)[$x := 1$]",
        "#worksheet(export: selected)[$x := 1$]",
        "#worksheet[$x := 1$ $y := 2$]",
        "#worksheet[Text: $x := 1$]",
        "#worksheet[ $x := 1$ ]",
        "#worksheet($x := 1$)",
        "#worksheet[$x := 1$", // unfinished block
        "#other[$x := 1$]",
    ] {
        assert!(diagnostics(doc, &Context::default()).iter().all(|p| !p.message.contains("export: auto")), "{doc}");
    }
}
