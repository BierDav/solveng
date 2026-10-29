mod support;
use solveng_core::{env::Env, eval::value, expr::{Expr, simp, sub}};

fn run(env: &mut Env, src: &str) {
    support::run(env, src);
}

fn equal(env: &mut Env, left: &str, right: &str) {
    run(env, &format!("actual := {left}"));
    run(env, &format!("expected := {right}"));
    let difference = solveng_core::poly::simplify(&sub(env.vars["actual"].expr.clone(), env.vars["expected"].expr.clone()));
    assert!(difference.is_zero(), "{left} != {right}: {difference:?}");
}

#[test]
fn forward_table_and_rules() {
    {
        let mut env = Env::default();
        for (input, expected) in [
            ("0", "0"), ("1", "1/s"), ("t^3 + 2*t", "6/s^4 + 2/s^2"),
            ("e^(-a*t)", "1/(s+a)"), ("exp(2*t+3)", "exp(3)/(s-2)"),
            ("sin(b*t)", "b/(s^2+b^2)"), ("cos(b*t)", "s/(s^2+b^2)"),
            ("sinh(b*t)", "b/(s^2-b^2)"), ("cosh(b*t)", "s/(s^2-b^2)"),
            ("t*exp(-a*t)", "1/(s+a)^2"),
            ("exp(-a*t)*sin(b*t)", "b/((s+a)^2+b^2)"),
            ("t*sin(b*t)", "2*b*s/(s^2+b^2)^2"),
        ] { equal(&mut env, &format!("laplace({input})"), expected); }
    }
}

#[test]
fn inverse_table_and_round_trips() {
    {
        let mut env = Env::default();
        for (input, expected) in [
            ("1/s", "1"), ("6/s^4", "t^3"), ("1/(s+a)", "exp(-a*t)"),
            ("1/(2*s+4)^3", "t^2*exp(-2*t)/16"),
            ("1/(s^2+4)", "sin(2*t)/2"), ("s/(s^2+4)", "cos(2*t)"),
            ("1/(s^2-4)", "sinh(2*t)/2"),
            ("1/(s^2-b^2)", "cases(sinh(b*t)/b,-b^2 != 0,t)"),
            ("1/(s^2*(s+1))", "t-1+exp(-t)"),
            ("1/(s^2+4*s+4)", "t*exp(-2*t)"),
            ("(s+3)/(s^2+2*s+5)", "exp(-t)*(cos(2*t)+sin(2*t))"),
            ("1/(s*(s+1)*(s+2))", "1/2-exp(-t)+exp(-2*t)/2"),
        ] { equal(&mut env, &format!("invlaplace({input})"), expected); }
        for input in ["1", "t^3+2*t", "exp(-2*t)", "sin(3*t)", "cos(3*t)", "t*exp(-2*t)", "exp(-2*t)*sin(3*t)"] {
            equal(&mut env, &format!("laplace(invlaplace(laplace({input})))"), &format!("laplace({input})"));
        }
    }
}

#[test]
fn definitions_custom_variables_and_local_input() {
    {
        let mut env = Env::default();
        run(&mut env, "t := 7");
        run(&mut env, "F := laplace(t^2, t, s) ->");
        equal(&mut env, "F", "2/s^3");
        run(&mut env, "g(x) := exp(-x)");
        run(&mut env, "G := laplace(g(x), x, p)");
        equal(&mut env, "G", "1/(p+1)");
        run(&mut env, "f(u) := invlaplace(1/(p+1), p, u)");
        run(&mut env, "answer := f(2) =>");
        assert!((value(&env.vars["answer"].expr).unwrap().v.f() - (-2.0_f64).exp()).abs() < 1e-12);
        assert_eq!(value(&env.vars["t"].expr).unwrap().v.f(), 7.0);
        equal(&mut env, "laplace(x^2, x, p)", "2/p^3");
    }
}

#[test]
fn invalid_and_unsupported_transforms_are_errors() {
    for src in ["laplace() ->", "laplace(t, 2) ->", "laplace(t, t, t) ->", "laplace(s*t) ->", "laplace(ln(t)) ->", "invlaplace(1) ->", "invlaplace(s) ->", "laplace(t^40) ->"] {
        let out = support::line(&mut Env::default(), src);
        assert!(!out.errors.is_empty(), "{src}: {out:?}");
    }
    // A transform result can be evaluated as an ordinary worksheet function.
    let mut env = Env::default();
    run(&mut env, "F(p) := laplace(t^2, t, p)");
    run(&mut env, "answer := F(2) =>");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 0.25);
    assert!(matches!(simp(&env.vars["answer"].expr), Expr::C(_)));
}

