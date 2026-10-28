//! A line of a worksheet, calculated like Mathcad does:
//!
//! - `:=` defines, symbolically: what's defined by then is put in, what isn't stays a name until it is.
//! - `=` inside an expression compares: `1 = 2` is 0, values of different units are never equal.
//! - a trailing `=` shows the value, after it a unit and keywords: `t = 1 h, digits(3)`, `t = min`.
//! - a trailing `->` shows the symbolic result (`-> simplify`), or solves (`-> solve(x)`).
//! - anything else with `=` states something: checked when everything in it is known, prose when not (math).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::dim::Dim;
use crate::env::{Env, Fun, Kind, Lower, Var};
use crate::eval::{NoValue, complex_value, dim_of, value};
use crate::complex::CVal;
use crate::expr::{self, Expr, Val, simp};
use crate::format::{self, Cfg, name, pick, shown};
use crate::linalg::{self, Array};
use crate::num::{Num, close};
use crate::parse::{self, Ast, Clause, Ctx, Head, Line, Param, ParamType, Rel, Stmt, Trail};
use crate::poly::{Form, shaped, simplify};
use crate::render::{self, R};
use crate::solve::{self, Fail};
use crate::units::{UnitExpr, Units};

/// What a line gives the Typst side.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Out {
    /// "=" or "->" when a result follows the equation as written.
    pub mark: Option<String>,
    /// The result after the mark.
    pub result: Option<String>,
    pub errors: Vec<String>,
    /// Math that isn't a calculation: left alone.
    pub prose: bool,
    /// What the line defined.
    pub defined: Vec<String>,
    /// The solutions of a solve, each the values of its unknowns in SI (NaN where one is symbolic).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solutions: Vec<Vec<f64>>,
}

/// What a solved unknown came out as.
#[derive(Clone, Debug)]
enum Sol {
    Value(CVal),
    Symbolic(Expr),
}

pub struct Sheet<'a> {
    pub env: &'a mut Env,
    pub units: Units,
    pub cfg: Cfg,
    pub listed: bool,
    out: Out,
}

