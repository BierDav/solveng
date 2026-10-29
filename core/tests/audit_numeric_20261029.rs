mod support;
use solveng_core::{
    complex::Z,
    env::Env,
    eval::complex_value,
    expr::simp,
    format::{Cfg, fmt},
    linalg::Array,
};
fn vals(s: &str) -> Vec<Z> {
    let mut e = Env::default();
    support::run(&mut e, &format!("answer := {s}"));
    let x = simp(&support::bind(&e, &e.vars["answer"].expr));
    Array::of(&x)
        .map_or_else(|| vec![x], |a| a.a)
        .iter()
        .map(|a| complex_value(a).unwrap().z())
        .collect()
}
fn near(a: f64, b: f64) {
    assert!(
        a.is_finite() && (a - b).abs() <= b.abs().max(1e-300) * 1e-10,
        "{a} != {b}"
    );
}
#[test]
fn n01_ceiling_minimum_exact_integer() {
    let mut e = Env::default();
    support::run(&mut e, "answer := ceil((-2)^127)");
}
#[test]
fn n02_inverse_sine_tiny_imaginary() {
    near(vals("Im(asin(#i*10^(-20)))")[0].re, 1e-20);
}
#[test]
fn n03_inverse_tangent_tiny_imaginary() {
    near(vals("Im(atan(#i*10^(-20)))")[0].re, 1e-20);
}
#[test]
fn n04_hyperbolic_tangent_saturation() {
    near(vals("tanh(1000+#i)")[0].re, 1.0);
}
#[test]
fn n05_quantile_cancelling_overflow() {
    near(
        vals("qnorm(0.99,-10^308,10^308)")[0].re,
        1.3263478740408408e308,
    );
}
#[test]
fn n06_polynomial_roots_extreme_scale() {
    let v = vals("polyroots(vec(10^(-200),0,10^200))");
    assert_eq!(v.len(), 2);
    for z in v {
        near(z.re, 0.0);
        near(z.im.abs(), 1e-200);
    }
}
#[test]
fn n07_power_subnormal_underflow() {
    let v = vals("10^(-320)")[0].re;
    assert!(v > 0.0, "representable subnormal lost");
    assert!((v / 1e-320 - 1.0).abs() < 1e-3);
}
#[test]
fn n08_format_subnormal() {
    let (m, e) = fmt(1e-320, &Cfg::default());
    assert!(!m.contains("inf") && !m.contains("NaN"), "{m}, {e:?}");
    let x = m.parse::<f64>().unwrap() * 10f64.powf(e.unwrap_or(0) as f64);
    assert!((x / 1e-320 - 1.0).abs() < 1e-3, "{m}, {e:?}");
}
#[test]
fn n09_quadratic_intermediate_overflow() {
    let mut e = Env::default();
    let o = support::line(&mut e, "x^2+10^200*x+1=0 -> solve(x)");
    assert!(o.errors.is_empty(), "{:?}", o.errors);
    assert_eq!(o.solutions.len(), 2, "{o:?}");
    near(o.solutions[0][0], -1e200);
    near(o.solutions[1][0], -1e-200);
}
#[test]
fn n10_complex_result_projection() {
    let z = vals("acos(1+#i*10^(-20))")[0];
    assert!((z.im / -1e-10 - 1.0).abs() < 1e-6, "{z:?}");
    assert!((z.re / 1e-10 - 1.0).abs() < 1e-6, "{z:?}");
}

#[test]
fn representative_variations_of_stable_algorithms() {
    for x in [1e-30_f64, -1e-30, 0.25, -0.25] {
        near(vals(&format!("Im(asin({x:.30}*#i))"))[0].re, x.asinh());
    }
    for z in [Z::new(1e-30, 1e-30), Z::new(-1e-30, -1e-30)] {
        near(z.asin().re, z.re);
        near(z.asin().im, z.im);
        near(z.atan().re, z.re);
        near(z.atan().im, z.im);
    }
    near(vals("tanh(-1000+#i)")[0].re, -1.0);
    near(
        vals("qnorm(0.01,10^308,10^308)")[0].re,
        -1.3263478740408408e308,
    );
    let roots = vals("polyroots(vec(-10^200,0,10^(-200)))");
    near(roots[0].re, -1e200);
    near(roots[1].re, 1e200);
    for z in vals("polyroots(vec(10^(-200),0,0,0,10^200))") {
        near(z.abs(), 1e-100);
    }
    let (m, e) = fmt(f64::from_bits(1), &Cfg::default());
    assert_eq!(e, Some(-324));
    assert_eq!(m, "4.941");
    let engineering = Cfg {
        engineering: true,
        ..Cfg::default()
    };
    let (m, e) = fmt(1e-320, &engineering);
    assert_eq!(e, Some(-321));
    assert_eq!(m, "10");
}
