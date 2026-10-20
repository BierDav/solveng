//! What the compiler saw: solveng's `solve-block`s leave `<solveng>` metadata, which editors query after
//! compiling (`query(<solveng>)`). Typewright's typst bridge collects it into a [`Report`].

use serde::{Deserialize, Serialize};

use crate::Integration;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Report {
    /// How many `solve-block`s (and resets in them) reported; none means solveng didn't run or predates reports.
    pub blocks: usize,
    /// Equations and raw elements a `solve-block` saw.
    pub elements: Vec<ReportElement>,
    /// Everything the blocks defined, with the values the compiler computed.
    pub defined: Vec<ReportValue>,
    /// The units `solve-config(units: …)` set.
    pub units: Vec<ReportValue>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
    /// `solve-config(integration: …)` there.
    pub integration: String,
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

    /// What the `solve-block`s calculate in the file at `path`, or None when the report can't tell, as with a
    /// solveng that predates reports or a file the compile may not have reached. `main` is the compiled file, which
    /// the compile did reach: without reported elements nothing in it is in a `solve-block`.
    pub fn integration(&self, path: &str, main: bool) -> Option<u8> {
        if self.blocks == 0 {
            return None;
        }
        let here: Vec<u8> = self.elements.iter().filter(|e| e.path == path).map(|e| flags(&e.integration)).collect();
        if here.is_empty() {
            return if main { Some(Integration::NONE) } else { None };
        }
        Some(here.into_iter().fold(Integration::NONE, |a, b| a | b))
    }
}

/// The flags of an `integration` setting.
pub(crate) fn flags(setting: &str) -> u8 {
    match setting {
        "full" => Integration::FULL,
        "code" => Integration::CODE,
        "none" => Integration::NONE,
        _ => Integration::MATH,
    }
}
