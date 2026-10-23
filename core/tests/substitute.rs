use solveng_core::{api::{self, LineReq}, env::Env, eval::value, format::Cfg, parse::Mode};

fn line(env: &mut Env, src: &str, mode: Mode) -> solveng_core::sheet::Out {
    api::line(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true })
}

fn result(env: &mut Env, src: &str, mode: Mode) -> String {
    let out = line(env, src, mode);
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out.result.unwrap()
}

fn error(env: &mut Env, src: &str) -> String {
    let out = line(env, src, Mode::Code);
    assert_eq!(out.errors.len(), 1, "{src}: {:?}", out.errors);
    out.errors[0].clone()
}

#[test]
fn puts_in_the_listed_formulas_and_keeps_every_other_name() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        line(&mut env, "k := a b", mode);
        line(&mut env, "m := k d", mode);
        assert_eq!(result(&mut env, "z := m c -> substitute(m)", mode), "c d k");
        // a formula using another listed name gets that one's formula too
        assert_eq!(result(&mut env, "z := m c -> substitute(m, k)", mode), "a b c d");
        assert_eq!(result(&mut env, "m c -> substitute(m)", mode), "c d k");
        assert_eq!(result(&mut env, "f(t) := m t -> substitute(m)", mode), "d k t");
    }
}

#[test]
fn keeps_known_values_out_and_the_value_unchanged() {
    let mut env = Env::default();
    line(&mut env, "R := 4.7 kΩ", Mode::Code);
    line(&mut env, "I := 2 mA", Mode::Code);
    line(&mut env, "U := R I", Mode::Code);
    assert_eq!(result(&mut env, "P := U^2 / R -> substitute(U)", Mode::Code), "I^(\"2\") R");
    assert!((value(&env.vars["P"].expr).unwrap().v.f() - 0.0188).abs() < 1e-12);
    // what's listed and has a value is a value
    assert!(!result(&mut env, "P2 := U^2 / R -> substitute(U, R, I)", Mode::Code).contains('R'));
}

#[test]
fn a_redefinition_puts_in_the_older_formula() {
    let mut env = Env::default();
    line(&mut env, "x := a + 1", Mode::Code);
    assert_eq!(result(&mut env, "x := 2 x -> substitute(x)", Mode::Code), "\"2\" lr((a + \"1\"))");
}

#[test]
fn says_what_it_can_and_cannot_put_in() {
    let mut env = Env::default();
    line(&mut env, "k := a b", Mode::Code);
    line(&mut env, "f(t) := 2 t", Mode::Code);
    line(&mut env, "n := 3", Mode::Code);
    assert!(error(&mut env, "z := k c -> substitute(q)").contains("'q' isn't defined"));
    assert!(error(&mut env, "z := k c -> substitute(f)").contains("is a function"));
    assert!(error(&mut env, "z := k -> substitute(n)").contains("doesn't occur"));
    assert!(error(&mut env, "g(k) := 2 k -> substitute(k)").contains("parameter"));
    assert!(error(&mut env, "z := k c = substitute(k)").contains("after ->"));
    assert!(error(&mut env, "k c -> substitute").contains("substitute(x)"));
    // @ is gone, and says what replaces it
    assert!(error(&mut env, "z := @k c").contains("-> substitute(k)"));
}
