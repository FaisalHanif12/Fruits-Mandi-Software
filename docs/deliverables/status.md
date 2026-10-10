# Build status

Kept by `tech-lead`. Updated on each feature branch before its pull request.

## Current

- Deliverable: D1 (foundation).
- Plan file in progress: 03-database next. 02-core is committed on `feature/core-money`, stacked on `feature/scaffold-repo` (PR #1 waits for Umar to merge it; GitHub workflows are off).

## Plan files

| Plan | Branch | Pull requests | reviewer | robustness | State |
|---|---|---|---|---|---|
| 01-scaffold | feature/scaffold-repo | #1 | PASS WITH MINOR | PASS | gate passed; waiting for CI |
| 02-core | feature/core-money | (pending) | PASS WITH MINOR | PASS | gate passed; committed locally |
| 03-database | | | | | waiting |
| 04-shell-and-login | | | | | waiting |

## Design

- Canvas: https://claude.ai/artifact/EtYUy8QMjbEkx7yy9aeAiF. Rules: `docs/design/design-system.md`. Board sources: `docs/design/canvas/`.

## Setup facts

- Requirement documents are in `.claude/rules/` (Umar's choice). A path `docs/<Doc>.md` in any file means `.claude/rules/<Doc>.md`.
- Deliverable files were moved from `deliverables/` to `docs/deliverables/`, as README Section 1 expects.
- Updater key: made on 10-10-2026. Private key and password are in `~/.tauri/` on Umar's Mac. Public key in `docs/release/updater-pubkey.txt`. Both GitHub secrets are set.
- `main` ruleset is active (id 24841987): pull request, squash only, required checks `Checks (Windows)` and `Windows E2E and speed`, up to date.

## Blockers

- 10-10-2026: GitHub Actions does not start any job: "The job was not started because your account is locked due to a billing issue." Only Umar can fix this (github.com/settings/billing). Until then no check can pass, so nothing can merge and no release can run. Work continues locally on stacked branches.

## Follow-ups (MINOR)

- 01-M1 and 01-M2: done in plan 02.
- 02: Windows WebView2 check of the Urdu field text position (design-system 8.4).
- 02: frontend `shared/` coverage (80%, Code Standards 9.4) is not measured yet; add `@vitest/coverage-v8` when `shared/` grows.

## Urdu words waiting for Umar's review (NFR-URD-4)

- Plan 02 words in amounts (`core::urdu_words`): the 0 to 99 list, "ایک روپیہ" for exactly one rupee, "ایک سو" for 100.
- Plan 02 error messages, all `# review: pending` in `frontend/src/i18n/ur/errors.ftl` (25 codes). New or reworded: `err-not-a-number` ("یہاں نمبر لکھیں، مثلاً 12,000۔ حروف اور غلط جگہ پر لگا ہوا کاما نہیں لکھ سکتے۔": check کاما, غلط جگہ پر لگا ہوا, مثلاً), `err-qty-not-whole`, `err-negative-not-allowed`, `err-control-chars` ("اس خانے میں چھپا ہوا حرف ہے، مثلاً ٹیب، نئی سطر یا تحریر کی سمت کا نشان۔ ...": check چھپا ہوا حرف, تحریر کی سمت کا نشان).
- Plan 01 glossary: 51 terms marked `# review: pending` in `glossary.ftl`.
- Dev-only gallery words (never shipped): not on the list.

## Open questions for Umar

- D1-Q1: screenshot comparison tool. Recommended: `@wdio/visual-service`. Plain screenshots until answered.
