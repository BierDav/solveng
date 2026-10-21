//! Editor support for solveng (`@preview/solveng`), the calculation language of Typst documents: ```` ```solveng ````
//! code, the strings of `#def`, `#solve`, `#graph` and `#lambdify`, and in a `solve-block` math or code without a
//! language, as `solve-config(integration: …)` says.
//!
//! Everything is a pure function of the document and a [`Context`] (the files around it, what the compiler
//! reported). Offsets are UTF-16 code units, as editors and LSP count them.

mod calls;
mod complete;
mod conceal;
mod convert;
mod elements;
mod highlight;
mod navigate;
pub mod params;
mod report;
mod scope;
mod text;
mod tokens;
mod unit_refs;
mod units;

use std::cell::OnceCell;
use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

pub use complete::complete;
pub use conceal::{conceals, conceals_of, resolved_name};
pub use convert::{conversions, from_math, to_math};
pub use navigate::{definition, rename, rename_target, usages};
pub use highlight::{diagnostics, document_tokens, hover, math_tokens, math_tokens_of, pretty, regions, tokens, tokens_with_units};
pub use report::{Report, ReportElement, ReportValue};
pub use units::{is_unit, unit_name};

use elements::Element;
use text::Doc;

/// Language tag of solveng code.
pub const TAG: &str = "solveng";

/// This crate's version, for editors to show what they bundle.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Kinds of [`Token`], the same numbers as Typewright's code highlighting.
pub struct TokenKind;
impl TokenKind {
    pub const KEYWORD: u8 = 0;
    pub const STRING: u8 = 1;
    pub const COMMENT: u8 = 2;
    pub const NUMBER: u8 = 3;
    pub const FUNCTION: u8 = 4;
    pub const TYPE: u8 = 5;
    pub const VARIABLE: u8 = 6;
    pub const ANNOTATION: u8 = 7;
    pub const OPERATOR: u8 = 8;
    pub const UNIT: u8 = 9;
    /// A function's parameter, where it's defined and used (`x` in `f(x: A) = 2 x`, what `..` takes), and what a
    /// graph plots against. Editors without it show a variable.
    pub const PARAMETER: u8 = 10;
}

/// What a `solve-block` calculates besides tagged code, as `solve-config(integration: …)` sets it; bit flags.
pub struct Integration;
impl Integration {
    pub const NONE: u8 = 0;
    pub const CODE: u8 = 1;
    pub const MATH: u8 = 2;
    pub const FULL: u8 = 3;
}

/// Kinds of [`Item`].
pub struct ItemKind;
impl ItemKind {
    pub const VARIABLE: u8 = 0;
    pub const FUNCTION: u8 = 1;
    pub const UNIT: u8 = 2;
    pub const CONSTANT: u8 = 3;
    /// A named parameter of a solveng function: `steps: `.
    pub const PARAMETER: u8 = 4;
    /// One of an option's values: `"value"`.
    pub const VALUE: u8 = 5;
    /// A word of solveng's own: `as`, `any`.
    pub const KEYWORD: u8 = 6;
}

/// A highlighted range: UTF-16 start and length, and a [`TokenKind`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Token {
    pub start: u32,
    pub length: u32,
    pub kind: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Item {
    pub label: String,
    /// An [`ItemKind`].
    pub kind: u8,
    /// What it is: `= 30 mA`, `milliampere`, `f(x) = x^2 + 1`.
    pub detail: String,
    /// Text inserted for the item; `$0` marks where the caret goes.
    pub insert: String,
    /// Markdown, for parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// What typing matches when it differs from `insert`: `Delta_R` for `Δ_R`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

impl Item {
    pub(crate) fn new(label: &str, kind: u8, detail: &str, insert: &str) -> Item {
        Item { label: label.into(), kind, detail: detail.into(), insert: insert.into(), documentation: None, filter: None }
    }
}

/// Completions at a caret: `items` replace the UTF-16 range `from..<offset`. `exclusive` ones (in solveng code
/// and strings) replace the Typst language server's; others (in math, parameter names) come first next to its.
#[derive(Clone, Debug, Serialize)]
pub struct Completions {
    pub from: u32,
    pub items: Vec<Item>,
    pub exclusive: bool,
}

/// Rewrites the call, equation or code at a caret into the other spelling: replace `start..<end` with `text`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Conversion {
    pub title: String,
    pub start: u32,
    pub end: u32,
    pub text: String,
}

