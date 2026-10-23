//! Random inputs checked against direct calculation: FUZZ_N cases per category (default 300), FUZZ_SEED.

use std::collections::BTreeMap;

use solveng_core::api::{LineReq, line};
use solveng_core::env::Env;
use solveng_core::eval::{eval_f, value};
use solveng_core::expr::{self, Expr, Func, int, simp, sym};
use solveng_core::format::{Cfg, fmt};
use solveng_core::parse::Mode;
use solveng_core::solve::solve1;

struct Rng(u64);
impl Rng {
    fn new(salt: u64) -> Rng {
        let seed: u64 = std::env::var("FUZZ_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(7);
        Rng(seed.wrapping_mul(6364136223846793005).wrapping_add(salt.wrapping_mul(1442695040888963407)) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn int(&mut self, a: i64, b: i64) -> i64 {
        a + (self.next() % (b - a + 1) as u64) as i64
    }
    fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.next() as usize % v.len()]
    }
}

fn n() -> usize {
    std::env::var("FUZZ_N").ok().and_then(|s| s.parse().ok()).unwrap_or(300)
}

fn run(env: &mut Env, src: &str) -> solveng_core::sheet::Out {
    line(env, &LineReq { src: src.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true })
}

fn val(env: &Env, name: &str) -> Option<f64> {
    let v = env.vars.get(name)?;
    value(&env.bind(&v.expr, &Default::default()).ok()?).ok().map(|v| v.v.f())
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * a.abs().max(b.abs()).max(1e-12)
}

/// A random expression in x as (code text, its value at a point).
fn random_expr(r: &mut Rng, depth: usize, x: Option<f64>) -> (String, Option<f64>) {
    if depth == 0 || r.f() < 0.25 {
        if x.is_some() && r.f() < 0.5 {
            return ("x".into(), x);
        }
        let k = r.int(1, 9) as f64 + if r.f() < 0.3 { r.int(1, 9) as f64 / 10.0 } else { 0.0 };
        return (format!("{k}"), Some(k));
    }
    let (a, av) = random_expr(r, depth - 1, x);
    let (b, bv) = random_expr(r, depth - 1, x);
    let op = *r.pick(&["+", "-", "*", "/", "^", "f"]);
    let m = |f: fn(f64, f64) -> f64| match (av, bv) {
        (Some(p), Some(q)) => Some(f(p, q)).filter(|v| v.is_finite()),
        _ => None,
    };
    match op {
        "+" => (format!("({a}) + ({b})"), m(|p, q| p + q)),
        "-" => (format!("({a}) - ({b})"), m(|p, q| p - q)),
        "*" => (format!("({a}) * ({b})"), m(|p, q| p * q)),
        "/" => (format!("({a}) / ({b})"), m(|p, q| if q == 0.0 { f64::NAN } else { p / q })),
        "^" => {
            let k = r.int(2, 3);
            (format!("({a})^{k}"), av.map(|p| p.powi(k as i32)))
        }
        _ => {
            let (name, f): (&str, fn(f64) -> f64) = *r.pick(&[("sin", f64::sin as fn(f64) -> f64), ("cos", f64::cos), ("exp", |v: f64| if v > 50.0 { f64::NAN } else { v.exp() }), ("sqrt", |v: f64| if v < 0.0 { f64::NAN } else { v.sqrt() })]);
            // sines of huge arguments depend on every last bit of them: no reference value
            let ill = matches!(name, "sin" | "cos") && av.is_some_and(|v| v.abs() > 1e5);
            (format!("{name}({a})"), av.filter(|_| !ill).map(f).filter(|v| v.is_finite()))
        }
    }
}

#[test]
fn fuzz_arithmetic() {
    let mut r = Rng::new(1);
    let mut bad = vec![];
    for i in 0..n() {
        let (src, v) = random_expr(&mut r, 4, None);
        let mut env = Env::default();
        let out = run(&mut env, &format!("y := {src} ="));
        match v {
            Some(v) if v.abs() < 1e12 => {
                let got = val(&env, "y");
                if !got.is_some_and(|g| close(g, v, 1e-9)) {
                    bad.push(format!("#{i} {src}: {got:?}, want {v} {:?}", out.errors));
                }
            }
            _ => {}
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn fuzz_solve1() {
    let mut r = Rng::new(2);
    let (mut tried, mut missed) = (0, vec![]);
    for i in 0..n() {
        let root = (r.int(-30, 30) as f64) / 4.0 + 0.1;
        let (src, v) = random_expr(&mut r, 3, Some(root));
        let Some(c) = v else { continue };
        if !src.replace("exp", "").contains('x') || c.abs() > 1e8 {
            continue;
        }
        let mut env = Env::default();
        let out = run(&mut env, &format!("{src} = {c} -> solve(x)"));
        if out.errors.iter().any(|e| e.contains("cancels out")) {
            continue;
        }
        tried += 1;
        // what was found must solve it
        let Some(x) = val(&env, "x") else {
            missed.push(format!("#{i} {src} = {c}: {:?} {:?}", out.result, out.errors));
            continue;
        };
        let mut e2 = Env::default();
        run(&mut e2, &format!("x := {x}"));
        let o = run(&mut e2, &format!("y := {src} ="));
        let y = val(&e2, "y");
        if !y.is_some_and(|y| close(y, c, 1e-6) || (y - c).abs() < 1e-9) {
            missed.push(format!("#{i} {src} = {c}: x = {x} gives {y:?} {:?}", o.errors));
        }
    }
    // numeric search may miss rare roots (between nearby poles, fast oscillations)
    assert!(missed.len() * 100 <= tried.max(1), "{} of {tried} missed:\n{}", missed.len(), missed.join("\n"));
}

#[test]
fn fuzz_poly() {
    let mut r = Rng::new(3);
    for i in 0..n() {
        let deg = r.int(1, 5) as usize;
        let mut roots: Vec<i64> = vec![];
        while roots.len() < deg {
            let k = r.int(-9, 9);
            if !roots.contains(&k) {
                roots.push(k);
            }
        }
        let lead = r.int(1, 4);
        let e = expr::mul(std::iter::once(int(lead as i128)).chain(roots.iter().map(|k| expr::sub(sym("x"), int(*k as i128)))).collect());
        let e = expr::expand(&e);
        let s = solve1(&e, "x", &|_| false).unwrap_or_else(|f| panic!("#{i} {e:?}: {f:?}"));
        let mut got: Vec<f64> = s.iter().map(|x| eval_f(x, &|_| None).unwrap()).collect();
        got.sort_by(|a, b| a.total_cmp(b));
        roots.sort();
        assert_eq!(got.len(), roots.len(), "#{i} {e:?}: {got:?} vs {roots:?}");
        for (g, k) in got.iter().zip(&roots) {
            assert!((g - *k as f64).abs() < 1e-7, "#{i} {e:?}: {got:?} vs {roots:?}");
        }
    }
}

#[test]
fn fuzz_system() {
    let mut r = Rng::new(4);
    let names = ["a", "b", "c"];
    for i in 0..n() {
        let k = r.int(2, 3) as usize;
        let sol: Vec<i64> = (0..k).map(|_| r.int(-9, 9)).collect();
        let mut eqs = vec![];
        let mut rows = vec![];
        for _ in 0..k {
            let co: Vec<i64> = (0..k).map(|_| r.int(-5, 5)).collect();
            let rhs: i64 = co.iter().zip(&sol).map(|(c, s)| c * s).sum();
            let lhs = co.iter().zip(names).map(|(c, n)| format!("({c}) * {n}")).collect::<Vec<_>>().join(" + ");
            eqs.push(format!("{lhs} = {rhs}"));
            rows.push(co);
        }
        // singular systems are skipped
        let det = if k == 2 {
            (rows[0][0] * rows[1][1] - rows[0][1] * rows[1][0]) as f64
        } else {
            let m = &rows;
            (m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])) as f64
        };
        if det == 0.0 {
            continue;
        }
        let mut env = Env::default();
        let src = format!("{} -> solve({})", eqs.join("; "), names[..k].join(", "));
        let out = run(&mut env, &src);
        for (j, s) in sol.iter().enumerate() {
            let g = val(&env, names[j]);
            assert!(g.is_some_and(|g| (g - *s as f64).abs() < 1e-7), "#{i} {src}: {} = {g:?}, want {s} {:?}", names[j], out.errors);
        }
    }
}

#[test]
fn fuzz_units() {
    let mut r = Rng::new(5);
    let pre = [("", 1.0), ("m", 1e-3), ("k", 1e3), ("µ", 1e-6), ("M", 1e6)];
    for i in 0..n() {
        let (pu, fu) = *r.pick(&pre);
        let (pi, fi) = *r.pick(&pre);
        let (u, iv) = (r.int(1, 999) as f64, r.int(1, 999) as f64);
        let mut env = Env::default();
        let src = [format!("U := {u} {pu}V"), format!("I := {iv} {pi}A"), "P := U I =".into(), "R := U/I =".into()];
        for s in &src {
            let o = run(&mut env, s);
            assert!(o.errors.is_empty(), "#{i} {s}: {:?}", o.errors);
        }
        assert!(close(val(&env, "P").unwrap(), u * fu * iv * fi, 1e-12), "#{i} P");
        assert!(close(val(&env, "R").unwrap(), u * fu / (iv * fi), 1e-12), "#{i} R");
        // and back through solving
        let o = run(&mut env, "P = U I_2 -> solve(I_2)");
        assert!(o.errors.is_empty(), "#{i}: {:?}", o.errors);
        assert!(close(val(&env, "I_2").unwrap(), iv * fi, 1e-9), "#{i} I_2");
    }
}

#[test]
fn fuzz_format() {
    let mut r = Rng::new(6);
    for i in 0..n() {
        let x = (r.f() - 0.5) * 10f64.powi(r.int(-20, 20) as i32);
        let digits = r.int(1, 15) as u32;
        let cfg = Cfg { digits, ..Cfg::default() };
        let (m, e) = fmt(x, &cfg);
        let back: f64 = m.parse::<f64>().unwrap() * 10f64.powi(e.unwrap_or(0));
        assert!(close(back, x, 10f64.powi(1 - digits as i32)), "#{i} {x} at {digits}: {m} e{e:?}");
        let sig = m.trim_start_matches('-').replace('.', "").trim_start_matches('0').trim_end_matches('0').len();
        assert!(sig <= digits as usize, "#{i} {x} at {digits}: {m}");
    }
}

#[test]
fn fuzz_hostile() {
    // whatever comes in, nothing panics: problems are messages
    let mut r = Rng::new(7);
    let toks = ["x", "y", ":=", "=", "->", "solve(x)", "(", ")", "[", "]", "2", "3.5", "mA", "kΩ", "^", "/", "*", "+", "-", ",", ";", "..", "f(", "sqrt(", "as", "[A]", "1e400", "0", "|", "≤", "@x", "\"", "frac", "digits(3)", "simplify", "π", "e"];
    for _ in 0..n() * 3 {
        let len = r.int(1, 12);
        let src: Vec<&str> = (0..len).map(|_| *r.pick(&toks)).collect();
        let src = src.join(" ");
        for mode in [Mode::Code, Mode::Math] {
            let mut env = Env::default();
            let _ = line(&mut env, &LineReq { src: "f(x) := x^2".into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
            let _ = line(&mut env, &LineReq { src: src.clone(), mode, cfg: Cfg::default(), listed: true });
        }
    }
    // symbolic algebra on random expressions: simplifying keeps the value
    let mut r = Rng::new(8);
    for i in 0..n() {
        let (src, _) = random_expr(&mut r, 3, Some(0.7));
        let mut env = Env::default();
        run(&mut env, &format!("g(x) := {src} -> simplify"));
        let Some(f) = env.funs.get("g") else { continue };
        let mut m = BTreeMap::new();
        m.insert("x".to_string(), expr::c(solveng_core::num::Num::F(0.7)));
        let a = eval_f(&simp(&f.body.subst_all(&m)), &|_| None);
        let mut e2 = Env::default();
        run(&mut e2, "x := 0.7");
        run(&mut e2, &format!("y := ({src}) ="));
        let b = val(&e2, "y");
        if let (Some(a), Some(b)) = (a, b) {
            assert!(close(a, b, 1e-7) || (a - b).abs() < 1e-12, "#{i} {src}: simplified {a}, direct {b}");
        }
        let _ = Func::Abs;
        let _: Option<Expr> = None;
    }
}

/// A function's return unit, inferred from its parameters' units and reported by defined(), is the unit of what
/// its calls give.
#[test]
fn return_units_match_the_calls() {
    use solveng_core::api::{DefinedReq, defined};
    let mut r = Rng::new(11);
    let units = ["V", "A", "Ω", "s", "m", "kg", "W", "mA", "kΩ", "Hz"];
    for _ in 0..n() {
        let mut env = Env::default();
        let (u, w, c) = (*r.pick(&units), *r.pick(&units), *r.pick(&units));
        let (a, b) = (r.int(-2, 2), r.int(-2, 2));
        let term = format!("x^({a}) y^({b})");
        let body = match r.int(0, 3) {
            0 => format!("3 * {term}"),
            1 => format!("[2 {c}] * {term}"),
            2 => format!("{term} + 2 {term} / [5 {c}] * [1 {c}]"),
            _ => format!("sqrt(x^({}) y^({}))", 2 * a, 2 * b),
        };
        let src = format!("f(x: {u}, y: {w}) := {body}");
        let out = run(&mut env, &src);
        assert!(out.errors.is_empty(), "{src}: {:?}", out.errors);
        let ret = env.funs["f"].ret.unwrap_or_else(|| panic!("{src}: no return unit"));
        let call = format!("v := f([2 {u}], [3 {w}])");
        let out = run(&mut env, &call);
        assert!(out.errors.is_empty(), "{call}: {:?}", out.errors);
        let v = value(&env.bind(&env.vars["v"].expr, &Default::default()).unwrap()).unwrap();
        assert_eq!(ret, v.d, "{src}; {call}");
        let rows = defined(&env, &DefinedReq { cfg: Cfg::default() }).unwrap();
        let plain = &rows.iter().find(|row| row.name == "f").unwrap().plain;
        assert!(plain.starts_with(&format!("(x: {u}, y: {w}): ")), "{src}: {plain}");
    }
}
