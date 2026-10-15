# mandi-core

Pure helpers with no business rules and no database (Tech Stack 5.4). Plan 02.

| Module | What it holds |
|---|---|
| `money` | `Paisa`, `Bps`, `Qty`, `Grams`; `line_value`, `per_unit`, `apply_bps`, `weight_value`, `avg_rate`; the one rounding (`round_to_rupee`, private); `split_by_bps`, `split_by_qty`; `parse_rupees`, `parse_bps`, `parse_weight`, `parse_qty`, `parse_sum_rupees`, `parse_sum_qty`; `format_paisa` |
| `dates` | `BusinessDate`, `parse_ddmmyyyy` (with the FR-7.19 shortcuts), `format_ddmmyyyy`, `check_posting_date` (FR-6.6) |
| `text` | `normalize_urdu`, `search_ur`, `search_en`, `search_phonetic`, `UrduCollator` (ICU4X, `ur`), script checks |
| `translit` | English letters to Urdu and back for names (PRD 7.4): `suggest_to_urdu`, `suggest_to_english`, `WordList` (the embedded `data/translit_seed.tsv`), `LearnedPair`. Pure: the services pass learned pairs in. |
| `urdu_words`, `english_words` | Amounts in words, 0 to 10,00,00,00,000 rupees |
| `validate` | One function per field type of Code Standards 4.2, returning `FieldError`; `FieldErrors` collects several |
| `error` | `ErrorCode` (the one list, exported to TypeScript), `ErrorValue`, `FieldError` |

Example: `apply_bps(Paisa::from_rupees(12_000)?, Bps::new(600)?)` is 720 rupees.

- May depend on: no crate of ours. The app reaches it through `mandi_services::core`.
- Never: database, files, network, the clock. "Today" is always a parameter.
- `clippy::arithmetic_side_effects` is denied for the whole crate (Code Standards B11). Use checked maths.
- No floats and no `as` casts. Widening uses `i128::from`, narrowing uses `try_from`.
  - **The one exception is `translit`.** Its spelling scoring (letter-pair chances, ranking) uses `f64`. It never touches money: it does not import `money` or use `Paisa`, `Bps`, `Qty` or `Grams`, and it only orders suggestions the user then looks at. `tests/no_floats_in_money.rs` keeps it that way: `money`, `validate`, `dates`, `text`, `urdu_words` and `english_words` have no `f32` or `f64`, no file outside `translit` has one, and `translit` has no money type.
- Invisible marks have one list, in `text` (`is_zero_width`, `is_direction_mark`, `is_bidi_control`). Number, code and phone text drops the zero-width marks and LRM (`is_ignorable_in_number`) and refuses RLM, ALM and the bidi controls. Search keys drop all of them. Free text refuses the bidi controls (Trojan Source).
- Search keys are stable: `search_ur(search_ur(x)) == search_ur(x)`, and the same for `search_en`. Marks are removed and text is lower-cased before the last NFC pass. `tests/core_properties.rs` checks every Unicode character and every base letter with two marks.
- Tests: worked numbers in `tests/money_worked_numbers.rs` and `tests/words_worked_numbers.rs`, property tests (10,000 cases each) in `tests/core_properties.rs`.

## Names in two scripts (`translit`, plan 05)

- Up to 3 suggestions, in this order: learned pairs (most used first), the word list (by weight), then the spelling rules. Phrases go word by word; a phrase that is a whole entry of the list is taken as one unit. Digits, punctuation and other scripts stay as typed. Any text is fine and never panics (Code Standards B28); only the first 500 characters are read.
- The rules are trained from the single-word pairs of the list when first used (about 30 ms in a release build): each English letter group is lined up with its Urdu letters, and the counts give the chance of each spelling by place in the word. Words made of known parts (`abdulsamad`, `fazlurrehman`, `kanganpur`) are put together from the list. A rule spelling that is also a word of the list is ranked up.
- Everything is counted in a fixed order with plus, times and divide only, so the same list gives the same answers on every computer.
- `data/translit_seed.tsv`: columns `roman`, `urdu`, `kind`, `weight`, `review`. Every entry is `pending` until the Urdu reviewer marks it `reviewed` (NFR-URD-4). The first Urdu spelling of a name is the usual one. The English spelling listed first for an Urdu name is the one offered for Urdu to English.
- Lists in `tests/`: `translit_names.tsv` is the frozen 1,000-name list of NFR-URD-24 and NFR-URD-25. It was composed from hand-written word pools kept apart from the seed, and about half of its names hold a word the seed lacks. **Never tune the rules to it and never change it.** The rules are tuned on `translit_dev.tsv`. Adding words to the seed after the list was frozen would make the figures look better than they are, so the seed and the frozen list move together only through a new numbered list.
- `tests/translit_accuracy.rs` prints and saves the report (`cargo test -p mandi-core --test translit_accuracy -- --nocapture`). The NFR gates are in an ignored test that shows the real figures.