#[test]
fn transforms_match_the_defining_integral() {
    use solveng_core::{calculus::quadrature, eval::eval_f};
    for source in ["sin(2*t+1)", "cos(2*t+1)", "sinh(2*t+1)", "cosh(2*t+1)", "t^2*exp(-t)*sin(2*t)"] {
        let mut env = Env::default();
        run(&mut env, &format!("original := {source}"));
        run(&mut env, &format!("transformed := laplace({source})"));
        let expected = quadrature(&|t| {
            let value = eval_f(&env.vars["original"].expr, &|n| (n == "t").then_some(t))?;
            Some(value * (-5.0 * t).exp())
        }, 0.0, 30.0).unwrap();
        let actual = eval_f(&env.vars["transformed"].expr, &|n| (n == "s").then_some(5.0)).unwrap();
        assert!((actual - expected).abs() < 1e-9, "{source}: {actual} != {expected}");
    }
}


#[test]
fn signal_transforms_and_transfer_function() {
    {
        let mut env = Env::default();
        for (input, expected) in [
            ("rect(t)", "(1-exp(-s/2))/s"),
            ("rect(-2*t)", "(1-exp(-s/4))/s"),
            ("rect(t-1)", "(exp(-s/2)-exp(-3*s/2))/s"),
            ("rect(t+1)", "0"),
            ("tri(2*t)", "1/s-2*(1-exp(-s/2))/s^2"),
            ("tri(-2*t)", "1/s-2*(1-exp(-s/2))/s^2"),
            ("tri(t-1)", "(1-2*exp(-s)+exp(-2*s))/s^2"),
            ("tri(t+1)", "0"),
            ("sgn(t-1)", "(2*exp(-s)-1)/s"),
            ("sgn(1-t)", "(1-2*exp(-s))/s"),
            ("rect(a*t)", "cases(cases((1-exp(-s/(2*abs(a))))/s,abs(a) != 0,1/s),Im(a)=0)"),
            ("tri(a*t)", "cases(cases(1/s-abs(a)*(1-exp(-s/abs(a)))/s^2,abs(a) != 0,1/s),Im(a)=0)"),
            ("rect(0*t)", "1/s"),
        ] { equal(&mut env, &format!("laplace({input})"), expected); }
        run(&mut env, "Lambda(t) := tri(t)");
        run(&mut env, "s(t) := rect(t)");
        run(&mut env, "g(t) := Lambda(2*t)");
        run(&mut env, "H(s) := laplace(s(t), t, s) / laplace(g(t), t, s) ->");
        equal(&mut env, "H(p)", "p*(1-exp(-p/2))/(p-2+2*exp(-p/2))");
        run(&mut env, "answer := H(2) =>");
        assert!((value(&env.vars["answer"].expr).unwrap().v.f() - (std::f64::consts::E-1.0)).abs() < 1e-12);
    }
}


#[test]
fn shifted_signals_match_integrals_split_at_their_corners() {
    use solveng_core::{calculus::{quadrature, signal_points}, eval::eval_f};
    for signal in ["rect", "tri", "sgn"] {
        for slope in [-2, 3] {
            for offset in [-2, -1, 0, 1, 2] {
                let source = format!("{signal}({slope}*t+({offset}))");
                let mut env = Env::default();
                run(&mut env, &format!("original := {source}"));
                run(&mut env, &format!("transformed := laplace({source})"));
                let original = &env.vars["original"].expr;
                let mut points = signal_points(original, "t", &|_| None);
                points.retain(|t| *t > 0.0 && *t < 30.0);
                points.extend([0.0, 30.0]);
                points.sort_by(f64::total_cmp);
                points.dedup();
                let expected: f64 = points.windows(2).map(|bounds| quadrature(&|t| {
                    Some(eval_f(original, &|n| (n == "t").then_some(t))? * (-5.0*t).exp())
                }, bounds[0], bounds[1]).unwrap()).sum();
                let actual = eval_f(&env.vars["transformed"].expr, &|n| (n == "s").then_some(5.0)).unwrap();
                assert!((actual-expected).abs() < 1e-9, "{source}: {actual} != {expected}");
            }
        }
    }
}

