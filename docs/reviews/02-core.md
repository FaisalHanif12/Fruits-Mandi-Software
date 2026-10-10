# Reviews: 02-core

## reviewer: PASS WITH MINOR (10-10-2026)

Every acceptance item of plan 02 Section 6 was met with real output:

- Every Section 3 row is a named test (`money_worked_numbers`, `words_worked_numbers`, `validate_fields`). The split example 3,948 gives 1,580 / 1,184 / 1,184 (D1-A6). The amount 1 gives "ایک روپیہ" and "One rupee only" (D1-A7).
- Property tests run 10,000 cases each; a test pins the number.
- `cargo llvm-cov -p mandi-core`: 100% lines on `money/mod.rs`, `parse.rs`, `split.rs`, `format.rs`; `text.rs` 99.60%, `validate.rs` 99.51%.
- No `f32` or `f64` and no ` as ` cast in `core/src`; unwrap and expect only in tests.
- Urdu look-alike letters match; the ICU4X `ur` collator sorts a shuffled 35-word list correctly (D1-A8).
- The `money-calculations` agent gave PASS on the current files, including a scratch check of 5,380,839 strings against an independent rule set (0 mismatches).
- Frontend: `formatPaisa`, 25 error codes with ur and en messages, tokens against `docs/design/design-system.md`, the gallery in a real browser at 1366 and 1024 wide (no sideways scroll), and dev-only code absent from the production bundle.

Three MINOR findings, all closed before the commit:

1. Plan text behind the code on the stripped marks (word joiner) and the phone rule. Fixed in `docs/plans/02-core.md`.
2. Comma grouping too loose: `0,500` read as 500, and weight `12,345` read as 12,345 kg. Fixed: a group may not start with 0; weight and percent take no commas. Money agent PASS on the change.
3. `err-control-chars` named only a tab and a line break. Reworded in both languages (Urdu line on Umar's review list).

Decisions made during the review (all in the plan 02 contract, Sections 4.6 and 4.7): only zero-width marks and LRM are stripped from number text; RLM, ALM and every embedding, override and isolate are refused (Trojan Source); Arabic-Indic digits are refused in a sum and in a phone with a hyphen; a typed comma is sent exactly as typed; error values are display units, never paisa.

Not verifiable yet: Windows WebView2 rendering (Urdu in a 44 px field sat slightly high in Mac Chrome; check with design-system item 8.4).

## robustness: PASS (final, 10-10-2026)

First verdict FAIL (1 MAJOR: R02-6, the frontend dropped typed commas so "850,50" became 85,050 rupees; 5 MINOR). Then PASS WITH MINOR (R02-7), then PASS after every finding closed.

- R02-1 word joiner in numbers; R02-2 bidi controls in search keys; R02-3 bidi controls in free text; R02-4 unstable search keys; R02-5 empty optional fields with only invisible marks; R02-6 typed commas; R02-7 account code with a pasted zero-width mark. All fixed, and their tests now run as normal tests.
- 58 robustness tests in `backend/crates/core/tests/robustness/`, 93 frontend robustness tests, and 9 new criterion benches. Every speed budget is far under its limit (for example the money maths of a 500-line preview takes 0.3 ms against a 10 ms budget).
- Money at 0, 1 rupee, the maximum and the `i64` limits: every result exact or an overflow error; splits always add back exactly with 10, 10,000 partners and 10,000 lots.
- ESLint and tsc both fail on money maths over a backend money field.
