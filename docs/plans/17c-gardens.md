# Plan 17c: gardens (باغات)

Version 1.0, 23-10-2026. Lead: tech-lead. Brief: `docs/deliverables/D4R-refresh-v1.6.md` Section 2 "17c-gardens" (full text, worked checks E2E-40 and E2E-40b). Branch `feature/refresh-v16`. Needs plan 17b done first (voucher screen and paged lists in their final shape) and plan 16b (ventures code).

## 1. Goal

The Owner opens a garden, pays installments and costs (each partner's part from the shop's cash or his own pocket), sells its fruit in بکری or another shop, sees its running balance by date, and closes it so its money goes to its partners by their %.

## 2. Requirements

PRD 12.11 (FR-12.34 to FR-12.38), FR-12.29, FR-12.30, C23, C26, C22, FR-5.2 (باغات, main bar item 5), FR-7.67, 9.1 (data for the tile), FR-10.4 (0306), FR-10.9 (no opening on 0306), 17.3 (open gardens and trips under each partner's card), FR-17.10 (garden lines), FR-6.7, FR-19.11, E61 to E66, E69, A54 to A57, A61 to A63; Architecture 6 and 15.5; money agent Section 5.5. Acceptance: D4R-A4, D4R-A5, D4R-A6, and the garden part of D4R-A8.

## 3. Worked numbers

E2E-40 and E2E-40b as in the D4R file Section 2 (17c) and PRD 27.2; `python3 .claude/skills/requirement-change/scripts/worked_day.py` prints them. Key values: parts 18,000 / 12,000, then 12,000 / 8,000 and 3,000 / 2,000; الف paid 33,000 from the shop (cash 17,000), ب 22,000 himself; sale of 100 at 600: ارشد pays 64,400, the garden's account جمع 56,400, commission 8,000 split 3,200 / 2,400 / 2,400; other shop 10,000, cash 27,000; money in 66,400, running balance 11,400; closing 66,400 to 39,840 / 26,560, the garden's account 0; partner totals 10,040 / 28,960 / 2,400; trial balance 106,400. E2E-40b: loss −45,000, closing 10,000 to 6,000 / 4,000, الف −27,000, ب +4,000, cash 27,000.

## 4. Contract

`backend` T1 writes the exact types (commands `gardens_create`, `gardens_list`, `gardens_get`, `gardens_summary`, `gardens_entry_save` for installment, cost and other-shop sale, `gardens_entry_cancel`, `gardens_close`, `gardens_reopen` (undo the closing), the garden lot in the بکری lot-seller list, `partners_overview` rows gaining his open gardens and trips, statement garden lines; error codes including `ERR_GARDEN_CLOSED`); the lead copies them into Section 10. Reuse the ventures, venture_entries and venture_entry_parts tables and the C23 split code of plan 16b. No migration.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, error codes, stubs, `pnpm gen:types`, command tables | `backend` | plan 17b T1 |
| T2 | Logic with the money-calculations agent before and for review: create, installments and costs (FR-6.7 on shop parts), garden lot in بکری, other-shop sale, running balance, closing and undo, closed-garden refusals, partner overview extras, statement lines, lists paged, goldens E2E-40 and E2E-40b, property tests, fixtures | `backend` | T1 |
| F1 | Main bar item باغات (item 5), gardens list, new garden form (shares, then sources, then details), one garden screen with type list قسط, باغ خرچ, دوسری دکان میں فروخت, each partner's part before save, running balance by date, close with confirm and undo, garden in the بکری lot-seller picker, open gardens and trips under each partner's card, Fluent ur and en | `frontend` | T1 |
| F2 | Wire to T2 fixtures, shape tests, keyboard-only and language-switch tests, screenshots at three sizes, Windows spec | `frontend` | T2 |

## 6. Acceptance checklist

D4R-A4 (E2E-40, E2E-40b and the venture property tests), D4R-A5 (keyboard in both languages, main bar, partner card), D4R-A6 (closed-garden refusals and undo, partner inactive refusal), D4R-A8 garden lines (الف's E2E-40 lines −18,000, −12,000, −3,000, +3,200, +39,840, ending 10,040).

## 7. Out of scope

Fruit taken (17d), R36 and the garden tile and analytics views (D5).

## 8. Open questions

None (A54 to A57, A61 to A63 are built as their defaults).

## 10. Contract as built in T1

Written by the lead from the backend T1 hand-off of 25-10-2026 (`pnpm gen:types` wrote 477 files, 31 new). Module `core/src/gardens.rs` (its doc table lists every command). 13 commands, all registered (build.rs, lib.rs, capabilities, the app test tables), answering `ERR_NOT_IMPLEMENTED` until T2:

- `gardens_defaults` gives `GardenDefaults` (partners with the firm's shares and default source shop, A56). `gardens_create_preview(GardenInput)` gives `GardenCreatePreview`; `gardens_create(GardenInput)` gives `GardenCreated {row, first_entry_id, journal_entry_id, preview}`; `gardens_set_partners` gives `GardenRow` (only before the first entry, `ERR_VENTURE_SHARES_LOCKED` after).
- `gardens_list(GardensListInput {from, to, status, partner_id, search, offset, limit})` gives `GardensList {rows, total_count, totals {rent, paid, money_in, balance, rent_due}}`. `gardens_get({id, history_offset, history_limit})` gives `GardenDetail {row, history[], history_count, partner_results[]}`. `gardens_summary` gives `GardensSummary {open_gardens, balance_total_paisa, rows[{id, number, names, balance, rent_due}]}` (shape kept stable for the D5 dashboard tile).
- `gardens_entry_preview` and `gardens_entry_save` take `GardenEntryInput`, tagged `installment`, `cost` or `other_sale`, and give `GardenEntryPreview` and `GardenEntrySaved`; `gardens_entry_cancel` (by reversal while the date is open; refused while the garden is closed).
- `gardens_close_preview` and `gardens_close({id, version, date})` give `GardenClosePreview {numbers {balance, total_cost, money_in, result, credits, partner_results, posts_entry}, earliest_date}` and `GardenClosed`; `gardens_reopen` (undo) gives `GardenReopened`.
- Types worth noting: `GardenRow` has the account (0306), rent, installments, costs, paid (shop and pocket), `rent_due` (signed), `money_in`, `other_shop`, `balance` (running balance, with its CalcInfo working), `account_balance`, `last_date`. `GardenHistoryRow` is ONE list (installment, cost, other_sale, sale, fruit, close), oldest first, with its effect and running balance on each row. `GardenPartnerResult.net` = money back minus paid from shop minus paid from pocket (derived from stored parts, never `split_by_bps` of the result). `PartnerOverviewRow` gains `open_ventures: PartnerVentureCard[]` (garden or trip, his %, paid shop and pocket, money back, balance or result), empty until T2.
- New error codes (126 in all): `ERR_GARDEN_CLOSED`, `ERR_GARDEN_NOT_CLOSED`, `ERR_DATE_BEFORE_GARDEN` (value `garden_date`), `ERR_CLOSE_BEFORE_LAST_ENTRY` (value `last_date`).

Decisions (money agent and lead, 25-10-2026): a loss closing stores `amount_paisa = |B|` (the CHECK says at least 0) and takes the sign from the signed parts, no migration; the closing entry is built by its own `close_entry` (`receipt_entry` drops negative parts); the closing and its undo are left out of "money in"; reads of parts join on entry status saved. The closing date must be an open date and not before the garden's last entry or lot posting (new code, listed for Umar); rent due is signed; garden and trip costs and other-shop sales share one entry number series (no visible entry number on the garden screen). Robustness-owned file touched mechanically: `app/tests/tauri_conf.rs` (13 `allow-gardens-*` lines). Merge note: Run B also edits `build.rs`, `lib.rs` and `capabilities/default.json`: keep both sides.

**Additions after T1 (lead, 25-10-2026, from the T2 hand-off).** `GardenInput` gained `paid_now_reason` (the screen pre-fills the Fluent reason text when a first installment is typed; the backend holds no user text). New error code `ERR_GARDEN_ENTRY_CANCELLED`. New working message id `working-garden-balance`. A closed garden is refused everywhere: entry, cancel, new lot, and edit, cancel or undo of ANY posting of a بکری that has a lot of that garden (stricter than the PRD, which names the garden's own posting); its account is out of every picker and search by its venture status. Closing: a loss closing stores `amount_paisa = |B|` and signs by its parts; a part of 0 makes no line; B = 0 posts nothing. A `جنرل` line on a garden account is refused (`ERR_FORMAT_INVALID`). Venture entry numbers are one series per kind shared by gardens and trips. Money review: first FAIL (HIGH: an edit of a بکری could take money back from a closed garden by changing the lot seller; MEDIUM: no closing property test), both fixed with tests, final re-check PASS. Known and accepted for the gate: the closing line of an UNDONE closing reads the payments as they are now; the history row of a closing shows `|B|` in `amount_paisa` (the signed parts are in `credits`). To do in the 17d follow-up commit (small): a service check so shares also lock when a lot sale has credited the garden account; a no-amount note line in the partner statement when a closing posts nothing (FR-17.10); force cash paid 0 and the advance tick off for a garden lot in the backend, not only hide them on the screen (FR-12.36).
