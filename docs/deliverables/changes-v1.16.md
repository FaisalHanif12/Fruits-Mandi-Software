# Changes in v1.16: ask the app (for the lead)

Version 2.1, 02-11-2026. Umar's third change of 01-11-2026, with his answers and his changes of 02-11-2026, approved by Umar. Read this before `D6-ai-assistant.md` and `D6-system-prompt.md`.

## 1. What Umar asked for

The Owner asks questions about his own books in plain words, in Urdu letters, Roman Urdu or English, in any wording, and gets the answer at once from the live data of every section. Examples: "ikram ke total bakaya kitne hai", "akram ka khata dikhao", "aaj ki bikri kitni hui", "partner ki kiya total hain abhi tak", "agar commission isi raftar se chala to agle 6 mahine mein kitna hoga". The panel sits at the bottom of the screen. The model is GPT-6 Luna through OpenRouter, with Umar's own key.

Umar (02-11-2026): the model is the brain. It does the heavy work, reaches every section through its "arms", goes straight to the numbers, checks them before answering, writes the answer itself in Urdu, and also estimates the future from the past. There must be no room for a wrong figure.

Umar (02-11-2026, later): the system prompt must be professional, perfect and robust, and the assistant well rounded: real-time data, and an expert for recommendations and the future. So the model's exact instructions, its 8 tools and its answer schema are written out in `D6-system-prompt.md` and copied word for word into the code, and advice from the books is a requirement of its own (FR-7.88).

## 2. Umar's answers

| Question | Answer |
|---|---|
| Who writes the answer? | The model, as the brain (02-11-2026; on 01-11-2026 it was "the app"). Every figure in its answer is a reference that the app fills from the books and checks |
| What may go to OpenRouter? | The question and the records the model's tools read (02-11-2026, the client agrees). Never phones, addresses, father or firm names, notes, reasons, keys or locked partner figures |
| The daily limit | A hard stop at 20 questions a day; one question counts once, however many tools it uses (01-11-2026, kept on 02-11-2026) |
| When to build | Now, before the single gate. The gate then covers D3, D4, D4R, D5 and D6 together |
| Advice (02-11-2026) | The assistant also recommends what to do, from the books only, under the band "مشورہ، فیصلہ آپ کا" (FR-7.88, A90) |
| Which endpoint (02-11-2026) | Azure, pinned, with zero data retention and no fallback. OpenRouter's sample code pins OpenAI's own endpoint, which is not zero-retention. Azure has the same model at the same price. See D6 Section 0.2 |

## 3. How it works, in one example

After Events 1 to 5, "ikram ke total bakaya kitne hai":

1. The model calls `find("ikram", buyer)`. The app finds اکرم ٹول ٹیکس (0020).
2. The model calls `party_card` and `report R13` for 0020. The app reads 2,880 بنام in both places, so the figure is marked checked.
3. The model writes: "{{r1.matches.0.name}} کے ذمے {{r2.balance}} باقی ہیں ...". It writes references, never digits.
4. The app checks the answer (every reference real, no typed digit, the figure checked, Urdu script, glossary words, length), fills the references and shows: "اکرم ٹول ٹیکس (0020) کے ذمے 2,880 بنام باقی ہیں ✓", his ledger rows, the source and "پوری رپورٹ کھولیں".

Sums and percents come only from the app's calculator (`domain::figures`); estimates only from the app's estimator (`domain::forecast`), by three fixed methods, always marked "اندازہ، یقینی نہیں". Advice rests only on figures read and checked, always marked "مشورہ، فیصلہ آپ کا". Example (E2E-63, today 15-11-26): "kis se pehle wasooli karun" puts ارشد ملتانی first (18,684 بنام ✓, 36 days), then اکرم ٹول ٹیکس (2,880 بنام ✓).

## 4. What changed in each document

All of these are inside v1.16 and its partner versions, which were not yet committed, so they were amended in place on 02-11-2026 with a note in each change log.

| Document | Version | What changed |
|---|---|---|
| PRD | 1.16 | New Section 7.7, FR-7.72 to FR-7.88: the model as the brain (7.72), the panel (7.73), the answer (7.74), every figure from the books (7.75), the checks (7.76), names and periods (7.77), the conversation (7.78), when it cannot answer (7.79), quick questions (7.80), the limit (7.81), the partner code (7.82), what leaves the computer (7.83), the log (7.84), Settings (7.85), estimates (7.86, new), the tools (7.87, new), advice (7.88, new). Also 1.2, G11, FR-5.5, 6.2, 7.3 (`F1`), Section 21, E82 to E98, A80 to A90, the Must list, demo step 10a, E2E-47 to E2E-63, the money agent at v1.8 in the document table. In the last amendment: a typed ✓, the other side's words and a figure of an earlier question are refused (7.76); a difference shows its direction word (7.75); a locked partner question pauses with no model call (7.82); the 7.79 advice example moved to 7.88, reading R20, which has the days, not R21 |
| NFR | 1.8 | Area AI, Section 10a, NFR-AI-1 to NFR-AI-9 (estimates new; the 100-question list and the Urdu review; speed 8 and 20 seconds; the cost limits). NFR-SEC-9 (HTTPS calls to OpenRouter, at most 5 for one question). NFR-SEC-15 for the key. NFR-OBS-2. A row in Section 11 |
| Tech Stack | 1.8 | `reqwest`. Section 4.8: the loop, the request fields (tools, no streaming, the Azure provider block), the instructions, the tools, `tool_result_json()`, the answer type, `check.rs`, the calculator and estimator, the screens, the tests. Module "assistant" and the crate |
| Code Standards | 1.13 | Section 5.7, Q1 to Q11 (Q11: the prompt files). M14 (the calculator and estimator). 9.1, 9.2 (E2E-47 to E2E-63, `prompt.rs`, `figures_forecast.rs`, the 100-question list), 9.3 (property tests), the checklist |
| Architecture | 1.15 | Diagrams 1 and 3. `assistant_questions` with `tool_calls_json`, `answer_json`, `model_calls`, the token columns and `finish_reason` (15.3). Section 21: the tool loop and the checks |
| D6 system prompt | 1.0 | New: the exact instructions (7,584 tokens with the tools and schema, measured, after an independent review), the 8 tool definitions, the answer schema, the result shapes, the word lists and the tests P1 to P7. Plan 48 copies it word for word |
| Money agent | 1.8 | Section 9.1: the calculator and estimator, display only, every average through `avg_rate`, with the E2E-59 and E2E-61 workings; a checklist line |
| CLAUDE.md | | Versions line; a "where to read" row; the crate in the layout; the network gotcha; the gotcha "the assistant's model thinks; the app reads, works out and checks" |

