use solveng_core::{api::{self, LineReq}, env::Env, eval::value, expr::{Expr, simp}, format::Cfg, parse::Mode};

fn run(env: &mut Env, src: &str, mode: Mode) -> solveng_core::sheet::Out {
    let out = api::line(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true });
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out
}

#[test]
fn relation_functions_solve_to_all_branches_in_both_modes() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        run(&mut env, "b := 99", mode);
        run(&mut env, "x := 50", mode);
        run(&mut env, "r := 100", mode);
        run(&mut env, "\"pytagoras\"(a, b, c) := a^2 + b^2 = c^2", mode);
        let out = run(&mut env, "\"circle\"(x, r) := \"pytagoras\"(x, b, r) -> solve(b)", mode);
        assert!(out.result.unwrap().starts_with("vec("));
        assert_eq!(value(&env.vars["b"].expr).unwrap().v.f(), 99.0);
        assert_eq!(env.funs["circle"].params.len(), 2);
        let out = run(&mut env, "v := \"circle\"(3, 5) =", mode);
        assert!(out.result.unwrap().starts_with("vec("));
        let Expr::Vector(v) = simp(&env.vars["v"].expr) else { panic!("expected vector") };
        let mut numbers: Vec<_> = v.iter().map(|v| value(v).unwrap().v.f()).collect();
        numbers.sort_by(f64::total_cmp);
        assert_eq!(numbers, [-4.0, 4.0]);
        run(&mut env, "upper(x, r) := at(circle(x, r), 0)", mode);
        let out = run(&mut env, "circle(3, 5) =", mode);
        assert!(out.result.unwrap().contains("vec("));
        let out = run(&mut env, "upper(3, 5) =", mode);
        assert!(out.result.is_some());
        let out = run(&mut env, "circle(3, 5) ->", mode);
        assert!(out.result.unwrap().starts_with("vec("));
    }
}

#[test]
fn solved_definitions_support_single_roots_systems_and_units() {
    for mode in [Mode::Code, Mode::Math] {
        let mut env = Env::default();
        run(&mut env, "linear(a) := 2z = a -> solve(z)", mode);
        run(&mut env, "v := linear(6) =", mode);
        assert_eq!(value(&env.vars["v"].expr).unwrap().v.f(), 3.0);
        run(&mut env, "pair(a) := vec(u + v = a, u - v = 2) -> solve(u, v)", mode);
        run(&mut env, "p := pair(6) =", mode);
        let Expr::Vector(v) = simp(&env.vars["p"].expr) else { panic!("expected vector") };
        assert_eq!(v.iter().map(|v| value(v).unwrap().v.f()).collect::<Vec<_>>(), [4.0, 2.0]);
        run(&mut env, "roots := z^2 = 4 -> solve(z)", mode);
        assert!(matches!(env.vars["roots"].expr, Expr::Vector(_)));
        run(&mut env, "radius(x: m, r: m) := x^2 + b^2 = r^2 -> solve(b)", mode);
        let out = run(&mut env, "radius(3 ⦃m⦄, 5 ⦃m⦄) =", mode);
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
        "f(x) := z^2 = -1 -> solve(z)",
        "f(x) := sin(z) + z = x -> solve(z)",
    ] {
        let mut env = Env::default();
        run(&mut env, "f(x) := 7", Mode::Code);
        let original = env.funs["f"].body.clone();
        let out = api::line(&mut env, &LineReq { src: src.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
        assert!(!out.errors.is_empty(), "{src}: {out:?}");
        assert_eq!(env.funs["f"].body, original);
    }
}

#[test]
fn vectors_survive_reports_serialization_graphs_and_domain_checks() {
    use solveng_core::{api::DefinedReq, graph::{self, GraphReq, Src, XV}};
    let mut env = Env::default();
    run(&mut env, "b := 99", Mode::Code);
    run(&mut env, "circle(x, r) := x^2 + b^2 = r^2 -> solve(b)", Mode::Code);
    run(&mut env, "v := circle(3, 5) =", Mode::Code);
    let rows = api::defined(&env, &DefinedReq { cfg: Cfg::default() }).unwrap();
    let v = rows.iter().find(|r| r.name == "v").unwrap();
    assert!(v.plain.starts_with('[') && v.plain.contains("4"), "{}", v.plain);
    let mut encoded = vec![];
    ciborium::into_writer(&env, &mut encoded).unwrap();
    let mut env: Env = ciborium::from_reader(encoded.as_slice()).unwrap();
    let out = run(&mut env, "circle(3, 5) =", Mode::Code);
    assert!(!out.show.unwrap().contains("99"));
    run(&mut env, "at(circle(3, 5), 0) =", Mode::Code);
    for src in ["circle(6, 5) =", "at(circle(3, 5), 2) =", "at(circle(3, 5), -1) =", "at(circle(3, 5), 0.5) =", "at(3, 0) ="] {
        let out = api::line(&mut env, &LineReq { src: src.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
        assert!(!out.errors.is_empty(), "{src}: {out:?}");
    }
    let plot = graph::graph(&env, &GraphReq {
        cfg: Cfg::default(), curves: vec![Src { src: "circle(x, 5)".into(), mode: Mode::Code, label: None }],
        xrange: vec![XV::Num(-5.0), XV::Num(5.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 100,
    }).unwrap();
    assert!(plot.series.iter().any(|s| s.points.iter().any(|(_, y)| *y < -4.9)));
    assert!(plot.series.iter().any(|s| s.points.iter().any(|(_, y)| *y > 4.9)));
    run(&mut env, "relation(a: m, b: m, c: m) := a^2 + b^2 = c^2", Mode::Code);
    run(&mut env, "typed(x, r) := relation(x, b, r) -> solve(b)", Mode::Code);
    assert!(env.funs["typed"].params.iter().all(|p| p.ty.is_some()));
}
