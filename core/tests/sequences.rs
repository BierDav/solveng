//! Range variables, elements x[i], discrete and continuous convolution, named arguments and explicit operation references.

use std::collections::BTreeSet;

use solveng_core::api::{self, LineReq};
use solveng_core::env::Env;
use solveng_core::eval::value;
use solveng_core::expr::{Expr, simp};
use solveng_core::format::Cfg;
use solveng_core::parse::Mode;
use solveng_core::units::Units;

fn out(env: &mut Env, src: &str, mode: Mode, cfg: &Cfg) -> solveng_core::sheet::Out {
    api::line(env, &LineReq { src: src.into(), mode, cfg: cfg.clone(), listed: true })
}
fn ok(env: &mut Env, src: &str) {
    let o = out(env, src, Mode::Code, &Cfg::default());
    assert!(o.errors.is_empty(), "{src}: {:?}", o.errors);
}
fn err(env: &mut Env, src: &str) -> String {
    let o = out(env, src, Mode::Code, &Cfg::default());
    assert!(!o.errors.is_empty(), "{src} should fail");
    o.errors.join("; ")
}
fn bound(env: &Env, n: &str) -> Expr {
    simp(&env.bind_with_units(&env.vars[n].expr, &BTreeSet::new(), &Units::default()).unwrap())
}
fn num(env: &Env, n: &str) -> f64 {
    value(&bound(env, n)).unwrap_or_else(|e| panic!("{n}: {e:?}")).v.f()
}
/// The SI values of a vector.
fn nums(env: &Env, n: &str) -> Vec<f64> {
    match bound(env, n) {
        Expr::Vector(v) => v.iter().map(|e| value(e).unwrap().v.f()).collect(),
        e => panic!("{n} is no vector: {e:?}"),
    }
}
fn close(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1e-9 * (1.0 + y.abs()))
}

#[test]
fn ranges() {
    let mut env = Env::default();
    ok(&mut env, "N := 4");
    ok(&mut env, "i := 0 .. N");
    assert!(close(&nums(&env, "i"), &[0.0, 1.0, 2.0, 3.0, 4.0]));
    ok(&mut env, "d := 3 .. 1");
    assert!(close(&nums(&env, "d"), &[3.0, 2.0, 1.0]));
    ok(&mut env, "h := 1/2 .. 3");
    assert!(close(&nums(&env, "h"), &[0.5, 1.5, 2.5]));
    // Typst's range: the end left out, a step by name
    ok(&mut env, "r := range(4)");
    assert!(close(&nums(&env, "r"), &[0.0, 1.0, 2.0, 3.0]));
    ok(&mut env, "q := range(1, 10, step: 3)");
    assert!(close(&nums(&env, "q"), &[1.0, 4.0, 7.0]));
    ok(&mut env, "w := range(5, 0, step: -2)");
    assert!(close(&nums(&env, "w"), &[5.0, 3.0, 1.0]));
    ok(&mut env, "s := range([0 s], [1 s], step: [250 ms])");
    assert!(close(&nums(&env, "s"), &[0.0, 0.25, 0.5, 0.75]));
    assert!(err(&mut env, "e := range(5, step: 0)").contains("0"));
    assert!(err(&mut env, "e := range([1 s])").contains("step"));
    assert!(err(&mut env, "e := 0 .. M").contains("needs a value"));
    assert!(err(&mut env, "e := range(1, 2, width: 3)").contains("unexpected argument: width"));
    // math writes them as Typst does: 0..N, 0 … N
    let o = out(&mut env, "k ≔ 0..N", Mode::Math, &Cfg::default());
    assert!(o.errors.is_empty(), "{:?}", o.errors);
    assert_eq!(nums(&env, "k").len(), 5);
}

