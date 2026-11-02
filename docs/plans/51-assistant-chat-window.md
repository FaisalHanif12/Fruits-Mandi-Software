# Plan 51: the chat window and the connection fix

Version 1.0, 02-11-2026. Lead: tech-lead. Asked by Umar on 02-11-2026, after he tried the app. Branch `feature/assistant`, folder `Fruit Mandi Parallel/d5`.

## 1. Goal

1. The connection test failed with HTTP 404, "no Azure server that keeps no data". Fix it.
2. Ask the app opens as a modern chat window above a floating round button, in the app's own colours, with more room than a small chat box, because answers carry tables and charts.

## 2. Requirements

- PRD v1.17 FR-7.73 (changed). FR-5.5 is kept: the status bar keeps "سوال پوچھیں F1" and today's count.
- FR-7.74 to FR-7.88 and NFR-AI-1 to NFR-AI-9 are unchanged. FR-7.85 covers the connection test.
- NFR-USE-15 to NFR-USE-28, and Code Standards F1 to F44 as they apply.

## 3. Finding: why the connection test returned 404

OpenRouter's public endpoint list (`/api/v1/models/openai/gpt-6-luna/endpoints` and `/api/v1/endpoints/zdr`, read 02-11-2026) shows three Azure endpoints for `openai/gpt-6-luna` (`azure`, `azure/us`, `azure/eu`). All three are on the zero-data-retention list.

- Their supported parameters are: `include_reasoning`, `max_completion_tokens`, `reasoning`, `reasoning_effort`, `response_format`, `seed`, `structured_outputs`, `tool_choice`, `tools` and `verbosity`.
- Our request body (`request.rs`) sends `max_tokens` and `parallel_tool_calls`. Azure lists neither.
- Our provider block asks `require_parameters: true`. So OpenRouter drops every Azure endpoint and answers 404.

The block of the goal ("D6 BLOCKED" on a 404) does not apply: the endpoint exists, and the 404 is our own request.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| B1 | Fix the request in `request.rs`, test it and get the money agent's view (see below) | backend |
| F1 | Build the chat window in `features/assistant/` (see below) | frontend |
| L1 | After B1 and F1, rebuild and run the app; Umar presses "Test the connection" | lead and Umar |

### B1: the request body

- Send `max_completion_tokens` (same value, 1,500 a call) instead of `max_tokens`.
- Remove `parallel_tool_calls`. Parallel tool calls are the model's default, so behaviour does not change.
- Keep the provider block exactly as it is: Azure only, no fallback, `zdr: true`, `data_collection: deny`, `require_parameters: true`.
- New test: every top-level key of the request body is in a constant list of the Azure parameters above, plus the always-allowed keys `model`, `messages` and `provider`. A key not on the list fails the test.
- Update any recorded request body or fixture that holds the old keys. No expected answer changes.
- Make the 404 message say what to check, in both languages: the model's Azure endpoints, or a parameter Azure does not take. The connection test shows it.
- The output budget is unchanged, so the money agent need not review. Run the assistant tests, golden_questions, robustness_assistant, fmt and clippy -p mandi-assistant.

### F1: the chat window

- **Launcher.** A round floating button in the app's own colours, above the status bar at the end side (bottom right in English, bottom left in Urdu, through logical CSS). It shows the assistant icon, and a small dot when the assistant is on and has a key. When the window is open it shows a close icon, as in Umar's picture.
- **Window.** It opens above the launcher.
  - A card with large rounded corners, a soft shadow and the tokens of `styles/tokens.css`. No raw colours or sizes (F9).
  - Header: the assistant's avatar or icon, its name ("سوال پوچھیں" / "Ask the app"), its state (ready, working, offline, off), the "نیا سوال" (New question) button, the size button and close.
  - Body: the conversation. Questions on the end side, answers as wide cards that can hold tables, charts and the bands. A day label at the top, as in the picture. The empty state is a greeting line plus the quick questions as round chips.
  - Footer: a rounded question box and a round send button.
  - Today's count stays visible, in the header or the footer.
- **Sizes.** The window is much larger than Umar's reference box:
  - normal: about half the window wide (at least 560 points) and most of its height above the status bar;
  - large: nearly the whole window.
  - `Ctrl+↑` and `Ctrl+↓` and the size button switch them. At 1024 × 600 both fill the space above the status bar. Tables scroll inside their own card, and the page never scrolls sideways.
