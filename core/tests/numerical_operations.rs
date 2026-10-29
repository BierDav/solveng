//! Numerical APIs through the worksheet: dimensions, domains, deferred calls,
//! independent identities and conditioning, beyond the gap-audit minimum cases.
mod support;
use solveng_core::{
    complex::{CVal, Z},
    dim::Dim,
    env::Env,
    eval::{complex_value, eval_f},
    expr::{Expr, simp},
    linalg::Array,
};
use support::{fails, result, run};

fn expression(env: &mut Env, source: &str) -> Expr {
    run(env, &format!("answer := {source}"));
    simp(&support::bind(env, &env.vars["answer"].expr))
}
fn scalar(env: &mut Env, source: &str, expected: f64, dimension: Dim, tolerance: f64) {
    let value = complex_value(&expression(env, source)).unwrap_or_else(|e| panic!("{source}: {e:?}"));
    assert!(value.is_real(), "{source}: {value:?}");
    assert_eq!(value.d(), &dimension, "{source}");
    assert!((value.re.v.f() - expected).abs() <= tolerance, "{source}: {} != {expected}", value.re.v.f());
}
fn values(env: &mut Env, source: &str) -> Vec<CVal> {
    Array::of(&expression(env, source)).unwrap().a.iter().map(|e| complex_value(e).unwrap()).collect()
}

#[test]
fn descriptive_statistics_keep_dimensions_and_handle_offsets() {
    let mut env = Env::default();
    scalar(&mut env, "median(vec(9 m, 1 m, 3 m))", 3.0, Dim::base(0), 1e-12);
    scalar(
        &mut env,
        "variance(vec(1000000000001 m, 1000000000002 m, 1000000000003 m))",
        1.0,
        Dim::base(0).powi(2),
        1e-12,
    );
    scalar(&mut env, "stdev(vec(2 m, 4 m, 6 m))", 2.0, Dim::base(0), 1e-12);
    scalar(&mut env, "variance(vec(7, 7, 7, 7))", 0.0, Dim::ONE, 0.0);
    for input in [
        "median(vec()) =",
        "median(mat(1,2;3,4)) =",
        "variance(vec(1)) =",
        "stdev(vec(1 m,2 s)) =",
        "median(vec(1,#i)) =",
    ] {
        fails(&mut env, input);
    }
}

#[test]
fn matrix_spectra_and_least_squares_cover_complex_and_physical_data() {
    let mut env = Env::default();
    let roots = values(&mut env, "eigenvals(mat(0,-1;1,0))");
    assert!(roots.iter().any(|z| z.z().sub(Z::I).abs() < 1e-12));
    assert!(roots.iter().any(|z| z.z().add(Z::I).abs() < 1e-12));
    let roots = values(&mut env, "eigenvals(mat(1 + #i,0;0,2 - #i))");
    assert!(roots.iter().any(|z| z.z().sub(Z::new(1.0, 1.0)).abs() < 1e-12));
    let spectrum = values(&mut env, "singularvals(mat(3 m,0;0,4 m;0,0))");
    assert_eq!(spectrum.iter().map(|z| z.d().clone()).collect::<Vec<_>>(), vec![Dim::base(0); 2]);
    assert!((spectrum[0].re.v.f() - 4.0).abs() < 1e-12);
    let coefficients = values(&mut env, "lsolve(mat(1,0 s;1,1 s;1,2 s),vec(1 V,2 V,2 V))");
    assert_eq!(coefficients[0].d(), &Dim::si([2, 1, -3, -1, 0, 0, 0]));
    assert_eq!(coefficients[1].d(), &Dim::si([2, 1, -4, -1, 0, 0, 0]));
    assert!((coefficients[0].re.v.f() - 7.0 / 6.0).abs() < 1e-12);
    let coefficients = values(&mut env, "lsolve(mat(1,#i;0,1;1,1),vec(1 + 2 #i,2,3))");
    assert!(coefficients[0].z().sub(Z::real(1.0)).abs() < 1e-12);
    assert!(coefficients[1].z().sub(Z::real(2.0)).abs() < 1e-12);
    scalar(&mut env, "cond(mat(0,2;1,0))", 2.0, Dim::ONE, 1e-12);
    for input in [
        "eigenvals(mat(1,2,3;4,5,6)) =",
        "singularvals(mat(1 m,1 s;0,1 m)) =",
        "cond(mat(1,1;2,2)) =",
        "lsolve(mat(1,1;2,2;3,3),vec(1,2,3)) =",
        "lsolve(mat(1,2),vec(1)) =",
        "lsolve(mat(1;2),vec(1,2,3)) =",
    ] {
        fails(&mut env, input);
    }
}

