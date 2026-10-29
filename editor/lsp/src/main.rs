//! solveng's language server: completions, highlighting, hovers, navigation, renaming and quick fixes for solveng's
//! calculations in Typst math. It runs next to tinymist and answers only where solveng reads the document.
//!
//! It doesn't compile. A client that does can send what the compile reported (`query(<solveng>)`, gathered into a
//! `solveng_ide::Report`) with the notification `solveng/report`, `{ "report": … }` as an object or JSON string:
//! then the errors solveng showed in the document are diagnostics too, and the compiler's values complete.

use std::collections::HashMap;
use std::error::Error;
use std::path::{Component, Path, PathBuf};

use lsp_server::{Connection, Message, Notification, Request, Response};
use serde_json::{Value, json};
use solveng_ide::{self as ide, Context, ContextFile, ItemKind, Location, Report};

/// Semantic token types, indexed by `solveng_ide::TokenKind`; units read as enum members in most themes.
// Function parameters, and what a graph plots against.
const TOKEN_TYPES: [&str; 11] =
    ["keyword", "string", "comment", "number", "function", "type", "variable", "decorator", "operator", "enumMember", "parameter"];
/// The `=` or `->` showing a result is an operator with `evaluate`, for themes to set bold; constants are variables
/// with LSP's standard `readonly`, as themes style constants.
const TOKEN_MODIFIERS: [&str; 2] = ["evaluate", "readonly"];

struct Server {
    /// Open documents by URI.
    open: HashMap<String, String>,
    root: Option<PathBuf>,
    /// The file typst compiles, whose setup (`worksheet`) counts for the files it includes.
    main: Option<PathBuf>,
    /// What the last compile reported, as the client sent it with `solveng/report`.
    report: Report,
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("solveng-lsp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let (connection, threads) = Connection::stdio();
    let (id, params) = connection.initialize_start()?;
    let root = params["rootUri"].as_str().and_then(path_of).or_else(|| params["workspaceFolders"][0]["uri"].as_str().and_then(path_of));
    let mut server = Server { open: HashMap::new(), root, main: None, report: Report::default() };
    server.configure(&params["initializationOptions"]);
    connection.initialize_finish(id, json!({ "capabilities": capabilities(), "serverInfo": { "name": "solveng-lsp", "version": env!("CARGO_PKG_VERSION") } }))?;
    for message in &connection.receiver {
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    break;
                }
                let id = request.id.clone();
                let result = server.request(request);
                connection.sender.send(Message::Response(Response { id, result: Some(result), error: None }))?;
            }
            Message::Notification(n) => {
                for out in server.notification(n) {
                    connection.sender.send(Message::Notification(out))?;
                }
            }
            Message::Response(_) => {}
        }
    }
    // The writer thread ends once the connection is gone.
    drop(connection);
    threads.join()?;
    Ok(())
}

fn capabilities() -> Value {
    json!({
        "positionEncoding": "utf-16",
        "textDocumentSync": 1,
        "completionProvider": { "triggerCharacters": ["\"", "(", ",", "#", "/", "_"] },
        "hoverProvider": true,
        "documentFormattingProvider": true,
        "definitionProvider": true,
        "referencesProvider": true,
        "renameProvider": { "prepareProvider": true },
        "codeActionProvider": { "codeActionKinds": ["quickfix"] },
        "semanticTokensProvider": { "legend": { "tokenTypes": TOKEN_TYPES, "tokenModifiers": TOKEN_MODIFIERS }, "full": true },
    })
}

/// The path of a `file://` URI.
fn path_of(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let rest = if rest.starts_with('/') { rest } else {
        let (authority, _) = rest.split_once('/')?;
        if !authority.eq_ignore_ascii_case("localhost") { return None; }
        &rest[authority.len()..]
    };
    let bytes = rest.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len()
            && let Ok(b) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?, 16) {
                out.push(b);
                i += 3;
                continue;
            }
        out.push(bytes[i]);
        i += 1;
    }
    Some(normalized(PathBuf::from(String::from_utf8(out).ok()?)))
}

/// The `file://` URI of a path, with anything but unreserved characters and `/` escaped.
fn uri_of(path: &Path) -> String {
    let mut out = String::from("file://");
    for b in path.to_string_lossy().bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// One key per local file, independent of URI escaping or localhost spelling.
fn canonical_uri(uri: &str) -> String {
    path_of(uri).map(|path| uri_of(&normalized(path))).unwrap_or_else(||uri.to_owned())
}

/// Normalize imports lexically so `exercise/../template.typ` matches the module path and open buffer.
/// This also works for unsaved files, which cannot be canonicalized through the filesystem.
fn normalized(path: PathBuf) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {},
            Component::ParentDir if out.file_name().is_some_and(|name| name != "..") => { out.pop(); },
            Component::ParentDir if out.has_root() => {},
            _ => out.push(component.as_os_str()),
        }
    }
    out
}