#[test]
fn delayed_feedback_step_response() {
    use solveng_core::eval::eval_f;
    {
        let mut env = Env::default();
        run(&mut env, "s(t) := rect(t)");
        run(&mut env, "g(t) := tri(2*t)");
        run(&mut env, "H(s) := laplace(s(t), t, s) / laplace(g(t), t, s) ->");
        run(&mut env, "s_2(t) := invlaplace(H(s)/s, s, t) ->");
        for t in [0.0_f64, 0.1, 0.49, 0.5, 0.6, 0.99, 1.0, 1.1, 1.5] {
            let base = |u: f64| {
                if u < 0.0 { return 0.0; }
                (0..=(2.0*u).floor() as u32).map(|k| {
                    let v = u - k as f64 / 2.0;
                    (-2.0_f64).powi(k as i32) * v.powi(k as i32) * (2.0*v).exp()
                        / (1..=k).fold(1.0, |a, b| a * b as f64)
                }).sum::<f64>()
            };
            run(&mut env, &format!("actual := s_2({t}) =>"));
            let actual = value(&env.vars["actual"].expr).unwrap().v.f();
            let expected = base(t) - base(t-0.5);
            assert!((actual-expected).abs() < 1e-10, "t={t}: {actual} != {expected}");
        }
        // Independent method-of-steps solution on the first two intervals.
        let body = &env.funs["s_2"].body;
        for t in [0.1_f64, 0.4, 0.6, 0.9] {
            let actual = eval_f(body, &|n| (n == "t").then_some(t)).unwrap();
            let expected = if t < 0.5 { (2.0*t).exp() }
                else { (2.0*t).exp() - 2.0*t*(2.0*t-1.0).exp() };
            assert!((actual-expected).abs() < 1e-10);
        }
    }
}

#[test]
fn delay_series_scaling_scoping_and_numeric_helpers() {
    use solveng_core::eval::eval_f;
    {
        let mut env = Env::default();
        equal(&mut env, "floor(-3/2)", "-2");
        equal(&mut env, "ceil(-3/2)", "-1");
        equal(&mut env, "ceil(7/2)", "4");
        equal(&mut env, "ceil(3)", "3");
        equal(&mut env, "factorial(0)+factorial(5)", "121");
        // The output name must not be captured by the generated summation index.
        run(&mut env, "response(k) := invlaplace((1+exp(-2*p))/(2*p+4+3*exp(-p)), p, k)");
        for t in [0.0_f64, 0.4, 1.0, 1.4, 2.0, 2.4] {
            let kernel = |u: f64| {
                if u < 0.0 { return 0.0; }
                let mut result = (-2.0*u).exp()/2.0;
                if u >= 1.0 { result -= 0.75*(u-1.0)*(-2.0*(u-1.0)).exp(); }
                if u >= 2.0 { result += 0.5625*(u-2.0).powi(2)*(-2.0*(u-2.0)).exp(); }
                result
            };
            let actual = eval_f(&env.funs["response"].body, &|n| (n == "k").then_some(t)).unwrap();
            assert!((actual-kernel(t)-kernel(t-2.0)).abs() < 1e-12);
        }
        for unsupported in ["invlaplace(1/(s+exp(s))) ->", "factorial(-1) =>", "factorial(1/2) =>"] {
            let out = support::line(&mut env, unsupported);
            assert!(!out.errors.is_empty(), "{unsupported}");
        }
    }
}

