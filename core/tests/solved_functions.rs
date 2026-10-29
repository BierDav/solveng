mod support;
use solveng_core::{api, env::Env, eval::value, expr::{Expr, simp}};
use support::run;

#[test]
fn relation_functions_solve_to_all_branches() {
    {
        let mut env = Env::default();
        run(&mut env, "b := 99");
        run(&mut env, "x := 50");
        run(&mut env, "r := 100");
        run(&mut env, "\"pytagoras\"(a, b, c) := a^2 + b^2 = c^2");
        let out = run(&mut env, "\"circle\"(x, r) := \"pytagoras\"(x, b, r) -> solve(b)");
        assert!(out.result.unwrap().starts_with("vec("));
        assert_eq!(value(&env.vars["b"].expr).unwrap().v.f(), 99.0);
        assert_eq!(env.funs["circle"].params.len(), 2);
        let out = run(&mut env, "v := \"circle\"(3, 5) =");
        assert!(out.result.unwrap().starts_with("vec("));
        let Expr::Vector(v) = simp(&env.vars["v"].expr) else { panic!("expected vector") };
        let mut numbers: Vec<_> = v.iter().map(|v| value(v).unwrap().v.f()).collect();
        numbers.sort_by(f64::total_cmp);
        assert_eq!(numbers, [-4.0, 4.0]);
        run(&mut env, "upper(x, r) := at(circle(x, r), 0)");
        let out = run(&mut env, "circle(3, 5) =>");
        assert!(out.result.unwrap().contains("vec("));
        let out = run(&mut env, "upper(3, 5) =>");
        assert!(out.result.is_some());
        let out = run(&mut env, "circle(3, 5) ->");
        assert!(out.result.unwrap().starts_with("vec("));
    }
}

#[test]
fn solved_definitions_support_single_roots_systems_and_units() {
    {
        let mut env = Env::default();
        run(&mut env, "linear(a) := 2z = a -> solve(z)");
        run(&mut env, "v := linear(6) =>");
        assert_eq!(value(&env.vars["v"].expr).unwrap().v.f(), 3.0);
        run(&mut env, "pair(a) := vec(u + v = a, u - v = 2) -> solve(u, v)");
        run(&mut env, "p := pair(6) =>");
        let Expr::Vector(v) = simp(&env.vars["p"].expr) else { panic!("expected vector") };
        assert_eq!(v.iter().map(|v| value(v).unwrap().v.f()).collect::<Vec<_>>(), [4.0, 2.0]);
        run(&mut env, "roots := z^2 = 4 -> solve(z)");
        assert!(matches!(env.vars["roots"].expr, Expr::Vector(_)));
        run(&mut env, "radius(x: m, r: m) := x^2 + b^2 = r^2 -> solve(b)");
        let out = run(&mut env, "radius(3 m, 5 m) =>");
        assert!(out.result.unwrap().contains("upright(\"m\")"));
    }
}

#[test]
fn invalid_solved_definitions_do_not_replace_existing_functions() {
    for src in [
        "f(x) := x = 1 -> solve(x)",
        "f(x) := z + 1 -> solve(z)",
        "f(x) := z < x -> solve(z)",
        "f(x) := z = x -> solve(z, z)",
        "f(x) := 1 = 2 -> solve(z)",
        "f(x) := sin(z) + z = x -> solve(z)",
    ] {
        let mut env = Env::default();
        run(&mut env, "f(x) := 7");
        let original = env.funs["f"].body.clone();
        let out = support::line(&mut env, src);
        assert!(!out.errors.is_empty(), "{src}: {out:?}");
        assert_eq!(env.funs["f"].body, original);
    }
    // solutions are complex where they have to be
    let mut env = Env::default();
    run(&mut env, "f(x) := z^2 = -x -> solve(z)");
    assert!(support::run(&mut env, "f(4) =>").result.unwrap().contains("\"2\" #svconst(\"imaginary\")"));
}

