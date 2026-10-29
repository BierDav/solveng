mod support;
use solveng_core::{api, env::Env, eval::value};
use support::run;

#[test]
fn simplification_descends_into_builtin_and_unspecified_function_arguments() {
    let mut env = Env::default();
    for function in ["tri", "sin", "unknown"] {
        let out = run(&mut env, &format!("{function}(2(t-1/2)) ->"));
        let result = out.result.unwrap();
        assert!(result.contains("(\"2\" t - \"1\")"), "{result}");
    }
    let out = run(&mut env, "sin((t^2-1)/(t-1)) ->");
    assert_eq!(out.result.as_deref(), Some("sin(t + \"1\")"));
}

#[test]
fn standard_operations_and_user_functions_work_without_configuration() {
    use solveng_core::{graph::{self, GraphReq, XV}, api::{LamReq, lam_eval}};
    let mut env = Env::default();
    run(&mut env, "Lambda(t) := tri(t)");
    run(&mut env, "g(t) := sum_(k=0)^infinity Lambda(t-k+1/2) ->");
    run(&mut env, "answer := g(0) =>");
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 1.0);
    let plot = graph::graph(&env, &GraphReq { axes: Default::default(),
        cfg: support::cfg(), curves: vec![support::curve("Lambda(t)", &env)],
        xrange: vec![XV::Num(-2.0), XV::Num(2.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 30,
    }).unwrap();
    assert!(!plot.series.is_empty());
    let lam = api::lambdify(&env, &LamReq { cfg: support::cfg(), src: support::src("tri(t)", &env), params: vec!["t".into()], units: Default::default(), unit: None }).unwrap();
    assert_eq!(lam_eval(&lam, &[api::Arg::Num(0.25)]).unwrap(), Some(0.75));
}
