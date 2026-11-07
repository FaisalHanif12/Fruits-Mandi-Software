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

Umar, 07-11-2026: accuracy and speed both matter; the aim is an assistant that is fully production ready.
Accuracy still comes first: a wrong figure is never traded for speed.

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

### 5.5 Reviewer study S2 (07-11-2026), findings as reported (paths under `backend/crates/assistant/src`)

CRITICAL (a wrong figure is shown)

- **R1 Q2436 last rate, "newest first" is not newest first.** `understand/reads.rs:1492-1498` (last_rate_plan sorts R2 by date desc, then entry desc, 1 row); `tools/cells.rs:368` (order_of has no arm for Cell::Doc, gives Order::None); `tools/report.rs:1001` (stable sort). The entry key compares nothing, so the row is the FIRST line of the newest day (R2 sorts entry_date, b.number, s.id ascending, `reports/src/reports/sales/register.rs:105`). Shows 3,830, right 3,940; also Q2444 (3,510, right 3,340), Q2447 (1,130, right 1,060), Q2449. Fix: give Cell::Doc an order (its number), ties by sale line id desc; better, R2 sorts date desc, number desc, s.id desc itself.
- **R2 Q2062 buyer-side commission, both sides read and added.** `understand/reads.rs:2319-2368` (party_item_figure): buyer + item + commission reads view A with side null, no instant line; the model adds seller_comm + buyer_comm; `tools/scope.rs:131-133` allows it. Shows 15,716 (8,130 + 7,586), right 7,586. Fix: for a buyer read view A with side buyer and give an instant line from buyer_comm, as figure.rs does for Who::Buyer.
- **R3 A بکری number is read but never used for a figure.** `understand/what.rs:1135` sets bikri_number; the guards of plan (`reads.rs:955-968`) never look at it; only Ask("audit") at `reads.rs:2019` uses it. "kal bikri number 5 ka commission kitna tha" gives the whole shop's commission for yesterday. Fix: plan() returns None for every figure or list when bikri_number is set (or plans R1 with the bikri filter).
- **R4 A quality is dropped for the last rate.** `reads.rs:972` checks last_one before `if u.quality` at `reads.rs:976`; last_rate_plan (`reads.rs:1454`) ignores u.grade; Rep has no grade. "awwal seb ka aakhri rate kya tha" shows the newest apple line of any quality. Fix: None when u.quality is set, or add grade to Rep and the R2 read.

HIGH (a filter is dropped but a figure is still shown)

- **R5 Same name in Urdu letters: the tag is ignored, a guess without asking.** `understand/who.rs:396` (tag added only when the question is not Urdu), `who.rs:211` (+4 for a whole name), `who.rs:716` (one top hit is Found at Medium; names_ok accepts Medium). "اکرم کا بقایا کتنا ہے" answers اکرم K (0) with no "کون سا؟". Any typed name that is another account's whole name ("faisal" with Faisal and Faisal Traders) is taken with no question, against FR-7.77. Fix: add the tag for Urdu too; `others` true on a short name is Several (ask).
- **R6 A packing word is dropped.** `reads.rs:516` (Cue::Unit only makes a Quantity measure); no packing filter in plan, An or Rep. "aaj kele ki peti ki bikri kitni hui" reads banana over all packings. Fix: None when a packing word is next to an item, and say it cannot split by packing (or add a packing filter to view A).
- **R7 A voucher number is dropped on Receipts.** `reads.rs:2571` (shop_figure Receipts reads R23 for the period). "aaj wasooli number 3 kitni thi" can cite all of today's receipts. Fix: refuse or ask when a digit follows a voucher word.
- **R8 Tiles pre-read with no filter before a not-sure question.** `run.rs:2618` (cues::tile_question), runs when plan_reads gave nothing; `scope_guard.rs:81` needs exactly one planned read. "aaj ki bikri kitni hui udhari" puts today_sales (cash plus credit) in front of the model. Fix: no tile pre-read when understanding found any filter word.
- **R9 A party pre-read over the whole books when the period word is not on the list.** `prefetch.rs:137-235` (PERIOD_WORDS) lacks مئی, may, parso, پرسوں, ab tak, mahiny, maheene, sall, hafty, septmber. "اکرم کا مئی کا کمیشن" pre-reads view C over the whole books. Fix: decide "period named" from understand, or skip the pre-read when not sure.
- **R10 A seller plus an item gets no instant answer and no two-figure rule.** `reads.rs:2347-2366`. "rahim ke kele ka is mahine commission" leaves the model to choose or add. Fix: two labelled instant lines, from the item row of view A, as seller_commission does for view B.

MEDIUM

