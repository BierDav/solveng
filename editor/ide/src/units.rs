//! The units of solveng's `units` module: their names, prefixes and spellings, what they're made of, and the names
//! the module exports. Units in math are Typst bindings; this table describes the module's.

use std::sync::LazyLock;


use crate::{Item, ItemKind};

/// Units that take prefixes, with their names.
pub(crate) const PREFIXABLE: &[(&str, &str)] = &[
    ("m", "metre"), ("g", "gram"), ("s", "second"), ("A", "ampere"), ("K", "kelvin"), ("mol", "mole"),
    ("cd", "candela"), ("V", "volt"), ("Ω", "ohm"), ("W", "watt"), ("J", "joule"), ("C", "coulomb"),
    ("F", "farad"), ("H", "henry"), ("S", "siemens"), ("Hz", "hertz"), ("N", "newton"), ("Pa", "pascal"),
    ("T", "tesla"), ("Wb", "weber"), ("Ah", "ampere hour"), ("Wh", "watt hour"), ("As", "ampere second"),
    ("Ws", "watt second"), ("eV", "electronvolt"), ("l", "litre"), ("L", "litre"), ("rad", "radian"),
];
pub(crate) const PLAIN: &[(&str, &str)] = &[
    ("min", "minute"), ("h", "hour"), ("%", "percent"), ("‰", "per mille"), ("°", "degree"),
    ("deg", "degree"), ("°C", "degree Celsius (a difference)"), ("degC", "degree Celsius (a difference)"),
];
/// Typeable spellings solveng reads as the unit after them; with a prefix glued as in math: `k""Omega`.
pub(crate) const SPELLINGS: &[(&str, &str)] = &[("Omega", "Ω"), ("Ω", "Ω")];
pub(crate) const PREFIXES: &[(&str, &str)] = &[
    ("m", "milli"), ("k", "kilo"), ("u", "micro"), ("M", "mega"), ("n", "nano"), ("G", "giga"), ("p", "pico"),
    ("c", "centi"), ("d", "deci"), ("T", "tera"), ("f", "femto"), ("a", "atto"), ("µ", "micro"), ("μ", "micro"),
];
/// Prefixed units offered in completions; the rest still highlight.
const COMMON_PREFIXES: &[&str] = &["m", "k", "u", "M", "n", "G", "p"];

fn lookup(table: &[(&'static str, &'static str)], key: &str) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

fn spelled(name: &str) -> &str {
    lookup(SPELLINGS, name).unwrap_or(name)
}

/// The name of a built-in unit (`mAh` → `milliampere hour`), or None.
pub fn unit_name(name: &str) -> Option<String> {
    // k""Omega is kΩ
    if name.contains("\"\"") {
        return unit_name(&crate::resolved_name(name));
    }
    if let Some(n) = lookup(PLAIN, name) {
        return Some(n.into());
    }
    if let Some(n) = lookup(PREFIXABLE, spelled(name)) {
        return Some(n.into());
    }
    if name.chars().count() < 2 {
        return None;
    }
    for (p, pn) in PREFIXES {
        // kOmega isn't kΩ, as DeltaT isn't ΔT
        let Some(rest) = name.strip_prefix(p).filter(|r| *r != "Omega") else { continue };
        if let Some(n) = lookup(PREFIXABLE, spelled(rest)) {
            return Some(format!("{pn}{n}"));
        }
    }
    None
}

/// Whether `name` is a built-in unit, or a prefix and one (`kΩ`, `k""Omega`, `mAh`).
pub fn is_unit(name: &str) -> bool {
    unit_name(name).is_some()
}

/// What typing matches for a built-in unit, words apart: the unit (`kΩ`), how it's typed when that differs
/// (`kOmega`) and its name (`kiloohm`), so `ohm` and `volt` find Ω and V.
fn unit_filter(unit: &str, prefix: &str, name: &str) -> String {
    let mut words = vec![format!("{prefix}{unit}")];
    words.extend(SPELLINGS.iter().filter(|(s, u)| *u == unit && *s != unit).map(|(s, _)| format!("{prefix}{s}")));
    // degree Celsius (a difference): the name without what's in parentheses
    words.push(name.split(" (").next().unwrap_or(name).replace(' ', ""));
    words.join(" ")
}