impl<'a> Sheet<'a> {
    pub fn new(env: &'a mut Env, cfg: &Cfg, listed: bool) -> Result<Sheet<'a>, String> {
        let units = Units::lexical(cfg, env, "")?;
        Ok(Sheet { env, units, cfg: cfg.clone(), listed, out: Out::default() })
    }

    /// Calculates a line.
    pub fn run(mut self, src: &str) -> Out {
        match crate::bindings::references(src) {
            Ok(references) => { self.units.map.extend(references.clone()); self.env.references.extend(references); }
            Err(e) => { self.out.errors.push(e); return self.out; }
        }
        // solveng's own markup (definitions, keywords) is meant for it: problems aren't prose
        let calc = src.contains(":=") || src.contains('≔') || src.contains('⦅');
        let line = {
            let env = &*self.env;
            let is_fn = |n: &str| env.funs.contains_key(n);
            let is_var = |n: &str| env.vars.contains_key(n);
            let ctx = Ctx { units: &self.units, is_fn: &is_fn, is_var: &is_var };
            parse::parse_line(src, &ctx).map_err(|e| (e, calc || parse::meant(src, &ctx)))
        };
        let line = match line {
            Ok(l) => l,
            Err((e, calc)) => {
                if !calc {
                    self.out.prose = true;
                } else {
                    self.out.errors.push(e);
                }
                return self.out;
            }
        };
        if let Err(e) = self.line(&line) {
            self.out.errors.push(e);
        }
        self.out
    }

    fn line(&mut self, l: &Line) -> Result<(), String> {
        let (trail, clause) = match &l.trail {
            Some((t, c)) => (Some(*t), c.clone()),
            None => (None, Clause::default()),
        };
        let cfg = self.clause_cfg(&clause);
        if !clause.real.is_empty() && (trail != Some(Trail::Sym) || clause.solve.is_some() || l.stmts.iter().any(|s| !matches!(s, Stmt::Expr(_)))) {
            return Err("assume applies to a symbolic expression or equation after ->, without solve or a definition".into());
        }
        if clause.steps && clause.solve.is_some() {
            return Err("steps shows how -> simplifies, not how solve solves: $F(a p) = a F(p) -> steps$ shows where an equation holds".into());
        }
        if let Some(xs) = &clause.solve {
            if !l.stmts.iter().any(|s| matches!(s, Stmt::Def { .. })) {
                return self.solve(&l.stmts, xs, &clause, &cfg);
            }
            if l.stmts.len() != 1 {
                return Err("put a solved definition on its own line".into());
            }
        }
        let n = l.stmts.len();
        for (i, s) in l.stmts.iter().enumerate() {
            let t = if i == n - 1 { trail } else { None };
            if let Err(e) = self.stmt(s, t, &clause, &cfg) {
                self.out.errors.push(e);
            }
        }
        Ok(())
    }

    fn clause_cfg(&self, c: &Clause) -> Cfg {
        let mut cfg = self.cfg.clone();
        if let Some(d) = c.digits {
            cfg.form = "decimal".into();
            cfg.digits = d;
        }
        if c.frac {
            cfg.form = "fraction".into();
        }
        if c.expand {
            cfg.symbolic = Form::Expanded;
        }
        if c.factor {
            cfg.symbolic = Form::Factored;
        }
        if c.sci {
            cfg.scientific = Some((0, 1));
            cfg.prefixes = crate::units::Prefixes::None;
        }
        if c.eng {
            cfg.engineering = true;
            cfg.scientific = Some((0, 1));
            cfg.prefixes = crate::units::Prefixes::None;
        }
        cfg
    }

    fn r<'b>(&'b self, cfg: &'b Cfg, exact: bool) -> R<'b> {
        R { units: &self.units, cfg, exact }
    }

    fn lower(&self, a: &Ast, keep: &[String], bind: bool) -> Result<Expr, String> {
        let mut lo = Lower::new(self.env, &self.units);
        lo.params = keep.iter().cloned().collect();
        lo.bind = bind;
        lo.expr(a)
    }

    /// One statement.
    fn stmt(&mut self, s: &Stmt, trail: Option<Trail>, clause: &Clause, cfg: &Cfg) -> Result<(), String> {
        if (clause.expand || clause.factor) && trail != Some(Trail::Sym) {
            return Err(format!("{} shows a symbolic result: it goes after ->", if clause.expand { "expand" } else { "factor" }));
        }
        if clause.steps && (trail != Some(Trail::Sym) || clause.solve.is_some() || !matches!(s, Stmt::Expr(_))) {
            return Err("steps shows how -> simplifies an expression or an equation: $F(a p) = a F(p) -> steps$".into());
        }
        if !clause.substitute.is_empty() && (trail != Some(Trail::Sym) || clause.solve.is_some()) {
            return Err("substitute shows a formula: it goes after ->, without solve".into());
        }
        match s {
            Stmt::Def { head: Head::Fn { name, params, rest }, rhs, src } => {
                if trail == Some(Trail::Eval) {
                    return Err(format!("a function has no value of its own – evaluate a call, {name}(2) ="));
                }
                self.define_fn(name, params, *rest, rhs, src, trail == Some(Trail::Sym), clause, cfg)
            }
            Stmt::Def { head: Head::Var(n), rhs, .. } => self.define_var(n, rhs, trail, clause, cfg),
            Stmt::Def { head: Head::Index { name, indices }, rhs, .. } => self.define_elements(name, indices, rhs, trail, clause, cfg),
            Stmt::Expr(a) => match trail {
                Some(Trail::Eval) => self.eval_expr(a, clause, cfg),
                Some(Trail::Sym) => self.sym_expr(a, clause, cfg),
                None => self.statement(a),
            },
        }
    }

    // ---------- definitions ----------

    /// x[i] := …, A[i, j] := …: elements, as Mathcad defines them. An index that is a range variable (a name
    /// whose value is a vector, i := 0 .. 10) defines an element for each of its values, the body calculated with
    /// the name standing for that value; other indices are whole numbers. Elements not defined yet are 0; the
    /// vector or matrix grows to hold the new ones and keeps its other elements.
    fn define_elements(&mut self, n: &str, indices: &[Ast], rhs: &Ast, trail: Option<Trail>, clause: &Clause, cfg: &Cfg) -> Result<(), String> {
        if clause.solve.is_some() || !clause.substitute.is_empty() {
            return Err(format!("{n}[…] := … defines elements – solve and substitute are for whole definitions"));
        }
        // each index: the values it takes, and the range variable standing for them
        let mut axes: Vec<(Option<String>, Vec<usize>)> = vec![];
        for a in indices {
            let e = simp(&self.lower(a, &[], true)?);
            let whole = |e: &Expr| -> Result<usize, String> {
                let v = value(e).map_err(|e| self.no_value(e, &ast_text(a, &self.units)))?;
                v.v.as_int().filter(|i| *i >= 0 && v.d.is_one() && *i < 1_000_000).map(|i| i as usize)
                    .ok_or_else(|| format!("{n}[…]: an index is a whole number from 0, or a range variable of them (i := 0 .. 10), got '{}'", ast_text(a, &self.units)))
            };
            let var = match a { Ast::Name(v) => Some(v.clone()), _ => None };
            match Array::of(&e) {
                Some(arr) if arr.vector() => {
                    let Some(v) = var else { return Err(format!("{n}[…]: a vector of indices is a range variable, written as its name: {n}[i] := …")) };
                    axes.push((Some(v), arr.a.iter().map(whole).collect::<Result<_, _>>()?));
                }
                Some(arr) => return Err(format!("{n}[…]: an index can't be {}", arr.describe())),
                None => axes.push((None, vec![whole(&e)?])),
            }
        }
        let vars: Vec<String> = axes.iter().filter_map(|(v, _)| v.clone()).collect();
        if let Some(v) = vars.iter().find(|v| vars.iter().filter(|w| w == v).count() > 1) {
            return Err(format!("{n}[{v}, {v}] := …: each index needs a range variable of its own"));
        }
        let body = simp(&self.lower(rhs, &vars, true)?);
        // what's there already
        let old = self.env.vars.get(n).map(|v| simp(&self.env.bind_with_units(&v.expr, &BTreeSet::new(), &self.units).unwrap_or(v.expr.clone())));
        let (mut rows, mut cols, mut cells): (usize, usize, BTreeMap<(usize, usize), Expr>) = (0, 0, BTreeMap::new());
        if let Some(a) = old.as_ref().and_then(Array::of) {
            if (indices.len() == 1) == a.vector() {
                (rows, cols) = (a.rows, a.cols);
                for i in 0..a.rows {
                    for j in 0..a.cols {
                        cells.insert((i, j), a.at(i, j).clone());
                    }
                }
            }
        }
        let (is, js) = (axes[0].clone(), axes.get(1).cloned().unwrap_or((None, vec![0])));
        if is.1.len() * js.1.len() > 1_000_000 {
            return Err(format!("{n}[…] := …: too many elements at once"));
        }
        for &i in &is.1 {
            for &j in &js.1 {
                let mut m = BTreeMap::new();
                if let Some(v) = &is.0 { m.insert(v.clone(), expr::int(i as i128)); }
                if let Some(v) = &js.0 { m.insert(v.clone(), expr::int(j as i128)); }
                let e = simp(&linalg::pick(&body.subst_all(&m)));
                if let Some(msg) = linalg::problem(&e) {
                    return Err(msg);
                }
                if linalg::is_array(&e) {
                    return Err(format!("{n}[…] := …: an element is a single value, not {}", Array::of(&e).unwrap().describe()));
                }
                cells.insert((i, j), e);
                rows = rows.max(i + 1);
                cols = cols.max(j + 1);
            }
        }
        let cols = if indices.len() == 1 { 1 } else { cols };
        let at = |i: usize, j: usize| cells.get(&(i, j)).cloned().unwrap_or_else(|| expr::int(0));
        let stored = if indices.len() == 1 {
            Expr::Vector((0..rows).map(|i| at(i, 0)).collect())
        } else {
            linalg::matrix((0..rows).map(|i| (0..cols).map(|j| at(i, j)).collect()).collect())
        };
        let unit = clause.unit.clone();
        self.env.define_var(n, Var { expr: stored.clone(), ast: None, unit: unit.clone(), rel: None }, Kind::Computed, self.listed);
        self.out.defined.push(n.to_string());
        match trail {
            Some(Trail::Eval) => {
                let (m, _) = self.eval_markup(&stored, unit.as_ref(), cfg).map_err(|e| self.no_value(e, n))?;
                self.show("=", m);
            }
            Some(Trail::Sym) => {
                let m = self.sym_markup(&stored, unit.as_ref(), cfg)?;
                self.show("->", m);
            }
            None => {}
        }
        Ok(())
    }

    fn define_var(&mut self, n: &str, rhs: &Ast, trail: Option<Trail>, clause: &Clause, cfg: &Cfg) -> Result<(), String> {
        let e = if let Some(xs) = &clause.solve { self.solve_body(rhs, &[], xs)? } else { self.lower(rhs, &[], true)? };
        if e.has(n) || e.contains(&|e| matches!(e, Expr::Call(name, _) if name == n)) {
            return Err(format!("'{n}' is used in its own definition but isn't defined yet – define it first, or solve for it: … -> solve({n})"));
        }
        // what substitute shows, from the definitions before this one
        let shown = if clause.substitute.is_empty() { None } else { Some(self.substituted(rhs, &[], &clause.substitute)?) };
        let given = literal(rhs);
        let s = simp(&e);
        // a symbolic result asked for: what's stored is the simplified form
        let stored = if trail == Some(Trail::Sym) { self.shaped(&s, cfg)? } else { s.clone() };
        let unit = clause.unit.clone();
        if let (Some(u), Ok(v)) = (&unit, value(&stored)) {
            pick(&v, Some(u), None, &self.units, cfg)?;
        }
        let var = Var { expr: stored.clone(), ast: if clause.solve.is_some() { None } else { Some(rhs.clone()) }, unit: unit.clone(), rel: None };
        self.env.define_var(n, var, if given { Kind::Given } else { Kind::Computed }, self.listed);
        self.out.defined.push(n.to_string());
        match trail {
            Some(Trail::Eval) => {
                let (m, _) = self.eval_markup(&stored, unit.as_ref(), cfg).map_err(|e| self.no_value(e, n))?;
                self.show("=", m);
            }
            Some(Trail::Sym) => {
                let m = self.sym_markup(shown.as_ref().unwrap_or(&stored), unit.as_ref(), cfg)?;
                self.show("->", m);
            }
            None => {}
        }
        Ok(())
    }

    fn solve_body(&self, rhs: &Ast, params: &[Param], xs: &[String]) -> Result<Expr, String> {
        let mut keep: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
        if xs.iter().any(|x| keep.contains(x)) {
            return Err("a solve variable is local to the equation; leave it out of the function parameters".into());
        }
        let mut unique = BTreeSet::new();
        if xs.iter().any(|x| !unique.insert(x)) { return Err("each solve variable must be different".into()); }
        keep.extend_from_slice(xs);
        let mut lo = Lower::new(self.env, &self.units);
        lo.params = keep.iter().cloned().collect();
        for p in params {
            if let Some(t) = &p.ty { lo.inferred.insert(p.name.clone(), t.clone()); }
        }
        let body = lo.expr(rhs)?;
        let bodies = match body { Expr::Vector(v) => v, e => vec![e] };
        let mut eqs = vec![];
        for body in bodies {
            let Expr::Cmp(v, rs) = body else {
                return Err("solve(…) solves an equation: use a relation such as a^2 + b^2 = c^2, or call a function containing one".into());
            };
            if rs != [Rel::Eq] || v.len() != 2 || v.iter().any(|e| e.contains(&|e| matches!(e, Expr::Cmp(..)))) {
                return Err("solve a definition using one equality per equation; a relation function already contains =, so omit the extra = b".into());
            }
            eqs.push((v[0].clone(), v[1].clone()));
        }
        if eqs.len() != xs.len() {
            return Err(format!("{} equations but {} solve variables – one equation per unknown", eqs.len(), xs.len()));
        }
        for x in xs {
            if !eqs.iter().any(|(l, r)| l.has(x) || r.has(x)) { return Err(format!("'{x}' does not appear in the equation")); }
        }
        let known = |n: &str| !keep.iter().any(|k| k == n) && self.env.vars.get(n).is_some_and(|v| value(&v.expr).is_ok());
        let solutions = if xs.len() == 1 {
            solve::solve1(&expr::sub(eqs[0].0.clone(), eqs[0].1.clone()), &xs[0], &known)
                .map_err(|e| match e {
                    Fail::Cancels => format!("'{}' cancels out of the equation", xs[0]),
                    Fail::NotSymbolic => "this equation cannot yet be solved symbolically as a function of its parameters".into(),
                })?
                .into_iter().map(|e| vec![e]).collect::<Vec<_>>()
        } else {
            solve::solve_system(&eqs, xs, &known).ok_or("this system cannot yet be solved symbolically")?
                .into_iter().map(|s| xs.iter().map(|x| s.get(x).cloned().ok_or_else(|| format!("no solution for '{x}'"))).collect::<Result<Vec<_>, _>>()).collect::<Result<Vec<_>, _>>()?
        };
        if solutions.is_empty() { return Err("no real solution".into()); }
        let mut branches = vec![];
        for solution in solutions {
            let mut v = solution.iter().map(simplify).collect::<Vec<_>>();
            let branch = if xs.len() == 1 { v.remove(0) } else { Expr::Vector(v) };
            if !branches.contains(&branch) { branches.push(branch); }
        }
        Ok(if branches.len() == 1 { branches.remove(0) } else { Expr::Vector(branches) })
    }

    #[allow(clippy::too_many_arguments)]
    fn define_fn(&mut self, n: &str, params: &[Param], rest: bool, rhs: &Ast, src: &str, sym: bool, clause: &Clause, cfg: &Cfg) -> Result<(), String> {
        if parse::is_builtin(n) {
            return Err(format!("{n} is Typst's math syntax and can't be defined"));
        }
        // named parameters: their defaults as they are now, as Typst evaluates them where the function is defined
        let mut defaults = BTreeMap::new();
        for p in params {
            if let Some(d) = &p.default {
                let e = self.lower(d, &[], true).map_err(|e| format!("the default value of {}: {e}", p.name))?;
                defaults.insert(p.name.clone(), simp(&e));
            }
        }
        let mut lo = Lower::new(self.env, &self.units);
        lo.params = params.iter().map(|p| p.name.clone()).collect();
        for p in params {
            if let Some(t) = &p.ty {
                lo.inferred.insert(p.name.clone(), t.clone());
            }
        }
        let body = if let Some(xs) = &clause.solve {
            lo.params.extend(xs.iter().cloned());
            let _ = lo.expr(rhs)?;
            self.solve_body(rhs, params, xs)?
        } else {
            lo.expr(rhs)?
        };
        let inferred = lo.inferred.clone();
        let mut ps: Vec<Param> = params.to_vec();
        // .. : the body's other free names are parameters too, in order
        if rest {
            for s in body.syms() {
                if !ps.iter().any(|p| p.name == s) && s != n {
                    ps.push(Param { name: s, ty: None, default: None });
                }
            }
        }
        for p in ps.iter_mut() {
            if p.ty.is_none() {
                p.ty = inferred.get(&p.name).cloned();
            }
        }
        // parameters are values: u(t) with a parameter u would be another function u, unrelated to it
        if let Some(p) = ps.iter().find(|p| body.contains(&|e| matches!(e, Expr::Call(name, _) if *name == p.name))) {
            return Err(format!("{n}: the parameter {0} is called like a function, {0}(…) – parameters are values; write {0} for its value at the point: {n}({0}) := {0}^2", p.name));
        }
        // a parameter of the same name is the parameter: y(y) := X(y)
        if body.contains(&|e| matches!(e, Expr::Call(name, _) if name == n)) || (body.has(n) && !ps.iter().any(|p| p.name == n)) {
            return Err(format!("{n} is used in its own definition – functions can't call themselves"));
        }
        let shown = if clause.substitute.is_empty() { None } else { Some(self.substituted(rhs, &ps.iter().map(|p| p.name.clone()).collect::<Vec<_>>(), &clause.substitute)?) };
        let written = (clause.solve.is_none() && body.contains(&|e| matches!(e, Expr::Mul(_)))).then(|| body.clone());
        let body = if sym { self.shaped(&body, cfg)? } else { simp(&body) };
        let pd: BTreeMap<String, Dim> = ps
            .iter()
            .filter_map(|p| match &p.ty {
                Some(ParamType::Unit(u)) => self.units.dim(u).map(|d| (p.name.clone(), d)),
                // a named parameter has the unit of its default
                _ => defaults.get(&p.name).and_then(|d| value(d).ok()).map(|v| (p.name.clone(), v.d)),
            })
            .collect();
        let ret = if ps.iter().all(|p| pd.contains_key(&p.name)) {
            match dim_of(&body, &|s| pd.get(s).cloned()) {
                Ok(d) => d,
                Err(e) => return Err(format!("{n}: {e}")),
            }
        } else {
            None
        };
        let f = Fun { params: ps.clone(), body: body.clone(), written, ast: if clause.solve.is_some() { None } else { Some(rhs.clone()) }, src: match &clause.solve { Some(xs) => format!("{src} -> solve({})", xs.join(", ")), None => src.to_string() }, ret, defaults };
        self.env.define_fun(n, f, self.listed);
        self.out.defined.push(n.to_string());
        if sym {
            let shown = match shown { Some(e) => self.shaped(&e, cfg)?, None => body.clone() };
            let m = self.r(cfg, true).expr(&shown).0;
            self.show("->", m);
        }
        Ok(())
    }

    /// What `-> substitute(…)` shows: `a` as written, with the formulas of the listed variables put in.
    fn substituted(&self, a: &Ast, params: &[String], names: &[String]) -> Result<Expr, String> {
        // constants (their bindings) and variables (their names)
        let consts = constants(names)?;
        let names: Vec<String> = names.iter().filter(|n| !n.starts_with('⟦')).cloned().collect();
        let names = names.as_slice();
        for n in names {
            if params.contains(n) {
                return Err(format!("substitute: '{n}' is a parameter here, it has no formula"));
            }
            if !self.env.vars.contains_key(n) {
                return Err(if self.env.funs.contains_key(n) {
                    format!("substitute puts in variables' formulas – '{n}' is a function, whose calls -> puts in already")
                } else {
                    format!("substitute: '{n}' isn't defined")
                });
            }
        }
        let mut lo = Lower::new(self.env, &self.units);
        lo.params = params.iter().cloned().collect();
        // variables listed: only theirs put in, other names kept; only constants: the values put in, as -> does
        lo.bind = names.is_empty();
        let e = lo.expr(a)?;
        let mut used = BTreeSet::new();
        let e = lo.substitute(&e, names, &mut used)?;
        let e = put_constants(&e, &consts, &mut used);
        if let Some(n) = names.iter().chain(consts.iter().map(|c| &c.identity)).find(|n| !used.contains(*n)) {
            return Err(format!("substitute: '{n}' doesn't occur in '{}'", ast_text(a, &self.units)));
        }
        Ok(e)
    }

    // ---------- values ----------

    fn no_value(&self, e: NoValue, what: &str) -> String {
        match e {
            NoValue::Unknown(ns) => {
                let one = ns.len() == 1;
                format!(
                    "{} in '{what}' {} defined – define {} first with :=, or solve for {}: … -> solve({})",
                    ns.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", "),
                    if one { "isn't" } else { "aren't" },
                    if one { "it" } else { "them" },
                    if one { "it" } else { "them" },
                    ns.join(", ")
                )
            }
            NoValue::Error(m) => m,
        }
    }

