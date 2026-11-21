# Plan 82: Party commission questions answered, not timed out

| | |
|---|---|
| Status | Umar's request, 20-11-2026 ("it took a lot of time and then failed, resolve that") |
| Branch | `fix/gardens-new-form` (same branch; one pull request) |
| PRD | FR-7.90 (instant answers), FR-7.77, FR-7.81, NFR-AI-3; Code Standards Q8, Q9 |

## What happened (question log, ids 42 to 44)

| Question | Calls | Result |
|---|---|---|
| mani ne abhi tak shop ko kitni commission banwa kr di ? | call 1 ok 9.8 s (find mani as seller, period up_to_now); call 2 timeout 12 s; hedge copy timeout | outcome error, 21.9 s |
| same again | 4 calls, every one timeout 12 s | error, 25.0 s |
| mani gahaq se hum ne kitni commission hasil ki abhi tak ? | call 1 ok 6.8 s (find mani as buyer, period up_to_now); calls 2 to 4 timeout | error, 25.0 s |

The tools took 0 to 4 ms. All the time was the model. Two faults:

1. FR-7.90 says one figure for a named party over a period ("a period's sale or commission, for ... a seller or a buyer") is answered by the app with no model call, in under a second. These wordings were not understood by the instant path, so they went to the model.
2. The per-try wait of 12 s and the question limit of 25 s (plan 55, NFR-AI-3 v1.21) are too tight on today's line: a call that carries tool results did not answer in 12 s.

## Decisions (lead, 20-11-2026)

- The instant path understands a party's commission or sale for a period in Urdu letters, Roman Urdu and English, including wordings like "X ne abhi tak shop ko kitni commission banwa kr di", "X gahak se hum ne kitni commission hasil ki", "X se kitna commission mila", "X ki bikri", "abhi tak", "is mahine", "pichle saal". The party is found by `find` rules (both roles; two to five matches ask "کون سا؟"; none gives "not found"). The answer is the checked figure of the commission page with that party filter (FR-9.3), and its button opens that page filtered (FR-7.90 v1.42). No model call, no count in the daily 20.
- Model path limits: a try waits at most 20 s (was 12), the question at most 45 s (was 25), the hedge copy after 10 s (was 7). NFR-AI-3, Code Standards Q8 and Q9 and Tech Stack 4.8 follow (lead).

## Acceptance

- The three questions above, and 20 more wordings of a party's commission or sale over a period (both roles, Urdu letters, Roman Urdu, English), answer from the instant path with the checked figure, under 1 s, with no model call (a test with a model client that fails if it is called).
- A recorded slow model (each call 15 s) still finishes a two-call question inside the new limits (test with the fake clock).
- Golden questions, prompt tests and every assistant test stay green. No network in tests.

## Widened scope (Umar, 20-11-2026): complex questions, accurate and fast

Umar: "two core goals: accuracy, the exact data, with speed. Simple questions are instant; now make complex queries robust: buyers, sellers, market trips, gardens, profit, loss, commission, sales, expense, with dates. Example: in the previous months, how much commission or sale did these buyers and sellers give the shop, and how much commission did we get; and from last year to this month, how much has the shop spent on expenses."

### What a complex question is

One or more of: several parties or sources at once; a date range or two periods to compare (pichle mahine, pichle 3 mahine, pichle saal se is mahine tak, month by month); a sum, difference or percent across results; a list ranked by a figure; a trip's or garden's result, profit or loss and each partner's part; expenses by head over a range.

### Lanes

| Lane | Owner | Work |
|---|---|---|
| A. Read first | `backend-ai` | The app understands the parts of a complex question it can be sure of (parties, roles, periods, sources, the figure asked) and runs those reads itself before the first model call (tests/read_first.rs exists), so the model gets the results at once and answers in one or two calls instead of four or five. Parts the app is not sure of are left to the model. Where every part is sure and the answer is figures only, the app answers itself (FR-7.90) with the reviewed words. |
| B. Model path | `backend-ai2` | Pooled connection, warm-up, limits, low reasoning, fewer tokens in later calls; the calls for a complex question must fit the limits with room to spare; live runs. |
| C. Independent check | `assistant-eval` | A list of at least 40 complex questions in Roman Urdu, Urdu letters and English, built from the real dev data copy, each with the expected figures worked out from the screens' own functions or SQL (never from the assistant); a live run after A and B, with a budget cap; every wrong figure, wrong party, wrong period, failure or slow answer reported with the question log row. |

