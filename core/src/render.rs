//! Expressions as Typst math markup: symbolic results, formulas as written, values put in.

use crate::expr::{self, Expr, Func, Val, base_exp, strip_coef};
use crate::linalg::Op;
use crate::format::{self, Cfg, frac_markup, name, number, pick, shown, unit};
use crate::num::Num;
use crate::parse::{Ast, MulOp, Rel};
use crate::units::{UnitExpr, Units};

/// Precedences: what binds how tightly, for parentheses.
fn function(name: &str) -> String {
    // Typst's math knows these names; the others are written as operators
    let typst = matches!(name, "exp" | "ln" | "log" | "sin" | "cos" | "tan" | "arcsin" | "arccos" | "arctan" | "sinh" | "cosh" | "tanh" | "det" | "tr" | "norm" | "at");
    if typst || crate::builtins::builtin(name).is_none() { name.into() } else { format!("op({})", format::q(name)) }
}

fn cases(branches: Vec<(String, String)>) -> String {
    format!("cases({})", branches.into_iter().map(|(value, condition)| {
        if condition == "\"1\"" { format!("{value} & upright(\"otherwise\")") }
        else { format!("{value} & upright(\"if\") {condition}") }
    }).collect::<Vec<_>>().join(", "))
}

/// A matrix from the markup of its rows.
pub fn mat(rows: Vec<Vec<String>>) -> String {
    format!("mat({})", rows.into_iter().map(|r| r.join(", ")).collect::<Vec<_>>().join("; "))
}

/// A matrix operation from the markup of its arguments: A^T, det(A), a × b.
fn matrix_op(op: Op, args: Vec<(String, u8)>) -> (String, u8) {
    match op {
        Op::Transpose => (format!("{}^(T)", wrap(args[0].clone(), P_ATOM)), P_POW),
        Op::Cross => (format!("{} times {}", wrap(args[0].clone(), P_POW), wrap(args[1].clone(), P_POW)), P_PROD),
        Op::Det | Op::Trace | Op::Norm => (format!("{}({})", op.name(), args[0].0), P_ATOM),
        op => (format!("{}({})", function(op.name()), args.into_iter().map(|a| a.0).collect::<Vec<_>>().join(", ")), P_ATOM),
    }
}

pub const P_REL: u8 = 0;
pub const P_SUM: u8 = 1;
pub const P_NEG: u8 = 2;
pub const P_PROD: u8 = 3;
pub const P_FRAC: u8 = 4;
pub const P_POW: u8 = 5;
pub const P_ATOM: u8 = 9;

pub fn rel(r: Rel) -> &'static str {
    match r {
        Rel::Eq => "=",
        Rel::Ne => "!=",
        Rel::Lt => "<",
        Rel::Gt => ">",
        Rel::Le => "<=",
        Rel::Ge => ">=",
    }
}

fn wrap((s, p): (String, u8), min: u8) -> String {
    if p < min { format!("lr(({s}))") } else { s }
}

pub struct R<'a> {
    pub units: &'a Units,
    pub cfg: &'a Cfg,
    /// Exact: rationals as fractions (symbolic results).
    pub exact: bool,
    /// Values put in for symbols (the step with numbers).
    pub sub: Option<&'a dyn Fn(&str) -> Option<(String, u8)>>,
}

