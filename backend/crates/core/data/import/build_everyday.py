#!/usr/bin/env python3
"""Builds the everyday word batches (data/everyday/*.tsv) from the lemma files
(data/everyday/src/*.lex). Plan 99 U1a, PRD FR-7.27, NFR-URD-30.

Run from backend/crates/core:   python3 data/import/build_everyday.py
Sweep report (close ties):      python3 data/import/build_everyday.py --check --sweep
It writes data/everyday/<batch>.tsv (for review) and data/translit_everyday.tsv (embedded).
Check only (writes nothing):    python3 data/import/build_everyday.py --check
After a checker pass:           python3 data/import/build_everyday.py --pass <rows.tsv>

Lemma file format (one lemma a line):

    urdu = roman roman roman [@weight] [:CODE]

- The first roman is the usual spelling. It gives the highest weight, so it is also the
  English spelling offered for the Urdu word.
- `roman^` is as strong as the usual spelling (it must win a shared spelling).
- `roman:75` gives that spelling the weight 75, for a spelling shared with another word.
- `roman!` is an English spelling: no other forms of the word are built from it, so no
  half-English forms (nurseein, judgeon) are made.
- `roman~` is weak: used only when no other word claims that spelling.
- `_` in a roman is a space. The Urdu side may hold several words.
- `@w NN` on its own line sets the weight of the lines after it (default 80).
- `@NN` at the end of a line sets the weight of that line.
- `:CODE` adds the other forms of the word:
    V    verb: the Urdu is the stem (کر is written by hand, see the irregular verbs)
    M1   masculine ending ا          (لڑکا لڑکے لڑکوں)
    MY   masculine ending ی          (دھوبی دھوبیوں; no ـیاں)
    M2   masculine ending ہ          (پیسہ پیسے پیسوں)
    M3   masculine, other ending     (آم آموں)
    F1   feminine ending ی           (روٹی روٹیاں روٹیوں)
    F2   feminine, consonant ending  (کتاب کتابیں کتابوں)
    F3   feminine ending ا or و      (دعا دعائیں دعاؤں)
    ADJ  adjective ending ا          (اچھا اچھی اچھے)
    AT   plural in ات, added to the code before it (سوال سوالات), as `:M3,AT`

A plural code is the check that the plural is real: a word has one only if its plurals are
used. Spellings typed in a hurry (kam for kaam, pani for paani, kist for qist) are added with a low
weight, so they never win over a word that uses that spelling as its own.

One English spelling keeps at most three Urdu words (the three the app can show).
"""
from __future__ import annotations

import re
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "everyday"
# The one file the app embeds: every batch, each under a comment line with its name.
COMBINED = ROOT.parent / "translit_everyday.tsv"
SRC = ROOT / "src"
# The rows a checker has passed, one by one (roman<TAB>urdu). A row not in this file is
# `pending`, whatever made it: a new lemma or a changed rule never inherits an old pass.
PASSED_FILE = ROOT / "passed_rows.tsv"
ENGLISH_STEMS_FILE = ROOT / "english_stems.txt"
MAX_PER_ROMAN = 3
DEFAULT_WEIGHT = 80
INDEX_STEP = 1
MAX_INDEX_PENALTY = 6
WEAK_PENALTY = 35
WEAK_CEILING = 50
AUTO_PENALTY = 40
AUTO_CEILING = 44