#[test]
fn measured_data_validate_knots_and_fit_in_physical_units() {
    let mut env = Env::default();
    scalar(&mut env, "linterp(vec(0 s,2 s,5 s),vec(1 V,5 V,11 V),3 s)", 7.0, Dim::si([2, 1, -3, -1, 0, 0, 0]), 1e-12);
    assert_eq!(expression(&mut env, "linterp(vec(0,2,5),vec(1/3,2/3,7/3),2)"), solveng_core::expr::rat(2, 3));
    let fit = values(&mut env, "linfit(vec(0 s,1 s,2 s),vec(3 V,5 V,7 V))");
    assert_eq!(fit[1].d(), &Dim::si([2, 1, -4, -1, 0, 0, 0]));
    assert!((fit[0].re.v.f() - 3.0).abs() < 1e-12 && (fit[1].re.v.f() - 2.0).abs() < 1e-12);
    let fit = values(&mut env, "linfit(vec(1000000000000,1000000000001,1000000000002),vec(1,3,5))");
    assert!((fit[1].re.v.f() - 2.0).abs() < 1e-12);
    for input in [
        "linterp(vec(0,0),vec(1,2),0) =",
        "linterp(vec(1,0),vec(1,2),0) =",
        "linterp(vec(0,1),vec(1,2),2) =",
        "linterp(vec(0,1),vec(1),0) =",
        "linterp(vec(0 s,1 s),vec(1,2),1 m) =",
        "linfit(vec(2,2),vec(1,2)) =",
    ] {
        fails(&mut env, input);
    }
}

#[test]
fn transforms_have_explicit_phase_normalization_and_resolvable_small_parts() {
    let mut env = Env::default();
    let spectrum = values(&mut env, "dft(vec(0,1,0,0))");
    for (got, want) in spectrum.iter().zip([Z::real(1.0), Z::I.neg(), Z::real(-1.0), Z::I]) {
        assert!(got.z().sub(want).abs() < 1e-12);
    }
    for n in [1, 3, 5, 7, 10] {
        let samples = (0..n).map(|j| format!("({j} + ({j}+1) #i) [1 V]")).collect::<Vec<_>>().join(",");
        let recovered = values(&mut env, &format!("idft(dft(vec({samples})))"));
        for (j, value) in recovered.iter().enumerate() {
            assert!(value.z().sub(Z::new(j as f64, j as f64 + 1.0)).abs() < 1e-10);
            assert_eq!(value.d(), &Dim::si([2, 1, -3, -1, 0, 0, 0]));
        }
    }
    let tiny = values(&mut env, "dft(vec(1 + #i/1000000000000000000))");
    assert!((tiny[0].im.v.f() - 1e-18).abs() < 1e-30);
    for input in ["dft(vec()) =", "idft(mat(1,2;3,4)) =", "dft(vec(1 m,2 s)) ="] {
        fails(&mut env, input);
    }
}

#[test]
fn probability_and_special_functions_cover_tails_symmetry_and_domains() {
    let mut env = Env::default();
    scalar(&mut env, "normalcdf(10 m,10 m,2 m)", 0.5, Dim::ONE, 1e-15);
    scalar(&mut env, "normalquantile(1/2,10 m,2 m)", 10.0, Dim::base(0), 1e-12);
    scalar(&mut env, "normalcdf(normalquantile(1/1000000000000,0,1),0,1)", 1e-12, Dim::ONE, 1e-24);
    scalar(&mut env, "gamma(-1/2)", -2.0 * std::f64::consts::PI.sqrt(), Dim::ONE, 1e-12);
    scalar(&mut env, "erf(-1)", -0.8427007929497149, Dim::ONE, 1e-12);
    scalar(&mut env, "besselj(2,1)", 0.1149034849319005, Dim::ONE, 1e-12);
    // Scalar operation dimensions are available inside numerical calculus too.
    scalar(&mut env, "integrate(erf(t),t,0,1)", 0.4860649581122559, Dim::ONE, 1e-10);
    for input in [
        "normalcdf(0,0,-1) =",
        "normalquantile(0,0,1) =",
        "normalquantile(1,0,1) =",
        "normalcdf(1 m,0 s,1 s) =",
        "gamma(0) =",
        "gamma(-2) =",
        "erf(1 m) =",
        "besselj(1/2,1) =",
    ] {
        fails(&mut env, input);
    }
}

