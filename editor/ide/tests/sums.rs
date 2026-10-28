use solveng_ide::*;

fn at(doc: &str, needle: &str) -> u32 {
    doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32
}

fn apply(doc: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|e| std::cmp::Reverse(e.start));
    let mut text: Vec<u16> = doc.encode_utf16().collect();
    for e in edits {
        assert!(e.path.is_none());
        text.splice(e.start as usize..e.end as usize, e.text.encode_utf16());
    }
    String::from_utf16(&text).unwrap()
}

fn def(doc: &str, needle: &str, declaration: &str) {
    let location = definition(doc, &Context::default(), at(doc, needle)).unwrap();
    assert_eq!(location.start, at(doc, declaration), "{needle}");
}

#[test]
fn notation_scopes_the_index_to_the_summand() {
    for expression in [
        "$answer := sum_(k=k)^(k+10) (k^2) + k$\n$after := k$".to_string(),
        "#worksheet($answer := sum_(k=k)^(k+10) (k^2) + k$)\n$after := k$".to_string(),
    ] {
        let doc = format!("#show: worksheet\n$k := 100$\n{expression}");
        def(&doc, "k^2", "k=k");
        def(&doc, "k+10", "k := 100");
        assert_eq!(definition(&doc, &Context::default(), at(&doc, "+ k") + 2).unwrap().start, at(&doc, "k := 100"));
        let uses = usages(&doc, &Context::default(), at(&doc, "k^2"));
        assert_eq!(uses.len(), 2, "{expression}: {uses:?}");
        let renamed = apply(&doc, rename(&doc, &Context::default(), at(&doc, "k^2"), "j"));
        assert!(renamed.contains("sum_(j=k)^(k+10) (j^2) + k"), "{renamed}");
        assert!(renamed.contains("$k := 100$"));
        let hover = hover(&doc, &Context::default(), at(&doc, "k^2")).unwrap();
        assert!(hover.markdown.contains("local sum index"), "{}", hover.markdown);
    }
}

#[test]
fn nested_indices_shadow_outer_indices_but_their_bounds_use_the_outer_scope() {
    let doc = "#show: worksheet\n$k := 10$\n$f(k) := sum_(k=0)^10 (k + sum_(k=k)^(k+2) k^2) + k$";
    def(doc, "k^2", "k=k");
    def(doc, "k + sum", "k=0");
    def(doc, "k+2", "k=0");
    def(doc, "k$", "k) :=");
    let renamed = apply(doc, rename(doc, &Context::default(), at(doc, "k^2"), "j"));
    assert!(renamed.contains("sum_(j=k)^(k+2) j^2"), "{renamed}");
    let renamed = apply(doc, rename(doc, &Context::default(), at(doc, "k=0"), "j"));
    assert!(renamed.contains("sum_(j=0)^10 (j + sum_(k=j)^(j+2) k^2) + k"), "{renamed}");
}

#[test]
fn independent_sums_and_case_sensitive_indices_do_not_share_bindings() {
    let doc = "#show: worksheet\n$k := 3$\n$a := sum_(k=0)^10 k^2 + sum_(k=0)^10 k^3$\n$b := sum_(K=0)^10 k^4$";
    assert_eq!(usages(doc, &Context::default(), at(doc, "k^2")).len(), 2);
    assert_eq!(usages(doc, &Context::default(), at(doc, "k^3")).len(), 2);
    assert_eq!(usages(doc, &Context::default(), at(doc, "K=0")).len(), 1);
    def(doc, "k^4", "k := 3");
}

#[test]
fn rename_preserves_utf16_and_quotes_long_names() {
    let doc = "😀 #worksheet($answer := sum_(k=0)^10 k^2$)";
    let renamed = apply(doc, rename(doc, &Context::default(), at(doc, "k^2"), "counter"));
    assert!(renamed.contains("sum_(\"counter\"=0)^10 \"counter\"^2"), "{renamed}");
    let edits = rename(doc, &Context::default(), at(doc, "k^2"), "2\"\"k");
    let renamed = apply(doc, edits);
    assert!(renamed.contains("sum_(2\"\"k=0)^10 2\"\"k^2"), "{renamed}");
    assert_eq!(usages(&renamed, &Context::default(), at(&renamed, "2\"\"k^2")).len(), 2);
    let doc = "#show: worksheet\n$a := sum_(k=0)^10 k^2$";
    let renamed = apply(doc, rename(doc, &Context::default(), at(doc, "k^2"), "counter"));
    assert!(renamed.contains("sum_(\"counter\"=0)^10 \"counter\"^2"), "{renamed}");
    assert_eq!(usages(&renamed, &Context::default(), at(&renamed, "\"counter\"^2")).len(), 2);
}