# Letters allowed on the Urdu side (the same alphabet as core::text::is_urdu_letter, without
# Arabic look-alikes), plus the few marks a dictionary word may carry and a space.
URDU_LETTERS = set(
    "ءآأؤإئابپتٹثجچحخدڈذرڑزژسشصضطظعغفقکگلمنںوہھیےۂۃۓ"
)
URDU_MARKS = set("ًَُِّٰٔ")
# Words that must never be offered, whatever made them (checker report 1: a plural ending made
# a slur). A row whose roman holds one of these is never built.
BANNED_ROMAN = {
    "chutiya", "chutiyan", "chutiyon", "chutia", "chutian", "chutiye", "chutya", "chutiay",
    "chuttiya", "chootiya", "chootiyan", "choot", "chod", "choda", "chodi", "chode", "chodo",
    "chodu", "chodna", "chodne", "chodta", "chodti", "gand", "gaand", "gandu", "lund", "lauda",
    "loda", "lodha", "lora", "randi", "kanjar", "bhenchod", "madarchod", "bhosdi", "ghasti",
    "tatti", "lan", "bund",
}
# Hurried spellings that read as another word, so they are never built.
AUTO_BLOCK = {"wake", "chekue book", "fi", "maa", "gal", "kai", "qanoono", "qanunon", "kanoono", "kanunon"}
# Made-up plurals and mass nouns a checker refused: no row may have them (plan 99).
REFUSED_URDU = {
    "آراموں", "پیاسیں", "خونوں", "بھوکیں", "خوفوں", "جگروں", "پسینوں", "موچیں", "موچوں",
    "پیٹھیں",
}
# English words the final "y" rule also reads as a word ending in ے (checker round 3). That
# reading stays, but low.
ENGLISH_Y_WORDS = {"lady", "badly", "dairy", "jury", "many", "mary"}
ENGLISH_Y_WEIGHT = 30
# Single rows a checker capped (the highest weight they may have) or refused.
ROW_CAPS = {
    ("tayyary", "طیارے"): 35,
    # `mandi` is the shop's منڈی; مندی (slump, and a form of مندا) stays far under it.
    ("mandi", "مندی"): 40,
    ("dabe", "ڈبے"): 50,
    ("sodda", "سودا"): 30,
    ("dreveron", "ڈرائیوروں"): 40,
}
# People say and write پیمنٹس: its Urdu plural stays low.
CAPPED_URDU = {"پیمنٹیں": 50}
ROW_REFUSED = {
    ("talashy", "تلاشے"),
    ("balloon", "بلوں"),
    ("diari", "دیہاڑی"),
    ("khelo", "کھیلوں"),
}

ARABIC_BANNED = {"ي": "ی", "ك": "ک", "ه": "ہ", "ى": "ی", "ة": "ہ"}
ROMAN_RE = re.compile(r"^[a-z]+( [a-z]+)*$")


class Pair:
    __slots__ = ("roman", "urdu", "weight", "base", "auto", "batch", "order")

    def __init__(self, roman, urdu, weight, base, auto, batch, order):
        self.roman = roman
        self.urdu = urdu
        self.weight = weight
        self.base = base
        self.auto = auto
        self.batch = batch
        self.order = order


class Builder:
    def __init__(self):
        self.pairs: dict[tuple[str, str], Pair] = {}
        self.errors: list[str] = []
        self.banned: list[str] = []
        self.order = 0

    def add(self, roman, urdu, weight, base, auto, batch, where):
        roman = " ".join(roman.split())
        if "aaa" in roman or "eee" in roman or "ooo" in roman:
            return
        if any(word in BANNED_ROMAN for word in roman.split(" ")):
            if not auto:
                self.banned.append(f"{where}: {roman} for {urdu}")
            return
        urdu = unicodedata.normalize("NFC", " ".join(urdu.split()))
        if not ROMAN_RE.match(roman):
            self.errors.append(f"{where}: bad roman {roman!r} for {urdu}")
            return
        for c in urdu:
            if c in ARABIC_BANNED:
                self.errors.append(f"{where}: Arabic letter {c!r} in {urdu}")
                return
            if c != " " and c not in URDU_LETTERS and c not in URDU_MARKS:
                self.errors.append(f"{where}: bad letter U+{ord(c):04X} in {urdu}")
                return
        weight = max(1, min(100, weight))
        if roman in ENGLISH_Y_WORDS and urdu.endswith("ے"):
            weight = min(weight, ENGLISH_Y_WEIGHT)
        if (roman, urdu) in ROW_REFUSED or urdu in REFUSED_URDU:
            return
        weight = min(weight, ROW_CAPS.get((roman, urdu), weight))
        weight = min(weight, CAPPED_URDU.get(urdu, weight))
        key = (roman, urdu)
        old = self.pairs.get(key)
        if old is None:
            self.order += 1
            self.pairs[key] = Pair(roman, urdu, weight, base, auto, batch, self.order)
        elif weight > old.weight or (old.auto and not auto):
            old.weight = max(weight, old.weight)
            old.base = max(base, old.base)
            old.auto = old.auto and auto


