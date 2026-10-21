//! Highlighting of solveng code and of the math solveng reads.

use crate::TokenKind as K;
use crate::text::{is_digit, is_letter, is_letter_or_digit, starts_with, sub};

/// A token in characters: start, length, kind.
pub(crate) type Tok = (usize, usize, u8);

pub(crate) const CONSTANTS: &[(&str, &str)] = &[("pi", "π ≈ 3,14159"), ("e", "Euler's number ≈ 2,71828")];
const UNIT_CHARS: &str = "°%‰µΩ";
const SUPERSCRIPTS: &str = "²³⁻¹";

/// Tokens of solveng code: numbers and their units, variables, functions, `@`-inlined formulas, relations and
/// `->`, comments. `unit` says whether a word after a number is a unit beyond the built-in ones.
pub(crate) fn code_tokens(code: &[char], unit: &dyn Fn(&str) -> bool) -> Vec<Tok> {
    tokens_of(code, unit, true)
}

/// Tokens of a formula, like a graph's curve: a word after a number is a variable (`12 A` is 12 times A), units
/// are written in brackets (`12 [A]`).
pub(crate) fn formula_tokens(code: &[char], unit: &dyn Fn(&str) -> bool) -> Vec<Tok> {
    tokens_of(code, unit, false)
}

