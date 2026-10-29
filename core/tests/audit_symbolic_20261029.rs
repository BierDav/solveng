//! Confirmed defects from the second audit; assertions state the desired behavior.
mod support;
use solveng_core::{env::Env, eval::complex_value, expr::simp};

fn number(source: &str) -> solveng_core::complex::CVal {
    let mut env = Env::default();
    support::run(&mut env, &format!("answer := {source}"));
    complex_value(&simp(&support::bind(&env, &env.vars["answer"].expr))).unwrap()
}
fn near(actual: f64, expected: f64) {
    assert!((actual-expected).abs() <= 1e-10, "{actual} != {expected}");
}
#[test]
fn s01_nested_inverse_preserves_all_preimages() {
    let o = support::run(&mut Env::default(), "sqrt(x^2)=1 -> solve(x)");
    assert_eq!(o.solutions.len(),2,"{o:?}");
}
#[test]
fn s02_taylor_rejects_undefined_coefficients() {
    for expression in ["abs(x)", "sqrt(x)", "ln(x)", "1/x"] {
        let o=support::line(&mut Env::default(), &format!("taylor({expression},x,0,2) ->"));
        assert!(!o.errors.is_empty(),"{expression}: {o:?}");
    }
}
#[test]
fn s03_reciprocal_complex_affine_primitive_preserves_phase() {
    let v=number("integrate(1/(x+#i),x,0,1)");
    near(v.re.v.f(), std::f64::consts::LN_2/2.0);
    near(v.im.v.f(), -std::f64::consts::FRAC_PI_4);
}
#[test]
fn s04_tangent_complex_primitive_preserves_phase() {
    let v=number("integrate(tan(x+#i),x,0,1)");
    let expected=solveng_core::complex::Z::new(0.0,1.0).cos().ln().sub(solveng_core::complex::Z::new(1.0,1.0).cos().ln());
    near(v.re.v.f(),expected.re); near(v.im.v.f(),expected.im);
}
#[test]
fn s05_cardinal_higher_derivatives_have_continuation_values() {
    let mut env=Env::default();
    support::run(&mut env,"f(x) := diff(diff(sinc(x),x),x)");
    support::run(&mut env,"answer := f(0) =>");
    near(complex_value(&support::bind(&env,&env.vars["answer"].expr)).unwrap().re.v.f(),-1.0/3.0);
    support::run(&mut env,"g(x) := diff(diff(tanc(x),x),x)");
    support::run(&mut env,"answer := g(0) =>");
    near(complex_value(&support::bind(&env,&env.vars["answer"].expr)).unwrap().re.v.f(),2.0/3.0);
    support::run(&mut env,"h(x) := taylor(sinc(x),x,0,4)");
    support::run(&mut env,"answer := h(1/10) =>");
    near(complex_value(&support::bind(&env,&env.vars["answer"].expr)).unwrap().re.v.f(),1.0-0.01/6.0+0.0001/120.0);
}
#[test]
fn s06_piecewise_derivative_is_undefined_at_corner() {
    let mut env=Env::default();
    support::run(&mut env,"f(x) := cases(x^2,x<0,x)");
    support::run(&mut env,"g(x) := diff(f(x),x)");
    let o=support::line(&mut env,"g(0) =>");
    assert!(!o.errors.is_empty(),"{o:?}");
}
#[test]
fn s07_inverse_residual_validation_is_scale_aware() {
    let o=support::run(&mut Env::default(),"sqrt(x)=-1/10000000000 -> solve(x)");
    assert!(o.solutions.is_empty(),"{o:?}");
    assert!(o.result.unwrap().contains("no solution"));
}
#[test]
fn s08_tiny_polynomial_complex_roots_are_not_discarded() {
    let o=support::run(&mut Env::default(),"x^3=10^(-60) -> solve(x)");
    let result=o.result.unwrap();
    assert_eq!(result.matches("x_(").count(),3,"{result}");
}
#[test]
fn s09_clustered_cubic_has_exactly_three_roots() {
    let o=support::run(&mut Env::default(),"(x-1)^3=10^(-30) -> solve(x)");
    let result=o.result.unwrap();
    assert_eq!(result.matches("x_(").count(),3,"{result}");
}
#[test]
fn s10_integrable_interior_log_singularity_is_split() {
    near(number("integrate(ln(abs(x)),x,-1,1)").re.v.f(),-2.0);
}
#[test]
fn s11_complex_log_primitives_respect_path_continuation() {
    near(number("integrate(1/(#i*x-1),x,-1,1)").re.v.f(),-std::f64::consts::FRAC_PI_2);
    near(number("integrate(tan(x+#i),x,0,2*pi)").im.v.f(),std::f64::consts::TAU);
}
fn function_value(body: &str, call: &str) -> solveng_core::complex::CVal {
    let mut env=Env::default(); support::run(&mut env,&format!("f(a,x) := {body}"));
    support::run(&mut env,&format!("answer := {call} =>"));
    complex_value(&support::bind(&env,&env.vars["answer"].expr)).unwrap()
}
#[test]
fn s12_abs_stationary_inner_derivative_exists() {
    near(function_value("diff(abs(x^3),x)","f(0,0)").re.v.f(),0.0);
}
#[test]
fn s13_square_root_smooth_composition_derivative_exists() {
    near(function_value("diff(sqrt(x^4),x)","f(0,0)").re.v.f(),0.0);
}
#[test]
fn s14_indefinite_integral_retains_parameter_degeneracy() {
    near(function_value("integrate(exp(a*x),x)","f(0,2)").re.v.f(),2.0);
    near(function_value("integrate(x^a,x)","f(-1,2)").re.v.f(),2.0_f64.ln());
}
#[test]
fn s15_inverse_laplace_frequency_continues_at_zero() {
    near(function_value("invlaplace(1/(s^2+a^2),s,x)","f(0,2)").re.v.f(),2.0);
}
#[test]
fn s16_signal_laplace_scale_continues_at_zero() {
    near(function_value("laplace(rect(a*t),t,x)","f(0,2)").re.v.f(),0.5);
}
#[test]
fn s17_signal_laplace_requires_real_arguments() {
    let o=support::line(&mut Env::default(),"laplace(rect(#i*t),t,s) ->");
    assert!(!o.errors.is_empty(),"{o:?}");
}

#[test]
fn s17_deferred_signal_scale_also_requires_real_values() {
    let mut env=Env::default();
    support::run(&mut env,"f(a,x) := laplace(rect(a*t),t,x)");
    let o=support::line(&mut env,"f(#i,2) =>");
    assert!(!o.errors.is_empty(),"{o:?}");
}
