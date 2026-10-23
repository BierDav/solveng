//! Units: the table (built-in and your own), prefixes, reading unit expressions, and picking a readable unit.

use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::dim::{BASE, Dim, OWN};
use crate::num::{Num, Rat};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitDef {
    /// Factor to SI.
    pub f: Num,
    pub d: Dim,
    /// Defined by the option `units` (or changed by it).
    pub own: bool,
}

pub const PREFIXES: &[(&str, i32)] = &[
    ("a", -18), ("f", -15), ("p", -12), ("n", -9), ("µ", -6), ("μ", -6), ("u", -6), ("m", -3), ("c", -2), ("d", -1),
    ("k", 3), ("M", 6), ("G", 9), ("T", 12),
];

fn prefix_exp(p: &str) -> Option<i32> {
    PREFIXES.iter().find(|(q, _)| *q == p).map(|(_, e)| *e)
}

/// How a definition of the option `units` reads: `(bar: "100 kPa", ü: auto, T: none, dozen: 12)`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnitOpt {
    /// "auto" (a base unit of its own) or "none" (removed) come as these strings.
    Key(KeyOpt),
    Num(f64),
    Str(String),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyOpt {
    Auto,
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Units {
    /// Every unit there is: the built-in ones (unless `builtin-units: false`) and those of the option `units`.
    pub map: IndexMap<String, UnitDef>,
    /// Names of the base units of your own, by slot.
    pub own_base: Vec<String>,
    /// Names the option `units` set to none: not read as units, still in definitions and prefixed.
    #[serde(default)]
    hidden: HashSet<String>,
    #[serde(default = "crate::format::d_true")]
    builtin_units: bool,
}

fn builtin() -> IndexMap<String, UnitDef> {
    let d = |e: [i32; 7]| Dim::si(e);
    let v = d([2, 1, -3, -1, 0, 0, 0]);
    let c = d([0, 0, 1, 1, 0, 0, 0]);
    let j = d([2, 1, -2, 0, 0, 0, 0]);
    let o = d([2, 1, -3, -2, 0, 0, 0]);
    let r = |n: i128, den: i128| Num::R(Rat::new(n, den).unwrap());
    let one = Num::ONE;
    let deg = Num::F(std::f64::consts::PI / 180.0);
    let list: Vec<(&str, Num, Dim)> = vec![
        ("m", one, d([1, 0, 0, 0, 0, 0, 0])),
        ("g", r(1, 1000), d([0, 1, 0, 0, 0, 0, 0])),
        ("s", one, d([0, 0, 1, 0, 0, 0, 0])),
        ("A", one, d([0, 0, 0, 1, 0, 0, 0])),
        ("K", one, d([0, 0, 0, 0, 1, 0, 0])),
        ("mol", one, d([0, 0, 0, 0, 0, 1, 0])),
        ("cd", one, d([0, 0, 0, 0, 0, 0, 1])),
        ("V", one, v),
        ("C", one, c),
        ("J", one, j),
        ("Ω", one, o),
        ("\u{2126}", one, o),
        ("Omega", one, o),
        ("W", one, d([2, 1, -3, 0, 0, 0, 0])),
        ("S", one, o.powi(-1)),
        ("F", one, d([-2, -1, 4, 2, 0, 0, 0])),
        ("H", one, d([2, 1, -2, -2, 0, 0, 0])),
        ("Hz", one, d([0, 0, -1, 0, 0, 0, 0])),
        ("N", one, d([1, 1, -2, 0, 0, 0, 0])),
        ("Pa", one, d([-1, 1, -2, 0, 0, 0, 0])),
        ("T", one, d([0, 1, -2, -1, 0, 0, 0])),
        ("Wb", one, d([2, 1, -2, -1, 0, 0, 0])),
        ("min", Num::int(60), d([0, 0, 1, 0, 0, 0, 0])),
        ("h", Num::int(3600), d([0, 0, 1, 0, 0, 0, 0])),
        ("Ah", Num::int(3600), c),
        ("As", one, c),
        ("Wh", Num::int(3600), j),
        ("Ws", one, j),
        // 1.602176634e-19 J, written out: no e-notation in input
        ("eV", Num::R(Rat::new(1602176634, 10i128.pow(28)).unwrap()), j),
        ("l", r(1, 1000), d([3, 0, 0, 0, 0, 0, 0])),
        ("L", r(1, 1000), d([3, 0, 0, 0, 0, 0, 0])),
        ("%", r(1, 100), Dim::ONE),
        ("‰", r(1, 1000), Dim::ONE),
        ("°C", one, d([0, 0, 0, 0, 1, 0, 0])),
        ("degC", one, d([0, 0, 0, 0, 1, 0, 0])),
        ("°", deg, Dim::ONE),
        ("deg", deg, Dim::ONE),
        ("rad", one, Dim::ONE),
    ];
    list.into_iter().map(|(n, f, d)| (n.to_string(), UnitDef { f, d, own: false })).collect()
}

/// Readable units for results, in the order they are tried.
const AUTO: &[&str] = &["V", "A", "Ω", "W", "J", "C", "F", "H", "S", "N", "Pa", "T", "Wb", "s", "m", "g", "K", "Hz", "mol", "cd"];

/// A unit written out: names with powers, those after a / negative. `kg m/s^2` → kg¹ m¹ s⁻².
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct UnitExpr(pub Vec<(String, i32)>);

impl UnitExpr {
    pub fn one(name: &str) -> UnitExpr {
        UnitExpr(vec![(name.to_string(), 1)])
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// As solveng writes it in text: `kg·m/s^2`.
    pub fn text(&self) -> String {
        let w = |ts: Vec<(&String, i32)>| {
            ts.iter().map(|(n, e)| if *e == 1 { (*n).clone() } else { format!("{n}^{e}") }).collect::<Vec<_>>().join("·")
        };
        let up: Vec<_> = self.0.iter().filter(|t| t.1 > 0).map(|(n, e)| (n, *e)).collect();
        let down: Vec<_> = self.0.iter().filter(|t| t.1 < 0).map(|(n, e)| (n, -e)).collect();
        if up.is_empty() && down.is_empty() {
            return String::new();
        }
        let mut s = if up.is_empty() { "1".to_string() } else { w(up) };
        if !down.is_empty() {
            s += "/";
            s += &w(down);
        }
        s
    }
}

/// Typeable spellings shown with their symbols: Omega → Ω, uA → µA, degC → °C.
pub fn display_name(n: &str, u: &Units) -> String {
    let normalized = solveng_names::normalize(n);
    let n = normalized.as_str();
    let sym = |r: &str| match r {
        "Omega" => "Ω".to_string(),
        "degC" => "°C".to_string(),
        "deg" => "°".to_string(),
        _ => r.to_string(),
    };
    if u.map.contains_key(n) {
        return sym(n);
    }
    let mut cs = n.chars();
    if let Some(p) = cs.next() {
        let rest: String = cs.collect();
        if u.prefixed(n).is_some() {
            return format!("{}{}", if p == 'u' { 'µ' } else { p }, sym(&rest));
        }
    }
    n.to_string()
}

impl Default for Units {
    fn default() -> Self {
        Units { map: builtin(), own_base: vec![], hidden: HashSet::new(), builtin_units: true }
    }
}

impl Units {
    /// The built-in units changed by the option `units`.
    pub fn build(opts: &IndexMap<String, UnitOpt>) -> Result<Units, String> {
        Self::build_visible(opts, true)
    }

    pub fn configured(cfg: &crate::format::Cfg) -> Result<Units, String> {
        Self::build_visible(&cfg.units, cfg.builtin_units)
    }

    /// The units of the option `units`, over the built-in ones if `builtin_units`, else alone.
    pub fn build_visible(opts: &IndexMap<String, UnitOpt>, builtin_units: bool) -> Result<Units, String> {
        let opts: IndexMap<String, UnitOpt> = opts.iter().map(|(name, value)| (solveng_names::normalize(name), value.clone())).collect();
        let map = if builtin_units { builtin() } else { IndexMap::new() };
        let mut units = Units { map, own_base: vec![], hidden: HashSet::new(), builtin_units };
        let mut resolver = Resolver { opts: &opts, builtin_units, cache: HashMap::new(), stack: Vec::new(), own_base: Vec::new() };
        for (name, value) in &opts {
            if !valid_name(name) { return Err(format!("units: '{name}' can't be a unit name (no spaces, digits or operators)")); }
            if matches!(value, UnitOpt::Key(KeyOpt::None)) {
                units.hidden.insert(name.clone());
            } else {
                let definition = resolver.resolve(name, true)?;
                units.map.insert(name.clone(), definition);
            }
        }
        units.own_base = resolver.own_base;
        Ok(units)
    }

    /// Whether a name reads as a unit where units are read by name (quoted, after a number, in a clause); hidden
    /// names (`T: none`) don't, though definitions and explicit references still reach them.
    pub fn visible(&self, name: &str) -> bool {
        let normalized = solveng_names::normalize(name);
        let name = normalized.as_str();
        self.known(name) && !self.hidden.contains(name)
    }

    /// A prefixed unit: kΩ → (factor, dims); a spelled Greek letter takes one only glued (k""Omega), as names do.
    pub fn prefixed(&self, n: &str) -> Option<(Num, Dim)> {
        let p = n.chars().next()?;
        let rest = &n[p.len_utf8()..];
        if rest.is_empty() || rest == "Omega" {
            return None;
        }
        let e = prefix_exp(&p.to_string())?;
        let def = self.map.get(rest)?;
        let pf = Num::R(Rat::int(10).powi(e as i64).unwrap());
        Some((def.f.mul(pf), def.d))
    }
    /// Whether n is a prefix on a unit, meaning just that: not when the table defines n itself as something
    /// else (with `km: "s"`, km is no kilometre).
    pub fn prefixed_form(&self, n: &str) -> bool {
        let Some((f, d)) = self.prefixed(n) else { return false };
        self.map.get(n).is_none_or(|u| u.d == d && crate::num::close(u.f.f(), f.f()))
    }
    pub fn lookup(&self, n: &str) -> Option<(Num, Dim)> {
        let normalized = solveng_names::normalize(n);
        let n = normalized.as_str();
        match self.map.get(n) {
            Some(d) => Some((d.f, d.d)),
            None => self.prefixed(n),
        }
    }
    pub fn known(&self, n: &str) -> bool {
        self.lookup(n).is_some()
    }
    /// Whether solveng has a unit of this name built in.
    pub fn was_builtin(n: &str) -> bool {
        Units::default().known(n)
    }
    pub fn unknown(&self, n: &str) -> String {
        format!("unknown unit '{n}'{}", if !self.builtin_units && Units::was_builtin(n) { " (built-in units are off, builtin-units: false: define it with the option units)" } else { "" })
    }

    /// A unit expression: `kg m/s^2`, `Ω mm^2/m`, `(A*as)/V`, `m²`, `1/min`; everything after a `/` is below it
    /// (`J/kg K`). With `numbers`, plain factors are allowed too (`100 kPa`, `12`).
    pub fn parse(&self, s: &str, numbers: bool) -> Result<Parsed, String> {
        let toks = utok(s);
        let mut p = UParser { t: &toks, i: 0, u: self, numbers, src: s };
        let (terms, mut f) = p.expr(0)?;
        if p.i < toks.len() {
            return Err(format!("unbalanced ')' in the unit '{s}'"));
        }
        let mut d = Dim::ONE;
        for (n, e) in &terms {
            let (uf, ud) = self.lookup(n).ok_or_else(|| self.unknown(n))?;
            f = f.mul(uf.pow(Num::int(*e as i128)).unwrap_or(Num::F(uf.f().powi(*e))));
            d = d.mul(&ud.powi(*e));
        }
        Ok(Parsed { f, d, u: UnitExpr(terms) })
    }

    /// Like parse, then simplified for showing: a unit written twice gets a power, units of one dimension merge into
    /// one of them when a prefix can say it exactly (mm·mm/m → µm, km·m stays).
    pub fn parse_simplified(&self, s: &str) -> Result<Parsed, String> {
        let p = self.parse(s, false)?;
        Ok(Parsed { u: self.simplify(&p.u), ..p })
    }

    pub fn simplify(&self, u: &UnitExpr) -> UnitExpr {
        // grouped by dimension, in the order written
        let mut groups: Vec<(Dim, Vec<(String, i32, Num)>)> = vec![];
        for (n, e) in &u.0 {
            let Some((f, d)) = self.lookup(n) else { return u.clone() };
            match groups.iter_mut().find(|g| g.0 == d) {
                Some(g) => g.1.push((n.clone(), *e, f)),
                None => groups.push((d, vec![(n.clone(), *e, f)])),
            }
        }
        let mut out = vec![];
        for (d, ts) in &groups {
            let mut names: Vec<String> = vec![];
            for t in ts {
                if !names.contains(&t.0) {
                    names.push(t.0.clone());
                }
            }
            let n: i32 = ts.iter().map(|t| t.1).sum();
            let f: f64 = ts.iter().map(|t| t.2.f().powi(t.1)).product();
            let merged: Vec<(String, i32)> = names
                .iter()
                .map(|m| (m.clone(), ts.iter().filter(|t| &t.0 == m).map(|t| t.1).sum()))
                .filter(|t: &(String, i32)| t.1 != 0)
                .collect();
            if names.len() == 1 || (n == 0 && !crate::num::close(f, 1.0)) {
                out.extend(merged);
                continue;
            }
            if n == 0 {
                continue;
            }
            let want = f.powf(1.0 / n as f64);
            let mut bases: Vec<String> = vec![];
            for m in &names {
                let b = if self.map.contains_key(m) { m.clone() } else { m.chars().skip(1).collect() };
                if !bases.contains(&b) {
                    bases.push(b);
                }
            }
            let mut cands = names.clone();
            for b in &bases {
                cands.push(b.clone());
                for (p, _) in PREFIXES {
                    let c = format!("{p}{b}");
                    if *p != "u" && *p != "μ" && self.prefixed_form(&c) && !self.hidden.contains(&c) {
                        cands.push(c);
                    }
                }
            }
            match cands.iter().find(|c| self.lookup(c).is_some_and(|(g, gd)| gd == *d && crate::num::close(g.f(), want))) {
                Some(c) => out.push((c.clone(), n)),
                None => out.extend(merged),
            }
        }
        UnitExpr(out)
    }

    /// A readable unit for dims d, of the built-in units still in the table (not redefined or hidden).
    pub fn auto(&self, d: &Dim) -> UnitExpr {
        if d.is_one() {
            return UnitExpr::default();
        }
        let has = |n: &str| self.map.get(n).is_some_and(|x| !x.own) && !self.hidden.contains(n);
        let bd = |n: &str| self.map[n].d;
        let au: Vec<&str> = AUTO.iter().copied().filter(|u| has(u)).collect();
        let one = |n: &str, e: i32| UnitExpr(vec![(n.to_string(), e)]);
        for u in &au {
            if bd(u) == *d {
                return one(u, 1);
            }
        }
        // m^2 before "m m"
        for base in ["m", "s"] {
            if !has(base) {
                continue;
            }
            for k in [2, 3, -1, -2, -3] {
                if bd(base).powi(k) == *d {
                    return one(base, k);
                }
            }
        }
        for u in &au {
            for base in ["m", "s", "A", "K"] {
                if !has(base) {
                    continue;
                }
                for k in [1, -1, -2] {
                    // A·A is A², found below
                    if *u == base && k == 1 {
                        continue;
                    }
                    if bd(u).mul(&bd(base).powi(k)) == *d {
                        return UnitExpr(vec![(u.to_string(), 1), (base.to_string(), k)]);
                    }
                }
            }
        }
        // V², Ω⁻¹ before base units
        for u in &au {
            for k in [2, 3, -1, -2] {
                if bd(u).powi(k) == *d {
                    return one(u, k);
                }
            }
        }
        // per base dimension: the SI base unit, else a unit of the table that is exactly it (base units of your
        // own by the names they were given); the SI name only when the table has none
        let mut ts = vec![];
        for i in 0..d.0.len() {
            let e = d.exp(i);
            if e.is_zero() {
                continue;
            }
            let unit = Dim::base(i);
            let exact = |n: &str| !self.hidden.contains(n) && self.lookup(n).is_some_and(|(f, ud)| ud == unit && f.is_one());
            let name = if i < BASE.len() && exact(BASE[i]) { BASE[i].to_string() }
                else if let Some(n) = self.own_base.get(i.wrapping_sub(BASE.len())).filter(|n| exact(n)) { n.clone() }
                else if let Some(n) = self.map.keys().find(|n| exact(n)) { n.clone() }
                else if i < BASE.len() { BASE[i].to_string() } else { self.own_base.get(i - BASE.len()).cloned().unwrap_or("?".into()) };
            if e.is_int() {
                ts.push((name, e.numer() as i32));
            } else {
                // a fractional power: shown as the closest whole power of the base; rare enough
                ts.push((name, e.to_f64().round() as i32));
            }
        }
        UnitExpr(ts)
    }

    /// The factor of a unit to SI (1 for an empty one).
    pub fn factor(&self, u: &UnitExpr) -> Num {
        let mut f = Num::ONE;
        for (n, e) in &u.0 {
            if let Some((uf, _)) = self.lookup(n) {
                f = f.mul(uf.pow(Num::int(*e as i128)).unwrap_or(Num::F(uf.f().powi(*e))));
            }
        }
        f
    }
    pub fn dim(&self, u: &UnitExpr) -> Option<Dim> {
        let mut d = Dim::ONE;
        for (n, e) in &u.0 {
            d = d.mul(&self.lookup(n)?.1.powi(*e));
        }
        Some(d)
    }

    /// The unit u (as picked by auto) with the largest allowed prefix that keeps the number ≥ 1.
    pub fn prefixed_for(&self, v: f64, u: &UnitExpr, prefixes: &Prefixes) -> UnitExpr {
        if u.0.is_empty() || v == 0.0 || !v.is_finite() || matches!(prefixes, Prefixes::None) {
            return u.clone();
        }
        let (n0, e0) = &u.0[0];
        let allowed = |p: &str| match prefixes {
            Prefixes::All => p != "c" && p != "d",
            Prefixes::None => false,
            Prefixes::Some(ps) => ps.iter().any(|q| q == p || (q == "u" && p == "µ")),
        };
        let lg = v.abs().log10();
        // areas and volumes: mm², m², km²
        if *e0 > 1 && u.0.len() == 1 {
            let ps: Vec<&str> = ["m", "", "k"].into_iter().filter(|p| p.is_empty() || (allowed(p) && self.prefixed_form(&format!("{p}{n0}")) && !self.hidden.contains(&format!("{p}{n0}")))).collect();
            let pick = ps.iter().rev().find(|p| (prefix_exp(p).unwrap_or(0) * e0) as f64 <= lg + 1.0).unwrap_or(&ps[0]);
            return UnitExpr(vec![(format!("{pick}{n0}"), *e0)]);
        }
        if *e0 != 1 {
            return u.clone();
        }
        // times of a minute and more in min / h, not ks (unless those units were removed)
        if u.0.len() == 1 && n0 == "s" && matches!(prefixes, Prefixes::All) && v.abs() >= 60.0 {
            let t = if v.abs() < 3600.0 { "min" } else { "h" };
            if self.map.get(t).is_some_and(|d| !d.own && Some(d.d) == self.lookup(n0).map(|x| x.1)) && !self.hidden.contains(t) {
                return UnitExpr::one(t);
            }
        }
        let mut ps: Vec<(&str, i32)> = vec![("", 0)];
        for (p, e) in PREFIXES {
            // a prefix only goes on a unit without one (g → kg, not kkg) and onto names not defined otherwise
            let name = format!("{p}{n0}");
            if *p != "u" && *p != "μ" && allowed(p) && self.prefixed_form(&name) && !self.hidden.contains(&name) {
                ps.push((p, *e));
            }
        }
        ps.sort_by_key(|p| p.1);
        let pick = ps.iter().rev().find(|p| p.1 as f64 <= lg + 1e-9).unwrap_or(&ps[0]);
        let name = format!("{}{n0}", pick.0);
        let mut ts = u.0.clone();
        ts[0].0 = name;
        UnitExpr(ts)
    }
}

/// Prefixes picked for results.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Prefixes {
    All,
    None,
    Some(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub f: Num,
    pub d: Dim,
    pub u: UnitExpr,
}

pub fn valid_name(k: &str) -> bool {
    !k.is_empty() && !k.chars().any(|c| c.is_whitespace() || c.is_ascii_digit() || "*/^()[]+-−·⋅∗=,.;²³".contains(c))
}

fn utok(s: &str) -> Vec<String> {
    let mut out = vec![];
    let cs: Vec<char> = s.replace('"', " ").chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            i += 1;
        } else if "()*/^·⋅∗²³".contains(c) {
            out.push(c.to_string());
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let st = i;
            while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '.' || cs[i] == ',') {
                i += 1;
            }
            if i < cs.len() && (cs[i] == 'e' || cs[i] == 'E') && cs.get(i + 1).is_some_and(|c| c.is_ascii_digit() || *c == '-') {
                i += 2;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
            }
            out.push(cs[st..i].iter().collect());
        } else if c == '-' || c == '−' {
            out.push("-".into());
            i += 1;
        } else {
            let st = i;
            while i < cs.len() && !cs[i].is_whitespace() && !"()*/^·⋅∗²³-−".contains(cs[i]) && !cs[i].is_ascii_digit() {
                i += 1;
            }
            out.push(cs[st..i].iter().collect());
        }
    }
    out
}

