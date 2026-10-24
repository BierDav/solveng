//! Highlighting of the math solveng reads.

use crate::TokenKind as K;
use crate::text::{is_digit, is_letter, is_letter_or_digit, starts_with, sub};

/// A token in characters: start, length, kind.
pub(crate) type Tok = (usize, usize, u8);

pub(crate) const CONSTANTS: &[(&str, &str)] = &[("infinity", "∞ · infinite upper bound")];
pub(crate) fn name_end(text: &[char], at: usize) -> usize {
    solveng_names::scan(text, at).ok().flatten().map_or_else(|| {
        let mut end = at;
        while let Some(&c) = text.get(end) {
            if c == '"' { end = solveng_names::quoted(text, end).map_or(text.len(), |(_, end)| end); }
            else if solveng_names::is_continue(c) { end += 1; }
            else { break; }
        }
        end.max(at + 1)
    }, |n| n.end)
}

fn subscript_at(text: &[char], start: usize, end: usize) -> Option<usize> {
    let mut i = start;
    while i < end {
        if text[i] == '"' { i = solveng_names::quoted(text, i).ok()?.1; }
        else if text[i] == '_' { return Some(i); }
        else { i += 1; }
    }
    None
}

/// Where a name in math that's called ends, with its subscripts (`m_2(`, `R_"load"(`, `x_(a b)(`): just before the
/// `(`; None when the whole name has no subscript or isn't called, which the name's own token tells.
fn called_subscripted(math: &[char], start: usize) -> Option<usize> {
    let mut j = name_end(math, start);
    // a name's scan takes the _ of a grouped subscript, v_(a b)
    if j > start && math[j - 1] == '_' { j -= 1; }
    while math.get(j) == Some(&'_') {
        j = match math.get(j + 1) {
            Some('(') => {
                let mut depth = 0;
                let close = (j + 1..math.len()).find(|&k| {
                    depth += match math[k] { '(' => 1, ')' => -1, _ => 0 };
                    depth == 0
                })?;
                close + 1
            }
            Some('"') => solveng_names::quoted(math, j + 1).ok()?.1,
            Some(&c) if is_letter_or_digit(c) => name_end(math, j + 1),
            _ => return None,
        };
    }
    let subscripted = sub(math, start, j).contains('_');
    (subscripted && math.get(j) == Some(&'(')).then_some(j)
}

/// The tokens of a clause, `start..end` after its marker: keywords, the names `solve` and `substitute` take, units.
/// `unit` says which words are units there; quoted text is a name.
fn clause_tokens(text: &[char], start: usize, end: usize, unit: &dyn Fn(&str) -> bool) -> Vec<Tok> {
    let mut out = Vec::new();
    let (mut i, mut solve, mut in_solve) = (start, false, false);
    while i < end {
        let c = text[i];
        if c == '"' {
            let close = (i + 1..end).find(|&j| text[j] == '"').map_or(end, |j| j + 1);
            out.push((i, close - i, K::VARIABLE));
            i = close;
        } else if solveng_names::is_start(c) || solveng_names::numeric_start(text, i) {
            let j = name_end(text, i).min(end);
            let word = sub(text, i, j);
            let kind = if crate::clause::is_keyword(&word) {
                solve = crate::clause::NAMES.contains(&word.as_str());
                K::KEYWORD
            } else if in_solve {
                K::VARIABLE
            } else if word == "dot" {
                K::OPERATOR
            } else if unit(&word) {
                K::UNIT
            } else {
                K::VARIABLE
            };
            out.push((i, j - i, kind));
            i = j;
        } else if is_digit(c) {
            let mut j = i;
            while j < end && is_digit(text[j]) {
                j += 1;
            }
            let exponent = text[start..i].iter().rev().find(|ch| !"(-− ".contains(**ch)) == Some(&'^');
            out.push((i, j - i, if exponent { K::UNIT } else { K::NUMBER }));
            i = j;
        } else {
            if c == '(' && solve {
                in_solve = true;
            } else if c == ')' {
                (solve, in_solve) = (false, false);
            }
            i += 1;
        }
    }
    out
}

