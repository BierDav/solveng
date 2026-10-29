//! Which definition a name means, as the compile sees it: the document flattened in compile order from the main
//! file (`#include` inlines a file), `worksheet` scopes, `inherit`, `export`, `clear-at`, `clear` and redefinitions.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use typst_syntax::{LinkedNode, SyntaxKind, ast};

use crate::calls::callee;
use crate::complete::Definition;
use crate::text::Doc;

/// Structure of a file that decides what's visible, at character positions.
#[derive(Clone, Debug)]
enum Mark {
    /// A `worksheet` starts; `clear_at` is where it starts over.
    Start { inherit: bool, export: bool, clear_at: Option<ClearAt> },
    End,
    /// `#clear(all: true)`.
    Reset,
    /// `#clear("x", $f$)`: the keys it forgets.
    Clear(Vec<String>),
    /// A heading of this level.
    Heading(usize),
    /// An element with this label (`<exercise>`, without the brackets).
    Label(String),
    /// `#include "path"`, as written.
    Include(String),
}

/// What `clear-at` takes: `heading`, `heading.where(level: n)` or a label.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ClearAt {
    Heading(Option<usize>),
    Label(String),
}

impl ClearAt {
    /// `clear-at`'s value as written; None for `none` and anything solveng rejects.
    pub(crate) fn parse(value: &str) -> Option<ClearAt> {
        let value = value.trim();
        if value == "heading" {
            return Some(ClearAt::Heading(None));
        }
        if let Some(m) = LEVEL.captures(value) {
            return m[1].parse().ok().filter(|l| *l > 0).map(|l| ClearAt::Heading(Some(l)));
        }
        let label = value.strip_prefix('<')?.strip_suffix('>')?;
        (!label.is_empty() && label.chars().all(|c| c.is_alphanumeric() || "_-.:".contains(c))).then(|| ClearAt::Label(label.into()))
    }
}

/// What a worksheet's `scope` makes of the definitions around it: (sees the outer ones, they keep its own).
pub(crate) fn scope_flags(scope: Option<&str>, expression: bool) -> (bool, bool) {
    match scope.map(|s| s.trim().trim_matches('"')) {
        Some("isolated") => (false, false),
        Some("inherit") => (true, false),
        Some("shared") => (true, true),
        _ if expression => (true, true),
        _ => (false, false),
    }
}

static LEVEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^heading\s*\.\s*where\s*\(\s*level\s*:\s*(\d+)\s*\)$").unwrap());

/// The named arguments of a call's arguments: name, value as written.
fn named_args(args: &LinkedNode) -> impl Iterator<Item = (String, String)> {
    args.children().filter(|c| c.kind() == SyntaxKind::Named).filter_map(|named| {
        let mut parts = named.children().filter(|c| !c.kind().is_trivia() && c.kind() != SyntaxKind::Colon);
        let (Some(name), Some(value)) = (parts.next(), parts.next()) else { return None };
        Some((name.get().leaf_text().to_string(), value.get().clone().full_text().trim().to_string()))
    })
}

/// The value of the named argument `name`, as written.
fn named(args: &LinkedNode, name: &str) -> Option<String> {
    named_args(args).find(|(n, _)| n == name).map(|(_, v)| v)
}

/// Scope options for worksheet content: `scope` (isolated by default) and `clear-at`.
fn options(args: Option<LinkedNode>) -> (bool, bool, Option<ClearAt>) {
    let scope = args.as_ref().and_then(|a| named(a, "scope"));
    let (inherit, export) = scope_flags(scope.as_deref(), false);
    let reset = args.as_ref().and_then(|a| named(a, "clear-at")).and_then(|v| ClearAt::parse(&v));
    (inherit, export, reset)
}

/// The key of a name `clear` takes: `"x"`, `"f()"`, `$Q_0$`; None for other arguments.
fn clear_key(arg: &LinkedNode) -> Option<String> {
    let text = match arg.kind() {
        SyntaxKind::Str => arg.get().cast::<ast::Str>()?.get().to_string(),
        SyntaxKind::Equation => arg.get().clone().full_text().trim_matches('$').to_string(),
        _ => return None,
    };
    let name = text.split('(').next().unwrap_or("").trim().trim_matches('"');
    (!name.is_empty()).then(|| crate::conceal::resolved_name(name))
}

