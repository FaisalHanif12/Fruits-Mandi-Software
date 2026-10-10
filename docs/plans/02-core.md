# Plan 02: Core (money, dates, text, words, validation) and the design system base

Deliverable: D1, plan file 02. Branch: `feature/core-money` (stacked on `feature/scaffold-repo` until 01 is merged).

## 1. Goal

Build `mandi-core`: the one place where money is parsed, rounded and split, where dates and the open-day rule live, where Urdu and English search keys are built, where amounts become words, and where every input field is checked. Prove it with the PRD numbers and with property tests. In parallel, `frontend` builds the display formatter `formatPaisa`, the error messages for every core error code, and the design system tokens and base components from `docs/design/design-system.md`.

## 2. Requirements

- PRD Section 22 (C1 to C19, rounding), FR-6.4, FR-6.6, FR-7.19, FR-7.48, FR-19.7, FR-7.28.
- Code Standards 3 (M1 to M12), 4.2 (field rules), 4.3 (error shape), 9.3 (property tests), 9.4 (coverage).
- `.claude/agents/money-calculations.md` Sections 1 to 5 and 11. Tech Stack 4.3 and 4.5.
- NFR-MNT-3, NFR-MNT-4, NFR-URD-14 to NFR-URD-16, NFR-URD-18, NFR-URD-19, NFR-URD-23.
- Acceptance items: D1-A5, D1-A6, D1-A7, D1-A8.

## 3. Worked numbers (every row is a unit test)

All rows of these tables in `.claude/agents/money-calculations.md`, copied exactly:

- 3.1 `parse_rupees`, `parse_bps`, `parse_weight`, `parse_sum` tables.
- 3.3 amounts in words: 0, 19,316, 1,00,000, 1,316,040 in Urdu, English international and English lakh; plus boundaries 99, 100, 999, 1,000, 99,999, 1,00,000, 9,99,99,999, 10,00,00,000 and the maximum 10,00,00,00,000 (دس ارب).
- 4.2 `apply_bps` table (720, 338, 338 half up, 337, 870, 25, 0).
- 4.3 `weight_value`: 12.345 kg at 100 = 1,235. Per unit 10 x 20 = 200.
- 4.4 `avg_rate`: 29,400 over 50 = 588.
- 5.1 split table: 3,948 -> 1,580 / 1,184 / 1,184 (D1-A6); 1,540 -> 616 / 462 / 462; 3,972 -> 1,590 / 1,191 / 1,191; 4,094 -> 1,638 / 1,228 / 1,228; 3,560 -> 1,424 / 1,068 / 1,068; 376 -> 152 / 112 / 112; 1,000 at 3,333/3,333/3,334 -> 333 / 333 / 334; 1,001 at 5,000/5,000 -> 501 / 500; 777 at 10,000 -> 777; 0 -> 0.
- 5.2 negatives: 3,000 -> -1,200 / -900 / -900; 3,384 -> -1,354 / -1,015 / -1,015 (E2E-20); -100 -> -40 / -30 / -30.
- 5.3 `split_by_qty`: 1,000 over 30 and 20 -> 600 / 400; 1,000 over 1/1/1 -> 334 / 333 / 333.

## 4. Contract

This plan has no Tauri command. The contract is the public Rust API of `mandi-core`, which every later crate uses, and the error codes the frontend must translate.

### 4.1 `mandi_core::money`