/// Whether the `:` at `colon` makes a named argument, `convolve(x, h, step: 10 ms)`, rather than a parameter's type
/// or default in a function's head, `f(I: mA) := …`, `P(U: V) = U^2/R`: it's in a call's parentheses, and the
/// call isn't a head (what := or a curve's = follow at the start of a statement).
pub(crate) fn named_argument(text: &[char], colon: usize) -> bool {
    let mut depth = 0;
    let Some(open) = (0..colon).rev().find(|&j| {
        depth += match text[j] { ')' | ']' => 1, '(' | '[' => -1, _ => 0 };
        depth < 0
    }) else { return false };
    if text[open] != '(' || !text[..open].iter().rev().find(|c| !c.is_whitespace()).is_some_and(|&c| is_letter_or_digit(c) || c == '"' || c == '_') {
        return false;
    }
    let mut depth = 0;
    let Some(close) = (open..text.len()).find(|&j| {
        depth += match text[j] { '(' | '[' => 1, ')' | ']' => -1, _ => 0 };
        depth == 0
    }) else { return true };
    let rest: String = text[close + 1..].iter().collect::<String>();
    let rest = rest.trim_start();
    let head = rest.starts_with(":=") || rest.starts_with('≔') || rest.starts_with(':')
        || (rest.starts_with('=') && !rest.starts_with("==") && !rest[1..].split(['\n', ';']).next().unwrap_or("").trim().is_empty());
    // a head starts its statement: only a name before the parentheses
    let before: String = text[..open].iter().collect();
    let statement = before.rsplit(['\n', ';', ',']).next().unwrap_or("").trim().to_string();
    let starts = solveng_names::scan(&statement.chars().collect::<Vec<_>>(), 0).ok().flatten().is_some_and(|n| n.end == statement.chars().count());
    !(head && starts)
}

/// Re-kinds the names before named arguments' colons as parameters: step in `convolve(x, h, step: …)`.
fn mark_named(out: &mut [Tok], text: &[char], colons: &[usize]) {
    for &colon in colons {
        let mut end = colon;
        while end > 0 && text[end - 1].is_whitespace() { end -= 1; }
        if let Some(t) = out.iter_mut().rev().find(|t| t.0 + t.1 == end) {
            t.2 = K::PARAMETER;
        }
    }
}

/// Words math writes for operators.
const MATH_OPERATORS: &[&str] = &["dot", "times", "div", "approx", "plus", "minus", "eq", "lt", "gt", "arrow", "dif", "integral", "sum"];

