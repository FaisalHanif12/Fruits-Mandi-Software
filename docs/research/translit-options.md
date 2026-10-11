# English-to-Urdu typing: data sources, method and test design

| | |
|---|---|
| Date | 12-10-2026 |
| For | `tech-lead`, `backend` (plan 05, `core::translit`) |
| Covers | PRD FR-7.22, FR-7.26, FR-7.27, FR-7.66; NFR-URD-24, NFR-URD-25; Tech Stack 4.4 |
| Status | Research. Nothing here changes a requirement. Items marked **(needs Umar)** would. |

Numbers marked **measured** were measured on 12-10-2026 with scripts run against the real downloads. Numbers marked **unverified** are estimates.

---

## 1. Summary and recommendation

1. **The word list is the main lever, not the model.** Published results show that, for Urdu words the model has never seen, even the best neural model gets about 37% of words right first time. A simple pair n-gram model gets about 36%. (Dakshina paper, Table 2: Urdu word error rate 64.3% pair 6-gram, 63.3% transformer.) No method reaches 95% on unseen names. So the 95% / 99% targets can only be met by names that are in the list, a learned pair, or a saved account.
2. **Best licence-clear source: Wikidata labels (CC0).** Pakistani people, places, given names and other Pakistani items with both an Urdu and an English label give **6,683 new Roman words** not in our seed (measured). Adding Urdu-labelled people of India gives **17,149** in total, but many of those are Hindi names a mandi in Kasur will rarely see. The most common Wikidata Urdu spelling agrees with our hand-made dev list for **344 of 384 words (89.6%)** for the Pakistan sets (measured), so every row still needs review. CC0 means no duties at all.
3. **Second source: GeoNames (CC BY 4.0) for place spellings in Roman letters.** Punjab and Khyber Pakhtunkhwa have **67,478 distinct populated-place names** in GeoNames (measured), but only 923 Pakistani features have an Urdu name. So GeoNames gives the Roman side; the Urdu side comes from our model and a human review. Duty: one attribution line.
4. **Best domain source: the client's own old data.** About 1,572 account names (PRD FR-7.27), plus the item and city lists. These are exactly the names the munshi will type. Urdu side is known; the Roman side comes from our reverse engine and a review.
5. **Do not ship Dakshina or OpenStreetMap data.** Dakshina is CC BY-SA 4.0 and OpenStreetMap is ODbL. Both are share-alike: the shipped word list would have to carry the same licence. Dakshina is fine for internal measurement only.
6. **Aksharantar needs care.** It claims CC-BY / CC0, but **73,666 of its 699,024 Urdu training rows are copied from Dakshina** (measured, field `source`), which is CC BY-SA. A CC0 label cannot remove that. Its mined rows (620,359, from IndicCorp, CC0) are noisy and Indian-centred: `muhammad`, `tariq`, `arshad`, `akram`, `mehmood` are all absent (measured). Use only the IndicCorp rows, only as extra training data for the model, never as list entries.
7. **Method for unseen words: a proper joint (pair) n-gram model in pure Rust.** Our engine already does an EM alignment plus an Urdu letter-pair model. Upgrading it to a pair n-gram of order 4 to 6 with smoothing is the standard method, matches neural accuracy for Urdu within about 1 point, needs no new crate, and fits in about 1 to 3 MB. A neural model (IndicXlit, 11M parameters, MIT) would add about 45 MB and a runtime crate for no real gain (its own Urdu top-1 is 48% on Aksharantar names).
8. **UX gives the last mile.** Add saved account names as a suggestion source, keep the rules-only amber mark, and add a second amber mark when a list word has two common spellings (for example شاہد / شاھد). These make wrong spellings visible instead of silent.
9. **Honest testing needs a set we did not write.** Build a held-out set from the client's real account names with Roman spellings typed by two or three people who never saw our list, plus a Wikidata hold-out chosen by item ID before any import. Report "silent wrong" (wrong first suggestion that is not marked) as the key safety number.

**Recommendation in one line:** import Wikidata (CC0) and the client's names now, add GeoNames place spellings after review, upgrade the rules to a pair n-gram model, add the saved-accounts source and the ambiguity mark, and measure on a held-out set we did not write.

