//! Equations of Typst markup.

use crate::text::{index_of, index_of_char, starts_with};

/// An equation, in characters: the whole element (with its dollars) and its body.
#[derive(Clone, Debug)]
pub(crate) struct Element {
    pub start: usize,
    pub end: usize,
    pub body_start: usize,
    pub body_end: usize,
}

/// Equations, skipping comments and raw text (where a `$` is just text); good enough for an editor, not a parser.
/// Unlike typst's parser it keeps going through unclosed ones, which is what's being typed.
pub(crate) fn elements(text: &[char]) -> Vec<Element> {
    let mut out = Vec::new();
    let n = text.len();
    let mut i = 0;
    while i < n {
        let c = text[i];
        if c == '\\' {
            i += 2;
        } else if c == '/' && starts_with(text, i, "//") && (i == 0 || text[i - 1] != ':') {
            // `//` starts a comment, except in a URL.
            i = index_of_char(text, '\n', i).unwrap_or(n);
        } else if c == '/' && starts_with(text, i, "/*") {
            i = index_of(text, "*/", i + 2).map_or(n, |j| j + 2);
        } else if c == '$' {
            let mut j = i + 1;
            while j < n && text[j] != '$' {
                if text[j] == '\\' {
                    j += 1;
                } else if text[j] == '"' {
                    j = index_of_char(text, '"', j + 1).unwrap_or(n - 1);
                }
                j += 1;
            }
            let close = j.min(n);
            out.push(Element { start: i, end: (close + 1).min(n), body_start: i + 1, body_end: close });
            i = close + 1;
        } else if c == '`' {
            // Raw text: skipped up to the run of backticks closing it.
            let mut ticks = 0;
            while i + ticks < n && text[i + ticks] == '`' {
                ticks += 1;
            }
            if ticks == 2 {
                i += 2;
                continue;
            }
            let fence = "`".repeat(ticks);
            let mut close = index_of(text, &fence, i + ticks);
            // A longer run doesn't close a shorter fence.
            while let Some(cl) = close {
                if cl + ticks < n && text[cl + ticks] == '`' {
                    let mut j = cl;
                    while j < n && text[j] == '`' {
                        j += 1;
                    }
                    close = index_of(text, &fence, j);
                } else {
                    break;
                }
            }
            i = close.map_or(n, |cl| cl + ticks);
        } else {
            i += 1;
        }
    }
    out
}
