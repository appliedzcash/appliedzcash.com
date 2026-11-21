"""Render one social card for each basics page, pattern page and wishlist page.

The card of a page is images/og/<page folder>.png (1200 x 630). The card text
is the kicker and the og:title of the page. The home page keeps images/og.png
(source: tools/og.svg). Needs fontTools, brotli and rsvg-convert.
Run: python3 tools/og_cards.py
"""

import html
import os
import re
import subprocess
import tempfile
from pathlib import Path

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "images/og"
FONTS = {
    "AZSerif400": ("source-serif-4.woff2", 400),
    "AZSerif600": ("source-serif-4.woff2", 600),
    "AZSerif700": ("source-serif-4.woff2", 700),
    "AZMono400": ("jetbrains-mono.woff2", 400),
}
LEFT, WIDTH = 96, 1008
TITLE_FONT, TITLE_SPACING = "AZSerif700", -1.5
SUB_FONT, SUB_SIZE, SUB_LINE = "AZSerif400", 38, 50


def static_fonts(folder):
    """Write one static font file for each family in FONTS. Return the fonts by family."""
    fonts = {}
    for family, (file, weight) in FONTS.items():
        font = instantiateVariableFont(TTFont(ROOT / "fonts" / file), {"wght": weight})
        for record in font["name"].names:
            if record.nameID in (1, 4, 16):
                record.string = family
            elif record.nameID in (2, 17):
                record.string = "Regular"
            elif record.nameID == 6:
                record.string = family + "-Regular"
        font.flavor = None
        font.save(folder / f"{family}.ttf")
        fonts[family] = font
    (folder / "fonts.conf").write_text(
        f"<fontconfig><dir>{folder}</dir><cachedir>{folder}/cache</cachedir></fontconfig>"
    )
    return fonts


def text_width(font, text, size, spacing=0.0):
    cmap, advances = font.getBestCmap(), font["hmtx"]
    units = sum(advances[cmap[ord(char)]][0] for char in text)
    return units * size / font["head"].unitsPerEm + spacing * len(text)


def wrap(font, text, size, spacing=0.0):
    lines = []
    for word in text.split():
        if lines and text_width(font, f"{lines[-1]} {word}", size, spacing) <= WIDTH:
            lines[-1] = f"{lines[-1]} {word}"
        else:
            lines.append(word)
    return lines


def tspans(lines, first_y, line_height):
    return "".join(
        f'<tspan x="{LEFT}" y="{first_y + i * line_height}">{html.escape(line)}</tspan>'
        for i, line in enumerate(lines)
    )


def card(fonts, kicker, title):
    head, _, sub = title.partition(": ")
    sub = sub[:1].upper() + sub[1:]
    # The largest title size that needs two lines at most.
    size = next(s for s in (96, 84, 72, 64) if len(wrap(fonts[TITLE_FONT], head, s, TITLE_SPACING)) <= 2 or s == 64)
    head_lines = wrap(fonts[TITLE_FONT], head, size, TITLE_SPACING)
    sub_lines = wrap(fonts[SUB_FONT], sub, SUB_SIZE)
    head_height = len(head_lines) * size * 1.08
    sub_height = len(sub_lines) * SUB_LINE + 14 if sub_lines else 0
    # The text block is in the middle of the space between the site name and the URL.
    top = 172 + (316 - 46 - head_height - sub_height) / 2
    head_y = top + 46 + size * 0.82
    sub_y = head_y + (len(head_lines) - 1) * size * 1.08 + 14 + SUB_LINE
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630">
  <rect width="1200" height="630" fill="#f6f4ee" />
  <rect y="620" width="1200" height="10" fill="#e8a317" />
  <rect x="98" y="84" width="44" height="44" rx="12" fill="none" stroke="#16171a" stroke-width="4.25" />
  <circle cx="128" cy="114" r="6.75" fill="#e8a317" />
  <text x="164" y="119" font-family="AZSerif600" font-size="36" fill="#16171a">Applied Zcash</text>
  <text x="{LEFT}" y="{top + 20:.0f}" font-family="AZMono400" font-size="24" letter-spacing="3" fill="#8a5a00">{html.escape(kicker.upper())}</text>
  <text font-family="{TITLE_FONT}" font-size="{size}" letter-spacing="{TITLE_SPACING}" fill="#16171a">{tspans(head_lines, round(head_y), round(size * 1.08))}</text>
  <text font-family="{SUB_FONT}" font-size="{SUB_SIZE}" fill="#585d61">{tspans(sub_lines, round(sub_y), SUB_LINE)}</text>
  <text x="{LEFT}" y="548" font-family="AZMono400" font-size="24" fill="#8a5a00">appliedzcash.com</text>
</svg>"""


def main():
    OUT.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        fonts = static_fonts(tmp)
        env = {**os.environ, "PANGOCAIRO_BACKEND": "fc", "FONTCONFIG_FILE": str(tmp / "fonts.conf")}
        for page in sorted(p for s in ("basics", "patterns", "wishlist") for p in ROOT.glob(f"{s}/*/index.html")):
            text = page.read_text()
            kicker = html.unescape(re.search(r'class="kicker">([^<]*)', text).group(1))
            title = html.unescape(re.search(r'og:title" content="([^"]*)', text).group(1))
            svg = tmp / "card.svg"
            svg.write_text(card(fonts, kicker, title))
            png = OUT / f"{page.parent.name}.png"
            subprocess.run(["rsvg-convert", svg, "-o", png], env=env, check=True)
            print(png.relative_to(ROOT))


if __name__ == "__main__":
    main()
