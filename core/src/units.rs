//! Units: the table (built-in and your own), prefixes, reading unit expressions, and picking a readable unit.


use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::dim::Dim;
use crate::num::{Num, Rat};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitDef {
    #[serde(default)]
    pub display: Option<String>,
    /// Factor to SI.
    pub f: Num,
    pub d: Dim,
    /// A unit binding, not a built-in unit.
    pub own: bool,
    /// Its display starts with a number (`2.54 cm`): after a number or another factor it's joined by a dot.
    #[serde(default)]
    pub numeric: bool,
    /// Its display is more than one atom (`2.54 cm`, `N · m`): parenthesized under a power or after a slash.
    #[serde(default)]
    pub compound: bool,
}

pub const PREFIXES: &[(&str, i32)] = &[
    ("a", -18), ("f", -15), ("p", -12), ("n", -9), ("µ", -6), ("μ", -6), ("u", -6), ("m", -3), ("c", -2), ("d", -1),
    ("k", 3), ("M", 6), ("G", 9), ("T", 12),
];

fn prefix_exp(p: &str) -> Option<i32> {
    PREFIXES.iter().find(|(q, _)| *q == p).map(|(_, e)| *e)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Units {
    /// The units there are: the unit bindings in use.
    pub map: IndexMap<String, UnitDef>,
    /// Units results may be shown in without being in use: the SI units, by name. Never read from what's written.
    #[serde(skip)]
    output: IndexMap<String, UnitDef>,
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
        ("C", one, c.clone()),
        ("J", one, j.clone()),
        ("Ω", one, o.clone()),
        ("\u{2126}", one, o.clone()),
        ("Omega", one, o.clone()),
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
        ("Ah", Num::int(3600), c.clone()),
        ("As", one, c),
        ("Wh", Num::int(3600), j.clone()),
        ("Ws", one, j.clone()),
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
    list.into_iter().map(|(n, f, d)| (n.to_string(), UnitDef { f, d, own: false, display: None, numeric: false, compound: false })).collect()
}

/// The units results can be shown in without being in use: the SI units, minutes and hours.
fn standard() -> IndexMap<String, UnitDef> {
    builtin().into_iter().filter(|(n, _)| AUTO.contains(&n.as_str()) || matches!(n.as_str(), "min" | "h")).collect()
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
    if let Some(display) = u.def(n).and_then(|d| d.display.as_ref()) { return display.clone(); }
    let normalized = solveng_names::normalize(n);
    let n = normalized.as_str();
    let sym = |r: &str| match r {
        "Omega" => "Ω".to_string(),
        "degC" => "°C".to_string(),
        "deg" => "°".to_string(),
        _ => r.to_string(),
    };
    if u.def(n).is_some() {
        return sym(n);
    }
    let mut cs = n.chars();
    if let Some(p) = cs.next() {
        let rest: String = cs.collect();
        if u.prefixed_in(n, true).is_some() {
            return format!("{}{}", if p == 'u' { 'µ' } else { p }, display_name(&rest, u));
        }
    }
    n.to_string()
}

impl Default for Units {
    fn default() -> Self {
        Units { map: IndexMap::new(), output: standard() }
    }
}

impl Units {
    pub fn lexical(cfg: &crate::format::Cfg, env: &crate::env::Env, src: &str) -> Result<Units, String> {
        let mut units = Self::configured(cfg)?;
        units.map.extend(env.references.clone());
        units.map.extend(crate::bindings::references(src)?);
        Ok(units)
    }

    /// The units of the preferred units; results can always be shown in the SI units.
    pub fn configured(cfg: &crate::format::Cfg) -> Result<Units, String> {
        let mut units = Units::default();
        for source in &cfg.preferred_units { units.map.extend(crate::bindings::references(source)?); }
        Ok(units)
    }

    /// The table of solveng's units by name, which the `units` module's bindings are made from.
    pub fn builtin_table() -> Units {
        Units { map: builtin(), output: IndexMap::new() }
    }

    /// The SI unit a unit of the table is: its SI name and the exponent of its prefix (kΩ → (Ω, 3)), told by its
    /// name or display together with its factor; None for others (h, inch, a unit displayed as m that isn't one).
    pub fn si(&self, key: &str) -> Option<(&'static str, i32)> {
        let f = self.def(key)?.f;
        let text = display_name(key, self);
        AUTO.iter().find_map(|name| {
            let e = if text == *name { 0 } else { prefix_exp(text.strip_suffix(name)?)? };
            let si = if *name == "g" { 1e-3 } else { 1.0 };
            crate::num::close(f.f(), si * 10f64.powi(e)).then_some((*name, e))
        })
    }

    /// The table's unit of an SI name without a prefix: the built-in one, or a unit binding that is it.
    /// A binding in use comes first, so that results show its display.
    fn coherent(&self, name: &str) -> Option<String> {
        if self.map.get(name).is_some_and(|d| !d.own) { return Some(name.to_string()); }
        self.map.keys().find(|k| self.map[*k].own && self.si(k) == Some((name, 0))).cloned()
            .or_else(|| self.output.contains_key(name).then(|| name.to_string()))
    }

    /// The unit shown as `text` (min, h), of dimension d.
    fn shown_as(&self, text: &str, d: &Dim) -> Option<String> {
        self.map.iter().chain(&self.output).find(|(k, u)| u.d == *d && display_name(k, self) == text).map(|(k, _)| k.clone())
    }

    /// A unit of the table, or one results may be shown in.
    pub fn def(&self, n: &str) -> Option<&UnitDef> {
        self.map.get(n).or_else(|| self.output.get(n))
    }

    /// A unit's factor and dimension, also of the units results may be shown in (mW): for what is shown, never
    /// for reading what's written.
    pub fn shown_lookup(&self, n: &str) -> Option<(Num, Dim)> {
        self.def(n).map(|d| (d.f, d.d.clone())).or_else(|| self.prefixed_in(n, true))
    }

    /// A unit expression in text, by the units' names or displays: kg·m/s^2.
    pub fn text(&self, u: &UnitExpr) -> String {
        // a compound display is one factor under a power or below the slash: (2.54 cm)^2
        let shown = |n: &str, e: i32| {
            let d = display_name(n, self);
            if (e != 1 || u.0.len() > 1) && self.def(n).is_some_and(|d| d.compound) { format!("({d})") } else { d }
        };
        UnitExpr(u.0.iter().map(|(n, e)| (shown(n, *e), *e)).collect()).text()
    }

    /// A prefixed unit: kΩ → (factor, dims); a spelled Greek letter takes one only glued (k""Omega), as names do.
    pub fn prefixed(&self, n: &str) -> Option<(Num, Dim)> {
        self.prefixed_in(n, false)
    }
    /// With `shown`, also prefixes on the units results may be shown in.
    fn prefixed_in(&self, n: &str, shown: bool) -> Option<(Num, Dim)> {
        let p = n.chars().next()?;
        let rest = &n[p.len_utf8()..];
        if rest.is_empty() || rest == "Omega" {
            return None;
        }
        let e = prefix_exp(&p.to_string())?;
        let def = if shown { self.def(rest)? } else { self.map.get(rest)? };
        // a prefix goes on a built-in unit, or on a unit binding that is an SI unit without a prefix (A, V, m)
        if def.display.is_some() && self.si(rest).is_none_or(|(_, e)| e != 0) { return None; }
        let pf = Num::R(Rat::int(10).powi(e as i64).unwrap());
        Some((def.f.mul(pf), def.d.clone()))
    }
    /// Whether n is a prefix on a unit, meaning just that: not when the table defines n itself as something
    /// else (with `km: "s"`, km is no kilometre).
    pub fn prefixed_form(&self, n: &str) -> bool {
        let Some((f, d)) = self.prefixed_in(n, true) else { return false };
        self.def(n).is_none_or(|u| u.d == d && crate::num::close(u.f.f(), f.f()))
    }
    pub fn lookup(&self, n: &str) -> Option<(Num, Dim)> {
        let normalized = solveng_names::normalize(n);
        let n = normalized.as_str();
        match self.map.get(n) {
            Some(d) => Some((d.f, d.d.clone())),
            None => self.prefixed(n),
        }
    }
    pub fn known(&self, n: &str) -> bool {
        self.lookup(n).is_some()
    }
    pub fn unknown(&self, n: &str) -> String {
        format!("unknown unit '{}'", display_name(n, self))
    }

    /// A unit expression: `kg m/s^2`, `Ω mm^2/m`, `(A*as)/V`, `m²`, `1/min`; everything after a `/` is below it
    /// (`J/kg K`). With `numbers`, plain factors are allowed too (`100 kPa`, `12`).
    pub fn parse(&self, s: &str, numbers: bool) -> Result<Parsed, String> {
        if s.contains('⟦') {
            let mut units = self.clone();
            let mut source = String::new();
            let mut rest = s;
            while let Some((before, after)) = rest.split_once('⟦') {
                source.push_str(before);
                let (encoded, tail) = after.split_once('⟧').ok_or("unclosed unit value")?;
                let binding = crate::bindings::Binding::decode(encoded)?;
                if !binding.unit { return Err("expected a unit value".into()); }
                source.push_str(&binding.id);
                units.map.extend(binding.units);
                rest = tail;
            }
            source.push_str(rest);
            return units.parse(&source, numbers);
        }
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
                    if *p != "u" && *p != "μ" && self.prefixed_form(&c)  {
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

    /// A readable unit for dims d, of the built-in units in the table.
    pub fn auto(&self, d: &Dim) -> UnitExpr {
        if d.is_one() {
            return UnitExpr::default();
        }
        // the SI units the table has, built in or bound: (SI name, its unit in the table)
        let si: Vec<(&str, String)> = AUTO.iter().filter_map(|n| Some((*n, self.coherent(n)?))).collect();
        let key = |n: &str| si.iter().find(|(m, _)| *m == n).map(|(_, k)| k.as_str());
        let has = |n: &str| key(n).is_some();
        let bd = |n: &str| self.def(key(n).unwrap()).unwrap().d.clone();
        let au: Vec<&str> = si.iter().map(|(n, _)| *n).collect();
        let one = |n: &str, e: i32| UnitExpr(vec![(key(n).unwrap().to_string(), e)]);
        for u in &au {
            if bd(u) == *d {
                return one(u, 1);
            }
        }
        // a unit of your own that is exactly it
        if let Some((name, _)) = self.map.iter().rev().find(|(_, unit)| unit.own && unit.d == *d && unit.f.is_one()) {
            return UnitExpr::one(name);
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
                        return UnitExpr(vec![(key(u).unwrap().to_string(), 1), (key(base).unwrap().to_string(), k)]);
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
        for (identity, power) in &d.0 {
            let e = Rat::new(*power as i128, 60).unwrap();
            let unit = Dim::named(identity);
            let name = self.map.keys().chain(self.output.keys()).find(|n| self.shown_lookup(n).is_some_and(|(f, ud)| ud == unit && f.is_one()))
                .cloned().unwrap_or_else(|| identity.clone());
            ts.push((name, e.to_f64().round() as i32));
        }
        UnitExpr(ts)
    }

    /// The factor of a unit to SI (1 for an empty one).
    pub fn factor(&self, u: &UnitExpr) -> Num {
        let mut f = Num::ONE;
        for (n, e) in &u.0 {
            if let Some((uf, _)) = self.shown_lookup(n) {
                f = f.mul(uf.pow(Num::int(*e as i128)).unwrap_or(Num::F(uf.f().powi(*e))));
            }
        }
        f
    }
    pub fn dim(&self, u: &UnitExpr) -> Option<Dim> {
        let mut d = Dim::ONE;
        for (n, e) in &u.0 {
            d = d.mul(&self.shown_lookup(n)?.1.powi(*e));
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
            let ps: Vec<&str> = ["m", "", "k"].into_iter().filter(|p| p.is_empty() || (allowed(p) && self.prefixed_form(&format!("{p}{n0}")))).collect();
            let pick = ps.iter().rev().find(|p| (prefix_exp(p).unwrap_or(0) * e0) as f64 <= lg + 1.0).unwrap_or(&ps[0]);
            return UnitExpr(vec![(format!("{pick}{n0}"), *e0)]);
        }
        if *e0 != 1 {
            return u.clone();
        }
        // times of a minute and more in min / h, not ks (unless those units were removed)
        let second = self.si(n0) == Some(("s", 0));
        if u.0.len() == 1 && second && matches!(prefixes, Prefixes::All) && v.abs() >= 60.0 {
            let t = if v.abs() < 3600.0 { "min" } else { "h" };
            // in minutes or hours where the table has them; a kilosecond reads worse than seconds
            return match self.def(n0).and_then(|u| self.shown_as(t, &u.d)) {
                Some(k) => UnitExpr::one(&k),
                None => u.clone(),
            };
        }
        let mut ps: Vec<(&str, i32)> = vec![("", 0)];
        for (p, e) in PREFIXES {
            // a prefix only goes on a unit without one (g → kg, not kkg) and onto names not defined otherwise
            let name = format!("{p}{n0}");
            if *p != "u" && *p != "μ" && allowed(p) && self.prefixed_form(&name) {
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
        Units::builtin_table()
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

    impl Units {
        fn auto_dim(&self, n: &str) -> Dim {
            self.lookup(n).unwrap().1
        }
    }
}
