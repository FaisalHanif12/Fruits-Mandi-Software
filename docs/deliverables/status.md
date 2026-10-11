# Build status

Kept by `tech-lead`. Updated on each feature branch before its pull request.

## Current

- Deliverable: D1 (foundation).
- Plan file in progress: 03-database, branch `feature/db-schema` (stacked on `feature/core-money`, PR #2). 02-core is committed on `feature/core-money`, stacked on `feature/scaffold-repo` (PR #1 waits for Umar to merge it; GitHub workflows are off).

## Plan files

**Review flow changed by Umar (11-10-2026):** plans 03 to 10 are built by `backend` and `frontend` only. `reviewer` and `robustness` run once, at the end, for the whole of D1 and D2, and cover every plan file 01 to 10. Plans 01 and 02 keep their earlier verdicts.

| Plan | Branch | Pull requests | reviewer | robustness | State |
|---|---|---|---|---|---|
| 01-scaffold | feature/scaffold-repo | #1 | PASS WITH MINOR | PASS | gate passed; waiting for CI |
| 02-core | feature/core-money | #2 | PASS WITH MINOR | PASS | gate passed; committed locally |
| 03-database | feature/db-schema | #3 | at the end | at the end | built, local checks green |
| 04-shell-and-login | feature/db-schema (groundwork) | | at the end | at the end | building: shell pieces, keyboard engine, entry grid done |

## Design

- Canvas: https://claude.ai/artifact/EtYUy8QMjbEkx7yy9aeAiF. Rules: `docs/design/design-system.md`. Board sources: `docs/design/canvas/`.

## Setup facts

- Requirement documents are in `.claude/rules/` (Umar's choice). A path `docs/<Doc>.md` in any file means `.claude/rules/<Doc>.md`.
- Deliverable files were moved from `deliverables/` to `docs/deliverables/`, as README Section 1 expects.
- Updater key: made on 10-10-2026. Private key and password are in `~/.tauri/` on Umar's Mac. Public key in `docs/release/updater-pubkey.txt`. Both GitHub secrets are set.
- `main` ruleset is active (id 24841987): pull request, squash only, required checks `Checks (Windows)` and `Windows E2E and speed`, up to date.

- `docs/deliverables/progress.json` feeds the goal-progress band above the prompt (the `goal-progress` mod). The lead updates it as work moves: a plan goes `waiting`, `building`, `reviewing`, `passed` (both verdicts PASS or PASS WITH MINOR); an acceptance id is added only with real command output; a deliverable is added to `reports` when its DONE REPORT is posted. A new goal changes `goal.key` and `goal.scope`, and its progress starts from zero.

## Blockers

- 10-10-2026: GitHub Actions does not start any job: "The job was not started because your account is locked due to a billing issue." Only Umar can fix this (github.com/settings/billing). Until then no check can pass, so nothing can merge and no release can run. Work continues locally on stacked branches.

## Follow-ups (MINOR)

- 01-M1 and 01-M2: done in plan 02.
- 02: Windows WebView2 check of the Urdu field text position (design-system 8.4).
- 02: frontend `shared/` coverage (80%, Code Standards 9.4) is not measured yet; add `@vitest/coverage-v8` when `shared/` grows.

## Urdu words waiting for Umar's review (NFR-URD-4)

- Plan 02 words in amounts (`core::urdu_words`): the 0 to 99 list, "ایک روپیہ" for exactly one rupee, "ایک سو" for 100.
- Plan 02 error messages, all `# review: pending` in `frontend/src/i18n/ur/errors.ftl` (25 codes). New or reworded: `err-not-a-number` ("یہاں نمبر لکھیں، مثلاً 12,000۔ حروف اور غلط جگہ پر لگا ہوا کاما نہیں لکھ سکتے۔": check کاما, غلط جگہ پر لگا ہوا, مثلاً), `err-qty-not-whole`, `err-negative-not-allowed`, `err-control-chars` ("اس خانے میں چھپا ہوا حرف ہے، مثلاً ٹیب، نئی سطر یا تحریر کی سمت کا نشان۔ ...": check چھپا ہوا حرف, تحریر کی سمت کا نشان).
- Plan 03 messages in `errors.ftl` and `common.ftl`: `err-disk-full`, `err-data-check-failed`, `err-migration-failed`, `err-db-busy`, and the title `common-data-check-title` ("ایپ آپ کا ڈیٹا ابھی نہیں کھول سکی").
- Plan 03 seed: provisional Urdu names of the 6 account heads and 10 controls (marked in the seed code), for example اثاثے, واجب الادا, پارٹیاں, آمدن, اخراجات, سرمایہ.
- Plan 01 glossary: 51 terms marked `# review: pending` in `glossary.ftl`.
- Dev-only gallery words (never shipped): not on the list.

## Notes carried into later plans

- Plan 07 (journal and accounts, from backend's plan 03 handoff): (a) write lot gross, deductions and net in ONE UPDATE; (b) write the posted `chain_hash` and `total_paisa` in the same UPDATE that sets status to posted; (c) Pukhta Khata defaults to cash (FR-13.4) while 12.4 follows credit_allowed, so the service must special-case it; (d) the domain checks computed values against the 1e14 paisa cap and returns ERR_AMOUNT_TOO_LARGE, not a raw CHECK failure; (e) the reversal date rule (same date) is a service rule; the database only checks the mirror and the total.
- Known audit limit: removing the LAST audit rows keeps the chain valid. Anchoring the tail hash outside the file belongs to the backup plan (NFR-SEC-6, D5).
- Money agent leftovers for plans 11 to 17: vouchers keep money columns editable (O-7), person key against the Pukhta account (B-6), a C6 check on lot deductions.
- `UiError.ref` is always present; the frontend does not show it for field errors (plan 03 4.2c).

## Open questions for Umar

- Architecture 15.6 gives `bikri_lots` only an amount and a rule id for labour, munshiana and phone. PRD FR-11.6 says the method, value, rule and amount are saved on each lot too. The PRD wins, so migration 0001 now has `labour_method`, `labour_value_paisa` and the same for munshiana and phone. Architecture 15.6 needs a v1.2 line (requirement-change, Umar approves).

- D1-Q1: screenshot comparison tool. Recommended: `@wdio/visual-service`. Plain screenshots until answered.
