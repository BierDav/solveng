//! Rewriting between solveng's two spellings: the lines of a ```` ```solveng ```` code block ⇄ math in a
//! worksheet, `t := Q0/I = [h]` ⇄ `$t := Q_0/I = h$`.

use std::sync::LazyLock;

use regex::Regex;

use crate::TokenKind as K;
use crate::clause::{is_keyword, marker, split_top};
use crate::complete::math_name;
use crate::text::{after, before, chars, is_digit, is_letter, sub};
use crate::tokens::code_tokens;
use crate::units::{SPELLINGS, is_unit};
use crate::{Analysis, Context, Conversion, TAG};

/// Rewrites what's at `offset` into its other spelling. Empty where nothing converts.
pub fn conversions(document: &str, context: &Context, offset: u32) -> Vec<Conversion> {
    let read = |t: &str| t.contains("worksheet") || t.contains(TAG);
    if !read(document) && !read(&context.text) {
        return Vec::new();
    }
    let a = Analysis::new(document, context);
    let at = a.doc.from16(offset as usize);
    let Some(e) = a.elements.iter().find(|e| e.start <= at && at <= e.end) else { return Vec::new() };
    let body = a.doc.sub(e.body_start, e.body_end);
    let conversion = |title: &str, text: String| Conversion { title: title.into(), start: a.doc.to16(e.start), end: a.doc.to16(e.end), text };
    if e.math && a.active(e) {
        return from_math(&body).map(|code| conversion("Convert to solveng Code", format!("```{TAG}\n{code}\n```"))).into_iter().collect();
    }
    if !e.math && a.shown(e) {
        let mut lines = Vec::new();
        for raw in body.split('\n') {
            let comment = after(raw, "//", "").trim();
            let line = before(raw, "//").trim();
            let math = if line.is_empty() { Some(String::new()) } else { to_math(line).map(|m| format!("${m}$")) };
            let Some(math) = math else { return Vec::new() };
            let comment = if comment.is_empty() { String::new() } else { format!(" // {comment}") };
            if !math.is_empty() || !comment.is_empty() {
                lines.push(format!("{math}{comment}").trim().to_string());
            }
        }
        if !lines.is_empty() {
            return vec![conversion("Convert to Math", lines.join("\n\n"))];
        }
    }
    Vec::new()
}

/// A unit as math writes it: a name of solveng's (`mA`, `kΩ`, a single letter), else `unit("…")`.
fn math_unit(unit: &str) -> String {
    let u = spell_unit(unit.trim());
    let named = u.chars().count() == 1 || (is_unit(&u) && u.chars().all(|c| is_letter(c) || "°%‰µΩ".contains(c)));
    if named { u } else { format!("unit(\"{u}\")") }
}

/// A clause as math writes it: keywords as they are, units as names.
fn math_clause(tail: &str) -> String {
    split_top(tail)
        .iter()
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            let word: String = p.chars().take_while(|c| is_letter(*c)).collect();
            if is_keyword(&word) {
                return p.to_string();
            }
            // a letter alone after = is a variable in math; after a 1 it's the unit
            let u = math_unit(p.trim_start_matches('[').trim_end_matches(']'));
            if u.chars().count() == 1 { format!("1 {u}") } else { u }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

const MATH_FUNCTIONS: &[&str] = &["sqrt", "exp", "ln", "log", "sin", "cos", "tan", "abs"];
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(" {2,}").unwrap());

/// A line of solveng code as math: `I := 30 mA` → `I := 30 mA`, `x = A [A]` → `x = A unit("A")`,
/// `t := Q0/I = [h]` → `t := Q0/I = 1 h`; None for what math can't say the same way.
pub fn to_math(code: &str) -> Option<String> {
    if code.contains('@') || code.contains("//") || code.contains('$') {
        return None;
    }
    let c = chars(code);
    let (head, clause) = match marker(&c, &|_| false) {
        Some((s, e, arrow)) => (sub(&c, 0, s), Some((if arrow { "->" } else { "=" }, sub(&c, e, c.len())))),
        None => (code.to_string(), None),
    };
    let c = chars(&head);
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
                        // a unit after a number is read as one; elsewhere ([A] on its own) it has to say so
                        let number = c[..i].iter().rev().find(|ch| !ch.is_whitespace() && **ch != '[').is_some_and(|ch| is_digit(*ch));
                        let u = math_unit(&word);
                        out.push_str(&if number || u.starts_with("unit(") { u } else { format!("unit(\"{}\")", spell_unit(&word)) });
                    }
                }
                K::FUNCTION => {
                    if !MATH_FUNCTIONS.contains(&word.as_str()) && word.chars().count() > 1 {
                        return None;
                    }
                    out.push_str(&word);
                }
                K::KEYWORD if word == "as" => out.push_str("\"as\""),
                // a system: equations separated by commas
                K::OPERATOR if word == ";" => out.push(','),
                K::VARIABLE | K::KEYWORD | K::PARAMETER => out.push_str(&math_name(&word)),
                _ => out.push_str(&word),
            }
            i += len;
            continue;
        }
        match c[i] {
            '[' | ']' => {}
            '*' => out.push_str(" dot "),
            ';' => out.push(','),
            ch => out.push(ch),
        }
        i += 1;
    }
    let mut out = SPACES.replace_all(&out, " ").trim().to_string();
    if let Some((m, tail)) = clause {
        let tail = math_clause(&tail);
        out = format!("{out} {m}") + &if tail.is_empty() { String::new() } else { format!(" {tail}") };
    }
    Some(out)
}