Exactly the types and functions of `.claude/agents/money-calculations.md` Section 2: `Paisa(i64)`, `Bps(i32)`, `Qty(i64)`, `Grams(i64)`; `checked_add`, `checked_sub`, `line_value`, `weight_value`, `apply_bps`, `per_unit`, `split_by_bps`, `split_by_qty`, `avg_rate`, private `round_to_rupee`. Plus `parse_rupees`, `parse_bps`, `parse_weight`, `parse_qty`, `parse_sum` (money and qty variants), `format_paisa(Paisa, Grouping) -> Result<String, MoneyError>`. `split_by_bps` takes a generic id (so `PartnerId` from later crates fits) and the "created first" tie rule uses the lowest id. `Paisa` serialises as a JSON integer. In the generated bindings every money field is typed `Paisa`, the opaque type from `frontend/src/shared/format/money.ts` (not a subtype of `number`, so TypeScript rejects any maths or widening outside `money.ts`; plan 01 review finding A) (`#[ts(type = "Paisa")]`), and `gen:types` adds `import type { Paisa } from '../shared/format/money'` to each binding file that uses it, so the frontend lint rule sees backend money (Code Standards F2, F3; lead decision 10-10-2026, robustness finding 01-3). At runtime it is still a JSON number, as the CLAUDE.md gotcha requires. `Grouping` is `International | Lakh`.

### 4.2 `mandi_core::dates`

`BusinessDate` (a plain date, `YYYY-MM-DD` in storage). `parse_ddmmyyyy(text, today)` accepts `DD-MM-YYYY`, and the shortcuts of FR-7.19: `10` = day 10 of today's month and year, `10-9` = 10 September of today's year. `format_ddmmyyyy`. `check_posting_date(date, closed_up_to, today)` returns `ERR_DATE_CLOSED` when date <= closed_up_to and `ERR_DATE_IN_FUTURE` when date > today (FR-6.6). `today` is always passed in; core never reads the clock (Code Standards B1).

### 4.3 `mandi_core::text`

- `normalize_urdu(s)`: NFC; ي and ى -> ی; ك -> ک; ه (Arabic heh) -> ہ; ۃ -> ہ; ھ unchanged; remove diacritics (zabar, zer, pesh, shadd, sukun, tanween, the small letters); remove zero-width characters (ZWJ, ZWNJ, ZWSP, BOM); collapse spaces; trim (Tech Stack 4.3, NFR-URD-18).
- `search_ur(name_ur)`, `search_en(name_en)` (lower case, trimmed, single spaces), `search_phonetic(name_en)` (lower case Latin letters only, vowels a e i o u removed except a first letter, doubled letters made single; "arshad" and "arshd" give the same key).
- `urdu_sort_key` or a comparator using `icu_collator` with the `ur` locale, so lists sort ا ب پ ت ٹ ... (NFR-URD-19). Test: a shuffled list sorts to the expected order.
- Script checks for FR-7.28 (`is_urdu_name`, `is_english_name`), used by `validate`.

### 4.4 `mandi_core::urdu_words`, `mandi_core::english_words`

`urdu_words(Paisa) -> Result<String, WordsError>` with ہزار، لاکھ، کروڑ، ارب; `english_words(Paisa, Grouping)`. Integer maths only. Out of range or not a multiple of 100 is an error.

### 4.5 `mandi_core::validate`

The field rules of Code Standards 4.2 as functions that return `Result<T, FieldError>`, where `FieldError { code, field, values }` matches the `UiErrorItem` shape of plan 01. Fields: money, rate (money > 0), percent, qty (with `parse_sum`), weight, date, account code (full 8 or 9 digits, or a 1 to 4 digit short code returned as `ShortCode`), Urdu name, English name, phone (`03XXXXXXXXX` or `03XX-XXXXXXX`, stored as digits), free text (500 max, no control characters). All text trimmed, NFC, zero-width removed from names. A helper collects several errors so a command can return them together.

### 4.6 Error codes (backend returns them; frontend translates them)

`ERR_REQUIRED`, `ERR_NOT_A_NUMBER`, `ERR_DECIMALS_NOT_ALLOWED`, `ERR_NEGATIVE_NOT_ALLOWED`, `ERR_AMOUNT_TOO_LARGE`, `ERR_MUST_BE_MORE_THAN_ZERO`, `ERR_TOO_MANY_DECIMALS`, `ERR_PERCENT_OVER_100`, `ERR_QTY_TOO_LARGE`, `ERR_WEIGHT_TOO_LARGE`, `ERR_DATE_INVALID`, `ERR_DATE_CLOSED`, `ERR_DATE_IN_FUTURE`, `ERR_CODE_INVALID`, `ERR_NAME_UR_SCRIPT`, `ERR_NAME_EN_SCRIPT`, `ERR_NAME_TOO_LONG`, `ERR_PHONE_INVALID`, `ERR_TEXT_TOO_LONG`, `ERR_CONTROL_CHARS`, `ERR_OVERFLOW`, `ERR_SHARES_NOT_100`. If backend needs another code, it tells the lead first.

