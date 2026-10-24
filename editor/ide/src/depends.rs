//! What a definition depends on: the names its formula uses, and theirs in turn, for hovers.

use std::collections::HashSet;

use crate::{Analysis, Context, Location, TokenKind as K};

fn slice(text: &str, start: u32, end: u32) -> String {
    let units: Vec<u16> = text.encode_utf16().skip(start as usize).take(end.saturating_sub(start) as usize).collect();
    String::from_utf16_lossy(&units)
}

/// A name as written in math or a string, as solveng stores it: `t_"half"` → `t_half`, `Delta""r` → `Δr`.
pub(crate) fn key(name: &str) -> String {
    crate::resolved_name(&name.replace("\"\"", "\0").replace('"', "").replace('\0', "\"\""))
}

/// The names the definition at `definition` uses, and the names those use, in the order found: (name, whether
/// it's defined). Parameters, sum indices, and solveng's operations and Typst's math functions don't count; `name` is the definition's own.
pub(crate) fn dependencies(document: &str, context: &Context, definition: Location, name: &str) -> Vec<(String, bool)> {
    let mut pending = vec![definition];
    let mut visited = HashSet::new();
    let mut names = HashSet::from([name.to_string()]);
    let mut out = Vec::new();
    while let Some(location) = pending.pop() {
        if !visited.insert((location.path.clone(), location.start)) {
            continue;
        }
        // The file the definition is in, read as that file.
        let (source, c) = match location.path.as_deref() {
            Some(path) if path != context.path => {
                let Some(f) = context.files.iter().find(|f| f.path == path) else { continue };
                let mut c = context.clone();
                c.path = path.into();
                (f.text.as_str(), c)
            }
            _ => (document, context.clone()),
        };
        // Its statement: the innermost equation or string around it; in a string what a line break or `;` ends.
        let Some(region) = crate::regions(source, &c)
            .into_iter()
            .filter(|r| r.kind != "call" && r.start <= location.start && location.end <= r.end)
            .min_by_key(|r| r.end - r.start)
        else {
            continue;
        };
        let mut end = region.end;
        if region.kind != "math" {
            let tail = slice(source, location.end, end);
            if let Some(at) = tail.find(['\n', ';']) {
                end = location.end + tail[..at].encode_utf16().count() as u32;
            }
        }
        let tokens = crate::document_tokens(source, &c);
        let Some(assignment) = tokens.iter().find(|t| {
            t.start >= location.end && t.start < end && t.kind == K::OPERATOR && matches!(slice(source, t.start, t.start + t.length).as_str(), ":=" | "≔")
        }) else {
            continue;
        };
        // what's shown after = or -> isn't the formula
        if let Some(result) = tokens.iter().find(|t| t.start > assignment.start && t.start < end && t.kind == K::EVALUATE) {
            end = result.start;
        }
        for token in tokens.iter().filter(|t| t.start > assignment.start && t.start < end && matches!(t.kind, K::VARIABLE | K::FUNCTION | K::FUNCTION_CALL | K::BUILTIN_FUNCTION)) {
            let target = crate::rename_target(source, &c, token.start);
            let name = key(&target.map_or_else(|| slice(source, token.start, token.start + token.length), |t| t.name));
            let definition = crate::definition(source, &c, token.start);
            // solveng's operations and Typst's math functions aren't dependencies: sin(x)
            if definition.is_none() && matches!(token.kind, K::FUNCTION | K::FUNCTION_CALL | K::BUILTIN_FUNCTION) {
                continue;
            }
            if names.insert(name.clone()) {
                out.push((name, definition.is_some()));
            }
            if let Some(mut definition) = definition {
                if definition.path.is_none() {
                    definition.path = location.path.clone();
                }
                pending.push(definition);
            }
        }
    }
    out
}

/// The `### Depends on` section of a defined name's hover, None when it depends on nothing: each name on a line,
/// with its value or formula where known.
pub(crate) fn section(a: &Analysis, document: &str, definition: Location, name: &str) -> Option<String> {
    let lines: Vec<String> = dependencies(document, a.context, definition, name)
        .into_iter()
        .map(|(name, defined)| match crate::complete::defined_line(a, &name) {
            Some(line) => line,
            None if !defined => format!("{name} // undefined"),
            None => name,
        })
        .collect();
    (!lines.is_empty()).then(|| format!("### Depends on\n\n```text\n{}\n```", lines.join("\n")))
}
