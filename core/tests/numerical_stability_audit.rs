//! Finite answers across the floating-point range, with independent identities.
mod support;
use solveng_core::{complex::{CVal, Z}, env::Env, eval::complex_value, expr::simp, linalg::Array};
fn values(source: &str) -> Vec<CVal> {
    let mut env = Env::default();
    support::run(&mut env, &format!("answer := {source}"));
    let expr = simp(&support::bind(&env, &env.vars["answer"].expr));
    let entries = Array::of(&expr).map_or_else(|| vec![expr], |a| a.a);
    entries.iter().map(|e| complex_value(e).unwrap()).collect()
}
fn near(actual: f64, expected: f64) {
    assert!(actual.is_finite() && (actual - expected).abs() <= expected.abs().max(1e-310) * 2e-12,
        "{actual} != {expected}");
}
#[test]
fn normal_probability_avoids_overflow_in_standardization() {
    near(values("pnorm(10^308,-10^308,10^308)")[0].re.v.f(), 0.9772498680518208);
    near(values("pnorm(-10^308,10^308,10^308)")[0].re.v.f(), 0.02275013194817921);
}
#[test]
fn interpolation_handles_intervals_wider_than_max_float() {
    near(values("linterp(vec(-10^308,10^308),vec(-1,1),0)")[0].re.v.f(), 0.0);
    near(values("linterp(vec(-10^308,10^308),vec(0,1),5*10^307)")[0].re.v.f(), 0.75);
}
#[test]
fn sample_deviation_scales_before_centering() {
    near(values("stdev(vec(-10^308,0,10^308))")[0].re.v.f(), 1e308);
    near(values("stdev(vec(-10^308,10^308))")[0].re.v.f(), 2f64.sqrt()*1e308);
}
#[test]
fn regression_scales_both_axes() {
    let v = values("linfit(vec(-10^308,10^308),vec(-1,1))");
    near(v[0].re.v.f(), 0.0); near(v[1].re.v.f(), 1e-308);
    let v = values("linfit(vec(-10^308,0,10^308),vec(-2,0,2))");
    near(v[1].re.v.f(), 2e-308);
}
#[test]
fn bounded_minimization_uses_finite_coordinates() {
    near(values("minimize(x/10^308,x,-10^308,10^308)")[0].re.v.f(), -1e308);
    near(values("minimize(-x/10^308,x,-10^308,10^308)")[0].re.v.f(), 1e308);
}
#[test]
fn ode_step_width_does_not_overflow() {
    near(values("odesolve(0,y,t,1,-10^308,10^308)")[0].re.v.f(), 1.0);
    near(values("odesolve(0,y,t,2,10^308,-10^308)")[0].re.v.f(), 2.0);
    near(values("odesolve(0,y,t,3,0,10^(-323))")[0].re.v.f(), 3.0);
}
#[test]
fn complex_inverse_sine_avoids_cancellation_and_overflow() {
    for x in [1e8_f64, -1e8, 1e200, -1e200] {
        let z = Z::new(0.0, x).asin(); near(z.re, 0.0); near(z.im, x.asinh());
    }
    near(values("asin(100000000*#i)")[0].im.v.f(), 1e8_f64.asinh());
    near(values("acos(100000000*#i)")[0].im.v.f(), -1e8_f64.asinh());
}
#[test]
fn complex_logarithm_does_not_require_representable_modulus() {
    let v = values("ln(13*10^307+13*10^307*#i)");
    near(v[0].re.v.f(), 1.3e308_f64.ln()+std::f64::consts::LN_2/2.0);
    near(v[0].im.v.f(), std::f64::consts::FRAC_PI_4);
    assert!(!Z::new(f64::INFINITY, 1.0).cleaned().re.is_finite());
}
#[test]
fn least_squares_normalizes_subnormal_components_directly() {
    near(values("lsolve(mat(10^(-308)),vec(10^(-308)))")[0].re.v.f(), 1.0);
    near(values("lsolve(mat(10^(-308),0;0,10^(-308)),vec(10^(-308),2*10^(-308)))")[1].re.v.f(), 2.0);
    near(values("lsolve(mat(13*10^307+13*10^307*#i),vec(13*10^307+13*10^307*#i))")[0].re.v.f(), 1.0);
}
#[test]
fn inverse_fourier_scales_before_accumulation() {
    let v = values("idft(vec(10^308,10^308))");
    near(v[0].re.v.f(), 1e308); near(v[1].re.v.f(), 0.0);
    let v = values("idft(vec(10^308,-10^308))");
    near(v[0].re.v.f(), 0.0); near(v[1].re.v.f(), 1e308);
}