    /// The unit of the inputs to reuse for a result: only non-SI ones (h, mAh, eV, your own), one of the
    /// result's dimension.
    fn hint(&self, a: &Ast, d: &Dim) -> Option<UnitExpr> {
        let mut names = vec![];
        ast_names(a, &mut names);
        let mut us: Vec<UnitExpr> = vec![];
        for n in names {
            let Some(v) = self.env.vars.get(&n) else { continue };
            let u = v.unit.clone().or_else(|| match &v.expr {
                Expr::C(c) => c.u.clone(),
                _ => None,
            });
            let Some(u) = u else { continue };
            if self.units.dim(&u) != Some(d.clone()) || us.contains(&u) {
                continue;
            }
            us.push(u);
        }
        if us.len() != 1 {
            return None;
        }
        let u = us.pop().unwrap();
        if u.0.len() == 1 && u.0[0].1 == 1 && !crate::format::si_unit(&u, &self.units) {
            Some(u)
        } else {
            None
        }
    }

    fn show_val(&self, v: &Val, want: Option<&UnitExpr>, hint: Option<&UnitExpr>, cfg: &Cfg) -> Result<(String, bool), String> {
        // a computed value isn't shown as typed
        let s = pick(v, want, hint, &self.units, cfg)?;
        let (m, exact) = shown(&s, &self.units, cfg);
        Ok((m, exact || !cfg.approximate))
    }

