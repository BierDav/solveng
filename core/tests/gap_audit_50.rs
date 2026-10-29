//! Desired behavior for the 50 findings in docs/gap-audit-50.md.
//!
//! Known failures are opt-in: cargo test -p solveng-core --test gap_audit_50 -- --ignored
//! Resolved findings run in normal CI; pending features remain opt-in.
//! Explicit operation tokens in feature tests specify proposed APIs, not existing promises.
mod support;

use solveng_core::{
    api::{self, LamReq},
    complex::{CVal, Z},
    dim::Dim,
    env::Env,
    eval::complex_value,
    expr::{Expr, simp},
    linalg::Array,
    num::Rat,
};

fn expression(env: &mut Env, source: &str) -> Expr {
    support::run(env, &format!("audit_answer := {source}"));
    simp(&support::bind(env, &env.vars["audit_answer"].expr))
}

fn evaluated(env: &mut Env, source: &str) -> CVal {
    complex_value(&expression(env, source)).unwrap_or_else(|e| panic!("{source}: {e:?}"))
}

fn close(got: f64, want: f64, tolerance: f64) {
    assert!(
        got.is_finite() && (got - want).abs() <= tolerance,
        "got {got:.17e}, expected {want:.17e}, absolute tolerance {tolerance:.3e}"
    );
}

fn real_in(env: &mut Env, source: &str, want: f64, tolerance: f64) {
    let got = evaluated(env, source);
    assert!(got.is_real(), "{source}: expected real, got {got:?}");
    assert!(got.d().is_one(), "{source}: expected unitless, got {got:?}");
    close(got.re.v.f(), want, tolerance);
}

fn real(source: &str, want: f64, tolerance: f64) {
    real_in(&mut Env::default(), source, want, tolerance);
}

fn entries(env: &mut Env, source: &str, shape: (usize, usize)) -> Vec<CVal> {
    let e = expression(env, source);
    let a = Array::of(&e).unwrap_or_else(|| panic!("{source}: expected array, got {e:?}"));
    assert_eq!((a.rows, a.cols), shape, "{source}");
    a.a.iter().map(|e| complex_value(e).unwrap()).collect()
}

fn array(source: &str, shape: (usize, usize), want: &[f64]) {
    let got = entries(&mut Env::default(), source, shape);
    assert_eq!(got.len(), want.len());
    for (got, want) in got.iter().zip(want) {
        assert!(got.is_real() && got.d().is_one(), "{source}: {got:?}");
        close(got.re.v.f(), *want, 1e-9);
    }
}

fn rejects(source: &str) {
    let out = support::line(&mut Env::default(), source);
    assert!(
        !out.prose && !out.errors.is_empty(),
        "{source}: expected error, got {out:?}"
    );
}

fn identity(source: &str) {
    assert_eq!(
        support::result(&mut Env::default(), source),
        "\"true\"",
        "{source}"
    );
}

fn lambda(source: &str, params: &[&str]) -> LamReq {
    LamReq {
        cfg: support::cfg(),
        src: support::src(source, &Env::default()),
        params: params.iter().map(|s| s.to_string()).collect(),
        units: Default::default(),
        unit: None,
    }
}

#[test]
fn b01_square_root_retains_small_imaginary_part() {
    for epsilon in [1e-12, 1e-20, -1e-20] {
        let got = Z::new(1.0, epsilon).sqrt();
        close(got.re, 1.0, 1e-15);
        close(got.im, epsilon / 2.0, epsilon.abs() * 1e-12);
    }
    real("Im(sqrt(1 + #i*10^(-20)))", 5e-21, 1e-32);
}

#[test]
fn b02_square_root_scales_large_components() {
    let got = Z::new(1e308, 1e308).sqrt();
    let root_two = 2.0_f64.sqrt();
    close(got.re / 1e154, ((root_two + 1.0) / 2.0).sqrt(), 1e-14);
    close(got.im / 1e154, ((root_two - 1.0) / 2.0).sqrt(), 1e-14);
}