#[test]
fn corrected_lti_step_response_is_periodic_triangle() {
    use solveng_core::eval::eval_f;
    {
        let mut env = Env::default();
        run(&mut env, "x(t) := rect(t-1/2)");
        run(&mut env, "y(t) := tri(2*(t-1/2))");
        run(&mut env, "H(s) := laplace(y(t), t, s)/laplace(x(t), t, s) ->");
        run(&mut env, "response(t) := invlaplace(H(s)/s, s, t) ->");
        // Check many periods, both sides of corners, and causal zero values.
        for i in -4..=200 {
            let t = i as f64 / 20.0;
            let expected = if t < 0.0 { 0.0 } else { 1.0-(2.0*t.rem_euclid(1.0)-1.0).abs() };
            let actual = eval_f(&env.funs["response"].body, &|n| (n == "t").then_some(t)).unwrap();
            assert!((actual-expected).abs() < 1e-10, "t={t}: {actual} != {expected}");
        }
        assert!(!env.funs["response"].body.contains(&|e| matches!(e, Expr::InverseLaplace(..))));
        run(&mut env, "alternative(t) := invlaplace(2*tanh(s/4)/s^2)");
        for t in [0.0_f64, 0.25, 0.5, 0.75, 1.0, 2.3, 4.8] {
            let actual = eval_f(&env.funs["alternative"].body, &|n| (n == "t").then_some(t)).unwrap();
            let expected = 1.0-(2.0*t.rem_euclid(1.0)-1.0).abs();
            assert!((actual-expected).abs() < 1e-10);
        }
    }
}

#[test]
fn rational_inverse_uses_state_space_for_repeated_and_complex_poles() {
    use solveng_core::eval::eval_f;
    for (source, expected) in [
        ("1/((s^2+1)*(s^2+4))", "sin(t)/3-sin(2*t)/6"),
        ("s/((s^2+1)*(s^2+4))", "(cos(t)-cos(2*t))/3"),
        ("1/(s^2+2*s+5)^2", "exp(-t)*(sin(2*t)-2*t*cos(2*t))/16"),
    ] {
        let mut env = Env::default();
        run(&mut env, &format!("actual := invlaplace({source})"));
        run(&mut env, &format!("expected := {expected}"));
        for t in [0.0_f64, 0.01, 0.3, 1.2, 3.1, 8.0] {
            let actual = eval_f(&env.vars["actual"].expr, &|n| (n == "t").then_some(t)).unwrap();
            let expected = eval_f(&env.vars["expected"].expr, &|n| (n == "t").then_some(t)).unwrap();
            assert!((actual-expected).abs() < 1e-10, "{source}, t={t}: {actual} != {expected}");
        }
    }
}

#[test]
fn general_delay_denominators_use_power_series_division() {
    use solveng_core::eval::eval_f;
    let mut env = Env::default();
    for (source, expected) in [
        ("1/(s^2+exp(-s))", "t-cases((t-1)^3/6, t>=1, 0)+cases((t-2)^5/120, t>=2, 0)-cases((t-3)^7/5040, t>=3, 0)"),
        ("1/(s+exp(-s)+exp(-2*s))", "1-cases(t-1, t>=1, 0)+cases((t-2)^2/2-(t-2), t>=2, 0)+cases((t-3)^2-(t-3)^3/6, t>=3, 0)"),
        ("exp(-s/3)/(s+1)", "cases(exp(-(t-1/3)), t>=1/3, 0)"),
        ("1/(s*(1-exp(-s)+exp(-2*s)))", "cases(1, t<1, 2, t<3, 1)"),
    ] {
        run(&mut env, &format!("actual := invlaplace({source})"));
        run(&mut env, &format!("expected := {expected}"));
        for t in [0.0_f64, 0.2, 0.9, 1.0, 1.7, 2.0, 2.6, 3.0, 3.8] {
            let actual = eval_f(&env.vars["actual"].expr, &|n| (n == "t").then_some(t)).unwrap();
            let expected = eval_f(&env.vars["expected"].expr, &|n| (n == "t").then_some(t)).unwrap();
            assert!((actual-expected).abs() < 1e-9, "{source}, t={t}: {actual} != {expected}");
        }
    }
}

#[test]
fn algorithmic_inverses_match_the_laplace_defining_integral() {
    use solveng_core::{calculus::quadrature, eval::eval_f};
    for source in [
        "1/((s^2+1)*(s^2+4))",
        "1/(s^2+exp(-s))",
        "(2*s+1+exp(-s))/(s^2+2*s+5+(s+1)*exp(-s)+exp(-2*s))",
    ] {
        let mut env = Env::default();
        run(&mut env, &format!("original := {source}"));
        run(&mut env, &format!("response := invlaplace({source})"));
        let transformed = (0..8).map(|i| quadrature(&|t| {
            Some(eval_f(&env.vars["response"].expr, &|n| (n == "t").then_some(t))? * (-4.0*t).exp())
        }, i as f64, i as f64+1.0).unwrap()).sum::<f64>();
        let expected = eval_f(&env.vars["original"].expr, &|n| (n == "s").then_some(4.0)).unwrap();
        assert!((transformed-expected).abs() < 1e-9, "{source}: {transformed} != {expected}");
        equal(&mut env, &format!("laplace(invlaplace({source}))"), source);
    }
}