    fn eval_markup(&self, e: &Expr, unit: Option<&UnitExpr>, cfg: &Cfg) -> Result<(String, bool), NoValue> {
        let s = simp(e);
        if let Some(m) = linalg::problem(&s) {
            return Err(NoValue::Error(m));
        }
        if let Some(a) = Array::of(&s) {
            let parts = a.a.iter().map(|e| self.eval_markup(e, unit, cfg)).collect::<Result<Vec<_>, _>>()?;
            let exact = parts.iter().all(|p| p.1);
            return Ok((array_markup(&a, parts.into_iter().map(|p| p.0).collect()), exact));
        }
        let z = complex_value(&s)?;
        let hint = self.hint_expr(e, z.d());
        let (m, exact) = crate::format::complex(&z, unit, hint.as_ref(), &self.units, cfg).map_err(NoValue::Error)?;
        Ok((m, exact || !cfg.approximate))
    }

    /// A symbolic result as markup: exact; a value once nothing unknown is left, in the unit asked for.
    fn sym_markup(&self, e: &Expr, unit: Option<&UnitExpr>, cfg: &Cfg) -> Result<String, String> {
        let s = self.shaped(e, cfg)?;
        if let Some(m) = linalg::problem(&s) {
            return Err(m);
        }
        if let Some(a) = Array::of(&s) {
            let parts = a.a.iter().map(|e| self.sym_markup(e, unit, cfg)).collect::<Result<Vec<_>, _>>()?;
            return Ok(array_markup(&a, parts));
        }
        if let (Expr::C(v), true) = (&s, unit.is_some() || cfg.form == "decimal" || cfg.scientific == Some((0, 1))) {
            return Ok(self.show_val(v, unit, None, cfg)?.0);
        }
        // exact in the unit asked for: e³ in, π/50 m; a value when nothing symbolic is left
        if let (Some(u), false) = (unit, cfg.form == "decimal")
            && !matches!(s, Expr::C(_))
            && let Some((m, _)) = self.r(cfg, true).in_unit(&s, u) {
            return Ok(m);
        }
        if unit.is_some() || cfg.form == "decimal" {
            if let Ok(z) = complex_value(&s) {
                return Ok(crate::format::complex(&z, unit, None, &self.units, cfg)?.0);
            }
        }
        Ok(self.r(cfg, cfg.form != "decimal").expr(&s).0)
    }

    fn eval_expr(&mut self, a: &Ast, clause: &Clause, cfg: &Cfg) -> Result<(), String> {
        let e = self.lower(a, &[], true)?;
        // a variable shown again: in the unit it was defined with
        let unit = match a {
            Ast::Name(n) => self.env.vars.get(n).and_then(|x| x.unit.clone()),
            _ => None,
        };
        let want = clause.unit.clone().or(unit);
        let (m, _) = self.eval_markup(&e, want.as_ref(), cfg).map_err(|e| self.no_value(e, &ast_text(a, &self.units)))?;
        self.show("=", m);
        Ok(())
    }

    fn sym_expr(&mut self, a: &Ast, clause: &Clause, cfg: &Cfg) -> Result<(), String> {
        // a constant alone with its expression put in: the expression as written (simplified, √−1 would be i again)
        let alone = match a {
            Ast::Bound(c) if constants(&clause.substitute)?.iter().any(|k| k.identity == c.identity) => Some(c.backing.clone()),
            _ => None,
        };
        let m = match alone {
            Some(backing) => self.r(cfg, true).expr(&backing).0,
            None => {
                let e = if clause.substitute.is_empty() { self.lower(a, &[], true)? } else { self.substituted(a, &[], &clause.substitute)? };
                self.sym_result(&e, clause, cfg)?
            }
        };
        self.show("->", m);
        Ok(())
    }

    /// A symbolic result in the form the clause asks for (simplest, expanded, in factors).
    fn shaped(&self, e: &Expr, cfg: &Cfg) -> Result<Expr, String> {
        shaped(e, cfg.symbolic).ok_or_else(|| "factor finds the factors of exact numbers – write decimals as fractions, or leave the numbers out".into())
    }

    /// What -> shows of an expression: its symbolic result; of an equation, where it holds (`a = 0 or a = 1`,
    /// true, false). With steps, the forms on the way, a line each in a block: the equation with the definitions put
    /// in, one side zero, in factors, where it holds.
    fn sym_result(&self, e: &Expr, clause: &Clause, cfg: &Cfg) -> Result<String, String> {
        let names: BTreeSet<_> = clause.real.iter().cloned().collect();
        for n in &names {
            if self.env.funs.contains_key(n) {
                return Err(format!("assume: {n} is a function, not a scalar"));
            }
            if let Some(v) = self.env.vars.get(n) {
                let bound = self.env.bind_with_units(&v.expr, &BTreeSet::new(), &self.units)?;
                if let Ok(value) = complex_value(&bound) {
                    if !value.is_real() { return Err(format!("assume: {n} has a nonreal value")); }
                } else {
                    return Err(format!("assume: {n} has a symbolic definition; assume its free scalar names instead"));
                }
            }
        }
        // Preserve the comparison itself so identities continue to render as true/false.
        let assumed = if names.is_empty() { e.clone() } else if matches!(e, Expr::Cmp(..)) {
            e.map(&mut |a| crate::complex::assuming_real(a, &names))
        } else { crate::complex::assuming_real(e, &names) };
        let e = &assumed;
        let r = self.r(cfg, true);
        let eq = |l: &Expr, x: &Expr| r.expr(&Expr::Cmp(vec![l.clone(), x.clone()], vec![Rel::Eq])).0;
        let reduced = match e {
            // expand and factor show the sides in their form
            Expr::Cmp(v, rs) if rs[..] == [Rel::Eq] && cfg.symbolic == Form::Simplest && clause.unit.is_none() && !v.iter().any(|x| x.contains(&linalg::is_array)) => {
                solve::reduce(&v[0], &v[1]).map(|red| (&v[0], &v[1], red))
            }
            _ => None,
        };
        let (mut steps, joint) = match reduced {
            Some((l, x, red)) => {
                let condition = if matches!(red.holds, solve::Holds::Where(_)) {
                    crate::complex::real_conditions(&expr::sub(l.clone(), x.clone()))
                } else { None };
                let note = condition.map(|ns| format!("\"true\" \"for\" {}", ns.iter().map(|n| format!("{} in RR", name(n))).collect::<Vec<_>>().join(" and ")));
                if !clause.steps {
                    if let Some(note) = &note { return Ok(note.clone()); }
                }
                let zero = expr::int(0);
                let mut holds = match &red.holds {
                    solve::Holds::Always => "\"true\"".to_string(),
                    solve::Holds::Never => "\"false\"".to_string(),
                    solve::Holds::Where(cs) => cs.iter().map(|(a, b)| eq(a, b)).collect::<Vec<_>>().join(" or "),
                };
                if let Some(note) = note { holds.push_str(&format!(" quad lr(({note}))")); }
                let mut steps = vec![eq(&simplify(l), &simplify(x)), eq(&red.difference, &zero)];
                if matches!(red.holds, solve::Holds::Where(_)) {
                    steps.push(eq(&red.factored, &zero));
                }
                steps.push(holds);
                (steps, "<=>")
            }
            None => {
                let mut steps = vec![];
                if clause.steps {
                    steps.push(r.expr(&simp(e)).0);
                }
                steps.push(self.sym_markup(e, clause.unit.as_ref(), cfg)?);
                (steps, if matches!(e, Expr::Cmp(..)) { "<=>" } else { "=" })
            }
        };
        if !clause.steps {
            return Ok(steps.pop().unwrap_or_default());
        }
        steps.dedup();
        if steps.len() == 1 {
            return Ok(steps.remove(0));
        }
        // in a block a line each, aligned; inline in a row
        Ok(if cfg.block { format!("& {}", steps.join(&format!(" \\ & {joint} "))) } else { steps.join(&format!(" {joint} ")) })
    }

