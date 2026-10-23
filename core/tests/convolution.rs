use solveng_core::{
    api::{self, LineReq},
    calculus::{piecewise_quadrature, signal_points},
    convolution::continuous,
    env::Env,
    eval::{eval_f, value},
    expr::{Expr, mul, rat, simp, sub, sym},
    format::Cfg,
    num::Num,
    parse::Mode,
};

/// The body of `name(x) := src`, lowered by the worksheet.
fn body(env: &mut Env, definition: &str) -> Expr {
    let out = api::line(env, &LineReq { src: definition.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
    assert!(out.errors.is_empty(), "{definition}: {:?}", out.errors);
    let name = definition.split('(').next().unwrap().trim();
    env.funs[name].body.clone()
}

fn param(env: &Env, name: &str) -> String {
    env.funs[name].params[0].name.clone()
}

/// ∫ f(x) g(t − x) dx by quadrature (SI numbers), over a range covering the overlap.
fn numeric(f: &Expr, g: &Expr, x: &str, t: f64, lo: f64, hi: f64) -> f64 {
    let product = mul(vec![f.clone(), g.subst(x, &sub(sym("t_ref"), sym(x)))]);
    let vars = |n: &str| (n == "t_ref").then_some(t);
    let points = signal_points(&product, x, &vars);
    piecewise_quadrature(&|v| eval_f(&product, &|n| if n == x { Some(v) } else { vars(n) }), lo, hi, &points).unwrap()
}

fn closed_at(e: &Expr, t: f64) -> f64 {
    eval_f(e, &|n| (n == "t").then_some(t)).unwrap_or_else(|| panic!("no value at {t}: {e:?}"))
}

fn exact_at(e: &Expr, t: Expr) -> Num {
    let v = value(&simp(&e.subst("t", &t))).unwrap();
    assert!(v.v.is_exact(), "{v:?}");
    v.v
}

fn r(n: i128, d: i128) -> Num {
    rat(n, d).as_num().unwrap()
}

/// Compare with quadrature on a grid and at all given corners.
fn agrees(e: &Expr, f: &Expr, g: &Expr, x: &str, corners: &[f64], range: (f64, f64)) {
    let mut ts: Vec<f64> = (-80..=80).map(|k| k as f64 / 8.0).collect();
    ts.extend(corners);
    for t in ts {
        let (a, b) = (closed_at(e, t), numeric(f, g, x, t, range.0, range.1));
        assert!((a - b).abs() < 1e-7 * (1.0 + b.abs()), "t = {t}: closed {a}, numeric {b}\n{e:?}");
    }
}

#[test]
fn rect_with_rect_is_tri() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := rect(x)");
    let e = continuous(&f, &f, &param(&env, "f"), &sym("t"));
    assert!(matches!(e, Expr::Cases(_)), "{e:?}");
    agrees(&e, &f, &f, "x", &[-1.0, 0.0, 1.0], (-3.0, 3.0));
    assert_eq!(exact_at(&e, rat(1, 4)), r(3, 4));
    assert_eq!(exact_at(&e, rat(-1, 1)), r(0, 1));
}

#[test]
fn tri_with_tri_is_a_piecewise_cubic() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := tri(x)");
    let e = continuous(&f, &f, "x", &sym("t"));
    let expected = |t: f64| {
        let a = t.abs();
        if a <= 1.0 { 2.0 / 3.0 - t * t + a.powi(3) / 2.0 } else if a <= 2.0 { (2.0 - a).powi(3) / 6.0 } else { 0.0 }
    };
    for k in -24..=24 {
        let t = k as f64 / 8.0;
        assert!((closed_at(&e, t) - expected(t)).abs() < 1e-12, "{t}");
    }
    agrees(&e, &f, &f, "x", &[-2.0, -1.0, 0.0, 1.0, 2.0], (-3.0, 3.0));
    assert_eq!(exact_at(&e, rat(1, 2)), r(23, 48));
    assert_eq!(exact_at(&e, rat(3, 2)), r(1, 48));
    assert_eq!(exact_at(&e, rat(0, 1)), r(2, 3));
    // the parts between the corners are polynomials: four nonzero branches and a zero default
    let Expr::Cases(branches) = &e else { panic!("{e:?}") };
    assert_eq!(branches.len(), 5, "{e:?}");
}

