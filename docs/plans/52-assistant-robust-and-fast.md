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

## 7. Lead decisions during the loop (03-11-2026)

- **Live run 2** (369 questions): 115 questions were lost to three internet drops and 3 timed out. Of the 254 scored, 232 passed: A 100/113, B 38/44, C 37/37, D 20/21, E 37/39. The real defects went to backend.
- **The harness survives a dropped connection.** A question that ends offline is asked again after the connection is back (waiting up to 10 minutes, at most 5 tries). A question still lost is reported apart and is never scored. While any question is lost, the run cannot say "meets". A timeout is still a real miss.
- **The fixed part is 10,997 tokens as the model counts it (about 10,652 by o200k), within 11,000.** The higher first-call numbers came from what the app reads ahead for that question (+86 to +1,204). That is reported apart and is not part of the NFR limit.
- **Scorer change S01 (shown to Umar):** "whom should I collect from first" asks for one party. The second party's figure becomes optional (`second?=`). The order check still applies when both parties are shown. Reason: Q223 and Q224 named the right first party and figure and showed the ordered list, so the answers were right. Refused: a no-side exception for list amounts. A Dr or Cr figure must still show its side.

## 8. Umar's request of 03-11-2026: mixed language, a party's day, and "no such party"

Umar: "ask these kind of the question as well which would be the mix english or urdu ... Faisal ne is din kiya or kitne beecha ... if not found then its just write that jo seller ap poch rahay hain aisa koi seller hai hi nahi ... keep the complete diversity in the existing facet or real time data check in any form."

### 8.1 Bank (robustness)
- **New facet M, mixed language:** Roman Urdu with English words ("Faisal ne yesterday kitna sell kiya", "kele ka commission this month kitna"), Urdu letters with English words ("رحیم کی sale آج کتنی"), and English with Urdu mandi words ("how much bakaya does Akram have").
- **Coverage:** every section of category A in mixed form at least once (item, seller, buyer, partner, period, cash, dues, stock, gardens, trips, fruit, expenses, capital, day, month).
- **A party's day:** "X ne is din / aaj / kal / 12 October ko kya aur kitna becha (or khareeda)". The answer gives each item with its qty and amount, plus the day's total.
- **No such party:** a made-up name gives not_found, saying plainly that no seller (or buyer) has that name, with the nearest names if any. A real name asked in the wrong role (a buyer asked as a seller) says that this name is a buyer, not a seller, and offers his purchases. A misspelled real name still finds him.
- **Expected figures come from the app's own reports (the oracle), never typed.** Each new question joins a same-meaning group with its Roman Urdu, Urdu and English twins.

### 8.2 Contract change (backend, then frontend)
- The not_found outcome carries the role that was asked: `asked_role`, one of seller, buyer, item, partner, garden, trip or any. When the name exists in another role, it also carries `found_as`, with that party.
- **Text, ur and en, frontend:** `assistant-not-found-role` selects on the asked role. Example: "آپ جس بیوپاری کا پوچھ رہے ہیں، اس نام کا کوئی بیوپاری نہیں۔" / "There is no seller by the name you asked about." `assistant-found-as-other` example: "یہ نام بیوپاری نہیں، گاہک ہے۔ ان کی خریداری دکھاؤں؟" / "This name is a buyer, not a seller. Show his purchases?"
- **Lead decision (easy to change):** the wrong-role case names the role it was found in instead of only saying "not found", because that is the true answer.
- The new Urdu text joins the pending Urdu review.

### 8.3 Cue attack tests (robustness_cues.rs, 03-11-2026)
- 24 offline tests: 18 pass, 6 fail. The failures are 4 findings, all owned by backend.
- **Findings 2 and 3, MAJOR, a wrong figure on screen.** The two-item guard covers only sum. It misses an average of two items (2,724), and an item's figure added to an unfiltered total (9,376). Both were shown for "kela ka commission". Must be fixed before the next live run.
- **Finding 1, MAJOR.** The new `asked` rule takes the advice word "sale" as a figure request, so a right answer can end ChecksFailed.
- **Finding 4, MINOR.** Prefind reads the seller Kela Traders for the item word "kela". Prefind is parked and off. Fixing this is a condition before it is ever turned on.

### 8.4 Backend run 2 fixes accepted (03-11-2026)
- **All 9 run 2 defects are fixed, each with a test.** Results: wrong_figures 42/42, assistant 459 passed with 3 ignored, golden_day 94 passed. The money agent reviewed the design and then the code, and found 3 bugs, all fixed.
- **Lead decisions:**
  - A partner per row in R37 while the section is locked must be hidden. Fix now.
  - The sum guard also covers two rows of one unfiltered result. کل / kal and جمع are not "together" words on their own.
  - The new fields and the `asked` rule wait for D6 v1.5, after the next live run.
