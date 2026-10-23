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
use crate::eval::{NoValue, dim_of, value};
use crate::expr::{self, Expr, Val, simp};
use crate::format::{self, Cfg, Shown, name, pick, shown};
use crate::linalg::{self, Array};
use crate::num::{Num, close};
use crate::parse::{self, Ast, Clause, Ctx, Head, Line, Mode, Param, ParamType, Rel, Stmt, Trail};
use crate::poly::simplify;
use crate::render::{self, P_ATOM, P_PROD, R};
use crate::solve::{self, Fail};
use crate::units::{UnitExpr, Units};

/// What a line gives the Typst side.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Out {
    /// Math: "=" or "->" when a result follows the equation as written.
    pub mark: Option<String>,
    /// Math: the result after the mark.
    pub result: Option<String>,
    /// Code: the line with its steps.
    pub show: Option<String>,
    pub errors: Vec<String>,
    /// Single letters read as units (set upright in the equation as written).
    pub letters: Vec<String>,
    /// Math that isn't a calculation: left alone.
    pub prose: bool,
    /// What the line defined.
    pub defined: Vec<String>,
    /// The solutions of a solve, each the values of its unknowns in SI (NaN where one is symbolic).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub solutions: Vec<Vec<f64>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PartKind {
    Formula,
    Sub,
    Res,
}

#[derive(Clone, Debug)]
struct Part {
    rel: String,
    rhs: String,
    kind: PartKind,
}

#[derive(Clone, Debug)]
struct Seg {
    lhs: String,
    parts: Vec<Part>,
}

#[derive(Clone, Debug)]
struct Stage {
    segs: Vec<Seg>,
    join: String,
}

fn stage(lhs: String, parts: Vec<Part>) -> Stage {
    Stage { segs: vec![Seg { lhs, parts }], join: String::new() }
}

fn part(rel: &str, rhs: String, kind: PartKind) -> Part {
    Part { rel: rel.into(), rhs, kind }
}

/// What a solved unknown came out as.
#[derive(Clone, Debug)]
enum Sol {
    Value(Val),
    Symbolic(Expr),
}

pub struct Sheet<'a> {
    pub env: &'a mut Env,
    pub units: Units,
    pub cfg: Cfg,
    pub mode: Mode,
    pub listed: bool,
    out: Out,
}

impl<'a> Sheet<'a> {
    pub fn new(env: &'a mut Env, cfg: &Cfg, mode: Mode, listed: bool) -> Result<Sheet<'a>, String> {
        let units = Units::build(&cfg.units)?;
        Ok(Sheet { env, units, cfg: cfg.clone(), mode, listed, out: Out::default() })
    }

    /// Calculates a line.
    pub fn run(mut self, src: &str) -> Out {
        let calc = src.contains(":=") || src.contains('≔') || src.contains('⦅');
        let line = {
            let env = &*self.env;
            let is_fn = |n: &str| env.funs.contains_key(n);
            let is_var = |n: &str| env.vars.contains_key(n);
            let ctx = Ctx { units: &self.units, mode: self.mode, is_fn: &is_fn, is_var: &is_var, strict: self.cfg.strict };
            parse::parse_line(src, &ctx)
        };
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                if self.mode == Mode::Math && !calc {
                    self.out.prose = true;
                } else {
                    self.out.errors.push(e);
                }
                return self.out;
            }
        };
        self.out.letters = line.unit_letters.clone();
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
        if let Some(xs) = &clause.solve {
            if !l.stmts.iter().any(|s| matches!(s, Stmt::Def { .. })) {
                return self.solve(&l.stmts, xs, &clause, &cfg);
            }
            if l.stmts.len() != 1 {
                return Err("put a solved definition on its own line".into());
            }
        }
        let mut stages = vec![];
        let n = l.stmts.len();
        for (i, s) in l.stmts.iter().enumerate() {
            let last = i == n - 1;
            let t = if last { trail } else { None };
            match self.stmt(s, t, &clause, &cfg) {
                Ok(st) => stages.extend(st),
                Err(e) => self.out.errors.push(e),
            }
        }
        if self.mode == Mode::Code && !stages.is_empty() {
            let shown: Vec<String> = stages.iter().map(|st| self.render_stages(std::slice::from_ref(st), &cfg)).collect();
            self.out.show = Some(shown.join(if cfg.block { " \\ " } else { ", " }));
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
        R { units: &self.units, cfg, exact, sub: None }
    }

    fn lower(&self, a: &Ast, keep: &[String], bind: bool) -> Result<Expr, String> {
        let mut lo = Lower::new(self.env, &self.units);
        lo.params = keep.iter().cloned().collect();
        lo.bind = bind;
        lo.expr(a)
    }

