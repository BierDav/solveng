use solveng_core::{api::{self, LineReq}, env::Env, format::Cfg, parse::Mode, eval::value};

fn run(env: &mut Env, src: &str, mode: Mode) -> solveng_core::sheet::Out {
    let out = api::line(env, &LineReq { src: src.into(), mode, cfg: Cfg::default(), listed: true });
    assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
    out
}

#[test]
fn signal_functions_and_their_boundaries_are_identical_in_both_modes() {
    for mode in [Mode::Math, Mode::Code] {
        let mut env = Env::default();
        for (expression, expected) in [
            ("rect(0)", 1.0), ("rect(0.5)", 0.5), ("rect(-0.5)", 0.5), ("rect(0.50000000001)", 0.0), ("rect(1)", 0.0),
            ("tri(0)", 1.0), ("tri(-0.5)", 0.5), ("tri(1)", 0.0), ("tri(2)", 0.0),
            ("sgn(-2)", -1.0), ("sgn(0)", 0.0), ("sgn(2)", 1.0), ("rect(0.25) * tri(0.25) * sgn(0.25)", 0.75),
        ] {
            run(&mut env, &format!("answer := {expression} ="), mode);
            assert_eq!(value(&env.vars["answer"].expr).unwrap().v.f(), expected, "{expression}");
        }
        let out = run(&mut env, "rect(t) + 2 rect(t) ->", mode);
        assert!(out.result.unwrap().contains("op(\"rect\")(t)"));
        run(&mut env, "a*rect(0) + b*tri(0) = 5; a*rect(1) + b*tri(0.5) = 1 -> solve(a,b)", mode);
        assert_eq!(value(&env.vars["a"].expr).unwrap().v.f(), 3.0);
        assert_eq!(value(&env.vars["b"].expr).unwrap().v.f(), 2.0);
    }
}

#[test]
fn signal_integrals_and_derivatives_keep_their_meaning() {
    let mut env = Env::default();
    for expression in ["integrate(rect(t), t, -2, 2)", "integrate(tri(t), t, -2, 2)"] {
        run(&mut env, &format!("answer := {expression} ="), Mode::Code);
        assert!((value(&env.vars["answer"].expr).unwrap().v.f() - 1.0).abs() < 1e-9);
    }
    let result = run(&mut env, "diff(sgn(t), t) ->", Mode::Code).result.unwrap();
    assert!(result.contains("dif"), "{result}");
    let out = api::line(&mut env, &LineReq { src: "rect([1 m]) =".into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
    assert!(out.errors.iter().any(|e| e.contains("unitless")));
}

#[test]
fn shifted_and_narrow_pulses_integrate_over_infinite_bounds() {
    let mut env = Env::default();
    for (expression, expected) in [
        ("integrate(rect((t - 12.3)/0.01), t, -infinity, infinity)", 0.01),
        ("integrate(tri(t - 0.3), t, -infinity, infinity)", 1.0),
        ("integrate(2*rect(t - 0.3) + 3*tri(t + 2.7), t, -infinity, infinity)", 5.0),
    ] {
        run(&mut env, &format!("answer := {expression} ="), Mode::Code);
        let actual = value(&env.vars["answer"].expr).unwrap().v.f();
        assert!((actual - expected).abs() < 1e-9, "{expression}: {actual}");
    }
}