#[test]
fn algorithmic_inverse_scopes_survive_binding_substitution_and_serialization() {
    {
        let mut env = Env::default();
        run(&mut env, "s := 7");
        run(&mut env, "response(x) := invlaplace(1/((p^2+1)*(p^2+a)), p, x)");
        run(&mut env, "a := 4");
        run(&mut env, "actual := response(1) =>");
        let expected = 1.0_f64.sin()/3.0-2.0_f64.sin()/6.0;
        assert!((value(&env.vars["actual"].expr).unwrap().v.f()-expected).abs() < 1e-12);
        // Substitution introduces the same spelling as the bound transform variable.
        run(&mut env, "identity(p) := response(p)");
        run(&mut env, "actual := identity(1) =>");
        assert!((value(&env.vars["actual"].expr).unwrap().v.f()-expected).abs() < 1e-12);
        let mut bytes = vec![];
        ciborium::into_writer(&env, &mut bytes).unwrap();
        let mut restored: Env = ciborium::from_reader(bytes.as_slice()).unwrap();
        run(&mut restored, "actual := response(1) =>");
        assert!((value(&restored.vars["actual"].expr).unwrap().v.f()-expected).abs() < 1e-12);
    }
}

#[test]
fn generating_functions_support_distinct_and_repeated_delay_poles() {
    use solveng_core::eval::eval_f;
    let mut env = Env::default();
    for (source, expected) in [
        ("1/(s*(1-exp(-s))^2)", "(floor(t)+1)*(floor(t)+2)/2"),
        ("1/(s*(1-exp(-s))*(1-2*exp(-s)))", "2^(floor(t)+2)-floor(t)-3"),
    ] {
        run(&mut env, &format!("actual := invlaplace({source})"));
        run(&mut env, &format!("expected := {expected}"));
        assert!(!env.vars["actual"].expr.contains(&|e| matches!(e, Expr::InverseLaplace(..))));
        for t in [0.0_f64, 0.8, 1.0, 1.4, 2.5, 3.0, 8.7] {
            let actual = eval_f(&env.vars["actual"].expr, &|n| (n == "t").then_some(t)).unwrap();
            let expected = eval_f(&env.vars["expected"].expr, &|n| (n == "t").then_some(t)).unwrap();
            assert!((actual-expected).abs() < 1e-10, "{source}, t={t}");
        }
    }
    run(&mut env, "actual := invlaplace(1/(s+1)^40)");
    let t = 10.0_f64;
    let actual = eval_f(&env.vars["actual"].expr, &|n| (n == "t").then_some(t)).unwrap();
    let expected = t.powi(39)*(-t).exp()/(1..=39).fold(1.0, |a, b| a*b as f64);
    assert!((actual/expected-1.0).abs() < 1e-12);
}

#[test]
fn commensurate_delays_are_normalized_to_their_common_step() {
    use solveng_core::eval::eval_f;
    let mut env = Env::default();
    run(&mut env, "response := invlaplace(1/(s+exp(-s/2)+exp(-s/3)))");
    for t in [0.0_f64, 0.2, 0.5, 0.75, 1.1, 1.6, 2.0] {
        let mut expected = 0.0;
        for j in 0..=4_i32 { for k in 0..=6_i32 {
            let shifted = t-j as f64/2.0-k as f64/3.0;
            if shifted >= 0.0 {
                let factorial = |n| (1..=n).fold(1.0, |a, b| a*b as f64);
                expected += (-1.0_f64).powi(j+k)*shifted.powi(j+k)/(factorial(j)*factorial(k));
            }
        } }
        let actual = eval_f(&env.vars["response"].expr, &|n| (n == "t").then_some(t)).unwrap();
        assert!((actual-expected).abs() < 1e-10, "t={t}: {actual} != {expected}");
    }
}
