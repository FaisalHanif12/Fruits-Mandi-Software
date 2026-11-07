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

### 5.0 Backend study S1 and F1 (HANDOFF 07-11-2026)

**5.1 Paths.** `ask()` run.rs:2551. `understand_question` :647; `plan_reads` (reads.rs:1072). LOCAL, no call:
`local_answer` :733 → `understand::compose` (compose_route.rs:24): how-to, print, navigate, not in app, change
request, name not found; never a figure. INSTANT, no call: `instant_answer` :798 runs the planned reads, then
`compose_figure` (figure.rs:64, needs plan lines) and `Check::run`. PRE-READ + 1 call: help topic, planned reads,
period pre-read, tiles, prefind (off). MODEL LOOP: `drive` :1062 → `call_model` :908 → `run_calls` :372 →
`finish` :1172 (checks, scope guard, measure guard :1250, repairs, one rewrite, plain tables). `pick` :2688 (the
continuation after "کون سا؟") has no understanding and no planned reads, so no scope or measure guard (F2).

**5.2 Filters dropped or guessed (backend, checked in code).** The scope guard acts only when exactly one view or
report is planned (scope_guard.rs:81-90), so model-loop, comparison, list and not-sure questions have no filter
check. AskScope has no side, city, adda, collector, status, bikri, voucher type, packing, marka, store, truck. The
guard never checked a column (cause of Q2062; now done for the commission side). "naqdi/udhari" is
Measure::Other with no read. party_item_figure Sale: view A `gross` is the seller's amount, not what the buyer
paid. Voucher and truck numbers are not used for a figure. Plus the reviewer's R5 to R13b.

**5.3 Still needing the model, ranked by instant coverage gained:** (1) a party's ledger, statement, purchases,
day sheet: needs an instant answer with a table; biggest; (2) period totals from one report (S-A5); (3) quantity
(S-A7); (4) "happened" questions (S-A6); (5) Measure::Other (S-A8); (6) group and graded figures; (7) comparisons;
(8) past-day balance, card warnings, truck.

**5.4 Causes.** Q2062: party_item_figure planned view A with no side and no line; the model cited `total_comm` =
seller 7,586 + buyer 7,586 + Laga 544 = 15,716. Q2436: the second sort key (entry) gave Order::None for a Doc cell,
so ties kept the register's ascending order; line 1 of بکری 367 (3,830) came first, the last line is 3,940.