/// Tokens of one equation's body: numbers, variables (quoted text too), functions, operators, the clause after the
/// `=` or `->` showing a result. `unit` says which words are units: Typst bindings of units (see `bindings`), which
/// also colour them where they're used; a bare single letter is a math symbol, a variable.
pub(crate) fn math_tokens(math: &[char], unit: &dyn Fn(&str) -> bool) -> Vec<Tok> {
    let spans = crate::text::equation_ranges(math);
    if spans.len() > 1 {
        return spans.into_iter().flat_map(|(s, e)| math_tokens(&math[s..e], unit).into_iter().map(move |(a, l, k)| (s + a, l, k))).collect();
    }
    let mut out = Vec::new();
    let mut emit = |start: usize, end: usize, kind: u8| {
        if end > start {
            out.push((start, end - start, kind));
        }
    };
    let n = math.len();
    let mut i = 0;
    let (mut unit_mode, mut subscript) = (false, false);
    let mut named = Vec::new();
    let marker = crate::clause::marker(math, unit);
    while i < n {
        let c = math[i];
        let sub_ = subscript;
        subscript = false;
        if let Some((s, e, _)) = marker.filter(|m| m.0 == i) {
            // the = or -> showing a result, then its clause
            emit(s, e, K::EVALUATE);
            for (ts, tl, k) in clause_tokens(math, e, n, unit) {
                emit(ts, ts + tl, k);
            }
            i = n;
        } else if c == '\\' {
            i += 2;
        } else if c == '#' {
            // Embedded code is Typst's.
            let mut j = i + 1;
            while j < n && (is_letter_or_digit(math[j]) || "_-.".contains(math[j])) {
                j += 1;
            }
            if j < n && math[j] == '(' {
                let mut depth = 0;
                while j < n {
                    if math[j] == '(' {
                        depth += 1;
                    } else if math[j] == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                j += 1;
            }
            unit_mode = false;
            i = j;
        } else if let Some(end) = called_subscripted(math, i).filter(|_| !sub_ && (c == '"' || solveng_names::is_start(c))) {
            // a function with a subscript, called: m_2(x), R_"load"(t), as one name
            let word = crate::resolved_name(&sub(math, i, end));
            let operator = MATH_OPERATORS.iter().any(|o| word.split('_').next() == Some(*o));
            emit(i, end, if operator { K::OPERATOR } else { K::FUNCTION });
            unit_mode = false;
            i = end;
        } else if c == '"' {
            let end = name_end(math, i);
            let end = subscript_at(math, i, end).unwrap_or(end);
            let call = math[end..].iter().find(|c| !c.is_whitespace()) == Some(&'(');
            emit(i, end, if call { K::FUNCTION } else { K::VARIABLE });
            if !sub_ { unit_mode = false; }
            i = end;
        } else if c == '_' && math.get(i + 1) == Some(&'(') {
            // A sum's lower attachment contains an index declaration and a bound,
            // rather than a subscript that belongs to a variable name.
            let prefix = sub(math, 0, i);
            if prefix.trim_end().strip_suffix("sum").is_some_and(|before| before.chars().last().is_none_or(|c| !solveng_names::is_continue(c))) {
                let mut depth = 1;
                let mut end = i + 2;
                while end < n {
                    if math[end] == '(' { depth += 1; }
                    if math[end] == ')' { depth -= 1; }
                    if depth == 0 { break; }
                    end += 1;
                }
                for (s, l, k) in math_tokens(&math[i + 2..end], unit) { emit(i + 2 + s, i + 2 + s + l, k); }
                i = (end + 1).min(n);
                continue;
            }
            // A subscript in parentheses belongs to the name: x_("in" Delta).
            let close = (i + 2..n).find(|&j| math[j] == ')' || math[j] == '(').filter(|&j| math[j] == ')');
            let Some(close) = close else {
                i += 1;
                continue;
            };
            let mut j = i + 2;
            while j < close {
                if math[j].is_whitespace() {
                    j += 1;
                    continue;
                }
                let end = if math[j] == '"' {
                    (j + 1..close).find(|&k| math[k] == '"').map_or(close, |k| k + 1)
                } else {
                    (j..close).find(|&k| math[k].is_whitespace() || math[k] == '"').unwrap_or(close)
                };
                emit(j, end, K::VARIABLE);
                j = end;
            }
            i = close + 1;
        } else if c == '_' {
            subscript = true;
            i += 1;
        } else if solveng_names::numeric_start(math, i) {
            let end = name_end(math, i);
            let call = math[end..].iter().find(|c| !c.is_whitespace()) == Some(&'(');
            emit(i, end, if call { K::FUNCTION } else { K::VARIABLE });
            unit_mode = false;
            i = end;
        } else if is_digit(c) {
            let mut j = i;
            while j < n && (is_digit(math[j]) || (".,".contains(math[j]) && j + 1 < n && is_digit(math[j + 1]))) {
                j += 1;
            }
            let exponent = i > 0 && math[i - 1] == '^';
            emit(i, j, if sub_ { K::VARIABLE } else if exponent && unit_mode { K::UNIT } else { K::NUMBER });
            if !exponent && !sub_ {
                unit_mode = true;
            }
            i = j;
        } else if solveng_names::is_start(c) {
            let mut j = name_end(math, i);
            if let Some(subscript) = subscript_at(math, i, j) { j = subscript; }
            while j + 1 < n && math[j] == '.' && is_letter(math[j + 1]) { j = name_end(math, j + 1); }
            let word = crate::resolved_name(&sub(math, i, j));
            let call = j < n && math[j] == '(';
            let base = word.split('.').next().unwrap_or("");
            let kind = if sub_ {
                K::VARIABLE
            } else if call && matches!(word.as_str(), "unit" | "op") {
                K::KEYWORD
            } else if call {
                K::FUNCTION
            } else if MATH_OPERATORS.contains(&base) {
                K::OPERATOR
            } else if CONSTANTS.iter().any(|(name, _)| crate::resolved_name(name) == word) {
                K::KEYWORD
            } else {
                K::VARIABLE
            };
            emit(i, j, kind);
            if kind != K::UNIT && !sub_ {
                unit_mode = false;
            }
            i = j;
        } else if starts_with(math, i, ":=") || c == '≔' {
            // := assigns
            let end = if c == '≔' { i + 1 } else { i + 2 };
            emit(i, end, K::OPERATOR);
            unit_mode = false;
            i = end;
        } else if c == ':' && named_argument(math, i) {
            // step: 10 ms, a named argument
            named.push(i);
            unit_mode = false;
            i += 1;
        } else if c == ':' {
            // A parameter's unit: q(x: #A)
            unit_mode = true;
            i += 1;
        } else if "=<>≤≥≠".contains(c) || (c == '!' && starts_with(math, i, "!=")) {
            let end = if i + 1 < n && math[i + 1] == '=' { i + 2 } else { i + 1 };
            emit(i, end, K::OPERATOR);
            unit_mode = false;
            i = end;
        } else if "/^ \t\n".contains(c) {
            i += 1;
        } else {
            unit_mode = false;
            i += 1;
        }
    }
    mark_named(&mut out, math, &named);
    out
}
