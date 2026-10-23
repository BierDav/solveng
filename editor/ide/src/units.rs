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

/// A unit the document defines, with what it stands for.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Custom {
    pub name: String,
    /// `100 kPa`, `auto` for a base unit of its own; empty when only the compiler knows it exists.
    pub value: String,
}

static UNITS_ARG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"units\s*:\s*\(").unwrap());
static UNIT_ENTRY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"([\p{L}][\p{L}\p{N}_]*|"[^"]+")\s*:\s*("[^"]*"|none|auto|[\d.]+)"#).unwrap());

/// Units a text adds with `worksheet(units: (bar: "100 kPa", ü: auto, dozen: 12, …))`, later ones winning.
pub(crate) fn custom_units(text: &str, out: &mut Vec<Custom>) {
    static BUILTIN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"builtin-units\s*:\s*(true|false)").unwrap());
    for c in BUILTIN.captures_iter(text) {
        out.retain(|u| !u.name.is_empty());
        out.push(Custom { name: String::new(), value: c[1].into() });
    }
    for m in UNITS_ARG.find_iter(text) {
        let start = m.end();
        let end = text[start..].find(')').map_or(text.len(), |i| start + i);
        for c in UNIT_ENTRY.captures_iter(&text[start..end]) {
            let name = crate::resolved_name(c[1].trim_matches('"'));
            out.retain(|u| u.name != name);
            out.push(Custom { name, value: c[2].trim_matches('"').to_string() });
        }
    }
}

static FUNCTIONS_ARG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|[^\w-])functions\s*:\s*\(").unwrap());
static FUNCTION_ENTRY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([\p{L}][\p{L}\p{N}_]*|"[^"]+")\s*:\s*("[^"]*"|none)"#).unwrap());

/// Names a text maps with `worksheet(functions: (conv: "convolve", tri: none))`: the operation each stands for, or
/// None for a hidden name; later ones winning.
pub(crate) fn custom_functions(text: &str) -> Vec<(String, Option<String>)> {
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    for m in FUNCTIONS_ARG.find_iter(text) {
        let start = m.end();
        let end = text[start..].find(')').map_or(text.len(), |i| start + i);
        for c in FUNCTION_ENTRY.captures_iter(&text[start..end]) {
            let name = crate::resolved_name(c[1].trim_matches('"'));
            let target = (&c[2] != "none").then(|| crate::resolved_name(c[2].trim_matches('"')));
            out.retain(|(n, _)| *n != name);
            out.push((name, target));
        }
    }
    // a name standing for another of the overlay: what that one stands for
    let resolved: Vec<(String, Option<String>)> = out.iter().map(|(n, t)| {
        let mut t = t.clone();
        for _ in 0..8 {
            match t.as_ref().and_then(|x| out.iter().find(|(m, _)| m == x && m != n)) {
                Some((_, next)) if next.is_some() => t = next.clone(),
                _ => break,
            }
        }
        (n.clone(), t)
    }).collect();
    resolved
}

static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}°%‰µΩ]+").unwrap());

