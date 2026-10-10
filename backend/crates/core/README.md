# mandi-core

Pure helpers with no business rules and no database (Tech Stack 5.4). Plan 02.

| Module | What it holds |
|---|---|
| `money` | `Paisa`, `Bps`, `Qty`, `Grams`; `line_value`, `per_unit`, `apply_bps`, `weight_value`, `avg_rate`; the one rounding (`round_to_rupee`, private); `split_by_bps`, `split_by_qty`; `parse_rupees`, `parse_bps`, `parse_weight`, `parse_qty`, `parse_sum_rupees`, `parse_sum_qty`; `format_paisa` |
| `dates` | `BusinessDate`, `parse_ddmmyyyy` (with the FR-7.19 shortcuts), `format_ddmmyyyy`, `check_posting_date` (FR-6.6) |
| `text` | `normalize_urdu`, `search_ur`, `search_en`, `search_phonetic`, `UrduCollator` (ICU4X, `ur`), script checks |
| `urdu_words`, `english_words` | Amounts in words, 0 to 10,00,00,00,000 rupees |
| `validate` | One function per field type of Code Standards 4.2, returning `FieldError`; `FieldErrors` collects several |
| `error` | `ErrorCode` (the one list, exported to TypeScript), `ErrorValue`, `FieldError` |

Example: `apply_bps(Paisa::from_rupees(12_000)?, Bps::new(600)?)` is 720 rupees.

- May depend on: no crate of ours. The app reaches it through `mandi_services::core`.
- Never: database, files, network, the clock. "Today" is always a parameter.
- `clippy::arithmetic_side_effects` is denied for the whole crate (Code Standards B11). Use checked maths.
- No floats and no `as` casts. Widening uses `i128::from`, narrowing uses `try_from`.
- Invisible marks have one list, in `text` (`is_zero_width`, `is_direction_mark`, `is_bidi_control`). Number, code and phone text drops the zero-width marks and LRM (`is_ignorable_in_number`) and refuses RLM, ALM and the bidi controls. Search keys drop all of them. Free text refuses the bidi controls (Trojan Source).
- Search keys are stable: `search_ur(search_ur(x)) == search_ur(x)`, and the same for `search_en`. Marks are removed and text is lower-cased before the last NFC pass. `tests/core_properties.rs` checks every Unicode character and every base letter with two marks.
- Tests: worked numbers in `tests/money_worked_numbers.rs` and `tests/words_worked_numbers.rs`, property tests (10,000 cases each) in `tests/core_properties.rs`.
