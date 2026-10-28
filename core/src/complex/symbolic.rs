//! Symbolic complex algebra. Unknown symbols and calls may be complex; being free of `i`
//! is not evidence of realness (consider ln(-x)). Keep opaque projections in that case.

use crate::expr::{Expr, Func, add, div, func, int, mul, neg, pow, simp, sub};
use std::collections::BTreeSet;

/// Simplify with a local real-domain context, without changing stored definitions.
/// Bound variables shadow assumptions of the same name.
pub fn assuming_real(e: &Expr, names: &BTreeSet<String>) -> Expr {
    fn walk(e: &Expr, names: &BTreeSet<String>) -> Expr {
        let bound = match e {
            Expr::Sum(_, v, _, _) | Expr::Integral(_, v, _, _) |
            Expr::Derivative(_, v) | Expr::Subs(_, v, _) |
            Expr::Antiderivative(_, v, _) | Expr::InverseLaplace(_, v, _) => Some(v),
            _ => None,
        };
        let mut local = names.clone();
        if let Some(v) = bound { local.remove(v); }
        // The first child is the body; limits and evaluation points use the outer context.
        let mut child = 0;
        let mapped = e.map(&mut |a| {
            child += 1;
            walk(a, if bound.is_some() && child == 1 { &local } else { names })
        });
        match mapped {
            Expr::F(f @ (Func::Re | Func::Im), a) => {
                let p = parts(&a, names);
                if f == Func::Re { p.re } else { p.im }
            }
            Expr::F(Func::Conj, a) => conjugate(&a, names),
            e => simp(&e),
        }
    }
    crate::poly::simplify(&walk(e, names))
}

/// Find a sufficient real-domain condition for an identity, without assuming that
/// opaque signals themselves are real. Greedy deletion gives an inclusion-minimal
/// set, not necessarily a minimum-cardinality set. Limits bound automatic proof work.
pub fn real_conditions(difference: &Expr) -> Option<BTreeSet<String>> {
    let d = simp(difference);
    if d.size() > 512 { return None; }
    fn candidates(e: &Expr, out: &mut BTreeSet<String>) {
        if let Expr::F(Func::Re | Func::Im | Func::Conj, a) = e {
            if let Expr::Sym(n) = a.as_ref() { out.insert(n.clone()); }
        }
        for a in e.children() { candidates(a, out); }
    }
    let mut names = BTreeSet::new();
    candidates(&d, &mut names);
    // Only free names can be assumed; sum/integral indices are local.
    let free = d.syms();
    names.retain(|n| free.contains(n));
    if names.is_empty() || names.len() > 16 || !assuming_real(&d, &names).is_zero() { return None; }
    for n in names.clone() {
        names.remove(&n);
        if !assuming_real(&d, &names).is_zero() { names.insert(n); }
    }
    (!names.is_empty()).then_some(names)
}

/// Conservative facts, valid before and after substituting complex function arguments.
fn real(e: &Expr, names: &BTreeSet<String>) -> bool {
    match e {
        Expr::C(_) | Expr::Pi | Expr::E | Expr::Cmp(..) => true,
        Expr::Sym(n) => names.contains(n),
        Expr::Const(_, value) => real(value, names),
        Expr::Add(v) | Expr::Mul(v) => v.iter().all(|a| real(a, names)),
        Expr::Pow(b, n) => real(b, names) && n.as_num().is_some_and(|n| n.is_int()),
        Expr::F(f, a) => match f {
            Func::Re | Func::Im | Func::Abs | Func::Arg => true,
            Func::Conj | Func::Exp | Func::Sin | Func::Cos | Func::Tan |
            Func::Sinh | Func::Cosh | Func::Tanh | Func::Atan => real(a, names),
            _ => false,
        },
        Expr::Cases(v) => v.iter().all(|(value, _)| real(value, names)),
        _ => false,
    }
}

#[derive(Clone)]
struct Parts {
    re: Expr,
    im: Expr,
}

impl Parts {
    fn opaque(e: &Expr) -> Self {
        Self { re: func(Func::Re, e.clone()), im: func(Func::Im, e.clone()) }
    }

