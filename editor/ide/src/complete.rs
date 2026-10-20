//! Completions: names, functions, constants and units in solveng code, math and call strings; the options of
//! solveng's functions; language tags after ```` ``` ````.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::calls::{Call, Role};
use crate::elements::Element;
use crate::text::{Doc, after, before, chars, is_digit, is_letter_or_digit, last_index_of_char};
use crate::tokens::CONSTANTS;
use crate::units::{self, Custom};
use crate::{Analysis, Completions, Context, Item, ItemKind, TAG};

pub(crate) const FUNCTIONS: &[&str] = &["sqrt", "exp", "ln", "log", "sin", "cos", "tan", "asin", "acos", "atan", "abs"];
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
    /// Where its statement ends, from where the name holds the new value.
    pub statement_end: usize,
}

impl Definition {
    /// The name without a function's parameters.
    pub fn base(&self) -> &str {
        before(&self.name, "(")
    }
}

static NAME_AT_START: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*([\p{L}][\p{L}\p{N}_]*)\s*(\([^()=]*\))?\s*(<=|>=|=|<|>|≤|≥)(.*)$").unwrap());
static MATH_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*[\p{L}][\p{L}\p{N}]*(_[\p{L}\p{N}]+)?\s*$").unwrap());
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

/// `I = 30 mA` or `f(x) = x^2` → a definition; anything else (`Q0/2 = I t`, `= x`) → None.
pub(crate) fn definition(line: &str) -> Option<Definition> {
    let m = NAME_AT_START.captures(line)?;
    let (name, params, relation, rest) = (&m[1], m.get(2).map_or("", |p| p.as_str()), &m[3], &m[4]);
    if relation == "=" && rest.starts_with('=') {
        return None;
    }
    let value = before(rest, "->").trim();
    let unit = after(rest, "->", "").trim();
    let detail = format!("{relation} {value}") + &if unit.is_empty() { String::new() } else { format!(" → {unit}") };
    Some(if params.is_empty() {
        Definition { name: name.into(), detail, function: false, start: 0, end: 0, statement_end: 0 }
    } else {
        Definition { name: format!("{name}{params}"), detail, function: true, start: 0, end: 0, statement_end: 0 }
    })
}

/// The definitions of solveng code in `start..end` of `doc`, line by line and `;` by `;`, comments left out.
fn code_definitions(doc: &Doc, start: usize, end: usize, found: &mut Vec<Definition>) {
    let c = &doc.c;
    let mut i = start;
    while i < end {
        let line_end = (i..end).find(|&j| c[j] == '\n').unwrap_or(end);
        let line = doc.sub(i, line_end);
        let mut at = i;
        for piece in before(&line, "//").split(';') {
            if let Some(mut d) = definition(piece) {
                d.start = at + piece.chars().take_while(|ch| ch.is_whitespace()).count();
                d.end = d.start + d.base().chars().count();
                d.statement_end = at + piece.chars().count();
                found.push(d);
            }
            at += piece.chars().count() + 1;
        }
        i = line_end + 1;
    }
}

/// Names defined in a text, in order: math equations and solveng code that `active` says are calculated, and
/// `def` / `solve` strings.
pub(crate) fn definitions(doc: &Doc, elements: &[Element], calls: &[Call], active: &dyn Fn(&Element) -> bool) -> Vec<Definition> {
    let mut found = Vec::new();
    for e in elements.iter().filter(|e| active(e)) {
        if !e.math {
            code_definitions(doc, e.body_start, e.body_end, &mut found);
            continue;
        }
        // Math writes Q_0, t_"half" and quotes units: Q_0 = 3450 "mAh".
        let body = doc.sub(e.body_start, e.body_end);
        let plain = body.replace('"', "");
        if !MATH_NAME.is_match(before(&plain, "=")) {
            continue;
        }
        if let Some(mut d) = definition(&SPACES.replace_all(&plain, " ")) {
            let c = &doc.c[e.body_start..e.body_end];
            let relation = c.iter().position(|ch| "=<>≤≥".contains(*ch)).unwrap_or(c.len());
            let lead = c.iter().take_while(|ch| ch.is_whitespace()).count();
            let end = (lead..relation).rev().find(|&k| !c[k].is_whitespace()).map_or(lead, |k| k + 1);
            d.start = e.body_start + lead;
            d.end = e.body_start + end;
            d.statement_end = e.body_end;
            found.push(d);
        }
    }
    for call in calls.iter().filter(|c| c.function.name == "def" || c.function.name == "solve") {
        for t in call.texts.iter().filter(|t| t.param.is_none() && t.role == Role::Code) {
            code_definitions(doc, t.start, t.end, &mut found);
        }
    }
    found.sort_by_key(|d| d.start);
    found
}

/// A solveng name as math writes it: `t_half` → `t_"half"`; `Q_0`, `rho` and `Q0` stay.
pub(crate) fn math_name(name: &str) -> String {
    let base = before(name, "_");
    let sub = after(name, "_", "");
    if sub.is_empty() {
        return name.into();
    }
    if sub.chars().count() == 1 || sub.chars().all(|c| c.is_ascii_digit()) { format!("{base}_{sub}") } else { format!("{base}_\"{sub}\"") }
}