/// A content block whose only emitted content is one equation collapses to
/// that equation in Typst. Spaces are meaningful content; comments are not.
pub(crate) fn expression_block(body: &LinkedNode) -> bool {
    if !body.children().any(|c| c.kind() == SyntaxKind::RightBracket) { return false; }
    let Some(markup) = body.children().find(|c| c.kind() == SyntaxKind::Markup) else { return false; };
    let mut children = markup.children().filter(|c| !matches!(c.kind(), SyntaxKind::LineComment | SyntaxKind::BlockComment));
    matches!(children.next().map(|c| c.kind()), Some(SyntaxKind::Equation)) && children.next().is_none()
}

struct Reader<'a> {
    warnings: Vec<crate::Problem>,
    doc: &'a Doc,
    out: Vec<(usize, Mark)>,
}

impl Reader<'_> {
    fn at(&self, byte: usize) -> usize {
        self.doc.of_byte(byte)
    }

    fn visit(&mut self, node: &LinkedNode) {
        match node.kind() {
            SyntaxKind::FuncCall => {
                let name = node.children().next().and_then(|c| callee(c.get()));
                let args = node.children().find(|c| c.kind() == SyntaxKind::Args);
                match name.as_deref() {
                    Some("worksheet") => {
                        let body = args.as_ref().and_then(|a| a.children().find(|c| c.kind() == SyntaxKind::ContentBlock));
                        if let Some(body) = body {
                            let expression = expression_block(&body);
                            let (inherit, export, reset) = if expression {
                                let scope = args.as_ref().and_then(|a| named(a, "scope"));
                                if scope.as_deref().is_none_or(|value| value == "auto") {
                                    self.warnings.push(crate::Problem {
                                        start: self.doc.to16(self.at(body.offset())),
                                        end: self.doc.to16(self.at(body.offset() + body.len())),
                                        message: "this worksheet block contains only one expression, so Typst passes it as an equation: scope: auto shares its definitions with the outer scope – set scope: \"isolated\" to keep them local, or scope: \"shared\" to make this explicit".into(),
                                        severity: "warning",
                                    });
                                }
                                let (inherit, export) = scope_flags(scope.as_deref(), true);
                                (inherit, export, None)
                            } else { options(args) };
                            self.out.push((self.at(body.offset() + 1), Mark::Start { inherit, export, clear_at: reset }));
                            self.out.push((self.at(body.offset() + body.len()).saturating_sub(1), Mark::End));
                        } else if let Some(args) = args.as_ref() {
                            // Equations passed to a worksheet share its scope unless `scope` says otherwise.
                            let (inherit, export) = scope_flags(named(args, "scope").as_deref(), true);
                            if !(inherit && export) {
                                self.out.push((self.at(args.offset()), Mark::Start { inherit, export, clear_at: None }));
                                self.out.push((self.at(args.offset() + args.len()), Mark::End));
                            }
                        }
                    }
                    Some("clear") => {
                        let Some(args) = args else { return };
                        // after the call, so the names in it still mean what they forget
                        let at = self.at(node.offset() + node.len());
                        if named(&args, "all").as_deref() == Some("true") {
                            self.out.push((at, Mark::Reset));
                        } else {
                            let keys = args.children().filter_map(|a| clear_key(&a)).collect();
                            self.out.push((at, Mark::Clear(keys)));
                        }
                    }
                    _ => {}
                }
            }
            SyntaxKind::ShowRule => self.show(node),
            SyntaxKind::Heading => {
                let level = node.children().find(|c| c.kind() == SyntaxKind::HeadingMarker).map_or(1, |m| m.len());
                self.out.push((self.at(node.offset()), Mark::Heading(level)));
            }
            SyntaxKind::Label => {
                let name = node.get().leaf_text().trim_start_matches('<').trim_end_matches('>').to_string();
                self.out.push((self.at(labelled(node).offset()), Mark::Label(name)));
            }
            SyntaxKind::ModuleInclude => {
                if let Some(path) = node.children().find(|c| c.kind() == SyntaxKind::Str) {
                    let text = path.get().leaf_text();
                    self.out.push((self.at(node.offset()), Mark::Include(text.trim_matches('"').to_string())));
                }
            }
            _ => {}
        }
        for child in node.children() {
            self.visit(&child);
        }
    }

    /// `#show: worksheet` (or `.with(…)`) runs to the end of the content around it.
    fn show(&mut self, node: &LinkedNode) {
        let parts: Vec<LinkedNode> = node.children().filter(|c| !c.kind().is_trivia()).collect();
        let Some(colon) = parts.iter().position(|c| c.kind() == SyntaxKind::Colon) else { return };
        // `show` and the colon only: no selector.
        if colon != 1 {
            return;
        }
        let Some(transform) = parts.get(colon + 1) else { return };
        let args = match transform.kind() {
            SyntaxKind::Ident if transform.get().leaf_text() == "worksheet" => None,
            SyntaxKind::FuncCall if transform.children().next().and_then(|c| callee(c.get())).as_deref() == Some("worksheet") => {
                transform.children().find(|c| c.kind() == SyntaxKind::Args)
            }
            _ => return,
        };
        let (inherit, export, reset) = options(args);
        // The markup around it; in a content block that ends before its `]`.
        let end = node.parent().map_or(self.doc.s.len(), |p| p.offset() + p.len());
        self.out.push((self.at(node.offset() + node.len()), Mark::Start { inherit, export, clear_at: reset }));
        self.out.push((self.at(end), Mark::End));
    }
}

