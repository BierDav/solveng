// Starts the server over stdio and asks it for completions, diagnostics and tokens.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

fn send(stdin: &mut ChildStdin, message: Value) {
    let body = message.to_string();
    write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
    stdin.flush().unwrap();
}

fn receive(stdout: &mut BufReader<ChildStdout>) -> Value {
    let mut length = 0;
    loop {
        let mut line = String::new();
        stdout.read_line(&mut line).unwrap();
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some(n) = line.strip_prefix("Content-Length: ") {
            length = n.parse().unwrap();
        }
    }
    let mut body = vec![0; length];
    stdout.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// The next message answering `id`, or the next notification of `method`.
fn wait(stdout: &mut BufReader<ChildStdout>, matches: impl Fn(&Value) -> bool) -> Value {
    loop {
        let m = receive(stdout);
        if matches(&m) {
            return m;
        }
    }
}

#[test]
fn answers_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_solveng-lsp")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}, "rootUri": null}}));
    let init = wait(&mut stdout, |m| m["id"] == 1);
    assert!(init["result"]["capabilities"]["semanticTokensProvider"]["full"].as_bool().unwrap());
    assert_eq!(init["result"]["capabilities"]["documentFormattingProvider"], true);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    let text = "#import \"@preview/solveng:1.0.0\": *\n#worksheet(`I := 30 m`)\n#lambdify($x$, \"x\", unit: \"furlong\")\n";
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": "file:///tmp/solveng-smoke.typ", "languageId": "typst", "version": 1, "text": text}}}));
    let diagnostics = wait(&mut stdout, |m| m["method"] == "textDocument/publishDiagnostics");
    assert_eq!(diagnostics["params"]["diagnostics"][0]["range"]["start"], json!({"line": 2, "character": 27}));

    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/completion", "params": {"textDocument": {"uri": "file:///tmp/solveng-smoke.typ"}, "position": {"line": 1, "character": 21}}}));
    let completion = wait(&mut stdout, |m| m["id"] == 2);
    assert_eq!(completion["result"]["isIncomplete"], true);
    let items = completion["result"]["items"].as_array().unwrap();
    let ma = items.iter().find(|i| i["label"] == "mA").unwrap();
    assert_eq!(ma["textEdit"]["range"]["start"], json!({"line": 1, "character": 20}));

    // Outside solveng tinymist answers.
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/completion", "params": {"textDocument": {"uri": "file:///tmp/solveng-smoke.typ"}, "position": {"line": 0, "character": 3}}}));
    assert!(wait(&mut stdout, |m| m["id"] == 3)["result"].is_null());

    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 4, "method": "textDocument/semanticTokens/full", "params": {"textDocument": {"uri": "file:///tmp/solveng-smoke.typ"}}}));
    let tokens = wait(&mut stdout, |m| m["id"] == 4);
    // `I` on line 1, column 12.
    assert_eq!(tokens["result"]["data"].as_array().unwrap()[..3], [json!(1), json!(12), json!(1)]);

    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": "file:///tmp/solveng-format.typ", "languageId": "typst", "version": 1, "text": "#show: worksheet\n$ x:=f( a :V,b )$"}}}));
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 50, "method": "textDocument/formatting", "params": {"textDocument": {"uri": "file:///tmp/solveng-format.typ"}, "options": {"tabSize": 2, "insertSpaces": true}}}));
    let formatted = wait(&mut stdout, |m| m["id"] == 50);
    assert_eq!(formatted["result"][0]["newText"], "x := f(a: V, b)");
    assert_eq!(formatted["result"][0]["range"]["start"], json!({"line": 1, "character": 1}));

    let changed = "#worksheet(`result := inte`)";
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {"textDocument": {"uri": "file:///tmp/solveng-smoke.typ", "version": 2}, "contentChanges": [{"text": changed}]}}));
    let at = changed.find("inte").unwrap() + 4;
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 6, "method": "textDocument/completion", "params": {"textDocument": {"uri": "file:///tmp/solveng-smoke.typ"}, "position": {"line": 0, "character": at}}}));
    let completion = wait(&mut stdout, |m| m["id"] == 6);
    let integrate = completion["result"]["items"].as_array().unwrap().iter().find(|item| item["label"] == "integrate").unwrap();
    assert_eq!(integrate["insertTextFormat"], 2);
    assert!(integrate["textEdit"]["newText"].as_str().unwrap().contains("${1:expression}"));

    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": null}));
    wait(&mut stdout, |m| m["id"] == 5);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "exit", "params": null}));
    // The server's reader stops when stdin closes.
    drop(stdin);
    assert!(child.wait().unwrap().success());
}