/// Resolves the definitions of the option `units`, which may use each other in any order; `own: false` looks
/// below them, at the built-in units (a self reference: `s: "2s"` doubles the built-in second).
struct Resolver<'a> {
    opts: &'a IndexMap<String, UnitOpt>,
    builtin_units: bool,
    cache: HashMap<(String, bool), UnitDef>,
    stack: Vec<(String, bool)>,
    own_base: Vec<String>,
}

impl Resolver<'_> {
    fn resolve(&mut self, name: &str, own: bool) -> Result<UnitDef, String> {
        let value = if own { self.opts.get(name).cloned() } else { None };
        let Some(value) = value else {
            if self.builtin_units && let Some(definition) = builtin().get(name) { return Ok(definition.clone()); }
            let off = !self.builtin_units;
            let unknown = || format!("unknown unit '{name}'{}", if off && Units::was_builtin(name) { " (built-in units are off)" } else { "" });
            let prefix = name.chars().next().ok_or_else(|| "unknown unit ''".to_string())?;
            let exponent = prefix_exp(&prefix.to_string()).ok_or_else(unknown)?;
            let rest = &name[prefix.len_utf8()..];
            if rest.is_empty() || rest == "Omega" { return Err(unknown()); }
            let mut definition = self.resolve(rest, own).map_err(|error| if error.starts_with("unknown unit") { unknown() } else { error })?;
            definition.f = definition.f.mul(Num::R(Rat::int(10).powi(exponent as i64).unwrap()));
            return Ok(definition);
        };
        if matches!(value, UnitOpt::Key(KeyOpt::None)) { return self.resolve(name, false); }
        let key = (name.to_string(), own);
        if let Some(definition) = self.cache.get(&key) { return Ok(definition.clone()); }
        if let Some(at) = self.stack.iter().position(|entry| entry == &key) {
            let cycle = self.stack[at..].iter().map(|(name, _)| name.as_str()).chain(std::iter::once(name)).collect::<Vec<_>>();
            return Err(format!("units: circular dependency: {}", cycle.join(" -> ")));
        }
        self.stack.push(key.clone());
        let definition = match value {
            UnitOpt::Key(KeyOpt::Auto) => {
                if self.own_base.len() == OWN { return Err(format!("units: at most {OWN} base units of your own, '{name}' is one more")); }
                let d = Dim::base(BASE.len() + self.own_base.len());
                self.own_base.push(name.into());
                UnitDef { f: Num::ONE, d, own: true }
            }
            UnitOpt::Num(value) => {
                let f = Rat::approx(value, 1 << 40).filter(|r| r.to_f64() == value).map(Num::R).unwrap_or(Num::F(value));
                UnitDef { f, d: Dim::ONE, own: true }
            }
            UnitOpt::Str(source) => {
                let mut table = Units { map: IndexMap::new(), own_base: vec![], hidden: HashSet::new(), builtin_units: self.builtin_units };
                let words = utok(&source).into_iter().map(|word| solveng_names::normalize(&word)).collect::<Vec<_>>();
                for word in words.iter().filter(|word| valid_name(word)) {
                    // a lone prefix (k Ω written apart) is read with the unit after it
                    let unit = self.opts.contains_key(word) || (self.builtin_units && builtin().contains_key(word));
                    if !unit && prefix_exp(word).is_some() { continue; }
                    let prefixed_self = word.chars().next().is_some_and(|prefix| prefix_exp(&prefix.to_string()).is_some() && &word[prefix.len_utf8()..] == name);
                    let own = !(word == name || prefixed_self);
                    let definition = self.resolve(word, own).map_err(|error| format!("units: {name}: {}", error.trim_start_matches("units: ")))?;
                    table.map.insert(word.clone(), definition);
                }
                let parsed = table.parse(&source, true).map_err(|error| format!("units: {name}: {error}"))?;
                UnitDef { f: parsed.f, d: parsed.d, own: true }
            }
            UnitOpt::Key(KeyOpt::None) => unreachable!(),
        };
        self.stack.pop();
        self.cache.insert(key, definition.clone());
        Ok(definition)
    }
}

