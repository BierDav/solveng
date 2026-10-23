"""Compiles every typ example of README.md and docs/reference.md on its own.

Problems must stop the compilation (errors: "panic"), except in examples that show an error
(a "// error" comment); those must still compile. Set TYPST to use another compiler.
"""
import os, re, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TYPST = os.environ.get("TYPST", "typst")
IMPORT = '#import "@preview/solveng:1.0.0": *'

def fatal(doc):
    """Problems stop the compilation: add errors: "panic" to the example's worksheet, or wrap it."""
    if "#show: worksheet.with(" in doc:
        return doc.replace("#show: worksheet.with(", '#show: worksheet.with(errors: "panic", ', 1)
    if "#show: worksheet\n" in doc:
        return doc.replace("#show: worksheet\n", '#show: worksheet.with(errors: "panic")\n', 1)
    # no worksheet of its own: one that only calculates the calls and code blocks
    show = '#show: worksheet.with(errors: "panic", evaluate: false)\n'
    marker = '#import "/lib.typ": *'
    return doc.replace(marker, marker + "\n" + show, 1) if marker in doc else show + doc

def examples(path):
    text = open(os.path.join(ROOT, path)).read()
    for m in re.finditer(r"^(`{3,})typ\n(.*?)^\1$", text, re.M | re.S):
        yield path, text[: m.start()].count("\n") + 1, m.group(2)

failed = 0
with tempfile.TemporaryDirectory() as tmp:
    for path, line, code in [e for p in ("README.md", "docs/reference.md") for e in examples(p)]:
        # examples that import a file of their own can't stand alone
        if re.search(r'#import "(?!@preview/solveng)', code): continue
        # examples from doc comments leave the import out
        if IMPORT not in code: code = IMPORT + "\n" + code
        shows_error = "// error" in code
        doc = code.replace(IMPORT, '#import "/lib.typ": *', 1)
        if not shows_error:
            doc = fatal(doc)
        src = os.path.join(ROOT, "tests", ".example.typ")
        open(src, "w").write("#set page(height: auto)\n" + doc)
        r = subprocess.run([TYPST, "compile", "--root", ROOT, src, os.path.join(tmp, "out.pdf")], capture_output=True, text=True)
        os.remove(src)
        if r.returncode != 0:
            failed += 1
            print(f"{path}:{line}: fails\n{r.stderr.strip()}\n")
print(f"{TYPST}: {'all examples compile' if not failed else str(failed) + ' failed'}")
sys.exit(1 if failed else 0)