/// The element a label in markup labels: the heading it ends, else the element before it.
fn labelled<'a>(label: &LinkedNode<'a>) -> LinkedNode<'a> {
    let heading = label.parent().and_then(|markup| markup.parent()).filter(|p| p.kind() == SyntaxKind::Heading);
    if let Some(heading) = heading {
        return heading.clone();
    }
    let mut previous = label.prev_sibling();
    while let Some(p) = previous.clone().filter(|p| matches!(p.kind(), SyntaxKind::Space | SyntaxKind::Parbreak)) {
        previous = p.prev_sibling();
    }
    previous.unwrap_or_else(|| label.clone())
}

fn structure(doc: &Doc) -> Vec<(usize, Mark)> {
    let root = typst_syntax::parse(&doc.s);
    let mut reader = Reader { doc, out: Vec::new(), warnings: Vec::new() };
    reader.visit(&LinkedNode::new(&root));
    reader.out
}

/// Scope surprises that can be determined from the source without compiling it.
pub(crate) fn export_warnings(doc: &Doc) -> Vec<crate::Problem> {
    let root = typst_syntax::parse(&doc.s);
    let mut reader = Reader { doc, out: Vec::new(), warnings: Vec::new() };
    reader.visit(&LinkedNode::new(&root));
    reader.warnings
}

/// One file of the flattened document: its path relative to the root (empty for the document without one), its
/// text, definitions and the names solveng reads in it (key, start, end in characters).
pub(crate) struct File<'a> {
    pub path: String,
    pub doc: &'a Doc,
    pub definitions: Vec<Definition>,
    pub names: Vec<(String, usize, usize)>,
    /// Equation bodies and the strings of solveng's calls, where a name is spelled differently than in code.
    pub maths: Vec<(usize, usize)>,
    pub strings: Vec<(usize, usize)>,
    /// Per graph call: its curves, and the names they give curves (`U` in `U(x) = …`), which aren't plotted against.
    pub graphs: Vec<(Vec<(usize, usize)>, Vec<(usize, usize)>)>,
    /// Functions defined where `definitions` doesn't look, like graph curves: name start, name end, end.
    pub functions: Vec<(usize, usize, usize)>,
    /// Sum index declarations and the summands they bind.
    pub locals: Vec<crate::locals::Local>,
    /// The Typst bindings it has: which clause words are modifiers.
    pub bindings: &'a crate::bindings::Bindings,
}

/// A name in the flattened document: in `file` (an index into the files), and the definition it means (an index
/// into the occurrences; a definition means itself).
#[derive(Clone, Debug)]
pub(crate) struct Occurrence {
    pub key: String,
    pub file: usize,
    pub start: usize,
    pub end: usize,
    pub definition: Option<usize>,
}

#[derive(Clone)]
enum Event {
    Mark(Mark),
    /// A definition of a key, by its index in the file's definitions.
    Define(usize),
    /// A name, by its index in the file's names.
    Use(usize),
    /// Where the caret is: what's visible there is kept.
    Probe,
}

