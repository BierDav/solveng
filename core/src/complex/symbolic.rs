//! Symbolic complex algebra. Unknown symbols and calls may be complex; being free of `i`
//! is not evidence of realness (consider ln(-x)). Keep opaque projections in that case.

use crate::expr::{Expr, Func, add, div, func, int, mul, neg, pow, simp, sub};

/// Conservative facts, valid before and after substituting complex function arguments.
fn real(e: &Expr) -> bool {
    match e {
        Expr::C(_) | Expr::Pi | Expr::E | Expr::Cmp(..) => true,
        Expr::Const(_, value) => real(value),
        Expr::Add(v) | Expr::Mul(v) => v.iter().all(real),
        Expr::Pow(b, n) => real(b) && n.as_num().is_some_and(|n| n.is_int()),
        Expr::F(f, a) => match f {
            Func::Re | Func::Im | Func::Abs | Func::Arg => true,
            Func::Conj | Func::Exp | Func::Sin | Func::Cos | Func::Tan |
            Func::Sinh | Func::Cosh | Func::Tanh | Func::Atan => real(a),
            _ => false,
        },
        Expr::Cases(v) => v.iter().all(|(value, _)| real(value)),
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

fn parts(e: &Expr) -> Parts {
    if real(e) {
        return Parts { re: e.clone(), im: int(0) };
    }
    let expanded = || -> Option<Parts> {
        (match e {
            Expr::I => Parts { re: int(0), im: int(1) },
            Expr::Add(ts) => {
                let ps: Vec<_> = ts.iter().map(parts).collect();
                Parts { re: simp(&add(ps.iter().map(|p| p.re.clone()).collect())), im: simp(&add(ps.into_iter().map(|p| p.im).collect())) }
            }
            Expr::Mul(fs) => {
                let mut acc = Parts { re: int(1), im: int(0) };
                for f in fs {
                    acc = acc.mul(&parts(f)).bounded()?;
                }
                acc
            }
            Expr::Pow(b, n) => {
                let n = n.as_num()?.as_int()?;
                let mut base = parts(b);
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
                let p = parts(a);
                Parts { re: p.re, im: simp(&neg(p.im)) }
            }
            Expr::Cases(v) => Parts {
                re: Expr::Cases(v.iter().map(|(v, c)| (parts(v).re, c.clone())).collect()),
                im: Expr::Cases(v.iter().map(|(v, c)| (parts(v).im, c.clone())).collect()),
            },
            _ => return None,
        }).bounded()
    };
    expanded().unwrap_or_else(|| Parts::opaque(e))
}

fn conjugate(e: &Expr) -> Expr {
    if real(e) { return e.clone(); }
    match e {
        Expr::I => simp(&neg(Expr::I)),
        Expr::Add(v) => simp(&add(v.iter().map(conjugate).collect())),
        Expr::Mul(v) => simp(&mul(v.iter().map(conjugate).collect())),
        Expr::Pow(b, n) if n.as_num().is_some_and(|n| n.is_int()) => simp(&pow(conjugate(b), (**n).clone())),
        Expr::F(Func::Conj, a) => (**a).clone(),
        // Entire functions commute with conjugation; branch-sensitive functions do not.
        Expr::F(f @ (Func::Exp | Func::Sin | Func::Cos | Func::Sinh | Func::Cosh), a) => simp(&func(*f, conjugate(a))),
        Expr::Cases(v) => Expr::Cases(v.iter().map(|(v, c)| (conjugate(v), c.clone())).collect()),
        _ => func(Func::Conj, e.clone()),
    }
}

pub fn projection(f: Func, a: &Expr) -> Expr {
    match f {
        Func::Re => parts(a).re,
        Func::Im => parts(a).im,
        Func::Conj => conjugate(a),
        _ => unreachable!("a complex projection"),
    }
}