/// `values`: a number may be followed by its unit, as in definitions.
fn tokens_of(code: &[char], unit: &dyn Fn(&str) -> bool, values: bool) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut emit = |start: usize, end: usize, kind: u8| {
        if end > start {
            out.push((start, end - start, kind));
        }
    };
    let name_part = |c: char| is_letter_or_digit(c) || c == '_';
    let n = code.len();
    let mut i = 0;
    // Where units are read: after a number, inside [ ], after ->.
    let (mut unit_mode, mut in_bracket, mut after_arrow) = (false, false, false);
    // A number with no unit after it yet: `5*A` multiplies by a variable, `5 A*s` keeps the unit going.
    let mut bare_number = false;
    // After `as`: the one unit cast to.
    let mut cast = false;
    // In a type after `:`, which a relation ends too: f(x: V): A = …
    let mut typed = false;
    while i < n {
        let c = code[i];
        if c == '\n' || c == ';' {
            if c == ';' {
                emit(i, i + 1, K::OPERATOR);
            }
            unit_mode = false;
            in_bracket = false;
            after_arrow = false;
            i += 1;
        } else if starts_with(code, i, "//") {
            let end = (i..n).find(|&j| code[j] == '\n').unwrap_or(n);
            emit(i, end, K::COMMENT);
            i = end;
        } else if starts_with(code, i, "->") {
            emit(i, i + 2, K::OPERATOR);
            // L := R = 7.2 kΩ -> R: after an assigned statement, a name of it is what's solved for, not a unit
            let line_start = (0..i).rev().find(|&j| code[j] == '\n').map_or(0, |j| j + 1);
            let line: String = code[line_start..i].iter().collect();
            let mut s0 = i + 2;
            while s0 < n && code[s0] == ' ' {
                s0 += 1;
            }
            let mut e0 = s0;
            while e0 < n && name_part(code[e0]) {
                e0 += 1;
            }
            let target = sub(code, s0, e0);
            let mentioned = |t: &str| line.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|w| w == t);
            if e0 > s0 && line.contains(":=") && mentioned(&target) && (e0..n).take_while(|&j| code[j] != '\n').all(|j| code[j].is_whitespace()) {
                emit(s0, e0, K::VARIABLE);
                i = e0;
                continue;
            }
            after_arrow = true;
            unit_mode = true;
            i += 2;
        } else if starts_with(code, i, ":=") || c == '≔' {
            // := assigns
            let end = if c == '≔' { i + 1 } else { i + 2 };
            emit(i, end, K::OPERATOR);
            (unit_mode, typed) = (false, false);
            if !after_arrow {
                in_bracket = false;
            }
            i = end;
        } else if c == '[' || c == ':' {
            // [5 V], and a parameter's unit: f(I: mA)
            typed = c == ':';
            in_bracket = true;
            unit_mode = true;
            i += 1;
        } else if c == ']' || (in_bracket && (c == ',' || c == ')')) {
            in_bracket = false;
            unit_mode = false;
            i += 1;
        } else if "=<>≤≥".contains(c) {
            let end = if i + 1 < n && code[i + 1] == '=' { i + 2 } else { i + 1 };
            emit(i, end, K::OPERATOR);
            unit_mode = false;
            if typed {
                (typed, in_bracket) = (false, false);
            }
            i = end;
        } else if c == '@' && i + 1 < n && is_letter(code[i + 1]) {
            let mut j = i + 1;
            while j < n && name_part(code[j]) {
                j += 1;
            }
            emit(i, j, K::ANNOTATION);
            unit_mode = false;
            i = j;
        } else if is_digit(c) || (c == '.' && i + 1 < n && is_digit(code[i + 1])) {
            let mut j = i;
            while j < n {
                let d = code[j];
                if is_digit(d) {
                    j += 1;
                } else if (d == '.' || d == ',') && j + 1 < n && is_digit(code[j + 1]) {
                    j += 1;
                } else if (d == 'e' || d == 'E')
                    && j + 1 < n
                    && (is_digit(code[j + 1]) || ("+-".contains(code[j + 1]) && j + 2 < n && is_digit(code[j + 2])))
                {
                    j += 2;
                } else {
                    break;
                }
            }
            // An exponent of a unit (m^2) keeps the unit going.
            let exponent = i > 0 && code[i - 1] == '^';
            emit(i, j, if exponent && unit_mode { K::UNIT } else { K::NUMBER });
            if !exponent {
                unit_mode = values || in_bracket || after_arrow;
                bare_number = true;
            }
            i = j;
        } else if (c == '*' || c == '·') && bare_number && !in_bracket && !after_arrow {
            unit_mode = false;
            bare_number = false;
            i += 1;
        } else if is_letter(c) || UNIT_CHARS.contains(c) {
            let after_number = bare_number;
            bare_number = false;
            let mut j = i + 1;
            while j < n && (name_part(code[j]) || UNIT_CHARS.contains(code[j]) || glue(code, j)) {
                j += if code[j] == '"' { 2 } else { 1 };
            }
            let word = sub(code, i, j);
            let mut k = j;
            while k < n && code[k] == ' ' {
                k += 1;
            }
            let call = k < n && code[k] == '(';
            // U as A casts, though as is also a unit (attosecond, [3 as])
            if cast {
                cast = false;
                emit(i, j, K::UNIT);
                unit_mode = false;
                i = j;
                continue;
            }
            // after a number it's the unit, attosecond (3 as)
            if word == "as" && !in_bracket && !after_number {
                emit(i, j, K::KEYWORD);
                cast = true;
                unit_mode = false;
                i = j;
                continue;
            }
            let kind = if call {
                K::FUNCTION
            } else if unit_mode && (in_bracket || after_arrow || crate::is_unit(&word) || unit(&word)) {
                K::UNIT
            } else if CONSTANTS.iter().any(|(c, _)| *c == word) {
                K::KEYWORD
            } else {
                K::VARIABLE
            };
            emit(i, j, kind);
            if kind != K::UNIT {
                unit_mode = in_bracket || after_arrow;
            }
            i = j;
        } else if "/^*· \t".contains(c) || SUPERSCRIPTS.contains(c) {
            if c != ' ' && c != '\t' {
                bare_number = false;
            }
            if SUPERSCRIPTS.contains(c) && unit_mode {
                emit(i, i + 1, K::UNIT);
            }
            i += 1;
        } else {
            // + - ( ) , end a value's unit.
            bare_number = false;
            if !in_bracket && !after_arrow {
                unit_mode = false;
            }
            i += 1;
        }
    }
    out
}

/// An empty string gluing a name to the letters after it: `Delta""r`.
fn glue(text: &[char], at: usize) -> bool {
    at + 2 < text.len() && text[at] == '"' && text[at + 1] == '"' && is_letter(text[at + 2])
}

/// Whether quoted math text `math[start..end]` (quotes included) names a variable rather than a unit: a name
/// that isn't a unit (`"tews"`), or anything with a subscript (`"x"_1`); `"k"Omega` is a prefix.
fn quoted_name(math: &[char], start: usize, end: usize, unit: &dyn Fn(&str) -> bool) -> bool {
    if end < start + 2 || math.get(end - 1) != Some(&'"') {
        return false;
    }
    if math.get(end) == Some(&'_') {
        return true;
    }
    let inner = sub(math, start + 1, end - 1);
    let mut chars = inner.chars();
    if !chars.next().is_some_and(is_letter) || !chars.all(is_letter_or_digit) {
        return false;
    }
    let known = |w: &str| crate::is_unit(w) || unit(w);
    let prefix = |w: &str| crate::units::PREFIXES.iter().any(|(p, _)| *p == w);
    // With a prefix ("mü" for a unit ü), or a prefix before its unit: "k"Omega, "m" "A".
    let first = inner.chars().next().map(String::from).unwrap_or_default();
    let prefixed = prefix(&first) && known(&inner[first.len()..]);
    !(known(&inner) || prefixed || prefix(&inner))
}