## 5. What changes in the built app

Nothing that is built changes its behaviour. D6 only adds:

- a new crate, two new pure files in `domain` with their `services` wrappers, a new migration (0015), new commands and one event, a new frontend feature folder and a new Fluent file;
- one line or one block appended in the files that register these (D6 Section 5 lists every one);
- extra address parameters for the report and analytics screens, which do nothing when absent.

No existing money function, posting, domain rule, report query, earlier migration or golden number changes. `golden_day`, `assist` and the reports tests run after plan 48 with no number changed (D6-A18).

## 6. Plans and order

| Plan | Owner | What |
|---|---|---|
| 48-assistant-core | `backend` (+ money agent twice) | `domain::figures` and `domain::forecast`; the crate: client, instructions, the 8 tools, tool results, the answer type, `check.rs`, the loop, quick questions; the log; the commands; the golden questions |
| 49-assistant-panel | `frontend` | The panel, the step lines, the answer block in every state, the estimate band and chart, the chips, Settings, the Fluent file, the address parameters |
| 50-assistant-proof | `robustness` | The 100-question list, its runner and the Urdu review sheet, the attack tests, the assistant benchmarks |

Order: 48 T1 (contract, types, `pnpm gen:types`); 48 T2 (money agent, then `figures` and `forecast`) and 49 together; 48 T3 (crate, tools, checks, loop); then 50. Branch `feature/assistant` from the head of `feature/d5-early` (`d6aff46` or later). Then "D6 BUILT REPORT", then wait for Umar's "continue" and the single gate.

## 7. Open items for Umar (shown at the demo)

| ID | Default built | Question |
|---|---|---|
| A80 | The answer is in the app language, whatever the question's language | Should an English question get an English answer in the Urdu app? |
| A81 | A call that fails before any reply is not counted in the 20 | OK? |
| A82 | A table shows up to 7 rows | More or fewer? |
| A83 | The conversation clears on logout, lock and closing the app | Keep it across days? |
| A84 | The quick questions of FR-7.80 | Which questions does the Owner ask most? |
| A85 | A new model name is checked with the 100 questions before it is used at the shop | OK? |
| A86 | Estimates by same pace, same growth and same season | Which one does the Owner trust most? |
| A87 | Up to 4 model calls, plus one rewrite, and 8 tool calls a question | Enough for the hardest questions? |
| A88 | Phones, addresses, father or firm names, notes and reasons are never sent | Should the father or firm name be sent, to tell two people apart in the text? |
| A89 | The answer is at most 130 words | Longer or shorter? |
| A90 | One advice point may be added unasked when the figures call for action; at most 3 when asked | More, fewer, or only when asked? |

## 8. Commit

The lead commits these documents on `feature/d5-early` before starting D6:

```
docs: requirements v1.16 ask the app; D6

PRD v1.16 Section 7.7 (FR-7.72 to FR-7.88, E2E-47 to E2E-63, E82 to E98, A80 to A90);
NFR v1.8 Section 10a (NFR-AI-1 to NFR-AI-9), NFR-SEC-9, NFR-SEC-15, NFR-OBS-2;
Tech Stack v1.8 Section 4.8; Code Standards v1.13 Section 5.7 (Q1 to Q11) and M14;
Architecture v1.15 Section 21; money agent v1.8 Section 9.1; CLAUDE.md;
deliverables README v1.9, D6-ai-assistant.md v2.2, D6-system-prompt.md v1.0 and changes-v1.16.md v2.1.
No code changes.
```

Files: `.claude/CLAUDE.md`, `.claude/agents/money-calculations.md`, `.claude/rules/PRD_Mandi_Software_v1.md`, `.claude/rules/NFR_Mandi_Software_v1.md`, `.claude/rules/Tech_Stack_v1.md`, `.claude/rules/Code_Standards_v1.md`, `.claude/rules/Architecture_v1.md`, `docs/deliverables/README.md`, `docs/deliverables/D6-ai-assistant.md`, `docs/deliverables/D6-system-prompt.md`, `docs/deliverables/changes-v1.16.md`.