#[test]
fn vectors_survive_reports_serialization_graphs_and_domain_checks() {
    use solveng_core::{api::DefinedReq, graph::{self, GraphReq, XV}};
    let mut env = Env::default();
    run(&mut env, "b := 99");
    run(&mut env, "circle(x, r) := x^2 + b^2 = r^2 -> solve(b)");
    run(&mut env, "v := circle(3, 5) =>");
    let rows = api::defined(&env, &DefinedReq { cfg: support::cfg() }).unwrap();
    let v = rows.iter().find(|r| r.name == "v").unwrap();
    assert!(v.plain.starts_with('[') && v.plain.contains("4"), "{}", v.plain);
    let mut encoded = vec![];
    ciborium::into_writer(&env, &mut encoded).unwrap();
    let mut env: Env = ciborium::from_reader(encoded.as_slice()).unwrap();
    let out = run(&mut env, "circle(3, 5) =>");
    assert!(!out.result.unwrap().contains("99"));
    run(&mut env, "at(circle(3, 5), 0) =>");
    // beyond the circle the solutions are imaginary
    assert!(run(&mut env, "circle(6, 5) =>").result.unwrap().contains("\"3.317\" #svconst(\"imaginary\")"));
    for src in ["at(circle(3, 5), 2) =>", "at(circle(3, 5), -1) =>", "at(circle(3, 5), 0.5) =>", "at(3, 0) =>"] {
        let out = support::line(&mut env, src);
        assert!(!out.errors.is_empty(), "{src}: {out:?}");
    }
    let plot = graph::graph(&env, &GraphReq { axes: Default::default(),
        cfg: support::cfg(), curves: vec![support::curve("circle(x, 5)", &env)],
        xrange: vec![XV::Num(-5.0), XV::Num(5.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 100,
    }).unwrap();
    assert!(plot.series.iter().any(|s| s.points.iter().any(|(_, y)| *y < -4.9)));
    assert!(plot.series.iter().any(|s| s.points.iter().any(|(_, y)| *y > 4.9)));
    run(&mut env, "relation(a: m, b: m, c: m) := a^2 + b^2 = c^2");
    run(&mut env, "typed(x, r) := relation(x, b, r) -> solve(b)");
    assert!(env.funs["typed"].params.iter().all(|p| p.ty.is_some()));
}

#[test]
fn graphs_after_selecting_a_solved_branch_are_over_the_solved_name() {
    use solveng_core::graph::{self, GraphReq, XV};
    let plot = |env: &Env, srcs: &[&str], xrange: Vec<XV>| graph::graph(env, &GraphReq { axes: Default::default(),
        cfg: support::cfg(), curves: srcs.iter().map(|s| support::curve(s, env)).collect(),
        xrange, yrange: vec![], marks: vec![], ymark: vec![], samples: 50,
    });
    let mut env = Env::default();
    run(&mut env, "f_0 := 1 kHz");
    run(&mut env, "S_1(f: Hz) := 1/sqrt(1 + (f/f_0)^2)");
    // untyped: f's unit comes from its value once it has one
    run(&mut env, "S_2(f) := 1/sqrt(1 + (f/f_0)^2)");
    let before = plot(&env, &["S_1(f)"], vec![]).unwrap();
    // Solved definitions retain every branch. Select a scalar explicitly before
    // checking the graph's fallback to a name that already has a scalar value.
    run(&mut env, "roots := S_1(f) = 1/2 -> solve(f)");
    run(&mut env, "f := at(roots, 0)");
    assert!(env.vars.contains_key("f"));
    let after = plot(&env, &["S_1(f)"], vec![]).unwrap();
    assert_eq!((after.x0, after.x1, &after.xlabel), (before.x0, before.x1, &before.xlabel));
    assert_eq!(after.series[0].points, before.series[0].points);
    // in the value's unit, also where only an expression or a constant line goes with it
    for srcs in [&["S_2(f)"][..], &["2 S_2(f) + 1"], &["S_1(f)", "1/2"], &["S_1(f)", "S_2(2 f)"]] {
        let p = plot(&env, srcs, vec![XV::Num(0.0), XV::Num(5000.0)]).unwrap_or_else(|e| panic!("{srcs:?}: {e}"));
        assert!(p.xlabel.contains("Hz"), "{srcs:?}: {}", p.xlabel);
        let s = &p.series[0].points;
        assert!(s.first().unwrap().1 > s.last().unwrap().1, "{srcs:?}: {s:?}");
    }
    // something else varies: the defined f stays its value
    let p = plot(&env, &["S_1(f) + x"], vec![XV::Num(0.0), XV::Num(1.0)]).unwrap();
    assert!(p.xlabel.contains('x'), "{}", p.xlabel);
    // two names with values: which one the graph is over isn't clear
    let e = plot(&env, &["S_1(f) + f/f_0"], vec![]).err().unwrap();
    assert!(e.contains("nothing varies") && e.contains("'f', 'f_0' have values"), "{e}");
}
