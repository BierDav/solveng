mod support;
use solveng_core::{env::Env, eval::value};
use support::run;

#[test]
fn signal_functions_and_their_boundaries() {
    {
        let mut env = Env::default();
        for (expression, expected) in [
            ("rect(0)", 1.0), ("rect(0.5)", 0.5), ("rect(-0.5)", 0.5), ("rect(0.50000000001)", 0.0), ("rect(1)", 0.0),
            ("tri(0)", 1.0), ("tri(-0.5)", 0.5), ("tri(1)", 0.0), ("tri(2)", 0.0),
            ("sgn(-2)", -1.0), ("sgn(0)", 0.0), ("sgn(2)", 1.0), ("rect(0.25) * tri(0.25) * sgn(0.25)", 0.75),
        ] {
            run(&mut env, &format!("answer := {expression} ="));
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected, "{expression}");
        }
        let out = run(&mut env, "rect(t) + 2 rect(t) ->");
        assert!(out.result.unwrap().contains("rect(t)"));
        run(&mut env, "a*rect(0) + b*tri(0) = 5; a*rect(1) + b*tri(0.5) = 1 -> solve(a,b)");
        assert_eq!(value(&env.vars["a"].expr).unwrap().v.f(), 3.0);
        assert_eq!(value(&env.vars["b"].expr).unwrap().v.f(), 2.0);
    }
}

#[test]
fn signal_integrals_and_derivatives_keep_their_meaning() {
    let mut env = Env::default();
    for expression in ["integrate(rect(t), t, -2, 2)", "integrate(tri(t), t, -2, 2)"] {
        run(&mut env, &format!("answer := {expression} ="));
        assert!((value(&env.vars["answer"].expr).unwrap().v.f() - 1.0).abs() < 1e-9);
    }
    let result = run(&mut env, "diff(sgn(t), t) ->").result.unwrap();
    assert!(result.contains("dif"), "{result}");
    let out = support::line(&mut env, "rect([1 m]) =");
    assert!(out.errors.iter().any(|e| e.contains("unitless")));
}

#[test]
fn triangular_derivatives_plot_without_assigning_values_at_corners() {
    use solveng_core::{expr::{Func, func, sym}, eval::eval_f, graph::{self, GraphReq, XV}};
    {
        let env = Env::default();
        for source in ["dif / (dif t) tri(t)", "diff(tri(t), t)"] {
            let plot = graph::graph(&env, &GraphReq {
                cfg: support::cfg(), curves: vec![support::curve(source, &env)],
                xrange: vec![XV::Num(-2.0), XV::Num(2.0)], yrange: vec![],
                marks: vec![], ymark: vec![], samples: 80,
            }).unwrap();
            assert!(plot.series.len() >= 4);
            for series in &plot.series {
                for &(t, y) in &series.points {
                    assert!(![-1.0, 0.0, 1.0].contains(&t));
                    let expected = if t.abs() > 1.0 { 0.0 } else if t < 0.0 { 1.0 } else { -1.0 };
                    assert_eq!(y, expected);
                }
                for pair in series.points.windows(2) { assert_eq!(pair[0].1, pair[1].1); }
            }
        }
        // Check shifted, reversed and nonlinear arguments against numerical slopes away from corners.
        for body in ["tri(2*t-1)", "tri(-3*t)", "tri(t^2)"] {
            let mut local = Env::default();
            run(&mut local, &format!("p(t) := {body}"));
            let pulse = &local.funs["p"].body;
            let derivative = pulse.diff("t");
            for t in [-1.5_f64, -0.7, -0.2, 0.2, 0.7, 1.5] {
                let at = |e: &solveng_core::expr::Expr, t: f64| eval_f(e, &|n| (n == "t").then_some(t)).unwrap();
                let slope = (at(pulse, t + 1e-6) - at(pulse, t - 1e-6)) / 2e-6;
                assert!((at(&derivative, t) - slope).abs() < 1e-8, "{body} at {t}");
            }
        }
    }
    let derivative = func(Func::Tri, sym("t")).diff("t");
    for corner in [-1.0, 0.0, 1.0] {
        assert!(eval_f(&derivative, &|_| Some(corner)).is_none());
    }
    let smooth = func(Func::Tri, solveng_core::expr::pow(sym("t"), solveng_core::expr::int(2))).diff("t");
    assert_eq!(eval_f(&smooth, &|_| Some(0.0)), Some(0.0));
    // Jumps and higher distributional derivatives must continue to stay symbolic.
    assert!(matches!(derivative.diff("t"), solveng_core::expr::Expr::Derivative(..)));
    for f in [Func::Rect, Func::Sgn] {
        assert!(matches!(func(f, sym("t")).diff("t"), solveng_core::expr::Expr::Derivative(..)));
    }
}

#[test]
fn shifted_and_narrow_pulses_integrate_over_infinite_bounds() {
    let mut env = Env::default();
    for (expression, expected) in [
        ("integrate(rect((t - 12.3)/0.01), t, -infinity, infinity)", 0.01),
        ("integrate(tri(t - 0.3), t, -infinity, infinity)", 1.0),
        ("integrate(2*rect(t - 0.3) + 3*tri(t + 2.7), t, -infinity, infinity)", 5.0),
    ] {
        run(&mut env, &format!("answer := {expression} ="));
        let actual = value(&env.vars["answer"].expr).unwrap().v.f();
        assert!((actual - expected).abs() < 1e-9, "{expression}: {actual}");
    }
}
