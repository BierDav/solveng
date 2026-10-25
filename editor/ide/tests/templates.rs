use solveng_ide::*;

const PACKAGE: &str = "#import \"@solveng/solveng:1.0.0\": *\n";
const TEMPLATE: &str = r#"
#let tri = operation("tri", display: $Λ$)
#let template(title: "Assignment", body) = {
  show: text.with(size: 11pt)
  show: worksheet
  worksheet($sigma(t) := cases(0 & t < 0, 1 & t >= 0)$, hidden: true)
  worksheet(body, scope: "inherit", clear-at: heading.where(level: 1))
}
"#;

fn context(template: &str) -> Context {
    Context { path: "course/exercise.typ".into(), ..Context::default() }.with_files(vec![
        ContextFile { path: "course/template.typ".into(), text: template.into() },
    ])
}

fn at(doc: &str, needle: &str) -> u32 {
    doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32
}

fn apply(doc: &str, edits: &[Edit]) -> String {
    let mut edits: Vec<_> = edits.iter().enumerate().collect();
    edits.sort_by_key(|(i, e)| (std::cmp::Reverse(e.start), std::cmp::Reverse(*i)));
    let mut text: Vec<_> = doc.encode_utf16().collect();
    for (_, edit) in edits {
        assert!(edit.path.is_none(), "imports should be added to the current file");
        text.splice(edit.start as usize..edit.end as usize, edit.text.encode_utf16());
    }
    String::from_utf16(&text).unwrap()
}

#[test]
fn complex_layouts_enable_highlighting_and_import_fixes_before_compiling() {
    let cx = context(&format!("{PACKAGE}{TEMPLATE}"));
    let doc = "#import \"template.typ\": *\n#show: template\n😀\n== $g(t) := Re(s(t))$";
    assert!(evaluates(doc, &cx));
    let tokens = document_tokens(doc, &cx);
    assert!(tokens.iter().any(|t| t.start == at(doc, ":=") && t.kind == TokenKind::OPERATOR), "{tokens:?}");
    for name in ["Re", "Im", "conj", "rect"] {
        let source = doc.replace("Re(s", &format!("{name}(s"));
        let fix = unit_fixes(&source, &cx, at(&source, &format!("{name}("))).into_iter()
            .find(|fix| fix.title == format!("Import {name} from solveng's operations")).expect("import fix through template");
        let fixed = apply(&source, &fix.edits);
        assert!(fixed.contains(&format!("#import operations: {name}")), "{fixed}");
        assert!(!diagnostics(&fixed, &cx).iter().any(|p| p.message.contains("Typst's own")), "{fixed}");
        assert!(document_tokens(&fixed, &cx).iter().any(|t| t.start == at(&fixed, &format!("{name}(")) && t.kind == TokenKind::BUILTIN_FUNCTION));
    }
}

#[test]
fn completion_imports_modules_through_selective_template_imports() {
    let cx = context(&format!("{PACKAGE}{TEMPLATE}"));
    let doc = "#import \"template.typ\": template\n#show: template\n$g(t) := Re$";
    let items = complete(doc, &cx, at(doc, "Re$") + 2).unwrap().items;
    let re = items.iter().find(|i| i.label == "Re").expect("real part completion");
    assert_eq!(re.insert, "Re($0)");
    let fixed = apply(doc, &re.edits);
    assert!(fixed.starts_with("#import \"template.typ\": template, operations\n#import operations: Re"), "{fixed}");
}

#[test]
fn reexported_operations_and_custom_bindings_keep_their_meaning() {
    let cx = context(&format!("{PACKAGE}#import operations: Re\n{TEMPLATE}"));
    let doc = "#import \"template.typ\": Re as real, tri, template\n#show: template\n$g(t) := real(tri(t))$";
    assert!(!diagnostics(doc, &cx).iter().any(|p| p.message.contains("import")));
    for name in ["real(", "tri("] {
        assert!(document_tokens(doc, &cx).iter().any(|t| t.start == at(doc, name) && t.kind == TokenKind::BUILTIN_FUNCTION));
    }
    assert!(unit_fixes(doc, &cx, at(doc, "tri(")).is_empty());
}

