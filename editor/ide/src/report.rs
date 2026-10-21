//! What the compiler saw: solveng's `worksheet`s leave `<solveng>` metadata, which editors query after
//! compiling (`query(<solveng>)`). Typewright's typst bridge collects it into a [`Report`].

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    /// How many `worksheet`s (and each `clear-at` match in them) reported; none means solveng didn't run or predates reports.
    pub blocks: usize,
    /// Equations and raw elements a `worksheet` saw.
    pub elements: Vec<ReportElement>,
    /// Everything the blocks defined, with the values the compiler computed.
    pub defined: Vec<ReportValue>,
    /// The units `worksheet-config(units: …)` set.
    pub units: Vec<ReportValue>,
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
    /// `30 mA`; for functions `(x) = x^2 + 1`.
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
