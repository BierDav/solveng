mod support;
use solveng_core::{env::Env, eval::{complex_value, eval_f}, expr::{self, Expr, simp}, num::Num, poly};
use support::run;

fn result(source: &str) -> String {
    run(&mut Env::default(), source).result.unwrap()
}

#[test]
fn entire_trigonometric_and_exponential_identities_are_exact() {
    for equation in [
        "#e^(#i*x) = cos(x) + #i*sin(x)",
        "#e^(#i*x) + #e^(-#i*x) = 2*cos(x)",
        "sin(x)^2 + cos(x)^2 = 1",
        "cos(2*x) = 2*cos(x)^2 - 1",
        "sin(2*x) = 2*sin(x)*cos(x)",
        "cos(x+y) = cos(x)*cos(y) - sin(x)*sin(y)",
        "exp(x+y) = exp(x)*exp(y)",
        "exp(x/2)^2 = exp(x)",
        "cosh(x)^2 - sinh(x)^2 = 1",
        "sinc(x)*x = sin(x)",
        "sinc(x) = cases(sin(x)/x, x != 0, 1)",
        "sinc(2*x+1) = cases(sin(2*x+1)/(2*x+1), x != -1/2, 1)",
    ] {
        assert_eq!(result(&format!("{equation} ->")), "\"true\"", "{equation}");
    }
}

#[test]
fn exceptional_points_and_branch_sensitive_functions_are_not_erased() {
    for equation in [
        "sinc(x) = cases(sin(x)/x, x != 0, 0)",
        "sinc(x) = cases(sin(x)/x, x != 0, 2)",
        "sinc(2*x+1) = cases(sin(2*x+1)/(2*x+1), x != -1/2, 0)",
        "sin(x)^2 + cos(x)^2 = 2",
        "exp(x+y) = exp(x) + exp(y)",
        "sqrt(x^2) = x",
        "ln(exp(x)) = x + 2*pi*#i",
        "sinc(x) = sinc(x) + 1/1000000000000",
    ] {
        assert_ne!(result(&format!("{equation} ->")), "\"true\"", "{equation}");
    }
}

#[test]
fn shifted_pulse_spectrum_matches_its_fourier_integral() {
    let mut env = Env::default();
    for source in [
        "p_1(t, t_0) := rect(t-t_0) + tri(2(t-t_0))",
        "p_2(t, t_0) := convolve(p_1(t, 0), dirac(t-t_0), t)",
        "s_2(t) := p_2(t, 1.5) + p_2(t, 3.5) + p_2(t, -1.5) + p_2(t, -3.5)",
        "P(f) := sinc(pi*f) + 1/2*sinc(pi*f/2)^2",
        "u(f, t_0) := #e^(-#i*2*t_0*pi*f)",
        "S_1(f) := P(f)*(u(f, 1.5) + u(f, 3.5) + u(f, -1.5) + u(f, -3.5))",
        "S_2(f) := integral_(-infinity)^infinity s_2(t)*#e^(-#i*2*pi*f*t) dif t",
    ] { run(&mut env, source); }
    let out = run(&mut env, "S_1(f) = S_2(f) ->");
    assert_eq!(out.result.as_deref(), Some("\"true\""), "{out:?}");
    run(&mut env, "S_c(f) := 2*P(f)*(cos(3*pi*f) + cos(7*pi*f))");
    assert_eq!(run(&mut env, "S_c(f) = S_2(f) ->").result.as_deref(), Some("\"true\""));
    assert_eq!(run(&mut env, "S_1(0) = S_2(0) ->").result.as_deref(), Some("\"true\""));
    let compact = run(&mut env, "S_2(f) ->").result.unwrap();
    assert!(compact.contains("sinc(") && compact.contains("cos(") && !compact.contains("cases(") && compact.len() < 350, "{compact}");
    // Also prove equality to the quotient form independently of the new transform table.
    run(&mut env, "S_raw(f) := cases(2*(cos(3*pi*f)+cos(7*pi*f))*(sin(pi*f)/(pi*f)+(1-cos(pi*f))/(pi*f)^2), f != 0, 6)");
    assert_eq!(run(&mut env, "S_1(f) = S_raw(f) ->").result.as_deref(), Some("\"true\""));
    let before = env.clone();
    let steps = run(&mut env, "S_1(f) = S_2(f) -> steps").result.unwrap();
    assert!(steps.ends_with("\"true\""), "{steps}");
    assert_eq!(env, before, "proof normalization must not change definitions");
}