/// Code shown as it renders while the caret is elsewhere: UTF-16 `start` and `length` show as `character` (a
/// Unicode scalar), in the element `element_start..<element_end`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Conceal {
    pub start: u32,
    pub length: u32,
    pub character: u32,
    pub element_start: u32,
    pub element_end: u32,
}

/// A problem solveng would report, found while typing: an unknown unit.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Problem {
    pub start: u32,
    pub end: u32,
    pub message: String,
    /// Always `warning`: the compile has the last word.
    pub severity: &'static str,
}

/// A range in the document (`path` None) or in one of the context's files (its `path`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Location {
    pub path: Option<String>,
    pub start: u32,
    pub end: u32,
}

/// Replace UTF-16 `start..<end` with `text`, in the document (`path` None) or one of the context's files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Edit {
    pub path: Option<String>,
    pub start: u32,
    pub end: u32,
    pub text: String,
}

/// A name that can be renamed, as written at its UTF-16 range.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RenameTarget {
    pub start: u32,
    pub end: u32,
    pub name: String,
}

/// One of the files around a document, by its path relative to the root.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, Serialize)]
pub struct ContextFile {
    pub path: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Hover {
    pub start: u32,
    pub end: u32,
    pub markdown: String,
}

/// A stretch solveng reads: a raw element's body, an equation's, a string argument's contents.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Region {
    pub start: u32,
    pub end: u32,
    /// `code`, `math` or `string`.
    pub kind: &'static str,
}

/// What surrounds a document: the text of the files around it (the compiled main file, what it and the document
/// `#import` or `#include`, see [`references`]) and what the last compile reported.
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub text: String,
    /// The compiler's [`Integration`] for the document, which beats reading the settings.
    pub integration: Option<u8>,
    /// Values the compiler computed, used as completion details.
    pub defined: Vec<ReportValue>,
    /// Units `solve-config(units: …)` set, as the compiler saw them.
    pub units: Vec<ReportValue>,
    /// The document's path, relative to the root, as given to [`Context::with_report`].
    pub path: String,
    /// The equations and raw elements the compile saw, in every file; empty without a report.
    pub reported: Vec<ReportElement>,
    /// The files around one by one, when known; `text` joins them.
    pub files: Vec<ContextFile>,
    /// The compiled main file, relative to the root: where the compile starts, for which definition a name means.
    pub main: Option<String>,
}

const COMPILED: &str = "\u{1}solveng-integration:";

impl Context {
    /// The text of the files around; one starting like [`with_compiled`]'s carries the compiler's integration.
    pub fn new(text: &str) -> Context {
        if let Some(rest) = text.strip_prefix(COMPILED)
            && let Some(d) = rest.chars().next().and_then(|c| c.to_digit(10)) {
                let text = rest.split_once('\n').map_or("", |(_, t)| t);
                return Context { text: text.into(), integration: Some(d as u8), ..Context::default() };
            }
        Context { text: text.into(), ..Context::default() }
    }

    /// Adds what the compile reported for the document at `path` (relative to the root); `main` is the compiled file.
    pub fn with_report(mut self, report: &Report, path: &str, main: bool) -> Context {
        if let Some(i) = report.integration(path, main) {
            self.integration = Some(i);
        }
        self.defined = report.defined.clone();
        self.units = report.units.clone();
        self.path = path.into();
        if main && self.main.is_none() {
            self.main = Some(path.into());
        }
        self.reported = if report.blocks == 0 { Vec::new() } else { report.elements.clone() };
        self
    }

    /// The files around one by one, so definitions and usages there can be found; `text` becomes their join.
    pub fn with_files(mut self, files: Vec<ContextFile>) -> Context {
        self.text = files.iter().map(|f| f.text.as_str()).collect::<Vec<_>>().join("\n");
        self.files = files;
        self
    }

    /// The compiled main file (relative to the root, the document's own path if it's the main file), where
    /// [`definition`] and [`usages`] start reading the document in compile order.
    pub fn with_main(mut self, path: &str) -> Context {
        self.main = Some(path.into());
        self
    }

