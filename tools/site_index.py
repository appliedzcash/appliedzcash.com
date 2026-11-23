"""Write the two machine indexes of the site: sitemap.xml and llms.txt.

The deploy step runs this tool. The sections and the page order come from the
home page. The date of a sitemap entry is the date of the last commit that
changed the page, so the checkout needs the full history. The text of llms.txt
is the og:title and the description of each page.
Run: python3 tools/site_index.py <output folder>
"""

import re
import subprocess
import sys
from html import unescape
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SITE = "https://appliedzcash.com/"


def meta(text, attribute, name):
    return unescape(re.search(rf'<meta {attribute}="{name}" content="([^"]*)"', text).group(1))


def last_change(path):
    log = ["git", "log", "-1", "--format=%cs", "--", path + "index.html"]
    return subprocess.run(log, cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()


home = (ROOT / "index.html").read_text()


def position(path):
    section = home.find(f'<h2 id="{path.split("/")[0]}">')
    found = home.find(f'href="{path}"', section)
    return found if found >= 0 else len(home)


# A section is an <h2> of the home page with the id of a page folder.
sections = []
for folder, heading in re.findall(r'<h2 id="([a-z0-9-]+)">([^<]*)</h2>', home):
    paths = [f"{folder}/{page.parent.name}/" for page in ROOT.glob(f"{folder}/*/index.html")]
    if paths:
        sections.append((unescape(heading), sorted(paths, key=position)))

out = Path(sys.argv[1])

sitemap = ['<?xml version="1.0" encoding="UTF-8"?>', '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">']
for path in [""] + [path for _, paths in sections for path in paths]:
    date = last_change(path)
    lastmod = f"<lastmod>{date}</lastmod>" if date else ""
    sitemap.append(f"  <url><loc>{SITE}{path}</loc>{lastmod}</url>")
sitemap.append("</urlset>")
(out / "sitemap.xml").write_text("\n".join(sitemap) + "\n")

llms = [f"# {meta(home, 'property', 'og:site_name')}", "", f"> {meta(home, 'name', 'description')}"]
for heading, paths in sections:
    llms += ["", f"## {heading}", ""]
    for path in paths:
        page = (ROOT / path / "index.html").read_text()
        llms.append(f"- [{meta(page, 'property', 'og:title')}]({SITE}{path}): {meta(page, 'name', 'description')}")
(out / "llms.txt").write_text("\n".join(llms) + "\n")
