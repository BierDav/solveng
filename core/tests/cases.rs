use solveng_core::{api::{self, LineReq}, env::Env, eval::{dim_of, eval_f, value}, format::Cfg, graph::{self, GraphReq, Src, XV}, parse::Mode};

fn run(env: &mut Env, src: &str, mode: Mode) -> solveng_core::sheet::Out {
    let out = api::line(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true });
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out
}

#[test]
fn case_functions_select_the_first_matching_branch_in_both_modes() {
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        run(&mut env, "sigma(t) := cases(0, t < 0, 1, t >= 0)", mode);
        let mut encoded = vec![];
        ciborium::into_writer(&env, &mut encoded).unwrap();
        let mut env: Env = ciborium::from_reader(encoded.as_slice()).unwrap();
        for (argument, expected) in [("-1", 0.0), ("0", 1.0), ("1", 1.0)] {
            run(&mut env, &format!("answer := σ({argument}) ="), mode);
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected);
        }
        run(&mut env, "f(t) := cases(2, t >= 0, 3, t >= -1, 4)", mode);
        for (argument, expected) in [("-2", 4.0), ("-1", 3.0), ("0", 2.0)] {
            run(&mut env, &format!("answer := f({argument}) ="), mode);
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected);
        }
    }
}

#[test]
fn unused_branches_are_not_evaluated_and_missing_matches_are_undefined() {
    let mut env = Env::default();
    for expression in ["cases(7, 1, 1/0, 1)", "cases(sqrt(-1), 0, 7)", "cases(unknown(t), 0, 7)", "cases(7)"] {
        run(&mut env, &format!("answer := {expression} ="), Mode::Code);
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 7.0);
    }
    run(&mut env, "f(t) := cases(1, t < 0)", Mode::Code);
    let out = api::line(&mut env, &LineReq { src: "f(0) =".into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
    assert!(out.errors.iter().any(|error| error.contains("no cases condition")), "{out:?}");
    assert_eq!(eval_f(&env.funs["f"].body, &|_| Some(0.0)), None);
}

#[test]
fn nested_cases_survive_substitution_and_render_as_cases() {
    let mut env = Env::default();
    run(&mut env, "f(t) := cases(cases(-1, t < -1, 0), t < 0, t^2)", Mode::Code);
    let out = run(&mut env, "g(t) := f(t-1) ->", Mode::Code);
    assert!(out.result.unwrap().starts_with("cases("));
    for (argument, expected) in [("-1", -1.0), ("1/2", 0.0), ("3", 4.0)] {
        run(&mut env, &format!("answer := g({argument}) ="), Mode::Code);
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected);
    }
}

#[test]
fn cases_validate_branch_units_and_conditions() {
    let mut env = Env::default();
    run(&mut env, "f(t) := cases(0, t < 0, [1 m])", Mode::Code);
    assert!(dim_of(&env.funs["f"].body, &|_| None).unwrap().is_some());
    run(&mut env, "g(t) := cases([1 m], t < 0, [1 s])", Mode::Code);
    assert!(dim_of(&env.funs["g"].body, &|_| None).unwrap_err().contains("different units"));
    let out = api::line(&mut env, &LineReq { src: "cases(1, [1 m]) =".into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
    assert!(out.errors.iter().any(|error| error.contains("unitless")), "{out:?}");
}

#[test]
fn graphs_split_at_case_boundaries_without_diagonal_segments() {
    let mut env = Env::default();
    run(&mut env, "sigma(t) := cases(0, t < 0, 1, t >= 0)", Mode::Code);
    let plot = graph::graph(&env, &GraphReq {
        cfg: Cfg::default(), curves: vec![Src { src: "σ(x)".into(), mode: Mode::Math, label: None, operators: vec![] }],
        xrange: vec![XV::Num(-1.0), XV::Num(1.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 100,
    }).unwrap();
    assert_eq!(plot.series.len(), 2);
    for series in &plot.series {
        assert!(series.points.iter().all(|(_, y)| *y == series.points[0].1));
    }
    assert!(plot.series[0].points.last().unwrap().0.abs() < 1e-9);
    assert!(plot.series[1].points.first().unwrap().0.abs() < 1e-9);
}

#[test]
fn graph_sampling_does_not_skip_narrow_case_intervals() {
    let mut env = Env::default();
    run(&mut env, "f(t) := cases(1, 0.123456 < t < 0.123457, 0)", Mode::Code);
    let plot = graph::graph(&env, &GraphReq {
        cfg: Cfg::default(), curves: vec![Src { src: "f(x)".into(), mode: Mode::Code, label: None, operators: vec![] }],
        xrange: vec![XV::Num(-1.0), XV::Num(1.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 20,
    }).unwrap();
    assert!(plot.series.iter().any(|series| series.points.iter().all(|(_, y)| *y == 1.0)));
    assert!(plot.series.iter().all(|series| series.points.iter().all(|(_, y)| *y == series.points[0].1)));
}