Reaching 15,000 reviewed entries is realistic: seed 4,573 entries (4,492 distinct Roman words) + about 6,700 Pakistan words from Wikidata (after review, perhaps 5,000 to 6,000 survive) + about 2,000 to 4,000 Punjab and KP place words from GeoNames + about 1,500 to 2,500 from the client's data. The Indian Wikidata set can fill any gap. Each entry still needs a human review (NFR-URD-24). At about 400 entries an hour, about 10,500 new entries need about 26 hours of reviewer time (unverified estimate).

---

## 2. Datasets and licences

| Source | What it has | Size for us | Licence | Duties | Safe to ship in a commercial app? |
|---|---|---|---|---|---|
| **Wikidata labels** | Items with `ur` and `en` labels: people, places, given names | Pakistani people with both labels: 11,009 items. Pakistani settlements: 2,131. Any item with country Pakistan: 10,405. Male given names: 318. Family names with both labels: 0. Indian people: 20,694. Pakistan sets together: 8,367 distinct Roman words, **6,683 not in our seed**. With Indian people: **17,149 not in our seed** (all measured) | CC0 1.0 for structured data | None | **Yes** |
| **GeoNames** | 229,955 Pakistani features; 150,402 populated places | Punjab (admin1 `04`) 62,924 and KP (`03`) 25,860 populated places; 67,478 distinct names; 6,331 distinct name parts used 3 or more times (measured). Urdu names: only 923 features (measured) | CC BY 4.0 | Attribution ("Contains data from GeoNames, CC BY 4.0") in About and in the repository | **Yes**, with attribution. Roman side only. |
| **Client's old software** | About 1,572 account names in Urdu, plus cities, items, packings | About 1,500 to 2,500 distinct words (unverified until exported) | Client's own data | Client's consent; keep out of the public repository if he asks | **Yes**, the best domain match |
| **Dakshina** (Google) | Urdu romanization lexicon: 25,000 train + 2,500 dev + 2,500 test native words, each with attested romanizations | 2.0 GB tar (measured, all 12 languages) | CC BY-SA 4.0 | Attribution and **share-alike**: our word list would have to be CC BY-SA too | **No** for shipping. **Yes** for internal measurement only (no distribution). |
| **Aksharantar** (AI4Bharat) | Urdu: 699,024 train, 12,419 valid, 14,878 test pairs (measured) | Sources in the Urdu train file: IndicCorp 620,359; Dakshina 73,666; Existing 2,554; AK-Uni 2,445 (measured) | Card says: manual data CC-BY, mined data CC0. An older card said CC BY-NC 4.0 | Attribution for CC-BY parts. Dakshina rows stay CC BY-SA whatever the card says | **Only the IndicCorp rows**, and only as model training data. Not as list entries: noisy (for example `doctorsrod` for ڈاکٹرسرود, glued words) and few Pakistani names |
| **IndicXlit model** | Trained transformer, 11M parameters | About 45 MB as fp32 (unverified) | MIT (code and models) | Keep the licence notice | Licence yes; size and value no (Section 3) |
| **OpenStreetMap** | `name:ur` and `name` tags on Pakistani places | Not measured | ODbL 1.0 | Attribution and **share-alike** for a derived database | **No**. Keep it out of the word list. |
| **CLE word lists** (cle.org.pk) | Urdu word lists | Not measured | Not open (terms on the site) | Unknown | **No** without written permission |

Notes:

- Wikidata quality is good but not perfect. Labels sometimes carry extra words, for example Q11745 "Arfa Karim" has the Urdu label ارفع کریم رندھاوا and Q28189 "Abdus Salam" has عبد السلام قادیانی. The import keeps only label pairs with the same number of words (8,240 of 11,009 people kept, measured), and the reviewer checks every new pair.
- 1,472 Roman words in the Pakistan sets have more than one Urdu spelling (measured). Some are real variants (شاہد / شاھد), some are label noise. Keep real variants with a weight from their count. This is useful data for the ambiguity mark in Section 6.
- Dev list words missing from the seed: 54. The Pakistan sets have 33 of them (measured). So Wikidata cuts the out-of-list rate on our own dev names by more than half.
- The Indian set alone agrees with the dev list for 207 of 220 words (measured) but finds only 5 of the 54 missing words: it adds many words, few of them ours.

