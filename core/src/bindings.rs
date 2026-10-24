//! Lexical values carry their meaning across the Typst/plugin boundary.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::{complex::Z, dim::Dim, env::{Env, Lower}, eval::{complex_value, value}, expr::{Expr, Val, simp}, format::Cfg, num::Num, parse::{self, Ast, Ctx, Stmt}, units::{UnitDef, UnitExpr, Units}};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Binding {
    pub id: String,
    pub identity: Option<String>,
    /// What calculations use: a unit's value, or a constant's expression (an exact symbol for the identities the
    /// engine knows).
    pub expr: Expr,
    /// A constant's own expression, as written.
    pub backing: Option<Expr>,
    pub unit: bool,
    pub units: IndexMap<String, UnitDef>,
}

/// A constant where it's used: its identity (how it's shown), what calculations use, and its expression.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Constant {
    pub identity: String,
    pub expr: Expr,
    pub backing: Expr,
}

/// An identity the engine calculates with exactly.
pub struct Known {
    pub identity: &'static str,
    pub symbol: Expr,
    pub value: Z,
    /// The value as text, for messages.
    pub text: &'static str,
}

pub const KNOWN: &[Known] = &[
    Known { identity: "pi", symbol: Expr::Pi, value: Z { re: std::f64::consts::PI, im: 0.0 }, text: "π" },
    Known { identity: "euler", symbol: Expr::E, value: Z { re: std::f64::consts::E, im: 0.0 }, text: "e" },
    Known { identity: "imaginary", symbol: Expr::I, value: Z { re: 0.0, im: 1.0 }, text: "i" },
];

/// The identity of an exact symbol: pi, euler, imaginary.
pub fn identity_of(e: &Expr) -> Option<&'static str> {
    KNOWN.iter().find(|k| k.symbol == *e).map(|k| k.identity)
}

impl Binding {
    pub fn encode(&self) -> Result<String, String> {
        Ok(serde_json::to_vec(self).map_err(|e| e.to_string())?.iter().map(|b| format!("{b:02x}")).collect())
    }
    pub fn decode(text: &str) -> Result<Self, String> {
        if text.len() % 2 != 0 || !text.is_ascii() { return Err("invalid lexical value".into()); }
        let bytes = (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i+2], 16).map_err(|_| "invalid lexical value")).collect::<Result<Vec<_>, _>>()?;
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid lexical value: {e}"))
    }
}

pub fn references(src: &str) -> Result<IndexMap<String, UnitDef>, String> {
    let mut units = IndexMap::new();
    for part in src.split('⟦').skip(1) {
        let (encoded, _) = part.split_once('⟧').ok_or("unclosed lexical value")?;
        units.extend(Binding::decode(encoded)?.units);
    }
    Ok(units)
}

#[derive(Deserialize)]
pub struct BindingReq {
    pub kind: String,
    #[serde(default)] pub src: String,
    pub identity: Option<String>,
    /// How a unit is shown (its markup), part of its identity.
    #[serde(default)] pub presentation: String,
    /// A unit's name in plain text.
    #[serde(default)] pub description: String,
    /// A unit shown as its expression: the engine writes it, as results are written ([`written`]).
    #[serde(default)] pub auto: bool,
}

/// How a unit binding is shown and joined: its display in plain text, whether it starts with a number, whether
/// it is more than one atom.
struct Shape { description: String, numeric: bool, compound: bool }

/// A formula in plain text, for a unit's description: 2.54 cm, 1/60 s.
fn plain(a: &Ast, units: &Units) -> String {
    match a {
        Ast::Num(raw, _) => raw.clone(),
        Ast::Qty(n, u) => format!("{} {}", plain(n, units), units.text(u)),
        Ast::Unit(u) => units.text(u),
        Ast::Name(n) => n.clone(),
        Ast::Bound(c) => c.identity.clone(),
        Ast::Neg(x) => format!("-{}", plain(x, units)),
        Ast::Add(ts) => ts.iter().enumerate().map(|(i, (neg, t))| {
            let sign = match (i, neg) { (0, true) => "-", (0, false) => "", (_, true) => " - ", (_, false) => " + " };
            format!("{sign}{}", plain(t, units))
        }).collect(),
        Ast::Mul(fs) => fs.iter().map(|(_, f)| plain(f, units)).collect::<Vec<_>>().join(" "),
        Ast::Div(x, y) => format!("{}/{}", plain(x, units), plain(y, units)),
        Ast::Pow(x, y) => format!("{}^{}", plain(x, units), plain(y, units)),
        _ => "…".into(),
    }
}

/// Whether a formula as written starts with a number: 2.54 cm, 1/60 s, a unit that does.
fn leads_numeric(a: &Ast, units: &Units) -> bool {
    match a {
        Ast::Num(..) | Ast::Qty(..) => true,
        Ast::Unit(u) => crate::format::unit_numeric(u, units),
        Ast::Mul(fs) => fs.first().is_some_and(|(_, f)| leads_numeric(f, units)),
        Ast::Div(x, _) | Ast::Pow(x, _) => leads_numeric(x, units),
        Ast::Add(ts) => ts.first().is_some_and(|(neg, t)| !neg && leads_numeric(t, units)),
        _ => false,
    }
}