    /// The reported elements of the file at `path`.
    pub(crate) fn reported_in(&self, path: &str) -> Vec<&ReportElement> {
        self.reported.iter().filter(|e| e.path == path).collect()
    }
}

/// `context` carrying the compiler's integration for the document, unless that's negative; [`Context::new`]
/// reads it back. For editors that pass the context around as one string.
pub fn with_compiled(context: &str, integration: i32) -> String {
    if integration < 0 { context.into() } else { format!("{COMPILED}{integration}\n{context}") }
}

/// What the compile reported for the file at `path`: an [`Integration`], or -1 when the report can't tell.
pub fn compiled_integration(report: &str, path: &str, main: bool) -> i32 {
    Report::parse(report).integration(path, main).map_or(-1, i32::from)
}

static REFERENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"#(?:import|include)\s+"([^"@][^"]*)""#).unwrap());
static IMPLICIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"implicit-parameters\s*:\s*"(no|empty|allow)""#).unwrap());

/// What a text sets with `solve-config(implicit-parameters: …)`, the last one: `"empty"` makes `f()` take what
/// `f(..)` does, `"allow"` every function; None without a setting.
pub(crate) fn implicit_parameters(text: &str) -> Option<String> {
    IMPLICIT.captures_iter(text).last().map(|m| m[1].to_string())
}

static SETTING: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"integration\s*:\s*"(full|code|math|none)""#).unwrap());

/// Local files a document imports or includes (`#import "defaults.typ": *`, `#include "ch1.typ"`), as written;
/// editors read them (and the compiled main file) into the [`Context`].
pub fn references(document: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in REFERENCE.captures_iter(document) {
        if !out.iter().any(|r| r == &c[1]) {
            out.push(c[1].to_string());
        }
    }
    out
}

/// What the document's `solve-block` calculates besides tagged code: `"math"` unless `integration` says
/// otherwise, nothing without a `solve-block` or an `integration` setting. The compiler's answer wins, then the document's own setting.
pub fn integration(document: &str, context: &Context) -> u8 {
    if let Some(i) = context.integration {
        return i;
    }
    fallback(document, &context.text)
}

/// What a text calculates by its settings: an `integration` setting (which calculates outside a `solve-block` too),
/// else `"math"` in a `solve-block`, else nothing.
fn fallback(text: &str, around: &str) -> u8 {
    match SETTING.captures(text).or_else(|| SETTING.captures(around)) {
        Some(m) => report::flags(&m[1]),
        None if text.contains("solve-block") || around.contains("solve-block") => Integration::MATH,
        None => Integration::NONE,
    }
}

pub(crate) fn is_solveng(e: &Element, integration: u8) -> bool {
    if e.math { integration & Integration::MATH != 0 } else { e.tag == TAG || (e.tag.is_empty() && integration & Integration::CODE != 0) }
}

/// The UTF-16 `(body_start, body_end)` of a raw element (```` `x` ````, ```` ```solveng x``` ````) that holds
/// solveng code; code without a language counts with [`Integration::CODE`].
pub fn body(raw: &str, integration: u8) -> Option<(u32, u32)> {
    let doc = Doc::new(raw);
    let r = elements::elements(&doc.c).into_iter().next()?;
    if r.start != 0 || r.math || !is_solveng(&r, integration) {
        return None;
    }
    Some((doc.to16(r.body_start), doc.to16(r.body_end)))
}

/// Raw elements and equations outside the strings of solveng's calls, where a `$` or a backtick is just text.
pub(crate) fn scan(doc: &Doc, calls: &[calls::Call]) -> Vec<Element> {
    let mut c = doc.c.clone();
    for t in calls.iter().flat_map(|call| &call.texts) {
        for ch in &mut c[t.start..t.end] {
            if *ch == '$' || *ch == '`' {
                *ch = ' ';
            }
        }
    }
    elements::elements(&c)
}

