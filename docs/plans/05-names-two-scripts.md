# Plan 05: Typing names in Urdu and English

Deliverable: D2, plan file 05 (`docs/deliverables/D2-accounts-names-rules.md`). Branch: `feature/names-two-scripts`.

## 1. Goal

A name is typed once and the other script fills by itself. `tariq mehmood` gives طارق محمود and "Tariq Mehmood". English letters become Urdu with up to three suggestions; Urdu letters give an English name; the app learns the corrections; and the user can also type Urdu with a built-in phonetic keyboard and an on-screen key map, with no Windows setting. Everything works offline.

## 2. Requirements

PRD 7.4 (FR-7.21 to FR-7.28, FR-7.66 v1.2), FR-7.23 (v1.2), FR-7.25, FR-7.29 later (D3), Settings "Language" (learned words list). NFR-URD-17, NFR-URD-20, NFR-URD-24 (v1.2) to NFR-URD-26, NFR-URD-29 (v1.2), NFR-SPD-20, NFR-URD-4. Architecture 19. Tech Stack 4.4. Code Standards 5.5 (B26, B28), 6.10 (F41). Acceptance: D2-A1 to D2-A4 and D2-A17. Research input: `docs/research/translit-options.md` (when it lands).

## 3. Worked examples (tests)

| Typed | Result |
|---|---|
| `tariq mehmood`, Urdu field | first suggestion طارق محمود; English name "Tariq Mehmood" |
| `arshad multani` | ارشد ملتانی; "Arshad Multani" |
| `shahid` | شاہد first, شاھد second (several spellings by weight) |
| ارشد ملتانی typed in Urdu | English suggestion "Arshad Multani" |
| `arshd` in a picker | no conversion (pickers never convert); search finds ارشد |
| a learned pair `xyz` -> زیڈ | offered first from the very next word, and after a restart |
| Urdu name field given Latin letters | `ERR_NAME_UR_SCRIPT` on the field; English field given Urdu letters: `ERR_NAME_EN_SCRIPT` |

## 4. Contract

### Backend

- `core::translit` (pure, in `backend/crates/core/src/translit/`): `suggest_to_urdu(word, learned, list) -> Vec<String>` (at most 3, learned first, then the word list by weight, then spelling rules with letter-pair scoring, duplicates removed); `suggest_to_english(urdu_word, learned, list) -> Vec<String>`; phrases are converted word by word. It never panics on any input and never changes a saved name (B28). Deterministic for the same inputs.
- Spelling rules from Tech Stack 4.4: longest match on letter groups (`kh`, `gh`, `sh`, `ch`, `th`, `ph`, `aa`, `ee`, `oo`, final `a`); Urdu to English rules (ش sh, خ kh ...) then capitals on first letters.
- `core/data/translit_seed.tsv` (columns roman, urdu, kind, weight, plus review and source columns; kinds name, place, item, word), embedded at build time. At least 15,000 entries (PRD FR-7.27 v1.2, NFR-URD-24 v1.2), each marked `review: pending` for Umar's reviewer and credited to its source (an open list is used only after its licence is checked and written in `backend/crates/core/data/SOURCES.md`). Entries: Pakistani first names, family and firm names, cities and towns of Punjab and Khyber Pakhtunkhwa, fruits, vegetables, packings, the glossary words. Each entry carries a `review` column value `pending` for Umar's Urdu reviewer (NFR-URD-4).
- `core/tests/translit_names.tsv`: the fixed test list of 1,000 names typical of the mandi (people, firms, cities, items), each with its correct Urdu spelling and the accepted English spelling. It must NOT be a copy of the seed list: at least 40% of its names contain a word that is not in the seed list, so the spelling rules are exercised. The accuracy test reports top-1 and top-3 for (a) the whole list and (b) only the names whose words are all outside the seed. Gates (NFR-URD-24 v1.2): for names whose words are all in the word list, top-1 at least 95% and top-3 at least 99%; for names with a word outside it (rules only), the top-3 rate is reported on every build and must not drop below the value stored in `core/tests/translit_baseline.txt` (the build fails if it does). NFR-URD-25: Urdu to English accepted at least 90%. Report all three views honestly; never tune the rules or the list to the frozen test list, and keep the test names out of the word list (a test checks there is no overlap).
- Every suggestion carries its source: `Suggestion { text, source: "learned" | "word_list" | "rules" }` (Tech Stack 4.4 v1.2). Phrases carry the source per word.
- Spelling check before save (PRD FR-7.66, E49): every service that saves a named record takes, for each name, `confirmed_rules_words: Vec<String>` from the screen; if the saved name contains a word whose best source is `rules` and it is not in that list, the save is refused with `ERR_SPELLING_UNCONFIRMED` on that name field, with `values.words`. Confirming learns the pair (FR-7.26). Free text is not checked. This rule is in `services` (one helper used by every named-record save), tested with a rules-only word refused, then accepted after confirmation, and learned.
- `services/translit`: learned pairs table `translit_pairs` (already in migration 0001) loaded into memory at start-up and updated on each learn. `learn(roman, urdu, kind)`: insert or `use_count + 1`, same transaction, audit row; survives a restart (test with a reopened database) and a crash right after learning (the transaction committed). Learning never changes a saved name.
- Commands (async, role first, thin): `translit_suggest { text, direction: "to_urdu" | "to_english" }` (signed-in Owner); `translit_learn { roman, urdu, kind }` -> (); `translit_pairs_list` -> rows with id, roman, urdu, kind, use_count, last_used_at; `translit_pairs_remove { id }` (audited); every command checks only that the Owner is signed in and the app is not locked (PRD v1.2). Errors: `ERR_NOT_FOUND`, plus the script codes. Speed: NFR-SPD-20, under 50 ms per key press (benchmark in `backend/crates/core/benches/`, ask robustness later; for now a timing test with a generous bound).
- `validate::name_ur` and `name_en` already exist in core; `NamePair` calls the backend for the script check on save, never relies on the frontend alone.

