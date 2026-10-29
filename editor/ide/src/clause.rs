//! The clause after the `=` or `->` that shows a result, as in Mathcad: `t = h, digits(2)`, `x^2 = 4 -> solve(x)`.
//! A trailing `=` is followed by nothing, a unit or modifiers; `->` by modifiers (or nothing). Modifiers are Typst
//! bindings of solveng's `modifiers` module (`#import modifiers: solve`), so a word is one by the binding in scope:
//! `#import modifiers: solve as löse` makes `löse` solve.

use crate::text::{is_letter, sub};

/// The modifiers, by their name in the `modifiers` module, with what they do.
pub(crate) const KEYWORDS: &[(&str, &str)] = &[
    ("assume", "assume(a in RR, x > 0, …): local real and sign assumptions for this symbolic calculation"),
    ("solve", "solve(x, …): solves the equation for these names and defines them"),
    ("substitute", "substitute(x, #c, …): the result with these variables' formulas and constants' expressions put in, other names kept"),
    ("simplify", "the symbolic result, with the known values put in; of an equation, where it holds"),
    ("expand", "the symbolic result multiplied out, (x + 1)² → x² + 2x + 1"),
    ("factor", "the symbolic result in factors, x³ − x → x (x − 1)(x + 1)"),
    ("steps", "the way to the symbolic result, a line each: one side zero, in factors, where it holds"),
    ("fraction", "exact fractions"),
    ("digits", "digits(n): decimals with n significant digits"),
    ("sci", "scientific notation, 2.2 · 10⁻⁶ F"),
    ("eng", "engineering notation, exponents in steps of 3"),
];

/// The modifiers whose arguments are names of the worksheet.
pub(crate) const NAMES: &[&str] = &["solve", "substitute"];

/// Whether a modifier takes arguments.
pub(crate) fn called(name: &str) -> bool {
    NAMES.contains(&name) || name == "digits" || name == "assume"
}

/// A word of a clause that is a modifier: its name in the `modifiers` module, and whether a binding in scope makes
/// it one. An unbound word spelled as one of the module's names counts for the clause's shape (Typst can't read it
/// unimported anyway, and editors offer the import), but isn't one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Modifier {
    pub name: String,
    pub bound: bool,
}

/// What clause words are modifiers, by a binding in scope or by spelling.
pub(crate) type Modifiers<'a> = &'a dyn Fn(&str) -> Option<Modifier>;

/// A word as one of the module's names, where no binding says otherwise.
pub(crate) fn spelled(word: &str) -> Option<Modifier> {
    KEYWORDS.iter().any(|(k, _)| *k == word).then(|| Modifier { name: word.to_string(), bound: false })
}

/// The modifiers called in a clause's text, in order: (byte offset of the word, byte offset after its `(`, the
/// modifier).
fn calls(tail: &str, modifier: Modifiers) -> Vec<(usize, usize, Modifier)> {
    let mut out = Vec::new();
    let mut chars = tail.char_indices().peekable();
    let mut previous: Option<char> = None;
    while let Some((i, c)) = chars.next() {
        if is_letter(c) && !previous.is_some_and(|p| is_letter(p) || p.is_ascii_digit() || p == '_' || p == '"') {
            let mut end = i + c.len_utf8();
            while let Some(&(j, d)) = chars.peek() {
                if !(is_letter(d) || d.is_ascii_digit()) { break; }
                end = j + d.len_utf8();
                chars.next();
            }
            let rest = &tail[end..];
            let open = rest.len() - rest.trim_start().len();
            if rest[open..].starts_with('(')
                && let Some(m) = modifier(&tail[i..end]) {
                    out.push((i, end + open + 1, m));
                }
            previous = tail[..end].chars().last();
            continue;
        }
        previous = Some(c);
    }
    out
}