impl Shape {
    fn of(ast: &Ast, units: &Units) -> Shape {
        let compound = match ast {
            Ast::Unit(u) => crate::format::unit_compound(u, units),
            Ast::Num(..) | Ast::Name(_) | Ast::Bound(_) => false,
            _ => true,
        };
        Shape { description: plain(ast, units), numeric: leads_numeric(ast, units), compound }
    }
    /// A display of its own, known by its text: a number first makes it numeric; it counts as one atom.
    fn custom(description: String) -> Shape {
        let numeric = description.trim_start().starts_with(|c: char| c.is_ascii_digit());
        Shape { description, numeric, compound: false }
    }
}

/// A unit's expression as written, in the worksheet's style: what a unit without a display of its own shows.
pub fn written(src: &str, cfg: &Cfg) -> Result<String, String> {
    let units = Units::lexical(cfg, &Env::default(), src)?;
    let ctx = Ctx { units: &units, is_fn: &|_| false, is_var: &|_| false };
    let line = parse::parse_line(src, &ctx)?;
    let [Stmt::Expr(ast)] = line.stmts.as_slice() else { return Err("a closed expression, without definitions".into()) };
    Ok(crate::render::R { units: &units, cfg, exact: false }.ast(ast).0)
}

pub fn construct(req: BindingReq) -> Result<Binding, String> {
    if !matches!(req.kind.as_str(), "unit" | "base-unit" | "constant") { return Err("unknown lexical value kind".into()); }
    let identity = req.identity.as_deref().map(str::trim);
    if identity.is_some_and(str::is_empty) { return Err(format!("{}: the identity must not be empty", req.kind)); }
    let mut units = Units::lexical(&Cfg::default(), &Env::default(), &req.src)?;
    let mut shape = Shape::custom(req.description.clone());
    // an automatic display is the expression as written: part of the unit's identity, as a display is
    let mut presentation = req.presentation.clone();
    let (expr, backing) = match req.kind.as_str() {
        "base-unit" => {
            let identity = identity.ok_or("base-unit: a nonempty dimension identity is required")?;
            (Expr::C(Val::new(Num::ONE, Dim::named(identity))), None)
        }
        kind => {
            let env = Env::default();
            let ctx = Ctx { units: &units, is_fn: &|_| false, is_var: &|_| false };
            let line = parse::parse_line(&req.src, &ctx)?;
            let [Stmt::Expr(ast)] = line.stmts.as_slice() else { return Err(format!("{kind}: a closed expression, without definitions")); };
            if line.trail.is_some() { return Err(format!("{kind}: an expression without a result clause")); }
            // a constant keeps its expression as written: √−1, which simplifies to i
            let written = Lower::new(&env, &units).expr(ast)?;
            let expr = simp(&written);
            if kind == "unit" {
                if req.auto {
                    shape = Shape::of(ast, &units);
                    presentation = crate::render::R { units: &units, cfg: &Cfg::default(), exact: false }.ast(ast).0;
                }
                let val = value(&expr).map_err(|_| "unit: an expression of numbers and units")?;
                if !val.v.f().is_finite() || val.v.f() <= 0.0 { return Err("unit: a positive, finite scale".into()); }
                (Expr::C(val), None)
            } else {
                let identity = identity.ok_or("constant: an identity is required")?;
                let (symbol, backing) = constant(identity, written)?;
                (symbol, Some(backing))
            }
        }
    };
    let semantic = serde_json::to_vec(&(&req.kind, &expr, &backing, &identity, &presentation)).map_err(|e| e.to_string())?;
    let id = format!("sv{}", blake3::hash(&semantic).to_hex().as_str().chars().map(|c| (b'a' + c.to_digit(16).unwrap() as u8) as char).collect::<String>());
    let is_unit = req.kind != "constant";
    let expr = if is_unit {
        let val = value(&expr).map_err(|_| "a unit must have a numeric value")?;
        units.map.insert(id.clone(), UnitDef { f: val.v, d: val.d.clone(), own: true, display: Some(shape.description.clone()), numeric: shape.numeric, compound: shape.compound });
        Expr::C(Val { u: Some(UnitExpr::one(&id)), ..val })
    } else { expr };
    Ok(Binding { id, identity: identity.map(str::to_owned), expr, backing, unit: is_unit, units: units.map })
}

/// A constant: an identity the engine knows becomes its exact symbol, once its expression has that value (π for
/// pi, √−1 for imaginary); any other stands for its expression, which must have a value. (symbol, expression)
fn constant(identity: &str, backing: Expr) -> Result<(Expr, Expr), String> {
    let known = KNOWN.iter().find(|k| k.identity == identity);
    // the engine's own identities are checked in the complex numbers, where √−1 has the value i
    let (z, d) = match known {
        Some(_) => crate::eval::eval_c(&backing),
        None => complex_value(&backing).map(|v| (v.z(), v.d().clone())).map_err(|e| e.message()),
    }.map_err(|m| format!("constant: the expression must have a value, of numbers, units and constants only – {m}"))?;
    if !z.is_finite() { return Err("constant: the value must be finite".into()); }
    let Some(k) = known else { return Ok((Expr::Const(identity.to_string(), Box::new(backing.clone())), backing)) };
    let same = |a: f64, b: f64| (a - b).abs() <= 1e-12 * b.abs().max(1.0);
    if !d.is_one() || !same(z.re, k.value.re) || !same(z.im, k.value.im) {
        return Err(format!("constant: the expression of '{identity}' must have its value, {}", k.text));
    }
    Ok((k.symbol.clone(), backing))
}
