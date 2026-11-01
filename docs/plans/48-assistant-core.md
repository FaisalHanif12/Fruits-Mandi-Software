# Plan 48: ask the app, the core (backend)

Version 1.0, 02-11-2026. Lead: tech-lead. Deliverable D6 (`docs/deliverables/D6-ai-assistant.md` v2.2, `D6-system-prompt.md` v1.0, `changes-v1.16.md`). Branch `feature/assistant`, made from `feature/d5-early` at `ad3d813` (the v1.16 docs commit on top of `d6aff46`). Folder `Fruit Mandi Parallel/d5`.

## 1. Goal

The Owner asks a question about his books in any wording. The model (GPT-6 Luna on OpenRouter, Azure only) is the brain: it calls 8 read-only tools, asks the app to add up and estimate, and writes the answer with references, never digits. The app checks every figure and fills it. This plan builds everything on the backend side. The panel is plan 49; the live proof is plan 50.

## 2. Requirements

- PRD v1.16 Section 7.7, FR-7.72 to FR-7.88 (backend side of each); E2E-47 to E2E-63; E82 to E98; A80 to A90.
- `D6-system-prompt.md`, all of it (the four files, the result shapes, the tests P1 to P7).
- NFR v1.8: NFR-AI-1, NFR-AI-3 (backend part), NFR-AI-4 to NFR-AI-7, NFR-AI-9; NFR-SEC-9; NFR-SEC-15; NFR-OBS-2.
- Tech Stack v1.8 Section 4.8 (the flow and every piece). Code Standards v1.13 Section 5.7 (Q1 to Q11) and M14. Architecture v1.15 Section 15.3 (`assistant_questions`, settings keys) and Section 21. Money agent v1.8 Section 9.1.

## 3. Worked numbers (from the PRD; the code must give exactly these)

- E2E-59 (today 10-10-26; commission by month April to September 2026: 36,000, 39,000, 42,000, 45,000, 48,000, 51,000):
  - same pace: (45,000 + 48,000 + 51,000) ÷ 3 = 48,000 a month; 48,000 × 6 = 288,000.
  - same growth: (51,000 − 36,000) ÷ 5 = 3,000 a month; 54,000 + 57,000 + 60,000 + 63,000 + 66,000 + 69,000 = 369,000.
  - same season: not shown (6 full months, needs 12).
  - target 3 lakh (prompt example 8): reached April 2027 by same pace, February 2027 by same growth.
- E2E-61: as written in the PRD (the money agent works it first, Section 9.1).
- Worked day after Events 1 to 5: اکرم 2,880 بنام (party card = R13 closing); اکرم + ارشد = 2,880 + 18,684 = 21,564 بنام; today's sales 41,400 in 2 بکری; today's commission 5,448; cash in hand 58,484; partner کل رقم الف −1,004, ب 746, ج 746 (E2E-50).
- E2E-63 (today 15-11-26): R20 order ارشد ملتانی 18,684 بنام (36 days), then اکرم ٹول ٹیکس 2,880 بنام.
- Every rounding through `core::money` (`avg_rate`, `apply_bps`); no float; checked maths.

## 4. Contract (D6 Section 2 is the contract; T1 writes the Rust types, the lead records them in Section 8)

Commands (all need the signed-in Owner; none writes to the books; `assistant_ask`, `assistant_quick`, `assistant_pick`, `assistant_rerun` and `assistant_test` write one log row each; only a model question and a connection test count in the 20):

