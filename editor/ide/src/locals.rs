//! Source ranges of sum binders. Keep bounds outside the index's scope, just as
//! the engine does, and retain source offsets through escaped Typst strings.

#[derive(Clone, Debug)]
pub(crate) struct Local {
    pub declaration: (usize, usize),
    pub body: (usize, usize),
    pub key: String,
}

#[derive(Debug)]
struct Token {
    text: String,
    start: usize,
    end: usize,
}

fn tokens(c: &[char]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if c[i].is_whitespace() { i += 1; continue; }
        if c[i] == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' { i += 1; }
            continue;
        }
        let start = i;
        if solveng_names::numeric_start(c, i) {
            i = solveng_names::scan(c, i).ok().flatten().map_or(i + 1, |name| name.end);
        } else if c[i].is_ascii_digit() {
            i += 1;
            while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.') { i += 1; }
            if c.get(i).is_some_and(|c| matches!(c, 'e' | 'E')) {
                let mut exponent = i + 1;
                if c.get(exponent).is_some_and(|c| matches!(c, '+' | '-' | '−')) { exponent += 1; }
                if c.get(exponent).is_some_and(char::is_ascii_digit) {
                    i = exponent + 1;
                    while i < c.len() && c[i].is_ascii_digit() { i += 1; }
                }
            }
        } else if let Some(name) = solveng_names::scan(c, i).ok().flatten() {
            i = name.end;
        } else {
            i += 1;
            if i < c.len() && matches!((c[start], c[i]), (':', '=') | ('-', '>') | ('<', '=') | ('>', '=') | ('!', '=')) { i += 1; }
        }
        out.push(Token { text: c[start..i].iter().collect(), start, end: i });
    }
    out
}

fn close(t: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, token) in t.iter().enumerate().skip(open) {
        match token.text.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => {
                depth -= 1;
                if depth == 0 { return Some(i); }
            }
            _ => {}
        }
    }
    None
}

fn atom_end(t: &[Token], at: usize) -> Option<usize> {
    match t.get(at)?.text.as_str() {
        "(" | "[" | "{" => close(t, at).map(|i| i + 1),
        _ => Some(at + 1),
    }
}

/// A summand is one product; top-level addition belongs outside the sum.
fn term_end(t: &[Token], start: usize) -> usize {
    let mut i = start;
    while i < t.len() {
        match t[i].text.as_str() {
            "(" | "[" | "{" => {
                let Some(end) = close(t, i) else { return t.len() };
                i = end + 1;
                continue;
            }
            "+" | "-" | "−" if i == start || matches!(t[i - 1].text.as_str(), "^" | "*" | "/" | "·" | "×") => {}
            "+" | "-" | "−" | "=" | ":=" | "≔" | "->" | "→" | "<" | ">" | "<=" | ">=" | "≤" | "≥" | "!=" | "≠" | "," | ";" | ")" | "]" | "}" | "\\" => break,
            _ => {}
        }
        i += 1;
    }
    i
}

/// Read each complete sum while tolerating unfinished text elsewhere.
pub(crate) fn read(source: &[char], base: usize, string: bool, code: bool) -> Vec<Local> {
    let (c, offsets) = if string { crate::text::unescape(source) } else { (source.to_vec(), (0..=source.len()).collect()) };
    let mut out = Vec::new();
    for (start, equation_end) in crate::text::equation_ranges(&c, code) {
        let t = tokens(&c[start..equation_end]);
        let at = |i: usize| base + offsets[start + i];
        for (i, token) in t.iter().enumerate() {
            let notation = token.text == "sum_";
            if !notation && token.text != "sum" { continue; }
            // Typst writes sum_(k=0)^n, code also accepts sum(body, k, lo, hi).
            if t.get(i + 1).map(|t| t.text.as_str()) != Some("(") { continue; }
            let Some(end) = close(&t, i + 1) else { continue };
            let (declaration, body) = if notation {
                if t.get(i + 3).map(|t| t.text.as_str()) != Some("=") || end <= i + 4 { continue; }
                if t.get(end + 1).map(|t| t.text.as_str()) != Some("^") { continue; }
                let Some(body_start) = atom_end(&t, end + 2) else { continue };
                let body_end = term_end(&t, body_start);
                let body = if body_start == t.len() { (equation_end - start, equation_end - start) }
                    else if body_start == body_end { continue; }
                    else { (t[body_start].start, t[body_end - 1].end) };
                (i + 2, body)
            } else {
                let mut commas = Vec::new();
                let mut j = i + 2;
                while j < end {
                    if matches!(t[j].text.as_str(), "(" | "[" | "{") {
                        let Some(last) = close(&t, j) else { break };
                        j = last + 1;
                        continue;
                    }
                    if t[j].text == "," { commas.push(j); }
                    j += 1;
                }
                if commas.len() != 3 || commas[0] <= i + 2 || commas[1] != commas[0] + 2 { continue; }
                (commas[0] + 1, (t[i + 2].start, t[commas[0] - 1].end))
            };
            let decl = &t[declaration];
            let raw: Vec<char> = decl.text.chars().collect();
            let Some(name) = solveng_names::scan(&raw, 0).ok().flatten().filter(|n| n.end == raw.len()) else { continue };
            out.push(Local { declaration: (at(decl.start), at(decl.end)), body: (at(body.0), at(body.1)), key: name.value });
        }
    }
    out
}
