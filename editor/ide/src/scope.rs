//! Which definition a name means, as the compile sees it: the document flattened in compile order from the main
//! file (`#include` inlines a file), `solve-block` scopes, `inherit`, resets and redefinitions.

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
    /// A `solve-block` starts; `reset` is `heading` (None) or `heading.where(level: n)`.
    Start { inherit: bool, reset: Option<Option<usize>> },
    End,
    /// `#reset`.
    Reset,
    /// A heading of this level.
    Heading(usize),
    /// `#include "path"`, as written.
    Include(String),
}

static LEVEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^heading\s*\.\s*where\s*\(\s*level\s*:\s*(\d+)\s*\)$").unwrap());

/// `inherit` and `reset` of a `solve-block` call's arguments.
fn options(args: Option<LinkedNode>) -> (bool, Option<Option<usize>>) {
    let (mut inherit, mut reset) = (false, None);
    for named in args.iter().flat_map(|a| a.children()).filter(|c| c.kind() == SyntaxKind::Named) {
        let mut parts = named.children().filter(|c| !c.kind().is_trivia() && c.kind() != SyntaxKind::Colon);
        let (Some(name), Some(value)) = (parts.next(), parts.next()) else { continue };
        let value = value.get().clone().full_text().trim().to_string();
        match name.get().leaf_text().as_str() {
            "inherit" => inherit = value == "true",
            "reset" if value == "heading" => reset = Some(None),
            "reset" => reset = LEVEL.captures(&value).and_then(|m| m[1].parse().ok()).map(Some),
            _ => {}
        }
    }
    (inherit, reset)
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
                    Some("solve-block") => {
                        let body = args.as_ref().and_then(|a| a.children().find(|c| c.kind() == SyntaxKind::ContentBlock));
                        if let Some(body) = body {
                            let (inherit, reset) = options(args);
                            self.out.push((self.at(body.offset() + 1), Mark::Start { inherit, reset }));
                            self.out.push((self.at(body.offset() + body.len()).saturating_sub(1), Mark::End));
                        }
                    }
                    Some("reset") => self.out.push((self.at(node.offset()), Mark::Reset)),
                    _ => {}
                }
            }
            SyntaxKind::ShowRule => self.show(node),
            SyntaxKind::Ident if node.get().leaf_text() == "reset" && node.parent_kind() == Some(SyntaxKind::Markup) => {
                self.out.push((self.at(node.offset()), Mark::Reset));
            }
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

    /// `#show: solve-block` (or `.with(…)`) runs to the end of the content around it.
    fn show(&mut self, node: &LinkedNode) {
        let parts: Vec<LinkedNode> = node.children().filter(|c| !c.kind().is_trivia()).collect();
        let Some(colon) = parts.iter().position(|c| c.kind() == SyntaxKind::Colon) else { return };
        // `show` and the colon only: no selector.
        if colon != 1 {
            return;
        }
        let Some(transform) = parts.get(colon + 1) else { return };
        let args = match transform.kind() {
            SyntaxKind::Ident if transform.get().leaf_text() == "solve-block" => None,
            SyntaxKind::FuncCall if transform.children().next().and_then(|c| callee(c.get())).as_deref() == Some("solve-block") => {
                transform.children().find(|c| c.kind() == SyntaxKind::Args)
            }
            _ => return,
        };
        let (inherit, reset) = options(args);
        // The markup around it; in a content block that ends before its `]`.
        let end = node.parent().map_or(self.doc.s.len(), |p| p.offset() + p.len());
        self.out.push((self.at(node.offset() + node.len()), Mark::Start { inherit, reset }));
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
}

/// A name in the flattened document: in `file` (an index into the files), and the definition it means (an index
/// into the occurrences; a definition means itself).
#[derive(Clone, Debug)]
pub(crate) struct Occurrence {
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
                Event::Mark(Mark::Start { inherit, reset }) => {
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
                    let id = self.out.len();
                    self.out.push(Occurrence { file: index, start: d.start, end: d.end, definition: Some(id) });
                    self.visible.insert(d.base().to_string(), id);
                }
                Event::Use(i) => {
                    let (key, start, end) = &file.names[i];
                    let definition = self.visible.get(key).copied();
                    self.out.push(Occurrence { file: index, start: *start, end: *end, definition });
                }
            }
        }
        self.visiting.pop();
    }
}

/// Every name of the flattened document with the definition it means: from `main` (an index into `files`) when
/// it reaches `document`, else the document alone.
pub(crate) fn occurrences(files: &[File], document: usize, main: Option<usize>) -> Vec<Occurrence> {
    let run = |start: usize| {
        let mut run = Run {
            files,
            events: vec![None; files.len()],
            visiting: Vec::new(),
            reached: vec![false; files.len()],
            visible: HashMap::new(),
            stack: Vec::new(),
            out: Vec::new(),
        };
        run.file(start);
        (run.out, run.reached[document])
    };
    if let Some(main) = main {
        let (out, reached) = run(main);
        if reached {
            return out;
        }
    }
    run(document).0
}