static ARROW_UNIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"->\s*[^=;]*$").unwrap());
static VALUE_UNITS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[^\p{L}\p{N}_])\d[\d.,]*(?:[eE][-+]?\d+)?\s*((?:[\p{L}°%‰µΩ]+(?:\^-?\d+|[²³])?\s*[/*·]?\s*)*)$").unwrap()
});
static UNIT_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\p{L}°%‰µΩ]+").unwrap());

/// Before the word being typed: a value's unit (`30 m|`, `[5 |`, `-> |`, `kg m/|`).
fn unit_context(line: &str, custom: &[Custom]) -> bool {
    if ARROW_UNIT.is_match(line) {
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
    let mut defs = Vec::new();
    for part in a.around() {
        defs.extend(definitions(&part.doc, &part.elements, &part.calls, &|e| part.active(e)));
    }
    defs.extend(definitions(&a.doc, &a.elements, &a.calls, &|e| a.active(e)));
    let compiled = |name: &str| a.context.defined.iter().find(|d| d.name == name && d.kind != "function" && !d.value.is_empty());
    let mut items = Vec::new();
    for d in defs.iter().rev() {
        let name = before(&d.name, "(");
        if !seen.insert(name.to_string()) || (d.function && !functions) {
            continue;
        }
        let spelled = if math { math_name(name) } else { name.to_string() };
        if d.function {
            let params = before(after(&d.name, "(", ""), ")");
            let caret = if params.trim().is_empty() { "" } else { "$0" };
            items.push(Item::new(&d.name, ItemKind::FUNCTION, &d.detail, &format!("{spelled}({caret})")));
        } else {
            let mut detail = d.detail.clone();
            if let Some(v) = compiled(name)
                && !detail.ends_with(&format!(" {}", v.value)) {
                    detail = format!("{detail} = {}", v.value);
                }
            items.push(Item::new(name, ItemKind::VARIABLE, &detail, &spelled));
        }
    }
    // Defined where the text doesn't say, e.g. in a package or by a call with a computed string.
    for d in &a.context.defined {
        if d.name.is_empty() || !seen.insert(d.name.clone()) {
            continue;
        }
        let spelled = if math { math_name(&d.name) } else { d.name.clone() };
        match d.value.split_once(") = ") {
            Some((params, body)) if d.kind == "function" && params.starts_with('(') => {
                if functions {
                    let label = format!("{}{params})", d.name);
                    items.push(Item::new(&label, ItemKind::FUNCTION, &format!("= {body}"), &format!("{spelled}($0)")));
                }
            }
            _ => items.push(Item::new(&d.name, ItemKind::VARIABLE, &format!("= {}", d.value), &spelled)),
        }
    }
    items
}

/// What solveng code completes at a word: names, solveng's functions and constants, or units after a value.
fn code_items(a: &Analysis, line: &str) -> Vec<Item> {
    // Comments complete nothing.
    if line.contains("//") {
        return Vec::new();
    }
    if unit_context(line, &a.units) {
        return units::unit_items(&a.units);
    }
    let mut seen = HashSet::new();
    let mut items = definition_items(a, &mut seen, false, true);
    for f in FUNCTIONS {
        if seen.insert(f.to_string()) {
            items.push(Item::new(f, ItemKind::FUNCTION, "built-in", &format!("{f}($0)")));
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
    Some(Completions { from: a.doc.to16(at - m[1].chars().count()), items, exclusive: true })
}

fn word_start(c: &[char], at: usize, floor: usize) -> usize {
    let mut from = at;
    while from > floor && (is_letter_or_digit(c[from - 1]) || "_°%‰µΩ".contains(c[from - 1])) {
        from -= 1;
    }
    from
}

/// Completions when `offset` is in solveng code, math or a string or option of a solveng function, else None
/// (and the Typst language server answers).
pub fn complete(document: &str, context: &Context, offset: u32) -> Option<Completions> {
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let done = |from: usize, items: Vec<Item>, exclusive: bool| Some(Completions { from: a.doc.to16(from), items, exclusive });
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

/// In math: units in the quotes after a number (`30 "m|"`), else the defined names next to Typst's.
fn complete_math(a: &Analysis, e: &Element, from: usize) -> Option<Completions> {
    let before_ = a.doc.sub(e.body_start, from);
    if before_.matches('"').count() % 2 == 1 {
        let quote = before_.rfind('"').unwrap();
        let previous = before_[..quote].trim_end().chars().last()?;
        if !is_digit(previous) && previous != '"' {
            return None;
        }
        return Some(Completions { from: a.doc.to16(from), items: units::unit_items(&a.units), exclusive: true });
    }
    if before_.ends_with('_') {
        return None;
    }
    let items = definition_items(a, &mut HashSet::new(), true, true);
    if items.is_empty() { None } else { Some(Completions { from: a.doc.to16(from), items, exclusive: false }) }
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
            return Some(Completions { from: a.doc.to16(t.start), items, exclusive: true });
        }
    };
    Some(Completions { from: a.doc.to16(from), items, exclusive: true })
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
        } else if matches!(param.name.as_str(), "unit" | "xunit" | "yunit") {
            units::unit_items(&a.units).iter().map(|u| quoted(&u.insert, ItemKind::UNIT, &u.detail)).collect()
        } else {
            return None;
        };
        return Some(Completions { from: a.doc.to16(at - typed), items, exclusive: false });
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
    Some(Completions { from: a.doc.to16(at - typed), items, exclusive: false })
}

/// The detail of a defined name, for hovers.
pub(crate) fn defined(a: &Analysis, name: &str) -> Option<String> {
    let mut seen = HashSet::new();
    definition_items(a, &mut seen, false, true).into_iter().find(|i| before(&i.label, "(") == name).map(|i| format!("`{}` {}", i.label, i.detail))
}