    /// One statement; stages for code.
    fn stmt(&mut self, s: &Stmt, trail: Option<Trail>, clause: &Clause, cfg: &Cfg) -> Result<Vec<Stage>, String> {
        if !clause.substitute.is_empty() && (trail != Some(Trail::Sym) || clause.solve.is_some()) {
            return Err("substitute shows a formula: it goes after ->, without solve".into());
        }
        if let Some((operation, variables)) = &clause.transform {
            if trail.is_some() {
                let wrap = |body: &Ast| Ast::Call(operation.clone(), std::iter::once(body.clone()).chain(variables.iter().cloned().map(Ast::Name)).collect());
                let transformed = match s {
                    Stmt::Def { head, rhs, src } => Stmt::Def { head: head.clone(), rhs: wrap(rhs), src: src.clone() },
                    Stmt::Expr(body) => Stmt::Expr(wrap(body)),
                };
                let mut local = clause.clone();
                local.transform = None;
                return self.stmt(&transformed, trail, &local, cfg);
            }
        }
        match s {
            Stmt::Def { head: Head::Fn { name, params, rest }, rhs, src } => {
                if trail == Some(Trail::Eval) {
                    return Err(format!("a function has no value of its own – evaluate a call, {name}(2) ="));
                }
                self.define_fn(name, params, *rest, rhs, src, trail == Some(Trail::Sym), clause, cfg)
            }
            Stmt::Def { head: Head::Var(n), rhs, .. } => self.define_var(n, rhs, trail, clause, cfg),
            Stmt::Expr(a) => match trail {
                Some(Trail::Eval) => self.eval_expr(a, clause, cfg),
                Some(Trail::Sym) => self.sym_expr(a, clause, cfg),
                None => self.statement(a, cfg),
            },
        }
    }

    // ---------- definitions ----------