/// A unit as solveng spells it: `k""Omega` → `kΩ`, `uA` → `µA`.
fn spell_unit(unit: &str) -> String {
    let mut u = unit.replace("\"\"", "");
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

/// An equation's body as a line of solveng code: `I := 30 mA` → `I := 30 mA`, `t = 1 h, digits(2)` →
/// `t = [h], digits(2)`; None for what isn't a calculation.
pub fn from_math(math: &str) -> Option<String> {
    // derivations (=>, ⇒) and alignment aren't calculations
    if math.chars().any(|c| "#&\\$⇒".contains(c)) || math.contains("=>") || !math.contains('=') && !math.contains("->") && !math.contains('≔') {
        return None;
    }
    let all = chars(math);
    let (head, clause) = match marker(&all, &|_| false) {
        Some((s, e, arrow)) => (sub(&all, 0, s), Some((if arrow { "->" } else { "=" }, sub(&all, e, all.len())))),
        None => (math.to_string(), None),
    };
    let mut out = code_of(&head)?;
    if let Some((m, tail)) = clause {
        let items: Vec<String> = split_top(&tail)
            .iter()
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(|p| {
                let word: String = p.chars().take_while(|c| is_letter(*c)).collect();
                if is_keyword(&word) {
                    p.to_string()
                } else {
                    let p = p.strip_prefix('1').filter(|r| r.starts_with(char::is_whitespace)).map_or(p, str::trim_start);
                    let u = p.strip_prefix("unit(").and_then(|r| r.strip_suffix(')')).unwrap_or(p).trim_matches('"');
                    format!("[{}]", u.replace(" dot ", " "))
                }
            })
            .collect();
        out = format!("{out} {m}") + &if items.is_empty() { String::new() } else { format!(" {}", items.join(", ")) };
    }
    Some(out)
}

/// Math before a clause as solveng code: units after a value bare, elsewhere in brackets; quoted text is a name.
fn code_of(math: &str) -> Option<String> {
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
    let unit = |out: &mut String, u: &str, after_value: bool| {
        if after_value { out.push_str(u) } else { out.push_str(&format!("[{u}]")) }
    };
    let mut i = 0;
    while i < n {
        let c = m[i];
        if solveng_names::numeric_start(&m, i) {
            let name = solveng_names::scan(&m, i).ok()??;
            out.push_str(&solveng_names::code(&name.value));
            i = name.end;
            after_value = false;
        } else if c == '"' {
            let end = (i + 1..n).find(|&j| m[j] == '"')?;
            let text = sub(&m, i + 1, end);
            i = end + 1;
            if out.ends_with('_') || !after_value || !is_unit(&text) {
                // a name of several letters, or a subscript
                out.push_str(&text);
                after_value = false;
            } else {
                out.push_str(&text);
            }
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
            } else if w == "unit" && m.get(i) == Some(&'(') {
                let close = matching(&m, i)?;
                let u = sub(&m, i + 1, close).trim().trim_matches('"').to_string();
                unit(&mut out, &u, after_value);
                after_value = true;
                i = close + 1;
            } else if w == "frac" && i < n && m[i] == '(' {
                let close = matching(&m, i)?;
                let parts = split_top(&sub(&m, i + 1, close));
                if parts.len() != 2 {
                    return None;
                }
                out.push_str(&format!("({})/({})", code_of(parts[0].trim())?, code_of(parts[1].trim())?));
                i = close + 1;
                after_value = false;
            } else if (w.chars().count() > 1 && is_unit(&w)) || (after_value && is_unit(&w)) {
                // solveng's names of units, and a letter after a number
                unit(&mut out, &w, after_value);
                after_value = true;
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
        } else if c == '≔' {
            out.push_str(":=");
            after_value = false;
            i += 1;
        } else if c == ',' {
            // a system: equations separated by commas
            out.push(';');
            after_value = false;
            i += 1;
        } else if c == ' ' || c == '\n' || c == '\t' {
            if !out.ends_with(' ') {
                out.push(' ');
            }
            i += 1;
        } else if c == '/' || c == '^' || c == 'Ω' {
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
