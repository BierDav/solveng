//! Solving equations: symbolically where it can be done (isolating, polynomials, linear systems, eliminating),
//! numerically otherwise (sign changes and Newton, all roots of polynomials, Newton for systems).

use std::collections::{BTreeMap, BTreeSet};

use crate::dim::Dim;
use crate::eval::eval_f;
use crate::complex::{CVal, Z};
use crate::expr::{self, Expr, Func, Val, int, simp};
use crate::num::{Num, Rat};
use crate::poly::{coeffs, factor, simplify, together};

#[derive(Clone, Debug, PartialEq)]
pub enum Fail {
    /// Not symbolically; numerically it may work.
    NotSymbolic,
    /// x isn't in it, or cancels out.
    Cancels,
}

/// Solutions of e = 0 for x, symbolic; other symbols are parameters. `numeric_params`: symbols whose values are
/// known – polynomials of degree 2 and more in only those are left to the numeric solver (all roots, as numbers).
pub fn solve1(e: &Expr, x: &str, numeric_params: &dyn Fn(&str) -> bool) -> Result<Vec<Expr>, Fail> {
    let e = simp(e);
    if !e.has(x) {
        return Err(Fail::Cancels);
    }
    let (n, d) = together(&e);
    // a product is zero where a factor is: each factor with x solved on its own, factors free of x left out
    let n = match factor(&n) {
        Some(f) if !f.is_zero() => {
            let with_x: Vec<&Expr> = f.num.iter().map(|(g, _)| g).filter(|g| g.has(x)).collect();
            match with_x.as_slice() {
                [] => return Err(Fail::Cancels),
                [g] => (*g).clone(),
                gs => {
                    let mut sols: Vec<Expr> = vec![];
                    for g in gs {
                        for s in solve1(g, x, numeric_params)? {
                            if !sols.contains(&s) && !simp(&d.subst(x, &s)).is_zero() {
                                sols.push(s);
                            }
                        }
                    }
                    if sols.iter().all(|s| eval_f(s, &|_| None).is_some()) {
                        sort_num(&mut sols);
                    }
                    return Ok(sols);
                }
            }
        }
        _ => n,
    };
    if let Some(cs) = coeffs(&n, x) {
        let mut cs = cs;
        while cs.len() > 1 && cs.last().unwrap().is_zero() {
            cs.pop();
        }
        let deg = cs.len() - 1;
        if deg == 0 {
            return Err(Fail::Cancels);
        }
        let only_known = cs.iter().all(|c| c.syms().iter().all(|s| numeric_params(s)));
        let root = |v: Expr| -> Option<Expr> { Some(simplify(&expr::sqrt(v))) };
        let sols: Vec<Expr> = if deg == 1 {
            vec![simplify(&expr::div(expr::neg(cs[0].clone()), cs[1].clone()))]
        } else if deg == 2 && !(only_known && cs.iter().any(|c| !c.syms().is_empty())) {
            let (c0, b, a) = (cs[0].clone(), cs[1].clone(), cs[2].clone());
            let disc = simplify(&expr::sub(expr::pow(b.clone(), int(2)), expr::mul(vec![int(4), a.clone(), c0.clone()])));
            if b.is_zero() {
                let square = simplify(&expr::div(expr::neg(c0), a));
                if square.is_zero() { vec![int(0)] }
                else if let Some(root) = root(square) {
                    vec![simp(&expr::neg(root.clone())), root]
                } else { vec![] }
            } else if disc.as_c().is_some() {
                if disc.is_zero() {
                    vec![simplify(&expr::div(expr::neg(b), expr::mul(vec![int(2), a])))]
                } else {
                    let Some(r) = root(disc.clone()) else { return Ok(vec![]) };
                    let mut v: Vec<Expr> = [-1, 1]
                        .iter()
                        .map(|s| simplify(&expr::div(expr::add(vec![expr::neg(b.clone()), expr::mul(vec![int(*s), r.clone()])]), expr::mul(vec![int(2), a.clone()]))))
                        .collect();
                    sort_num(&mut v);
                    v
                }
            } else {
                [-1, 1]
                    .iter()
                    .map(|s| simplify(&expr::div(expr::add(vec![expr::neg(b.clone()), expr::mul(vec![int(*s), expr::sqrt(disc.clone())])]), expr::mul(vec![int(2), a.clone()]))))
                    .collect()
            }
        } else if cs.iter().all(|c| c.syms().is_empty()) {
            // numbers only: every root, the real ones exactly where they can be
            let Some(cz) = cs.iter().map(crate::complex::parts).collect::<Option<Vec<CVal>>>() else { return Err(Fail::NotSymbolic) };
            let dx = poly_dim(&cz.iter().map(|z| if z.re.v.is_zero() { z.im.clone() } else { z.re.clone() }).collect::<Vec<_>>()).unwrap_or(Dim::ONE);
            let val = |x: f64| Val::new(snap(x), dx.clone());
            let all = complex_roots(&cz.iter().map(CVal::z).collect::<Vec<_>>());
            let complex = |z: &Z| z.im.abs() > 1e-9 * z.abs();
            let mut out: Vec<Expr> = vec![];
            if cz.iter().all(CVal::is_real) {
                out.extend(roots(&cz.iter().map(|z| z.re.v.f()).collect::<Vec<_>>()).into_iter().map(|r| Expr::C(val(r))));
                out.extend(all.iter().filter(|z| complex(z)).map(|z| crate::complex::expr(CVal { re: val(z.re), im: val(z.im) })));
            } else {
                out.extend(all.iter().map(|z| if complex(z) { crate::complex::expr(CVal { re: val(z.re), im: val(z.im) }) } else { Expr::C(val(z.re)) }));
            }
            out
        } else {
            return Err(Fail::NotSymbolic);
        };
        // roots of the denominator aren't solutions
        let sols = sols.into_iter().filter(|s| !simp(&d.subst(x, s)).is_zero()).collect();
        return Ok(sols);
    }
    // x once: isolate it
    if count(&e, x) == 1 {
        if let Some(s) = isolate(e.clone(), int(0), x) {
            return Ok(vec![simplify(&s)]);
        }
    }
    // both sides over one denominator may leave x once: (x + 1)/x = 3
    if count(&n, x) == 1 {
        if let Some(s) = isolate(n.clone(), int(0), x) {
            return Ok(vec![simplify(&s)]);
        }
    }
    Err(Fail::NotSymbolic)
}

