# ai-eval: the live check of "ask the app"

Plan 52 items 12 and 13. Only the lead runs the live part, with Umar's key. Nothing here reads, prints
or writes the key.

## Files

| File | What it is |
|---|---|
| `run.mjs` | The runner and the scorer. `--check`, `--self-test`, `--category`, `--world`, `--only`, `--threads`, `--results`. |
| `questions.tsv` | The bank (367 questions). Built, never edited by hand. |
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

Tightened:

6. Strict names (`!`): the right party, item, month or head must be named.
7. A target month belongs to its method (`pace_month` is the same-pace month).
8. General advice must be marked and carry no figure, table or estimate; advice on data must have a source
   and not be marked general. A reply that is marked general although the books have data is a miss.
9. A change request must point to the right entry screen (`route=`).
10. "Not understood after reading" and network failures are counted on their own lines and must be 0.

Time: the model's time only. Questions that end on a network failure or a timeout are counted apart. A
follow-up counts its own time. The fixed part of the prompt is the prompt tokens of the first call of a fresh
question less the question itself (`user_tokens`).

A category meets plan 52 Section 2 when: A and C have no wrong figure and at least 98% answered; B at least
95% good; D at least 98% the right topic; E only safe ends; and its p50 and p95 are inside its target.
