mod support;
use solveng_core::{env::Env, eval::value, parse::statements};
use support::{line, line_raw, run, unit};

#[test]
fn definitions_keep_formula_and_operator() {
    let mut env = Env::default();
    let out = run(&mut env, "t := 23 + 3");
    assert!(out.result.is_none());
    assert_eq!(value(&env.vars["t"].expr).unwrap().v.f(), 26.0);
}

#[test]
fn separators_and_continuations() {
    assert_eq!(statements("t := 1 \\ g := 2"), ["t := 1", "g := 2"]);
}

#[test]
fn calculus_agrees() {
    let mut env = Env::default();
    run(&mut env, "f(x) := e^x");
    let out = run(&mut env, "dif / (dif t) f(t) ->");
    assert_eq!(out.result.as_deref(), Some("#svconst(\"euler\")^(t)"));
    run(&mut env, "a := integral_(0)^(1) x^2 dif x =");
    assert!((value(&env.vars["a"].expr).unwrap().v.f() - 1.0 / 3.0).abs() < 1e-12);
    run(&mut env, "b := integral_(0)^(1) exp(-x^2) dif x =");
    assert!((value(&env.vars["b"].expr).unwrap().v.f() - 0.746824132812427).abs() < 1e-10);
    run(&mut env, "c := integral_(-infinity)^(0) e^x dif x =");
    assert!((value(&env.vars["c"].expr).unwrap().v.f() - 1.0).abs() < 1e-9, "{:?}", env.vars["c"].expr);
    run(&mut env, "x := 25");
    run(&mut env, "d := integrate(exp(-x^2), x, 0, 1) =");
    assert!((value(&env.vars["d"].expr).unwrap().v.f() - 0.746824132812427).abs() < 1e-10);
    run(&mut env, "sigma(tau) := e^tau");
    run(&mut env, "F(t) := integral_(-infinity)^(t) sigma(tau) dif tau");
    let out = run(&mut env, "dif / (dif t) F(t) ->");
    assert_eq!(out.result.as_deref(), Some("#svconst(\"euler\")^(t)"));
}

#[test]
fn numerical_integrals_validate_domains_and_units() {
    for source in ["integral_(-1)^(1) 1/x dif x =", "integrate(exp(-x^2), x, [1 m], [2 s]) =", "integral_(0)^(infinity) 1 dif x ="] {
        let out = line(&mut Env::default(), source);
        assert!(!out.errors.is_empty(), "{source}: {out:?}");
    }
    let out = run(&mut Env::default(), "integral_0^1 x^2 dif x ->");
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
    let mut env = Env::default();
    run(&mut env, "f(t) := integral_(-infinity)^t sigma(tau) dif tau");
    assert_eq!(env.funs["f"].body.syms(), ["t"]);
    let out = run(&mut env, "dif / (dif t) f(t) ->");
    assert_eq!(out.result.as_deref(), Some("σ(t)"));
    let out = run(&mut env, "f(t) ->");
    assert!(out.result.unwrap().contains("σ(τ)"));
    for src in ["f(0) =", "sigma(0) ="] {
        let out = line(&mut env, src);
        assert!(out.errors.iter().any(|e| e.contains("σ isn't defined")), "{out:?}");
    }
    run(&mut env, "sigma(u) := e^u");
    let out = run(&mut env, "f(0) =");
    assert!(out.result.is_some());
    run(&mut env, "answer := f(0) =");
    assert!((value(&env.vars["answer"].expr).unwrap().v.f() - 1.0).abs() < 1e-9);
    assert_eq!(run(&mut env, "dif / (dif t) f(t) ->").result.as_deref(), Some("#svconst(\"euler\")^(t)"));
}

