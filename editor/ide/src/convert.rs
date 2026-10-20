//! Rewriting between solveng's spellings: `#def("I = 30 mA")` ⇄ `$I = 30 "mA"$` and
//! `#solve("R = U/I", unit: "mA")` ⇄ ```` ```solveng R = U/I -> mA``` ````.

use std::sync::LazyLock;

use regex::Regex;

use crate::TokenKind as K;
use crate::complete::math_name;
use crate::conceal::greek;
use crate::text::{after, before, chars, is_digit, is_letter, sub};
use crate::tokens::code_tokens;
use crate::units::{SPELLINGS, is_unit};
use crate::{Analysis, Context, Conversion, TAG};

/// Rewrites what's at `offset` into its other spelling. Empty where nothing converts.
pub fn conversions(document: &str, context: &Context, offset: u32) -> Vec<Conversion> {
    if !document.contains(TAG) && !context.text.contains(TAG) {
        return Vec::new();
    }
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let mut out = Vec::new();
    let conversion = |title: &str, start: usize, end: usize, text: String| Conversion {
        title: title.into(),
        start: a.doc.to16(start),
        end: a.doc.to16(end),
        text,
    };
    // The innermost `#def(…)` or `#solve(…)` around the caret.
    let call = a.calls.iter().rev().find(|c| {
        (c.function.name == "def" || c.function.name == "solve") && c.start > 0 && a.doc.c[c.start - 1] == '#'
            && c.start - 1 <= at && at <= c.end && c.args_end < c.end
    });
    if let Some(c) = call {
        match c.function.name.as_str() {
            "def" => {
                if let Some(t) = def_to_math(&c.args) {
                    out.push(conversion("Convert to Math", c.start - 1, c.end, t));
                }
            }
            _ => {
                if let Some(t) = solve_to_code(&c.args) {
                    out.push(conversion("Convert to solveng Code", c.start - 1, c.end, t));
                }
            }
        }
    }
    let e = a.elements.iter().find(|e| e.start <= at && at <= e.end);
    if let Some(e) = e {
        let body = a.doc.sub(e.body_start, e.body_end);
        if e.math {
            if let Some(code) = from_math(&body) {
                out.push(conversion("Convert to #def", e.start, e.end, format!("#def({})", quote(&code))));
            }
        } else if a.shown(e)
            && let Some(t) = code_to_solve(&body, e.ticks() >= 3 && body.contains('\n')) {
                out.push(conversion("Convert to #solve", e.start, e.end, t));
            }
    }
    out
}