    /// The result after the equation as written: = or -> and the markup.
    fn show(&mut self, mark: &str, m: String) {
        self.out.mark = Some(mark.into());
        self.out.result = Some(m);
    }

    /// = states something: checked when everything is known; prose in math when not.
    fn statement(&mut self, a: &Ast) -> Result<(), String> {
        let Ast::Cmp(first, rest) = a else {
            self.out.prose = true;
            return Ok(());
        };
        let mut sides = vec![(**first).clone()];
        sides.extend(rest.iter().map(|(_, x)| x.clone()));
        let mut vals = vec![];
        for s in &sides {
            let Ok(e) = self.lower(s, &[], true) else {
                self.out.prose = true;
                return Ok(());
            };
            // a function not defined yet inside a definition is unknown like a name: y(t) = … with y(t) := s(t)^2
            // states, it isn't checked; one written here is a mistake (f(x) = … where f isn't defined)
            let mut written = vec![];
            ast_calls(s, &mut written);
            if e.contains(&|x| matches!(x, Expr::Call(n, _) if !written.contains(n))) && !e.contains(&|x| matches!(x, Expr::Call(n, _) if written.contains(n))) {
                self.out.prose = true;
                return Ok(());
            }
            match complex_value(&e) {
                Ok(v) => vals.push(v),
                Err(NoValue::Unknown(_)) => {
                    self.out.prose = true;
                    return Ok(());
                }
                Err(NoValue::Error(m)) => return Err(m),
            }
        }
        // inequalities in math are prose unless solved
        if rest.iter().any(|(r, _)| *r != Rel::Eq) {
            self.out.prose = true;
            return Ok(());
        }
        for (i, (r, _)) in rest.iter().enumerate() {
            let (x, y) = (&vals[i], &vals[i + 1]);
            let same = close_rel(x.re.v, y.re.v) && close_rel(x.im.v, y.im.v);
            let ok = if x.d() != y.d() { *r == Rel::Ne } else { same == (*r == Rel::Eq) };
            if !ok {
                let why = if x.d() != y.d() { "has another unit than" } else { "has another value than" };
                return Err(if *r == Rel::Eq {
                    format!("'{}' {why} '{}' – = states what's true; := defines", ast_text(&sides[i + 1], &self.units), ast_text(&sides[i], &self.units))
                } else {
                    format!("'{}' doesn't hold", ast_text(a, &self.units))
                });
            }
        }
        Ok(())
    }

    // ---------- solving ----------

    /// An equation of vectors or matrices as the equations of its elements (with the known values put in), a
    /// scalar side standing for every element; None for an equation of scalars.
    fn elementwise(&self, e: &Expr, xs: &[String]) -> Result<Option<Vec<Expr>>, String> {
        let Expr::Cmp(v, rs) = e else { return Ok(None) };
        if v.len() != 2 || !v.iter().any(|x| x.contains(&linalg::is_array) || x.contains(&|x| matches!(x, Expr::Sym(_)))) {
            return Ok(None);
        }
        let keep: BTreeSet<String> = xs.iter().cloned().collect();
        let side = |x: &Expr| -> Result<Expr, String> { Ok(simp(&self.env.bind_with_units(x, &keep, &self.units)?)) };
        let (l, r) = (side(&v[0])?, side(&v[1])?);
        let (a, b) = match (Array::of(&l), Array::of(&r)) {
            (None, None) => return Ok(None),
            (Some(a), Some(b)) if (a.rows, a.cols) != (b.rows, b.cols) => return Err(format!("an equation of {} and {}: their sizes must match", a.describe(), b.describe())),
            (Some(a), Some(b)) => (a.a, b.a),
            (Some(a), None) => { let n = a.a.len(); (a.a, vec![r; n]) }
            (None, Some(b)) => { let n = b.a.len(); (vec![l; n], b.a) }
        };
        Ok(Some(a.into_iter().zip(b).map(|(x, y)| Expr::Cmp(vec![x, y], rs.clone())).collect()))
    }