#[test]
fn unspecified_calls_support_substitution_cancellation_and_multiple_arguments() {
    let mut env = Env::default();
    run(&mut env, "g(x, ..) := q(x, y)");
    assert_eq!(env.funs["g"].params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["x", "y"]);
    assert_eq!(run(&mut env, "g(2, 3) ->").result.as_deref(), Some(r#"q("2", "3")"#));
    run(&mut env, "a := q(2, 3)");
    assert_eq!(run(&mut env, "q(x, y) - q(x, y) =").result.as_deref(), Some(r#""0""#));
    assert!(run(&mut env, "diff(q(x), x) ->").result.unwrap().contains("dif"));
    run(&mut env, "q(u, v) := u + v");
    run(&mut env, "answer := a =");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 5.0);
    assert_eq!(run(&mut env, "g(2, 3) =").result.as_deref(), Some(r#""5""#));
}

#[test]
fn deferred_calls_still_validate_arity_recursion_and_units() {
    let mut env = Env::default();
    run(&mut env, "f(x) := q(x)");
    run(&mut env, "q(x, y) := x + y");
    let bad = line(&mut env, "f(2) =");
    assert!(bad.errors.iter().any(|e| e.contains("takes 2 arguments")), "{bad:?}");
    let bad = line(&mut Env::default(), "f(x) := f(x)");
    assert!(!bad.errors.is_empty());
    let mut env = Env::default();
    run(&mut env, "f(x) := q(x)");
    let bad = line(&mut env, "q(x) := f(x)");
    assert!(!bad.errors.is_empty());
    run(&mut env, "q(x: m) := x");
    run(&mut env, "answer := f(2 m) =");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().d, support::dim("m"));
    let bad = line(&mut env, "f(2 s) =");
    assert!(bad.errors.iter().any(|e| e.contains("takes x in")), "{bad:?}");
}

#[test]
fn deferred_derivatives_differentiate_before_substituting_arguments() {
    let mut env = Env::default();
    run(&mut env, "g(x) := diff(q(x), x)");
    run(&mut env, "a := g(2)");
    run(&mut env, "q(t) := t^2");
    for src in ["answer := a =", "answer := g(2) ="] {
        run(&mut env, src);
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 4.0);
    }
}

#[test]
fn deferred_calls_use_custom_parameter_units_in_values_and_reports() {
    use solveng_core::bindings::{self, BindingReq};
    let construct = |kind: &str, src: String, identity: Option<&str>| {
        let b = bindings::construct(BindingReq { kind: kind.into(), src, identity: identity.map(str::to_owned), presentation: String::new(), description: String::new(), auto: false }).unwrap();
        format!("⟦{}⟧", b.encode().unwrap())
    };
    let metre = construct("base-unit", String::new(), Some("length"));
    let span = construct("unit", format!("2 {metre}"), None);
    let mut env = Env::default();
    for src in ["f(x) := q(x)".to_string(), format!("a := f(3 {span})"), format!("q(x: {span}) := x"), "answer := a =".to_string()] {
        run(&mut env, &src);
    }
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 6.0);
    let rows = solveng_core::api::defined(&env, &solveng_core::api::DefinedReq { cfg: support::cfg() }).unwrap();
    assert_eq!(rows.iter().find(|r| r.name == "a").unwrap().si, Some(6.0));
}

#[test]
fn typed_function_arguments_require_explicit_dimensions() {
    let mut env = Env::default();
    run(&mut env, "f(x: A) := x");
    for source in ["f(1) =", "f(1 + 2) =", "f(1 s) ="] {
        let out = line(&mut env, source);
        assert!(out.errors.iter().any(|error| error.contains("takes x in A")), "{source}: {out:?}");
    }
    let out = run(&mut env, "answer := f(1 A) =");
    assert_eq!(out.result.as_deref(), Some("\"1\" thin upright(\"A\")"));
    let answer = value(&env.vars["answer"].expr).unwrap();
    assert_eq!(answer.v.f(), 1.0);
    assert_eq!(answer.d, support::dim("A"));
    run(&mut env, "g(y) := f(y)");
    run(&mut env, "answer := g(2 A) =");
    // a plain letter after a number is a name: A, not the ampere
    let mut env = Env::default();
    run(&mut env, "f(x: A) := x");
    let out = line_raw(&mut env, "f(1A) =");
    assert!(out.errors.iter().any(|error| error.contains("A")), "{out:?}");
    line_raw(&mut env, &format!("A := 2{}", unit("A")));
    let out = line_raw(&mut env, "answer := f(1A) =");
    assert!(out.errors.is_empty(), "{out:?}");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 2.0);
}