    fn define_var(&mut self, n: &str, rhs: &Ast, trail: Option<Trail>, clause: &Clause, cfg: &Cfg) -> Result<Vec<Stage>, String> {
        let e = if let Some(xs) = &clause.solve { self.solve_body(rhs, &[], xs)? } else { self.lower(rhs, &[], true)? };
        if e.has(n) || e.contains(&|e| matches!(e, Expr::Call(name, _) if name == n)) {
            return Err(format!("'{n}' is used in its own definition but isn't defined yet – define it first, or solve for it: … -> solve({n})"));
        }
        // what substitute shows, from the definitions before this one
        let shown = if clause.substitute.is_empty() { None } else { Some(self.substituted(rhs, &[], &clause.substitute)?) };
        let given = literal(rhs);
        let s = simp(&e);
        // a symbolic result asked for: what's stored is the simplified form
        let stored = if trail == Some(Trail::Sym) { simplify(&s) } else { s.clone() };
        let unit = clause.unit.clone();
        if let (Some(u), Ok(v)) = (&unit, value(&stored)) {
            pick(&v, Some(u), None, &self.units, cfg)?;
        }
        let var = Var { expr: stored.clone(), ast: if clause.solve.is_some() { None } else { Some(rhs.clone()) }, unit: unit.clone(), rel: None };
        self.env.define_var(n, var, if given { Kind::Given } else { Kind::Computed }, self.listed);
        self.out.defined.push(n.to_string());
        let lhs = name(n);
        match trail {
            Some(Trail::Eval) => {
                let (m, exact) = self.eval_markup(&stored, unit.as_ref(), cfg).map_err(|e| self.no_value(e, n))?;
                self.out.mark = Some("=".into());
                self.out.result = Some(m.clone());
                let mut stages = self.eval_stages(lhs, rhs, m, exact, cfg, given);
                if let Some(p) = stages.first_mut().and_then(|s| s.segs.first_mut()).and_then(|s| s.parts.first_mut()) {
                    p.rel = "≔".into();
                }
                Ok(stages)
            }
            Some(Trail::Sym) => {
                let m = self.sym_markup(shown.as_ref().unwrap_or(&stored), unit.as_ref(), cfg)?;
                self.out.mark = Some("->".into());
                self.out.result = Some(m.clone());
                let f = self.r(cfg, false).ast(rhs, &|_| None).0;
                Ok(vec![stage(lhs, vec![part("≔", f, PartKind::Formula), part("->", m, PartKind::Res)])])
            }
            None => {
                if self.mode == Mode::Math {
                    return Ok(vec![]);
                }
                let f = self.r(cfg, false).ast(rhs, &|_| None).0;
                Ok(vec![stage(lhs, vec![part("≔", f, PartKind::Formula)])])
            }
        }
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
    fn define_fn(&mut self, n: &str, params: &[Param], rest: bool, rhs: &Ast, src: &str, sym: bool, clause: &Clause, cfg: &Cfg) -> Result<Vec<Stage>, String> {
        if parse::is_builtin(n) {
            return Err(format!("{n} is a built-in function and can't be defined again"));
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
                    ps.push(Param { name: s, ty: None });
                }
            }
        }
        for p in ps.iter_mut() {
            if p.ty.is_none() {
                p.ty = inferred.get(&p.name).cloned();
            }
        }
        // a parameter of the same name is the parameter: y(y) := X(y)
        if body.contains(&|e| matches!(e, Expr::Call(name, _) if name == n)) || (body.has(n) && !ps.iter().any(|p| p.name == n)) {
            return Err(format!("{n} is used in its own definition – functions can't call themselves"));
        }
        let shown = if clause.substitute.is_empty() { None } else { Some(self.substituted(rhs, &ps.iter().map(|p| p.name.clone()).collect::<Vec<_>>(), &clause.substitute)?) };
        let written = (clause.solve.is_none() && body.contains(&|e| matches!(e, Expr::Mul(_)))).then(|| body.clone());
        let body = if sym { simplify(&body) } else { simp(&body) };
        let pd: BTreeMap<String, Dim> = ps
            .iter()
            .filter_map(|p| match &p.ty {
                Some(ParamType::Unit(u)) => self.units.dim(u).map(|d| (p.name.clone(), d)),
                _ => None,
            })
            .collect();
        let ret = if ps.iter().all(|p| pd.contains_key(&p.name)) {
            match dim_of(&body, &|s| pd.get(s).copied()) {
                Ok(d) => d,
                Err(e) => return Err(format!("{n}: {e}")),
            }
        } else {
            None
        };
        let f = Fun { params: ps.clone(), body: body.clone(), written, ast: if clause.solve.is_some() { None } else { Some(rhs.clone()) }, src: match &clause.solve { Some(xs) => format!("{src} -> solve({})", xs.join(", ")), None => src.to_string() }, ret };
        self.env.define_fun(n, f, self.listed);
        self.out.defined.push(n.to_string());
        let lhs = self.fn_head(n, &ps, cfg);
        if sym {
            let m = self.r(cfg, true).expr(&shown.map_or_else(|| body.clone(), |e| simplify(&e))).0;
            self.out.mark = Some("->".into());
            self.out.result = Some(m.clone());
            let f = self.r(cfg, false).ast(rhs, &|_| None).0;
            return Ok(vec![stage(lhs, vec![part("≔", f, PartKind::Formula), part("->", m, PartKind::Res)])]);
        }
        let f = self.r(cfg, false).ast(rhs, &|_| None).0;
        Ok(vec![stage(lhs, vec![part("≔", f, PartKind::Formula)])])
    }

    /// What `-> substitute(…)` shows: `a` as written, with the formulas of the listed variables put in.
    fn substituted(&self, a: &Ast, params: &[String], names: &[String]) -> Result<Expr, String> {
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
        lo.bind = false;
        let e = lo.expr(a)?;
        let mut used = BTreeSet::new();
        let e = lo.substitute(&e, names, &mut used)?;
        if let Some(n) = names.iter().find(|n| !used.contains(*n)) {
            return Err(format!("substitute: '{n}' doesn't occur in '{}'", ast_text(a)));
        }
        Ok(e)
    }

    fn fn_head(&self, n: &str, ps: &[Param], cfg: &Cfg) -> String {
        let r = self.r(cfg, false);
        let args: Vec<String> = ps
            .iter()
            .map(|p| match &p.ty {
                Some(ParamType::Unit(u)) => format!("{}{}", name(&p.name), r.cast_mark(u, "type")),
                Some(ParamType::Any) => format!("{}{}", name(&p.name), r.cast_mark(&UnitExpr::one("any"), "type")),
                None => name(&p.name),
            })
            .collect();
        format!("{}({})", name(n), args.join(", "))
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
            if self.units.dim(&u) != Some(*d) || us.contains(&u) {
                continue;
            }
            us.push(u);
        }
        if us.len() != 1 {
            return None;
        }
        let u = us.pop().unwrap();
        let si = ["V", "A", "Ω", "Omega", "W", "J", "C", "F", "H", "S", "N", "Pa", "T", "Wb", "s", "m", "g", "K", "Hz", "mol", "cd"];
        let base = |n: &str| -> String {
            if self.units.map.contains_key(n) { n.to_string() } else { n.chars().skip(1).collect() }
        };
        if u.0.len() == 1 && u.0[0].1 == 1 && !si.contains(&base(&u.0[0].0).as_str()) {
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

    fn val_markup(&self, v: &Val, unit: Option<&UnitExpr>, cfg: &Cfg) -> (String, u8) {
        match pick(v, unit, None, &self.units, cfg) {
            Ok(s) => {
                let p = if s.n.is_neg() { 2 } else if s.u.is_empty() || s.bare { P_ATOM } else { P_PROD };
                (shown(&s, &self.units, cfg).0, p)
            }
            Err(_) => (format!("{}", v.v.f()), P_ATOM),
        }
    }

    /// Values of defined names, for the step with numbers.
    fn values(&self, cfg: &Cfg) -> impl Fn(&str) -> Option<(String, u8)> + '_ {
        let cfg = cfg.clone();
        move |n: &str| {
            let v = self.env.vars.get(n)?;
            let val = value(&self.env.bind_with_units(&v.expr, &BTreeSet::new(), &self.units).ok()?).ok()?;
            Some(self.val_markup(&val, v.unit.as_ref(), &cfg))
        }
    }

    /// x = formula = numbers = result.
    #[allow(clippy::too_many_arguments)]
    fn eval_stages(&self, lhs: String, a: &Ast, res: String, exact: bool, cfg: &Cfg, given: bool) -> Vec<Stage> {
        let rel = if exact { "=" } else { "approx" };
        if given {
            return vec![stage(lhs, vec![part("=", res, PartKind::Res)])];
        }
        let r = self.r(cfg, false);
        let vals = self.values(cfg);
        let formula = r.ast(a, &|_| None).0;
        let sub = r.ast(a, &vals).0;
        let mut parts = vec![];
        // a name alone: its formula, if it has one
        if let Ast::Name(n) = a {
            if let Some(Some(f)) = self.env.vars.get(n).map(|x| x.ast.clone()) {
                if !literal(&f) {
                    let ff = r.ast(&f, &|_| None).0;
                    let fs = r.ast(&f, &vals).0;
                    parts.push(part("=", ff.clone(), PartKind::Formula));
                    if fs != ff && fs != res {
                        parts.push(part("=", fs, PartKind::Sub));
                    }
                }
            }
            parts.push(part(rel, res, PartKind::Res));
            if lhs.is_empty() {
                return vec![stage(formula, parts)];
            }
            return vec![stage(lhs, parts)];
        }
        if !lhs.is_empty() {
            parts.push(part("=", formula.clone(), PartKind::Formula));
        }
        // a call of a function: its body with the arguments put in, P(5 V) = 5 V · I
        let mut sub = sub;
        if let Ast::Call(n, args) = a {
            if let Some(f) = self.env.funs.get(n) {
                if f.params.len() == args.len() {
                    let m: BTreeMap<String, Ast> = f.params.iter().map(|p| p.name.clone()).zip(args.iter().cloned()).collect();
                    let (bf, bs) = if let Some(ast) = &f.ast {
                        let body = subst_ast(ast, &m);
                        (r.ast(&body, &|_| None).0, r.ast(&body, &vals).0)
                    } else {
                        match self.lower(a, &[], false) {
                            Ok(body) => (r.expr(&body).0, self.r(cfg, false).expr(&self.env.bind_with_units(&body, &BTreeSet::new(), &self.units).unwrap_or(body.clone())).0),
                            Err(_) => (formula.clone(), formula.clone()),
                        }
                    };
                    if bf != formula && bf != res {
                        parts.push(part("=", bf.clone(), PartKind::Formula));
                    }
                    sub = bs;
                    if sub == bf {
                        sub = formula.clone();
                    }
                }
            }
        }
        if sub != formula && sub != res {
            parts.push(part("=", sub, PartKind::Sub));
        }
        if res != formula || lhs.is_empty() {
            parts.push(part(rel, res, PartKind::Res));
        }
        vec![stage(if lhs.is_empty() { formula } else { lhs }, parts)]
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
        let v = value(&s)?;
        let hint = self.hint_expr(e, &v.d);
        self.show_val(&v, unit, hint.as_ref(), cfg).map_err(NoValue::Error)
    }

    /// A symbolic result as markup: exact; a value once nothing unknown is left, in the unit asked for.
    fn sym_markup(&self, e: &Expr, unit: Option<&UnitExpr>, cfg: &Cfg) -> Result<String, String> {
        let s = simplify(e);
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
        if let (Some(u), Ok(v)) = (unit, value(&s)) {
            return Ok(self.show_val(&v, Some(u), None, cfg)?.0);
        }
        if cfg.form == "decimal" {
            if let Ok(v) = value(&s) {
                return Ok(self.show_val(&v, None, None, cfg)?.0);
            }
        }
        Ok(self.r(cfg, cfg.form != "decimal").expr(&s).0)
    }

    fn eval_expr(&mut self, a: &Ast, clause: &Clause, cfg: &Cfg) -> Result<Vec<Stage>, String> {
        let e = self.lower(a, &[], true)?;
        let src = self.r(cfg, false).ast(a, &|_| None).0;
        // a variable shown again: in the unit it was defined with, with its relation (R ≤ 166.7 Ω)
        let (unit, relv) = match a {
            Ast::Name(n) => self.env.vars.get(n).map(|x| (x.unit.clone(), x.rel)).unwrap_or((None, None)),
            _ => (None, None),
        };
        let want = clause.unit.clone().or(unit);
        let (m, exact) = self.eval_markup(&e, want.as_ref(), cfg).map_err(|e| self.no_value(e, &ast_text(a)))?;
        self.out.mark = Some("=".into());
        self.out.result = Some(m.clone());
        // code shows the relation of an inequality's boundary: R ≤ 166.7 Ω
        let m = match relv {
            Some(r) if r != Rel::Eq => format!("{} {m}", render::rel(r)),
            _ => m,
        };
        let _ = src;
        Ok(self.eval_stages(String::new(), a, m, exact, cfg, false))
    }

    fn sym_expr(&mut self, a: &Ast, clause: &Clause, cfg: &Cfg) -> Result<Vec<Stage>, String> {
        let e = if clause.substitute.is_empty() { self.lower(a, &[], true)? } else { self.substituted(a, &[], &clause.substitute)? };
        let m = self.sym_markup(&e, clause.unit.as_ref(), cfg)?;
        self.out.mark = Some("->".into());
        self.out.result = Some(m.clone());
        let f = self.r(cfg, false).ast(a, &|_| None).0;
        Ok(vec![stage(f, vec![part("->", m, PartKind::Res)])])
    }

    /// = states something: checked when everything is known; prose in math when not.
    fn statement(&mut self, a: &Ast, cfg: &Cfg) -> Result<Vec<Stage>, String> {
        let Ast::Cmp(first, rest) = a else {
            if self.mode == Mode::Math {
                self.out.prose = true;
                return Ok(vec![]);
            }
            return Err(format!("'{}' does nothing – x := … defines, a trailing = shows a value", ast_text(a)));
        };
        let mut sides = vec![(**first).clone()];
        sides.extend(rest.iter().map(|(_, x)| x.clone()));
        let mut vals = vec![];
        for s in &sides {
            let e = match self.lower(s, &[], true) {
                Ok(e) => e,
                Err(e) if self.mode == Mode::Code => return Err(e),
                Err(_) => {
                    self.out.prose = true;
                    return Ok(vec![]);
                }
            };
            match value(&e) {
                Ok(v) => vals.push(v),
                Err(NoValue::Unknown(ns)) => {
                    if self.mode == Mode::Math {
                        self.out.prose = true;
                        return Ok(vec![]);
                    }
                    let one = ns.len() == 1;
                    return Err(format!(
                        "{} in '{}' {} defined – define {} with :=, or solve for {}: {} -> solve({})",
                        ns.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", "),
                        ast_text(a),
                        if one { "isn't" } else { "aren't" },
                        if one { "it" } else { "them" },
                        if one { "it" } else { "them" },
                        ast_text(a),
                        ns[0]
                    ));
                }
                Err(NoValue::Error(m)) => return Err(m),
            }
        }
        // inequalities in math are prose unless solved
        if self.mode == Mode::Math && rest.iter().any(|(r, _)| *r != Rel::Eq) {
            self.out.prose = true;
            return Ok(vec![]);
        }
        for (i, (r, _)) in rest.iter().enumerate() {
            let (x, y) = (&vals[i], &vals[i + 1]);
            let ok = if x.d != y.d { *r == Rel::Ne } else { r.holds(x.v.cmp(y.v), close_rel(x.v, y.v)) };
            if !ok {
                let why = if x.d != y.d { "has another unit than" } else { "has another value than" };
                return Err(if *r == Rel::Eq {
                    format!("'{}' {why} '{}' – = states what's true; := defines", ast_text(&sides[i + 1]), ast_text(&sides[i]))
                } else {
                    format!("'{}' doesn't hold", ast_text(a))
                });
            }
        }
        let r = self.r(cfg, false);
        Ok(vec![stage(r.ast(a, &|_| None).0, vec![])])
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
        let mut eq_m = vec![];
        let mut eq_text = vec![];
        for s in stmts {
            let Stmt::Expr(a) = s else { return Err("put a solved definition on its own line".into()); };
            let body = self.lower(a, xs, false)?;
            let (bodies, split) = match body {
                Expr::Vector(v) => (v, false),
                e => match self.elementwise(&e, xs)? {
                    Some(v) => (v, true),
                    None => (vec![e], false),
                },
            };
            for body in bodies {
                let Expr::Cmp(v, rs) = body else {
                    return Err(format!("'{}' isn't an equation – solve(…) needs one with =, like x^2 = 4", ast_text(a)));
                };
                if rs.len() != 1 || v.len() != 2 || v.iter().any(|e| e.contains(&|e| matches!(e, Expr::Cmp(..)))) {
                    return Err("one relation per equation; a relation function already contains =".into());
                }
                let markup = if matches!(a, Ast::Cmp(..)) && !split { self.r(cfg, false).ast(a, &|_| None).0 } else {
                    format!("{} {} {}", self.r(cfg, false).expr(&v[0]).0, render::rel(rs[0]), self.r(cfg, false).expr(&v[1]).0)
                };
                eq_m.push(markup);
                eq_text.push(ast_text(a));
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
                    (d, ..) => d.unwrap_or_else(|| { let _ = i; Dim::ONE }),
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
                sols.push(s.iter().zip(&dims).map(|(v, d)| (None, Sol::Value(Val::new(solve::snap(*v), *d)))).collect());
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
            let Sol::Value(b) = &sols[0][0].1 else { return Err("the boundary of an inequality needs a value".into()) };
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
        self.out.solutions = sols.iter().map(|s| s.iter().map(|(_, sol)| match sol { Sol::Value(v) => v.v.f(), Sol::Symbolic(_) => f64::NAN }).collect()).collect();
        // which solution is kept: the only all-positive one, else the first
        let pick_i = {
            let pos: Vec<usize> = (0..sols.len())
                .filter(|&i| sols[i].iter().all(|(_, s)| matches!(s, Sol::Value(v) if v.v.f() > 0.0)))
                .collect();
            if pos.len() == 1 { Some(pos[0]) } else if sols.is_empty() { None } else { Some(0) }
        };
        if let Some(i) = pick_i {
            for (j, x) in xs.iter().enumerate() {
                if let Sol::Value(v) = &sols[i][j].1 {
                    let unit = if xs.len() == 1 { clause.unit.clone() } else { None };
                    let var = Var { expr: Expr::C(Val { u: None, raw: None, ..v.clone() }), ast: None, unit, rel };
                    self.env.define_var(x, var, Kind::Computed, self.listed);
                    self.out.defined.push(x.clone());
                }
            }
        }
        // what's shown
        let (stages, results) = {
        let rel_m = rel.map(render::rel).unwrap_or("=");
        let r = self.r(cfg, false);
        let vals = self.values(cfg);
        let rs = R { units: &self.units, cfg, exact: false, sub: Some(&vals) };
        let mut stages = vec![];
        stages.push(if eq_m.len() == 1 { stage(eq_m[0].clone(), vec![]) } else { stage(format!("cases({})", eq_m.join(", ")), vec![]) });
        let n = cfg.max_solutions.max(1);
        let mut results: Vec<String> = vec![];
        let mut segs = vec![];
        if sols.is_empty() {
            segs.push(Seg { lhs: "\"no real solution\"".into(), parts: vec![] });
            results.push("\"no real solution\"".into());
        }
        for (k, s) in sols.iter().enumerate().take(n) {
            let mut lines = vec![];
            for (j, x) in xs.iter().enumerate() {
                let lhs = if sols.len() > 1 && xs.len() == 1 { format!("{}_({})", name_base(x), k + 1) } else { name(x) };
                let (formula, sol) = &s[j];
                let unit = if xs.len() == 1 { clause.unit.as_ref() } else { None };
                let (res, exact) = match sol {
                    Sol::Value(v) => {
                        let hint = formula.as_ref().and_then(|f| self.hint_expr(f, &v.d));
                        self.show_val(v, unit, hint.as_ref(), cfg)?
                    }
                    Sol::Symbolic(e) => (self.r(cfg, true).expr(e).0, true),
                };
                let mut parts = vec![];
                if let (Some(f), Sol::Value(_)) = (formula, sol) {
                    let fm = r.expr(f).0;
                    let fsub = rs.expr(f).0;
                    if !matches!(f, Expr::C(_)) {
                        parts.push(part(rel_m, fm.clone(), PartKind::Formula));
                        if fsub != fm && fsub != res {
                            parts.push(part("=", fsub, PartKind::Sub));
                        }
                    }
                }
                let first = parts.is_empty();
                parts.push(part(if first { rel_m } else if exact { "=" } else { "approx" }, res.clone(), PartKind::Res));
                results.push(format!("{lhs} {rel_m} {res}"));
                lines.push(Seg { lhs, parts });
            }
            if xs.len() == 1 {
                segs.extend(lines);
            } else {
                let body: Vec<String> = lines.iter().map(|sg| format!("{} {}", sg.lhs, sg.parts.iter().map(|p| format!("{} {}", p.rel, p.rhs)).collect::<Vec<_>>().join(" "))).collect();
                segs.push(Seg { lhs: format!("cases({})", body.join(", ")), parts: vec![] });
            }
        }
        if sols.len() > n {
            segs.push(Seg { lhs: "dots.h".into(), parts: vec![] });
            results.push("dots.h".into());
        }
        let join = if xs.len() == 1 { ", quad " } else { " quad or quad " };
        stages.push(Stage { segs, join: join.into() });
        (stages, results)
        };
        self.out.mark = Some("->".into());
        self.out.result = Some(results.join(if xs.len() == 1 { ", quad " } else { ", " }));
        if self.mode == Mode::Code {
            self.out.show = Some(self.render_stages(&stages, cfg));
        }
        Ok(())
    }

    /// A solution's formula (names kept) → (formula, its value or symbolic result).
    fn finish(&self, f: Option<Expr>) -> Result<(Option<Expr>, Sol), String> {
        let Some(f) = f else { return Err("no solution".into()) };
        let b = simplify(&self.env.bind_with_units(&f, &BTreeSet::new(), &self.units)?);
        Ok(match value(&b) {
            Ok(v) => (Some(f), Sol::Value(v)),
            Err(NoValue::Unknown(_)) => (Some(f), Sol::Symbolic(b)),
            Err(NoValue::Error(e)) => return Err(e),
        })
    }

    fn hint_expr(&self, f: &Expr, d: &Dim) -> Option<UnitExpr> {
        let a = Ast::Mul(f.syms().into_iter().map(|s| (parse::MulOp::Implicit, Ast::Name(s))).collect());
        self.hint(&a, d)
    }

    // ---------- steps ----------

    fn render_stages(&self, stages: &[Stage], cfg: &Cfg) -> String {
        let st: Vec<Stage> = match cfg.steps.as_str() {
            "result" | "value" => {
                let l = stages.last().cloned().unwrap();
                vec![Stage {
                    segs: l
                        .segs
                        .into_iter()
                        .map(|s| {
                            let parts = s.parts.iter().rev().take(1).cloned().collect();
                            Seg { lhs: s.lhs, parts }
                        })
                        .collect(),
                    join: l.join,
                }]
            }
            "formula" => stages
                .iter()
                .map(|s| Stage { segs: s.segs.iter().map(|g| Seg { lhs: g.lhs.clone(), parts: g.parts.iter().filter(|p| p.kind != PartKind::Sub).cloned().collect() }).collect(), join: s.join.clone() })
                .collect(),
            _ => stages.to_vec(),
        };
        let value_only = cfg.steps == "value";
        if cfg.layout == "column" {
            let mut lines = vec![];
            for (i, s) in st.iter().enumerate() {
                for (j, g) in s.segs.iter().enumerate() {
                    let pre = if i > 0 && j == 0 { "#svimp #h(0.3em) " } else { "" };
                    if g.parts.is_empty() {
                        lines.push(format!("{pre}{} &", g.lhs));
                    }
                    for (k, p) in g.parts.iter().enumerate() {
                        lines.push(if k == 0 { format!("{pre}{} & {} {}", g.lhs, p.rel, p.rhs) } else { format!("& {} {}", p.rel, p.rhs) });
                    }
                }
            }
            return lines.join(" \\ ");
        }
        st.iter()
            .map(|s| {
                s.segs
                    .iter()
                    .map(|g| {
                        if value_only && !g.parts.is_empty() {
                            return g.parts.last().unwrap().rhs.clone();
                        }
                        let mut m = g.lhs.clone();
                        for p in &g.parts {
                            m += &format!(" {} {}", p.rel, p.rhs);
                        }
                        m
                    })
                    .collect::<Vec<_>>()
                    .join(&s.join)
            })
            .collect::<Vec<_>>()
            .join(" quad #svimp quad ")
    }
}

