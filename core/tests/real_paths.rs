//! Magnitudes and complex parts of unknowns: the general result, and the real one where it is simpler.
mod support;
use solveng_core::env::Env;
use support::result;

fn sym(source: &str) -> String {
    result(&mut Env::default(), source)
}

#[test]
fn magnitudes_resolve_for_real_unknowns() {
    for (source, want) in [
        ("abs(1 + e^(-#i 2 #pi f T))^2 -> assume(f in RR, T in RR)", r#""4" dot cos(#svconst("pi") dot f dot T)^("2")"#),
        ("abs(1 - e^(#i 2 x))^2 -> assume(x in RR)", r#""4" dot sin(x)^("2")"#),
        ("abs(1 + e^(#i x))^2 -> assume(x in RR)", r#""2" cos(x) + "2""#),
        ("abs(e^(#i x)) -> assume(x in RR)", r#""1""#),
        ("abs(x + #i y) -> assume(x in RR, y in RR)", r#"sqrt(x^("2") + y^("2"))"#),
        ("Re(e^(a + #i b)) -> assume(a in RR, b in RR)", r#"#svconst("euler")^(a) cos(b)"#),
        ("Im(cosh(x + #i y)) -> assume(x in RR, y in RR)", "sin(y) sinh(x)"),
        ("Im(sin(x + #i y)) -> assume(x in RR, y in RR)", "cos(x) sinh(y)"),
    ] {
        assert_eq!(sym(source), want, "{source}");
    }
}

#[test]
fn without_assumptions_both_results_show() {
    let mut env = Env::default();
    support::run(&mut env, "S(f) := 1 + e^(-#i 2 #pi f T)");
    let general = r#"abs("1" + #svconst("euler")^(-"2" #svconst("imaginary") dot #svconst("pi") dot f dot T))^("2")"#;
    let both = format!(r#"{general} = "4" dot cos(#svconst("pi") dot f dot T)^("2") quad "for" T in RR and f in RR"#);
    assert_eq!(support::run(&mut env, "Psi_s(f) := abs(S(f))^2 ->").result.unwrap(), both);
    assert_eq!(result(&mut env, "Psi_s(f) ->"), both);
    // only the names the real result needs
    assert_eq!(sym("abs(e^(#i x)) + y ->"), r#"y + abs(#svconst("euler")^(#svconst("imaginary") dot x)) = y + "1" quad "for" x in RR"#);
    // nothing simpler for real unknowns: the general result alone
    assert_eq!(sym("abs(x) ->"), "abs(x)");
    assert_eq!(sym("Re(ln(z)) ->"), "Re(ln(z))");
}

#[test]
fn trigonometric_squares_and_half_angles() {
    for (source, want) in [
        ("(1 + cos(x))^2 + sin(x)^2 ->", r#""2" cos(x) + "2""#),
        ("cos(x)^2 - sin(x)^2 + 2 sin(x)^2 ->", r#""1""#),
        ("2 + 2 cos(2 x) ->", r#""4" dot cos(x)^("2")"#),
        ("3 - 3 cos(4 x) ->", r#""6" dot sin("2" x)^("2")"#),
        // the half angle only where it is simpler
        ("1 - cos(x) ->", r#""1" - cos(x)"#),
        ("cosh(2 x) - 1 ->", r#""2" dot sinh(x)^("2")"#),
        ("cosh(x) + 1 ->", r#"cosh(x) + "1""#),
        ("sin(-x) + tanh(-y) ->", "-lr((sin(x) + tanh(y)))"),
        ("sin(-x) + cos(-x) ->", "cos(x) - sin(x)"),
    ] {
        assert_eq!(sym(source), want, "{source}");
    }
}

#[test]
fn impulses_are_even_and_scale() {
    for (source, want) in [
        ("dirac(-t) ->", "dirac(t)"),
        ("dirac(-(T + t)) ->", "dirac(T + t)"),
        ("dirac(-2 t) ->", r#"frac(dirac(t), "2")"#),
        ("dirac(2 t - 4) ->", r#"frac(dirac(t - "2"), "2")"#),
        ("dirac(t/3 - 1) ->", r#""3" dirac(t - "3")"#),
        // no numeric content to take out, mixed signs: as written
        ("dirac(T - t) ->", "dirac(T - t)"),
    ] {
        assert_eq!(sym(source), want, "{source}");
    }
}

#[test]
fn convolved_impulses_are_shifts_of_the_variable() {
    let mut env = Env::default();
    support::run(&mut env, "s(t) := dirac(t) + dirac(t-T)");
    support::run(&mut env, "R_s(tau) := convolve(s(tau), s(-tau), tau)");
    assert_eq!(result(&mut env, "R_s(t) ->"), r#"dirac(t - T) + dirac(T + t) + "2" dirac(t)"#);
    assert_eq!(result(&mut env, "R_s(tau) ->"), r#"dirac(τ - T) + dirac(T + τ) + "2" dirac(τ)"#);
    // a reversed, scaled impulse: δ(T − 2t) = δ(t − T/2)/2, shown with primitive integer coefficients
    support::run(&mut env, "q(tau) := convolve(dirac(tau), dirac(T - 2 tau), tau)");
    assert_eq!(result(&mut env, "q(t) ->"), r#"dirac("2" t - T)"#);
    support::run(&mut env, "p(tau) := convolve(dirac(tau - 1), dirac(-3 tau - 6), tau)");
    assert_eq!(result(&mut env, "p(t) ->"), r#"frac(dirac(t + "1"), "3")"#);
}
