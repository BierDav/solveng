//! Editor support for solveng (`@preview/solveng`), the calculation language of Typst documents: ```` ```solveng ````
//! code blocks, math in a `worksheet` (unless the innermost one says `evaluate: false`) and equations and code
//! passed to `#worksheet(…)`, and explicit unit and operation names. Like Mathcad, `:=` defines, a
//! trailing `=` shows a value and `->` a symbolic result or a solution, each followed by a clause of a unit and
//! keywords.
//!
//! Everything is a pure function of the document and a [`Context`] (the files around it, what the compiler
//! reported). Offsets are UTF-16 code units, as editors and LSP count them.

mod bindings;
mod calls;
mod clause;
mod complete;
mod conceal;
mod convert;
mod depends;
mod errors;
mod format;
mod elements;
mod highlight;
mod locals;
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

pub use bindings::unit_fixes;
pub use format::format;
pub use complete::complete;
pub use conceal::{conceals, conceals_of, resolved_name};
pub use convert::{conversions, from_math, to_math};
pub use navigate::{definition, rename, rename_target, usages};
pub use highlight::{diagnostics, document_tokens, hover, math_tokens, math_tokens_of, pretty, regions, tokens, tokens_with_units};
pub use report::{Report, ReportElement, ReportError, ReportValue};
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
    /// The `=` or `->` that shows a result (`$t := Q_0/I = 1 h$`), set bold as Mathcad does. Editors without it show
    /// an operator.
    pub const EVALUATE: u8 = 11;
    /// A call to a user function, shown in the normal text colour.
    pub const FUNCTION_CALL: u8 = 12;
    /// A built-in calculation function, shown as a predefined symbol.
    pub const BUILTIN_FUNCTION: u8 = 13;
    /// The gray hash of an explicit Typst unit binding reference.
    pub const UNIT_REFERENCE: u8 = 14;
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
    /// Edits elsewhere applied with it, like the `#import units: km` a shorthand needs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edits: Vec<Edit>,
}

impl Item {
    pub(crate) fn new(label: &str, kind: u8, detail: &str, insert: &str) -> Item {
        Item { label: label.into(), kind, detail: detail.into(), insert: insert.into(), documentation: None, filter: None, edits: Vec::new() }
    }
}

/// Completions at a caret: `items` replace the UTF-16 range `from..<(to.unwrap_or(offset))`. `exclusive` ones (in solveng code
/// and strings) replace the Typst language server's; others (in math, parameter names) come first next to its.
#[derive(Clone, Debug, Serialize)]
pub struct Completions {
    pub from: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<u32>,
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
    /// Edits elsewhere applied with it, like an import.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub edits: Vec<Edit>,
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

/// A problem in the document: an inspection found while typing (unknown units, redefinitions or scope surprises),
/// or an error the last compile reported where it still applies.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Problem {
    pub start: u32,
    pub end: u32,
    pub message: String,
    /// `warning` for inspections, where the compile has the last word; `error` for what the compile reported.
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
    /// Whether the compiler calculated math in the document, which beats reading the settings.
    pub evaluate: Option<bool>,
    /// Values the compiler computed, used as completion details.
    pub defined: Vec<ReportValue>,
    /// Units the `worksheet(units: …)` option set, as the compiler saw them.
    pub units: Vec<ReportValue>,
    /// The document's path, relative to the root, as given to [`Context::with_report`].
    pub path: String,
    /// The equations and raw elements the compile saw, in every file; empty without a report.
    pub reported: Vec<ReportElement>,
    /// The files around one by one, when known; `text` joins them.
    pub files: Vec<ContextFile>,
    /// The compiled main file, relative to the root: where the compile starts, for which definition a name means.
    pub main: Option<String>,
    /// What the compile reported as going wrong, in every file; empty without a report.
    pub errors: Vec<ReportError>,
}

const COMPILED: &str = "\u{1}solveng-evaluate:";

