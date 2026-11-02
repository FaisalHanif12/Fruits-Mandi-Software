# Plan 50: ask the app, the proof (robustness and the live check)

Version 1.0, 02-11-2026. Lead: tech-lead. Deliverable D6 (`docs/deliverables/D6-ai-assistant.md` v2.2, Section 3 "50-assistant-proof" and Section 6). Branch `feature/assistant`. Folder `Fruit Mandi Parallel/d5`. Owner: `robustness`. This is build work in files `robustness` owns, NOT the review gate (the gate waits for Umar's "continue").

## 1. Goal

Prove the assistant: a 100-question list with its runner and the Urdu review sheet, the attack tests of D6 Section 6 on recorded transcripts, and the assistant speed benchmarks.

## 2. Requirements

NFR v1.8 NFR-AI-2, NFR-AI-3, NFR-AI-4, NFR-AI-5, NFR-AI-7, NFR-AI-9; NFR-SEC-15; D6 Section 6 (attacks 1 to 15); PRD E2E-54.

## 3. What to build

| # | Task | Owner | Blocked by |
|---|---|---|---|
| R1 | `scripts/ai-eval/questions.tsv`: at least 100 questions in the mix of D6 Section 3 / 50 (about 40 Roman Urdu, 35 Urdu letters, 25 English; at least 15 misspelled; at least 8 groups of 4 that must reach the same figures), each with expected tools, filters and figures, on the worked day and the E2E-59 data | robustness | 48 T3 |
| R2 | `scripts/ai-eval/run.mjs`: runs the list through the app's own `assistant_ask` path on a seeded database; prints score, every miss, tool calls, model calls, tokens (input, cached, output, reasoning; cached of each second call), bad parallel inputs and time; writes `scripts/ai-eval/urdu-review.tsv`. Run only by the lead with Umar's key; never in a commit test. Its syntax is checked (`node --check`) | robustness | 48 T3 |
| R3 | `backend/crates/assistant/tests/robustness_assistant.rs`: attacks 1 to 15 with recorded transcripts (injection in question and data, typed figures in every form, tool misuse, broken replies, network errors 401 402 404 429 500 and timeouts, the limit of 20, privacy scan E2E-54, the key scan, read-only row counts, one snapshot, estimates at the edges, advice that goes too far) | robustness | 48 T3 |
| R4 | `backend/crates/assistant/benches/`: a quick question under 300 ms; the app's part of one round (tools and checks) under 300 ms on the NFR-SCL-1 data; added to `pnpm perf`, but only the assistant benches run in this run | robustness | 48 T3 |

## 4. Rules

- The key is never written anywhere (file, test, fixture, log, message, commit). The runner reads it only from the app (Credential Manager), never from an argument or environment variable printed anywhere.
- Targeted checks only: `cargo test -p mandi-assistant --test robustness_assistant`, the assistant benches, fmt, `clippy -p mandi-assistant`. No whole-suite run, no whole `pnpm perf`.
- Builders never run git add, mv, rm, stash, checkout or commit.

## 5. Acceptance

D6-A6, D6-A7, D6-A8 and D6-A9 evidence (attacks), D6-A16 (live run, or WAITING FOR KEY with the recorded results), D6-A17 (benchmarks; live timings with the key).

## 6. As built

(The lead fills this in.)
- Robustness HANDOFF (02-11-2026), accepted:
  - R1: questions.tsv has 100 rows: Roman Urdu 40, Urdu 35, English 25; 17 misspelled; 8 groups of 4.
  - R2: run.mjs passes `node --check`. It is not run: WAITING FOR KEY.
  - R3:
    - robustness_assistant: 48 passed, 1 ignored. The ignored one is the 45 s test; run it with `-- --ignored attack_06_a_question_past_45_seconds`.
    - robustness_live: 1 passed, 1 ignored. The ignored one is the live run.
    - New app/tests/robustness_assistant_key.rs: 2 passed. It covers attack 9 and D6-A7 through the real commands, including backups and the support file.
    - Attack 13 vitest: 2 passed.
  - R4: benches/assistant.rs. scripts/perf/thresholds.json has 67 entries, with dashboard::get and R34 at 1 s.
  - Checks: rustfmt and clippy clean on its files.
- Findings:
  - (1) MAJOR: report.rs read_all ran R20 once per page of 1,000, which took about 64 s on NFR-SCL-1 data, so the advice question timed out. Sent to backend as P48-S2.
  - (2) The four chips over 300 ms, dashboard::get (8.1 s) and R34 (11.1 s) go to Umar's speed plan.
  - (3) D6 attack 15 versus E98, on Umar's list.
  - (4) "Delete", "back-date" and "dishonest" advice cannot be caught by check.rs; it is left to the Urdu review of the live run.
