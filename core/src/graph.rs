//! Graphs: what's plotted against what, in which units, over which range, and where the interesting points are.
//! The Typst side draws what this gives with simple-plot.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::dim::Dim;
use crate::env::{Env, Lower};
use crate::eval::{dim_of, eval_f, value};
use crate::expr::{self, Expr, simp};
use crate::format::{self, Cfg, dim_text, name, num_markup, number};
use crate::num::Num;
use crate::parse::{self, Ctx, Mode, ParamType};
use crate::render::R;
use crate::solve::{infer_dims, scan};
use crate::units::{UnitExpr, Units};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Src {
    pub src: String,
    pub mode: Mode,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub operators: Vec<String>,
}

/// A value for a range or a mark: a plain number (in the axis unit) or an expression.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum XV {
    Num(f64),
    Src(Src),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Mark {
    /// "auto", "crossings", "roots", "extrema"
    Key(String),
    Val(XV),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphReq {
    pub cfg: Cfg,
    pub curves: Vec<Src>,
    pub xrange: Vec<XV>,
    pub yrange: Vec<XV>,
    /// (mark, y labelled): mark writes x and y on the axes, xmark only x.
    pub marks: Vec<(Mark, bool)>,
    pub ymark: Vec<XV>,
    pub samples: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Series {
    pub points: Vec<(f64, f64)>,
    pub curve: usize,
    pub label: Option<String>,
    pub label_pos: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    /// "xy": both guide lines, "x": the vertical one, "": none.
    pub guides: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GraphOut {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
    pub xticks: Vec<f64>,
    pub xlabels: Vec<String>,
    pub yticks: Vec<f64>,
    pub ylabels: Vec<String>,
    pub xlabel: String,
    pub ylabel: String,
    /// Where the axes cross: 0 when it's in range.
    pub x_axis_zero: bool,
    pub y_axis_zero: bool,
    pub series: Vec<Series>,
    pub points: Vec<Point>,
    /// The axis lines' positions for the guides (display units).
    pub ax: f64,
    pub ay: f64,
}

fn nice(r: f64) -> f64 {
    let b = 10f64.powf(r.log10().floor());
    for m in [1.0, 2.0, 5.0, 10.0] {
        if m * b >= r * (1.0 - 1e-9) {
            return m * b;
        }
    }
    10.0 * b
}

struct Curve {
    e: Expr,
    label: String,
    name: Option<String>,
    /// The function a single call is to (for the unit the result is typed with).
    call: Option<String>,
    /// The parameters written in its head.
    params: Vec<String>,
}

/// An x or y value: (SI value, dims, unit text, source) or a plain number.
enum V {
    Plain(f64),
    Q(f64, Dim, String, String),
}

/// The curves read: what each plots, what they're plotted against, and the ones written with `:=`.
struct Read {
    curves: Vec<Curve>,
    x: String,
    xunit_typed: Option<UnitExpr>,
    defines: Vec<Src>,
    eq: String,
}

fn branches(e: &Expr, vars: &dyn Fn(&str) -> Option<f64>, selected: &mut Vec<usize>) {
    if let Expr::Cases(v) = e {
        let branch = v.iter().position(|(_, condition)| eval_f(condition, vars).is_some_and(|v| v != 0.0));
        selected.push(branch.unwrap_or(usize::MAX));
        if let Some(i) = branch { branches(&v[i].0, vars, selected); }
    } else {
        for child in e.children() { branches(child, vars, selected); }
    }
}

fn case_boundaries(e: &Expr, x: &str, scale: f64, points: &mut Vec<f64>) {
    fn conditions(e: &Expr, x: &str, scale: f64, points: &mut Vec<f64>) {
        if let Expr::Cmp(values, _) = e {
            for pair in values.windows(2) { points.extend(scan(&expr::sub(pair[0].clone(), pair[1].clone()), x, scale)); }
        }
        for child in e.children() { conditions(child, x, scale, points); }
    }
    if let Expr::Cases(v) = e {
        for (_, condition) in v { conditions(condition, x, scale, points); }
    }
    for child in e.children() { case_boundaries(child, x, scale, points); }
}

fn read(env: &Env, req: &GraphReq, units: &Units) -> Result<Read, String> {
    let cfg = &req.cfg;
    let is_fn = |n: &str| env.funs.contains_key(n);
    let is_var = |n: &str| env.vars.contains_key(n);
    let r = R { units, cfg, exact: false, sub: None };
    let tidy = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ").replace("( ", "(").replace(" )", ")");
    let mut curves = vec![];
    let mut takes: Vec<String> = vec![];
    let mut xunit_typed: Option<UnitExpr> = None;
    let mut defines = vec![];
    for c in &req.curves {
        let ctx = Ctx { operators: &[], units, mode: c.mode, is_fn: &is_fn, is_var: &is_var, strict: cfg.strict };
        let cv = match parse::parse_curve(&c.src, &ctx) {
            Ok(cv) => cv,
            Err(e) if e.starts_with("NAMED:") => {
                let n = &e[6..];
                return Err(format!("a named curve is a function of what it's plotted against: #graph(\"{n}(..) = …\"), where .. takes the one variable that isn't defined"));
            }
            Err(e) if e.starts_with("EQUATION:") => {
                let (l, rr) = e[9..].split_once('|').unwrap_or(("", ""));
                return Err(format!(
                    "a curve is an expression or a function like U(x) = …; for where both sides of '{}' meet, plot them as two curves: #graph(\"{}\", \"{}\")",
                    tidy(&c.src),
                    tidy(l),
                    tidy(rr)
                ));
            }
            Err(e) => return Err(e),
        };
        // the free names: what the curve is plotted against
        let mut lo = Lower::new(env, units);
        let mut written: Vec<String> = vec![];
        if let Some((_, ps, _)) = &cv.head {
            for p in ps {
                lo.params.insert(p.name.clone());
                written.push(p.name.clone());
                if let Some(ParamType::Unit(u)) = &p.ty {
                    xunit_typed.get_or_insert(u.clone());
                }
                if let Some(t) = &p.ty {
                    lo.inferred.insert(p.name.clone(), t.clone());
                }
            }
        }
        let e0 = lo.expr(&cv.body)?;
        let mut free = e0.syms();
        // names written but not defined vary too, also where they cancel: X(x) with X(x) := 20 is a constant line
        let mut names = vec![];
        crate::sheet::ast_names(&cv.body, &mut names);
        for n in names {
            if !free.contains(&n) && !is_var(&n) && !is_fn(&n) && !matches!(n.as_str(), "π" | "pi" | "e") {
                free.push(n);
            }
        }
        // and the parameters written out, whatever the name means outside: y(y) := X(y) with y defined
        for p in written.iter().rev() {
            if !free.contains(p) {
                free.insert(0, p.clone());
            }
        }
        // a function with its parameters written out may use only them and defined names, as any function
        if let Some((n, _, false)) = &cv.head
            && let Some(f) = free.iter().find(|f| !written.contains(f)) {
                return Err(format!("'{f}' in '{n}' isn't defined – define it first with :=, or plot against it: {n}({f}) or {n}(..)"));
            }
        // read again with every free name as a parameter: the units they get from the functions they're passed to
        let mut lo2 = Lower::new(env, units);
        lo2.params = free.iter().cloned().collect();
        lo2.inferred = lo.inferred.clone();
        let e = simp(&lo2.expr(&cv.body)?);
        for (n, t) in &lo2.inferred {
            if let ParamType::Unit(u) = t {
                if free.contains(n) {
                    xunit_typed.get_or_insert(u.clone());
                }
            }
        }
        for f in &free {
            if !takes.contains(f) {
                takes.push(f.clone());
            }
        }
        let call = match &cv.body {
            parse::Ast::Call(n, _) => Some(n.clone()),
            _ => None,
        };
        let display = c.label.as_ref().map(|label| parse::parse_curve(label, &ctx)).transpose()?;
        let label = r.ast_ops(display.as_ref().map_or(&cv.body, |cv| &cv.body), &|_| None, &c.operators).0;
        if cv.defines {
            defines.push(c.clone());
        }
        let branches = match e { Expr::Vector(v) => v, e => vec![e] };
        let count = branches.len();
        if count == 0 { return Err("an empty vector has no curves to plot".into()); }
        for (i, e) in branches.into_iter().enumerate() {
            if matches!(e, Expr::Vector(_)) { return Err("a graph needs scalar components – select a solution with at(vector, index)".into()); }
            curves.push(Curve {
                e, label: if count == 1 { label.clone() } else { format!("{}_({})", label, i + 1) },
                name: cv.head.as_ref().map(|h| h.0.clone()), call: call.clone(), params: written.clone(),
            });
        }
    }
    let eq = req.curves.iter().map(|c| tidy(&c.src)).collect::<Vec<_>>().join("', '");
    if takes.len() > 1 {
        return Err(format!(
            "a graph has one x axis, but {} vary in '{eq}' – define all but one",
            takes.iter().map(|n| format!("'{n}'")).collect::<Vec<_>>().join(", ")
        ));
    }
    let Some(x) = takes.first().cloned() else { return Err(format!("nothing varies in '{eq}' – a curve takes what it's plotted against: U(x) = …")) };
    // y(y) := … names its parameter so on purpose
    if curves.iter().any(|c| c.name.as_deref() == Some(x.as_str()) && !c.params.contains(&x)) {
        return Err(format!("'{x}' is what the curve is plotted against, so it can't name it"));
    }
    Ok(Read { curves, x, xunit_typed, defines, eq })
}

/// The curves written with `:=`, if the graph can read its curves: what calculating them as lines defines.
pub fn defines(env: &Env, req: &GraphReq) -> Result<Vec<Src>, String> {
    let units = Units::build(&req.cfg.units)?;
    Ok(read(env, req, &units)?.defines)
}

pub fn graph(env: &Env, req: &GraphReq) -> Result<GraphOut, String> {
    let units = Units::build(&req.cfg.units)?;
    let cfg = &req.cfg;
    if req.curves.is_empty() {
        return Err("nothing to plot – pass an equation like \"P = U^2/R\"".into());
    }
    if req.samples < 2 {
        return Err("samples: an integer ≥ 2".into());
    }
    let Read { curves, x, xunit_typed, eq, .. } = read(env, req, &units)?;
    let tidy = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ").replace("( ", "(").replace(" )", ")");
    // values of ranges and marks
    let xv = |v: &XV| -> Result<V, String> {
        match v {
            XV::Num(n) => Ok(V::Plain(*n)),
            XV::Src(s) => {
                let t = s.src.trim();
                if let Ok(n) = t.replace(',', ".").replace('−', "-").parse::<f64>() {
                    return Ok(V::Plain(n));
                }
                let (val, d) = crate::api::quantity(env, cfg, t, s.mode).map_err(|e| format!("{e} in '{t}'"))?;
                let a = units.auto(&d);
                let u = units.prefixed_for(val / units.factor(&a).f(), &a, &cfg.prefixes);
                Ok(V::Q(val, d, if d.is_one() { String::new() } else { u.text() }, tidy(t)))
            }
        }
    };
    let rvs: Vec<V> = req.xrange.iter().map(xv).collect::<Result<_, _>>()?;
    let mvs: Vec<(V, bool)> = req
        .marks
        .iter()
        .filter_map(|(m, ly)| match m {
            Mark::Val(v) => Some(xv(v).map(|x| (x, *ly))),
            _ => None,
        })
        .collect::<Result<_, _>>()?;
    let yrv: Vec<V> = req.yrange.iter().map(xv).collect::<Result<_, _>>()?;
    let ymv: Vec<V> = req.ymark.iter().map(xv).collect::<Result<_, _>>()?;
    // x's unit: given with it (U(x: A)), by a function it's passed to (q(I: mA)), else from the range and marks
    let withu: Vec<(Dim, String)> = rvs
        .iter()
        .chain(mvs.iter().map(|m| &m.0))
        .filter_map(|v| match v {
            V::Q(_, d, u, _) if !d.is_one() => Some((*d, u.clone())),
            _ => None,
        })
        .collect();
    let mut xunit = match (&xunit_typed, withu.first()) {
        (Some(u), _) => u.clone(),
        (None, Some((_, u))) => units.parse(u, false).map(|p| p.u).unwrap_or_default(),
        _ => UnitExpr::default(),
    };
    let mut xd = units.dim(&xunit).unwrap_or(Dim::ONE);
    // no unit for x yet, but y values have one: x's unit follows from the curve (P = U0²/R2 with P0 in W: R2 in Ω)
    let yu = yrv.iter().chain(ymv.iter()).find_map(|v| match v {
        V::Q(_, d, ..) if !d.is_one() => Some(*d),
        _ => None,
    });
    if xunit.is_empty() && withu.is_empty() {
        if let Some(yd) = yu {
            let known = |n: &str| env.vars.get(n).and_then(|v| value(&env.bind_with_units(&v.expr, &BTreeSet::new(), &units).ok()?).ok()).map(|v| v.d);
            let y = Expr::C(crate::expr::Val::new(Num::ONE, yd));
            if let Ok(ds) = infer_dims(&[(curves[0].e.clone(), y)], std::slice::from_ref(&x), &known) {
                if let Some(Some(d)) = ds.first() {
                    if !d.is_one() {
                        xd = *d;
                        let a = units.auto(d);
                        xunit = a;
                    }
                }
            }
        }
    }
    let xf = units.factor(&xunit).f();
    let misfit = |v: &V, axis: &str, u: &str| -> String {
        let V::Q(val, d, vu, src) = v else { return String::new() };
        let shown = if d.is_one() {
            format!("{} (no unit)", (val * 1e4).round() / 1e4)
        } else {
            let f = units.parse(vu, false).map(|p| p.f.f()).unwrap_or(1.0);
            format!("{} {vu}", ((val / f) * 1e4).round() / 1e4)
        };
        format!("'{src}' is {shown}, but {axis}{} – plain numbers count in the axis unit", if u.is_empty() { " has no unit".to_string() } else { format!(" is in {u}") })
    };
    if let Some(bad) = rvs.iter().chain(mvs.iter().map(|m| &m.0)).find(|v| matches!(v, V::Q(_, d, ..) if *d != xd)) {
        return Err(misfit(bad, &x, &xunit.text()));
    }
    let si = |v: &V, f: f64| match v {
        V::Plain(n) => n * f,
        V::Q(val, ..) => *val,
    };
    let rv: Vec<f64> = rvs.iter().map(|v| si(v, xf)).collect();
    let mv: Vec<(f64, bool)> = mvs.iter().map(|(v, ly)| (si(v, xf), *ly)).collect();
    // y's unit from the curves
    let known = |n: &str| -> Option<Dim> {
        if n == x {
            return Some(xd);
        }
        env.vars.get(n).and_then(|v| value(&env.bind_with_units(&v.expr, &BTreeSet::new(), &units).ok()?).ok()).map(|v| v.d)
    };
    let yd = dim_of(&curves[0].e, &known)?.unwrap_or(Dim::ONE);
    for c in &curves[1..] {
        if let Ok(Some(d)) = dim_of(&c.e, &known) {
            if d != yd && !yd.is_one() {
                return Err(format!("the curves have different units ({} and {}) – plot them separately", units.auto(&yd).text(), if d.is_one() { "1".into() } else { units.auto(&d).text() }));
            }
        }
    }
    let named: Vec<String> = curves.iter().filter_map(|c| c.name.clone()).collect();
    let yname = if named.len() == curves.len() { Some(named.clone()) } else { None };
    let yaxis = match &yname {
        Some(ns) => {
            let mut d = ns.clone();
            d.dedup();
            d.join(", ")
        }
        None => "y".into(),
    };
    if let Some(bad) = yrv.iter().chain(ymv.iter()).find(|v| matches!(v, V::Q(_, d, ..) if *d != yd)) {
        return Err(misfit(bad, &yaxis, &if yd.is_one() { String::new() } else { units.auto(&yd).text() }));
    }
    let yb = units.factor(&units.auto(&yd)).f();
    let yrv: Vec<f64> = yrv.iter().map(|v| si(v, yb)).collect();
    let ymv: Vec<f64> = ymv.iter().map(|v| si(v, yb)).collect();
    let f = |c: &Curve, t: f64| eval_f(&c.e, &|n| (n == x).then_some(t));
    // where the curves reach the y marks: the nearest ones must be visible
    let mut yneed = vec![];
    for y in &ymv {
        for c in &curves {
            let mut rs = scan(&simp(&expr::sub(c.e.clone(), Expr::C(crate::expr::Val::new(Num::F(*y), Dim::ONE)))), &x, xf);
            rs.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
            if let Some(first) = rs.first().copied() {
                yneed.extend(rs.into_iter().filter(|t| t.abs() <= 2.0 * first.abs() + 1e-9 * xf).take(4));
            }
        }
    }
    let mut boundaries = vec![];
    for c in &curves { case_boundaries(&c.e, &x, xf, &mut boundaries); }
    boundaries.retain(|point| point.is_finite());
    boundaries.sort_by(f64::total_cmp);
    boundaries.dedup();
    let mut need: Vec<f64> = mv.iter().map(|m| m.0).chain(yneed).collect();
    if rv.is_empty() { need.extend(&boundaries); }
    let mut dedup_rv = rv.clone();
    dedup_rv.sort_by(|a, b| a.total_cmp(b));
    dedup_rv.dedup();
    let (mut xa, mut xb) = if dedup_rv.len() >= 2 {
        (dedup_rv[0], *dedup_rv.last().unwrap())
    } else if !need.is_empty() {
        // the marks are what's interesting: around them, not back to 0
        let lo = need.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = need.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let (a, b) = if lo == hi {
            let w = if lo == 0.0 { xf } else { lo.abs() / 2.0 };
            (lo - w, lo + w)
        } else {
            let p = (hi - lo) * 0.25;
            (lo - p, hi + p)
        };
        let st = nice((b - a) / 5.0);
        ((a / st + 1e-9).floor() * st, (b / st - 1e-9).ceil() * st)
    } else {
        autorange(&curves, &x, xf)
    };
    // a single range value and marked values widen the range; marks get a little room
    for v in &rv {
        xa = xa.min(*v);
        xb = xb.max(*v);
    }
    let w = xb - xa;
    for v in &need {
        if *v < xa {
            xa = v - 0.05 * w;
        } else if *v > xb {
            xb = v + 0.05 * w;
        }
    }
    if !(xa < xb) {
        return Err("the range is empty – give two different x values".into());
    }
    let n = req.samples;
    let mut ts: Vec<f64> = (0..=n).map(|i| xa + i as f64 * (xb - xa) / n as f64).collect();
    if !boundaries.is_empty() {
        let mut edges = vec![xa];
        edges.extend(boundaries.iter().copied().filter(|point| *point > xa && *point < xb));
        edges.push(xb);
        ts.extend(&edges);
        ts.extend(edges.windows(2).map(|pair| pair[0] + (pair[1] - pair[0]) / 2.0));
        ts.sort_by(f64::total_cmp);
        ts.dedup();
    }
    let ys: Vec<Vec<Option<f64>>> = curves.iter().map(|c| ts.iter().map(|&t| f(c, t)).collect()).collect();
    let mut all: Vec<f64> = ys.iter().flatten().filter_map(|y| *y).collect();
    all.sort_by(|a, b| a.total_cmp(b));
    if all.is_empty() {
        return Err(format!("'{eq}' can't be evaluated in this range"));
    }
    let (mut lo, mut hi) = (all[0], *all.last().unwrap());
    let (q1, q2) = (all[(all.len() as f64 * 0.05) as usize], all[((all.len() as f64 * 0.95) as usize).min(all.len() - 1)]);
    if hi - lo > 20.0 * (q2 - q1) && q2 > q1 {
        (lo, hi) = (q1 - (q2 - q1) / 2.0, q2 + (q2 - q1) / 2.0); // poles
    }
    let mut dedup_y = yrv.clone();
    dedup_y.sort_by(|a, b| a.total_cmp(b));
    dedup_y.dedup();
    // y marks are what's interesting: the y range is around them
    if !ymv.is_empty() && dedup_y.len() < 2 {
        let vs: Vec<f64> = ymv.iter().chain(yrv.iter()).copied().collect();
        lo = vs.iter().copied().fold(f64::INFINITY, f64::min);
        hi = vs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if lo == hi {
            let w = if lo == 0.0 { yb } else { lo.abs() };
            (lo, hi) = (lo - w, lo + w);
        } else {
            let p = (hi - lo) * 0.25;
            (lo, hi) = (lo - p, hi + p);
        }
    }
    // a function's result in the type it returns (A², not W from V·A)
    let ret = if curves.len() == 1 { curves[0].call.as_ref().and_then(|c| env.funs.get(c)).and_then(|fun| fun.ret) } else { None };
    let ybase = match ret {
        Some(d) if d == yd => units.auto(&d),
        _ => units.auto(&yd),
    };
    let yunit = units.prefixed_for(lo.abs().max(hi.abs()) / units.factor(&ybase).f(), &ybase, &cfg.prefixes);
    let yf = units.factor(&yunit).f();
    let (x0, x1) = (xa / xf, xb / xf);
    let (mut y0, mut y1) = if dedup_y.len() >= 2 {
        (dedup_y[0] / yf, dedup_y.last().unwrap() / yf)
    } else {
        for v in yrv.iter().chain(ymv.iter()) {
            lo = lo.min(*v);
            hi = hi.max(*v);
        }
        let (mut y0, mut y1) = (lo / yf, hi / yf);
        if y0 > 0.0 && y0 < 0.5 * y1 {
            y0 = 0.0;
        }
        if y1 < 0.0 && y1 > 0.5 * y0 {
            y1 = 0.0;
        }
        if y0 == y1 {
            (y0, y1) = (y0 - 1.0, y1 + 1.0);
        }
        let pad = (y1 - y0) * 0.08;
        (if y0 == 0.0 { 0.0 } else { y0 - pad }, if y1 == 0.0 { 0.0 } else { y1 + pad })
    };
    // marked y values beyond an exact y range still show
    for v in &ymv {
        let w = y1 - y0;
        if v / yf < y0 {
            y0 = v / yf - 0.05 * w;
        } else if v / yf > y1 {
            y1 = v / yf + 0.05 * w;
        }
    }
    let (xs_, ys_) = (nice((x1 - x0) / 5.0), nice((y1 - y0) / 5.0));
    let xc = x0 <= 0.0 && 0.0 <= x1;
    let yc = y0 <= 0.0 && 0.0 <= y1;
    let ax = if yc { 0.0 } else { y0 };
    let ay = if xc { 0.0 } else { x0 };
    let ticks = |a: f64, b: f64, st: f64, skip0: bool| -> (Vec<f64>, Vec<String>) {
        let mut ps = vec![];
        let mut v = (a / st - 1e-9).ceil() * st;
        while v <= b + st * 1e-9 {
            if !(skip0 && v.abs() < st * 1e-9) {
                ps.push((v * 1e12).round() / 1e12);
            }
            v += st;
        }
        let ls = ps.iter().map(|p| num_markup(&fixed10(*p), None, cfg)).collect();
        (ps, ls)
    };
    let mut xt = ticks(x0, x1, xs_, xc && yc);
    let mut yt = ticks(y0, y1, ys_, xc && yc);
    // the samples, split at gaps and poles; a label where a curve is farthest from the others
    let mut series = vec![];
    let mut used: Vec<(f64, f64)> = vec![];
    let names: Vec<Option<String>> = curves.iter().map(|c| c.name.clone()).collect();
    for (i, c) in curves.iter().enumerate() {
        let label = match &c.name {
            None => c.label.clone(),
            Some(nm) if names.iter().filter(|m| m.as_ref() == Some(nm)).count() > 1 => format!("{} = {}", name(nm), c.label),
            Some(nm) => name(nm),
        };
        let mut runs: Vec<Vec<(f64, f64, usize)>> = vec![vec![]];
        let piecewise = c.e.contains(&|e| matches!(e, Expr::Cases(_)));
        let branch = |t: f64| {
            let mut selected = vec![];
            branches(&c.e, &|n| (n == x).then_some(t), &mut selected);
            selected
        };
        for (j, &t) in ts.iter().enumerate() {
            let Some(v) = ys[i][j] else {
                if !runs.last().unwrap().is_empty() {
                    runs.push(vec![]);
                }
                continue;
            };
            let p = (t / xf, v / yf, j);
            if let Some(q) = runs.last().unwrap().last().copied() {
                if piecewise && branch(q.0 * xf) != branch(t) {
                    let (mut a, mut b) = (q.0 * xf, t);
                    let before = branch(a);
                    for _ in 0..40 {
                        let mid = a + (b - a) / 2.0;
                        if mid == a || mid == b { break; }
                        if branch(mid) == before { a = mid; } else { b = mid; }
                    }
                    if let Some(y) = f(c, a) { runs.last_mut().unwrap().push((a / xf, y / yf, q.2)); }
                    runs.push(vec![]);
                    if let Some(y) = f(c, b) { runs.last_mut().unwrap().push((b / xf, y / yf, j)); }
                }
                if (q.1 > 0.0) != (p.1 > 0.0) && (q.1 - p.1).abs() > 2.0 * (y1 - y0) {
                    runs.push(vec![]);
                }
            }
            runs.last_mut().unwrap().push(p);
        }
        runs.retain(|r| r.len() > 1);
        let longest = runs.iter().map(|r| r.len()).max().unwrap_or(0);
        for run in runs {
            let lab = curves.len() > 1 && run.len() == longest;
            let mut lp = 0.8;
            if lab {
                let (mut best, mut bd) = (None, -1.0);
                for (k, &(px, py, j)) in run.iter().enumerate().rev() {
                    if (k as f64) < run.len() as f64 * 0.15 || (k as f64) > run.len() as f64 * 0.95 || py < y0 || py > y1 {
                        continue;
                    }
                    if used.iter().any(|&(ux, uy)| (ux - px).abs() < 0.2 * (x1 - x0) && (uy - py).abs() < 0.15 * (y1 - y0)) {
                        continue;
                    }
                    let dd = (0..curves.len())
                        .filter(|&o| o != i)
                        .map(|o| match ys[o][j] {
                            None => 1.0,
                            Some(w) => (w / yf - py).abs() / (y1 - y0),
                        })
                        .fold(1.0f64, f64::min);
                    if dd > bd + 0.02 {
                        best = Some(k);
                        bd = dd;
                    }
                }
                if let Some(b) = best {
                    lp = b as f64 / (run.len() - 1) as f64;
                    used.push((run[b].0, run[b].1));
                }
            }
            series.push(Series { points: run.iter().map(|p| (p.0, p.1)).collect(), curve: i, label: lab.then(|| label.clone()), label_pos: lp });
        }
    }
    // marks
    let zeros = |h: &dyn Fn(f64) -> Option<f64>| -> Vec<f64> {
        let vs: Vec<Option<f64>> = ts.iter().map(|&t| h(t)).collect();
        let mut out = vec![];
        for i in 0..n {
            let (Some(u), Some(w)) = (vs[i], vs[i + 1]) else { continue };
            if u == 0.0 {
                out.push(ts[i]);
            } else if w != 0.0 && (u > 0.0) != (w > 0.0) {
                out.push(bisect(h, ts[i], ts[i + 1]));
            }
        }
        if vs[n] == Some(0.0) {
            out.push(ts[n]);
        }
        out
    };
    // (x, y, guides, x labelled, y labelled) in SI
    let mut pts: Vec<(f64, Option<f64>, &str, bool, bool)> = vec![];
    let crossings = |pairs: &[(usize, usize)], ly: bool| -> Vec<(f64, Option<f64>, &'static str, bool, bool)> {
        let mut out = vec![];
        for &(a, b) in pairs {
            let h = |t: f64| Some(f(&curves[a], t)? - f(&curves[b], t)?);
            for t in zeros(&h) {
                out.push((t, f(&curves[a], t), if ly { "xy" } else { "x" }, true, ly));
            }
        }
        out
    };
    let pairs = if curves.len() == 2 { vec![(0, 1)] } else { vec![] };
    for (m, ly) in &req.marks {
        if let Mark::Key(k) = m {
            match k.as_str() {
                "auto" => pts.extend(crossings(&pairs, *ly)),
                "crossings" => {
                    let all: Vec<(usize, usize)> = (0..curves.len()).flat_map(|i| (i + 1..curves.len()).map(move |j| (i, j))).collect();
                    pts.extend(crossings(&all, *ly));
                }
                "roots" => {
                    for c in &curves {
                        for t in zeros(&|t| f(c, t)) {
                            pts.push((t, Some(0.0), "", false, false));
                        }
                    }
                }
                "extrema" => {
                    for c in &curves {
                        let d = c.e.diff(&x);
                        for t in zeros(&|t| eval_f(&d, &|nn| (nn == x).then_some(t))) {
                            pts.push((t, f(c, t), "x", false, false));
                        }
                    }
                }
                _ => return Err(format!("mark: unknown '{k}' – \"crossings\", \"roots\", \"extrema\" or auto")),
            }
        }
    }
    for (t, ly) in &mv {
        for c in &curves {
            pts.push((*t, f(c, *t), if *ly || curves.len() == 1 { "xy" } else { "x" }, true, *ly));
        }
    }
    for y in &ymv {
        for c in &curves {
            for t in zeros(&|t| Some(f(c, t)? - y)) {
                pts.push((t, Some(*y), "xy", false, true));
            }
        }
    }
    let mut seen: Vec<(f64, f64)> = vec![];
    let (mut labx, mut laby): (Vec<f64>, Vec<f64>) = (vec![], vec![]);
    let mut points = vec![];
    for (t, v, gl, lx, ly) in pts {
        let px = t / xf;
        if lx && px >= x0 && px <= x1 && !labx.iter().any(|a| (a - px).abs() <= 1e-9 * (x1 - x0)) {
            labx.push(px);
        }
        let Some(v) = v else { continue };
        let py = v / yf;
        if ly && py >= y0 && py <= y1 && px >= x0 && px <= x1 && !laby.iter().any(|a| (a - py).abs() <= 1e-9 * (y1 - y0)) {
            laby.push(py);
        }
        if px < x0 || px > x1 || py < y0 || py > y1 || seen.iter().any(|&(a, b)| (a - px).abs() <= 1e-9 * (x1 - x0) && (b - py).abs() <= 1e-9 * (y1 - y0)) {
            continue;
        }
        seen.push((px, py));
        points.push(Point { x: px, y: py, guides: gl.into() });
    }
    // marked values are written on the axes; regular ticks too close to them make room
    let relabel = |tk: &mut (Vec<f64>, Vec<String>), ls: &[f64], w: f64| {
        if ls.is_empty() {
            return;
        }
        let mut keep_ls: Vec<f64> = vec![];
        for a in ls {
            if !keep_ls.iter().any(|b| (a - b).abs() < 0.06 * w) {
                keep_ls.push(*a);
            }
        }
        let mut all: Vec<(f64, String)> = tk.0.iter().zip(&tk.1).filter(|(p, _)| keep_ls.iter().all(|a| (a - **p).abs() > 0.08 * w)).map(|(p, l)| (*p, l.clone())).collect();
        for a in keep_ls {
            all.push((a, number(Num::F(a), None, cfg).0));
        }
        all.sort_by(|a, b| a.0.total_cmp(&b.0));
        *tk = (all.iter().map(|p| p.0).collect(), all.into_iter().map(|p| p.1).collect());
    };
    relabel(&mut xt, &labx, x1 - x0);
    relabel(&mut yt, &laby, y1 - y0);
    let axlabel = |n: &str, u: &UnitExpr| if u.is_empty() { n.to_string() } else { format!("{n} thin \\/ thin {}", format::unit(u, &units, cfg)) };
    let ylab = match &yname {
        Some(ns) => {
            let mut d = ns.clone();
            d.dedup();
            d.iter().map(|n| name(n)).collect::<Vec<_>>().join(", ")
        }
        None if curves.len() == 1 => curves[0].label.clone(),
        None => String::new(),
    };
    let _ = dim_text;
    Ok(GraphOut {
        x0,
        x1,
        y0,
        y1,
        xticks: xt.0,
        xlabels: xt.1,
        yticks: yt.0,
        ylabels: yt.1,
        xlabel: axlabel(&name(&x), &xunit),
        ylabel: axlabel(&ylab, &yunit),
        x_axis_zero: yc,
        y_axis_zero: xc,
        series,
        points,
        ax,
        ay,
    })
}

fn fixed10(p: f64) -> String {
    let s = format!("{p:.10}");
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s == "-0" { "0".into() } else { s }
}

fn bisect(h: &dyn Fn(f64) -> Option<f64>, mut a: f64, mut b: f64) -> f64 {
    let mut ha = h(a);
    for _ in 0..60 {
        let m = (a + b) / 2.0;
        if m == a || m == b {
            break;
        }
        let hm = h(m);
        let left = matches!((hm, ha), (Some(x), Some(y)) if (x > 0.0) == (y > 0.0));
        if left {
            a = m;
            ha = hm;
        } else {
            b = m;
        }
    }
    (a + b) / 2.0
}

fn bisect_edge(h: &dyn Fn(f64) -> Option<f64>, mut a: f64, mut b: f64) -> f64 {
    let ha = h(a).is_some();
    for _ in 0..60 {
        let m = (a + b) / 2.0;
        if m == a || m == b {
            break;
        }
        if h(m).is_some() == ha {
            a = m;
        } else {
            b = m;
        }
    }
    (a + b) / 2.0
}

/// The x range: the cluster of interesting points nearest 0 (roots, extrema, poles, edges of the domain and,
/// above all, where curves cross), with some room.
fn autorange(cs: &[Curve], x: &str, s: f64) -> (f64, f64) {
    let ks: Vec<f64> = (-48..=48).map(|k| s * 10f64.powf(k as f64 / 8.0)).collect();
    let mut ts: Vec<f64> = ks.iter().rev().map(|t| -t).collect();
    ts.push(0.0);
    ts.extend(ks.iter());
    let at = |e: &Expr, t: f64| eval_f(e, &|n| (n == x).then_some(t));
    let cross = |h: &dyn Fn(f64) -> Option<f64>| -> Vec<f64> {
        let vs: Vec<Option<f64>> = ts.iter().map(|&t| h(t)).collect();
        let mut r = vec![];
        for i in 0..ts.len() {
            let Some(u) = vs[i] else { continue };
            if u == 0.0 {
                if i > 0 && i < ts.len() - 1 && vs[i - 1].is_some_and(|v| v != 0.0) && vs[i + 1].is_some_and(|v| v != 0.0) {
                    r.push(ts[i]);
                }
                continue;
            }
            if let Some(Some(w)) = vs.get(i + 1) {
                if *w != 0.0 && (u > 0.0) != (*w > 0.0) {
                    r.push(bisect(h, ts[i], ts[i + 1]));
                }
            }
        }
        r
    };
    let (mut pts, mut edges, mut xs) = (vec![], vec![], vec![]);
    for c in cs {
        let d = c.e.diff(x);
        pts.extend(cross(&|t| at(&c.e, t)));
        pts.extend(cross(&|t| at(&d, t)));
        for i in 0..ts.len() - 1 {
            let (u, w) = (at(&c.e, ts[i]), at(&c.e, ts[i + 1]));
            // overflow (exp of a big number) isn't an edge of the domain
            if u.is_none() != w.is_none() && u.or(w).unwrap().abs() < 1e30 {
                edges.push(bisect_edge(&|t| at(&c.e, t), ts[i], ts[i + 1]));
            }
        }
    }
    for i in 0..cs.len() {
        for j in i + 1..cs.len() {
            xs.extend(cross(&|t| Some(at(&cs[i].e, t)? - at(&cs[j].e, t)?)));
        }
    }
    let mut pts: Vec<f64> = pts.into_iter().chain(xs.iter().copied()).chain(edges.iter().copied()).collect();
    // where curves cross is what the graph is about: only what lies near the crossings
    if !xs.is_empty() {
        let w = 2.0 * xs.iter().map(|v| v.abs()).fold(0.0, f64::max);
        pts.retain(|t| t.abs() <= w);
    }
    let pts: Vec<f64> = pts.into_iter().map(|t| if t.abs() < 1e-9 * s { 0.0 } else { t }).collect();
    // distances from 0, outward while each is at most 10× the previous, 4 at most
    let mut absd: Vec<f64> = pts.iter().filter(|t| **t != 0.0).map(|t| t.abs()).collect();
    absd.sort_by(|a, b| a.total_cmp(b));
    let mut ds: Vec<f64> = vec![];
    for a in absd {
        if let Some(&l) = ds.last() {
            if a <= l * (1.0 + 1e-6) {
                continue;
            }
            if ds.len() >= 4 || a > 10.0 * l {
                break;
            }
        }
        ds.push(a);
    }
    let m = ds.last().copied().unwrap_or(0.0);
    let core: Vec<f64> = pts.iter().copied().filter(|t| t.abs() <= m * (1.0 + 1e-6)).collect();
    if core.is_empty() {
        return (-s, s);
    }
    let lo = core.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = core.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (mut a, mut b) = if lo == hi {
        let w = if lo == 0.0 { s } else { lo.abs() };
        (lo - w, lo + w)
    } else {
        let p = (hi - lo) * 0.2;
        (lo - p, hi + p)
    };
    // physical ranges start at 0 when everything interesting is positive and not far from it
    if lo > 0.0 && lo < 0.5 * (hi - lo) {
        a = 0.0;
    }
    if hi < 0.0 && -hi < 0.5 * (hi - lo) {
        b = 0.0;
    }
    // don't run past the edge of the domain (sqrt, ln)
    let ok = |t: f64| cs.iter().any(|c| at(&c.e, t).is_some());
    let mut es = edges.clone();
    es.sort_by(|a, b| a.total_cmp(b));
    for &t in &es {
        if t <= lo && t > a && !ok(t - (t - a) / 2.0) {
            a = t;
        }
    }
    for &t in es.iter().rev() {
        if t >= hi && t < b && !ok(t + (b - t) / 2.0) {
            b = t;
        }
    }
    // round to a tick step, but not past a domain edge
    let st = nice((b - a) / 5.0);
    let (a2, b2) = ((a / st + 1e-9).floor() * st, (b / st - 1e-9).ceil() * st);
    (if ok(a2) || a2 == a { a2 } else { a }, if ok(b2) || b2 == b { b2 } else { b })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{LineReq, line};

    #[test]
    fn operator_labels_are_independent_of_calculation_names() {
        let mut req = GraphReq {
            cfg: Cfg::default(), curves: vec![Src {
                src: "tri(2x)".into(), mode: Mode::Code, label: Some("Λ(2x)".into()), operators: vec!["Λ".into(), "dreieick".into(), "sqrt".into()],
            }],
            xrange: vec![XV::Num(-1.0), XV::Num(1.0)], yrange: vec![], marks: vec![], ymark: vec![], samples: 101,
        };
        let first = graph(&Env::default(), &req).unwrap();
        assert!(first.series.iter().filter_map(|s| s.label.as_ref()).all(|s| s.contains("Λ") && !s.contains("tri")));
        assert!(first.ylabel.contains("Λ"), "{}", first.ylabel);
        req.curves[0].label = Some("dreieick(2x)".into());
        let second = graph(&Env::default(), &req).unwrap();
        assert_eq!(first.series[0].points, second.series[0].points);
        assert!(second.ylabel.contains("dreieick"), "{}", second.ylabel);
        req.curves[0].label = Some("sqrt(2x)".into());
        let third = graph(&Env::default(), &req).unwrap();
        assert!(third.ylabel.contains("upright(\"sqrt\")"), "{}", third.ylabel);
    }

    #[test]
    fn graphs() {
        let mut env = Env::default();
        for s in ["R := 50 Ω", "P0 := 2 W", "U0 := 10 V"] {
            let o = line(&mut env, &LineReq { src: s.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
            assert!(o.errors.is_empty());
        }
        let req = |curves: Vec<&str>, ymark: Vec<&str>| GraphReq {
            cfg: Cfg::default(),
            curves: curves.into_iter().map(|c| Src { src: c.into(), mode: Mode::Code, label: None, operators: vec![] }).collect(),
            xrange: vec![],
            yrange: vec![],
            marks: vec![(Mark::Key("auto".into()), true)],
            ymark: ymark.into_iter().map(|c| XV::Src(Src { src: c.into(), mode: Mode::Code, label: None, operators: vec![] })).collect(),
            samples: 100,
        };
        let g = graph(&env, &req(vec!["P(U: V) = U^2/R"], vec!["P0"])).unwrap();
        assert!(g.xlabel.contains("upright(\"V\")"), "{}", g.xlabel);
        assert!(g.points.iter().any(|p| (p.x - 10.0).abs() < 1e-6), "{:?}", g.points);
        let g = graph(&env, &req(vec!["cos(w)", "w"], vec![])).unwrap();
        assert!(g.points.iter().any(|p| (p.x - 0.739085).abs() < 1e-5), "{:?}", g.points);
        let e = graph(&env, &req(vec!["cos(w) = w"], vec![])).unwrap_err();
        assert!(e.contains("two curves"), "{e}");
        let g = graph(&env, &req(vec!["P(..) = U0^2/R_L"], vec!["P0"])).unwrap();
        assert!(g.xlabel.contains("Ω"), "{}", g.xlabel);
    }

    #[test]
    fn curves_with_define() {
        let mut env = Env::default();
        let run = |env: &mut Env, s: &str| line(env, &LineReq { src: s.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
        assert!(run(&mut env, "X(x) := x^2 + 1").errors.is_empty());
        let req = |c: &str| GraphReq { cfg: Cfg::default(), curves: vec![Src { src: c.into(), mode: Mode::Code, label: None, operators: vec![] }], xrange: vec![], yrange: vec![], marks: vec![], ymark: vec![], samples: 50 };
        // y := X(x): plotted over x, and as a line it defines y with x left free
        graph(&env, &req("y := X(x)")).unwrap();
        assert_eq!(defines(&env, &req("y := X(x)")).unwrap().len(), 1);
        assert!(run(&mut env, "y := X(x)").errors.is_empty());
        assert!(env.vars.contains_key("y"));
        // y := y*2: the value before, doubled
        let g = graph(&env, &req("y := y*2")).unwrap();
        assert!(g.series.iter().flat_map(|s| &s.points).all(|p| (p.1 - 2.0 * (p.0 * p.0 + 1.0)).abs() < 1e-9));
        // a parameter of the curve's name is the parameter
        graph(&env, &req("y(y) := X(y)")).unwrap();
        // also over a y that is defined as a value, and with a constant X
        assert!(run(&mut env, "y := 20").errors.is_empty());
        assert!(run(&mut env, "K(x) := 20").errors.is_empty());
        graph(&env, &req("y(y) := K(y)")).unwrap();
        assert!(run(&mut env, "y(y) := X(y)").errors.is_empty());
        // with its parameters written out, a curve may use only them and defined names
        let e = graph(&env, &req("z(y) := X(x)")).unwrap_err();
        assert!(e.contains("'x' in 'z' isn't defined"), "{e}");
        // a name passed to a constant function still varies: a constant line over it
        assert!(run(&mut env, "C(x) := 20").errors.is_empty());
        let g = graph(&env, &req("c := C(x)")).unwrap();
        assert!(g.series.iter().flat_map(|s| &s.points).all(|p| (p.1 - 20.0).abs() < 1e-9));
        assert!(g.y0 < 20.0 && 20.0 < g.y1, "{} {}", g.y0, g.y1);
        // = plots without defining
        assert!(defines(&env, &req("w(t) = X(t)")).unwrap().is_empty());
    }
}
