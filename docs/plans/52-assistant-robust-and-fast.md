# Plan 52: ask the app, robust, fast and proven before hand-over

Version 1.0, 02-11-2026. Lead: tech-lead. Asked by Umar on 02-11-2026, after his fourth failed test. In his words: "take your time to make it well rounded, make it versatile and the robust ... then test it each category and then you have 100% satisfaction ... then make it hand over to me." Branch `feature/assistant`, folder `Fruit Mandi Parallel/d5`.

## 1. Where we are, measured

- **The live run of 110 questions scored 81.** The misses fall into five groups:
  - 6 network failures, with no retry;
  - 4 help questions taken as a change request;
  - 6 misunderstood: the item "kela" and the partner "alif" were not found, a count, a comparison, a percent, and an advice question;
  - 13 answers that are right or incomplete (to read one by one).
- **Umar's question 13, "sales bherane ke tareeky batao?", failed:**
  - call 1 asked for compare with no dates, so 3 tools were refused;
  - call 2 asked for a period;
  - call 3 got 2 views and 1 refusal;
  - call 4 asked for more;
  - the call limit was reached after 25.3 s and the answer was "not understood".
- **Time.** One model call takes 3.9 s at p50 and 8.9 s at p95, and sometimes up to 21 s. A question needs 2.6 calls on average. The app's own work is about 40 ms.
- **His real data is almost empty:** 0 sales, 3 journal entries. Advice "from the books only" has nothing to stand on.

## 2. What "done" means (acceptance)

Measured live, with the real model and the app's own question path, on a rich test year and on a copy of Umar's own data.

| Category | Examples | Correct | Time p50 / p95 |
|---|---|---|---|
| A. Real-time data | commission and sale by item, seller, buyer and partner, any period; "kela ke is saal kitne commission or sale, saib ke kitne"; "Faisal ne kitne ka saman khareeda, kya kya, konsi item"; dues, cash, stock, gardens, trips, fruit, expenses, partners | 100% of figures right, at least 98% answered | 6 s / 12 s |
| B. Suggestions and recommendations | "sale kaise barhayen", "kis se pehle wasooli", "kisko udhar na dun", "commission kaise barhe" | at least 95% good (FR-7.88 and the Urdu review) | 6 s / 12 s |
| C. Forecasts and what-if | "is raftar se saal ke aakhir tak commission kitna", "agar raftar double ho to", "target 5 lakh kab tak" | 100% of figures right | 15 s / 25 s |
| D. Help with the app | "bikri kaise add karun", "market safar kya hai", every screen | at least 98% the right topic | 5 s / 10 s |
| E. Mixed and hard | follow-ups, misspellings, Roman Urdu, Urdu, English, two-part questions, nonsense, injection, "change it" requests | safe ends only, never a wrong figure | as A |

Also:
- no "not understood" after the model has read data;
- no network failure without one retry;
- every group of same-meaning questions reaches the same figures.

## 3. The work

### 3.1 Robust core (backend; money agent where marked)

1. **A guaranteed answer.** The last allowed model call goes out with tools switched off (`tool_choice: "none"`), so the model must answer from what it has read. A question never ends as "not understood" after reading.
2. **Forgiving tools, still honest.** A tool fixes what is plainly meant instead of refusing, and says what it did in a `note`:
   - compare with no dates means this month against last month (or this year against last year when the period is a year);
   - a sort word a view does not have is dropped;
   - an `up_to_now` start means the first date of the books.
   - Wrong figures are impossible: the tool still reads only real records. A refusal stays only for input that has no sensible meaning.
