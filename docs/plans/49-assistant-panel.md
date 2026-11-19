# Plan 49: ask the app, the panel (frontend)

Version 1.0, 02-11-2026. Lead: tech-lead. Deliverable D6 (`docs/deliverables/D6-ai-assistant.md` v2.2, Section 3 "49-assistant-panel", Sections 2 and 4). Branch `feature/assistant`. Folder `Fruit Mandi Parallel/d5`. Owner: `frontend`.

## 1. Goal

`F1` opens a panel at the bottom of every screen. The Owner types a question; step lines show the work; the checked answer shows with its figures (✓, ⓘ), tables, a chart, the estimate and advice bands, buttons, the source line and next questions. The frontend shows only what the backend sends: it never sums, never builds a figure (rule 1, F2, M12).

## 2. Requirements

PRD v1.16 FR-7.73, FR-7.74, FR-7.76 (the difference state), FR-7.77, FR-7.79 to FR-7.82, FR-7.85, FR-7.86 (band and chart), FR-7.88 (advice band), FR-5.5 (status bar button and counter), Section 7.3 (`F1`); NFR-AI-3 (screen part), NFR-AI-8, NFR-USE-15 to NFR-USE-28; Code Standards F1 to F44 as they apply. Skills: `bilingual-screen`, `urdu-english-text`.

## 3. Contract

Plan 48 Section 8 (the types are generated into `frontend/src/bindings/` by `pnpm gen:types`; fixtures from plan 48 T1 are copied into the frontend fixtures as in earlier plans).

## 4. What to build (D6 Section 3, plan 49, is the full list)

