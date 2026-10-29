mod support;
use solveng_core::{Worksheet, env::Env, graph::{self, AxisOptions, GraphReq, XV}};

#[test]
fn result_arrows_leave_equality_available_inside_expressions() {
    let mut env = Env::default();
    for marker in ["=>", "⇒"] {
        assert_eq!(support::result(&mut env, &format!("1 = 2 {marker}")), "\"0\"");
        let out = support::line(&mut env, &format!("2 + 3 {marker}"));
        assert_eq!(out.mark.as_deref(), Some("=>"));
        assert_eq!(out.result.as_deref(), Some("\"5\""));
        assert!(!out.prose && out.errors.is_empty());
    }
}

#[test]
fn solve_is_read_only_and_explicit_assignment_keeps_all_branches() {
    let mut env = Env::default();
    support::run(&mut env, "x := 10");
    let out = support::run(&mut env, "x^2 = 4 -> solve(x)");
    assert!(out.defined.is_empty());
    assert_eq!(out.solutions.len(), 2);
    assert_eq!(support::result(&mut env, "x =>"), "\"10\"");
    support::run(&mut env, "roots := x^2 = 4 -> solve(x)");
    assert_eq!(support::result(&mut env, "at(roots, 0) =>"), "\"−2\"");
    assert_eq!(support::result(&mut env, "at(roots, 1) =>"), "\"2\"");
}

#[test]
fn canonical_probability_operations_and_deprecated_aliases_agree() {
    assert_eq!(solveng_core::builtins::canonical("normalcdf"), "pnorm");
    assert_eq!(solveng_core::builtins::canonical("normalquantile"), "qnorm");
    let mut env = Env::default();
    for name in ["pnorm", "normalcdf"] {
        assert_eq!(support::result(&mut env, &format!("{name}(0, 0, 1) =>")), "\"0.5\"");
    }
}

#[test]
fn explicit_graph_variable_shadows_only_locally_and_bounds_stay_exact() {
    let mut env = Env::default();
    support::run(&mut env, "x := 100");
    let request = GraphReq {
        axes: AxisOptions { variable: Some("x".into()), include: vec![XV::Num(-4.0)], ..Default::default() },
        cfg: support::cfg(), curves: vec![support::curve("x^2", &env)],
        xrange: vec![], yrange: vec![], marks: vec![], ymark: vec![], samples: 30,
    };
    let plot = graph::graph(&env, &request).unwrap();
    assert!(plot.x0 <= -4.0);
    assert_eq!(support::result(&mut env, "x =>"), "\"100\"");
    let mut bounded = request;
    bounded.axes.include.clear();
    bounded.xrange = vec![XV::Num(-1.0), XV::Num(1.0)];
    bounded.marks = vec![(graph::Mark::Val(XV::Num(5.0)), true)];
    let plot = graph::graph(&env, &bounded).unwrap();
    assert_eq!((plot.x0, plot.x1), (-1.0, 1.0));
}

#[test]
fn worksheet_facade_reports_diagnostics_and_keeps_state() {
    let mut worksheet = Worksheet::default();
    assert!(worksheet.evaluate("x := 3").errors.is_empty());
    assert_eq!(worksheet.value("x + 2").unwrap(), "\"5\"");
    assert_eq!(worksheet.definitions().unwrap()[0].name, "x");
    assert!(!worksheet.value("unknown + 1").unwrap_err().message.is_empty());
    worksheet.reset();
    assert!(worksheet.definitions().unwrap().is_empty());
}
