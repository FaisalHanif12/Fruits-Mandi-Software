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

## 8. Contract as agreed (T1, 02-11-2026)

- Crate `backend/crates/assistant` (`mandi-assistant`), app -> assistant -> services + reports. Types with ts-rs in `frontend/src/bindings/` (45 new files). The full shapes are the generated bindings; summary:
  - 11 commands: `assistant_status`, `assistant_ask`, `assistant_quick`, `assistant_pick`, `assistant_rerun`, `assistant_new_conversation`, `assistant_settings_set`, `assistant_key_save`, `assistant_key_remove`, `assistant_test`, `assistant_log_list`. Event `assistant://step` with `AssistantStep { id, args: ReportMessageArg[] }`.
  - `AskReply { question_id, conversation_id, outcome, answer, options, more_options, entry_route, locked, failure, count_today, limit }`; `AskOutcome` the 16 outcomes of Section 4; `AssistantFailure { kind, http_status }`, kinds offline, timeout, bad_key (401), no_credit (402), no_endpoint (404), busy (429), server (5xx), bad_reply, other.
  - `Answer { parts, tables, charts, estimate, advice, sources, differences, next, read_at }`. Parts: text, figure (`ReportCell`, `dir`, `checked`, `working`), name (`AnswerName` fits `displayName`), date, month, doc. Tables: up to 7 rows of the report's own `ReportRow`. Charts: months and series (actual, same_pace, same_growth, same_season). Estimate with its methods and workings. `AnswerWorking` steps (sum, difference, average, percent_of, change_percent, times).
  - Sources: party_card, report (the screen's own `ReportFilters`), analytics (`AnalyticsFilters`), tiles. Lead decision: every filter that names a record also carries the whole record (id, code, both names, tag), so the panel puts whole picker values in the address and `ReportPage` is not changed.
- Lead-accepted additions: `more_options` (FR-7.77), `failure` (robustness item 6), `conversation_id` on a quick question, `default_model` in the status, part kinds month and doc, `differences` (records_differ). The app language is read from the saved `language` pref, not sent. A question is 1 to 500 characters [A, Code Standards 4.2 free text].
- New error codes (core `ErrorCode`, 155 to 159): `ERR_ASSISTANT_OFF`, `ERR_ASSISTANT_KEY_MISSING` (field `enabled` when switching on with no key), `ERR_ASSISTANT_BUSY`, `ERR_ASSISTANT_NOT_WAITING` (field `question_id`). Limit, offline, locked and HTTP failures are reply outcomes, not errors.
- Lead decisions on backend's questions: (1) chips and rerun work when the assistant is off or has no key (PRD FR-7.81: "the panel with the chips only"). The connection test needs a saved key but not the on switch, so the Owner can test a key before turning questions on; it counts (FR-7.85). (2) A Credential Manager failure is `ERR_INTERNAL`.
- Fixtures: `backend/crates/assistant/tests/fixtures/` (60 files from `tests/contract_fixtures.rs`, compare mode), copied by `frontend`.
- Existing files touched, with the lead's acceptance: appends in `backend/Cargo.toml`, `Cargo.lock`, `app/Cargo.toml`, `app/build.rs`, the capabilities, `commands/mod.rs`, `app/src/lib.rs`, `app/tests/{bindings,ipc_auth,robustness_security}.rs` (table rows). Not pure appends, accepted: `app/tests/workspace_rules.rs` (crate count 7 to 8 and the new crate's deps), `app/tests/ipc_auth.rs` (the no-delete check lets `assistant_key_remove` through, with a comment: it removes a key from Credential Manager, never a record), `crates/core/src/error.rs` (4 variants).
- Checks (backend): fmt clean; clippy -p mandi-assistant -p mandi-app -p mandi-core clean; mandi-assistant 3 passed; app ipc_auth 19, workspace_rules 6, bindings 6; robustness_security 3 named tests; core error 6; gen:types and typecheck clean.
- After T1: every source filter that names a record carries the whole record; `WorkingOp::divided_by` added; fixtures and bindings regenerated.

## 9. T2 as built (02-11-2026), accepted