/// Which elements solveng calculates. With the compiler's elements for the file (`reported`), an element whose
/// source matches one (the nearest by position, each used once) takes its flag; one written since takes the
/// `integration` of the matched element before it, and nothing before the first. Without them `fallback` decides.
/// Equations passed to `def` and the like are calculated wherever they are.
pub(crate) fn activity(doc: &Doc, elements: &[Element], calls: &[calls::Call], reported: &[&ReportElement], fallback: u8) -> Vec<bool> {
    let in_call = |e: &Element| e.math && calls.iter().any(|c| c.maths.iter().any(|m| m.0 == e.start));
    if reported.is_empty() {
        return elements.iter().map(|e| is_solveng(e, fallback) || in_call(e)).collect();
    }
    let mut used = vec![false; reported.len()];
    let mut before: Option<u8> = None;
    elements
        .iter()
        .map(|e| {
            let text = doc.sub(e.start, e.end);
            let at = doc.to16(e.start) as i64;
            let found = (0..reported.len())
                .filter(|&i| !used[i] && reported[i].text == text)
                .min_by_key(|&i| (reported[i].start as i64 - at).abs());
            let active = match found {
                Some(i) => {
                    used[i] = true;
                    before = Some(report::flags(&reported[i].integration));
                    reported[i].active
                }
                None => before.is_some_and(|f| is_solveng(e, f)),
            };
            active || in_call(e)
        })
        .collect()
}

/// A text read once: one of the files around a document, or their join.
pub(crate) struct Part {
    /// None for the joined text, whose files aren't known.
    pub path: Option<String>,
    pub doc: Doc,
    pub calls: Vec<calls::Call>,
    pub elements: Vec<Element>,
    pub active: Vec<bool>,
}

impl Part {
    fn new(path: Option<String>, text: &str, context: &Context) -> Part {
        let doc = Doc::new(text);
        let calls = if text.is_empty() { Vec::new() } else { calls::calls(&doc) };
        let elements = scan(&doc, &calls);
        let reported = path.as_deref().map(|p| context.reported_in(p)).unwrap_or_default();
        // A file's own setting, else the ones around.
        let fallback = fallback(text, &context.text);
        let active = activity(&doc, &elements, &calls, &reported, fallback);
        Part { path, doc, calls, elements, active }
    }

    pub fn active(&self, e: &Element) -> bool {
        self.elements.binary_search_by_key(&e.start, |x| x.start).ok().is_some_and(|i| self.active[i])
    }
}

/// A document read once for the functions above.
pub(crate) struct Analysis<'a> {
    pub doc: Doc,
    pub context: &'a Context,
    pub elements: Vec<Element>,
    /// Per element: calculated.
    pub calculated: Vec<bool>,
    pub calls: Vec<calls::Call>,
    pub units: Vec<units::Custom>,
    around: OnceCell<Vec<Part>>,
}

impl<'a> Analysis<'a> {
    pub fn new(document: &str, context: &'a Context) -> Analysis<'a> {
        let doc = Doc::new(document);
        let calls = calls::calls(&doc);
        let elements = scan(&doc, &calls);
        let mut units = Vec::new();
        units::custom_units(&context.text, &mut units);
        units::custom_units(document, &mut units);
        for u in &context.units {
            if !units.iter().any(|c| c.name == u.name) {
                units.push(units::Custom { name: u.name.clone(), value: u.value.clone() });
            }
        }
        let integration = integration(document, context);
        let calculated = activity(&doc, &elements, &calls, &context.reported_in(&context.path), integration);
        Analysis { doc, context, elements, calculated, calls, units, around: OnceCell::new() }
    }

    fn index(&self, e: &Element) -> Option<usize> {
        self.elements.binary_search_by_key(&e.start, |x| x.start).ok()
    }

    /// Whether solveng calculates an element; see [`activity`].
    pub fn active(&self, e: &Element) -> bool {
        self.index(e).is_some_and(|i| self.calculated[i])
    }

    /// Whether an element is solveng to the editor: calculated, or code tagged `solveng`, which is the language
    /// wherever it is.
    pub fn shown(&self, e: &Element) -> bool {
        (!e.math && e.tag == TAG) || self.active(e)
    }

    /// The files around, read when first needed: one by one when known, else their join.
    pub fn around(&self) -> &[Part] {
        self.around.get_or_init(|| {
            if !self.context.files.is_empty() {
                self.context.files.iter().map(|f| Part::new(Some(f.path.clone()), &f.text, self.context)).collect()
            } else if self.context.text.is_empty() {
                Vec::new()
            } else {
                vec![Part::new(None, &self.context.text, self.context)]
            }
        })
    }

    pub fn is_custom_unit(&self, word: &str) -> bool {
        self.units.iter().any(|u| u.name == word)
    }
}
