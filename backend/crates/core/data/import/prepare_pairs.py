#!/usr/bin/env python3
"""Turns downloaded label files into one plain file of (English label, Urdu label) pairs.

Part of the word list tooling (docs/research/translit-options.md Section 4, plan 05).
Run it with `python3 -I prepare_pairs.py ...` so nothing is imported from the folder of the
downloads. It reads, never executes, the downloads (they are untrusted data).

Usage:
  prepare_pairs.py wikidata <kind> <input.tsv> >> pairs.tsv
      <kind> is name or place. Input is the TSV a Wikidata SPARQL query returns with the
      columns ?item ?en ?ur (see README.md).
  prepare_pairs.py geonames <main PK.txt> <alternatenames PK.txt> >> pairs.tsv
      Populated places and administrative areas of Pakistan that have an Urdu alternate
      name. The English side is the GeoNames `name`, the Urdu side is the `ur` alternate name.
  prepare_pairs.py geonames-words <main PK.txt> [min_count] > place_words.tsv
      The words (letters only) of the names of populated places in Punjab (admin1 04) and
      Khyber Pakhtunkhwa (03) that occur in at least `min_count` place names (default 3), with
      their counts. These have an English side only; the word list tool proposes the Urdu with
      the spelling rules and marks the row `engine`, so the app still asks about it.

Output columns (tab separated): english label, urdu label, kind, source, id.
`id` is the number of the Wikidata item (Q123 gives 123) or the GeoNames id. The word list
tool keeps every item whose Wikidata number ends in 0 out of the word list (the hold-out set
of the accuracy test), so the id has to stay.
"""
import re
import sys

ITEM = re.compile(r"/entity/Q(\d+)>$")
LABEL = re.compile(r'^"(.*)"@(en|ur)$')


def unescape(text: str) -> str:
    return text.replace('\\"', '"').replace("\\\\", "\\")


def clean(text: str) -> str:
    return " ".join(text.split())


def wikidata(kind: str, path: str) -> None:
    with open(path, encoding="utf-8") as handle:
        next(handle, None)  # header
        for line in handle:
            columns = line.rstrip("\n").split("\t")
            if len(columns) < 3:
                continue
            item = ITEM.search(columns[0])
            english = LABEL.match(columns[1])
            urdu = LABEL.match(columns[2])
            if not (item and english and urdu):
                continue
            en, ur = clean(unescape(english.group(1))), clean(unescape(urdu.group(1)))
            if en and ur:
                sys.stdout.write(f"{en}\t{ur}\t{kind}\twikidata\t{item.group(1)}\n")


def geonames(main_path: str, alt_path: str) -> None:
    # Feature classes: P populated place, A administrative division (district, tehsil).
    wanted = {}
    with open(main_path, encoding="utf-8") as handle:
        for line in handle:
            c = line.rstrip("\n").split("\t")
            if len(c) > 8 and c[6] in ("P", "A") and c[8] == "PK":
                wanted[c[0]] = c[1]
    seen = set()
    with open(alt_path, encoding="utf-8") as handle:
        for line in handle:
            c = line.rstrip("\n").split("\t")
            if len(c) < 4 or c[2] != "ur" or c[1] not in wanted:
                continue
            key = (c[1], c[3])
            if key in seen:
                continue
            seen.add(key)
            en, ur = clean(wanted[c[1]]), clean(c[3])
            if en and ur:
                sys.stdout.write(f"{en}\t{ur}\tplace\tgeonames\t{c[1]}\n")


def geonames_words(main_path: str, min_count: int) -> None:
    counts = {}
    with open(main_path, encoding="utf-8") as handle:
        for line in handle:
            c = line.rstrip("\n").split("\t")
            if len(c) < 11 or c[6] != "P" or c[8] != "PK" or c[10] not in ("03", "04"):
                continue
            for word in re.split(r"[^A-Za-z]+", c[2] or c[1]):
                word = word.lower()
                if 3 <= len(word) <= 20 and word.isascii():
                    counts[word] = counts.get(word, 0) + 1
    for word, count in sorted(counts.items(), key=lambda item: (-item[1], item[0])):
        if count >= min_count:
            sys.stdout.write(f"{word}\t{count}\n")


def main(argv: list) -> int:
    if len(argv) == 4 and argv[1] == "wikidata" and argv[2] in ("name", "place"):
        wikidata(argv[2], argv[3])
        return 0
    if len(argv) == 4 and argv[1] == "geonames":
        geonames(argv[2], argv[3])
        return 0
    if len(argv) in (3, 4) and argv[1] == "geonames-words":
        geonames_words(argv[2], int(argv[3]) if len(argv) == 4 else 3)
        return 0
    sys.stderr.write(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
