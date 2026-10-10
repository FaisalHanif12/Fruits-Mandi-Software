---
name: requirement-change
description: Applies a requirement change that Umar has approved (a new rate, a corrected rule, a new or removed feature) to every build document, recalculates every affected number with a script, raises the document versions, updates tests and checks that nothing contradicts. Run it only when Umar asks for a change.
disable-model-invocation: true
argument-hint: "[the change, in Umar's words]"
---

# Apply a requirement change

**The change:** $ARGUMENTS

The six build documents are one connected system. A single change, such as "لاگا is 8 per نگ, not 3", touched more than 60 numbers across the PRD, the tests, the money agent and the diagrams. Missing one of them means Claude Code later builds the wrong thing. This skill makes sure every copy is updated, every number is recalculated, and the documents still agree.

Only `tech-lead` (or the main session) runs this skill. Teammates never edit `docs/`.

## Step 1: Pin down the change

1. Write the change in Umar's own words, and add a concrete example of what will be different. Example: "اکرم buys 20 at 600. Before: he pays 12,760. After: 12,880."
2. If the change can be read two ways, ask **one** question with 2 or 3 options, each with its example numbers (AskUserQuestion). Do not guess. A guessed rule costs more than a question.
3. Classify it:
   - **Rate or default** (commission, لاگا, a charge): numbers change everywhere.
   - **Rule** (a new field, a changed behaviour): FRs, edge cases, assumptions and tests change.
   - **Scope** (feature added or removed): MVP scope, the side menu, reports, data model, posting types, diagrams and agents change.
   - **Wording only:** the glossary and Fluent terms change.

## Step 2: Find everything it touches

Search all of these:

- `docs/*.md`
- `CLAUDE.md`
- `.claude/agents/*.md`
- `.claude/skills/*/*.md`
- once development has started: the golden tests, fixtures and Fluent files

Search for the requirement IDs, the Urdu and English words, the old numbers, and the table and column names. Make a list of every hit before editing anything.

Example searches:

```bash
grep -rn "لاگا\|laga" docs CLAUDE.md .claude
grep -rn -P "12,760|3,654|84,234" docs CLAUDE.md .claude
```

## Step 3: Recalculate, never by hand

If any number changes, run the worked-day calculator with the new values:

```bash
python3 ${CLAUDE_SKILL_DIR}/scripts/worked_day.py                      # today's numbers
python3 ${CLAUDE_SKILL_DIR}/scripts/worked_day.py --buyer-pct 5        # with the change
```

- The script follows the same rounding and split rules as `core::money`. Its defaults reproduce the current PRD. If they do not, stop: the PRD or the script has drifted, and that must be fixed first.
- If the change adds a new kind of calculation, extend the script first. Add the new rule, check that the defaults still give today's numbers, then apply the change.
- Make a before and after table of every number that moves. The trial balance must still balance.
- **Update the script's defaults** (the `--...` default values at the bottom of `worked_day.py`) to the new approved values in the same change, so the next run reproduces the new PRD.
- If development has started, also run the real golden tests after the code changes. The PRD and the code must give the same numbers.

## Step 4: Edit the documents

Edit in this order, so the master leads:

1. **PRD:** the requirement text, the defaults table (4.2), the worked day (4.4 to 4.10), the formulas (22), the edge cases (23), the assumptions (24, mark a confirmed one `[C]`), the scope (26), the tests (27), and the glossary if a word changed.
2. **NFR, Tech Stack, Code Standards, Architecture:** only where they repeat or depend on the changed point. Remove repeats rather than updating several copies of one rule.
3. **The money-calculations agent:** its golden numbers and required split tests.
4. **CLAUDE.md, the other agents and the skills:** any example, path or rule that mentions the change.
5. **Diagrams:** if Architecture changed, render every Mermaid block and fix any that fail:
   ```bash
   npx -y @mermaid-js/mermaid-cli -i diagram.mmd -o diagram.svg
   ```

Writing rules: plain, simple English, short sentences, no em dashes, and a concrete example for each new rule.

## Step 5: Versions

For each document that changed:

- raise its version: 1.1 becomes 1.2, then 1.3;
- add a change log row with the date, what changed, and "Approved by Umar";
- update the PRD document table (Section 0.1) and every "Depends on" or "Derived from" line that names it;
- for the money agent, update its version line.

## Step 6: Check

1. Run the document checker from the repository root:
   ```bash
   python3 ${CLAUDE_SKILL_DIR}/scripts/check_docs.py
   ```
   It must print `OK`. It checks FR, E2E, NFR and report IDs everywhere, PRD section references, document versions and em dashes. It does **not** check Code Standards rule IDs (M, B, F, S), assumption and formula IDs (A, C), or section references inside the other documents. Check those by search.
2. Ask an independent subagent, one that has not seen your edits, to review the changed sections for contradictions, missed copies and wrong numbers. Fix what it finds and run the checker again.
3. If development has started:
   - list the code changes for `tech-lead` as a plan file (`docs/plans/`), including the golden test numbers that change;
   - golden test expected numbers change **only** through this skill, never in a code task.

## Step 7: Report to Umar

Keep it short and plain:

- the change in one sentence;
- a before and after table of the numbers that moved, with one worked example shown step by step;
- the documents changed, with their new versions;
- any open question, or any assumption still to confirm with the client.