# English spellings (`roman!`): no other forms are built from them.
ENGLISH_SPELLINGS: set[str] = set()


def parse_romans(tokens):
    """[(roman, strength)], strength in {'usual', 'normal', 'strong', 'weak'} or a whole
    number that is the weight of that spelling."""
    out = []
    for i, tok in enumerate(tokens):
        strength = "usual" if i == 0 else "normal"
        english = False
        if "!" in tok:
            tok, english = tok.replace("!", ""), True
        if ":" in tok:
            tok, number = tok.split(":", 1)
            strength = int(number)
        elif tok.endswith("^"):
            strength, tok = "strong", tok[:-1]
        elif tok.endswith("~"):
            strength, tok = "weak", tok[:-1]
        if english:
            ENGLISH_SPELLINGS.add(tok.replace("_", " "))
        out.append((tok.replace("_", " "), strength))
    return out


def weight_of(base, index, strength):
    if isinstance(strength, int):
        return strength
    if strength in ("usual", "strong"):
        return base
    if strength == "weak":
        return min(base - WEAK_PENALTY, WEAK_CEILING)
    return base - min(index, MAX_INDEX_PENALTY) * INDEX_STEP


def auto_variants(roman, urdu):
    """Spellings people type in a hurry. Always weak."""
    found = set()

    def add(v):
        if v and v != roman and ROMAN_RE.match(v) and len(v) >= 3 and v not in AUTO_BLOCK:
            found.add(v)

    if "aa" in roman:
        add(roman.replace("aa", "a"))
    if "ee" in roman:
        add(roman.replace("ee", "i"))
    if "oo" in roman:
        add(roman.replace("oo", "u"))
    if "q" in roman and "que" not in roman:
        add(roman.replace("q", "k"))
    if roman.endswith("ah") and urdu.endswith("ہ"):
        add(roman[:-1])
    if urdu.endswith("ے"):
        if roman.endswith("ay"):
            add(roman[:-2] + "e")
            add(roman[:-2] + "ey")
        elif roman.endswith("e") and not roman.endswith("ee"):
            add(roman[:-1] + "ay")
    # nahin -> nahi, yahan -> yaha. Not yahn -> yah, which reads as another word.
    if urdu.endswith("ں") and roman.endswith("n") and len(roman) > 4 and roman[-2] in "aeiou":
        add(roman[:-1])
    if urdu.endswith("یں") and roman.endswith("ain") and len(roman) > 4:
        add(roman[:-3] + "en")
    if urdu.endswith("ی") and roman.endswith("y") and not roman.endswith("ay"):
        add(roman[:-1] + "i")
    return sorted(found)


def emit(builder, urdu, romans, base, batch, where, auto=True):
    """Adds one Urdu form with its romans, and the hurried spellings of each strong one."""
    for index, (roman, strength) in enumerate(romans):
        builder.add(roman, urdu, weight_of(base, index, strength), base, False, batch, where)
    if auto:
        for roman, strength in romans:
            if strength == "weak":
                continue
            for variant in auto_variants(roman, urdu):
                weight = min(base - AUTO_PENALTY, AUTO_CEILING)
                builder.add(variant, urdu, weight, base, True, batch, where)


def plain_strength(strength):
    return "normal" if isinstance(strength, int) else strength


def combine_roman(stems, endings, secondary_endings=1):
    """Every stem with the endings; the first stem takes all endings, the others only the
    first `secondary_endings`. A weak stem or ending gives a weak spelling."""
    out = []
    seen = set()
    for si, (stem, sstrength) in enumerate(stems):
        sstrength = plain_strength(sstrength)
        use = endings if si == 0 else endings[:secondary_endings]
        for ei, end in enumerate(use):
            weak = sstrength == "weak" or end.endswith("~")
            end = end.rstrip("~")
            if end == "y" and stem.endswith(("y", "i", "e", "a", "o", "u")):
                continue
            word = stem + end
            if word in seen:
                continue
            seen.add(word)
            if weak:
                strength = "weak"
            elif si == 0 and ei == 0:
                strength = "usual"
            elif sstrength == "strong" and ei == 0:
                strength = "strong"
            else:
                strength = "normal"
            out.append((word, strength))
    return out


