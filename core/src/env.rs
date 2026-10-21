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
use crate::num::{Num, Rat};
use crate::parse::{Ast, Param, ParamType, Rel};
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
    /// As written, for showing it.
    pub ast: Ast,
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
    fn note(&mut self, name: &str, kind: Kind, listed: bool) {
        self.order.retain(|e| e.name != name);
        self.order.push(Entry { name: name.to_string(), kind, listed });
    }

    /// Defined names put in, recursively (free symbols of their values too, if those are defined by now);
    /// `keep` are parameters, never replaced.
    pub fn bind(&self, e: &Expr, keep: &BTreeSet<String>) -> Result<Expr, String> {
        self.bind_(e, keep, &mut vec![])
    }
    fn bind_(&self, e: &Expr, keep: &BTreeSet<String>, path: &mut Vec<String>) -> Result<Expr, String> {
        Ok(match e {
            Expr::Sym(n) if !keep.contains(n) => match self.vars.get(n) {
                Some(v) => {
                    if path.contains(n) {
                        return Err(format!("'{n}' is defined by itself ({} → {n})", path.join(" → ")));
                    }
                    path.push(n.clone());
                    let r = self.bind_(&v.expr, keep, path)?;
                    path.pop();
                    r
                }
                None => e.clone(),
            },
            Expr::Sym(_) | Expr::C(_) | Expr::Pi | Expr::E => e.clone(),
            _ => {
                let mut err = None;
                let r = e.map(&mut |c| match self.bind_(c, keep, path) {
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
        if self.bind { self.env.bind(&e, &self.params) } else { Ok(e) }
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
                let mut v = vec![];
                for (_, f) in fs {
                    v.push(self.raw(f)?);
                }
                expr::mul(v)
            }
            Ast::Div(x, y) => expr::div(self.raw(x)?, self.raw(y)?),
            Ast::Pow(x, y) => expr::pow(self.raw(x)?, self.raw(y)?),
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
            Ast::Inline(n) => match self.env.vars.get(n).and_then(|v| v.ast.clone()) {
                Some(a) => self.raw(&a)?,
                None => return Err(format!("no stored formula for '{n}' (@{n} puts in how {n} was computed)")),
            },
            Ast::Call(n, args) => self.call(n, args)?,
        })
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
            _ => Expr::Sym(n.to_string()),
        })
    }

    fn call(&mut self, n: &str, args: &[Ast]) -> Result<Expr, String> {
        if n == "sqrt" || Func::of(n).is_some() {
            if args.len() != 1 {
                return Err(format!("{n} takes one argument, got {}", args.len()));
            }
            let a = self.raw(&args[0])?;
            return Ok(if n == "sqrt" { expr::sqrt(a) } else { expr::func(Func::of(n).unwrap(), a) });
        }
        let Some(f) = self.env.funs.get(n) else { return Err(format!("{n} isn't defined")) };
        if args.len() != f.params.len() {
            return Err(format!("{n} takes {} argument{} ({}), got {}", f.params.len(), if f.params.len() == 1 { "" } else { "s" }, f.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "), args.len()));
        }
        let mut m = BTreeMap::new();
        for (p, a) in f.params.iter().zip(args) {
            // a parameter passed on as it is takes the type of the one it's passed to
            if let (Ast::Name(x), Some(t)) = (a, &p.ty) {
                if self.params.contains(x) {
                    self.inferred.entry(x.clone()).or_insert(t.clone());
                }
            }
            let mut x = self.raw(a)?;
            if self.bind {
                x = self.env.bind(&x, &self.params)?;
            }
            if let Some(ParamType::Unit(u)) = &p.ty {
                x = self.typed(x, u, n, &p.name)?;
            }
            m.insert(p.name.clone(), x);
        }
        Ok(f.body.subst_all(&m))
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
