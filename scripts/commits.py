#!/usr/bin/env python3
"""Conventional Commits (https://www.conventionalcommits.org): checks commit messages.

  commits.py file PATH        check a message file (the commit-msg hook)
  commits.py range A..B       check every commit in a git range
  commits.py message TEXT     check one message, e.g. a pull request title
"""
import re, subprocess, sys

TYPES = ("feat", "fix", "perf", "refactor", "docs", "style", "test", "build", "ci", "chore", "revert")
HEADER = re.compile(r"^(?P<type>[a-z]+)(?:\((?P<scope>[\w./-]+(?:, ?[\w./-]+)*)\))?(?P<breaking>!)?: (?P<subject>\S.*)$")
BREAKING_FOOTER = re.compile(r"^BREAKING[ -]CHANGE: ", re.M)
# messages git and GitHub write themselves
GENERATED = re.compile(r'^(Merge |Revert "|fixup! |squash! |amend! )')
MAX_HEADER = 100


def parse(message):
    """The parts of a conventional commit message, or None."""
    header = message.strip().split("\n", 1)[0]
    m = HEADER.match(header)
    if not m or m["type"] not in TYPES:
        return None
    return {
        "type": m["type"],
        "scope": m["scope"],
        "subject": m["subject"],
        "breaking": bool(m["breaking"] or BREAKING_FOOTER.search(message)),
    }


def problem(message):
    """Why a message isn't a conventional commit, or None if it is."""
    message = message.strip()
    if not message:
        return "the message is empty"
    header = message.split("\n", 1)[0]
    if GENERATED.match(header):
        return None
    m = HEADER.match(header)
    if not m:
        return 'the first line must read "type(scope): subject" or "type: subject", e.g. "feat(units): add furlongs"'
    if m["type"] not in TYPES:
        return f'unknown type "{m["type"]}", use one of: {", ".join(TYPES)}'
    if len(header) > MAX_HEADER:
        return f"the first line is {len(header)} characters long, at most {MAX_HEADER} are allowed"
    if "\n" in message and message.split("\n")[1].strip():
        return "the first line must be followed by an empty line"
    return None


def report(name, message):
    error = problem(message)
    if error:
        print(f"✗ {name}: {error}\n    {message.strip().splitlines()[0] if message.strip() else ''}", file=sys.stderr)
    return error is None


def main(args):
    if len(args) != 2 or args[0] not in ("file", "range", "message"):
        print(__doc__, file=sys.stderr)
        return 2
    kind, arg = args
    if kind == "file":
        text = open(arg, encoding="utf-8").read()
        # git leaves comments and, with --verbose, the diff below a scissors line in the file
        text = text.split("# ------------------------ >8 ------------------------", 1)[0]
        text = "\n".join(l for l in text.splitlines() if not l.startswith("#"))
        ok = report("commit message", text)
        if not ok:
            print(f"\nCommit messages follow Conventional Commits. Types: {', '.join(TYPES)}.", file=sys.stderr)
        return 0 if ok else 1
    if kind == "message":
        return 0 if report("message", arg) else 1
    shas = subprocess.run(["git", "rev-list", "--reverse", arg], check=True, capture_output=True, text=True).stdout.split()
    ok = True
    for sha in shas:
        message = subprocess.run(["git", "log", "-1", "--format=%B", sha], check=True, capture_output=True, text=True).stdout
        ok &= report(sha[:9], message)
    print(f"{len(shas)} commit(s) checked", file=sys.stderr)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