def strip_end(romans, endings):
    """The stem romans of a noun: each roman without its final vowel letters."""
    out = []
    for roman, strength in romans:
        for e in endings:
            if roman.endswith(e) and len(roman) > len(e) + 1:
                out.append((roman[: -len(e)], strength))
                break
        else:
            out.append((roman, strength))
    return out


SILENT_E_RE = re.compile(r"[b-df-hj-np-tv-z]e$")


def oblique(stem_ur):
    """The plural before a postposition: ـوں, and ـؤں after و (آلو آلوؤں)."""
    return stem_ur + ("ؤں" if stem_ur.endswith("و") else "وں")


def noun_forms(code, urdu, romans):
    """(urdu form, romans, weight drop) for every other form of a noun or adjective."""
    usable = [r for r in romans if r[0] not in ENGLISH_SPELLINGS]
    if code in ("M3", "F2"):
        # An English silent e (rate, phone, mobile) would give rateon: those are not built.
        usable = [r for r in usable if not SILENT_E_RE.search(r[0])]
    two = usable[:2]
    forms = []
    if not two:
        return forms
    if code in ("M1", "M2"):
        want = "ا" if code == "M1" else "ہ"
        if not urdu.endswith(want):
            raise ValueError(f"{code} needs a final {want}: {urdu}")
        stem_ur = urdu[:-1]
        stems = strip_end(two, ["aah", "aa", "ah", "a", "h"])
        plural = stem_ur + "ے"
        if code == "M2" and urdu.endswith("ایہ"):
            # کرایہ کرائے, سرمایہ سرمائے: the standard plural takes hamza.
            plural = urdu[:-2] + "ئے"
        forms.append((plural, combine_roman(stems, ["e", "ay", "y", "ey~"]), 2))
        forms.append((oblique(stem_ur), combine_roman(stems, ["on", "o~", "oon~"]), 5))
    elif code == "M3":
        forms.append((oblique(urdu), combine_roman(two, ["on", "o~", "oon~"]), 6))
    elif code == "MY":
        if not urdu.endswith("ی"):
            raise ValueError(f"MY needs a final ی: {urdu}")
        stems = strip_end(two, ["ey", "ee", "ie", "i", "y"])
        forms.append((urdu + "وں", combine_roman(stems, ["iyon", "ion"]), 6))
    elif code == "F1":
        if not urdu.endswith("ی"):
            raise ValueError(f"F1 needs a final ی: {urdu}")
        stems = strip_end(two, ["ey", "ee", "ie", "i", "y"])
        forms.append((urdu + "اں", combine_roman(stems, ["iyan", "ian", "iyaan~"]), 3))
        forms.append((urdu + "وں", combine_roman(stems, ["iyon", "ion", "iyo~", "io~"]), 6))
    elif code == "F2":
        forms.append((urdu + "یں", combine_roman(two, ["ein", "ain", "en"], 2), 3))
        forms.append((urdu + "وں", combine_roman(two, ["on", "o~", "oon~"]), 6))
    elif code == "F3":
        forms.append((urdu + "ئیں", combine_roman(two, ["ein", "ain", "yen", "en~", "yain~"], 2), 4))
        forms.append((urdu + "ؤں", combine_roman(two, ["on", "oon", "o~"]), 7))
    elif code == "ADJ":
        if not urdu.endswith("ا"):
            raise ValueError(f"ADJ needs a final ا: {urdu}")
        stem_ur = urdu[:-1]
        stems = strip_end(two, ["aa", "ah", "a"])
        forms.append((stem_ur + "ی", combine_roman(stems, ["i", "ee~"]), 1))
        forms.append((stem_ur + "ے", combine_roman(stems, ["e", "ay", "y", "ey~"]), 2))
    elif code == "AT":
        base_ur = urdu[:-1] if urdu.endswith("ہ") else urdu
        stems = strip_end(two, ["ah", "a"]) if urdu.endswith("ہ") else two
        forms.append((base_ur + "ات", combine_roman(stems, ["at", "aat"], 2), 6))
    elif code == "N":
        pass
    else:
        raise ValueError(f"unknown code {code}")
    return forms


def verb_class(stem):
    last = stem[-1]
    if last in ("ا", "آ"):
        return "A"
    if last == "و":
        return "O"
    if last == "ی":
        return "I"
    return "C"