/// What an equation l = r says, on the way there: l − r, simplified, and its numerator in factors.
#[derive(Clone, Debug, PartialEq)]
pub struct Reduced {
    pub difference: Expr,
    pub factored: Expr,
    pub holds: Holds,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Holds {
    /// For every value of its names.
    Always,
    Never,
    /// Where one of these equations holds: x = value where a factor gives one exactly, else the factor = 0.
    Where(Vec<(Expr, Expr)>),
}

/// l = r reduced to where it holds: a product is zero where one of its factors is, so the equation holds where a
/// factor of the numerator of l − r is zero (and its denominator isn't). Factors that are never zero (numbers,
/// units, e^…) say nothing. A factor polynomial in one name is solved for it exactly; one with a name in degree one
/// with a numeric coefficient is solved for that name; any other factor is kept as factor = 0. None where l − r
/// isn't exact (floats).
pub fn reduce(l: &Expr, r: &Expr) -> Option<Reduced> {
    let d = simp(&expr::sub(l.clone(), r.clone()));
    let f = factor(&d)?;
    let difference = simplify(&d);
    if f.is_zero() {
        return Some(Reduced { difference, factored: int(0), holds: Holds::Always });
    }
    let fs: Vec<Expr> = f.num.iter().map(|(g, _)| g.clone()).filter(|g| !never_zero(g)).collect();
    let factored = f.numerator();
    if fs.is_empty() {
        return Some(Reduced { difference, factored, holds: Holds::Never });
    }
    let exact = |e: &Expr| !e.contains(&|x| matches!(x, Expr::C(v) if !v.v.is_exact()));
    let mut out: Vec<(Expr, Expr)> = vec![];
    let mut push = |c: (Expr, Expr)| if !out.contains(&c) { out.push(c) };
    for g in fs {
        let names = g.syms();
        if let [x] = names.as_slice()
            && coeffs(&g, x).is_some()
            && let Ok(sols) = solve1(&g, x, &|_| false)
            && sols.iter().all(exact)
        {
            for s in sols {
                push((expr::sym(x), s));
            }
            continue;
        }
        let linear = names.iter().find_map(|x| match coeffs(&g, x)?.as_slice() {
            [c0, c1] if c1.as_c().is_some_and(|c| !c.is_zero()) => {
                // 3 − y rather than −(y − 3)
                let v = simplify(&expr::div(expr::neg(c0.clone()), c1.clone()));
                let flat = simp(&expr::expand(&v));
                Some((expr::sym(x), if flat.size() <= v.size() { flat } else { v }))
            }
            _ => None,
        });
        push(linear.unwrap_or((g, int(0))));
    }
    // by name, in the order the names appear, values ascending: a = 0 or a = 1 or p = 0
    let names = d.syms();
    let at = |c: &(Expr, Expr)| c.0.syms().first().and_then(|n| names.iter().position(|m| m == n)).unwrap_or(usize::MAX);
    let num = |c: &(Expr, Expr)| eval_f(&c.1, &|_| None);
    out.sort_by(|a, b| at(a).cmp(&at(b)).then_with(|| match (num(a), num(b)) {
        (Some(x), Some(y)) => x.total_cmp(&y),
        (x, y) => y.is_some().cmp(&x.is_some()),
    }));
    Some(Reduced { difference, factored, holds: Holds::Where(out) })
}

/// Whether a factor is zero nowhere: a nonzero number (a unit, π), or e to a power.
fn never_zero(g: &Expr) -> bool {
    match g {
        Expr::F(Func::Exp, _) => true,
        Expr::Pow(b, _) if **b == Expr::E => true,
        _ => g.syms().is_empty() && eval_f(g, &|_| None).is_some_and(|v| v != 0.0),
    }
}

fn sort_num(v: &mut [Expr]) {
    v.sort_by(|a, b| {
        let (x, y) = (eval_f(a, &|_| None), eval_f(b, &|_| None));
        match (x, y) {
            (Some(x), Some(y)) => x.total_cmp(&y),
            _ => std::cmp::Ordering::Equal,
        }
    });
}

/// The unit of x in Σ c_k x^k = 0, from two nonzero coefficients.
fn poly_dim(cs: &[Val]) -> Option<Dim> {
    let nz: Vec<(usize, &Val)> = cs.iter().enumerate().filter(|(_, c)| !c.is_zero()).collect();
    if nz.len() < 2 {
        return None;
    }
    let (i, a) = nz[0];
    let (j, b) = nz[nz.len() - 1];
    a.d.div(&b.d).pow(Rat::new(1, (j - i) as i128)?)
}

/// A float that is a simple fraction exactly: 0.5 → 1/2.
pub fn snap(x: f64) -> Num {
    match Rat::approx(x, 1000) {
        Some(r) if (r.to_f64() - x).abs() <= 1e-13 * x.abs().max(1e-300) => Num::R(r),
        _ => Num::F(x),
    }
}

fn count(e: &Expr, x: &str) -> usize {
    match e {
        Expr::Sym(n) => (n == x) as usize,
        _ => e.children().iter().map(|c| count(c, x)).sum(),
    }
}

/// l = r with x once in l: undo what's done to x, outside in.
fn isolate(mut l: Expr, mut r: Expr, x: &str) -> Option<Expr> {
    if !l.has(x) {
        std::mem::swap(&mut l, &mut r);
    }
    let mut steps = 0;
    while l != Expr::Sym(x.to_string()) {
        steps += 1;
        if steps > 200 {
            return None;
        }
        match l {
            Expr::Add(ts) => {
                let (inx, rest): (Vec<Expr>, Vec<Expr>) = ts.into_iter().partition(|t| t.has(x));
                r = expr::sub(r, expr::add(rest));
                l = inx.into_iter().next()?;
            }
            Expr::Mul(fs) => {
                let (inx, rest): (Vec<Expr>, Vec<Expr>) = fs.into_iter().partition(|t| t.has(x));
                r = expr::div(r, expr::mul(rest));
                l = inx.into_iter().next()?;
            }
            Expr::Pow(b, k) => {
                if b.has(x) {
                    // x^k = r → x = r^(1/k); even powers only the positive root (polynomials give both)
                    r = expr::pow(r, expr::pow(*k, int(-1)));
                    l = *b;
                } else {
                    // b^k = r → k = ln r / ln b
                    r = if *b == Expr::E { expr::func(Func::Ln, r) } else { expr::div(expr::func(Func::Ln, r), expr::func(Func::Ln, *b)) };
                    l = *k;
                }
            }
            Expr::F(f, a) => {
                r = match f {
                    Func::Log => expr::pow(int(10), r),
                    Func::Abs | Func::Cosh => return None,
                    _ => expr::func(f.inverse().or(match f {
                        Func::Sinh | Func::Tanh => None,
                        _ => None,
                    })?, r),
                };
                l = *a;
            }
            Expr::Number(..) | Expr::Cmp(..) => return None,
            _ => return None,
        }
    }
    Some(r)
}

// ---------- numeric ----------

/// Real roots of Σ c_i x^i (ascending, the highest nonzero), sorted.
pub fn roots(c: &[f64]) -> Vec<f64> {
    let mut c: Vec<f64> = c.to_vec();
    while c.len() > 1 && *c.last().unwrap() == 0.0 {
        c.pop();
    }
    let mut z = 0;
    while z < c.len() && c[z] == 0.0 {
        z += 1;
    }
    let mut rs = if z > 0 { vec![0.0] } else { vec![] };
    let c = &c[z..];
    let n = c.len().saturating_sub(1);
    if n == 1 {
        rs.push(-c[0] / c[1]);
    } else if n == 2 {
        let (cc, b, a) = (c[0], c[1], c[2]);
        let mut d = b * b - 4.0 * a * cc;
        if d < 0.0 && -d <= 1e-12 * b * b {
            d = 0.0;
        }
        if d >= 0.0 {
            let q = -0.5 * (b + b.signum() * d.sqrt());
            if q != 0.0 {
                rs.push(q / a);
                rs.push(cc / q);
            } else {
                rs.push(0.0);
            }
        }
    } else if n > 2 {
        // Durand–Kerner on the rescaled monic polynomial (roots of size ~1)
        let r0 = (c[0] / c[n]).abs().powf(1.0 / n as f64);
        let r0 = if r0 > 0.0 && r0.is_finite() { r0 } else { 1.0 };
        let m: Vec<f64> = (0..=n).map(|i| c[i] / c[n] * r0.powi(i as i32 - n as i32)).collect();
        let mut zs: Vec<(f64, f64)> = (0..n).map(|k| {
            let t = 2.0 * std::f64::consts::PI * k as f64 / n as f64 + 0.4;
            (t.cos(), t.sin())
        }).collect();
        let cm = |a: (f64, f64), b: (f64, f64)| (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0);
        let cs = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0, a.1 - b.1);
        let cabs = |a: (f64, f64)| (a.0 * a.0 + a.1 * a.1).sqrt();
        let cd = |a: (f64, f64), b: (f64, f64)| {
            let q = b.0 * b.0 + b.1 * b.1;
            ((a.0 * b.0 + a.1 * b.1) / q, (a.1 * b.0 - a.0 * b.1) / q)
        };
        for _ in 0..800 {
            let mut mx: f64 = 0.0;
            for k in 0..n {
                let zk = zs[k];
                let mut p = (1.0, 0.0);
                for i in (0..n).rev() {
                    p = cm(p, zk);
                    p.0 += m[i];
                }
                let mut den = (1.0, 0.0);
                for j in 0..n {
                    if j != k {
                        den = cm(den, cs(zk, zs[j]));
                    }
                }
                if cabs(den) == 0.0 {
                    den = (1e-12, 0.0);
                }
                let dz = cd(p, den);
                zs[k] = cs(zk, dz);
                mx = mx.max(cabs(dz));
            }
            if mx < 1e-15 {
                break;
            }
        }
        let pv = |y: f64| -> (f64, f64) { (0..=n).rev().fold((0.0, 0.0), |(v, g), i| (v * y + m[i], g * y + v)) };
        for (re, im) in zs {
            if im.abs() > 1e-6 * cabs((re, im)).max(1e-9) {
                continue;
            }
            let mut y = re;
            for _ in 0..8 {
                let (v, g) = pv(y);
                if g == 0.0 {
                    break;
                }
                let y2 = y - v / g;
                if pv(y2).0.abs() < v.abs() { y = y2 } else { break }
            }
            rs.push(y * r0);
        }
    }
    uniq(rs)
}