#[test]
fn bounded_minimization_and_ode_scopes_survive_calls_and_substitution() {
    let mut env = Env::default();
    scalar(&mut env, "minimize(-x,x,-2,3)", 3.0, Dim::ONE, 1e-10);
    scalar(&mut env, "minimize((x - [3 m])^2,x,[0 m],[5 m])", 3.0, Dim::base(0), 1e-8);
    run(&mut env, "f(a) := odesolve(a*t,y,t,0,0,1)");
    run(&mut env, "g(t) := f(t)");
    scalar(&mut env, "g(3)", 1.5, Dim::ONE, 1e-10);
    run(&mut env, "t := 40");
    run(&mut env, "y := 99");
    scalar(&mut env, "odesolve(-y/[2 s],y,t,[3 m],[0 s],[2 s])", 3.0 * (-1.0_f64).exp(), Dim::base(0), 1e-8);
    scalar(&mut env, "odesolve(-y,y,t,1,0,-1)", 1.0_f64.exp(), Dim::ONE, 1e-8);
    scalar(&mut env, "odesolve(t,y,t,2,3,3)", 2.0, Dim::ONE, 1e-12);
    scalar(&mut env, "odesolve(0,y,t,2 m,0 s,1 s)", 2.0, Dim::base(0), 0.0);
    assert_eq!(solveng_core::eval::value(&env.vars["t"].expr).unwrap().v.f(), 40.0);
    for input in [
        "minimize(x,x,2,1) =",
        "minimize(x,x,0,infinity) =",
        "minimize(x,x,1 m,2 s) =",
        "odesolve(y,y,y,1,0,1) =",
        "odesolve(y,y,t,1 m,0 s,1 s) =",
    ] {
        fails(&mut env, input);
    }
    let expr = expression(&mut Env::default(), "minimize((x-z)^2,x,0,4)");
    assert!((eval_f(&expr, &|n| (n == "z").then_some(2.0)).unwrap() - 2.0).abs() < 1e-8);
}

#[test]
fn absolute_temperatures_do_not_redefine_difference_units() {
    let mut env = Env::default();
    scalar(&mut env, "number(fahrenheit(212),[K])", 373.15, Dim::ONE, 1e-10);
    scalar(&mut env, "number(celsius(-273.15),[K])", 0.0, Dim::ONE, 1e-12);
    scalar(&mut env, "number(fahrenheit(-459.67),[K])", 0.0, Dim::ONE, 0.0);
    scalar(&mut env, "number(celsius(15)-celsius(-5),[K])", 20.0, Dim::ONE, 1e-12);
    for input in ["celsius(-274) =", "fahrenheit(-460) =", "celsius(1 K) =", "fahrenheit(#i) ="] {
        fails(&mut env, input);
    }
}

#[test]
fn interval_inequalities_keep_poles_and_isolated_points() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "x^2 > 1 -> solve(x)"), "x < \"−1\" or x > \"1\"");
    assert_eq!(result(&mut env, "x^2 <= 0 -> solve(x)"), "x = \"0\"");
    assert_eq!(result(&mut env, "x^2 >= 0 -> solve(x)"), "x in RR");
    assert_eq!(result(&mut env, "x^2 < 0 -> solve(x)"), "\"no real solution\"");
    let rendered = result(&mut env, "1/(x^2-1) > 0 -> solve(x)");
    assert_eq!(rendered, "x < \"−1\" or x > \"1\"");
    assert!(!env.vars.contains_key("x"), "set-valued inequalities must not choose an endpoint");
}

