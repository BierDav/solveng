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
    /// The values of the named parameters, as they were when the function was defined.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub defaults: BTreeMap<String, Expr>,
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
    #[serde(default)]
    pub references: IndexMap<String, crate::units::UnitDef>,
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
        self.references.extend(other.references.clone());
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
            Expr::Antiderivative(body, var, by) => {
                let mut local = keep.clone();
                local.insert(var.clone());
                crate::calculus::antiderivative_at(self.bind_(body, &local, path, units)?, var.clone(), self.bind_(by, keep, path, units)?)
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
            Expr::Sym(_) | Expr::C(_) | Expr::Pi | Expr::E | Expr::I => e.clone(),
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
            Expr::Sym(_) | Expr::C(_) | Expr::Pi | Expr::E | Expr::I => return Ok(e.clone()),
            Expr::Sum(_, var, ..) | Expr::Integral(_, var, ..) | Expr::Derivative(_, var) | Expr::Subs(_, var, _) | Expr::InverseLaplace(_, var, _) | Expr::Antiderivative(_, var, _) => Some(var.clone()),
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
            Ast::Bound(c) => c.expr.clone(),
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
            // as branches: a and b is b where a holds, else 0; a or b is 1 where a holds, else b
            Ast::And(all) => {
                let mut e = self.raw(all.last().unwrap())?;
                for a in all[..all.len() - 1].iter().rev() {
                    e = Expr::Cases(vec![(e, self.raw(a)?), (expr::int(0), expr::int(1))]);
                }
                e
            }
            Ast::Or(any) => {
                let mut e = self.raw(any.last().unwrap())?;
                for a in any[..any.len() - 1].iter().rev() {
                    e = Expr::Cases(vec![(expr::int(1), self.raw(a)?), (e, expr::int(1))]);
                }
                e
            }
            Ast::Call(n, args) => self.call(n, args)?,
            Ast::Builtin(n, args) => self.builtin(n, args)?,
            Ast::Arg(n, _) => return Err(format!("'{n}: …' is a named argument – it goes in the parentheses of a call, f(x, {n}: …)")),
            // A[i, :], A[:, j]: a row or a column; `:` alone or for every index is all of it
            Ast::Index(x, idx) if idx.contains(&Ast::All) => match idx.as_slice() {
                [i, Ast::All] if *i != Ast::All => { let args = vec![self.raw(x)?, self.raw(i)?]; self.op(Op::Row, args)? }
                [Ast::All, j] if *j != Ast::All => { let args = vec![self.raw(x)?, self.raw(j)?]; self.op(Op::Col, args)? }
                [Ast::All] | [Ast::All, Ast::All] => self.raw(x)?,
                _ => return Err("':' as an index selects a whole row or column: A[i, :], A[:, j]".into()),
            },
            Ast::All => return Err("':' as an index selects a whole row or column: A[i, :], A[:, j]".into()),
            Ast::Index(x, idx) => {
                let mut args = vec![self.raw(x)?];
                for i in idx {
                    args.push(self.raw(i)?);
                }
                self.op(Op::At, args)?
            }
            Ast::Range(a, b) => {
                let (a, b) = (self.raw(a)?, self.raw(b)?);
                range(self.known(&a, "the start of a range")?, self.known(&b, "the end of a range")?, None, true)?
            }
        })
    }

    /// The value of a bound of a range or a step: known now, since it decides the number of elements.
    fn known(&self, e: &Expr, what: &str) -> Result<Val, String> {
        let b = simp(&self.env.bind_with_units(e, &self.params, self.units)?);
        crate::eval::value(&b).map_err(|e| match e {
            crate::eval::NoValue::Unknown(ns) => format!("{what} needs a value now – {} {} not defined", ns.join(", "), if ns.len() == 1 { "is" } else { "are" }),
            crate::eval::NoValue::Error(m) => m,
        })
    }

    /// A matrix operation: what can't be done with the values known now is an error now; a name may still
    /// become a matrix.
    fn op(&self, op: Op, args: Vec<Expr>) -> Result<Expr, String> {
        let e = Expr::Op(op, args);
        let bound = simp(&if self.bind { self.env.bind_with_units(&e, &self.params, self.units)? } else { e.clone() });
        if let Some(m) = crate::linalg::problem(&bound) {
            return Err(m);
        }
        Ok(e)
    }

    /// Whether what's written is a vector or matrix with the values known now.
    fn is_array(&mut self, a: &Ast) -> Result<bool, String> {
        let e = self.raw(a)?;
        Ok(crate::linalg::is_array(&simp(&self.env.bind_with_units(&e, &self.params, self.units)?)))
    }

    fn unit(&self, u: &UnitExpr) -> Result<(Num, Dim), String> {
        let d = self.units.dim(u).ok_or_else(|| format!("unknown unit '{}'", self.units.text(u)))?;
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
            "infinity" | "∞" => Expr::C(Val::num(Num::F(f64::INFINITY))),
            _ => Expr::Sym(n.to_string()),
        })
    }

    /// A built-in operation, its arguments bound as in Typst.
    fn builtin(&mut self, n: &str, args: &[Ast]) -> Result<Expr, String> {
        let b = crate::builtins::builtin(n).ok_or_else(|| format!("'{n}' is no built-in operation"))?;
        let written: Vec<crate::builtins::Arg<&Ast>> = args.iter().map(|a| match a {
            Ast::Arg(k, v) => crate::builtins::Arg::Named(k.clone(), &**v),
            a => crate::builtins::Arg::Pos(a),
        }).collect();
        let bound = crate::builtins::bind(b, written)?;
        let args: Vec<&Ast> = bound.positional.clone();
        let n = b.name;
        if n == "cases" {
            if args.is_empty() { return Err("cases needs a value and condition for each branch, with an optional final default value".into()); }
            let mut branches = vec![];
            for pair in args.chunks(2) {
                branches.push((self.raw(pair[0])?, if pair.len() == 2 { self.raw(pair[1])? } else { expr::int(1) }));
            }
            return Ok(Expr::Cases(branches));
        }
        if n == "vec" { return Ok(Expr::Vector(args.iter().map(|a| self.raw(a)).collect::<Result<_, _>>()?)); }
        if let Some(op) = Op::of(n) {
            if !op.arity().contains(&args.len()) {
                return Err(op.usage());
            }
            let args = args.iter().map(|a| self.raw(a)).collect::<Result<Vec<_>, _>>()?;
            return self.op(op, args);
        }
        if n == "laplace" || n == "invlaplace" {
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
            let body = self.expr(args[0]);
            if added { self.params.remove(&input); }
            return crate::transforms::transform(&body?, &input, &output, inverse);
        }
        if n == "diff" || n == "integrate" || n == "sum" {
            let Ast::Name(var) = args[1] else { return Err("the calculus variable must be a name".into()); };
            let body = self.local(var, args[0])?;
            if n == "diff" { return Ok(body.diff(var)); }
            if n == "integrate" && args.len() != 4 {
                if args.len() == 3 { return Err("integrate: an indefinite integral has no bounds, a definite one both: integrate(f, x), integrate(f, x, a, b)".into()); }
                return Ok(crate::calculus::antiderivative(&body, var));
            }
            let lo = self.raw(args[2])?;
            let hi = self.raw(args[3])?;
            return Ok(if n == "sum" { crate::calculus::sum(body, var.clone(), lo, hi) } else { crate::calculus::integral(body, var.clone(), lo, hi) });
        }
        if n == "convolve" {
            let step = bound.named.get("step").map(|s| self.raw(s)).transpose()?;
            if args.len() == 2 {
                // sequences: the discrete convolution, times the step if one is given (a Riemann sum)
                let (x, h) = (self.raw(args[0])?, self.raw(args[1])?);
                let e = self.op(Op::Convolve, vec![x, h])?;
                return Ok(match step { Some(s) => expr::mul(vec![e, s]), None => e });
            }
            if step.is_some() {
                return Err("convolve: step is for sequences, convolve(x, h, step: …); a convolution of functions, convolve(f(τ), g(τ), τ, t), is exact".into());
            }
            let Ast::Name(var) = args[2] else { return Err("convolve: the variable must be a name, convolve(f(τ), g(τ), τ, t)".into()); };
            let f = self.local(var, args[0])?;
            let g = self.local(var, args[1])?;
            let at = match args.get(3) {
                Some(a) => self.raw(a)?,
                None => Expr::Sym(var.clone()),
            };
            return Ok(crate::convolution::continuous(&f, &g, var, &at));
        }
        if n == "range" {
            let (start, end) = match args.len() {
                1 => (None, args[0]),
                _ => (Some(args[0]), args[1]),
            };
            let end = self.raw(end)?;
            let end = self.known(&end, "range: the end")?;
            let start = match start {
                Some(a) => { let a = self.raw(a)?; self.known(&a, "range: the start")? }
                None => Val::new(Num::ZERO, end.d.clone()),
            };
            let step = match bound.named.get("step") {
                Some(s) => { let s = self.raw(s)?; Some(self.known(&s, "range: the step")?) }
                None => None,
            };
            return range(start, end, step, false);
        }
        if n == "number" {
            let value = self.raw(args[0])?;
            let u = self.raw(args[1])?;
            let u = self.known(&u, "number: the unit")?;
            if u.v.is_zero() || u.v.is_neg() {
                return Err("number: the unit must be positive, like number(U, mV)".into());
            }
            let e = Expr::Number(Box::new(value), u.v, u.d.clone(), u.u.clone().unwrap_or_default());
            // what can be told already: a value of another dimension
            crate::eval::dim_of(&e, &|n| self.param_dim(n))?;
            return Ok(e);
        }
        if n == "sqrt" || Func::of(n).is_some() {
            let a = self.raw(args[0])?;
            return Ok(if n == "sqrt" { expr::sqrt(a) } else { expr::func(Func::of(n).unwrap(), a) });
        }
        Err(format!("'{n}' is no built-in operation"))
    }

    /// An expression in a local variable (of an integral, a sum, a convolution): the variable stays a name.
    fn local(&mut self, var: &str, a: &Ast) -> Result<Expr, String> {
        let added = self.params.insert(var.to_string());
        let body = self.expr(a);
        if added { self.params.remove(var); }
        body
    }

    fn call(&mut self, n: &str, args: &[Ast]) -> Result<Expr, String> {
        let Some(f) = self.env.funs.get(n) else {
            if self.env.vars.contains_key(n) { return Err(format!("{n} is a variable – it can't be called as a function")); }
            if let Some(Ast::Arg(k, _)) = args.iter().find(|a| matches!(a, Ast::Arg(..))) {
                return Err(format!("{n} isn't defined yet, so it has no parameter {k} – define {n} before calling it with named arguments"));
            }
            return Ok(Expr::Call(n.to_string(), args.iter().map(|a| self.raw(a)).collect::<Result<_, _>>()?));
        };
        // as in Typst: positional arguments for the parameters without a default, in order; named ones for those with
        let positional: Vec<&Param> = f.params.iter().filter(|p| p.default.is_none()).collect();
        let mut given: Vec<Option<&Ast>> = vec![None; f.params.len()];
        let mut k = 0;
        for a in args {
            match a {
                Ast::Arg(name, v) => {
                    let Some(i) = f.params.iter().position(|p| &p.name == name) else {
                        return Err(format!("{n}: unexpected argument: {name}{}", signature(n, f)));
                    };
                    if f.params[i].default.is_none() {
                        return Err(format!("{n}: unexpected argument: {name} – {name} is positional, pass it without its name"));
                    }
                    if given[i].replace(v).is_some() {
                        return Err(format!("{n}: duplicate argument: {name}"));
                    }
                }
                a => {
                    let Some(p) = positional.get(k) else {
                        return Err(format!("{n} takes {} argument{} ({}), got {}", positional.len(), if positional.len() == 1 { "" } else { "s" }, positional.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "), args.iter().filter(|a| !matches!(a, Ast::Arg(..))).count()));
                    };
                    let i = f.params.iter().position(|q| q.name == p.name).unwrap();
                    given[i] = Some(a);
                    k += 1;
                }
            }
        }
        if let Some(p) = positional.get(k) {
            return Err(if f.params.iter().all(|p| p.default.is_none()) {
                format!("{n} takes {} argument{} ({}), got {}", f.params.len(), if f.params.len() == 1 { "" } else { "s" }, f.params.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join(", "), k)
            } else {
                format!("{n}: missing argument: {}{}", p.name, signature(n, f))
            });
        }
        let params = f.params.clone();
        let defaults = f.defaults.clone();
        let mut lowered = vec![];
        for (p, a) in params.iter().zip(given) {
            let Some(a) = a else {
                lowered.push(defaults.get(&p.name).cloned().ok_or_else(|| format!("{n}: missing argument: {}", p.name))?);
                continue;
            };
            if let (Ast::Name(x), Some(t)) = (a, &p.ty) {
                if self.params.contains(x) { self.inferred.entry(x.clone()).or_insert(t.clone()); }
            }
            let mut x = self.raw(a)?;
            if self.bind { x = self.env.bind_with_units(&x, &self.params, self.units)?; }
            lowered.push(x);
        }
        self.apply(n, lowered)
    }

    fn apply(&self, n: &str, mut args: Vec<Expr>) -> Result<Expr, String> {
        let f = &self.env.funs[n];
        // a call written before the function was defined: its named parameters take their defaults
        while args.len() < f.params.len() {
            match f.defaults.get(&f.params[args.len()].name) {
                Some(d) => args.push(d.clone()),
                None => break,
            }
        }
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
        // a number for a parameter a closed form is guarded in is put in as a value (see simp of Subs)
        let names: Vec<String> = f.params.iter()
            .filter(|p| m.get(&p.name).is_some_and(|x| x.syms().is_empty()) && crate::calculus::guarded_in(body, &p.name))
            .map(|p| p.name.clone())
            .collect();
        let kept: Vec<(String, Expr)> = names.into_iter().filter_map(|n| Some((n.clone(), m.remove(&n)?))).collect();
        Ok(kept.into_iter().fold(body.subst_all(&m), |e, (name, x)| simp(&Expr::Subs(Box::new(e), name, Box::new(x)))))
    }

    /// Typed arguments must carry the declared dimension, including literal numbers.
    fn typed(&self, x: Expr, u: &UnitExpr, f: &str, p: &str) -> Result<Expr, String> {
        let (_, d) = self.unit(u)?;
        let s = simp(&x);
        if let Expr::C(v) = &s {
            if v.d == d {
                return Ok(x);
            }
            return Err(format!("{f} takes {p} in {}, got {}", self.units.text(u), if v.d.is_one() { "1 (no unit)".into() } else { self.units.text(&self.units.auto(&v.d)) }));
        }
        // a dimension that can be told without values: a parameter of the caller with a unit
        if let Expr::Sym(n) = &s {
            if let Some(ParamType::Unit(w)) = self.inferred.get(n) {
                if self.units.dim(w) != Some(d.clone()) {
                    return Err(format!("{f} takes {p} in {}, got {}", self.units.text(u), self.units.text(w)));
                }
            }
        }
        if let Ok(Some(dx)) = crate::eval::dim_of(&s, &|n| self.param_dim(n)) {
            if dx != d {
                return Err(format!("{f} takes {p} in {}, got {}", self.units.text(u), if dx.is_one() { "1 (no unit)".into() } else { self.units.text(&self.units.auto(&dx)) }));
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

/// How a worksheet function is called, for messages: " – f(x, n: …)".
fn signature(n: &str, f: &Fun) -> String {
    let ps: Vec<String> = f.params.iter().map(|p| if p.default.is_some() { format!("{}: …", p.name) } else { p.name.clone() }).collect();
    format!(" – {n}({})", ps.join(", "))
}

/// The values from `start` to `end`: with `inclusive`, a range variable a .. b (steps of 1, counting down when
/// b < a); else Typst's range(start, end, step: …), without the end.
pub fn range(start: Val, end: Val, step: Option<Val>, inclusive: bool) -> Result<Expr, String> {
    if start.d != end.d {
        return Err("a range's start and end must have the same unit".into());
    }
    let step = match step {
        Some(s) => s,
        None if !start.d.is_one() => {
            return Err(if inclusive { "a range of values with a unit takes a step: range(start, end, step: …)".into() } else { "range: values with a unit take a step of that unit, step: …".into() });
        }
        None => Val::num(if inclusive && end.v.cmp(start.v) == std::cmp::Ordering::Less { Num::int(-1) } else { Num::ONE }),
    };
    if step.d != start.d {
        return Err("range: the step must have the unit of the start and end".into());
    }
    if step.v.is_zero() {
        return Err("range: the step can't be 0".into());
    }
    let span = end.v.sub(start.v).f() / step.v.f();
    if !span.is_finite() {
        return Err("a range needs finite bounds".into());
    }
    // the last index: within rounding of the end, which counts when the range includes it
    let last = if inclusive { (span + 1e-9).floor() } else { (span - 1e-9).ceil() - 1.0 };
    if last >= 100_000.0 {
        return Err(format!("a range of {} values is too long – at most 100000", last + 1.0));
    }
    let count = if last < 0.0 { 0 } else { last as i128 + 1 };
    Ok(Expr::Vector((0..count).map(|k| Expr::C(Val::new(start.v.add(step.v.mul(Num::int(k))), start.d.clone()))).collect()))
}

/// e^(p/q) for a value's dims with a rational exponent.
pub fn rat_of(x: f64) -> Option<Rat> {
    Rat::approx(x, 60)
}
