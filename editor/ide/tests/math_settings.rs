use solveng_ide::*;

fn coloured(doc: &str, needle: &str) -> bool {
    let at = doc[..doc.find(needle).unwrap()].encode_utf16().count() as u32;
    document_tokens(doc, &Context::default()).iter().any(|t| t.start <= at && at < t.start + t.length)
}

fn labels(doc: &str) -> Vec<String> {
    let at = doc.find('|').unwrap();
    complete(&doc.replace('|', ""), &Context::default(), doc[..at].encode_utf16().count() as u32)
        .map(|c| c.items.into_iter().map(|i| i.label.split(": ").next().unwrap().to_owned()).collect()).unwrap_or_default()
}

#[test]
fn math_settings_and_scopes_inherit_evaluation() {
    let doc = "#show: math\n$a := 1$\n#math(evaluate: false)[$b := 2$ #math(digits: 8)[$c := 3$] #mathscope[$d := 4$] #math(evaluate: true)[$e := 5$]]\n$f := 6$";
    for (name, active) in [("a", true), ("b", false), ("c", false), ("d", false), ("e", true), ("f", true)] {
        assert_eq!(coloured(doc, &format!("{name} :=")), active, "{name}");
    }
    assert!(!coloured("$x := 1$", "x :="));
    assert!(!coloured("#show: worksheet\n$x := 1$", "x :="));
    assert!(!coloured("#worksheet($x := 1$)", "x :="));
    assert!(!coloured("#math(evaluate: false, digits: 8)($x := 1$)", "x :="));
    assert!(coloured("#math($x := 1$)", "x :="));
    assert!(!coloured("#let forced = math.with(evaluate: true)\n#math(evaluate: false)[#forced(evaluate: auto)[$x := 1$]]", "x :="));
    assert!(evaluates("#show: math", &Context::default()));
}

#[test]
fn settings_preserve_definitions_and_scopes_contain_them() {
    let doc = "#show: math\n$a := 1$\n#show: math.with(digits: 8)\n#math[$b := 2$]\n#mathscope[$c := 3$]\n$|$";
    let names = labels(doc);
    assert!(names.iter().any(|n| n == "a"), "{names:?}");
    assert!(names.iter().any(|n| n == "b"), "{names:?}");
    assert!(!names.iter().any(|n| n == "c"), "{names:?}");
    let names = labels("#show: math\n$a := 1$\n#mathscope(scope: \"shared\")[$b := 2$]\n$|$");
    assert!(names.iter().any(|n| n == "b"), "{names:?}");
}

#[test]
fn closest_reset_configuration_wins() {
    let prefix = "#show: math.with(clear-at: heading.where(level: 1))\n$a := 1$\n";
    for settings in ["clear-at: none", "clear-at: heading.where(level: 2)"] {
        let names = labels(&format!("{prefix}#math({settings})[\n= Keep\n$|$\n]"));
        assert!(names.iter().any(|n| n == "a"), "{settings}: {names:?}");
    }
    let names = labels(&format!("{prefix}= Reset\n$|$"));
    assert!(!names.iter().any(|n| n == "a"), "{names:?}");
}

#[test]
fn new_api_signatures_separate_settings_from_scopes() {
    let functions = params::functions();
    let math = functions.iter().find(|f| f.name == "math").unwrap();
    assert!(math.named.iter().any(|p| p.name == "digits"));
    assert!(!math.named.iter().any(|p| p.name == "scope"));
    let scope = functions.iter().find(|f| f.name == "mathscope").unwrap();
    assert_eq!(scope.named.len(), 1);
    assert_eq!(scope.named[0].name, "scope");
    assert!(!functions.iter().any(|f| matches!(f.name.as_str(), "calculate" | "worksheet")));
}