# A two-syllable stem ending consonant, short vowel, one or two consonants.
SYNCOPE_RE = re.compile(r"^(.*[aeiou].*[b-df-hj-np-tv-z])[aiu]([b-df-hj-np-tv-z]{1,2})$")


def contracted(stem_ur, roman):
    """The spelling before a vowel ending when Urdu writes no vowel there: نکل nikal gives
    nikl (نکلا nikla, and نکالا keeps nikala). None if the vowel is a written long one."""
    letters = stem_ur[:-2] if stem_ur.endswith("ھ") else stem_ur[:-1]
    if not letters or letters[-1] in "اوییآ":
        return None
    match = SYNCOPE_RE.match(roman)
    return match.group(1) + match.group(2) if match else None


def vowel_stems(stem_ur, two):
    """The stems used before a vowel ending: each spelling with its short vowel dropped where
    Urdu writes none (پکڑ pakar, pakad give pakra, pakda), then the first full spelling, weak
    (pakara)."""
    short = [(contracted(stem_ur, roman) or roman, strength) for roman, strength in two]
    if short == two:
        return two
    return short + [(two[0][0], "weak")]


def verb_forms(stem_ur, stems):
    """(urdu form, romans, weight drop) for a regular verb. `stems` are the stem romans."""
    cls = verb_class(stem_ur)
    s = stem_ur
    two = stems[:2]
    out = [(s, stems, 0)]

    def f(urdu, endings, drop, secondary=1, use=None):
        out.append((urdu, combine_roman(use or two, endings, secondary), drop))

    # The infinitive and the habitual: the same for every class.
    f(s + "نا", ["na", "naa~"], 1, 1)
    f(s + "نے", ["ne", "ny", "nay", "ney~"], 2, 2)
    f(s + "نی", ["ni", "ny~", "nee~"], 6, 1)
    f(s + "تا", ["ta"], 2, 1)
    f(s + "تی", ["ti", "ty~", "tee~"], 3, 1)
    f(s + "تے", ["te", "ty", "tay", "tey~"], 3, 2)
    f(s + "تیں", ["tin", "teen", "ti~"], 9, 1)
    # Joined `kar` of a vowel stem reads as another word (pakar پکڑ, sukar سکڑ), so it is weak.
    joined = ["kar", "kr~"] if cls == "C" else ["kar~", "kr~"]
    f(s + " کر", joined, 12, 1)

    if cls == "C":
        before_vowel = vowel_stems(s, two)

        def f(urdu, endings, drop, secondary=1):
            out.append((urdu, combine_roman(before_vowel, endings, secondary), drop))

        f(s + "ا", ["a", "aa~"], 2, 1)
        f(s + "ی", ["i", "ee~"], 4, 1)
        f(s + "ے", ["e", "ay", "y", "ey~"], 3, 2)
        f(s + "یں", ["ein", "en", "ain"], 5, 2)
        f(s + "وں", ["un", "oon", "on~"], 6, 2)
        f(s + "و", ["o"], 4, 1)
        f(s + "یے", ["iye", "iay~", "ie~"], 14, 1)
        f(s + "وں گا", ["unga", "oonga", "onga~"], 8, 1)
        f(s + "وں گی", ["ungi", "oongi", "ongi~"], 10, 1)
        f(s + "ے گا", ["ega", "ayga", "ay ga~", "eyga~"], 7, 1)
        f(s + "ے گی", ["egi", "aygi", "eygi~"], 9, 1)
        f(s + "یں گے", ["enge", "ainge", "einge", "engy~", "aingay~"], 8, 1)
        f(s + "یں گی", ["engi", "aingi", "eingi~"], 12, 1)
        f(s + "و گے", ["oge", "ogay", "ogy~"], 11, 1)
        f(s + "و گی", ["ogi", "ogee~"], 14, 1)
    elif cls == "I":
        # پی سی جی are irregular: they are written out by hand.
        raise ValueError(f"write the forms of {s} by hand")
    else:
        # Vowel stems: آ کھا سو. The perfective takes یا.
        f(s + "یا", ["ya", "ia"], 2, 2)
        f(s + "ئی", ["i", "yi", "ee", "ie~"], 4, 2)
        f(s + "ئے", ["ye", "e", "ay", "ey", "aye~"], 3, 2)
        f(s + "ئیں", ["en", "yen", "ein", "in", "een~"], 7, 3)
        f(s + "ؤں", ["un", "oon", "on~", "ou~"], 6, 2)
        f(s + "ؤ", ["o~", "ou~"] if cls == "O" else ["o", "ou~"], 4, 1)
        f(s + "ئیے", ["iye", "iay~", "ie~"], 14, 1)
        f(s + "ؤں گا", ["unga", "oonga", "onga~"], 8, 1)
        f(s + "ؤں گی", ["ungi", "oongi~"], 10, 1)
        f(s + "ئے گا", ["yega", "ega", "ayga", "ye ga~"], 7, 2)
        f(s + "ئے گی", ["yegi", "egi", "aygi~"], 9, 2)
        f(s + "ئیں گے", ["yenge", "enge", "ayenge~", "einge~"], 8, 2)
        f(s + "ئیں گی", ["yengi", "engi"], 12, 1)
        f(s + "ؤ گے", ["oge", "ogay", "aoge~"], 11, 1)
        f(s + "ؤ گی", ["ogi"], 14, 1)
    return out


