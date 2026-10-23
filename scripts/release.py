#!/usr/bin/env python3
"""Release helpers, used by .github/workflows/release.yml.

  release.py next CHANNEL [X.Y.Z]   the next version for alpha, rc or stable
  release.py bump VERSION           writes VERSION into typst.toml, Cargo.toml and the docs
  release.py notes TAG              release notes: the commits since the release before TAG
  release.py bundle DIR [REF]       the package as Typst Universe gets it (REF's files without typst.toml's exclude)

The next version comes from the commits since the last stable tag: a breaking change bumps the major
version (the minor one before 1.0.0), a feat the minor one, anything else the patch. A version already
in typst.toml but not released yet is the lowest it can be, and an explicit X.Y.Z replaces it.
alpha and rc append -alpha.N / -rc.N, counting up per version. typst.toml only takes X.Y.Z, so
prereleases write the version they lead up to there and the full one into Cargo.toml.
"""
import fnmatch, os, re, shutil, subprocess, sys, tarfile, io, tomllib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from commits import parse

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TAG = re.compile(r"^v(\d+)\.(\d+)\.(\d+)(?:-(alpha|rc)\.(\d+))?$")
CHANNELS = ("alpha", "rc", "stable")
# files whose imports and links name the version; editor tests keep theirs
DOC_FILES = re.compile(r"^(README\.md|lib\.typ|docs/.*\.(md|typ)|src/.*\.typ|tests/examples\.py)$")


def git(*args):
    return subprocess.run(["git", *args], cwd=ROOT, check=True, capture_output=True, text=True).stdout


def tags(merged_into="HEAD"):
    """Release tags reachable from a ref, as (version tuple, prerelease, tag), newest first."""
    out = []
    for tag in git("tag", "--list", "v*", "--merged", merged_into).split():
        m = TAG.match(tag)
        if m:
            core = tuple(int(m[i]) for i in (1, 2, 3))
            pre = (m[4], int(m[5])) if m[4] else None
            # stable sorts after its prereleases, rc after alpha
            out.append((core + ((0, CHANNELS.index(pre[0]), pre[1]) if pre else (1, 0, 0)), pre, tag))
    return sorted(out, reverse=True)


def manifest_version():
    with open(os.path.join(ROOT, "typst.toml"), "rb") as f:
        return tomllib.load(f)["package"]["version"]


def messages(since, until="HEAD"):
    rng = f"{since}..{until}" if since else until
    return [m for m in git("log", "--format=%B%x00", rng).split("\x00") if m.strip()]


def parse_version(text):
    m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", text.strip())
    if not m:
        sys.exit(f"not a version X.Y.Z: {text!r}")
    return tuple(int(x) for x in m.groups())


def next_version(channel, explicit=None):
    stable = [t for t in tags() if t[1] is None]
    last = stable[0][0][:3] if stable else (0, 0, 0)
    last_tag = stable[0][2] if stable else None
    if explicit:
        target = parse_version(explicit)
    else:
        parsed = [p for p in map(parse, messages(last_tag)) if p]
        if not parsed and channel == "stable" and last_tag:
            sys.exit(f"nothing to release: no conventional commits since {last_tag}")
        major, minor, patch = last
        if any(p["breaking"] for p in parsed):
            target = (major, minor + 1, 0) if major == 0 else (major + 1, 0, 0)
        elif any(p["type"] == "feat" for p in parsed):
            target = (major, minor + 1, 0)
        else:
            target = (major, minor, patch + 1)
        planned = parse_version(manifest_version())
        if planned > last:
            target = max(target, planned)
    if target <= last:
        sys.exit(f"{'.'.join(map(str, target))} is not newer than the last release {last_tag}")
    version = ".".join(map(str, target))
    if channel != "stable":
        # every tag counts, also ones on other branches
        taken = []
        for tag in git("tag", "--list", f"v{version}-{channel}.*").split():
            m = TAG.match(tag)
            if m:
                taken.append(int(m[5]))
        version += f"-{channel}.{max(taken, default=0) + 1}"
    if git("tag", "--list", f"v{version}").strip():
        sys.exit(f"v{version} is already tagged")
    return version


