//! Bounded, exact identity proofs, separate from choosing a compact display form.
//! Entire trigonometric functions become Laurent polynomials in exponential generators.
//! Cardinal continuations and affine piecewise guards are checked at their exceptional points too.

use crate::expr::{self, Expr, Func, add, div, int, mul, neg, pow, rat, simp, sub};
use crate::num::Rat;
use crate::parse::Rel;
use crate::poly;

const MAX_NODES: usize = 2048;
const GENERATOR: &str = "\0identity-exp";
type Point = (String, Expr);

/// Failure means unknown, never false. Numeric sampling is deliberately not a proof rule.
pub(crate) fn proves(l: &Expr, r: &Expr) -> bool {
    if l.size() + r.size() > MAX_NODES { return false; }
    let relevant = |e: &Expr| matches!(e, Expr::F(Func::Sin | Func::Cos | Func::Tan | Func::Sinh | Func::Cosh | Func::Tanh | Func::Sinc | Func::Exp, _) | Expr::Cases(_))
        || matches!(e, Expr::Pow(b, _) if **b == Expr::E);
    if !l.contains(&relevant) && !r.contains(&relevant) { return false; }
    let unsupported = |e: &Expr| matches!(e,
        Expr::Numerical(..) | Expr::Integral(..) | Expr::Sum(..) | Expr::Derivative(..) | Expr::Subs(..) |
        Expr::Antiderivative(..) | Expr::InverseLaplace(..) | Expr::Vector(..) |
        Expr::Matrix(..) | Expr::Op(..)
    ) || matches!(e, Expr::Call(n, _) if n == GENERATOR);
    if l.contains(&unsupported) || r.contains(&unsupported) { return false; }
    let mut budget = 31;
    prove(l, r, &[], &mut budget).unwrap_or(false)
}

pub(crate) fn nonzero(e: &Expr) -> bool {
    match e {
        Expr::C(v) => v.v.is_exact() && !v.is_zero(),
        Expr::Pi | Expr::E | Expr::I => true,
        Expr::Mul(v) => v.iter().all(nonzero),
        Expr::Pow(b, n) if n.as_num().is_some_and(|n| n.is_int()) => nonzero(b),
        Expr::F(Func::Exp, _) => true,
        _ => false,
    }
}

/// Only split where the root and a nonzero coefficient are known exactly.
fn root(e: &Expr) -> Option<Point> {
    let names = e.syms();
    let [name] = names.as_slice() else { return None };
    let cs = poly::coeffs(e, name)?;
    let [c0, c1] = cs.as_slice() else { return None };
    nonzero(c1).then(|| (name.clone(), poly::simplify(&neg(div(c0.clone(), c1.clone())))))
}

pub(crate) fn guard(e: &Expr) -> Option<(Point, bool)> {
    let Expr::Cmp(v, rs) = e else { return None };
    if v.len() != 2 || rs.len() != 1 { return None; }
    let equal = match rs[0] { Rel::Eq => true, Rel::Ne => false, _ => return None };
    Some((root(&simp(&sub(v[0].clone(), v[1].clone())))?, equal))
}

fn contextual(e: &Expr, excluded: &[Point]) -> Expr {
    if let Some((point, equal)) = guard(e) {
        if excluded.contains(&point) { return int(if equal { 0 } else { 1 }); }
    }
    simp(&e.map(&mut |a| contextual(a, excluded)))
}

fn split(e: &Expr, excluded: &[Point]) -> Option<Point> {
    match e {
        Expr::Cases(v) => {
            for (_, condition) in v {
                if let Some((point, _)) = guard(condition) {
                    if !excluded.contains(&point) { return Some(point); }
                }
            }
        }
        Expr::F(Func::Sinc, a) => {
            if let Some(point) = root(a) {
                if !excluded.contains(&point) { return Some(point); }
            }
        }
        _ => {}
    }
    e.children().into_iter().find_map(|a| split(a, excluded))
}

fn prove(l: &Expr, r: &Expr, excluded: &[Point], budget: &mut usize) -> Option<bool> {
    if *budget == 0 { return None; }
    *budget -= 1;
    let l = contextual(l, excluded);
    let r = contextual(r, excluded);
    if l == r { return Some(true); }
    if let Some((name, value)) = split(&l, excluded).or_else(|| split(&r, excluded)) {
        // Prove the exceptional point before introducing any quotient for sinc.
        if !prove(&l.subst(&name, &value), &r.subst(&name, &value), excluded, budget)? { return Some(false); }
        let mut next = excluded.to_vec();
        next.push((name, value));
        return prove(&l, &r, &next, budget);
    }
    let l = exponential_form(&l, excluded)?;
    let r = exponential_form(&r, excluded)?;
    if l.size() + r.size() > MAX_NODES * 4 { return None; }
    let mut generators = vec![];
    collect(&l, &mut generators)?;
    collect(&r, &mut generators)?;
    let d = sub(replace(&l, &generators)?, replace(&r, &generators)?);
    let d = expr::expand(&d);
    if d.size() > MAX_NODES * 4 { return None; }
    Some(poly::factor(&d)?.is_zero())
}