    fn solve(&mut self, stmts: &[Stmt], xs: &[String], clause: &Clause, cfg: &Cfg) -> Result<(), String> {
        let mut unique = BTreeSet::new();
        if xs.iter().any(|x| !unique.insert(x)) { return Err("each solve variable must be different".into()); }
        let mut eqs: Vec<(Expr, Expr, Rel)> = vec![];
        let mut eq_text = vec![];
        for s in stmts {
            let Stmt::Expr(a) = s else { return Err("put a solved definition on its own line".into()); };
            let body = self.lower(a, xs, false)?;
            let bodies = match body {
                Expr::Vector(v) => v,
                e => self.elementwise(&e, xs)?.unwrap_or_else(|| vec![e]),
            };
            for body in bodies {
                let Expr::Cmp(v, rs) = body else {
                    return Err(format!("'{}' isn't an equation – solve(…) needs one with =, like x^2 = 4", ast_text(a, &self.units)));
                };
                if rs.len() != 1 || v.len() != 2 || v.iter().any(|e| e.contains(&|e| matches!(e, Expr::Cmp(..)))) {
                    return Err("one relation per equation; a relation function already contains =".into());
                }
                eq_text.push(ast_text(a, &self.units));
                eqs.push((v[0].clone(), v[1].clone(), rs[0]));
            }
        }
        let ineq = eqs.iter().any(|e| e.2 != Rel::Eq);
        if ineq && eqs.len() > 1 {
            return Err("inequalities are solved one at a time".into());
        }
        if eqs.iter().any(|e| e.2 == Rel::Ne) {
            return Err("≠ can't be solved".into());
        }
        if eqs.len() != xs.len() {
            return Err(format!(
                "{} equation{} but {} unknown{} ({}) – one equation per unknown",
                eqs.len(),
                if eqs.len() == 1 { "" } else { "s" },
                xs.len(),
                if xs.len() == 1 { "" } else { "s" },
                xs.join(", ")
            ));
        }
        if xs.len() > 1 && clause.unit.is_some() {
            return Err("several unknowns – a unit after -> is for one".into());
        }
        // the equations with names kept (the formula), and with values put in
        let keep: Vec<String> = xs.to_vec();
        let mut named = vec![];
        let mut bound = vec![];
        for (l, r, _) in &eqs {
            let (ln, rn) = (l.clone(), r.clone());
            for x in xs {
                if !ln.has(x) && !rn.has(x) && eqs.len() == 1 {
                    return Err(format!("'{x}' does not appear in '{} = {}'", self.r(cfg, false).expr(l).0, self.r(cfg, false).expr(r).0));
                }
            }
            let ks: BTreeSet<String> = keep.iter().cloned().collect();
            bound.push((self.env.bind_with_units(&ln, &ks, &self.units)?, self.env.bind_with_units(&rn, &ks, &self.units)?));
            named.push((ln, rn));
        }
        // names with numeric values (the formula keeps them; polynomials in them are solved numerically)
        let known_num = |n: &str| -> bool {
            self.env.vars.get(n).is_some_and(|v| self.env.bind_with_units(&v.expr, &BTreeSet::new(), &self.units).ok().is_some_and(|e| value(&e).is_ok()))
        };
        let known_dim = |n: &str| -> Option<Dim> {
            let v = self.env.vars.get(n)?;
            value(&self.env.bind_with_units(&v.expr, &BTreeSet::new(), &self.units).ok()?).ok().map(|v| v.d)
        };
        // solutions: per solution, per unknown (formula with names, or None), and the value or symbolic result
        let mut sols: Vec<Vec<(Option<Expr>, Sol)>> = vec![];
        let mut done = false;
        if eqs.len() == 1 {
            let x = &xs[0];
            let (l, r) = &named[0];
            let e = expr::sub(l.clone(), r.clone());
            match solve::solve1(&e, x, &known_num) {
                Ok(ss) => {
                    for s in ss {
                        sols.push(vec![self.finish(Some(s))?]);
                    }
                    done = true;
                }
                Err(Fail::Cancels) => return Err(format!("'{x}' cancels out of '{} = {}'", self.r(cfg, false).expr(&eqs[0].0).0, self.r(cfg, false).expr(&eqs[0].1).0)),
                Err(Fail::NotSymbolic) => {}
            }
            if !done {
                // with the values put in
                let (l, r) = &bound[0];
                let e = expr::sub(l.clone(), r.clone());
                match solve::solve1(&e, x, &|_| true) {
                    Ok(ss) => {
                        for s in ss {
                            sols.push(vec![(None, self.finish(Some(s))?.1)]);
                        }
                        done = true;
                    }
                    Err(Fail::Cancels) => return Err(format!("'{x}' cancels out of '{} = {}'", self.r(cfg, false).expr(&eqs[0].0).0, self.r(cfg, false).expr(&eqs[0].1).0)),
                    Err(Fail::NotSymbolic) => {}
                }
            }
        } else if let Some(ss) = solve::solve_system(&named, xs, &known_num) {
            for s in ss {
                let mut one = vec![];
                for x in xs {
                    one.push(self.finish(s.get(x).cloned())?);
                }
                sols.push(one);
            }
            done = true;
        }
        // solved symbolically: in the complex numbers; numerically: real solutions only
        let symbolic = done;
        if !done {
            // numerically: everything but the unknowns must be known
            let mut free = vec![];
            for (l, r) in &bound {
                for s in l.syms().into_iter().chain(r.syms()) {
                    if !xs.contains(&s) && !free.contains(&s) {
                        free.push(s);
                    }
                }
            }
            if !free.is_empty() {
                let one = free.len() == 1;
                return Err(format!(
                    "{} {} defined, and the equation can't be solved for {} with {} as {} – define {} first",
                    free.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", "),
                    if one { "isn't" } else { "aren't" },
                    xs.join(", "),
                    if one { "it" } else { "them" },
                    if one { "a name" } else { "names" },
                    if one { "it" } else { "them" }
                ));
            }
            let dims = solve::infer_dims(&bound, xs, &known_dim)?;
            let dims: Vec<Dim> = dims
                .iter()
                .enumerate()
                .map(|(i, d)| match (d, xs.len() == 1, &clause.unit) {
                    (None, true, Some(u)) => self.units.dim(u).unwrap_or(Dim::ONE),
                    (d, ..) => d.clone().unwrap_or_else(|| { let _ = i; Dim::ONE }),
                })
                .collect();
            let scale: Vec<f64> = xs.iter().map(|_| clause.unit.as_ref().map(|u| self.units.factor(u).f()).unwrap_or(1.0)).collect();
            let found: Vec<Vec<f64>> = if xs.len() == 1 {
                let (l, r) = &bound[0];
                // nearest to zero first: periodic functions have roots everywhere
                let mut ts = solve::scan(&simp(&expr::sub(l.clone(), r.clone())), &xs[0], scale[0]);
                ts.sort_by(|a, b| a.abs().total_cmp(&b.abs()).then(b.total_cmp(a)));
                ts.into_iter().map(|t| vec![t]).collect()
            } else {
                solve::NumSys::new(&bound, xs).multi(&scale)
            };
            for s in found {
                sols.push(s.iter().zip(&dims).map(|(v, d)| (None, Sol::Value(CVal::real(Val::new(solve::snap(*v), d.clone()))))).collect());
            }
        }
        // inequalities: the side of the boundary
        let mut rel = None;
        if ineq {
            let r0 = eqs[0].2;
            let x = &xs[0];
            if sols.len() != 1 {
                return Err(format!(
                    "{} for the inequality '{}' – only inequalities with one boundary are supported",
                    if sols.is_empty() { "no boundary" } else { "several boundaries" },
                    eq_text[0]
                ));
            }
            let Some(b) = (match &sols[0][0].1 { Sol::Value(z) if z.is_real() => Some(&z.re), _ => None }) else { return Err("the boundary of an inequality needs a real value".into()) };
            let (l, r) = &bound[0];
            let f = simp(&expr::sub(l.clone(), r.clone()));
            // more boundaries the isolating didn't see (y² < 4)
            let more: Vec<f64> = solve::scan(&f, x, if b.v.f() == 0.0 { 1.0 } else { b.v.f().abs() })
                .into_iter()
                .filter(|t| (t - b.v.f()).abs() > 1e-6 * b.v.f().abs().max(1e-300))
                .collect();
            if !more.is_empty() {
                return Err(format!("several boundaries for the inequality '{}' – only inequalities with one boundary are supported", eq_text[0]));
            }
            let d = f.diff(x);
            let slope = crate::eval::eval_f(&d, &|n| (n == x).then_some(b.v.f())).filter(|s| *s != 0.0);
            let Some(slope) = slope else { return Err(format!("can't tell on which side of its boundary {x} satisfies the inequality")) };
            rel = Some(if slope > 0.0 { r0 } else { r0.flip() });
        }
        self.out.solutions = sols.iter().map(|s| s.iter().map(|(_, sol)| match sol { Sol::Value(z) if z.is_real() => z.re.v.f(), _ => f64::NAN }).collect()).collect();
        // which solution is kept: the only all-positive one, else the first
        let pick_i = {
            let pos: Vec<usize> = (0..sols.len())
                .filter(|&i| sols[i].iter().all(|(_, s)| matches!(s, Sol::Value(z) if z.is_real() && z.re.v.f() > 0.0)))
                .collect();
            if pos.len() == 1 { Some(pos[0]) } else if sols.is_empty() { None } else { Some(0) }
        };
        if let Some(i) = pick_i {
            for (j, x) in xs.iter().enumerate() {
                if let Sol::Value(z) = &sols[i][j].1 {
                    let unit = if xs.len() == 1 { clause.unit.clone() } else { None };
                    let plain = |v: &Val| Val { u: None, raw: None, ..v.clone() };
                    let var = Var { expr: crate::complex::expr(CVal { re: plain(&z.re), im: plain(&z.im) }), ast: None, unit, rel };
                    self.env.define_var(x, var, Kind::Computed, self.listed);
                    self.out.defined.push(x.clone());
                }
            }
        }
        // what's shown: x = …, the solutions of one unknown in a row, those of several each in its own line
        let rel_m = rel.map(render::rel).unwrap_or("=");
        let n = cfg.max_solutions.max(1);
        let mut results: Vec<String> = vec![];
        if sols.is_empty() {
            results.push(if symbolic { "\"no solution\"" } else { "\"no real solution\"" }.into());
        }
        for (k, s) in sols.iter().enumerate().take(n) {
            for (j, x) in xs.iter().enumerate() {
                let lhs = if sols.len() > 1 && xs.len() == 1 { solution_name(x, k + 1) } else { name(x) };
                let (formula, sol) = &s[j];
                let unit = if xs.len() == 1 { clause.unit.as_ref() } else { None };
                let res = match sol {
                    Sol::Value(z) => {
                        let hint = formula.as_ref().and_then(|f| self.hint_expr(f, z.d()));
                        crate::format::complex(z, unit, hint.as_ref(), &self.units, cfg)?.0
                    }
                    Sol::Symbolic(e) => self.r(cfg, true).expr(&self.shaped(e, cfg)?).0,
                };
                results.push(format!("{lhs} {rel_m} {res}"));
            }
        }
        if sols.len() > n {
            results.push("dots.h".into());
        }
        self.show("->", results.join(if xs.len() == 1 { ", quad " } else { ", " }));
        Ok(())
    }

    /// A solution's formula (names kept) → (formula, its value or symbolic result).
    fn finish(&self, f: Option<Expr>) -> Result<(Option<Expr>, Sol), String> {
        let Some(f) = f else { return Err("no solution".into()) };
        let b = simplify(&self.env.bind_with_units(&f, &BTreeSet::new(), &self.units)?);
        Ok(match complex_value(&b) {
            Ok(z) => (Some(f), Sol::Value(z)),
            Err(NoValue::Unknown(_)) => (Some(f), Sol::Symbolic(b)),
            Err(NoValue::Error(e)) => return Err(e),
        })
    }

    fn hint_expr(&self, f: &Expr, d: &Dim) -> Option<UnitExpr> {
        let a = Ast::Mul(f.syms().into_iter().map(|s| (parse::MulOp::Implicit, Ast::Name(s))).collect());
        self.hint(&a, d)
    }
}