def parse_line(line):
    """urdu, [(roman, strength)], weight or None, [codes]."""
    codes = []
    weight = None
    if "=" not in line:
        raise ValueError("no =")
    urdu, rest = line.split("=", 1)
    tokens = []
    for tok in rest.split():
        if tok.startswith("@"):
            weight = int(tok[1:])
        elif tok.startswith(":"):
            codes = [c for c in tok[1:].split(",") if c]
        else:
            tokens.append(tok)
    if not tokens:
        raise ValueError("no romans")
    return urdu.strip(), parse_romans(tokens), weight, codes


def read_sources(builder):
    batches = []
    ENGLISH_SPELLINGS.update(
        line.split()[0]
        for line in ENGLISH_STEMS_FILE.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    )
    for path in sorted(SRC.glob("*.lex")):
        batch = path.stem
        batches.append(batch)
        default = DEFAULT_WEIGHT
        for number, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            line = raw.split("#", 1)[0].strip()
            where = f"{path.name}:{number}"
            if not line:
                continue
            if line.startswith("@w "):
                default = int(line[3:])
                continue
            try:
                urdu, romans, weight, codes = parse_line(line)
                base = weight if weight is not None else default
                if codes == ["V"]:
                    for form, forms_romans, drop in verb_forms(urdu, romans):
                        emit(builder, form, forms_romans, base - drop, batch, where)
                    continue
                emit(builder, urdu, romans, base, batch, where)
                for code in codes:
                    for form, form_romans, drop in noun_forms(code, urdu, romans):
                        emit(builder, form, form_romans, base - drop, batch, where, auto=False)
            except (ValueError, IndexError) as error:
                builder.errors.append(f"{where}: {error}")
    return batches


def resolve(builder):
    """At most MAX_PER_ROMAN Urdu words for one roman. Returns kept pairs and the dropped
    pairs that a person wrote (for review)."""
    by_roman: dict[str, list[Pair]] = {}
    for pair in builder.pairs.values():
        by_roman.setdefault(pair.roman, []).append(pair)
    kept, dropped = [], []
    for roman, items in by_roman.items():
        items.sort(key=lambda p: (p.auto, -p.weight, -p.base, p.order))
        kept.extend(items[:MAX_PER_ROMAN])
        dropped.extend(p for p in items[MAX_PER_ROMAN:] if not p.auto)
    return kept, dropped


# Ties a person has looked at: both readings are real, and this one is meant to be first.
REVIEWED_TIES_FILE = ROOT / "reviewed_ties.txt"


def reviewed_ties():
    if not REVIEWED_TIES_FILE.exists():
        return set()
    out = set()
    for line in REVIEWED_TIES_FILE.read_text(encoding="utf-8").splitlines():
        line = line.split("#", 1)[0].strip()
        if line:
            out.add(line)
    return out


