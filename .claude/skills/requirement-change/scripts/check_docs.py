#!/usr/bin/env python3
"""Checks the build documents for broken references and drift.

Run from the repository root after any change to docs/, CLAUDE.md, .claude/agents or .claude/skills:

    python3 .claude/skills/requirement-change/scripts/check_docs.py

It checks:
  1. Every FR-, E2E-, NFR- and report (R..) ID that is mentioned is defined.
  2. Every "Section N.M" in the PRD points to a heading that exists.
  3. No em dashes (Umar's writing rule).
  4. Each document's version line matches the PRD document table and its own last change log row.
Exits with code 1 if anything is wrong.
"""
import glob
import os
import re
import sys

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
DOCS = os.path.join(ROOT, "docs")
NAMES = {
    "prd": "PRD_Mandi_Software_v1.md",
    "nfr": "NFR_Mandi_Software_v1.md",
    "tech": "Tech_Stack_v1.md",
    "code": "Code_Standards_v1.md",
    "arch": "Architecture_v1.md",
}


def read(path):
    with open(path, encoding="utf-8") as f:
        return f.read()


docs = {k: read(os.path.join(DOCS, v)) for k, v in NAMES.items()}
extra = {}
for pattern in ("CLAUDE.md", ".claude/agents/*.md", ".claude/skills/*/*.md"):
    for path in glob.glob(os.path.join(ROOT, pattern)):
        extra[os.path.relpath(path, ROOT)] = read(path)
everything = {**{NAMES[k]: v for k, v in docs.items()}, **extra}

prd, nfr = docs["prd"], docs["nfr"]
fr_ids = set(re.findall(r"^-? ?(FR-\d+\.\d+) \[", prd, re.M))
e2e_ids = set(re.findall(r"^\| (E2E-\d+b?) \|", prd, re.M))
nfr_ids = set(re.findall(r"^\| (NFR-[A-Z]+-\d+) \|", nfr, re.M))
report_ids = set(re.findall(r"^\| (R\d+) \|", prd, re.M))
sections = set(re.findall(r"^#{2,3} (\d+(?:\.\d+)?)[. ]", prd, re.M))

problems = []
for name, text in everything.items():
    for ref in sorted(set(re.findall(r"\bFR-\d+\.\d+", text)) - fr_ids):
        problems.append(f"{name}: {ref} is not defined in the PRD")
    for ref in sorted(set(re.findall(r"\bE2E-\d+b?", text)) - e2e_ids):
        problems.append(f"{name}: {ref} is not a PRD test")
    for ref in sorted(set(re.findall(r"\bNFR-[A-Z]+-\d+", text)) - nfr_ids):
        problems.append(f"{name}: {ref} is not defined in the NFR")
    if name != NAMES["tech"]:  # Tech Stack mentions Cloudflare "R2"
        for ref in sorted(set(re.findall(r"\bR\d{1,2}\b", text)) - report_ids):
            problems.append(f"{name}: report {ref} is not in the PRD catalogue (ignore if it is a change log line)")
    if "\u2014" in text:
        problems.append(f"{name}: contains {text.count(chr(0x2014))} em dash(es)")

for ref in sorted(set(re.findall(r"Section (\d+\.\d+|\d+)\b", prd)) - sections):
    problems.append(f"PRD: 'Section {ref}' has no heading")

# Versions
table = dict(re.findall(r"^\| \d \| `(?:\.claude/agents/)?([^`]+)`[^|]*\|[^|]*\| v(\d+\.\d+)", prd, re.M))
if len(table) < 5:
    problems.append(f"PRD: document table not read correctly ({len(table)} rows found)")
for key, fname in NAMES.items():
    text = docs[key]
    m = re.search(r"^\| Version \| (\d+\.\d+)", text, re.M)
    head = m.group(1) if m else None
    rows = re.findall(r"^\| (\d+\.\d+) \| \d\d-\d\d-\d{4} \|", text, re.M)
    last = rows[-1] if rows else None
    if head is None:
        problems.append(f"{fname}: no version line")
        continue
    if table.get(fname) and table[fname] != head:
        problems.append(f"{fname}: version {head}, but the PRD document table says {table[fname]}")
    if last and last != head:
        problems.append(f"{fname}: version {head}, but the last change log row is {last}")

if problems:
    print("PROBLEMS FOUND:")
    for p in problems:
        print(" -", p)
    sys.exit(1)
print(f"OK: {len(fr_ids)} FR, {len(e2e_ids)} E2E, {len(nfr_ids)} NFR, {len(report_ids)} reports checked "
      f"across {len(everything)} files. Versions consistent. No em dashes.")
