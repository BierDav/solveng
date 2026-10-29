//! The options, and values as Typst math markup: numbers, units, quantities, names.
//!
//! Markup refers to a few names the Typst side puts in scope: `svsep` (between units), `svimp` (between steps),
//! `svgroup` (digit groups, when they are content), `svu` (unit-style), `svchip` (casts and types).

use serde::{Deserialize, Serialize};

use crate::dim::Dim;
use crate::complex::CVal;
use crate::expr::Val;
use crate::num::{Num, Rat};
use crate::units::{Prefixes, UnitExpr, Units, display_name};

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
    pub preferred_units: Vec<String>,
    #[serde(default = "d_true")]
    pub approximate: bool,
    #[serde(default = "d_true")]
    pub block: bool,
    #[serde(default = "d_maxsol")]
    pub max_solutions: usize,
    #[serde(default = "d_chip")]
    pub show_types: String,
    /// The word shown for any-unit types.
    pub words: Words,
    /// A unit-style function is set: units are wrapped in svu(…).
    pub unit_style: bool,
    /// How -> shows a symbolic result: from the clause (expand, factor), not an option.
    #[serde(skip)]
    pub symbolic: crate::poly::Form,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Words {
    pub any: String,
}
impl Default for Words {
    fn default() -> Self {
        Words { any: "any".into() }
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
        preferred_units: vec![],
        approximate: true,
        block: true,
        max_solutions: 6,
        show_types: "chip".into(),
        words: Words::default(),
        unit_style: false,
        symbolic: crate::poly::Form::Simplest,
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

fn compound_def(n: &str, units: &Units) -> bool {
    units.def(n).is_some_and(|d| d.compound)
}

/// Whether a unit's markup starts with a number (a display like `2.54 cm`, or the 1 of `1/s`), so that a number
/// or factor before it is joined by a dot rather than a space: 1 · 2.54 cm.
pub fn unit_numeric(u: &UnitExpr, units: &Units) -> bool {
    match u.0.iter().find(|t| t.1 > 0) {
        None => !u.0.is_empty(),
        // a compound display among other factors is parenthesized: (2.54 cm)·s
        Some((n, e)) => *e == 1 && units.def(n).is_some_and(|d| d.numeric && !(d.compound && u.0.len() > 1)),
    }
}

/// Whether a unit's markup is more than one atom: several units, a power, or a compound display.
pub fn unit_compound(u: &UnitExpr, units: &Units) -> bool {
    match u.0.as_slice() {
        [(n, 1)] => compound_def(n, units),
        _ => true,
    }
}

/// What joins a value and its unit: a thin space, or a dot before a unit that starts with a number.
pub fn unit_join(u: &UnitExpr, units: &Units) -> &'static str {
    if unit_numeric(u, units) { " dot " } else { " thin " }
}

/// A unit as markup: upright names, powers, · between them, / before what's below.
pub fn unit(u: &UnitExpr, units: &Units, cfg: &Cfg) -> String {
    if u.0.len() == 1 && u.0[0].0 == "any" {
        return format!("upright({})", q(&cfg.words.any));
    }
    // `below`: after the slash, where a compound display is one factor too
    let w = |ts: Vec<(&String, i32)>, below: bool| {
        ts.iter()
            .map(|(n, e)| {
                // a unit binding: as its display, which the Typst side knows once it was placed; its name meanwhile
                let s = if n.starts_with("sv") && n.len() == 66 { format!("#svref({}, {})", q(n), q(&display_name(n, units))) } else { format!("upright({})", q(&display_name(n, units))) };
                // a display of several atoms is one factor: (2.54 cm)², m/(2.54 cm), (2.54 cm)·s
                let s = if compound_def(n, units) && (*e != 1 || below || u.0.len() > 1) { format!("lr(({s}))") } else { s };
                if *e == 1 { s } else { format!("{s}^({e})") }
            })
            .collect::<Vec<_>>()
            .join(" #svsep ")
    };
    let up: Vec<_> = u.0.iter().filter(|t| t.1 > 0).map(|(n, e)| (n, *e)).collect();
    let down: Vec<_> = u.0.iter().filter(|t| t.1 < 0).map(|(n, e)| (n, -e)).collect();
    let mut s = if up.is_empty() { "1".into() } else { w(up, false) };
    if !down.is_empty() {
        // no spaces: next to a #call, math keeps them as spaces (m / s)
        s = format!("{s}\\/{}", w(down, true));
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

// Split scaling into finite factors: 10^-324 cannot itself be represented,
// but dividing a subnormal by that mathematical scale has a finite mantissa.
fn decimal_scale(x: f64, exponent: i32) -> f64 {
    if exponent < -308 { (x * 1e308) / 10f64.powi(exponent + 308) }
    else { x / 10f64.powi(exponent) }
}

/// A number rounded to `digits` significant digits: (mantissa, exponent if scientific).
pub fn fmt(x: f64, cfg: &Cfg) -> (String, Option<i32>) {
    if x == 0.0 || !x.is_finite() {
        return (if x == 0.0 { "0".into() } else { format!("{x}") }, None);
    }
    let n = cfg.digits as i32;
    let mut e = x.abs().log10().floor() as i32;
    let m = decimal_scale(x, e);
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
    let mut y = decimal_scale(x, p);
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

/// First configured preferred unit matching a dimension, without automatic prefixing.
pub fn preferred_unit(d: Dim, units: &Units, cfg: &Cfg) -> Option<UnitExpr> {
    if d.is_one() { return None; }
    cfg.preferred_units.iter().filter_map(|p| units.parse(p, false).ok())
        .find(|p| p.d == d).map(|p| p.u)
}

/// Which unit a value is shown in: the one asked for, else a preferred one, the one it was written in, the
/// non-SI unit of the inputs (hint), or SI with a prefix.
pub fn pick(v: &Val, want: Option<&UnitExpr>, hint: Option<&UnitExpr>, units: &Units, cfg: &Cfg) -> Result<Shown, String> {
    let si = v.v;
    if let Some(u) = want {
        let d = units.dim(u).ok_or_else(|| format!("unknown unit '{}'", units.text(u)))?;
        if d != v.d {
            return Err(format!(
                "the result has unit {}, not {}",
                if v.d.is_one() { "1 (no unit)".to_string() } else { units.text(&units.auto(&v.d)) },
                units.text(u)
            ));
        }
        let f = units.factor(u);
        return Ok(Shown { n: si.div(f).unwrap_or(si), u: u.clone(), raw: None, bare: false });
    }
    if let Some(u) = preferred_unit(v.d.clone(), units, cfg) {
        return Ok(Shown { n: si.div(units.factor(&u)).unwrap_or(si), u, raw: None, bare: false });
    }
    // the unit it was written in (7.2 kΩ), or a unit that isn't SI it came from (2 p in bar); a zero, which has
    // no size to pick a prefix by, in the unit it came with (0 ms)
    if let Some(u) = v.u.as_ref().filter(|u| v.raw.is_some() || si.is_zero() || !si_unit(u, units)) {
        if units.dim(u) == Some(v.d.clone()) {
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
        if units.dim(h) == Some(v.d.clone()) {
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
    u.0.iter().all(|(n, _)| match units.def(n) {
        Some(_) => units.si(n).is_some(),
        // a prefix on an SI unit: kΩ
        None => n.char_indices().nth(1).is_some_and(|(i, _)| units.si(&n[i..]).is_some_and(|(_, e)| e == 0)),
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
    (format!("{n}{}{}", unit_join(&s.u, units), unit(&s.u, units, cfg)), exact)
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
        out += &units.text(&s.u);
    }
    out
}

/// A complex value as markup, (re + im i) unit: both parts in the unit picked for its magnitude; (markup, exact).
/// `i` is shown as the worksheet shows the identity imaginary.
pub fn complex(z: &CVal, want: Option<&UnitExpr>, hint: Option<&UnitExpr>, units: &Units, cfg: &Cfg) -> Result<(String, bool), String> {
    if z.is_real() {
        return pick(&z.re, want, hint, units, cfg).map(|s| shown(&s, units, cfg));
    }
    let size = Val::new(Num::F(z.z().abs()), z.d().clone());
    let u = pick(&size, want, hint, units, cfg)?.u;
    let part = |v: &Val| -> Result<Num, String> {
        Ok(if u.is_empty() { v.v } else { pick(&Val { u: None, raw: None, ..v.clone() }, Some(&u), None, units, cfg)?.n })
    };
    let (re, im) = (part(&z.re)?, part(&z.im)?);
    let i = crate::render::constant("imaginary", None);
    let (im_s, im_exact) = number(im.abs(), None, cfg);
    let im_term = if im.abs().is_one() { i } else { format!("{im_s} {i}") };
    let (body, exact, sum) = if re.is_zero() {
        (format!("{}{im_term}", if im.is_neg() { "-" } else { "" }), im_exact, false)
    } else {
        let (re_s, re_exact) = number(re, None, cfg);
        (format!("{re_s} {} {im_term}", if im.is_neg() { "-" } else { "+" }), re_exact && im_exact, true)
    };
    if u.is_empty() {
        return Ok((body, exact));
    }
    let body = if sum { format!("lr(({body}))") } else { body };
    Ok((format!("{body}{}{}", unit_join(&u, units), unit(&u, units, cfg)), exact))
}

/// A complex value in plain text, for editors: "(3 + 4i) Ω".
pub fn complex_plain(z: &CVal, unit_: Option<&UnitExpr>, units: &Units, cfg: &Cfg) -> String {
    if z.is_real() {
        return plain(&z.re, unit_, units, cfg);
    }
    let c = Cfg { digits: cfg.digits.max(4), ..cfg.clone() };
    let size = Val::new(Num::F(z.z().abs()), z.d().clone());
    let u = pick(&size, unit_, None, units, &c).map(|s| s.u).unwrap_or_default();
    let n = |v: &Val| {
        let x = if u.is_empty() { v.v.f() } else { v.v.f() / units.factor(&u).f() };
        let (m, e) = fmt(x, &c);
        if let Some(e) = e { format!("{m}e{e}") } else { m }
    };
    let im = n(&Val { v: z.im.v.abs(), ..z.im.clone() });
    let body = if z.re.v.is_zero() { format!("{}{im}i", if z.im.v.is_neg() { "-" } else { "" }) } else { format!("{} {} {im}i", n(&z.re), if z.im.v.is_neg() { "-" } else { "+" }) };
    if u.is_empty() { return body; }
    let ut = units.text(&u);
    if z.re.v.is_zero() { format!("{body} {ut}") } else { format!("({body}) {ut}") }
}

pub fn dim_text(d: &Dim, units: &Units) -> String {
    if d.is_one() { "1 (no unit)".into() } else { units.text(&units.auto(d)) }
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
