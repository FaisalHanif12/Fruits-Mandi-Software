# Requirement changes since v1.2 (read before resuming)

Written for `tech-lead`, 15-10-2026. Umar approved every change below. The run stopped on 13-10-2026 (usage limit) in the D1+D2 end gate. Since then the requirement documents moved from v1.2 to v1.3 and v1.4. They are not committed yet: they sit in the working tree next to the team's unfinished fix work.

## 1. Short answer: what this means for the D1+D2 run

- **No D1 or D2 code needs to change.** Every v1.3 and v1.4 change is built in D3, D4 or D5. The D1 and D2 acceptance items (D1-A1 to D1-A16, D2-A1 to D2-A17) are the same.
- **No golden test number changed.** E2E-19 and E2E-23 are the same. The worked day still balances at 85,048 on both sides (`worked_day.py`).
- **The D2 پیشگی screen stays as built.** In D4 it becomes the پیشگی type of one shared voucher screen (PRD FR-15.10). Do not rebuild it now.
- **New seeded accounts and controls (0104, 0305, 01040001, 030119998, 04010006, 05020001) are D4 work.** They go in a new numbered migration then. Do not add them to the D1+D2 migrations.
- **Commit the documents on their own** before any fix work: one commit, `docs: requirements v1.3 and v1.4`, with only the files listed in Section 4. Do not mix them with backend or frontend changes.

## 2. What changed, in plain words

### v1.3 (Umar's changes of 14-10-2026)

| Change | Where in the PRD | Built in |
|---|---|---|
| Partners put money in and take it back (capital) | 17.4, FR-17.7, FR-17.8; opening capital per partner in 17.1 | D3 (opening), D4 |
| Partner total (کل رقم): investment + commission + profit as one number, with a dated record of every movement | FR-17.9, C22, R31 | D4, D5 |
| The firm's own goods: buy, sell with commission, cost of goods sold, write-off, profit or loss | 12.10, FR-12.24 to FR-12.28, C21, R33 | D4 (plan 16b) |
| Print any day and any party; day summary | FR-19.10, R34 | D5 |
| الگ کھاتہ (a separate khata, for example a garden): minus when money goes out, plus when it comes back or its goods sell; never part of any partner's total | 16.6, FR-16.6, FR-16.7, control 0305, A50 | D4 (plan 17) |
| New edge cases, assumptions and tests | E50 to E55, A45 to A50, E2E-34 to E2E-38 | D4, D5 |

### v1.4 (Umar's change of 15-10-2026, for ease of use)

| Change | Where in the PRD | Built in |
|---|---|---|
| One خرچہ screen. A type list at the top: دکان خرچہ (split between partners by share), ذاتی خرچہ (one partner only, asks which partner and why), ذاتی خرچہ واپسی | 16.2, 16.3, FR-16.8 | D4 (plan 17) |
| Before save, the خرچہ screen shows the split (3,000 → 1,200 / 900 / 900) or the partner's total before and after (الف 996 → −1,004), from a backend preview that uses the same functions as the save | FR-16.8, Code Standards B5 | D4 |
| Related entries share one screen with a type list; changing the type keeps date, amount, mode and narration | FR-7.67, Code Standards F44 | D4 |
| وصولی, ادائیگی and پیشگی are one voucher screen; menu items and `Alt+V`, `Alt+A`, `Alt+P` open it with the type chosen | FR-15.10 | D4 (plan 15) |
| The app points to the right type and never switches by itself: an ادائیگی to a seller with nothing payable offers پیشگی; a دکان خرچہ paid to a partner's name offers ذاتی خرچہ | FR-7.68, A51, E56, E57 | D4 |
| New test of the خرچہ screen | E2E-39 | D4 (golden_day preview numbers, Windows run) |

No table, column or posting type changed in v1.4. Each type on a shared screen keeps its own voucher type, number series, posting and checks.

## 3. Document versions now

| Document | Version |
|---|---|
| PRD | 1.4 |
| Architecture | 1.4 |
| Code Standards | 1.4 |
| Money-calculations agent | 1.3 |
| NFR, Tech Stack | 1.2 (unchanged) |

`python3 .claude/skills/requirement-change/scripts/check_docs.py` passes: 209 FR, 43 E2E, versions consistent, no em dashes.

## 4. Files changed since commit `b0902a6` (requirements v1.2)

- `.claude/rules/PRD_Mandi_Software_v1.md`, `Architecture_v1.md`, `Code_Standards_v1.md`
- `.claude/agents/money-calculations.md`, `.claude/CLAUDE.md`
- `.claude/skills/requirement-change/scripts/worked_day.py`, `check_docs.py`
- `.claude/skills/print-and-export/SKILL.md`
- `docs/deliverables/README.md`, `D3-bikri.md`, `D4-money-in-out.md`, `D5-reports-backup-golive.md`, and this file

Also Umar's, and also to go in the same documents commit:

- `.claude/agents/tech-lead.md` and `code-reviewer.md`: the `model` line is now `sonnet` instead of `opus`, to save usage. Keep it.
- `docs/deliverables/D1-foundation.md`: the v1.2 one-user wording (no roles, D1-A13 is the sign-in check). The team already builds to it.

Check with `git diff b0902a6 -- .claude docs/deliverables/D1-foundation.md docs/deliverables/D3-bikri.md docs/deliverables/D4-money-in-out.md docs/deliverables/D5-reports-backup-golive.md docs/deliverables/README.md docs/deliverables/changes-v1.3-v1.4.md`. Everything else in the working tree is the team's own work: the end-gate fixes in `backend/` and `frontend/`, `docs/plans/`, `docs/reviews/`, `status.md`, `progress.json` and `docs/release/whats-new.md`.

## 5. Where the run stands (from status.md, 13-10-2026)

Plans 01 to 10 are built and committed (`c114967`). The end gate was in the fix phase:

- Done: backend's fix list (819 tests passed, money review PASS WITH MINOR); frontend's double-save fix.
- Still open: backend's balance speed fix (migration `0002_balance_indexes.sql`, still uncommitted), frontend's hand-off, the `pnpm perf` rerun, and the reviewer recheck of `docs/reviews/d1-d2-run-review.md`.

Before resuming, run `git status` and the full check set, so you know which fix work in the tree is finished and which is half done.