#[test]
fn sign_assumptions_validate_contradictions_and_preserve_complex_branches() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "sqrt(x^2) -> assume(x < 0)"), "-x");
    assert_eq!(result(&mut env, "sqrt(x^2) -> assume(0 <= x)"), "x");
    assert_eq!(result(&mut env, "abs(x) -> assume(x <= 0)"), "-x");
    fails(&mut env, "abs(x) -> assume(x > 0, x <= 0)");
    run(&mut env, "x := -2");
    fails(&mut env, "abs(x) -> assume(x > 0)");
    let mut env = Env::default();
    let before = env.clone();
    result(&mut env, "sqrt(x^2) -> assume(x > 0)");
    assert_eq!(env, before);
    assert_ne!(result(&mut env, "sqrt(x^2) ->"), "x");
    assert_ne!(result(&mut env, "ln(x*y) - ln(x) - ln(y) ->"), "\"0\"");
}

#[test]
fn repeated_real_root_locations_are_stable_under_coefficient_scaling() {
    let expected = [-3.0, 0.5, 4.0, 6.0];
    let mut coefficients = vec![1.0];
    for root in [-3.0, -3.0, 0.5, 0.5, 0.5, 4.0, 6.0] {
        let mut next = vec![0.0; coefficients.len() + 1];
        for (i, c) in coefficients.iter().enumerate() {
            next[i] -= root * c;
            next[i + 1] += c;
        }
        coefficients = next;
    }
    for scale in [1e-100, 1.0, 1e100] {
        let roots = solveng_core::solve::roots(&coefficients.iter().map(|c| c * scale).collect::<Vec<_>>());
        assert_eq!(roots.len(), expected.len(), "{roots:?}");
        for (got, want) in roots.iter().zip(expected) {
            assert!((got - want).abs() < 1e-10, "{roots:?}");
        }
    }
}

#[test]
fn parameterized_polynomial_solutions_and_partial_cancellation_are_general() {
    let mut env = Env::default();
    run(&mut env, "f(p,q) := x^3 + p*x + q = 0 -> solve(x)");
    for (p, q) in [(-3.0, 1.0), (0.0, 2.0), (2.0, -3.0)] {
        let roots = values(&mut env, &format!("f({p},{q})"));
        assert_eq!(roots.len(), 3);
        for root in roots {
            let z = root.z();
            assert!(z.mul(z).mul(z).add(z.scale(p)).add(Z::real(q)).abs() < 1e-10);
        }
    }
    run(&mut env, "answer := ((x+y+p)*(x^2+y^2+1))/((x+y+p)*(p+1)) -> expand");
    run(&mut env, "reduced := answer -> simplify");
    let (_, den) = solveng_core::poly::together(&env.vars["reduced"].expr);
    assert!(!den.has("x") && !den.has("y"), "{den:?}");
    for input in ["polyroots(vec(1,2,0)) =", "polyroots(vec(1 m,2 m)) ="] {
        fails(&mut env, input);
    }
}

#[test]
fn series_acceleration_handles_noninteger_tails_and_rejects_divergence() {
    let mut env = Env::default();
    scalar(&mut env, "sum(1/k^(3/2),k,1,infinity)", 2.612375348685488, Dim::ONE, 1e-8);
    scalar(&mut env, "sum((-1)^k/(k+3),k,0,infinity)", 2.0_f64.ln() - 0.5, Dim::ONE, 1e-10);
    scalar(&mut env, "sum((-1)^(k+1)/k^2,k,1,infinity)", std::f64::consts::PI.powi(2) / 12.0, Dim::ONE, 1e-10);
    for input in ["sum(1/k,k,1,infinity) =", "sum((-1)^k,k,0,infinity) =", "sum(2^k,k,0,infinity) ="] {
        fails(&mut env, input);
    }
}

#[test]
fn sign_assumptions_do_not_enter_numerical_binders() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "sqrt(x^2) = x -> assume(x > 0)"), "\"true\"");
    let rendered = result(&mut env, "minimize(abs(x),x,-2,2) -> assume(x > 0)");
    assert!(rendered.contains("abs"), "{rendered}");
}

#[test]
fn proven_constant_reduction_is_not_limited_to_a_fixed_set_of_constants() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "7*(sin(x)^2 + cos(x)^2) - 3 -> simplify"), "\"4\"");
    assert_eq!(result(&mut env, "2*(cosh(x)^2 - sinh(x)^2) -> simplify"), "\"2\"");
}

#[test]
fn numerical_operations_do_not_project_complex_samples_onto_the_real_axis() {
    let mut env = Env::default();
    let expr = expression(&mut env, "at(eigenvals(mat(0,-x;x,0)),0)");
    assert!(eval_f(&expr, &|n| (n == "x").then_some(1.0)).is_none());
    run(&mut env, "x := 7");
    let error = fails(&mut env, "minimize(x^2,x,0,2) -> substitute(x)");
    assert!(error.contains("doesn't occur"), "{error}");
}


