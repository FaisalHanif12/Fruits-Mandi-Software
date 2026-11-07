# Plan 59: D7, real-time figures that are always right, and fast

| | |
|---|---|
| Deliverable | D7 (Umar, 07-11-2026, `docs/plans/59-realtime-accuracy-session-prompt.md`) |
| Branch | `feature/assistant-realtime`, from `feature/assistant` at 6735550 |
| Owner | tech-lead |
| Status | Step 1 (study) started 07-11-2026 |

## 1. Goal

When the Owner asks the assistant for a figure, list or count from the books, in any wording and with any
mix of filters, the answer equals the books exactly, or the app asks (which one, which dates) or says it
cannot. It never shows a guessed figure. Most such questions are answered by the app itself with no model
call, and fast. Help, advice, forecasts and what-ifs are not changed and not tested in D7.

## 2. Requirements

- PRD v1.31 7.7: FR-7.72, FR-7.74 to FR-7.79, FR-7.87, FR-7.90 (instant answers); the data sections the
  assistant reads: PRD 12, 13, 15, 16, 17, 18, 19 (R1 to R38), 22.
- NFR-AI-1 (every figure equals its source), NFR-AI-3 (speed), NFR-AI-5 (cost and size), NFR-AI-6 (read-only).
- Plan 55 Section 3 (hard limit: nothing over 25 s) and A2 targets (Section 10).
- Code Standards Q1 to Q11, M14; money agent 9.1 for any money read or sum.
- Tests that must stay green: `cargo test -p mandi-assistant`, `golden_day`, `assist`, `pnpm typecheck`,
  `pnpm lint`, `pnpm i18n:check`. No golden number changes.

Targets for real-time questions (D7 prompt Section 1, plan 55 A2):

| Measure | Target |
|---|---|
| Wrong figure, list or count | 0 (CRITICAL, always fixed) |
| D7 bank right | at least 98% |
| Right with no model call | at least 60% |
| Right with no call or one call | at least 85% |
| Wait | p50 2 s, p95 8 s, nothing over 25 s |
| Live test money | at most $2 for D7 (goal); Umar's prompt allows $3, the goal is stricter |

## 3. Worked numbers (from the oracle of the rich year, `tests/robustness_year`)

The two open CRITICAL wrong figures of plan 55 Section 15:

1. **Q2062** "pichle mahine Faisal se kele par kitna commission liya". Faisal is a buyer, so the commission
   taken FROM him is the buyer-side commission on his banana lines of last month (October 2026).
   Shown: 15,716 (seller + buyer commission). Right: 7,586 (buyer side only, oracle
   `ix.last_month.banana.faisal.comm_buyer`). 15,716 − 7,586 = 8,130 is the seller-side commission on the same
   lines, which the seller pays, not Faisal.
2. **Q2436** apple, last rate this week. Shown: 3,830 with an empty date ("مورخہ ۔"). Right: 3,940, the rate of
   the newest apple sale line this week (date descending, then entry descending; oracle
   `ix.this_week.apple.last_rate`).

Every other expected figure in the D7 bank is an oracle key (`@key`), never typed.

## 4. Contract between backend and frontend

No new Tauri command, input or output. `AskReply` (D6 Section 2) is unchanged. If a fix needs a new
outcome or field, backend proposes it here first and the lead records it before frontend builds.

## 5. Study results (Step 1, filled from the backend and reviewer HANDOFFs)

5.1 Paths a real-time question takes (file and line): instant answer; pre-read plus one model call; model loop.

5.2 Every place a filter can be dropped, changed or guessed (period, party, item, group, grade, cash or
credit, side, and the others).

5.3 Real-time question types that still need the model, why, and whether they could be answered with no call.

5.4 Plan 55 Section 15 open items, CRITICAL first, each with its cause.

(To be filled.)

## 6. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| S1 | Study the assistant paths and write 5.1 to 5.4 findings with file and line | backend | |
| S2 | Independent read of the same code for dropped filters and guessed figures (read-only) | reviewer | |
| B1 | The D7 bank `scripts/ai-eval/realtime-d7.tsv`: list every kind of record and figure the app keeps, about 200 questions, every expected figure an oracle key; an OFFLINE mode of the runner (no model) | robustness | |
| F1 | Fix wrong figures (Q2062, Q2436 and any found in S1, S2, B1 offline), each with a test on the rich year | backend | S1 |
| F2 | Fix dropped filters, then "not found" or time limit on a valid question | backend | F1 |
| F3 | Speed: more instant answers and pre-reads, fewer model calls | backend | F2 |
| M1 | Money agent review of any change to money reads or sums | backend (calls money-calculations) | F1 to F3 |
| O1 | The whole D7 bank offline, fixed until clean offline | backend, robustness | B1, F1 to F3 |
| L1 | Live run 1 of the D7 bank, after Umar's yes on the cost | tech-lead | O1 |
| F4 | Fixes from live run 1, each with its test | backend | L1 |
| L2 | Live run 2: the failed questions plus a few that passed, after Umar's yes | tech-lead | F4 |
| D1 | Final checks, "D7 REPORT", `status.md` | tech-lead | L2 |

Prompt files (`src/prompt/`) are edited only by the lead; the fixed part stays within 11,600 tokens.

## 7. Acceptance checklist

- [ ] D7-1 No wrong figure, list or count on the last live run (scorer: no `MISS (wrong)`).
- [ ] D7-2 At least 98% of the D7 bank right on the last live run (real-time questions).
- [ ] D7-3 Right with no model call at least 60%; right with no call or one at least 85%.
- [ ] D7-4 Wait p50 at most 2 s, p95 at most 8 s, none over 25 s.
- [ ] D7-5 Every fix has its test on the rich year with the oracle's figure.
- [ ] D7-6 Q2062 and Q2436 are fixed with tests.
- [ ] D7-7 Every kind of record and figure the app keeps has at least one D7 bank question.
- [ ] D7-8 Live money spent on D7 at most $2, written per run.
- [ ] D7-9 `cargo fmt --check`, `clippy -D warnings`, `cargo test -p mandi-assistant`, `golden_day`, `assist`,
      `pnpm typecheck`, `pnpm lint`, `pnpm i18n:check` clean. No golden number changed.
- [ ] D7-10 Money agent review PASS for every money read or sum changed.

## 8. Out of scope

Help and how-to, advice, recommendations, forecasts, what-ifs and estimates (not changed unless a real-time fix
needs it, not tested). The panel UI. Writing to the books. The reviewer and robustness gate (only after
Umar's "continue").

## 9. Open questions for Umar

1. Plan 55 item 5: "aaj ka rate" on a day with no sale says "not available"; the old bank expected the last
   rate (Q1598). Default for D7: the answer says there was no sale today and gives the last rate with its
   date. Needs Umar's yes.

## 10. Live runs (cost written before each run, Umar's yes recorded here)

| Run | Questions | Expected model calls | Estimated cost | Umar's yes | Actual |
|---|---|---|---|---|---|