fn close_rel(a: Num, b: Num) -> bool {
    match (a, b) {
        (Num::R(x), Num::R(y)) => x == y,
        _ => close(a.f(), b.f()),
    }
}

/// The constants among substitute's names: their bindings.
fn constants(names: &[String]) -> Result<Vec<crate::bindings::Constant>, String> {
    names.iter().filter_map(|n| n.strip_prefix('⟦')?.strip_suffix('⟧')).map(|token| {
        let b = crate::bindings::Binding::decode(token)?;
        match (b.unit, b.identity, b.backing) {
            (false, Some(identity), Some(backing)) => Ok(crate::bindings::Constant { identity, expr: b.expr, backing }),
            _ => Err("substitute: a unit has no formula to put in".to_string()),
        }
    }).collect()
}

/// The constants' expressions put in for them, wherever they are: π for pi, √−1 for i.
fn put_constants(e: &Expr, consts: &[crate::bindings::Constant], used: &mut BTreeSet<String>) -> Expr {
    if consts.is_empty() {
        return e.clone();
    }
    let identity = match e {
        Expr::Const(id, _) => Some(id.as_str()),
        e => crate::bindings::identity_of(e),
    };
    if let Some(c) = identity.and_then(|id| consts.iter().find(|c| c.identity == id)) {
        used.insert(c.identity.clone());
        return c.backing.clone();
    }
    e.map(&mut |x| put_constants(x, consts, used))
}

/// The k-th solution's name: x₁, and for a name with an index of its own x_(a, 1).
fn solution_name(x: &str, k: usize) -> String {
    match x.split_once('_') {
        Some((base, sub)) => format!("{}_(upright({}), {k})", name(base), format::q(sub)),
        None => format!("{}_({k})", name(x)),
    }
}

/// A vector or matrix from the markup of its elements.
fn array_markup(a: &Array, parts: Vec<String>) -> String {
    if a.cols == 1 { format!("vec({})", parts.join(", ")) } else { render::mat(parts.chunks(a.cols).map(<[String]>::to_vec).collect()) }
}

/// A value written out: 7.2 kΩ, -3, [5 V].
fn literal(a: &Ast) -> bool {
    match a {
        Ast::Bound(_) | Ast::Num(..) | Ast::Qty(..) | Ast::Unit(..) => true,
        Ast::Neg(x) => literal(x),
        // vectors and matrices of values
        Ast::Matrix(rs) => rs.iter().flatten().all(literal),
        Ast::Builtin(n, args) if n == "vec" => args.iter().all(literal),
        _ => false,
    }
}

pub(crate) fn ast_names(a: &Ast, out: &mut Vec<String>) {
    match a {
        Ast::Name(n) => {
            if !out.contains(n) {
                out.push(n.clone())
            }
        }
        Ast::Qty(x, _) | Ast::Neg(x) | Ast::Abs(x) => ast_names(x, out),
        Ast::Add(ts) => ts.iter().for_each(|t| ast_names(&t.1, out)),
        Ast::Mul(fs) => fs.iter().for_each(|f| ast_names(&f.1, out)),
        Ast::Div(x, y) | Ast::Pow(x, y) => {
            ast_names(x, out);
            ast_names(y, out);
        }
        // a local variable (of an integral, a sum, a derivative, a convolution) is no name of the worksheet
        Ast::Builtin(n, args) if matches!((n.as_str(), args.len()), ("integrate" | "sum", 4) | ("integrate" | "diff", 2) | ("convolve", 3 | 4)) => {
            let (bodies, var) = if n == "convolve" { (&args[..2], &args[2]) } else { (&args[..1], &args[1]) };
            let mut body = vec![];
            for b in bodies { ast_names(b, &mut body); }
            for n in body {
                if !matches!(var, Ast::Name(v) if *v == n) && !out.contains(&n) {
                    out.push(n);
                }
            }
            let rest = if n == "convolve" { &args[3..] } else if n == "diff" { &args[2..] } else { &args[2..] };
            for bound in rest { ast_names(bound, out); }
        }
        Ast::Call(_, args) | Ast::Builtin(_, args) => args.iter().for_each(|x| ast_names(x, out)),
        Ast::Arg(_, x) => ast_names(x, out),
        Ast::Index(x, idx) => {
            ast_names(x, out);
            idx.iter().for_each(|i| ast_names(i, out));
        }
        Ast::Range(x, y) => {
            ast_names(x, out);
            ast_names(y, out);
        }
        Ast::Matrix(rs) => rs.iter().flatten().for_each(|x| ast_names(x, out)),
        Ast::Cmp(f, rest) => {
            ast_names(f, out);
            rest.iter().for_each(|r| ast_names(&r.1, out));
        }
        Ast::And(v) | Ast::Or(v) => v.iter().for_each(|x| ast_names(x, out)),
        _ => {}
    }
}

/// The functions an expression calls by name, as written.
fn ast_calls(a: &Ast, out: &mut Vec<String>) {
    match a {
        Ast::Call(n, args) => {
            if !out.contains(n) { out.push(n.clone()); }
            args.iter().for_each(|x| ast_calls(x, out));
        }
        Ast::Builtin(_, args) => args.iter().for_each(|x| ast_calls(x, out)),
        Ast::Qty(x, _) | Ast::Neg(x) | Ast::Abs(x) | Ast::Arg(_, x) => ast_calls(x, out),
        Ast::Add(ts) => ts.iter().for_each(|t| ast_calls(&t.1, out)),
        Ast::Mul(fs) => fs.iter().for_each(|f| ast_calls(&f.1, out)),
        Ast::Div(x, y) | Ast::Pow(x, y) | Ast::Range(x, y) => { ast_calls(x, out); ast_calls(y, out); }
        Ast::Index(x, idx) => { ast_calls(x, out); idx.iter().for_each(|i| ast_calls(i, out)); }
        Ast::Matrix(rs) => rs.iter().flatten().for_each(|x| ast_calls(x, out)),
        Ast::Cmp(f, rest) => { ast_calls(f, out); rest.iter().for_each(|r| ast_calls(&r.1, out)); }
        Ast::And(v) | Ast::Or(v) => v.iter().for_each(|x| ast_calls(x, out)),
        _ => {}
    }
}

