//! Units: the table (built-in and your own), prefixes, reading unit expressions, and picking a readable unit.

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
    pub map: IndexMap<String, UnitDef>,
    /// Names of the base units of your own, by slot.
    pub own_base: Vec<String>,
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
        ("eV", Num::R(Rat::parse_decimal("1.602176634e-19").unwrap()), j),
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
        Units { map: builtin(), own_base: vec![] }
    }
}

impl Units {
    /// The built-in units changed by the option `units`.
    pub fn build(opts: &IndexMap<String, UnitOpt>) -> Result<Units, String> {
        let mut u = Units::default();
        for (k, v) in opts {
            match v {
                UnitOpt::Key(KeyOpt::None) => {
                    if u.map.shift_remove(k).is_none() {
                        return Err(format!("units: '{k}' is not a unit, so it can't be removed"));
                    }
                }
                _ if !valid_name(k) => {
                    return Err(format!("units: '{k}' can't be a unit name (no spaces, digits or operators)"));
                }
                UnitOpt::Key(KeyOpt::Auto) => {
                    if u.own_base.len() == OWN {
                        return Err(format!("units: at most {OWN} base units of your own, '{k}' is one more"));
                    }
                    let d = Dim::base(BASE.len() + u.own_base.len());
                    u.own_base.push(k.clone());
                    u.map.insert(k.clone(), UnitDef { f: Num::ONE, d, own: true });
                }
                UnitOpt::Num(x) => {
                    let f = Rat::approx(*x, 1 << 40).filter(|r| r.to_f64() == *x).map(Num::R).unwrap_or(Num::F(*x));
                    u.map.insert(k.clone(), UnitDef { f, d: Dim::ONE, own: true });
                }
                UnitOpt::Str(s) => {
                    let p = u.parse(s, true).map_err(|e| format!("units: {k}: {e}"))?;
                    u.map.insert(k.clone(), UnitDef { f: p.f, d: p.d, own: true });
                }
            }
        }
        Ok(u)
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
    pub fn lookup(&self, n: &str) -> Option<(Num, Dim)> {
        match self.map.get(n) {
            Some(d) => Some((d.f, d.d)),
            None => self.prefixed(n),
        }
    }
    pub fn known(&self, n: &str) -> bool {
        self.lookup(n).is_some()
    }
    /// Whether a unit with this name exists without the option `units`.
    pub fn was_builtin(n: &str) -> bool {
        Units::default().known(n)
    }
    fn unknown(&self, n: &str) -> String {
        format!("unknown unit '{n}'{}", if Units::was_builtin(n) { " (it was removed with the option units)" } else { "" })
    }

    /// A unit expression: `kg m/s^2`, `Ω mm^2/m`, `(A*as)/V`, `m²`, `1/min`; everything after a `/` is below it
    /// (`J/kg K`). With `numbers`, plain factors are allowed too (`100 kPa`, `12`).
    pub fn parse(&self, s: &str, numbers: bool) -> Result<Parsed, String> {
        let toks = utok(s);
        let mut p = UParser { t: &toks, i: 0, u: self, numbers, src: s, f: Num::ONE };
        let terms = p.expr(0)?;
        if p.i < toks.len() {
            return Err(format!("unbalanced ')' in the unit '{s}'"));
        }
        let mut d = Dim::ONE;
        let mut f = p.f;
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
        for (_, ts) in groups {
            let mut names: Vec<String> = vec![];
            for t in &ts {
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
                    if *p != "u" && *p != "μ" {
                        cands.push(format!("{p}{b}"));
                    }
                }
            }
            match cands.iter().find(|c| self.lookup(c).is_some_and(|(g, _)| crate::num::close(g.f(), want))) {
                Some(c) => out.push((c.clone(), n)),
                None => out.extend(merged),
            }
        }
        UnitExpr(out)
    }

