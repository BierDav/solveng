//! What the Typst plugin calls: requests and answers as plain data.

use std::collections::BTreeSet;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::env::{Env, Kind, Lower};
use crate::eval::{dim_of, eval_f, value};
use crate::expr::{Expr, simp};
use crate::format::{self, Cfg, pick, shown};
use crate::parse::{self, Ctx, Mode, ParamType};
use crate::render::R;
use crate::sheet::{Out, Sheet};
use crate::units::{UnitExpr, Units};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LineReq {
    pub src: String,
    pub mode: Mode,
    pub cfg: Cfg,
    #[serde(default = "yes")]
    pub listed: bool,
}

fn yes() -> bool {
    true
}

/// Calculates a line against the worksheet so far.
pub fn line(env: &mut Env, req: &LineReq) -> Out {
    match Sheet::new(env, &req.cfg, req.mode, req.listed) {
        Ok(s) => s.run(&req.src),
        Err(e) => Out { errors: vec![e], ..Out::default() },
    }
}

/// `#value("…")`: the result of an expression alone, as markup, against the worksheet so far, which it leaves as
/// it is. An expression is shown like `… =` (a clause of its own picks the unit or form: `"t = min"`), a
/// function's name alone like its body. Nothing can be defined here.
pub fn show_value(env: &Env, req: &LineReq) -> Result<String, String> {
    let src = req.src.trim();
    let chars: Vec<char> = src.chars().collect();
    if let Ok(Some(n)) = solveng_names::scan(&chars, 0)
        && n.end == chars.len()
        && let Some(f) = env.funs.get(&n.value)
    {
        let units = Units::configured(&req.cfg)?;
        return Ok(R { units: &units, cfg: &req.cfg, exact: true, sub: None }.expr(&f.body).0);
    }
    let run = |src: &str| line(&mut env.clone(), &LineReq { src: src.into(), listed: false, ..req.clone() });
    // as written when it shows a result (t = min, x ->), else its value
    let written = run(src);
    let out = if written.mark.is_some() || !written.defined.is_empty() { written } else { run(&format!("{src} =")) };
    if !out.defined.is_empty() {
        let n = &out.defined[0];
        return Err(format!("nothing can be defined here – define {n} in the worksheet with :=, then #value(\"{n}\") shows it"));
    }
    if let Some(e) = out.errors.into_iter().next() {
        return Err(e);
    }
    out.result.ok_or_else(|| "this has no value to show – an expression like \"U / 2\" or \"f(2)\"".to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DefinedReq {
    pub cfg: Cfg,
}

/// One definition, for defined() and editors.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Row {
    pub name: String,
    pub kind: Kind,
    pub listed: bool,
    /// The name as markup (with its parameters for a function).
    pub lhs: String,
    /// The value, and the SI value when asked for and different.
    pub values: Vec<String>,
    /// "=" or the relation of an inequality's boundary.
    pub rel: String,
    /// The value in plain text: "2.988 V", "(x: A): V = x [2 Ω]" (a function's return unit when its parameters
    /// have units).
    pub plain: String,
    /// The value in SI units and the unit it's shown in, when it has a value.
    pub si: Option<f64>,
    pub unit: Option<String>,
}

pub fn defined(env: &Env, req: &DefinedReq) -> Result<Vec<Row>, String> {
    let units = Units::configured(&req.cfg)?;
    let cfg = &req.cfg;
    let r = R { units: &units, cfg, exact: true, sub: None };
    let mut rows = vec![];
    for e in &env.order {
        if let Some(f) = env.funs.get(&e.name) {
            let ps: Vec<String> = f.params.iter().map(|p| format::name(&p.name)).collect();
            let ptext: Vec<String> = f
                .params
                .iter()
                .map(|p| match &p.ty {
                    Some(ParamType::Unit(u)) => format!("{}: {}", p.name, u.text()),
                    Some(ParamType::Any) => format!("{}: any", p.name),
                    None => p.name.clone(),
                })
                .collect();
            rows.push(Row {
                name: e.name.clone(),
                kind: Kind::Function,
                listed: e.listed,
                lhs: format!("{}({})", format::name(&e.name), ps.join(", ")),
                values: vec![r.expr(&f.body).0],
                rel: "=".into(),
                // what it returns, when every parameter has a unit: (x: Ω): A = x/R
                plain: match f.ret {
                    Some(d) => format!("({}): {} = {}", ptext.join(", "), if d.is_one() { "1".into() } else { units.auto(&d).text() }, f.src),
                    None => format!("({}) = {}", ptext.join(", "), f.src),
                },
                si: None,
                unit: None,
            });
            continue;
        }
        let Some(v) = env.vars.get(&e.name) else { continue };
        let bound = env.bind_with_units(&v.expr, &BTreeSet::new(), &units).unwrap_or(v.expr.clone());
        let mut si = None;
        let mut shown_unit = None;
        let (values, plain) = match value(&bound) {
            Ok(val) => {
                si = Some(val.v.f());
                let s = pick(&val, v.unit.as_ref(), None, &units, cfg)?;
                shown_unit = Some(s.u.text());
                let mut vs = vec![shown(&s, &units, cfg).0];
                if cfg.show_si && !val.d.is_one() {
                    let si = units.auto(&val.d);
                    if si != s.u {
                        let s2 = pick(&val, Some(&si), None, &units, cfg)?;
                        vs.push(shown(&s2, &units, cfg).0);
                    }
                }
                (vs, format::plain(&val, v.unit.as_ref(), &units, cfg))
            }
            Err(_) => {
                fn vector_plain(e: &Expr, want: Option<&UnitExpr>, units: &Units, cfg: &Cfg) -> String {
                    if let Expr::Vector(v) = e {
                        return format!("[{}]", v.iter().map(|e| vector_plain(e, want, units, cfg)).collect::<Vec<_>>().join(", "));
                    }
                    value(e).map(|v| format::plain(&v, want, units, cfg)).unwrap_or_default()
                }
                let plain = if matches!(bound, Expr::Vector(_)) { vector_plain(&bound, v.unit.as_ref(), &units, cfg) } else { String::new() };
                (vec![r.expr(&simp(&bound)).0], plain)
            },
        };
        rows.push(Row {
            name: e.name.clone(),
            kind: e.kind,
            listed: e.listed,
            lhs: format::name(&e.name),
            values,
            rel: v.rel.map(crate::render::rel).unwrap_or("=").to_string(),
            plain,
            si,
            unit: shown_unit,
        });
    }
    Ok(rows)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitReq {
    pub cfg: Cfg,
    pub src: String,
}

/// Whether text reads as a unit (for restyling casts and types in an equation as written): its markup.
pub fn unit_markup(req: &UnitReq) -> Option<String> {
    let units = Units::configured(&req.cfg).ok()?;
    let s = req.src.trim();
    if s == "any" || s == "\"any\"" {
        return Some(format::unit(&UnitExpr::one("any"), &units, &req.cfg));
    }
    let p = units.parse_simplified(&s.replace('"', " ")).ok()?;
    Some(format::unit(&p.u, &units, &req.cfg))
}

/// Problems with the option units: none or the message.
pub fn check_units(cfg: &Cfg) -> Option<String> {
    Units::configured(&cfg).err()
}

/// Problems with the option functions: none or the message.
pub fn check_functions(cfg: &Cfg) -> Option<String> {
    crate::builtins::Functions::configured(cfg).err()
}

// ---------- lambdify ----------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LamReq {
    pub cfg: Cfg,
    pub src: String,
    pub mode: Mode,
    pub params: Vec<String>,
    pub unit: Option<String>,
    #[serde(default)]
    pub units: IndexMap<String, String>,
}

/// An expression ready to evaluate: parameters in order, their unit factors, the result's factor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lam {
    pub expr: Expr,
    pub params: Vec<String>,
    /// Factor to SI of plain numbers per parameter.
    pub factors: Vec<f64>,
    /// Dimension per parameter with a unit (as exponents ×60), for checking values passed as text.
    pub dims: Vec<Option<crate::dim::Dim>>,
    pub out: f64,
    pub cfg: Cfg,
}

