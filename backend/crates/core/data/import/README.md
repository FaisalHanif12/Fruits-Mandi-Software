# Making the word list again

Tools for `data/translit_seed.tsv` (see `../SOURCES.md` and `docs/research/translit-options.md`).
The downloads are untrusted data: keep them in their own folder outside the repository, and run
the script with `python3 -I`.

1. Download (Wikidata Query Service; add a `User-Agent` with a contact address):

   ```bash
   q() { curl -s -G https://query.wikidata.org/sparql --data-urlencode "query=$2" \
         -H 'Accept: text/tab-separated-values' -A 'MandiSoftware-wordlist/0.1 (contact)' -o "$1.tsv"; sleep 3; }
   q people_pk   'SELECT ?item ?en ?ur WHERE { ?item wdt:P27 wd:Q843 ; wdt:P31 wd:Q5 ; rdfs:label ?ur . FILTER(LANG(?ur) = "ur") ?item rdfs:label ?en . FILTER(LANG(?en) = "en") }'
   q settlements_pk 'SELECT DISTINCT ?item ?en ?ur WHERE { ?item wdt:P17 wd:Q843 ; wdt:P31/wdt:P279* wd:Q486972 ; rdfs:label ?ur . FILTER(LANG(?ur) = "ur") ?item rdfs:label ?en . FILTER(LANG(?en) = "en") }'
   q given_male   'SELECT ?item ?en ?ur WHERE { ?item wdt:P31 wd:Q12308941 ; rdfs:label ?ur . FILTER(LANG(?ur) = "ur") ?item rdfs:label ?en . FILTER(LANG(?en) = "en") }'
   q given_female 'SELECT ?item ?en ?ur WHERE { ?item wdt:P31 wd:Q11879590 ; rdfs:label ?ur . FILTER(LANG(?ur) = "ur") ?item rdfs:label ?en . FILTER(LANG(?en) = "en") }'
   q given_any    'SELECT ?item ?en ?ur WHERE { ?item wdt:P31 wd:Q202444 ; rdfs:label ?ur . FILTER(LANG(?ur) = "ur") ?item rdfs:label ?en . FILTER(LANG(?en) = "en") }'
   curl -O https://download.geonames.org/export/dump/PK.zip                 # main file
   curl -O https://download.geonames.org/export/dump/alternatenames/PK.zip   # in another folder
   ```

2. Make the pairs file and the place words:

   ```bash
   P=prepare_pairs.py
   for f in people_pk; do python3 -I $P wikidata name $f.tsv >> pk.tsv; done
   for f in given_male given_female given_any; do python3 -I $P wikidata name $f.tsv >> pk.tsv; done
   python3 -I $P wikidata place settlements_pk.tsv >> pk.tsv
   python3 -I $P geonames main/PK.txt alt/PK.txt >> pk.tsv
   python3 -I $P geonames-words main/PK.txt 3 > place_words.tsv
   ```

3. Make the list and the hold-out file. `--seed` must be the list WITHOUT the import (the
   4,572 `team` rows), so the result is the same every time:

   ```bash
   cargo run -p mandi-core --release --example translit_import -- \
     --seed team_rows.tsv --pairs pk.tsv --place-words place_words.tsv --target 15500 \
     --out crates/core/data/translit_seed.tsv --holdout crates/core/tests/translit_holdout.tsv
   ```

   `team_rows.tsv` is `translit_seed.tsv` with the rows whose `source` is not `team` removed.

4. Run `cargo test -p mandi-core --test translit_seed --test translit_accuracy`. If the numbers
   went up, raise `tests/translit_baseline.txt`. Never lower it.

The tool never reads `tests/translit_names.tsv` (the frozen list).
