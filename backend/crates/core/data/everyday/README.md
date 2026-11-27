# Everyday words (plan 99 U1a, PRD FR-7.27, NFR-URD-30)

The checked everyday dictionary: common Urdu words, each with the Roman spellings people type.

- `src/*.lex` are the lemma files, written by hand. Edit these, never the `.tsv` files.
- `*.tsv` are the built batches (columns: `roman`, `urdu`, `kind`, `weight`, `review`,
  `source`). Build them with `python3 data/import/build_everyday.py` from `backend/crates/core`.
- `passed_rows.tsv` lists every row an independent checker has passed, one by one
  (`roman<TAB>urdu`). Only these get `review = checked`; every other row is `pending`, so a new
  lemma or a changed generator rule never inherits an old pass. After a checker pass:
  `python3 data/import/build_everyday.py --pass <rows.tsv>` (first two columns roman, urdu).

## Checker notes

The checker reads each `.tsv` row (or the `.lex` lemma it came from) and reports every wrong
Urdu spelling (standard dictionary spelling: Urdu Lughat, Feroz-ul-Lughat) and every roman
spelling nobody would type for that word, as `file:line problem -> fix`.

## Progress (27-11-2026)

Every row has passed a check (urdu-checker, urdu-checker-2, urdu-checker-3), row by row, in
six rounds and a last joint re-check of rounds 5 and 6: 69,942 rows, 16,417 Urdu words,
64,278 spellings. `passed_rows.tsv` holds each passed row.

`english_stems.txt` is a reviewed list of English loanword spellings: no Urdu forms are built
on them (no `bookingein`), and the engine reads the same list to make the plural when one is
typed (`doctoron` is ڈاکٹروں). `f` after a word marks it feminine, so `wicketein` is وکٹیں.

Each new roman that two words share within 10 weight is listed by
`build_everyday.py --check --sweep`; the ones looked at are in `reviewed_ties.txt`.
