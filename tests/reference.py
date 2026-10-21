"""Writes docs/reference.md from the doc comments in src/ (and tests/util.typ).

Doc comments (`///`) directly before a public `#let` document that function, in tidy's format:
a description, then `- name (types): text` per parameter and `-> type`. A `///` block at the top
of a file, followed by an empty line, is a chapter of its own. Option defaults come from `_DEF`.

    python3 tests/reference.py           # write docs/reference.md
    python3 tests/reference.py --check   # fail if docs/reference.md is out of date
"""
import os, re, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "docs", "reference.md")
# public functions in the order they are documented; worksheet-config comes with the option tables
FUNCTIONS = ["worksheet", "unit", "solve", "simplify", "digits", "sci", "eng", "defined", "graph", "lambdify", "clear", "rect", "tri", "sgn", "floor", "factorial", "diff", "integrate", "laplace", "invlaplace", "at", "asin", "acos", "atan", "worksheet-config"]
CHAPTERS = ["src/builtins.typ", "src/engine.typ", "src/display.typ", "tests/util.typ"]
PARAM = re.compile(r"^- ((?:\.\.)?[a-z][\w-]*(?:, [a-z][\w-]*)*(?:, …)?) \(([^)]*)\): (.*)$")


def read(path):
    return open(os.path.join(ROOT, path), encoding="utf-8").read()


def doc_before(lines, i):
    """The /// lines right above line i, without the slashes."""
    out = []
    while i > 0 and lines[i - 1].startswith("///"):
        i -= 1
        out.insert(0, lines[i][4:] if lines[i].startswith("/// ") else lines[i][3:])
    return out


def signature(text, start):
    """Parameters of `#let name(…)` at start: name -> default (None when positional)."""
    if re.match(r"#let [\w-]+\s*=", text[start:]):
        return {}
    i = text.index("(", start)
    depth, j = 0, i
    while True:
        depth += {"(": 1, ")": -1}.get(text[j], 0)
        if depth == 0:
            break
        j += 1
    params, part, depth = {}, "", 0
    for c in text[i + 1 : j] + ",":
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        if c == "," and depth == 0:
            p = part.strip()
            if p:
                name, _, default = p.partition(":")
                params[name.strip()] = default.strip() or None
            part = ""
        else:
            part += c
    return params


def functions():
    found = {}
    for f in sorted(os.listdir(os.path.join(ROOT, "src"))):
        if not f.endswith(".typ"):
            continue
        text = read("src/" + f)
        lines = text.split("\n")
        for n, line in enumerate(lines):
            m = re.match(r"#let ([a-z][\w-]*)\b", line)
            if m and m.group(1) in FUNCTIONS:
                start = sum(len(l) + 1 for l in lines[:n])
                found[m.group(1)] = (doc_before(lines, n), signature(text, start))
    return found


def split(doc):
    """Description, parameters (names, types, text) and return type of a doc comment."""
    desc, params, ret = [], [], None
    for line in doc:
        m = PARAM.match(line)
        if m:
            params.append(m.groups())
        elif line.startswith("-> "):
            ret = line[3:].strip()
        elif not params:
            desc.append(line)
    return "\n".join(desc).strip(), params, ret


def options():
    """(group, [(option, default)]) from _DEF in src/config.typ, in its order."""
    body = re.search(r"#let _DEF = \((.*?)\n\)", read("src/config.typ"), re.S).group(1)
    groups = []
    for line in body.split("\n"):
        line = line.strip()
        if line.startswith("//"):
            groups.append((line[2:].split(":")[0].strip().capitalize(), []))
        elif line:
            # entries: key: value, split at top-level commas
            part, depth = "", 0
            for c in line + ",":
                depth += c in "([" and 1 or (c in ")]" and -1 or 0)
                if c == "," and depth == 0:
                    if part.strip():
                        k, _, v = part.partition(":")
                        groups[-1][1].append((k.strip(), v.strip()))
                    part = ""
                else:
                    part += c
    return groups


def chapter(path):
    """The /// block at the top of a file, when an empty line follows it."""
    lines = read(path).split("\n")
    i = 0
    while i < len(lines) and lines[i].startswith("//") and not lines[i].startswith("///"):
        i += 1
    j = i
    while j < len(lines) and lines[j].startswith("///"):
        j += 1
    if j == i or j >= len(lines) or lines[j].strip():
        return ""
    return "\n".join(l[4:] if l.startswith("/// ") else l[3:] for l in lines[i:j]).strip()


