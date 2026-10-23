//! The options, and values as Typst math markup: numbers, units, quantities, names.
//!
//! Markup refers to a few names the Typst side puts in scope: `svsep` (between units), `svimp` (between steps),
//! `svgroup` (digit groups, when they are content), `svu` (unit-style), `svchip` (casts and types).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::builtins::FnOpt;
use crate::dim::Dim;
use crate::expr::Val;
use crate::num::{Num, Rat};
use crate::units::{Prefixes, UnitExpr, UnitOpt, Units, display_name};

fn d_digits() -> u32 {
    4
}
fn d_sep() -> String {
    ".".into()
}
fn d_sci() -> Option<(i32, i32)> {
    Some((-3, 6))
}
fn d_dot() -> String {
    "dot".into()
}
fn d_auto() -> String {
    "auto".into()
}
fn d_maxden() -> i64 {
    1000
}
fn d_stacked() -> String {
    "stacked".into()
}
fn d_full() -> String {
    "full".into()
}
fn d_result() -> String {
    "result".into()
}
fn d_row() -> String {
    "row".into()
}
pub(crate) fn d_true() -> bool {
    true
}
fn d_maxsol() -> usize {
    6
}
fn d_chip() -> String {
    "chip".into()
}
fn d_prefixes() -> Prefixes {
    Prefixes::All
}

/// The options that matter for calculating and writing results (worksheet's, resolved by the Typst side).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Cfg {
    #[serde(default = "d_digits")]
    pub digits: u32,
    pub trailing_zeros: bool,
    /// The decimal separator, resolved from the language.
    #[serde(default = "d_sep")]
    pub separator: String,
    /// A string between digit groups, or "#" for content the Typst side has as svgroup.
    pub grouping: Option<String>,
    #[serde(default = "d_sci")]
    pub scientific: Option<(i32, i32)>,
    pub engineering: bool,
    #[serde(default = "d_dot")]
    pub multiplication: String,
    #[serde(default = "d_auto")]
    pub form: String,
    #[serde(default = "d_maxden")]
    pub max_denominator: i64,
    #[serde(default = "d_stacked")]
    pub fractions: String,
    #[serde(default = "d_prefixes")]
    pub prefixes: Prefixes,
    pub show_si: bool,
    pub units: IndexMap<String, UnitOpt>,
    #[serde(default = "d_true")]
    pub builtin_units: bool,
    pub unit_layers: Vec<IndexMap<String, UnitOpt>>,
    /// The built-in overlay of function names: standard names reach operations by plain lookup.
    #[serde(default = "d_true")]
    pub builtin_functions: bool,
    /// The innermost user overlay of function names, `(conv: "convolve", tri: none)`.
    pub functions: IndexMap<String, FnOpt>,
    /// Every user overlay of function names, outermost first.
    pub function_layers: Vec<IndexMap<String, FnOpt>>,
    pub preferred_units: Vec<String>,
    pub strict: bool,
    #[serde(default = "d_full")]
    pub steps: String,
    #[serde(default = "d_result")]
    pub inline_steps: String,
    #[serde(default = "d_row")]
    pub layout: String,
    #[serde(default = "d_true")]
    pub approximate: bool,
    #[serde(default = "d_true")]
    pub block: bool,
    #[serde(default = "d_maxsol")]
    pub max_solutions: usize,
    #[serde(default = "d_chip")]
    pub show_casts: String,
    #[serde(default = "d_chip")]
    pub show_types: String,
    /// The word shown for casts and any-unit types.
    pub words: Words,
    /// A unit-style function is set: units are wrapped in svu(…).
    pub unit_style: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Words {
    pub cast: String,
    pub any: String,
}
impl Default for Words {
    fn default() -> Self {
        Words { cast: "as".into(), any: "any".into() }
    }
}

impl Default for Cfg {
    fn default() -> Self {
        serde_json_like_default()
    }
}

fn serde_json_like_default() -> Cfg {
    Cfg {
        digits: 4,
        trailing_zeros: false,
        separator: ".".into(),
        grouping: None,
        scientific: Some((-3, 6)),
        engineering: false,
        multiplication: "dot".into(),
        form: "auto".into(),
        max_denominator: 1000,
        fractions: "stacked".into(),
        prefixes: Prefixes::All,
        show_si: false,
        units: IndexMap::new(),
        builtin_units: true,
        unit_layers: Vec::new(),
        builtin_functions: true,
        functions: IndexMap::new(),
        function_layers: Vec::new(),
        preferred_units: vec![],
        strict: false,
        steps: "full".into(),
        inline_steps: "result".into(),
        layout: "row".into(),
        approximate: true,
        block: true,
        max_solutions: 6,
        show_casts: "chip".into(),
        show_types: "chip".into(),
        words: Words::default(),
        unit_style: false,
    }
}