/// UTF-16 offset of an LSP position.
fn offset(text: &str, position: &Value) -> u32 {
    let line = position["line"].as_u64().unwrap_or(0) as usize;
    let character = position["character"].as_u64().unwrap_or(0) as u32;
    let mut at = 0u32;
    for (n, l) in text.split('\n').enumerate() {
        let len = l.encode_utf16().count() as u32;
        if n == line {
            return at + character.min(len);
        }
        at += len + 1;
    }
    at.saturating_sub(1)
}

/// Line starts in UTF-16, for turning offsets back into positions.
fn line_starts(text: &str) -> Vec<u32> {
    let mut starts = vec![0];
    let mut at = 0;
    for c in text.chars() {
        at += c.len_utf16() as u32;
        if c == '\n' {
            starts.push(at);
        }
    }
    starts
}

fn position(starts: &[u32], offset: u32) -> Value {
    let line = starts.partition_point(|&s| s <= offset).saturating_sub(1);
    json!({ "line": line, "character": offset - starts[line] })
}

fn range(starts: &[u32], start: u32, end: u32) -> Value {
    json!({ "start": position(starts, start), "end": position(starts, end) })
}

impl Server {
    fn configure(&mut self, options: &Value) {
        if let Some(main) = options["main"].as_str() {
            let path = PathBuf::from(main);
            self.main = Some(if path.is_absolute() { path } else { self.root.clone().unwrap_or_default().join(path) });
        }
    }

    fn text_of(&self, path: &Path) -> Option<String> {
        self.open.get(&uri_of(path)).cloned()
            .or_else(|| self.open.iter().find(|(uri,_)| path_of(uri).is_some_and(|p| normalized(p) == normalized(path.to_path_buf()))).map(|(_,text)|text.clone()))
            .or_else(|| std::fs::read_to_string(path).ok())
    }

    /// The folder imports starting with `/` and the context's paths are relative to.
    fn root_for(&self, file: &Path) -> PathBuf {
        self.root.clone().or_else(|| file.parent().map(Path::to_path_buf)).unwrap_or_default()
    }

    /// A location as LSP has it: in the document, or in a file around it by its path relative to the root.
    fn location(&self, uri: &str, text: &str, location: &Location) -> Option<Value> {
        let (uri, text) = match &location.path {
            None => (uri.to_string(), text.to_string()),
            Some(path) => {
                let root = self.root_for(&path_of(uri)?);
                let file = if Path::new(path).is_absolute() { PathBuf::from(path) } else { root.join(path) };
                let text = self.text_of(&file)?;
                (uri_of(&file), text)
            }
        };
        Some(json!({ "uri": uri, "range": range(&line_starts(&text), location.start, location.end) }))
    }

    /// The files around a document: the main file and what it and the document import or include, so a
    /// `worksheet` and definitions set up elsewhere count here too.
    fn context(&self, uri: &str, text: &str) -> Context {
        let Some(file) = path_of(uri) else { return Context::default() };
        let root = self.root_for(&file);
        let resolve = |reference: &str, from: &Path| normalized(match reference.strip_prefix('/') {
            Some(rest) => root.join(rest),
            None => from.parent().unwrap_or(Path::new("")).join(reference),
        });
        let mut queue: Vec<PathBuf> = ide::references(text).iter().map(|r| resolve(r, &file)).collect();
        if let Some(main) = &self.main
            && main != &file {
                queue.insert(0, main.clone());
            }
        let mut seen = vec![file.clone()];
        let mut parts = Vec::new();
        let mut size = 0;
        // Bounded, since a project can be large and imports can loop.
        while !queue.is_empty() && parts.len() < 24 && size < 1_000_000 {
            let next = queue.remove(0);
            if next.extension().is_none_or(|e| e != "typ") || seen.contains(&next) {
                continue;
            }
            seen.push(next.clone());
            let Some(t) = self.text_of(&next) else { continue };
            queue.extend(ide::references(&t).iter().map(|r| resolve(r, &next)));
            size += t.len();
            let path = next.strip_prefix(&root).unwrap_or(&next).to_string_lossy().into_owned();
            parts.push(ContextFile { path, text: t });
        }
        let relative = |p: &Path| p.strip_prefix(&root).unwrap_or(p).to_string_lossy().into_owned();
        // Names resolve in compile order from the main file; without one the document stands alone.
        let context = Context::new("").with_files(parts).with_report(&self.report, &relative(&file), self.main.as_ref() == Some(&file));
        match &self.main {
            Some(main) => context.with_main(&relative(main)),
            None => context,
        }
    }

