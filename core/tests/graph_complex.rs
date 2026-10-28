mod support;
use solveng_core::{env::Env, graph::{self, GraphOut, GraphReq, XV}, eval::{complex_value, eval_f}, expr::{Expr, func, Func, sym}};
use support::{run, curve, cfg};

fn plot(env: &Env, source: &str, a: f64, b: f64) -> Result<GraphOut, String> {
    graph::graph(env, &GraphReq { cfg: cfg(), curves: vec![curve(source, env)], xrange: vec![XV::Num(a), XV::Num(b)], yrange: vec![], marks: vec![], ymark: vec![], samples: 80 })
}
fn check(g: &GraphOut, f: impl Fn(f64) -> f64, tol: f64) {
    let points: Vec<_> = g.series.iter().filter(|s| !s.beyond).flat_map(|s| &s.points).collect();
    assert!(points.len() > 50, "{} points", points.len());
    for &&(x,y) in &points { assert!((y - f(x)).abs() < tol, "x={x}, y={y}, expected {}", f(x)); }
}
fn fourier_env() -> Env {
    let mut env = Env::default();
    for source in [
        "p(t_0) := rect(t-t_0) + tri(2(t-t_0))",
        "s(t) := p(1.5) + p(3.5) + p(-1.5) + p(-3.5)",
        "S(f) := integral_(-infinity)^infinity s(t) * e^(-#i*2*pi*f*t) dif t",
        "T(f) := integral_(-infinity)^infinity rect(t-1.5) * e^(-#i*2*pi*f*t) dif t",
    ] { run(&mut env, source); }
    env
}
fn sinc(x: f64) -> f64 { if x == 0.0 { 1.0 } else { x.sin()/x } }
fn analytic(f: f64) -> f64 {
    let p = std::f64::consts::PI;
    // Shift theorem: rectangle width 1 plus triangle width 1, height 1.
    (sinc(p*f) + 0.5*sinc(p*f/2.0).powi(2)) * 2.0 * ((3.0*p*f).cos() + (7.0*p*f).cos())
}
#[test]
fn fourier_graph_agrees_with_independent_analytic_expression() {
    let mut env = fourier_env();
    for f in [0.0, 0.2, -0.37, 1.0, 2.3] {
        run(&mut env, &format!("answer := S({f}) ="));
        let value = complex_value(&env.vars["answer"].expr).unwrap();
        assert!((value.re.v.f() - analytic(f)).abs() < 2e-9, "f={f}: {value:?}");
        assert!(value.im.v.f().abs() < 2e-9);
    }
    check(&plot(&env,"S(f)",-5.0,5.0).unwrap(), analytic, 2e-8);
}
#[test]
fn nonsymmetric_transform_requires_projection() {
    let env = fourier_env();
    for (a,b) in [(-0.4,0.4),(0.1,0.3)] {
        let err = plot(&env,"T(f)",a,b).unwrap_err();
        assert!(err.contains("complex") && err.contains("Re(") && err.contains("Im("), "{err}");
    }
    let p = std::f64::consts::PI;
    check(&plot(&env,"Re(T(f))",0.1,0.3).unwrap(), |f| sinc(p*f)*(3.0*p*f).cos(), 2e-9);
    check(&plot(&env,"Im(T(f))",0.1,0.3).unwrap(), |f| -sinc(p*f)*(3.0*p*f).sin(), 2e-9);
    check(&plot(&env,"abs(T(f))",0.1,0.3).unwrap(), |f| sinc(p*f), 2e-9);
    check(&plot(&env,"arg(T(f))",0.1,0.3).unwrap(), |f| -3.0*p*f, 2e-9);
}
#[test]
fn complex_sums_cancellation_and_implicit_complex_intermediates() {
    let mut env = Env::default();
    for source in [
        "G(x) := sum_(k=0)^300 e^(#i*x*k)/301",
        "H(x) := sum_(k=0)^infinity (#i*x)^k / 2^k",
        "C(x) := integral_(-1)^1 e^(#i*x*t) dif t",
    ] { run(&mut env, source); }
    check(&plot(&env,"Im(G(x))",0.1,0.3).unwrap(), |x| (0..=300).map(|k| (x*k as f64).sin()/301.0).sum(), 1e-9);
    check(&plot(&env,"Re(H(x))",0.1,0.3).unwrap(), |x| 1.0/(1.0+x*x/4.0), 1e-9);
    check(&plot(&env,"C(x)",-3.0,3.0).unwrap(), |x| 2.0*sinc(x), 1e-9);
    check(&plot(&env,"abs(sqrt(-x))",0.1,2.0).unwrap(), f64::sqrt, 1e-10);
    // No fixed absolute tolerance: even a tiny purely imaginary result is complex.
    assert!(plot(&env,"10^(-20) * #i * x",1.0,2.0).unwrap_err().contains("complex"));
    assert!(plot(&env,"C(x) + #i * 0.00001",1.0,2.0).unwrap_err().contains("complex"));
    assert!(plot(&env,"10^20*x + #i*0.001",1.0,2.0).unwrap_err().contains("complex"));
    assert!(plot(&env,"(integral_0^1 e^(t^2) dif t) + #i*x*10^(-15)",1.0,2.0).unwrap_err().contains("complex"));
    check(&plot(&env,"C(x)^2",-3.0,3.0).unwrap(), |x| (2.0*sinc(x)).powi(2), 1e-9);
    check(&plot(&env,"e^(#i*x)+e^(-#i*x)",-3.0,3.0).unwrap(), |x| 2.0*x.cos(), 1e-10);
    check(&plot(&env,"Re(cases(e^(#i*x), x < 0, e^(-#i*x)))",-1.0,1.0).unwrap(), f64::cos, 1e-10);
    assert!(plot(&env,"#i*floor(x)",1.0,2.0).unwrap_err().contains("complex"));
    assert!(plot(&env,"#i*sin(pi*x)",-5.0,5.0).unwrap_err().contains("complex"));
    // The shared floating evaluator projects only at the outermost expression.
    let e = func(Func::Im, solveng_core::expr::mul(vec![Expr::I, sym("x")]));
    assert_eq!(eval_f(&e, &|_| Some(2.0)), Some(2.0));
}
#[test]
fn scoped_variables_units_and_late_binding() {
    let mut env = Env::default();
    for source in ["t := 99 s", "Q(x: s) := integral_(-1 s)^(1 s) e^(#i * t / x) * 1 A dif t"] { run(&mut env, source); }
    // Output is A*s; x is in seconds. The outer t does not capture the local t.
    check(&plot(&env,"Q(x)",0.5,2.0).unwrap(), |x| 2.0*x*(1.0/x).sin(), 1e-9);
    let si_value = eval_f(&env.funs["Q"].body, &|n| (n == "x").then_some(0.75)).unwrap();
    assert!((si_value - 1.5*(1.0_f64/0.75).sin()).abs() < 1e-9);
    run(&mut env,"L(x) := Re(later(x))");
    run(&mut env,"later(x) := e^(#i*3*x)");
    run(&mut env,"a := 3");
    check(&plot(&env,"L(x)",-1.0,1.0).unwrap(), |x| (3.0*x).cos(), 1e-10);
    // Nested reuse of a local name must restore its surrounding binding.
    run(&mut env,"N(x) := Im(integral_0^1 (integral_0^1 #i*t*x dif t) + #i*t dif t)");
    check(&plot(&env,"N(x)",0.1,1.0).unwrap(), |x| x/2.0+0.5, 1e-10);
}
#[test]
fn ordinary_real_graphs_keep_domains_and_discontinuities() {
    let env = Env::default();
    check(&plot(&env,"x^2 - 2*x",-2.0,2.0).unwrap(), |x| x*x-2.0*x, 1e-10);
    check(&plot(&env,"sin(x)",-3.0,3.0).unwrap(), f64::sin, 1e-10);
    let g = plot(&env,"sqrt(x)",-1.0,1.0).unwrap();
    assert!(g.series.iter().flat_map(|s| &s.points).all(|p| p.0 >= 0.0));
    let g = plot(&env,"rect(x)",-2.0,2.0).unwrap();
    assert_eq!(g.series.len(), 3);
    assert!(plot(&env,"1/x",-1.0,1.0).is_ok());
    assert!(plot(&env,"ln(x)",-2.0,-1.0).unwrap_err().contains("complex"));
}
#[test]
fn fourier_transform_of_pulses_is_closed_and_solvable() {
    let mut env = fourier_env();
    // Compact entire form, including the continuation at f = 0.
    let out = run(&mut env, "S(f) ->");
    assert!(out.result.as_deref().is_some_and(|r| r.contains("sinc(") && r.contains("cos(") && !r.contains("cases(") && !r.contains("integral")), "{out:?}");
    for f in [0.0, 1e-13, -1e-9, 1e-6, 0.2] {
        let value = eval_f(&env.funs["S"].body, &|n| (n == "f").then_some(f)).unwrap();
        assert!((value - analytic(f)).abs() < 1e-9, "f={f}: {value}");
    }
    // a number put in close to 0, directly or through a variable
    for source in ["answer := S(10^(-9)) =", "answer := S(-10^(-13)) + 0 =", "answer := S(2 x) ="] {
        run(&mut env, "x := 10^(-10)");
        run(&mut env, source);
        let value = complex_value(&env.vars["answer"].expr).unwrap();
        assert!((value.re.v.f() - 6.0).abs() < 1e-9 && value.im.v.f().abs() < 1e-9, "{source}: {value:?}");
    }
    // the zeros nearest the origin: cos(5πf) cos(2πf) = 0 (it used to crash the engine)
    let out = run(&mut env, "S(f) = 0 -> solve(f)");
    let result = out.result.unwrap();
    for root in ["f_(1) = \"0.1\"", "f_(3) = \"0.25\"", "f_(5) = \"0.3\""] {
        assert!(result.contains(root), "{root} in {result}");
    }
    assert_eq!(env.funs["S"].params.len(), 1);
}