- `features/assistant/`: `AssistantDock` (mounted once in `AppShell` above the status bar; two heights `Ctrl+↑` `Ctrl+↓`; slide 160 ms or less; focus trap; `Esc` closes and returns the focus; never unmounts on a route change).
- Question box: raw text (no Roman-to-Urdu), `Enter` sends, `↑` recalls the last question. First line "سوال سمجھ رہا ہوں…" within 100 ms, then the `assistant://step` lines.
- `AnswerBlock`: parts (`text` as written; `figure` through `<Num>` with ✓ when checked and `CalcInfo` when it has a working; `name` through `displayName`; `date` through the date formatter); tables through the report screens' own cell renderer (`features/reports/cells.ts`, exported from `features/reports/index.ts`), row clicks through `useOpenRowSource`; charts through the shared chart wrapper; the amber estimate band "اندازہ، یقینی نہیں" and the calm advice band "مشورہ، فیصلہ آپ کا"; buttons through `useOpenReport` and the print route; source line; next-question chips.
- Every state designed (F8): working, answer, estimate, advice, advice with estimate, which one, not found, need dates, need party, locked (`PartnerLockedBox`, then `assistant_pick`), outside the books, change refused (button to the entry screen, amounts empty), not understood, checks failed (plain tables), records differ, limit reached, offline, error.
- Quick-question chips (FR-7.80) with a party picker through `AccountPicker`.
- Status bar button "سوال پوچھیں F1" and the counter "آج 5 / 20". `F1` registered in `shared/keyboard/shortcuts.ts`.
- Settings group "سوال پوچھیں" (FR-7.85): on or off, the key (paste, then masked with last 4; replace and remove ask the Owner's password), model name, test the connection, today's count, the question log page by page (each row reruns).
- `assistant.ftl` in `ur` and `en`: every fixed message of PRD 7.7 and every step line, each `# review: pending`.
- Addresses, additive only: the report address (`features/reports/search.ts`) learns the remaining report filters; the analytics address learns its filters. With no extra parameter both screens behave exactly as today.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| F1 | Panel, answer block, every state, chips, status bar button, Settings group, Fluent, addresses, against the plan 48 fixtures | frontend | 48 T1 |
| F2 | Wire to the real commands and the step event once 48 T3 is built; tests and screenshots | frontend | 48 T3, F1 |

Tests: every state (F8); keyboard-only run (F30: `F1`, type, `Enter`, `Tab` to a button, `Enter`, `Esc`); language switch with the panel open over an unsaved بکری, nothing lost (F39); no `text` part ever drawn as a number cell and every `figure` part is; the existing report and analytics tests pass unchanged; screenshots ur and en at 1024 × 600, 1366 × 768 and 1920 × 1080 (F14, F23).

## 6. Rules

- Touch only the files D6 Section 5 lists, and only by adding: `AppShell.tsx` (mount), `StatusBar.tsx` (button), `shared/keyboard/shortcuts.ts` (`F1`), `features/reports/search.ts` and `index.ts`, the analytics address parsing, the Settings page (one group), Fluent files (new `assistant.ftl`, new error ids).
- The frontend never sees or stores the key except the text the Owner pastes into the field before it is sent; never in a store, log or test.
- Targeted checks only: `pnpm typecheck`, eslint and prettier on changed files, `pnpm i18n:check`, the assistant tests and the touched report and analytics test files (`--maxWorkers=2`). No whole vitest.
- Builders never run git add, mv, rm, stash, checkout or commit.

## 7. Acceptance

D6-A10 (screen part), D6-A13, D6-A14, D6-A15, D6-A18 (frontend part).

## 8. As built

### F1 (02-11-2026), accepted

- The panel works end to end against the 60 backend fixtures in both languages: `F1` and the status bar button with the counter, two heights, focus trap, `Esc`, the conversation kept across screens and a language switch, step lines, `AnswerBlock` in every state, the chips with a party picker, the Settings group, the report and analytics addresses. It calls the real commands through `features/assistant/api.ts`; until 48 T3 the stubs answer `ERR_NOT_IMPLEMENTED`, shown as words with a way to Settings, never as fake data.
- Existing files touched (all additions): `AppShell.tsx`, `StatusBar.tsx`, `shared/keyboard/shortcuts.ts` and `index.ts`, `features/reports/search.ts` and `index.ts`, `features/analytics/index.ts`, `AnalyticsPage.tsx` (reads the address once), `app/router.tsx` (validateSearch on the two analytics routes, the dev route), `styles/tokens.css`, `styles/globals.css`, `i18n/ur|en/errors.ftl` (4 `ERR_ASSISTANT_*`, `ERR_NO_BASE`, `ERR_TOO_LITTLE_HISTORY`). Existing tests changed, accepted: `keyboard-registry.test.tsx` (+3 rows: `F1`, `Ctrl+↑`, `Ctrl+↓`), `settings.test.tsx` (nine tabs become ten, the new group).
- Checks (frontend): typecheck clean; eslint, prettier, stylelint clean on changed files; i18n:check passed; vitest 14 files, 682 tests passed. Screenshots ur and en at 1366 x 768 for 20 states and Settings, plus 1024 x 600 and 1920 x 1080 for six states, no sideways overflow: `frontend/tests/e2e/output/screenshots/plan49/` (68). Urdu shaping and printing on Windows: LOCAL-ONLY.
- Lead decisions: (1) change refused opens the entry screen with the party chosen (FR-7.79): an optional `party` address parameter, additive only, on the voucher, expense, lent, khata, capital, fruit and general screens; with no parameter each screen is as before; amounts stay empty. (2) An average's working carries n as its last input, written (a + b + c) ÷ n. (3) Answer buttons use the sources' filters; Print goes to A4 in the app language. (4) The shut panel's body leaves the page after its slide.

### F2 (02-11-2026), accepted with follow-ups

- The panel is wired to the real commands and the step event `assistant://step`, with one api.ts wrapper per command. All 61 backend fixtures are byte-identical to the frontend copies. The 13 that changed in T3 carry whole party records in sources and filters; no frontend change was needed for them. All 64 `assistant-*` ids the backend sends are in ur and en. `assistant-step-writing` was removed, because it is never sent. All 9 failure kinds and all 16 outcomes are designed.
- Checks (frontend):
  - typecheck clean; eslint, prettier and stylelint clean on changed files; i18n:check passed.
  - Assistant and settings files: 9 files, 375 tests passed.
  - Report and analytics files: 9 files, 495 tests passed.
  - Screenshots ur and en, 24 states at 1024 x 600, 1366 x 768 and 1920 x 1080 (144 files), in `tests/e2e/output/screenshots/plan49/`. No sideways overflow.
  - The replies come from the backend's contract fixtures, not a live run.
  - Urdu shaping and printing on Windows: LOCAL-ONLY.
- Lead decisions:
  - (1) A party's name in a step line goes out as `ArgValue::Names` (ur, en, tag), not as Urdu text (rule 8, F38). This is a backend fix; the frontend test then uses the real argument.
  - (2) "New question" calls `assistant_new_conversation` first, so the runtime drops the old memory at once.
  - (3) At 1024 x 600 the chips fold in the tall height too, once a conversation has begun.
- F2 follow-ups as built, accepted:
  - "New question" calls `assistant_new_conversation` with the old id, and the next ask sends null.
  - Step lines read `ArgValue::Names`, in the app's script. A card or ledger step with no name shows the plain "preparing" line.
  - At 1024 x 600 the chips fold in both heights once a conversation has begun. Change refused, which one and sum then fit with no scrolling.
  - All 62 fixtures are byte-identical to the backend's.
  - Checks: typecheck, eslint, prettier and i18n:check clean; 9 assistant files, 383 tests passed.
- E98 (02-11-2026), accepted: an answer or advice that carries an entry route shows the change refused line and the entry-screen button right under its words, using the shared `ChangeRefusedView`; an estimate never shows it. No new Fluent text. 391 tests passed. At 1024 x 600 the button is one scroll below the refusal line. The tests move to backend's real fixtures `assistant_reply_answered_route` and `assistant_reply_advice_route`.
