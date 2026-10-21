//! Symbolic expressions: constants with units, free symbols, sums, products, powers, functions, comparisons;
//! kept in a canonical form by `simp`.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::dim::Dim;
use crate::num::{Num, Rat};
use crate::parse::Rel;
use crate::units::UnitExpr;

/// A constant: a number (in SI) with its dimension, and how it likes to be shown.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Val {
    pub v: Num,
    pub d: Dim,
    /// The unit it was written in (7.2 kΩ), for showing it again.
    pub u: Option<UnitExpr>,
    /// The number as written in that unit ("7.20").
    pub raw: Option<String>,
}

impl PartialEq for Val {
    fn eq(&self, o: &Val) -> bool {
        self.d == o.d && self.v.same(o.v)
    }
}

impl Val {
    pub fn num(v: Num) -> Val {
        Val { v, d: Dim::ONE, u: None, raw: None }
    }
    pub fn new(v: Num, d: Dim) -> Val {
        Val { v, d, u: None, raw: None }
    }
    pub fn int(n: i128) -> Val {
        Val::num(Num::int(n))
    }
    pub fn is_zero(&self) -> bool {
        self.v.is_zero()
    }
    pub fn is_one(&self) -> bool {
        self.v.is_one() && self.d.is_one()
    }
    pub fn mul(&self, o: &Val) -> Val {
        let u = match (&self.u, &o.u) {
            (Some(a), None) if o.d.is_one() => Some(a.clone()),
            (None, Some(b)) if self.d.is_one() => Some(b.clone()),
            _ => None,
        };
        Val { v: self.v.mul(o.v), d: self.d.mul(&o.d), u, raw: None }
    }
    /// None for different dimensions.
    pub fn add(&self, o: &Val) -> Option<Val> {
        if self.d != o.d {
            return None;
        }
        let u = if self.u == o.u { self.u.clone() } else { None };
        Some(Val { v: self.v.add(o.v), d: self.d, u, raw: None })
    }
    pub fn neg(&self) -> Val {
        Val { v: self.v.neg(), d: self.d, u: self.u.clone(), raw: None }
    }
    /// self^e for a dimensionless rational or float e.
    pub fn pow(&self, e: Num) -> Option<Val> {
        let d = if self.d.is_one() {
            Dim::ONE
        } else {
            match e {
                Num::R(r) => self.d.pow(r)?,
                Num::F(x) if x.fract() == 0.0 => self.d.powi(x as i32),
                _ => return None,
            }
        };
        let u = match (&self.u, e.as_int()) {
            (Some(u), Some(k)) if (k as i32) as i64 == k => Some(UnitExpr(u.0.iter().map(|(n, p)| (n.clone(), p * k as i32)).collect())),
            _ => None,
        };
        Some(Val { v: self.v.pow(e)?, d, u, raw: None })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Func {
    Exp,
    Ln,
    Log,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Abs,
    Rect,
    Tri,
    Sgn,
    Floor,
    Factorial,
}

impl Func {
    pub fn of(n: &str) -> Option<Func> {
        Some(match n {
            "exp" => Func::Exp,
            "ln" => Func::Ln,
            "log" => Func::Log,
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "asin" | "arcsin" => Func::Asin,
            "acos" | "arccos" => Func::Acos,
            "atan" | "arctan" => Func::Atan,
            "sinh" => Func::Sinh,
            "cosh" => Func::Cosh,
            "tanh" => Func::Tanh,
            "abs" => Func::Abs,
            "rect" => Func::Rect,
            "tri" => Func::Tri,
            "sgn" => Func::Sgn,
            "floor" => Func::Floor,
            "factorial" => Func::Factorial,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            Func::Exp => "exp",
            Func::Ln => "ln",
            Func::Log => "log",
            Func::Sin => "sin",
            Func::Cos => "cos",
            Func::Tan => "tan",
            Func::Asin => "arcsin",
            Func::Acos => "arccos",
            Func::Atan => "arctan",
            Func::Sinh => "sinh",
            Func::Cosh => "cosh",
            Func::Tanh => "tanh",
            Func::Abs => "abs",
            Func::Rect => "rect",
            Func::Tri => "tri",
            Func::Sgn => "sgn",
            Func::Floor => "floor",
            Func::Factorial => "factorial",
        }
    }
    pub fn inverse(self) -> Option<Func> {
        Some(match self {
            Func::Exp => Func::Ln,
            Func::Ln => Func::Exp,
            Func::Sin => Func::Asin,
            Func::Cos => Func::Acos,
            Func::Tan => Func::Atan,
            Func::Asin => Func::Sin,
            Func::Acos => Func::Cos,
            Func::Atan => Func::Tan,
            _ => return None,
        })
    }
    /// The value at x, None outside the domain.
    pub fn eval(self, x: f64) -> Option<f64> {
        let r = match self {
            Func::Exp => x.exp(),
            Func::Ln if x > 0.0 => x.ln(),
            Func::Log if x > 0.0 => x.log10(),
            Func::Sin => x.sin(),
            Func::Cos => x.cos(),
            Func::Tan if x.cos() != 0.0 => x.tan(),
            Func::Asin if x.abs() <= 1.0 => x.asin(),
            Func::Acos if x.abs() <= 1.0 => x.acos(),
            Func::Atan => x.atan(),
            Func::Sinh => x.sinh(),
            Func::Cosh => x.cosh(),
            Func::Tanh => x.tanh(),
            Func::Abs => x.abs(),
            Func::Rect => if x.abs() < 0.5 { 1.0 } else if x.abs() == 0.5 { 0.5 } else { 0.0 },
            Func::Tri => (1.0 - x.abs()).max(0.0),
            Func::Sgn => if x == 0.0 { 0.0 } else { x.signum() },
            Func::Floor => x.floor(),
            Func::Factorial if (0.0..=170.0).contains(&x) && x.fract() == 0.0 => (1..=x as u32).fold(1.0, |a, b| a * b as f64),
            _ => return None,
        };
        (r.is_finite()).then_some(r)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    C(Val),
    Sym(String),
    Pi,
    E,
    Vector(Vec<Expr>),
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    F(Func, Box<Expr>),
    /// Ordered value/condition pairs; only the first matching value is evaluated.
    Cases(Vec<(Expr, Expr)>),
    /// An unspecified function, resolved when its definition becomes available.
    Call(String, Vec<Expr>),
    /// a = b < c: 1 where all hold, else 0.
    Cmp(Vec<Expr>, Vec<Rel>),
    /// e as u: e's SI number as a value in u (factor, dims, unit).
    Cast(Box<Expr>, Num, Dim, UnitExpr),
    Sum(Box<Expr>, String, Box<Expr>, Box<Expr>),
    Integral(Box<Expr>, String, Box<Expr>, Box<Expr>),
    Derivative(Box<Expr>, String),
    /// Substitute after a formal derivative has been resolved, preserving its variable.
    Subs(Box<Expr>, String, Box<Expr>),
    /// An algorithmically evaluable causal inverse, with a bound transform variable.
    InverseLaplace(Box<Expr>, String, Box<Expr>),
}

pub fn c(v: Num) -> Expr {
    Expr::C(Val::num(v))
}
pub fn int(n: i128) -> Expr {
    c(Num::int(n))
}
pub fn rat(n: i128, d: i128) -> Expr {
    c(Num::R(Rat::new(n, d).unwrap()))
}
pub fn sym(n: &str) -> Expr {
    Expr::Sym(n.to_string())
}
pub fn add(v: Vec<Expr>) -> Expr {
    Expr::Add(v)
}
pub fn mul(v: Vec<Expr>) -> Expr {
    Expr::Mul(v)
}
pub fn pow(a: Expr, b: Expr) -> Expr {
    Expr::Pow(Box::new(a), Box::new(b))
}
pub fn neg(a: Expr) -> Expr {
    mul(vec![int(-1), a])
}
pub fn sub(a: Expr, b: Expr) -> Expr {
    add(vec![a, neg(b)])
}
pub fn div(a: Expr, b: Expr) -> Expr {
    mul(vec![a, pow(b, int(-1))])
}
pub fn sqrt(a: Expr) -> Expr {
    pow(a, rat(1, 2))
}
pub fn func(f: Func, a: Expr) -> Expr {
    Expr::F(f, Box::new(a))
}

impl Expr {
    pub fn is_zero(&self) -> bool {
        matches!(self, Expr::C(v) if v.is_zero())
    }
    pub fn is_one(&self) -> bool {
        matches!(self, Expr::C(v) if v.is_one())
    }
    pub fn as_c(&self) -> Option<&Val> {
        match self {
            Expr::C(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_num(&self) -> Option<Num> {
        match self {
            Expr::C(v) if v.d.is_one() => Some(v.v),
            _ => None,
        }
    }
    pub fn children(&self) -> Vec<&Expr> {
        match self {
            Expr::Cases(v) => v.iter().flat_map(|(value, condition)| [value, condition]).collect(),
            Expr::Vector(v) | Expr::Add(v) | Expr::Mul(v) | Expr::Cmp(v, _) | Expr::Call(_, v) => v.iter().collect(),
            Expr::Pow(a, b) | Expr::Subs(a, _, b) | Expr::InverseLaplace(a, _, b) => vec![a, b],
            Expr::Integral(body, _, lo, hi) | Expr::Sum(body, _, lo, hi) => vec![body, lo, hi],
            Expr::F(_, a) | Expr::Cast(a, ..) | Expr::Derivative(a, _) => vec![a],
            _ => vec![],
        }
    }
    pub fn map(&self, f: &mut impl FnMut(&Expr) -> Expr) -> Expr {
        match self {
            Expr::Cases(v) => Expr::Cases(v.iter().map(|(value, condition)| (f(value), f(condition))).collect()),
            Expr::Call(n, v) => Expr::Call(n.clone(), v.iter().map(&mut *f).collect()),
            Expr::Vector(v) => Expr::Vector(v.iter().map(&mut *f).collect()),
            Expr::Add(v) => Expr::Add(v.iter().map(&mut *f).collect()),
            Expr::Mul(v) => Expr::Mul(v.iter().map(&mut *f).collect()),
            Expr::Cmp(v, r) => Expr::Cmp(v.iter().map(&mut *f).collect(), r.clone()),
            Expr::Pow(a, b) => pow(f(a), f(b)),
            Expr::F(g, a) => func(*g, f(a)),
            Expr::Derivative(a, x) => Expr::Derivative(Box::new(f(a)), x.clone()),
            Expr::Subs(a, x, by) => Expr::Subs(Box::new(f(a)), x.clone(), Box::new(f(by))),
            Expr::InverseLaplace(a, x, by) => Expr::InverseLaplace(Box::new(f(a)), x.clone(), Box::new(f(by))),
            Expr::Cast(a, k, d, u) => Expr::Cast(Box::new(f(a)), *k, *d, u.clone()),
            Expr::Sum(a, v, lo, hi) => Expr::Sum(Box::new(f(a)), v.clone(), Box::new(f(lo)), Box::new(f(hi))),
            Expr::Integral(a, v, lo, hi) => Expr::Integral(Box::new(f(a)), v.clone(), Box::new(f(lo)), Box::new(f(hi))),
            e => e.clone(),
        }
    }
    /// Free symbols, in the order they first appear.
    pub fn syms(&self) -> Vec<String> {
        let mut out = vec![];
        self.syms_into(&mut out);
        out
    }
    fn syms_into(&self, out: &mut Vec<String>) {
        if let Expr::Subs(body, var, by) | Expr::InverseLaplace(body, var, by) = self {
            for n in body.syms().into_iter().filter(|n| n != var).chain(by.syms()) {
                if !out.contains(&n) { out.push(n); }
            }
            return;
        }
        if let Expr::Integral(body, var, lo, hi) | Expr::Sum(body, var, lo, hi) = self {
            for n in body.syms().into_iter().filter(|n| n != var).chain(lo.syms()).chain(hi.syms()) {
                if !out.contains(&n) { out.push(n); }
            }
            return;
        }
        if let Expr::Sym(n) = self {
            if !out.contains(n) {
                out.push(n.clone());
            }
        }
        for c in self.children() {
            c.syms_into(out);
        }
    }
    pub fn has(&self, x: &str) -> bool {
        match self {
            Expr::Sum(..) | Expr::Integral(..) | Expr::Subs(..) | Expr::InverseLaplace(..) => self.syms().iter().any(|n| n == x),
            Expr::Sym(n) => n == x,
            _ => self.children().iter().any(|c| c.has(x)),
        }
    }
    pub fn subst(&self, x: &str, by: &Expr) -> Expr {
        match self {
            Expr::Sum(..) | Expr::Integral(..) | Expr::Derivative(..) | Expr::Subs(..) | Expr::InverseLaplace(..) => self.subst_all(&BTreeMap::from([(x.to_string(), by.clone())])),
            Expr::Sym(n) if n == x => by.clone(),
            _ => self.map(&mut |c| c.subst(x, by)),
        }
    }
    pub fn subst_all(&self, m: &BTreeMap<String, Expr>) -> Expr {
        match self {
            Expr::Sum(a, v, lo, hi) => {
                let scope = Expr::Integral(a.clone(), v.clone(), lo.clone(), hi.clone()).subst_all(m);
                let Expr::Integral(a, v, lo, hi) = scope else { unreachable!() };
                Expr::Sum(a, v, lo, hi)
            }
            Expr::Integral(a, v, lo, hi) => {
                let mut local = m.clone();
                local.remove(v);
                let mut body = (**a).clone();
                let mut var = v.clone();
                if local.iter().any(|(key, by)| body.has(key) && by.has(v)) {
                    let mut i = 0;
                    loop {
                        let fresh = format!("{v}_integral{i}");
                        if !self.has(&fresh) && !local.values().any(|by| by.has(&fresh)) {
                            body = body.subst(v, &sym(&fresh));
                            var = fresh;
                            break;
                        }
                        i += 1;
                    }
                }
                Expr::Integral(Box::new(body.subst_all(&local)), var, Box::new(lo.subst_all(m)), Box::new(hi.subst_all(m)))
            }
            Expr::Derivative(_, var) if m.contains_key(var) => {
                // Differentiate first, substitute the evaluation point afterwards.
                let mut local = m.clone();
                let by = local.remove(var).unwrap();
                let scope = Expr::Integral(Box::new(self.clone()), var.clone(), Box::new(int(0)), Box::new(int(0))).subst_all(&local);
                let Expr::Integral(body, var, _, _) = scope else { unreachable!() };
                Expr::Subs(body, var, Box::new(by))
            }
            Expr::InverseLaplace(body, var, by) => {
                let scope = Expr::Integral(body.clone(), var.clone(), Box::new(int(0)), by.clone()).subst_all(m);
                let Expr::Integral(body, var, _, by) = scope else { unreachable!() };
                Expr::InverseLaplace(body, var, by)
            }
            Expr::Subs(body, var, by) => {
                // The same capture-avoiding scope as an integral, with `by` outside it.
                let scope = Expr::Integral(body.clone(), var.clone(), Box::new(int(0)), by.clone()).subst_all(m);
                let Expr::Integral(body, var, _, by) = scope else { unreachable!() };
                Expr::Subs(body, var, by)
            }
            Expr::Sym(n) => m.get(n).cloned().unwrap_or_else(|| self.clone()),
            _ => self.map(&mut |c| c.subst_all(m)),
        }
    }
    /// How big it is, for picking the simpler of two forms.
    pub fn size(&self) -> usize {
        1 + self.children().iter().map(|c| c.size()).sum::<usize>()
            + match self {
                // a fraction written out costs: 1/3 is bigger than 3
                Expr::C(v) => match v.v {
                    Num::R(r) if !r.is_int() => 1,
                    _ => 0,
                },
                Expr::Pow(_, b) if b.as_num().is_some_and(|x| x.is_neg()) => 1,
                _ => 0,
            }
    }
    pub fn contains(&self, p: &impl Fn(&Expr) -> bool) -> bool {
        p(self) || self.children().iter().any(|c| c.contains(p))
    }
    /// The symbolic derivative by x (simplified).
    pub fn diff(&self, x: &str) -> Expr {
        simp(&self.d(x))
    }
    fn d(&self, x: &str) -> Expr {
        if let Expr::Vector(v) = self { return Expr::Vector(v.iter().map(|e| e.d(x)).collect()); }
        if !self.has(x) {
            return int(0);
        }
        match self {
            Expr::Sym(_) => int(1),
            Expr::Add(v) => add(v.iter().map(|t| t.d(x)).collect()),
            Expr::Mul(v) => add(
                (0..v.len())
                    .map(|i| mul(v.iter().enumerate().map(|(j, f)| if i == j { f.d(x) } else { f.clone() }).collect()))
                    .collect(),
            ),
            Expr::Pow(a, b) => {
                if !b.has(x) {
                    mul(vec![(**b).clone(), pow((**a).clone(), add(vec![(**b).clone(), int(-1)])), a.d(x)])
                } else {
                    // a^b = exp(b ln a)
                    mul(vec![self.clone(), add(vec![mul(vec![b.d(x), func(Func::Ln, (**a).clone())]), mul(vec![(**b).clone(), a.d(x), pow((**a).clone(), int(-1))])])])
                }
            }
            Expr::Call(..) | Expr::Cases(..) | Expr::Derivative(..) | Expr::Subs(..) | Expr::InverseLaplace(..) => return Expr::Derivative(Box::new(self.clone()), x.into()),
            Expr::F(f, a) => {
                let a0 = (**a).clone();
                let inner = match f {
                    Func::Exp => func(Func::Exp, a0),
                    Func::Ln => pow(a0, int(-1)),
                    Func::Log => pow(mul(vec![a0, func(Func::Ln, int(10))]), int(-1)),
                    Func::Sin => func(Func::Cos, a0),
                    Func::Cos => neg(func(Func::Sin, a0)),
                    Func::Tan => pow(func(Func::Cos, a0), int(-2)),
                    Func::Asin => pow(sub(int(1), pow(a0, int(2))), rat(-1, 2)),
                    Func::Acos => neg(pow(sub(int(1), pow(a0, int(2))), rat(-1, 2))),
                    Func::Atan => pow(add(vec![int(1), pow(a0, int(2))]), int(-1)),
                    Func::Sinh => func(Func::Cosh, a0),
                    Func::Cosh => func(Func::Sinh, a0),
                    Func::Tanh => pow(func(Func::Cosh, a0), int(-2)),
                    Func::Tri => {
                        // The pulse is continuous, so its first derivative has no impulses.
                        // Strict intervals leave its three corners undefined, unless a
                        // stationary inner function smooths the corner (tri is Lipschitz).
                        let below = |n| Expr::Cmp(vec![a0.clone(), int(n)], vec![Rel::Lt]);
                        let between = |lo, hi| Expr::Cmp(vec![int(lo), a0.clone(), int(hi)], vec![Rel::Lt, Rel::Lt]);
                        Expr::Cases(vec![
                            (int(0), Expr::Cmp(vec![a.d(x), int(0)], vec![Rel::Eq])),
                            (int(0), below(-1)),
                            (int(1), between(-1, 0)),
                            (int(-1), between(0, 1)),
                            (int(0), Expr::Cmp(vec![a0.clone(), int(1)], vec![Rel::Gt])),
                        ])
                    }
                    Func::Rect | Func::Sgn | Func::Floor | Func::Factorial => return Expr::Derivative(Box::new(self.clone()), x.into()),
                    Func::Abs => mul(vec![a0.clone(), pow(func(Func::Abs, a0), int(-1))]),
                };
                mul(vec![inner, a.d(x)])
            }
            Expr::Cast(a, k, d, u) => Expr::Cast(Box::new(a.d(x)), *k, *d, u.clone()),
            Expr::Sum(..) => Expr::Derivative(Box::new(self.clone()), x.into()),
            Expr::Integral(a, v, lo, hi) => {
                let boundary = sub(mul(vec![a.subst(v, hi), hi.d(x)]), mul(vec![a.subst(v, lo), lo.d(x)]));
                if v == x || !a.has(x) { boundary } else { add(vec![boundary, Expr::Integral(Box::new(a.d(x)), v.clone(), lo.clone(), hi.clone())]) }
            }
            _ => int(0),
        }
    }
}

// ---------- canonical order ----------

fn kind_rank(e: &Expr) -> u8 {
    match e {
        Expr::C(_) => 0,
        Expr::Pi | Expr::E => 1,
        Expr::Sym(_) => 2,
        Expr::Pow(b, _) if matches!(**b, Expr::Sym(_)) => 2,
        Expr::Pow(..) => 3,
        Expr::F(..) => 4,
        Expr::Add(_) => 5,
        _ => 6,
    }
}

/// The base and exponent of a factor: x^2 → (x, 2), x → (x, 1).
pub fn base_exp(e: &Expr) -> (Expr, Expr) {
    match e {
        Expr::Pow(b, x) => ((**b).clone(), (**x).clone()),
        _ => (e.clone(), int(1)),
    }
}

fn sym_name(e: &Expr) -> Option<&str> {
    match e {
        Expr::Sym(n) => Some(n),
        Expr::Pow(b, _) => sym_name(b),
        _ => None,
    }
}

/// Order of factors in a product: numbers, constants, symbols by name, the rest.
pub fn factor_cmp(a: &Expr, b: &Expr) -> Ordering {
    kind_rank(a).cmp(&kind_rank(b)).then_with(|| match (sym_name(a), sym_name(b)) {
        (Some(x), Some(y)) => name_cmp(x, y),
        _ => format!("{a:?}").cmp(&format!("{b:?}")),
    })
}

fn name_cmp(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase()).then(a.cmp(b))
}

/// The symbols of a term with their (numeric) exponents, for ordering sums.
fn degrees(e: &Expr) -> Vec<(String, f64)> {
    let fs: Vec<&Expr> = match e {
        Expr::Mul(v) => v.iter().collect(),
        _ => vec![e],
    };
    let mut out: Vec<(String, f64)> = vec![];
    for f in fs {
        let (b, x) = base_exp(f);
        if let Expr::Sym(n) = b {
            out.push((n, x.as_num().map(|n| n.f()).unwrap_or(1.0)));
        }
    }
    out.sort_by(|a, b| name_cmp(&a.0, &b.0));
    out
}

/// Order of terms in a sum: like a polynomial, x² + 2x + 1, x + y² + 1 (lexicographic in the symbols' names),
/// constants last.
pub fn term_cmp(a: &Expr, b: &Expr) -> Ordering {
    let (da, db) = (degrees(a), degrees(b));
    let ca = matches!(strip_coef(a).1, None);
    let cb = matches!(strip_coef(b).1, None);
    if ca != cb {
        return ca.cmp(&cb);
    }
    let mut names: Vec<&String> = da.iter().chain(db.iter()).map(|x| &x.0).collect();
    names.sort_by(|a, b| name_cmp(a, b));
    names.dedup();
    for n in names {
        let ea = da.iter().find(|x| &x.0 == n).map(|x| x.1).unwrap_or(0.0);
        let eb = db.iter().find(|x| &x.0 == n).map(|x| x.1).unwrap_or(0.0);
        if ea != eb {
            return eb.total_cmp(&ea);
        }
    }
    format!("{a:?}").cmp(&format!("{b:?}"))
}

/// A term split into its constant factor and the rest (None if it's only a constant).
pub fn strip_coef(e: &Expr) -> (Val, Option<Expr>) {
    match e {
        Expr::C(v) => (v.clone(), None),
        Expr::Mul(fs) => {
            let mut coef = Val::int(1);
            let mut rest = vec![];
            for f in fs {
                match f {
                    Expr::C(v) => coef = coef.mul(v),
                    _ => rest.push(f.clone()),
                }
            }
            let rest = match rest.len() {
                0 => None,
                1 => rest.pop(),
                _ => Some(Expr::Mul(rest)),
            };
            (coef, rest)
        }
        _ => (Val::int(1), Some(e.clone())),
    }
}

fn with_coef(c: Val, rest: Option<Expr>) -> Expr {
    match rest {
        None => Expr::C(c),
        Some(r) if c.is_one() => r,
        Some(Expr::Mul(mut fs)) => {
            fs.insert(0, Expr::C(c));
            Expr::Mul(fs)
        }
        Some(r) => Expr::Mul(vec![Expr::C(c), r]),
    }
}

// ---------- simplification ----------

/// The canonical form: flat sums and products, numbers folded exactly, like terms and equal bases combined,
/// sorted.
pub fn simp(e: &Expr) -> Expr {
    match e {
        Expr::Cases(v) => {
            let mut branches = vec![];
            for (value, condition) in v {
                let condition = simp(condition);
                if let Some(n) = condition.as_num() {
                    if n.is_zero() { continue; }
                    if branches.is_empty() { return simp(value); }
                    branches.push((simp(value), int(1)));
                    break;
                }
                branches.push((simp(value), condition));
            }
            Expr::Cases(branches)
        }
        Expr::C(_) | Expr::Sym(_) | Expr::Pi | Expr::E => e.clone(),
        Expr::Subs(body, var, by) => {
            let body = simp(body);
            let by = simp(by);
            if body.contains(&|e| matches!(e, Expr::Derivative(..))) {
                Expr::Subs(Box::new(body), var.clone(), Box::new(by))
            } else {
                simp(&body.subst(var, &by))
            }
        }
        Expr::Call(n, v) => Expr::Call(n.clone(), v.iter().map(simp).collect()),
        Expr::Derivative(a, x) => Expr::Derivative(Box::new(simp(a)), x.clone()),
        Expr::InverseLaplace(a, x, by) => Expr::InverseLaplace(Box::new(simp(a)), x.clone(), Box::new(simp(by))),
        Expr::Sum(a, v, lo, hi) => crate::calculus::sum(simp(a), v.clone(), simp(lo), simp(hi)),
        Expr::Integral(a, v, lo, hi) => crate::calculus::integral(simp(a), v.clone(), simp(lo), simp(hi)),
        Expr::Vector(v) => Expr::Vector(v.iter().map(simp).collect()),
        Expr::Add(v) => simp_add(v.iter().map(simp).collect()),
        Expr::Mul(v) => simp_mul(v.iter().map(simp).collect()),
        Expr::Pow(a, b) => simp_pow(simp(a), simp(b)),
        Expr::F(f, a) => simp_f(*f, simp(a)),
        Expr::Cmp(v, r) => simp_cmp(v.iter().map(simp).collect(), r.clone()),
        Expr::Cast(a, k, d, u) => {
            let a = simp(a);
            match &a {
                Expr::C(v) => Expr::C(Val { v: v.v.mul(*k), d: *d, u: Some(u.clone()), raw: None }),
                _ => Expr::Cast(Box::new(a), *k, *d, u.clone()),
            }
        }
    }
}

fn simp_add(v: Vec<Expr>) -> Expr {
    let mut flat = vec![];
    for t in v {
        match t {
            Expr::Add(ts) => flat.extend(ts),
            t => flat.push(t),
        }
    }
    // like terms: equal rest, coefficients of one dimension
    let mut groups: Vec<(Option<Expr>, Val)> = vec![];
    for t in flat {
        let (c, r) = strip_coef(&t);
        match groups.iter_mut().find(|g| g.0 == r && g.1.d == c.d) {
            Some(g) => g.1 = g.1.add(&c).unwrap(),
            None => groups.push((r, c)),
        }
    }
    let mut ts: Vec<Expr> = groups
        .into_iter()
        // a zero term goes, unless it's all there is with a unit (0 V)
        .filter(|(r, c)| !c.is_zero() || (r.is_none() && false))
        .map(|(r, c)| match r {
            None => Expr::C(c),
            Some(r) => simp_mul(vec![Expr::C(c), r]),
        })
        .collect();
    ts.sort_by(term_cmp);
    match ts.len() {
        0 => int(0),
        1 => ts.pop().unwrap(),
        _ => Expr::Add(ts),
    }
}

fn simp_mul(v: Vec<Expr>) -> Expr {
    let mut flat = vec![];
    for f in v {
        match f {
            Expr::Mul(fs) => flat.extend(fs),
            f => flat.push(f),
        }
    }
    let mut coef = Val::int(1);
    // base → exponent
    let mut bases: Vec<(Expr, Expr)> = vec![];
    for f in flat {
        if let Expr::C(v) = &f {
            coef = coef.mul(v);
            continue;
        }
        let (b, x) = base_exp(&f);
        // a number to a power that isn't exact stays: √2
        match bases.iter_mut().find(|g| g.0 == b) {
            Some(g) => g.1 = simp_add(vec![g.1.clone(), x]),
            None => bases.push((b, x)),
        }
    }
    if coef.is_zero() {
        return Expr::C(Val::new(Num::ZERO, coef.d));
    }
    let mut fs = vec![];
    for (b, x) in bases {
        match simp_pow(b, x) {
            Expr::C(v) => coef = coef.mul(&v),
            Expr::Mul(inner) => {
                for f in inner {
                    match f {
                        Expr::C(v) => coef = coef.mul(&v),
                        f => fs.push(f),
                    }
                }
            }
            f => fs.push(f),
        }
    }
    if coef.is_zero() {
        return Expr::C(Val::new(Num::ZERO, coef.d));
    }
    // equal bases may have come together again (√2·√2): one more round if anything repeats
    let n = fs.len();
    let mut seen: Vec<Expr> = vec![];
    let mut again = false;
    for f in &fs {
        let b = base_exp(f).0;
        if seen.contains(&b) {
            again = true;
        }
        seen.push(b);
    }
    if again && n > 1 {
        let mut v = vec![Expr::C(coef)];
        v.extend(fs);
        return simp_mul(v);
    }
    fs.sort_by(factor_cmp);
    if fs.is_empty() {
        return Expr::C(coef);
    }
    // a constant times a sum stays a product: 2 (x + 1)
    with_coef(coef, Some(if fs.len() == 1 { fs.pop().unwrap() } else { Expr::Mul(fs) }))
}

fn simp_pow(b: Expr, x: Expr) -> Expr {
    if x.is_zero() {
        return int(1);
    }
    if x.is_one() {
        return b;
    }
    if b.is_one() {
        return b;
    }
    if let (Expr::C(bv), Some(xn)) = (&b, x.as_num()) {
        if bv.is_zero() && !xn.is_neg() {
            return b;
        }
        // exact, or a float base: the value; a rational base to a non-exact power stays (√2)
        if let Some(r) = bv.pow(xn) {
            if r.v.is_exact() || !bv.v.is_exact() || !xn.is_exact() {
                return Expr::C(r);
            }
        }
        // √8 = 2√2: the largest exact factor comes out
        if let (Num::R(br), Num::R(xr)) = (bv.v, xn) {
            if bv.d.is_one() && !xr.is_int() && br.numer() > 0 && br.is_int() && xr.denom() == 2 {
                let n = br.numer();
                let mut k = 1i128;
                let mut i = 2i128;
                while i * i <= n && i < 10000 {
                    if (n / k) % (i * i) == 0 {
                        k *= i * i;
                    } else {
                        i += 1;
                    }
                }
                if k > 1 {
                    let out = simp_pow(Expr::C(Val::num(Num::R(Rat::int(k)))), x.clone());
                    let rest = simp_pow(Expr::C(Val::num(Num::R(Rat::int(n / k)))), x);
                    return simp_mul(vec![out, rest]);
                }
            }
        }
    }
    match b {
        // (x^a)^b = x^(ab) for whole b, or when a·b keeps the sign (√(x²) is |x|, left alone)
        Expr::Pow(bb, a) if x.as_num().is_some_and(|n| n.is_int()) || a.as_num().is_some_and(|n| n.is_int() && n.as_int().unwrap() % 2 == 1) => {
            simp_pow(*bb, simp_mul(vec![*a, x]))
        }
        // (a b)^n = a^n b^n for whole n
        Expr::Mul(fs) if x.as_num().is_some_and(|n| n.is_int()) => simp_mul(fs.into_iter().map(|f| simp_pow(f, x.clone())).collect()),
        // a constant with a unit to a whole power
        Expr::F(Func::Exp, a) => simp_f(Func::Exp, simp_mul(vec![*a, x])),
        b => pow(b, x),
    }
}

fn simp_f(f: Func, a: Expr) -> Expr {
    // inverse pairs: exp(ln x) = x, ln(exp x) = x
    if let Expr::F(g, inner) = &a {
        if (f == Func::Exp && *g == Func::Ln) || (f == Func::Ln && *g == Func::Exp) {
            return (**inner).clone();
        }
        if f == Func::Abs && *g == Func::Abs {
            return a;
        }
    }
    if f == Func::Ln && a == Expr::E {
        return int(1);
    }
    if f == Func::Ln {
        if let Expr::Pow(b, x) = &a {
            if **b == Expr::E {
                return (**x).clone();
            }
        }
    }
    if let Some(n) = a.as_num() {
        let exact = |r: Num| Expr::C(Val::num(r));
        match (f, n) {
            (Func::Sin | Func::Tan | Func::Asin | Func::Atan | Func::Sinh | Func::Tanh, x) if x.is_zero() => return exact(Num::ZERO),
            (Func::Cos | Func::Cosh | Func::Exp, x) if x.is_zero() => return exact(Num::ONE),
            (Func::Ln, x) if x.is_one() => return exact(Num::ZERO),
            (Func::Log, Num::R(r)) if r.is_int() && r.numer() > 0 => {
                let mut k = r.numer();
                let mut e = 0;
                while k % 10 == 0 {
                    k /= 10;
                    e += 1;
                }
                if k == 1 {
                    return exact(Num::int(e));
                }
            }
            (Func::Abs, x) => return exact(x.abs()),
            (Func::Rect, x) => return exact(if x.abs().cmp(Num::R(Rat::new(1, 2).unwrap())) == Ordering::Less { Num::ONE } else if x.abs().cmp(Num::R(Rat::new(1, 2).unwrap())) == Ordering::Equal { Num::R(Rat::new(1, 2).unwrap()) } else { Num::ZERO }),
            (Func::Tri, x) => return exact(if x.abs().cmp(Num::ONE) == Ordering::Less { Num::ONE.sub(x.abs()) } else { Num::ZERO }),
            (Func::Sgn, x) => return exact(if x.is_zero() { Num::ZERO } else if x.is_neg() { Num::int(-1) } else { Num::ONE }),
            (Func::Floor, Num::R(r)) => return int(r.numer().div_euclid(r.denom())),
            (Func::Factorial, x) if x.as_int().is_some_and(|k| (0..=33).contains(&k)) => return int((1..=x.as_int().unwrap() as i128).product()),
            (_, Num::F(x)) => {
                if let Some(y) = f.eval(x) {
                    return exact(Num::F(y));
                }
            }
            _ => {}
        }
    }
    if f == Func::Abs {
        if let Expr::C(v) = &a {
            return Expr::C(Val { v: v.v.abs(), ..v.clone() });
        }
    }
    func(f, a)
}

fn simp_cmp(v: Vec<Expr>, r: Vec<Rel>) -> Expr {
    // all known: 1 where every relation holds; values of different units are never equal (and not ordered)
    if v.iter().all(|e| matches!(e, Expr::C(_))) {
        let ok = r.iter().enumerate().all(|(i, rel)| {
            let (a, b) = (v[i].as_c().unwrap(), v[i + 1].as_c().unwrap());
            if a.d != b.d {
                return *rel == Rel::Ne;
            }
            rel.holds(a.v.cmp(b.v), a.v.same(b.v))
        });
        return int(ok as i128);
    }
    Expr::Cmp(v, r)
}

// ---------- expanding ----------

/// Products of sums multiplied out, whole powers of sums too (up to a size limit).
pub fn expand(e: &Expr) -> Expr {
    let e = simp(e);
    simp(&exp_(&e))
}

fn exp_(e: &Expr) -> Expr {
    match e {
        Expr::Add(v) => simp_add(v.iter().map(exp_).collect()),
        Expr::Mul(v) => {
            let fs: Vec<Expr> = v.iter().map(exp_).collect();
            let mut acc = vec![int(1)];
            for f in fs {
                let ts = match f {
                    Expr::Add(ts) => ts,
                    f => vec![f],
                };
                if acc.len() * ts.len() > 4096 {
                    return simp_mul(v.clone());
                }
                let mut next = vec![];
                for a in &acc {
                    for t in &ts {
                        next.push(simp_mul(vec![a.clone(), t.clone()]));
                    }
                }
                acc = next;
            }
            simp_add(acc)
        }
        Expr::Pow(b, x) => {
            let b = exp_(b);
            if let (Expr::Add(_), Some(n)) = (&b, x.as_num().and_then(|n| n.as_int())) {
                if (2..=12).contains(&n) {
                    return exp_(&Expr::Mul(vec![b; n as usize]));
                }
            }
            simp_pow(b, (**x).clone())
        }
        _ => e.clone(),
    }
}

/// Numerator and denominator of a term: x^-1 and powers with negative exponents go below.
pub fn num_den(e: &Expr) -> (Expr, Expr) {
    let fs: Vec<Expr> = match e {
        Expr::Mul(v) => v.clone(),
        _ => vec![e.clone()],
    };
    let (mut n, mut d) = (vec![], vec![]);
    for f in fs {
        match &f {
            Expr::Pow(b, x) if x.as_num().is_some_and(|k| k.is_neg()) => d.push(simp_pow((**b).clone(), simp_mul(vec![int(-1), (**x).clone()]))),
            Expr::C(v) if v.d.is_one() && matches!(v.v, Num::R(r) if !r.is_int()) => {
                let r = v.v.rat().unwrap();
                n.push(int(r.numer()));
                d.push(int(r.denom()));
            }
            _ => n.push(f),
        }
    }
    (simp_mul(n), simp_mul(d))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn x() -> Expr {
        sym("x")
    }
    fn y() -> Expr {
        sym("y")
    }

    #[test]
    fn folding() {
        // 1 + 3/3 x + y^2 = x + y² + 1
        let e = add(vec![int(1), mul(vec![div(int(3), int(3)), x()]), pow(y(), int(2))]);
        assert_eq!(simp(&e), add(vec![x(), pow(y(), int(2)), int(1)]));
        // x·x = x², x/x = 1
        assert_eq!(simp(&mul(vec![x(), x()])), pow(x(), int(2)));
        assert_eq!(simp(&div(x(), x())), int(1));
        // √8 = 2√2, √2·√2 = 2
        assert_eq!(simp(&sqrt(int(8))), mul(vec![int(2), sqrt(int(2))]));
        assert_eq!(simp(&mul(vec![sqrt(int(2)), sqrt(int(2))])), int(2));
        // 2x + 3x = 5x
        assert_eq!(simp(&add(vec![mul(vec![int(2), x()]), mul(vec![int(3), x()])])), mul(vec![int(5), x()]));
        // a·b/a = b
        assert_eq!(simp(&div(mul(vec![sym("a"), sym("b")]), sym("a"))), sym("b"));
    }

    #[test]
    fn order() {
        let e = simp(&add(vec![int(1), mul(vec![int(2), x()]), pow(x(), int(2))]));
        assert_eq!(e, add(vec![pow(x(), int(2)), mul(vec![int(2), x()]), int(1)]));
    }

    #[test]
    fn comparisons() {
        let eq = |a, b| simp(&Expr::Cmp(vec![a, b], vec![Rel::Eq]));
        assert_eq!(eq(int(1), int(2)), int(0));
        assert_eq!(eq(rat(1, 2), rat(2, 4)), int(1));
        let amp = Expr::C(Val::new(Num::ONE, Dim::base(3)));
        let volt = Expr::C(Val::new(Num::int(2), Dim::si([2, 1, -3, -1, 0, 0, 0])));
        assert_eq!(eq(amp, volt), int(0));
    }

    #[test]
    fn expanding() {
        let e = expand(&pow(add(vec![x(), int(1)]), int(2)));
        assert_eq!(e, add(vec![pow(x(), int(2)), mul(vec![int(2), x()]), int(1)]));
    }

    #[test]
    fn derivative() {
        let e = add(vec![pow(x(), int(3)), func(Func::Sin, x())]);
        assert_eq!(e.diff("x"), add(vec![mul(vec![int(3), pow(x(), int(2))]), func(Func::Cos, x())]));
    }
}
