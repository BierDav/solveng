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
use crate::parse::{self, Ctx, ParamType};
use crate::render::R;
use crate::solve::{infer_dims, scan};
use crate::units::{UnitExpr, Units};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Src {
    pub src: String,
    #[serde(default)]
    pub label: Option<String>,
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
    /// Sampled data with few points: each point is drawn as a dot too.
    #[serde(default)]
    pub dots: bool,
    /// Sampled data rather than a function: it can be drawn as points alone.
    #[serde(default)]
    pub data: bool,
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
    /// Which ticks write a marked value rather than a step of the scale (these get no grid line).
    pub xmarked: Vec<bool>,
    pub ymarked: Vec<bool>,
    /// The axes' names and units as math, empty without one (for the values an interactive graph shows).
    pub xname: String,
    pub yname: String,
    pub xunit: String,
    pub yunit: String,
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
    /// Sampled data instead of a function of x (`e` is then unused).
    data: Option<Data>,
}

/// Data with at most this many points is drawn with a dot at every point, a line alone hides where they are.
const DOTS_MAX: usize = 30;

/// Sampled data: the points (x_k, y_k) in SI, drawn in index order.
struct Data {
    xs: Vec<f64>,
    ys: Vec<f64>,
    xd: Dim,
    yd: Dim,
    /// What x is: (identifier, axis label markup); `None`: the index k = 0 … n−1.
    x: Option<(String, String)>,
}

impl Data {
    fn segments(&self) -> impl Iterator<Item = ((f64, f64), (f64, f64))> + '_ {
        (1..self.xs.len()).map(|k| ((self.xs[k - 1], self.ys[k - 1]), (self.xs[k], self.ys[k])))
    }
    /// y on every segment through x = t, linearly interpolated (several where x runs back).
    fn at(&self, t: f64) -> Vec<f64> {
        let mut out: Vec<f64> = vec![];
        let mut add = |y: f64| if out.last() != Some(&y) { out.push(y) };
        if self.xs.len() == 1 && self.xs[0] == t { add(self.ys[0]); }
        for ((x1, y1), (x2, y2)) in self.segments() {
            if t < x1.min(x2) || t > x1.max(x2) { continue; }
            if t == x1 { add(y1); } else if t == x2 { add(y2); } else { add(y1 + (y2 - y1) * (t - x1) / (x2 - x1)); }
        }
        out
    }
    /// The x values where the data reaches y = level: samples on it and sign changes between samples, interpolated.
    fn level(&self, level: f64) -> Vec<f64> {
        let n = self.xs.len();
        let mut out = vec![];
        for k in 0..n {
            let u = self.ys[k] - level;
            if u == 0.0 {
                out.push(self.xs[k]);
            } else if k + 1 < n {
                let w = self.ys[k + 1] - level;
                if w != 0.0 && (u > 0.0) != (w > 0.0) { out.push(self.xs[k] + (self.xs[k + 1] - self.xs[k]) * u / (u - w)); }
            }
        }
        out
    }
    /// Local extreme samples: higher or lower than the samples on both sides (a plateau counts once, at its middle).
    fn extrema(&self) -> Vec<(f64, f64)> {
        let (ys, n) = (&self.ys, self.ys.len());
        let mut out = vec![];
        let mut a = 0;
        while a < n {
            let mut b = a;
            while b + 1 < n && ys[b + 1] == ys[a] { b += 1; }
            if a > 0 && b + 1 < n && (ys[a - 1] < ys[a]) == (ys[b + 1] < ys[a]) {
                let m = (a + b) / 2;
                out.push((self.xs[m], ys[m]));
            }
            a = b + 1;
        }
        out
    }
    /// The parts of the polyline within a ≤ x ≤ b, cut where it leaves.
    fn clip(&self, a: f64, b: f64) -> Vec<Vec<(f64, f64)>> {
        let inside = |x: f64| a <= x && x <= b;
        let mut runs: Vec<Vec<(f64, f64)>> = vec![];
        if self.xs.len() == 1 && inside(self.xs[0]) { runs.push(vec![(self.xs[0], self.ys[0])]); }
        let mut open = false;
        for (p, q) in self.segments() {
            // the part of the segment p + s (q − p), 0 ≤ s ≤ 1, within the range
            let (mut s0, mut s1) = (0.0f64, 1.0f64);
            if p.0 == q.0 {
                if !inside(p.0) { open = false; continue; }
            } else {
                let (sa, sb) = ((a - p.0) / (q.0 - p.0), (b - p.0) / (q.0 - p.0));
                s0 = s0.max(sa.min(sb));
                s1 = s1.min(sa.max(sb));
                if s0 > s1 { open = false; continue; }
            }
            let at = |s: f64| if s == 0.0 { p } else if s == 1.0 { q } else { (p.0 + s * (q.0 - p.0), p.1 + s * (q.1 - p.1)) };
            if !open || s0 > 0.0 { runs.push(vec![at(s0)]); }
            runs.last_mut().unwrap().push(at(s1));
            open = s1 == 1.0;
        }
        runs
    }
}

/// The numeric values of a vector (a matrix of one row or one column): SI numbers of one dimension. `None` if
/// `e` isn't a vector; `what` names it in messages.
fn data_values(env: &Env, units: &Units, e: &Expr, what: &str) -> Result<Option<(Vec<f64>, Dim)>, String> {
    let e = simp(&env.bind_with_units(e, &BTreeSet::new(), units)?);
    let Some(a) = crate::linalg::Array::of(&e) else { return Ok(None) };
    if a.rows > 1 && a.cols > 1 {
        return Err(format!("{what} is {} – plot a vector, for example a column of it", a.describe()));
    }
    let mut xs = vec![];
    let mut dim: Option<Dim> = None;
    for (k, el) in a.a.iter().enumerate() {
        let v = value(el).map_err(|m| format!("{what}: element {k} has no value – {}", m.message()))?;
        let x = v.v.f();
        if !x.is_finite() { return Err(format!("{what}: element {k} isn't a finite number")); }
        match dim {
            Some(d) if d != v.d => {
                let t = |d: &Dim| if d.is_one() { "no unit".to_string() } else { units.text(&units.auto(d)) };
                return Err(format!("{what}: element {k} is in {}, but element 0 in {} – data needs one unit for all elements", t(&v.d), t(&d)));
            }
            _ => dim = Some(v.d),
        }
        xs.push(x);
    }
    if xs.is_empty() { return Err(format!("{what} is empty – nothing to plot")); }
    Ok(Some((xs, dim.unwrap_or(Dim::ONE))))
}