#[test]
fn shifted_and_scaled_operands() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := tri((x - 2)/4)");
    let g = body(&mut env, "g(x) := rect(3 x)");
    let e = continuous(&f, &g, "x", &sym("t"));
    // corners at −2, 2, 6 ± 1/6
    let corners = [-2.0 - 1.0 / 6.0, -2.0 + 1.0 / 6.0, 2.0 - 1.0 / 6.0, 2.0 + 1.0 / 6.0, 6.0 - 1.0 / 6.0, 6.0 + 1.0 / 6.0];
    agrees(&e, &f, &g, "x", &corners, (-8.0, 10.0));
    // at t = 2: ∫_{−1/6}^{1/6} tri(s/4) ds = 1/3 − 1/144
    assert_eq!(exact_at(&e, rat(2, 1)), r(47, 144));
    // polynomials and abs with affine arguments are piecewise polynomial too
    let h = body(&mut env, "h(x) := (x^2 + abs(2 x - 1)) * rect(x/2) - 3 sgn(x + 1/3) * tri(x)");
    let e = continuous(&h, &f, "x", &sym("t"));
    assert!(!e.contains(&|e| matches!(e, Expr::Integral(..))), "{e:?}");
    agrees(&e, &h, &f, "x", &[-3.0, -1.0, 1.0 / 3.0, 3.0, 7.0], (-10.0, 12.0));
}

#[test]
fn convolution_commutes() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := tri((x - 2)/4) + x rect(x - 1)");
    let g = body(&mut env, "g(x) := rect(3 x) - 2 tri(2 x + 1)");
    let a = continuous(&f, &g, "x", &sym("t"));
    let b = continuous(&g, &f, "x", &sym("t"));
    assert_eq!(a, b);
    agrees(&a, &f, &g, "x", &[], (-8.0, 10.0));
}

#[test]
fn a_unit_step_with_a_finite_pulse() {
    let mut env = Env::default();
    let step = body(&mut env, "u(x) := cases(0, x < 0, 1)");
    let pulse = body(&mut env, "p(x) := rect((x - 1)/2)");
    let e = continuous(&step, &pulse, "x", &sym("t"));
    assert_eq!(exact_at(&e, rat(-1, 1)), r(0, 1));
    assert_eq!(exact_at(&e, rat(1, 2)), r(1, 2));
    assert_eq!(exact_at(&e, rat(5, 1)), r(2, 1));
    agrees(&e, &step, &pulse, "x", &[0.0, 2.0], (-10.0, 20.0));
    assert_eq!(continuous(&pulse, &step, "x", &sym("t")), e);
    // a step with a ramp that grows the other way diverges: the integral stays
    let back = body(&mut env, "w(x) := cases(x, x < 0, 0)");
    assert!(matches!(continuous(&step, &back, "x", &sym("t")), Expr::Integral(..)));
}

#[test]
fn symbolic_coefficients_stay() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := a tri(x)");
    let g = body(&mut env, "g(x) := rect(x)");
    let e = continuous(&f, &g, "x", &sym("t"));
    assert!(matches!(e, Expr::Cases(_)), "{e:?}");
    // (a·tri ∗ rect)(0) = a·∫_{−1/2}^{1/2} tri = 3a/4
    let at0 = simp(&e.subst("t", &rat(0, 1)));
    assert_eq!(simp(&sub(at0, mul(vec![rat(3, 4), sym("a")]))), rat(0, 1));
    for k in -16..=16 {
        let t = k as f64 / 8.0;
        let closed = eval_f(&e, &|n| match n { "t" => Some(t), "a" => Some(2.5), _ => None }).unwrap();
        let reference = 2.5 * numeric(&body(&mut env, "q(x) := tri(x)"), &g, "x", t, -3.0, 3.0);
        assert!((closed - reference).abs() < 1e-9, "{t}");
    }
}