/// Words math writes for operators.
const MATH_OPERATORS: &[&str] = &["dot", "times", "div", "approx", "plus", "minus", "eq", "lt", "gt", "arrow"];

/// Tokens of one equation's body, coloured like solveng code: numbers, quoted units, variables, functions.
pub(crate) fn math_tokens(math: &[char], unit: &dyn Fn(&str) -> bool) -> Vec<Tok> {
    let mut out = Vec::new();
    let mut emit = |start: usize, end: usize, kind: u8| {
        if end > start {
            out.push((start, end - start, kind));
        }
    };
    let n = math.len();
    let mut i = 0;
    let (mut unit_mode, mut subscript) = (false, false);
    while i < n {
        let c = math[i];
        let sub_ = subscript;
        subscript = false;
        if c == '\\' {
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
        } else if c == '"' {
            let end = (i + 1..n).find(|&j| math[j] == '"').map_or(n, |j| j + 1);
            // "as" casts: U "as" "mA"
            // after a number it's the unit, attosecond (3 "as")
            if !sub_ && !unit_mode && sub(math, i, end) == "\"as\"" {
                emit(i, end, K::KEYWORD);
                unit_mode = true;
                i = end;
                continue;
            }
            // Quoted text is a unit, except in a subscript (R_"load") and a name no unit spells ("tews", "x"_"in").
            let name = sub_ || quoted_name(math, i, end, unit);
            emit(i, end, if name { K::VARIABLE } else { K::UNIT });
            if !name {
                unit_mode = true;
            } else if !sub_ {
                unit_mode = false;
            }
            i = end;
        } else if c == '_' && math.get(i + 1) == Some(&'(') {
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
        } else if c == '@' && i + 1 < n && is_letter(math[i + 1]) {
            // An inlined formula, with its subscript: @x, @Q_0, @t_"half".
            let mut j = i + 1;
            while j < n && is_letter_or_digit(math[j]) {
                j += 1;
            }
            if j + 1 < n && math[j] == '_' {
                j += 1;
                if math[j] == '"' {
                    j = (j + 1..n).find(|&k| math[k] == '"').map_or(n, |k| k + 1);
                } else {
                    while j < n && is_letter_or_digit(math[j]) {
                        j += 1;
                    }
                }
            }
            emit(i, j, K::ANNOTATION);
            unit_mode = false;
            i = j;
        } else if is_letter(c) {
            let mut j = i + 1;
            while j < n && (is_letter(math[j]) || (math[j] == '.' && j + 1 < n && is_letter(math[j + 1])) || glue(math, j)) {
                j += if math[j] == '"' { 2 } else { 1 };
            }
            let word = sub(math, i, j);
            let call = j < n && math[j] == '(';
            let base = word.split('.').next().unwrap_or("");
            let kind = if sub_ {
                K::VARIABLE
            } else if call {
                K::FUNCTION
            } else if MATH_OPERATORS.contains(&base) {
                K::OPERATOR
            } else if unit_mode && (crate::is_unit(&word) || unit(&word)) {
                K::UNIT
            } else if word == "pi" || word == "e" {
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
        } else if c == ':' {
            // A parameter's unit: q(x: A)
            unit_mode = true;
            i += 1;
        } else if "=<>≤≥≠".contains(c) || (c == '!' && starts_with(math, i, "!=")) {
            let end = if i + 1 < n && math[i + 1] == '=' { i + 2 } else { i + 1 };
            emit(i, end, K::OPERATOR);
            unit_mode = false;
            i = end;
        } else if "/^ \t\nΩ".contains(c) {
            if c == 'Ω' && unit_mode {
                emit(i, i + 1, K::UNIT);
            }
            i += 1;
        } else {
            unit_mode = false;
            i += 1;
        }
    }
    out
}
