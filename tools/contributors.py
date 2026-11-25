"""Write the list of contributors into a copy of the home page.

The list is the GitHub contributors of the repo, most commits first. Bots are
not in the list. Each name links to the GitHub profile. The deploy step runs
this tool on the built home page, so each merge to main refreshes the list.
Run it on index.html to refresh the copy in the repo.
The tool needs the gh CLI (in CI, with GH_TOKEN set).
Run: python3 tools/contributors.py <home page>
"""

import json
import re
import subprocess
import sys
from html import escape
from pathlib import Path

REPO = "appliedzcash/appliedzcash.com"
START, END = "<!-- contributors -->", "<!-- /contributors -->"


def api(path):
    out = subprocess.run(["gh", "api", path], capture_output=True, text=True, check=True).stdout
    return json.loads(out)


people = []
for user in api(f"repos/{REPO}/contributors?per_page=100"):
    if user["type"] != "User":
        continue
    name = api(f"users/{user['login']}")["name"] or user["login"]
    people.append(f'<a href="{escape(user["html_url"])}">{escape(name)}</a>')

if not people:
    sys.exit("GitHub returned no contributors")
names = people[0] if len(people) == 1 else ", ".join(people[:-1]) + " and " + people[-1]

page = Path(sys.argv[1])
text = page.read_text()
block = re.compile(re.escape(START) + ".*?" + re.escape(END), re.S)
if not block.search(text):
    sys.exit(f"{page} has no {START} block")
page.write_text(block.sub(lambda _: START + names + END, text))
