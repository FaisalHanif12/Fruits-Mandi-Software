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
| `engine` | 4,730 | **no** | Words of the names of places in Punjab and Khyber Pakhtunkhwa that occur in at least 3 GeoNames place names. Only the ENGLISH side is from GeoNames; the Urdu was proposed by our own spelling rules | CC BY 4.0 (English side) | The attribution below |

The file has 12,942 entries; 8,212 of them count as known. A row with source `engine` stays in
the file so the reviewer can confirm it in bulk (set `review` to `reviewed`), but it does not
count as a known word until then: the screen marks it "check the spelling" and a save asks the
user to confirm it, exactly like a word the rules made (PRD FR-7.66; `Entry::counts_as_known`).
The PRD asks for 15,000 known entries (FR-7.27, NFR-URD-24). That is NOT met; the gap is closed
by the reviewer confirming the `engine` rows and by the shop's own 1,572 account names (Phase 2
import). The list is not padded; `the_list_has_fifteen_thousand_known_entries` is kept as an
ignored test.

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

All rows are `pending`. Nothing in the list was checked by a person who reads Urdu.
