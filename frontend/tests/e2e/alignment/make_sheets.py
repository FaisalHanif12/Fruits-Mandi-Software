"""Plan 39 (A3): one sheet for each screen, the picture before and the picture after side by side.

    python3 tests/e2e/alignment/make_sheets.py

Reads tests/e2e/output/screenshots/plan39/before/<screen>-<lang>-<width>[-t1].png and .../after/...,
writes tests/e2e/output/screenshots/plan39/sheets/<screen>.png. A sheet has a row for Urdu and one for
English at 1366 x 768, and the same at 1024 x 600; on the left the picture before, on the right the
picture after. The close-up of the first table (-t1) is used when both exist, because it shows the
alignment at full size; otherwise the whole window. Needs Pillow (python3 -m pip install pillow).
"""

import os
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "output", "screenshots", "plan39")
BEFORE = os.path.join(ROOT, "before")
AFTER = os.path.join(ROOT, "after")
SHEETS = os.path.join(ROOT, "sheets")

LANGS = [("ur", "Urdu"), ("en", "English")]
WIDTHS = [(1366, "1366 x 768"), (1024, "1024 x 600")]
PICTURE_W = 760
GAP = 12
LABEL_H = 26
BACKGROUND = (246, 244, 239)
INK = (28, 31, 35)
FAIL = (179, 38, 30)


def font(size):
    for name in ("/System/Library/Fonts/Helvetica.ttc", "/Library/Fonts/Arial.ttf"):
        if os.path.exists(name):
            return ImageFont.truetype(name, size)
    return ImageFont.load_default()


def pick(folder, screen, lang, width, close_up):
    suffix = "-t1" if close_up else ""
    path = os.path.join(folder, f"{screen}-{lang}-{width}{suffix}.png")
    return path if os.path.exists(path) else None


def fitted(path):
    image = Image.open(path).convert("RGB")
    if image.width > PICTURE_W:
        height = round(image.height * PICTURE_W / image.width)
        image = image.resize((PICTURE_W, height), Image.LANCZOS)
    return image


def screens():
    found = set()
    for folder in (BEFORE, AFTER):
        if not os.path.isdir(folder):
            continue
        for name in os.listdir(folder):
            if name.endswith(".png") and not name.endswith("-t1.png"):
                parts = name[:-4].rsplit("-", 2)
                if len(parts) == 3 and parts[2].isdigit():
                    found.add(parts[0])
    return sorted(found)


def make(screen):
    rows = []
    for width, size_name in WIDTHS:
        for lang, lang_name in LANGS:
            close_up = pick(BEFORE, screen, lang, width, True) and pick(AFTER, screen, lang, width, True)
            before = pick(BEFORE, screen, lang, width, bool(close_up))
            after = pick(AFTER, screen, lang, width, bool(close_up))
            if before is None and after is None:
                continue
            rows.append((f"{lang_name} {size_name}", before, after))
    if not rows:
        return None
    pictures = []
    for caption, before, after in rows:
        left = fitted(before) if before else None
        right = fitted(after) if after else None
        height = max(image.height for image in (left, right) if image is not None)
        pictures.append((caption, left, right, height))
    total_h = sum(LABEL_H + height + GAP for _, _, _, height in pictures) + GAP + 34
    sheet = Image.new("RGB", (PICTURE_W * 2 + GAP * 3, total_h), BACKGROUND)
    draw = ImageDraw.Draw(sheet)
    draw.text((GAP, 8), f"{screen}     before (left)  |  after (right)", fill=INK, font=font(18))
    y = 34
    for caption, left, right, height in pictures:
        draw.text((GAP, y + 4), caption + " - before", fill=FAIL, font=font(14))
        draw.text((PICTURE_W + GAP * 2, y + 4), caption + " - after", fill=INK, font=font(14))
        y += LABEL_H
        if left is not None:
            sheet.paste(left, (GAP, y))
        else:
            draw.text((GAP, y + 8), "(no picture before)", fill=INK, font=font(14))
        if right is not None:
            sheet.paste(right, (PICTURE_W + GAP * 2, y))
        else:
            draw.text((PICTURE_W + GAP * 2, y + 8), "(no picture after)", fill=INK, font=font(14))
        y += height + GAP
    return sheet


def main():
    os.makedirs(SHEETS, exist_ok=True)
    made = 0
    for screen in screens():
        sheet = make(screen)
        if sheet is None:
            continue
        sheet.save(os.path.join(SHEETS, f"{screen}.png"), optimize=True)
        made += 1
    print(f"{made} sheets in {os.path.normpath(SHEETS)}")


if __name__ == "__main__":
    sys.exit(main())