fn close_rel(a: Num, b: Num) -> bool {
    match (a, b) {
        (Num::R(x), Num::R(y)) => x == y,
        _ => close(a.f(), b.f()),
    }
}

fn name_base(x: &str) -> String {
    // x₁, x₂: the solution's index below the name (a name with an index of its own keeps it: x_a,1)
    match x.find('_') {
        Some(i) => format!("{}_(upright({}), ", name(&x[..i]), format::q(&x[i + 1..])).trim_end_matches(", ").to_string(),
        None => name(x),
    }
}

/// Names replaced by expressions as written: a function's body with its arguments.
fn subst_ast(a: &Ast, m: &BTreeMap<String, Ast>) -> Ast {
    let s = |x: &Ast| Box::new(subst_ast(x, m));
    match a {
        Ast::Name(n) => match m.get(n) {
            // a sum put into a product keeps its parentheses through the precedence of what's shown
            Some(x) => x.clone(),
            None => a.clone(),
        },
        Ast::Qty(x, u) => Ast::Qty(s(x), u.clone()),
        Ast::Neg(x) => Ast::Neg(s(x)),
        Ast::Abs(x) => Ast::Abs(s(x)),
        Ast::Cast(x, u) => Ast::Cast(s(x), u.clone()),
        Ast::Add(ts) => Ast::Add(ts.iter().map(|(n, t)| (*n, subst_ast(t, m))).collect()),
        Ast::Mul(fs) => Ast::Mul(fs.iter().map(|(o, f)| (*o, subst_ast(f, m))).collect()),
        Ast::Div(x, y) => Ast::Div(s(x), s(y)),
        Ast::Pow(x, y) => Ast::Pow(s(x), s(y)),
        Ast::Call(n, args) => Ast::Call(n.clone(), args.iter().map(|x| subst_ast(x, m)).collect()),
        Ast::Matrix(rs) => Ast::Matrix(rs.iter().map(|r| r.iter().map(|x| subst_ast(x, m)).collect()).collect()),
        Ast::Cmp(f, rest) => Ast::Cmp(s(f), rest.iter().map(|(r, x)| (*r, subst_ast(x, m))).collect()),
        _ => a.clone(),
    }
}