| Command | Input | Output |
|---|---|---|
| `assistant_status` | none | on or off, key saved (last 4 characters only), model name, today's count, limit 20 |
| `assistant_ask` | `text`, `conversation_id` (null for new) | `AskReply` |
| `assistant_quick` | `chip` (FR-7.80), `party_id` for the party chip | `AskReply` (no model call) |
| `assistant_pick` | `question_id`, the chosen match, or that the partner code was given | `AskReply` (not counted again) |
| `assistant_rerun` | `question_id` | `AskReply` (same tool calls on today's books, plain tables, a note; no model call, not counted) |
| `assistant_new_conversation` | none | a new `conversation_id` |
| `assistant_settings_set` | on or off, model name | status |
| `assistant_key_save`, `assistant_key_remove` | the key (save only), the Owner's password | status (never the key) |
| `assistant_test` | none | worked or not, milliseconds, the error (for example HTTP 404) |
| `assistant_log_list` | page (offset, limit 100) | rows and total count (B31) |

Event `assistant://step`: a Fluent message id and its values.

`AskReply`: `question_id`, `conversation_id`, `outcome` (answered, estimate, advice, which_one, need_dates, need_party, not_found, outside_books, change_refused, not_understood, checks_failed, records_differ, locked, limit_reached, offline, error), `answer`, `options` (up to 5 matches: name, tag, code, city, balance), `entry_route`, `locked`, `count_today`, `limit`.

`answer`: `parts` (text | figure with Cell, side, checked, working | name | date), `tables` (source, columns, up to 7 rows of the report's own `Row` type, totals), `charts`, `estimate`, `advice`, `sources` (label id, screen address and filters, print route), `next`, `read_at`.

Money in every output as paisa through the existing report `Cell` types (the frontend formats only). New error codes for the assistant go to `errors.ftl` through the frontend; tell the lead and `frontend` the list in T1.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: crate skeleton `backend/crates/assistant` (`mandi-assistant`, workspace member), all command input and output types with ts-rs, command stubs returning `ERR_NOT_IMPLEMENTED`, registered (append only), `pnpm gen:types`, a few fixtures of `AskReply` in every outcome for the frontend. Message the lead and `frontend` with the exact types. | backend | |
| T2 | Money agent FIRST on `domain::figures` and `domain::forecast` (worked numbers of Section 3, money agent 9.1). Then build them (pure, checked `Paisa`, M14) with `services::figures` and `services::forecast` (monthly series through the analytics view functions). Unit and property tests in `crates/domain/tests/figures_forecast.rs`, including `question_numbers` ("agle 6 mahine" 6 months, "3 lakh" 3,00,000 amount, "10%" 10 percent). Money agent AGAIN to review; its findings fixed; attach its final report. | backend (+ money agent twice) | T1 |
| T3 | The crate: `client.rs` (`ModelClient`, `OpenRouterClient` with reqwest and rustls, `RecordedClient`), `prompt/` (the four files copied BYTE FOR BYTE from `D6-system-prompt.md` Sections 2, 3, 4, 6) and `prompt.rs`, `request_body()` with the provider block as a constant, `tools/` (8 tools), `results.rs` and `tool_result_json()`, `answer.rs`, `check.rs`, `run.rs`, `quick.rs`; migration `0015_assistant_questions.sql` and `db/src/assistant.rs`; settings keys; the key in `keyring` under its own entry; audit entries; the real commands. Tests: `tests/prompt.rs` (P1 to P7, prints the token count), `tests/golden_questions.rs` with `tests/fixtures/model/*.json` (E2E-47 to E2E-63 plus the extra checks of D6 Section 3 / 48). Then `golden_day`, `assist` and the reports tests with no number changed. | backend | T2 |

## 6. Rules for this plan (lead decisions)

- **Only add.** Existing files touched only as D6 Section 5 lists, and only by appending. No existing money function, posting, report query or earlier migration changes. If a report cannot give what a tool needs, ask the lead; do not change the report.
- **The prompt files are never edited.** They are byte copies. Lead decision: the example keys in `system.md` (`je4518`, `je4521`, `a20`, `a21`, `i41`, `lt88`, `s1` to `s3`) stay as written. The P5 test data must produce those keys (for example: key schemes from record ids or codes, and the test database's starting row ids set in the fixture before seeding, which is test setup only). If that is impossible for a key, stop and tell the lead; do not edit the prompt or `D6-system-prompt.md`.
- **The OpenRouter key is never written** in a file, a test, a fixture, a log, a message or a commit. Tests use `RecordedClient` only. No commit test calls the network.
- **No arithmetic in `mandi-assistant`**: `#![deny(clippy::arithmetic_side_effects)]` in the crate; every sum, percent, average and estimate goes through `services::figures` / `services::forecast`.
- Limits as named constants (Q9): 4 model calls plus one rewrite, 8 tool calls, 3,000 output tokens a question (1,500 a call), 60,000 input tokens, 15 s a call, 45 s a question, 20 questions a day (Pakistan date), 16,000 bytes a result, 130 words, 7 table rows, 12 months ahead.
- New dev-dependency `tiktoken-rs` (P1) and dependency `reqwest` (rustls, no default features): check `cargo deny check licenses bans` for them only.
- Targeted checks only (status.md rules in force): `cargo test -p mandi-assistant`, `-p mandi-domain --test figures_forecast`, the new db and app tests, `golden_day`, `assist`, the reports tests; `cargo fmt --all --check`; `cargo clippy -p <touched crates> --all-targets -- -D warnings`. No `--workspace`.
- Builders never run git add, mv, rm, stash, checkout or commit.

## 7. Acceptance (from D6 Section 7)

D6-A1 to D6-A12, D6-A18 and D6-A19 (backend parts), D6-A5 clippy evidence, D6-A4 money agent PASS.

## 8. Contract as agreed

(The lead fills this in after T1.)