### Frontend

- `RomanUrduInput` (`shared/ui` or `shared/urdu`): the three typing modes of FR-7.21 switched with `Alt+U` and shown as a badge inside the field (رومن, اردو, EN). Mode رومن سے اردو (default in Urdu text fields): the current word goes to `translit_suggest` on each key press (debounced to the key rate, previous request cancelled), up to 3 suggestions under the field, the first highlighted; `Space`, `Enter` or leaving the field takes the highlighted one; `Space` stays in the field, `Enter` moves on; up and down move the highlight; `Esc` closes the list and keeps the English letters; `Backspace` right after a conversion brings back the English letters of that word. Mode اردو کی بورڈ: the phonetic keyboard in `shared/urdu/phonetic_keyboard.ts` on the CRULP layout (a to ا, b to ب, p to پ, t to ت, T to ٹ, h to ہ, H to ھ and the rest), with an on-screen key map that opens with one click and shows the key for each Urdu letter. Mode انگریزی: no conversion. Windows Urdu keyboard letters pass through unchanged in every mode. Only Urdu text fields convert; code, number and phone fields and every picker never convert (F41).
- `NamePair`: Urdu name and English name side by side. Typing one fills the other by itself (English name: exactly what was typed with capital first letters; from Urdu: the suggestion); once the user edits the filled one it is never overwritten again; in the English app the roles swap. Field errors from the backend (`ERR_NAME_UR_SCRIPT`, `ERR_NAME_EN_SCRIPT`) show under the right field. When the user picks another suggestion or corrects a word, `translit_learn` is called.
- Spelling check (FR-7.66): a converted word whose chosen suggestion has source `rules` is marked in amber with "ہجے دیکھ لیں" (Fluent) inside `RomanUrduInput` and `NamePair`; on save, if a name still has such a word, a confirm dialog shows the name with the word marked and two actions, confirm (learns the pair and sends it in `confirmed_rules_words`) or fix (back to the field). The same applies to an English name suggested from Urdu by rules only. `ERR_SPELLING_UNCONFIRMED` from the backend shows the same dialog.
- NFR-URD-29: every Urdu message in this plan is written to say exactly what its English message says (same meaning, numbers and glossary word), and each pair is marked `# review: pending` with the English beside it for the reviewer.
- The learned words list in Settings (a new tab "Language and text" gets a section): table of pairs with remove (Owner only), four states.
- Design: boards for the account form with NamePair, the suggestion list and the on-screen keyboard are drawn by the lead (see `docs/design/canvas/NamesUr.dc.html` and `NamesEn.dc.html` when they exist; until then use the design system and the FormUr/En board).
- Tests: every key of the RomanUrduInput table in Urdu and English apps; a language switch in the middle of typing keeps the text; keyboard-only run; the `tariq mehmood` flow end to end against the real commands in the dev shell.

## 5. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Contract types and command stubs, `pnpm gen:types`, message frontend | backend |
| T2 | core::translit, word list, test list, services, commands, tests, accuracy report | backend |
| T3 | RomanUrduInput, phonetic keyboard, on-screen keyboard, NamePair, learned words list | frontend |
| T4 | Wire to the real commands, end-to-end flow | frontend |

No per-plan review (Umar, 11-10-2026); reviewer and robustness run once at the end of D2 and cover this plan.

## 6. Acceptance checklist

1. `tariq mehmood` gives طارق محمود and "Tariq Mehmood" (backend test and a screen test) (D2-A1).
2. Accuracy meets NFR-URD-24 and NFR-URD-25 on the fixed 1,000-name list, with the subset report (D2-A2).
3. A learned word works from the next word and survives a restart (D2-A3).
4. All three typing modes work with no Windows setting (D2-A4).
5. A rules-only spelling is marked and not saved until confirmed or fixed; confirming learns the pair (D2-A17). The word list has at least 15,000 entries (D2-A2).
6. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check` exit 0.

## 7. Out of scope

Pickers and search (06), accounts screens (07), the Phase 2 import.

## 8. Open questions

None. The seed list's Urdu spellings are reviewed by Umar's Urdu reviewer later; they are marked `pending`.
