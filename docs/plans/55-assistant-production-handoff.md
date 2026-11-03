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