#[test]
fn elements() {
    let mut env = Env::default();
    ok(&mut env, "v := vec(3, 5, 7)");
    ok(&mut env, "a := v[1]");
    assert_eq!(num(&env, "a"), 5.0);
    ok(&mut env, "A := mat(1, 2; 3, 4)");
    ok(&mut env, "b := A[1, 0]");
    assert_eq!(num(&env, "b"), 3.0);
    // a vector of indices selects several
    ok(&mut env, "j := 0 .. 1");
    ok(&mut env, "c := v[j]");
    assert!(close(&nums(&env, "c"), &[3.0, 5.0]));
    // elements for every value of a range variable
    ok(&mut env, "N := 4");
    ok(&mut env, "i := 0 .. N");
    ok(&mut env, "t[i] := -1 + i/2");
    assert!(close(&nums(&env, "t"), &[-1.0, -0.5, 0.0, 0.5, 1.0]));
    ok(&mut env, "x[i] := tri(t[i])");
    assert!(close(&nums(&env, "x"), &[0.0, 0.5, 1.0, 0.5, 0.0]));
    // a single element: the vector grows, with zeros between
    ok(&mut env, "x[7] := 9");
    assert!(close(&nums(&env, "x"), &[0.0, 0.5, 1.0, 0.5, 0.0, 0.0, 0.0, 9.0]));
    ok(&mut env, "x[0] := 2");
    assert_eq!(nums(&env, "x")[0], 2.0);
    // with units
    ok(&mut env, "T := [10 ms]");
    ok(&mut env, "τ[i] := i T");
    assert!(close(&nums(&env, "τ"), &[0.0, 0.01, 0.02, 0.03, 0.04]));
    // a matrix
    ok(&mut env, "k := 0 .. 2");
    ok(&mut env, "M[i, k] := i + 10 k");
    match bound(&env, "M") {
        Expr::Matrix(rs) => {
            assert_eq!((rs.len(), rs[0].len()), (5, 3));
            assert_eq!(value(&rs[4][2]).unwrap().v.f(), 24.0);
        }
        e => panic!("{e:?}"),
    }
    assert!(err(&mut env, "y[2 i] := 1").contains("range variable"));
    assert!(err(&mut env, "y[-1] := 1").contains("whole number"));
    assert!(err(&mut env, "y[i] := v").contains("single value"));
    assert!(err(&mut env, "e := v[3]").contains("outside"));
    // code: a value with a unit after a space stays one, an index follows without
    ok(&mut env, "R := 2");
    ok(&mut env, "U := R [1 mA]");
    assert!((num(&env, "U") - 0.002).abs() < 1e-12);
}

#[test]
fn discrete_convolution() {
    let mut env = Env::default();
    ok(&mut env, "y := convolve(vec(1, 2), vec(1, 1, 1))");
    assert!(close(&nums(&env, "y"), &[1.0, 3.0, 3.0, 2.0]));
    ok(&mut env, "z := convolve(vec(1, 2), vec(1, 1, 1), step: [2 s])");
    assert!(close(&nums(&env, "z"), &[2.0, 6.0, 6.0, 4.0]));
    // a symbol waits: the operation is kept until it's a vector
    ok(&mut env, "w := convolve(h, vec(1, 1))");
    ok(&mut env, "h := vec(1, -1)");
    ok(&mut env, "w2 := w");
    assert!(close(&nums(&env, "w2"), &[1.0, 0.0, -1.0]));
    // sampled triangles: their convolution approximates tri ∗ tri (2/3 at 0)
    ok(&mut env, "N := 200");
    ok(&mut env, "i := 0 .. N");
    ok(&mut env, "Δt := 1/100");
    ok(&mut env, "t[i] := -1 + i Δt");
    ok(&mut env, "x[i] := tri(t[i])");
    ok(&mut env, "c := convolve(x, x, step: Δt)");
    let c = nums(&env, "c");
    assert_eq!(c.len(), 401);
    assert!((c[200] - 2.0 / 3.0).abs() < 1e-3, "{}", c[200]);
    assert!((c[250] - 23.0 / 48.0).abs() < 1e-3, "{}", c[250]);
    assert!(err(&mut env, "e := convolve(mat(1, 2; 3, 4), vec(1, 1))").contains("sequences"));
    assert!(err(&mut env, "e := convolve(x)").contains("missing argument: g"));
    assert!(err(&mut env, "e := convolve(x, x, width: 2)").contains("unexpected argument: width"));
}

#[test]
fn continuous_convolution() {
    let mut env = Env::default();
    ok(&mut env, "y(t) := convolve(tri(τ), tri(τ), τ, t)");
    ok(&mut env, "a := y(1/2)");
    let a = bound(&env, "a");
    assert_eq!(value(&a).unwrap().v.f(), 23.0 / 48.0);
    assert!(matches!(&a, Expr::C(v) if v.v.rat().is_some()), "exact: {a:?}");
    // the result in the convolved variable itself
    ok(&mut env, "z(t) := convolve(rect(t), rect(t), t)");
    ok(&mut env, "b := z(1/4)");
    assert_eq!(num(&env, "b"), 0.75);
    assert!(err(&mut env, "e(t) := convolve(tri(τ), tri(τ), τ, t, step: 1)").contains("step"));
    assert!(err(&mut env, "e(t) := convolve(tri(τ), tri(τ), 2, t)").contains("name"));
}