3. **Find everything that is named.**
   - `find` also searches items (kela, saib, mausami, in Roman Urdu, Urdu and English, through the transliteration words and the item names), partners ("alif", "A", the partner's name), gardens and trips.
   - A buyer's purchases come by item, with quantity and amount: "Faisal ne kya kya khareeda" reads R13 and the sales register for that buyer, grouped by item.
4. **Help is never a change request.** A question with "kaise", "tareeka", "how do I" or "kya hai" and no amount to write is help. An app-side guard turns a change request with no amount and no instruction verb ("likh do", "save karo", "add kar do", "delete") into app_help.
5. **Forecast what-ifs.** The estimator answers "by the end of this year" (the months left in the year) and "if the pace doubles" (a factor applied to the same-pace month, through domain::forecast). The money agent works the numbers first and reviews after. The worked example goes in the plan before the code.
6. **Network.** One quick retry on a connect error, 429 or 5xx, inside the 60 s. A real loss of internet says so in plain words.

### 3.2 Speed (backend)

7. **Fewer round trips.**
   - Help and general advice answer in ONE call: no data tool is needed for help.
   - The forgiving tools remove the error rounds.
   - The playbook tells the model which single tool answers each common question.
8. **Chain find and read in one call** (Umar to decide). The model may call `find` and the tool that reads that party in the same batch; the app checks that the find had exactly one match before it reads. About one round trip less on every named-party question.
9. **The fastest Azure endpoint** (Umar to decide). `provider.sort: "latency"`; still Azure only, no fallback, zero retention.
10. **Measure, then tune.** Use the timing columns. Lower reasoning only for the categories where the live tests show no loss of quality.

### 3.3 General advice (Umar to decide)

11. **When the books have too little.**
    - When the books have too little to answer a suggestion question, or the question is general ("sale kaise barhayen"), the assistant gives short, practical mandi advice: keep rates fair and consistent, collect old dues weekly, follow up buyers past 30 days, keep sellers paid on time, record every sale the same day.
    - It is marked "عام مشورہ" (general advice), carries no figure of its own, and answers in one call.
    - When the books do have data, it advises from the books first (FR-7.88).
    - This changes PRD FR-7.88 ("from the books only").

### 3.4 Testing, before any hand-over

12. **A rich test year.** One seeded "full year" book:
    - 12 months, 6 items (kela, saib, mausami, aam, angoor, aloo), 10 sellers, 15 buyers including "Faisal" and two Akrams, 3 partners, gardens, a trip, fruit taken, expenses, dues of every age;
    - its true answers worked out in a fixture by the app's own reports, so the expected figures are proved, never typed.
13. **The test bank, about 300 questions.**
    - Categories A to E, each in Roman Urdu, Urdu and English, with misspellings and different wordings, plus groups of the same meaning.
    - Category A covers every section: item, seller, buyer, partner, period, cash, dues, stock, gardens, trips, fruit, expenses, capital, day and month.
    - Run on the rich year, on the worked day, and on a copy of Umar's own data (read only).
14. **The loop.**
    - Run the whole bank live.
    - Read every miss (the answer, the calls, the time).
    - Fix the cause, never the scorer, unless the answer was truly right; every scorer change is shown to Umar.
    - Run again, until Section 2 is met twice in a row.
15. **Hand-over.** Only then:
    - a one-page report to Umar: the score per category, the time per category, 20 sample answers, and what the assistant will not do;
    - a short list of questions for him to try himself.

## 4. Owners and order

| Step | Owner | After |
|---|---|---|
| Umar's decisions: 8, 9 and 11 | Umar | |
| 1, 2, 4 and 6 (core) | backend | |
| 3 (find items, partners, purchases) | backend | |
| 5 (what-if forecasts) | backend, with the money agent before and after | |
| 7 and 10 (speed) | backend | 1, 2 |
| 12 and 13 (test year, bank, scorer) | robustness | |
| The panel: the general advice band, the notes, any new text | frontend | 3.1 |
| 14 and 15 (the live loop, the report) | lead | all |

## 5. Rules

- No figure is ever typed by the model; every figure is checked (FR-7.76).
- Golden numbers never change. Requirement docs change only after Umar approves, with version and change log.
- The key is never read, printed or written.
- Targeted checks only; the live runs are by the lead.

## 6. Umar's decisions (02-11-2026)

- Item 11, general advice: APPROVED (PRD v1.19 FR-7.88).
- Item 8, find and read in one call: APPROVED (PRD v1.19 FR-7.87).
- Item 9, the fastest Azure endpoint: NOT approved. The provider block stays as it is.
- Added by Umar: "increase the tool call or timeout so we won't see the error". The new limits are 6 model calls plus one rewrite, 12 tool calls, 90,000 input and 4,000 output tokens, 30 s a call and 90 s a question (NFR v1.12, Code Standards v1.16, PRD v1.19 E96 and A87). Item 1 still holds: the last call always answers.
- The prompt changes these need (system.md: the limits line, chaining, general advice; the answer schema: the general advice kind or flag) are drafted by backend and go into D6-system-prompt v1.3 through the lead, as before.
- Umar, 02-11-2026: "9,500 limit we can go up to align or achieve the goal". The lead sets the fixed-part limit to 11,000 tokens (o200k_base, as P1 measures). The live run also reports the fixed part as the model counts it, about 3.4% more. The documents change with the v1.3 prompt: NFR-AI-5, Code Standards Q9 and D6-system-prompt.