#[test]
fn finds_definitions_and_references_across_files() {
    let dir = std::env::temp_dir().join(format!("solveng lsp {}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.typ"), "#show: worksheet\n#worksheet(`U := 5 V`)\n#include \"chapter.typ\"\n").unwrap();
    let root = format!("file://{}", dir.display()).replace(' ', "%20");
    let uri = format!("{root}/chapter.typ");
    let mut child = Command::new(env!("CARGO_BIN_EXE_solveng-lsp")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}, "rootUri": root, "initializationOptions": {"main": "main.typ"}}}));
    let init = wait(&mut stdout, |m| m["id"] == 1);
    assert!(init["result"]["capabilities"]["definitionProvider"].as_bool().unwrap());
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    let text = "Chapter\n#worksheet(`P := U^2 / R`)\n#worksheet(`I := U / 2`)\n";
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": uri, "languageId": "typst", "version": 1, "text": text}}}));

    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/definition", "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 17}}}));
    let def = wait(&mut stdout, |m| m["id"] == 2);
    assert_eq!(def["result"]["uri"], json!(format!("{root}/main.typ")));
    assert_eq!(def["result"]["range"], json!({"start": {"line": 1, "character": 12}, "end": {"line": 1, "character": 13}}));

    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/references", "params": {"textDocument": {"uri": uri}, "position": {"line": 2, "character": 17}, "context": {"includeDeclaration": true}}}));
    let refs = wait(&mut stdout, |m| m["id"] == 3);
    assert_eq!(refs["result"].as_array().unwrap().len(), 3);
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 4, "method": "textDocument/references", "params": {"textDocument": {"uri": uri}, "position": {"line": 2, "character": 17}, "context": {"includeDeclaration": false}}}));
    assert_eq!(wait(&mut stdout, |m| m["id"] == 4)["result"].as_array().unwrap().len(), 2);

    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": null}));
    wait(&mut stdout, |m| m["id"] == 5);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "exit", "params": null}));
    drop(stdin);
    assert!(child.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn shows_the_errors_a_client_reports() {
    let dir = std::env::temp_dir().join(format!("solveng-lsp-errors-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let root = format!("file://{}", dir.display());
    let uri = format!("{root}/main.typ");
    let mut child = Command::new(env!("CARGO_BIN_EXE_solveng-lsp")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}, "rootUri": root, "initializationOptions": {"main": "main.typ"}}}));
    wait(&mut stdout, |m| m["id"] == 1);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    let text = "#show: worksheet\n$U := 5 V$\n$t := U + 2 s =$\n";
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": uri, "languageId": "typst", "version": 1, "text": text}}}));
    let before = wait(&mut stdout, |m| m["method"] == "textDocument/publishDiagnostics");
    assert_eq!(before["params"]["diagnostics"], json!([]));
    // what the compile reported, as typst saw the file
    let start = text.find("$t").unwrap();
    let element = "$t := U + 2 s =$";
    let report = json!({"blocks": 1, "errors": [{"path": "main.typ", "start": start, "end": start + element.len(), "text": element, "message": "adding different units: s and V"}]});
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "solveng/report", "params": {"report": report.to_string()}}));
    let after = wait(&mut stdout, |m| m["method"] == "textDocument/publishDiagnostics");
    let d = &after["params"]["diagnostics"][0];
    assert_eq!((d["severity"].clone(), d["message"].clone()), (json!(1), json!("adding different units: s and V")));
    assert_eq!(d["range"], json!({"start": {"line": 2, "character": 1}, "end": {"line": 2, "character": 15}}));
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": null}));
    wait(&mut stdout, |m| m["id"] == 5);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "exit", "params": null}));
    drop(stdin);
    assert!(child.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn leaves_math_outside_a_worksheet_as_written() {
    let dir = std::env::temp_dir().join(format!("solveng-lsp-plain-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // A main file with a worksheet around other content says nothing about a file it doesn't include.
    std::fs::write(dir.join("main.typ"), "#show: worksheet\n$U := 5 V$\n").unwrap();
    let root = format!("file://{}", dir.display());
    let uri = format!("{root}/doc.typ");
    let mut child = Command::new(env!("CARGO_BIN_EXE_solveng-lsp")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}, "rootUri": root, "initializationOptions": {"main": "main.typ"}}}));
    wait(&mut stdout, |m| m["id"] == 1);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    let text = "#import \"@solveng/solveng:1.0.0\": *\n\n$R := 5\"cm\"$\n";
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": uri, "languageId": "typst", "version": 1, "text": text}}}));
    wait(&mut stdout, |m| m["method"] == "textDocument/publishDiagnostics");
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/semanticTokens/full", "params": {"textDocument": {"uri": uri}}}));
    assert_eq!(wait(&mut stdout, |m| m["id"] == 2)["result"]["data"], json!([]));
    // Wrapped in a worksheet it is solveng's, unless that worksheet says `evaluate: false`.
    for (changed, tokens) in [
        ("#show: worksheet\n$R := 5\"cm\"$\n", true),
        ("#worksheet(evaluate: false)[$R := 5\"cm\"$]\n", false),
    ] {
        send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {"textDocument": {"uri": uri, "version": 2}, "contentChanges": [{"text": changed}]}}));
        wait(&mut stdout, |m| m["method"] == "textDocument/publishDiagnostics");
        send(&mut stdin, json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/semanticTokens/full", "params": {"textDocument": {"uri": uri}}}));
        let data = wait(&mut stdout, |m| m["id"] == 3)["result"]["data"].as_array().unwrap().clone();
        assert_eq!(!data.is_empty(), tokens, "{changed}");
    }
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown", "params": null}));
    wait(&mut stdout, |m| m["id"] == 4);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "exit", "params": null}));
    drop(stdin);
    assert!(child.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unit_shortcut_tokens_cover_only_the_unit_string_and_evaluated_use() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_solveng-lsp")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}, "rootUri": null}}));
    wait(&mut stdout, |m| m["id"] == 1);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    let uri = "file:///tmp/solveng-unit-shortcut.typ";
    let text = "#show: worksheet.with(units: (iwef: $unit(\"km\")$))\n#let cusom = unit(\"iwef\")\n$t := 1cusom$\n$y := op(\"tri\")(x)$\n$y := 1unit(\"km\")$\n$y := 1 unit(\"ms\")$\n";
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": uri, "languageId": "typst", "version": 1, "text": text}}}));
    wait(&mut stdout, |m| m["method"] == "textDocument/publishDiagnostics");
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 2, "method": "textDocument/semanticTokens/full", "params": {"textDocument": {"uri": uri}}}));
    let result = wait(&mut stdout, |m| m["id"] == 2);
    let data = result["result"]["data"].as_array().unwrap();
    let (mut line, mut col) = (0, 0);
    let mut declaration = Vec::new();
    let mut uses = Vec::new();
    let mut inline = Vec::new();
    for token in data.chunks_exact(5) {
        let delta = token[0].as_u64().unwrap();
        line += delta;
        col = if delta == 0 { col + token[1].as_u64().unwrap() } else { token[1].as_u64().unwrap() };
        let record = (col, token[2].as_u64().unwrap(), token[3].as_u64().unwrap());
        if line == 1 { declaration.push(record); }
        if line == 2 && record.2 == 9 { uses.push(record); }
        if line >= 3 { inline.push((line, record)); }
    }
    assert_eq!(declaration, vec![(19, 4, 9)]);
    assert_eq!(uses, vec![(7, 5, 9)]);
    for (line, start, length, kind) in [(3, 10, 3, 4), (4, 13, 2, 9), (5, 14, 2, 9)] {
        let hits: Vec<_> = inline.iter().filter(|(l, (s, n, _))| *l == line && *s < start + length + 1 && start - 1 < *s + *n).copied().collect();
        assert_eq!(hits, vec![(line, (start, length, kind))]);
    }

    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/hover", "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 20}}}));
    let hover = wait(&mut stdout, |m| m["id"] == 3);
    assert!(hover["result"]["contents"]["value"].as_str().unwrap().contains("`cusom` unit shortcut for `iwef`"));
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 4, "method": "textDocument/completion", "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 21}}}));
    let completion = wait(&mut stdout, |m| m["id"] == 4);
    assert!(completion["result"]["items"].as_array().unwrap().iter().any(|item| item["label"] == "iwef"));
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 6, "method": "textDocument/completion", "params": {"textDocument": {"uri": uri}, "position": {"line": 3, "character": 12}}}));
    let completion = wait(&mut stdout, |m| m["id"] == 6);
    assert!(completion["result"]["items"].as_array().unwrap().iter().any(|item| item["label"] == "tri" && item["textEdit"]["newText"] == "tri"));
    send(&mut stdin, json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": null}));
    wait(&mut stdout, |m| m["id"] == 5);
    send(&mut stdin, json!({"jsonrpc": "2.0", "method": "exit", "params": null}));
    drop(stdin);
    assert!(child.wait().unwrap().success());
}
