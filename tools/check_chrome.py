"""Check that the shared chrome is the same on every page.

Each page copies the header, the footer and the shared <head> links. This check
fails when one copy drifts. It also checks that the deploy step copies each
top-level folder that a page links to. Run: python3 tools/check_chrome.py
"""

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
pages = sorted({*ROOT.glob("index.html"), *ROOT.glob("*/*/index.html"), *ROOT.glob("404.html")})


def normalize(block):
    # Relative ("../../", "./") and absolute ("/") paths to the site root become one form.
    block = re.sub(r'(href|src)="(?:\.\./)*(?:\./)?/?', r'\1="', block)
    return re.sub(r"\s+", " ", block).strip()


def part(text, pattern):
    match = re.search(pattern, text, re.S)
    return normalize(match.group(0)) if match else None


SHARED = {
    "header": r"<header>.*?</header>",
    "footer": r"<footer>.*?</footer>",
    "head links": r'<link rel="icon".*?<link rel="apple-touch-icon"[^>]*>',
    "font preload": r'<link rel="preload"[^>]*>',
    "site script": r'<script type="module" src="[./]*scripts/site\.js"></script>',
    "skip link": r'<a class="skip" href="#main">[^<]*</a>',
}

errors = []
reference = {}
for page in pages:
    text = page.read_text()
    name = page.relative_to(ROOT)
    if '<main id="main">' not in text:
        errors.append(f"{name}: no <main id=\"main\">")
    for key, pattern in SHARED.items():
        value = part(text, pattern)
        if value is None:
            errors.append(f"{name}: no {key}")
            continue
        if key == "header":
            # The home page links its own sections; the other pages link the home page.
            value = value.replace('href="#', 'href="')
        reference.setdefault(key, (name, value))
        first, expected = reference[key]
        if value != expected:
            errors.append(f"{name}: {key} differs from {first}")

# The deploy step copies a fixed list. Each local folder that a page links to must be on it.
workflow = (ROOT / ".github/workflows/pages.yml").read_text()
copied = set(re.search(r"cp -r ([^\n]*) _site/", workflow).group(1).split())
linked = set()
for page in pages:
    for path in re.findall(r'(?:href|src)="(?:\.\./)*(?:\./)?/?([a-z0-9-]+)/', page.read_text()):
        linked.add(path)
tracked = set(subprocess.run(["git", "ls-files"], cwd=ROOT, capture_output=True, text=True).stdout.split())
for folder in sorted(linked):
    if any(t.startswith(folder + "/") for t in tracked) and folder not in copied:
        errors.append(f"pages.yml: the deploy step does not copy {folder}/")

if errors:
    print("\n".join(errors))
    sys.exit(1)
print(f"Shared chrome is the same on {len(pages)} pages.")