#[test]
fn inverse_trigonometric_exact_values_and_simplification() {
    let mut env = Env::default();
    use solveng_core::expr::{int, mul, rat};

    // asin
    assert_eq!(expression(&mut env, "asin(0)"), int(0));
    assert_eq!(expression(&mut env, "asin(1)"), simp(&mul(vec![rat(1, 2), Expr::Pi])));
    assert_eq!(expression(&mut env, "asin(-1)"), simp(&mul(vec![rat(-1, 2), Expr::Pi])));
    assert_eq!(expression(&mut env, "asin(1/2)"), simp(&mul(vec![rat(1, 6), Expr::Pi])));
    assert_eq!(expression(&mut env, "asin(-1/2)"), simp(&mul(vec![rat(-1, 6), Expr::Pi])));

    // acos
    assert_eq!(expression(&mut env, "acos(1)"), int(0));
    assert_eq!(expression(&mut env, "acos(0)"), simp(&mul(vec![rat(1, 2), Expr::Pi])));
    assert_eq!(expression(&mut env, "acos(-1)"), Expr::Pi);
    assert_eq!(expression(&mut env, "acos(1/2)"), simp(&mul(vec![rat(1, 3), Expr::Pi])));
    assert_eq!(expression(&mut env, "acos(-1/2)"), simp(&mul(vec![rat(2, 3), Expr::Pi])));

    // atan
    assert_eq!(expression(&mut env, "atan(0)"), int(0));
    assert_eq!(expression(&mut env, "atan(1)"), simp(&mul(vec![rat(1, 4), Expr::Pi])));
    assert_eq!(expression(&mut env, "atan(-1)"), simp(&mul(vec![rat(-1, 4), Expr::Pi])));

    // Symbolic evaluation ->
    assert_eq!(result(&mut env, "asin(1) = pi/2 ->"), r#""true""#);
    assert_eq!(result(&mut env, "acos(0) = pi/2 ->"), r#""true""#);
    assert_eq!(result(&mut env, "atan(1) = pi/4 ->"), r#""true""#);
}

#[test]
fn cardinal_functions_exact_evaluation() {
    let mut env = Env::default();
    use solveng_core::expr::{div, int};

    // sinc
    assert_eq!(expression(&mut env, "sinc(0)"), int(1));
    assert_eq!(expression(&mut env, "sinc(pi)"), int(0));
    assert_eq!(expression(&mut env, "sinc(2*pi)"), int(0));
    assert_eq!(expression(&mut env, "sinc(-pi)"), int(0));
    assert_eq!(expression(&mut env, "sinc(-3*pi)"), int(0));
    assert_eq!(expression(&mut env, "sinc(pi/2)"), simp(&div(int(2), Expr::Pi)));
    assert_eq!(expression(&mut env, "sinc(-pi/2)"), simp(&div(int(2), Expr::Pi)));

    // cosc
    assert_eq!(expression(&mut env, "cosc(0)"), int(0));

    // tanc
    assert_eq!(expression(&mut env, "tanc(0)"), int(1));
    assert_eq!(expression(&mut env, "tanc(pi)"), int(0));
    assert_eq!(expression(&mut env, "tanc(2*pi)"), int(0));
    assert_eq!(expression(&mut env, "tanc(-pi)"), int(0));
    assert_eq!(expression(&mut env, "tanc(-4*pi)"), int(0));
}

#[test]
fn atan2_exact_symbolic_values_and_algebra() {
    let mut env = Env::default();
    use solveng_core::expr::{add, div, int, mul, neg, pow, rat, sym};
    use solveng_core::linalg::Op;

    // exact values
    assert_eq!(expression(&mut env, "atan2(0, 5)"), int(0));
    assert_eq!(expression(&mut env, "atan2(0, -3)"), Expr::Pi);
    assert_eq!(expression(&mut env, "atan2(2, 0)"), simp(&mul(vec![rat(1, 2), Expr::Pi])));
    assert_eq!(expression(&mut env, "atan2(-2, 0)"), simp(&mul(vec![rat(-1, 2), Expr::Pi])));

    assert_eq!(expression(&mut env, "atan2(3, 3)"), simp(&mul(vec![rat(1, 4), Expr::Pi])));
    assert_eq!(expression(&mut env, "atan2(4, -4)"), simp(&mul(vec![rat(3, 4), Expr::Pi])));
    assert_eq!(expression(&mut env, "atan2(-5, 5)"), simp(&mul(vec![rat(-1, 4), Expr::Pi])));
    assert_eq!(expression(&mut env, "atan2(-6, -6)"), simp(&mul(vec![rat(-3, 4), Expr::Pi])));

    // matching units
    assert_eq!(expression(&mut env, "atan2(1 m, 1 m)"), simp(&mul(vec![rat(1, 4), Expr::Pi])));
    assert_eq!(expression(&mut env, "atan2(0 m, 2 m)"), int(0));

    // undefined at (0, 0)
    fails(&mut env, "atan2(0, 0) =");

    // symbolic differentiation
    let theta = Expr::Op(Op::Atan2, vec![sym("y"), sym("x")]);
    let d_dx = theta.diff("x");
    let expected_dx = simp(&div(neg(sym("y")), add(vec![pow(sym("x"), int(2)), pow(sym("y"), int(2))])));
    assert_eq!(d_dx, expected_dx);

    let d_dy = theta.diff("y");
    let expected_dy = simp(&div(sym("x"), add(vec![pow(sym("x"), int(2)), pow(sym("y"), int(2))])));
    assert_eq!(d_dy, expected_dy);

    let atan_x = Expr::Op(Op::Atan2, vec![sym("x"), int(1)]);
    let expected_atan = simp(&div(int(1), add(vec![int(1), pow(sym("x"), int(2))])));
    assert_eq!(atan_x.diff("x"), expected_atan);
}

#[test]
fn special_functions_exact_values_and_symmetries() {
    let mut env = Env::default();
    use solveng_core::expr::{int, sqrt};

    // erf
    assert_eq!(expression(&mut env, "erf(0)"), int(0));
    assert_eq!(result(&mut env, "erf(0) ->"), r#""0""#);

    // gamma
    assert_eq!(expression(&mut env, "gamma(1)"), int(1));
    assert_eq!(expression(&mut env, "gamma(2)"), int(1));
    assert_eq!(expression(&mut env, "gamma(3)"), int(2));
    assert_eq!(expression(&mut env, "gamma(1/2)"), simp(&sqrt(Expr::Pi)));
    assert_eq!(result(&mut env, "gamma(1/2) = sqrt(pi) ->"), r#""true""#);

    // besselj
    assert_eq!(expression(&mut env, "besselj(0, 0)"), int(1));
    assert_eq!(expression(&mut env, "besselj(1, 0)"), int(0));
    assert_eq!(expression(&mut env, "besselj(2, 0)"), int(0));
    assert_eq!(expression(&mut env, "besselj(-1, 0)"), int(0));
    assert_eq!(expression(&mut env, "besselj(-3, 0)"), int(0));
}

#[test]
fn trigonometric_and_hyperbolic_identity_simplification() {
    let mut env = Env::default();
    assert_eq!(result(&mut env, "cos(x)*tan(x) = sin(x) ->"), r#""true""#);
    assert_eq!(result(&mut env, "tan(x) = sin(x)/cos(x) ->"), r#""true""#);
    assert_eq!(result(&mut env, "1 - tanh(x)^2 = 1/cosh(x)^2 ->"), r#""true""#);
    assert_eq!(result(&mut env, "tanh(x) = sinh(x)/cosh(x) ->"), r#""true""#);
}

#[test]
fn exact_special_values_preserve_argument_validation() {
    use solveng_core::{expr::{Val, int}, linalg::Op, num::Num};
    let dimensioned_zero = Expr::C(Val::new(Num::ZERO, Dim::base(0)));
    for (op, args) in [
        (Op::Erf, vec![dimensioned_zero.clone()]),
        (Op::BesselJ, vec![int(0), dimensioned_zero]),
        (Op::BesselJ, vec![int(10001), int(0)]),
    ] {
        let original = Expr::Op(op, args);
        assert_eq!(simp(&original), original, "invalid arguments must survive simplification");
    }
    let mut env = Env::default();
    for source in ["erf(0 m) =", "besselj(0, 0 m) =", "besselj(10001, 0) ="] {
        fails(&mut env, source);
    }
}
