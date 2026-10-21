use solveng_core::{api::{self, DefinedReq, LineReq}, env::{Env, Operator}, eval::value, expr::{Expr, Func, int, mul, simp, sub, sym}, format::Cfg, parse::Mode};

fn run(env: &mut Env, src: &str, mode: Mode, operators: &[Operator]) -> solveng_core::sheet::Out {
    let out = api::line_with_operators(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true }, operators);
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out
}

#[test]
fn worksheet_operators_keep_names_and_simplify_substituted_arguments() {
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        let operators = [Operator { name: "Λ".into(), function: "tri".into() }];
        run(&mut env, "g(t) := \"Lambda\"(2t)", mode, &operators);
        let mut bytes = vec![];
        ciborium::into_writer(&env, &mut bytes).unwrap();
        let mut env: Env = ciborium::from_reader(bytes.as_slice()).unwrap();
        let out = run(&mut env, "g_1(t) := g(t-1/2) -> simplify", mode, &[]);
        assert_eq!(out.result.as_deref(), Some("Λ(\"2\" t - \"1\")"));
        assert_eq!(env.funs["g_1"].body, Expr::Op("Λ".into(), Func::Tri, Box::new(simp(&sub(mul(vec![int(2), sym("t")]), int(1))))));
        for (argument, expected) in [("0", 0.0), ("1/2", 1.0), ("3/4", 0.5)] {
            run(&mut env, &format!("answer := g_1({argument}) ="), mode, &[]);
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected);
        }
        let rows = api::defined(&env, &DefinedReq { cfg: Cfg::default() }).unwrap();
        assert_eq!(rows.iter().find(|row| row.name == "g_1").unwrap().values, ["Λ(\"2\" t - \"1\")"]);
    }
}

#[test]
fn different_aliases_and_builtin_collisions_survive_expansion_and_formula_steps() {
    let mut env = Env::default();
    for (function, alias) in [("f", "ΛΩ"), ("g", "dreieick"), ("h", "sqrt")] {
        let operators = [Operator { name: alias.into(), function: "tri".into() }];
        run(&mut env, &format!("{function}(t) := \"{alias}\"(2t)"), Mode::Math, &operators);
    }
    let out = run(&mut env, "f(t-1/2) + g(t-1/2) + h(t-1/2) ->", Mode::Code, &[]);
    let result = out.result.unwrap();
    for name in ["Λ Ω", "dreieick", "sqrt"] { assert!(result.contains(name), "{result}"); }
    assert!(!result.contains("tri"), "{result}");
    let out = run(&mut env, "h(1/4) =", Mode::Code, &[]);
    assert!(out.show.as_ref().unwrap().contains("upright(\"sqrt\")"), "{out:?}");
}

#[test]
fn simplification_descends_into_builtin_and_unspecified_function_arguments() {
    let mut env = Env::default();
    for function in ["tri", "sin", "unknown"] {
        let out = run(&mut env, &format!("{function}(2(t-1/2)) ->"), Mode::Code, &[]);
        let result = out.result.unwrap();
        assert!(result.contains("(\"2\" t - \"1\")"), "{result}");
    }
    let out = run(&mut env, "sin((t^2-1)/(t-1)) ->", Mode::Code, &[]);
    assert_eq!(out.result.as_deref(), Some("sin(t + \"1\")"));
}

#[test]
fn operator_bindings_do_not_replace_native_functions_or_units() {
    let mut env = Env::default();
    for alias in ["sqrt", "Ω"] {
        let operators = [Operator { name: alias.into(), function: "tri".into() }];
        run(&mut env, &format!("f(t) := \"{alias}\"(2(t-1/2)) + sqrt(9) ->"), Mode::Math, &operators);
        run(&mut env, "answer := f(1/2) =", Mode::Code, &[]);
        assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 4.0);
    }
}

#[test]
fn stored_functions_keep_the_operation_from_their_defining_scope() {
    let mut env = Env::default();
    for (name, function) in [("f", "tri"), ("g", "rect")] {
        let operators = [Operator { name: "Λ".into(), function: function.into() }];
        run(&mut env, &format!("{name}(t) := \"Λ\"(t)"), Mode::Math, &operators);
    }
    run(&mut env, "answer := f(3/4) - g(3/4) =", Mode::Code, &[]);
    assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), 0.25);
}

#[test]
fn aliased_signals_keep_calculus_and_unit_checks() {
    let mut env = Env::default();
    let operators = [Operator { name: "Λ".into(), function: "tri".into() }];
    run(&mut env, "g(t) := \"Λ\"(2t)", Mode::Math, &operators);
    run(&mut env, "area := integrate(g(t-1/2), t, -infinity, infinity) =", Mode::Code, &[]);
    assert!((value(&env.vars["area"].expr).unwrap().v.f() - 0.5).abs() < 1e-9);
    let out = run(&mut env, "diff(g(t), t) ->", Mode::Code, &[]);
    let result = out.result.unwrap();
    assert!(result.contains("Λ") && !result.contains("tri"), "{result}");
    let out = api::line(&mut env, &LineReq { src: "g([1 m]) =".into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
    assert!(out.errors.iter().any(|error| error.contains("unitless")), "{out:?}");
}
