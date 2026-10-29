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
    run(&mut env, "a := integral_(0)^(1) x^2 dif x =>");
    assert!((value(&env.vars["a"].expr).unwrap().v.f() - 1.0 / 3.0).abs() < 1e-12);
    run(&mut env, "b := integral_(0)^(1) exp(-x^2) dif x =>");
    assert!((value(&env.vars["b"].expr).unwrap().v.f() - 0.746824132812427).abs() < 1e-10);
    run(&mut env, "c := integral_(-infinity)^(0) e^x dif x =>");
    assert!((value(&env.vars["c"].expr).unwrap().v.f() - 1.0).abs() < 1e-9, "{:?}", env.vars["c"].expr);
    run(&mut env, "x := 25");
    run(&mut env, "d := integrate(exp(-x^2), x, 0, 1) =>");
    assert!((value(&env.vars["d"].expr).unwrap().v.f() - 0.746824132812427).abs() < 1e-10);
    run(&mut env, "sigma(tau) := e^tau");
    run(&mut env, "F(t) := integral_(-infinity)^(t) sigma(tau) dif tau");
    let out = run(&mut env, "dif / (dif t) F(t) ->");
    assert_eq!(out.result.as_deref(), Some("#svconst(\"euler\")^(t)"));
}

#[test]
fn numerical_integrals_validate_domains_and_units() {
    for source in ["integral_(-1)^(1) 1/x dif x =>", "integrate(exp(-x^2), x, [1 m], [2 s]) =>", "integral_(0)^(infinity) 1 dif x =>"] {
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
    for src in ["f(0) =>", "sigma(0) =>"] {
        let out = line(&mut env, src);
        assert!(out.errors.iter().any(|e| e.contains("σ isn't defined")), "{out:?}");
    }
    run(&mut env, "sigma(u) := e^u");
    let out = run(&mut env, "f(0) =>");
    assert!(out.result.is_some());
    run(&mut env, "answer := f(0) =>");
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
    assert_eq!(run(&mut env, "q(x, y) - q(x, y) =>").result.as_deref(), Some(r#""0""#));
    assert!(run(&mut env, "diff(q(x), x) ->").result.unwrap().contains("dif"));
    run(&mut env, "q(u, v) := u + v");
    run(&mut env, "answer := a =>");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 5.0);
    assert_eq!(run(&mut env, "g(2, 3) =>").result.as_deref(), Some(r#""5""#));
}

#[test]
fn deferred_calls_still_validate_arity_recursion_and_units() {
    let mut env = Env::default();
    run(&mut env, "f(x) := q(x)");
    run(&mut env, "q(x, y) := x + y");
    let bad = line(&mut env, "f(2) =>");
    assert!(bad.errors.iter().any(|e| e.contains("takes 2 arguments")), "{bad:?}");
    let bad = line(&mut Env::default(), "f(x) := f(x)");
    assert!(!bad.errors.is_empty());
    let mut env = Env::default();
    run(&mut env, "f(x) := q(x)");
    let bad = line(&mut env, "q(x) := f(x)");
    assert!(!bad.errors.is_empty());
    run(&mut env, "q(x: m) := x");
    run(&mut env, "answer := f(2 m) =>");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().d, support::dim("m"));
    let bad = line(&mut env, "f(2 s) =>");
    assert!(bad.errors.iter().any(|e| e.contains("takes x in")), "{bad:?}");
}

#[test]
fn deferred_derivatives_differentiate_before_substituting_arguments() {
    let mut env = Env::default();
    run(&mut env, "g(x) := diff(q(x), x)");
    run(&mut env, "a := g(2)");
    run(&mut env, "q(t) := t^2");
    for src in ["answer := a =>", "answer := g(2) =>"] {
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
    for src in ["f(x) := q(x)".to_string(), format!("a := f(3 {span})"), format!("q(x: {span}) := x"), "answer := a =>".to_string()] {
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
    for source in ["f(1) =>", "f(1 + 2) =>", "f(1 s) =>"] {
        let out = line(&mut env, source);
        assert!(out.errors.iter().any(|error| error.contains("takes x in A")), "{source}: {out:?}");
    }
    let out = run(&mut env, "answer := f(1 A) =>");
    assert_eq!(out.result.as_deref(), Some("\"1\" thin upright(\"A\")"));
    let answer = value(&env.vars["answer"].expr).unwrap();
    assert_eq!(answer.v.f(), 1.0);
    assert_eq!(answer.d, support::dim("A"));
    run(&mut env, "g(y) := f(y)");
    run(&mut env, "answer := g(2 A) =>");
    // a plain letter after a number is a name: A, not the ampere
    let mut env = Env::default();
    run(&mut env, "f(x: A) := x");
    let out = line_raw(&mut env, "f(1A) =>");
    assert!(out.errors.iter().any(|error| error.contains("A")), "{out:?}");
    line_raw(&mut env, &format!("A := 2{}", unit("A")));
    let out = line_raw(&mut env, "answer := f(1A) =>");
    assert!(out.errors.is_empty(), "{out:?}");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 2.0);
}

/// The differential ends the integrand wherever it stands: after it, closing a group around it, in a numerator.
#[test]
fn differentials_end_the_integrand_where_they_stand() {
    let cases = [
        ("z := integral_(0)^(pi/2) (e^(#i t) dif t) + 12 ->", r#""13" + #svconst("imaginary")"#),
        ("integral_(0)^(1) (x^2 dif x) ->", r#"frac("1", "3")"#),
        ("integral_(0)^(1) (x^2 + 1 dif x) * 2 ->", r#"frac("8", "3")"#),
        ("integral_(0)^(1) ((x dif x)) + 1 ->", r#"frac("3", "2")"#),
        ("integral_(0)^(1) x^2 + 1 dif x ->", r#"frac("4", "3")"#),
        ("integral_(0)^(1) x dif x / 2 ->", r#"frac("1", "4")"#),
        ("integral_(0)^(1) x dif x integral_(0)^(1) y dif y ->", r#"frac("1", "4")"#),
        ("integral_(0)^(2) dif x ->", r#""2""#),
        ("integral_(0)^(2) dif x + 1 ->", r#""3""#),
        ("integral_(0)^(1) frac(dif x, x + 1) ->", r#"ln("2")"#),
        ("integral_(0)^(1) frac(2 dif x, 1) ->", r#""2""#),
        ("integral_(0)^(1) (integral_(0)^(x) y dif y) dif x ->", r#"frac("1", "6")"#),
        ("integral_(0)^(1) integral_(0)^(x) (y dif y) dif x ->", r#"frac("1", "6")"#),
        ("integral_(0)^(1) \"xy\" dif \"xy\" ->", r#"frac("1", "2")"#),
    ];
    for (source, want) in cases {
        assert_eq!(run(&mut Env::default(), source).result.as_deref(), Some(want), "{source}");
    }
    let refused = [
        ("integral_(0)^(1) dif x x ->", "differential ends the integrand"),
        ("integral_(0)^(1) x dif x dif y ->", "dif stands for a differential"),
        ("integral_(0)^(1) sin(x dif x) ->", "dif stands for a differential"),
        ("integral_(0)^(1) x^(dif x) ->", "dif stands for a differential"),
        ("integral_(0)^(1) frac(1, dif x) ->", "dif stands for a differential"),
        ("integral_(0)^(1) |x dif x| ->", "dif stands for a differential"),
        ("integral_(0)^(1) x ->", "ends with dif"),
        ("x dif x ->", "dif stands for a differential"),
        ("∫_(0) x dif x ->", "both bounds"),
        ("∬ x dif x ->", "nest integrals"),
        ("∑ k ->", "both bounds"),
        ("∏ k ->", "products"),
    ];
    for (source, message) in refused {
        let out = line(&mut Env::default(), source);
        assert!(out.errors.iter().any(|e| e.contains(message)), "{source}: {out:?}");
    }
    // unreadable lines asking for a result are errors, not prose
    let out = line(&mut Env::default(), "integral_(0)^(1) (x dif =");
    assert!(!out.errors.is_empty() && !out.prose, "{out:?}");
    assert!(line(&mut Env::default(), "x -> (").prose);
}

/// Leibniz's notation, partial derivatives, higher and mixed orders; dif / (dif x) applies to one term.
#[test]
fn derivatives_in_leibniz_notation() {
    let mut env = Env::default();
    run(&mut env, "f(x, y) := x^2 y^3");
    run(&mut env, "L := 2");
    let cases = [
        ("frac(dif x^2, dif x) ->", r#""2" x"#),
        ("frac(dif (x^2 + x), dif x) + 1 ->", r#""2" x + "2""#),
        ("frac(dif^(2) x^3, dif x^(2)) ->", r#""6" x"#),
        ("frac(dif^(2), dif x^(2)) x^3 ->", r#""6" x"#),
        ("frac(∂ f(x, y), ∂ x) ->", r#""2" x dot y^("3")"#),
        ("frac(∂^(2) f(x, y), ∂ x ∂ y) ->", r#""6" x dot y^("2")"#),
        ("frac(∂^(3) f(x, y), ∂ x ∂ y^(2)) ->", r#""12" x dot y"#),
        ("frac(∂^(2), ∂ x^(2)) f(x, y) ->", r#""2" dot y^("3")"#),
        ("dif / (dif x) x^2 + x ->", r#""3" x"#),
        ("L dif / (dif x) x^2 ->", r#""4" x"#),
        ("2 dif / (dif x) x^2 ->", r#""4" x"#),
        ("frac(dif, dif x) x^3 + x ->", r#""3" dot x^("2") + x"#),
    ];
    for (source, want) in cases {
        assert_eq!(run(&mut env, source).result.as_deref(), Some(want), "{source}");
    }
    for (source, message) in [
        ("frac(dif^(2) x^3, dif x) ->", "order 2"),
        ("frac(∂ f(x, y), dif x) ->", "below a derivative"),
        ("∂ x ->", "partial derivative"),
        ("frac(dif^(0) x, dif x^(0)) ->", "positive whole number"),
    ] {
        let out = line(&mut env, source);
        assert!(out.errors.iter().any(|e| e.contains(message)), "{source}: {out:?}");
    }
}

/// 0 and ±∞ fit every unit, so the other bound gives the variable its unit; ∞ shows as ∞.
#[test]
fn integral_bounds_of_zero_and_infinity_take_the_other_unit() {
    for source in ["integral_(0)^(2 s) 3 A dif t =>", "integral_(0)^(2 s) 3 A dif t ->", "integral_(0 s)^(2 s) 3 A dif t =>"] {
        assert_eq!(run(&mut Env::default(), source).result.as_deref(), Some(r#""6" thin upright("C")"#), "{source}");
    }
    assert_eq!(run(&mut Env::default(), "integral_(2 s)^(0) 3 A dif t =>").result.as_deref(), Some(r#""−6" thin upright("C")"#));
    let mut env = Env::default();
    run(&mut env, "q(T) := integral_(0)^(T) 3 A dif t");
    assert_eq!(run(&mut env, "q(2 s) =>").result.as_deref(), Some(r#""6" thin upright("C")"#));
    assert_eq!(run(&mut Env::default(), "integral_(0 s)^(infinity) e^(-t/[1 s]) dif t =>").result.as_deref(), Some(r#""1" thin upright("s")"#));
    assert_eq!(run(&mut Env::default(), "integral_(-infinity)^(0 s) e^(t/[1 s]) dif t =>").result.as_deref(), Some(r#""1" thin upright("s")"#));
    // only 0 and ∞ adapt; with both, the variable has no unit
    let out = line(&mut Env::default(), "integral_(1)^(2 s) 3 A dif t =>");
    assert!(out.errors.iter().any(|e| e.contains("different units")), "{out:?}");
    let out = line(&mut Env::default(), "integral_(0)^(infinity) e^(-t/[1 s]) dif t =>");
    assert!(out.errors.iter().any(|e| e.contains("write it on a bound")), "{out:?}");
    let shown = run(&mut Env::default(), "integral_(0)^(infinity) e^(-t) dif t ->").result.unwrap();
    assert!(shown.contains("^(infinity)") && !shown.contains("inf\""), "{shown}");
}

/// Without bounds an integral is the antiderivative (without the constant of integration), in math and as
/// integrate(f, x).
#[test]
fn indefinite_integrals_give_the_antiderivative() {
    let cases = [
        ("∫ e^x dif x ->", r#"#svconst("euler")^(x)"#),
        ("∫ 3 dif x ->", r#""3" x"#),
        ("∫ a x dif x ->", r#"frac(a dot x^("2"), "2")"#),
        ("∫ 1/x dif x ->", "ln(abs(x))"),
        ("∫ ln(x) dif x ->", "-x + x ln(x)"),
        ("∫ tan(2 x) dif x ->", r#"-frac(ln(abs(cos("2" x))), "2")"#),
        // products multiplied out, and a polynomial times a factor integrated by parts
        ("∫ (x + 1)(x - 1) dif x ->", r#"frac(x^("3"), "3") - x"#),
        ("∫ (x^2 + 1)^2 dif x ->", r#"frac(x^("5"), "5") + frac("2" dot x^("3"), "3") + x"#),
        ("∫ x e^x dif x ->", r#"x dot #svconst("euler")^(x) - #svconst("euler")^(x)"#),
        ("∫ x^2 sin(x) dif x ->", r#"-x^("2") cos(x) + "2" x sin(x) + "2" cos(x)"#),
        ("∫ x^2 + 1 dif x ->", r#"frac(x^("3"), "3") + x"#),
        ("integrate(x^2, x) ->", r#"frac(x^("3"), "3")"#),
        ("∫ 3 A dif t ->", r#""3" t thin upright("A")"#),
    ];
    for (source, want) in cases {
        assert_eq!(run(&mut Env::default(), source).result.as_deref(), Some(want), "{source}");
    }
    // an antiderivative differentiates back to the integrand
    for integrand in ["x^3 e^(-2 x)", "(2 x + 1) cos(3 x)", "x sinh(x)", "(x - 2)^3 (x + 1)", "5^x x"] {
        let mut env = Env::default();
        run(&mut env, &format!("F(x) := ∫ {integrand} dif x"));
        run(&mut env, &format!("g(x) := {integrand}"));
        let out = run(&mut env, "dif / (dif x) F(x) - g(x) -> simplify");
        assert_eq!(out.result.as_deref(), Some(r#""0""#), "{integrand}");
    }
    let mut env = Env::default();
    run(&mut env, "F(x) := ∫ x^2 dif x");
    assert_eq!(run(&mut env, "F(3) =>").result.as_deref(), Some(r#""9""#));
    for (source, message) in [
        ("integrate(x^2, x, 1) ->", "no bounds"),
        ("∫ x ∂ x ->", "dif, not diff"),
    ] {
        let out = line(&mut Env::default(), source);
        assert!(out.errors.iter().any(|e| e.contains(message)), "{source}: {out:?}");
    }
    // a definite integral uses the antiderivative only where it's continuous and defined at the bounds
    assert_eq!(run(&mut Env::default(), "integral_1^2 ln(x) dif x ->").result.as_deref(), Some(r#""2" ln("2") - "1""#));
    assert_eq!(run(&mut Env::default(), "integral_0^1 x e^x dif x ->").result.as_deref(), Some(r#""1""#));
    for source in ["integral_0^1 ln(x) dif x ->", "integral_0^3 tan(x) dif x ->"] {
        let shown = run(&mut Env::default(), source).result.unwrap();
        assert!(shown.starts_with("integral_"), "{source}: {shown}");
    }
}

#[test]
fn unresolved_antiderivatives_stay_symbolic_and_resolve_later() {
    use solveng_core::graph::{self, GraphReq, XV};
    for integrand in ["sin(x^2)", "rect(x) * e^x", "q(x)"] {
        let mut env = Env::default();
        run(&mut env, &format!("F(x) := ∫ {integrand} dif x"));
        let shown = run(&mut env, "F(x) ->").result.unwrap();
        assert!(shown.contains("integral"), "{shown}");
        assert_eq!(run(&mut env, "F(2) - F(2) ->").result.as_deref(), Some(r#""0""#));
        if integrand != "q(x)" {
            assert!(line(&mut env, "F(2) =>").errors.iter().any(|e| e.contains("no antiderivative")));
        }
        // The fundamental theorem applies even when a primitive cannot be found.
        assert_eq!(run(&mut env, &format!("diff(F(x), x) - ({integrand}) -> simplify")).result.as_deref(), Some(r#""0""#));
        let err = graph::graph(&env, &GraphReq { axes: Default::default(),
            cfg: support::cfg(), curves: vec![support::curve("F(x)", &env)],
            xrange: vec![XV::Num(-2.0), XV::Num(2.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 20,
        }).unwrap_err();
        assert!(err.contains(if integrand == "q(x)" { "isn't defined" } else { "no antiderivative" }), "{err}");
    }
    let mut env = Env::default();
    run(&mut env, "F(u) := ∫ q(x, u) dif x");
    run(&mut env, "H(x) := F(x)");
    run(&mut env, "q(a, b) := a * b");
    assert_eq!(run(&mut env, "H(2) =>").result.as_deref(), Some(r#""4""#));
    let mut env = Env::default();
    run(&mut env, "x := 99");
    run(&mut env, "F(u) := ∫ q(x, u) dif x");
    run(&mut env, "q(a, b) := a * b");
    // The integration variable stays local; the argument x stays free.
    assert_eq!(run(&mut env, "F(2) =>").result.as_deref(), Some(r#""9801""#));
    run(&mut env, "G(x) := ∫ r(x) dif x");
    run(&mut env, "H(y) := G(y)");
    run(&mut env, "r(u) := u^2");
    assert_eq!(run(&mut env, "H(3) =>").result.as_deref(), Some(r#""9""#));
}

#[test]
fn pulse_transform_definition_defers_unsupported_primitive_to_graph() {
    use solveng_core::graph::{self, GraphReq, XV};
    let mut env = Env::default();
    run(&mut env, "p(t_0) := rect(t - t_0) + tri((t - t_0) * 2)");
    run(&mut env, "s(t) := p(1.5) + p(3.5) + p(-1.5) + p(-3.5)");
    run(&mut env, "S(f) := ∫ s(t) * e^(-#i*2*pi*f*t) dif t");
    assert!(run(&mut env, "S(f) ->").result.unwrap().contains("integral"));
    assert_eq!(run(&mut env, "S(f) - S(f) ->").result.as_deref(), Some(r#""0""#));
    let err = graph::graph(&env, &GraphReq { axes: Default::default(),
        cfg: support::cfg(), curves: vec![support::curve("S(t)", &env)],
        xrange: vec![XV::Num(-5.0), XV::Num(5.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 20,
    }).unwrap_err();
    assert!(err.contains("no antiderivative"), "{err}");
}

#[test]
fn symmetric_pulse_fourier_integral_with_bounds_is_plottable() {
    use solveng_core::graph::{self, GraphReq, XV};
    let mut env = Env::default();
    run(&mut env, "p(t_0) := rect(t - t_0) + tri((t - t_0) * 2)");
    run(&mut env, "s(t) := p(1.5) + p(3.5) + p(-1.5) + p(-3.5)");
    run(&mut env, "S(f) := integral_(-infinity)^infinity s(t) * cos(2*pi*f*t) dif t");
    run(&mut env, "area := S(0) =>");
    assert!((value(&env.vars["area"].expr).unwrap().v.f() - 6.0).abs() < 1e-9);
    for f in [-0.7, 0.2, 1.3] {
        let sinc = |x: f64| (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x);
        let expected = (sinc(f) + 0.5 * sinc(f / 2.0).powi(2))
            * 2.0 * ((3.0 * std::f64::consts::PI * f).cos() + (7.0 * std::f64::consts::PI * f).cos());
        run(&mut env, &format!("answer := S({f}) =>"));
        assert!((value(&env.vars["answer"].expr).unwrap().v.f() - expected).abs() < 1e-8);
    }
    let plot = graph::graph(&env, &GraphReq { axes: Default::default(),
        cfg: support::cfg(), curves: vec![support::curve("S(f)", &env)],
        xrange: vec![XV::Num(-5.0), XV::Num(5.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 20,
    }).unwrap();
    assert!(!plot.series.is_empty());
}

#[test]
fn closed_forms_hold_where_their_divisor_vanishes() {
    let mut env = Env::default();
    // e^(a t) / a is the integral only for a ≠ 0; at a = 0 the integrand is 1
    run(&mut env, "F(a) := integral_0^1 e^(a t) dif t ->");
    // near a = 0 the parts of (e^a − 1)/a cancel: a number put in is still exact to rounding
    for (a, expected) in [("0", 1.0), ("1", std::f64::consts::E - 1.0), ("-2", (1.0 - (-2.0f64).exp()) / 2.0), ("10^(-12)", 1.0 + 0.5e-12)] {
        run(&mut env, &format!("y := F({a}) =>"));
        assert!((value(&env.vars["y"].expr).unwrap().v.f() - expected).abs() < 1e-12, "F({a})");
    }
    // t^(n + 1)/(n + 1) except at n = −1
    run(&mut env, "G(n) := integral_1^2 t^n dif t ->");
    run(&mut env, "y := G(-1) =>");
    assert!((value(&env.vars["y"].expr).unwrap().v.f() - 2f64.ln()).abs() < 1e-12);
    assert_eq!(run(&mut env, "F(1) ->").result.as_deref(), Some("#svconst(\"euler\") - \"1\""));
    run(&mut env, "y := G(2) =>");
    assert!((value(&env.vars["y"].expr).unwrap().v.f() - 7.0 / 3.0).abs() < 1e-12);
}

#[test]
fn signal_integrals_are_exact_piecewise() {
    let mut env = Env::default();
    for (source, expected) in [
        ("integral_(-infinity)^(infinity) rect(t) dif t ->", "\"1\""),
        ("integral_(-2)^(0.25) tri(t) dif t ->", "frac(\"23\", \"32\")"),
        ("integral_(0.25)^(-2) tri(t) dif t ->", "-frac(\"23\", \"32\")"),
        ("integral_(-infinity)^(infinity) t rect(t - 1) dif t ->", "\"1\""),
    ] {
        assert_eq!(run(&mut env, source).result.as_deref(), Some(expected), "{source}");
    }
    run(&mut env, "y := integral_0^infinity rect(t) e^(-t) dif t =>");
    assert!((value(&env.vars["y"].expr).unwrap().v.f() - (1.0 - (-0.5f64).exp())).abs() < 1e-12);
    // unbounded support stays an integral (and diverges numerically)
    assert!(!line(&mut Env::default(), "integral_0^infinity sgn(t) dif t =>").errors.is_empty());
}

#[test]
fn quadrature_controls_global_error_at_endpoints_and_oscillatory_tails() {
    use solveng_core::calculus::quadrature;
    let check = |f: &dyn Fn(f64) -> f64, lo, hi, expected: f64| {
        let got = quadrature(&|t| Some(f(t)), lo, hi).unwrap();
        assert!((got - expected).abs() < 1e-9, "[{lo}, {hi}]: {got} != {expected}");
    };
    for upper in [0.2_f64, 0.75, 1.5, 12.0] {
        let expected = upper * (upper.ln() - 1.0);
        check(&|t| t.ln(), 0.0, upper, expected);
        check(&|t| t.ln(), upper, 0.0, -expected);
        check(&|t| (upper - t).ln(), 0.0, upper, expected);
    }
    check(&|t| t.ln() + (1.0 - t).ln(), 0.0, 1.0, -2.0);
    for decay in [0.25_f64, 1.0, 3.0] {
        for frequency in [8.0_f64, 28.0, 36.0, 120.0] {
            let denominator = decay * decay + frequency * frequency;
            check(&|t| (-decay*t).exp() * (frequency*t).sin(), 0.0, f64::INFINITY, frequency / denominator);
            check(&|t| (decay*t).exp() * (frequency*t).cos(), f64::NEG_INFINITY, 0.0, decay / denominator);
        }
    }
    check(&|t| (-t.abs()).exp(), f64::NEG_INFINITY, f64::INFINITY, 2.0);
    for (lo, hi) in [(f64::NAN, 1.0), (f64::INFINITY, f64::INFINITY)] {
        assert!(quadrature(&|_| Some(1.0), lo, hi).is_err());
    }
    assert!(quadrature(&|t| Some(1.0/t), 0.0, 1.0).is_err());
    assert!(quadrature(&|t| Some(t.sin()), 0.0, f64::INFINITY).is_err());
}

#[test]
fn endpoint_quadrature_preserves_complex_values_units_and_scope() {
    use solveng_core::eval::complex_value;
    let mut env = Env::default();
    run(&mut env, "t := 42");
    run(&mut env, "a := integrate((1 + #i)*ln(t), t, 0, 1)");
    let got = complex_value(&support::bind(&env, &env.vars["a"].expr)).unwrap();
    assert!((got.re.v.f() + 1.0).abs() < 1e-9);
    assert!((got.im.v.f() + 1.0).abs() < 1e-9);
    run(&mut env, "b := integrate(ln(t/[1 s]), t, [0 s], [2 s])");
    let got = value(&support::bind(&env, &env.vars["b"].expr)).unwrap();
    assert!((got.v.f() - 2.0*(2.0_f64.ln() - 1.0)).abs() < 1e-9);
    assert_eq!(got.d, solveng_core::dim::Dim::base(2));
    assert_eq!(value(&env.vars["t"].expr).unwrap().v.f(), 42.0);
}
