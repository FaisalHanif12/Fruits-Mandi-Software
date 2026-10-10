# Build status

Kept by `tech-lead`. Updated on each feature branch before its pull request.

## Current

- Deliverable: D1 (foundation).
- Plan file in progress: 01-scaffold, branch `feature/scaffold-repo`.

## Plan files

| Plan | Branch | Pull requests | reviewer | robustness | State |
|---|---|---|---|---|---|
| 01-scaffold | feature/scaffold-repo | (pending) | PASS WITH MINOR | PASS | gate passed; waiting for CI |
| 02-core | | | | | waiting |
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

- 01-M1: Paisa fields read through Object.values on an interface are not followed by the money lint rule. Do in plan 02.
- 01-M2: `background` and `mask` shorthands with left/right pass Stylelint. Do in plan 02.

## Open questions for Umar

- D1-Q1: screenshot comparison tool. Recommended: `@wdio/visual-service`. Plain screenshots until answered.