pub fn lambdify(env: &Env, req: &LamReq) -> Result<Lam, String> {
    let units = Units::configured(&req.cfg)?;
    let is_fn = |n: &str| env.funs.contains_key(n);
    let is_var = |n: &str| env.vars.contains_key(n);
    let functions = crate::builtins::Functions::configured(&req.cfg)?;
    let ctx = Ctx { units: &units, functions: &functions, mode: req.mode, is_fn: &is_fn, is_var: &is_var, strict: req.cfg.strict };
    let line = parse::parse_line(&req.src, &ctx)?;
    let [parse::Stmt::Expr(a)] = line.stmts.as_slice() else { return Err("an expression, like \"U^2/R\"".into()) };
    let ps: Vec<String> = req.params.iter().map(|p| parse::greek(p)).collect();
    let mut lo = Lower::new(env, &units);
    lo.params = ps.iter().cloned().collect();
    let e = simp(&lo.expr(a)?);
    let ps = if ps.is_empty() { e.syms() } else { ps };
    let miss: Vec<String> = e.syms().into_iter().filter(|s| !ps.contains(s)).collect();
    if !miss.is_empty() {
        return Err(format!("{} is neither defined nor a parameter", miss.iter().map(|m| format!("'{m}'")).collect::<Vec<_>>().join(", ")));
    }
    let mut factors = vec![];
    let mut dims = vec![];
    for p in &ps {
        match req.units.iter().find(|(k, _)| parse::greek(k) == *p) {
            Some((_, u)) => {
                let pu = units.parse(u, false).map_err(|e| format!("units: {e}"))?;
                factors.push(pu.f.f());
                dims.push(Some(pu.d));
            }
            None => {
                factors.push(1.0);
                dims.push(None);
            }
        }
    }
    for k in req.units.keys() {
        if !ps.contains(&parse::greek(k)) {
            return Err(format!("units: '{k}' is not a parameter (parameters: {})", ps.join(", ")));
        }
    }
    let mut out = 1.0;
    if let Some(u) = &req.unit {
        let pu = units.parse(u, false).map_err(|e| format!("unit: {e}"))?;
        out = pu.f.f();
        // with the units of all parameters known, the unit of the result can be checked now
        if dims.iter().all(|d| d.is_some()) {
            let pd = |n: &str| ps.iter().position(|p| p == n).and_then(|i| dims[i]);
            if let Some(d) = dim_of(&e, &pd)? {
                if d != pu.d {
                    return Err(format!("the result has unit {}, not {u}", format::dim_text(&d, &units)));
                }
            }
        }
    }
    Ok(Lam { expr: e, params: ps, factors, dims, out, cfg: req.cfg.clone() })
}

