use solveng_core::{
    complex::Z,
    eval::{complex_value, eval_f, value},
    expr::{Expr, Func, Val, func, int, simp, sym},
    dim::Dim,
    num::Num,
};

fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 3e-13 * b.abs().max(1e-30), "{a} != {b}");
}

#[test]
fn cardinal_values_are_continuous_at_zero_and_stable_near_it() {
    for f in [Func::Sinc, Func::Cosc, Func::Tanc] {
        let origin = if f == Func::Cosc { 0 } else { 1 };
        assert_eq!(simp(&func(f, int(0))), int(origin));
        for x in [-12.3_f64, -1.1, -0.5, -0.49, -1e-8, -1e-100, 0.0, 1e-100, 1e-8, 0.49, 0.5, 1.1, 12.3] {
            let want = if x == 0.0 { origin as f64 } else {
                match f {
                    Func::Sinc => x.sin() / x,
                    Func::Cosc => (x / 2.0).sin() * ((x / 2.0).sin() / x) * 2.0,
                    _ => x.tan() / x,
                }
            };
            near(eval_f(&func(f, sym("x")), &|_| Some(x)).unwrap(), want);
            near(f.eval(x).unwrap(), want);
        }
    }
}

#[test]
fn cardinal_functions_accept_complex_arguments_and_reject_units() {
    for f in [Func::Sinc, Func::Cosc, Func::Tanc] {
        for z in [Z { re: 0.1, im: 0.2 }, Z { re: 1.3, im: -0.8 }] {
            let arg = solveng_core::expr::add(vec![solveng_core::expr::c(Num::F(z.re)), solveng_core::expr::mul(vec![Expr::I, solveng_core::expr::c(Num::F(z.im))])]);
            let actual = complex_value(&func(f, arg)).unwrap();
            let expected = match f {
                Func::Sinc => z.sin().div(z),
                Func::Cosc => Z::real(1.0).sub(z.cos()).div(z),
                _ => z.sin().div(z.cos()).div(z),
            };
            near(actual.re.v.f(), expected.re);
            near(actual.im.v.f(), expected.im);
        }
        let unit = Expr::C(Val::new(Num::ONE, Dim::base(0)));
        assert!(value(&func(f, unit)).is_err());
    }
}

#[test]
fn cardinal_derivatives_include_the_continuation_at_zero() {
    for f in [Func::Sinc, Func::Cosc, Func::Tanc] {
        let d = func(f, sym("x")).diff("x");
        near(eval_f(&d, &|_| Some(0.0)).unwrap(), if f == Func::Cosc { 0.5 } else { 0.0 });
        for x in [-1.2_f64, -0.3, 0.3, 1.2] {
            let v = f.eval(x).unwrap();
            let expected = match f {
                Func::Sinc => (x.cos() - v) / x,
                Func::Cosc => (x.sin() - v) / x,
                _ => (1.0 / x.cos().powi(2) - v) / x,
            };
            near(eval_f(&d, &|_| Some(x)).unwrap(), expected);
        }
    }
}
