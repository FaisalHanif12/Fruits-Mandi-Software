# Plan 55: ask the app, production-ready hand-off

Version 1.0, 04-11-2026. Lead: tech-lead. Asked by Umar on 04-11-2026: "make sure that you hand off an AI assistant that is diverse, robust and perfectly aligned with our existing data ... if we need to redesign it, rebuild it, do whatever you want ... take your time, but this time it must be flawless in every aspect ... speed is the second thing we can't compromise ... the hand-off must be production ready and deliverable."

This plan sits over plans 52, 53 and 54. Nothing is handed to Umar until every gate in Section 4 passes.

## 1. Why Umar keeps finding flaws in one or two questions

1. **We tested what we wrote.** The team writes the question bank, fixes against it, and passes it. Umar asks in his own words and finds the gaps the bank did not hold.
2. **The model decides too much.** For a plain question ("Faisal ki aaj ki bikri ka print") the model chooses the tool, the party, the period and the screen. Each choice costs a round trip of 4 to 9 s, and each can go wrong.
3. **Buttons were not checked against the data.** An answer can be right and still open the wrong screen, or the right screen with the wrong filters.
4. **Speed was measured on a good line in our runs, not on Umar's.** No question had a hard upper bound.

## 2. The design change: understand first, model last

Every question goes through three steps.

1. **Understand (the app, no model, about 50 ms).** The app reads the question for its intent (figure, list, print, how-to, forecast, advice, change request, not in the app), the parties (in their real roles), the items, the period, and the report or screen.
   - It uses the find index, the transliteration words, the item aliases, the guide and the period words. It works in Roman Urdu, Urdu, English and mixed, and forgives misspellings.
   - Each part carries a confidence.
2. **Read (the app, no model, about 50 ms).** When the understanding is sure, the app runs the reads itself: the same read-only tools, the same filters, the same checks. It also builds the buttons: the report or screen with its exact filters, built from the same filters as the read, so the button and the figure always match.
3. **Write (one model call, or none).**
   - **No model call:** help, how-to, print and navigation answers when the guide answers them; a "no such party" answer; a "the app does not keep this" answer.
   - **One model call:** figures and lists. The model gets the question and the read results, and writes the sentences with checked references (FR-7.76). It may still call a tool if it truly needs more.
   - **The current tool loop:** advice, forecasts, what-ifs, two-part and unusual questions, and anything the understand step is not sure about. It has a hard limit (Section 3).

Plan 54's S2 and S3 become steps 1 and 2. S1, S4, S5 and S6 stay as written.

## 3. Speed budget (Umar: over 30 s has no purpose)

| Kind | Target p50 / p95 | Hard limit |
|---|---|---|
| How-to, print, navigation, no such party, not in the app | under 1 s / 2 s | 3 s |
| Figures and lists (category A) | 5 s / 10 s | 25 s |
| Advice, forecasts, what-ifs (B, C) | 10 s / 20 s | 25 s |
| Mixed and hard (E) | 6 s / 12 s (plan 52) | 25 s |

- **The hard limit is real.** At 20 s the last call goes out with the tools off and answers from what has been read. At 25 s the panel shows what is read (the checked tables) with a plain line: "جواب میں دیر ہو رہی ہے، یہ پڑھا ہوا ریکارڈ ہے". No answer is ever lost.
- A slow line is shown as a slow line, never as an error.

## 4. Gates before the hand-off (all must pass, twice in a row where marked)

1. **The bank (twice in a row).** Every category meets plan 52 §2 and the speed budget above. It includes facets M, day, nosuch and print (plans 52 §8 and 54 §6). There are no lost or pending questions.
2. **A hidden bank.** An independent tester who is not on the build team writes 120 new questions in Umar's style, on every section, with no access to the team's bank or code. The builders never see these questions. The assistant must meet the same targets on them. A miss is fixed at its cause; the hidden question stays hidden, and the tester writes new ones for the next round.
3. **Every button is checked against the data.** For every answer in the bank and the hidden bank that has a button, a test opens that report or screen with the button's filters through the same backend command, and finds the answer's figures in it. A button that opens the wrong place or the wrong filters is a miss.
4. **Umar's own questions.** Every question in Umar's question log (read from a copy) is run again and must pass, including "main ne faisal ki aj ki bikri ka print nikalna hai?", "sales bherane ke tareeky batao?" and the market safar question.
5. **Not in the app.** Questions about things the app does not keep (weather, the rate in Lahore today, a bank balance, GST) get a polite "this app does not keep this" with the nearest thing it does keep, never a guess.
6. **Slow and lost lines.** The harness slows the line (delay and drop) and checks the hard limits: no question goes over 25 s, and no question is lost.
7. **The lead's own test in the running app.** I ask 30 questions of my own in the app window, in both languages. I check each answer, button and time myself, before I tell Umar it is ready.