### Acceptance (added)

- Every question of the lane C list ends with the right figures or an honest ask ("کون سا؟", dates), never a wrong figure; at least 38 of 40 right on the first run after fixes.
- Complex questions: p95 under 15 s, none over 30 s, no time_limit or error outcome; simple ones under 1 s.
- Every new reading of a question has a test with recorded transcripts and no network; golden questions and prompt tests stay green.

## Resume note (paused 21-11-2026)

Written by `backend-ai` when Umar paused the assistant work. Everything below is in the tree; it compiles, and `cargo clippy -p mandi-assistant --all-targets -- -D warnings` was clean at the last run. No cargo run of mine is left going.

### Done (instant path, understanding, read-first, guard)

- **Wrong-column guard** (model path): `src/sale_guard.rs`. Net sale must cite `net`, the buyers' bills `buyer_total`. The same row's right cell is put in by the app, else the answer is refused with the cells that hold it. Tests `tests/plan82_measure.rs` (6). Hand-worked D8 constants: gross 260,500, net 244,870, bills 278,610.
- **Understanding** (`src/understand/`): `party_flow.rs` (a party's commission or sale, which one, filler words), `measure_words.rs` (net, bills, laga apart / not counting laga), range words (`when.rs`: pichle saal se is mahine tak, since, ke beech), ordinals pehla/akhri safar (`who.rs`, first trip = number 0 = code 0070), month/head/day-wise goes to the model with R29 read ahead, Y23 ("مدوں" never a name), D29 (role from later words), D21/D19/Y09/Y35 (a buyer's purchase AND commission together, answered by the app), D03 (each buyer or seller: view C or B read ahead), D04 (who gave the most commission reads view C sorted by `buyer_comm`; a bare "top 5 buyers" keeps view H), D11 ("paisa laga" beside a garden is the verb), several named parties read ahead.
- **Tests:** `tests/plan82_party_commission.rs` (7), `tests/plan82_measure.rs` (6), `tests/plan82_more.rs` (6), all with a model client that fails if called, each under 1 s. Changed assertions: `tests/d8_hard.rs` h3 and the helper of `tests/d7_realtime.rs` (a question the app now answers has no model request).
- **Offline sweep of the 98** (`scripts/ai-eval/complex/run_live.py --offline`, dev + year copies): 22 answered or asked by the app, all right, 0 wrong, 0 local not_found, none over 1 s. Model-bound with no read ahead: 44 of 82 at the start, 20 of 76 now.

### Half-done

- **Instant sentences for four readings are written in code but have NO Fluent message yet**, so the app finds no sentence and hands those questions to the model (safe, slower): net sale of the shop, the buyers' bills, the shop's profit (view G), the commission with its two sides. See "Fluent ids owed" below.
- **Decision of the lead, not yet built:** an unexplained word in the question must stop EVERY instant answer, party lines included (now it stops only some). Part 1 is accepted: no period stays with the model.

### Exact next steps (in the lead's order)