/// What a custom unit stands for, with the custom units it uses spelled out: `= 10 bar (bar = 100 kPa)`.
pub(crate) fn resolve(unit: &Custom, all: &[Custom]) -> String {
    if unit.value.is_empty() {
        return "defined unit".into();
    }
    if unit.value == "auto" {
        return "defined base unit".into();
    }
    let mut out = format!("= {}", unit.value);
    let mut seen = vec![unit.name.clone()];
    let mut queue = vec![unit.value.clone()];
    while let Some(value) = queue.pop() {
        for w in WORD.find_iter(&value) {
            let Some(u) = all.iter().find(|u| u.name == w.as_str()) else { continue };
            if seen.contains(&u.name) || u.value.is_empty() || u.value == "auto" || seen.len() > 8 {
                continue;
            }
            seen.push(u.name.clone());
            out.push_str(&format!(" ({} = {})", u.name, u.value));
            queue.push(u.value.clone());
        }
    }
    out
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

/// Every unit a completion offers: the document's first, then the built-in ones with common prefixes.
pub(crate) fn unit_items(custom: &[Custom]) -> Vec<Item> { items(custom, true) }
pub(crate) fn explicit_items(custom: &[Custom]) -> Vec<Item> { items(custom, false) }

fn items(custom: &[Custom], implicit: bool) -> Vec<Item> {
    let mut items = Vec::new();
    for u in custom.iter().filter(|u| !u.name.is_empty() && u.value != "none") {
        let r = resolve(u, custom);
        let detail = if r.starts_with("defined") { r } else { format!("defined · {r}") };
        let mut item = Item::new(&u.name, ItemKind::UNIT, &detail, &u.name);
        let spelling = crate::conceal::spelled(&u.name);
        if spelling != u.name { item.filter = Some(format!("{} {spelling}", u.name)); }
        items.push(item);
    }
    for (u, name) in PREFIXABLE {
        let mut item = Item::new(u, ItemKind::UNIT, &format!("{name} · built-in"), u);
        item.filter = Some(unit_filter(u, "", name));
        items.push(item);
        for p in COMMON_PREFIXES {
            if *u == "g" && *p != "k" && *p != "m" {
                continue;
            }
            let label = format!("{p}{u}");
            let full = format!("{}{name}", lookup(PREFIXES, p).unwrap());
            let mut item = Item::new(&label, ItemKind::UNIT, &format!("{full} · built-in"), &label);
            item.filter = Some(unit_filter(u, p, &full));
            items.push(item);
        }
    }
    for (u, name) in PLAIN {
        let mut item = Item::new(u, ItemKind::UNIT, &format!("{name} · built-in"), u);
        item.filter = Some(unit_filter(u, "", name));
        items.push(item);
    }
    let mut seen = std::collections::HashSet::new();
    items.retain(|i| seen.insert(i.label.clone()));
    if implicit { items.retain(|item| visible(&item.insert, custom)); }
    // Common engineering units precede uncommon prefixes; custom overlays stay first.
    const COMMON: &[&str] = &["m", "s", "kg", "A", "K", "V", "Ω", "W", "Hz", "N", "Pa", "J", "C", "F", "H", "mol", "cd", "mm", "cm", "km", "ms", "mA", "mV", "kΩ", "kW", "kHz", "MHz", "µA", "uA", "nF", "µF", "uF", "pF", "h", "min", "°C", "L", "l", "mAh", "Wh", "kWh", "%", "rad", "°"];
    items.sort_by_key(|item| {
        if custom.iter().any(|u| u.name == item.label) { (0, 0) }
        else if let Some(rank) = COMMON.iter().position(|name| *name == item.label) { (1, rank) }
        else if PREFIXABLE.iter().chain(PLAIN).any(|(name, _)| *name == item.label) { (2, 0) }
        else { (3, 0) }
    });
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

/// What a unit is, for hovers: its name and kind, then where it comes from down to the SI base units, a line each:
/// `**kΩ** kiloohm · built-in unit`, `kΩ = 10³ Ω`, `Ω = V/A = kg·m²·s⁻³·A⁻²`; `**bar** defined unit`, `bar = 100 kPa`, ….
pub(crate) fn describe(word: &str, custom: &[Custom]) -> Option<String> {
    let (title, lines) = if let Some(u) = custom.iter().find(|u| u.name == word) {
        if u.value == "auto" {
            (format!("`{word}` defined base unit: a dimension of its own"), Vec::new())
        } else {
            // the defined units it's made of, then the built-in ones those use
            let mut lines = Vec::new();
            let mut queue = vec![u.clone()];
            let mut builtin = Vec::new();
            while let Some(d) = queue.pop() {
                if d.value.is_empty() || lines.len() > 8 {
                    continue;
                }
                lines.push(format!("{} = {}", d.name, d.value));
                for w in WORD.find_iter(&d.value).map(|w| w.as_str().to_string()) {
                    match custom.iter().find(|c| c.name == w) {
                        Some(c) if c.value != "auto" && !lines.iter().any(|l| l.starts_with(&format!("{} =", c.name))) => queue.push(c.clone()),
                        Some(_) => {}
                        None if is_unit(&w) && !builtin.contains(&w) => builtin.push(w),
                        None => {}
                    }
                }
            }
            for w in builtin {
                for l in heritage(&w) {
                    if !lines.contains(&l) {
                        lines.push(l);
                    }
                }
            }
            (format!("`{word}` defined unit"), lines)
        }
    } else {
        let name = unit_name(word)?;
        let kind = if BASE.contains(&spelled(&crate::resolved_name(word))) { "built-in SI base unit" } else { "built-in unit" };
        (format!("`{word}` {name} · {kind}"), heritage(word))
    };
    Some(if lines.is_empty() { title } else { format!("{title}\n\n{}", lines.iter().map(|l| format!("`{l}`")).collect::<Vec<_>>().join(" · ")) })
}

/// Whether solveng reads `word` as a unit: one of its table or the document's, or any prefix and one.
pub(crate) fn known(word: &str, custom: &[Custom]) -> bool {
    let normalized = crate::resolved_name(word);
    let word = normalized.as_str();
    if word.contains("\"\"") {
        return known(&crate::resolved_name(word), custom);
    }
    let table = |w: &str| {
        lookup(PREFIXABLE, w).is_some() || lookup(PLAIN, w).is_some() || lookup(SPELLINGS, w).is_some() || w == "\u{2126}"
            || custom.iter().any(|u| u.name == w && u.value != "none")
    };
    if table(word) {
        return true;
    }
    let mut c = word.chars();
    let first = c.next().map(String::from).unwrap_or_default();
    let rest = c.as_str();
    !rest.is_empty() && rest != "Omega" && lookup(PREFIXES, &first).is_some() && table(rest)
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

/// A hidden name remains available to explicit unit references and definition expressions.
pub(crate) fn visible(word: &str, custom: &[Custom]) -> bool {
    let normalized = crate::resolved_name(word);
    let word = normalized.as_str();
    if custom.iter().any(|u| u.name == word && u.value == "none") { return false; }
    let builtin = custom.iter().find(|u| u.name.is_empty()).is_none_or(|u| u.value != "false");
    known(word, custom) && (builtin || !is_unit(word) || custom.iter().any(|u| u.name == word && u.value != "none"))
}

/// User overlays do not create Typst bindings for bare names.
pub(crate) fn bare(word: &str, custom: &[Custom]) -> bool {
    let normalized = crate::resolved_name(word);
    let word = normalized.as_str();
    is_unit(word) && custom.iter().find(|u| u.name.is_empty()).is_none_or(|u| u.value != "false")
        && !custom.iter().any(|u| u.name == word && u.value == "none")
}
