# Plan 54: ask the app, fast enough to be worth asking

Version 1.0, 04-11-2026. Lead: tech-lead. Asked by Umar on 04-11-2026, after he waited 55.6 s for "main ne faisal ki aj ki bikri ka print nikalna hai?". In his words: "with that response time there is no purpose to use. We have to make it optimized in terms of response time without any quality compromise." Branch `feature/assistant`, folder `Fruit Mandi Parallel/d5`.

## 1. What the log shows (question 17 in Umar's own question log, read from a copy)

| | Call 1 | Call 2 | Whole question |
|---|---|---|---|
| Model time | 38,379 ms | 17,205 ms | 55,626 ms |
| Prompt tokens (cached) | 11,369 (0) | 11,762 (11,366) | |
| Reasoning tokens | 78 | 451 | |
| Tools | app_help (printing), 2 ms | none | the app's own work: 2 ms |

- Two tries a minute earlier ended "offline": the internet had just dropped. Call 1 began on a weak connection and with a cold cache.
- Call 1 ran 38 s although the limit is 30 s a call. That is a defect: the per-call limit is not cutting a slow call.
- Even on a good line, Umar's earlier questions took 7 to 9 s a call, with 2 calls a question. Our test runs take 3.5 to 5 s a call.
- **The answer was also wrong.** It pointed to the entry screen and the dashboard and said "the amount stays empty, you type it", which is the note for a change request. It should have opened the party's day sheet (R35), or the buyer bills (R5), for Faisal and today.

## 2. Targets (on a normal connection, measured live on the bank)

| Kind | p50 / p95 | How |
|---|---|---|
| Help and how-to that the guide answers | under 1 s | no model call (S2) |
| A. Real-time data | 6 s / 12 s (plan 52) | one model call for most questions (S3, S4) |
| B, C, E | as plan 52 | S1, S4 |

No quality loss: every category scores at least what it scored in the run before the change, and still meets plan 52 §2.

## 3. The work

- **S1, the fastest Azure server.** Add `"sort": "latency"` to the provider block. It stays Azure only, with no fallback, zero retention and no data collection. Umar approved this on 04-11-2026 (D6 v2.3); it replaces his "no" of 02-11.
- **S2, help with no model call, and prints that open the right report.**
  - A local matcher over guide.json answers a how-to question when it is sure: Roman Urdu, Urdu and English, with misspellings. When it is not sure, the model answers as now.
  - A print request opens the right report with the party and the date already filled. Examples: "X ki aaj ki bikri ka print" opens R35, the party's day slip. "bill nikalna" opens R5.
  - A print request is never a change request, and never shows "amount stays empty".
- **S3, one call for most data questions.**
  - The app's cues already know the party, item, period and kind of question for many questions. Run that read before call 1 and put the result in call 1, the way the read-ahead already works for some questions.
  - Cover: a party's balance, khata or day; an item's commission or sale in a period; dues; cash; stock; top lists; a partner's total.
  - The model then answers in its first call. Target: at least 70% of category A in one call.
  - Every figure is still checked (FR-7.76).
- **S4, less model thinking where it adds nothing.**
  - Use reasoning effort "low" for a call whose data is already read (S3) and for help.
  - Keep "medium" for forecasts, what-ifs and advice.
  - It is kept only if the bank shows no quality drop.
- **S5, the per-call limit really cuts.**
  - A call with no first byte after 12 s, or that passes 30 s, is cancelled and tried again once. The question still ends within 90 s.
  - Each call's time is logged as now.
- **S6, the panel shows progress.** A line under the question says what is happening: "پڑھ رہا ہوں: کھاتہ فیصل", then "جواب لکھ رہا ہوں". The waiting is visible; no figure is shown before it is checked.

## 4. Owners and order

| Step | Owner | After |
|---|---|---|
| S1, S5 | backend | the current safety fixes and plan 53 |
| S2 matcher and print routing (Rust), S3, S4 | backend | S1 |
| S2 open-report button with filters, S6 progress line | frontend2 | backend's contract message |
| The timing in the scorer: one-call share, local-help share, time per stage. Attack tests for S2 and S3. | robustness | now |
| The live runs, before and after | lead | all |

## 5. Rules

- No figure typed by the model (FR-7.76).
- The provider block changes only as approved.
- Golden numbers never change.
- The key is never read, printed or written.
- Targeted tests only; the live runs are by the lead.
