# Plan 09: Commission rules and which rule wins

Deliverable: D2, plan file 09. Branch: the run branch (goal run).

## 1. Goal

Charges are data, never code. The rules table holds every allowed charge of PRD 11.2; the defaults of PRD 4.2 are seeded; resolution picks the most specific active rule at six levels in under 1 ms with 10,000 rules; "Test a rule" shows the winner and the levels it skipped.

## 2. Requirements

PRD 11 (FR-11.1 to FR-11.10), 4.2 defaults, Settings "Charges" and "Overrides". NFR-SCL-6, NFR-MNT-5, NFR-SEC-5. Architecture 8 and 15.5. The money-calculations agent before and after. Acceptance: D2-A12, D2-A13.

## 3. Worked numbers (tests)

Seeded defaults (PRD 4.2): seller commission 6% (600 bps), buyer commission 6%, لاگا 8 per نگ as an item-group rule on پھل; no default for مزدوری, منشیانہ, فون (they resolve to "no rule", amount typed on the lot).

| Case | Winner | Amount on 20 نگ x 600 = 12,000 |
|---|---|---|
| seller رحیم, buyer اکرم, item کیلا (پھل) | seller 6%, buyer 6%, لاگا 8 per نگ (group) | seller 720, buyer 720, لاگا 160 |
| same with مولی (سبزی) | seller 6%, buyer 6%, no لاگا | 720, 720, 0 |
| rule "اکرم buys at 7%" added | buyer party level wins over the default | buyer 840 |
| PRD 11.3 examples: رحیم + کیلا 5% (party + item) | level 2 | 600 |
| رحیم any item 4% (party) selling سیب | level 3 | 480 |
| جاپانی پھل 30 per نگ (item) sold by کریم | level 4 | 600 |
| کریم selling کیلا | level 6 default 6% | 720 |
| line override 4% when Overrides is on | level 1, method manual | 480 |
| two rules at one level overlapping in dates | the newer wins, and the rule screen warns (FR-11.5) | |

Each of the six levels has a test that shows it winning over every level below it.

## 4. Contract

### Backend

- Table `charge_rules` (Architecture 15.5) with its CHECKs (exactly one of value_bps and value_paisa matching the method; only the combinations of PRD 11.2). Seeding of the defaults in the migration seed (they are data, NFR-MNT-5).
- `domain::rules::resolve(side, charge, party_id?, item_id, item_group, date, rules, line_override?) -> Resolution { rule_id?, level, method, value, skipped: Vec<SkippedLevel> }`, pure. Amount through `core::money` only (`apply_bps`, `per_unit`; per_lot as the value). Under 1 ms with 10,000 rules (criterion bench; in-memory index by side, charge, party and item).
- Commands: `rules_list { filter }`, `rules_save { id?, version?, side, charge_type, account_id?, item_id?, item_group?, method, value (text), from_date, to_date?, is_active, note }` (refuses combinations outside 11.2 with `ERR_RULE_NOT_ALLOWED`, percent over 100 `ERR_PERCENT_OVER_100`, negative or zero per-نگ values, `ERR_DATE_RANGE` when to_date is before from_date; returns overlap warnings), `rules_set_active`, `rules_test { seller_id, buyer_id, item_id, date, qty (text), rate (text) }` -> for each charge the winner, its level, the skipped levels and the amount with its working (FR-11.8). Settings `settings_set_charges` (the global defaults, written as rules) and `settings_set_overrides { allow_line_override, credit_limit_mode }`. Audit row on every rule change (S7). Sign-in check first.

- Added 12-10-2026 (frontend's request, accepted): `rules_get { id }` -> `RuleRow` for the edit form; `settings_get_charges` and `settings_get_overrides` for the two Settings tabs.
- Working shape (B30, FR-7.43): `Working { message_id, args, rule_id? }`. `message_id` is a Fluent ID (`working-percent`, `working-per-unit`, `working-per-lot`, `working-manual`, `working-none`), `args` are the formatted inputs and result (money as Paisa, rates as bps), and the rule's name or level for the "(اصول: ...)" part. The frontend never builds a working; it only renders this through `CalcInfo`.
- Error messages needed in ur and en once the codes exist: `ERR_RULE_NOT_ALLOWED`, `ERR_PERCENT_OVER_100`, `ERR_DATE_RANGE`.

- Decided 12-10-2026 (team default, open question for Umar in status.md): an item group only on a rule with no item and no party, because PRD 11.3 has no party + group level. The three global defaults are rows with `is_default`, changed only from Settings, Charges.

- Decided 12-10-2026 after backend's T2:
  - A zero value is allowed for per نگ and per lot (value 0 or more), so "no لاگا for this buyer" can be written as 0 per نگ. Negative values stay refused. The plan's earlier "zero per نگ refused" was the lead's wording, not a PRD rule.
  - The three global defaults are dated like every other rule (PRD 11.3: the rule date is the posting date). Changing a default in Settings ends the current default row on the day before the change day and adds a new row from the change day; a second change on the same day updates that day's row. So a back-dated posting on an open day, and "Test a rule" on a past date, use the default that was in force on that date. The one-default-per-side guard becomes "no two defaults overlap in dates". E2E-13b numbers do not change.
  - "Newer wins" (FR-11.5) is read as: the rule that starts later; on the same start day, the one made last. One function decides it. Open question for Umar (status.md).
  - The criterion bench for NFR-SCL-6 (10,000 rules, under 1 ms) on `mandi_domain::rules::{RuleSet, resolve}` is written by `robustness` at the end gate (D2-A13).

### Frontend

- Rules list (filters by side, charge, party, item, active), rule form (side, charge type limited to 11.2, party with `AccountPicker`, item with the plan 06 Picker, item group, method limited by charge, value as text with the right unit, dates, active, note), overlap warning, four states, keyboard only, both languages.
- "Test a rule" panel: seller, buyer, item, date, qty, rate; shows each charge's winner, level, the skipped levels and the amount with `CalcInfo`-style working from the backend.
- Settings "Charges" (defaults) and "Overrides" (allow line override; credit limit warn or block).

## 5. Tasks

T1, T2 (with the money agent before and after), T3, T4.

## 6. Acceptance checklist

1. Seeded rules match PRD 4.2; each of the six levels wins over the ones below it (D2-A12).
2. Rule resolution under 1 ms with 10,000 rules (D2-A13, bench output).
3. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check` exit 0.