def option_text(s):
    """An option's doc without its default, which has a column of its own."""
    s = re.sub(r"\s*Default `[^`]*`: ", " e.g. ", s)
    s = re.sub(r"\s*Default `[^`]*`[.;]?", "", s)
    return s.strip()


def cell(s):
    return s.replace("|", "\\|")


def code(s):
    return f"`{s}`" if "`" not in s else f"`` {s} ``"


def anchor(title):
    return re.sub(r"[^\w\- ]", "", title.lower()).strip().replace(" ", "-")


def first_sentence(desc):
    para = desc.split("\n\n")[0].replace("\n", " ")
    m = re.match(r"(.+?\.)(\s|$)", para)
    return m.group(1) if m else para.rstrip(":") + "."


def generate():
    fns = functions()
    missing = [f for f in FUNCTIONS if f not in fns or not fns[f][0]]
    if missing:
        sys.exit("undocumented: " + ", ".join(missing))
    opt_groups = options()
    opt_names = {k for _, kv in opt_groups for k, _ in kv}
    chapters = [c for c in (chapter(p) for p in CHAPTERS) if c]
    chapter_titles = [l[3:] for c in chapters for l in c.split("\n") if l.startswith("## ")]

    out = ["# solveng reference", "",
           "<!-- Generated from the doc comments in src/ by `python3 tests/reference.py` – edit those, not this file. -->", "",
           "Everything solveng does, for looking things up. For an introduction see the [README](../README.md).", ""]
    titles = [f"`{f}`" for f in FUNCTIONS if f != "worksheet-config"] + ["Options"] + chapter_titles
    out += [f"- [{t}](#{anchor(t)})" for t in titles] + [""]

    out += ["## Functions", "", "| function | does |", "|---|---|"]
    for f in FUNCTIONS:
        target = "options" if f == "worksheet-config" else anchor(f"`{f}`")
        out.append(f"| [`{f}`](#{target}) | {cell(first_sentence(split(fns[f][0])[0]))} |")
    out.append("")

    for f in FUNCTIONS:
        doc, sig = fns[f]
        desc, params, ret = split(doc)
        if f == "worksheet-config":
            out += ["## Options", "", f"`worksheet-config` – {desc}", ""]
            texts = {n: (t, d) for n, t, d in params}
            for group, kv in opt_groups:
                out += [f"### {group}", "", "| option | type | default | |", "|---|---|---|---|"]
                for k, v in kv:
                    t, d = texts.get(k, ("", ""))
                    out.append(f"| `{k}` | {cell(t)} | {cell(code(v))} | {cell(option_text(d))} |")
                out.append("")
            continue
        out += [f"## `{f}`", "", desc, ""]
        rows = []
        skipped_options = False
        for names, types, text in params:
            # per-call steps and layout are arguments of their own; the other options are in the tables
            if names in opt_names and names not in ("steps", "layout"):
                skipped_options = True
                continue
            first = names.split(", ")[0]
            default = sig.get(first, sig.get(".." + first.lstrip(".")))
            shown = ", ".join(f"`{n}`" for n in names.split(", ") if n != "…") + (", …" if "…" in names else "")
            if "…" in names:
                text = text + " See [Options](#options)."
            rows.append(f"| {shown} | {cell(types)} | {cell(code(default)) if default else ''} | {cell(text)} |")
        if skipped_options:
            rows.append("| `digits`, `separator`, … | auto | `auto` | Any [option](#options) for this call; `auto` keeps the current setting. |")
        if rows:
            out += ["| parameter | type | default | |", "|---|---|---|---|"] + rows
            if ret:
                out += ["", f"Returns {ret}."]
            out.append("")
    for c in chapters:
        out += [c, ""]
    return "\n".join(out).rstrip() + "\n"


if __name__ == "__main__":
    text = generate()
    if "--check" in sys.argv:
        current = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if current != text:
            sys.exit("docs/reference.md is out of date – run python3 tests/reference.py")
        print("docs/reference.md is up to date")
    else:
        open(OUT, "w", encoding="utf-8").write(text)
        print("wrote docs/reference.md")
