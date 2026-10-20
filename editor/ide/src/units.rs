//! solveng's units: the built-in ones with prefixes and spellings, and the document's own.

use std::sync::LazyLock;

use regex::Regex;

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
/// Typeable spellings solveng reads as the unit after them.
pub(crate) const SPELLINGS: &[(&str, &str)] = &[("Ohm", "Ω"), ("ohm", "Ω"), ("Omega", "Ω"), ("Ω", "Ω")];
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
        let Some(rest) = name.strip_prefix(p) else { continue };
        if let Some(n) = lookup(PREFIXABLE, spelled(rest)) {
            return Some(format!("{pn}{n}"));
        }
    }
    None
}

/// Whether `name` is a built-in unit, or a prefix and one (`kOhm`, `mAh`).
pub fn is_unit(name: &str) -> bool {
    unit_name(name).is_some()
}

/// A unit the document defines, with what it stands for.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Custom {
    pub name: String,
    /// `100 kPa`; empty when only the compiler knows it exists.
    pub value: String,
}

static UNITS_ARG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"units\s*:\s*\(").unwrap());
static UNIT_ENTRY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"([\p{L}][\p{L}\p{N}_]*|"[^"]+")\s*:\s*("[^"]*"|none)"#).unwrap());

/// Units a text adds with `solve-config(units: (bar: "100 kPa", …))`, later ones winning.
pub(crate) fn custom_units(text: &str, out: &mut Vec<Custom>) {
    for m in UNITS_ARG.find_iter(text) {
        let start = m.end();
        let end = text[start..].find(')').map_or(text.len(), |i| start + i);
        for c in UNIT_ENTRY.captures_iter(&text[start..end]) {
            let name = c[1].trim_matches('"').to_string();
            out.retain(|u| u.name != name);
            if &c[2] != "none" {
                out.push(Custom { name, value: c[2].trim_matches('"').to_string() });
            }
        }
    }
}

static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}°%‰µΩ]+").unwrap());

/// What a custom unit stands for, with the custom units it uses spelled out: `= 10 bar (bar = 100 kPa)`.
pub(crate) fn resolve(unit: &Custom, all: &[Custom]) -> String {
    if unit.value.is_empty() {
        return "custom unit".into();
    }
    let mut out = format!("= {}", unit.value);
    let mut seen = vec![unit.name.clone()];
    let mut queue = vec![unit.value.clone()];
    while let Some(value) = queue.pop() {
        for w in WORD.find_iter(&value) {
            let Some(u) = all.iter().find(|u| u.name == w.as_str()) else { continue };
            if seen.contains(&u.name) || u.value.is_empty() || seen.len() > 8 {
                continue;
            }
            seen.push(u.name.clone());
            out.push_str(&format!(" ({} = {})", u.name, u.value));
            queue.push(u.value.clone());
        }
    }
    out
}

/// Every unit a completion offers: the document's first, then the built-in ones with common prefixes.
pub(crate) fn unit_items(custom: &[Custom]) -> Vec<Item> {
    let mut items = Vec::new();
    for u in custom {
        items.push(Item::new(&u.name, ItemKind::UNIT, &resolve(u, custom), &u.name));
    }
    for (u, name) in PREFIXABLE {
        let typed = if *u == "Ω" { "Ohm" } else { u };
        items.push(Item::new(u, ItemKind::UNIT, name, typed));
        for p in COMMON_PREFIXES {
            if *u == "g" && *p != "k" && *p != "m" {
                continue;
            }
            let label = format!("{p}{typed}");
            let full = format!("{}{name}", lookup(PREFIXES, p).unwrap());
            items.push(Item::new(&label, ItemKind::UNIT, &full, &label));
        }
    }
    for (u, name) in PLAIN {
        items.push(Item::new(u, ItemKind::UNIT, name, u));
    }
    let mut seen = std::collections::HashSet::new();
    items.retain(|i| seen.insert(i.label.clone()));
    items
}

/// What a unit is, for hovers: `kΩ: kiloohm`, `bar = 100 kPa`.
pub(crate) fn describe(word: &str, custom: &[Custom]) -> Option<String> {
    if let Some(u) = custom.iter().find(|u| u.name == word) {
        return Some(format!("`{word}` {}", resolve(u, custom)));
    }
    unit_name(word).map(|n| format!("`{word}` {n}"))
}

/// Whether solveng reads `word` as a unit: one of its table or the document's, or any prefix and one.
pub(crate) fn known(word: &str, custom: &[Custom]) -> bool {
    let table = |w: &str| {
        lookup(PREFIXABLE, w).is_some() || lookup(PLAIN, w).is_some() || lookup(SPELLINGS, w).is_some() || w == "\u{2126}"
            || custom.iter().any(|u| u.name == w)
    };
    if table(word) {
        return true;
    }
    let mut c = word.chars();
    let first = c.next().map(String::from).unwrap_or_default();
    let rest = c.as_str();
    !rest.is_empty() && lookup(PREFIXES, &first).is_some() && table(rest)
}

/// Unit words of a unit expression (`kg m/s^2`) that neither solveng nor the document knows, as character ranges.
pub(crate) fn unknown_words(expr: &str, custom: &[Custom]) -> Vec<(usize, usize, String)> {
    let words: Vec<_> = WORD.find_iter(expr).collect();
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if known(w.as_str(), custom) {
            continue;
        }
        // A prefix written apart: "k Ω".
        let next = words.get(i + 1).map(|n| n.as_str());
        if lookup(PREFIXES, w.as_str()).is_some() && next.is_some_and(|n| known(n, custom)) {
            continue;
        }
        out.push((crate::text::char_index(expr, w.start()), crate::text::char_index(expr, w.end()), w.as_str().to_string()));
    }
    out
}