impl Context {
    /// The text of the files around; one starting like [`with_compiled`]'s carries whether the compiler calculated math.
    pub fn new(text: &str) -> Context {
        if let Some(rest) = text.strip_prefix(COMPILED)
            && let Some(d) = rest.chars().next().and_then(|c| c.to_digit(10)) {
                let text = rest.split_once('\n').map_or("", |(_, t)| t);
                return Context { text: text.into(), evaluate: Some(d != 0), ..Context::default() };
            }
        Context { text: text.into(), ..Context::default() }
    }

    /// Adds what the compile reported for the document at `path` (relative to the root); `main` is the compiled file.
    pub fn with_report(mut self, report: &Report, path: &str, main: bool) -> Context {
        if let Some(e) = report.evaluates(path, main) {
            self.evaluate = Some(e);
        }
        self.defined = report.defined.clone();
        self.units = report.units.clone();
        self.path = path.into();
        if main && self.main.is_none() {
            self.main = Some(path.into());
        }
        self.reported = if report.blocks == 0 { Vec::new() } else { report.elements.clone() };
        self.errors = report.errors.clone();
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

/// `context` carrying whether the compiler calculated math in the document (1 or 0), unless that's negative;
/// [`Context::new`] reads it back. For editors that pass the context around as one string.
pub fn with_compiled(context: &str, evaluate: i32) -> String {
    if evaluate < 0 { context.into() } else { format!("{COMPILED}{}\n{context}", i32::from(evaluate != 0)) }
}

/// What the compile reported for the file at `path`: 1 when it calculated math, 0 when not, -1 when the report
/// can't tell.
pub fn compiled_evaluate(report: &str, path: &str, main: bool) -> i32 {
    Report::parse(report).evaluates(path, main).map_or(-1, i32::from)
}

static REFERENCE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"#(?:import|include)\s+"([^"@][^"]*)""#).unwrap());
static IMPLICIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"implicit-parameters\s*:\s*"(no|empty|allow)""#).unwrap());

/// What a text sets with `implicit-parameters: …`, the last one: `"empty"` makes `f()` take what
/// `f(..)` does, `"allow"` every function; None without a setting.
pub(crate) fn implicit_parameters(text: &str) -> Option<String> {
    IMPLICIT.captures_iter(text).last().map(|m| m[1].to_string())
}

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

/// Whether the document's math is calculated: in a `worksheet`, nothing without one. The compiler's answer wins;
/// a `worksheet(evaluate: false)[…]` still leaves the math in it alone, see [`activity`].
pub fn evaluates(document: &str, context: &Context) -> bool {
    context.evaluate.unwrap_or_else(|| {
        let doc = Doc::new(document);
        let aliases = calls::aliases(&doc, &context.files, &context.path);
        let (_, scopes) = calls::read(&doc, &aliases);
        scopes.iter().any(|s| s.evaluate) || wrapped(context, &context.path) == Some(true)
    })
}

/// Whether a worksheet in a file around the one at `path` includes it where math is evaluated. Only what the file
/// itself wraps in a worksheet counts; a worksheet elsewhere in the project leaves unrelated files alone.
fn included_in_worksheet(context: &Context, path: &str) -> bool {
    if context.files.is_empty() || path.is_empty() {
        return false;
    }
    let mut seen = vec![path.to_string()];
    let mut queue = vec![path.to_string()];
    while let Some(current) = queue.pop() {
        for file in context.files.iter().filter(|f| f.path != current) {
            let doc = Doc::new(&file.text);
            let aliases = calls::aliases(&doc, &context.files, &file.path);
            let (_, scopes) = calls::read(&doc, &aliases);
            for (reference, at) in calls::includes(&doc) {
                if crate::scope::resolve(&reference, &file.path) != current {
                    continue;
                }
                match calls::evaluates_at(&scopes, at) {
                    Some(true) => return true,
                    // Not in a worksheet of its own: the file may be included in one further out.
                    Some(false) => {}
                    None => {
                        if !seen.contains(&file.path) && seen.len() < 32 {
                            seen.push(file.path.clone());
                            queue.push(file.path.clone());
                        }
                    }
                }
            }
        }
    }
    false
}

/// Whether math in the file at `path` is evaluated from outside its own worksheets: the compile's answer when it
/// has one (`Some`, which wins), else whether a file around includes it in a worksheet.
fn wrapped(context: &Context, path: &str) -> Option<bool> {
    context.evaluate.or_else(|| {
        // Without files the surrounding source is one text: a worksheet in it may be the one this document is
        // included in, and nothing else there tells which.
        if context.files.is_empty() && !context.text.is_empty() {
            let doc = Doc::new(&context.text);
            let aliases = calls::aliases(&doc, &[], "");
            let (_, scopes) = calls::read(&doc, &aliases);
            if scopes.iter().any(|scope| scope.evaluate) {
                return Some(true);
            }
        }
        included_in_worksheet(context, path).then_some(true)
    })
}

/// Math if `math`, unless the innermost worksheet around it says otherwise; of raw elements only ```` ```solveng ````
/// code blocks, wherever they are.
pub(crate) fn is_solveng(e: &Element, math: Option<bool>, scopes: &[calls::Scope]) -> bool {
    if e.math {
        // The compile reached the file and found no math in a worksheet: it has the last word.
        if math == Some(false) {
            return false;
        }
        calls::evaluates_at(scopes, e.start).or(math).unwrap_or(false)
    } else {
        e.tag == TAG && e.block
    }
}

/// The UTF-16 `(body_start, body_end)` of a raw element that holds solveng code: a ```` ```solveng ```` block.
pub fn body(raw: &str) -> Option<(u32, u32)> {
    let doc = Doc::new(raw);
    let r = elements::elements(&doc.c).into_iter().next()?;
    if r.start != 0 || r.math || !is_solveng(&r, None, &[]) {
        return None;
    }
    Some((doc.to16(r.body_start), doc.to16(r.body_end)))
}

/// Raw elements and equations outside the strings of solveng's calls, where a `$` or a backtick is just text.
pub(crate) fn scan(doc: &Doc, _calls: &[calls::Call]) -> Vec<Element> {
    use typst_syntax::{LinkedNode, SyntaxKind};
    fn mask(node: &LinkedNode, doc: &Doc, c: &mut [char]) {
        if node.kind() == SyntaxKind::Str || node.kind() == SyntaxKind::Error && node.get().clone().full_text().starts_with('"') {
            for ch in &mut c[doc.of_byte(node.offset())..doc.of_byte(node.offset() + node.len())] {
                if *ch == '$' || *ch == '`' { *ch = ' '; }
            }
        } else {
            for child in node.children() { mask(&child, doc, c); }
        }
    }
    let mut c = doc.c.clone();
    let root = typst_syntax::parse(&doc.s);
    mask(&LinkedNode::new(&root), doc, &mut c);
    elements::elements(&c)
}

/// Which elements solveng calculates. Without the compiler's elements for the file (`reported`) the worksheets
/// written in the source decide, and `math` for what none of them covers.
///
/// With them, an element whose source matches one takes its flag; copies of the same source are aligned in order,
/// preferring the copy whose worksheets agree with the report, then the least movement. A worksheet written around
/// an element decides for one written since the compile (and `evaluate: false` there wins over a stale report).
/// Outside every written worksheet, a template solveng can't read may hold the worksheet: an element there is
/// calculated like the last calculated element before it outside written worksheets, if that one's block holds it,
/// else when `included` says a worksheet of another file includes this one.
/// Equations and code passed to `worksheet` are calculated wherever they are.
pub(crate) fn activity(
    doc: &Doc,
    elements: &[Element],
    calls: &[calls::Call],
    scopes: &[calls::Scope],
    reported: &[&ReportElement],
    math: Option<bool>,
    included: &dyn Fn() -> bool,
) -> Vec<bool> {
    let in_call = |e: &Element| calls.iter().any(|c| c.maths.iter().any(|m| m.0 == e.start));
    if reported.is_empty() {
        return elements.iter().map(|e| is_solveng(e, math, scopes) || in_call(e)).collect();
    }
    // What the worksheets written around say: math as their `evaluate`, solveng code blocks always, other raw never.
    let written: Vec<Option<bool>> = elements.iter().map(|e| {
        let around = calls::evaluates_at(scopes, e.start);
        if e.math { around } else if e.tag == TAG && e.block { around.map(|_| true) } else { Some(false) }
    }).collect();
    // Disagreeing with the written worksheets costs more than any movement; being calculated outside every one
    // (only a template explains it) less so.
    let cost = |i: usize, j: usize| {
        let r = reported[j];
        let moved = (doc.to16(elements[i].start) as usize).abs_diff(r.start) as u64;
        let penalty = match written[i] {
            Some(evaluates) if evaluates != r.active && !in_call(&elements[i]) => 1 << 40,
            None if r.active && !in_call(&elements[i]) => 1 << 32,
            _ => 0,
        };
        moved + penalty
    };
    let matches = report::match_elements(doc, elements, reported, &cost);
    let blocks = calls::blocks(doc);
    // The innermost block around a position; the whole document outside every one.
    let block = |at: usize| blocks.iter().filter(|b| b.0 < at && at < b.1).min_by_key(|b| b.1 - b.0).copied().unwrap_or((0, doc.c.len()));
    // Matched elements outside written worksheets: their block and, when calculated, the worksheet's `evaluate`.
    let mut unwritten: Vec<((usize, usize), Option<bool>)> = Vec::new();
    let mut included_once: Option<bool> = None;
    elements
        .iter()
        .enumerate()
        .map(|(index, e)| {
            let active = match (matches[index], written[index]) {
                (Some(i), written) => {
                    if written.is_none() {
                        unwritten.push((block(e.start), reported[i].active.then_some(reported[i].evaluate)));
                    }
                    reported[i].active && !(e.math && written == Some(false))
                }
                (None, Some(evaluates)) => evaluates,
                (None, None) => match unwritten.iter().rev().find(|(b, _)| b.0 <= e.start && e.end <= b.1) {
                    Some((_, evaluate)) => evaluate.is_some_and(|evaluate| evaluate || !e.math),
                    None => *included_once.get_or_insert_with(included),
                },
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
        let aliases = if text.is_empty() { calls::Aliases::new() } else { calls::aliases(&doc, &context.files, path.as_deref().unwrap_or("")) };
        let (calls, scopes) = if text.is_empty() { Default::default() } else { calls::read(&doc, &aliases) };
        let elements = scan(&doc, &calls);
        let reported = path.as_deref().map(|p| context.reported_in(p)).unwrap_or_default();
        // Included where a worksheet of another file evaluates math.
        let math = path.as_deref().filter(|p| included_in_worksheet(context, p)).map(|_| true);
        let active = activity(&doc, &elements, &calls, &scopes, &reported, math, &|| math == Some(true));
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
    unit_equations: Vec<(usize, usize)>,
    pub bindings: bindings::Bindings,
    unit_scopes: Vec<calls::Scope>,
    unit_context: Vec<units::Custom>,
    reported_units: Vec<units::Custom>,
    around: OnceCell<Vec<Part>>,
}

impl<'a> Analysis<'a> {
    pub fn new(document: &str, context: &'a Context) -> Analysis<'a> {
        let doc = Doc::new(document);
        let aliases = calls::aliases(&doc, &context.files, &context.path);
        let (calls, scopes) = calls::read(&doc, &aliases);
        let elements = scan(&doc, &calls);
        let mut units = Vec::new();
        units::custom_units(&context.text, &mut units);
        let unit_context = units.clone();
        units::custom_units(document, &mut units);
        // The compiler's units the text doesn't define, e.g. from a package.
        let reported_units: Vec<units::Custom> = context.units.iter().filter(|u| !units.iter().any(|c| c.name == u.name))
            .map(|u| units::Custom { name: u.name.clone(), value: u.value.clone() }).collect();
        units.extend(reported_units.iter().cloned());
        let math = wrapped(context, &context.path);
        let calculated = activity(&doc, &elements, &calls, &scopes, &context.reported_in(&context.path), math,
            &|| included_in_worksheet(context, &context.path));
        let bindings = bindings::Bindings::new(&doc);
        let entries = units::entries(document);
        let unit_equations = elements.iter().filter(|e| e.math && entries.iter().any(|entry| entry.start < doc.sub(0, e.start).len() && doc.sub(0, e.end).len() <= entry.value_end)).map(|e| (e.start, e.end)).collect();
        Analysis { doc, context, elements, calculated, calls, units, unit_equations, bindings, unit_scopes: scopes, unit_context, reported_units, around: OnceCell::new() }
    }

    fn index(&self, e: &Element) -> Option<usize> {
        self.elements.binary_search_by_key(&e.start, |x| x.start).ok()
    }

    /// Whether solveng calculates an element; see [`activity`].
    pub fn active(&self, e: &Element) -> bool {
        self.index(e).is_some_and(|i| self.calculated[i])
    }

    /// Whether an element is solveng to the editor: calculated, or a ```` ```solveng ```` block, which is the
    /// language wherever it is.
    pub fn shown(&self, e: &Element) -> bool {
        (!e.math && e.tag == TAG && e.block) || self.active(e) || self.unit_equations.contains(&(e.start, e.end))
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

    pub fn builtin_functions_at(&self, at: usize) -> bool {
        let mut enabled = true;
        let mut scopes: Vec<_> = self.unit_scopes.iter().filter(|s| s.start <= at && at < s.end).collect();
        scopes.sort_by_key(|s| s.start);
        for scope in scopes { if let Some(value) = scope.builtin_functions { enabled = value; } }
        for call in self.calls.iter().filter(|c| c.args_start <= at && at <= c.args_end) {
            for (name, value) in &call.args {
                if name.as_deref() == Some("builtin-functions") {
                    if let Ok(value) = value.trim().parse() { enabled = value; }
                }
            }
        }
        enabled
    }

    /// String contents of explicit constructors. Quotes stay ordinary Typst syntax.
    pub fn reference_strings(&self) -> Vec<(usize, usize)> {
        self.bindings.operator_strings().iter().map(|(s, e, _)| (*s, *e))
            .chain(self.calls.iter().filter(|call| call.function.name == "unit")
                .flat_map(|call| call.texts.iter().filter(|t| t.param.is_none()).map(|t| (t.start, t.end)))).collect()
    }

    pub fn evaluates_at(&self, at: usize) -> bool {
        self.elements.iter().any(|e| e.math && self.active(e) && e.body_start <= at && at < e.body_end)
            || calls::evaluates_at(&self.unit_scopes, at) == Some(true)
    }

    /// The units in effect at `at`: the files around's and the compiler's, then each worksheet around from the
    /// outside in. An isolated one starts over with only the built-in units (if `builtin-units` is still on).
    /// `explicit`: for `unit("…")`, which still reaches names removed with `none`.
    fn local_units_at(&self, at: usize, explicit: bool) -> Vec<units::Custom> {
        let mut units = self.unit_context.clone();
        units.extend(self.reported_units.iter().filter(|u| !self.unit_context.iter().any(|c| c.name == u.name)).cloned());
        let mut scopes = self.unit_scopes.iter().filter(|s| s.start <= at && at < s.end).collect::<Vec<_>>();
        scopes.sort_by_key(|s| s.start);
        for scope in scopes {
            if scope.isolated {
                units.retain(|u| u.name.is_empty());
            }
            for unit in &scope.units {
                if explicit && unit.value == "none" { continue; }
                units.retain(|u| u.name != unit.name);
                units.push(unit.clone());
            }
        }
        for call in self.calls.iter().filter(|c| c.args_start <= at && at <= c.args_end) {
            for (name, value) in &call.args {
                if matches!(name.as_deref(), Some("units" | "builtin-units")) {
                    units::custom_units(&format!("{}: {value}", name.as_deref().unwrap()), &mut units);
                }
            }
        }
        units
    }

    pub fn unit_completion_context(&self, at: usize) -> Vec<units::Custom> { self.local_units_at(at, true) }

    pub fn units_at(&self, at: usize) -> Vec<units::Custom> {
        self.local_units_at(at, false)
    }

    pub fn is_custom_unit(&self, word: &str) -> bool {
        self.units.iter().any(|u| u.name == word)
    }
}