/// Every root of Σ c_k x^k with complex coefficients (Durand–Kerner, then Newton), in order of their real parts.
pub fn complex_roots(c: &[Z]) -> Vec<Z> {
    let zero = |z: &Z| z.re == 0.0 && z.im == 0.0;
    let mut c: Vec<Z> = c.to_vec();
    while c.len() > 1 && zero(c.last().unwrap()) {
        c.pop();
    }
    let lead = c.iter().take_while(|z| zero(z)).count();
    let mut out = vec![Z::real(0.0); lead.min(c.len().saturating_sub(1))];
    let c = &c[lead.min(c.len())..];
    let n = c.len().saturating_sub(1);
    if n == 0 {
        return out;
    }
    // monic and rescaled, so that the roots are of size ~1
    let r0 = c[0].div(c[n]).abs().powf(1.0 / n as f64);
    let r0 = if r0 > 0.0 && r0.is_finite() { r0 } else { 1.0 };
    let m: Vec<Z> = (0..=n).map(|i| c[i].div(c[n]).scale(r0.powi(i as i32 - n as i32))).collect();
    let p = |z: Z| (0..=n).rev().fold((Z::real(0.0), Z::real(0.0)), |(v, g), i| (v.mul(z).add(m[i]), g.mul(z).add(v)));
    let mut zs: Vec<Z> = (0..n).map(|k| {
        let t = 2.0 * std::f64::consts::PI * k as f64 / n as f64 + 0.4;
        Z::new(t.cos(), t.sin())
    }).collect();
    for _ in 0..800 {
        let mut most: f64 = 0.0;
        for k in 0..n {
            let mut den = Z::real(1.0);
            for j in 0..n {
                if j != k {
                    den = den.mul(zs[k].sub(zs[j]));
                }
            }
            if den.abs() == 0.0 {
                den = Z::real(1e-12);
            }
            let dz = p(zs[k]).0.div(den);
            zs[k] = zs[k].sub(dz);
            most = most.max(dz.abs());
        }
        if most < 1e-15 {
            break;
        }
    }
    for z in &mut zs {
        for _ in 0..8 {
            let (v, g) = p(*z);
            if g.abs() == 0.0 {
                break;
            }
            let next = z.sub(v.div(g));
            if p(next).0.abs() < v.abs() { *z = next } else { break }
        }
    }
    out.extend(zs.into_iter().map(|z| z.scale(r0).cleaned()));
    out.sort_by(|a, b| a.re.total_cmp(&b.re).then(a.im.total_cmp(&b.im)));
    out
}