/// The units of solveng's `units` module a completion offers: the base and common prefixed ones, the engineering
/// ones first. What typing matches includes their names: `ohm` finds Ω.
pub(crate) fn unit_items() -> Vec<Item> {
    let mut items = Vec::new();
    for (u, name) in PREFIXABLE {
        let mut item = Item::new(u, ItemKind::UNIT, &format!("{name} · solveng units"), u);
        item.filter = Some(unit_filter(u, "", name));
        items.push(item);
        for p in COMMON_PREFIXES {
            if *u == "g" && *p != "k" && *p != "m" {
                continue;
            }
            let label = format!("{p}{u}");
            let full = format!("{}{name}", lookup(PREFIXES, p).unwrap());
            let mut item = Item::new(&label, ItemKind::UNIT, &format!("{full} · solveng units"), &label);
            item.filter = Some(unit_filter(u, p, &full));
            items.push(item);
        }
    }
    for (u, name) in PLAIN {
        let mut item = Item::new(u, ItemKind::UNIT, &format!("{name} · solveng units"), u);
        item.filter = Some(unit_filter(u, "", name));
        items.push(item);
    }
    let mut seen = std::collections::HashSet::new();
    // Only what the module exports: `°` and `%` are no Typst names.
    items.retain(|i| seen.insert(i.label.clone()) && shorthand(&i.label));
    // Common engineering units precede uncommon prefixes.
    const COMMON: &[&str] = &["m", "s", "kg", "A", "K", "V", "Ω", "W", "Hz", "N", "Pa", "J", "C", "F", "H", "mol", "cd", "mm", "cm", "km", "ms", "mA", "mV", "kΩ", "kW", "kHz", "MHz", "µA", "uA", "nF", "µF", "uF", "pF", "h", "min", "degC", "L", "l", "mAh", "Wh", "kWh", "rad", "deg"];
    items.sort_by_key(|item| COMMON.iter().position(|name| *name == item.label).map_or((1, 0), |rank| (0, rank)));
    items
}

/// Built-in units in SI base units, exponents of (kg, m, s, A, K, mol, cd).
const DIMS: &[(&str, [i8; 7])] = &[
    ("m", [0, 1, 0, 0, 0, 0, 0]), ("g", [1, 0, 0, 0, 0, 0, 0]), ("s", [0, 0, 1, 0, 0, 0, 0]), ("A", [0, 0, 0, 1, 0, 0, 0]),
    ("K", [0, 0, 0, 0, 1, 0, 0]), ("mol", [0, 0, 0, 0, 0, 1, 0]), ("cd", [0, 0, 0, 0, 0, 0, 1]),
    ("V", [1, 2, -3, -1, 0, 0, 0]), ("Ω", [1, 2, -3, -2, 0, 0, 0]), ("W", [1, 2, -3, 0, 0, 0, 0]), ("J", [1, 2, -2, 0, 0, 0, 0]),
    ("C", [0, 0, 1, 1, 0, 0, 0]), ("F", [-1, -2, 4, 2, 0, 0, 0]), ("H", [1, 2, -2, -2, 0, 0, 0]), ("S", [-1, -2, 3, 2, 0, 0, 0]),
    ("Hz", [0, 0, -1, 0, 0, 0, 0]), ("N", [1, 1, -2, 0, 0, 0, 0]), ("Pa", [1, -1, -2, 0, 0, 0, 0]), ("T", [1, 0, -2, -1, 0, 0, 0]),
    ("Wb", [1, 2, -2, -1, 0, 0, 0]), ("Ah", [0, 0, 1, 1, 0, 0, 0]), ("As", [0, 0, 1, 1, 0, 0, 0]), ("Wh", [1, 2, -2, 0, 0, 0, 0]),
    ("Ws", [1, 2, -2, 0, 0, 0, 0]), ("eV", [1, 2, -2, 0, 0, 0, 0]), ("l", [0, 3, 0, 0, 0, 0, 0]), ("L", [0, 3, 0, 0, 0, 0, 0]),
    ("min", [0, 0, 1, 0, 0, 0, 0]), ("h", [0, 0, 1, 0, 0, 0, 0]), ("°C", [0, 0, 0, 0, 1, 0, 0]), ("degC", [0, 0, 0, 0, 1, 0, 0]),
];
/// What built-in units are made of, when not a base unit.
const DEFINITIONS: &[(&str, &str)] = &[
    ("g", "10⁻³ kg"), ("V", "W/A"), ("Ω", "V/A"), ("W", "J/s"), ("J", "N·m"), ("C", "A·s"), ("F", "C/V"), ("H", "Wb/A"),
    ("S", "1/Ω"), ("Hz", "1/s"), ("N", "kg·m/s²"), ("Pa", "N/m²"), ("T", "Wb/m²"), ("Wb", "V·s"), ("Ah", "3600 C"),
    ("As", "C"), ("Wh", "3600 J"), ("Ws", "J"), ("eV", "1.602176634·10⁻¹⁹ J"), ("l", "10⁻³ m³"), ("L", "10⁻³ m³"),
    ("rad", "m/m = 1"), ("min", "60 s"), ("h", "60 min"), ("%", "10⁻²"), ("‰", "10⁻³"), ("°", "π/180 rad"),
    ("deg", "π/180 rad"), ("°C", "K, as a difference"), ("degC", "K, as a difference"),
];
const BASE: &[&str] = &["kg", "m", "s", "A", "K", "mol", "cd"];
const POWERS: &[(&str, i32)] = &[
    ("m", -3), ("k", 3), ("u", -6), ("µ", -6), ("μ", -6), ("M", 6), ("n", -9), ("G", 9), ("p", -12), ("c", -2), ("d", -1),
    ("T", 12), ("f", -15), ("a", -18),
];