fn map(e: &Expr, mut f: impl FnMut(&Expr) -> Option<Expr>) -> Option<Expr> {
    let mut ok = true;
    let mapped = e.map(&mut |a| match f(a) {
        Some(a) => a,
        None => { ok = false; a.clone() }
    });
    ok.then_some(mapped)
}

fn exponential_form(e: &Expr, excluded: &[Point]) -> Option<Expr> {
    let exp = |a| pow(Expr::E, a);
    let trig = |a: &Expr, cosine| {
        let z = simp(&mul(vec![Expr::I, a.clone()]));
        if cosine { mul(vec![rat(1, 2), add(vec![exp(z.clone()), exp(neg(z))])]) }
        else { div(sub(exp(z.clone()), exp(neg(z))), mul(vec![int(2), Expr::I])) }
    };
    Some(simp(&match e {
        Expr::F(Func::Sin, a) => trig(a, false),
        Expr::F(Func::Cos, a) => trig(a, true),
        Expr::F(Func::Tan, a) => div(trig(a, false), trig(a, true)),
        Expr::F(Func::Sinh, a) => mul(vec![rat(1, 2), sub(exp((**a).clone()), exp(neg((**a).clone())))]),
        Expr::F(Func::Cosh, a) => mul(vec![rat(1, 2), add(vec![exp((**a).clone()), exp(neg((**a).clone()))])]),
        Expr::F(Func::Tanh, a) => div(sub(exp((**a).clone()), exp(neg((**a).clone()))), add(vec![exp((**a).clone()), exp(neg((**a).clone()))])),
        Expr::F(Func::Sinc, a) if nonzero(a) || root(a).is_some_and(|p| excluded.contains(&p)) => div(trig(a, false), (**a).clone()),
        Expr::F(Func::Exp, a) => exp((**a).clone()),
        Expr::Add(_) | Expr::Mul(_) => map(e, |a| exponential_form(a, excluded))?,
        Expr::Pow(b, n) if **b != Expr::E && n.as_num().is_some_and(|n| n.is_int()) => map(e, |a| exponential_form(a, excluded))?,
        Expr::Cases(_) => return None,
        _ => e.clone(),
    }))
}

/// Terms of an exponent, with exact rational coefficients pulled out. Shared bases get a
/// common denominator, so exp(x/2) and exp(x) become powers of the same generator.
fn terms(e: &Expr) -> Option<Vec<(Expr, Rat)>> {
    let e = poly::simplify(&expr::expand(e));
    let ts = match e { Expr::Add(ts) => ts, e => vec![e] };
    ts.into_iter().map(|e| {
        let (c, base) = expr::strip_coef(&e);
        if !c.d.is_one() { return None; }
        Some((base.unwrap_or(int(1)), c.v.rat()?))
    }).collect()
}

fn collect(e: &Expr, generators: &mut Vec<(Expr, i128)>) -> Option<()> {
    if let Expr::Pow(b, a) = e {
        if **b == Expr::E {
            for (base, q) in terms(a)? {
                if let Some((_, den)) = generators.iter_mut().find(|(b, _)| *b == base) {
                    // lcm, via the reduced ratio; checked arithmetic keeps this bounded.
                    *den = den.checked_mul(Rat::new(q.denom(), *den)?.numer())?;
                    if *den > 120 { return None; }
                } else {
                    if generators.len() >= 12 || q.denom() > 120 { return None; }
                    generators.push((base, q.denom()));
                }
            }
            return Some(());
        }
    }
    // Only arithmetic is normalized; opaque functions retain their original meaning.
    if matches!(e, Expr::Add(_) | Expr::Mul(_) | Expr::Pow(..)) {
        for a in e.children() { collect(a, generators)?; }
    }
    Some(())
}

fn replace(e: &Expr, generators: &[(Expr, i128)]) -> Option<Expr> {
    if let Expr::Pow(b, a) = e {
        if **b == Expr::E {
            let mut fs = vec![];
            for (base, q) in terms(a)? {
                let (_, den) = generators.iter().find(|(b, _)| *b == base)?;
                let k = q.mul(Rat::int(*den))?;
                if !k.is_int() || k.numer().unsigned_abs() > 30 { return None; }
                let generator = Expr::Call(GENERATOR.into(), vec![base]);
                fs.push(pow(generator, int(k.numer())));
            }
            return Some(simp(&mul(fs)));
        }
    }
    if matches!(e, Expr::Add(_) | Expr::Mul(_) | Expr::Pow(..)) {
        return Some(simp(&map(e, |a| replace(a, generators))?));
    }
    Some(e.clone())
}
