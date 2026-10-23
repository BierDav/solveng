//! What the compiler saw: solveng's `worksheet`s leave `<solveng>` metadata, which editors query after
//! compiling (`query(<solveng>)`). Typewright's typst bridge collects it into a [`Report`].

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::elements::Element;
use crate::text::Doc;

/// For each element, the reported one it is, by source text. Copies of the same source are aligned in order at the
/// least total `cost` (of element, report), so an earlier copy can't steal a later one's report.
pub(crate) fn match_elements(doc: &Doc, elements: &[Element], reported: &[&ReportElement], cost: &dyn Fn(usize, usize) -> u64) -> Vec<Option<usize>> {
    let mut groups: HashMap<&str, (Vec<usize>, Vec<usize>)> = HashMap::new();
    for (i, report) in reported.iter().enumerate() {
        groups.entry(&report.text).or_default().1.push(i);
    }
    for (i, element) in elements.iter().enumerate() {
        if let Some(group) = groups.get_mut(doc.sub(element.start, element.end).as_str()) { group.0.push(i); }
    }
    let mut matches = vec![None; elements.len()];
    for (current, mut previous) in groups.into_values() {
        previous.sort_by_key(|&i| (reported[i].start, i));
        for (i, j) in match_occurrences(current.len(), previous.len(), &|i, j| cost(current[i], previous[j])) {
            matches[current[i]] = Some(previous[j]);
        }
    }
    matches
}

/// Pairs `(current, previous)` keeping both in order, as many as the shorter side has, at the least total cost.
fn match_occurrences(current: usize, previous: usize, cost: &dyn Fn(usize, usize) -> u64) -> Vec<(usize, usize)> {
    let reverse = current > previous;
    let (n, m) = if reverse { (previous, current) } else { (current, previous) };
    if n == 0 { return Vec::new(); }
    if n == m { return (0..n).map(|i| (i, i)).collect(); }
    let pair = |i: usize, j: usize| if reverse { cost(j, i) } else { cost(i, j) };
    let mut costs = vec![0u64; m + 1];
    let mut chosen = vec![vec![false; m + 1]; n + 1];
    for i in 1..=n {
        let mut next = vec![u64::MAX; m + 1];
        for j in i..=m {
            let take = costs[j - 1].saturating_add(pair(i - 1, j - 1));
            if take <= next[j - 1] {
                next[j] = take;
                chosen[i][j] = true;
            } else { next[j] = next[j - 1]; }
        }
        costs = next;
    }
    let (mut i, mut j) = (n, m);
    let mut matches = Vec::with_capacity(n);
    while i > 0 {
        if chosen[i][j] {
            matches.push(if reverse { (j - 1, i - 1) } else { (i - 1, j - 1) });
            i -= 1;
        }
        j -= 1;
    }
    matches.reverse();
    matches
}

#[cfg(test)]
mod tests {
    fn match_occurrences(current: &[usize], previous: &[usize]) -> Vec<(usize, usize)> {
        super::match_occurrences(current.len(), previous.len(), &|i, j| current[i].abs_diff(previous[j]) as u64)
    }

    #[test]
    fn repeated_source_keeps_order_and_minimizes_total_movement() {
        assert_eq!(match_occurrences(&[0, 25, 50], &[25]), [(1, 0)]);
        assert_eq!(match_occurrences(&[0, 10], &[9, 11, 100]), [(0, 0), (1, 1)]);
        assert_eq!(match_occurrences(&[10, 20], &[0, 10, 20]), [(0, 1), (1, 2)]);
        assert_eq!(match_occurrences(&[0, 10, 20], &[10, 20]), [(1, 0), (2, 1)]);
        assert!(match_occurrences(&[], &[10]).is_empty());
        assert!(match_occurrences(&[10], &[]).is_empty());
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    /// How many `worksheet`s (and each `clear-at` match in them) reported; none means solveng didn't run or predates reports.
    pub blocks: usize,
    /// Equations and raw elements a `worksheet` saw.
    pub elements: Vec<ReportElement>,
    /// Everything the blocks defined, with the values the compiler computed.
    pub defined: Vec<ReportValue>,
    /// The units the `worksheet(units: …)` option set.
    pub units: Vec<ReportValue>,
    /// What went wrong calculating, where: the problems a `worksheet` shows in the document as red notes.
    pub errors: Vec<ReportError>,
}

/// A problem solveng showed in the document, from its `kind: "error"` metadata: the message and where it is.
/// Bridges fill `path`, `start`, `end` and `text` from the metadata's `element` as for [`ReportElement`]; equations
/// and code passed to `worksheet` have none, so those stay empty and `source` alone finds the statement.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportError {
    /// The file, relative to the root as typst names it; empty without an element.
    pub path: String,
    /// The element's UTF-16 range in the file when compiled, and its source there.
    pub start: usize,
    pub end: usize,
    pub text: String,
    /// The statement solveng read: a line of code (`x := R + U =`), empty for math, where the element is the
    /// statement.
    pub source: String,
    /// `adding different units: Ω and V`, without the `solveng:` the document shows before it.
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportElement {
    /// The file, relative to the root as typst names it.
    pub path: String,
    /// UTF-16 range in the file when compiled, and the source there.
    pub start: usize,
    pub end: usize,
    pub text: String,
    /// `math` or `code`.
    pub kind: String,
    /// Whether it was calculated.
    pub active: bool,
    /// Whether the innermost `worksheet` around calculates its math: `evaluate` there.
    pub evaluate: bool,
}

impl Default for ReportElement {
    fn default() -> ReportElement {
        ReportElement { path: String::new(), start: 0, end: 0, text: String::new(), kind: String::new(), active: false, evaluate: true }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ReportValue {
    pub name: String,
    /// `given`, `computed` or `function` for definitions.
    pub kind: String,
    /// `30 mA`; for functions `(x) = x^2 + 1`, with what they return when every parameter has a unit:
    /// `(x: Ω): Ω = x`.
    pub value: String,
}

impl Report {
    /// Reads the JSON report; anything unreadable is an empty report.
    pub fn parse(json: &str) -> Report {
        serde_json::from_str(json).unwrap_or_default()
    }

    /// Whether a `worksheet` calculates math anywhere in the file at `path`, or None when the report can't tell, as
    /// with a solveng that predates reports or a file the compile may not have reached. `main` is the compiled file,
    /// which the compile did reach: without reported elements nothing in it is in a `worksheet`.
    pub fn evaluates(&self, path: &str, main: bool) -> Option<bool> {
        if self.blocks == 0 {
            return None;
        }
        let mut here = self.elements.iter().filter(|e| e.path == path).peekable();
        if here.peek().is_none() {
            return if main { Some(false) } else { None };
        }
        Some(here.any(|e| e.evaluate))
    }
}
