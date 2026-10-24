mod support;
use solveng_core::{env::Env, eval::value};
use support::run;

#[test]
fn finite_sums_are_exact_and_indices_are_local() {
    {
        let mut env = Env::default();
        run(&mut env, "k := 42");
        for (source, expected) in [
            ("sum_(k=0)^3 k^2", 14.0),
            ("sum(k^2, k, 0, 3)", 14.0),
            (r#"sum_("counter"=0)^3 "counter"^2"#, 14.0),
            ("sum_(k=3)^2 k", 0.0),
            ("sum_(k=-2)^2 k", 0.0),
            ("sum_(k=1)^3 (k+1) + 5", 14.0),
            ("sum_(k=1)^2 (sum_(j=1)^k j)", 4.0),
        ] {
            run(&mut env, &format!("answer := {source} ="));
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected, "{source}");
        }
        assert_eq!(value(&env.vars["k"].expr).unwrap().v.f(), 42.0);
        run(&mut env, "answer := sum_(k=1)^3 1/k ->");
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v, solveng_core::num::Num::R(solveng_core::num::Rat::new(11, 6).unwrap()));
    }
}

#[test]
fn infinite_signal_sums_resolve_late_and_plot() {
    use solveng_core::graph::{self, GraphReq, XV};
    {
        let mut env = Env::default();
        let out = run(&mut env, "s_2(t) := sum_(k=0)^infinity sigma(t-k+3/4) ->");
        assert!(out.result.unwrap().contains("sum_"));
        assert_eq!(env.funs["s_2"].body.syms(), ["t"]);
        run(&mut env, "sigma(t) := cases(0, t < 0, 1, t >= 0)");
        for (t, expected) in [(-2.0, 0.0), (-0.750001, 0.0), (-0.75, 1.0), (0.0, 1.0), (0.25, 2.0), (2.0, 3.0), (200.0, 201.0), (1_000_000.0, 1_000_001.0)] {
            run(&mut env, &format!("answer := s_2({t}) ="));
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected);
        }
        let plot = graph::graph(&env, &GraphReq {
            cfg: support::cfg(), curves: vec![support::curve("s_2(t)", &env)],
            xrange: vec![XV::Num(-2.0), XV::Num(3.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 40,
        }).unwrap();
        let points: Vec<_> = plot.series.iter().flat_map(|s| &s.points).collect();
        assert!(points.len() >= 40);
        for (t, y) in points {
            assert_eq!(*y, (t + 0.75).floor().max(-1.0) + 1.0);
        }
    }
}

#[test]
fn numerical_sums_validate_bounds_units_and_convergence() {
    let mut env = Env::default();
    run(&mut env, "answer := sum_(k=0)^infinity (1/2)^k =");
    assert!((value(&env.vars["answer"].expr).unwrap().v.f() - 2.0).abs() < 1e-10);
    for (expression, expected) in [
        ("sum_(k=0)^infinity rect(k-200000)", 1.0),
        ("sum_(k=0)^infinity rect(k-0.5)", 1.0),
        ("sum_(k=0)^1000 (k < 500)", 500.0),
        ("sum_(k=0)^1000 (k <= 500)", 501.0),
    ] {
        run(&mut env, &format!("answer := {expression} ="));
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected);
    }
    run(&mut env, "answer := sum_(k=1)^3 [2 m] =");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 6.0);
    for source in ["sum_(k=0)^infinity k =", "sum_(k=0)^infinity 1 =", "sum_(k=0)^infinity 10^(-20) =", "sum_(k=0)^infinity (-1)^k =", "sum_(k=0.5)^4 k =", "sum(k, k, [1 m], 4) ="] {
        let out = support::line(&mut env, source);
        assert!(!out.errors.is_empty(), "{source}: {out:?}");
    }
}

#[test]
fn substitution_avoids_capturing_sum_index() {
    use solveng_core::expr::{Expr, int, mul, sym};
    let sum = Expr::Sum(Box::new(mul(vec![sym("t"), sym("k")])), "k".into(), Box::new(int(1)), Box::new(int(3)));
    let replaced = sum.subst("t", &sym("k"));
    assert_eq!(replaced.syms(), ["k"]);
    assert_eq!(value(&replaced.subst("k", &int(2))).unwrap().v.f(), 12.0);
}

