//! Executable completion criteria for the Mathcad gap audit.
//! Run: cargo test -p solveng-core --test mathcad_gaps
//! These criteria run in normal CI. Public operation contracts are documented
//! in src/builtins.typ and docs/reference.md.
mod support;

use solveng_core::{
    env::Env,
    eval::complex_value,
    expr::{Expr, simp},
    linalg::Array,
    poly::{coeffs, together},
};

fn expression(env: &mut Env, source: &str) -> Expr {
    support::run(env, &format!("answer := {source}"));
    simp(&support::bind(env, &env.vars["answer"].expr))
}

fn near_in(env: &mut Env, source: &str, expected: f64, tolerance: f64) {
    let expr = expression(env, source);
    let got = complex_value(&expr).unwrap_or_else(|e| panic!("{source}: {e:?}"));
    assert!(got.is_real(), "{source}: expected real, got {got:?}");
    assert!(got.d().is_one(), "{source}: expected dimensionless result");
    let got = got.re.v.f();
    assert!(
        got.is_finite() && (got - expected).abs() <= tolerance,
        "{source}: got {got:.17}, expected {expected:.17}, absolute tolerance {tolerance}"
    );
}

fn near(source: &str, expected: f64, tolerance: f64) {
    near_in(&mut Env::default(), source, expected, tolerance);
}

fn array(source: &str, shape: (usize, usize), expected: &[f64], tolerance: f64) {
    let expr = expression(&mut Env::default(), source);
    let got = Array::of(&expr).unwrap_or_else(|| panic!("{source}: expected array, got {expr:?}"));
    assert_eq!((got.rows, got.cols), shape, "{source}");
    assert_eq!(got.a.len(), expected.len());
    for (i, (entry, expected)) in got.a.iter().zip(expected).enumerate() {
        let value = complex_value(entry).unwrap_or_else(|e| panic!("{source}[{i}]: {e:?}"));
        assert!(
            value.is_real() && (value.re.v.f() - expected).abs() <= tolerance,
            "{source}[{i}]: got {value:?}, expected {expected}"
        );
    }
}

#[test]
fn oscillatory_improper_integrals() {
    // Analytic Laplace integral; unsplit numerical oracles can alias the tail.
    for frequency in [8.0_f64, 28.0, 36.0] {
        near(
            &format!("integrate(exp(-t)*sin({frequency}*t), t, 0, infinity)"),
            frequency / (1.0 + frequency * frequency),
            1e-9,
        );
    }
}

#[test]
fn logarithmic_endpoint_integrals() {
    for upper in [1.5_f64, 0.75, 0.2] {
        near(
            &format!("integrate(ln(t), t, 0, {upper})"),
            upper * (upper.ln() - 1.0),
            1e-9,
        );
    }
}

#[test]
fn slowly_converging_series() {
    near(
        "sum(1/k^2, k, 1, infinity)",
        std::f64::consts::PI.powi(2) / 6.0,
        1e-8,
    );
    near(
        "sum((-1)^k/(2 k + 1), k, 0, infinity)",
        std::f64::consts::PI / 4.0,
        1e-8,
    );
    near("sum((-1)^(k + 1)/k, k, 1, infinity)", 2.0_f64.ln(), 1e-8);
}

#[test]
fn repeated_polynomial_roots() {
    // Expanded oracle fixture: factoring the test input would bypass the affected path.
    let out = support::run(
        &mut Env::default(),
        "-27 - x^5 + 9 x^4 - 81/4 x^3 - 19/2 x^2 + 195/4 x = 0 -> solve(x)",
    );
    let mut roots: Vec<f64> = out
        .solutions
        .iter()
        .map(|s| {
            assert_eq!(s.len(), 1);
            s[0]
        })
        .collect();
    roots.sort_by(f64::total_cmp);
    assert_eq!(roots.len(), 4, "distinct real roots: {roots:?}");
    for (got, expected) in roots.iter().zip([-1.5, 1.0, 4.0, 4.5]) {
        assert!(
            (got - expected).abs() <= 1e-10,
            "{roots:?}: root near {expected}"
        );
    }
}

