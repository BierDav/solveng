//! The clause after the `=` or `->` that shows a result, as in Mathcad: `t = h, digits(2)`, `x^2 = 4 -> solve(x)`.
//! A trailing `=` is followed by nothing, a unit or keywords; `->` by keywords (or nothing).

use crate::text::{is_letter, sub};

/// The keywords, with what they do.
pub(crate) const KEYWORDS: &[(&str, &str)] = &[
    ("solve", "solve(x, …): solves the equation for these names and defines them"),
    ("simplify", "the symbolic result, with the known values put in"),
    ("frac", "exact fractions"),
    ("digits", "digits(n): decimals with n significant digits"),
    ("sci", "scientific notation, 2.2 · 10⁻⁶ F"),
    ("eng", "engineering notation, exponents in steps of 3"),
];

pub(crate) fn is_keyword(word: &str) -> bool {
    KEYWORDS.iter().any(|(k, _)| *k == word)
}

/// The marker of a line of code or an equation's body: the last `->` (or `→`) at the top level, else the last `=`
/// at the top level that only a unit and keywords follow. Its range in characters, and whether it's an arrow.
pub(crate) fn marker(text: &[char], unit: &dyn Fn(&str) -> bool) -> Option<(usize, usize, bool)> {
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
        return clause_like(&sub(text, e, n), true, unit).then_some((s, e, true));
    }
    let (s, e) = equals?;
    clause_like(&sub(text, e, n), false, unit).then_some((s, e, false))
}

/// Whether text after a marker is a clause: parts split at top-level commas, each a keyword (`digits(3)`), a unit
/// (`mA`, `[h]`, `m/s^2`, `unit("bar")`) or, after an arrow, anything that calls `solve`. A number makes it a
/// step: `= 2 A`.
pub(crate) fn clause_like(tail: &str, arrow: bool, unit: &dyn Fn(&str) -> bool) -> bool {
    let parts = split_top(tail);
    if parts.len() == 1 && parts[0].trim().is_empty() {
        return true;
    }
    parts.iter().all(|p| {
        let p = p.trim();
        let word: String = p.chars().take_while(|c| is_letter(*c)).collect();
        if is_keyword(&word) && p[word.len()..].trim().chars().next().is_none_or(|c| c == '(') {
            return true;
        }
        if arrow && p.is_empty() {
            return false;
        }
        unit_like(p, unit)
    })
}

/// A unit as written in a clause: unit words, powers, `/`, `*`, brackets, `unit("…")`; no numbers but exponents.
fn unit_like(p: &str, unit: &dyn Fn(&str) -> bool) -> bool {
    if p.is_empty() {
        return false;
    }
    let mut s = p.to_string();
    while let Some(i) = s.find("unit(") {
        let end = s[i..].find(')').map_or(s.len(), |j| i + j + 1);
        s.replace_range(i..end, " ");
    }
    let c: Vec<char> = s.chars().collect();
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
        if is_letter(ch) || "°%‰µΩ".contains(ch) {
            let start = i;
            while i < c.len() && (is_letter(c[i]) || c[i].is_ascii_digit() || "°%‰µΩ_".contains(c[i])) {
                i += 1;
            }
            let w: String = c[start..i].iter().collect();
            if w != "dot" && !crate::is_unit(&w) && !unit(&w) {
                return false;
            }
            words += 1;
            continue;
        }
        if ch.is_ascii_digit() || !" []\"*/·⋅()".contains(ch) {
            return false;
        }
        i += 1;
    }
    words > 0 || p.contains("unit(")
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
pub(crate) fn solved(tail: &str) -> Vec<String> {
    split_top(tail)
        .iter()
        .filter_map(|p| p.trim().strip_prefix("solve").map(str::trim).and_then(|r| r.strip_prefix('(')).map(|r| r.trim_end().trim_end_matches(')').to_string()))
        .flat_map(|args| split_top(&args).into_iter().map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect::<Vec<_>>())
        .collect()
}

/// Whether the caret, at the end of `line`, is in a clause: after a top-level `->`, or after a top-level `=` that
/// only words, brackets and commas follow. `Some(true)` inside `solve(…)`, where names go.
pub(crate) fn at_clause(line: &str) -> Option<bool> {
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
        } else if depth == 0 && (ch == '→' || (ch == '>' && i > 0 && c[i - 1] == '-')) {
            found = Some((i + 1, true));
        } else if depth == 0 && ch == '=' && !":<>!=".contains(if i > 0 { c[i - 1] } else { ' ' }) && c.get(i + 1) != Some(&'=') {
            found = Some((i + 1, false));
        }
    }
    let (from, arrow) = found?;
    let tail: String = c[from..].iter().collect();
    // a number makes it a step (= 2 A); digits(3) and exponents (m^2) hold numbers that aren't values
    let mut plain = tail.clone();
    while let Some(i) = plain.find("digits(") {
        let end = plain[i..].find(')').map_or(plain.len(), |j| i + j + 1);
        plain.replace_range(i..end, " ");
    }
    let t: Vec<char> = plain.chars().collect();
    let value = (0..t.len()).any(|i| t[i].is_ascii_digit() && !t[..i].iter().rev().find(|ch| !ch.is_ascii_digit() && !"(-− ".contains(**ch)).is_some_and(|ch| *ch == '^'));
    if !arrow && value {
        return None;
    }
    let open = tail.rfind("solve(");
    Some(open.is_some_and(|o| !tail[o..].contains(')')))
}
