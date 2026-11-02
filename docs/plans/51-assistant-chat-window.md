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