    fn mul(&self, b: &Self) -> Self {
        Self {
            re: simp(&sub(mul(vec![self.re.clone(), b.re.clone()]), mul(vec![self.im.clone(), b.im.clone()]))),
            im: simp(&add(vec![mul(vec![self.re.clone(), b.im.clone()]), mul(vec![self.im.clone(), b.re.clone()])])),
        }
    }

    fn bounded(self) -> Option<Self> {
        // Products/powers of opaque complex expressions can grow exponentially. Leave
        // an exact projection when expansion exceeds this expression-node budget.
        (self.re.size() + self.im.size() <= 512).then_some(self)
    }
}

fn parts(e: &Expr, names: &BTreeSet<String>) -> Parts {
    if real(e, names) {
        return Parts { re: e.clone(), im: int(0) };
    }
    let expanded = || -> Option<Parts> {
        (match e {
            Expr::I => Parts { re: int(0), im: int(1) },
            Expr::Add(ts) => {
                let ps: Vec<_> = ts.iter().map(|a| parts(a, names)).collect();
                Parts { re: simp(&add(ps.iter().map(|p| p.re.clone()).collect())), im: simp(&add(ps.into_iter().map(|p| p.im).collect())) }
            }
            Expr::Mul(fs) => {
                let mut acc = Parts { re: int(1), im: int(0) };
                for f in fs {
                    acc = acc.mul(&parts(f, names)).bounded()?;
                }
                acc
            }
            Expr::Pow(b, n) => {
                let n = n.as_num()?.as_int()?;
                let mut base = parts(b, names);
                if n < 0 {
                    let norm = simp(&add(vec![pow(base.re.clone(), int(2)), pow(base.im.clone(), int(2))]));
                    base = Parts { re: simp(&div(base.re, norm.clone())), im: simp(&div(neg(base.im), norm)) };
                }
                let mut acc = Parts { re: int(1), im: int(0) };
                let mut k = n.unsigned_abs();
                while k > 0 {
                    if k & 1 != 0 { acc = acc.mul(&base).bounded()?; }
                    k >>= 1;
                    if k > 0 { base = base.mul(&base).bounded()?; }
                }
                acc
            }
            Expr::F(Func::Conj, a) => {
                let p = parts(a, names);
                Parts { re: p.re, im: simp(&neg(p.im)) }
            }
            Expr::Cases(v) => Parts {
                re: Expr::Cases(v.iter().map(|(v, c)| (parts(v, names).re, c.clone())).collect()),
                im: Expr::Cases(v.iter().map(|(v, c)| (parts(v, names).im, c.clone())).collect()),
            },
            _ => return None,
        }).bounded()
    };
    expanded().unwrap_or_else(|| Parts::opaque(e))
}

fn conjugate(e: &Expr, names: &BTreeSet<String>) -> Expr {
    if real(e, names) { return e.clone(); }
    match e {
        Expr::I => simp(&neg(Expr::I)),
        Expr::Add(v) => simp(&add(v.iter().map(|a| conjugate(a, names)).collect())),
        Expr::Mul(v) => simp(&mul(v.iter().map(|a| conjugate(a, names)).collect())),
        Expr::Pow(b, n) if n.as_num().is_some_and(|n| n.is_int()) => simp(&pow(conjugate(b, names), (**n).clone())),
        Expr::F(Func::Conj, a) => (**a).clone(),
        // Entire functions commute with conjugation; branch-sensitive functions do not.
        Expr::F(f @ (Func::Exp | Func::Sin | Func::Cos | Func::Sinh | Func::Cosh), a) => simp(&func(*f, conjugate(a, names))),
        Expr::Cases(v) => Expr::Cases(v.iter().map(|(v, c)| (conjugate(v, names), c.clone())).collect()),
        _ => func(Func::Conj, e.clone()),
    }
}

pub fn projection(f: Func, a: &Expr) -> Expr {
    let names = &BTreeSet::new();
    match f {
        Func::Re => parts(a, names).re,
        Func::Im => parts(a, names).im,
        Func::Conj => conjugate(a, names),
        _ => unreachable!("a complex projection"),
    }
}