/// An expression as text, for messages; units by their names.
pub fn ast_text(a: &Ast, units: &Units) -> String {
    fn p(a: &Ast, min: u8, u: &Units) -> String {
        let (s, prec) = t(a, u);
        if prec < min { format!("({s})") } else { s }
    }
    fn t(a: &Ast, u: &Units) -> (String, u8) {
        let list = |xs: &[Ast]| xs.iter().map(|x| t(x, u).0).collect::<Vec<_>>().join(", ");
        match a {
            Ast::Bound(c) => (c.identity.clone(), 9),
            Ast::Num(r, _) => (r.clone(), 9),
            Ast::Qty(n, w) => (format!("{} {}", p(n, 9, u), u.text(w)), 3),
            Ast::Unit(w) => (u.text(w), 9),
            Ast::Name(n) => (n.clone(), 9),
            Ast::Neg(x) => (format!("-{}", p(x, 3, u)), 2),
            Ast::Add(ts) => (
                ts.iter().enumerate().map(|(i, (n, x))| format!("{}{}", if *n { if i == 0 { "-" } else { " - " } } else if i > 0 { " + " } else { "" }, p(x, 2, u))).collect(),
                1,
            ),
            Ast::Mul(fs) => (
                fs.iter().enumerate().map(|(i, (o, x))| format!("{}{}", if i == 0 { "" } else if *o == parse::MulOp::Implicit { " " } else { " * " }, p(x, 3, u))).collect(),
                3,
            ),
            Ast::Div(x, y) => (format!("{}/{}", p(x, 3, u), p(y, 4, u)), 3),
            Ast::Pow(x, y) => (format!("{}^{}", p(x, 9, u), p(y, 9, u)), 5),
            Ast::Call(n, args) | Ast::Builtin(n, args) => (format!("{n}({})", list(args)), 9),
            Ast::Arg(n, x) => (format!("{n}: {}", t(x, u).0), 0),
            Ast::Index(x, idx) => (format!("{}[{}]", p(x, 9, u), list(idx)), 9),
            Ast::Range(x, y) => (format!("{} .. {}", p(x, 1, u), p(y, 1, u)), 0),
            Ast::All => (":".into(), 9),
            Ast::Abs(x) => (format!("|{}|", t(x, u).0), 9),
            Ast::Matrix(rs) => (format!("mat({})", rs.iter().map(|r| list(r)).collect::<Vec<_>>().join("; ")), 9),
            Ast::Cmp(f, rest) => {
                let mut s = p(f, 1, u);
                for (r, x) in rest {
                    s += &format!(" {} {}", r.text(), p(x, 1, u));
                }
                (s, 0)
            }
            Ast::And(v) => (v.iter().map(|x| if matches!(x, Ast::Or(_)) { format!("({})", t(x, u).0) } else { t(x, u).0 }).collect::<Vec<_>>().join(" ∧ "), 0),
            Ast::Or(v) => (v.iter().map(|x| t(x, u).0).collect::<Vec<_>>().join(" ∨ "), 0),
        }
    }
    t(a, units).0
}


#[cfg(test)]
mod tests {
    use super::*;

    struct T {
        env: Env,
        cfg: Cfg,
    }
    impl T {
        fn new() -> T {
            T { env: Env::default(), cfg: Cfg::default() }
        }
        /// A readable line, as Typst sends it; results and errors readable again.
        fn math(&mut self, s: &str) -> Out {
            let source = crate::test_support::src(s, &self.env);
            let mut o = Sheet::new(&mut self.env, &self.cfg, true).unwrap().run(&source);
            o.result = o.result.map(|r| crate::test_support::readable(&r));
            o.errors = o.errors.iter().map(|e| crate::test_support::readable(e)).collect();
            o
        }
        fn ok(&mut self, s: &str) -> Out {
            let o = self.math(s);
            assert!(o.errors.is_empty(), "{s}: {:?}", o.errors);
            o
        }
        fn val(&self, n: &str) -> f64 {
            let v = &self.env.vars[n];
            value(&self.env.bind(&v.expr, &BTreeSet::new()).unwrap()).unwrap().v.f()
        }
    }

    #[test]
    fn users_examples() {
        let mut t = T::new();
        let o = t.ok("q(x: A, ..) ≔ 1 + frac(3, 3)x + y^(2) → ⦅simplify⦆");
        assert_eq!(o.result.as_deref(), Some("x + y^(\"2\") + \"1\""));
        let o = t.ok("0 = 1 + frac(3, x) + y^(2) → ⦅solve(x)⦆");
        assert_eq!(o.result.as_deref(), Some("x = -frac(\"3\", y^(\"2\") + \"1\")"));
        let o = t.math("f(y_2) = q(x_2, y_2)");
        assert!(o.errors.iter().any(|e| e.contains("f isn't defined")));
        let o = t.ok("M ≔ 1 = 2 =");
        assert_eq!(o.result.as_deref(), Some("\"0\""));
        let o = t.ok("M ≔ 1 #A = 2 #V →");
        assert_eq!(o.result.as_deref(), Some("\"0\""));
        let o = t.ok("M =");
        assert_eq!(o.result.as_deref(), Some("\"0\""));
    }

    #[test]
    fn substitution() {
        let mut t = T::new();
        t.ok("h ≔ 5 z");
        t.ok("z ≔ 20 #cm");
        let o = t.ok("U(x: m) ≔ h x a →");
        assert_eq!(o.result.as_deref(), Some("a dot x thin upright(\"m\")"));
        let o = t.ok("h(x: m) ≔ frac(U(x), a) →");
        assert_eq!(o.result.as_deref(), Some("x thin upright(\"m\")"));
        let o = t.ok("y ≔ h(2 #m) =");
        assert_eq!(o.result.as_deref(), Some("\"2\" thin upright(\"m\")^(2)"));
    }

    #[test]
    fn late_binding() {
        let mut t = T::new();
        t.ok("h ≔ 5 z");
        let o = t.math("h =");
        assert!(o.errors[0].contains("'z'"), "{:?}", o.errors);
        t.ok("z ≔ 2");
        let o = t.ok("h =");
        assert_eq!(o.result.as_deref(), Some("\"10\""));
    }

    #[test]
    fn worksheet() {
        let mut t = T::new();
        t.ok("Q_0 ≔ 3450 #mAh");
        t.ok("I ≔ 30 #mA");
        let o = t.ok("t ≔ frac(Q_0, I) = 1 #h");
        assert_eq!(o.result.as_deref(), Some("\"115\" thin upright(\"h\")"));
        let o = t.ok("frac(Q_0, 2) = I t → ⦅solve(t)⦆");
        assert_eq!(o.result.as_deref(), Some("t = \"57.5\" thin upright(\"h\")"));
        assert!((t.val("t") - 207000.0).abs() < 1e-6);
        let o = t.ok("x^(2) + 2x - 3 = 0 → ⦅solve(x)⦆");
        assert!(o.result.unwrap().contains("x_(1) = \"−3\""));
        assert_eq!(t.val("x"), 1.0);
        t.ok("U ≔ 5 #V");
        let o = t.ok("R ≤ frac(U, I) → ⦅solve(R)⦆");
        assert!(o.result.unwrap().starts_with("R <= \"166.7\""));
        let o = t.ok("c ≔ frac(1, 3) → ⦅frac⦆");
        assert_eq!(o.result.as_deref(), Some("frac(\"1\", \"3\")"));
        let o = t.ok("a + b = 3, a - b = 1 → ⦅solve(a, b)⦆");
        assert!(o.errors.is_empty());
        assert_eq!(t.val("a"), 2.0);
        // a check that fails
        let o = t.math("U = 6 #V");
        assert!(o.errors[0].contains("another value"));
        // prose
        assert!(t.math("E = m c^(2)").prose);
        let o = t.ok("cos(w) = w → ⦅solve(w)⦆");
        assert!(o.result.unwrap().contains("0.7391"));
    }

    #[test]
    fn typed_parameters() {
        let mut t = T::new();
        t.ok("U(x: A) := x · 2 Ω");
        t.ok("z := U(3 A) =");
        assert_eq!(t.val("z"), 6.0);
        let o = t.math("w := U(2 V)");
        assert!(o.errors[0].contains("takes x in A, got V"), "{:?}", o.errors);
    }
}