#[test]
fn parameterized_cubic_solutions() {
    let mut env = Env::default();
    support::run(&mut env, "roots(p, q) := x^3 + p x + q = 0 -> solve(x)");
    for (p, q) in [(1.0, 0.25), (1.5, -1.0)] {
        let expr = expression(&mut env, &format!("roots({p}, {q})"));
        let roots = Array::of(&expr).expect("all three cubic roots");
        assert_eq!(roots.a.len(), 3);
        let mut distinct = vec![];
        for root in &roots.a {
            let z = complex_value(root).unwrap();
            let z = solveng_core::complex::Z::new(z.re.v.f(), z.im.v.f());
            assert!(
                z.mul(z)
                    .mul(z)
                    .add(z.scale(p))
                    .add(solveng_core::complex::Z::real(q))
                    .abs()
                    < 1e-9
            );
            assert!(
                !distinct
                    .iter()
                    .any(|other: &solveng_core::complex::Z| z.sub(*other).abs() < 1e-8)
            );
            distinct.push(z);
        }
    }
}

#[test]
fn multivariate_rational_cancellation() {
    let fixture = include_str!("oracle/identities.txt")
        .lines()
        .find(|line| line.split('\t').nth(1) == Some("cancel-5"))
        .unwrap();
    let source = fixture.split('\t').nth(2).unwrap();
    let mut env = Env::default();
    support::run(&mut env, &format!("answer := {source} -> simplify"));
    let (numerator, denominator) = together(&env.vars["answer"].expr);
    for variable in ["p", "x", "y"] {
        assert!(
            coeffs(&numerator, variable).unwrap().len() <= 3,
            "degree <= 2 in {variable}"
        );
        assert!(
            !denominator.has(variable),
            "uncancelled denominator in {variable}: {denominator:?}"
        );
    }
}

#[test]
fn symbolic_identities_simplify_to_constants() {
    for source in ["sin(x)^2 + cos(x)^2", "cosh(x)^2 - sinh(x)^2"] {
        let mut env = Env::default();
        support::run(&mut env, &format!("answer := {source} -> simplify"));
        assert_eq!(
            simp(&env.vars["answer"].expr),
            solveng_core::expr::int(1),
            "{source}"
        );
    }
}

#[test]
fn quadratic_inequality_interval() {
    let rendered = support::result(&mut Env::default(), "x^2 < 1 -> solve(x)");
    // Canonical interval rendering; endpoints must be open, not <=.
    assert_eq!(rendered, "\"−1\" < x < \"1\"");
}

#[test]
fn positive_assumptions_are_local_and_branch_safe() {
    let mut env = Env::default();
    let before = env.clone();
    assert_eq!(support::result(&mut env, "sqrt(x^2) -> assume(x > 0)"), "x");
    assert_eq!(env, before);
    assert_ne!(support::result(&mut env, "sqrt(x^2) ->"), "x");
}

#[test]
fn overdetermined_linear_least_squares() {
    array(
        "⦇lsolve⦈(mat(1, 0; 1, 1; 1, 2), vec(1, 2, 2))",
        (2, 1),
        &[7.0 / 6.0, 0.5],
        1e-10,
    );
}

#[test]
fn constrained_minimization() {
    // Binder: minimize(objective, variable, lower, upper), returns argmin.
    near("⦇minimize⦈((x - 3)^2, x, 0, 2)", 2.0, 1e-8);
    near("⦇minimize⦈((x + 1)^2, x, -4, 4)", -1.0, 1e-8);
}

#[test]
fn matrix_eigenvalues() {
    let expr = expression(&mut Env::default(), "⦇eigenvals⦈(mat(2, 1; 1, 2))");
    let values = Array::of(&expr).expect("eigenvalue vector");
    assert_eq!((values.rows, values.cols), (2, 1));
    let mut values: Vec<f64> = values
        .a
        .iter()
        .map(|e| {
            let z = complex_value(e).unwrap();
            assert!(z.is_real());
            z.re.v.f()
        })
        .collect();
    values.sort_by(f64::total_cmp);
    for (got, expected) in values.iter().zip([1.0, 3.0]) {
        assert!((got - expected).abs() < 1e-10);
    }
}

