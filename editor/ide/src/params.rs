//! solveng's public functions and their parameters, read from the package's own sources and doc comments, so
//! completions stay in step with the package.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Serialize;

const SOURCES: &[&str] = &[
    include_str!("../../../src/api.typ"),
    include_str!("../../../src/core.typ"),
    include_str!("../../../src/defined.typ"),
    include_str!("../../../src/graph.typ"),
    include_str!("../../../src/lambdify.typ"),
    include_str!("../../../src/value.typ"),
    include_str!("../../../src/modifiers.typ"),
    include_str!("../../../src/unit-words.typ"),
    include_str!("../../../src/operation.typ"),
];
const CONFIG: &str = include_str!("../../../src/config.typ");

/// A function of the package.
#[derive(Clone, Debug, Serialize)]
pub struct Function {
    pub name: String,
    /// The first paragraph of its docs.
    pub doc: String,
    /// Positional parameters, `..eqs` without the dots.
    pub positional: Vec<Param>,
    pub named: Vec<Param>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Param {
    pub name: String,
    /// `auto, str`, as the docs list them.
    pub types: String,
    pub doc: String,
    /// The default in the signature, or the option's default for `auto`.
    pub default: String,
    /// The strings it takes, for options like `steps`.
    pub values: Vec<String>,
    pub variadic: bool,
}

/// Splits `a, b: (1, 2), c` at the top level.
fn split_top(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let (mut depth, mut start, mut in_str) = (0i32, 0, false);
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' if in_str => i += 1,
            b'"' => in_str = !in_str,
            b'(' | b'[' | b'{' if !in_str => depth += 1,
            b')' | b']' | b'}' if !in_str => depth -= 1,
            b',' if !in_str && depth == 0 => {
                parts.push(s[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(s[start..].trim().to_string());
    parts.retain(|p| !p.is_empty());
    parts
}

/// The text inside the parenthesis opening at `open`.
fn inside(s: &str, open: usize) -> Option<&str> {
    let b = s.as_bytes();
    let (mut depth, mut in_str) = (0, false);
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'\\' if in_str => i += 1,
            b'"' => in_str = !in_str,
            b'(' if !in_str => depth += 1,
            b')' if !in_str => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[open + 1..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// `key: value` pairs of a dictionary literal like `#let _DEF = (…)`.
fn dictionary(source: &str, name: &str) -> Vec<(String, String)> {
    let Some(at) = source.find(&format!("#let {name} = (")) else { return Vec::new() };
    let open = at + name.len() + 8;
    let body: String = inside(source, open)
        .unwrap_or("")
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    split_top(&body)
        .into_iter()
        .filter_map(|e| e.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
        .collect()
}

struct Docs {
    summary: String,
    params: HashMap<String, (String, String)>,
}

fn docs(lines: &[&str]) -> Docs {
    let mut summary = Vec::new();
    let mut params = HashMap::new();
    let mut in_summary = true;
    for line in lines {
        let l = line.trim_start().trim_start_matches("///").trim();
        if let Some(rest) = l.strip_prefix("- ") {
            in_summary = false;
            let Some((head, doc)) = rest.split_once("):") else { continue };
            let Some((names, types)) = head.split_once(" (") else { continue };
            // `digits, separator, form, … (auto)`: worksheet's options, documented there.
            if names.contains('…') {
                continue;
            }
            params.insert(names.trim().trim_start_matches("..").to_string(), (types.trim().to_string(), doc.trim().to_string()));
        } else if l.is_empty() || l.starts_with("```") {
            if !summary.is_empty() {
                in_summary = false;
            }
        } else if in_summary {
            summary.push(l.to_string());
        }
    }
    Docs { summary: summary.join(" "), params }
}

fn read() -> Vec<Function> {
    let defaults: HashMap<String, String> = dictionary(CONFIG, "_DEF").into_iter().collect();
    let choices: HashMap<String, Vec<String>> = dictionary(CONFIG, "_CH")
        .into_iter()
        .map(|(k, v)| {
            let inner = v.trim_start_matches('(').trim_end_matches(')').to_string();
            (k, split_top(&inner).into_iter().filter(|s| s.starts_with('"')).map(|s| s.trim_matches('"').to_string()).collect())
        })
        .collect();
    let mut functions = Vec::new();
    for source in SOURCES {
        let lines: Vec<&str> = source.lines().collect();
        let mut offset = 0;
        let mut doc_start = 0;
        for (n, line) in lines.iter().enumerate() {
            let here = offset;
            offset += line.len() + 1;
            if !line.trim_start().starts_with("///") && !line.starts_with("#let ") {
                doc_start = n + 1;
                continue;
            }
            let Some(rest) = line.strip_prefix("#let ") else { continue };
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
            if name.starts_with('_') || !rest[name.len()..].starts_with('(') || doc_start >= n {
                doc_start = n + 1;
                continue;
            }
            let Some(signature) = inside(source, here + 5 + name.len()) else { continue };
            let d = docs(&lines[doc_start..n]);
            doc_start = n + 1;
            let mut f = Function { name: name.clone(), doc: d.summary.clone(), positional: Vec::new(), named: Vec::new() };
            for part in split_top(signature) {
                let (pname, default) = match part.split_once(':') {
                    Some((k, v)) => (k.trim().to_string(), Some(v.trim().to_string())),
                    None => (part.trim().to_string(), None),
                };
                let variadic = pname.starts_with("..");
                let pname = pname.trim_start_matches("..").to_string();
                let (types, doc) = d.params.get(&pname).cloned().unwrap_or_default();
                let option = match pname.as_str() {
                    "style" if name == "defined" => "defined-style",
                    p => p,
                };
                let default = match default {
                    Some(v) if v == "auto" => defaults.get(option).cloned().map_or(v, |o| format!("auto ({o})")),
                    Some(v) => v,
                    None => String::new(),
                };
                let mut values = choices.get(option).cloned().unwrap_or_default();
                if name == "defined" && (pname == "included" || pname == "excluded") {
                    values = ["all", "given", "computed", "function"].map(String::from).to_vec();
                }
                let p = Param { name: pname, types, doc, default, values, variadic };
                if part.contains(':') && !variadic { f.named.push(p) } else { f.positional.push(p) }
            }
            functions.push(f);
        }
    }
    // Options are documented at worksheet; defined and graph take the same ones.
    let config = functions.iter().find(|f| f.name == "worksheet").map(|f| f.named.clone()).unwrap_or_default();
    for f in &mut functions {
        for p in &mut f.named {
            if p.doc.is_empty()
                && let Some(o) = config.iter().find(|o| o.name == p.name) {
                    p.doc = o.doc.clone();
                    p.types = o.types.clone();
                }
        }
    }
    functions
}

/// Calculation operations are math content, not Typst function calls. Keep their
/// docs separate from `functions()` so call discovery does not interpret them as
/// package functions or their string arguments as worksheet code.
fn read_builtins() -> Vec<Function> {
    let mut found = Vec::new();
    let mut comments = Vec::new();
    for line in include_str!("../../../src/builtins.typ").lines() {
        if line.starts_with("///") {
            comments.push(line);
            continue;
        }
        let name = line.strip_prefix("#let ").and_then(|s| s.split_once(" = ").map(|(name, _)| name))
            .or_else(|| line.strip_prefix("// @builtin "));
        if let Some(name) = name {
            let d = docs(&comments);
            let positional = comments.iter().filter_map(|line| {
                let rest = line.trim_start_matches("///").trim().strip_prefix("- ")?;
                let (head, _) = rest.split_once("):")?;
                let (name, _) = head.split_once(" (")?;
                let key = name.trim_start_matches("..");
                let (types, doc) = d.params.get(key)?;
                Some(Param {
                    name: key.into(), types: types.clone(), doc: doc.clone(), default: String::new(),
                    values: Vec::new(), variadic: name.starts_with(".."),
                })
            }).collect();
            found.push(Function { name: name.into(), doc: comments.iter().map(|s| s.trim_start_matches("///").trim()).take_while(|s| !s.starts_with("- ") && !s.starts_with("-> ")).collect::<Vec<_>>().join("\n").trim().into(), positional, named: Vec::new() });
        }
        comments.clear();
    }
    found
}

static BUILTINS: LazyLock<Vec<Function>> = LazyLock::new(read_builtins);

/// Whether solveng's `functions` module exports the operation `name`, which math needs imported; Typst's own math
/// operations (`sin`, `sqrt`) need no import.
pub(crate) fn automatic_math(name: &str) -> bool {
    include_str!("../../../src/builtins.typ").lines().filter_map(|line| line.strip_prefix("#let "))
        .any(|line| line.split_once(" = ").is_some_and(|(binding, _)| binding == name))
}

/// The names solveng's `functions` module holds, for `#import functions: *`.
pub(crate) fn automatic_names() -> Vec<String> {
    include_str!("../../../src/builtins.typ").lines().filter_map(|line| line.strip_prefix("#let "))
        .filter_map(|line| line.split_once(" = ").map(|(binding, _)| binding.trim().to_string()))
        .filter(|name| !name.starts_with('_')).collect()
}

/// Documentation of a calculation operation, including its alternative spellings.
pub(crate) fn builtin(name: &str) -> Option<&'static Function> {
    let name = match name { "arcsin" => "asin", "arccos" => "acos", "arctan" => "atan", name => name };
    BUILTINS.iter().find(|f| f.name == name)
}

static FUNCTIONS: LazyLock<Vec<Function>> = LazyLock::new(read);

/// The package's public functions with their parameters.
pub fn functions() -> &'static [Function] {
    &FUNCTIONS
}

pub(crate) fn function(name: &str) -> Option<&'static Function> {
    FUNCTIONS.iter().find(|f| f.name == name)
}
