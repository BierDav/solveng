//! Completions: names, functions, constants and units in solveng code, math and call strings; the options of
//! solveng's functions; language tags after ```` ``` ````.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::calls::{Call, Role};
use crate::elements::Element;
use crate::text::{Doc, after, before, chars, is_digit, last_index_of_char};
use crate::tokens::CONSTANTS;
use crate::units::{self, Custom};
use crate::{Analysis, Completions, Context, Item, ItemKind, TAG};

pub(crate) const FUNCTIONS: &[&str] = &["sqrt", "exp", "ln", "log", "sin", "cos", "tan", "asin", "acos", "atan", "abs", "diff", "integrate", "vec", "at", "rect", "tri", "sgn", "cases"];
const NATIVE_FUNCTIONS: &[&str] = &["sqrt", "exp", "ln", "log", "sin", "cos", "tan", "abs", "cases"];
/// Fence languages Typewright highlights, offered after ```` ``` ````.
const TAGS: &[&str] =
    &["sql", "bash", "javascript", "typescript", "kotlin", "java", "python", "csharp", "markdown", "typst", "typc", "mermaid"];
/// Keywords `mark:` takes besides values.
const MARKS: &[(&str, &str)] =
    &[("crossings", "where the curves cross"), ("roots", "zeros of the curves"), ("extrema", "minima and maxima")];

#[derive(Clone, Debug)]
pub(crate) struct Definition {
    pub name: String,
    pub detail: String,
    pub function: bool,
    /// Where the name is, in characters of its text; set by [`definitions`].
    pub start: usize,
    pub end: usize,
    /// How many characters the name takes as written (`Delta""r` for `Δr`).
    pub written: usize,
    /// Where its statement ends, from where the name holds the new value.
    pub statement_end: usize,
    /// The unit a variable is declared with: `A` in `V: A = …`.
    pub unit: Option<String>,
    /// Written with `:=`, which assigns; a name solved for (`-> solve(x)`) is defined without.
    pub assign: bool,
}

impl Definition {
    /// The name without a function's parameters.
    pub fn base(&self) -> &str {
        before(&self.name, "(")
    }
}

