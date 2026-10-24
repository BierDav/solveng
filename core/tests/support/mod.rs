//! The test harness: worksheets as production writes them, in math mode with explicit references.
//!
//! Fixtures stay readable (`I := 30 mA`, `R := [4.7 kΩ]`, `t -> [h]`, `tri(1/4) =`, `x^2 = 4 -> solve(x)`);
//! [`src`] turns them into what Typst sends: units and constants as binding tokens `⟦…⟧`, operations as `⦇tri⦈`,
//! clause keywords as `⦅solve(x)⦆`. [`readable`] maps result markup back (`#svref("sv…", "mA")` → `upright("mA")`,
//! `#svop("sin") (x)` → `sin(x)`).
//!
//! What a word becomes:
//! - `#name`: always the unit or constant token of that name (`#m`, `#h`, `#i`, `#e`, `#pi`);
//! - a known unit name: a unit token in `[…]` (code-style brackets, which become parentheses), after a number
//!   (`5 km`, `2 s`), after `:` (`f(x: Ω)`), continuing a unit (`72 km/h`, `Ω mm^2/m`), and in the trailing
//!   clause when it has more than one letter (`-> km`, `= mV`);
//! - `e`, `pi`, `π`: the constants;
//! - an operation name: `⦇name⦈`;
//! - a keyword in the clause after the trailing `->`/`=`: `⦅…⦆`;
//! - names the worksheet defines, or the line defines (left of `:=`) or solves for, stay names.
//!
//! The built-in unit table of the engine is not used: the units are this harness's own bindings.
#![allow(dead_code)]
use std::collections::{BTreeSet, HashMap};
use std::sync::{Mutex, OnceLock};

use solveng_core::{api::{self, LineReq}, bindings::{self, Binding, BindingReq}, dim::{Dim, IDENTITIES}, env::Env, eval::value, expr::Expr, format::Cfg, sheet::Out, units::Units};

/// The worksheet configuration production uses: no built-in unit names, no automatic operation names.
pub fn cfg() -> Cfg {
    Cfg::default()
}

// ---------- units and constants ----------

/// SI base units: (name, dimension identity).
const BASE: &[(&str, &str)] = &[("m", IDENTITIES[0]), ("kg", IDENTITIES[1]), ("s", IDENTITIES[2]), ("A", IDENTITIES[3]), ("K", IDENTITIES[4]), ("mol", IDENTITIES[5]), ("cd", IDENTITIES[6])];

/// Every other unit: (name, factor in SI, exponents of m, kg, s, A, K, mol, cd).
const DERIVED: &[(&str, &str, [i32; 7])] = &[
    ("g", "frac(1, 1000)", [0, 1, 0, 0, 0, 0, 0]),
    ("V", "1", [2, 1, -3, -1, 0, 0, 0]),
    ("C", "1", [0, 0, 1, 1, 0, 0, 0]),
    ("J", "1", [2, 1, -2, 0, 0, 0, 0]),
    ("Ω", "1", [2, 1, -3, -2, 0, 0, 0]),
    ("\u{2126}", "1", [2, 1, -3, -2, 0, 0, 0]),
    ("Omega", "1", [2, 1, -3, -2, 0, 0, 0]),
    ("W", "1", [2, 1, -3, 0, 0, 0, 0]),
    ("S", "1", [-2, -1, 3, 2, 0, 0, 0]),
    ("F", "1", [-2, -1, 4, 2, 0, 0, 0]),
    ("H", "1", [2, 1, -2, -2, 0, 0, 0]),
    ("Hz", "1", [0, 0, -1, 0, 0, 0, 0]),
    ("N", "1", [1, 1, -2, 0, 0, 0, 0]),
    ("Pa", "1", [-1, 1, -2, 0, 0, 0, 0]),
    ("T", "1", [0, 1, -2, -1, 0, 0, 0]),
    ("Wb", "1", [2, 1, -2, -1, 0, 0, 0]),
    ("min", "60", [0, 0, 1, 0, 0, 0, 0]),
    ("h", "3600", [0, 0, 1, 0, 0, 0, 0]),
    ("Ah", "3600", [0, 0, 1, 1, 0, 0, 0]),
    ("As", "1", [0, 0, 1, 1, 0, 0, 0]),
    ("Wh", "3600", [2, 1, -2, 0, 0, 0, 0]),
    ("Ws", "1", [2, 1, -2, 0, 0, 0, 0]),
    ("eV", "1602176634 * 10^(-28)", [2, 1, -2, 0, 0, 0, 0]),
    ("l", "frac(1, 1000)", [3, 0, 0, 0, 0, 0, 0]),
    ("L", "frac(1, 1000)", [3, 0, 0, 0, 0, 0, 0]),
    ("%", "frac(1, 100)", [0; 7]),
    ("‰", "frac(1, 1000)", [0; 7]),
    ("°C", "1", [0, 0, 0, 0, 1, 0, 0]),
    ("degC", "1", [0, 0, 0, 0, 1, 0, 0]),
    ("°", "frac(3.141592653589793, 180)", [0; 7]),
    ("deg", "frac(3.141592653589793, 180)", [0; 7]),
    ("rad", "1", [0; 7]),
];