/// A vector or matrix from the markup of its elements.
fn array_markup(a: &Array, parts: Vec<String>) -> String {
    if a.cols == 1 { format!("vec({})", parts.join(", ")) } else { render::mat(parts.chunks(a.cols).map(<[String]>::to_vec).collect()) }
}

/// A value written out: 7.2 kΩ, -3, [5 V].
fn literal(a: &Ast) -> bool {
    match a {
        Ast::Num(..) | Ast::Qty(..) | Ast::Unit(..) => true,
        Ast::Neg(x) => literal(x),
        // vectors and matrices of values
        Ast::Matrix(rs) => rs.iter().flatten().all(literal),
        Ast::Call(n, args) if n == "vec" => args.iter().all(literal),
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
        Ast::Qty(x, _) | Ast::Neg(x) | Ast::Abs(x) | Ast::Cast(x, _) => ast_names(x, out),
        Ast::Add(ts) => ts.iter().for_each(|t| ast_names(&t.1, out)),
        Ast::Mul(fs) => fs.iter().for_each(|f| ast_names(&f.1, out)),
        Ast::Div(x, y) | Ast::Pow(x, y) => {
            ast_names(x, out);
            ast_names(y, out);
        }
        Ast::Call(n, args) if matches!(n.as_str(), "integrate" | "sum") && args.len() == 4 => {
            // The integration variable or sum index is local to the body, not the bounds.
            let mut body = vec![];
            ast_names(&args[0], &mut body);
            for n in body {
                if !matches!(&args[1], Ast::Name(var) if *var == n) && !out.contains(&n) {
                    out.push(n);
                }
            }
            for bound in &args[2..] { ast_names(bound, out); }
        }
        Ast::Call(_, args) => args.iter().for_each(|x| ast_names(x, out)),
        Ast::Matrix(rs) => rs.iter().flatten().for_each(|x| ast_names(x, out)),
        Ast::Cmp(f, rest) => {
            ast_names(f, out);
            rest.iter().for_each(|r| ast_names(&r.1, out));
        }
        _ => {}
    }
}