/// The events of a file in order. At one position a block ends before anything else and starts after it; a
/// definition takes effect at its statement's end, after the names in it.
fn events(file: &File) -> Vec<(usize, u8, Event)> {
    let mut out: Vec<(usize, u8, Event)> = structure(file.doc)
        .into_iter()
        .map(|(at, mark)| {
            let rank = match mark {
                Mark::End => 0,
                Mark::Start { .. } => 3,
                _ => 1,
            };
            (at, rank, Event::Mark(mark))
        })
        .collect();
    let defined: Vec<usize> = file.definitions.iter().map(|d| d.start).collect();
    for (i, (_, start, _)) in file.names.iter().enumerate() {
        if !defined.contains(start) {
            out.push((*start, 2, Event::Use(i)));
        }
    }
    for (i, d) in file.definitions.iter().enumerate() {
        out.push((d.statement_end, 2, Event::Define(i)));
    }
    out.sort_by_key(|(at, rank, _)| (*at, *rank));
    out
}

/// A path as written in `#include`, from the file at `from`, normalized; `/` starts at the root.
pub(crate) fn resolve(reference: &str, from: &str) -> String {
    let joined = match reference.strip_prefix('/') {
        Some(rest) => rest.to_string(),
        None => match from.rfind('/') {
            Some(i) => format!("{}/{reference}", &from[..i]),
            None => reference.to_string(),
        },
    };
    let mut parts: Vec<&str> = Vec::new();
    for p in joined.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.join("/")
}

struct Frame {
    outer: HashMap<String, usize>,
    start: HashMap<String, usize>,
    reset: Option<ClearAt>,
    export: bool,
}

struct Run<'f, 'a> {
    files: &'f [File<'a>],
    events: Vec<Option<Vec<(usize, u8, Event)>>>,
    visiting: Vec<usize>,
    reached: Vec<bool>,
    visible: HashMap<String, usize>,
    stack: Vec<Frame>,
    out: Vec<Occurrence>,
    statements: Vec<(usize, usize)>,
    replaced: Vec<((usize, usize), (usize, usize))>,
    /// The document's index and the caret in it, for [`visible_at`].
    document: usize,
    probe: Option<usize>,
    probed: Option<HashMap<String, usize>>,
}

impl Run<'_, '_> {
    fn new<'f, 'a>(files: &'f [File<'a>], document: usize, probe: Option<usize>) -> Run<'f, 'a> {
        Run {
            files,
            events: vec![None; files.len()],
            visiting: Vec::new(),
            reached: vec![false; files.len()],
            visible: HashMap::new(),
            stack: Vec::new(),
            out: Vec::new(),
            statements: Vec::new(),
            replaced: Vec::new(),
            document,
            probe,
            probed: None,
        }
    }

    fn restart(&mut self) {
        self.visible = self.stack.last().map_or_else(HashMap::new, |f| f.start.clone());
    }

    fn file(&mut self, index: usize) {
        if self.visiting.contains(&index) || self.visiting.len() > 32 {
            return;
        }
        self.visiting.push(index);
        self.reached[index] = true;
        let mut events = self.events[index].get_or_insert_with(|| events(&self.files[index])).clone();
        if let Some(at) = self.probe.filter(|_| index == self.document) {
            // after marks at the caret, before definitions ending there
            let i = events.partition_point(|(p, rank, _)| (*p, *rank) <= (at, 1));
            events.insert(i, (at, 1, Event::Probe));
        }
        let file = &self.files[index];
        for (_, _, event) in events {
            match event {
                Event::Mark(Mark::Start { inherit, export, clear_at: reset }) => {
                    let start = if inherit { self.visible.clone() } else { HashMap::new() };
                    let outer = std::mem::replace(&mut self.visible, start.clone());
                    self.stack.push(Frame { outer, start, reset, export });
                }
                Event::Mark(Mark::End) => {
                    if let Some(frame) = self.stack.pop() {
                        let inner = std::mem::replace(&mut self.visible, frame.outer);
                        if frame.export { self.visible.extend(inner); }
                    }
                }
                Event::Mark(Mark::Reset) => self.restart(),
                Event::Mark(Mark::Clear(keys)) => {
                    for key in keys {
                        self.visible.remove(&key);
                    }
                }
                Event::Mark(Mark::Heading(level)) => {
                    if self.stack.iter().any(|f| matches!(f.reset, Some(ClearAt::Heading(r)) if r.is_none_or(|l| l == level))) {
                        self.restart();
                    }
                }
                Event::Mark(Mark::Label(label)) => {
                    if self.stack.iter().any(|f| f.reset.as_ref() == Some(&ClearAt::Label(label.clone()))) {
                        self.restart();
                    }
                }
                Event::Probe => {
                    if index == self.document {
                        self.probed = Some(self.visible.clone());
                    }
                }
                Event::Mark(Mark::Include(path)) => {
                    let path = resolve(&path, &file.path);
                    if let Some(i) = self.files.iter().position(|f| !f.path.is_empty() && f.path == path) {
                        self.file(i);
                    }
                }
                Event::Define(i) => {
                    let d = &file.definitions[i];
                    // = on a name the scope has states something about it: a use, := assigns anew
                    if !d.assign && let Some(&known) = self.visible.get(d.base()) {
                        self.statements.push((index, d.start));
                        self.out.push(Occurrence { key: d.base().to_string(), file: index, start: d.start, end: d.end, definition: Some(known) });
                        continue;
                    }
                    if let Some(&before) = self.visible.get(d.base()) {
                        let o = &self.out[before];
                        self.replaced.push(((o.file, o.start), (index, d.start)));
                    }
                    let id = self.out.len();
                    self.out.push(Occurrence { key: d.base().to_string(), file: index, start: d.start, end: d.end, definition: Some(id) });
                    self.visible.insert(d.base().to_string(), id);
                }
                Event::Use(i) => {
                    let (key, start, end) = &file.names[i];
                    let definition = self.visible.get(key).copied();
                    self.out.push(Occurrence { key: key.clone(), file: index, start: *start, end: *end, definition });
                }
            }
        }
        self.visiting.pop();
    }
}

