use solveng_core::{api::{self, LineReq}, env::Env, eval::value, format::Cfg, parse::{Mode, statements}};

fn run(env: &mut Env, src: &str, mode: Mode) -> solveng_core::sheet::Out {
    let out = api::line(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true });
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out
}

#[test]
fn definitions_keep_formula_and_operator() {
    let out = run(&mut Env::default(), "t := 23 + 3", Mode::Code);
    assert_eq!(out.show.as_deref(), Some("t ≔ \"23\" + \"3\""));
    assert!(out.result.is_none());
}

#[test]
fn separators_and_continuations() {
    assert_eq!(statements("t := 1 \\ g := 2", Mode::Math), ["t := 1", "g := 2"]);
    assert_eq!(statements("t := 1\ng := 2", Mode::Code), ["t := 1", "g := 2"]);
    assert_eq!(statements("t := 1 +\n2\ng := (3 +\n4)", Mode::Code), ["t := 1 +\n2", "g := (3 +\n4)"]);
    assert_eq!(statements("t := 1 // comment\ng := 2", Mode::Code), ["t := 1", "g := 2"]);
    assert_eq!(statements("x := \"a\\b\"", Mode::Code), ["x := \"a\\b\""]);
}

#[test]
fn calculus_agrees_in_both_modes() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        run(&mut env, "f(x) := e^x", mode);
        let out = run(&mut env, "dif / (dif t) f(t) ->", mode);
        assert_eq!(out.result.as_deref(), Some("e^(t)"));
        run(&mut env, "a := integral_(0)^(1) x^2 dif x =", mode);
        assert!((value(&env.vars["a"].expr).unwrap().v.f() - 1.0 / 3.0).abs() < 1e-12);
        run(&mut env, "b := integral_(0)^(1) exp(-x^2) dif x =", mode);
        assert!((value(&env.vars["b"].expr).unwrap().v.f() - 0.746824132812427).abs() < 1e-10);
        run(&mut env, "c := integral_(-infinity)^(0) e^x dif x =", mode);
        assert!((value(&env.vars["c"].expr).unwrap().v.f() - 1.0).abs() < 1e-9, "{:?}", env.vars["c"].expr);
        run(&mut env, "x := 25", mode);
        run(&mut env, "d := integrate(exp(-x^2), x, 0, 1) =", mode);
        assert!((value(&env.vars["d"].expr).unwrap().v.f() - 0.746824132812427).abs() < 1e-10);
        run(&mut env, "sigma(tau) := e^tau", mode);
        run(&mut env, "F(t) := integral_(-infinity)^(t) sigma(tau) dif tau", mode);
        let out = run(&mut env, "dif / (dif t) F(t) ->", mode);
        assert_eq!(out.result.as_deref(), Some("e^(t)"));
    }
}

#[test]
fn numerical_integrals_validate_domains_and_units() {
    for source in ["integral_(-1)^(1) 1/x dif x =", "integrate(exp(-x^2), x, [1 m], [2 s]) =", "integral_(0)^(infinity) 1 dif x ="] {
        let out = api::line(&mut Env::default(), &LineReq { src: source.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
        assert!(!out.errors.is_empty(), "{source}: {out:?}");
    }
    let out = run(&mut Env::default(), "integral_0^1 x^2 dif x ->", Mode::Code);
    assert_eq!(out.result.as_deref(), Some("frac(\"1\", \"3\")"));
}

#[test]
fn integral_substitution_does_not_capture_function_arguments() {
    use solveng_core::expr::{Expr, func, Func, int, mul, pow, simp, sym};
    let body = mul(vec![sym("t"), func(Func::Exp, pow(sym("x"), int(2)))]);
    let integral = Expr::Integral(Box::new(body), "x".into(), Box::new(int(0)), Box::new(int(1)));
    let replaced = integral.subst("t", &sym("x"));
    assert_eq!(replaced.syms(), ["x"]);
    let evaluated = simp(&replaced.subst("x", &int(2)));
    let result = value(&evaluated).unwrap().v.f();
    assert!((result - 2.925303491814363).abs() < 1e-9, "{result}");
}

#[test]
fn unspecified_functions_stay_symbolic_and_resolve_later() {
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        run(&mut env, "f(t) := integral_(-infinity)^t sigma(tau) dif tau", mode);
        assert_eq!(env.funs["f"].body.syms(), ["t"]);
        let out = run(&mut env, "dif / (dif t) f(t) ->", mode);
        assert_eq!(out.result.as_deref(), Some("σ(t)"));
        let out = run(&mut env, "f(t) ->", mode);
        assert!(out.result.unwrap().contains("σ(τ)"));
        for src in ["f(0) =", "sigma(0) ="] {
            let out = api::line(&mut env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true });
            assert!(out.errors.iter().any(|e| e.contains("σ isn't defined")), "{out:?}");
        }
        run(&mut env, "sigma(u) := e^u", mode);
        let out = run(&mut env, "f(0) =", mode);
        assert!(out.result.is_some());
        run(&mut env, "answer := f(0) =", mode);
        assert!((value(&env.vars["answer"].expr).unwrap().v.f() - 1.0).abs() < 1e-9);
        assert_eq!(run(&mut env, "dif / (dif t) f(t) ->", mode).result.as_deref(), Some("e^(t)"));
    }
}