const PREFIXES: &[(&str, i32)] = &[
    ("a", -18), ("f", -15), ("p", -12), ("n", -9), ("µ", -6), ("μ", -6), ("u", -6), ("m", -3), ("c", -2), ("d", -1),
    ("k", 3), ("M", 6), ("G", 9), ("T", 12),
];

fn symbol(name: &str) -> &str {
    match name {
        "Omega" | "\u{2126}" => "Ω",
        "degC" => "°C",
        "deg" => "°",
        n => n,
    }
}

/// A unit name: its display, factor source and exponents; or a base unit (display, identity).
enum Def {
    Base(String, &'static str),
    Derived(String, String, [i32; 7]),
}

fn def(name: &str) -> Option<Def> {
    if let Some((n, identity)) = BASE.iter().find(|(n, _)| *n == name) {
        return Some(Def::Base(n.to_string(), identity));
    }
    if let Some((n, f, e)) = DERIVED.iter().find(|(n, ..)| *n == name) {
        return Some(Def::Derived(symbol(n).to_string(), f.to_string(), *e));
    }
    let p = name.chars().next()?;
    let rest = &name[p.len_utf8()..];
    let (_, exp) = PREFIXES.iter().find(|(q, _)| q.starts_with(p) && q.chars().count() == 1)?;
    let shown = if matches!(p, 'u' | 'μ') { 'µ' } else { p };
    if rest == "Omega" || rest.is_empty() { return None; }
    if !rest.chars().next().is_some_and(char::is_alphabetic) && !rest.starts_with('Ω') { return None; }
    let (display, factor, e) = match def(rest)? {
        // kg is the base; g carries its own factor
        Def::Base(n, _) if n == "kg" => return None,
        Def::Base(n, _) => (n.clone(), "1".to_string(), BASE.iter().position(|(b, _)| *b == n).map(|i| { let mut e = [0; 7]; e[i] = 1; e }).unwrap()),
        Def::Derived(d, f, e) => (d, f, e),
    };
    Some(Def::Derived(format!("{shown}{display}"), format!("{factor} * 10^({exp})"), e))
}

struct Made {
    binding: Binding,
    token: String,
    display: String,
}

fn cache() -> &'static Mutex<HashMap<String, Option<&'static Made>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<&'static Made>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Displays by binding id, to show results readably.
fn ids() -> &'static Mutex<HashMap<String, String>> {
    static IDS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    IDS.get_or_init(Default::default)
}

fn construct(kind: &str, src: String, identity: Option<&str>, display: &str) -> &'static Made {
    let binding = bindings::construct(BindingReq { kind: kind.into(), src: src.clone(), identity: identity.map(str::to_owned), presentation: display.into(), description: display.into(), auto: false })
        .unwrap_or_else(|e| panic!("{kind} {display} := {src}: {e}"));
    let token = format!("⟦{}⟧", binding.encode().unwrap());
    ids().lock().unwrap().insert(binding.id.clone(), display.to_string());
    Box::leak(Box::new(Made { binding, token, display: display.to_string() }))
}

fn made(name: &str) -> Option<&'static Made> {
    if let Some(m) = cache().lock().unwrap().get(name) { return *m; }
    let m = match name {
        "pi" | "π" => Some(construct("constant", "3.141592653589793".into(), Some("pi"), "π")),
        "e" => Some(construct("constant", "2.718281828459045".into(), Some("euler"), "e")),
        "i" => Some(construct("constant", "⦇sqrt⦈(-1)".into(), Some("imaginary"), "i")),
        _ => match def(name) {
            None => None,
            Some(Def::Base(n, identity)) => Some(construct("base-unit", String::new(), Some(identity), &n)),
            Some(Def::Derived(display, factor, e)) => {
                let bases: Vec<String> = e.iter().enumerate().filter(|(_, p)| **p != 0)
                    .map(|(i, p)| format!("{}^({p})", unit(BASE[i].0))).collect();
                Some(construct("unit", format!("{factor} {}", bases.join(" ")), None, &display))
            }
        },
    };
    cache().lock().unwrap().insert(name.to_string(), m);
    m
}