pub fn uniq(mut rs: Vec<f64>) -> Vec<f64> {
    rs.sort_by(|a, b| a.total_cmp(b));
    let mut out: Vec<f64> = vec![];
    for r in rs {
        match out.last() {
            Some(&l) if (r - l).abs() <= 1e-7 * r.abs().max(l.abs()) => {}
            _ => out.push(r),
        }
    }
    out
}

/// A system as plain functions: residuals and Jacobian at a point.
pub struct NumSys {
    pub f: Vec<Expr>,
    pub j: Vec<Vec<Expr>>,
    pub xs: Vec<String>,
    /// Scale of each residual: |l| + |r| of its equation, for telling 0 from rounding.
    pub scale: Vec<Expr>,
}

impl NumSys {
    pub fn new(eqs: &[(Expr, Expr)], xs: &[String]) -> NumSys {
        let f: Vec<Expr> = eqs.iter().map(|(l, r)| simp(&expr::sub(l.clone(), r.clone()))).collect();
        let j = f.iter().map(|e| xs.iter().map(|x| e.diff(x)).collect()).collect();
        let scale = eqs.iter().map(|(l, r)| expr::add(vec![expr::func(Func::Abs, l.clone()), expr::func(Func::Abs, r.clone())])).collect();
        NumSys { f, j, xs: xs.to_vec(), scale }
    }
    fn at(&self, x: &[f64]) -> Option<(Vec<f64>, Vec<Vec<f64>>, Vec<f64>)> {
        let vars = |n: &str| self.xs.iter().position(|y| y == n).map(|i| x[i]);
        let f = self.f.iter().map(|e| eval_f(e, &vars)).collect::<Option<Vec<_>>>()?;
        let j = self.j.iter().map(|r| r.iter().map(|e| eval_f(e, &vars)).collect::<Option<Vec<_>>>()).collect::<Option<Vec<_>>>()?;
        let s = self.scale.iter().map(|e| eval_f(e, &vars)).collect::<Option<Vec<_>>>()?;
        Some((f, j, s))
    }
    /// Damped Newton from x0.
    pub fn newton(&self, x0: &[f64]) -> Option<Vec<f64>> {
        let mut x = x0.to_vec();
        let mut r = self.at(&x)?;
        let w: Vec<f64> = r.2.iter().map(|s| if *s > 0.0 { 1.0 / s } else { 1.0 }).collect();
        let norm = |f: &[f64]| f.iter().zip(&w).map(|(f, w)| (f * w).powi(2)).sum::<f64>();
        for _ in 0..60 {
            if norm(&r.0) == 0.0 {
                break;
            }
            let Some(dx) = gauss(&r.1, &r.0.iter().map(|f| -f).collect::<Vec<_>>()) else { break };
            let mut t = 1.0;
            let mut acc = None;
            for _ in 0..14 {
                let xn: Vec<f64> = x.iter().zip(&dx).map(|(a, b)| a + t * b).collect();
                if let Some(rn) = self.at(&xn) {
                    if norm(&rn.0) < norm(&r.0) {
                        acc = Some((xn, rn));
                        break;
                    }
                }
                t /= 2.0;
            }
            let Some((xn, rn)) = acc else { break };
            let done = x.iter().zip(&xn).all(|(a, b)| (a - b).abs() <= 1e-14 * a.abs().max(b.abs()));
            x = xn;
            r = rn;
            if done {
                break;
            }
        }
        let ok = r.0.iter().zip(&r.2).all(|(f, s)| f.abs() <= 1e-8 * s.max(1e-300) || *f == 0.0);
        ok.then_some(x)
    }
    /// Several solutions: Newton from deterministic pseudo-random starts around the scales.
    pub fn multi(&self, sc: &[f64]) -> Vec<Vec<f64>> {
        let n = self.xs.len();
        let mut starts = vec![sc.to_vec()];
        let mut seed: u64 = 20261019;
        let mut rnd = || {
            seed = (seed.wrapping_mul(1103515245).wrapping_add(12345)) % 2147483648;
            seed as f64 / 2147483648.0
        };
        for _ in 0..40 {
            let p: Vec<f64> = (0..n).map(|j| {
                let u = rnd();
                let sg = if rnd() < 0.3 { -1.0 } else { 1.0 };
                sg * sc[j] * 10f64.powf(8.0 * u - 4.0)
            }).collect();
            starts.push(p);
        }
        let mut sols: Vec<Vec<f64>> = vec![];
        let mut idle = 0;
        for s in starts {
            idle += 1;
            if !sols.is_empty() && idle > 12 {
                break;
            }
            let Some(x) = self.newton(&s) else { continue };
            if !sols.iter().any(|y| y.iter().zip(&x).all(|(a, b)| (a - b).abs() <= 1e-6 * a.abs().max(b.abs()) + 1e-300)) {
                sols.push(x);
                idle = 0;
            }
        }
        sols.sort_by(|a, b| a.iter().zip(b).map(|(x, y)| x.total_cmp(y)).find(|o| o.is_ne()).unwrap_or(std::cmp::Ordering::Equal));
        sols
    }
}