struct UParser<'a> {
    t: &'a [String],
    i: usize,
    u: &'a Units,
    numbers: bool,
    src: &'a str,
}

/// x^e, exactly where it fits.
fn powi(x: Num, e: i32) -> Num {
    x.pow(Num::int(e as i128)).unwrap_or(Num::F(x.f().powi(e)))
}

impl UParser<'_> {
    fn at(&self, k: usize) -> Option<&str> {
        self.t.get(k).map(|s| s.as_str())
    }
    /// Units with powers, and the factor of the numbers among them (`10^3 m` → m, 1000).
    fn expr(&mut self, depth: usize) -> Result<(Vec<(String, i32)>, Num), String> {
        let mut out = vec![];
        let mut f = Num::ONE;
        let mut below = false;
        while let Some(k) = self.at(self.i).map(str::to_string) {
            let k = k.as_str();
            if k == ")" {
                if depth == 0 {
                    return Err(format!("unbalanced ')' in the unit '{}'", self.src));
                }
                break;
            }
            if matches!(k, "*" | "·" | "⋅" | "∗") {
                self.i += 1;
                continue;
            }
            if k == "/" {
                below = true;
                self.i += 1;
                continue;
            }
            let (terms, x): (Vec<(String, i32)>, Num) = if k == "(" {
                self.i += 1;
                let r = self.expr(depth + 1)?;
                if self.at(self.i) != Some(")") {
                    return Err(format!("missing ')' in the unit '{}'", self.src));
                }
                self.i += 1;
                r
            } else if k.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
                self.i += 1;
                if k != "1" && !self.numbers {
                    return Err(format!("a number in the unit '{}'", self.src));
                }
                (vec![], Rat::parse_decimal(k).map(Num::R).ok_or_else(|| format!("cannot read the number '{k}'"))?)
            } else if k == "-" || k == "^" || k == "²" || k == "³" {
                return Err(format!("can't read the unit '{}'", self.src));
            } else {
                // "k" Ω: a prefix written apart
                let mut n = solveng_names::normalize(k);
                self.i += 1;
                if prefix_exp(&n).is_some() && !self.u.map.contains_key(&n) {
                    if let Some(nx) = self.at(self.i) {
                        let joined = format!("{n}{nx}");
                        if nx.starts_with(|c: char| c.is_alphabetic() || c == 'Ω' || c == '\u{2126}') && self.u.map.contains_key(nx) && self.u.prefixed_form(&joined) {
                            n = joined;
                            self.i += 1;
                        }
                    }
                }
                if self.u.lookup(&n).is_none() {
                    return Err(self.u.unknown(&n));
                }
                (vec![(n, 1)], Num::ONE)
            };
            // m^2, m^-1, m^(2), m², 10^3, 10^(-3)
            let mut p = 1;
            match self.at(self.i) {
                Some("²") => {
                    p = 2;
                    self.i += 1
                }
                Some("³") => {
                    p = 3;
                    self.i += 1
                }
                Some("^") => {
                    let par = self.at(self.i + 1) == Some("(");
                    let mut k = self.i + 1 + par as usize;
                    let neg = self.at(k) == Some("-");
                    k += neg as usize;
                    let n: i32 = self.at(k).and_then(|x| x.parse().ok()).ok_or_else(|| format!("a whole number as the power in the unit '{}'", self.src))?;
                    if par && self.at(k + 1) != Some(")") {
                        return Err(format!("a whole number as the power in the unit '{}'", self.src));
                    }
                    p = if neg { -n } else { n };
                    self.i = k + 1 + par as usize;
                }
                _ => {}
            }
            let sign = if below { -1 } else { 1 };
            f = f.mul(powi(x, p * sign));
            for (n, e) in terms {
                out.push((n, e * p * sign));
            }
        }
        Ok((out, f))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u() -> Units {
        Units::default()
    }

    #[test]
    fn parse() {
        let p = u().parse("kg m/s^2", false).unwrap();
        assert_eq!(p.d, u().auto_dim("N"));
        assert!((p.f.f() - 1.0).abs() < 1e-12);
        let p = u().parse("Ω mm^2/m", false).unwrap();
        assert!((p.f.f() - 1e-6).abs() < 1e-18);
        assert_eq!(u().parse("mAh", false).unwrap().f, Num::R(Rat::new(18, 5).unwrap()));
        assert!(u().parse("furlong", false).is_err());
        assert_eq!(u().parse("100 kPa", true).unwrap().f, Num::int(100000));
        assert_eq!(u().parse("1/min", true).unwrap().f, Num::R(Rat::new(1, 60).unwrap()));
        assert_eq!(u().parse("k Ω", false).unwrap().f, Num::int(1000));
        assert!(u().parse("kOmega", false).is_err());
    }

    #[test]
    fn auto() {
        let un = u();
        assert_eq!(un.auto(&un.auto_dim("V")).text(), "V");
        assert_eq!(un.auto(&Dim::si([2, 0, 0, 0, 0, 0, 0])).text(), "m^2");
        assert_eq!(un.auto(&Dim::si([1, 0, -1, 0, 0, 0, 0])).text(), "m/s");
        assert_eq!(un.auto(&Dim::si([0, 0, 0, 2, 0, 0, 0])).text(), "A^2");
        assert_eq!(un.prefixed_for(0.03, &UnitExpr::one("A"), &Prefixes::All).text(), "mA");
        assert_eq!(un.prefixed_for(207000.0, &UnitExpr::one("s"), &Prefixes::All).text(), "h");
        assert_eq!(un.prefixed_for(5.0, &UnitExpr::one("g"), &Prefixes::All).text(), "g");
        assert_eq!(un.prefixed_for(5000.0, &UnitExpr::one("g"), &Prefixes::All).text(), "kg");
    }

    #[test]
    fn simplify() {
        let un = u();
        assert_eq!(un.parse_simplified("mm mm/m").unwrap().u.text(), "µm");
        assert_eq!(un.parse_simplified("(A*as)/V").unwrap().u.text(), "A·as/V");
        assert_eq!(un.parse_simplified("A A").unwrap().u.text(), "A^2");
    }

    #[test]
    fn own() {
        let mut o = IndexMap::new();
        o.insert("bar".to_string(), UnitOpt::Str("100 kPa".into()));
        o.insert("ü".to_string(), UnitOpt::Key(KeyOpt::Auto));
        o.insert("T".to_string(), UnitOpt::Key(KeyOpt::None));
        let un = Units::build(&o).unwrap();
        assert_eq!(un.lookup("mbar").unwrap().0, Num::int(100));
        assert!(un.lookup("T").is_some());
        assert!(!un.visible("T"));
        assert_eq!(un.auto(&Dim::base(7)).text(), "ü");
    }

    fn options(pairs: &[(&str, UnitOpt)]) -> IndexMap<String, UnitOpt> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }
    fn text(s: &str) -> UnitOpt {
        UnitOpt::Str(s.into())
    }

    #[test]
    fn hidden_names_and_dependencies() {
        let hidden = Units::build(&options(&[("s", UnitOpt::Key(KeyOpt::None)), ("mys", text("s"))])).unwrap();
        assert!(!hidden.visible("s"));
        assert!(hidden.visible("ms"));
        assert_eq!(hidden.lookup("mys"), hidden.lookup("s"));
        let redefined = Units::build(&options(&[("mys", text("s")), ("s", text("2s"))])).unwrap();
        assert_eq!(redefined.lookup("mys").unwrap().0, Num::int(2));
        for pairs in [vec![("zu", text("uz")), ("uz", text("zu"))], vec![("zu", text("uz")), ("uz", text("mzu"))]] {
            assert!(Units::build(&options(&pairs)).unwrap_err().contains("circular dependency"));
        }
        let forward = Units::build(&options(&[("zu", text("2uz")), ("uz", text("3m"))])).unwrap();
        assert_eq!(forward.lookup("zu").unwrap().0, Num::int(6));
        // a self reference is the built-in unit
        let doubled = Units::build(&options(&[("s", text("2s")), ("span", text("3s"))])).unwrap();
        assert_eq!(doubled.lookup("s").unwrap().0, Num::int(2));
        assert_eq!(doubled.lookup("span").unwrap().0, Num::int(6));
        assert_eq!(doubled.lookup("ms").unwrap().0, Num::R(Rat::new(1, 500).unwrap()));
    }

    #[test]
    fn without_builtin_units_only_own_units_exist() {
        let own = Units::build_visible(&options(&[("m", UnitOpt::Key(KeyOpt::Auto)), ("km", text("1000 m")), ("s", UnitOpt::Num(1.0)), ("x", text("2 km/s"))]), false).unwrap();
        for name in ["m", "km", "s", "mm", "ms", "x"] {
            assert!(own.visible(name), "{name}");
        }
        for name in ["g", "kg", "min", "h", "A", "V", "Ω", "%", "°"] {
            assert!(!own.known(name), "{name}");
        }
        assert!(own.parse("min", false).unwrap_err().contains("built-in units are off"));
        assert_eq!(own.lookup("mm").unwrap().0, Num::R(Rat::new(1, 1000).unwrap()));
        assert_eq!(own.lookup("x").unwrap().0, Num::int(2000));
        assert_eq!(own.lookup("s").unwrap(), (Num::ONE, Dim::ONE));
        let m = own.lookup("m").unwrap().1;
        assert_eq!(own.auto(&m).text(), "m");
        assert_eq!(own.auto(&m.powi(2)).text(), "m^2");
        assert_eq!(own.prefixed_for(5000.0, &UnitExpr::one("m"), &Prefixes::All).text(), "km");
        // definitions can only use units that exist
        for pairs in [vec![("km", text("1000 m"))], vec![("s", text("2s"))], vec![("q", text("3 h"))]] {
            assert!(Units::build_visible(&options(&pairs), false).unwrap_err().contains("unknown unit"), "{pairs:?}");
        }
        let empty = Units::build_visible(&IndexMap::new(), false).unwrap();
        assert!(empty.map.is_empty());
        assert!(!empty.visible("m") && !empty.known("km"));
    }

    #[test]
    fn numeric_powers_in_definitions() {
        for (source, factor) in [("10^3m", 1000.0), ("10^3 m", 1000.0), ("10^(3) m", 1000.0), ("1000m", 1000.0), ("10^-3 m", 1e-3), ("10^(-3) m", 1e-3), ("2^10 m", 1024.0), ("2^3 10^2 m", 800.0), ("1/10^3 m^-1", 1e-3), ("5 10^(-2) km", 50.0), ("2² m", 4.0)] {
            let units = Units::build(&options(&[("q", text(source))])).unwrap();
            let (f, d) = units.lookup("q").unwrap();
            assert!(crate::num::close(f.f(), factor), "{source}: {}", f.f());
            assert_eq!(d, Dim::base(0), "{source}");
        }
        let square = Units::build(&options(&[("q", text("(10 m)^2"))])).unwrap();
        assert_eq!(square.lookup("q").unwrap(), (Num::int(100), Dim::base(0).powi(2)));
        assert_eq!(u().parse("10^3", true).unwrap().f, Num::int(1000));
        assert!(u().parse("10^3 m", false).is_err());
    }

    #[test]
    fn redefined_names_are_not_prefixed() {
        let units = Units::build(&options(&[("km", text("s"))])).unwrap();
        assert!(!units.prefixed_form("km"));
        assert!(units.prefixed_form("mm") && units.prefixed_form("Mm"));
        assert_eq!(units.lookup("km").unwrap().1, Dim::base(2));
        assert_eq!(units.prefixed_for(1000.0, &UnitExpr::one("m"), &Prefixes::All).text(), "m");
        assert_eq!(units.prefixed_for(5e6, &UnitExpr::one("m"), &Prefixes::All).text(), "Mm");
        assert_eq!(units.prefixed_for(1500.0, &UnitExpr::one("m"), &Prefixes::Some(vec!["k".into(), "m".into()])).text(), "m");
        // k m written apart is a kilometre only when km is one
        assert_eq!(units.parse("k m", false).unwrap_err(), "unknown unit 'k'");
        assert_eq!(u().parse("k m", false).unwrap().f, Num::int(1000));
        let areas = Units::build(&options(&[("km", text("s"))])).unwrap();
        assert_eq!(areas.prefixed_for(5e6, &UnitExpr(vec![("m".into(), 2)]), &Prefixes::All).text(), "m^2");
        // merged units of one dimension skip redefined names
        let micro = Units::build(&options(&[("µm", text("s"))])).unwrap();
        assert_eq!(micro.parse_simplified("mm mm/m").unwrap().u.text(), "mm^2/m");
        assert_eq!(u().parse_simplified("mm mm/m").unwrap().u.text(), "µm");
        // a redefined minute isn't the time shown for long durations
        let minutes = Units::build(&options(&[("min", text("2 m"))])).unwrap();
        assert_eq!(minutes.prefixed_for(100.0, &UnitExpr::one("s"), &Prefixes::All).text(), "s");
        let hidden = Units::build(&options(&[("km", UnitOpt::Key(KeyOpt::None)), ("V", UnitOpt::Key(KeyOpt::None))])).unwrap();
        assert_eq!(hidden.prefixed_for(5000.0, &UnitExpr::one("m"), &Prefixes::All).text(), "m");
        assert_ne!(hidden.auto(&u().auto_dim("V")).text(), "V");
    }

    impl Units {
        fn auto_dim(&self, n: &str) -> Dim {
            self.lookup(n).unwrap().1
        }
    }
}
