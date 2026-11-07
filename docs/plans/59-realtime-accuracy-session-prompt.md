# Next session prompt: D7, real-time data accuracy and speed

Paste everything below the line into a new chat started in the main folder with `claude --agent tech-lead`.

---

I am Umar. Start a new deliverable, **D7: the assistant answers real-time data questions with exact figures, fast.**

## 0. Before anything
1. Work only in the main folder "Fruit Mandi Project". Run `git branch --show-current`. It must say `feature/assistant` (latest commit 6735550 or later). If it does not, stop and ask me; do not switch it yourself.
2. Read, in this order and only these: `docs/plans/55-assistant-production-handoff.md` Section 15; `docs/deliverables/status.md` (top D6 item); `docs/deliverables/D6-ai-assistant.md` Sections 1 to 3 and 7; the PRD sections for the data the assistant reads (PRD 7.7, 12, 13, 15, 16, 17, 18, 19, 22).
3. Create the branch `feature/assistant-realtime` from `feature/assistant`, and the plan file `docs/plans/59-assistant-realtime-accuracy.md` with the sections the tech-lead agent file asks for.

## 1. The goal (in priority order)
1. **Accuracy of real-time data: zero tolerance, for EVERYTHING the books hold.** Any figure, list or count the assistant shows must equal the books exactly. This covers every record and every figure the app keeps or works out (every screen, report R1 to R38, analytics view, tile, ledger, register, voucher, posting, balance, matching and stock record), in any wording and with any mix of filters, including ones not named here. The filters below are only examples, never the limit: date, day, range or "up to now"; party of any kind (buyer, seller, partner, collector, staff, pukhta person, khata, garden, trip, transporter, any account); item, group (phal, sabzi), quality (اول، دوم، سوم), packing, marka, adda, city, store, truck, بکری number, voucher number; cash or credit (نقدی، ادھاری); side (seller, buyer); paid, pending, open, settled, returned; counts, totals, averages, last and first, top and bottom, per item, per party, per day. When a question mixes several of them, every one must hold. A wrong figure is CRITICAL and is always fixed, whatever the usage. If the app cannot read a question for sure, it asks (which one, which dates) or says it cannot; it never shows a guessed figure.
2. **Speed.** Targets for real-time questions (plan 55, A2): right with no model call at least 60%; right with no call or one call at least 85%; wait p50 2 s, p95 8 s; nothing over 25 s.
3. **Secondary, NOT tested in D7:** help and how-to, advice, recommendations, forecasts, what-ifs, estimates. Do not change them unless a real-time fix needs it, and do not spend test money on them.

Example of the questions D7 must answer exactly: "پچھلے مہینے 14 تاریخ کو اکرم نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار رہا" (one buyer, one past day: each line with item, qty, rate, amount; cash paid on the spot; credit left; balance).

## 2. Step 1: study the existing code (no test money)
With 2 teammates at most (one backend, `backend-dev`; one read-only `code-reviewer`), go through the whole assistant: `backend/crates/assistant/src/` (understand/, plan, reads, tools/, check.rs, scope_guard.rs, run.rs, fill.rs, echo.rs), the reports and analytics it calls, and the prompt files. Write in the plan file, with file and line:
- every path a real-time question takes (instant answer, pre-read plus one model call, model loop);
- every place a filter can be dropped, changed or guessed (period, party, item, group, grade, cash or credit, side);
- every real-time question type that still needs the model, and why it could be answered with no call;
- the open items of plan 55 Section 15, first the two CRITICAL wrong figures: Q2062 buyer-side commission (15,716 shown, 7,586 right) and Q2436 last rate (3,830 shown, 3,940 right, empty date).

## 3. Step 2: make it more accurate and faster
Fix in this order: wrong figures, then dropped filters, then "not found" or time limit on a valid question, then speed (more instant answers and pre-reads, fewer model calls). Each fix gets a test on the rich year (`tests/robustness_year`) with the exact figure from the oracle. The money agent reviews any change that touches money reads or sums.

## 4. Step 3: the picky test bank (the only live testing)
Build ONE new bank `scripts/ai-eval/realtime-d7.tsv` (robustness owns it), about **200 questions**, every one with its expected figures from the oracle, chosen to cover every area of the app's data, each with simple and complex (several filters) questions, in Roman Urdu, Urdu letters and English, some misspelt. The list below is the minimum, not the limit: first list every kind of record and figure the app keeps (from the PRD, the reports catalogue and the database tables), then make sure each one has at least one question, including the ones not named here:
- buyers: bought on a day or range, by item, cash and credit, paid, pending, balance, oldest bill, day sheet (R35);
- sellers: sold, commission, deductions, net, payable, advances, paid;
- partners: profit share, drawings taken and returned, capital in and out, کل رقم;
- expenses by head and period; money lent and returned; الگ کھاتہ;
- payables, buyers' dues (اگراہی, R20 total), dues by collector and city, dead amount (R38);
- cash book, day book (روزنامچہ), trial balance, day summary (R34);
- stock (باقی مال) by item and lot; items, groups and quality; rates (average and last);
- market trips and their purchases; gardens; fruit taken; پختہ کھاتہ persons;
- zero days (nothing happened must be 0, not "not found").
Before any live call, check every expected figure against the oracle and run the whole bank OFFLINE first (the app's understanding, plans and instant answers with no model; `cargo test`), and fix what fails there. That costs nothing.

## 5. Budget for live runs: $3 in total, never more
- Before a live run: write the number of questions, the expected model calls and the cost (about $0.0004 a call). Ask me, and wait for my yes.
- Check the OpenRouter key's own total limit first. It does not reset; a 403 "Key limit exceeded" means the limit, not a bug.
- Probe one question first. Stop the run at the first 403 or 402 and tell me `failure.reason`.
- At most two live runs of the D7 bank: one after Step 2, one after its fixes, and the second only on the questions that failed plus a few that passed.

## 6. Done
D7 is done when, on the last live run: no wrong figure at all; at least 98% of the real-time questions right; the speed targets of Section 1 met; every fix has its test; and fmt, clippy -D warnings, `cargo test -p mandi-assistant`, `golden_day`, `assist`, `pnpm typecheck`, lint and `i18n:check` are clean. Then post "D7 REPORT" with each item, its command and real output, and update `status.md`.

## 7. Restrictions (all still apply)
- Never read, write or print the OpenRouter key; the runner uses the app's stored key only.
- Read my real data only through copies.
- Never delete `~/Library/Application Support/pk.mandisoftware.app.old-20261026-220000`.
- Never change a golden number. If code and a golden test disagree, the code is wrong.
- Requirement docs stay in `.claude/rules/`; they change only with my approval, with a version and a change log line.
- Prompt files: only the lead edits them, and the fixed part stays within 11,600 tokens.
- Teammates never use git. The lead commits with explicit paths, Conventional Commits ending "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>", and pushes after each finished task.
- Never commit `.claude/CLAUDE 6.39.30 PM.md` or `.claude/.w`.
- Mac phase: no pull request, merge or release.
- No reviewer or robustness gate until I say "continue".
- A critical or high problem is never skipped.
- At most 2 cargo jobs at once, with `-j 4`.

## 8. Saving Claude usage
- At most 2 or 3 teammates at a time; close them when their task ends.
- Each teammate sends ONE HANDOFF when done: no progress or "still waiting" messages.
- No broadcast messages; message only the teammate who needs it.
- When the conversation gets long, write a hand-over into the plan file and ask me to start a fresh session.