/// An expression as text, for messages.
pub fn ast_text(a: &Ast) -> String {
    fn p(a: &Ast, min: u8) -> String {
        let (s, prec) = t(a);
        if prec < min { format!("({s})") } else { s }
    }
    fn t(a: &Ast) -> (String, u8) {
        match a {
            Ast::Num(r, _) => (r.clone(), 9),
            Ast::Qty(n, u) => (format!("{} {}", p(n, 9), u.text()), 3),
            Ast::Unit(u) => (u.text(), 9),
            Ast::Name(n) => (n.clone(), 9),
            Ast::Neg(x) => (format!("-{}", p(x, 3)), 2),
            Ast::Add(ts) => (
                ts.iter().enumerate().map(|(i, (n, x))| format!("{}{}", if *n { if i == 0 { "-" } else { " - " } } else if i > 0 { " + " } else { "" }, p(x, 2))).collect(),
                1,
            ),
            Ast::Mul(fs) => (
                fs.iter().enumerate().map(|(i, (o, x))| format!("{}{}", if i == 0 { "" } else if *o == parse::MulOp::Implicit { " " } else { " * " }, p(x, 3))).collect(),
                3,
            ),
            Ast::Div(x, y) => (format!("{}/{}", p(x, 3), p(y, 4)), 3),
            Ast::Pow(x, y) => (format!("{}^{}", p(x, 9), p(y, 9)), 5),
            Ast::Call(n, args) => (format!("{n}({})", args.iter().map(|x| t(x).0).collect::<Vec<_>>().join(", ")), 9),
            Ast::Abs(x) => (format!("|{}|", t(x).0), 9),
            Ast::Matrix(rs) => (format!("mat({})", rs.iter().map(|r| r.iter().map(|x| t(x).0).collect::<Vec<_>>().join(", ")).collect::<Vec<_>>().join("; ")), 9),
            Ast::Cmp(f, rest) => {
                let mut s = p(f, 1);
                for (r, x) in rest {
                    s += &format!(" {} {}", r.text(), p(x, 1));
                }
                (s, 0)
            }
            Ast::Cast(x, u) => (format!("{} as {}", p(x, 5), u.text()), 3),
        }
    }
    t(a).0
}

