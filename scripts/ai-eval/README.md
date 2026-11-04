# ai-eval: the live check of "ask the app"

Plan 52 items 12 and 13. Only the lead runs the live part, with Umar's key. Nothing here reads, prints
or writes the key.

## Files

| File | What it is |
|---|---|
| `run.mjs` | The runner and the scorer. `--check`, `--self-test`, `--category`, `--world`, `--only`, `--threads`, `--results`. |
| `questions.tsv` | The bank (1,126 questions). Built, never edited by hand. |
| `questions-110.tsv` | The first 110 questions, frozen. They keep the D6 mix. |
| `bank-spec.mjs` | The new questions, one entry for each meaning, with its wordings. |
| `bank-spec-a2.mjs` | Facet A2: the 120 real-time figure questions (plan 55 Section 10), imported by `bank-spec.mjs`. |
| `bank-spec-mix.mjs` | Facet MIX: one question with a real figure AND a forecast (Umar, 05-11-2026). |
| `bank-spec-cap.mjs` | Facet CAP: the cells of the capability matrix that had no question (plan 55 Section 11): open and write to every screen. |
| `bank-spec-cap-more.mjs`, `-registers.mjs`, `-nosuch.mjs`, `-print.mjs`, `-forecast.mjs` | The other parts of facet CAP: reports, views, tiles and records with oracle figures; the registers and lists; "no such record" for every row that names one; print and open of the mapped reports; the forecast and the mixed cells. |
| `capability-rows.mjs`, `build-capability.mjs`, `capability.tsv` | The capability matrix (plan 55 Section 11): every thing the app holds against every question type, read from the code. |
| `build-bank.mjs` | Builds `questions.tsv` from the two files above. `run.mjs --check` fails if it is out of date. |
| `oracle-keys.txt` | The facts the oracle gives (the `@key` of a question). Checked by a Rust test. |
| `urdu-review.tsv` | The sheet for the Urdu reviewer, written by every run. |
| `check-modes.mjs` | A scripted check of `--bank` and `--threads` (no model, no key, no cargo). |

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
estimate shown as a figure of the books, is a miss. A group whose estimate has no route yet (a head, cash, a
party's balance: gaps F2, F5, F4) carries `pending=<reason>` in its filters: it is PENDING (neither a pass nor a
miss) and the run cannot say "meets" while it waits.

Facet **CAP** (`facet=CAP`): the questions that fill cells of `capability.tsv`. This file's first part holds the
screens: 29 groups that OPEN a screen (answered, the guide topic and the screen route are both checked) and 11
that ask to WRITE (change_refused with the entry screen). The reports and views with oracle figures follow.

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
node scripts/ai-eval/run.mjs --results backend/target/ai-eval/results.json   # score a file again
node scripts/ai-eval/run.mjs --threads 1    # the live run on one line (default 3; see below)
node scripts/ai-eval/run.mjs --bank FILE --check   # another bank (a hidden one): the per-row rules only
node scripts/ai-eval/run.mjs --bank FILE    # ... run live and scored the same way
node scripts/ai-eval/run.mjs --oracle-dump out.json   # the oracle facts with values, and the names
node scripts/ai-eval/check-modes.mjs        # --bank and --threads behave as written (no model, no key)
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

14. **Speed is part of "meets"** (plan 55 Section 3, lead 04-11-2026). "Meets" is plan 52 Section 2 and: the
    speed budget by kind on the time the Owner waits (how-to, print, navigation, no such party, not in the
    app: p50 1 s, p95 2 s; figures and lists: 5 s and 10 s; advice, forecasts and what-ifs: 10 s and 20 s; the
    mixed and hard questions keep 6 s and 12 s); the hard limits (no question over 25 s, how-to, print and
    navigation under 3 s); and category A answered in one call or none at least 70% of the time. The outcome
    `time_limit` (a slow line ended at the limit with the tables that were read) is a safe end, not an
    answer.

13. **Every button against the data** (plan 55 gate 3). The harness opens each report or view an answer offers as
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