#[test]
fn plotting_an_undefined_function_explains_the_definition_marker() {
    use solveng_core::graph::{self, GraphReq};
    let mut env = Env::default();
    run(&mut env, "sigma(t) := cases(0, t < 0, 1, t >= 0)");
    // An equality leaves g_2 undefined, even when its right side is a valid sum.
    let _ = support::line(&mut env, "g_2(t) = sum_(k=0)^infinity sigma(t-k+3/4) ->");
    let error = graph::graph(&env, &GraphReq {
        cfg: support::cfg(), curves: vec![support::curve("g_2(t)", &env)],
        xrange: vec![], yrange: vec![], marks: vec![], ymark: vec![], samples: 40,
    }).unwrap_err();
    assert!(error.contains("function 'g_2' isn't defined"), "{error}");
    assert!(error.contains("g_2(t) :="), "{error}");
}

#[test]
fn triangular_signal_sums_skip_zero_prefixes_and_sum_linear_intervals() {
    use solveng_core::{graph::{self, GraphReq, XV}};
    {
        let mut env = Env::default();
        let out = support::line_raw(&mut env, "g_2(t) := sum_(k=0)^infinity ⦇tri⦈(t-k+1/2) ->");
        assert!(out.errors.is_empty(), "{out:?}");
        assert!(out.result.unwrap().contains("sum_"));
        for (t, expected) in [("-2", 0.0), ("-3/2", 0.0), ("-1", 0.5), ("-1/2", 1.0), ("0", 1.0), ("1000000", 1.0)] {
            run(&mut env, &format!("answer := g_2({t}) ="));
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected);
        }
        let plot = graph::graph(&env, &GraphReq {
            cfg: support::cfg(), curves: vec![support::curve("g_2(t)", &env)],
            xrange: vec![XV::Num(1_000_000.0), XV::Num(1_000_003.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 40,
        }).unwrap();
        assert!(plot.series.iter().flat_map(|s| &s.points).count() >= 40);
        assert!(plot.series.iter().flat_map(|s| &s.points).all(|(_, y)| (*y - 1.0).abs() < 1e-10));
    }
    let mut env = Env::default();
    for (source, expected) in [
        ("sum_(k=0)^infinity tri((k-1000000)/1000)", 1000.0),
        ("sum_(k=0)^1000000 k", 500000500000.0),
        ("sum_(k=0)^1000 (k * (k < 500))", 124750.0),
        ("sum_(k=0)^1000 cases(k, k < 500, 0)", 124750.0),
        ("sum_(k=0)^1000 (tri((k-500)/500) * tri((k-500)/500))", 333.334),
        // Products of varying linear pieces must still use the general evaluator.
        ("sum_(k=0)^1000 tri((k-500)/500)^2", 333.334),
    ] {
        run(&mut env, &format!("answer := {source} ="));
        assert!((value(&env.vars["answer"].expr).unwrap().v.f() - expected).abs() < 1e-8, "{source}");
    }
}

#[test]
fn linear_signal_series_work_is_bounded_by_corners() {
    let calls = std::cell::Cell::new(0);
    let result = solveng_core::calculus::series(&|k| {
        calls.set(calls.get() + 1);
        Some((1.0 - ((k - 1_000_000.0) / 1000.0).abs()).max(0.0))
    }, 0.0, f64::INFINITY, &[999_000.0, 1_000_000.0, 1_001_000.0], true).unwrap();
    assert!((result - 1000.0).abs() < 1e-10);
    assert!(calls.get() <= 14, "{} term evaluations", calls.get());
}

#[test]
fn triangular_sum_graph_accepts_a_different_parameter_on_a_flat_range() {
    use solveng_core::graph::{self, GraphReq, XV};
    let mut env = Env::default();
    run(&mut env, "g_2(t) := sum_(k=0)^infinity tri(t-k+1/2) ->");
    let plot = graph::graph(&env, &GraphReq {
        cfg: support::cfg(), curves: vec![support::curve("g_2(x)", &env)],
        xrange: vec![XV::Num(0.0), XV::Num(1.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 400,
    }).unwrap();
    assert!(plot.series.iter().flat_map(|s| &s.points).count() >= 400);
    assert!(plot.series.iter().flat_map(|s| &s.points).all(|(_, y)| (*y - 1.0).abs() < 1e-10));
    assert!(plot.y0 < 1.0 && plot.y1 > 1.0);
    assert!(plot.y1 - plot.y0 >= 1.0);
    assert!((2..=20).contains(&plot.yticks.len()));
}
