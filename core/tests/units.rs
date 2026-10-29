mod support;
use solveng_core::{env::Env, eval::value};
use support::{line, line_raw, run, unit};

fn ok(env: &mut Env, src: &str) -> String {
    run(env, src).result.unwrap_or_default()
}

#[test]
fn a_single_letter_after_the_arrow_is_a_name() {
    let mut env = Env::default();
    ok(&mut env, "x := 5 km");
    ok(&mut env, "v := 72 km/h");
    for (src, want) in [("x -> #m", "5000"), ("x -> [m]", "5000"), ("x -> #m, digits(2)", "5000"), ("v -> frac(#m, #s)", "20"), ("v -> [m/s]", "20"), ("x/v -> #s", "250"), ("v -> frac(km, #h)", "72"), ("x^2 -> #m^2", "10^(7)"), ("x -> km", "\"5\"")] {
        let shown = ok(&mut env, src);
        assert!(shown.contains(want), "{src}: {shown}");
    }
    // a plain single letter after -> is a name, never a unit
    for (src, name) in [("x -> m", "m"), ("x/v -> s", "s"), ("x -> m, digits(2)", "m")] {
        let out = line(&mut env.clone(), src);
        assert!(out.errors.iter().any(|e| e.contains(&format!("'{name}'"))), "{src}: {out:?}");
    }
    let out = line(&mut env.clone(), "x -> q");
    assert!(!out.errors.is_empty() || out.prose, "{out:?}");
    // after =, a single letter stays a variable: a statement
    let out = line(&mut env, "x => m");
    assert!(out.mark.is_none(), "{out:?}");
}

#[test]
fn quoted_math_names_do_not_substitute_units_or_operations() {
    let mut env = Env::default();
    ok(&mut env, "x := 2 \"s\"");
    let out = line(&mut env, "x =>");
    assert!(out.errors.iter().any(|e| e.contains("'s'")), "{out:?}");
    ok(&mut env, &format!("t := 2 {}", unit("s")));
    assert_eq!(value(&env.vars["t"].expr).unwrap().v.f(), 2.0);
    let out = line(&mut env, "\"tri\"(0) =");
    assert!(out.errors.iter().any(|e| e.contains("tri isn't defined")), "{out:?}");
    let out = line_raw(&mut env, "⦇tri⦈(0) =>");
    assert!(out.errors.is_empty(), "{out:?}");
    assert_eq!(out.result.as_deref(), Some("\"1\""));
    // a plain name is no operation either
    let out = line_raw(&mut env, "tri(0) =>");
    assert!(out.errors.iter().any(|e| e.contains("tri isn't defined")), "{out:?}");
    ok(&mut env, "\"s\" := 3");
    ok(&mut env, "y := 2 \"s\"");
    assert_eq!(value(&env.vars["y"].expr).unwrap().v.f(), 6.0);
}
