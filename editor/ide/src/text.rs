//! Text as characters, with the UTF-16 offsets editors use and the byte offsets typst-syntax uses.

pub(crate) struct Doc {
    pub s: String,
    pub c: Vec<char>,
    /// Byte offset of each character, and the length at the end.
    bytes: Vec<usize>,
    /// UTF-16 offset of each character, and the length at the end.
    utf16: Vec<usize>,
}

impl Doc {
    pub fn new(s: &str) -> Doc {
        let c: Vec<char> = s.chars().collect();
        let mut bytes = Vec::with_capacity(c.len() + 1);
        let mut utf16 = Vec::with_capacity(c.len() + 1);
        let (mut b, mut u) = (0, 0);
        for ch in &c {
            bytes.push(b);
            utf16.push(u);
            b += ch.len_utf8();
            u += ch.len_utf16();
        }
        bytes.push(b);
        utf16.push(u);
        Doc { s: s.to_string(), c, bytes, utf16 }
    }

    pub fn to16(&self, i: usize) -> u32 {
        self.utf16[i.min(self.c.len())] as u32
    }

    pub fn from16(&self, u: usize) -> usize {
        self.utf16.partition_point(|&x| x < u).min(self.c.len())
    }

    pub fn of_byte(&self, b: usize) -> usize {
        self.bytes.partition_point(|&x| x < b).min(self.c.len())
    }

    pub fn sub(&self, a: usize, b: usize) -> String {
        sub(&self.c, a, b)
    }
}

pub(crate) fn sub(c: &[char], a: usize, b: usize) -> String {
    let b = b.min(c.len());
    if a >= b { String::new() } else { c[a..b].iter().collect() }
}

pub(crate) fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

pub(crate) fn is_letter(c: char) -> bool {
    c.is_alphabetic()
}

pub(crate) fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub(crate) fn is_letter_or_digit(c: char) -> bool {
    is_letter(c) || is_digit(c)
}

pub(crate) fn starts_with(c: &[char], at: usize, pat: &str) -> bool {
    let mut i = at;
    for p in pat.chars() {
        if c.get(i) != Some(&p) {
            return false;
        }
        i += 1;
    }
    true
}

pub(crate) fn index_of(c: &[char], pat: &str, from: usize) -> Option<usize> {
    let p: Vec<char> = pat.chars().collect();
    if p.is_empty() {
        return Some(from.min(c.len()));
    }
    (from..c.len().saturating_sub(p.len() - 1)).find(|&i| c[i..i + p.len()] == p[..])
}

pub(crate) fn index_of_char(c: &[char], ch: char, from: usize) -> Option<usize> {
    (from..c.len()).find(|&i| c[i] == ch)
}

pub(crate) fn last_index_of_char(c: &[char], ch: char, before: usize) -> Option<usize> {
    (0..before.min(c.len())).rev().find(|&i| c[i] == ch)
}

/// Character offset of a byte offset in `s`.
pub(crate) fn char_index(s: &str, byte: usize) -> usize {
    s[..byte].chars().count()
}

/// The text before `sep`, or all of it.
pub(crate) fn before<'a>(s: &'a str, sep: &str) -> &'a str {
    s.find(sep).map_or(s, |i| &s[..i])
}

/// The text after `sep`, or `missing`.
pub(crate) fn after<'a>(s: &'a str, sep: &str, missing: &'a str) -> &'a str {
    s.find(sep).map_or(missing, |i| &s[i + sep.len()..])
}