fn superscript(n: i32) -> String {
    n.to_string().chars().map(|c| if c == '-' { '⁻' } else { "⁰¹²³⁴⁵⁶⁷⁸⁹".chars().nth(c.to_digit(10).unwrap_or(0) as usize).unwrap_or(c) }).collect()
}

/// SI base units of exponents: kg·m²·s⁻³·A⁻².
fn si(d: &[i8; 7]) -> String {
    let names = ["kg", "m", "s", "A", "K", "mol", "cd"];
    let part = |n: &str, e: i8| if e == 1 { n.to_string() } else { format!("{n}{}", superscript(e as i32)) };
    let up = (0..7).filter(|&i| d[i] > 0).map(|i| part(names[i], d[i]));
    let down = (0..7).filter(|&i| d[i] < 0).map(|i| part(names[i], d[i]));
    up.chain(down).collect::<Vec<_>>().join("·")
}

/// A built-in unit's heritage, a line each: `kΩ = 10³ Ω`, `Ω = V/A = kg·m²·s⁻³·A⁻²`; empty for an SI base unit.
fn heritage(word: &str) -> Vec<String> {
    let word = crate::resolved_name(word);
    let word = spelled(&word).to_string();
    let mut lines = Vec::new();
    if BASE.contains(&word.as_str()) {
        return lines;
    }
    let base = if lookup(PLAIN, &word).is_some() || lookup(PREFIXABLE, &word).is_some() {
        word.clone()
    } else {
        let Some((p, power, rest)) = POWERS.iter().find_map(|(p, e)| word.strip_prefix(p).map(|r| (p, e, spelled(r).to_string()))).filter(|(_, _, r)| lookup(PREFIXABLE, r).is_some()) else {
            return lines;
        };
        let _ = p;
        lines.push(format!("{word} = 10{} {rest}", superscript(*power)));
        rest
    };
    let def = lookup(DEFINITIONS, &base);
    let dims = DIMS.iter().find(|(n, _)| *n == base).map(|(_, d)| si(d));
    match (def, dims) {
        (Some(def), Some(si)) if !def.ends_with(&si) && def != si && !def.contains("difference") => lines.push(format!("{base} = {def} = {si}")),
        (Some(def), _) => lines.push(format!("{base} = {def}")),
        (None, _) => {}
    }
    lines
}

/// What a unit of solveng's `units` module is, for hovers: its name and kind, then where it comes from down to the SI
/// base units, a line each: `**kΩ** kiloohm · built-in unit`, `kΩ = 10³ Ω`, `Ω = V/A = kg·m²·s⁻³·A⁻²`.
pub(crate) fn describe(word: &str) -> Option<String> {
    let name = unit_name(word)?;
    let kind = if BASE.contains(&spelled(&crate::resolved_name(word))) { "built-in SI base unit" } else { "built-in unit" };
    let title = format!("`{word}` {name} · {kind}");
    let lines = heritage(word);
    Some(if lines.is_empty() { title } else { format!("{title}\n\n{}", lines.iter().map(|l| format!("`{l}`")).collect::<Vec<_>>().join(" · ")) })
}

/// The names of solveng's `units` module, for `#import units: *`, read from the package.
static SHORTHANDS: LazyLock<Vec<String>> = LazyLock::new(|| {
    include_str!("../../../src/unit-words.typ").lines().filter_map(|line| line.strip_prefix("#let "))
        .filter_map(|line| line.split_once(" = ").map(|(name, _)| name.trim().to_string()))
        .filter(|name| !name.starts_with('_') && !name.contains('(')).collect()
});

/// Whether solveng's `units` module exports `name`.
pub(crate) fn shorthand(name: &str) -> bool {
    SHORTHANDS.iter().any(|n| n == name)
}

/// Every name of solveng's `units` module.
pub(crate) fn shorthands() -> Vec<String> {
    SHORTHANDS.clone()
}
