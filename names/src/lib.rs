use unicode_general_category::{get_general_category, GeneralCategory as Category};
use unicode_normalization::UnicodeNormalization;

/// Built-in operation names for editor completion.
pub const OPERATIONS: &[&str] = &[
    "sqrt", "exp", "ln", "log", "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "abs",
    "arcsin", "arccos", "arctan", "diff", "integrate", "laplace", "invlaplace", "sum", "vec", "at", "rect", "tri", "sgn", "floor", "factorial", "cases",
    "mat", "det", "tr", "norm", "rank", "identity", "rows", "cols", "convolve", "range",
];

pub const GREEK: &[(&str, char)] = &[
    ("alpha", 'α'), ("Alpha", 'Α'), ("beta", 'β'), ("Beta", 'Β'), ("gamma", 'γ'), ("Gamma", 'Γ'),
    ("delta", 'δ'), ("Delta", 'Δ'), ("epsilon", 'ε'), ("Epsilon", 'Ε'), ("zeta", 'ζ'), ("Zeta", 'Ζ'),
    ("eta", 'η'), ("Eta", 'Η'), ("theta", 'θ'), ("Theta", 'Θ'), ("iota", 'ι'), ("Iota", 'Ι'),
    ("kappa", 'κ'), ("Kappa", 'Κ'), ("lambda", 'λ'), ("Lambda", 'Λ'), ("mu", 'μ'), ("Mu", 'Μ'),
    ("nu", 'ν'), ("Nu", 'Ν'), ("xi", 'ξ'), ("Xi", 'Ξ'), ("omicron", 'ο'), ("Omicron", 'Ο'),
    ("pi", 'π'), ("Pi", 'Π'), ("rho", 'ρ'), ("Rho", 'Ρ'), ("sigma", 'σ'), ("Sigma", 'Σ'),
    ("tau", 'τ'), ("Tau", 'Τ'), ("upsilon", 'υ'), ("Upsilon", 'Υ'), ("phi", 'φ'), ("Phi", 'Φ'),
    ("chi", 'χ'), ("Chi", 'Χ'), ("psi", 'ψ'), ("Psi", 'Ψ'), ("omega", 'ω'), ("Omega", 'Ω'),
];


pub fn greek(word: &str) -> Option<char> {
    GREEK.iter().find(|(name, _)| *name == word).map(|(_, symbol)| *symbol)
}

pub fn is_start(c: char) -> bool {
    matches!(c, '´' | '`') || matches!(get_general_category(c), Category::UppercaseLetter | Category::LowercaseLetter | Category::TitlecaseLetter | Category::ModifierLetter | Category::OtherLetter | Category::OtherSymbol)
}

pub fn is_continue(c: char) -> bool {
    is_start(c) || c.is_ascii_digit() || c == '_' || c == '\u{200d}'
        || matches!(get_general_category(c), Category::NonspacingMark | Category::SpacingMark | Category::EnclosingMark) || ('\u{1f3fb}'..='\u{1f3ff}').contains(&c)
}

/// Digits, Unicode symbols and explicit joins delimit Latin runs, never the name itself.
pub fn normalize(name: &str) -> String {
    let mut result = String::new();
    let mut letters = String::new();
    let flush = |letters: &mut String, result: &mut String| {
        if let Some(symbol) = greek(letters) { result.push(symbol); } else { result.push_str(letters); }
        letters.clear();
    };
    for c in name.nfc() {
        if c.is_ascii_alphabetic() { letters.push(c); } else {
            flush(&mut letters, &mut result);
            if c != '"' { result.push(c); }
        }
    }
    flush(&mut letters, &mut result);
    result.nfc().collect()
}

pub struct Name {
    pub end: usize,
    pub value: String,
}

/// Decode a transport string without changing the source offsets used by the editor.
pub fn quoted(text: &[char], start: usize) -> Result<(String, usize), String> {
    let mut value = String::new();
    let mut i = start + 1;
    while let Some(&c) = text.get(i) {
        if c == '"' { return Ok((value, i + 1)); }
        if c == '\\' {
            i += 1;
            match text.get(i) {
                Some('"') => value.push('"'),
                Some('\\') => value.push('\\'),
                _ => return Err("only escaped quotes and backslashes are allowed in a variable name".into()),
            }
        } else { value.push(c); }
        i += 1;
    }
    Err("a variable name's quote isn't closed".into())
}