#[test]
fn wrappers_follow_import_chains_and_evaluate_overrides() {
    let mut cx = context(&format!("{PACKAGE}{TEMPLATE}"));
    cx.files.push(ContextFile { path: "course/defaults.typ".into(), text: "#import \"template.typ\": template as layout\n#let setup = layout.with(title: \"Default\")".into() });
    let doc = "#import \"defaults.typ\": setup\n$outside := 0$\n#show: setup\n$inside := 1$\n#worksheet(evaluate: false)[Text $disabled := 2$]\n$after := 3$";
    let tokens = document_tokens(doc, &cx);
    for (name, enabled) in [("outside", false), ("inside", true), ("disabled", false), ("after", true)] {
        assert_eq!(tokens.iter().any(|t| t.start == at(doc, name)), enabled, "{name}: {tokens:?}");
    }
    cx.files.push(ContextFile { path: "course/disabled.typ".into(), text: "#let quiet = worksheet.with(evaluate: false)\n#let setup = quiet.with(digits: 3)".into() });
    let disabled = "#import \"disabled.typ\": setup\n#show: setup.with(digits: 5)\n$x := 1$";
    assert!(!evaluates(disabled, &cx));
}

#[test]
fn unrelated_and_block_local_worksheets_do_not_enable_template_bodies() {
    for template in [
        "#let template(body) = { worksheet[$x := 1$]; body }",
        "#let template(body) = { let ignored = worksheet(body); body }",
        "#let template(body) = { let ignored = [#show: worksheet]; body }",
    ] {
        let cx = context(&format!("{PACKAGE}{template}"));
        assert!(!evaluates("#import \"template.typ\": *\n#show: template\n$x := 2$", &cx), "{template}");
    }
    let cx = context(&format!("#[{PACKAGE}]\n#let template(body) = worksheet(body)"));
    let doc = "#import \"template.typ\": *\n#show: template\n$Re(x)$";
    assert!(unit_fixes(doc, &cx, at(doc, "Re(")).is_empty(), "block-local modules must not be exported");
}

#[test]
fn cyclic_template_imports_are_bounded() {
    let mut cx = context("#import \"cycle.typ\": *\n#let template(body) = worksheet(body)");
    cx.files.push(ContextFile { path: "course/cycle.typ".into(), text: "#import \"template.typ\": *".into() });
    let doc = "#import \"template.typ\": *\n#show: template\n$Re(x)$";
    assert!(evaluates(doc, &cx));
    assert!(unit_fixes(doc, &cx, at(doc, "Re(")).is_empty());
}

#[test]
fn all_binding_modules_and_renamed_module_exports_can_be_imported() {
    let cx = context(&format!("{PACKAGE}{TEMPLATE}"));
    for (name, module) in [("km", "units"), ("pi", "constants"), ("conj", "operations")] {
        let doc = format!("#import \"template.typ\": *\n#show: template\n$x := {name}$");
        let fix = unit_fixes(&doc, &cx, at(&doc, &format!("{name}$"))).into_iter()
            .find(|f| f.title == format!("Import {name} from solveng's {module}")).unwrap();
        assert!(apply(&doc, &fix.edits).contains(&format!("#import {module}: {name}")));
    }
    let cx = context(&format!("#import \"@solveng/solveng:1.0.0\": worksheet, operations as ops\n{TEMPLATE}"));
    let doc = "#import \"template.typ\": template, ops as functions\n#show: template\n$Re(x)$";
    let fix = unit_fixes(doc, &cx, at(doc, "Re(")).into_iter().find(|f| f.title.starts_with("Import Re")).unwrap();
    let fixed = apply(doc, &fix.edits);
    assert!(fixed.contains("#import functions: Re"), "{fixed}");
    assert!(!diagnostics(&fixed, &cx).iter().any(|p| p.message.contains("Typst's own")));
}

#[test]
fn templates_can_wrap_content_blocks_and_do_not_leak_after_a_call() {
    for template in [
        "#let template(body) = { show: worksheet; body }",
        "#let template(body) = worksheet[Text #body]",
        "#let template(body) = { show: worksheet; block(body) }",
    ] {
        let cx = context(&format!("{PACKAGE}{template}"));
        let doc = "#import \"template.typ\": template\n#template[Text $inside := 1$]\n$outside := 2$";
        let tokens = document_tokens(doc, &cx);
        assert!(tokens.iter().any(|t| t.start == at(doc, "inside")), "{template}");
        assert!(!tokens.iter().any(|t| t.start == at(doc, "outside")), "{template}");
    }
}

#[test]
fn nested_bindings_at_eof_are_not_reexported() {
    let cx = context(&format!("{PACKAGE}{TEMPLATE}#[#import operations: Re]"));
    let doc = "#import \"template.typ\": *\n#show: template\n$Re(x)$";
    assert!(diagnostics(doc, &cx).iter().any(|p| p.message.contains("Typst's own")));
    assert!(unit_fixes(doc, &cx, at(doc, "Re(")).iter().any(|f| f.title.starts_with("Import Re")));
}