#[test]
fn named_parameters() {
    let mut env = Env::default();
    ok(&mut env, "f(x, n: 2) := x^n");
    ok(&mut env, "a := f(3)");
    assert_eq!(num(&env, "a"), 9.0);
    ok(&mut env, "b := f(3, n: 3)");
    assert_eq!(num(&env, "b"), 27.0);
    assert!(err(&mut env, "c := f(3, m: 3)").contains("unexpected argument: m"));
    assert!(err(&mut env, "c := f(x: 3)").contains("positional"));
    assert!(err(&mut env, "c := f(3, n: 1, n: 2)").contains("duplicate"));
    assert!(err(&mut env, "c := f()").contains("missing argument: x") || err(&mut env, "c := f()").contains("takes"));
    assert!(err(&mut env, "c := f(1, 2)").contains("takes 1 argument"));
    // the default is taken where the function is defined, as Typst evaluates it
    ok(&mut env, "k := 2");
    ok(&mut env, "g(x, s: k) := x s");
    ok(&mut env, "k := 5");
    ok(&mut env, "c := g(1)");
    assert_eq!(num(&env, "c"), 2.0);
    // with a unit: the default's
    ok(&mut env, "p(U, R: [2 Ω]) := U^2/R");
    ok(&mut env, "P := p([4 V])");
    assert_eq!(num(&env, "P"), 8.0);
    // a type stays a type
    ok(&mut env, "q(I: mA) := I");
    assert!(err(&mut env, "e := q(2)").contains("mA"));
    // named parameters come last
    assert!(err(&mut env, "h(n: 2, x) := x").contains("come after"));
    // a call written before its function: defaults fill the rest
    ok(&mut env, "u(x) := v(x)");
    ok(&mut env, "v(x, n: 3) := x n");
    ok(&mut env, "d := u(2)");
    assert_eq!(num(&env, "d"), 6.0);
}

#[test]
fn explicit_operations_and_disabled_automatic_names() {
    let mut env = Env::default();
    let cfg = Cfg { builtin_functions: false, ..Cfg::default() };
    assert!(!out(&mut env, "a := tri(0) =", Mode::Code, &cfg).errors.is_empty());
    assert!(!out(&mut env, "a ≔ tri(0) =", Mode::Math, &cfg).errors.is_empty());
    assert!(out(&mut env, "b ≔ ⦇tri⦈(1/2)", Mode::Math, &cfg).errors.is_empty());
    assert_eq!(num(&env, "b"), 0.5);
    for name in ["tri", "sin", "rect"] {
        let src = format!("a ≔ \"{name}\"(0) =");
        assert!(!out(&mut env, &src, Mode::Math, &Cfg::default()).errors.is_empty(), "{src}");
    }
    assert!(out(&mut env, "\"pulse\"(x) ≔ 2 x", Mode::Math, &cfg).errors.is_empty());
    assert!(out(&mut env, "c ≔ \"pulse\"(3)", Mode::Math, &cfg).errors.is_empty());
    assert_eq!(num(&env, "c"), 6.0);
}

#[test]
fn worksheet_functions_take_precedence() {
    let mut env = Env::default();
    // a worksheet's own function of a standard name replaces it for plain lookup
    ok(&mut env, "tri(x) := 5 x");
    ok(&mut env, "a := tri(2)");
    assert_eq!(num(&env, "a"), 10.0);
    // the operation stays reachable by an explicit reference
    let o = out(&mut env, "b ≔ ⦇tri⦈(0)", Mode::Math, &Cfg::default());
    assert!(o.errors.is_empty(), "{:?}", o.errors);
    assert_eq!(num(&env, "b"), 1.0);
    // a definition can't have an explicit reference as its name
    let o = out(&mut env, "⦇tri⦈(x) ≔ x", Mode::Math, &Cfg::default());
    assert!(o.errors.iter().any(|e| e.contains("built-in operation")), "{:?}", o.errors);
    // math.op of a name that is no operation is a name: 5 min
    let o = out(&mut env, "t ≔ 5 ⦇min⦈", Mode::Math, &Cfg::default());
    assert!(o.errors.is_empty(), "{:?}", o.errors);
}