#[test]
fn indices_are_highlighted_and_completed_only_in_their_scope() {
    let doc = "#show: worksheet\n$f(t, ..) := sum_(k=0)^infinity (k^2 + t)$\n$after := k$";
    let tokens = document_tokens(doc, &Context::default());
    for needle in ["k=0", "k^2"] {
        let pos = at(doc, needle);
        assert!(tokens.iter().any(|t| t.start == pos && t.length == 1 && t.kind == TokenKind::PARAMETER), "{tokens:?}");
    }
    let items = complete(doc, &Context::default(), at(doc, "k^2") + 1).unwrap().items;
    assert!(items.iter().any(|i| i.label == "k" && i.detail.contains("sum index")), "{items:?}");
    let items = complete(doc, &Context::default(), at(doc, "k$") + 1).unwrap().items;
    assert!(!items.iter().any(|i| i.label == "k" && i.detail.contains("sum index")));
    let h = hover(doc, &Context::default(), at(doc, "t)$")).unwrap();
    assert!(h.markdown.contains("parameter of `f`"), "{}", h.markdown);
    assert!(!h.markdown.contains("f(t, k)"), "{}", h.markdown);
}

#[test]
fn local_indices_do_not_leak_through_includes_or_become_implicit_parameters() {
    let doc = "$answer := sum_(k=0)^10 k^2$\n$outside := k$";
    let main = "#show: worksheet\n$k := 20$\n#include \"part.typ\"\n$after := k$";
    let context = Context {
        path: "part.typ".into(),
        ..Context::default().with_files(vec![ContextFile { path: "main.typ".into(), text: main.into() }]).with_main("main.typ")
    };
    let index = definition(doc, &context, at(doc, "k^2")).unwrap();
    assert!(index.path.is_none());
    assert_eq!(index.start, at(doc, "k=0"));
    let global = definition(doc, &context, at(doc, "k$")).unwrap();
    assert_eq!(global.path.as_deref(), Some("main.typ"));
    assert_eq!(global.start, at(main, "k := 20"));
    let edits = rename(doc, &context, at(doc, "k^2"), "j");
    assert_eq!(edits.len(), 2);
    assert!(edits.iter().all(|e| e.path.is_none()));

    let doc = "#show: worksheet\n$f(t, ..) := sum_(k=0)^infinity (k^2 + t)$";
    let h = hover(doc, &Context::default(), at(doc, "f(t")).unwrap();
    assert!(h.markdown.starts_with("```text\nf(t)"), "{}", h.markdown);
}

#[test]
fn completions_work_while_typing_a_summand_and_after_a_coefficient() {
    for doc in [
        "#show: worksheet\n$a := sum_(k=0)^10 $",
        "#show: worksheet\n$a := sum_(k=0)^10 (2 )$",
        "#worksheet($a := sum_(k=0)^10 $)",
    ] {
        let cursor = if doc.contains("(2 )") { at(doc, ")$") }
            else { at(doc, "$a") + "$a := sum_(k=0)^10 ".encode_utf16().count() as u32 };
        let items = complete(doc, &Context::default(), cursor).unwrap().items;
        assert!(items.iter().any(|i| i.label == "k" && i.detail.contains("sum index")), "missing local completion in {doc}");
    }
}

#[test]
fn ode_and_optimization_locals_have_distinct_scopes_and_help() {
    let doc="#import operations: odesolve, minimize as minimum\n#show: worksheet\n$y := 9$ $t := 2$ $x := 3$\n$a := odesolve(-y + t,y,t,y,t,4)$\n$b := minimum(x^2,x,x,10)$";
    def(doc,"y + t","y,t,y");
    def(doc,"t,y,t,y","t,y,t,4");
    def(doc,"y,t,4","y := 9");
    def(doc,"t,4","t := 2");
    def(doc,"x^2","x,x,10");
    def(doc,"x,10","x := 3");
    let state=hover(doc,&Context::default(),at(doc,"-y + t")+1).unwrap();
    assert!(state.markdown.contains("local ODE state"),"{}",state.markdown);
    let renamed=apply(doc,rename(doc,&Context::default(),at(doc,"-y + t")+1,"z"));
    assert!(renamed.contains("odesolve(-z + t,z,t,y,t,4)"),"{renamed}");
}