pub fn gauss(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = b.len();
    let mut m: Vec<Vec<f64>> = a.iter().zip(b).map(|(r, y)| {
        let mut r = r.clone();
        r.push(*y);
        r
    }).collect();
    for c in 0..n {
        let p = (c..n).max_by(|&i, &j| m[i][c].abs().total_cmp(&m[j][c].abs()))?;
        if m[p][c] == 0.0 {
            return None;
        }
        m.swap(p, c);
        for r in 0..n {
            if r != c && m[r][c] != 0.0 {
                let f = m[r][c] / m[c][c];
                for k in c..=n {
                    m[r][k] -= f * m[c][k];
                }
            }
        }
    }
    let x: Vec<f64> = (0..n).map(|i| m[i][n] / m[i][i]).collect();
    x.iter().all(|v| v.is_finite()).then_some(x)
}

/// All real solutions of f(x) = 0 (f numeric, x in SI): scan ±1e-15…1e15 × scale, bisect sign changes, Newton
/// from local minima of |f| (roots that touch zero).
pub fn scan(e: &Expr, x: &str, s: f64) -> Vec<f64> {
    let d = e.diff(x);
    let f = |t: f64| eval_f(e, &|n| (n == x).then_some(t));
    let scale = {
        // |terms| for telling a root from rounding
        let ts: Vec<Expr> = match e {
            Expr::Add(ts) => ts.clone(),
            t => vec![t.clone()],
        };
        move |t: f64| ts.iter().map(|u| eval_f(u, &|n| (n == x).then_some(t)).map(|v| v.abs()).unwrap_or(0.0)).sum::<f64>()
    };
    let ks: Vec<f64> = (-60..=60).map(|k| s * 10f64.powf(k as f64 / 4.0)).collect();
    let mut pts: Vec<f64> = ks.iter().rev().map(|t| -t).collect();
    pts.push(0.0);
    pts.extend(ks.iter());
    let vs: Vec<Option<f64>> = pts.iter().map(|&t| f(t)).collect();
    let mut rs = vec![];
    for i in 0..pts.len() {
        let Some(v) = vs[i] else { continue };
        if v == 0.0 {
            rs.push(pts[i]);
            continue;
        }
        let Some(Some(w)) = vs.get(i + 1) else { continue };
        if *w == 0.0 || (v > 0.0) == (*w > 0.0) {
            continue;
        }
        let (mut a, mut b, mut fa) = (pts[i], pts[i + 1], v);
        let mut ok = true;
        for _ in 0..200 {
            let m = (a + b) / 2.0;
            if m == a || m == b {
                break;
            }
            let Some(fm) = f(m) else {
                ok = false;
                break;
            };
            if fm == 0.0 {
                a = m;
                b = m;
                break;
            }
            if (fm > 0.0) == (fa > 0.0) {
                a = m;
                fa = fm;
            } else {
                b = m;
            }
        }
        let m = (a + b) / 2.0;
        if ok {
            if let Some(fm) = f(m) {
                if fm.abs() <= 1e-7 * scale(m).max(1e-300) {
                    rs.push(m);
                }
            }
        }
    }
    // roots without a sign change: Newton from local minima of |f|
    let sys = NumSys { f: vec![e.clone()], j: vec![vec![d]], xs: vec![x.to_string()], scale: vec![simp(&abs_terms(e))] };
    for i in 1..pts.len() - 1 {
        let (Some(u), Some(v), Some(w)) = (vs[i - 1], vs[i], vs[i + 1]) else { continue };
        if v.abs() > u.abs() || v.abs() > w.abs() {
            continue;
        }
        if let Some(r) = sys.newton(&[pts[i]]) {
            rs.push(r[0]);
        }
    }
    uniq(rs)
}