#[test]
fn guarded_quotients_become_cardinals_but_keep_actual_poles_and_wrong_continuations() {
    for (source, want) in [
        ("cases(sin(x)/x, x != 0, 1)", "sinc(x)"),
        ("cases((1-cos(x))/x^2, x != 0, 1/2)", "1/2*sinc(x/2)^2"),
    ] {
        let mut env = Env::default();
        run(&mut env, &format!("F(x) := {source}"));
        run(&mut env, &format!("G(x) := {want}"));
        assert_eq!(poly::simplify(&env.funs["F"].body), poly::simplify(&env.funs["G"].body));
        assert!(!run(&mut env, "F(x) ->").result.unwrap().contains("cases("));
    }
    for source in [
        "cases(sin(x)/x, x != 0, 0)",
        "cases(sin(x)/x^2, x != 0, 1)",
        "cases((1-cos(x))/x^3, x != 0, 0)",
    ] {
        assert!(result(&format!("{source} ->")).contains("cases("), "{source}");
    }
    assert!(!result("sin(x)/x ->").contains("sinc("), "unguarded quotient has no continuation at zero");
}

fn simpson(f: impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let n = 2000;
    let h = (b-a)/n as f64;
    (f(a) + f(b) + (1..n).map(|k| (if k%2 == 0 { 2.0 } else { 4.0 })*f(a+k as f64*h)).sum::<f64>())*h/3.0
}

#[test]
fn scaled_shifted_and_weighted_pulse_transforms_match_independent_quadrature() {
    // Integrate each half separately so Simpson's rule never crosses a triangle's corner.
    for triangle in [false, true] {
        for scale in [-3.0_f64, 0.5, 2.0] {
            for centre in [-1.5_f64, 0.0, 2.5] {
                for degree in [0, 1, 2] {
                    let mut env = Env::default();
                    let signal = if triangle { "tri" } else { "rect" };
                    run(&mut env, &format!("F(w) := integral_(-infinity)^infinity t^{degree}*{signal}({scale}*(t-({centre})))*exp(-#i*w*t) dif t"));
                    assert!(!env.funs["F"].body.contains(&|e| matches!(e, Expr::Integral(..))));
                    let radius = if triangle { 1.0 } else { 0.5 } / scale.abs();
                    for w in [0.0_f64, 0.4, -1.3] {
                        let actual = complex_value(&simp(&env.funs["F"].body.subst("w", &expr::c(Num::F(w))))).unwrap();
                        let base = |t: f64| t.powi(degree)*(if triangle { 1.0-(scale*(t-centre)).abs() } else { 1.0 });
                        let re = |t: f64| base(t)*(w*t).cos();
                        let im = |t: f64| -base(t)*(w*t).sin();
                        let expected_re = simpson(re, centre-radius, centre) + simpson(re, centre, centre+radius);
                        let expected_im = simpson(im, centre-radius, centre) + simpson(im, centre, centre+radius);
                        assert!((actual.re.v.f()-expected_re).abs() < 3e-8, "{signal}, a={scale}, centre={centre}, degree={degree}, w={w}: {actual:?}, expected {expected_re}");
                        assert!((actual.im.v.f()-expected_im).abs() < 3e-8);
                    }
                }
            }
        }
    }
}

#[test]
fn finite_exponential_moments_include_zero_frequency_and_reversed_bounds() {
    for degree in [0, 1, 2] {
        let mut env = Env::default();
        run(&mut env, &format!("F(w) := integral_(-2)^3 t^{degree}*exp(-#i*w*t) dif t"));
        run(&mut env, &format!("G(w) := integral_3^(-2) t^{degree}*exp(-#i*w*t) dif t"));
        for w in [0.0_f64, 0.2, -1.1] {
            let sample = expr::c(Num::F(w));
            let actual = complex_value(&simp(&env.funs["F"].body.subst("w", &sample))).unwrap();
            let reverse = complex_value(&simp(&env.funs["G"].body.subst("w", &sample))).unwrap();
            assert!((actual.re.v.f()-simpson(|t| t.powi(degree)*(w*t).cos(), -2.0, 3.0)).abs() < 1e-8);
            assert!((actual.im.v.f()+simpson(|t| t.powi(degree)*(w*t).sin(), -2.0, 3.0)).abs() < 1e-8);
            assert!((actual.re.v.f()+reverse.re.v.f()).abs() < 1e-9);
            assert!((actual.im.v.f()+reverse.im.v.f()).abs() < 1e-9);
        }
        assert!(eval_f(&env.funs["F"].body, &|n| (n == "w").then_some(0.0)).is_some());
    }
}