#[test]
fn b03_dividing_large_complex_number_by_itself_is_one() {
    for sign in [1.0, -1.0] {
        let z = Z::new(1e308, sign * 1e308);
        let got = z.div(z);
        close(got.re, 1.0, 1e-15);
        close(got.im, 0.0, 1e-15);
    }
}

#[test]
fn b04_inverse_sine_accepts_large_real_complex_continuation() {
    // asin(x) = pi/2 - i*acosh(x); acosh(1e300) differs negligibly from ln(2e300).
    let got = evaluated(&mut Env::default(), "arcsin(10^300)");
    close(got.re.v.f(), std::f64::consts::FRAC_PI_2, 1e-12);
    close(got.im.v.f(), -(300.0 * 10.0_f64.ln() + 2.0_f64.ln()), 1e-10);
}

#[test]
fn b05_tangent_of_large_imaginary_argument_is_finite() {
    // tan(i*y) = i*tanh(y), which rounds to i here.
    let got = evaluated(&mut Env::default(), "tan(1000 #i)");
    close(got.re.v.f(), 0.0, 1e-12);
    close(got.im.v.f(), 1.0, 1e-12);
}

#[test]
fn b06_vector_norm_scales_before_squaring() {
    for exponent in [300, -300] {
        let got = evaluated(
            &mut Env::default(),
            &format!("norm(vec(10^{exponent},10^{exponent}))"),
        );
        assert!(got.is_real());
        close(
            got.re.v.f() / 10.0_f64.powi(exponent),
            2.0_f64.sqrt(),
            1e-12,
        );
    }
}

#[test]
fn b07_rational_order_is_consistent_with_exact_equality() {
    let n = 10_i128.pow(30);
    let a = Rat::new(n + 1, n).unwrap();
    let b = Rat::new(n + 2, n + 1).unwrap();
    // a-b = 1/(n*(n+1)) > 0. Both fit i128; their cross products do not.
    assert_ne!(a, b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Greater);
}

#[test]
fn b08_large_perfect_square_has_exact_rational_root() {
    let root = 1_234_567_890_123_456_789_i128;
    assert_eq!(Rat::int(root * root).root(2), Some(Rat::int(root)));
}

#[test]
fn b09_minimum_i128_is_a_valid_rational_numerator() {
    // A panic is a failing test; do not turn it into an expected panic.
    let r = Rat::new(i128::MIN, 1).expect("representable numerator and positive denominator");
    assert_eq!((r.numer(), r.denom()), (i128::MIN, 1));
}

#[test]
fn b10_odd_denominator_above_sixty_four_keeps_real_root() {
    real("(-8)^(1/65)", -8.0_f64.powf(1.0 / 65.0), 1e-12);
}

#[test]
fn b11_modulus_derivative_along_real_parameter_is_real() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := diff(abs(x+#i),x)");
    real_in(&mut env, "f(1)", 1.0 / 2.0_f64.sqrt(), 1e-12);
}

#[test]
fn b12_modulus_of_square_is_differentiable_at_zero() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := diff(abs(x^2),x)");
    real_in(&mut env, "f(0)", 0.0, 0.0);
}

#[test]
fn b13_lambdify_rejects_duplicate_parameters() {
    for params in [["x", "x"], ["alpha", "α"]] {
        let req = lambda(params[0], &params);
        assert!(
            api::lambdify(&Env::default(), &req).is_err(),
            "duplicate parameters: {params:?}"
        );
    }
}

#[test]
fn b14_lambdify_checks_known_parameter_dimensions() {
    for source in ["x + 1 s", "sin(x)"] {
        let mut req = lambda(source, &["x"]);
        req.units.insert("x".into(), support::unit("m"));
        assert!(
            api::lambdify(&Env::default(), &req).is_err(),
            "invalid dimensions: {source}"
        );
    }
}