fn abs_terms(e: &Expr) -> Expr {
    match e {
        Expr::Add(ts) => expr::add(ts.iter().map(|t| expr::func(Func::Abs, t.clone())).collect()),
        t => expr::func(Func::Abs, t.clone()),
    }
}

// ---------- units of unknowns ----------

/// A dimension linear in the unknowns' dimensions: c + Σ a_j·dim(x_j); None is a literal 0 (fits any unit).
#[derive(Clone, Debug)]
struct Lin {
    c: BTreeMap<String, f64>,
    a: Vec<f64>,
}

fn combine_dims(a: &BTreeMap<String, f64>, b: &BTreeMap<String, f64>, scale: f64) -> BTreeMap<String, f64> {
    let mut c = a.clone();
    for (k, v) in b { *c.entry(k.clone()).or_default() += scale * v; }
    c
}

fn lin(e: &Expr, xs: &[String], known: &dyn Fn(&str) -> Option<Dim>, cons: &mut Vec<Lin>) -> Result<Option<Lin>, String> {
    let n = xs.len();
    let k = |d: Dim| Lin { c: d.0.iter().map(|(k, e)| (k.clone(), *e as f64 / 60.0)).collect(), a: vec![0.0; n] };
    let addl = |x: &Lin, y: &Lin, s: f64| Lin { c: combine_dims(&x.c, &y.c, s), a: x.a.iter().zip(&y.a).map(|(p, q)| p + s * q).collect() };
    let scale = |x: &Lin, f: f64| Lin { c: x.c.iter().map(|(k, z)| (k.clone(), z * f)).collect(), a: x.a.iter().map(|z| z * f).collect() };
    Ok(match e {
        Expr::Cases(v) => {
            let mut result = None;
            for (value, condition) in v {
                if let Some(test) = lin(condition, xs, known, cons)? { cons.push(test); }
                if let Some(branch) = lin(value, xs, known, cons)? {
                    if let Some(previous) = &result { cons.push(addl(&branch, previous, -1.0)); }
                    else { result = Some(branch); }
                }
            }
            result
        }
        Expr::C(v) => {
            if v.is_zero() && v.d.is_one() {
                None
            } else {
                Some(k(v.d.clone()))
            }
        }
        Expr::Pi | Expr::E | Expr::I => Some(k(Dim::ONE)),
        Expr::Const(_, backing) => crate::eval::dim_of(backing, &|_| None)?.map(k),
        Expr::Sym(s) => {
            if let Some(i) = xs.iter().position(|y| y == s) {
                let mut a = vec![0.0; n];
                a[i] = 1.0;
                Some(Lin { c: BTreeMap::new(), a })
            } else {
                Some(k(known(s).ok_or_else(|| format!("'{s}' isn't defined"))?))
            }
        }
        Expr::Add(ts) => {
            let mut first: Option<Lin> = None;
            for t in ts {
                if let Some(l) = lin(t, xs, known, cons)? {
                    match &first {
                        None => first = Some(l),
                        Some(f) => cons.push(addl(&l, f, -1.0)),
                    }
                }
            }
            first
        }
        Expr::Mul(fs) => {
            let mut acc = Some(k(Dim::ONE));
            for f in fs {
                let l = lin(f, xs, known, cons)?;
                acc = match (acc, l) {
                    (Some(a), Some(b)) => Some(addl(&a, &b, 1.0)),
                    _ => None,
                };
            }
            acc
        }
        Expr::Pow(b, x) => {
            let bl = lin(b, xs, known, cons)?;
            if xs.iter().any(|y| x.has(y)) {
                if let Some(bl) = bl {
                    cons.push(bl);
                }
                if let Some(xl) = lin(x, xs, known, cons)? {
                    cons.push(xl);
                }
                Some(k(Dim::ONE))
            } else {
                if let Some(xl) = lin(x, xs, known, cons)? {
                    cons.push(xl);
                }
                let p = eval_f(x, &|_| None).ok_or("cannot evaluate an exponent")?;
                bl.map(|b| scale(&b, p))
            }
        }
        Expr::F(f, a) => {
            let l = lin(a, xs, known, cons)?;
            if *f == Func::Abs {
                l
            } else {
                if let Some(l) = l {
                    cons.push(l);
                }
                Some(k(Dim::ONE))
            }
        }
        Expr::Vector(_) | Expr::Matrix(_) => return Err("solving requires scalar equations, not a vector or matrix".into()),
        Expr::Op(_, args) => {
            for a in args { lin(a, xs, known, cons)?; }
            None
        }
        Expr::Cmp(v, _) => {
            let ls: Vec<Option<Lin>> = v.iter().map(|x| lin(x, xs, known, cons)).collect::<Result<_, _>>()?;
            for w in ls.windows(2) {
                if let (Some(a), Some(b)) = (&w[0], &w[1]) {
                    cons.push(addl(a, b, -1.0));
                }
            }
            Some(k(Dim::ONE))
        }
        Expr::Call(_, args) => {
            for a in args { lin(a, xs, known, cons)?; }
            None
        }
        Expr::Subs(body, var, by) => {
            let d = crate::eval::dim_of(by, known)?;
            lin(body, xs, &|n| if n == var { d.clone() } else { known(n) }, cons)?
        }
        Expr::Derivative(body, var) => {
            let d = crate::eval::dim_of(body, known)?;
            match (d, known(var)) { (Some(a), Some(b)) => Some(k(a.mul(&b.powi(-1)))), _ => None }
        }
        Expr::InverseLaplace(..) => {
            crate::eval::dim_of(e, known)?;
            Some(k(Dim::ONE))
        }
        Expr::Sum(body, var, lo, hi) => {
            for bound in [lo, hi] {
                if let Some(d) = lin(bound, xs, known, cons)? { cons.push(d); }
            }
            lin(body, xs, &|n| if n == var { Some(Dim::ONE) } else { known(n) }, cons)?
        }
        Expr::Integral(body, var, lo, hi) => {
            let l = lin(lo, xs, known, cons)?;
            let h = lin(hi, xs, known, cons)?;
            if let (Some(a), Some(b)) = (&l, &h) { cons.push(addl(a, b, -1.0)); }
            let d = crate::eval::dim_of(lo, known)?.or(crate::eval::dim_of(hi, known)?);
            let body = lin(body, xs, &|s| if s == var { d.clone() } else { known(s) }, cons)?;
            match (l.or(h), body) { (Some(a), Some(b)) => Some(addl(&a, &b, 1.0)), _ => None }
        }
        // the value has the unit's dimension, the number none
        Expr::Number(a, _, d, _) => {
            if let Some(l) = lin(a, xs, known, cons)? { cons.push(addl(&l, &k(d.clone()), -1.0)); }
            Some(k(Dim::ONE))
        }
    })
}

