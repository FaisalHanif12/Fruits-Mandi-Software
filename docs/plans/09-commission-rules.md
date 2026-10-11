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