/// A curve over another vector: `y(tau)` with y a defined vector and tau anything without free names that is a
/// vector of the same length. `None`: not of this form, read as a function curve. The only place a data curve is
/// told by how it's written; `src` is the curve as written, for messages.
fn data_curve(env: &Env, cv: &parse::Curve, src: &str, units: &Units, r: &R) -> Result<Option<(String, Data)>, String> {
    let parse::Ast::Call(n, args) = &cv.body else { return Ok(None) };
    if cv.head.as_ref().is_some_and(|h| !h.1.is_empty()) || env.funs.contains_key(n) { return Ok(None); }
    let Some(var) = env.vars.get(n) else { return Ok(None) };
    if cv.defines {
        return Err(format!("'{src}' plots data, it defines nothing – write it with = or without a name"));
    }
    let [arg] = args.as_slice() else {
        return Err(format!("'{n}' is a variable: '{n}(…)' plots the vector {n} over one vector of the same length, like {n}(t)"));
    };
    let Some((ys, yd)) = data_values(env, units, &var.expr, n)? else {
        return Err(format!("'{n}' isn't a vector, so '{src}' can't plot it over another – {n} is a variable and can't be called"));
    };
    let what = match arg { parse::Ast::Name(m) => m.clone(), _ => format!("the argument of {n}") };
    let xe = Lower::new(env, units).expr(arg)?;
    let free = env.bind_with_units(&xe, &BTreeSet::new(), units)?.syms();
    if let Some(f) = free.first() {
        return Err(format!("'{f}' isn't defined – '{src}' plots the vector {n} over a vector of the same length; a function of {f} is written {n}({f}) := …"));
    }
    let Some((xs, xd)) = data_values(env, units, &xe, &what)? else {
        return Err(format!("{what} isn't a vector – '{src}' plots {n} over a vector of the same length; one element is at({n}, k)"));
    };
    if xs.len() != ys.len() {
        return Err(format!("'{src}': {n} has {} elements, but {what} has {} – data is plotted over a vector of the same length", ys.len(), xs.len()));
    }
    let xlabel = r.ast(arg).0;
    let id = match arg { parse::Ast::Name(m) => m.clone(), _ => xlabel.clone() };
    let name = cv.head.as_ref().map_or(n, |h| &h.0).clone();
    Ok(Some((name, Data { xs, ys, xd, yd, x: Some((id, xlabel)) })))
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
    /// The x axis's label where data sets it (the names of the x vectors).
    xlabel: Option<String>,
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
    let r = R { units, cfg, exact: false };
    let tidy = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ").replace("( ", "(").replace(" )", ")");
    let mut curves = vec![];
    let mut takes: Vec<String> = vec![];
    let mut xunit_typed: Option<UnitExpr> = None;
    let mut defines = vec![];
    // the names data is plotted over (tau in y(tau)) are what the function curves alongside take: c(tau), 2 tau
    let mut data_x = BTreeSet::new();
    for c in &req.curves {
        let ctx = Ctx { units, is_fn: &is_fn, is_var: &is_var };
        if let Ok(cv) = parse::parse_curve(&c.src, &ctx)
            && let Ok(Some((_, Data { x: Some((id, _)), .. }))) = data_curve(env, &cv, &tidy(&c.src), units, &r)
            && is_var(&id) {
            data_x.insert(id);
        }
    }
    for c in &req.curves {
        let ctx = Ctx { units, is_fn: &is_fn, is_var: &is_var };
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
        let display = c.label.as_ref().map(|label| parse::parse_curve(label, &ctx)).transpose()?;
        let label = r.ast(display.as_ref().map_or(&cv.body, |cv| &cv.body)).0;
        if let Some((n, data)) = data_curve(env, &cv, &tidy(&c.src), units, &r)? {
            curves.push(Curve { e: Expr::C(crate::expr::Val::new(Num::ZERO, Dim::ONE)), label, name: Some(n), call: None, params: vec![], data: Some(data) });
            continue;
        }
        // the free names: what the curve is plotted against
        let mut lo = Lower::new(env, units);
        lo.params.extend(data_x.iter().cloned());
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
            if !free.contains(&n) && !is_var(&n) && !is_fn(&n)
                && lo.expr(&parse::Ast::Name(n.clone()))?.syms().contains(&n) {
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
        // nothing varies and it's a vector of values: data, plotted against its index
        if free.is_empty() && let Some((ys, yd)) = data_values(env, units, &e0, &label)? {
            let name = match (&cv.head, &cv.body) {
                (Some(h), _) => Some(h.0.clone()),
                (None, parse::Ast::Name(n)) => Some(n.clone()),
                _ => None,
            };
            if cv.defines { defines.push(c.clone()); }
            let xs = (0..ys.len()).map(|k| k as f64).collect();
            curves.push(Curve { e: Expr::C(crate::expr::Val::new(Num::ZERO, Dim::ONE)), label, name, call: None, params: written.clone(), data: Some(Data { xs, ys, xd: Dim::ONE, yd, x: None }) });
            continue;
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
                name: cv.head.as_ref().map(|h| h.0.clone()), call: call.clone(), params: written.clone(), data: None,
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
    // data is over its x vectors (or its index); function curves alongside take the same x
    let datax: Vec<(String, String)> = curves.iter().filter_map(|c| c.data.as_ref()?.x.clone()).collect();
    let has_data = curves.iter().any(|c| c.data.is_some());
    if let (Some(t), Some(dx)) = (takes.first(), data_x.iter().find(|d| Some(*d) != takes.first())) {
        return Err(format!("a graph has one x axis: the data is over '{dx}', but '{t}' varies in '{eq}' – write the curves in {dx}"));
    }
    let Some(x) = takes.first().or(data_x.first()).or(datax.first().map(|d| &d.0)).cloned().or(has_data.then(|| "k".to_string())) else {
        return Err(format!("nothing varies in '{eq}' – a curve takes what it's plotted against: U(x) = …"));
    };
    let xlabel = has_data.then(|| {
        let mut ls: Vec<String> = takes.iter().map(|t| name(t)).chain(datax.iter().map(|d| d.1.clone())).collect();
        ls.dedup();
        if ls.is_empty() { name(&x) } else { ls.join(", ") }
    });
    // y(y) := … names its parameter so on purpose
    if curves.iter().any(|c| c.data.is_none() && c.name.as_deref() == Some(x.as_str()) && !c.params.contains(&x)) {
        return Err(format!("'{x}' is what the curve is plotted against, so it can't name it"));
    }
    Ok(Read { curves, x, xlabel, xunit_typed, defines, eq })
}

/// The curves written with `:=`, if the graph can read its curves: what calculating them as lines defines.
pub fn defines(env: &Env, req: &GraphReq) -> Result<Vec<Src>, String> {
    let source = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let units = Units::lexical(&req.cfg, env, &source)?;
    Ok(read(env, req, &units)?.defines)
}

pub fn graph(env: &Env, req: &GraphReq) -> Result<GraphOut, String> {
    let source = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let units = Units::lexical(&req.cfg, env, &source)?;
    let cfg = &req.cfg;
    if req.curves.is_empty() {
        return Err("nothing to plot – pass an equation like \"P = U^2/R\"".into());
    }
    if req.samples < 2 {
        return Err("samples: an integer ≥ 2".into());
    }
    let Read { curves, x, xlabel, xunit_typed, eq, .. } = read(env, req, &units)?;
    let data: Vec<&Data> = curves.iter().filter_map(|c| c.data.as_ref()).collect();
    let utext = |d: &Dim| if d.is_one() { "no unit".to_string() } else { units.text(&units.auto(d)) };
    if let Some(d) = data.iter().find(|d| d.xd != data[0].xd) {
        return Err(format!("the data's x values have different units ({} and {}) – plot them separately", utext(&data[0].xd), utext(&d.xd)));
    }
    let tidy = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ").replace("( ", "(").replace(" )", ")");
    // values of ranges and marks
    let xv = |v: &XV| -> Result<V, String> {
        match v {
            XV::Num(n) => Ok(V::Plain(*n)),
            XV::Src(s) => {
                let t = s.src.trim();
                // no e-notation: `1e5` is a number and the name e5, not a range value
                if !t.contains(['e', 'E'])
                    && let Ok(n) = t.replace(',', ".").replace('−', "-").parse::<f64>() {
                    return Ok(V::Plain(n));
                }
                let (val, d) = crate::api::quantity(env, cfg, t).map_err(|e| format!("{e} in '{t}'"))?;
                let a = units.auto(&d);
                let u = units.prefixed_for(val / units.factor(&a).f(), &a, &cfg.prefixes);
                Ok(V::Q(val, d.clone(), if d.is_one() { String::new() } else { units.text(&u) }, tidy(t)))
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
            V::Q(_, d, u, _) if !d.is_one() => Some((d.clone(), u.clone())),
            _ => None,
        })
        .collect();
    // data's x unit: its elements', with a prefix fitting the largest
    let data_xunit = data.first().map(|d| {
        let a = units.auto(&d.xd);
        let m = data.iter().flat_map(|d| &d.xs).fold(0.0f64, |m, x| m.max(x.abs()));
        units.prefixed_for(m / units.factor(&a).f(), &a, &cfg.prefixes)
    });
    // the data's unit, if the functions alongside take x in the same dimension (c(t: s) next to data in ms)
    if let (Some(d), Some(u)) = (data.first(), &xunit_typed) && units.dim(u).unwrap_or(Dim::ONE) != d.xd {
        return Err(format!("the data is over x values in {}, but the curves take {x} in {} – x needs one unit", utext(&d.xd), units.text(&u)));
    }
    let mut xunit = match (&data_xunit, &xunit_typed, withu.first()) {
        (Some(u), ..) | (None, Some(u), _) => u.clone(),
        (None, None, Some((_, u))) => units.parse(u, false).map(|p| p.u).unwrap_or_default(),
        _ => UnitExpr::default(),
    };
    let mut xd = units.dim(&xunit).unwrap_or(Dim::ONE);
    // no unit for x yet, but y values have one: x's unit follows from the curve (P = U0²/R2 with P0 in W: R2 in Ω)
    let yu = yrv.iter().chain(ymv.iter()).find_map(|v| match v {
        V::Q(_, d, ..) if !d.is_one() => Some(d.clone()),
        _ => None,
    });
    if xunit.is_empty() && withu.is_empty() && data.is_empty() {
        if let Some(yd) = yu {
            let known = |n: &str| env.vars.get(n).and_then(|v| value(&env.bind_with_units(&v.expr, &BTreeSet::new(), &units).ok()?).ok()).map(|v| v.d);
            let y = Expr::C(crate::expr::Val::new(Num::ONE, yd));
            if let Ok(ds) = infer_dims(&[(curves[0].e.clone(), y)], std::slice::from_ref(&x), &known) {
                if let Some(Some(d)) = ds.first() {
                    if !d.is_one() {
                        xd = d.clone();
                        let a = units.auto(d);
                        xunit = a;
                    }
                }
            }
        }
    }
    if let Some(u) = format::preferred_unit(xd.clone(), &units, cfg) { xunit = u; }
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
        return Err(misfit(bad, &x, &units.text(&xunit)));
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
            return Some(xd.clone());
        }
        env.vars.get(n).and_then(|v| value(&env.bind_with_units(&v.expr, &BTreeSet::new(), &units).ok()?).ok()).map(|v| v.d)
    };
    let cdim = |c: &Curve| match &c.data { Some(d) => Ok(Some(d.yd.clone())), None => dim_of(&c.e, &known) };
    let yd = cdim(&curves[0])?.unwrap_or(Dim::ONE);
    for c in &curves[1..] {
        if let Ok(Some(d)) = cdim(c) {
            // a unitless function next to others may be a scale; data has the unit it has
            if d != yd && (!yd.is_one() || c.data.is_some()) {
                return Err(format!("the curves have different units ({} and {}) – plot them separately", if yd.is_one() { "1".into() } else { units.text(&units.auto(&yd)) }, if d.is_one() { "1".into() } else { units.text(&units.auto(&d)) }));
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
        return Err(misfit(bad, &yaxis, &if yd.is_one() { String::new() } else { units.text(&units.auto(&yd)) }));
    }
    let preferred_yunit = format::preferred_unit(yd.clone(), &units, cfg);
    let yb = units.factor(&preferred_yunit.clone().unwrap_or_else(|| units.auto(&yd))).f();
    let yrv: Vec<f64> = yrv.iter().map(|v| si(v, yb)).collect();
    let ymv: Vec<f64> = ymv.iter().map(|v| si(v, yb)).collect();
    let f = |c: &Curve, t: f64| match &c.data {
        Some(d) => d.at(t).first().copied(),
        None => eval_f(&c.e, &|n| (n == x).then_some(t)),
    };
    // where the curves reach the y marks: the nearest ones must be visible
    let mut yneed = vec![];
    for y in &ymv {
        for c in curves.iter().filter(|c| c.data.is_none()) {
            let mut rs = scan(&simp(&expr::sub(c.e.clone(), Expr::C(crate::expr::Val::new(Num::F(*y), Dim::ONE)))), &x, xf);
            rs.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
            if let Some(first) = rs.first().copied() {
                yneed.extend(rs.into_iter().filter(|t| t.abs() <= 2.0 * first.abs() + 1e-9 * xf).take(4));
            }
        }
    }
    let mut boundaries = vec![];
    for c in curves.iter().filter(|c| c.data.is_none()) { case_boundaries(&c.e, &x, xf, &mut boundaries); }
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
    } else if !data.is_empty() {
        // data: from its first to its last x value, as it was taken
        let lo = data.iter().flat_map(|d| &d.xs).copied().fold(f64::INFINITY, f64::min);
        let hi = data.iter().flat_map(|d| &d.xs).copied().fold(f64::NEG_INFINITY, f64::max);
        if lo == hi {
            let w = if lo == 0.0 { xf } else { lo.abs() / 2.0 };
            (lo - w, lo + w)
        } else {
            (lo, hi)
        }
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
    let ys: Vec<Vec<Option<f64>>> = curves.iter().map(|c| if c.data.is_some() { vec![None; ts.len()] } else { ts.iter().map(|&t| f(c, t)).collect() }).collect();
    // data within the x range, as drawn
    let clipped: Vec<Vec<Vec<(f64, f64)>>> = curves.iter().map(|c| c.data.as_ref().map_or(vec![], |d| d.clip(xa, xb))).collect();
    let dys: Vec<f64> = clipped.iter().flatten().flatten().map(|p| p.1).collect();
    let mut all: Vec<f64> = ys.iter().flatten().filter_map(|y| *y).collect();
    all.sort_by(|a, b| a.total_cmp(b));
    if all.is_empty() && dys.is_empty() && !data.is_empty() {
        return Err("the data has no points within the x range".into());
    }
    if all.is_empty() && dys.is_empty() {
        let mut missing = BTreeSet::new();
        for curve in &curves {
            let mut pending = vec![&curve.e];
            while let Some(e) = pending.pop() {
                if let Expr::Call(n, _) = e { missing.insert(n.clone()); }
                pending.extend(e.children());
            }
        }
        if let Some(n) = missing.first() {
            return Err(format!("function '{n}' isn't defined – define it with := before plotting, for example {n}({x}) := …"));
        }
        return Err(format!("'{eq}' can't be evaluated in this range"));
    }
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    if !all.is_empty() {
        (lo, hi) = (all[0], *all.last().unwrap());
        let (q1, q2) = (all[(all.len() as f64 * 0.05) as usize], all[((all.len() as f64 * 0.95) as usize).min(all.len() - 1)]);
        if hi - lo > 20.0 * (q2 - q1) && q2 > q1 {
            (lo, hi) = (q1 - (q2 - q1) / 2.0, q2 + (q2 - q1) / 2.0); // poles
        }
    }
    // data has no poles: all of it shows
    for y in &dys {
        lo = lo.min(*y);
        hi = hi.max(*y);
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
        // but all of the data
        for y in &dys {
            lo = lo.min(*y);
            hi = hi.max(*y);
        }
    }
    // a function's result in the type it returns (A², not W from V·A)
    let ret = if curves.len() == 1 { curves[0].call.as_ref().and_then(|c| env.funs.get(c)).and_then(|fun| fun.ret.clone()) } else { None };
    let ybase = match ret {
        Some(d) if d == yd => units.auto(&d),
        _ => units.auto(&yd),
    };
    let yunit = preferred_yunit.unwrap_or_else(||
        units.prefixed_for(lo.abs().max(hi.abs()) / units.factor(&ybase).f(), &ybase, &cfg.prefixes));
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
        // Flat curves can differ by a few ulps after numerical evaluation.
        // Give them a useful range before choosing a tick step: a sub-ulp
        // step at this magnitude cannot advance a floating-point tick loop.
        let scale = y0.abs().max(y1.abs()).max(1.0);
        if y1 - y0 <= 32.0 * f64::EPSILON * scale {
            let center = y0 * 0.5 + y1 * 0.5;
            let half = scale.max(2.0) * 0.5;
            (y0, y1) = (center - half, center + half);
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
            let next = v + st;
            if next <= v { break; }
            v = next;
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
        // functions: their samples; data: its points as they are
        let samples: &[f64] = if c.data.is_some() { &[] } else { &ts };
        for (j, &t) in samples.iter().enumerate() {
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
        if c.data.is_some() {
            runs = clipped[i].iter().map(|r| r.iter().map(|p| (p.0 / xf, p.1 / yf, usize::MAX)).collect()).collect();
        }
        let longest = runs.iter().map(|r| r.len()).max().unwrap_or(0);
        // one label per curve, on its longest run (a piecewise curve has several of the same length)
        let mut labelled = false;
        for run in runs {
            let lab = curves.len() > 1 && run.len() == longest && !labelled;
            labelled |= lab;
            let mut lp = 0.8;
            if lab {
                let (mut best, mut bd) = (None, -1.0);
                for (k, &(px, py, j)) in run.iter().enumerate().rev() {
                    // not on the axes' ends, where their labels are
                    if (k as f64) < run.len() as f64 * 0.15 || (k as f64) > run.len() as f64 * 0.95 || py < y0 + 0.08 * (y1 - y0) || py > y1 - 0.12 * (y1 - y0) {
                        continue;
                    }
                    if used.iter().any(|&(ux, uy)| (ux - px).abs() < 0.2 * (x1 - x0) && (uy - py).abs() < 0.15 * (y1 - y0)) {
                        continue;
                    }
                    let dd = (0..curves.len())
                        .filter(|&o| o != i)
                        .map(|o| match if j == usize::MAX || curves[o].data.is_some() { f(&curves[o], px * xf) } else { ys[o][j] } {
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
            series.push(Series { points: run.iter().map(|p| (p.0, p.1)).collect(), curve: i, label: lab.then(|| label.clone()), label_pos: lp, dots: c.data.as_ref().is_some_and(|d| d.xs.len() <= DOTS_MAX), data: c.data.is_some() });
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
            let (ca, cb) = (&curves[a], &curves[b]);
            let met = match (&ca.data, &cb.data) {
                (None, None) => {
                    let h = |t: f64| Some(f(ca, t)? - f(cb, t)?);
                    zeros(&h).into_iter().map(|t| (t, f(ca, t))).collect()
                }
                (Some(d), None) => data_meets(d, &|t| f(cb, t), &ts),
                (None, Some(d)) => data_meets(d, &|t| f(ca, t), &ts),
                (Some(p), Some(q)) => data_cross(p, q),
            };
            for (t, y) in met {
                out.push((t, y, if ly { "xy" } else { "x" }, true, ly));
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
                        let ts = match &c.data { Some(d) => d.level(0.0), None => zeros(&|t| f(c, t)) };
                        for t in ts {
                            pts.push((t, Some(0.0), "", false, false));
                        }
                    }
                }
                "extrema" => {
                    for c in &curves {
                        if let Some(d) = &c.data {
                            pts.extend(d.extrema().into_iter().map(|(t, y)| (t, Some(y), "x", false, false)));
                            continue;
                        }
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
        let guides = if *ly || curves.len() == 1 { "xy" } else { "x" };
        for c in &curves {
            match &c.data {
                // every point of the data at t: where its x runs back, more than one
                Some(d) if !d.at(*t).is_empty() => pts.extend(d.at(*t).into_iter().map(|y| (*t, Some(y), guides, true, *ly))),
                _ => pts.push((*t, f(c, *t), guides, true, *ly)),
            }
        }
    }
    for y in &ymv {
        for c in &curves {
            let ts = match &c.data { Some(d) => d.level(*y), None => zeros(&|t| Some(f(c, t)? - y)) };
            for t in ts {
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
    let relabel = |tk: &mut (Vec<f64>, Vec<String>), ls: &[f64], w: f64| -> Vec<bool> {
        if ls.is_empty() {
            return vec![false; tk.0.len()];
        }
        let mut keep_ls: Vec<f64> = vec![];
        for a in ls {
            if !keep_ls.iter().any(|b| (a - b).abs() < 0.06 * w) {
                keep_ls.push(*a);
            }
        }
        let mut all: Vec<(f64, String)> = tk.0.iter().zip(&tk.1).filter(|(p, _)| keep_ls.iter().all(|a| (a - **p).abs() > 0.08 * w)).map(|(p, l)| (*p, l.clone())).collect();
        for &a in &keep_ls {
            all.push((a, number(Num::F(a), None, cfg).0));
        }
        all.sort_by(|a, b| a.0.total_cmp(&b.0));
        let marked = all.iter().map(|p| keep_ls.contains(&p.0)).collect();
        *tk = (all.iter().map(|p| p.0).collect(), all.into_iter().map(|p| p.1).collect());
        marked
    };
    let xmarked = relabel(&mut xt, &labx, x1 - x0);
    let ymarked = relabel(&mut yt, &laby, y1 - y0);
    let axlabel = |n: &str, u: &UnitExpr| if u.is_empty() { n.to_string() } else {
        // a compound display is one unit: t / (2.54 cm)
        let m = format::unit(u, &units, cfg);
        format!("{n} thin \\/ thin {}", if matches!(u.0.as_slice(), [(_, 1)]) && format::unit_compound(u, &units) { format!("lr(({m}))") } else { m })
    };
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
    let xname = xlabel.unwrap_or_else(|| name(&x));
    Ok(GraphOut {
        x0,
        x1,
        y0,
        y1,
        xticks: xt.0,
        xlabels: xt.1,
        yticks: yt.0,
        ylabels: yt.1,
        xlabel: axlabel(&xname, &xunit),
        ylabel: axlabel(&ylab, &yunit),
        xmarked,
        ymarked,
        yname: ylab.clone(),
        xname,
        xunit: if xunit.is_empty() { String::new() } else { format::unit(&xunit, &units, cfg) },
        yunit: if yunit.is_empty() { String::new() } else { format::unit(&yunit, &units, cfg) },
        x_axis_zero: yc,
        y_axis_zero: xc,
        series,
        points,
        ax,
        ay,
    })
}

/// Where data meets a function g: on each segment, sign changes of the difference at the samples within it,
/// refined by bisection.
fn data_meets(d: &Data, g: &dyn Fn(f64) -> Option<f64>, ts: &[f64]) -> Vec<(f64, Option<f64>)> {
    let mut out = vec![];
    if d.xs.len() == 1 && g(d.xs[0]) == Some(d.ys[0]) { out.push((d.xs[0], Some(d.ys[0]))); }
    for ((x1, y1), (x2, y2)) in d.segments() {
        if x1 == x2 { continue; }
        let lerp = |t: f64| y1 + (y2 - y1) * (t - x1) / (x2 - x1);
        let h = |t: f64| Some(lerp(t) - g(t)?);
        let (a, b) = (x1.min(x2), x1.max(x2));
        let mut grid = vec![a];
        grid.extend(ts.iter().copied().filter(|t| *t > a && *t < b));
        grid.push(b);
        for w in grid.windows(2) {
            let (Some(u), Some(v)) = (h(w[0]), h(w[1])) else { continue };
            if u == 0.0 {
                out.push((w[0], Some(lerp(w[0]))));
            } else if v != 0.0 && (u > 0.0) != (v > 0.0) {
                let t = bisect(&h, w[0], w[1]);
                out.push((t, Some(lerp(t))));
            }
        }
        if h(b) == Some(0.0) { out.push((b, Some(lerp(b)))); }
    }
    out
}

/// Where two data polylines cross: intersections of their segments (parallel ones, overlapping or not, don't count).
fn data_cross(p: &Data, q: &Data) -> Vec<(f64, Option<f64>)> {
    let mut out = vec![];
    for (a1, a2) in p.segments() {
        for (b1, b2) in q.segments() {
            if a1.0.max(a2.0) < b1.0.min(b2.0) || b1.0.max(b2.0) < a1.0.min(a2.0) || a1.1.max(a2.1) < b1.1.min(b2.1) || b1.1.max(b2.1) < a1.1.min(a2.1) {
                continue;
            }
            // a1 + s·r = b1 + u·v with 0 ≤ s, u ≤ 1
            let (r, v, w) = ((a2.0 - a1.0, a2.1 - a1.1), (b2.0 - b1.0, b2.1 - b1.1), (b1.0 - a1.0, b1.1 - a1.1));
            let den = r.0 * v.1 - r.1 * v.0;
            if den == 0.0 { continue; }
            let (s, u) = ((w.0 * v.1 - w.1 * v.0) / den, (w.0 * r.1 - w.1 * r.0) / den);
            if (0.0..=1.0).contains(&s) && (0.0..=1.0).contains(&u) {
                out.push((a1.0 + s * r.0, Some(a1.1 + s * r.1)));
            }
        }
    }
    out
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
    use crate::api::line;

    #[test]
    fn graph_axes_respect_constants_and_calculus_scope() {
        let req = |src: &str| GraphReq {
            cfg: Cfg::default(), curves: vec![crate::test_support::curve(src, &Env::default())],
            xrange: vec![XV::Num(-1.0), XV::Num(1.0)], yrange: vec![],
            marks: vec![], ymark: vec![], samples: 20,
        };
        let mut env = Env::default();
        for src in [
            "integral_(-infinity)^t sigma(tau) dif tau",
            "integrate(exp(-u^2), u, -infinity, t)",
            "integrate(exp(-u^2), u, t, infinity)",
            "integrate(exp(-u^2), u, -∞, t)",
            "integrate(1, u, 0, t)",
            "integrate(1, t, 0, t)",
            "integrate(integrate(1, u, 0, u), u, 0, t)",
            "sum(t, k, 0, 2)",
            "sum(0, k, 0, t)",
            "sum(integrate(t, u, 0, 1), k, 0, 2)",
            "t + pi + e",
        ] {
            let r = req(src);
            let units = Units::lexical(&Cfg::default(), &env, &r.curves[0].src).unwrap();
            assert_eq!(read(&env, &r, &units).unwrap().x, "t", "{src}");
        }
        let r = req("integrate(1, u, 0, t) + u");
        let error = read(&env, &r, &Units::default()).err().unwrap();
        assert!(error.contains("'t', 'u' vary"), "{error}");
        for src in ["tau := 1", "sigma(u) := e^u"] {
            let r = crate::test_support::req(src, &env);
            let out = line(&mut env, &r);
            assert!(out.errors.is_empty(), "{:?}", out.errors);
        }
        let plot = graph(&env, &req("integral_(-infinity)^t sigma(tau) dif tau")).unwrap();
        let points: Vec<_> = plot.series.iter().flat_map(|s| &s.points).collect();
        assert!(points.len() >= 20);
        assert!(points.iter().all(|(t, y)| (y - t.exp()).abs() < 1e-8));
    }

    #[test]
    fn ticks_terminate_for_explicit_ranges_one_ulp_wide() {
        let plot = graph(&Env::default(), &GraphReq {
            cfg: Cfg::default(), curves: vec![crate::test_support::curve("x", &Env::default())],
            xrange: vec![XV::Num(1.0), XV::Num(1.0 + f64::EPSILON)],
            yrange: vec![XV::Num(1.0), XV::Num(1.0 + f64::EPSILON)],
            marks: vec![], ymark: vec![], samples: 40,
        }).unwrap();
        assert!(!plot.series.is_empty());
        assert!((1..=20).contains(&plot.xticks.len()));
        assert!((1..=20).contains(&plot.yticks.len()));
    }

    #[test]
    fn graphs() {
        let mut env = Env::default();
        for s in ["R := 50 Ω", "P0 := 2 W", "U0 := 10 V"] {
            let o = { let r = crate::test_support::req(s, &env); line(&mut env, &r) };
            assert!(o.errors.is_empty());
        }
        let req = |curves: Vec<&str>, ymark: Vec<&str>| GraphReq {
            cfg: Cfg::default(),
            curves: curves.into_iter().map(|c| crate::test_support::curve(c, &Env::default())).collect(),
            xrange: vec![],
            yrange: vec![],
            marks: vec![(Mark::Key("auto".into()), true)],
            ymark: ymark.into_iter().map(|c| XV::Src(crate::test_support::curve(c, &Env::default()))).collect(),
            samples: 100,
        };
        let g = graph(&env, &req(vec!["P(U: V) = U^2/R"], vec!["P0"])).unwrap();
        assert!(g.xlabel.contains("\"V\")"), "{}", g.xlabel);
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
        let run = |env: &mut Env, s: &str| { let r = crate::test_support::req(s, env); line(env, &r) };
        assert!(run(&mut env, "X(x) := x^2 + 1").errors.is_empty());
        let req = |c: &str| GraphReq { cfg: Cfg::default(), curves: vec![crate::test_support::curve(c, &Env::default())], xrange: vec![], yrange: vec![], marks: vec![], ymark: vec![], samples: 50 };
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

    fn data_env(lines: &[&str]) -> Env {
        let mut env = Env::default();
        for s in lines {
            let o = { let r = crate::test_support::req(s, &env); line(&mut env, &r) };
            assert!(o.errors.is_empty(), "{s}: {:?}", o.errors);
        }
        env
    }

    fn data_req(curves: &[&str], marks: Vec<Mark>) -> GraphReq {
        GraphReq {
            cfg: Cfg::default(),
            curves: curves.iter().map(|c| crate::test_support::curve(c, &Env::default())).collect(),
            xrange: vec![], yrange: vec![],
            marks: marks.into_iter().map(|m| (m, true)).collect(),
            ymark: vec![], samples: 200,
        }
    }

    fn key(k: &str) -> Mark { Mark::Key(k.into()) }

    #[test]
    fn data_against_its_index() {
        let env = data_env(&["y := vec(1, 4, 9)"]);
        for src in ["y", "2 y", "vec(1, 4, 9)"] {
            let g = graph(&env, &data_req(&[src], vec![])).unwrap();
            assert_eq!(g.series.len(), 1, "{src}");
            let s = &g.series[0];
            let f = if src == "2 y" { 2.0 } else { 1.0 };
            assert_eq!(s.points, vec![(0.0, f), (1.0, 4.0 * f), (2.0, 9.0 * f)], "{src}");
            assert!(s.dots && s.data);
            assert_eq!((g.x0, g.x1), (0.0, 2.0));
            assert!(g.y0 <= f && g.y1 >= 9.0 * f);
        }
        let g = graph(&env, &data_req(&["y"], vec![])).unwrap();
        assert_eq!(g.ylabel, name("y"));
        // a vector with a free variable is still several curves over it
        let g = graph(&env, &data_req(&["vec(t, t^2)"], vec![])).unwrap();
        assert!(g.series.len() >= 2 && !g.series[0].dots && !g.series[0].data);
        // a single sample is a dot
        let env = data_env(&["one := vec(3)"]);
        let g = graph(&env, &data_req(&["one"], vec![])).unwrap();
        assert_eq!(g.series[0].points, vec![(0.0, 3.0)]);
    }

    #[test]
    fn preferred_units_scale_both_axes_and_keep_the_requested_prefix() {
        let env = data_env(&["tau := vec(0, 1, 2) * 1 s", "y := vec(0, 2, 4) * 1 V"]);
        let mut req = data_req(&["y(tau)"], vec![]);
        req.cfg.preferred_units = ["kg", "ms", "mV", "h"].iter().map(|u| crate::test_support::unit(u)).collect();
        let g = graph(&env, &req).unwrap();
        assert!(g.xlabel.contains("\"ms\""), "{}", g.xlabel);
        assert!(g.ylabel.contains("\"mV\""), "{}", g.ylabel);
        assert_eq!(g.series[0].points, vec![(0.0, 0.0), (1000.0, 2000.0), (2000.0, 4000.0)]);
        assert_eq!((g.x0, g.x1), (0.0, 2000.0));
        let mut req = data_req(&["U(t: s) = t * 1 V/s"], vec![]);
        req.cfg.preferred_units = ["ms", "mV"].iter().map(|u| crate::test_support::unit(u)).collect();
        req.xrange = vec![XV::Num(0.0), XV::Num(2.0)];
        let g = graph(&Env::default(), &req).unwrap();
        assert_eq!((g.x0, g.x1), (0.0, 2.0));
        assert!(g.ylabel.contains("mV"), "{}", g.ylabel);
        assert!(g.series.iter().flat_map(|s| &s.points).all(|(x, y)| (x - y).abs() < 1e-8));
        req.yrange = vec![XV::Num(0.0), XV::Num(4.0)];
        let g = graph(&Env::default(), &req).unwrap();
        assert_eq!((g.y0, g.y1), (0.0, 4.0));
    }

    #[test]
    fn data_over_a_vector_with_units() {
        let env = data_env(&["tau := vec(0, 1, 2, 3) * 1 s", "y := vec(0 V, 2 V, 1 V, -1 V)", "ms := vec(0, 1, 2, 3) * 1 #ms"]);
        let g = graph(&env, &data_req(&["y(tau)"], vec![])).unwrap();
        assert_eq!(g.series[0].points, vec![(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, -1.0)]);
        assert!(g.xlabel.contains("τ") && g.xlabel.contains("\"s\")"), "{}", g.xlabel);
        assert!(g.ylabel.contains("\"V\")"), "{}", g.ylabel);
        assert_eq!((g.x0, g.x1), (0.0, 3.0));
        // the x prefix fits the values, the argument may be any vector expression
        let g = graph(&env, &data_req(&["y(ms)"], vec![])).unwrap();
        assert!(g.xlabel.contains("ms"), "{}", g.xlabel);
        assert_eq!(g.series[0].points[3].0, 3.0);
        let g = graph(&env, &data_req(&["y(2 tau)"], vec![])).unwrap();
        assert_eq!(g.series[0].points[3].0, 6.0);
        // an explicit range cuts the polyline where it leaves
        let mut req = data_req(&["y(tau)"], vec![]);
        req.xrange = vec![XV::Num(0.5), XV::Num(2.5)];
        let g = graph(&env, &req).unwrap();
        assert_eq!(g.series[0].points, vec![(0.5, 1.0), (1.0, 2.0), (2.0, 1.0), (2.5, 0.0)]);
        // the range's units are checked against the data's
        req.xrange = vec![XV::Src(crate::test_support::curve("2 V", &Env::default()))];
        assert!(graph(&env, &req).is_err());
    }

    #[test]
    fn data_errors() {
        let env = data_env(&["tau := vec(0, 1, 2) * 1 s", "y := vec(1, 2, 3, 4) * 1 V", "u := vec(1 V, 2 A, 3 V)", "w := vec(1, a, 3)", "c := 5", "M := mat(1, 2; 3, 4)"]);
        let e = graph(&env, &data_req(&["y(tau)"], vec![])).unwrap_err();
        assert!(e.contains("4 elements") && e.contains("3"), "{e}");
        let e = graph(&env, &data_req(&["u(tau)"], vec![])).unwrap_err();
        assert!(e.contains("u: element 1") && e.contains("one unit"), "{e}");
        let e = graph(&env, &data_req(&["w(tau)"], vec![])).unwrap_err();
        assert!(e.contains("w: element 1 has no value") && e.contains("'a'"), "{e}");
        let e = graph(&env, &data_req(&["y(t)"], vec![])).unwrap_err();
        assert!(e.contains("'t' isn't defined"), "{e}");
        let e = graph(&env, &data_req(&["c(tau)"], vec![])).unwrap_err();
        assert!(e.contains("isn't a vector"), "{e}");
        let e = graph(&env, &data_req(&["y(c)"], vec![])).unwrap_err();
        assert!(e.contains("isn't a vector"), "{e}");
        let e = graph(&env, &data_req(&["M"], vec![])).unwrap_err();
        assert!(e.contains("2×2 matrix"), "{e}");
        let e = graph(&env, &data_req(&["z := y(tau)"], vec![])).unwrap_err();
        assert!(e.contains("defines nothing"), "{e}");
        // x and y units must agree with the other curves
        let env = data_env(&["tau := vec(0, 1, 2) * 1 s", "y := vec(1, 2, 3) * 1 V", "p(t: A) := t * 1 V/A", "q(t: s) := t * 1 A/s"]);
        let e = graph(&env, &data_req(&["y(tau)", "p(tau)"], vec![])).unwrap_err();
        assert!(e.contains("one unit"), "{e}");
        let e = graph(&env, &data_req(&["y(tau)", "q(tau)"], vec![])).unwrap_err();
        assert!(e.contains("different units"), "{e}");
        let e = graph(&env, &data_req(&["y(tau)", "t^2"], vec![])).unwrap_err();
        assert!(e.contains("one x axis"), "{e}");
    }

    #[test]
    fn data_with_functions() {
        let env = data_env(&["tau := vec(0, 1, 2, 3, 4) * 1 s", "y := vec(0, 1, 4, 9, 16) * 1 V", "c(t: s) := 2 V/s * t + 1 V"]);
        for curves in [["y(tau)", "c(tau)"], ["y(tau)", "2 V/s * tau + 1 V"]] {
            let g = graph(&env, &data_req(&curves, vec![key("crossings")])).unwrap();
            // the function is sampled over the data's extent
            assert_eq!((g.x0, g.x1), (0.0, 4.0), "{curves:?}");
            let fun: Vec<_> = g.series.iter().filter(|s| s.curve == 1).flat_map(|s| &s.points).collect();
            assert!(fun.len() > 100 && fun.iter().all(|p| (p.1 - (2.0 * p.0 + 1.0)).abs() < 1e-9));
            // a label for each
            assert!(g.series.iter().all(|s| s.label.is_some()));
            // y = 2t + 1 meets the polyline between 2 and 3: 4 + 5 (t − 2) = 2 t + 1 → t = 7/3
            assert!(g.points.iter().any(|p| (p.x - 7.0 / 3.0).abs() < 1e-9 && (p.y - 17.0 / 3.0).abs() < 1e-9), "{:?}", g.points);
            assert!(g.xlabel.contains("τ"), "{}", g.xlabel);
        }
        // xrange still decides
        let mut req = data_req(&["y(tau)", "c(tau)"], vec![]);
        req.xrange = vec![XV::Num(-1.0), XV::Num(6.0)];
        let g = graph(&env, &req).unwrap();
        assert_eq!((g.x0, g.x1), (-1.0, 6.0));
        // data against the index next to a function of the index
        let env = data_env(&["y := vec(0, 1, 4)"]);
        let g = graph(&env, &data_req(&["y", "k"], vec![key("auto")])).unwrap();
        assert!(g.points.iter().any(|p| p.x == 0.0 && p.y == 0.0) && g.points.iter().any(|p| p.x == 1.0 && p.y == 1.0), "{:?}", g.points);
    }

    #[test]
    fn data_marks() {
        let env = data_env(&["tau := vec(0, 1, 2, 3, 4) * 1 s", "y := vec(-1, 1, 3, 1, -3) * 1 V", "z := vec(2, 2, 2, 0, 0) * 1 V", "t0 := 2.5 s"]);
        let pts = |curves: &[&str], marks: Vec<Mark>| graph(&env, &data_req(curves, marks)).unwrap().points;
        let has = |ps: &[Point], x: f64, y: f64| ps.iter().any(|p| (p.x - x).abs() < 1e-9 && (p.y - y).abs() < 1e-9);
        // roots by sign changes, interpolated
        let ps = pts(&["y(tau)"], vec![key("roots")]);
        assert!(has(&ps, 0.5, 0.0) && has(&ps, 3.25, 0.0) && ps.len() == 2, "{ps:?}");
        // the local extreme samples
        let ps = pts(&["y(tau)"], vec![key("extrema")]);
        assert!(has(&ps, 2.0, 3.0) && ps.len() == 1, "{ps:?}");
        // x values interpolate: a number in s and an expression
        let ps = pts(&["y(tau)"], vec![Mark::Val(XV::Num(0.25)), Mark::Val(XV::Src(crate::test_support::curve("t0", &Env::default())))]);
        assert!(has(&ps, 0.25, -0.5) && has(&ps, 2.5, 2.0), "{ps:?}");
        // data–data crossings, also along a flat part
        let ps = pts(&["y(tau)", "z(tau)"], vec![key("crossings")]);
        assert!(has(&ps, 1.5, 2.0) && has(&ps, 3.25, 0.0) && ps.len() == 2, "{ps:?}");
        // y marks
        let mut req = data_req(&["y(tau)"], vec![]);
        req.ymark = vec![XV::Num(2.0)];
        let ps = graph(&env, &req).unwrap().points;
        assert!(has(&ps, 1.5, 2.0) && has(&ps, 2.5, 2.0), "{ps:?}");
        // where x runs back, an x value is on several segments
        let env = data_env(&["xs := vec(0, 2, 1)", "ys := vec(0, 2, 4)"]);
        let ps = graph(&env, &data_req(&["ys(xs)"], vec![Mark::Val(XV::Num(1.5))])).unwrap().points;
        assert!(has(&ps, 1.5, 1.5) && has(&ps, 1.5, 3.0), "{ps:?}");
    }
}