/// The marker of an equation's body: the last `->` (or `→`) at the top level, else the last `=` at the top level
/// that only a unit and modifiers follow. Its range in characters, and whether it's an arrow. `unit` says which words
/// are units: the unit bindings in scope; `modifier` which are modifiers.
pub(crate) fn marker(text: &[char], unit: &dyn Fn(&str) -> bool, modifier: Modifiers) -> Option<(usize, usize, bool)> {
    let clean = crate::text::without_math_comments(text);
    let text = clean.as_slice();
    let n = text.len();
    let (mut depth, mut quoted) = (0i32, false);
    let (mut arrow, mut equals) = (None, None);
    let mut i = 0;
    while i < n {
        let c = text[i];
        if c == '"' {
            quoted = !quoted;
        } else if quoted {
        } else if "([{".contains(c) {
            depth += 1;
        } else if ")]}".contains(c) {
            depth -= 1;
        } else if depth == 0 && c == '=' && text.get(i + 1) == Some(&'>') {
            equals = Some((i, i + 2));
            i += 1;
        } else if depth == 0 && c == '⇒' {
            equals = Some((i, i + 1));
        } else if depth == 0 && c == '-' && text.get(i + 1) == Some(&'>') {
            arrow = Some((i, i + 2));
            i += 1;
        } else if depth == 0 && c == '→' {
            arrow = Some((i, i + 1));
        } else if depth == 0 && c == '=' {
            let before = if i > 0 { text[i - 1] } else { ' ' };
            let after = text.get(i + 1).copied().unwrap_or(' ');
            if !":<>!=".contains(before) && after != '=' {
                equals = Some((i, i + 1));
            }
        }
        i += 1;
    }
    if let Some((s, e)) = arrow {
        // Only a clause follows an arrow: its words are units, known or not.
        return clause_like(&sub(text, e, n), true, &|_| true, modifier).then_some((s, e, true));
    }
    let (s, e) = equals?;
    clause_like(&sub(text, e, n), false, unit, modifier).then_some((s, e, false))
}

/// Whether text after a marker is a clause: parts split at top-level commas, each a modifier (`digits(3)`) or a unit
/// (`mA`, `#h`, `#m / #s^2`). A number makes it a step: `= 2 A`.
pub(crate) fn clause_like(tail: &str, arrow: bool, unit: &dyn Fn(&str) -> bool, modifier: Modifiers) -> bool {
    let parts = split_top(tail);
    if parts.len() == 1 && parts[0].trim().is_empty() {
        return true;
    }
    parts.iter().all(|p| {
        let p = p.trim();
        let word: String = p.chars().take_while(|c| is_letter(*c)).collect();
        if modifier(&word).is_some() && p[word.len()..].trim().chars().next().is_none_or(|c| c == '(') {
            return true;
        }
        if arrow && p.is_empty() {
            return false;
        }
        unit_like(p, unit, arrow)
    })
}

/// A unit as written in a clause: unit bindings (`mA`, `#h`, a single letter needs its `#`), powers, `/`, `*`,
/// `dot` and parentheses; no numbers but exponents and a leading 1. The words of solveng's `units` module count
/// where no binding says otherwise, as Typst can't read them unimported anyway. After an arrow nothing else can
/// follow, so its words are units.
fn unit_like(p: &str, unit: &dyn Fn(&str) -> bool, arrow: bool) -> bool {
    if p.is_empty() {
        return false;
    }
    let one = p.strip_prefix('1').filter(|r| r.starts_with(|c: char| c.is_whitespace() || is_letter(c) || c == '#'));
    let p = one.unwrap_or(p);
    let c: Vec<char> = p.chars().collect();
    let mut i = 0;
    let mut words = 0;
    while i < c.len() {
        let ch = c[i];
        if ch == '^' {
            // an exponent: ^2, ^-1, ^(2)
            i += 1;
            while i < c.len() && "(-−)0123456789 ".contains(c[i]) {
                i += 1;
            }
            continue;
        }
        let hash = ch == '#' && c.get(i + 1).is_some_and(|&n| is_letter(n));
        if is_letter(ch) || hash {
            let start = i + usize::from(hash);
            i = start;
            while i < c.len() && (is_letter(c[i]) || c[i].is_ascii_digit() || c[i] == '_') {
                i += 1;
            }
            let w: String = c[start..i].iter().collect();
            // A bare single letter is a math symbol: `t = A` compares.
            if crate::resolved_name(&w).chars().count() == 1 && !hash && !arrow { return false; }
            if w != "dot" && !crate::is_unit(&w) && !unit(&w) {
                return false;
            }
            words += usize::from(w != "dot");
            continue;
        }
        if ch.is_ascii_digit() || !" */·⋅()".contains(ch) {
            return false;
        }
        i += 1;
    }
    words > 0
}

/// `a, f(b, c), d` split at the top level.
pub(crate) fn split_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut depth, mut cur, mut quoted) = (0i32, String::new(), false);
    for ch in s.chars() {
        match ch {
            '"' => quoted = !quoted,
            '(' | '[' if !quoted => depth += 1,
            ')' | ']' if !quoted => depth -= 1,
            ',' if depth == 0 && !quoted => {
                out.push(std::mem::take(&mut cur));
                continue;
            }
            _ => {}
        }
        cur.push(ch);
    }
    out.push(cur);
    out
}

