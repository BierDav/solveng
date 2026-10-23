//! What a worksheet has defined so far, and turning what's written into expressions with it.
//!
//! Definitions are symbolic: `h := 5 z` is stored as 5·z even before z is defined. Names that are defined when
//! something is defined are put in right away (early), names that aren't stay free symbols and are looked up
//! when a value is needed (late): after `z := 20 cm`, `h =` is 1 m. A function's body works the same way, so
//! it needn't define everything it uses: `U(x) := h x a` with an undefined a, and `U(x)/a` cancels it.

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::dim::Dim;
use crate::expr::{self, Expr, Func, Val, simp};
use crate::linalg::Op;
use crate::num::{Num, Rat};
use crate::parse::{Ast, MulOp, Param, ParamType, Rel};
use crate::units::{UnitExpr, Units};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Given,
    Computed,
    Function,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Var {
    /// The value, symbolic: names defined when it was defined are put in, others are free.
    pub expr: Expr,
    /// As written, for @x and for the formula in steps.
    pub ast: Option<Ast>,
    /// The unit to show it in (picked when it was defined).
    pub unit: Option<UnitExpr>,
    /// The boundary of an inequality: x ≤ this.
    pub rel: Option<Rel>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fun {
    pub params: Vec<Param>,
    /// The body with the parameters as symbols.
    pub body: Expr,
    /// The body in the order it was written, before the canonical form sorted its products: for calls with
    /// matrices, whose products don't commute.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written: Option<Expr>,
    /// As written, unless solving produced the body.
    pub ast: Option<Ast>,
    pub src: String,
    /// The dimension of the result, when every parameter has a unit.
    pub ret: Option<Dim>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    /// Shown by defined().
    pub listed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Env {
    pub vars: IndexMap<String, Var>,
    pub funs: IndexMap<String, Fun>,
    /// What was defined, in order, for defined() and editors.
    pub order: Vec<Entry>,
}

impl Env {
    pub fn define_var(&mut self, name: &str, v: Var, kind: Kind, listed: bool) {
        self.funs.shift_remove(name);
        self.vars.insert(name.to_string(), v);
        self.note(name, kind, listed);
    }
    pub fn define_fun(&mut self, name: &str, f: Fun, listed: bool) {
        self.vars.shift_remove(name);
        self.funs.insert(name.to_string(), f);
        self.note(name, Kind::Function, listed);
    }
    /// Forgets a name, a value or a function alike (they share names); whether it was defined.
    pub fn clear(&mut self, name: &str) -> bool {
        let known = self.vars.shift_remove(name).is_some() | self.funs.shift_remove(name).is_some();
        self.order.retain(|e| e.name != name);
        known
    }
    /// Takes over what `other` defined, in its order, over what was here. Used when a worksheet
    /// exports its final definitions to the enclosing scope; absent names do not delete outer definitions.
    pub fn merge(&mut self, other: &Env) {
        for e in &other.order {
            if let Some(v) = other.vars.get(&e.name) {
                self.define_var(&e.name, v.clone(), e.kind, e.listed);
            } else if let Some(f) = other.funs.get(&e.name) {
                self.define_fun(&e.name, f.clone(), e.listed);
            }
        }
    }
    fn note(&mut self, name: &str, kind: Kind, listed: bool) {
        self.order.retain(|e| e.name != name);
        self.order.push(Entry { name: name.to_string(), kind, listed });
    }

    /// Defined names put in, recursively (free symbols of their values too, if those are defined by now);
    /// `keep` are parameters, never replaced.
    pub fn bind(&self, e: &Expr, keep: &BTreeSet<String>) -> Result<Expr, String> {
        self.bind_with_units(e, keep, &Units::default())
    }
    pub fn bind_with_units(&self, e: &Expr, keep: &BTreeSet<String>, units: &Units) -> Result<Expr, String> {
        self.bind_(e, keep, &mut vec![], units)
    }
    fn bind_(&self, e: &Expr, keep: &BTreeSet<String>, path: &mut Vec<String>, units: &Units) -> Result<Expr, String> {
        Ok(match e {
            Expr::Sym(n) if !keep.contains(n) => match self.vars.get(n) {
                Some(v) => {
                    if path.contains(n) {
                        return Err(format!("'{n}' is defined by itself ({} → {n})", path.join(" → ")));
                    }
                    path.push(n.clone());
                    let r = self.bind_(&v.expr, keep, path, units)?;
                    path.pop();
                    r
                }
                None => e.clone(),
            },
            Expr::Call(n, args) => {
                let args = args.iter().map(|a| self.bind_(a, keep, path, units)).collect::<Result<Vec<_>, _>>()?;
                if self.funs.contains_key(n) {
                    if path.contains(n) {
                        return Err(format!("'{n}' is defined by itself ({} → {n})", path.join(" → ")));
                    }
                    path.push(n.clone());
                    let lower = Lower::new(self, units);
                    let expanded = lower.apply(n, args)?;
                    let result = self.bind_(&expanded, keep, path, units)?;
                    path.pop();
                    result
                } else if self.vars.contains_key(n) {
                    return Err(format!("{n} is a variable – it can't be called as a function"));
                } else {
                    Expr::Call(n.clone(), args)
                }
            }
            Expr::InverseLaplace(body, var, by) => {
                let mut local = keep.clone();
                local.insert(var.clone());
                Expr::InverseLaplace(Box::new(self.bind_(body, &local, path, units)?), var.clone(), Box::new(self.bind_(by, keep, path, units)?))
            }
            Expr::Subs(body, var, by) => {
                let mut local = keep.clone();
                local.insert(var.clone());
                let body = self.bind_(body, &local, path, units)?;
                let by = self.bind_(by, keep, path, units)?;
                simp(&Expr::Subs(Box::new(body), var.clone(), Box::new(by)))
            }
            Expr::Derivative(body, var) => {
                let mut local = keep.clone();
                local.insert(var.clone());
                let body = self.bind_(body, &local, path, units)?;
                body.diff(var)
            }
            Expr::Sum(body, var, lo, hi) => {
                let mut local = keep.clone();
                local.insert(var.clone());
                crate::calculus::sum(self.bind_(body, &local, path, units)?, var.clone(), self.bind_(lo, keep, path, units)?, self.bind_(hi, keep, path, units)?)
            }
            Expr::Integral(body, var, lo, hi) => {
                let mut local = keep.clone();
                local.insert(var.clone());
                crate::calculus::integral(self.bind_(body, &local, path, units)?, var.clone(), self.bind_(lo, keep, path, units)?, self.bind_(hi, keep, path, units)?)
            }
            Expr::Sym(_) | Expr::C(_) | Expr::Pi | Expr::E => e.clone(),
            _ => {
                let mut err = None;
                let r = e.map(&mut |c| match self.bind_(c, keep, path, units) {
                    Ok(x) => x,
                    Err(m) => {
                        err.get_or_insert(m);
                        c.clone()
                    }
                });
                if let Some(m) = err {
                    return Err(m);
                }
                r
            }
        })
    }
}

/// Turning what's written into an expression.
pub struct Lower<'a> {
    pub env: &'a Env,
    pub units: &'a Units,
    /// Parameters of the function being defined (stay symbols).
    pub params: BTreeSet<String>,
    /// Units parameters get from the functions they're passed to: f(y) := q(y) with q(x: A) makes y an A.
    pub inferred: BTreeMap<String, ParamType>,
    /// Known values put in (false: names stay, for showing a formula).
    pub bind: bool,
}

impl<'a> Lower<'a> {
    pub fn new(env: &'a Env, units: &'a Units) -> Lower<'a> {
        Lower { env, units, params: BTreeSet::new(), inferred: BTreeMap::new(), bind: true }
    }

    pub fn expr(&mut self, a: &Ast) -> Result<Expr, String> {
        let e = self.raw(a)?;
        if self.bind { self.env.bind_with_units(&e, &self.params, self.units) } else { Ok(e) }
    }

    /// `-> substitute(x, …)`: the named variables replaced by their formulas as written, every other name kept.
    /// A formula using another listed name gets that one's formula too; the name being put in stays inside its own
    /// formula (x := x + 1 shows x + 1, x the older one), as does a sum's or integral's index. The names put in
    /// are added to `used`.
    pub fn substitute(&self, e: &Expr, names: &[String], used: &mut BTreeSet<String>) -> Result<Expr, String> {
        let mut path: Vec<String> = self.params.iter().cloned().collect();
        self.substitute_(e, names, &mut path, used)
    }

    fn substitute_(&self, e: &Expr, names: &[String], path: &mut Vec<String>, used: &mut BTreeSet<String>) -> Result<Expr, String> {
        let local = match e {
            Expr::Sym(n) if names.contains(n) && !path.contains(n) => {
                let Some(v) = self.env.vars.get(n) else { return Ok(e.clone()) };
                let formula = match &v.ast {
                    Some(a) => {
                        let mut lo = Lower::new(self.env, self.units);
                        lo.bind = false;
                        lo.expr(a)?
                    }
                    None => v.expr.clone(),
                };
                used.insert(n.clone());
                path.push(n.clone());
                let r = self.substitute_(&formula, names, path, used);
                path.pop();
                return r;
            }
            Expr::Sym(_) | Expr::C(_) | Expr::Pi | Expr::E => return Ok(e.clone()),
            Expr::Sum(_, var, ..) | Expr::Integral(_, var, ..) | Expr::Derivative(_, var) | Expr::Subs(_, var, _) | Expr::InverseLaplace(_, var, _) => Some(var.clone()),
            _ => None,
        };
        path.extend(local.clone());
        let mut err = None;
        let r = e.map(&mut |c| match self.substitute_(c, names, path, used) {
            Ok(x) => x,
            Err(m) => {
                err.get_or_insert(m);
                c.clone()
            }
        });
        if local.is_some() {
            path.pop();
        }
        err.map_or(Ok(r), Err)
    }

    fn raw(&mut self, a: &Ast) -> Result<Expr, String> {
        Ok(match a {
            Ast::Num(raw, v) => Expr::C(Val { v: *v, d: Dim::ONE, u: None, raw: Some(raw.clone()) }),
            Ast::Qty(n, u) => {
                let x = simp(&self.raw(n)?);
                let Expr::C(nv) = x else { return Err("a number before a unit".into()) };
                let (f, d) = self.unit(u)?;
                let raw = match &**n {
                    Ast::Num(r, _) => Some(r.clone()),
                    _ => None,
                };
                Expr::C(Val { v: nv.v.mul(f), d, u: Some(u.clone()), raw })
            }
            Ast::Unit(u) => {
                let (f, d) = self.unit(u)?;
                Expr::C(Val { v: f, d, u: Some(u.clone()), raw: Some("1".into()) })
            }
            Ast::Name(n) => self.name(n)?,
            Ast::Neg(x) => expr::neg(self.raw(x)?),
            Ast::Add(ts) => {
                let mut v = vec![];
                for (n, t) in ts {
                    let x = self.raw(t)?;
                    v.push(if *n { expr::neg(x) } else { x });
                }
                expr::add(v)
            }
            Ast::Mul(fs) => {
                let mut v: Vec<Expr> = vec![];
                for (op, f) in fs {
                    let x = self.raw(f)?;
                    // × is the cross product (of scalars, their product)
                    match (op, v.pop()) {
                        (MulOp::Times, Some(l)) => v.push(Expr::Op(Op::Cross, vec![l, x])),
                        (_, l) => v.extend(l.into_iter().chain([x])),
                    }
                }
                expr::mul(v)
            }
            Ast::Div(x, y) => expr::div(self.raw(x)?, self.raw(y)?),
            // A^⊤ transposes, and A^T unless T is a name with a value and A isn't a matrix
            Ast::Pow(x, y) if matches!(&**y, Ast::Name(t) if t == "⊤") || matches!(&**y, Ast::Name(t) if t == "T") && (!self.params.contains("T") && !self.env.vars.contains_key("T") || self.is_array(x)?) => {
                Expr::Op(Op::Transpose, vec![self.raw(x)?])
            }
            Ast::Pow(x, y) => expr::pow(self.raw(x)?, self.raw(y)?),
            Ast::Matrix(rs) => Expr::Matrix(rs.iter().map(|r| r.iter().map(|a| self.raw(a)).collect::<Result<_, _>>()).collect::<Result<_, _>>()?),
            Ast::Abs(x) => expr::func(Func::Abs, self.raw(x)?),
            Ast::Cmp(first, rest) => {
                let mut v = vec![self.raw(first)?];
                let mut rs = vec![];
                for (r, x) in rest {
                    rs.push(*r);
                    v.push(self.raw(x)?);
                }
                Expr::Cmp(v, rs)
            }
            Ast::Cast(x, u) => {
                let (f, d) = self.unit(u)?;
                Expr::Cast(Box::new(self.raw(x)?), f, d, u.clone())
            }
            Ast::Call(n, args) => self.call(n, args)?,
        })
    }

    /// Whether what's written is a vector or matrix with the values known now.
    fn is_array(&mut self, a: &Ast) -> Result<bool, String> {
        let e = self.raw(a)?;
        Ok(crate::linalg::is_array(&simp(&self.env.bind_with_units(&e, &self.params, self.units)?)))
    }

    fn unit(&self, u: &UnitExpr) -> Result<(Num, Dim), String> {
        let d = self.units.dim(u).ok_or_else(|| format!("unknown unit '{}'", u.text()))?;
        Ok((self.units.factor(u), d))
    }

    fn name(&self, n: &str) -> Result<Expr, String> {
        if self.params.contains(n) || self.env.vars.contains_key(n) {
            return Ok(Expr::Sym(n.to_string()));
        }
        if self.env.funs.contains_key(n) {
            return Err(format!("{n} is a function – call it, {n}(…)"));
        }
        Ok(match n {
            "π" | "pi" => Expr::Pi,
            "e" => Expr::E,
            "infinity" | "∞" => Expr::C(Val::num(Num::F(f64::INFINITY))),
            _ => Expr::Sym(n.to_string()),
        })
    }

    fn call(&mut self, n: &str, args: &[Ast]) -> Result<Expr, String> {
        if n == "cases" {
            if args.is_empty() { return Err("cases needs a value and condition for each branch, with an optional final default value".into()); }
            let mut branches = vec![];
            for pair in args.chunks(2) {
                branches.push((self.raw(&pair[0])?, if pair.len() == 2 { self.raw(&pair[1])? } else { expr::int(1) }));
            }
            return Ok(Expr::Cases(branches));
        }
        if n == "vec" { return Ok(Expr::Vector(args.iter().map(|a| self.raw(a)).collect::<Result<_, _>>()?)); }
        if let Some(op) = Op::of(n).filter(|_| !self.env.funs.contains_key(n)) {
            if !op.arity().contains(&args.len()) {
                return Err(op.usage());
            }
            let args = args.iter().map(|a| self.raw(a)).collect::<Result<Vec<_>, _>>()?;
            // what can't be done with the values known now is an error now; a name may still become a matrix
            let e = Expr::Op(op, args);
            let bound = simp(&if self.bind { self.env.bind_with_units(&e, &self.params, self.units)? } else { e.clone() });
            if let Some(m) = crate::linalg::problem(&bound) {
                return Err(m);
            }
            return Ok(e);
        }
        if n == "laplace" || n == "invlaplace" {
            if !(1..=3).contains(&args.len()) { return Err(format!("{n} takes an expression and up to two variable names")); }
            let inverse = n == "invlaplace";
            let variable = |i: usize, default: &str| -> Result<String, String> {
                match args.get(i) {
                    None => Ok(default.into()),
                    Some(Ast::Name(n)) => Ok(n.clone()),
                    _ => Err(format!("{n}: transform variables must be names")),
                }
            };
            let input = variable(1, if inverse { "s" } else { "t" })?;
            let output = variable(2, if inverse { "t" } else { "s" })?;
            let added = self.params.insert(input.clone());
            let body = self.expr(&args[0]);
            if added { self.params.remove(&input); }
            return crate::transforms::transform(&body?, &input, &output, inverse);
        }
        if n == "diff" || n == "integrate" || n == "sum" {
            let count = if n == "diff" { 2 } else { 4 };
            if args.len() != count { return Err(format!("{n} takes {count} arguments")); }
            let Ast::Name(var) = &args[1] else { return Err("the calculus variable must be a name".into()); };
            let added = self.params.insert(var.clone());
            let body = self.expr(&args[0]);
            if added { self.params.remove(var); }
            let body = body?;
            if n == "diff" { return Ok(body.diff(var)); }
            let lo = self.raw(&args[2])?;
            let hi = self.raw(&args[3])?;
            return Ok(if n == "sum" { crate::calculus::sum(body, var.clone(), lo, hi) } else { crate::calculus::integral(body, var.clone(), lo, hi) });
        }
        if n == "sqrt" || Func::of(n).is_some() {
            if args.len() != 1 {
                return Err(format!("{n} takes one argument, got {}", args.len()));
            }
            let a = self.raw(&args[0])?;
            return Ok(if n == "sqrt" { expr::sqrt(a) } else { expr::func(Func::of(n).unwrap(), a) });
        }
        let Some(f) = self.env.funs.get(n) else {
            if self.env.vars.contains_key(n) { return Err(format!("{n} is a variable – it can't be called as a function")); }
            return Ok(Expr::Call(n.to_string(), args.iter().map(|a| self.raw(a)).collect::<Result<_, _>>()?));
        };
        if args.len() != f.params.len() {
            return Err(format!("{n} takes {} argument{} ({}), got {}", f.params.len(), if f.params.len() == 1 { "" } else { "s" }, f.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "), args.len()));
        }
        let mut lowered = vec![];
        for (p, a) in f.params.iter().zip(args) {
            if let (Ast::Name(x), Some(t)) = (a, &p.ty) {
                if self.params.contains(x) { self.inferred.entry(x.clone()).or_insert(t.clone()); }
            }
            let mut x = self.raw(a)?;
            if self.bind { x = self.env.bind_with_units(&x, &self.params, self.units)?; }
            lowered.push(x);
        }
        self.apply(n, lowered)
    }

    fn apply(&self, n: &str, args: Vec<Expr>) -> Result<Expr, String> {
        let f = &self.env.funs[n];
        if args.len() != f.params.len() {
            return Err(format!("{n} takes {} arguments, got {}", f.params.len(), args.len()));
        }
        let arrays = args.iter().any(|a| a.contains(&crate::linalg::is_array));
        let mut m = BTreeMap::new();
        for (p, mut x) in f.params.iter().zip(args) {
            if let Some(ParamType::Unit(u)) = &p.ty { x = self.typed(x, u, n, &p.name)?; }
            m.insert(p.name.clone(), x);
        }
        let body = match &f.written {
            Some(w) if arrays => w,
            _ => &f.body,
        };
        Ok(body.subst_all(&m))
    }

    /// An argument for a parameter with a unit: a plain number gets the unit (f(2) is f(2 mA)), a value must
    /// have its dimension.
    fn typed(&self, x: Expr, u: &UnitExpr, f: &str, p: &str) -> Result<Expr, String> {
        let (k, d) = self.unit(u)?;
        let s = simp(&x);
        if let Expr::C(v) = &s {
            if v.d == d {
                return Ok(x);
            }
            if v.d.is_one() {
                return Ok(Expr::C(Val { v: v.v.mul(k), d, u: Some(u.clone()), raw: v.raw.clone() }));
            }
            return Err(format!("{f} takes {p} in {}, got {}", u.text(), self.units.auto(&v.d).text()));
        }
        // a dimension that can be told without values: a parameter of the caller with a unit
        if let Expr::Sym(n) = &s {
            if let Some(ParamType::Unit(w)) = self.inferred.get(n) {
                if self.units.dim(w) != Some(d) {
                    return Err(format!("{f} takes {p} in {}, got {}", u.text(), w.text()));
                }
            }
        }
        if let Ok(Some(dx)) = crate::eval::dim_of(&s, &|n| self.param_dim(n)) {
            if dx != d && !(dx.is_one() && matches!(s, Expr::C(_))) {
                return Err(format!("{f} takes {p} in {}, got {}", u.text(), if dx.is_one() { "1 (no unit)".into() } else { self.units.auto(&dx).text() }));
            }
        }
        Ok(x)
    }

    fn param_dim(&self, n: &str) -> Option<Dim> {
        match self.inferred.get(n) {
            Some(ParamType::Unit(u)) => self.units.dim(u),
            _ => None,
        }
    }
}

/// e^(p/q) for a value's dims with a rational exponent.
pub fn rat_of(x: f64) -> Option<Rat> {
    Rat::approx(x, 60)
}