#[test]
fn rectangular_matrix_singular_values() {
    array("⦇singularvals⦈(mat(3, 0; 0, 4; 0, 0))", (2, 1), &[4.0, 3.0], 1e-10);
    array(
        "⦇singularvals⦈(mat(1, 1; 2, 2; 3, 3))",
        (2, 1),
        &[28.0_f64.sqrt(), 0.0],
        1e-10,
    );
}

#[test]
fn matrix_condition_number() {
    near("⦇cond⦈(mat(1, 0; 0, 1/100000000))", 100000000.0, 1e-3);
    near("⦇cond⦈(mat(2, 0; 0, 2))", 1.0, 1e-12);
}

#[test]
fn descriptive_statistics() {
    near("⦇median⦈(vec(9, 1, 3, 7))", 5.0, 1e-12);
    // Sample variance and standard deviation (n-1 denominator).
    near("⦇variance⦈(vec(1, 2, 3))", 1.0, 1e-12);
    near("⦇stdev⦈(vec(2, 4, 6))", 2.0, 1e-12);
}

#[test]
fn normal_distribution_and_quantiles() {
    near("⦇normalcdf⦈(0, 0, 1)", 0.5, 1e-12);
    near("⦇normalcdf⦈(1, 0, 1)", 0.8413447460685429, 1e-10);
    near("⦇normalquantile⦈(0.975, 0, 1)", 1.959963984540054, 1e-8);
}

#[test]
fn interpolation_of_measured_data() {
    near("⦇linterp⦈(vec(0, 2, 5), vec(1, 5, 11), 3)", 7.0, 1e-12);
    near("⦇linterp⦈(vec(0, 2, 5), vec(1, 5, 11), 2)", 5.0, 1e-12);
}

#[test]
fn linear_regression() {
    // Result: vector(intercept, slope), minimizing vertical residuals.
    array(
        "⦇linfit⦈(vec(0, 1, 2), vec(1, 2, 2))",
        (2, 1),
        &[7.0 / 6.0, 0.5],
        1e-10,
    );
    array(
        "⦇linfit⦈(vec(-1, 0, 1), vec(5, 3, 1))",
        (2, 1),
        &[3.0, -2.0],
        1e-10,
    );
}

#[test]
fn discrete_fourier_transform() {
    // Forward unnormalized DFT; inverse carries 1/n.
    array("⦇dft⦈(vec(1, 0, 0, 0))", (4, 1), &[1.0; 4], 1e-12);
    array(
        "⦇dft⦈(vec(1, 1, 1, 1))",
        (4, 1),
        &[4.0, 0.0, 0.0, 0.0],
        1e-12,
    );
    array(
        "⦇idft⦈(⦇dft⦈(vec(2, -1, 3, 0)))",
        (4, 1),
        &[2.0, -1.0, 3.0, 0.0],
        1e-12,
    );
}

#[test]
fn engineering_special_functions() {
    near("⦇gamma⦈(1/2)", std::f64::consts::PI.sqrt(), 1e-10);
    near("⦇erf⦈(1)", 0.8427007929497149, 1e-10);
    near("⦇besselj⦈(0, 1)", 0.7651976865579666, 1e-10);
}

#[test]
fn ode_initial_value_problem() {
    // Endpoint API: odesolve(rhs, state, time, initial, start, end).
    near("⦇odesolve⦈(-y, y, t, 1, 0, 1)", (-1.0_f64).exp(), 1e-8);
    near("⦇odesolve⦈(t, y, t, 2, 0, 2)", 4.0, 1e-8);
}

#[test]
fn absolute_temperatures_and_temperature_differences() {
    // Explicit affine conversions avoid silently changing existing degC differences.
    near("number(⦇celsius⦈(20), [K])", 293.15, 1e-10);
    near("number(⦇fahrenheit⦈(32), [K])", 273.15, 1e-10);
    near("number(⦇celsius⦈(30) - ⦇celsius⦈(20), [K])", 10.0, 1e-10);
    near("number(20 degC, [K])", 20.0, 1e-10);
}