impl R<'_> {
    pub fn cast_mark(&self, u: &UnitExpr, kind: &str) -> String {
        let how = if kind == "cast" { &self.cfg.show_casts } else { &self.cfg.show_types };
        let um = unit(u, self.units, self.cfg);
        let mark = if kind == "cast" { format!("upright({})", format::q(&self.cfg.words.cast)) } else { "class(\"normal\", :)".into() };
        match how.as_str() {
            "chip" => format!(" #h(0.15em) #svchip({}, ${mark}$, ${um}$)", format::q(kind)),
            "plain" => format!(" med {mark} med {um}"),
            _ => String::new(),
        }
    }

    pub fn val(&self, v: &Val) -> (String, u8) {
        if v.d.is_one() && v.u.is_none() {
            if let (true, Num::R(r)) = (self.exact, v.v) {
                if !r.is_int() && r.denom() <= self.cfg.max_denominator.max(1000) as i128 {
                    let s = frac_markup(r, self.cfg);
                    return (s, if r.is_neg() { P_NEG } else { P_FRAC });
                }
            }
            let (s, _) = number(v.v, v.raw.as_deref(), self.cfg);
            let p = if v.v.is_neg() { P_NEG } else { P_ATOM };
            return (s, p);
        }
        let cfg = if self.exact && v.v.rat().is_some_and(|r| !r.is_int()) { Cfg { form: "fraction".into(), ..self.cfg.clone() } } else { self.cfg.clone() };
        match pick(v, None, None, self.units, &cfg) {
            Ok(s) => {
                let (m, _) = shown(&s, self.units, &cfg);
                (m, if s.bare { P_ATOM } else if s.n.is_neg() { P_NEG } else { P_PROD })
            }
            Err(_) => (format!("{}", v.v.f()), P_ATOM),
        }
    }

    pub fn expr(&self, e: &Expr) -> (String, u8) {
        match e {
            Expr::Cases(v) => (cases(v.iter().map(|(value, condition)| (self.expr(value).0, self.expr(condition).0)).collect()), P_ATOM),
            Expr::Vector(v) => (format!("vec({})", v.iter().map(|e| self.expr(e).0).collect::<Vec<_>>().join(", ")), P_ATOM),
            Expr::Matrix(rs) => (mat(rs.iter().map(|r| r.iter().map(|e| self.expr(e).0).collect()).collect()), P_ATOM),
            Expr::Op(op, args) => matrix_op(*op, args.iter().map(|a| self.expr(a)).collect()),
            Expr::C(v) => self.val(v),
            Expr::Sym(n) => self.sub.and_then(|f| f(n)).unwrap_or_else(|| (name(n), P_ATOM)),
            Expr::Call(n, args) => (format!("{}({})", name(n), args.iter().map(|a| self.expr(a).0).collect::<Vec<_>>().join(", ")), P_ATOM),
            Expr::Pi => ("pi".into(), P_ATOM),
            Expr::E => ("e".into(), P_ATOM),
            Expr::Add(ts) => {
                let mut s = String::new();
                for (i, t) in ts.iter().enumerate() {
                    let (c, r) = strip_coef(t);
                    if i > 0 && c.v.is_neg() {
                        let pos = match r {
                            None => Expr::C(c.neg()),
                            Some(r) => expr::simp(&expr::mul(vec![Expr::C(c.neg()), r])),
                        };
                        s += " - ";
                        s += &wrap(self.expr(&pos), P_PROD);
                    } else {
                        if i > 0 {
                            s += " + ";
                        }
                        s += &wrap(self.expr(t), P_NEG);
                    }
                }
                (s, P_SUM)
            }
            Expr::Mul(_) | Expr::Pow(..) => self.product(e),
            Expr::F(f, a) => match f {
                Func::Abs => (format!("abs({})", self.expr(a).0), P_ATOM),
                Func::Exp => (format!("e^({})", self.expr(a).0), P_POW),
                _ => (format!("{}({})", function(f.name()), self.expr(a).0), P_ATOM),
            },
            Expr::Cmp(v, rs) => {
                let mut s = wrap(self.expr(&v[0]), P_SUM);
                for (r, x) in rs.iter().zip(&v[1..]) {
                    s += &format!(" {} {}", rel(*r), wrap(self.expr(x), P_SUM));
                }
                (s, P_REL)
            }
            Expr::InverseLaplace(a, x, by) => (format!("op(\"invlaplace\")({}, {}, {})", self.expr(a).0, name(x), self.expr(by).0), P_ATOM),
            Expr::Subs(a, x, by) => (format!("lr(({})) limits(|)_({} = {})", self.expr(a).0, name(x), self.expr(by).0), P_PROD),
            Expr::Derivative(a, x) => (format!("frac(dif, dif {}) lr(({}))", name(x), self.expr(a).0), P_PROD),
            Expr::Sum(a, v, lo, hi) => (format!("sum_({} = {})^({}) ({})", name(v), self.expr(lo).0, self.expr(hi).0, self.expr(a).0), P_PROD),
            Expr::Integral(a, v, lo, hi) => (format!("integral_({})^({}) {} dif {}", self.expr(lo).0, self.expr(hi).0, self.expr(a).0, name(v)), P_PROD),
            Expr::Cast(a, _, _, u) => (format!("{}{}", wrap(self.expr(a), P_POW), self.cast_mark(u, "cast")), P_PROD),
        }
    }

    fn product(&self, e: &Expr) -> (String, u8) {
        // a power on its own: √x, x², e^x
        if let Expr::Pow(b, x) = e {
            if let Some(n) = x.as_num() {
                if !n.is_neg() {
                    if n.same(Num::R(crate::num::Rat::new(1, 2).unwrap())) {
                        return (format!("sqrt({})", self.expr(b).0), P_ATOM);
                    }
                    if let Num::R(r) = n {
                        if r.numer() == 1 && r.denom() > 2 {
                            return (format!("root({}, {})", r.denom(), self.expr(b).0), P_ATOM);
                        }
                    }
                }
            }
            if !x.as_num().is_some_and(|n| n.is_neg()) {
                if **b == Expr::E {
                    return (format!("e^({})", self.expr(x).0), P_POW);
                }
                return (format!("{}^({})", wrap(self.expr(b), P_ATOM), self.expr(x).0), P_POW);
            }
        }
        let (c, rest) = strip_coef(e);
        let neg = c.v.is_neg() && c.d.is_one();
        let c = if neg { c.neg() } else { c };
        let body = match rest {
            None => return self.val(&if neg { c.neg() } else { c }),
            Some(r) => {
                let full = if c.is_one() { r } else { expr::simp(&expr::mul(vec![Expr::C(c), r])) };
                let (n, d) = expr::num_den(&full);
                if d.is_one() {
                    self.factors(&n)
                } else {
                    format!("frac({}, {})", self.factors(&n), self.factors(&d))
                }
            }
        };
        if neg {
            (format!("-{body}"), P_NEG)
        } else {
            let p = if body.starts_with("frac(") { P_FRAC } else { P_PROD };
            (body, p)
        }
    }

    /// Factors joined: 2 x y, 3 mA · R, 2 (x + 1).
    fn factors(&self, e: &Expr) -> String {
        let fs: Vec<Expr> = match e {
            Expr::Mul(v) => v.clone(),
            e => vec![e.clone()],
        };
        let mut s = String::new();
        let mut prev_num = false;
        let mut prev_qty = false;
        for (i, f) in fs.iter().enumerate() {
            let (m, p) = self.expr(f);
            let put_in = matches!(f, Expr::Sym(n) if self.sub.is_some_and(|s| s(n).is_some()));
            let is_num = matches!(f, Expr::C(_)) || put_in;
            let is_qty = matches!(f, Expr::C(v) if !v.d.is_one() || v.u.is_some()) || put_in;
            if i > 0 {
                s += if is_num || prev_qty || (prev_num && p < P_ATOM && !matches!(f, Expr::Add(_))) { " dot " } else { " " };
            }
            let m = if fs.len() > 1 && p < P_PROD { format!("lr(({m}))") } else if fs.len() > 1 && p == P_PROD && is_qty && i > 0 { m } else { m };
            s += &m;
            prev_num = is_num;
            prev_qty = is_qty;
        }
        if fs.is_empty() {
            s = "1".into();
        }
        s
    }

    /// A formula as written; names with a value from `sub` are put in.
    pub fn ast(&self, a: &Ast, sub: &dyn Fn(&str) -> Option<(String, u8)>) -> (String, u8) {
        match a {
            Ast::Num(raw, v) => {
                let (m, _) = number(*v, Some(raw), self.cfg);
                (m, if v.is_neg() { P_NEG } else { P_ATOM })
            }
            Ast::Qty(n, u) => {
                let (m, _) = self.ast(n, sub);
                (format!("{} thin {}", wrap((m, P_ATOM), P_POW), unit(u, self.units, self.cfg)), P_PROD)
            }
            Ast::Unit(u) => (unit(u, self.units, self.cfg), P_ATOM),
            Ast::Name(n) => sub(n).unwrap_or_else(|| (name(n), P_ATOM)),
            Ast::Neg(x) => (format!("-{}", wrap(self.ast(x, sub), P_PROD)), P_NEG),
            Ast::Add(ts) => {
                let mut s = String::new();
                for (i, (n, t)) in ts.iter().enumerate() {
                    let m = wrap(self.ast(t, sub), if *n { P_PROD } else { P_NEG });
                    if i == 0 {
                        s += &if *n { format!("-{m}") } else { m };
                    } else {
                        s += &format!(" {} {m}", if *n { "-" } else { "+" });
                    }
                }
                (s, P_SUM)
            }
            Ast::Mul(fs) => {
                let mut s = String::new();
                // what the factor before was: a plain number, a value with a unit (or put in), anything else
                let mut prev = 0u8;
                for (i, (op, f)) in fs.iter().enumerate() {
                    let substituted = matches!(f, Ast::Name(n) if sub(n).is_some());
                    let (m, p) = self.ast(f, sub);
                    let kind = if substituted || matches!(f, Ast::Qty(..)) { 2 } else if matches!(f, Ast::Num(..)) { 1 } else { 0 };
                    if i > 0 {
                        s += match op {
                            MulOp::Times => " times ",
                            MulOp::Dot => " dot ",
                            // values are joined by a dot: 2 · 30 mA, 3 V · 2 A; 2 x stays
                            MulOp::Implicit if kind > 0 || prev == 2 => " dot ",
                            MulOp::Implicit => " ",
                        };
                    }
                    s += &if p < P_PROD || (p == P_PROD && i > 0 && kind == 0) { format!("lr(({m}))") } else { m };
                    prev = kind;
                }
                (s, P_PROD)
            }
            Ast::Div(x, y) => (format!("frac({}, {})", self.ast(x, sub).0, self.ast(y, sub).0), P_FRAC),
            Ast::Pow(x, y) => (format!("{}^({})", wrap(self.ast(x, sub), P_ATOM), self.ast(y, sub).0), P_POW),
            Ast::Call(n, args) => (format!("{}({})", name(n), args.iter().map(|x| self.ast(x, sub).0).collect::<Vec<_>>().join(", ")), P_ATOM),
            Ast::Builtin(n, args) => {
                if n == "cases" {
                    return (cases(args.chunks(2).map(|pair| (self.ast(&pair[0], sub).0,
                        pair.get(1).map(|a| self.ast(a, sub).0).unwrap_or_else(|| "\"1\"".into()))).collect()), P_ATOM);
                }
                let a: Vec<String> = args.iter().map(|x| self.ast(x, sub).0).collect();
                match n.as_str() {
                    "diff" if a.len() == 2 => (format!("frac(dif, dif {}) lr(({}))", a[1], a[0]), P_PROD),
                    "sum" if a.len() == 4 => (format!("sum_({} = {})^({}) ({})", a[1], a[2], a[3], a[0]), P_PROD),
                    "integrate" if a.len() == 4 => (format!("integral_({})^({}) {} dif {}", a[2], a[3], a[0], a[1]), P_PROD),
                    "sqrt" => (format!("sqrt({})", a.join(", ")), P_ATOM),
                    "abs" => (format!("abs({})", a.join(", ")), P_ATOM),
                    "exp" => (format!("e^({})", a.join(", ")), P_POW),
                    "det" | "tr" | "vec" | "at" | "norm" => (format!("{n}({})", a.join(", ")), P_ATOM),
                    n if Func::of(n).is_some() => (format!("{}({})", function(Func::of(n).unwrap().name()), a.join(", ")), P_ATOM),
                    n => (format!("{}({})", function(n), a.join(", ")), P_ATOM),
                }
            }
            Ast::Arg(n, x) => (format!("{} : {}", name(n), self.ast(x, sub).0), P_ATOM),
            Ast::Index(x, idx) => (format!("{}[{}]", wrap(self.ast(x, sub), P_ATOM), idx.iter().map(|i| self.ast(i, sub).0).collect::<Vec<_>>().join(", ")), P_ATOM),
            Ast::Range(a, b) => (format!("{} .. {}", wrap(self.ast(a, sub), P_SUM), wrap(self.ast(b, sub), P_SUM)), P_REL),
            Ast::Abs(x) => (format!("abs({})", self.ast(x, sub).0), P_ATOM),
            Ast::Matrix(rs) => (mat(rs.iter().map(|r| r.iter().map(|x| self.ast(x, sub).0).collect()).collect()), P_ATOM),
            Ast::Cmp(first, rest) => {
                let mut s = wrap(self.ast(first, sub), P_SUM);
                for (r, x) in rest {
                    s += &format!(" {} {}", rel(*r), wrap(self.ast(x, sub), P_SUM));
                }
                (s, P_REL)
            }
            Ast::Cast(x, u) => (format!("{}{}", wrap(self.ast(x, sub), P_POW), self.cast_mark(u, "cast")), P_PROD),
        }
    }
}