- **R11 Quality questions read the wrong thing or sum a cut list.** (a) quality_plan (`reads.rs:1521`) does not check the figure kind: "doyam kele kitne bache hain" reads sales. (b) A quality with no grade named reads R2 at 100 rows (`reads.rs:1558`): the model sums a cut list. Fix: (a) limit to Sale, Commission, Laga, Rate; (b) view A grouped by grade, or refuse when R2 has more than 100 rows.
- **R12 An expense head with a period reads R29 with the default 20 rows** (`reads.rs:2419`): a head outside the first 20 is "not found" or 0. Fix: rows 100 or a `where` on the head.
- **R13 A digit the model types becomes the Owner's own number** (`echo.rs:154`, put_digits_back): "kya akram ka bakaya 5000 hai" → "ہاں، 5000" can pass with no read. Fix: echo a question number only with its unit word, never a bare amount next to a money word.
- **R13b Roman "kal" with no verb before a figure noun** ("kal commission kitna hai") is yesterday at Medium and answered at once (`understand/when.rs:1039`); Urdu "کل کمیشن" is unsure (`when.rs:1016-1036`). Fix: same Low confidence for Roman.

LOW

- **R14 The empty date "مورخہ ۔" comes from the eval runner**, not the app: `scripts/ai-eval/run.mjs:699` (collect() keeps only text parts). Fix: show date parts in the runner text (robustness owns the runner).

### 5.6 Reviewer study S2 addendum: speed (07-11-2026). Lines in reads.rs moved about 70 while backend edits; functions are named.

A. Real-time questions sent to the model that the app could answer alone or with one call (most common first):

- **S-A1** A party or item with another filter, for commission or sale: party_item_figure, item_figure rate branch (what_of(Rate) None). View A's row holds gross, seller_comm, buyer_comm, laga, total_comm, avg_rate. Instant lines: buyer → buyer_comm (also fixes Q2062); seller → two labelled lines; rate → avg_rate labelled average.
- **S-A2** Last rate: last_rate_plan plans no line. After the sort fix the one R2 row holds rate, date, item, buyer: instant "آخری ریٹ X، مورخہ D".
- **S-A3** Group (sabzi, phal) sale, commission, laga, stock: group_plan plans view A or R10 with no line. Instant from totals.
- **S-A4** Quality with a grade named: quality_plan grade branch, no line. Instant from view A.
- **S-A5** Period totals from one report: R23 receipts, R29 expenses, R12 cash book, R37 fruit, R29 partner money taken, R15 capital (capital_flow_plan): no line. Instant from each report's whole-filter totals.
- **S-A6** "Happened" questions get no planned read (happened guard in plan()): full loop, 3 calls, 9 to 12 s ("rahim ko is mahine kitni peshgi di", "karim ko kitni adaigi hui", "aaj kitna cash aya"). One call or instant from R15 by voucher type plus party, R23, or R12.
- **S-A7** Quantity (نگ): Measure::Quantity empties the lines though views A, B, C hold qty. Instant "X نگ".
- **S-A8** Measure::Other gets no read: naqdi or udhari bikri (R3, R4 totals), one commission side (view A with side), safi bikri (view B net), average rate (view A avg_rate). Planned, they are guarded and 1 call or instant.
- **S-A9** A party's balance on a past day: party_figure returns None when a period is set. One call or instant: R13 closing as of that day.
- **S-A10** A party card with a warning (`figure.rs:357`), and a seller with an open advance (`figure.rs:363`), go to the model. Instant: the same sentence plus the reviewed warning line, or two labelled lines.
- **S-A11** Comparisons of two periods (comparison_plan) are read but written by the model; view A compare holds earlier, difference, change. Instant.
- **S-A12** A truck number gets only a find; after one match with one بکری today, pre-read R1.

B. Slow spots:

- **S-B1** last_rate_plan's two-key sort forces read_all of the item's whole R2 (up to 10,000 rows, sorted in memory) for 1 row; with no period a rich year has over 10,000 banana lines and the read is refused, so it falls to the model. Fix: one SQL ORDER BY entry_date DESC, number DESC, s.id DESC LIMIT 1 in R2 (also fixes R1).
- **S-B2** Every sorted report is built twice (probe of 1 row, then the real build), `tools/report.rs:770` and `:816`: R20, R24, R10. Fix: check the sort column against the static column list.
- **S-B3** The shop dues answer builds R20 in full, twice, only for the button; the tile already holds the checked figure. Fix: rows 1, or the button opens R20 with no read.
- **S-B4** The partner overview runs one statement query per partner in a loop (`tools/report.rs:1509`); every کل رقم instant answer pays it. Fix: one query for every partner's closing.
- **S-B5** The name book of every account is built once per question (`run.rs:339`); at 20,000 accounts that is real work on each instant answer. Fix: keep it beside the search index, rebuild when the index changes.
- **S-B6** The model-path pre-read reads whole-book party figures (view C or B over all years) when the period word is missed (R9). Fix: as R9.

Reviewer's suggested order: S-B1 with R1, then S-A1 (carries Q2062), S-A5, S-A6, S-A7, S-A8, then S-B2, S-B3.

Checked and right: today, this and last week, month and year, a named month, up to now; zero-day answers in the instant path; shop dues against R20 and the tile; stock against the tile; group_plan and its guard; the capital plan; the ranking lists; "kul ugrahi" as total dues; one-day shop figures against R34.

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