1. (a) Measure guard and the six findings M04, M08, M24, M11, M03, D04: done on the model path with tests; re-check with the official 98 once the Fluent ids are in.
2. (b) The seven no-answer findings: D14 ranges and M06 "so far" (ranges done; "so far" net sale needs the id `assistant-instant-sale-net-shop`), D16/D25 shop profit and commission share must not be locked (profit has the view G read and sentence; D25 share percent NOT built), D13 first/last trip or garden (first trip done; "akhri" and gardens: check on the dev copy), M22 role from the party's own type (answered locally on the offline run), D28 chukta found by name (`chukta_group` flag does not fire for "chukta khate walon ne": check the cue), Y23 done, D29 done.
3. (c) Read-first: pre-read EVERY period and party before call 1. Still no read ahead (20 of 76): follow-ups (D26b/c, Y21b/c, Y31b/c), forecasts (Y19), comparisons (D24, Y05, Y18), expense comparisons by head (Y02, Y04), gardens and trips results (D11, D12, Y13, Y14, Y27), seller two-measure questions (M16, M23), M25. Zero-model answers for ranked "which seller/buyer sold or gave the most" (top-N table) and for the net sale: not built.
4. Close the leak (decision 2 of the lead): make `Doubt::UnknownWords` / `unexplained` stop `instant_answer` for party lines too; add a test with a wording that has one unknown word.
5. Update `tests/playbook.rs` line 505 (`up_to_now_works_for_every_view_and_report_that_takes_a_period...`): view F now takes `compare` by backend-ai2's approved tool repair (left out with a note). The test must assert the repair and the note, not ERR_OUT_OF_RANGE. Not done yet.
6. Then run the full assistant tests (`cargo test -p mandi-assistant --no-fail-fast`, CARGO_BUILD_JOBS=2, one cargo command at a time), `cargo test -p mandi-services --test dashboard_tiles`, clippy, rustfmt on my files; then the HANDOFF to the lead, who asks `assistant-eval` to run the official 98.

### Fluent ids still owed to `frontend-capital` (ur and en, `# review: pending`)

All four use `$period` (the message `assistant-instant-period`, already formatted) and a reference `$figure` (or `$seller`, `$buyer`). Sent to `frontend-capital` on 21-11-2026:

- `assistant-instant-sale-net-shop` ($period, $figure): the shop's net sale, R6 total `net`.
- `assistant-instant-bills` ($role = buyer or shop, $name only for buyer, $period, $figure): the buyers' bills, R2 total `buyer_total`.
- `assistant-instant-profit` ($period, $figure): the shop's profit, view G total `profit`.
- `assistant-instant-commission-sides` ($period, $seller, $buyer): the commission with no Laga, its seller side and buyer side, view A totals `seller_comm` and `buyer_comm`.
`assistant-figure-balance` in `src/results.rs:242` is only an example in a doc comment, not an id the code asks for.

### Tests that were red at the pause

- `tests/playbook.rs` `up_to_now_works_for_every_view_and_report_that_takes_a_period_and_a_comparison_over_it_is_left_out`: red, see step 5 (not caused by plan 82 code).
- `tests/understand.rs` `the_year_gardens_and_the_speed_of_a_full_set_of_books`: red once under machine load (56 ms against the 50 ms limit); green alone and with `--test-threads=1`.
- Everything else of the last full run was green (79 binaries, 1132 passed).
- Not mine, still unformatted: `tests/arm_up.rs`, `tests/robustness_complex_live.rs`.

### Update after the resume (21-11-2026, `backend-ai`)

Done since the note above: (1) `Understanding::instant_stop`: an unexplained word, a known word that changes the figure ("koi aur", "kam", "wapas", "nahi", "se pehle") or a net/bills word beside another figure stops every instant line (the reads are still made); (2) `tests/playbook.rs` now asserts the view F `compare` repair and its note; (3) the net sale, the buyers' bills, the shop's profit and the commission without Laga are answered by the app (the four Fluent ids are in); D28 chukta by name (views A and C read ahead); D04 reads view C sorted by `buyer_comm`. Offline sweep of the 98: 31 answered or asked by the app, all right, 0 wrong, 0 local not_found, none over 1 s; model-bound with no read ahead 20 of 67. Full `cargo test -p mandi-assistant --no-fail-fast`: 79 binaries, 1138 passed, 0 failed (before the last one-phrase lexicon edit; the 7 files that read it were re-run green).
Still open: view C has no "commission + Laga" column, so D04 ranks by the buyer commission and the model adds the Laga (a column in `reports` would give the exact rank); ranked top-N tables and D25 not built; the 20 questions with no read ahead are follow-ups, forecasts, comparisons of two periods, expenses by head, gardens and trips results and the two-measure seller questions.