#[allow(dead_code)]
fn unused(_: Shown) {}

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
        fn math(&mut self, s: &str) -> Out {
            Sheet::new(&mut self.env, &self.cfg, Mode::Math, true).unwrap().run(s)
        }
        fn code(&mut self, s: &str) -> Out {
            Sheet::new(&mut self.env, &self.cfg, Mode::Code, true).unwrap().run(s)
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
        let o = t.math("f(y_2: A, ..) = q(x_2, y_2)");
        assert!(o.errors.is_empty(), "{:?}", o.errors);
        let o = t.ok("M ≔ 1 = 2 =");
        assert_eq!(o.result.as_deref(), Some("\"0\""));
        let o = t.ok("M ≔ 1 A = 2 V →");
        assert_eq!(o.result.as_deref(), Some("\"0\""));
        let o = t.ok("M =");
        assert_eq!(o.result.as_deref(), Some("\"0\""));
    }

    #[test]
    fn substitution() {
        let mut t = T::new();
        t.ok("h ≔ 5 z");
        t.ok("z ≔ 20 ⦃cm⦄");
        let o = t.ok("U(x: m) ≔ h x a →");
        assert_eq!(o.result.as_deref(), Some("\"1\" thin upright(\"m\") dot a x"));
        let o = t.ok("h(x: m) ≔ frac(U(x), a) →");
        assert_eq!(o.result.as_deref(), Some("\"1\" thin upright(\"m\") dot x"));
        let o = t.ok("y ≔ h(2) =");
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
        t.ok("Q_0 ≔ 3450 ⦃mAh⦄");
        t.ok("I ≔ 30 ⦃mA⦄");
        let o = t.ok("t ≔ frac(Q_0, I) = 1 h");
        assert_eq!(o.result.as_deref(), Some("\"115\" thin upright(\"h\")"));
        let o = t.ok("frac(Q_0, 2) = I t → ⦅solve(t)⦆");
        assert_eq!(o.result.as_deref(), Some("t = \"57.5\" thin upright(\"h\")"));
        assert!((t.val("t") - 207000.0).abs() < 1e-6);
        let o = t.ok("x^(2) + 2x - 3 = 0 → ⦅solve(x)⦆");
        assert!(o.result.unwrap().contains("x_(1) = \"−3\""));
        assert_eq!(t.val("x"), 1.0);
        t.ok("U ≔ 5 V");
        let o = t.ok("R ≤ frac(U, I) → ⦅solve(R)⦆");
        assert!(o.result.unwrap().starts_with("R <= \"166.7\""));
        let o = t.ok("c ≔ frac(1, 3) → ⦅frac⦆");
        assert_eq!(o.result.as_deref(), Some("frac(\"1\", \"3\")"));
        let o = t.ok("a + b = 3, a - b = 1 → ⦅solve(a, b)⦆");
        assert!(o.errors.is_empty());
        assert_eq!(t.val("a"), 2.0);
        // a check that fails
        let o = t.math("U = 6 V");
        assert!(o.errors[0].contains("another value"));
        // prose
        assert!(t.math("E = m c^(2)").prose);
        let o = t.ok("cos(w) = w → ⦅solve(w)⦆");
        assert!(o.result.unwrap().contains("0.7391"));
    }

    #[test]
    fn code_steps() {
        let mut t = T::new();
        t.code("Q0 := 3450 mAh; I := 30 mA");
        let o = t.code("t := Q0/I = [h], digits(3)");
        assert!(o.errors.is_empty(), "{:?}", o.errors);
        let s = o.show.unwrap();
        assert!(s.contains("frac(upright(\"Q0\"), I)"), "{s}");
        assert!(s.contains("\"115\" thin upright(\"h\")"), "{s}");
        let o = t.code("y := 3 h");
        assert!(o.errors.is_empty());
        let o = t.code("U(x: A) := x [2 Ω]");
        assert!(o.errors.is_empty(), "{:?}", o.errors);
        let o = t.code("z := U(3) =");
        assert!(o.errors.is_empty(), "{:?}", o.errors);
        assert_eq!(t.val("z"), 6.0);
        let o = t.code("w := U([2 V])");
        assert!(o.errors[0].contains("takes x in A"), "{:?}", o.errors);
    }
}