/// Whether a name is a unit of the harness.
pub fn is_unit(name: &str) -> bool {
    !matches!(name, "pi" | "π" | "e" | "i") && made(name).is_some()
}

/// The token of a unit (`mA`, `kΩ`, `h`, `°C`).
pub fn unit(name: &str) -> String {
    assert!(def(name).is_some(), "no unit '{name}'");
    made(name).unwrap().token.clone()
}

/// The binding of a unit or constant.
pub fn binding(name: &str) -> Binding {
    made(name).unwrap_or_else(|| panic!("no unit or constant '{name}'")).binding.clone()
}

/// The token of a constant: `pi`, `e`, `i`.
pub fn constant(name: &str) -> String {
    assert!(matches!(name, "pi" | "π" | "e" | "i"), "no constant '{name}'");
    made(name).unwrap().token.clone()
}

/// The dimension of a unit.
pub fn dim(name: &str) -> Dim {
    value(&binding(name).expr).unwrap().d
}

/// The units a worksheet knows: those of its bindings.
pub fn units(env: &Env) -> Units {
    Units::lexical(&cfg(), env, "").unwrap()
}

/// An expression with the worksheet's values put in.
pub fn bind(env: &Env, e: &Expr) -> Expr {
    env.bind_with_units(e, &BTreeSet::new(), &units(env)).unwrap_or_else(|e| panic!("{e}"))
}

// ---------- sources ----------

const OPERATIONS_SKIPPED: &[&str] = &["mat", "frac"];
const KEYWORDS: &[&str] = &["solve", "substitute", "simplify", "digits", "sci", "eng", "frac", "laplace", "invlaplace"];

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '´' | '`')
}