fn validate(name: &str) -> Result<(), String> {
    let text: Vec<char> = name.chars().collect();
    let mut i = 0;
    let mut begun = false;
    while i < text.len() {
        let c = text[i];
        if c == '"' && text.get(i + 1) == Some(&'"') {
            i += 2;
            continue;
        }
        let numeric = c.is_ascii_digit() && name.contains("\"\"");
        if (!begun && !is_start(c) && !numeric) || (begun && !is_continue(c)) {
            return Err(format!("'{c}' isn't allowed in a variable name; use spaces or operators between expressions"));
        }
        if c == '\u{200d}' {
            let previous = text[..i].iter().rev().find(|&&p| !matches!(get_general_category(p), Category::NonspacingMark | Category::ModifierSymbol));
            if previous.is_none_or(|&p| get_general_category(p) != Category::OtherSymbol)
                || text.get(i + 1).is_none_or(|&p| get_general_category(p) != Category::OtherSymbol) {
                return Err("a zero-width joiner is only allowed inside an emoji sequence".into());
            }
        }
        begun = true;
        i += 1;
    }
    if !begun { return Err("a variable name cannot be empty".into()); }
    Ok(())
}

/// One name includes adjacent quoted fragments, digits and explicit empty-string joins.
pub fn scan(text: &[char], start: usize) -> Result<Option<Name>, String> {
    if text.get(start).is_none_or(|&c| !is_start(c) && c != '"') && !numeric_start(text, start) { return Ok(None); }
    let mut raw = String::new();
    let mut i = start;
    while let Some(&c) = text.get(i) {
        if c == '"' {
            let (part, end) = quoted(text, i)?;
            raw.push_str("\"\"");
            raw.push_str(&part);
            raw.push_str("\"\"");
            i = end;
        } else if is_continue(c) {
            raw.push(c);
            i += 1;
        } else { break; }
    }
    if raw.replace("\"\"", "").is_empty() { return Ok(None); }
    validate(&raw)?;
    if text.get(i) == Some(&'.') && text.get(i + 1).is_some_and(char::is_ascii_digit) {
        return Err("a decimal cannot continue a variable name; write an explicit multiplication".into());
    }
    Ok(Some(Name { end: i, value: normalize(&raw) }))
}

pub fn resolve(source: &str) -> Result<String, String> {
    let text: Vec<char> = source.chars().collect();
    let name = scan(&text, 0)?.ok_or("expected a variable name")?;
    if name.end != text.len() { return Err("a variable name must not contain whitespace or operators".into()); }
    Ok(name.value)
}

/// A leading number is a coefficient unless an explicit quoted fragment joins it to a name.
pub fn numeric_start(text: &[char], start: usize) -> bool {
    text.get(start).is_some_and(char::is_ascii_digit)
        && text[start..].iter().position(|c| !c.is_ascii_digit()).is_some_and(|i| text.get(start + i..start + i + 2) == Some(&['"', '"']))
}

/// Preserve the requested name, using a join when its leading digits would otherwise be a coefficient.
pub fn for_rename(source: &str) -> Option<String> {
    let source = source.trim();
    if source.is_empty() { return None; }
    if let Ok(name) = resolve(source) { return Some(name); }
    if source.contains('"') { return None; }
    let name = normalize(source);
    resolve(&code(&name)).ok()
}

pub fn code(name: &str) -> String {
    let digits = name.chars().take_while(char::is_ascii_digit).count();
    if digits == 0 { name.into() } else { format!("{}\"\"{}", &name[..digits], &name[digits..]) }
}