    /// A readable unit for dims d (only built-in units that are still in the table).
    pub fn auto(&self, d: &Dim) -> UnitExpr {
        if d.is_one() {
            return UnitExpr::default();
        }
        let b = Units::default();
        let has = |n: &str| self.map.get(n).is_some_and(|x| !x.own);
        let bd = |n: &str| b.map[n].d;
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
        // base units of your own by the names they were given
        let mut ts = vec![];
        for i in 0..d.0.len() {
            let e = d.exp(i);
            if e.is_zero() {
                continue;
            }
            let name = if i < BASE.len() { BASE[i].to_string() } else { self.own_base.get(i - BASE.len()).cloned().unwrap_or("?".into()) };
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
            let ps: Vec<&str> = ["m", "", "k"].into_iter().filter(|p| p.is_empty() || allowed(p)).collect();
            let pick = ps.iter().rev().find(|p| (prefix_exp(p).unwrap_or(0) * e0) as f64 <= lg + 1.0).unwrap_or(&ps[0]);
            return UnitExpr(vec![(format!("{pick}{n0}"), *e0)]);
        }
        if *e0 != 1 {
            return u.clone();
        }
        // times of a minute and more in min / h, not ks (unless those units were removed)
        if u.0.len() == 1 && n0 == "s" && matches!(prefixes, Prefixes::All) && v.abs() >= 60.0 {
            let t = if v.abs() < 3600.0 { "min" } else { "h" };
            if self.map.get(t).is_some_and(|d| !d.own) {
                return UnitExpr::one(t);
            }
        }
        let mut ps: Vec<(&str, i32)> = vec![("", 0)];
        for (p, e) in PREFIXES {
            if *p != "u" && *p != "μ" && allowed(p) {
                ps.push((p, *e));
            }
        }
        ps.sort_by_key(|p| p.1);
        let pick = ps.iter().rev().find(|p| p.1 as f64 <= lg + 1e-9).unwrap_or(&ps[0]);
        // a prefix only goes on a unit without one: g → kg, not kkg
        let name = format!("{}{n0}", pick.0);
        if !pick.0.is_empty() && !self.known(&name) {
            return u.clone();
        }
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
    !k.is_empty() && !k.chars().any(|c| c.is_whitespace() || c.is_ascii_digit() || "*/^()[]+-−·⋅=,.;".contains(c))
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
    f: Num,
}

impl UParser<'_> {
    fn at(&self, k: usize) -> Option<&str> {
        self.t.get(k).map(|s| s.as_str())
    }
    fn expr(&mut self, depth: usize) -> Result<Vec<(String, i32)>, String> {
        let mut out = vec![];
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
            let terms: Vec<(String, i32)> = if k == "(" {
                self.i += 1;
                let r = self.expr(depth + 1)?;
                if self.at(self.i) != Some(")") {
                    return Err(format!("missing ')' in the unit '{}'", self.src));
                }
                self.i += 1;
                r
            } else if k.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
                self.i += 1;
                if k == "1" {
                    vec![]
                } else if self.numbers {
                    let x = Rat::parse_decimal(k).map(Num::R).ok_or_else(|| format!("cannot read the number '{k}'"))?;
                    let x = if below { Num::ONE.div(x).ok_or("division by zero")? } else { x };
                    self.f = self.f.mul(x);
                    vec![]
                } else {
                    return Err(format!("a number in the unit '{}'", self.src));
                }
            } else if k == "-" || k == "^" || k == "²" || k == "³" {
                return Err(format!("can't read the unit '{}'", self.src));
            } else {
                // "k" Ω: a prefix written apart
                let mut n = k.to_string();
                self.i += 1;
                if prefix_exp(&n).is_some() && !self.u.map.contains_key(&n) {
                    if let Some(nx) = self.at(self.i) {
                        if nx.starts_with(|c: char| c.is_alphabetic() || c == 'Ω' || c == '\u{2126}') && self.u.map.contains_key(nx) {
                            n += nx;
                            self.i += 1;
                        }
                    }
                }
                if self.u.lookup(&n).is_none() {
                    return Err(self.u.unknown(&n));
                }
                vec![(n, 1)]
            };
            // m^2, m^-1, m^(2), m²
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
            for (n, e) in terms {
                out.push((n, e * p * if below { -1 } else { 1 }));
            }
        }
        Ok(out)
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
        assert!(un.lookup("T").is_none());
        assert_eq!(un.auto(&Dim::base(7)).text(), "ü");
    }

    impl Units {
        fn auto_dim(&self, n: &str) -> Dim {
            self.lookup(n).unwrap().1
        }
    }
}
