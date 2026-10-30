use solveng_ide::{Context, TokenKind, complete, definition, document_tokens};

fn offset(text: &str, needle: &str) -> u32 {
    text[..text.find(needle).unwrap()].encode_utf16().count() as u32
}

fn tokens(doc: &str) -> Vec<(String, u8)> {
    let all: Vec<u16> = doc.encode_utf16().collect();
    document_tokens(doc, &Context::new("")).iter().map(|t| {
        (String::from_utf16(&all[t.start as usize..(t.start + t.length) as usize]).unwrap(), t.kind)
    }).collect()
}

#[test]
fn case_functions_and_their_parameters_resolve_from_guards_and_graphs() {
    let doc = "#show: math\n$sigma(t) := cases(0 & \"für\" t < 0, 1 & \"für\" t >= 0)$\n#graph($σ(x)$)";
    let context = Context::new("");
    let target = definition(doc, &context, offset(doc, "σ(x)")).unwrap();
    assert_eq!(target.start, offset(doc, "sigma"));
    for guard in ["t < 0", "t >= 0"] {
        let target = definition(doc, &context, offset(doc, guard)).unwrap();
        assert_eq!(target.start, offset(doc, "t)"));
    }
}

#[test]
fn cases_complete_with_math_syntax() {
    for doc in ["#show: math\n$x := cas", "#graph($cas"] {
        let found = complete(doc, &Context::new(""), doc.encode_utf16().count() as u32).unwrap();
        let item = found.items.iter().find(|item| item.label == "cases").unwrap();
        assert!(item.insert.starts_with("cases(") && item.insert.contains('&'), "{}", item.insert);
    }
}

#[test]
fn case_labels_are_text_not_names() {
    let doc = "#show: math\n$g_1(t) := cases(\n  2t & \"für\" 0 <= t <= 1/2,\n  2-2t & \"für\" 1/2 < t <= 1,\n  0 & \"sonst.\"\n)$\n$\"tews\" := 1$";
    let found = tokens(doc);
    for label in ["\"für\"", "\"sonst.\""] {
        assert!(found.contains(&(label.to_string(), TokenKind::STRING)), "{label} in {found:?}");
    }
    // quoted text elsewhere is still a name
    assert!(found.contains(&("\"tews\"".to_string(), TokenKind::VARIABLE)), "{found:?}");
}