/// A string literal's value, or None for anything else.
fn string(arg: &str) -> Option<String> {
    let c = chars(arg);
    if c.len() < 2 || c[0] != '"' || c[c.len() - 1] != '"' {
        return None;
    }
    let mut out = String::new();
    let mut i = 1;
    while i < c.len() - 1 {
        let mut ch = c[i];
        if ch == '"' {
            return None;
        }
        if ch == '\\' {
            i += 1;
            ch = *c.get(i)?;
        }
        out.push(ch);
        i += 1;
    }
    Some(out)
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn def_to_math(args: &[(Option<String>, String)]) -> Option<String> {
    if args.is_empty() || args.iter().any(|a| a.0.is_some()) {
        return None;
    }
    let equations: Option<Vec<String>> = args.iter().map(|a| string(&a.1).and_then(|s| to_math(&s))).collect();
    Some(equations?.iter().map(|e| format!("${e}$")).collect::<Vec<_>>().join(" "))
}

static RELATION: LazyLock<Regex> = LazyLock::new(|| Regex::new("[=<>≤≥]").unwrap());

fn solve_to_code(args: &[(Option<String>, String)]) -> Option<String> {
    let equations: Option<Vec<String>> = args.iter().filter(|a| a.0.is_none()).map(|a| string(&a.1)).collect();
    let equations = equations?;
    if equations.is_empty() || equations.iter().any(|e| e.contains("//") || e.contains("->") || e.contains('\n')) {
        return None;
    }
    let named: Vec<(&str, &str)> = args.iter().filter_map(|a| a.0.as_deref().map(|n| (n, a.1.as_str()))).collect();
    if named.iter().any(|(n, _)| !["unit", "inline", "steps"].contains(n)) {
        return None;
    }
    let get = |k: &str| named.iter().find(|(n, _)| *n == k).map(|(_, v)| *v);
    let unit = match get("unit") {
        Some(u) => Some(string(u)?),
        None => None,
    };
    let inline = match get("inline") {
        None | Some("false") => false,
        Some("true") => true,
        _ => return None,
    };
    let steps = match get("steps") {
        Some(s) => Some(string(s)?),
        None => None,
    };
    // Inline code shows the result; "= x" only the value.
    let value = steps.as_deref() == Some("value");
    if steps.is_some() && !value && !(inline && steps.as_deref() == Some("result")) {
        return None;
    }
    if value && (!inline || equations.len() != 1 || RELATION.is_match(&equations[0])) {
        return None;
    }
    let line = format!(
        "{}{}{}",
        if value { "= " } else { "" },
        equations.join("; "),
        unit.map(|u| format!(" -> {u}")).unwrap_or_default()
    );
    Some(if inline { format!("```solveng {line}```") } else { format!("```solveng\n{line}\n```") })
}

fn code_to_solve(code: &str, block: bool) -> Option<String> {
    let mut calls = Vec::new();
    for raw in code.split('\n') {
        let comment = after(raw, "//", "").trim();
        let line = before(raw, "//").trim();
        if line.is_empty() {
            if !comment.is_empty() {
                calls.push(format!("// {comment}"));
            }
            continue;
        }
        let equation = before(line, "->").trim();
        let unit = after(line, "->", "").trim();
        let value = equation.starts_with('=');
        let eqs: Vec<&str> = (if value { &equation[1..] } else { equation }).split(';').map(str::trim).filter(|e| !e.is_empty()).collect();
        if eqs.is_empty() {
            return None;
        }
        let mut args: Vec<String> = eqs.iter().map(|e| quote(e)).collect();
        if !unit.is_empty() {
            args.push(format!("unit: {}", quote(unit)));
        }
        if !block {
            args.push("inline: true".into());
        }
        if value {
            args.push("steps: \"value\"".into());
        }
        calls.push(format!("#solve({})", args.join(", ")) + &if comment.is_empty() { String::new() } else { format!(" // {comment}") });
    }
    if calls.is_empty() {
        return None;
    }
    Some(calls.join(if block { "\n" } else { " " }))
}

const MATH_FUNCTIONS: &[&str] = &["sqrt", "exp", "ln", "log", "sin", "cos", "tan", "abs"];
static LETTER_DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{L}\d+$").unwrap());
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(" {2,}").unwrap());

/// A `def` string as math: `I = 30 mA` → `I = 30 "mA"`; None for what math can't say the same way.
pub fn to_math(code: &str) -> Option<String> {
    if code.contains('@') || code.contains("->") || code.contains("//") || code.contains('$') {
        return None;
    }
    let c = chars(code);
    let tokens = code_tokens(&c, &|_| false);
    let mut out = String::new();
    let mut i = 0;
    while i < c.len() {
        if let Some(&(_, len, kind)) = tokens.iter().find(|t| t.0 == i) {
            let word = sub(&c, i, i + len);
            match kind {
                K::UNIT => {
                    if word.chars().all(|ch| is_digit(ch) || "²³⁻¹".contains(ch)) {
                        out.push_str(&word);
                    } else {
                        out.push('"');
                        out.push_str(&spell_unit(&word));
                        out.push('"');
                    }
                }
                K::FUNCTION => {
                    if !MATH_FUNCTIONS.contains(&word.as_str()) && word.chars().count() > 1 {
                        return None;
                    }
                    out.push_str(&word);
                }
                K::VARIABLE | K::KEYWORD => {
                    let base = before(&word, "_");
                    if base.chars().count() > 1 && greek(base).is_none() && !LETTER_DIGITS.is_match(base) {
                        return None;
                    }
                    out.push_str(&math_name(&word));
                }
                _ => out.push_str(&word),
            }
            i += len;
            continue;
        }
        match c[i] {
            '[' | ']' => {}
            '*' => out.push_str(" dot "),
            ch => out.push(ch),
        }
        i += 1;
    }
    Some(SPACES.replace_all(&out, " ").trim().to_string())
}