#[test]
fn b15_zero_quantities_with_different_units_are_not_an_identity() {
    let out = support::line(&mut Env::default(), "0 m = 0 s ->");
    assert!(
        !out.errors.is_empty() || out.result.as_deref() == Some("\"false\""),
        "{out:?}"
    );
}

#[test]
fn b16_ode_resolves_periodic_forcing_instead_of_false_convergence() {
    // Exact solution sin(256*pi*t)/(256*pi), zero at both endpoints.
    real("odesolve(cos(256*pi*t),y,t,0,0,1)", 0.0, 1e-8);
}

#[test]
fn b17_minimizer_retains_local_resolution_at_large_coordinate_offset() {
    real(
        "minimize((x-1000000000000.25)^2,x,1000000000000,1000000000001)",
        1_000_000_000_000.25,
        1e-3,
    );
}

#[test]
fn b18_function_quotient_retains_its_original_domain() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := x/x");
    let out = support::line(&mut env, "f(0) =>");
    assert!(!out.errors.is_empty(), "x/x is undefined at zero: {out:?}");
}

#[test]
fn b19_zero_to_purely_imaginary_power_is_undefined() {
    for source in ["0^#i =>", "0^(-#i) =>"] {
        rejects(source);
    }
}

#[test]
fn b20_tangent_rejects_exact_symbolic_poles() {
    for source in ["tan(pi/2) =>", "tan(3*pi/2) =>", "tanc(pi/2) =>"] {
        rejects(source);
    }
}

#[test]
fn b21_nested_power_preserves_the_principal_square_root() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := sqrt(x^3)");
    let got = evaluated(&mut env, "f(-1)");
    close(got.re.v.f(), 0.0, 0.0);
    close(got.im.v.f(), 1.0, 0.0);
}

#[test]
fn b22_large_symbolic_exponent_does_not_panic() {
    let out = support::line(&mut Env::default(), "(x^10000000000000000)^(1/2) ->");
    assert!(
        !out.prose,
        "must evaluate or report a calculation limit: {out:?}"
    );
    assert!(out.result.is_some() || !out.errors.is_empty(), "{out:?}");
}

#[test]
fn f23_vector_extrema_reductions() {
    real("min(vec(3,-2,7))", -2.0, 0.0);
    real("max(vec(3,-2,7))", 7.0, 0.0);
}

#[test]
fn f24_differentiate_piecewise_function_away_from_boundaries() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := diff(cases(x^2,x<0,x^3),x)");
    real_in(&mut env, "f(-2)", -4.0, 1e-12);
    real_in(&mut env, "f(2)", 12.0, 1e-12);
}

#[test]
fn f25_error_function_derivative() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := diff(erf(x),x)");
    real_in(&mut env, "f(0)", 2.0 / std::f64::consts::PI.sqrt(), 1e-12);
}

#[test]
#[ignore = "F26: digamma operation is absent (proposed API)"]
fn f26_digamma_recurrence() {
    // psi(x+1)-psi(x)=1/x avoids a rounded Euler-constant fixture.
    real("⦇digamma⦈(2)-⦇digamma⦈(1)", 1.0, 1e-12);
}

#[test]
fn f27_bessel_derivative_recurrence() {
    let mut env = Env::default();
    support::run(&mut env, "f(x) := diff(besselj(0,x),x) + besselj(1,x)");
    real_in(&mut env, "f(1)", 0.0, 1e-12);
}

#[test]
#[ignore = "F28: Bessel orders are restricted to integers"]
fn f28_half_integer_bessel_order() {
    // J_(1/2)(pi/2) = sqrt(2/(pi*pi/2))*sin(pi/2) = 2/pi.
    real("besselj(1/2,pi/2)", 2.0 / std::f64::consts::PI, 1e-12);
}