/// A Typst string literal.
pub fn q(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o += "\\\"",
            '\\' => o += "\\\\",
            _ => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Only explicit underscores make subscripts; digits inside a name stay on the baseline.
pub fn name(n: &str) -> String {
    let (base, sub) = match n.split_once('_') {
        Some((base, sub)) => (base, Some(sub.to_string())),
        None => (n, None),
    };
    let b = base_markup(base);
    match sub {
        Some(s) if !s.is_empty() => format!("{b}_(upright({}))", q(&s)),
        _ => b,
    }
}

fn base_markup(b: &str) -> String {
    let cs: Vec<char> = b.chars().collect();
    if cs.len() == 1 {
        return letter(cs[0]);
    }
    // a Greek letter glued to more: Δr
    if cs.len() > 1 && is_greek(cs[0]) && cs[1].is_alphabetic() {
        return format!("{} {}", letter(cs[0]), base_markup(&cs[1..].iter().collect::<String>()));
    }
    format!("upright({})", q(b))
}

fn is_greek(c: char) -> bool {
    ('\u{0391}'..='\u{03C9}').contains(&c)
}

fn letter(c: char) -> String {
    if c.is_ascii_alphabetic() || is_greek(c) { c.to_string() } else { q(&c.to_string()) }
}

/// A unit as markup: upright names, powers, · between them, / before what's below.
pub fn unit(u: &UnitExpr, units: &Units, cfg: &Cfg) -> String {
    if u.0.len() == 1 && u.0[0].0 == "any" {
        return format!("upright({})", q(&cfg.words.any));
    }
    let w = |ts: Vec<(&String, i32)>| {
        ts.iter()
            .map(|(n, e)| {
                let s = format!("upright({})", q(&display_name(n, units)));
                if *e == 1 { s } else { format!("{s}^({e})") }
            })
            .collect::<Vec<_>>()
            .join(" #svsep ")
    };
    let up: Vec<_> = u.0.iter().filter(|t| t.1 > 0).map(|(n, e)| (n, *e)).collect();
    let down: Vec<_> = u.0.iter().filter(|t| t.1 < 0).map(|(n, e)| (n, -e)).collect();
    let mut s = if up.is_empty() { "1".into() } else { w(up) };
    if !down.is_empty() {
        s = format!("{s} \\/ {}", w(down));
    }
    if cfg.unit_style { format!("#svu(${s}$)") } else { s }
}

// ---------- numbers ----------

/// |v| with d decimals as a plain digit string.
fn fixed(v: f64, d: i32, zeros: bool) -> String {
    let mut s = format!("{:.*}", d.max(0) as usize, v.abs());
    if d > 0 && !zeros {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if v < 0.0 && s.chars().any(|c| c != '0' && c != '.') { format!("-{s}") } else { s }
}

/// A number rounded to `digits` significant digits: (mantissa, exponent if scientific).
pub fn fmt(x: f64, cfg: &Cfg) -> (String, Option<i32>) {
    if x == 0.0 || !x.is_finite() {
        return (if x == 0.0 { "0".into() } else { format!("{x}") }, None);
    }
    let n = cfg.digits as i32;
    let mut e = x.abs().log10().floor() as i32;
    let m = x / 10f64.powi(e);
    if (m * 10f64.powi(n - 1)).round().abs() >= 10f64.powi(n) {
        e += 1; // 9.9996 → 10.00
    }
    let (lo, hi) = cfg.scientific.unwrap_or((i32::MIN, i32::MAX));
    let sci = e < lo || e >= hi || x.abs() >= 1e15 || x.abs() < 1e-15;
    let p = if !sci {
        0
    } else if cfg.engineering {
        e.div_euclid(3) * 3
    } else {
        e
    };
    let d = n - 1 - e + p;
    let mut y = x / 10f64.powi(p);
    if d < 0 {
        y = (y / 10f64.powi(-d)).round() * 10f64.powi(-d); // 123456 → 123500 at 4 digits
    }
    (fixed(y, d.max(0), cfg.trailing_zeros), if sci { Some(p) } else { None })
}

fn group(s: &str, cfg: &Cfg) -> String {
    let Some(g) = &cfg.grouping else { return q(s) };
    let cs: Vec<char> = s.chars().collect();
    if cs.len() < 5 {
        return q(s);
    }
    let mut parts = vec![];
    let mut i = cs.len();
    while i > 0 {
        let st = i.saturating_sub(3);
        parts.push(cs[st..i].iter().collect::<String>());
        i = st;
    }
    parts.reverse();
    if g == "#" { parts.iter().map(|p| q(p)).collect::<Vec<_>>().join("#svgroup") } else { q(&parts.join(g)) }
}

/// A mantissa and exponent as markup: −1,5 · 10⁻³.
pub fn num_markup(m: &str, e: Option<i32>, cfg: &Cfg) -> String {
    let neg = m.starts_with('-') || m.starts_with('−');
    let m = m.trim_start_matches(['-', '−', '+']);
    let (ip, fp) = match m.find('.') {
        Some(i) => (&m[..i], Some(&m[i + 1..])),
        None => (m, None),
    };
    let mut s = group(ip, cfg);
    if let Some(fp) = fp {
        // the separator and the decimals in the same text as the last group
        s.pop();
        s += &format!("{}{fp}\"", cfg.separator.replace('\\', "\\\\").replace('"', "\\\""));
    }
    if neg {
        s = format!("−{s}");
        if s.starts_with("−\"") {
            s = format!("\"−{}", &s["−\"".len()..]);
        }
    }
    let t = if cfg.multiplication == "times" { "times" } else { "dot.op" };
    match e {
        None | Some(0) => s,
        Some(e) if m == "1" && !neg => format!("10^({e})"),
        Some(e) => format!("{s} {t} 10^({e})"),
    }
}

/// How a number is shown: (markup, exact).
pub fn number(v: Num, raw: Option<&str>, cfg: &Cfg) -> (String, bool) {
    if cfg.form == "fraction" {
        let r = match v {
            Num::R(r) if r.denom() <= cfg.max_denominator as i128 => Some(r),
            _ => Rat::approx(v.f(), cfg.max_denominator as i128),
        };
        if let Some(r) = r {
            return (frac_markup(r, cfg), true);
        }
    }
    if let (Some(raw), true) = (raw, cfg.form != "decimal") {
        // as typed: no e-notation, so the raw number has no exponent to show
        return (num_markup(raw, None, cfg), true);
    }
    let x = v.f();
    let (m, e) = fmt(x, cfg);
    let shown: f64 = m.parse::<f64>().unwrap_or(f64::NAN) * 10f64.powi(e.unwrap_or(0));
    let exact = !x.is_finite() || (shown - x).abs() <= 1e-9 * x.abs();
    (num_markup(&m, e, cfg), exact)
}

/// An exact fraction: ⅙ stacked or 1/6.
pub fn frac_markup(r: Rat, cfg: &Cfg) -> String {
    if r.is_int() {
        return num_markup(&r.numer().to_string(), None, cfg);
    }
    let a = num_markup(&r.numer().abs().to_string(), None, cfg);
    let b = num_markup(&r.denom().to_string(), None, cfg);
    let f = if cfg.fractions == "slash" { format!("{a} \\/ {b}") } else { format!("frac({a}, {b})") };
    if r.is_neg() { format!("-{f}") } else { f }
}

/// A value to show: the number in its unit.
#[derive(Clone, Debug)]
pub struct Shown {
    pub n: Num,
    pub u: UnitExpr,
    pub raw: Option<String>,
    /// A bare unit: [A] is shown as A.
    pub bare: bool,
}

/// Which unit a value is shown in: the one asked for, else a preferred one, the one it was written in, the
/// non-SI unit of the inputs (hint), or SI with a prefix.
pub fn pick(v: &Val, want: Option<&UnitExpr>, hint: Option<&UnitExpr>, units: &Units, cfg: &Cfg) -> Result<Shown, String> {
    let si = v.v;
    if let Some(u) = want {
        let d = units.dim(u).ok_or_else(|| format!("unknown unit '{}'", u.text()))?;
        if d != v.d {
            return Err(format!(
                "the result has unit {}, not {}",
                if v.d.is_one() { "1 (no unit)".to_string() } else { units.auto(&v.d).text() },
                u.text()
            ));
        }
        let f = units.factor(u);
        return Ok(Shown { n: si.div(f).unwrap_or(si), u: u.clone(), raw: None, bare: false });
    }
    for p in &cfg.preferred_units {
        if let Ok(pu) = units.parse(p, false) {
            if pu.d == v.d && !v.d.is_one() {
                return Ok(Shown { n: si.div(pu.f).unwrap_or(si), u: pu.u, raw: None, bare: false });
            }
        }
    }
    // the unit it was written in (7.2 kΩ), or a unit that isn't SI it came from (2 p in bar)
    if let Some(u) = v.u.as_ref().filter(|u| v.raw.is_some() || !si_unit(u, units)) {
        if units.dim(u) == Some(v.d) {
            let f = units.factor(u);
            let n = si.div(f).unwrap_or(si);
            // as written: 7.20 kΩ keeps its zero
            let raw = v.raw.clone().filter(|r| Rat::parse_decimal(r).is_some_and(|x| Num::R(x).same(n)));
            return Ok(Shown { n, u: u.clone(), raw, bare: false });
        }
    }
    if v.d.is_one() {
        let raw = v.raw.clone().filter(|r| Rat::parse_decimal(r).is_some_and(|x| Num::R(x).same(si)));
        return Ok(Shown { n: si, u: UnitExpr::default(), raw, bare: false });
    }
    if let Some(h) = hint {
        if units.dim(h) == Some(v.d) {
            return Ok(Shown { n: si.div(units.factor(h)).unwrap_or(si), u: h.clone(), raw: None, bare: false });
        }
    }
    let a = units.auto(&v.d);
    let fa = units.factor(&a);
    let na = si.div(fa).unwrap_or(si);
    // fractions read better in the base unit (1/6 A, not 500/3 mA)
    if cfg.form == "fraction" {
        if let Some(r) = na.rat().or_else(|| Rat::approx(na.f(), cfg.max_denominator as i128)) {
            if !r.is_int() {
                return Ok(Shown { n: na, u: a, raw: None, bare: false });
            }
        }
    }
    let u = units.prefixed_for(na.f(), &a, &cfg.prefixes);
    Ok(Shown { n: si.div(units.factor(&u)).unwrap_or(si), u, raw: None, bare: false })
}

/// Whether a unit is SI (with a prefix or not): those get picked anew for results.
pub fn si_unit(u: &UnitExpr, units: &Units) -> bool {
    const SI: [&str; 21] = ["V", "A", "Ω", "Omega", "W", "J", "C", "F", "H", "S", "N", "Pa", "T", "Wb", "s", "m", "g", "K", "Hz", "mol", "cd"];
    u.0.iter().all(|(n, _)| {
        let base: String = if units.map.contains_key(n) { n.clone() } else { n.chars().skip(1).collect() };
        SI.contains(&base.as_str())
    })
}

/// A shown value as markup: (markup, exact).
pub fn shown(s: &Shown, units: &Units, cfg: &Cfg) -> (String, bool) {
    if s.bare {
        return (unit(&s.u, units, cfg), true);
    }
    let (n, exact) = number(s.n, s.raw.as_deref(), cfg);
    if s.u.is_empty() {
        return (n, exact);
    }
    (format!("{n} thin {}", unit(&s.u, units, cfg)), exact)
}

/// A value in plain text, for editors: "2.988 V".
pub fn plain(v: &Val, unit_: Option<&UnitExpr>, units: &Units, cfg: &Cfg) -> String {
    let c = Cfg { digits: cfg.digits.max(4), ..cfg.clone() };
    let s = match pick(v, unit_, None, units, &c) {
        Ok(s) => s,
        Err(_) => return format!("{}", v.v.f()),
    };
    let (m, e) = fmt(s.n.f(), &c);
    let mut out = m;
    if let Some(e) = e {
        out += &format!("e{e}");
    }
    if !s.u.is_empty() {
        out += " ";
        out += &s.u.text();
    }
    out
}

pub fn dim_text(d: &Dim, units: &Units) -> String {
    if d.is_one() { "1 (no unit)".into() } else { units.auto(d).text() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        let c = Cfg::default();
        assert_eq!(fmt(57.5, &c), ("57.5".into(), None));
        assert_eq!(fmt(123456.0, &c), ("123500".into(), None));
        assert_eq!(fmt(2.5e-7, &c), ("2.5".into(), Some(-7)));
        assert_eq!(fmt(9.99996, &c), ("10".into(), None));
        assert_eq!(num_markup("-1.5", None, &Cfg { separator: ",".into(), ..c.clone() }), "\"−1,5\"");
        assert_eq!(num_markup("2.5", Some(-7), &c), "\"2.5\" dot.op 10^(-7)");
        assert_eq!(group("12345", &Cfg { grouping: Some(" ".into()), ..c.clone() }), "\"12 345\"");
    }

    #[test]
    fn names() {
        assert_eq!(name("Q_0"), "Q_(upright(\"0\"))");
        assert_eq!(name("Q0"), "upright(\"Q0\")");
        assert_eq!(name("tews"), "upright(\"tews\")");
        assert_eq!(name("Δr_R"), "Δ r_(upright(\"R\"))");
        assert_eq!(name("x"), "x");
    }

    #[test]
    fn picking() {
        let u = Units::default();
        let c = Cfg::default();
        let v = Val::new(Num::F(207000.0), Dim::base(2));
        let s = pick(&v, None, None, &u, &c).unwrap();
        assert_eq!(s.u.text(), "h");
        assert!((s.n.f() - 57.5).abs() < 1e-12);
        let v = Val::new(Num::F(0.03), Dim::base(3));
        assert_eq!(pick(&v, None, None, &u, &c).unwrap().u.text(), "mA");
    }
}