/// A unit as written in math quotes: `kOhm` → `kΩ`, `uA` → `µA`.
fn spell_unit(unit: &str) -> String {
    let mut u = unit.to_string();
    for (spelling, symbol) in SPELLINGS {
        if let Some(rest) = u.strip_suffix(spelling) {
            u = format!("{rest}{symbol}");
        }
    }
    if u.chars().count() > 1 && u.starts_with('u') && is_unit(&u[1..]) {
        u = format!("µ{}", &u[1..]);
    }
    u
}

fn matching(s: &[char], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (j, c) in s.iter().enumerate().skip(open) {
        if *c == '(' {
            depth += 1;
        } else if *c == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(j);
            }
        }
    }
    None
}

fn split_top(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for c in s.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    parts.push(current);
    parts.iter().map(|p| p.trim().to_string()).collect()
}

/// An equation's body as a `def` string: `I = 30 "mA"` → `I = 30 mA`; None for what isn't a calculation.
pub fn from_math(math: &str) -> Option<String> {
    if math.chars().any(|c| "#&\\$".contains(c)) || RELATION.find_iter(math).count() != 1 {
        return None;
    }
    let m = chars(math);
    let n = m.len();
    let mut out = String::new();
    // The last thing written: a number or unit lets a unit follow bare; else it needs brackets.
    let mut after_value = false;
    let word = |start: usize| {
        let mut j = start;
        while j < n && (is_letter(m[j]) || (m[j] == '.' && j + 1 < n && is_letter(m[j + 1]))) {
            j += 1;
        }
        j
    };
    let mut i = 0;
    while i < n {
        let c = m[i];
        if c == '"' {
            let end = (i + 1..n).find(|&j| m[j] == '"')?;
            let mut text = sub(&m, i + 1, end);
            i = end + 1;
            if out.ends_with('_') {
                out.push_str(&text);
                continue;
            }
            // A quoted prefix before a symbol: "k"Omega.
            let next = word(i);
            if next > i && ["Omega", "ohm"].contains(&sub(&m, i, next).as_str()) {
                text.push('Ω');
                i = next;
            } else if i < n && m[i] == 'Ω' {
                text.push('Ω');
                i += 1;
            }
            let unit = text.replace('Ω', "Ohm");
            if after_value { out.push_str(&unit) } else { out.push_str(&format!("[{unit}]")) }
            after_value = true;
        } else if is_digit(c) {
            let mut j = i;
            while j < n && (is_digit(m[j]) || (".,".contains(m[j]) && j + 1 < n && is_digit(m[j + 1]))) {
                j += 1;
            }
            after_value = !out.ends_with('^');
            out.push_str(&sub(&m, i, j).replace(',', "."));
            i = j;
        } else if is_letter(c) {
            let j = word(i);
            let w = sub(&m, i, j);
            i = j;
            if w == "dot" || w == "times" || w == "dot.op" {
                out.push('*');
                after_value = false;
            } else if w == "Omega" && after_value {
                out.push_str("Ohm");
            } else if w == "frac" && i < n && m[i] == '(' {
                let close = matching(&m, i)?;
                let parts = split_top(&sub(&m, i + 1, close));
                if parts.len() != 2 {
                    return None;
                }
                let num = from_math(&format!("{} = 0", parts[0]))?;
                let den = from_math(&format!("{} = 0", parts[1]))?;
                let strip = |s: &str| s.strip_suffix(" = 0").unwrap_or(s).to_string();
                out.push_str(&format!("({})/({})", strip(&num), strip(&den)));
                i = close + 1;
                after_value = false;
            } else {
                if w.contains('.') {
                    return None;
                }
                out.push_str(&w);
                after_value = false;
            }
        } else if c == '_' && i + 1 < n && m[i + 1] == '(' {
            let close = matching(&m, i + 1)?;
            out.push('_');
            out.push_str(sub(&m, i + 2, close).replace('"', "").trim());
            i = close + 1;
        } else if "·×⋅".contains(c) {
            out.push('*');
            after_value = false;
            i += 1;
        } else if c == ' ' || c == '\n' || c == '\t' {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            i += 1;
        } else if c == '/' || c == '^' {
            out.push(c);
            i += 1;
        } else {
            out.push(c);
            after_value = false;
            i += 1;
        }
    }
    let out = out.trim().to_string();
    if out.is_empty() { None } else { Some(out) }
}