/// The source as production sends it: see the module documentation.
pub fn src(text: &str, env: &Env) -> String {
    let cs: Vec<char> = text.chars().collect();
    // the top-level structure: where the definition and the clause are
    let (mut depth, mut i, mut def_at, mut clause_at) = (0i32, 0usize, None, None);
    let mut arrow = false;
    while i < cs.len() {
        let c = cs[i];
        match c {
            '"' => { i += 1; while i < cs.len() && cs[i] != '"' { if cs[i] == '\\' { i += 1; } i += 1; } }
            '⟦' | '⦇' | '⦅' => { while i < cs.len() && !matches!(cs[i], '⟧' | '⦈' | '⦆') { i += 1; } }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ':' if depth == 0 && cs.get(i + 1) == Some(&'=') => { if def_at.is_none() { def_at = Some(i); } i += 1; }
            '≔' if depth == 0 && def_at.is_none() => def_at = Some(i),
            '-' if depth == 0 && cs.get(i + 1) == Some(&'>') => { clause_at = Some(i + 2); arrow = true; i += 1; }
            '→' if depth == 0 => { clause_at = Some(i + 1); arrow = true; }
            '=' if depth == 0 && !arrow && !matches!(i.checked_sub(1).map(|j| cs[j]), Some('<' | '>' | '!' | ':' | '=')) => clause_at = Some(i + 1),
            _ => {}
        }
        i += 1;
    }
    let clause_at = clause_at.filter(|c| def_at.is_none_or(|d| *c > d + 1)).unwrap_or(usize::MAX);
    // names this line defines or solves for
    let mut declared: BTreeSet<String> = BTreeSet::new();
    let words = |from: usize, to: usize, typed: bool, declared: &mut BTreeSet<String>| {
        let mut i = from;
        let mut bracket = 0;
        while i < to.min(cs.len()) {
            let c = cs[i];
            if c == '"' { i += 1; while i < cs.len() && cs[i] != '"' { if cs[i] == '\\' { i += 1; } i += 1; } i += 1; continue; }
            if c == '[' && !(i > 0 && word_char(cs[i - 1])) { bracket += 1; }
            if c == ']' && bracket > 0 { bracket -= 1; }
            if c.is_alphabetic() {
                let start = i;
                while i < cs.len() && word_char(cs[i]) { i += 1; }
                let before: String = cs[..start].iter().collect();
                let before = before.trim_end();
                let w: String = cs[start..i].iter().collect();
                let after_number = before.chars().last().is_some_and(|c| c.is_ascii_digit());
                if !typed || (bracket == 0 && !before.ends_with(':') && !after_number) { declared.insert(w); }
                continue;
            }
            i += 1;
        }
    };
    if let Some(d) = def_at { words(0, d, true, &mut declared); }
    // what's solved for
    let mut k = 0;
    let joined: String = cs.iter().collect();
    while let Some(p) = joined[k..].find("solve(") {
        let start = k + p + "solve(".len();
        let end = joined[start..].find(')').map_or(joined.len(), |e| start + e);
        for n in joined[start..end].split(',') { declared.insert(n.trim().to_string()); }
        k = end;
    }
    let is_declared = |w: &str| declared.contains(w) || env.vars.contains_key(w) || env.funs.contains_key(w);

    let mut out = String::new();
    let mut brackets: Vec<bool> = vec![];
    let mut run = false;
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        let in_clause = i >= clause_at;
        let before = out.trim_end().to_string();
        let after_number = before.chars().last().is_some_and(|c| c.is_ascii_digit());
        match c {
            '"' => {
                out.push(c); i += 1;
                while i < cs.len() {
                    let d = cs[i]; out.push(d); i += 1;
                    if d == '\\' && i < cs.len() { out.push(cs[i]); i += 1; continue; }
                    if d == '"' { break; }
                }
                run = false;
                continue;
            }
            '⟦' | '⦇' | '⦅' => {
                let close = match c { '⟦' => '⟧', '⦇' => '⦈', _ => '⦆' };
                while i < cs.len() { let d = cs[i]; out.push(d); i += 1; if d == close { break; } }
                run = c == '⟦';
                continue;
            }
            '#' if cs.get(i + 1).is_some_and(|c| c.is_alphabetic() || *c == '°' || *c == '%') => {
                let start = i + 1;
                i = start + 1;
                while i < cs.len() && (word_char(cs[i]) || (cs[start] == '°' && cs[i] == 'C')) { i += 1; }
                let w: String = cs[start..i].iter().collect();
                let m = made(&w).unwrap_or_else(|| panic!("#{w}: no unit or constant"));
                out += &m.token;
                run = is_unit(&w);
                continue;
            }
            '°' => {
                let name = if cs.get(i + 1) == Some(&'C') { i += 2; "°C" } else { i += 1; "°" };
                out += &unit(name);
                run = true;
                continue;
            }
            '%' | '‰' => {
                out += &unit(&c.to_string());
                i += 1;
                run = true;
                continue;
            }
            '[' => {
                let index = i > 0 && (word_char(cs[i - 1]) || matches!(cs[i - 1], ')' | ']' | '"'));
                brackets.push(!index);
                if index { out.push('['); } else if !in_clause { out.push('('); }
                i += 1;
                run = false;
                continue;
            }
            ']' => {
                match brackets.pop() {
                    Some(false) | None => out.push(']'),
                    Some(true) => if !in_clause { out.push(')') },
                }
                i += 1;
                continue;
            }
            c if c.is_alphabetic() => {
                let start = i;
                while i < cs.len() && word_char(cs[i]) { i += 1; }
                let w: String = cs[start..i].iter().collect();
                let next = cs[i..].iter().find(|c| !c.is_whitespace()).copied();
                let in_bracket = brackets.last() == Some(&true);
                // a keyword of the clause, with its arguments
                if in_clause && KEYWORDS.contains(&w.as_str()) && !(w == "frac" && next == Some('(')) && !is_declared(&w) {
                    let mut kw = w.clone();
                    let mut j = i;
                    while j < cs.len() && cs[j].is_whitespace() { j += 1; }
                    if cs.get(j) == Some(&'(') {
                        let mut d = 0;
                        while j < cs.len() {
                            kw.push(cs[j]);
                            if cs[j] == '(' { d += 1; }
                            if cs[j] == ')' { d -= 1; if d == 0 { j += 1; break; } }
                            j += 1;
                        }
                        i = j;
                    }
                    out += &format!("⦅{kw}⦆");
                    run = false;
                    continue;
                }
                let unit_here = is_unit(&w) && (in_bracket || (!is_declared(&w) && (run || after_number || (before.ends_with(':') && !before.ends_with(":=")) || (in_clause && w.chars().count() > 1))));
                if unit_here {
                    out += &unit(&w);
                    run = true;
                    continue;
                }
                run = false;
                if is_declared(&w) {
                    out += &w;
                } else if matches!(w.as_str(), "e" | "pi" | "π") {
                    out += &constant(&w);
                } else if solveng_names::OPERATIONS.contains(&w.as_str()) && !OPERATIONS_SKIPPED.contains(&w.as_str()) {
                    out += &format!("⦇{w}⦈");
                } else {
                    out += &w;
                }
                continue;
            }
            '^' if run => {
                // an exponent of a unit: ^2, ^(-1)
                out.push(c);
                i += 1;
                let paren = cs.get(i) == Some(&'(');
                while i < cs.len() && (cs[i].is_ascii_digit() || cs[i] == '-' || cs[i] == '−' || (paren && (cs[i] == '(' || cs[i] == ')'))) {
                    let d = cs[i];
                    out.push(d);
                    i += 1;
                    if d == ')' { break; }
                }
                continue;
            }
            c if c.is_whitespace() || matches!(c, '/' | '*' | '·') => {}
            _ => run = false,
        }
        out.push(c);
        i += 1;
    }
    out
}