static NAME_AT_START: LazyLock<Regex> = LazyLock::new(|| {
    // name, parameters, a variable's unit (V: A = …), relation, the rest
    Regex::new(r#"^\s*([\p{L}](?:[\p{L}\p{N}_]|""\p{L})*)\s*(\([^()=]*\))?(?:\s*:\s*([^-+−=<>():]+?))?\s*(:=|≔|<=|>=|=|<|>|≤|≥)(.*)$"#).unwrap()
});
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

/// `I := 30 mA` or `f(x) := x^2` → a definition; anything else (`Q0/2 = I t`, `x =`) → None.
pub(crate) fn definition(line: &str) -> Option<Definition> {
    let raw: Vec<char> = line.chars().collect();
    let (decoded, offsets) = crate::text::unescape(&raw);
    let start = decoded.iter().position(|c| !c.is_whitespace())?;
    let name = solveng_names::scan(&decoded, start).ok()??;
    let written = offsets[name.end] - offsets[start];
    let tail: String = decoded[name.end..].iter().collect();
    let matched = format!("x{tail}");
    let m = NAME_AT_START.captures(&matched)?;
    let normalized = name.value;
    let (name, params, relation, rest) = (normalized.as_str(), m.get(2).map_or("", |p| p.as_str()), &m[4], &m[5]);
    let unit = m.get(3).map(|u| u.as_str().trim().trim_matches(|c| c == '"' || c == '[' || c == ']').trim().to_string()).filter(|u| !u.is_empty());
    // only := defines; = states, which is checked
    if relation != ":=" && relation != "≔" {
        return None;
    }
    let assign = true;
    // what's after the = or -> showing a result is its clause: a unit and keywords
    let rc = chars(rest);
    let rest = match crate::clause::marker(&rc, &|_| false) {
        Some((s, _, _)) => crate::text::sub(&rc, 0, s),
        None => rest.to_string(),
    };
    // a variable has one value: a chain V := 7.2 kΩ = 7200 Ω gives it its last step; a function keeps its body, an
    // equation too (U(x, y) := x = y)
    let value = if params.is_empty() { rest.rsplit('=').next().unwrap_or("").trim() } else { rest.trim() };
    let detail = format!("= {value}");
    // Variables go by their stored name (Delta_R is Δ_R); functions keep theirs.
    let resolved = crate::resolved_name(name);
    let name = resolved.as_str();
    Some(if params.is_empty() {
        Definition { name: name.into(), detail, function: false, start: 0, end: 0, written, statement_end: 0, unit, assign }
    } else {
        Definition { name: format!("{name}{params}"), detail, function: true, start: 0, end: 0, written, statement_end: 0, unit: None, assign }
    })
}

/// The names a statement solves for (`x^2 = 4 -> solve(x)`): definitions at the names in the clause, which
/// starts at `clause` in `doc`; `equation` is what's solved.
fn solved_definitions(doc: &Doc, clause: usize, end: usize, equation: &str, found: &mut Vec<Definition>) {
    if equation.contains(":=") || equation.contains('≔') { return; }
    let tail = doc.sub(clause, end);
    let names = crate::clause::solved(&tail);
    let mut from = 0;
    for name in names {
        let Some(at) = tail[from..].find(&name).map(|i| from + i) else { continue };
        from = at + name.len();
        let start = clause + tail[..at].chars().count();
        let written = name.chars().count();
        let detail = format!("from {}", equation.trim());
        let key = crate::navigate::math_name_of(&chars(&name)).map_or_else(|| crate::resolved_name(&name), |n| n.0);
        found.push(Definition { name: key, detail, function: false, start, end: start + written, written, statement_end: end, unit: None, assign: false });
    }
}

/// The definitions of solveng code in `start..end` of `doc`, line by line and `;` by `;`, comments left out.
fn code_definitions(doc: &Doc, start: usize, end: usize, found: &mut Vec<Definition>) {
    let c = &doc.c;
    let (decoded, offsets) = crate::text::unescape(&c[start..end]);
    for (a, b) in crate::text::equation_ranges(&decoded, true) {
        let (i, line_end) = (start + offsets[a], start + offsets[b]);
        let text_end = i + before(&doc.sub(i, line_end), "//").chars().count();
        // -> solve(x) defines x; the rest is what's solved
        let marker = crate::clause::marker(&c[i..text_end], &|_| false).map(|(s, e, _)| (i + s, i + e));
        let head_end = marker.map_or(text_end, |m| m.0);
        if let Some((_, e)) = marker {
            solved_definitions(doc, e, text_end, &doc.sub(i, head_end), found);
        }
        let line = doc.sub(i, head_end);
        let mut at = i;
        for piece in line.split(';') {
            if let Some(mut d) = definition(piece) {
                d.start = at + piece.chars().take_while(|ch| ch.is_whitespace()).count();
                d.end = d.start + d.written;
                d.statement_end = if marker.is_some() { text_end } else { at + piece.chars().count() };
                found.push(d);
            }
            at += piece.chars().count() + 1;
        }
    }
}

/// Names defined in a text, in order: math equations and solveng code that `active` says are calculated, and
/// `worksheet` strings.
pub(crate) fn definitions(doc: &Doc, elements: &[Element], calls: &[Call], active: &dyn Fn(&Element) -> bool) -> Vec<Definition> {
    let mut found = Vec::new();
    let equations: Vec<Element> = elements.iter().filter(|e| active(e)).flat_map(|e| {
        if !e.math { return vec![e.clone()]; }
        crate::text::equation_ranges(&doc.c[e.body_start..e.body_end], false).into_iter().map(|(a, b)| {
            let mut part = e.clone();
            part.body_start += a;
            part.body_end = e.body_start + b;
            part
        }).collect()
    }).collect();
    for e in &equations {
        if !e.math {
            code_definitions(doc, e.body_start, e.body_end, &mut found);
            continue;
        }
        // Math writes Q_0, t_"half", "tews" and units as names: Q_0 := 3450 mAh.
        let marker = crate::clause::marker(&doc.c[e.body_start..e.body_end], &|_| false);
        if let Some((s, m, _)) = marker {
            solved_definitions(doc, e.body_start + m, e.body_end, &doc.sub(e.body_start, e.body_start + s), &mut found);
        }
        let c = &doc.c[e.body_start..marker.map_or(e.body_end, |m| e.body_start + m.0)];
        let relation = c.iter().position(|ch| "=<>≤≥≔".contains(*ch)).unwrap_or(c.len());
        // := assigns: the colon belongs to the relation
        let assign = c.get(relation) == Some(&'≔') || (relation > 0 && c.get(relation) == Some(&'=') && c[relation - 1] == ':');
        let left = if assign && c.get(relation) == Some(&'=') { relation - 1 } else { relation };
        let lead = c.iter().take_while(|ch| ch.is_whitespace()).count();
        let end = (lead..left).rev().find(|&k| !c[k].is_whitespace()).map_or(lead, |k| k + 1);
        // Quotes go, except empty ones gluing a name (Delta""r_R).
        let unquote = |t: String| t.replace("\"\"", "\u{0}").replace('"', "").replace('\u{0}', "\"\"");
        let plain = unquote(crate::text::sub(c, relation, c.len()));
        let plain = if assign { format!(":={}", plain.trim_start_matches(['=', '≔'])) } else { plain };
        // A function: q(x) = x "ü"; its name is what's defined, its parameters are its own.
        let lhs = crate::text::sub(c, lead, end);
        let lhs_chars = chars(&lhs);
        let function_name = solveng_names::scan(&lhs_chars, 0).ok().flatten().filter(|name| lhs_chars.get(name.end.wrapping_sub(1)) != Some(&'_') && lhs_chars[name.end..].iter().find(|c| !c.is_whitespace()) == Some(&'('));
        if let Some(name) = function_name {
            if let Some(mut d) = definition(&SPACES.replace_all(&format!("{lhs} {plain}"), " ")) {
                d.start = e.body_start + lead;
                d.end = d.start + name.end;
                d.written = d.end - d.start;
                d.statement_end = e.body_end;
                found.push(d);
            }
            continue;
        }
        // Else the left side is one name, maybe with its unit: V: A, I: "mA".
        let colon = (lead..end).find(|&k| c[k] == ':');
        let unit = colon.map(|k| crate::text::sub(c, k + 1, end).trim().trim_matches(|ch| ch == '"' || ch == '[' || ch == ']').trim().to_string());
        let end = match colon {
            Some(k) => (lead..k).rev().find(|&i| !c[i].is_whitespace()).map_or(lead, |i| i + 1),
            None => end,
        };
        let Some(key) = crate::navigate::math_name_of(&c[..colon.unwrap_or(left)]).filter(|n| n.1 == lead && n.2 == end).map(|n| n.0) else {
            continue;
        };
        if let Some(mut d) = definition(&SPACES.replace_all(&format!("x {plain}"), " ")) {
            d.name = key;
            d.unit = unit.filter(|u| !u.is_empty());
            d.start = e.body_start + lead;
            d.end = e.body_start + end;
            d.written = end - lead;
            d.statement_end = e.body_end;
            found.push(d);
        }
    }
    for call in calls.iter().filter(|c| c.function.name == "worksheet") {
        for t in call.texts.iter().filter(|t| t.param.is_none() && t.role == Role::Code) {
            code_definitions(doc, t.start, t.end, &mut found);
        }
    }
    found.sort_by_key(|d| d.start);
    found
}

/// A solveng name as math writes it: `t_half` → `t_"half"`, `Δr` → `Δ""r`, `tews` → `"tews"`; `Q_0`, `rho` and
/// `Q0` stay.
pub(crate) fn math_name(name: &str) -> String { solveng_names::math(&crate::resolved_name(name)) }

static ARROW_UNIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"->\s*[^=;]*$").unwrap());
static VALUE_UNITS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[^\p{L}\p{N}_])\d[\d.,]*(?:[eE][-+]?\d+)?\s*((?:[\p{L}°%‰µΩ]+(?:\^-?\d+|[²³])?\s*[/*·]?\s*)*)$").unwrap()
});
static UNIT_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}°%‰µΩ]+").unwrap());

/// After `name:`, a variable's or a parameter's unit: `V: |`, `f(x: m|`, `f(x, y: kg m/|`.
static DECLARED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\p{L}[\p{L}\p{N}_]*\s*:[^=<>]*$").unwrap());

/// After `as`, the unit a cast takes: `U as |`, in math `U "as" |`.
static CAST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?:^|[\s(])(?:as|"as")\s+$"#).unwrap());

/// After a term that isn't a number (after one, `as` is the unit attosecond): `U |`, `f(x) |`, `[R I] |`.
static TERM_END: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:\p{L}[\p{L}\p{N}_]*|[)\]])\s+$").unwrap());

/// The keywords that fit before the word being typed: `as` after a term, `any` and `1` where a type goes; `math`
/// quotes the words, as math needs them.
fn keyword_items(line: &str, math: bool) -> Vec<Item> {
    let word = |w: &str, detail: &str| {
        let mut item = Item::new(w, ItemKind::KEYWORD, detail, &if math && w != "1" { format!("\"{w}\"") } else { w.to_string() });
        if math && w != "1" {
            item.filter = Some(w.to_string());
        }
        item
    };
    if declared_unit(line) {
        let mut items = vec![word("1", "type: a plain number")];
        if !CAST.is_match(line) {
            items.push(word("any", "type: every unit, the result's follows it"));
        }
        return items;
    }
    let last = line.trim_end().rsplit(|c: char| !(c.is_alphanumeric() || c == '_')).next().unwrap_or("");
    if TERM_END.is_match(line) && !CAST.is_match(line) && !last.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return vec![word("as", "cast: the number of what's before in SI units as a value in a unit, U as A")];
    }
    Vec::new()
}

/// After `:` in a parameter list, where a type goes (`f(x: |`, `f(x, y: |`), or after `as`. Variables and what a
/// function returns take no type.
fn declared_unit(line: &str) -> bool {
    let Some(open) = line.rfind('(') else { return CAST.is_match(line) };
    let list = &line[open..];
    let segment = list.rsplit(['(', ',']).next().unwrap_or(list);
    (!list.contains(')') && DECLARED.is_match(segment)) || CAST.is_match(line)
}

/// Before the word being typed: a value's unit (`30 m|`, `[5 |`, `-> |`, `kg m/|`), or a declared one (`V: |`).
fn unit_context(line: &str, custom: &[Custom]) -> bool {
    if ARROW_UNIT.is_match(line) || declared_unit(line) {
        return true;
    }
    if line.rfind('[') > line.rfind(']') {
        return true;
    }
    // A number, then units: 9.81 m/s^, 3 kg m; not 2 I, where I is a variable.
    let Some(m) = VALUE_UNITS.captures(line) else { return false };
    UNIT_WORD.find_iter(&m[1]).all(|w| units::is_unit(w.as_str()) || custom.iter().any(|u| u.name == w.as_str()))
}

/// The defined names, newest first and each once; `math` spells them for equations. The compiler's values, where
/// known, follow what's written.
fn definition_items(a: &Analysis, seen: &mut HashSet<String>, math: bool, functions: bool) -> Vec<Item> {
    // (the file's path, None for the document; the definition)
    let mut defs = Vec::new();
    for part in a.around() {
        let path = part.path.clone().filter(|p| *p != a.context.path);
        defs.extend(definitions(&part.doc, &part.elements, &part.calls, &|e| part.active(e)).into_iter().map(|d| (path.clone(), d)));
    }
    // = on a name already defined in its scope states something about it and defines nothing
    let statements = crate::navigate::statements(a);
    defs.extend(definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e)).into_iter().filter(|d| !statements.contains(&d.start)).map(|d| (None, d)));
    let compiled = |name: &str| a.context.defined.iter().find(|d| d.name == name && d.kind != "function" && !d.value.is_empty());
    // Functions as they resolve: f(x, ..) shows what .. takes.
    let signatures = if functions && defs.iter().any(|(_, d)| d.function) { crate::navigate::signatures(a) } else { Vec::new() };
    let mut items = Vec::new();
    for (path, d) in defs.iter().rev() {
        let name = before(&d.name, "(");
        if !seen.insert(name.to_string()) || (d.function && !functions) {
            continue;
        }
        let spelled = if math { math_name(name) } else { solveng_names::code(name) };
        if d.function {
            // As compiled where known: with the types its parameters were given or took and the type it returns.
            let reported = a.context.defined.iter().filter(|r| r.name == name && r.kind == "function").find_map(|r| signature(&r.value));
            let resolved = signatures.iter().find(|(p, start, _)| p == path && *start == d.start).map(|(_, _, ps)| format!("({ps})"));
            let label = reported.or(resolved).unwrap_or_else(|| format!("({})", before(after(&d.name, "(", ""), ")")));
            let caret = if label.starts_with("()") { "" } else { "$0" };
            items.push(Item::new(&format!("{name}{label}"), ItemKind::FUNCTION, &d.detail, &format!("{spelled}({caret})")));
        } else {
            let mut detail = d.detail.clone();
            if let Some(v) = compiled(name)
                && !detail.ends_with(&format!(" {}", v.value)) {
                    detail = format!("{detail} = {}", v.value);
                }
            // Its unit: as declared (V: A = …), else as its value has it, written or computed.
            let unit = d.unit.clone().or_else(|| value_unit(&d.detail, &a.units)).or_else(|| compiled(name).and_then(|v| value_unit(&format!("= {}", v.value), &a.units)));
            items.push(typed(variable_item(name, &detail, &spelled), unit));
        }
    }
    // Defined where the text doesn't say, e.g. in a package or by a call with a computed string.
    for d in &a.context.defined {
        if d.name.is_empty() || !seen.insert(d.name.clone()) {
            continue;
        }
        let spelled = if math { math_name(&d.name) } else { solveng_names::code(&d.name) };
        match signature(&d.value).zip(d.value.split_once(" = ")) {
            Some((params, (_, body))) if d.kind == "function" => {
                if functions {
                    let label = format!("{}{params}", d.name);
                    items.push(Item::new(&label, ItemKind::FUNCTION, &format!("= {body}"), &format!("{spelled}($0)")));
                }
            }
            _ => items.push(typed(variable_item(&d.name, &format!("= {}", d.value), &spelled), value_unit(&format!("= {}", d.value), &a.units))),
        }
    }
    items
}

/// The signature in a reported function's value, `(x: A, y: any): A^2` of `(x: A, y: any): A^2 = x*y`.
fn signature(value: &str) -> Option<String> {
    value.split_once(" = ").map(|(sig, _)| sig.to_string()).filter(|s| s.starts_with('('))
}

/// A variable, inserted as stored (`Δ_R`) but also found by its spelling (`Delta_R`).
static VALUE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^=\s*\[?\s*[-+−]?(?:\d[\d.,]*|\.\d+)(?:[eE][-+−]?\d+)?\s*([^\]]*?)\s*\]?$").unwrap());

/// The unit of a variable's value as a detail shows it: `= 4 A` → `A`, `= 0.004 A → mA` → `mA`; None for a plain
/// number or a formula.
fn value_unit(detail: &str, custom: &[Custom]) -> Option<String> {
    let (value, arrow) = match detail.split_once(" → ") {
        Some((v, a)) => (v, Some(a.trim())),
        None => (detail, None),
    };
    if let Some(a) = arrow.filter(|a| !a.is_empty()) {
        return Some(a.to_string());
    }
    let u = VALUE.captures(value.trim())?.get(1)?.as_str().trim().to_string();
    let words: Vec<&str> = UNIT_WORD.find_iter(&u).map(|w| w.as_str()).collect();
    (!words.is_empty() && words.iter().all(|w| units::known(w, custom))).then_some(u)
}

/// A variable's item labelled with its unit, `V: A`; typing still matches the name.
fn typed(mut item: Item, unit: Option<String>) -> Item {
    if let Some(u) = unit {
        item.filter = Some(item.filter.take().unwrap_or_else(|| item.label.clone()));
        item.label = format!("{}: {u}", item.label);
    }
    item
}

fn variable_item(name: &str, detail: &str, insert: &str) -> Item {
    let mut item = Item::new(name, ItemKind::VARIABLE, detail, insert);
    let spelled = crate::conceal::spelled(name);
    if spelled != name {
        item.filter = Some(spelled);
    }
    item
}

/// In the clause after the `=` or `->` showing a result: its keywords and units, or names inside `solve(…)`;
/// `math` spells them for equations.
fn clause_items(a: &Analysis, names: bool, math: bool) -> Vec<Item> {
    if names {
        return definition_items(a, &mut HashSet::new(), math, false);
    }
    let mut items: Vec<Item> = crate::clause::KEYWORDS
        .iter()
        .map(|(k, d)| Item::new(k, ItemKind::KEYWORD, d, &if *k == "solve" || *k == "digits" { format!("{k}($0)") } else { k.to_string() }))
        .collect();
    items.extend(units::unit_items(&a.units).into_iter().map(|mut u| {
        // math writes units of its own as unit("…")
        if math && a.units.iter().any(|c| c.name == u.insert) {
            u.filter.get_or_insert_with(|| u.insert.clone());
            u.insert = format!("unit(\"{}\")", u.insert);
        }
        u
    }));
    items
}

/// What solveng code completes at a word: names, solveng's functions and constants, or units after a value.
fn code_items(a: &Analysis, line: &str) -> Vec<Item> {
    // Comments complete nothing.
    if line.contains("//") {
        return Vec::new();
    }
    if let Some(names) = crate::clause::at_clause(line) {
        return clause_items(a, names, false);
    }
    if unit_context(line, &a.units) {
        return keyword_items(line, false).into_iter().chain(units::unit_items(&a.units)).collect();
    }
    let mut seen = HashSet::new();
    let mut items = definition_items(a, &mut seen, false, true);
    items.extend(keyword_items(line, false));
    for f in FUNCTIONS {
        if seen.insert(f.to_string()) {
            let insert = match *f {
                "at" => "at(${1:vector}, ${2:0})$0".into(),
                "vec" => "vec(${1:first}, ${2:second})$0".into(),
                "cases" => "cases(${1:0}, ${2:t < 0}, ${3:1})$0".into(),
                "diff" => "diff(${1:expression}, ${2:x})$0".into(),
                "integrate" => "integrate(${1:expression}, ${2:x}, ${3:0}, ${4:1})$0".into(),
                _ => format!("{f}($0)"),
            };
            let detail = match *f {
                "at" => "at(vector, index): select a component, starting at 0",
                "vec" => "vec(a, b, …): a column vector",
                "cases" => "cases(value, condition, …, default): first matching branch",
                "rect" => "rect(x): 1 for |x| < ½, ½ at ±½, otherwise 0",
                "tri" => "tri(x): max(1 − |x|, 0)",
                "sgn" => "sgn(x): −1, 0 or 1 according to the sign of x",
                _ => "built-in",
            };
            items.push(Item::new(f, ItemKind::FUNCTION, detail, &insert));
        }
    }
    for (c, d) in CONSTANTS {
        if seen.insert(c.to_string()) {
            items.push(Item::new(c, ItemKind::CONSTANT, d, c));
        }
    }
    items
}

static FENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|[^`])```([\w+#.-]*)$").unwrap());

/// Language tags after an opening fence (```` ```sol| ````), solveng first.
fn tag_completions(a: &Analysis, at: usize) -> Option<Completions> {
    let line_start = last_index_of_char(&a.doc.c, '\n', at).map_or(0, |i| i + 1);
    let line = a.doc.sub(line_start, at);
    let m = FENCE.captures(&line)?;
    let typed = m[1].to_lowercase();
    let items = std::iter::once(TAG)
        .chain(TAGS.iter().copied())
        .filter(|t| t.to_lowercase().starts_with(&typed))
        .map(|t| Item::new(t, ItemKind::CONSTANT, if t == TAG { "calculation" } else { "code" }, t))
        .collect();
    Some(Completions { to: None, from: a.doc.to16(at - m[1].chars().count()), items, exclusive: true })
}

fn word_start(c: &[char], at: usize, floor: usize) -> usize {
    let (text, offsets) = crate::text::unescape(&c[floor..at]);
    let (mut start, mut begun, mut quoted, mut escaped, mut grouped) = (text.len(), false, false, false, false);
    for (i, &ch) in text.iter().enumerate() {
        if escaped { escaped = false; continue; }
        if quoted {
            if ch == '\\' { escaped = true; }
            if ch == '"' { quoted = false; }
            continue;
        }
        if ch == '"' {
            if !begun { start = i; }
            begun = true;
            quoted = true;
        } else if begun && ch == '(' && i > 0 && text[i - 1] == '_' { grouped = true; }
        else if grouped && (ch.is_whitespace() || ch == ')') { if ch == ')' { grouped = false; } }
        else if solveng_names::is_start(ch) || (begun && solveng_names::is_continue(ch)) {
            if !begun { start = i; }
            begun = true;
        } else { start = i + 1; begun = false; }
    }
    floor + offsets[start]
}

/// Completions when `offset` is in solveng code, math or a string or option of a solveng function, else None
/// (and the Typst language server answers).
pub fn complete(document: &str, context: &Context, offset: u32) -> Option<Completions> {
    let mut result = complete_at(document, context, offset)?;
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let start = a.doc.from16(result.from as usize);
    let math = a.elements.iter().any(|e| e.math && e.body_start <= at && at <= e.body_end);
    let string = a.calls.iter().flat_map(|c| &c.texts).find(|t| t.start <= at && at <= t.end);
    let name_context = string.is_none_or(|t| matches!(t.role, Role::Code | Role::Mark | Role::Name));
    if !name_context { return Some(result); }
    let prefix = a.doc.sub(start, at);
    let mut has_names = false;
    result.items.retain_mut(|item| {
        if !matches!(item.kind, ItemKind::VARIABLE | ItemKind::FUNCTION) { return true; }
        let label = before(before(&item.label, "("), ": ");
        let canonical = crate::resolved_name(label);
        if !solveng_names::matches_prefix(&canonical, &prefix) { return false; }
        has_names = true;
        // Client-side filtering must accept the exact spelling typed, including transport quotes.
        if !prefix.is_empty() { item.filter = Some(prefix.clone()); }
        if math {
            let standard = NATIVE_FUNCTIONS.contains(&canonical.as_str());
            let base = if item.typst { label.to_string() }
                else if item.kind == ItemKind::FUNCTION && standard { canonical.clone() }
                else { math_name(&canonical) };
            let suffix = item.insert.find('(').map(|at| item.insert[at..].to_string()).unwrap_or_default();
            item.insert = if item.kind == ItemKind::FUNCTION { base + &suffix } else { base };
        } else if string.is_some() {
            item.insert = item.insert.replace('\\', "\\\\").replace('"', "\\\"");
        }
        true
    });
    if has_names || prefix.starts_with('"') || prefix.starts_with("\\\"") {
        let exact = crate::resolved_name(&prefix);
        result.items.sort_by_key(|item| {
            let name = before(before(&item.label, "("), ": ");
            if matches!(item.kind, ItemKind::VARIABLE | ItemKind::FUNCTION) && crate::resolved_name(name) == exact { 0 } else { 1 }
        });
        let floor = string.map_or_else(|| a.elements.iter().find(|e| e.body_start <= at && at <= e.body_end).map_or(start, |e| e.body_end), |t| t.end);
        let (decoded, offsets) = crate::text::unescape(&a.doc.c[start..floor]);
        let local_at = offsets.partition_point(|&i| i < at - start);
        let end = solveng_names::scan(&decoded, 0).ok().flatten().map(|n| n.end).filter(|&e| e >= local_at);
        if let Some(end) = end { result.to = Some(a.doc.to16(start + offsets[end])); }
        if let Some(group) = decoded.windows(2).position(|w| w == ['_', '(']) {
            let mut i = group + 2;
            while i < decoded.len() {
                if decoded[i] == '"' { i = solveng_names::quoted(&decoded, i).map_or(decoded.len(), |(_, end)| end); }
                else if decoded[i] == ')' { result.to = Some(a.doc.to16(start + offsets[i + 1])); break; }
                else { i += 1; }
            }
        }
        let end = result.to.map_or(at, |to| a.doc.from16(to as usize));
        if a.doc.c[end..].iter().find(|c| !c.is_whitespace()) == Some(&'(') {
            for item in &mut result.items {
                if item.kind == ItemKind::FUNCTION && let Some(open) = item.insert.find('(') { item.insert.truncate(open); }
            }
        }
    }
    Some(result)
}

fn complete_at(document: &str, context: &Context, offset: u32) -> Option<Completions> {
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let done = |from: usize, items: Vec<Item>, exclusive: bool| Some(Completions { to: None, from: a.doc.to16(from), items, exclusive });
    // The innermost call's string around the caret.
    let call = a.calls.iter().rev().find(|c| c.args_start <= at && at <= c.args_end);
    if let Some(call) = call
        && let Some(t) = call.texts.iter().find(|t| t.start <= at && at <= t.end) {
            return string_completions(&a, call, t, at);
        }
    let element = a.elements.iter().find(|e| e.body_start <= at && at <= e.body_end);
    // Right after ``` or a tag being typed.
    if element.is_none_or(|e| !e.math && e.ticks() >= 3 && at == e.body_start)
        && let Some(c) = tag_completions(&a, at) {
            return Some(c);
        }
    let Some(e) = element else { return call.and_then(|c| option_completions(&a, c, at)) };
    if !a.shown(e) {
        return None;
    }
    let from = word_start(&a.doc.c, at, e.body_start);
    if e.math {
        return complete_math(&a, e, from);
    }
    let line_start = e.body_start.max(last_index_of_char(&a.doc.c, '\n', from).map_or(0, |i| i + 1));
    let line = a.doc.sub(line_start, from);
    done(from, code_items(&a, &line), true)
}

/// Whether the document imports all of solveng (`#import "@preview/solveng:…": *`), which brings the names of
/// units longer than a letter, so math needs no quotes for them: `$4 mA$`.
fn imports_unit_words(text: &str) -> bool {
    static IMPORT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"#import\s+"@[^"]*/solveng:[^"]*"\s*:\s*\*"#).unwrap());
    IMPORT.is_match(text)
}

/// In math: units in the quotes after a number (`30 "m|"`), else the defined names next to Typst's.
fn complete_math(a: &Analysis, e: &Element, from: usize) -> Option<Completions> {
    let before_ = a.doc.sub(e.body_start, from);
    if let Some(names) = crate::clause::at_clause(&before_) {
        return Some(Completions { to: None, from: a.doc.to16(from), items: clause_items(a, names, true), exclusive: !names });
    }
    // V: |, q(x: |: units only; quoted when longer than a letter, as math needs them, unless solveng's import brings
    // their names (built-in units only)
    let bare = imports_unit_words(&a.doc.s);
    let quote = |mut i: Item| {
        let defined = a.units.iter().any(|u| u.name == i.insert);
        if i.insert.chars().count() > 1 && (!bare || defined || i.kind == ItemKind::KEYWORD) {
            i.filter.get_or_insert_with(|| i.insert.clone());
            i.insert = format!("\"{}\"", i.insert);
        }
        i
    };
    if declared_unit(&before_) {
        let items = keyword_items(&before_, false).into_iter().chain(units::unit_items(&a.units)).map(quote).collect();
        return Some(Completions { to: None, from: a.doc.to16(from), items, exclusive: true });
    }
    // after a number: its unit, or a variable it multiplies
    if before_.trim_end().chars().last().is_some_and(is_digit) {
        let mut items: Vec<Item> = units::unit_items(&a.units).into_iter().map(quote).collect();
        if a.doc.c.get(from) == Some(&'"') {
            return Some(Completions { to: None, from: a.doc.to16(from), items, exclusive: true });
        }
        items.extend(definition_items(a, &mut HashSet::new(), true, true));
        return Some(Completions { to: None, from: a.doc.to16(from), items, exclusive: false });
    }
    if before_.ends_with('_') {
        return None;
    }
    let mut items = code_items(a, "");
    items.retain(|i| !FUNCTIONS.iter().any(|f| i.label == *f || i.label.starts_with(&format!("{f}("))) || NATIVE_FUNCTIONS.iter().any(|f| i.label == *f || i.label.starts_with(&format!("{f}("))));
    if let Some(item) = items.iter_mut().find(|item| item.label == "cases") {
        item.insert = "cases(${1:0} & ${2:t < 0}, ${3:1} & \"otherwise\")$0".into();
        item.detail = "cases(value & condition, …): first matching branch".into();
    }
    items.extend(crate::operators::completions(a, from));
    items.extend(keyword_items(&before_, true));
    if items.is_empty() { None } else { Some(Completions { to: None, from: a.doc.to16(from), items, exclusive: false }) }
}

/// In a string argument: code, units, names or the option's values, as the parameter takes.
fn string_completions(a: &Analysis, call: &Call, t: &crate::calls::Text, at: usize) -> Option<Completions> {
    let c = &a.doc.c;
    let from = word_start(c, at, t.start);
    let items = match t.role {
        Role::Code => {
            let line_start = t.start.max(last_index_of_char(c, '\n', from).map_or(0, |i| i + 1));
            code_items(a, &a.doc.sub(line_start, from))
        }
        Role::Unit => units::unit_items(&a.units),
        Role::Name => definition_items(a, &mut HashSet::new(), false, false),
        Role::Mark => {
            let mut items: Vec<Item> = MARKS.iter().map(|(m, d)| Item::new(m, ItemKind::VALUE, d, m)).collect();
            items.extend(code_items(a, &a.doc.sub(t.start, from)));
            items
        }
        Role::Choice => {
            let param = call.function.named.iter().find(|p| Some(&p.name) == t.param.as_ref())?;
            let mut items: Vec<Item> = param.values.iter().map(|v| Item::new(v, ItemKind::VALUE, &param.types, v)).collect();
            if param.name == "included" || param.name == "excluded" {
                items.extend(definition_items(a, &mut HashSet::new(), false, true).into_iter().map(|mut i| {
                    i.insert = crate::text::before(&i.label, "(").into();
                    i
                }));
            }
            // Values like `inline-steps` hold dashes, so the whole string typed so far is replaced.
            return Some(Completions { to: None, from: a.doc.to16(t.start), items, exclusive: true });
        }
    };
    Some(Completions { to: None, from: a.doc.to16(from), items, exclusive: true })
}

static VALUE_AT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|[,(\s])([a-z][\w-]*)\s*:\s*([\w-]*)$").unwrap());
static NAME_AT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:^|[,(])\s*([a-z][\w-]*)?$").unwrap());

/// Between a call's arguments: the names of its parameters, or after `name:` the option's values.
fn option_completions(a: &Analysis, call: &Call, at: usize) -> Option<Completions> {
    let prefix = a.doc.sub(call.args_start, at);
    // Only at the top level of the arguments.
    let mut depth = 0i32;
    let mut in_str = false;
    let pc = chars(&prefix);
    let mut i = 0;
    while i < pc.len() {
        match pc[i] {
            '\\' if in_str => i += 1,
            '"' => in_str = !in_str,
            '(' | '[' | '{' if !in_str => depth += 1,
            ')' | ']' | '}' if !in_str => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    if in_str || depth != 0 {
        return None;
    }
    let f = call.function;
    if let Some(m) = VALUE_AT.captures(&prefix) {
        let param = f.named.iter().find(|p| p.name == m[1])?;
        let typed = m[2].chars().count();
        let quoted = |v: &str, kind: u8, detail: &str| {
            let q = format!("\"{v}\"");
            Item::new(&q, kind, detail, &q)
        };
        let items: Vec<Item> = if !param.values.is_empty() {
            param.values.iter().map(|v| quoted(v, ItemKind::VALUE, &param.types)).collect()
        } else if param.name == "unit" {
            units::unit_items(&a.units).iter().map(|u| quoted(&u.insert, ItemKind::UNIT, &u.detail)).collect()
        } else {
            return None;
        };
        return Some(Completions { to: None, from: a.doc.to16(at - typed), items, exclusive: false });
    }
    let m = NAME_AT.captures(&prefix)?;
    let typed = m.get(1).map_or(0, |t| t.as_str().chars().count());
    let given: HashSet<&str> = call.named.iter().filter(|n| n.name_end < at - typed || n.name_start > at).map(|n| n.name.as_str()).collect();
    let items = f
        .named
        .iter()
        .filter(|p| !given.contains(p.name.as_str()) && !p.doc.is_empty())
        .map(|p| {
            let detail = if p.default.is_empty() { p.types.clone() } else { format!("{} = {}", p.types, p.default) };
            let mut item = Item::new(&p.name, ItemKind::PARAMETER, &detail, &format!("{}: ", p.name));
            item.documentation = Some(p.doc.clone());
            item
        })
        .collect();
    Some(Completions { to: None, from: a.doc.to16(at - typed), items, exclusive: false })
}

/// The info of a solveng code fence, with the document's own units for [`crate::pretty`]: `solveng units=bar,ü`.
fn fence(a: &Analysis) -> String {
    let units: Vec<&str> = a.units.iter().map(|u| u.name.as_str()).collect();
    if units.is_empty() { "solveng".into() } else { format!("solveng units={}", units.join(",")) }
}

/// The detail of a defined name, for hovers: a value or formula as solveng code, which editors highlight and show
/// with [`crate::pretty`].
pub(crate) fn defined(a: &Analysis, name: &str) -> Option<String> {
    let mut seen = HashSet::new();
    let resolved = crate::resolved_name(name);
    definition_items(a, &mut seen, false, true)
        .into_iter()
        .find(|i| before(&i.label, "(") == name || (i.kind == ItemKind::VARIABLE && before(&i.label, ":") == resolved))
        .map(|i| if i.detail.starts_with('=') { format!("```{}\n{} {}\n```", fence(a), i.label, i.detail) } else { format!("`{}` {}", i.label, i.detail) })
}
