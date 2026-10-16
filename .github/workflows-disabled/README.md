# Workflows turned off

GitHub Actions cannot run on this account at the moment (the account is locked for billing), so the three workflows are kept here, where GitHub does not run them.

To turn them on again, move the three files back to `.github/workflows/` and add the required checks `Checks (Windows)` and `Windows E2E and speed` back to the `main` ruleset (`.claude/rules/Release_Pipeline.md` 3.4).

Until then, every check runs locally before a merge: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, and in `backend/` `cargo deny check`.