// ---------- results ----------

/// Result markup and messages with units by name: `#svref("sv…", "mA")` → `upright("mA")`, `#svop("sin") (x)` →
/// `sin(x)`, a binding's id → its name.
pub fn readable(markup: &str) -> String {
    let mut s = markup.to_string();
    while let Some(p) = s.find("#svref(\"") {
        let rest = &s[p + "#svref(\"".len()..];
        let Some(q) = rest.find("\", \"") else { break };
        let after = &rest[q + 4..];
        let Some(end) = after.find("\")") else { break };
        let name = after[..end].to_string();
        let tail = after[end + 2..].to_string();
        s = format!("{}upright(\"{name}\"){tail}", &s[..p]);
    }
    while let Some(p) = s.find("#svop(\"") {
        let rest = &s[p + "#svop(\"".len()..];
        let Some(q) = rest.find("\")") else { break };
        let name = rest[..q].to_string();
        let tail = rest[q + 2..].strip_prefix(' ').unwrap_or(&rest[q + 2..]).to_string();
        s = format!("{}{name}{tail}", &s[..p]);
    }
    let ids = ids().lock().unwrap();
    let mut out = String::new();
    let mut rest = s.as_str();
    while let Some(p) = rest.find("sv") {
        let id = rest[p..].get(..66).filter(|id| id[2..].chars().all(|c| ('a'..='p').contains(&c)));
        match id.and_then(|id| ids.get(id)) {
            Some(name) => { out += &rest[..p]; out += name; rest = &rest[p + 66..]; }
            None => { out += &rest[..p + 2]; rest = &rest[p + 2..]; }
        }
    }
    out + rest
}

fn readable_out(mut out: Out) -> Out {
    out.result = out.result.map(|r| readable(&r));
    out.errors = out.errors.iter().map(|e| readable(e)).collect();
    out
}

/// A line as written (already explicit), in math, with the harness's configuration.
pub fn line_raw_cfg(env: &mut Env, source: &str, cfg: &Cfg) -> Out {
    readable_out(api::line(env, &LineReq { src: source.into(), cfg: cfg.clone(), listed: true }))
}
pub fn line_raw(env: &mut Env, source: &str) -> Out {
    line_raw_cfg(env, source, &cfg())
}

/// A readable line, made explicit; its output readable.
pub fn line_cfg(env: &mut Env, text: &str, cfg: &Cfg) -> Out {
    let source = src(text, env);
    line_raw_cfg(env, &source, cfg)
}
pub fn line(env: &mut Env, text: &str) -> Out {
    line_cfg(env, text, &cfg())
}

/// A line that must work.
pub fn run(env: &mut Env, text: &str) -> Out {
    let out = line(env, text);
    assert!(out.errors.is_empty(), "{text}: {:?}", out.errors);
    out
}

/// A line that must fail: its messages.
pub fn fails(env: &mut Env, text: &str) -> String {
    let out = line(env, text);
    assert!(!out.errors.is_empty(), "{text} should fail: {out:?}");
    out.errors.join("; ")
}

/// The result of a line that must work.
pub fn result(env: &mut Env, text: &str) -> String {
    run(env, text).result.unwrap_or_default()
}

/// A request for a single line, made explicit (for show_value).
pub fn req(text: &str, env: &Env) -> LineReq {
    LineReq { src: src(text, env), cfg: cfg(), listed: true }
}

/// A graph curve, made explicit.
pub fn curve(text: &str, env: &Env) -> solveng_core::graph::Src {
    solveng_core::graph::Src { src: src(text, env), label: None }
}