#[test]
fn dimensioned_variable_and_breakpoints() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := 3 V * tri((x - 2 s)/(4 s))");
    let g = body(&mut env, "g(x) := rect(x/(2 s))");
    let e = continuous(&f, &g, "x", &sym("t"));
    assert!(matches!(e, Expr::Cases(_)), "{e:?}");
    agrees(&e, &f, &g, "x", &[-3.0, 1.0, 3.0, 5.0, 7.0], (-10.0, 12.0));
    // at t = 2 s: 3 V · ∫_{−1 s}^{1 s} tri(u/4 s) du = 3 V · 7/4 s
    let seconds = |n: i128| Expr::C(solveng_core::expr::Val::new(Num::int(n), solveng_core::dim::Dim::si([0, 0, 1, 0, 0, 0, 0])));
    let v = value(&simp(&e.subst("t", &seconds(2)))).unwrap();
    assert_eq!(v.v, r(21, 4));
    let volt_second = solveng_core::dim::Dim::si([2, 1, -2, -1, 0, 0, 0]);
    assert_eq!(v.d, volt_second);
    // outside the support it is a zero of the same dimension
    assert_eq!(value(&simp(&e.subst("t", &seconds(20)))).unwrap().d, volt_second);
    // the same with `at` given as a quantity
    let direct = value(&continuous(&f, &g, "x", &seconds(2))).unwrap();
    assert_eq!((direct.v, direct.d), (r(21, 4), volt_second));
}

#[test]
fn non_polynomial_operands_fall_back_to_the_integral() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := exp(-x^2)");
    let g = body(&mut env, "g(x) := rect(x)");
    let e = continuous(&f, &g, "x", &sym("t"));
    assert!(matches!(e, Expr::Integral(..)), "{e:?}");
    for k in -12..=12 {
        let t = k as f64 / 4.0;
        let reference = numeric(&f, &g, "x", t, t - 1.0, t + 1.0);
        let closed = closed_at(&e, t);
        assert!((closed - reference).abs() < 1e-8, "{t}: {closed} {reference}");
    }
    // at a number, the engine's value of it
    let v = value(&continuous(&f, &g, "x", &rat(1, 2))).unwrap();
    assert!((v.v.f() - numeric(&f, &g, "x", 0.5, -0.5, 1.5)).abs() < 1e-8);
    // dimensioned: the bounds carry var's dimension
    let h = body(&mut env, "h(x) := exp(-(x/(1 s))^2)");
    let k = body(&mut env, "k(x) := rect(x/(1 s))");
    let seconds = Expr::C(solveng_core::expr::Val::new(rat(1, 2).as_num().unwrap(), solveng_core::dim::Dim::si([0, 0, 1, 0, 0, 0, 0])));
    let v = value(&continuous(&h, &k, "x", &seconds)).unwrap();
    assert!((v.v.f() - numeric(&f, &g, "x", 0.5, -0.5, 1.5)).abs() < 1e-8, "{v:?}");
    assert_eq!(v.d, solveng_core::dim::Dim::si([0, 0, 1, 0, 0, 0, 0]));
}

#[test]
fn the_evaluation_point_may_be_the_variable_itself() {
    let mut env = Env::default();
    let f = body(&mut env, "f(x) := rect(x)");
    let e = continuous(&f, &f, "x", &sym("x"));
    assert_eq!(value(&simp(&e.subst("x", &rat(1, 2)))).unwrap().v, r(1, 2));
    let g = body(&mut env, "g(x) := exp(-x^2)");
    let e = continuous(&g, &f, "x", &sym("x"));
    assert!((closed_at(&e.subst("x", &sym("t")), 0.5) - numeric(&g, &f, "x", 0.5, -0.5, 1.5)).abs() < 1e-8);
}