#[test]
#[ignore = "F29: erf is restricted to real arguments"]
fn f29_complex_error_function() {
    // erf(i) / i = 2/sqrt(pi) * sum_(k>=0) 1/(k!*(2k+1)).
    let mut factorial = 1.0;
    let series: f64 = (0..40)
        .map(|k| {
            if k > 0 {
                factorial *= k as f64;
            }
            1.0 / (factorial * (2 * k + 1) as f64)
        })
        .sum();
    let got = evaluated(&mut Env::default(), "erf(#i)");
    close(got.re.v.f(), 0.0, 1e-12);
    close(
        got.im.v.f(),
        2.0 * series / std::f64::consts::PI.sqrt(),
        1e-12,
    );
}

#[test]
#[ignore = "F30: gamma is restricted to real arguments"]
fn f30_complex_gamma_magnitude() {
    real(
        "abs(gamma(1+#i))^2",
        std::f64::consts::PI / std::f64::consts::PI.sinh(),
        1e-12,
    );
}

#[test]
#[ignore = "F31: inverse hyperbolic operations are absent (proposed APIs)"]
fn f31_inverse_hyperbolic_operations() {
    real("⦇asinh⦈(sinh(1/2))", 0.5, 1e-12);
    real("⦇acosh⦈(cosh(1/2))", 0.5, 1e-12);
    real("⦇atanh⦈(tanh(1/2))", 0.5, 1e-12);
}

#[test]
#[ignore = "F32: limit operation is absent (proposed API)"]
fn f32_removable_singularity_limit() {
    real("⦇limit⦈(sin(x)/x,x,0)", 1.0, 1e-12);
}

#[test]
fn f33_taylor_polynomial_through_requested_degree() {
    identity("⦇taylor⦈(exp(x),x,0,4) = 1+x+x^2/2+x^3/6+x^4/24 ->");
}

#[test]
fn f34_vector_function_jacobian() {
    let mut env = Env::default();
    support::run(&mut env, "J(x,y) := ⦇jacobian⦈(vec(x*y,x+y),vec(x,y))");
    let got = entries(&mut env, "J(2,3)", (2, 2));
    for (got, want) in got.iter().zip([3.0, 2.0, 1.0, 1.0]) {
        assert!(got.is_real());
        close(got.re.v.f(), want, 1e-12);
    }
}

#[test]
fn f35_multivariate_hessian() {
    array(
        "⦇hessian⦈(x^2+x*y+3*y^2,vec(x,y))",
        (2, 2),
        &[2.0, 1.0, 1.0, 6.0],
    );
}

#[test]
#[ignore = "F36: eigenvectors are absent despite eigenvalue support (proposed API)"]
fn f36_eigenvectors_form_a_normalized_eigenbasis() {
    let basis = entries(&mut Env::default(), "⦇eigenvecs⦈(mat(2,0;0,3))", (2, 2));
    // Accept either eigenvalue order and either sign/complex phase of each column.
    let magnitudes: Vec<f64> = basis.iter().map(|z| z.z().abs()).collect();
    for j in 0..2 {
        close(
            magnitudes[j].powi(2) + magnitudes[2 + j].powi(2),
            1.0,
            1e-10,
        );
        close(magnitudes[j] * magnitudes[2 + j], 0.0, 1e-10);
    }
    close(magnitudes[0].powi(2) + magnitudes[1].powi(2), 1.0, 1e-10);
}

#[test]
#[ignore = "F37: Moore-Penrose pseudoinverse is absent (proposed API)"]
fn f37_rank_deficient_matrix_pseudoinverse() {
    array("⦇pinv⦈(mat(1,2;2,4))", (2, 2), &[0.04, 0.08, 0.08, 0.16]);
}

#[test]
#[ignore = "F38: lsolve requires full column rank and disallows minimum-norm solutions"]
fn f38_minimum_norm_linear_solutions() {
    array("lsolve(mat(1,1),vec(2))", (2, 1), &[1.0, 1.0]);
    array("lsolve(mat(1,1;2,2),vec(1,2))", (2, 1), &[0.5, 0.5]);
}

#[test]
#[ignore = "F39: sample covariance operation is absent (proposed API)"]
fn f39_sample_covariance() {
    real("⦇covariance⦈(vec(1,2,3),vec(2,4,6))", 2.0, 1e-12);
}

