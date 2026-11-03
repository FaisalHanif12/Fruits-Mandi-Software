# ai-eval: the live check of "ask the app"

Plan 52 items 12 and 13. Only the lead runs the live part, with Umar's key. Nothing here reads, prints
or writes the key.

## Files

| File | What it is |
|---|---|
| `run.mjs` | The runner and the scorer. `--check`, `--self-test`, `--category`, `--world`, `--only`, `--threads`, `--results`. |
| `questions.tsv` | The bank (434 questions). Built, never edited by hand. |
| `questions-110.tsv` | The first 110 questions, frozen. They keep the D6 mix. |
| `bank-spec.mjs` | The new questions, one entry for each meaning, with its wordings. |
| `build-bank.mjs` | Builds `questions.tsv` from the two files above. `run.mjs --check` fails if it is out of date. |
| `oracle-keys.txt` | The facts the oracle gives (the `@key` of a question). Checked by a Rust test. |
| `urdu-review.tsv` | The sheet for the Urdu reviewer, written by every run. |

The oracle itself is Rust: `backend/crates/assistant/tests/robustness_year/` (`seed.rs` builds the books
through the app's own save functions, `oracle.rs` asks the app's own reports and analytics).

## The bank

Five categories (plan 52 Section 2), each in Roman Urdu, Urdu and English, with misspellings and groups of
the same meaning:

| Cat | Questions | What |
|---|---|---|
| A | real-time data | every section: item, seller, buyer, partner, period, cash, dues, stock, garden, trip, fruit, expense, capital, day, month |
| B | suggestions | with data (from the books, not marked general) and on thin books (general advice, one call, no figure) |
| C | forecasts and what-ifs | year end, pace doubled, next months, targets, +10% |
| D | help | every one of the 38 topics of the guide |
| E | mixed and hard | follow-ups, misspellings, two parts, nonsense, injection, change requests, unknown names |

Three facets of Umar's request of 03-11-2026 (plan 52 Section 8.1) run through the categories. A question
carries its facet in `filters` (`facet=M`, `facet=day`, `facet=nosuch`), and each one has its Roman Urdu,
Urdu and English twins in a group:

| Facet | Questions | What |
|---|---|---|
| M, mixed language | 36 | Roman Urdu with English words ("Rahim ne this month kitna sell kiya"), Urdu letters with English words ("رحیم کی sale this month کتنی رہی"), English with Urdu mandi words ("how much bakaya does Faisal have"). Every section of category A at least once: item, seller, buyer, partner, period, cash, dues, stock, garden, trip, fruit, expense, capital, day, month. |
| day, a party's day | 15 | "X ne aaj / kal / 18 September ko kya aur kitna khareeda (becha)": a buyer today, a buyer yesterday, a buyer on a date with two items, a seller yesterday, a seller on a date with two items. Each item's qty and amount and the day's total, all from the oracle (`buyer_day.*`, `seller_day.*`). A buyer's amount is the value or the value with commission and laga (`with_charges`); both are true figures of the app. |
| nosuch, no such party | 14 | A made-up seller and a made-up buyer end in `not_found` with `asked_role`. A buyer asked as a seller ends in `not_found` with `asked_role=seller` and `found_as=buyer`; a seller asked as a buyer the reverse. A misspelled real seller (Raheem for Rahim) is still found, with his figures (category A). |

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
node scripts/ai-eval/run.mjs --results backend/target/ai-eval/results.json   # score a file again
```

To write `oracle-keys.txt` again after changing the books or the oracle:

```bash
MANDI_ORACLE_MANIFEST="$PWD/scripts/ai-eval/oracle-keys.txt" \
  cargo test -p mandi-assistant --test robustness_live dump_the_oracle   # from backend/
```

Tests that need no key (from `backend/`): `cargo test -p mandi-assistant --test robustness_live`.

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
    (`asked_role`, `found_as` at the top of the reply). It is neither a pass nor a miss: it is not in the
    score, it is listed on its own, `--misses` asks it again, and the run cannot say "meets" while any is
    pending. Once a field is there, `asked_role` must be the asked role, and `found_as` (a word, or a record
    with `role`, `type`, `party_type` or `kind`) must be the role the name has; a made-up name must have no
    `found_as`.

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
