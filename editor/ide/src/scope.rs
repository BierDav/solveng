//! Which definition a name means, as the compile sees it: the document flattened in compile order from the main
//! file (`#include` inlines a file), `worksheet` scopes, `inherit`, `clear-at`, `clear` and redefinitions.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;
use typst_syntax::{LinkedNode, SyntaxKind};

use crate::calls::callee;
use crate::complete::Definition;
use crate::text::Doc;

/// Structure of a file that decides what's visible, at character positions.
#[derive(Clone, Debug)]
enum Mark {
    /// A `worksheet` starts; `clear_at` is `heading` (None) or `heading.where(level: n)`.
    Start { inherit: bool, clear_at: Option<Option<usize>> },
    End,
    /// `#clear(all: true)`.
    Reset,
    /// `#clear("x", $f$)`: the keys it forgets.
    Clear(Vec<String>),
    /// A heading of this level.
    Heading(usize),
    /// `#include "path"`, as written.
    Include(String),
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

/// `inherit` and `clear-at` of a `worksheet` call's arguments, for its content (`inherit: auto` means no there).
fn options(args: Option<LinkedNode>) -> (bool, Option<Option<usize>>) {
    let (mut inherit, mut reset) = (false, None);
    for (name, value) in args.iter().flat_map(named_args) {
        match name.as_str() {
            "inherit" => inherit = value == "true",
            "clear-at" if value == "heading" => reset = Some(None),
            "clear-at" => reset = LEVEL.captures(&value).and_then(|m| m[1].parse().ok()).map(Some),
            _ => {}
        }
    }
    (inherit, reset)
}

/// The key of a name `clear` takes: `"x"`, `"f()"`, `$Q_0$`; None for other arguments.
fn clear_key(arg: &LinkedNode) -> Option<String> {
    let text = match arg.kind() {
        SyntaxKind::Str => arg.get().clone().full_text().trim_matches('"').to_string(),
        SyntaxKind::Equation => arg.get().clone().full_text().trim_matches('$').to_string(),
        _ => return None,
    };
    let name = text.split('(').next().unwrap_or("").trim().trim_matches('"');
    (!name.is_empty()).then(|| crate::conceal::resolved_name(name))
}

struct Reader<'a> {
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
                            let (inherit, reset) = options(args);
                            self.out.push((self.at(body.offset() + 1), Mark::Start { inherit, clear_at: reset }));
                            self.out.push((self.at(body.offset() + body.len()).saturating_sub(1), Mark::End));
                        } else if let Some(args) = args.as_ref().filter(|a| named(a, "inherit").as_deref() == Some("false")) {
                            // Equations see the definitions around them, unless `inherit: false`: then they're on their own.
                            self.out.push((self.at(args.offset()), Mark::Start { inherit: false, clear_at: None }));
                            self.out.push((self.at(args.offset() + args.len()), Mark::End));
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
        let (inherit, reset) = options(args);
        // The markup around it; in a content block that ends before its `]`.
        let end = node.parent().map_or(self.doc.s.len(), |p| p.offset() + p.len());
        self.out.push((self.at(node.offset() + node.len()), Mark::Start { inherit, clear_at: reset }));
        self.out.push((self.at(end), Mark::End));
    }
}

fn structure(doc: &Doc) -> Vec<(usize, Mark)> {
    let root = typst_syntax::parse(&doc.s);
    let mut reader = Reader { doc, out: Vec::new() };
    reader.visit(&LinkedNode::new(&root));
    reader.out
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
fn resolve(reference: &str, from: &str) -> String {
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
    reset: Option<Option<usize>>,
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
}

impl Run<'_, '_> {
    fn restart(&mut self) {
        self.visible = self.stack.last().map_or_else(HashMap::new, |f| f.start.clone());
    }

    fn file(&mut self, index: usize) {
        if self.visiting.contains(&index) || self.visiting.len() > 32 {
            return;
        }
        self.visiting.push(index);
        self.reached[index] = true;
        let events = self.events[index].get_or_insert_with(|| events(&self.files[index])).clone();
        let file = &self.files[index];
        for (_, _, event) in events {
            match event {
                Event::Mark(Mark::Start { inherit, clear_at: reset }) => {
                    let start = if inherit { self.visible.clone() } else { HashMap::new() };
                    let outer = std::mem::replace(&mut self.visible, start.clone());
                    self.stack.push(Frame { outer, start, reset });
                }
                Event::Mark(Mark::End) => {
                    if let Some(frame) = self.stack.pop() {
                        self.visible = frame.outer;
                    }
                }
                Event::Mark(Mark::Reset) => self.restart(),
                Event::Mark(Mark::Clear(keys)) => {
                    for key in keys {
                        self.visible.remove(&key);
                    }
                }
                Event::Mark(Mark::Heading(level)) => {
                    if self.stack.iter().any(|f| f.reset.is_some_and(|r| r.is_none_or(|l| l == level))) {
                        self.restart();
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
/// `implicit`: what `worksheet-config(implicit-parameters: …)` says, `"empty"` makes `f()` take what `f(..)` does,
/// `"allow"` every function.
pub(crate) fn occurrences(files: &[File], document: usize, main: Option<usize>, implicit: String) -> (Vec<Occurrence>, Params) {
    let run = |start: usize| {
        let mut run = Run {
            files,
            events: vec![None; files.len()],
            visiting: Vec::new(),
            reached: vec![false; files.len()],
            visible: HashMap::new(),
            stack: Vec::new(),
            out: Vec::new(),
            statements: Vec::new(),
            replaced: Vec::new(),
        };
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
    let mut params = parameters(files, &mut out, &implicit);
    params.statements = statements;
    params.replaced = replaced;
    (out, params)
}

/// Functions' parameters: every occurrence that is one or a use of one, and per function (file, start of its
/// name) its parameters resolved, as written (`I: mA`) and then those `..` takes.
#[derive(Default)]
pub(crate) struct Params {
    pub uses: Vec<usize>,
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

/// A function's parameters are its own: `x` in `f(x) = x^2` means the parameter, not a variable `x`; with `..` the
/// body's other undefined names are parameters too.
fn parameters(files: &[File], out: &mut [Occurrence], implicit: &str) -> Params {
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
                Some((name, unit)) => format!("{}: {}", crate::resolved_name(name.trim()), unit.trim()),
                None => crate::resolved_name(p),
            }).collect();
            let in_body = |o: &Occurrence| o.file == index && o.start > close && o.end <= statement_end;
            let mut declared: Vec<(String, usize)> = Vec::new();
            for i in 0..out.len() {
                let o = &out[i];
                if o.file == index && o.start > open && o.end <= close {
                    declared.push((o.key.clone(), i));
                    out[i].definition = Some(i);
                    params.uses.push(i);
                    params.owners.push(params.signatures.len());
                }
            }
            if let Some((_, marker_end, _)) = crate::clause::marker(&c[close + 1..statement_end], &|_| false) {
                let clause_start = close + 1 + marker_end;
                let tail = file.doc.sub(clause_start, statement_end);
                let locals = crate::clause::solved(&tail).into_iter().map(|n| crate::resolved_name(&n)).collect::<Vec<_>>();
                for i in 0..out.len() {
                    if out[i].file == index && out[i].start >= clause_start && out[i].end <= statement_end
                        && locals.contains(&out[i].key) && !declared.iter().any(|(k, _)| *k == out[i].key) {
                        declared.push((out[i].key.clone(), i));
                    }
                }
            }
            let mut inferred: Vec<(String, usize)> = Vec::new();
            for i in 0..out.len() {
                if !in_body(&out[i]) {
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