/// An argument: a number (in the parameter's unit) or a value as text ("5 V").
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    Num(f64),
    Text(String),
}

pub fn lam_eval(l: &Lam, args: &[Arg]) -> Result<Option<f64>, String> {
    if args.len() != l.params.len() {
        return Err(format!("the function takes {} argument(s) ({}), got {}", l.params.len(), l.params.join(", "), args.len()));
    }
    let units = Units::configured(&l.cfg)?;
    let mut xs = vec![];
    for (i, a) in args.iter().enumerate() {
        xs.push(match a {
            Arg::Num(x) => x * l.factors[i],
            Arg::Text(s) => {
                let p = units.parse(s, true).map_err(|e| format!("{}: {e}", l.params[i]))?;
                if let Some(d) = l.dims[i] {
                    if d != p.d {
                        return Err(format!("{} = {s} doesn't have the unit of {}", l.params[i], l.params[i]));
                    }
                }
                p.f.f()
            }
        });
    }
    let r = eval_f(&l.expr, &|n| l.params.iter().position(|p| p == n).map(|i| xs[i]));
    Ok(r.map(|v| v / l.out))
}

/// A value of an expression of defined names (for graph ranges): (SI value, dims) or a message.
pub fn quantity(env: &Env, cfg: &Cfg, src: &str, mode: Mode) -> Result<(f64, crate::dim::Dim), String> {
    let units = Units::configured(&cfg)?;
    let is_fn = |n: &str| env.funs.contains_key(n);
    let is_var = |n: &str| env.vars.contains_key(n);
    let functions = crate::builtins::Functions::configured(cfg)?;
    let ctx = Ctx { units: &units, functions: &functions, mode, is_fn: &is_fn, is_var: &is_var, strict: cfg.strict };
    let line = parse::parse_line(src, &ctx)?;
    let [parse::Stmt::Expr(a)] = line.stmts.as_slice() else { return Err(format!("'{src}' isn't a value")) };
    let e = Lower::new(env, &units).expr(a)?;
    let v = value(&e).map_err(|e| e.message())?;
    Ok((v.v.f(), v.d))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClearReq {
    /// The names as written, in code or math: `x`, `f`, `f(x)` (a function's parameters are ignored).
    pub names: Vec<(String, Mode)>,
}

/// Forgets the names; problems for what isn't a name or isn't defined.
pub fn clear(env: &mut Env, req: &ClearReq) -> Vec<String> {
    let mut errors = vec![];
    for (src, mode) in &req.names {
        let name = match parse::lex(src, *mode).as_deref() {
            Ok([parse::Tok::Name(n) | parse::Tok::Quoted(n), rest @ ..])
                if rest.is_empty() || (rest.first() == Some(&parse::Tok::Op("(")) && rest.last() == Some(&parse::Tok::Op(")"))) =>
            {
                n.clone()
            }
            _ => {
                errors.push(format!("clear: '{}' isn't a name – clear(\"x\", \"f\") or clear(all: true)", src.trim()));
                continue;
            }
        };
        if !env.clear(&name) {
            errors.push(format!("clear: '{name}' isn't defined"));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(env: &mut Env, s: &str) {
        let o = line(env, &LineReq { src: s.into(), mode: Mode::Code, cfg: Cfg::default(), listed: true });
        assert!(o.errors.is_empty(), "{s}: {:?}", o.errors);
    }

    #[test]
    fn clear_forgets_values_and_functions() {
        let mut env = Env::default();
        run(&mut env, "x := 2");
        run(&mut env, "f(t) := t^2");
        run(&mut env, "tau := 3");
        let req = |names: &[&str]| ClearReq { names: names.iter().map(|n| (n.to_string(), Mode::Code)).collect() };
        assert!(clear(&mut env, &req(&["x", "f(t)", "tau"])).is_empty());
        assert!(env.vars.is_empty() && env.funs.is_empty() && env.order.is_empty());
        assert_eq!(clear(&mut env, &req(&["x", "1 + y"])), ["clear: 'x' isn't defined", "clear: '1 + y' isn't a name – clear(\"x\", \"f\") or clear(all: true)"]);
        // a value and a function share a name: defining one replaces the other
        run(&mut env, "g(t) := t");
        run(&mut env, "g := 1");
        assert!(env.funs.is_empty());
        assert!(clear(&mut env, &req(&["g()"])).is_empty());
        assert!(env.vars.is_empty());
    }

    #[test]
    fn lambdify_works() {
        let mut env = Env::default();
        run(&mut env, "R := 50 Ω");
        let mut units = IndexMap::new();
        units.insert("U".to_string(), "V".to_string());
        let l = lambdify(&env, &LamReq { cfg: Cfg::default(), src: "U^2/R".into(), mode: Mode::Code, params: vec!["U".into()], unit: Some("W".into()), units }).unwrap();
        assert_eq!(lam_eval(&l, &[Arg::Num(10.0)]).unwrap(), Some(2.0));
        assert_eq!(lam_eval(&l, &[Arg::Text("20000 mV".into())]).unwrap(), Some(8.0));
        let l = lambdify(&env, &LamReq { cfg: Cfg::default(), src: "sqrt(x)".into(), mode: Mode::Code, params: vec![], unit: None, units: IndexMap::new() }).unwrap();
        assert_eq!(lam_eval(&l, &[Arg::Num(-1.0)]).unwrap(), None);
    }

    #[test]
    fn rows() {
        let mut env = Env::default();
        run(&mut env, "R := 7.2 kΩ");
        run(&mut env, "f(x) := x^2");
        let rs = defined(&env, &DefinedReq { cfg: Cfg::default() }).unwrap();
        assert_eq!(rs.len(), 2);
        assert_eq!(rs[0].plain, "7.2 kΩ");
        assert_eq!(rs[1].plain, "(x) = x^2");
    }

    #[test]
    fn rows_show_what_functions_return() {
        let mut env = Env::default();
        run(&mut env, "R := 2 Ω");
        run(&mut env, "o(x: Ω) := x");
        run(&mut env, "i(u: V) := u/R");
        run(&mut env, "p(u: V, t: s) := u^2/R t");
        run(&mut env, "q(x: m, y: m) := x/y");
        run(&mut env, "a(x: any) := 2 x");
        run(&mut env, "n(x) := x");
        let rs = defined(&env, &DefinedReq { cfg: Cfg::default() }).unwrap();
        let plain = |name: &str| rs.iter().find(|r| r.name == name).unwrap().plain.clone();
        assert_eq!(plain("o"), "(x: Ω): Ω = x");
        assert_eq!(plain("i"), "(u: V): A = u / R");
        assert_eq!(plain("p"), "(u: V, t: s): J = u^2 / R t");
        assert_eq!(plain("q"), "(x: m, y: m): 1 = x / y");
        // a type that follows what's passed, or none, leaves the result's unit open
        assert_eq!(plain("a"), "(x: any) = 2 x");
        assert_eq!(plain("n"), "(x) = x");
    }
}