#[test]
fn pulse_transform_units_and_complex_frequencies_are_preserved() {
    let mut env = Env::default();
    run(&mut env, "F(w: Hz) := integral_(-infinity)^infinity rect(t/(2 s)-3/2)*exp(-#i*w*t) dif t");
    run(&mut env, "answer := F(0 Hz) =>");
    let value = complex_value(&env.vars["answer"].expr).unwrap();
    assert_eq!(value.re.v.f(), 2.0);
    assert_eq!(value.re.d, solveng_core::dim::Dim::named("time"));
    run(&mut env, "Q(z) := integral_(-infinity)^infinity tri(2*(t-1))*exp(z*t) dif t");
    for z in ["0", "1/2", "1/2 + 3/4*#i"] {
        run(&mut env, &format!("answer := Q({z}) =>"));
        let actual = complex_value(&env.vars["answer"].expr).unwrap();
        let (re, im) = if z == "0" { (0.0, 0.0) } else if z == "1/2" { (0.5, 0.0) } else { (0.5, 0.75) };
        let base = |t: f64| (1.0-2.0*(t-1.0).abs())*(re*t).exp();
        let expected_re = simpson(|t| base(t)*(im*t).cos(), 0.5, 1.0) + simpson(|t| base(t)*(im*t).cos(), 1.0, 1.5);
        let expected_im = simpson(|t| base(t)*(im*t).sin(), 0.5, 1.0) + simpson(|t| base(t)*(im*t).sin(), 1.0, 1.5);
        assert!((actual.re.v.f()-expected_re).abs() < 1e-10);
        assert!((actual.im.v.f()-expected_im).abs() < 1e-10);
    }
}

#[test]
fn pulse_kernels_and_dirac_sifting_share_the_compact_transform_rules() {
    let mut env = Env::default();
    for (integrand, expected) in [
        ("rect(t)*cos(w*t)", "sinc(w/2)"),
        ("rect(t)*sin(w*t)", "0"),
        ("rect(t-2)*cos(w*t)", "cos(2*w)*sinc(w/2)"),
        ("rect(t-2)*sin(w*t)", "sin(2*w)*sinc(w/2)"),
        ("dirac(-2*(t-3))*exp(-#i*w*t)", "1/2*exp(-#i*3*w)"),
        ("t^2*dirac(2*(t-3))*exp(-#i*w*t)", "9/2*exp(-#i*3*w)"),
    ] {
        run(&mut env, &format!("F(w) := integral_(-infinity)^infinity {integrand} dif t"));
        run(&mut env, &format!("G(w) := {expected}"));
        assert!(!env.funs["F"].body.contains(&|e| matches!(e, Expr::Integral(..) | Expr::Derivative(..))), "{integrand}");
        assert_eq!(run(&mut env, "F(w) = G(w) ->").result.as_deref(), Some("\"true\""), "{integrand}");
    }
}

#[test]
fn weighted_moments_stay_stable_near_zero_in_calls_and_graph_samples() {
    for degree in [1, 2] {
        for centre in [0.0_f64, 2.0] {
            let mut env = Env::default();
            run(&mut env, &format!("F(w) := integral_(-infinity)^infinity t^{degree}*rect(t-({centre}))*exp(-#i*w*t) dif t"));
            for w in [1e-10_f64, -1e-9, 1e-6] {
                run(&mut env, &format!("answer := F({w}) =>"));
                let actual = complex_value(&env.vars["answer"].expr).unwrap();
                let expected_re = simpson(|t| t.powi(degree)*(w*t).cos(), centre-0.5, centre+0.5);
                let expected_im = -simpson(|t| t.powi(degree)*(w*t).sin(), centre-0.5, centre+0.5);
                assert!((actual.re.v.f()-expected_re).abs() < 1e-8, "degree={degree}, centre={centre}, w={w}: {actual:?}, expected {expected_re}");
                assert!((actual.im.v.f()-expected_im).abs() < 1e-8);
                let real = expr::func(solveng_core::expr::Func::Re, env.funs["F"].body.clone());
                let sample = eval_f(&real, &|n| (n == "w").then_some(w)).unwrap();
                assert!((sample-expected_re).abs() < 1e-8);
            }
        }
    }
}