#[test]
#[ignore = "F40: Pearson correlation operation is absent (proposed API)"]
fn f40_pearson_correlation() {
    real("⦇correlation⦈(vec(1,2,3),vec(-2,-4,-6))", -1.0, 1e-12);
}

#[test]
#[ignore = "F41: weighted mean operation is absent (proposed API)"]
fn f41_weighted_mean() {
    real("⦇weightedmean⦈(vec(1,3),vec(1,3))", 2.5, 1e-12);
}

#[test]
#[ignore = "F42: empirical quantile operation is absent (proposed API)"]
fn f42_linearly_interpolated_empirical_quantile() {
    // Proposed convention: index (n-1)*p, linear interpolation between sorted observations.
    real("⦇quantile⦈(vec(20,0,10),1/4)", 5.0, 1e-12);
}

#[test]
#[ignore = "F43: natural cubic spline interpolation is absent (proposed API)"]
fn f43_natural_cubic_spline_interpolation() {
    // Natural endpoint second derivatives are zero; on [0,1], S(x)=3x/2-x^3/2.
    real("⦇spline⦈(vec(0,1,2),vec(0,1,0),1/2)", 11.0 / 16.0, 1e-12);
}

#[test]
#[ignore = "F44: odesolve accepts only one scalar state"]
fn f44_coupled_ode_system() {
    array(
        "odesolve(vec(v,-u),vec(u,v),t,vec(1,0),0,pi/2)",
        (2, 1),
        &[0.0, -1.0],
    );
}

#[test]
#[ignore = "F45: odesolve cannot return states at a vector of requested times"]
fn f45_ode_sampled_trajectory() {
    array(
        "odesolve(-y,y,t,1,0,vec(0,1,2))",
        (3, 1),
        &[1.0, (-1.0_f64).exp(), (-2.0_f64).exp()],
    );
}

#[test]
#[ignore = "F46: odesolve rejects complex states and right sides"]
fn f46_complex_scalar_ode() {
    let got = evaluated(&mut Env::default(), "odesolve(#i*y,y,t,1,0,1)");
    close(got.re.v.f(), 1.0_f64.cos(), 1e-8);
    close(got.im.v.f(), 1.0_f64.sin(), 1e-8);
}

#[test]
#[ignore = "F47: minimize accepts only one scalar variable"]
fn f47_multivariate_box_constrained_minimization() {
    array(
        "minimize((x-1)^2+(y+2)^2,vec(x,y),vec(-3,-3),vec(3,3))",
        (2, 1),
        &[1.0, -2.0],
    );
}

#[test]
#[ignore = "F48: solve cannot be combined with domain assumptions"]
fn f48_solve_filters_roots_by_positive_assumption() {
    let mut env = Env::default();
    let out = support::run(&mut env, "x^2=1 -> assume(x>0),solve(x)");
    assert_eq!(out.solutions, vec![vec![1.0]], "{out:?}");
    let got = complex_value(&support::bind(&env, &env.vars["x"].expr)).unwrap();
    close(got.re.v.f(), 1.0, 0.0);
}

#[test]
#[ignore = "F49: polyroots requires unitless coefficients"]
fn f49_polynomial_roots_with_dimensioned_coefficients() {
    let mut roots = entries(&mut Env::default(), "polyroots(vec(-4 m^2,0,1))", (2, 1));
    roots.sort_by(|a, b| a.re.v.f().total_cmp(&b.re.v.f()));
    for (got, want) in roots.iter().zip([-2.0, 2.0]) {
        assert!(got.is_real());
        assert_eq!(got.d(), &Dim::base(0));
        close(got.re.v.f(), want, 1e-10);
    }
}

#[test]
fn f50_sine_of_arcsine_symbolic_identity() {
    // This direction is valid on the principal complex continuation, unlike asin(sin(x)).
    identity("sin(arcsin(x)) = x ->");
}