## 5. Owners

| Work | Owner |
|---|---|
| Plan 53 (opening on any day, collection list, munshi header, today_open), already started | backend |
| Step 1, understand: a new module `assistant/src/understand/` (intent, parties in their roles, items, period words, report or screen, confidence), in new files only, with its own tests | backend2 (new) |
| Wiring steps 1 to 3 into the question path; read first; plan 54 S1 to S5; the hard limits | backend, after plan 53 |
| The panel: buttons, steps, progress line, the slow-line message | frontend2 |
| The bank, button checks, slow-line harness, attack tests | robustness |
| The hidden bank and its scoring | an independent tester the lead spawns; it reports only to the lead |
| Live runs, Umar's log replay, my own test, the hand-off report | lead |

## 6. Rules

- No figure is typed by the model. Every figure is checked (FR-7.76).
- Golden numbers never change.
- Requirement docs change only with Umar's approval.
- The key is never read, printed or written.
- The formal review gate (reviewer and robustness reviews of the whole run) waits for Umar's "continue", as before. These gates are acceptance tests of the assistant, not that gate.

## 7. Contract (agreed 04-11-2026, backend's proposal, lead decisions)

1. **A help or print answer made with no model call.**
   - `answered`, whose `answer.parts` hold only the guide's own words, names and dates.
   - A new field `how: HowStep[]` holds `{id, args}`, which frontend2 turns into reviewed text. The step ids:
     - `assistant-how-open-report` {report};
     - `assistant-how-set-party` {role, party, code};
     - `assistant-how-set-day` {day}, or `assistant-how-set-dates` {from, to};
     - `assistant-how-press-print`.
   - Dates are ISO. The steps use the same values as the button.
2. **The buttons.**
   - `answer.sources[0]` is `Report { result_id: "", id, filters }`. An empty `result_id` means the source is a link only.
   - `sources[1]` is the second button.
   - The filters per report are as in plan 54 §6.4.
3. **No such party, or the wrong role:** `not_found` with `asked_role` and `found_as`, and no model call.
4. **Progress,** `assistant://step`. New ids:
   - `assistant-step-understood` {kind};
   - `assistant-step-writing`;
   - `assistant-step-retrying` (after the 12 s first-byte limit);
   - `assistant-step-tools-off` (at 20 s).
5. **The slow line.**
   - At 25 s there is a new outcome `time_limit`: the plain tables of what was read, plus the "جواب میں دیر ہو رہی ہے، یہ پڑھا ہوا ریکارڈ ہے" line.
   - When nothing was read, FR-7.81's "جواب نہیں آ سکا" stands.
   - A slow line is never shown as offline or as an error.
   - Lead decision, to show Umar at the hand-over: the slow line keeps what was read.
6. **Counting.** A question with no model call does not count toward the 20 a day; this is already FR-7.81, which counts at the model's first reply. It is logged with 0 model calls and a `local` mark.
7. **The cued read (S3)** runs before call 1, and its results are the sources. Every figure is still checked.

## 9. Paused by Umar, 04-11-2026 about 04:20 (resume on his "continue")

Umar went to sleep and asked for every agent to be stopped. All work below is on disk in the d5 worktree, uncommitted unless a commit is named. Resume each item exactly from here.