/// The names a clause solves for: `x` and `y` of `-> solve(x, y)`, as written.
pub(crate) fn solved(tail: &str, modifier: Modifiers) -> Vec<String> {
    split_top(tail)
        .iter()
        .filter_map(|p| {
            let p = p.trim();
            let (start, open, m) = calls(p, modifier).into_iter().next()?;
            (start == 0 && m.name == "solve").then(|| p[open..].trim_end().trim_end_matches(')').to_string())
        })
        .flat_map(|args| split_top(&args).into_iter().map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect::<Vec<_>>())
        .collect()
}

/// Inside `solve(…)` or `substitute(…)` of a clause the caret ends `line` in: where the marker starts (in
/// characters of `line`), the modifier, and the names listed before the one being typed, as written.
pub(crate) fn names_call(line: &str, modifier: Modifiers) -> Option<(usize, String, Vec<String>)> {
    at_clause(line, modifier).filter(|(names, ..)| *names)?;
    let c: Vec<char> = line.chars().collect();
    let (mut depth, mut quoted, mut marker) = (0i32, false, None);
    for (i, &ch) in c.iter().enumerate() {
        if ch == '"' {
            quoted = !quoted;
        } else if quoted {
        } else if "([{".contains(ch) {
            depth += 1;
        } else if ")]}".contains(ch) {
            depth -= 1;
        } else if depth == 0 && matches!(ch, '→' | '⇒') {
            marker = Some((i, i + 1));
        } else if depth == 0 && ch == '>' && i > 0 && matches!(c[i - 1], '-' | '=') {
            marker = Some((i - 1, i + 1));
        } else if depth == 0 && ch == '=' && !":<>!=".contains(if i > 0 { c[i - 1] } else { ' ' }) && c.get(i + 1) != Some(&'=') {
            marker = Some((i, i + 1));
        }
    }
    let (start, end) = marker?;
    let tail = sub(&c, end, c.len());
    let (open, keyword) = open_names_call(&tail, modifier)?;
    let mut listed = split_top(&tail[open..]);
    listed.pop();
    Some((start, keyword, listed.into_iter().map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).collect()))
}

/// Whether the caret, at the end of `line`, is in a clause: after a top-level `->`, or after a top-level `=` that
/// only words, brackets and commas follow: (whether inside `solve(…)` or `substitute(…)`, where names go; whether
/// after `->`, where nothing but the clause can follow – after `=` a comparison can; what follows the marker).
pub(crate) fn at_clause(line: &str, modifier: Modifiers) -> Option<(bool, bool, String)> {
    let c: Vec<char> = line.chars().collect();
    let (mut depth, mut quoted, mut found) = (0i32, false, None);
    for (i, &ch) in c.iter().enumerate() {
        if ch == '"' {
            quoted = !quoted;
        } else if quoted {
        } else if "([{".contains(ch) {
            depth += 1;
        } else if ")]}".contains(ch) {
            depth -= 1;
        } else if depth == 0 && (matches!(ch, '→' | '⇒') || (ch == '>' && i > 0 && matches!(c[i - 1], '-' | '='))) {
            found = Some((i + 1, true));
        } else if depth == 0 && ch == '=' && !":<>!=".contains(if i > 0 { c[i - 1] } else { ' ' }) && c.get(i + 1) != Some(&'=') {
            found = Some((i + 1, false));
        }
    }
    let (from, arrow) = found?;
    let tail: String = c[from..].iter().collect();
    // a number makes it a step (= 2 A); digits(3) and exponents (m^2) hold numbers that aren't values
    let mut plain = tail.clone();
    for (i, open, _) in calls(&tail, modifier).into_iter().rev().filter(|c| c.2.name == "digits") {
        let end = tail[open..].find(')').map_or(tail.len(), |j| open + j + 1);
        plain.replace_range(i..end, " ");
    }
    let t: Vec<char> = plain.chars().collect();
    let value = (0..t.len()).any(|i| t[i].is_ascii_digit() && !t[..i].iter().rev().find(|ch| !ch.is_ascii_digit() && !"(-− ".contains(**ch)).is_some_and(|ch| *ch == '^'));
    if !arrow && value {
        return None;
    }
    Some((open_names_call(&tail, modifier).is_some(), arrow, tail))
}

/// The call of `solve` or `substitute` still open at the end of a clause's text: after its `(`, and the modifier.
fn open_names_call(tail: &str, modifier: Modifiers) -> Option<(usize, String)> {
    calls(tail, modifier).into_iter().rev().find(|(_, open, m)| NAMES.contains(&m.name.as_str()) && !tail[*open..].contains(')'))
        .map(|(_, open, m)| (open, m.name))
}
