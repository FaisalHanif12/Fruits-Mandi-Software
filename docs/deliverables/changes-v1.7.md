# Requirement changes v1.7 (read with changes-v1.6.md before resuming the D3+D4 run)

Written for `tech-lead`, 22-10-2026. Umar approved every change below (his answers of 20-10-2026 and 21-10-2026, from the client's own prints of 18-10-26). They are in the working tree with the v1.6 changes, not committed yet.

## 1. Short answer

- **What changed.** The client showed four of his old prints. Umar settled what they mean:
  - The buyer pays commission and لاگا on every line; the seller pays commission only. This is how the app already works, so no code changes. The client's real rates per item are typed in as rules at setup.
  - Three prints follow the client's own layout: the buyer's day slip and the all-buyers balance sheet (both R35), and ادھاری تفصیل (R5, lines written "(item qty . rate)").
  - Every other print uses our own clear layout (FR-19.13): one header with the firm, up to 3 contact persons and the trade mark; each party with its short code and name with tag; totals; "Page x of y".
  - A short tag (نشان) after a name, in English capital letters, tells two people with the same name apart: "عاشق ظفر K".
- **No money rule, posting, formula or test number changed.** The worked day, every E2E number and the trial balance of 85,048 are the same. E2E-43 keeps its numbers; the PRD now also shows them in the two R35 layouts.
- **One database column.** `accounts.tag` goes into the unreleased migration 0006 of plan 16b, with the v1.6 changes. Contacts and trade mark are two keys in the `settings` table, so they need no column.
- **Built work stays as built.** The tag on the account form, search and screens, the firm contacts, and the new header on the prints already built are a new plan in D4R: `17e-tag-and-prints`. D5 gives every new print the same header and footer, and builds R35 in the client's layouts.
- **Order is unchanged:** commit the documents (Section 5), finish plan 16b (with the tag column in 0006), then plan 17, the end gate and the D3+D4 DONE REPORT; then D4R (17b, 17c, 17d, 17e); then D5.

## 2. The changes

| # | Change | PRD | Built today? | Who builds it | Proved by |
|---|---|---|---|---|---|
| 1 | The buyer pays commission and لاگا on every line; the seller pays commission only; rates per item are rules | 11.2 note, A3 resolved | Yes, already so | Nothing to build; the D4R review checks that لاگا is still buyer side only | D4R review focus |
| 2 | The tag نشان: 1 to 4 English capital letters and dots, its own field, never inside a name; shown after the name on every screen and print; in the search keys; part of the duplicate check | 10.2, FR-7.25, FR-7.28, FR-7.40, FR-10.7, E38; Code Standards 4.2, B26, F38; Architecture 15.4 | No | Column: plan 16b (migration 0006). Use: D4R plan 17e | D4-A32, D4R-A10 |
| 3 | Firm settings: up to 3 contact persons (name in both scripts, phone) and the trade mark | 21 (Firm), Architecture 15.3 | No (plan 04 has name, mandi, address, phones) | D4R plan 17e | D4R-A11 |
| 4 | Clear prints everywhere: the same header and footer, each party with short code and name with tag, labelled columns, totals under their columns, "Page x of y" | FR-19.13 | No | Prints already built (seller bill, buyer slips, پختہ slip, receipt slip, اگراہی فہرست): D4R 17e. Every D5 print: D5 plans 20 and 21 | D4R-A12, D5-A26 |
| 5 | R5 ادھاری تفصیل in the client's layout: serial, party, تفصیل with "(item qty . rate)", رقم, رقم بعد لاگا | 19.2 R5 | Built in plan 14 as "(item qty @ rate)" | D4R 17e | D4R-A12 |
| 6 | R35 in the client's two layouts: one party's day slip (تازہ بنام, سابقہ رقم, موقع پر ادا, وصولی, ٹوٹل رقم) and all parties as one table (کوڈ, پارٹی, سابقہ, بنام, جمع, بقایا) | FR-19.12, 19.2 R35 | No | D5 plan 20 | D5-A23 |
| 7 | Glossary words: تازہ بنام, سابقہ رقم, ٹوٹل رقم, بقایا, نشان | 3 | No | `glossary.ftl` in both languages: نشان in 17e, the R35 words in D5 | `pnpm i18n:check` |
| 8 | E2E-36 writes R5 the new way: "(کیلا 20 . 600), 12,000, 12,880"; no number changed | 27.2 | Built as the old form in plan 14 | D4R 17e | D4R-A12, D5-A21 |

Example for change 2: the client has two buyers called عاشق ظفر. One gets the tag K. The prints read "عاشق ظفر K" and "عاشق ظفر", and typing "ashiq k" in a picker finds the first one, listed first. Same name and phone with a different tag is not a duplicate.

Example for change 5, after Events 1 to 5: R5 for 10-10-26 shows اکرم ٹول ٹیکس "(کیلا 20 . 600)", 12,000, 12,880, and ارشد ملتانی "(کیلا 30 . 580)", 17,400, 18,684.

## 3. Plan 16b and plan 17

- **Plan 16b:** migration 0006 also adds `accounts.tag` (TEXT, null). Its migration test checks the column exists and is null on every migrated account (D4-A32). Nothing else in 16b changes.
- **Plan 17:** no change.
- New screens of 16b and 17 show names only through `displayName`, so they show the tag once 17e adds it there. Nothing extra to build in 16b or 17.

## 4. D4R and D5

- **D4R** (`D4R-refresh-v1.6.md` 1.1) gains plan 17e and acceptance items D4R-A10 to D4R-A12. The file keeps its name so every link works. It still needs no migration.
- **D5** (1.6): plan 20 builds R35 in the client's two layouts and puts the 17e header and footer on every report; plan 21 does the same for analytics prints; D5-A23 is wider and D5-A26 is new.

## 5. Files in the documents commit

The v1.6 documents are not committed yet either, so one commit holds both: `docs: requirements v1.6 and v1.7`. It replaces the commit named in `changes-v1.6.md` Section 6. Only these files:

- `.claude/rules/PRD_Mandi_Software_v1.md` (1.7), `Architecture_v1.md` (1.7), `Code_Standards_v1.md` (1.6), `Tech_Stack_v1.md` (1.5), `NFR_Mandi_Software_v1.md` (1.4, its "Depends on" line now names PRD v1.7)
- `.claude/agents/money-calculations.md` (1.5, its source line now names PRD v1.7), `.claude/CLAUDE.md`
- `.claude/skills/requirement-change/scripts/worked_day.py`, `.claude/skills/tauri-command/templates.md`, `.claude/skills/urdu-english-text/SKILL.md`, `.claude/skills/urdu-english-text/style-guide.md`, `.claude/skills/print-and-export/SKILL.md`
- `docs/deliverables/README.md` (1.7), `D4-money-in-out.md` (1.6), `D4R-refresh-v1.6.md` (1.1), `D5-reports-backup-golive.md` (1.6), `changes-v1.6.md`, and this file

Do not put the uncommitted plan 16b code in this commit. Delete any transfer file (`.v16src.tgz`, `.v17src.tgz`) in the project folder; never commit it.

## 6. Files the lead updates (lead-owned)

- `docs/plans/16b-own-goods.md`: migration 0006 also adds `accounts.tag`.
- `docs/deliverables/status.md`: the "Current" note says v1.7; a row for D4R plan 17e.
- `docs/deliverables/progress.json`: plan `17e-tag-and-prints` as waiting.
- `docs/design/design-system.md`: the 17e designs (D4R Section 3) during D4R; the R35 layouts during D5.

## 7. Still open, with the team default (do not stop for these)

- **A42** (Umar will answer later): some old کیلا lines have no rate and a lump amount (4 نگ = 2,950). The default stays: such items are sold by weight, with a rate per kg. Nothing in plans 16b, 17 or D4R depends on the answer.
- **A2:** no لاگا on vegetables by default. Umar's answer of 21-10-2026 confirms لاگا is on the buyer only; whether vegetables carry it is still a rule the Owner sets.
- **Buyer slips and R5** (team reading, no number changes): R5 ادھاری تفصیل lists credit buyers only, as its name says. The بکری buyer slips (FR-12.12) still show a named buyer's نقدی lines, because FR-12.9 says so; they use the R5 columns. D4R 17e builds it this way. If Umar reads it differently, only the slip filter changes. **Decided 25-10-2026:** Umar left this to the lead's judgement ("use the best way through your competency"). The lead keeps the reading above. Reasons: FR-12.9 says a نقدی line for a named buyer appears on his ledger and on his buyer slip, so the slip must show the whole day of that buyer; and ادھاری تفصیل is by its name the credit list, so cash lines stay off R5. This is a team decision, not a PRD change, so no document version changes. It stays in the DONE REPORT list so Umar can still reverse it (only the slip filter would change).
- A53 to A64 as in `changes-v1.6.md` Section 9.

List them in the DONE REPORT under open questions.

## 8. Document versions now

| Document | Version |
|---|---|
| PRD | 1.7 |
| Architecture | 1.7 |
| Code Standards | 1.6 |
| NFR | 1.4 |
| Tech Stack | 1.5 |
| Money-calculations agent | 1.5 |
| Deliverables | README 1.7, D4 1.6, D4R 1.1, D5 1.6 (D1, D2 and D3 unchanged) |

`python3 .claude/skills/requirement-change/scripts/check_docs.py` passes.
