mod support;
use solveng_core::{env::Env, eval::value};
use support::{line, result};

fn error(env: &mut Env, src: &str) -> String {
    let out = line(env, src);
    assert_eq!(out.errors.len(), 1, "{src}: {:?}", out.errors);
    out.errors[0].clone()
}

#[test]
fn puts_in_the_listed_formulas_and_keeps_every_other_name() {
    let mut env = Env::default();
    line(&mut env, "k := a b");
    line(&mut env, "m := k d");
    assert_eq!(result(&mut env, "z := m c -> substitute(m)"), "c dot d dot k");
    // a formula using another listed name gets that one's formula too
    assert_eq!(result(&mut env, "z := m c -> substitute(m, k)"), "a dot b dot c dot d");
    assert_eq!(result(&mut env, "m c -> substitute(m)"), "c dot d dot k");
    assert_eq!(result(&mut env, "f(t) := m t -> substitute(m)"), "d dot k dot t");
}

#[test]
fn keeps_known_values_out_and_the_value_unchanged() {
    let mut env = Env::default();
    line(&mut env, "R := 4.7 kΩ");
    line(&mut env, "I := 2 mA");
    line(&mut env, "U := R I");
    assert_eq!(result(&mut env, "P := U^2 / R -> substitute(U)"), "I^(\"2\") dot R");
    assert!((value(&env.vars["P"].expr).unwrap().v.f() - 0.0188).abs() < 1e-12);
    // what's listed and has a value is a value
    assert!(!result(&mut env, "P2 := U^2 / R -> substitute(U, R, I)").contains('R'));
}

#[test]
fn a_redefinition_puts_in_the_older_formula() {
    let mut env = Env::default();
    line(&mut env, "x := a + 1");
    assert_eq!(result(&mut env, "x := 2 x -> substitute(x)"), "\"2\" lr((a + \"1\"))");
}

#[test]
fn says_what_it_can_and_cannot_put_in() {
    let mut env = Env::default();
    line(&mut env, "k := a b");
    line(&mut env, "f(t) := 2 t");
    line(&mut env, "n := 3");
    assert!(error(&mut env, "z := k c -> substitute(q)").contains("'q' isn't defined"));
    assert!(error(&mut env, "z := k c -> substitute(f)").contains("is a function"));
    assert!(error(&mut env, "z := k -> substitute(n)").contains("doesn't occur"));
    assert!(error(&mut env, "g(k) := 2 k -> substitute(k)").contains("parameter"));
    assert!(error(&mut env, "z := k c = substitute(k)").contains("after ->"));
    assert!(error(&mut env, "k c -> substitute").contains("substitute(x)"));
    // @ is gone, and says what replaces it
    assert!(error(&mut env, "z := @k c").contains("-> substitute(k)"));
}