/// Every name of the flattened document with the definition it means: from `main` (an index into `files`) when
/// it reaches `document`, else the document alone.
/// `implicit`: what `implicit-parameters: …` says, `"empty"` makes `f()` take what `f(..)` does,
/// `"allow"` every function.
pub(crate) fn occurrences(files: &[File], document: usize, main: Option<usize>, implicit: String) -> (Vec<Occurrence>, Params) {
    let run = |start: usize| {
        let mut run = Run::new(files, document, None);
        run.file(start);
        (run.out, run.reached[document], run.statements, run.replaced)
    };
    let (mut out, statements, replaced) = match main.map(run) {
        Some((out, true, statements, replaced)) => (out, statements, replaced),
        _ => {
            let (out, _, statements, replaced) = run(document);
            (out, statements, replaced)
        }
    };
    let locals = local_indices(files, &mut out);
    let mut params = parameters(files, &mut out, &implicit, &locals);
    params.locals = locals;
    params.statements = statements;
    params.replaced = replaced;
    (out, params)
}

/// The definitions visible at character `at` of the document, as the compile sees them: (file index, start) of
/// each, and per file whether the run read it (definitions of files it didn't read can't be judged). None when
/// the document isn't reached.
pub(crate) fn visible_at(files: &[File], document: usize, main: Option<usize>, at: usize) -> Option<(Vec<(usize, usize)>, Vec<bool>)> {
    let run = |start: usize| {
        let mut run = Run::new(files, document, Some(at));
        run.file(start);
        let probed = run.probed.take()?;
        Some((probed.values().map(|&id| (run.out[id].file, run.out[id].start)).collect(), run.reached))
    };
    main.and_then(run).or_else(|| run(document))
}

/// Functions' parameters: every occurrence that is one or a use of one, and per function (file, start of its
/// name) its parameters resolved, as written (`I: mA`) and then those `..` takes.
#[derive(Default)]
pub(crate) struct Params {
    pub uses: Vec<usize>,
    /// Occurrences bound by sum indices, kept out of inferred function parameters.
    pub locals: Vec<usize>,
    /// Per use, the function it belongs to: an index into `signatures` and `names`.
    pub owners: Vec<usize>,
    pub signatures: Vec<(usize, usize, Vec<String>)>,
    /// The functions' names.
    pub names: Vec<String>,
    /// Definitions with `=` of a name their scope already has: statements about it, by file and start.
    pub statements: Vec<(usize, usize)>,
    /// Definitions a later `:=` in their scope replaces: (file, start) of the earlier one and of the later one.
    pub replaced: Vec<((usize, usize), (usize, usize))>,
}