`backend` exports the list as a ts-rs type `ErrorCode` (a string union) so `frontend` can check every code has a message.

### 4.7 Frontend contract

- `frontend/src/shared/format/money.ts`: `formatPaisa(p: Paisa, opts: { grouping: 'international' | 'lakh' }): string`. Integer `%`, `Math.trunc` and string work only; throws in development (logs in production) if `p` is not a multiple of 100 or not a safe integer. Unit tests mirror the Rust `format_paisa` cases, including 1,316,040 and 13,16,040.
- `frontend/src/shared/format/digits.ts`: Urdu digits (۰-۹) for prints only (NFR-URD-14), off by default.
- `errors.ftl` in `ur` and `en`: one message `err-<code-in-kebab>` for every code in 4.6, with values (for example the max amount). A test checks that every `ErrorCode` from the bindings has a message in both languages.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Public types and function signatures in `mandi-core` (bodies may return a not-implemented error), `ErrorCode` export, `pnpm gen:types`. Message the lead with the API. | backend | 01 done |
| T2 | Bodies and tests. Call the `money-calculations` agent before writing `money` and again before handoff; attach its final report. Property tests with at least 10,000 cases each (money agent 11.2 items 1, 2, 5, 6, 9, plus words never panic and `normalize_urdu` / search keys never panic on any Unicode). Coverage: `cargo llvm-cov` 100% lines on `core::money`, at least 95% on `text`, `urdu_words`, `english_words`. | backend | T1 |
| T3 | `formatPaisa`, digits helper, error messages for every code, the bindings check test. Then the design system: `styles/tokens.css` exactly from `docs/design/design-system.md` Sections 2 to 5, bundled font faces, and the base components in `shared/ui`: `Button` (with `KeyChip`), `Field`, `TextInput`, `MoneyInput`, `QtyInput`, `DateInput`, `CodeInput`, `Select`, `Card`, `EmptyState`, `ErrorState`, `LoadingState`, `Num`, `Code`, `Toast`, `ConfirmDialog`. Each with a component test in both languages. A development-only gallery page (`/dev/gallery`, not in the shipped build) shows every component in Urdu and English. | frontend | 01 done |
| T5 | Review | reviewer | T2, T3 |
| T6 | Robustness: the D1 attack list item 1 (extreme values, i64 limits, overflow returns an error, malformed texts `1,00,0`, Urdu digits, spaces, signs, empty) | robustness | T2 |
| T7 | Gate and merge | lead | T5, T6 |

## 6. Acceptance checklist

1. Every row in Section 3 is a passing unit test (names listed in the handoff).
2. Property tests: at least 10,000 cases each, all pass (`PROPTEST_CASES` or config shown).
3. Coverage: 100% lines on `core::money` (report pasted).
4. No float anywhere in `mandi-core` (`grep -rn "f32\|f64" backend/crates/core/src` returns nothing outside tests). No `as` casts that can truncate on money.
5. `normalize_urdu("اكرم") == normalize_urdu("اکرم")`, and every mapping of 4.3 has a test. Urdu sort order test passes.
6. Words: the four table rows in three variants and all boundaries pass.
7. `money-calculations` agent final review: PASS, attached.
8. Frontend: `formatPaisa` tests pass; every `ErrorCode` has `ur` and `en` messages; `pnpm i18n:check` passes; tokens match `design-system.md`; gallery screenshots in both languages.
9. `pnpm lint`, `pnpm typecheck`, `pnpm test` exit 0.

## 7. Out of scope

Database, commands, screens other than the dev gallery. Commission rules and journal building (D2 and D3).

## 8. Open questions

None.