/// Whether an expression has a factor with a negative exponent (shown as a fraction).
pub fn has_den(e: &Expr) -> bool {
    match e {
        Expr::Mul(fs) => fs.iter().any(|f| base_exp(f).1.as_num().is_some_and(|n| n.is_neg())),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::*;

    #[test]
    fn markup() {
        let u = Units::default();
        let c = Cfg::default();
        let r = R { units: &u, cfg: &c, exact: true, sub: None };
        let e = simp(&add(vec![sym("x"), pow(sym("y"), int(2)), int(1)]));
        assert_eq!(r.expr(&e).0, "x + y^(\"2\") + \"1\"");
        let e = simp(&div(int(-3), add(vec![pow(sym("y"), int(2)), int(1)])));
        assert_eq!(r.expr(&e).0, "-frac(\"3\", y^(\"2\") + \"1\")");
        let e = simp(&sub(sym("x"), int(1)));
        assert_eq!(r.expr(&e).0, "x - \"1\"");
        let e = simp(&mul(vec![int(2), sym("x")]));
        assert_eq!(r.expr(&e).0, "\"2\" x");
        let e = simp(&div(sym("x"), int(3)));
        assert_eq!(r.expr(&e).0, "frac(x, \"3\")");
        let e = simp(&sqrt(int(2)));
        assert_eq!(r.expr(&e).0, "sqrt(\"2\")");
    }
}