| Who | State when paused | Next step on resume |
|---|---|---|
| backend | Wired understand, then compose (local answers), then plan_reads (read first, one call, low reasoning), the hard limits and warm-up into run.rs. Its full workspace test run was going. Plan 53 is done but not yet handed off. | Re-run the workspace tests (fmt, clippy, assistant, golden_day, services, reports, app). Send the plan 53 and plan 54/55 wiring HANDOFFs. |
| backend2 | Done: understand, compose_* (help, print, navigate, missing, outside, change), plan_reads (129 cases). On standby. | Re-run tests/understand.rs speed asserts on a quiet machine. Fix questions from the live runs. |
| backend3 | Done: plan 56 R2 (first-run restore, chain key in the copy), the MANDI_PROFILE dev switch, and the endpoint `reason`. Was starting robustness's 5 R4 findings. | Fix, in order: shop set up during a restore; a copy with no owner; a huge scrypt factor; the two nearly-full-disk checks. Then HANDOFF. |
| frontend2 | Built: the corner-band fix (content runs to the status bar; the round button floats), plan 54 screens (how steps, progress, time_limit, link buttons, find-row, outside texts). The wide run was clean (3,220 tests). A run on the layout-touched screens was going. | Finish that run and the endpoint-reason tests. Fix assistant-addresses.test.tsx (trip removed). Send the corner-fix and plan 54 HANDOFFs, with screenshots. |
| frontend3 | Built: plan 56 R3 (first screen, 4 restore steps, reminder card), 235 tests passing. Its full run was going. | Add the eye on the backup key, and the endpoint reason message. Send the HANDOFF. |
| robustness | Partly built: run.mjs --threads, --bank and --oracle-dump. The oracle dump is NOT written yet. Wrote robustness_restore_new_computer.rs (5 findings for backend3). It has 3 clippy errors there, plus robustness_security.rs:2078 and robustness_live.rs:952. | Finish --threads (default 3), --bank and --oracle-dump into the scratchpad. Fix its clippy errors. Add backend3's 3 command-table rows. Then R4 (2-year restore under 5 min). |
| hiddentester | The hidden bank is 124 questions in the lead's scratchpad (hidden-bank.tsv). Waiting for oracle-dump.json to fix its guessed names, the a2 key and the trip keys. | When the dump exists, fix those and re-check. Never show the questions to the build team. |
| lead | Docs up to date (PRD v1.24, NFR v1.14, Architecture v1.18, plans 52 to 56). | When every HANDOFF is in: commit all, rebuild the app, re-run the speed asserts on a quiet machine. Then the live bank (567, --threads), fixes, a second run, the hidden bank (--bank), Umar's log replay, R5 (MANDI_PROFILE=r5 against Umar's bucket), and the lead's own 30-question test. Then the hand-over report and stop the agents. |

Reminders for Umar: after the build, press Back up now once (older copies carry no chain key), and make sure his printed backup key sheet exists.
- **Note at the pause:** the generated `frontend/src/bindings/` folder in the working tree was empty, because a `pnpm gen:types` run was stopped part way. On resume, backend runs `pnpm gen:types` first, once the workspace compiles, before any frontend check. The committed bindings are in git.

## 10. Umar, 05-11-2026: focus on real-time figures, instant answers (PRD v1.25 FR-7.90)

After live run 1 (365/567; category A p50 9.8 s, 55% in one call or none), Umar asked for the most focus on real-time figure questions, with accuracy and speed, asked any way. He approved instant answers for simple figures.

- **backend2, the composer.** A new `compose_figure`, pure: it takes the Understanding plus the results of the planned reads and returns the Answer. It writes a reviewed sentence (a Fluent id per figure kind and role) with checked references and the buttons.
  - It covers: a party's balance, dues, payable or advance; a sale or commission (shop, item, seller, buyer) for today or a period; cash in hand; the shop's total dues or payables; goods left; a partner's کل رقم.
  - It returns None for anything else.
- **backend, the wiring.** When the read plan is sure and compose_figure gives an answer, there is no model call.
  - The same FR-7.76 checks run, including the two places that must agree. If a check fails, the question falls back to the one-call path.
  - It is logged as local, with the read entries.
