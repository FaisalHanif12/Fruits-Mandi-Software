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