#[test]
fn unspecified_calls_support_substitution_cancellation_and_multiple_arguments() {
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        run(&mut env, "g(x, ..) := q(x, y)", mode);
        assert_eq!(env.funs["g"].params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["x", "y"]);
        assert_eq!(run(&mut env, "g(2, 3) ->", mode).result.as_deref(), Some(r#"q("2", "3")"#));
        run(&mut env, "a := q(2, 3)", mode);
        assert_eq!(run(&mut env, "q(x, y) - q(x, y) =", mode).result.as_deref(), Some(r#""0""#));
        assert!(run(&mut env, "diff(q(x), x) ->", mode).result.unwrap().contains("dif"));
        run(&mut env, "q(u, v) := u + v", mode);
        run(&mut env, "answer := a =", mode);
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 5.0);
        assert_eq!(run(&mut env, "g(2, 3) =", mode).result.as_deref(), Some(r#""5""#));
    }
}

#[test]
fn deferred_calls_still_validate_arity_recursion_and_units() {
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        run(&mut env, "f(x) := q(x)", mode);
        run(&mut env, "q(x, y) := x + y", mode);
        let bad = api::line(&mut env, &LineReq { src: "f(2) =".into(), mode, cfg: Cfg::default(), listed: true });
        assert!(bad.errors.iter().any(|e| e.contains("takes 2 arguments")), "{bad:?}");
        let bad = api::line(&mut Env::default(), &LineReq { src: "f(x) := f(x)".into(), mode, cfg: Cfg::default(), listed: true });
        assert!(!bad.errors.is_empty());
        let mut env = Env::default();
        run(&mut env, "f(x) := q(x)", mode);
        let bad = api::line(&mut env, &LineReq { src: "q(x) := f(x)".into(), mode, cfg: Cfg::default(), listed: true });
        assert!(!bad.errors.is_empty());
        run(&mut env, "q(x: m) := x", mode);
        run(&mut env, "answer := f(2) =", mode);
        assert_eq!(value(&env.vars["answer"].expr).unwrap().d, solveng_core::units::Units::default().dim(&solveng_core::units::UnitExpr::one("m")).unwrap());
        let bad = api::line(&mut env, &LineReq { src: "f(2 s) =".into(), mode, cfg: Cfg::default(), listed: true });
        assert!(bad.errors.iter().any(|e| e.contains("takes x in")), "{bad:?}");
    }
}

#[test]
fn deferred_derivatives_differentiate_before_substituting_arguments() {
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        run(&mut env, "g(x) := diff(q(x), x)", mode);
        run(&mut env, "a := g(2)", mode);
        run(&mut env, "q(t) := t^2", mode);
        for src in ["answer := a =", "answer := g(2) ="] {
            run(&mut env, src, mode);
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 4.0);
        }
    }
}

#[test]
fn deferred_calls_use_custom_parameter_units_in_values_and_reports() {
    use solveng_core::units::UnitOpt;
    for mode in [Mode::Math, Mode::Code] {
        let mut cfg = Cfg::default();
        cfg.units.insert("span".into(), UnitOpt::Str("2 m".into()));
        let mut env = Env::default();
        for src in ["f(x) := q(x)", "a := f(3)", "q(x: span) := x", "answer := a ="] {
            let out = api::line(&mut env, &LineReq { src: src.into(), mode, cfg: cfg.clone(), listed: true });
            assert!(out.errors.is_empty(), "{src}: {out:?}");
        }
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 6.0);
        let rows = api::defined(&env, &api::DefinedReq { cfg: cfg.clone() }).unwrap();
        assert_eq!(rows.iter().find(|r| r.name == "a").unwrap().si, Some(6.0));
    }
}
