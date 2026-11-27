# Sources of the transliteration word list

`translit_seed.tsv` holds the pairs (English spelling, Urdu spelling) used to turn English
letters into Urdu and back (PRD FR-7.27, Tech Stack 4.4). Every row has a `source` column and
every row waits for the Urdu reviewer (`review = pending`, NFR-URD-4) until it is marked
`reviewed`. Licence checks: `docs/research/translit-options.md` Section 2.

| `source` | Rows | Known? | What | Licence | Duty |
|---|---|---|---|---|---|
| `team` | 4,588 | yes | Written by the team for this project: Pakistani first names, family and firm names, cities and towns, fruits, vegetables, packings, the glossary words of PRD Section 3, the shop-expense words of the PRD examples and the Urdu particles | Ours | None |
| `wikidata` | 3,312 | yes | English and Urdu labels of Wikidata items written by people: people of Pakistan, Pakistani given names and settlements of Pakistan. Labels are cut into words and paired by place | CC0 1.0 | None |
| `geonames` | 133 | yes | Populated places and administrative areas of Pakistan that have an Urdu alternate name. English side: the GeoNames `name`. Urdu side: the `ur` alternate name | CC BY 4.0 | The attribution below |
| `geonames,wikidata` | 179 | yes | A pair found in both | CC BY 4.0 | The attribution below |

Plan 99 (27-11-2026): the 4,730 rows with source `engine` (words of place names whose Urdu only
our own spelling rules had proposed) no longer ship. Many were wrong (ghar غار, aya عیہ,
kar کر as a place) and they outranked the spelling rules. They are in
`translit_engine_places.tsv` (not built into the app) for a reviewer; a row the reviewer
confirms moves back with `review = reviewed` (the English side is from GeoNames, CC BY 4.0,
credited below). Wrong rows of the other sources were also removed (hai حئی, darkhast درخاست,
sheher شیر, kirae کرایہ, hun ھن) and pakka now gives پکا.

## The everyday dictionary (`translit_everyday.tsv`)

| `source` | Rows | Words | What | Licence | Duty |
|---|---|---|---|---|---|
| `everyday` | about 69,900 | about 16,400 | Common Urdu words with the Roman spellings people type: verbs in every form, pronouns, numbers, days, relations, home, food, health, travel, money, shop and mandi words, English loanwords and common phrases (plan 99, PRD FR-7.27, NFR-URD-30) | Ours | None |

Every row was written by one agent from the lemma files in `everyday/src/` and checked by a
second, independent checker agent against standard dictionary spelling (Urdu Lughat,
Feroz-ul-Lughat). Rows of checked batches have `review = checked` (`everyday/checked.txt`);
Umar's Urdu reviewer has not read them yet (NFR-URD-4). Build:
`python3 data/import/build_everyday.py` from `backend/crates/core`.

**Coverage check only:** to find common words that were missing, the word frequency list of the
Leipzig Corpora Collection corpus `urd_newscrawl_2016_1M` was read (Leipzig terms of use:
downloadable corpora are CC BY 4.0). No row, spelling or count of it is in this repository or
the app. D. Goldhahn, T. Eckart, U. Quasthoff: Building Large Monolingual Dictionaries at the
Leipzig Corpora Collection: From 100 to 200 Languages. LREC 2012.

The list now has 8,209 name list entries and the everyday words, so it meets the 15,000 known
entries of FR-7.27 (`the_list_has_fifteen_thousand_known_entries`).

Attribution (CC BY 4.0): **This product contains data from GeoNames (https://www.geonames.org),
licensed under CC BY 4.0.** The same line must be shown in the app's About screen and stays in
this file.

## Not used

Dakshina (CC BY-SA 4.0) and OpenStreetMap (ODbL) are share-alike and are not in the list.
Aksharantar rows copied from Dakshina are not in the list. The Wikidata people-of-India set was
tried and taken out again: it filled the list with names the shop will never see and made
unreviewed spellings count as known.

## How the list was made

`data/import/README.md` has the steps. In short: `prepare_pairs.py` turns the downloads into
pairs and place words, `cargo run -p mandi-core --release --example translit_import` cuts the
pairs into words, leaves out the hold-out items, lines the pairs up with the spelling rules,
proposes the Urdu of the place words, and writes the list.

- Every Wikidata item whose number ends in 0 is kept OUT of the list and becomes a test name in
  `tests/translit_holdout.tsv` (research 5.2, T2).
- Pairs that the spelling model cannot line up are dropped.
- The first row of an English word keeps weight 100; other spellings get a lower weight, and
  for a word the team list already had they never go above 50, so the team's first choice stays
  first.
- The frozen test list `tests/translit_names.tsv` was never read by the import.

Counts are printed by the test `the_known_entries_are_reported_and_proposed_ones_stay_out_of_them`
(`-- --nocapture`) and the accuracy report is `translit_accuracy.txt` in the build folder.

## Review needed (NFR-URD-4)

All name list rows are `pending`; the everyday rows are `checked` by a second agent. Nothing
was read yet by a person who reads Urdu.