/// Quotes only carry the normalized name through Typst's parser.
pub fn math(name: &str) -> String {
    fn quote(s: &str) -> String { format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")) }
    fn fragment(s: &str) -> String {
        let digits = s.chars().take_while(char::is_ascii_digit).count();
        if digits > 0 {
            return format!("{}\"\"{}", &s[..digits], if digits == s.len() { String::new() } else { fragment(&s[digits..]) });
        }
        if s.chars().count() == 1 && s.chars().all(char::is_alphabetic) { s.into() } else { quote(s) }
    }
    match name.split_once('_') {
        Some((base, sub)) => {
            let sub = if !sub.is_empty() && sub.chars().all(|c| c.is_ascii_digit()) { sub.into() }
                else if sub.starts_with(|c: char| c.is_ascii_digit()) { quote(sub) }
                else { fragment(sub) };
            format!("{}_{}", fragment(base), sub)
        }
        None => fragment(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_greek_letter_resolves_in_both_cases() {
        let names = ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Iota", "Kappa", "Lambda", "Mu",
            "Nu", "Xi", "Omicron", "Pi", "Rho", "Sigma", "Tau", "Upsilon", "Phi", "Chi", "Psi", "Omega"];
        for (name, symbol) in names.into_iter().zip("ΑΒΓΔΕΖΗΘΙΚΛΜΝΞΟΠΡΣΤΥΦΧΨΩ".chars()) {
            for (name, symbol) in [(name.to_string(), symbol), (name.to_lowercase(), symbol.to_lowercase().next().unwrap())] {
                assert_eq!(greek(&name), Some(symbol));
                assert_eq!(resolve(&name).unwrap(), symbol.to_string());
                assert_eq!(resolve(&format!("{name}2_{name}")).unwrap(), format!("{symbol}2_{symbol}"));
            }
        }
        assert_eq!(normalize("AlphaBeta"), "AlphaBeta");
        assert_eq!(normalize("ALPHA"), "ALPHA");
    }

    #[test]
    fn rename_formats_numeric_names_without_changing_them() {
        for (typed, expected) in [("2x", "2x"), ("123", "123"), ("Delta2tau", "Δ2τ"), ("😅2tau", "😅2τ")] {
            let name = for_rename(typed).unwrap();
            assert_eq!(name, expected, "{typed}");
            assert_eq!(resolve(&code(&name)).unwrap(), name);
            assert_eq!(resolve(&math(&name)).unwrap(), name);
        }
        for invalid in ["", "   ", "^", "2 bad-name", "a+b", "!!!", "a\u{200d}b"] {
            assert!(for_rename(invalid).is_none(), "{invalid}");
        }
        assert_eq!(code("2x"), "2\"\"x");
        assert!(resolve("2x").is_err());
    }

    #[test]
    fn identifiers_have_one_canonical_spelling() {
        for (source, canonical) in [
            ("x´", "x´"), ("r`", "r`"), ("´x", "´x"), ("x´´2_tau", "x´´2_τ"), ("Delta´tau", "Δ´τ"),
            ("t2", "t2"), ("t\"\"2", "t2"), ("\"t2\"", "t2"),
            ("Delta2tau", "Δ2τ"), ("\"Delta2tau\"", "Δ2τ"), ("Δ2τ", "Δ2τ"),
            ("Delta\"\"tau", "Δτ"), ("\"Delta\\\"\\\"tau\"", "Δτ"),
            ("t\"\"tau", "tτ"), ("Deltatau", "Deltatau"), ("DeltaT", "DeltaT"),
            ("foo2tau", "foo2τ"), ("x_tau", "x_τ"), ("x_\"tau\"", "x_τ"),
            ("😅2Delta_🫠", "😅2Δ_🫠"), ("👩🏽‍💻2tau", "👩🏽‍💻2τ"), ("e\u{301}2tau", "é2τ"),
        ] {
            assert_eq!(resolve(source).unwrap(), canonical, "{source}");
            assert_eq!(resolve(&math(canonical)).unwrap(), canonical, "{source}");
        }
    }

    #[test]
    fn names_reject_hidden_delimiters_and_invalid_joiners() {
        for source in ["2t", "t2.5", "t 2", "\"t 2\"", "t\u{200b}2", "t\u{200d}x", "t٢", "\"t+2\"", "\"\"", "\"t\\n\""] {
            assert!(resolve(source).is_err(), "{source}");
        }
    }

    #[test]
    fn numeric_subscripts_do_not_need_a_name_join() {
        for (name, expected) in [("g_2", "g_2"), ("g_12", "g_12"), ("g_2x", "g_\"2x\""), ("2g_2", "2\"\"g_2")] {
            assert_eq!(math(name), expected);
            assert_eq!(resolve(&math(name)).unwrap(), name);
        }
    }
}

/// Match partial spellings even when the last Greek word is still being typed.
pub fn matches_prefix(name: &str, prefix: &str) -> bool {
    let mut typed = prefix.to_string();
    for _ in 0..3 { typed = typed.replace("\\\"", "\"").replace("\\\\", "\\"); }
    let grouped = typed.contains("_(");
    typed = typed.replace('"', "").replace("_(", "_").replace(')', "");
    if grouped { typed.retain(|c| !c.is_whitespace()); }
    let typed: Vec<char> = typed.nfc().flat_map(char::to_lowercase).collect();
    let name: Vec<char> = name.nfc().collect();
    fn matches(name: &[char], typed: &[char]) -> bool {
        if typed.is_empty() { return true; }
        let Some((&first, rest)) = name.split_first() else { return false; };
        let mut choices = vec![first.to_string()];
        if let Some((spelling, _)) = GREEK.iter().find(|(_, symbol)| *symbol == first) { choices.push(spelling.to_string()); }
        choices.into_iter().any(|choice| {
            let choice: Vec<char> = choice.chars().flat_map(char::to_lowercase).collect();
            choice.starts_with(typed) || (typed.starts_with(&choice) && matches(rest, &typed[choice.len()..]))
        })
    }
    matches(&name, &typed)
}
