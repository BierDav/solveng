use solveng_ide::{Context, diagnostics};

#[test]
fn explicit_scope_and_actual_content_blocks_do_not_warn() {
    for doc in [
        "#mathscope(scope: \"shared\")[$x := 1$]",
        "#mathscope(scope: \"isolated\")[$x := 1$]",
        "#mathscope(scope: selected)[$x := 1$]",
        "#mathscope[$x := 1$]",
        "#mathscope[$x := 1$ $y := 2$]",
        "#mathscope[Text: $x := 1$]",
        "#mathscope[ $x := 1$ ]",
        "#math($x := 1$)",
        "#mathscope[$x := 1$", // unfinished block
        "#other[$x := 1$]",
        // raw text is content, not an equation: the block stays isolated
        "#mathscope[```solveng\nx := 1\n```]",
    ] {
        assert!(diagnostics(doc, &Context::default()).iter().all(|p| !p.message.contains("scope: auto")), "{doc}");
    }
}