- **Keep everything else as it is.** Everything inside the panel stays: `AnswerBlock`, every state, the step lines, chips, pick, rerun, the change refused line and the Settings group. The panel only gets the new frame.
  - `F1` and both buttons open and close it.
  - Focus trap, `Esc` and focus back.
  - Kept across screens, and nothing lost under it.
  - Enter sends, and ↑ recalls the last question.
  - A short open animation, 200 ms or less (NFR-USE-24).
- **Keyboard registry.** Rename the shortcut rows if the labels change.
- **Tests.**
  - Update the dock tests for the launcher and window: open and close by `F1`, the status bar button and the launcher; the two sizes; Esc and focus.
  - Both languages; nothing under the window is lost.
- **Screenshots**, ur and en at 1024 × 600, 1366 × 768 and 1920 × 1080:
  - the launcher alone;
  - the empty window;
  - a working answer;
  - a tall answer with a table;
  - the large size.
- **Checks.** Targeted only: typecheck, eslint and prettier on changed files, i18n:check, and the assistant test files with `--maxWorkers=2`.

## 5. Acceptance

- Umar's "Test the connection" answers OK on Azure.
- The chat window looks and works as Section 4 says, in both languages and at all three sizes.
- Every assistant test still passes, and golden numbers are unchanged.

## 6. B1 as built (02-11-2026), accepted

- The request body now has 9 keys: model, messages, tools, tool_choice, response_format, reasoning, max_completion_tokens, seed and provider.
- `max_tokens`, `parallel_tool_calls` and `stream` are gone. Their defaults keep the same behaviour: answers are shown whole, and parallel tool calls stay the model's default.
- The provider block is unchanged.
- `BODY_KEYS_ALLOWED` and the harness scan fail every test on any other key.
- In robustness_assistant.rs, three key names changed, mechanically (gate note).
- Checks: assistant tests 0 failed; fmt and clippy -p clean.
- D6-ai-assistant.md Section 5 still shows the old keys (lines 85, 86, 93, 109 and 112). It is updated after Umar's connection test passes.

## 7. P51-B2: a failed advice question and slow answers (02-11-2026)

Umar asked "apni sale ko ziyada kaise karain ?" and got "not understood". He also said answers are too slow.

Diagnosis from the question log, row 7:
- The model passed the null `from` of `period up_to_now` together with `compare` (ERR_REQUIRED), then sorted by an invented column `sales` (ERR_OUT_OF_RANGE).
- The error texts named engine fields with an empty detail and no list of allowed words, so the model could not correct itself.
- The limits (4 calls, 8 tools) then ended the loop as not_understood.
- Time: the app's own work is about 40 ms a question. Each model round trip is the rest, and a question takes 2 to 4 of them. Error rounds add more.

Lead decisions:
1. Errors teach. Each refusal gives the tool's own field name, a reason, and `allowed` (the columns, the views and the filters).
2. A null period `from` is "no date". compare without dates gets a clear message.
3. Contract change: the reply carries `stop_reason` when a limit stops the loop, and `finish_reason` logs it. The panel then says it could not finish reading, suggests smaller questions, and shows any plain tables already read.
4. A new migration adds timing columns to `assistant_questions` (additions only).
5. Umar approved ONE prompt change: the tool descriptions list the exact column words of each view and report. That makes D6-system-prompt.md v1.1, and the prompt files are re-copied verbatim. He did not approve chained calls, an alerts tool, or low reasoning.
6. The robust set: one golden transcript per playbook line, plus the failure list.

