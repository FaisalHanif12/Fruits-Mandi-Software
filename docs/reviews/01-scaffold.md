# Reviews: 01-scaffold

## reviewer: PASS WITH MINOR (recheck 3, 10-10-2026)

Rounds: first review FAIL (3 MAJOR, 3 MINOR), recheck 1 FAIL (1 MAJOR, 3 MINOR), recheck 2 FAIL (1 MAJOR, 1 MINOR), recheck 3 PASS WITH MINOR.

Fixed during the review:

1. Inline disable comments switched off lint rules. ESLint now has `noInlineConfig` and `reportUnusedDisableDirectives`; Stylelint has `ignoreDisables`.
2. `#[allow(clippy::unwrap_used)]` switched off CLAUDE.md rule 5. The ten lints are now `forbid`.
3. Rules 7 and 8 only saw literals written directly in JSX. They now check every string, object literal, both branches of conditionals, JSX attributes outside an allow list, and lookups in const arrays and objects.
4. The updater capability allowed download and install. It is now `updater:allow-check` only.
5. `i18n:check` could exit 0 silently. The CLI now always runs and prints a result line.
6. Tokens and Button did not follow `docs/design/design-system.md`. They now do.
7. Paisa could be widened to `number`. Paisa is now an opaque type; TypeScript rejects maths and widening.
8. Physical CSS shorthands passed Stylelint. A custom rule rejects them.
9. Words in arrays, objects, Fluent variables and `document.title` passed rule 7. Now caught.
10. The unsafe guard test did not call its helper. The scan now catches every allow, expect, warn and cfg_attr form.
11. Paisa escaped through `map(Number)` and `new Number(a)`. The rule now follows Paisa into call arguments and callbacks.
12. `left` and `right` position keywords passed. Now rejected in CSS and in style objects.

Regression on the final round: `pnpm lint`, `pnpm typecheck`, `pnpm test` (Vitest 266 passed, all Rust tests ok), `pnpm i18n:check`, `pnpm audit --audit-level moderate` (one dev-only advisory ignored with its reason), `pnpm gen:types` (no diff), `cargo deny check`: all exit 0.

Open MINOR follow-ups (to plan 02):

- M1. `Object.values(bill).map(Number)` on an interface with Paisa fields is not followed (Object.values returns `any[]`). Either report Object.values, Object.entries and JSON.stringify on object types with a Paisa property, or write this case in the scope note next to the rule.
- M2. The `background` and `mask` shorthands with `left` or `right` pass Stylelint. Apply the keyword check to them.

Not verifiable yet: checklist item 9 (Windows CI), because GitHub Actions is blocked by the account billing lock.

## robustness: PASS (recheck, 10-10-2026)

First verdict FAIL (1 MAJOR: `pnpm audit` red; 4 MINOR). All fixed:

1. `pnpm audit --audit-level moderate` exit 0, with GHSA-vfj7-8cjw-p6xm (braces, dev-only through Stylelint, no patched version) ignored with its reason.
2. Paisa lint bypasses closed.
3. Bindings type money as `Paisa` (plan 02 contract; `gen:types` adds the import).
4. `frontend/tests/robustness/` is in the TypeScript and Vitest include lists; the guard test for CSP, capabilities and plugin order runs in `pnpm test` (10 tests; each check proven to fail on a mutated copy).
5. `windows.yml` runs `pnpm perf --require-install` and uploads `backend/target/perf/report.json`.

Runner factor: 1.0 (PassMark single-thread: runner EPYC 7763 2,517 vs reference i3-7100 2,268). Method and threshold table in `scripts/perf/README.md`.

Evidence still due from the first Windows run: the install time against NFR-INS-1 (D1-A3) and the runner CPU model printed by `pnpm perf`.