/// Resolve outer sum binders first, then let nested binders shadow their bodies.
fn local_indices(files: &[File], out: &mut [Occurrence]) -> Vec<usize> {
    let mut uses = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let mut locals: Vec<_> = file.locals.iter().collect();
        locals.sort_by_key(|l| std::cmp::Reverse(l.body.1 - l.body.0));
        for local in locals {
            let Some(id) = out.iter().position(|o| o.file == index && (o.start, o.end) == local.declaration) else { continue };
            for (i, o) in out.iter_mut().enumerate() {
                if o.file == index && o.key == local.key && (i == id || (local.body.0 <= o.start && o.end <= local.body.1)) {
                    o.definition = Some(id);
                    if !uses.contains(&i) { uses.push(i); }
                }
            }
        }
    }
    uses
}

/// A function's parameters are its own: `x` in `f(x) = x^2` means the parameter, not a variable `x`; with `..` the
/// body's other undefined names are parameters too.
fn parameters(files: &[File], out: &mut [Occurrence], implicit: &str, locals: &[usize]) -> Params {
    let mut params = Params::default();
    for (index, file) in files.iter().enumerate() {
        let c = &file.doc.c;
        let spans = file.definitions.iter().filter(|d| d.function).map(|d| (d.start, d.end, d.statement_end)).chain(file.functions.iter().copied());
        for (start, end, statement_end) in spans {
            let Some(open) = (end..statement_end).find(|&i| c[i] == '(') else { continue };
            let Some(close) = (open..statement_end).find(|&i| c[i] == ')') else { continue };
            let list = file.doc.sub(open + 1, close);
            // .., and as implicit-parameters says: f() ("empty") or every function ("allow")
            let rest = implicit == "allow"
                || (implicit == "empty" && list.trim().is_empty())
                || list.split(',').any(|p| p.trim() == ".." || p.trim() == "…");
            let mut written: Vec<String> = list.split(',').map(str::trim).filter(|p| !p.is_empty() && *p != ".." && *p != "…").map(|p| match p.split_once(':') {
                // a unit binding's `#` is math's, not the unit's: `x: #A` takes A
                Some((name, unit)) => format!("{}: {}", crate::resolved_name(name.trim()), unit.trim().replace('#', "")),
                None => crate::resolved_name(p),
            }).collect();
            let in_body = |o: &Occurrence| o.file == index && o.start > close && o.end <= statement_end;
            let mut declared: Vec<(String, usize)> = Vec::new();
            for i in 0..out.len() {
                let o = &out[i];
                if o.file == index && o.start > open && o.end <= close {
                    let before = file.doc.sub(open + 1, o.start);
                    if before.rsplit(',').next().is_some_and(|p| p.contains(':')) { continue; }
                    declared.push((o.key.clone(), i));
                    out[i].definition = Some(i);
                    params.uses.push(i);
                    params.owners.push(params.signatures.len());
                }
            }
            let modifier = |w: &str| file.bindings.modifier(start, w);
            if let Some((_, marker_end, _)) = crate::clause::marker(&c[close + 1..statement_end], &|_| false, &modifier) {
                let clause_start = close + 1 + marker_end;
                let tail = file.doc.sub(clause_start, statement_end);
                let locals = crate::clause::solved(&tail, &modifier).into_iter().map(|n| crate::resolved_name(&n)).collect::<Vec<_>>();
                for i in 0..out.len() {
                    if out[i].file == index && out[i].start >= clause_start && out[i].end <= statement_end
                        && locals.contains(&out[i].key) && !declared.iter().any(|(k, _)| *k == out[i].key) {
                        declared.push((out[i].key.clone(), i));
                    }
                }
            }
            let mut inferred: Vec<(String, usize)> = Vec::new();
            for i in 0..out.len() {
                if locals.contains(&i) || !in_body(&out[i]) {
                    continue;
                }
                let key = out[i].key.clone();
                let id = match declared.iter().find(|(k, _)| *k == key) {
                    Some((_, id)) => *id,
                    None if rest && out[i].definition.is_none() => match inferred.iter().find(|(k, _)| *k == key) {
                        Some((_, id)) => *id,
                        None => {
                            inferred.push((key, i));
                            i
                        }
                    },
                    None => continue,
                };
                out[i].definition = Some(id);
                params.uses.push(i);
                params.owners.push(params.signatures.len());
            }
            written.extend(inferred.into_iter().map(|(k, _)| k));
            params.signatures.push((index, start, written));
            params.names.push(crate::resolved_name(file.doc.sub(start, end).trim()));
        }
    }
    params
}