As built so far:
- Fixes 1 and 2 are in.
  - Money agent reviewed fix 2: no figure changes. Its Medium finding (only a period's `from` may be empty) and its Low finding are fixed.
  - Tests: teach 11 and loop_teach 2. Umar's question now ends as advice in the recorded loop.
  - R20 single-key sort by name now works.
  - The assistant suite: 0 failed. fmt and clippy are clean.
- P51-B2 as built (02-11-2026), accepted:
  - Fixes 1 to 4 and 6 are in, and step c: tools.json equals the D6-system-prompt v1.1 block (P7); MAX_PREFIX_TOKENS is 9,000; P1 measures 8,811; the new P8 passes and fails on broken drafts.
  - New tests: teach 12, playbook 16, loop_teach 3, loop_stop 4, loop_timing 3, prompt_columns 1. golden_questions 24 is unchanged, and golden_day 94 passes.
  - Migration 0017 adds timing; it ran on a copy of Umar's data (16 to 17, rows unchanged, integrity ok).
  - Bugs fixed on the way: `where` on an empty report column, and a single-key sort by a shown word.
  - Not changed: chained calls, an alerts tool, low reasoning (Umar).
  - Gate follow-up: services/tests/robustness_balance.rs (robustness's) has 3 tests that failed before this work. They pin LATEST_VERSION 10, while the schema is now at 17, and need rewriting for migrations 11 to 17.

## 8. P51-B3: the timeout, app help and answer length (02-11-2026)

Umar asked "is main market safar kiya hai iski mujhay samaj nahi aa rahi ?" and saw "the model's answer could not be read".

The log, with timing from 0017:
- Row 12: one call, failed after 18,309 ms, no tokens.
- The cause: CALL_TIMEOUT_SECS is 15. The timeout fired while the body was being read, and `client.rs` maps every body-read error to BadReply, not Timeout.
- Real calls take 7 to 9 s each: row 8 took 8.85 s and 7.02 s (2 calls, advice); row 11 took 9.0 s (1 call). The app's own part is 0 to 2 ms.
- Row 11, "mujhay bikri add krne ka tareeka batao?", got advice with an entry route. The panel added "میں صرف بتا سکتا ہوں کچھ لکھ نہیں سکتا", which reads wrongly for a how-to question.

Umar's decisions (02-11-2026):
1. Longer limits: 30 s a model call and 60 s a question (NFR-AI-3, NFR v1.10). A timeout is reported as a timeout, never as an unreadable reply.
2. An app help tool: the assistant answers how-to and "what is this" questions from a written guide of every screen and module. The guide is taken from the PRD, in plain steps. This is a new FR in PRD v1.18 and D6-system-prompt v1.2.
3. Answer length follows the question. A data question ("faisal ka khata") gets a short, direct answer. A recommendation or a guide may explain, inside the 130-word cap. This is a system.md rule, in v1.2.
4. Not approved: low reasoning, and a fixed verbosity setting.

Tasks:
- B3 (backend):
  - (a) the limits and the timeout mapping, with tests;
  - (b) `app_help`: a tool whose topic enum covers every screen and module, returning that topic's guide text. Draft the guide text from the PRD, each topic with what it is for, the steps, the keys and a worked example. Draft the tool definition and the system.md lines (when to call app_help; the length rule);
  - (c) send the drafts to the lead, who puts them into D6-system-prompt v1.2; then copy verbatim;
  - (d) recorded tests for "market safar kya hai", "bikri kaise add karun", "wasooli kaise likhun", "partner ka hisaab kaise dekhun", in Roman Urdu, Urdu and English;
  - (e) set the stale PREFIX_TOKENS 8000 in scripts/ai-eval/run.mjs to 9000, and add help questions to questions.tsv (robustness has stopped; this is mechanical).
- F3 (frontend): on an answer or advice with an entry route, show only the neutral line and the button ("اندراج کی اسکرین کھولیں"; the amounts stay empty). The full refusal line stays for the change_refused outcome. The timeout text reads "the answer took too long, please ask again".
- L3 (lead): the docs, then the 100-question live run with Umar's key.
- P51-B3 as built (02-11-2026), accepted:
  - 30 s a call and 60 s a question; a timeout at any stage, the body read included, is reported as Timeout.
  - `app_help` with 38 topics. The guide is cited by reference and shown word for word (Val::Guide).
  - The help button opens any of 30 screens; a main screen never keeps a party.
  - The prompt files equal D6-system-prompt v1.2. P1 measures 9,402 of 9,500. P7, P8 and P9 pass.
  - Tests: the assistant crate, 248 passed, 0 failed, 2 ignored (the slow and the live ones); golden_questions unchanged; 12 recorded help questions; prompt-ex-16.
  - scripts/ai-eval: 110 questions, PREFIX_TOKENS 9500.
  - Open for Umar: the Urdu review of the guide, and 3 unverified labels (how Close the day is reached, the capital kind labels, the name of the print list).