/// The units of the unknowns from the equations: (dims per unknown, or None where the equations don't fix it).
pub fn infer_dims(eqs: &[(Expr, Expr)], xs: &[String], known: &dyn Fn(&str) -> Option<Dim>) -> Result<Vec<Option<Dim>>, String> {
    let n = xs.len();
    let mut cons = vec![];
    for (l, r) in eqs {
        let a = lin(l, xs, known, &mut cons)?;
        let b = lin(r, xs, known, &mut cons)?;
        if let (Some(a), Some(b)) = (a, b) {
            cons.push(Lin { c: combine_dims(&a.c, &b.c, -1.0), a: a.a.iter().zip(&b.a).map(|(p, q)| p - q).collect() });
        }
    }
    let identities: Vec<String> = cons.iter().flat_map(|l| l.c.keys().cloned()).collect::<BTreeSet<_>>().into_iter().collect();
    // rows: a | -c
    let mut rows: Vec<Vec<f64>> = cons.iter().map(|l| l.a.iter().copied().chain(identities.iter().map(|k| -l.c.get(k).copied().unwrap_or(0.0))).collect()).collect();
    let mut piv = vec![];
    let mut r = 0;
    for col in 0..n {
        let mut p = None;
        for i in r..rows.len() {
            if rows[i][col].abs() > 1e-9 && p.is_none_or(|q: usize| rows[i][col].abs() > rows[q][col].abs()) {
                p = Some(i);
            }
        }
        let Some(p) = p else { continue };
        rows.swap(p, r);
        let pv = rows[r][col];
        rows[r] = rows[r].iter().map(|z| z / pv).collect();
        for i in 0..rows.len() {
            if i != r && rows[i][col] != 0.0 {
                let f = rows[i][col];
                let rr = rows[r].clone();
                rows[i] = rows[i].iter().zip(&rr).map(|(x, y)| x - f * y).collect();
            }
        }
        piv.push((col, r));
        r += 1;
    }
    for row in &rows[r..] {
        if row[n..].iter().any(|z| z.abs() > 1e-6) {
            return Err("the units in these equations don't match".into());
        }
    }
    let mut out = vec![None; n];
    for (col, i) in piv {
        let mut d = Dim::ONE;
        for (k, z) in rows[i][n..].iter().enumerate() {
            let q = Rat::approx(*z, 60).ok_or("a unit to a power that isn't a simple fraction")?;
            let power = (q.mul(Rat::int(60)).unwrap().to_f64().round()) as i32;
            if power != 0 { d.0.insert(identities[k].clone(), power); }
        }
        out[col] = Some(d);
    }
    Ok(out)
}

// ---------- systems ----------

