# mandi-domain

Business rules as pure functions (Code Standards B4). Never touches the database, files, clock or network.

- May depend on: `mandi-core` only.
- Money lives here, so every `+ - * /` on integers is a checked call (`clippy::arithmetic_side_effects` is denied for the whole crate, Code Standards B11).
- Later plans add commission rule resolution (PRD 11), بکری line and lot maths (PRD 12.5, 22), journal building (PRD 6, 12.6), partner split (PRD 17) and oldest-first matching.
- Never: database, files, clock, network (Code Standards B1).

## Modules

| Module | What it does |
|---|---|
| `rules` | Which charge rule wins (PRD 11.3): `RuleSet`, `resolve`, `charge_amount` with its working, the one `newer_key` order. |
| `partners` | The partner split (PRD FR-6.5, C9 to C12): `share_bases` takes the income base (head 04) and the expense base (head 05) of an entry, `split_posting` splits each with `split_by_bps` and makes expense parts negative; a negative base is split as a positive and negated. Shares that do not total 10,000 bps give `SharesNot100`. |
| `bikri` | `calc_posting`, the ONE function that works out a بکری for the live preview and (plan 13) the save (Code Standards B5): lines (C1, C5, C19), the three charges of each line from the rule that wins or the value typed on the line, lot charges, the totals C6 to C9, remaining quantity, rate hints, and every check (quantity over the lot, credit limit, rate check, one-time person, payment type, amount limits). Each amount carries its `Working`. A row is worked out only when its lot is complete, and the summary and the journal use exactly those rows. `PostingKind::Later` charges no lot charge and no per-lot commission (PRD FR-12.17, used by plan 16). |
| `cash` | `check(before, change)`: cash in hand after an entry and whether the user must confirm it (below 0 AND the entry takes cash out, PRD FR-6.7); `net_change(entry, cash)` reads the change from the journal entry, the one source for every module. |
| `ledger` | Journal entries as pure functions: `build_entry`, `build_reversal`, `build_opening` (PRD FR-6.2, FR-6.3, FR-10.9); `ledger::bikri::build_bikri_entry` builds the journal of a بکری posting, sale part and cash part (PRD FR-12.9). |
| `auth::validate` | Sign-in name, password, display name, idle lock minutes, text size, last screen (PRD FR-8, NFR-SEC-1). |
| `auth::lockout` | The lockout steps: 5 wrong tries lock 60 seconds, 10 lock 300, 15, 20, 25 ... lock 900 (PRD FR-8.2). |
| `auth::recovery` | The Owner recovery code: 16 characters from an alphabet without look-alikes, shown as `XXXX-XXXX-XXXX-XXXX` (NFR-SEC-12). |
