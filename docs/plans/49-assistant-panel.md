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

(The lead fills this in.)