- **frontend2.** The Fluent sentences in ur and en (# review: pending), and the panel shows them like any answer.
- **robustness.** 120 more category A questions in the team bank (facet A2), in every wording, every figure kind and role, today, yesterday, a month, a range and "ab tak", in Roman Urdu, Urdu, English and mixed, with misspellings. The scorer reports the instant-answer share for A.
- **hiddentester.** 100 more HIDDEN category A questions, written in Umar's style, which the builders never see.
- **Target for A:** at least 60% instant, at least 85% instant or one call, p50 2 s, p95 8 s, no wrong figure.

## 11. Umar, 05-11-2026: "every question type exposes a flaw". Cover the whole app, not just the questions we think of

Until now the banks grew from the questions we, or Umar, happened to ask. So each new kind of question, such as a forecast for one expense head, found a new gap. From now on, coverage starts from the app itself.

1. **Capability matrix (robustness).**
   - **Rows:** every thing the app holds or shows. That is every report R1 to R38, every analytics view A to K, every dashboard tile, every master list, every entry screen, every guide topic, and every partner, garden, trip and fruit figure.
   - **Columns:** every question type: figure, list, top, compare, a party's day, print or open, how-to, forecast or what-if, advice, mixed (real plus forecast), no such party or wrong role, not in the app, change request.
   - **Each cell** is either "answerable", with at least one bank question in three languages and a figure from the oracle, or "not answerable by design", with the honest answer it must give, which the lead approves.
   - No cell may stay empty. A cell the assistant can't answer but should is a defect: it is fixed, as the expense-head forecast is being fixed.
2. **Rounds until dry (hidden tester and lead).**
   - After each live run, the hidden tester writes 50 new hidden questions aimed at the types and wordings that failed or were thin.
   - The lead runs them.
   - The hand-over needs two rounds in a row with no new failure.
3. **The gates of §4 stay.** The full bank twice in a row, the hidden bank, button-vs-data, Umar's log, slow line, and the lead's own test.

### 11.1 First capability gaps (robustness, read from the code) and the lead's rulings (05-11-2026)

| Gap | Ruling | Owner |
|---|---|---|
| F1: estimate offers commission_buyer, laga, receipts, garden_sales and trip_sales, but they always fail | Build their monthly series (view G or the journal) so they work. Money agent. | backend3 |
| F2: forecast per expense head | Build: estimate takes `head`. Money agent. | backend3 (moved from backend) |
| F3: partner profit forecast | Build from the partner's monthly share rows (view F) | backend3 |
| F4: a party's balance "at this pace" | Build: the current balance plus the average monthly net change (bought minus paid) over the months he was active, shown as an estimate with its band. Money agent. | backend3 |
| F5: cash, total dues and stock forecast | Cash and total dues: the same month-end balance method as F4. Stock "when will it sell": not by design for now (an honest line). | backend3 |
| R1: commission and charge rules | Build a read-only read of the rules for a party, item, group or default. | backend4 (new) |
| R2: the attention list, "آج کے کام" | Build a read of assist_attention. | backend4 |
| R3: closed days and month close | Build a small read: closed up to, today open, last month close. | backend4 |
| R4: audit log | A read of the audit rows for one entry. | backend4, after the above |
| R5: master lists (staff, munshis, expense heads) | find roles for staff and expense heads, and counts. | backend4 |
| R6: a truck number | The bikri find by truck through the report or find tool. | backend4 |
| P1 to P4: report filters the tool doesn't pass (guests, voucher_type, status, detail, trip_view, garden_view, seller and zamindar on the sales reports, collector) | Pass them all through, as each report's spec allows. | backend4 |
| 18 reports with no print or open plan | Plan rows for all 38 reports. frontend2 checks each link. | backend2 |
| Change requests for garden or trip entries, later sale, edit, cancel | Entry routes for them. | backend2, with frontend2 for the routes |

- **Prompt.** New reads should extend existing tools, for example a `what` value on one small `app_state` tool, rather than add many tools, so the fixed part stays near 11,000 tokens. If it must grow, the limit may go up (Umar, 02-11-2026: "we can go up to align or achieve the goal"). The lead records the new limit in D6 v1.5 and NFR-AI-5.
- **Not by design (approved):**
  - phone, address and father or firm name (FR-7.83);
  - weather, tax, bank and outside market rates (an honest line);
  - a partner figure while the section is locked (it pauses for the code);
  - comparisons on views G and J, which are answered by two reads and a difference.

### 11.2 Not-by-design cells approved (05-11-2026)
These are forecasts the app does not keep. Each gives its honest answer, and the mixed twin still gives the real figure and invents no forecast figure:
- cash-only or credit-only sales (answered with the total sales forecast);
- per adda;
- payments;
- fruit taken (answered with what was taken so far).

Not approved: the audit log how-to. It becomes gap Q4b, a guide topic in D6 v1.5.

## 12. Paused by Umar, 05-11-2026 about 01:55 (agents kept alive, NOT stopped)

Umar: "stop the working of the agents ... do not kill or cancel any agents, just stop the working." Each teammate finishes only the edit it is in the middle of, sends the lead one state note, and waits. Everyone keeps its context. On "continue" the lead tells each to resume from its note.

The state when paused (the lead's view; each teammate's own note is the detail):
- **backend:** the deadline and hedge are done. Next: the WRONG FIGURE group (fixed, money review running), then buttons, the click, auto-repair and the compose_figure wiring.
- **backend2:** the 30 run-1 understand defects and compose_figure, with the money review running. Then print plans for all 38 reports, entry routes, and plan_reads for backend4's new inputs.
- **backend3:** forecast gaps F1 to F6, money review running, calc.rs mid-edit.
- **backend4:** R1 to R6 and P1 to P4 are done (handoff accepted). Next: a screen AnswerSource and the collector filter.
- **robustness:** bank 1,126 committed. Next: the 264 remaining capability cells, one wording each, then clippy on oracle.rs and robustness_live.rs.
- **frontend2:** the instant sentences are done. It waits for the screen source shape.
- **frontend3 and the hidden tester:** idle. Hidden bank 244, plus 50 in round 2.
- **lead:** D6 v1.5 (one change: backend4's and backend3's drafts, the collected prompt items, the fixed-part limit raised to 11,500), commit, rebuild, then live run 2 with --threads 4.
- **Pause, completed about 03:45.** All eight teammates sent their state notes, and none was killed.
- **For robustness on "continue":** fix bank rows Q1353, Q1359, Q1363, Q1371 and Q1374. A print with no period follows the app's rule (no period, or the whole books), not "this month".
- **The lead's first steps on "continue":**
  1. Ask Umar to approve the balance forecasts in PRD FR-7.86 and Code Standards M14 (backend3's finding).
  2. Then D6 v1.5: backend4's drafts, backend3's estimate drafts, the new EntryScreenName values from backend, the collected prompt items, and the limit raised to 11,500.

## 13. Paused by Umar again, 05-11-2026 about 18:45 (usage limit; agents kept alive)

Done since the 12 resume:
- PRD v1.26 and Code Standards v1.18: balance estimates (commit 1e335de).
- backend: the instant answers re-checked; the 10 entry routes; the v1.5 switch (TOOL_NAMES has 10 tools, ALL is 40, phone_charge shown, an audit help cue).
- backend2: compose_figure, the real-plus-forecast plan, print plans for 37 reports.
- backend4: the Screen source, the collector filter on R22, R23 and R35, and ERR_BACKUP_CLOUD_ADDRESS_PATH (plan 53 §10).
- frontend3: the restore fixes.
- robustness: bank 1,494, oracle 6,255 facts, matrix A 705, N 26, G 29.
- hidden tester: round 3 (50) written; all 344 hidden rows pass the checker.
- D6 v1.5 assembled by the lead's helper (uncommitted). The prompt files equal the doc, and the fixed part is 11,494 tokens.

To do on "continue":
1. **Ask Umar:**
   - raise the fixed-part limit 11,000 to 11,500 (NFR-AI-5, Code Standards Q9). backend was told to revert the 11,500 it set on its own.
   - the PRD FR-7.86 wording that also leaves out capital moves, make-regular moves and the dead mark (backend3's S3).
2. **backend3:** the balance pace uses the last 3 full months (lead's decision, PRD text). Then B1, S1, S4 and N1, the money review and the HANDOFF. robustness and the hidden tester re-dump est_balance afterwards.
3. **frontend2** (unanswered at the pause):
   - the `seller-own` role (it blocks backend2's two seller lines);
   - SourceFooter.tsx TS2366 (the screen case);
   - err-backup-cloud-address-path;
   - the earlier list.
4. **robustness:**
   - remove the 21 pending=Q* flags once v1.5 is committed;
   - the 4 seeded rules, and Rasheed as collector for two buyers (yes, with a dump diff);
   - pass the new dump with the staff names to the hidden tester.
5. **The 29 gaps left:**
   - W2: open or print plans for analytics views A to K (backend2 with frontend2);
   - W1: R1 by بکری number (backend2);
   - W3: a print that names an item: R7, R27 and R10 links (frontend2, then backend2);
   - P1: the guests' fruit print button (backend2);
   - Q5: packing, marka, store and narration find roles (backend4);
   - Q4: open the audit log by words (backend2 and backend4);
   - Q4b: the audit guide topic is in v1.5; robustness re-checks it.
6. **The lead:** commit v1.5 and all work, rebuild the app, live run 2 (`--threads 4`), then the fix rounds and the hidden rounds per §11.
7. **backend's crate run after the v1.5 switch:** 723 passed, 6 failed.
   - The loop_limits failure is fixed by backend.
   - robustness: the nine-tools attack test now needs 10 tools; oracle-keys.txt must be rewritten.
   - backend2:
     - 10 print rows: Q1192, Q1204, Q1262, Q1411, Q1428, Q1439, Q1443, Q1447, Q1453, Q1458;
     - the planned report reads lack layout, seller, status, voucher_type and zamindar.
   - lead (v1.5): system.md lost `voucher_type` in the trim, and state_prompt fails on it. Put it back.
8. **frontend2 batch 3 is done** (all its items, seller-own, the screen source, the collector box, the entry routes, and the W2 and W3 links). Vitest passed 8,578 tests, with the 4 failures fixed. On "continue":
   - tell backend2 that seller-own is in;
   - backend2 builds the plans for W2 (views A to K) and W3 (item prints) against frontend2's links;
   - backend adds a fixture with a screen source;
   - question for Umar: should the collector box list only staff whose job is collector?

## 14. Paused by Umar, 06-11-2026 about 03:40 (agents kept alive)

Done since §13:
- PRD v1.27 to v1.30, NFR v1.16, Code Standards v1.20 and Architecture v1.19, all approved by Umar:
  - the balance exclusions;
  - the 11,600 limit;
  - the Group filter;
  - plan 57 (a line over the lot gets no figures; the lot labels name the seller and item);
  - plan 58 (the sale line quality).
- D6 prompt v1.5 is committed at 11,595 tokens.
- K8: the dues series takes 0.45 s, down from 12 s.
- Plan 57 T1 and T2 are done.
- robustness: the item axis (2,070 cells), and a bank of 3,261 questions.

To do on "continue":
1. **backend:** P3. Add the 7 glossary terms to the not-needed list in tests/prompt.rs.
2. **backend4:** the item_group HANDOFF, which includes views B, C and H. Its full test run was going at the pause.
3. **backend3:** the plan 57 fixtures, and the plan 58 T1 HANDOFF. reports/src/reports/ids.rs:405 still fails clippy (items_after_test_module).
4. **frontend3:**
   - the party card cut-off at 1024x600;
   - plan 58 T2 with option C, including the report-grade-* Fluent words;
   - retake the plan 57 photos with the real fixtures.
5. **frontend2:**
   - the five step messages plus assistant-how-set-group;
   - the Group box.
6. **backend2:**
   - plan 58 T3;
   - the compare plans;
   - group prints and opens;
   - the last rate of a buyer's item from R13;
   - Q1463;
   - "up to now".
7. **robustness:**
   - apply the lead's N/G rulings to capability-items.tsv;
   - plan 57 T3;
   - the bench note.
8. **The lead:**
   - commit every plan's work;
   - rebuild the app;
   - live run 2 with `--threads 4`; for speed, run the ITEM facet with `--every 5`;
   - the fix round, hidden round 4, my own test, then the hand-over.
9. **First on "continue": backend4 runs `pnpm gen:types`.** The bindings were restored from HEAD at the pause, so they lack item_group, collector and SaleGrade. No frontend typecheck runs before that.
10. **backend4's three questions for Umar:**
    - view G's gardens column, for a garden with no main item;
    - the item group is read live, so changing an item's group moves its history;
    - R27 shows part of a bill under a group.
11. **Other open items:**
    - robustness's plan 57 T3: 6 of 7 tests pass, and the 7th has a fault in the test's own setup;
    - backend3: the plan 57 fixtures, the full run, and one HANDOFF for plans 57 and 58.
12. **backend2's HANDOFF arrived at the pause:** the group reads, plan 58 T3 grade words (R2 lines, no instant answer, no tools.json change), and the print bank at 245 of 245. On "continue":
    - plan the group rankings, now that backend4 allows B, C and H;
    - the last rate of a buyer's item from R13;
    - the compare plans;
    - "up to now".