def sweep_report(kept):
    """Romans that give two words within 10 of each other, not yet looked at."""
    by_roman: dict[str, list[Pair]] = {}
    for pair in kept:
        if not pair.auto:
            by_roman.setdefault(pair.roman, []).append(pair)
    reviewed = reviewed_ties()
    ties = []
    for roman, items in by_roman.items():
        items.sort(key=lambda p: -p.weight)
        if len(items) < 2 or items[0].weight - items[1].weight > 10:
            continue
        if items[0].weight < 60:
            continue
        line = f"{roman} {items[0].urdu}"
        if line not in reviewed:
            ties.append((items[0].weight, line, items))
    ties.sort(key=lambda t: (-t[0], t[1]))
    for _, line, items in ties:
        print("tie\t" + line + "\t" + " ".join(f"{p.urdu}({p.weight},{p.batch[:2]})" for p in items))
    print(f"ties not yet reviewed: {len(ties)}")


def row_keys(path):
    """The (roman, urdu) pairs of a list file, comments skipped: the first two columns, or the
    second and third if a header says the first is `batch`."""
    keys = []
    first = 0
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        cols = line.split("\t")
        if cols[0] in ("roman", "batch"):
            first = 1 if cols[0] == "batch" else 0
            continue
        keys.append((cols[first], cols[first + 1]))
    return keys


def passed_rows():
    return set(row_keys(PASSED_FILE)) if PASSED_FILE.exists() else set()


def review_of(pair, passed):
    return "checked" if (pair.roman, pair.urdu) in passed else "pending"


def mark_passed(path):
    """Adds the rows of a list a checker has passed (`roman<TAB>urdu ...`)."""
    new = set(row_keys(path))
    both = sorted(passed_rows() | new)
    head = "".join(
        line + "\n"
        for line in PASSED_FILE.read_text(encoding="utf-8").splitlines()
        if line.startswith("#")
    )
    PASSED_FILE.write_text(head + "".join(f"{r}\t{u}\n" for r, u in both), encoding="utf-8")
    print(f"passed rows: {len(both)} ({len(new)} in {path.name})")


def main():
    if "--pass" in sys.argv:
        mark_passed(Path(sys.argv[sys.argv.index("--pass") + 1]).resolve())
    check_only = "--check" in sys.argv
    verbose = "--dropped" in sys.argv
    sweep = "--sweep" in sys.argv
    builder = Builder()
    batches = read_sources(builder)
    if builder.errors:
        print("\n".join(builder.errors), file=sys.stderr)
        sys.exit(1)
    kept, dropped = resolve(builder)
    if builder.banned:
        print("banned spellings written in a lemma file:", file=sys.stderr)
        print("\n".join(builder.banned), file=sys.stderr)
        sys.exit(1)
    passed = passed_rows()
    by_batch: dict[str, list[Pair]] = {b: [] for b in batches}
    for pair in sorted(kept, key=lambda p: p.order):
        by_batch[pair.batch].append(pair)
    total = 0
    words = set()
    combined = ["roman\turdu\tkind\tweight\treview\tsource\n"]
    for batch in batches:
        rows = by_batch[batch]
        total += len(rows)
        words.update(p.urdu for p in rows)
        body = "".join(
            f"{p.roman}\t{p.urdu}\tword\t{p.weight}\t{review_of(p, passed)}\teveryday\n"
            for p in rows
        )
        waiting = sum(1 for p in rows if review_of(p, passed) == "pending")
        combined.append(f"# batch {batch}\n{body}")
        if not check_only:
            (ROOT / f"{batch}.tsv").write_text(
                "roman\turdu\tkind\tweight\treview\tsource\n" + body, encoding="utf-8"
            )
        print(f"{batch}: {len(rows)} rows, {len({p.urdu for p in rows})} urdu, {waiting} pending")
    if not check_only:
        COMBINED.write_text("".join(combined), encoding="utf-8")
    print(f"total: {total} rows, {len(words)} distinct urdu words, {len(dropped)} dropped")
    if sweep:
        sweep_report(kept)
    if verbose:
        for p in sorted(dropped, key=lambda p: p.roman):
            winners = [q for q in kept if q.roman == p.roman]
            winners.sort(key=lambda q: -q.weight)
            print(
                f"  dropped {p.roman} -> {p.urdu} ({p.weight}); kept "
                + ", ".join(f"{q.urdu}({q.weight})" for q in winners)
            )


if __name__ == "__main__":
    main()
