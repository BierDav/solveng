mod support;
use solveng_core::{api, env::Env};
use support::req;

fn sheet() -> Env {
    let mut env = Env::default();
    for src in ["R := 4.7 kΩ", "I := 2 mA", "U := R I", "f(x: Ω) := 2 x", "g(t) := a t"] {
        support::run(&mut env, src);
    }
    env
}

fn value(env: &Env, src: &str) -> Result<String, String> {
    api::show_value(env, &req(src, env)).map(|s| support::readable(&s)).map_err(|e| support::readable(&e))
}

#[test]
fn shows_the_result_alone() {
    let env = sheet();
    assert_eq!(value(&env, "U").unwrap(), "\"9.4\" thin upright(\"V\")");
    // a clause of its own picks the unit
    assert!(value(&env, "U => mV").unwrap().contains("9400"), "{:?}", value(&env, "U => mV"));
    assert_eq!(value(&env, "U / 2").unwrap(), "\"4.7\" thin upright(\"V\")");
    assert!(value(&env, "f(2 Ω)").unwrap().contains("4"), "{:?}", value(&env, "f(2 Ω)"));
    // a function alone is its body; a symbolic result shows as one
    assert!(value(&env, "g").unwrap().contains('a'), "{:?}", value(&env, "g"));
    assert!(value(&env, "g(2) ->").unwrap().contains('a'), "{:?}", value(&env, "g(2) ->"));
}

#[test]
fn leaves_the_worksheet_as_it_is() {
    let env = sheet();
    assert!(value(&env, "x := 2").unwrap_err().contains("nothing can be defined"));
    assert!(value(&env, "x := 2 =>").unwrap_err().contains("nothing can be defined"));
    assert!(!env.vars.contains_key("x"));
    assert!(value(&env, "q + 1").unwrap_err().contains("'q'"));
}

#[test]
fn a_unit_after_the_trailing_mark_stays_a_unit() {
    let env = sheet();
    for src in ["U => mV", "U => [mV]", "U -> mV", "U => mV, digits(2)"] {
        let shown = value(&env, src).unwrap_or_else(|e| panic!("{src}: {e}"));
        assert!(shown.contains("9400") && shown.contains("mV"), "{src}: {shown}");
    }
    // a value that can't be calculated says why, not that the unit is a variable
    let mut env = Env::default();
    support::run(&mut env, "U := 5 \"V\"");
    let e = value(&env, "U => mV").unwrap_err();
    assert!(e.contains("'V'") && !e.contains("'mV'"), "{e}");
    let mut env = Env::default();
    support::run(&mut env, "Y := 3 V");
    for src in ["Y => mV", "Y -> mV", "Y => [mV]"] {
        let out = support::run(&mut env, src);
        assert!(out.result.unwrap().contains("3000"), "{src}");
    }
}