---

## 3. Method options for words not in the list

| Option | How it works | Expected accuracy on unseen words | Size | Speed on Core i3 | New dependency | Effort |
|---|---|---|---|---|---|---|
| A. Today: letter groups + EM alignment + Urdu letter pairs + beam search (`model.rs`, `forward.rs`) | Each Roman group has Urdu options; scored by the group's own counts and a 2-letter Urdu model | "Much lower" than in-list (team measurement; not repeated here) | Built from the list at start-up | Within 50 ms today | None | Done |
| **B. Pair (joint) n-gram, order 4 to 6, Kneser-Ney or Witten-Bell smoothing (recommended)** | Align each pair (EM, as today). Treat each aligned unit `sh:ش` as one symbol. Train an n-gram model over the symbol sequence. Decode with a beam search that also yields the k best. This is Sequitur / Phonetisaurus in plain Rust. | Urdu, general words, 25,000 training words: about 36% first-time right (Dakshina pair 6-gram: WER 64.3%, CER 20.0%). On names, with our domain list: first-time 40% to 55%, top-3 60% to 75% (**unverified**, must be measured) | 1 to 3 MB after pruning (unverified) | Under 5 ms per word for beam 8 (unverified; same class as today's beam) | None | 4 to 6 days, including a training tool and tests |
| C. Option B plus an Urdu letter 5-gram trained on a large CC0 Urdu name and word list (Wikidata labels, IndicCorp rows of Aksharantar) | Re-scores the k best from B with a stronger model of what Urdu names look like | Small gain over B, perhaps 2 to 5 points top-3 (**unverified**) | +1 to 2 MB | +1 ms | None | +2 days |
| D. Tiny neural model (IndicXlit transformer through `tract` or `ort`) | Encoder-decoder over letters | Dakshina transformer: about 37% (WER 63.3%), 1 point above B. IndicXlit Urdu top-1: 48.37 (Aksharantar), 47.77 (named entities), 42.12 (Dakshina) | About 45 MB fp32, about 11 MB int8 (unverified) | 10 to 40 ms per word with beam 4 (unverified); risky against NFR-SPD-20 on a slow PC | `tract` (pure Rust) or `ort` (native ONNX Runtime DLL) | 2 to 3 weeks; not deterministic across CPUs unless care is taken (Code Standards B28) |
| E. Hand rules only | Fixed letter rules | Lower than B | Tiny | Fast | None | Already part of A |

Why B: for Urdu, all three model types in the Dakshina study land within 1 point of each other (pair 6-gram 64.3%, transformer 63.3%, LSTM 63.4% WER). The hard part is not the model. It is the script: one Roman sound has several Urdu letters (s: س ص ث; z: ز ذ ض ظ; t: ت ط; h: ح ہ ھ), and short vowels are not written. Only a known word or a person can choose. B keeps the engine pure, deterministic and small, which our standards require.

How B fits the current code:

- `model.rs` already does the EM alignment. Keep it, but output aligned symbol sequences.
- Add `pair_ngram.rs`: counts of symbol n-grams, smoothing, and back-off. Keep `BTreeMap` order so the model is the same on every computer (B28).
- Train in a small dev tool (for example `cargo run -p mandi-core --bin translit_train`) and commit the model file next to `translit_seed.tsv`, embedded with `include_bytes!`. A test checks that training the committed list gives the same file (hash), so nobody edits the model by hand.
- `forward.rs`: replace the group-chance times letter-pair score with the pair n-gram score. Keep the 3-result limit and the source tag `rules`.

---

## 4. Exact download steps for the cleared sources

All queries run at https://query.wikidata.org/ (or with `curl`, below). Add a `User-Agent` with a contact address, as Wikimedia asks. Each query below ran on 12-10-2026 within the 60 second limit.

### 4.1 Wikidata: Pakistani people (11,009 rows measured)

```sparql
SELECT ?item ?en ?ur WHERE {
  ?item wdt:P27 wd:Q843 ;          # country of citizenship: Pakistan
        wdt:P31 wd:Q5 ;            # instance of: human
        rdfs:label ?ur .
  FILTER(LANG(?ur) = "ur")
  ?item rdfs:label ?en .
  FILTER(LANG(?en) = "en")
}
```

### 4.2 Wikidata: Pakistani settlements (2,131 rows measured)

```sparql
SELECT DISTINCT ?item ?en ?ur WHERE {
  ?item wdt:P17 wd:Q843 ;                       # country: Pakistan
        wdt:P31/wdt:P279* wd:Q486972 ;          # any kind of human settlement
        rdfs:label ?ur .
  FILTER(LANG(?ur) = "ur")
  ?item rdfs:label ?en .
  FILTER(LANG(?en) = "en")
}
```

### 4.3 Wikidata: given names (318 male rows measured)

```sparql
SELECT ?item ?en ?ur WHERE {
  VALUES ?cls { wd:Q12308941 wd:Q11879590 wd:Q202444 }   # male, female, any given name
  ?item wdt:P31 ?cls ;
        rdfs:label ?ur .
  FILTER(LANG(?ur) = "ur")
  ?item rdfs:label ?en .
  FILTER(LANG(?en) = "en")
}
```

The version with all three classes timed out once on the public endpoint. Run one class at a time if so. Family names (`wd:Q101352`) with both labels returned 0 rows.

### 4.4 Optional Wikidata extra: Urdu-labelled people of India and other countries

Same as 4.1 with `wd:Q668` (India, 20,694 rows measured) or `wd:Q889` (Afghanistan, not measured). Many Muslim names in North India match Pakistani spelling. Review more strictly: Indian Urdu sometimes uses other spellings, and many entries are Hindu names. Import last, only to fill the gap to 15,000.

### 4.4b Wikidata: any item with country Pakistan (10,405 rows measured)

Catches firms, markets, mosques, rivers and other places that 4.2 misses. Review harder: organisation names hold English words.

```sparql
SELECT ?item ?en ?ur WHERE {
  ?item wdt:P17 wd:Q843 ;
        rdfs:label ?ur .
  FILTER(LANG(?ur) = "ur")
  ?item rdfs:label ?en .
  FILTER(LANG(?en) = "en")
}
```

### 4.5 Running a query from the shell

```bash
curl -s -G https://query.wikidata.org/sparql \
  --data-urlencode "query@people_pk.rq" \
  -H 'Accept: text/tab-separated-values' \
  -A 'MandiSoftware-wordlist/0.1 (contact address)' > people_pk.tsv
```

### 4.6 GeoNames (CC BY 4.0)

```bash
curl -O https://download.geonames.org/export/dump/PK.zip                # all features
curl -O https://download.geonames.org/export/dump/alternatenames/PK.zip  # other names, with language codes
```

- Main file columns: 2 = name, 7 = feature class (`P` = populated place), 8 = feature code, 11 = admin1 code (`04` Punjab, `03` Khyber Pakhtunkhwa, measured from the Lahore, Okara and Kasur rows), 15 = population.
- Alternate names file: column 3 = language (`ur` rows: 1,067, measured).
- Take place names from Punjab and KP, split into words, keep words used in 3 or more place names (6,331 words, measured), then start with the districts and tehsils near Kot Radha Kishan (Kasur, Lahore, Okara, Sheikhupura, Pattoki, Chunian) and the trucks' usual sources (Swat, Peshawar, Mardan).
- The Urdu side comes from our engine (top 3) and a reviewer picks or fixes it.

### 4.7 The client's data

When Umar exports the old software's CHART table (PRD FR-7.27), take `URDU_NAME`, split into words, run our Urdu-to-English engine, and have the reviewer confirm the Roman spelling. Add common Roman variants a munshi would type (for example `muhammad`, `mohammad`, `muhammed`), each pointing to the same Urdu.

### 4.8 Import rules (all sources)

- One row per (Roman, Urdu) pair. Columns as today: `roman`, `urdu`, `kind`, `weight`, `review`, plus a new `source` column (`wikidata`, `geonames`, `client`, `team`). **(needs Umar only if the file format in Tech Stack 4.4 must change; it lists four columns, the file already has five)**
- Normalise Urdu with `core::text` rules (ي to ی, ك to ک, remove diacritics) before comparing.
- Weight from count: the most common Urdu spelling for a Roman word gets 100, others in proportion.
- `review = pending` until a reviewer sets `ok` and their initials (NFR-URD-4). Only reviewed rows count toward 15,000.
- Keep a `NOTICE` or `THIRD_PARTY.md` with the GeoNames attribution and the Wikidata CC0 note.

---

## 5. Measurement design

### 5.1 What is wrong with measuring on our own list today

The frozen test list (`translit_names.tsv`) and the seed were both written by the team. Even with separate pools, the same people choose the spelling style. That makes the in-list rate look better than real use. The measured 94.8% / 97.1% is a fair number for that list, not yet for the munshi.

### 5.2 Three test sets

| Set | How it is made | Size | Use |
|---|---|---|---|
| T1 Frozen list (exists) | As today | 1,000 names | Regression guard on every build. Never tune to it. |
| T2 Wikidata hold-out | Before any import, put every Wikidata item whose numeric ID ends in 0 into T2 and **never** into the seed. Rule fixed in code, so it is repeatable | About 10% of imported items, about 1,500 to 2,000 names | Unseen-name accuracy from a source we did not write |
| T3 Real typing | Take 500 names from the client's accounts (Urdu known). Show each Urdu name to two or three people who never saw our list (the munshi, a staff member, one more). Ask "how would you type this in English letters?" Keep every answer. Same method as Dakshina | 500 names, 1,000 to 1,500 Roman spellings | The number that matters for go-live |

T3 names must be removed from the seed, or split 50/50 by a fixed hash with only one half allowed into the seed. Write the split rule down before looking at results.

### 5.3 What to report, per set and per kind (person, place, item)

| Metric | Meaning |
|---|---|
| Top-1 and top-3, in-list | NFR-URD-24 targets 95% / 99% |
| Top-1 and top-3, not in list | Must not drop below the last release |
| Coverage | Share of typed words found in list, learned pairs or accounts |
| **Silent wrong rate** | First suggestion wrong **and** not marked amber. This is the safety number: the user may accept it without looking. Target close to 0 |
| Flag rate | Share of words marked amber. Too high and users stop reading the mark |
| Whole-name rate | All words of a full name right (a name with 3 words fails if one word fails) |
| English accuracy (NFR-URD-25) | English suggestion accepted without change, 90% target |

### 5.4 Is 1,000 names enough?

With 1,000 words and a true rate of 95%, the 95% confidence band is about ±1.4 points. So a measured 95.0% does not prove the target. Report the lower bound and aim for a measured 96.5% or more. For 99% top-3 the band is about ±0.6 points. T1 + T2 + T3 together (about 3,500 Roman spellings) give a tighter band.

### 5.5 Checks against leakage

- A test fails if any T2 or T3 Urdu name (after normalisation) is in the seed.
- The training tool reads only `translit_seed.tsv`. It never reads a test file.

---

## 6. UX that raises real correctness

| Idea | Effect | Change needed |
|---|---|---|
| **Saved names as a source.** Words from saved account, city, item and پختہ کھاتہ names (both scripts, already in the search index) are offered right after learned pairs | Every party typed once is right forever after, on every screen. In a mandi most typing is repeat parties | Tech Stack 4.4 lists three sources. Adding a fourth **(needs Umar)** |
| **Ambiguity mark.** A list word whose Roman form has two common Urdu spellings (1,472 such words in the Wikidata Pakistan sets, measured, some of them noise; for example `shahid` شاہد / شاھد) shows a small marker so the user looks at the three choices | Turns a "silent wrong" into a visible choice | FR-7.66 covers only rules-only words. Extending the amber mark **(needs Umar)**; a softer non-amber hint does not change behaviour |
| **Keep the rules-only check (FR-7.66)** | A rules-only spelling is never saved without a look | Already required |
| **Learned pairs (FR-7.26)** | Corrections stick from the next word | Already required. Also learn when the user types Urdu with the phonetic keyboard and the English name is filled: the pair (English, Urdu) is free, reviewed data |
| **Show the English next to each Urdu choice** in the list, for example "شاہد (Shahid)" | Helps a user who reads English better tell choices apart | Display only; no requirement change |
| **More than 3 choices** | Top-5 would lift the "right answer is shown" rate | FR-7.22 says up to 3. **(needs Umar)**. Recommend not changing: 3 is easier to read; the ambiguity mark does the same job |
| **Phonetic keyboard fallback (`Alt+U`)** | Exact spelling for rare names | Already required |
| **Reviewer screen for learned pairs** | Owner removes bad pairs (FR-7.26) | Already required; make learned pairs exportable so the team can fold good ones into the next word list release |

---

## 7. Step by step plan for `backend`

Each step ends green before the next. Numbers come from the report of each step, not from this document.

1. **Freeze test sets first.** Add T2 split rule (Wikidata ID ends in 0) and the leakage test (5.5). Keep T1 as is.
2. **Import tool.** A small script or Rust dev binary that reads the Wikidata TSVs (Section 4.1 to 4.3), keeps pairs with equal word counts, normalises Urdu, counts spellings, sets weights, writes `review=pending`, `source=wikidata`, and skips anything in T2. Output goes to a staging file, not straight into the seed.
3. **Review.** Reviewer accepts or fixes staged rows. Only `ok` rows move to `translit_seed.tsv`. Track the reviewed count toward 15,000 in the accuracy test output.
4. **GeoNames place words.** Same tool: Punjab and KP populated places, words used 3 or more times, nearest districts first. Urdu from the engine's top 3; reviewer picks. Add the attribution file.
5. **Client words** when Umar exports them (4.7). Then build T3 from them (5.2).
6. **Pair n-gram model** (Section 3, option B): aligned symbols, smoothing, k-best decode, training tool, committed model file, hash test, property tests (never panics, at most 3, same input gives same output).
7. **Measure** on T1, T2, T3 with all metrics of 5.3. Write the numbers into the plan file.
8. **Only if Umar approves:** saved-names source and the ambiguity mark (Section 6).
9. **Optional (C):** Urdu letter 5-gram re-scoring, only if step 7 shows top-3 on unseen words below the last release.

Do not start option D (neural). Re-open it only if T3 shows the model, not the coverage, is the limit.

---

## 8. Sources

- Dakshina dataset (licence CC BY-SA 4.0, lexicon sizes, download): https://github.com/google-research-datasets/dakshina
- Roark et al., "Processing South Asian Languages Written in the Latin Script: the Dakshina Dataset", LREC 2020, Table 2 (Urdu: pair6g CER 20.0 / WER 64.3; transformer 19.5 / 63.3; LSTM 19.4 / 63.4): https://aclanthology.org/2020.lrec-1.294.pdf and https://arxiv.org/abs/2007.01176
- Aksharantar dataset card (licence statement, Urdu split sizes): https://huggingface.co/datasets/ai4bharat/Aksharantar
- Aksharantar paper: https://arxiv.org/abs/2205.03018
- IndicXlit (MIT licence, 11M parameters, Urdu top-1 48.37 / 47.77 / 42.12): https://github.com/AI4Bharat/IndicXlit
- GeoNames dump readme (CC BY 4.0, file formats): https://download.geonames.org/export/dump/readme.txt
- Wikidata licensing (structured data CC0): https://www.wikidata.org/wiki/Wikidata:Licensing (not re-fetched on 12-10-2026; well known)
- Wikidata Query Service: https://query.wikidata.org/
- OpenStreetMap licence (ODbL): https://www.openstreetmap.org/copyright (not re-fetched)
- Bisani and Ney, "Joint-sequence models for grapheme-to-phoneme conversion", Speech Communication 2008 (pair n-gram method; cited by the Dakshina paper)
- Novak et al., "Phonetisaurus: exploring grapheme-to-phoneme conversion with joint n-gram models in the WFST framework", Natural Language Engineering: https://www.cambridge.org/core/journals/natural-language-engineering/article/abs/phonetisaurus-exploring-graphemetophoneme-conversion-with-joint-ngram-models-in-the-wfst-framework/F1160C3866842F0B707924EB30B8E753

### Measurement notes (12-10-2026)

- Measured on downloads made today: Wikidata query results, Aksharantar `urd.zip` (17.8 MB), GeoNames `PK.zip` and `alternatenames/PK.zip`.
- "Agreement with dev list" means the most common Wikidata Urdu spelling for a Roman word equals one of the accepted spellings in `backend/crates/core/tests/translit_dev.tsv`, word by word. The frozen test list was not used for any measurement here.
- Downloads and scripts were kept in a scratch folder and are not part of the repository.