/// A system solved symbolically: one unknown at a time where an equation has only that one, then linear
/// elimination. Solutions as expressions in the parameters, or None.
pub fn solve_system(eqs: &[(Expr, Expr)], xs: &[String], numeric_params: &dyn Fn(&str) -> bool) -> Option<Vec<BTreeMap<String, Expr>>> {
    let mut es: Vec<Expr> = eqs.iter().map(|(l, r)| simp(&expr::sub(l.clone(), r.clone()))).collect();
    let mut left: Vec<String> = xs.to_vec();
    let mut got: Vec<(String, Expr)> = vec![];
    // one unknown in one equation, with one solution
    loop {
        let mut progress = false;
        for i in 0..es.len() {
            let vs: Vec<&String> = left.iter().filter(|x| es[i].has(x)).collect();
            if vs.len() != 1 {
                continue;
            }
            let x = vs[0].clone();
            if let Ok(s) = solve1(&es[i], &x, numeric_params) {
                if s.len() == 1 {
                    let v = s[0].clone();
                    es.remove(i);
                    for e in es.iter_mut() {
                        *e = simp(&e.subst(&x, &v));
                    }
                    for g in got.iter_mut() {
                        g.1 = simplify(&g.1.subst(&x, &v));
                    }
                    left.retain(|y| *y != x);
                    got.push((x, v));
                    progress = true;
                    break;
                }
            }
        }
        if !progress || es.is_empty() {
            break;
        }
    }
    if !left.is_empty() {
        if es.len() != left.len() {
            return None;
        }
        // linear: Σ a_ij x_j = b_i
        let mut a: Vec<Vec<Expr>> = vec![];
        let mut b: Vec<Expr> = vec![];
        for e in &es {
            let mut row = vec![];
            let mut rest = e.clone();
            for x in &left {
                let cs = coeffs(e, x)?;
                if cs.len() > 2 {
                    return None;
                }
                let c1 = cs.get(1).cloned().unwrap_or(int(0));
                if left.iter().any(|y| c1.has(y)) {
                    return None;
                }
                rest = simp(&rest.subst(x, &int(0)));
                row.push(c1);
            }
            a.push(row);
            b.push(simp(&expr::neg(rest)));
        }
        let n = left.len();
        for c in 0..n {
            let p = (c..n).find(|&i| !simplify(&a[i][c]).is_zero())?;
            a.swap(p, c);
            b.swap(p, c);
            for r in 0..n {
                if r == c || simplify(&a[r][c]).is_zero() {
                    continue;
                }
                let f = simplify(&expr::div(a[r][c].clone(), a[c][c].clone()));
                for k in c..n {
                    a[r][k] = simplify(&expr::sub(a[r][k].clone(), expr::mul(vec![f.clone(), a[c][k].clone()])));
                }
                b[r] = simplify(&expr::sub(b[r].clone(), expr::mul(vec![f.clone(), b[c].clone()])));
            }
        }
        let sol: Vec<(String, Expr)> = (0..n).map(|i| (left[i].clone(), simplify(&expr::div(b[i].clone(), a[i][i].clone())))).collect();
        for (x, v) in &sol {
            for g in got.iter_mut() {
                g.1 = simplify(&g.1.subst(x, v));
            }
        }
        got.extend(sol);
    } else if !es.is_empty() {
        // equations left over without unknowns: they must hold
        for e in &es {
            if let Some(v) = simp(e).as_c() {
                if !v.is_zero() {
                    return Some(vec![]);
                }
            }
        }
    }
    Some(vec![got.into_iter().collect()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::*;

    fn none(_: &str) -> bool {
        false
    }

    #[test]
    fn symbolic() {
        // 0 = 1 + 3/x + y² → x = −3/(y² + 1)
        let e = add(vec![int(1), div(int(3), sym("x")), pow(sym("y"), int(2))]);
        let s = solve1(&e, "x", &none).unwrap();
        assert_eq!(s, vec![simp(&div(int(-3), add(vec![pow(sym("y"), int(2)), int(1)])))]);
        // Q/2 = I t → t = Q/(2 I)
        let e = sub(div(sym("Q"), int(2)), mul(vec![sym("I"), sym("t")]));
        let s = solve1(&e, "t", &|_| true).unwrap();
        assert_eq!(s, vec![simp(&div(sym("Q"), mul(vec![int(2), sym("I")])))]);
        // x² + 2x − 3 = 0 → −3, 1
        let e = add(vec![pow(sym("x"), int(2)), mul(vec![int(2), sym("x")]), int(-3)]);
        assert_eq!(solve1(&e, "x", &none).unwrap(), vec![int(-3), int(1)]);
        // x² = 2 → ±√2
        let e = sub(pow(sym("x"), int(2)), int(2));
        let s = solve1(&e, "x", &none).unwrap();
        assert_eq!(s[1], sqrt(int(2)));
        // e^x = 2 → ln 2
        let e = sub(pow(Expr::E, sym("x")), int(2));
        assert_eq!(solve1(&e, "x", &none).unwrap(), vec![func(Func::Ln, int(2))]);
    }

    #[test]
    fn numeric() {
        // cos w = w
        let e = sub(func(Func::Cos, sym("w")), sym("w"));
        assert!(solve1(&e, "w", &none).is_err());
        let r = scan(&e, "w", 1.0);
        assert_eq!(r.len(), 1);
        assert!((r[0] - 0.7390851332151607).abs() < 1e-12);
        let r = roots(&[-6.0, 11.0, -6.0, 1.0]);
        assert_eq!(r.len(), 3);
        assert!((r[2] - 3.0).abs() < 1e-12);
    }

    #[test]
    fn systems() {
        let eqs = vec![(add(vec![sym("a"), sym("b")]), int(3)), (sub(sym("a"), sym("b")), int(1))];
        let s = solve_system(&eqs, &["a".into(), "b".into()], &none).unwrap();
        assert_eq!(s[0]["a"], int(2));
        assert_eq!(s[0]["b"], int(1));
        // linear with parameters
        let eqs = vec![(add(vec![sym("x"), sym("y")]), sym("p")), (sub(sym("x"), sym("y")), sym("q"))];
        let s = solve_system(&eqs, &["x".into(), "y".into()], &none).unwrap();
        assert_eq!(s[0]["x"], simp(&div(add(vec![sym("p"), sym("q")]), int(2))));
    }

    #[test]
    fn units_of_unknowns() {
        // Q/2 = I t with Q in C and I in A: t in s
        let eqs = vec![(div(sym("Q"), int(2)), mul(vec![sym("I"), sym("t")]))];
        let known = |n: &str| match n {
            "Q" => Some(Dim::si([0, 0, 1, 1, 0, 0, 0])),
            "I" => Some(Dim::base(3)),
            _ => None,
        };
        let d = infer_dims(&eqs, &["t".into()], &known).unwrap();
        assert_eq!(d[0], Some(Dim::base(2)));
    }
}
