# ai-eval: the live check of "ask the app"

Plan 52 items 12 and 13. Only the lead runs the live part, with Umar's key. Nothing here reads, prints
or writes the key.

## Files

| File                                                                                                | What it is                                                                                                                                                                                                                           |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `run.mjs`                                                                                           | The runner and the scorer. `--check`, `--self-test`, `--category`, `--world`, `--only`, `--facet`, `--every`, `--threads`, `--results`.                                                                                              |
| `questions.tsv`                                                                                     | The bank (1,126 questions). Built, never edited by hand.                                                                                                                                                                             |
| `questions-110.tsv`                                                                                 | The first 110 questions, frozen. They keep the D6 mix.                                                                                                                                                                               |
| `bank-spec.mjs`                                                                                     | The new questions, one entry for each meaning, with its wordings.                                                                                                                                                                    |
| `bank-spec-a2.mjs`                                                                                  | Facet A2: the 120 real-time figure questions (plan 55 Section 10), imported by `bank-spec.mjs`.                                                                                                                                      |
| `bank-spec-mix.mjs`                                                                                 | Facet MIX: one question with a real figure AND a forecast (Umar, 05-11-2026).                                                                                                                                                        |
| `bank-spec-cap.mjs`                                                                                 | Facet CAP: the cells of the capability matrix that had no question (plan 55 Section 11): open and write to every screen.                                                                                                             |
| `bank-spec-cap-more.mjs`, `-registers.mjs`, `-nosuch.mjs`, `-print.mjs`, `-forecast.mjs`            | The other parts of facet CAP: reports, views, tiles and records with oracle figures; the registers and lists; "no such record" for every row that names one; print and open of the mapped reports; the forecast and the mixed cells. |
| `capability-rows.mjs`, `build-capability.mjs`, `capability.tsv`                                     | The capability matrix (plan 55 Section 11): every thing the app holds against every question type, read from the code.                                                                                                               |
| `capability-items.mjs`, `build-capability-items.mjs`, `capability-items.tsv`, `item-axis-sold.json` | The ITEM AXIS of the matrix (Umar, 06-11-2026): every measure about an item against every kind of subject in every period (2,070 cells). One bank question for each answerable cell, facet `ITEM`.                                   |
| `capability-money.mjs`, `capability-money.tsv`                                                      | The PERIOD AXIS of the money and party subjects (lead, 06-11-2026): 950 cells, facet `MONEY`. Built by the same `build-capability-items.mjs`.                                                                                        |
| `build-bank.mjs`                                                                                    | Builds `questions.tsv` from the two files above. `run.mjs --check` fails if it is out of date.                                                                                                                                       |
| `oracle-keys.txt`                                                                                   | The facts the oracle gives (the `@key` of a question). Checked by a Rust test.                                                                                                                                                       |
| `urdu-review.tsv`                                                                                   | The sheet for the Urdu reviewer, written by every run.                                                                                                                                                                               |
| `check-modes.mjs`                                                                                   | A scripted check of `--bank`, `--threads` and `--offline` (no model, no key, no cargo).                                                                                                                                              |
| `bank-spec-d7.mjs`, `build-bank-d7.mjs`, `realtime-d7.tsv`, `realtime-d7-coverage.md`               | The D7 bank (plan 59 B1): 455 real-time figure questions, picky on purpose, with the list of every kind of record and figure the app keeps and the question ids that cover it. See "The D7 bank" below.                              |
| `realtime-d7-gaps.md`, `../../backend/crates/assistant/tests/robustness_gap.rs` | The gap list of the D7 bank (plan 59 R2): which wordings still go to the model, ranked by cause, with the instant plan each needs. The test dumps what `understand` and `plan_reads` made of every question of a bank. See "The gap dump" below. |

The oracle itself is Rust: `backend/crates/assistant/tests/robustness_year/` (`seed.rs` builds the books
through the app's own save functions, `oracle.rs` asks the app's own reports and analytics).

## The bank

Five categories (plan 52 Section 2), each in Roman Urdu, Urdu and English, with misspellings and groups of
the same meaning:

| Cat | Questions              | What                                                                                                                      |
| --- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| A   | real-time data         | every section: item, seller, buyer, partner, period, cash, dues, stock, garden, trip, fruit, expense, capital, day, month |
| B   | suggestions            | with data (from the books, not marked general) and on thin books (general advice, one call, no figure)                    |
| C   | forecasts and what-ifs | year end, pace doubled, next months, targets, +10%                                                                        |
| D   | help                   | every one of the 38 topics of the guide                                                                                   |
| E   | mixed and hard         | follow-ups, misspellings, two parts, nonsense, injection, change requests, unknown names                                  |

Three facets of Umar's request of 03-11-2026 (plan 52 Section 8.1) run through the categories. A question
carries its facet in `filters` (`facet=M`, `facet=day`, `facet=nosuch`), and each one has its Roman Urdu,
Urdu and English twins in a group:

| Facet                 | Questions | What                                                                                                                                                                                                                                                                                                                                                                                                                       |
| --------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| M, mixed language     | 36        | Roman Urdu with English words ("Rahim ne this month kitna sell kiya"), Urdu letters with English words ("رحیم کی sale this month کتنی رہی"), English with Urdu mandi words ("how much bakaya does Faisal have"). Every section of category A at least once: item, seller, buyer, partner, period, cash, dues, stock, garden, trip, fruit, expense, capital, day, month.                                                    |
| day, a party's day    | 15        | "X ne aaj / kal / 18 September ko kya aur kitna khareeda (becha)": a buyer today, a buyer yesterday, a buyer on a date with two items, a seller yesterday, a seller on a date with two items. Each item's qty and amount and the day's total, all from the oracle (`buyer_day.*`, `seller_day.*`). A buyer's amount is the value or the value with commission and laga (`with_charges`); both are true figures of the app. |
| nosuch, no such party | 14        | A made-up seller and a made-up buyer end in `not_found` with `asked_role`. A buyer asked as a seller ends in `not_found` with `asked_role=seller` and `found_as=buyer`; a seller asked as a buyer the reverse. A misspelled real seller (Raheem for Rahim) is still found, with his figures (category A).                                                                                                                  |

A fourth facet, **print** (plan 54 Section 6, `facet=print`, 104 questions): "print X's sales for a period"
for every row of the report map (a buyer, a seller, no party, a khata) and every period in words (aaj, kal,
is mahine, pichle mahine, a named month, "1 se 15 October tak", pichle hafte), each in Roman Urdu, Urdu and
English, the buyer cells also mixed; "akram ka bill nikalna" (two Akrams: which one) and a name nobody has.
The expected report, party and dates are in the `figures` column as `print.report`, `print.party` or
`print.seller`, `print.as_of` or `print.from` and `print.to`, and `print.steps`. The dates are the oracle's
calendar (`@cal.today`, `@cal.last_month_start` ...), never typed; "last week" is the calendar week before
this one, Monday to Sunday. The dates are not the bank's own rule: the oracle asks the app's period helper
(`mandi_assistant::tools::period::compute`, `Kind::LastWeek`, the one behind the `period` tool). The report
date shortcuts (FR-19.2) and the analytics period filter have no "last week"; the helper is the app's rule.
"1 se 15 tak" with no month is the latest month where the 15th has come (today 05-11-2026: 1 to 15 October).
A print that names no period follows the app's rule (`understand/plan.rs`, `dated`): a report of a person or a
thing (a partner's R31, a garden's R36, the trips' R33) opens on the whole books with no dates, written
`print.dates=none` (the button's from and to must both be empty, and the steps set no dates); a report of a
period with no person (R29, R12, R37, R30) takes today to today and says so, written `print.from=@cal.today;print.to=@cal.today`.

Facet **U** (plan 55 gate 4, `facet=U`, 25 questions): Umar's own questions from his log, word for word with his
spelling, each with its Urdu and English twin: "mujhay faisal ka khata batao?", "usman shirakat daar ka kitna
hai abhi ?", the three sales-growth questions, "mujhay bikri add krne ka tareeka batao?", the market safar
question and the two word orders of "faisal ki aj ki bikri ka print". In the year world Faisal is a buyer and
the third partner is Usman (he was Partner C). His quick chips (total_dues, today_sales, party_ledger,
sellers_payable) are checked against the oracle in `robustness_live.rs` with no model call.

Facet **A2** (plan 55 Section 10, `facet=A2`, 120 questions): real-time figures asked any way, the questions the
app answers by itself with no model call (PRD FR-7.90). 40 groups of 3 wordings: a buyer's balance (9 buyers, one at
zero), a seller's payable (and an advance), the shop's cash, total dues, total payable and open advances, the shop's
sale and commission for today, yesterday, this week, this month, last month, September, August, 15 October, this
year and up to now, an item (banana today, apple, mango), a seller, a buyer, goods left (mosambi) and a partner's
total. The wordings are very short, with no question word, long and polite, rude, Roman Urdu, Urdu letters, English,
Roman Urdu or Urdu with English words, and misspelled (17). Every figure is an oracle key. The report ends with the
REAL-TIME FIGURES lines: the share answered right with no model call ("instant", target 60%), right in no call or
one (85%), and the wait (p50 2 s, p95 8 s). A2 is gated in "meets"; the same lines for all of category A are shown
but not gated, because A also holds comparisons, two-part questions and follow-ups.

Facet **MIX** (`facet=MIX`, category C, 10 groups of 3): ONE question that asks for a real figure and a forecast or a
what-if ("pichle mahine electricity ka kitna kharcha hua aur agle 3 mahine ka estimate kitna hai"). The labels of
`figures` split the two parts: `real_*` (a figure of the books) and `est_*` (an estimate, any of the three methods
counts: the band). The scorer keeps them apart (PRD FR-7.76 point 4): a real figure shown as an estimate, or an
estimate shown as a figure of the books, is a miss. A group whose estimate has no route yet carries
`pending=<reason>` in its filters: it is PENDING (neither a pass nor a miss) and the run cannot say "meets" while it
waits. (The head, balance and cash estimates have landed: PRD FR-7.86 as approved by Umar; six questions wait for the
commission rules, the attention list and the day state.)

**The year books for the state and list questions (06-11-2026).** The rich year has a munshi (Sultan), a collector
(Rasheed, who collects from Akram Toll Tax and Faisal) and a labourer (Naseem), and, added at the very end and dated
today, four seller commission rules from PRD 11.3: Rahim + banana 5% (party and item), Rahim any item 3% (party),
Karim any item 4% (party) and apple, any seller, 30 for each نگ (item). The oracle's `rules.<party>.<item>.<side>_<charge>`
and `rules.<item>.<side>_<charge>` facts are its OWN reading of 11.3 from the rows of `charge_rules` (party + item,
party, item, group, default; the newer rule wins a tie), never the tool's. The balance estimates
(`est_balance.<subject>.*`) use the average of the LAST 3 full months (`BALANCE_MONTHS_MAX`), as the tool does.
The other new facts: `staff.*`, `heads.count`, `truck.*`, `attention.*`, `days.*`, `audit.bikri<N>.count`, `r15_type.*`.
Before and after the seed change the dump was diffed (`dumpdiff.py` in the session's scratchpad): only the rule facts,
the balance estimates and the collector facts moved.

Two scorer rules added on 05-11-2026 (lead's rulings):

- **`noforecast=1`** (plan 55 Section 11.2, the stock and what is payable to sellers: forecasts the app does not
  keep). The answer gives today's real figure and shows no forecast of any kind: an estimate block, a forecast
  figure or a forecast chart is a miss. Saying "I do not forecast this" is the honest answer. One question for each
  such cell of the matrix (status N).
- **The log of a local or instant answer is its buttons** (PRD FR-7.90, `figureLogProblems`). For an answer whose
  finish reason is `local`, every read of a report, a view, the tiles or a party card in the log has its button
  (the same report or view), every such button has a read, and when a read and its button both carry dates they
  are the same dates. A print answer keeps its own stricter check (`logButtonProblems`: the same entries in the
  same order with the same filters). The total dues forecast is written against the buyers' dues: the tile total
  is the real figure and the estimate is of the buyers' dues only.

Facet **CAP** (`facet=CAP`): the questions that fill cells of `capability.tsv`. This file's first part holds the
screens: 29 groups that OPEN a screen (answered, the guide topic and the screen route are both checked) and 11
that ask to WRITE (change_refused with the entry screen). The reports and views with oracle figures follow.

Facet **ITEM** (`facet=ITEM`, the item axis, Umar 06-11-2026: "thousands of question types on real-time item
data"). A full cross product, written by `capability-items.mjs` and listed cell by cell in `capability-items.tsv`:

- **21 measures**: sale (gross), quantity, seller commission, buyer commission, laga, total commission, average
  rate, last rate, buyers (count and the top one), sellers (count and the top one), rank among the items, the top
  item of a group, against the period before, best month, the monthly trend, stock left, a forecast, a print and
  an open screen;
- **9 subjects**: one fruit, one vegetable, the fruit group, the vegetable group, all the items, an item for one
  party (Faisal: banana and potato), an item of one quality (plan 58), an item that is NOT in the books (Capsicum,
  `not_found` with `asked_role=item`), and an item that IS in the books with no sale in the period (the answer
  is 0, never `not_found`; for a rate, a buyer or a rank there is nothing to say, so any `safe` ending counts);
- **14 periods**: today, yesterday, two named days (3 November, 14 October), this and last week, this and last
  month, two named months (September, August), this and last year, a range (1 to 15 October) and "ab tak". The
  period before each is the baseline of "against ...".

Every cell ends as **A** (one question: one wording, rotating Roman Urdu, Urdu, English and two kinds of mixed
wording, about a quarter misspelled), **X** (no meaning: "the top buyer of an item for one buyer"), **N** (not by
design, with the reason, waiting for the lead's approval) or **G** (a gap, with its owner). A line of
`capability-items.tsv` that is the same in every period of its measure is written once with period `*`.

The figures are the oracle's `ix.<period>.<unit>.*` facts (`oracle.rs` `item_axis_facts`): view A, B and C, and SQL
on the lines of the books, never the assistant's tools. A unit is an item, `fruit`, `vegetable` or `all`. For an item:
`gross qty comm_seller comm_buyer laga comm_total avg_rate vs_gross vs_qty rank last_rate buyers_n sellers_n
top_buyer top_seller`; for a group and all: the same, with `top_item`; for Faisal's banana and potato
`ix.<period>.<item>.faisal.*` (with `with_charges`, `sellers_n`, `top_seller`); `ix.<period>.<unit>.{months,best_month,
best_month_gross}` for a year, last year and so far; `ix.stock.<unit>`. The facts `a.<period>.<item>.{gross,qty,total_comm}`
are 0 for an item with no sale. Which items sold in which period is the static table `item-axis-sold.json`, written
from an oracle dump:

    node scripts/ai-eval/build-capability-items.mjs --sold-from <oracle-dump.json>   write item-axis-sold.json
    node scripts/ai-eval/build-capability-items.mjs                                  write capability-items.tsv
    node scripts/ai-eval/build-capability-items.mjs --check                          is it up to date, is every cell A, X, N or G
    node scripts/ai-eval/build-capability-items.mjs --summary                        the counts, and the N and G lists with owners

Run the oracle again (`dump_the_oracle`), then `--sold-from` (and `prettier --write` on the json), then the builder, then
`build-bank.mjs` and `build-capability.mjs`, when the seed or the oracle changes.

Facet **MONEY** (`facet=MONEY`, the period axis of the money and party subjects, lead 06-11-2026): 30 subjects (receipts R23, payments R25,
the cash book R12 in, out and closing, the shop expense R29 and one head of it, personal drawings taken and returned, the balance lists
advances R26, money lent R32, buyers' dues R20 and sellers' payable R24, one buyer paid and bought, one seller paid and sold, a collector's
receipts, a partner's profit share and drawings R30, capital in and out and one partner's, gardens R36, trips R33 and fruit taken R37)
crossed with four questions (figure, list, compare against the period before, top) and the 14 periods of the item axis. A balance list has
no range (FR-19.10): it is asked "as of" the last day of the period, and the periods whose last day is today collapse into "today" (X).
Same A, X, N and G rules, `capability-money.tsv`, `capability-money.mjs`, facts `mx.<period>.*` and `mx.asof.<period>.*`
(oracle.rs `money_axis_facts`, from the reports and views of the app). 801 questions, one wording each.

**Quality of a sale line (plan 58 T4, PRD FR-12.42).** The lots of the last 10 days, and of every third day before them, carry a
quality on their sale lines: inside a lot the highest rate is `first`, the next `second`, the rest `third`, a tie by line order
(`seed.rs` `grade_lines`; no random number is used). The oracle's `ixq.<period>.<unit>.<first|second|third>.*` facts are SQL on those lines.
Three offline tests keep it honest: `a_quality_changes_no_amount_and_follows_the_rate_inside_a_lot` (the books built twice, with and
without a quality: every non-quality fact is equal, and inside each lot the quality follows the rate),
`dump_the_oracle_without_grades` (`MANDI_ORACLE_UNGRADED_JSON=<file>` writes the facts of the books with no quality, to diff against the
graded dump: only the quality facts differ, `ixq.*` and the `d7.sale.*` combinations that filter on a quality) and `the_print_data_of_r5_and_r1_carries_the_quality` (R5 "Print All", the seller bill and the buyer
slips of a بکری carry the quality, in English and in Urdu; one with none prints as before). The bank asks "awwal seb kal kitne ka bika",
"doyam kele ka aaj ka rate" and three more in three languages (groups `IQ-*`) and the quality cells of the item axis. A group whose key is missing waits in `run.mjs --check` as for any facet.

A group whose figures name an oracle fact that is not in `oracle-keys.txt` yet is left out of the bank and listed by
`run.mjs --check` ("waiting for new oracle facts"); it comes in by itself when the oracle has been run again.

Worlds: `day`, `late`, `e59`, `e42`, `e34` (the PRD's), `year` (12 months, 369 بکری, today 05-11-2026) and
`thin` (set up, no sale: the Owner's data today).

Expected figures are never typed in the bank. `@key` is a fact of the oracle. Forms of the `figures` column:

- `label=@key` the number must be in the answer;
- `label=@a|@b` either number (two honest readings of the question; each is a true figure of the app);
- `label?=@key` optional;
- `label!=@key` a name that must be in the answer (`|` lists names that all fit);
- `first=@key;second=@key` the first name comes before the second.

## Commands

```bash
node scripts/ai-eval/run.mjs --check        # the bank meets its rules (no model, no cargo)
node scripts/ai-eval/run.mjs --self-test    # the scorer on made-up replies
node scripts/ai-eval/build-bank.mjs         # after editing bank-spec.mjs or questions-110.tsv
node scripts/ai-eval/run.mjs --category A   # the live run of one category (the lead, with the key)
node scripts/ai-eval/run.mjs --misses       # ask again every miss and every question lost to the network,
                                            # and put the new rows in results.json (the score is of the whole bank)
node scripts/ai-eval/run.mjs --only Q111,Q112 --merge   # the same for a list you give
node scripts/ai-eval/run.mjs --facet ITEM                  # only the questions of one facet (ITEM, VEG, CAP, A2 ...)
node scripts/ai-eval/run.mjs --facet ITEM --every 5       # every fifth question of the selection: a sample that still touches every measure, subject and period
node scripts/ai-eval/run.mjs --results backend/target/ai-eval/results.json   # score a file again
node scripts/ai-eval/run.mjs --threads 1    # the live run on one line (default 3; see below)
node scripts/ai-eval/run.mjs --bank FILE --check   # another bank (a hidden one): the per-row rules only
node scripts/ai-eval/run.mjs --bank FILE    # ... run live and scored the same way
node scripts/ai-eval/run.mjs --bank FILE --out-dir DIR   # put every file of the run in DIR (outside the repository), not in backend/target
node scripts/ai-eval/run.mjs --oracle-dump out.json   # the oracle facts with values, and the names
node scripts/ai-eval/check-modes.mjs        # --bank, --threads and --offline behave as written (no model, no key)
node scripts/ai-eval/build-bank-d7.mjs      # rebuild realtime-d7.tsv and realtime-d7-coverage.md from bank-spec-d7.mjs
node scripts/ai-eval/run.mjs --bank scripts/ai-eval/realtime-d7.tsv --check      # the D7 bank meets the per-row rules
node scripts/ai-eval/run.mjs --bank scripts/ai-eval/realtime-d7.tsv --offline    # NO KEY, NO NETWORK: see "Offline" below
node scripts/ai-eval/run.mjs --results F --summary-out S --detail-out D --review-out R   # score a file again, write elsewhere
```

**Threads.** The live run asks 3 questions at once by default (`--threads N`, 1 to 8), which takes the
1,126 questions from about 3 hours to about 60 minutes. Each question has its own copy of its books and its
own conversation, and its time is the question's own (the clock starts when that question is asked).
The rows come back in the order they finish; the scorer finds a row by its id. The provider may answer
a little slower, or say it is busy, when several run at once: use `--threads 1` for a timing check on
the quietest line. `cargo test -p mandi-assistant --test robustness_live a_scripted_run_gives_the_same_rows`
proves, with a scripted model and no key, that 1, 3 and 4 threads give the same rows.

**Another bank (`--bank FILE`).** Only the rules of each row apply (unique ids, a known follow-up, a
question and an outcome, a known outcome, world and category, a section, no reference in a question,
every `@key` a fact of the oracle). The size and mix rules, the frozen 110, the facets and the spec
rebuild belong to the team bank and are skipped. The file needs the same columns as `questions.tsv`.
The results, the misses, the summary and the Urdu review sheet go to
`backend/target/ai-eval-<name of the file>/`, never into `backend/target/ai-eval/`.

**The oracle dump (`--oracle-dump FILE`).** One offline test through cargo (no model, no key, no
network). It writes JSON: `facts` (every key of `oracle-keys.txt` with its value, the rich year and the
thin books), `parties` (id, code, role, control, Urdu and English names, tag, city), `items`,
`partners` (with the share), `gardens`, `trips` (number, day, status, market), `trip_purchases` (trip,
purchase number, day, item, qty), and `worlds.year` and `worlds.day` with the same lists for the PRD
worked day. It builds the rich year twice, which takes a few minutes in a debug build.

To write `oracle-keys.txt` again after changing the books or the oracle:

```bash
MANDI_ORACLE_MANIFEST="$PWD/scripts/ai-eval/oracle-keys.txt" \
  cargo test -p mandi-assistant --test robustness_live dump_the_oracle   # from backend/
```

Tests that need no key (from `backend/`): `cargo test -p mandi-assistant --test robustness_live`.

## The D7 bank (plan 59 task B1)

`realtime-d7.tsv` is a bank of its own (`--bank scripts/ai-eval/realtime-d7.tsv`), built by `build-bank-d7.mjs` from
`bank-spec-d7.mjs`; never edited by hand. Its goal is to FIND flaws in real-time figures, not to pass:

- category A only (a figure, a list or a count from the books); no help, advice, forecast or what-if;
- 455 questions in 193 groups. Every question has `mix=short` (one figure asked, a plain wording: the common
  questions of the shop, about half of the bank) or `mix=picky` (several figures or filters at once) in its
  filters, so the share answered with no model call is honest for both. `--mix short` or `--mix picky` runs one
  kind;
- the rich year, every expected figure an oracle fact (`@key`), never typed;
- simple questions (one figure, the plain wording the Owner uses) and complex ones (a buyer and a seller and an item
  and a period, cash or credit, a quality: every filter must hold);
- Roman Urdu, Urdu letters and English in even thirds, about 15% to 20% misspelt, new wordings (no question of
  `questions.tsv` is repeated);
- zero days and zero filters ("nothing happened") must say 0, never "not found";
- Umar's own example is T01: "پچھلے مہینے 14 تاریخ کو اکرم نے کیا کیا خریدا، موقع پر کتنا نقد دیا اور کتنا ادھار
  رہا". The rich year has two Akrams, so the app asks "which one" and the harness clicks Akram Toll Tax
  (`click=1` in the filters; the answer after the click is scored).

Group `N01` to `N25` are one question for each finding of the reviewer study (plan 59 Section 5.5, R1 to R13b: the
newest line, a quality with the last rate, a بکری number, a voucher number, the same name in two accounts, a packing
word, a cash or credit word, uncommon period words, a typed number that must not become the answer). `S01` to `S70` are
the common short questions.

The newest sale line and the last rate (plan 59 Section 11, the lead's rule): "rate" is the SELLER rate; the newest
line has the latest posting date, then the highest بکری number, then the highest line number, never the row id. Every
`last_rate` fact of the oracle (`ix.*`, `ixq.*`, `veg.*`, `d7.sale.*`) follows it. The rich year has three more بکری
for it (`seed.rs`, `d7_last_rate_cases`, grapes, out of season in November, so they are the only grapes lines of the
week): one of 03-11 with a LATER SALE on 05-11 (4,600), one of 05-11 whose lines are EDITED last (so they have the
newest row ids, 5,200), and the highest بکری of 05-11 with four lines and an edit (its last line, 5,000). The newest
grapes rate is 5,000; a sort by row id says 5,200 (for Shahid Raza: 4,900, against 5,050). All these lines are to cash
buyers, so the dues of the books are what they were without them. The oracle checks both
rates when it is built, so the cases keep telling the two rules apart. Note: an edit does NOT keep a line's number in
this app; it writes every line again with the next numbers, in the same order.

`realtime-d7-coverage.md` (written by the builder) lists every kind of record and figure the app keeps, with the
question ids for each, and says why a kind has none (cities and addas, settings, working tables).

The oracle facts of this bank are the `d7.*` family of `oracle.rs` (`d7_facts`): SQL on the books, never the
assistant's tools. `d7.bd.<buyer>.<day>.*` is one buyer's day (each item with qty, rate and amount, the cash on the
spot, the credit, receipts, what is left, the balance before and after), cross-checked against the app's own day
sheet R35 when the oracle is built; `d7.sale.<combination>.*` are the filter combinations (lines, qty, gross,
commissions, laga, charges, average, highest, lowest, last and first rate and date, how many buyers and sellers),
cross-checked against view A where they overlap; and `d7.recv.*`, `d7.stock.*`, `d7.bikri.*`, `d7.partner.*`,
`d7.last_bill.*`, `d7.cmp.*`, `d7.exp.*`. A mistake in the SQL stops the oracle from being built.

The rich year holds what Umar's example needs (`seed.rs`, `d7_richer_cases`, added at the very end, add only, through the
app's own saves, on past days that are still open): a buyer with BOTH a cash line and a credit line on one day
(Naveed Traders, 14-10-2026: 10 banana for cash, 12 apple on credit), a city (Kasur) on three new accounts with a
balance, a چکتا buyer of the shared چکتا کھاتہ who pays in full on the spot (Aamir) and a buyer with a balance (Nadir), a separate khata
(Family Khata, three entries), a line with ریٹ مختلف (22-10-2026, seller 4,300, buyer 4,350, the newest apple line of
its day) and a later sale of an older بکری's leftover on a day with a newer بکری of the same item (mosambi,
19-10-2026: the newest line is 2,550, the row id says 2,400). The oracle checks the rule against the row id for both
cases, and the city and the cash and credit day against R21 and the day sheet R35, when it is built.

To write a new copy of the books for reading with a SQL tool (the facts of this bank were written that way):

    MANDI_YEAR_DB_OUT=/some/folder/year.db cargo test -p mandi-assistant --test robustness_live -- --nocapture --exact export_the_year_snapshot   # from backend/

## Key limit (`--key-limit`): what is left on the key, before a live run

    node scripts/ai-eval/run.mjs --key-limit

Only the lead runs it. It reads the key from the app's own store the way the live run does, calls
`GET https://openrouter.ai/api/v1/key` once and prints ONLY `limit`, `usage`, `limit_remaining` and `is_free_tier`
(lines that start with `KEY-LIMIT`). It never prints the key, a header, the label of the key or the rest of the
reply. A 403 or a 402 prints its reason (the provider's words, the key cut out) and stops (exit 1); no key saved is
exit 3. The work is the ignored test `key_limit_left` in `robustness_live.rs`; `check-modes.mjs` checks from the
source that none of its print lines names the key, a header, the label or the body.

## Offline (`--offline`): no key, no network

    node scripts/ai-eval/run.mjs --bank scripts/ai-eval/realtime-d7.tsv --offline

It runs the same path as the live run (`run::ask` on the seeded books) with a model that never answers: every call ends
the way a call ends when the computer has no internet, with no wait. So the app's own understanding, its plans, its
reads before the first call and its instant answers (PRD FR-7.90) run for real on the rich year and are scored by the
same scorer; only what needs the model ends as `offline`. Nothing here reads, opens or prints the key: the runtime has an
in-memory placeholder that is not a key, the real client (`OpenRouterClient`) and the real key store are never built, and
`robustness_live.rs` has a test that proves it (`offline_mode_never_builds_the_real_client_or_opens_the_key_store`).
The test is `offline_questions`, not ignored but it does nothing unless `MANDI_EVAL_OFFLINE=1` (which `--offline` sets).
Results go to `backend/target/ai-eval-<name of the bank>-offline/` (`ai-eval-offline/` for the team bank), never into
the live folders, and no Urdu sheet is written.

The report (`offline-report.txt`, `offline-summary.json`) puts every question in one class:

| Class         | Meaning                                                                                                                                                                                                                            |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| LOCAL RIGHT   | the app answered by itself (no model) and every expected figure is there                                                                                                                                                           |
| LOCAL WRONG   | the app answered by itself and the scorer says no. Always a defect. Split in `wrong figure` (it shows none of the figures asked), `incomplete` (a right figure, an asked figure left out) and `other` (a button, a log or a route) |
| NEEDS MODEL   | the app could not answer alone. The estimate of the model calls a live run needs: 1 when the app read the books before the call, else 2, 3 when a name is to be found first                                                        |
| ASKED/REFUSED | the app asked or refused with no model ("which one", "need dates", "not found", "outside the books"); `expected` or UNEXPECTED (the question wanted an answer)                                                                     |

It also gives the share by section, by language, by number of figures asked (the 60% target is for plain questions),
the wordings of one meaning that end differently, the speed of the app's own answers (a debug build: only a guide),
and the estimated cost of a live run. The exit code is 1 when any local answer is wrong. It also writes `offline-detail.tsv` (one line for each question: its
class, the model calls it would need, the time the app took) and a SPEED AND THE TARGETS block for every question, the
common short ones and the picky ones: the share answered right with no model call (target 60%), with no call or one
(85%), the app's own time (measured), and the wait with the model PROJECTED for an assumed time of one call (1.5, 3 and
6 s; an assumption, not a measurement) against p50 2 s, p95 8 s and nothing over 25 s.

If the offline process is stopped from outside (a signal: exit code null) the runner asks again only the questions that
have no row yet, up to 8 times; every row of every try goes into `results.json`.

The date and name parts of a reply are part of the scored text (a date part is `{type: 'date', iso}`; the runner
read `date` and showed an empty date, "مورخہ ۔", until 07-11-2026), and a date figure of the bank is now a real check:
the date must be shown, as a date part or as a date cell.

## The gap dump (plan 59 R2)

`backend/crates/assistant/tests/robustness_gap.rs` answers "why did this question need the model?" for any bank. It does
nothing unless both variables are set, so a plain `cargo test` skips it:

    MANDI_GAP_BANK=$PWD/scripts/ai-eval/realtime-d7.tsv MANDI_GAP_OUT=/some/gap.tsv \
      cargo test -j 4 -p mandi-assistant --test robustness_gap -- --nocapture      # from backend/, about 4 minutes

Each line of the output has the intent, whether the app was sure, the figures, the doubts, what each name led to, the period,
the flags that switch the instant plan off, whether `plan_reads` planned any read, and the unknown word found by dropping
words one or two at a time (a guide to the wording, not a proof). Join it with `offline-detail.tsv` on the id to rank the
causes. `realtime-d7-gaps.md` is that ranking for the main D7 bank.

## The partner overview button (plan 59 D331)

A capital figure is read through R31, but the R31 page holds no capital cell, so its button is the partner overview SCREEN
(`kind: screen`, `route: partner_overview`). The runner's log-against-buttons rule (`figureLogProblems`) accepts that screen
button for a log read of R31 for ONE partner, with the same result id, and for nothing else: a `partner_overview` button
with no such read is a problem, and so is another screen button on an R31 read.

## The speed report (plan 54)

Every run prints a SPEED block: for each category, and for the help and print questions, how many model
calls a question needed (none, one, two, three or more), the share answered in one call or none (category A
has a target of 70%), the share of help and print answered with no model call, the time of each stage
(before the first call, call 1, call 2, call 3 and after, between calls, after the last call, the app's own
tools, and the whole question when there is no model call), and the reasoning tokens of each call. The stage
times come from `calls[].at_ms`, written by the harness. The block ends with the hard limit of plan 55: no
question over 25 s, and local help or print over 3 s. `summary.json` keeps the same numbers (`speed`).

## Slow and lost lines (plan 55 gate 6)

`backend/crates/assistant/tests/robustness_slow_line.rs` puts a bad line in front of a scripted model: a call
can be delayed or dropped. It checks the hard limits of plan 55 Section 3: no question over 25 s, the last
call at 20 s with the tools off, what was read shown at 25 s (never an error), and no answer lost after a drop.
The tests that really wait are `#[ignore]`d with their reason; the fast drop tests run every time.

## What a run writes

- `backend/target/ai-eval/results.json`: the answers, the tool calls, the calls, the tokens, the log fields
  of every question (`calls_json`, `answer_json`, `finish_reason`, `total_millis` ...) and the oracle;
- `backend/target/ai-eval/misses.txt`: every miss with the question, what was expected, the answer as shown,
  the tools, the calls and the time. This is the file to read for the loop (item 14);
- `backend/target/ai-eval/summary.json`: the numbers of each category, for "twice in a row";
- `scripts/ai-eval/urdu-review.tsv`: the review sheet.

## The scorer: what a miss is

An expected figure that is not shown fails the question. A wrong figure is always a miss. Rules, and what
each one loosens or tightens compared with the first 110-question scorer:

Loosened (each one asked for, or an honest second reading):

1. `?` optional figures: the count in Q016 to Q019 and the percent in Q081 (lead, from the 81-of-110 read).
2. A zero said in words passes when the expected value is 0, the tool route matched and no other figure is
   shown (lead). A zero beside another figure, or with another route, still fails.
3. `a|b` alternatives, only where the question has two true readings: Faisal's purchases (all time or this
   year), total dues (with or without the dead amount), the garden's payments (all, or the shop's part), the
   year end (the months left, or the whole year), the doubled pace (same two), +10% (the new total, or the
   10%), and the name lists of the advice questions (S02, S03, S04, S08: the two or three parties or heads
   that all fit).
4. Groups agree on the required figures only. The old rule compared every figure shown, so one wording
   adding a figure was a disagreement (lead).
5. Q087 expects `first=Karim 10160` without a side; Q034 says "آج" (lead).

Pending (plan 52 Section 8.2, lead, 03-11-2026):

11. A question whose `filters` hold `asked_role=` or `found_as=` is **PENDING** while the reply has neither field
    (`asked_role`, `found_as` at the top of the reply) and its row comes from an older build. The harness marks
    every row of a build that has the fields with `asked_role_field: true`; for such a row a `not_found` with no
    `asked_role` is a miss, never pending. Pending is neither a pass nor a miss: it is not in the score, it is
    listed on its own, `--misses` asks it again, and the run cannot say "meets" while any is pending. Once a
    field is there, `asked_role` must be the asked role (seller, buyer, item, partner, garden, trip or any),
    and `found_as` (`{key, id, role, name}`) must have the role the name has; a made-up name must have no
    `found_as`.

    A name found in another role (N03 Faisal asked as a seller, N04 Karim asked as a buyer) leaves the
    question waiting. The harness clicks his name (an `assistant_pick`, as the panel's "show his purchases"
    button does) and keeps the reply before the click in the row as `first_reply`. The scorer judges the
    outcome and the roles on `first_reply`, and the figures on the answer after the click: his purchases (a
    buyer) or his sales (a seller), never the other. A source filtered to him in the other role, or no
    answer after the click, is a miss.

12. A print question is **PENDING** while the answer has no `how` key (plan 55 Section 7) and the row is not
    marked `print_contract: true`. The button is `answer.sources[0]` (kind report: the report id, the party
    or seller, the exact `from` and `to`, or `as_of` for the day slip R35); a shortcut instead of dates is a
    miss. While pending, the button check is only a note. Once `answer.how` is in the reply it must hold the
    steps, made from the button's own values: open-report (the same report), set-party (his real role, name and
    code; none for a report with no party), set-day or set-dates (the same day or dates) and press-print last.
    A print request answered as a change request is a miss.

13. **Speed is part of "meets"** (plan 55 Section 3, lead 04-11-2026). "Meets" is plan 52 Section 2 and: the
    speed budget by kind on the time the Owner waits (how-to, print, navigation, no such party, not in the
    app: p50 1 s, p95 2 s; figures and lists: 5 s and 10 s; advice, forecasts and what-ifs: 10 s and 20 s; the
    mixed and hard questions keep 6 s and 12 s); the hard limits (no question over 25 s, how-to, print and
    navigation under 3 s); and category A answered in one call or none at least 70% of the time. The outcome
    `time_limit` (a slow line ended at the limit with the tables that were read) is a safe end, not an
    answer.

14. **Every button against the data** (plan 55 gate 3). The harness opens each report or view an answer offers as
    a button, with the button's own filters, through the same backend command the screen uses (a party card
    opens his ledger R13), and writes the figures it holds into the row (`buttons`). A figure of the answer
    (or, for a calculated figure, the inputs of its working; a number of the question is left out) that is not
    in what the buttons open is a miss: a wrong place or wrong filters. A button that fails to open is a miss.
    The dashboard tiles are not run: an answer whose only button is the tiles is a note, not checked.

15. **Local answers** (plan 55 Section 2, FR-7.90; finish_reason `local`, no model call). The app writes what it used
    into `tool_calls_json`, each entry marked `"local": true` with an id `l<n>`: `app_help` with `input.topic`, one
    `report` entry for each button in order with `input.id` and `input.filters` (only the filters that are set),
    `tiles`, `find` (a name nobody has). A help question's topic is read from the entry. A local answer with no
    such entry (a build before this) is PENDING: neither a pass nor a miss, and the run cannot say "meets". For a
    print answer the report entries must be the same as the buttons (the same report and filters), or it is a miss.

Tightened:

6. Strict names (`!`): the right party, item, month or head must be named.
7. A target month belongs to its method (`pace_month` is the same-pace month).
8. General advice must be marked and carry no figure, table or estimate; advice on data must have a source
   and not be marked general. A reply that is marked general although the books have data is a miss.
9. A change request must point to the right entry screen (`route=`).
10. "Not understood after reading" and network failures are counted on their own lines and must be 0.

The internet: when a question ends offline (or on a connection that dropped) the harness waits, up to
10 minutes, until the host can be reached, and asks the SAME question again on a fresh copy of its books
(up to 5 times). A question that stays lost is marked `network_lost`; the scorer never counts it as a result,
lists it on its own ("LOST TO THE NETWORK") and `--misses` asks it again. A timeout is the service and stays a
real miss.

Time: the model's time only. Questions that end on a network failure or a timeout are counted apart. A
follow-up counts its own time. The fixed part of the prompt is the prompt tokens of the first call of a fresh
question less the question itself (`user_tokens`), for the questions whose first call carried nothing the app
read for it. What the app reads before the first call (the guide of a how-to, a period, the tiles) is added to
that call; it is reported apart as "READ AHEAD", not as fixed part.

A category meets plan 52 Section 2 when: A and C have no wrong figure and at least 98% answered; B at least
95% good; D at least 98% the right topic; E only safe ends; and its p50 and p95 are inside its target.