def bump(version):
    m = TAG.match("v" + version)
    if not m:
        sys.exit(f"not a release version: {version!r}")
    package = ".".join(m.group(1, 2, 3))
    old = manifest_version()

    def edit(path, *subs):
        path = os.path.join(ROOT, path)
        text = open(path, encoding="utf-8").read()
        new = text
        for pattern, repl in subs:
            new = re.sub(pattern, repl, new, flags=re.M)
        if new != text:
            open(path, "w", encoding="utf-8").write(new)
            print(f"updated {os.path.relpath(path, ROOT)}", file=sys.stderr)

    edit("typst.toml", (r'^(version\s*=\s*")[^"]*(")', rf"\g<1>{package}\g<2>"))
    for cargo in git("ls-files", "Cargo.toml", "*/Cargo.toml").split():
        edit(cargo,
             # the workspace version
             (r'^(\[workspace\.package\][^\[]*?^version\s*=\s*")[^"]*(")', rf"\g<1>{version}\g<2>"),
             # exact pins between the workspace crates
             (r'^(solveng-[\w-]+\s*=\s*\{[^}\n]*version\s*=\s*")=[^"]*(")', rf"\g<1>={version}\g<2>"))
    if old != package:
        for path in git("ls-files").split():
            if DOC_FILES.match(path):
                edit(path,
                     (rf"@preview/solveng:{re.escape(old)}\b", f"@preview/solveng:{package}"),
                     (rf"(solveng/(?:blob|tree)/)v{re.escape(old)}\b", rf"\g<1>v{package}"))


def notes(tag):
    """Markdown notes for TAG: what changed since the previous stable release (for a stable TAG)
    or the previous release of any kind (for a prerelease)."""
    m = TAG.match(tag)
    if not m:
        sys.exit(f"not a release tag: {tag!r}")
    previous = [t for t in tags(tag) if t[2] != tag and (m[4] or t[1] is None)]
    since = previous[0][2] if previous else None
    groups = {"breaking": [], "feat": [], "fix": [], "perf": [], "other": []}
    for message in messages(since, tag):
        p = parse(message) or {"type": "other", "scope": None, "breaking": False,
                                "subject": message.strip().splitlines()[0]}
        if (p["type"] == "chore" and p["scope"] == "release") or re.match(r"Merge ", p["subject"]):
            continue
        line = f"- {'**' + p['scope'] + ':** ' if p['scope'] else ''}{p['subject']}"
        groups["breaking" if p["breaking"] else p["type"] if p["type"] in groups else "other"].append(line)
    titles = {"breaking": "Breaking changes", "feat": "Features", "fix": "Fixes",
              "perf": "Performance", "other": "Other changes"}
    out = [f"## {titles[k]}\n\n" + "\n".join(v) + "\n" for k, v in groups.items() if v]
    if since:
        repo = os.environ.get("GITHUB_REPOSITORY")
        out.append(f"**Full changelog:** " + (f"https://github.com/{repo}/compare/{since}...{tag}" if repo else f"{since}...{tag}"))
    return "\n".join(out) or "No notable changes.\n"


def excluded(path, patterns):
    """Whether typst.toml's exclude globs leave a file out of the package."""
    parts = path.split("/")
    prefixes = ["/".join(parts[: i + 1]) for i in range(len(parts))]
    for pattern in patterns:
        anchored = pattern.startswith("/")
        pattern = pattern.strip("/")
        if any(fnmatch.fnmatchcase(p, pattern) for p in prefixes):
            return True
        if not anchored and "/" not in pattern and any(fnmatch.fnmatchcase(p, pattern) for p in parts):
            return True
    return False


def bundle(dest, ref="HEAD"):
    with open(os.path.join(ROOT, "typst.toml"), "rb") as f:
        exclude = tomllib.load(f)["package"].get("exclude", [])
    archive = subprocess.run(["git", "archive", "--format=tar", ref], cwd=ROOT, check=True, capture_output=True).stdout
    if os.path.exists(dest):
        shutil.rmtree(dest)
    os.makedirs(dest)
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        members = [m for m in tar.getmembers() if m.isfile() and not excluded(m.name, exclude)]
        tar.extractall(dest, members=members, filter="data")
    for m in members:
        print(m.name)


def main(args):
    if args[:1] == ["next"] and len(args) in (2, 3) and args[1] in CHANNELS:
        version = next_version(args[1], args[2] if len(args) == 3 and args[2].strip() else None)
        print(version)
        if os.environ.get("GITHUB_OUTPUT"):
            with open(os.environ["GITHUB_OUTPUT"], "a") as f:
                f.write(f"version={version}\ntag=v{version}\nprerelease={str(args[1] != 'stable').lower()}\n")
    elif args[:1] == ["bump"] and len(args) == 2:
        bump(args[1])
    elif args[:1] == ["notes"] and len(args) == 2:
        print(notes(args[1]))
    elif args[:1] == ["bundle"] and len(args) in (2, 3):
        bundle(*args[1:])
    else:
        print(__doc__, file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