**F1 done (accepted 07-11-2026).** Q2062 15,716 → 7,586, Q2063 5,054 → 2,527, Q2436 3,830 → 3,940 (also Q2444,
Q2447, Q2449), all instant now; seller + item commission two labelled lines; buyer + item Laga instant; last rate
instant (seller rate; buyer rate for a named buyer) with gates (no sale, several packings, sold by weight, a
quality named: no instant line); newest line in one SQL page (date, بکری number, lot, line_no); R3 and R4 fixed;
`measure_guard.rs` on the model path. R2 to R4 gain `seller_rate` and `packing` columns only when needed. 16 tests
in `tests/d7_realtime.rs`. Money review: eighth pass PASS WITH MINOR (the minor, buyer_comm + laga in a calculate,
is F1b item 1). Run 9: 895 passed, 2 failed (backend's own, fixed and re-run green); golden_day 94, assist 23,
report tests green. Offline after F1 on the 412-question bank: 84 right with no call (20.4%), 6 wrong, 10 refused.
No prompt change: the check_failed reply names the reference to cite, and the prompt does not list rules one by one.

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

### 5.7 First offline run of the D7 bank (robustness B1, 07-11-2026; code mid-F1)

Bank `scripts/ai-eval/realtime-d7.tsv`: 239 questions, 103 groups (Roman 82, Urdu 74, English 83; 45 misspelt);
coverage of 50 kinds of record and figure in `realtime-d7-coverage.md`. Offline:
`node scripts/ai-eval/run.mjs --bank scripts/ai-eval/realtime-d7.tsv --offline` (no key, no network).

| Result | Count |
|---|---|
| Right with no model call | 17 (7.1%; target 60%) |
| WRONG with no model call | 6 |
| Needs the model | 208 |
| Asked or refused, not wanted | 8 |
| Estimated right in 0 or 1 call, if every 1-call answer is right | 43.5% (target 85%) |

Local wrong (all CRITICAL until shown otherwise):
- **O1 D084** "what is the total dues of the shop" (and Roman "kul ugrahi kitni banti hai"): shows buyers' dues
  20,324,367; right 21,751,319 = tile کل اگراہی = buyers' + sellers' dues + dead amount (PRD glossary, FR-19.9).
  Plan 55's ruling "kul ugrahi is R20" is out of date against PRD v1.31.
- **O2 D213** "what is Partner As totel in the shop": shows the partner screen help text, not 3,205,867.
- **O3 D045** "kitne bayopariyon ko abhi paise dene hain aur kul kitne": count 11 left out (total 17,560,964 shown).
- **O4 D121** "today's sales, the number of bills and the commission": bill count 2 left out.
- **O5 D144** "this month which sold more, banana or apple, and by how much": no difference or winner (apple by
  645,860); the row text "کیلا کی سے تک کی بکری" has empty from and to dates.
- **O6 D215** "usman ka sarmaya kitna hai": 200,000 right, the R31 button does not hold it (app or harness: robustness
  checks).

Refused as not found on a valid question (an ordinary word taken as a name): D062 "himself", D066 "themselves", D098
"raat", D105 and D106 "banaam", D111 "shuru", D119 "ریکارڈ"; D088 the collector رشید not found as a collector.

Wording gaps: Roman and Urdu forms go to the model where English is instant (Roman 5, Urdu 2, English 10 right);
53 of 74 one-figure questions need the model.

### 5.8 robustness B2 (HANDOFF 07-11-2026, accepted)

- Seed additions at the end of the rich year (`d7_richer_cases`, add-only, past-dated, through the app's saves):
  city Kasur; Naveed Traders with cash 20,220 and credit 54,792 on 14-10 (Umar's T01); Rafique Brothers; Aamir
  (پختہ کھاتہ person) and Nadir; Family Khata; a ریٹ مختلف line on 22-10; mosambi old بکری 16-10, new 19-10 and a
  later sale of the old leftover on 19-10. Moved: buyers' dues 20,844,363, total dues 22,271,315, cash 11,965,977.
- Oracle last_rate facts follow Section 11 item 2 (12 facts changed value). The oracle asserts that an edited line
  last by id is not last by the rule.
- Bank: 455 questions, 193 groups (Roman 156, Urdu 145, English 154; 55 misspelt; 221 short, 234 picky); N01 to
  N25 cover R1 to R13b; runner shows date and name parts (R14).
- Key-limit check built and not run: `node scripts/ai-eval/run.mjs --key-limit` (prints only limit, usage,
  limit_remaining, is_free_tier).
- Offline on the tree of B2 (before backend's F1b ended): right alone 90 (19.8%), wrong alone 1 (D331, the R31
  button: an APP problem, backend), needs the model 364, refused unexpectedly 0. A live run of the whole bank: about
  581 to 798 calls, $0.23 to $0.32. 25 of 193 groups end differently by wording.

### 5.9 backend F1b and F2 (HANDOFF 08-11-2026, accepted)

- F1b: the laga verb after a commission word; O1 total dues is the tile (buyers + sellers + dead amount); O2 to O5
  an answer the app writes never leaves out a part of the question.
- F2: R5 (the tag joins the name in Urdu typing; two accounts of one base name switch off the whole-name bonus), R6
  (a packing filter goes to the model), R7 (voucher number without the word "number"), R8 (tile read ahead also
  when not sure), R10, R11 (a) (the quality plan reads only its measures), R12 (R29 rows 100), R13 (digit echo only
  for months, days, years, percent or a plain number below 100; new `own_number_guard.rs`: an answer that cites
  only the Owner's own amount, or nothing while the question holds an amount, fails with rule `own_number`), R13b.
  R9 is code only (no test is possible). Tests: `tests/d7_realtime.rs`, 28 tests, each red before its fix except
  r5b_ (a regression guard).
- Offline on 455 questions: right alone 90 (19.8%), WRONG 1 (D331 only), refused unexpectedly 0; model calls
  0: 91, 1: 145, 2: 219; right with no call or one at most 51.6%.
- Money reviews 10 to 14 on F1b and F2: 10 PASS WITH MINOR, 11 to 13 FAIL (each fixed), 14 PASS WITH MINOR. Two
  small edits after review 14 were not reviewed again: the skip conditions of the own-number guard (they only skip
  it in more cases) and Purchases back in the R11 list (it re-admits what the plan read before F2).
- Addendum (08-11-2026): D331, a capital figure no longer gets an R31 button (the R31 page has no cell that holds
  it; `sources_cited` in fill.rs). R5, no account gets the whole-name bonus; "faisal" with Faisal and Faisal
  Traders, "اکرم" with اکرم ٹول ٹیکس and اکرم K, and "اسلم" with اسلم BA ask (tests r5_, r5b_, r5c_, r5d_,
  d331_). Money review 15: PASS WITH MINOR; its M2 to M4 fixed as proposed, not reviewed again; M1 open (Section 12).
  Offline unchanged: right alone 90, WRONG 1 (D331, now the runner's rule "every read source gets a button").

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

## 11. Lead decisions

1. (07-11-2026) "Rate" and "last rate" mean the seller rate (ریٹ بیوپاری), the auction rate typed by the munshi
   and shown by the rate hint (FR-7.53). The buyer rate only when the question names the buyer's rate.
2. (07-11-2026) The newest sale line is ordered by its own posting date desc, then the بکری number desc, then
   line_no desc. Not the row id: an edit keeps line_no, a later sale counts on its own date. The app and the
   oracle use the same rule.
3. (07-11-2026) Q2062 reading: commission taken from a named buyer is the buyer side; from a named seller, the
   seller side; a seller with no side word follows Umar's ruling of 05-11-2026.
4. (07-11-2026) The R2 to R4 screen order of lines on the same day may change with item 2 (a fix). No golden or
   services report number may change.
6. (07-11-2026, refines 1) When a buyer is named as the one who was sold to ("akram ko kele ka last rate", "akram
   ko kis rate pe diya"), the rate is the buyer rate, the rate charged to him. The seller rate is for an item
   alone, a seller, or the shop.
7. (07-11-2026) A money review that ends FAIL never closes a task. Items that over-refuse (no wrong figure) may be
   listed as MEDIUM follow-ups.
8. (07-11-2026) LOW follow-up, not in D7: an edit of a first posting after a later sale of the same lot on the
   same day puts the first posting's lines after the later sale in the newest-first order.
9. (07-11-2026) Registers R2 to R4 at the compact step (1024): with grade, packing and seller_rate all present,
   grade is shown first; packing and seller_rate wait for the wider steps (still in print and Excel).
   `REGISTER_WAITING` in `frontend/src/features/reports/columns.ts` holds the order.
10. Follow-up for the next plan (found 07-11-2026, already failing at the start of D7): two frontend contract tests
   in `tests/unit/reports-api-contract.test.ts` fail because the backend catalogue fixture lists a `grade` filter
   on R2 to R4 and the frontend copy does not; the frontend has no quality filter box on R2 to R4 yet.
11. Windows check LOCAL-ONLY: the new Urdu heading "ریٹ گاہک" and the register steps were checked in headless Chrome
   on the Mac only.
12. (08-11-2026) R6, R11, R13 and the R9 code stay in D7: they only ask, refuse or skip a read-ahead, and they
   are tested and money-reviewed. A typed name that is the whole name of one account and the first word of another
   ("faisal" with Faisal and Faisal Traders) asks "کون سا؟" (PRD FR-7.77: two to five matches ask). D331: a
   button opens only the report or screen the figure was read from.
5. New instant-answer words go to the Fluent files through a frontend teammate in one batch; prompt edits by
   the lead only.

## 10. Live runs (cost written before each run, Umar's yes recorded here)

| Run | Questions | Expected model calls | Estimated cost | Umar's yes | Actual |
|---|---|---|---|---|---|

## 12. Hand-over (08-11-2026, all agents stopped, waiting for Umar's "continue")

**State: D7 is NOT done.** F1, F1b and F2 are built and committed. F3 (speed), O1 (clean offline), the live runs
L1 and L2 and the D7 REPORT are not started. Live money spent on D7: $0.

Offline, 455 questions, no model: right alone 90 (19.8%, target 60%); right with no call or one at most 51.6%
(target 85%); WRONG 1 (D331, see item 1); refused unexpectedly 0.

Acceptance: D7-6 and D7-7 met. D7-5 met for every fix built (28 and more tests in `tests/d7_realtime.rs`).
D7-10 met: last money review 15, PASS WITH MINOR. D7-9 met on 08-11-2026 (lead run, see the commit). D7-1 to
D7-4 and D7-8 wait for F3 and the live runs.

Open, in the order to take them on "continue":
1. D331 ("usman ka sarmaya"): the figure is right and checked. The answer has no R31 button, because the R31 page
   holds no capital cell, but the runner wants a button for every read source. Lead's choice for the next step:
   a new `PartnerOverview` screen route with its Fluent label, so the answer opens the partner overview, which
   shows the capital (backend and frontend, and `pnpm gen:types`).
2. F3 speed: more instant answers and pre-reads (reviewer S-A1 to S-A12, S-B1 to S-B6 in Section 5.6). The biggest
   gap is in Roman and Urdu wordings (Section 5.7).
3. Review 15 M1: in the differences answer a capital figure keeps the R31 button and is called the partner's کل رقم.
   It needs Fluent words `assistant-figure-partner-capital` and a place word for `capital_history`.
4. Known gaps (no wrong figure; they ask or read more):
   - R6: a packing word further than two words from the sales word reads all packings. Design: a real packing filter
     on view A and R2.
   - R11 (b): rows listed one by one from a cut R2.
   - R13: a model-written `{{q.n1}}` next to a decorative book reference.
   - Review 14 item 1a (one tied hit with the role asked).
   - A party named like a STOP word.
   - "tadaad" and "kitni baar" are not count words.
   - "kitne bill aur kitne gahak" counts only the first.
5. MEDIUM: estimate and R33 over-refusals. LOW: the edit order of item 8 of Section 11. The `pick` continuation runs
   no guards.
6. Section 11 item 10: two frontend contract tests in `reports-api-contract.test.ts` (the grade filter on R2 to R4).
7. Section 11 item 11: Windows LOCAL-ONLY check of the new Urdu headings and the register steps.
8. 25 of 193 bank groups end differently by wording.

For Umar:
- The "aaj ka rate" default (Section 9, item 1).
- A prompt line for the new check rule `own_number`. Suggested: "own_number: you cited only the Owner's own amount,
  or nothing; read the books for this question and cite the figure." It needs his approval (D6 prompt Section 8);
  the fixed part must stay within 11,600 tokens.
- Before live run 1: run the key-limit check (`node scripts/ai-eval/run.mjs --key-limit`, not run yet). A whole-bank
  run is about 581 to 798 calls, $0.23 to $0.32. Ask Umar and wait for his yes.

## 13. Continue (08-11-2026, Umar: "continue and complete the D7 task")

Work restarts from Section 12. Umar's "continue" also lifts the hold on the reviewer and robustness gate, which
runs once at the end over the whole of plan 59. Live runs still need his yes on the cost before each one.

Contract change (agreed by the lead, both sides build it):
- `ScreenRoute` (backend `src/answer.rs`) gets `PartnerOverview` (`partner_overview`). It opens the partner overview
  (PRD 17.3), which is behind the partner code (FR-17.11) like any partner screen. A capital figure (`capital`,
  `capital_history`) gets this button instead of R31 (D331, Section 12 item 1).
- New Fluent IDs from backend, written by frontend in both languages: `assistant-figure-partner-capital`, the place
  word for `capital_history`, the label of the partner overview button, and each new instant-answer sentence of F3.

| # | Task | Owner | Blocked by |
|---|---|---|---|
| C1 | D331 backend: `PartnerOverview` route, capital figures get it; M1 words used; `pnpm gen:types` | backend | |
| C2 | D331 frontend: route mapping in `features/assistant/navigation.ts`, button label; M1 words; the two contract tests of Section 11 item 10 | frontend | C1 (types) |
| F3a | Speed batch 1: S-B1, S-A1, S-A2, S-A5, S-A6, S-A7, S-A8 | backend | C1 |
| F3b | Speed batch 2: S-A3, S-A4, S-A9 to S-A12, S-B2 to S-B6, Roman and Urdu wording gaps, Section 12 item 4 gaps, guards on the `pick` continuation | backend | F3a |
| F3w | Fluent words for every new instant sentence, in batches | frontend | F3a, F3b |
| O1 | Whole bank offline until clean; runner accepts the partner overview button for a capital read | robustness | F3b |
| L1, F4, L2, D1 | As Section 6 | lead, backend | O1 |
| G | Reviewer and robustness gate over all of plan 59 | reviewer, robustness | L2 |

Umar's answers (08-11-2026):
- "aaj ka rate" on a day with no sale of the item: the answer says there was no sale today, then gives the last rate
  with its date (Section 9 item 1 closed).
- The `own_number` prompt line is approved. D6 prompt v1.7: step 7 names the rule with Umar's wording; a few lines
  are shortened with the same meaning. Fixed part 11,600 tokens (P1 passes); `golden_prompt`, `golden_questions`,
  `prompt_columns` and `state_prompt` pass.
