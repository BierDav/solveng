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
            after_arrow = true;
            unit_mode = true;
            i += 2;
        } else if c == '[' {
            in_bracket = true;
            unit_mode = true;
            i += 1;
        } else if c == ']' {
            in_bracket = false;
            unit_mode = false;
            i += 1;
        } else if "=<>≤≥".contains(c) {
            let end = if i + 1 < n && code[i + 1] == '=' { i + 2 } else { i + 1 };
            emit(i, end, K::OPERATOR);
            unit_mode = false;
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
                unit_mode = true;
            }
            i = j;
        } else if is_letter(c) || UNIT_CHARS.contains(c) {
            let mut j = i + 1;
            while j < n && (name_part(code[j]) || UNIT_CHARS.contains(code[j])) {
                j += 1;
            }
            let word = sub(code, i, j);
            let mut k = j;
            while k < n && code[k] == ' ' {
                k += 1;
            }
            let call = k < n && code[k] == '(';
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
            if SUPERSCRIPTS.contains(c) && unit_mode {
                emit(i, i + 1, K::UNIT);
            }
            i += 1;
        } else {
            // + - ( ) , end a value's unit.
            if !in_bracket && !after_arrow {
                unit_mode = false;
            }
            i += 1;
        }
    }
    out
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
            // Quoted text is a unit, except in a subscript: R_"load".
            emit(i, end, if sub_ { K::VARIABLE } else { K::UNIT });
            if !sub_ {
                unit_mode = true;
            }
            i = end;
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
        } else if is_letter(c) {
            let mut j = i + 1;
            while j < n && (is_letter(math[j]) || (math[j] == '.' && j + 1 < n && is_letter(math[j + 1]))) {
                j += 1;
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