    fn document(&self, params: &Value) -> Option<(String, String)> {
        let uri = params["textDocument"]["uri"].as_str()?.to_string();
        let text = self.open.get(&canonical_uri(&uri)).or_else(||self.open.get(&uri))?.clone();
        Some((uri, text))
    }

    fn request(&mut self, request: Request) -> Value {
        let p = &request.params;
        let Some((uri, text)) = self.document(p) else { return Value::Null };
        let context = self.context(&uri, &text);
        let starts = line_starts(&text);
        match request.method.as_str() {
            "textDocument/completion" => {
                let at = offset(&text, &p["position"]);
                let Some(c) = ide::complete(&text, &context, at) else { return Value::Null };
                let items: Vec<Value> = c
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let snippet = item.insert.contains("$0");
                        let new_text = item.insert.clone();
                        let mut v = json!({
                            "label": item.label,
                            "kind": match item.kind {
                                ItemKind::FUNCTION => 3,
                                ItemKind::UNIT => 11,
                                ItemKind::CONSTANT => 21,
                                ItemKind::PARAMETER => 10,
                                ItemKind::VALUE => 20,
                                ItemKind::KEYWORD => 14,
                                _ => 6,
                            },
                            "detail": item.detail,
                            "sortText": format!("{i:05}"),
                            "filterText": item.filter.clone().unwrap_or_else(|| item.insert.replace("$0", "")),
                            "insertTextFormat": if snippet { 2 } else { 1 },
                            "textEdit": { "range": range(&starts, c.from, c.to.unwrap_or(at)), "newText": new_text },
                        });
                        let edits: Vec<_> = item.edits.iter().filter(|e| e.path.is_none())
                            .map(|e| json!({ "range": range(&starts, e.start, e.end), "newText": e.text }))
                            .collect();
                        if !edits.is_empty() { v["additionalTextEdits"] = json!(edits); }
                        if let Some(doc) = &item.documentation {
                            v["documentation"] = json!({ "kind": "markdown", "value": doc });
                        }
                        v
                    })
                    .collect();
                json!({ "isIncomplete": true, "items": items })
            }
            "textDocument/hover" => match ide::hover(&text, &context, offset(&text, &p["position"])) {
                Some(h) => json!({ "contents": { "kind": "markdown", "value": h.markdown }, "range": range(&starts, h.start, h.end) }),
                None => Value::Null,
            },
            "textDocument/definition" => match ide::definition(&text, &context, offset(&text, &p["position"])) {
                Some(l) => self.location(&uri, &text, &l).unwrap_or(Value::Null),
                None => Value::Null,
            },
            "textDocument/references" => {
                let at = offset(&text, &p["position"]);
                let mut found = ide::usages(&text, &context, at);
                if p["context"]["includeDeclaration"] == json!(false)
                    && let Some(d) = ide::definition(&text, &context, at) {
                        found.retain(|l| *l != d);
                    }
                json!(found.iter().filter_map(|l| self.location(&uri, &text, l)).collect::<Vec<_>>())
            }
            "textDocument/formatting" => json!(ide::format(&text, &context).iter().map(|e| {
                json!({ "range": range(&starts, e.start, e.end), "newText": e.text })
            }).collect::<Vec<_>>()),
            "textDocument/prepareRename" => match ide::rename_target(&text, &context, offset(&text, &p["position"])) {
                Some(t) => json!({ "range": range(&starts, t.start, t.end), "placeholder": t.name }),
                None => Value::Null,
            },
            "textDocument/rename" => {
                let name = p["newName"].as_str().unwrap_or_default();
                let mut changes = serde_json::Map::new();
                for e in ide::rename(&text, &context, offset(&text, &p["position"]), name) {
                    let at = Location { path: e.path.clone(), start: e.start, end: e.end };
                    let Some(l) = self.location(&uri, &text, &at) else { continue };
                    let file = l["uri"].as_str().unwrap_or_default().to_string();
                    let list = changes.entry(file).or_insert_with(|| json!([]));
                    list.as_array_mut().unwrap().push(json!({ "range": l["range"], "newText": e.text }));
                }
                if changes.is_empty() { Value::Null } else { json!({ "changes": changes }) }
            }
            "textDocument/codeAction" => {
                let at = offset(&text, &p["range"]["start"]);
                let actions: Vec<Value> = ide::unit_fixes(&text, &context, at).into_iter()
                    .map(|c| {
                        // The fix and the edits that go with it, like an import; all in the document.
                        let edits: Vec<Value> = std::iter::once((c.start, c.end, c.text.clone()))
                            .chain(c.edits.iter().filter(|e| e.path.is_none()).map(|e| (e.start, e.end, e.text.clone())))
                            .map(|(start, end, text)| json!({ "range": range(&starts, start, end), "newText": text }))
                            .collect();
                        json!({ "title": c.title, "kind": "quickfix", "edit": { "changes": { uri.clone(): edits } } })
                    })
                    .collect();
                json!(actions)
            }
            "textDocument/semanticTokens/full" => {
                let mut data = Vec::new();
                let utf16: Vec<u16> = text.encode_utf16().collect();
                let (mut line, mut col) = (0u32, 0u32);
                for t in ide::document_tokens(&text, &context) {
                    let (kind, modifiers) = if t.kind == ide::TokenKind::EVALUATE { (ide::TokenKind::OPERATOR, 1) } else if matches!(t.kind, ide::TokenKind::FUNCTION_CALL | ide::TokenKind::BUILTIN_FUNCTION) { (ide::TokenKind::FUNCTION, 0) } else if t.kind == ide::TokenKind::UNIT_REFERENCE { (ide::TokenKind::COMMENT, 0) } else if t.kind == ide::TokenKind::CONSTANT { (ide::TokenKind::VARIABLE, 2) } else { (t.kind, 0) };
                    if usize::from(kind) >= TOKEN_TYPES.len() { continue; }
                    // LSP tokens cannot cross newlines. Emit every line of a
                    // multiline token instead of truncating its remainder.
                    let end = (t.start + t.length).min(utf16.len() as u32);
                    let mut at = t.start;
                    while at < end {
                        let l = starts.partition_point(|&s| s <= at).saturating_sub(1);
                        let next = starts.get(l+1).copied();
                        let mut line_end = next.map_or(utf16.len() as u32, |v|v-1);
                        if line_end > starts[l] && utf16[(line_end-1) as usize] == 13 { line_end -= 1; }
                        let length = end.min(line_end).saturating_sub(at);
                        if length > 0 {
                            let c = at-starts[l];
                            let l = l as u32;
                            let delta_col = if l == line { c-col } else { c };
                            data.extend([l-line,delta_col,length,u32::from(kind),modifiers]);
                            line=l; col=c;
                        }
                        at = next.filter(|n|*n <= end).unwrap_or(end);
                    }
                }
                json!({ "data": data })
            }
            _ => Value::Null,
        }
    }

    fn notification(&mut self, n: Notification) -> Vec<Notification> {
        let p = &n.params;
        match n.method.as_str() {
            "textDocument/didOpen" => {
                let uri = p["textDocument"]["uri"].as_str().unwrap_or_default().to_string();
                self.open.insert(canonical_uri(&uri), p["textDocument"]["text"].as_str().unwrap_or_default().to_string());
                self.publish(&uri)
            }
            "textDocument/didChange" => {
                let uri = p["textDocument"]["uri"].as_str().unwrap_or_default().to_string();
                // Full sync: the last change is the whole text.
                if let Some(text) = p["contentChanges"].as_array().and_then(|c| c.last()).and_then(|c| c["text"].as_str()) {
                    self.open.insert(canonical_uri(&uri), text.to_string());
                }
                self.publish(&uri)
            }
            "textDocument/didClose" => {
                let uri = p["textDocument"]["uri"].as_str().unwrap_or_default().to_string();
                self.open.remove(&canonical_uri(&uri));
                vec![Notification::new("textDocument/publishDiagnostics".into(), json!({ "uri": uri, "diagnostics": [] }))]
            }
            "workspace/didChangeConfiguration" => {
                self.configure(&p["settings"]["solveng"]);
                Vec::new()
            }
            "solveng/report" => {
                self.report = match &p["report"] {
                    Value::String(json) => Report::parse(json),
                    other => serde_json::from_value(other.clone()).unwrap_or_default(),
                };
                let open: Vec<String> = self.open.keys().cloned().collect();
                open.iter().flat_map(|uri| self.publish(uri)).collect()
            }
            _ => Vec::new(),
        }
    }

    fn publish(&self, uri: &str) -> Vec<Notification> {
        let Some(text) = self.open.get(&canonical_uri(uri)).or_else(|| self.open.get(uri)) else { return Vec::new() };
        let starts = line_starts(text);
        let diagnostics: Vec<Value> = ide::diagnostics(text, &self.context(uri, text))
            .into_iter()
            .map(|d| json!({ "range": range(&starts, d.start, d.end), "severity": if d.severity == "error" { 1 } else { 2 }, "source": "solveng", "message": d.message }))
            .collect();
        vec![Notification::new("textDocument/publishDiagnostics".into(), json!({ "uri": uri, "diagnostics": diagnostics }))]
    }
}

#[cfg(test)]
#[path = "../tests/audit_20261029/unit.rs"]
mod audit_unit_20261029;