- `domain::figures` (calculator and `question_numbers`) and `domain::forecast` (same pace, same growth, same season, targets), pure with checked maths; `services::figures` and `services::forecast`; `core::money::hundredths` appended (the "% of" formula, proved equal to `reports::analytics::share_hundredths` over 10,000 cases); `ERR_NO_BASE`, `ERR_TOO_LITTLE_HISTORY` (model-facing only). 53 domain tests (17 property tests), 26 doc tests.
- Money agent: before the build, then FAIL (2 must-fix in `question_numbers`, 3 should-fix), fixed with tests, re-verified **PASS**.
- Worked numbers as tests: E2E-59 288,000 and 369,000; target 3,00,000 April 2027 (pace) and February 2027 (growth, exactly 300,000); E2E-61 21,564 بنام; 3,928 of 5,448 = 72.10%; 48,000 to 51,000 = 6.25% up.
- Value kinds for `calculate` (T3 must keep this rule): a value with a `side` is a Balance (بنام positive, جمع negative); a question number with unit amount is Money; months, days, years are Count; plain takes the kind of the other inputs; percent is never an input. A sum or average of balances is signed (net); a difference, percent-of or change of balances works on sizes and needs the same side. A difference is shown as its size with the direction word (48,000 to 51,000 shows "3,000 زیادہ"; the reverse "3,000 کم").
- Deviation accepted: `services` cannot depend on `reports` (workspace rule), so the `estimate` tool reads the monthly series through `mandi_reports::analytics` (the screen's own functions, Q3) and passes it to `services::forecast::estimate`. Too little history is a result (`TooShort { full_months }`), not an error.
- Lead decisions on backend's questions:
  - (a), (b) Floors: the PRD is the master. Built as FR-7.86 says: only same growth is floored at 0; same pace and same season follow the books. Code Standards M14 and money agent 9.1 say "no projected month below 0" for all, and the floor can hide a projected loss on the profit measure. Both go to Umar (open questions); no doc is changed in this run.
  - (c) "thousand" and "million" are not read (FR-7.86 and D6 5.3 name lakh, hazar, crore). Open question for Umar.
  - (d) "3 lakh 50 hazar" reads as one amount, 3,50,000. Accepted [A]; shown to Umar.
  - (e) Months: the first month with a posting counts as a full month (E2E-59: August and September give "only 2 full months"); empty months inside the span are 0; the current month is never used.
  - (f) The unit words (mahine, din, saal ...) stay constants in `figures.rs`, because `words.tsv` is a verbatim prompt file with no unit kind and may not be edited. "10,000" alone is plain (D6 5.3). `percent`, `fisad`, `فیصد` after a number read as percent [A]. All three go to Umar's list.
  - (g) `share_hundredths` is NOT changed (only add); the equality test keeps the two equal.

- Frontend (plan 49 part a), accepted additive touches outside the D6 list: `styles/tokens.css` (dock heights), `styles/globals.css` (slide), `AnalyticsPage.tsx` (reads the address), `tests/unit/keyboard-registry.test.tsx` (rows for `F1`, `Ctrl+↑`, `Ctrl+↓`, now PRD 7.3 keys).

## 10. T3 lead decisions (02-11-2026)

Backend reported T3 built: 108 assistant tests (E2E-47 to E2E-63, P1 to P7, one test per FR-7.76 rule, the 8 tools, loop limits); golden_day 94, assist 23 and the touched services and app suites passing; fmt and clippy clean. The HANDOFF waits for the money agent's after-review.

- (a) P3 glossary: 7 glossary terms in the Fluent files and «پیشگی باقی» are missing from the prompt's glossary line. They are kept as known gaps in the test. The prompt files are verbatim and do not change. This goes to Umar.
- (b) P5 keys: `je4518`, `i41` and `lt88` cannot be produced exactly, because ids are `max+1`, and dummy rows would shift the real بکری numbers. Accepted: each fixture maps the example key to the world's real key, and lists the mapping. No dummy rows. The prompt is not edited.
- (c) E2E-49: the 72.10% is read from view A unfiltered, citing the banana row. With the item filter on, the share is 100%. No expected number changes.
- (d) `cargo deny`: licenses, bans and sources pass. Advisories fail on 6 new RUSTSEC-2026-03xx entries in crates already in the tree (tokio, crypto-bigint, ctutils, zstd). None come from `reqwest` or `tiktoken-rs`. There is no upgrade in this run. This is a gate follow-up for Umar.
- (e) Not done, listed for Umar:
  - The attention-list item for a records difference (FR-7.76 point 2) is not in the T3 task list.
  - The estimate measures `laga`, `commission_buyer`, `receipts`, `garden_sales` and `trip_sales` are refused, because the books have no monthly series for them yet.
- (f) Frontend F2 starts against the real commands. robustness starts plan 50 in its own files.
- (g) Contract change (02-11-2026), agreed by backend and frontend through the lead: `assistant_new_conversation` takes `AssistantNewConversationInput { conversation_id: Option<String> }`. With an id, the runtime drops that conversation's memory at once. With null or an unknown id, nothing is dropped. It always returns the new id. "New question" in the panel calls it with the old id.
- (h) Step lines carry a party's name as `ArgValue::Names` (ur, en, tag), so a language switch during a question shows the right script (rule 8, F38).
- (i) robustness findings during plan 50, fixed in T3:
  - (1) MAJOR: Urdu number words outside words.tsv (27, 28, 31, 99 ...) passed the check. Lead decision: words.tsv stays verbatim; check.rs refuses a word from words.tsv or from its own constant list (Urdu 1 to 99 with spelling variants, سو, ہزار, لاکھ, کروڑ, ارب, and Roman Urdu 1 to 99). Adding the list to words.tsv goes to Umar.
  - (2) MINOR: negative token counts are clamped at 0, so the question is not lost.
  - (3) MINOR: only a change request carries entry_route.
- (j) The bench lines (`criterion` dev-dependency, `[[bench]] assistant`) are added to the assistant Cargo.toml by backend, for robustness.

## 11. T3 as built (02-11-2026), accepted when robustness findings (1) and (3) are in

- Backend HANDOFF:
  - The 8 read-only tools, every FR-7.76 check, fill, the loop with limits as named constants, the plain-table fallback, chips, rerun, the connection test, the question log, the real commands and the `assistant://step` event.
  - NFR-AI-6 the literal way: `Db::open_snapshot` and `Context::snapshot`, one WAL snapshot per question; a pick opens a new one.
  - Late items in: (a) the average's n as its last input; (b) EntryRoute.party is a whole record, a partner record on kharcha, sarmaya and phal; (c) step-line names as `ArgValue::Names`, with a fix for an always-empty name; (d) `assistant_new_conversation` with the old id (`Runtime::forget`); (e) the bench lines; (f) token counts clamped at 0.
- Money agent: three passes.
  - Pass 1 FAILED calc.rs and report.rs (8 items, among them heading lines summed into R21 as 43,128, a ✓ on a typed number, and a false "records differ").
  - Pass 2 FAILED analytics.rs (2 items).
  - Pass 3 PASS on analytics, calc, report and cells. No money arithmetic outside services::figures and services::forecast; no float, no unwrap.
  - Added by the review: a typed number beside balances on both sides, or all exactly 0, is refused (ERR_OUT_OF_RANGE). Untested cases named by the agent are gate follow-ups.
- Checks (backend):
  - fmt clean; clippy clean on assistant, app, services and db.
  - Assistant: lib 33, check_rules 14, contract_fixtures 3, golden_prompt 1, golden_questions 23, loop_limits 14, prompt 7, tools 24, settings_service 6, shared_formulas 2, all passed.
  - golden_day 94, assist 23, all passed.
  - Reports suites passed, with no expected number changed.
  - db: read_moment 2, assistant_questions 21, schema 4, migration 8, all passed.
  - app: bindings 6, ipc_auth 19, workspace_rules 6, robustness_security 11, all passed.
- `cargo deny`: advisories fail, with 16 ids over 5 crates already in the tree: crypto-bigint 0387, ctutils 0385, tokio 0386 (needs 1.53.3 or later), zstd 0391, zstd-safe 0395 to 0406. bans, licenses and sources pass. The deny.toml zip@4.6.1 yanked entry matches nothing now. These are gate follow-ups for Umar.
- Robustness findings (1) to (3) fixed (02-11-2026):
  - (1) `number_words.rs`: Urdu 1 to 99 with spelling variants, the big steps and Roman Urdu. words.tsv is untouched. Bare «اسی» and «بہتر» are refused only before a unit word, and Roman `do`, `teen` and `char` are left out, because they are everyday words. These go to Umar.
  - (2) Token counts are clamped at 0.
  - (3) Lead correction: PRD E98 and the answer schema (D6-system-prompt.md line 535) allow `entry` on an answer or advice when the question also asks for a change; the panel then shows the change refused line and button. So answer and advice keep the route the model sends, and an estimate carries none. D6 attack 15's "never shows a change button for advice" conflicts with E98. The PRD wins, and the conflict goes to Umar. Frontend draws the change refused line on an answer or advice with a route.
