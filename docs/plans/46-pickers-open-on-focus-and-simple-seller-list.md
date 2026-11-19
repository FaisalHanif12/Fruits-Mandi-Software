# Plan 46: pickers open on focus, a simple lot seller list, and the line override note

Version 1.0, 01-11-2026. Lead: tech-lead. Asked by Umar on 01-11-2026 while testing بکری. Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`.

Umar's words, in short:
- "Keep it simple in the seller section: the sellers we added in the index or on the spot, or the پختہ کھاتہ seller."
- "When the blue focus comes into an input, in بکری or other pages, the dropdown must appear immediately."
- He also saw "Changing the Commission and Laga on a line is switched off" on an empty sale line, although the switch is on.

## 1. Requirements

PRD v1.14 (approved by Umar, 01-11-2026):
- FR-7.16: the list opens on focus, by keyboard too. While open: `↑` `↓` choose, `Enter` picks and moves on, `←` `→` and `Tab` leave, `Esc` closes. With the list closed, `↑` `↓` move between fields and rows. Replaces the v1.8 rule.
- 12.3 seller code, FR-12.26: the lot seller list shows sellers and پختہ کھاتہ بیوپاری always; اپنا مال only while a trip has a purchase with goods left; a garden only while it is open; the الگ کھاتے that exist.
- FR-11.10 (unchanged): the line override is on by default. The note "switched off" must show only when the Owner really turned it off.

Also kept: FR-7.14 and FR-7.15 (grid and form keys), FR-7.41 (recent first), FR-7.69 (add row in the list), NFR-USE-7 and 8, NFR-SPD-6, Code Standards F27 (keys only from `shared/keyboard` and `shared/grid`), F39.

No money rule, posting or golden number changes.

## 2. Contract

- **Seller list (backend).** The existing lot-seller search scope (`search_accounts`, lot seller) returns only usable entries, by the rule above. "Goods left" and "open" are read from the database at query time (the in-memory index is a speed aid only, B27), so a trip that sells its last نگ drops اپنا مال from the next search. No new command, no type change unless backend finds one is needed; if it is, backend tells the lead and frontend first.
- **Line override (frontend).** `LineDetails` takes the switch from the overrides setting (the existing settings read used by `OverridesTab`), not from the preview. Until the setting has loaded, the fields are not shown as switched off and the note is hidden.

## 3. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| B1 | Lot seller list by the usable rule. Integration tests: اپنا مال missing with no trip, present with a trip purchase with goods left, gone after the last نگ is sold or written off; an open garden present, a closed one missing; a الگ کھاتہ present; sellers and پختہ کھاتہ always present. Run golden_day, assist and the search tests. | backend | |
| F1 | Open on focus in the shared picker base (`shared/ui/PickerBase`, `Picker`, `AccountPicker`, `ControlAccountPicker`) and in the grid picker cells (`shared/grid`), with the key rules of FR-7.16. Filter boxes (report, analytics, list filters, find, rule test) follow the same rule unless that breaks typing into them; tell the lead if it does. | frontend | |
| F2 | Line override note and read-only state from the setting, not the preview. | frontend | |
| F3 | Tests: keyboard path on the بکری lot and line grids (Enter into the buyer cell opens the list; ↑ ↓ choose; Enter picks and moves to qty; Esc then ↓ moves a row), a form picker (voucher account), mouse path, language switch with a list open, override note hidden when the setting is on and shown when off. Screenshots ur and en at 1366 x 900 of an open seller list on a lot and the empty line details. | frontend | F1, F2 |

Targeted checks only (Umar's rule). The full `pnpm typecheck`, eslint on changed files and `pnpm i18n:check` must be clean. Existing keyboard tests that expected "no list on keyboard arrival" change to the new rule; list each one you change in the HANDOFF.

## 4. Acceptance

- In بکری, Enter onto the seller or buyer cell shows its list at once; Enter on a choice picks it and moves on.
- With Umar's data today, the lot seller list shows his sellers and پختہ کھاتہ (بیوپاری) only.
- An empty sale line no longer says the override is switched off.
- Golden tests unchanged.

## 5. Out of scope

New picker kinds; any change to commission rules.

## 6. B1 as built (01-11-2026), accepted

- The lot-seller scope of `search_accounts` and `recent_accounts` leaves out the unusable entries, read from the database on every call (`db::accounts::lot_seller_unusable`). No command, type or migration change.
- اپنا مال counts as usable only when an open trip has a saved purchase with نگ left for our shop; a purchase wholly for another shop does not count. The date to sell is not part of the rule.
- A typed full code (030119998) still resolves, as `useReadyLots` needs it; the save refuses an unusable one (ERR_REQUIRED, ERR_QTY_EXCEEDS_LOT, ERR_GARDEN_CLOSED, tested).
- Two old assertions changed for v1.14: `search.rs` (no "own" with no trip) and `pickers_all.rs` (seller list = sellers + 0031).
- Checks: fmt and clippy clean; golden_day 94, assist 23, search 12, pickers_all 5, venture_codes 4, garden_arrivals 25, lot_seller_list 10, all passed. 20,000 sellers: list under 20 ms in a debug build.

## 7. F1 to F3 as built (01-11-2026), accepted

- Every picker and every picker cell of the entry grids opens on focus; keys as FR-7.16 v1.14. Filter boxes follow the same rule; Ctrl+K search is not a picker and is unchanged.
- Lead decisions (accepted from frontend):
  - A. An empty field's open list highlights nothing: Enter leaves it empty and moves on; ↓ then Enter picks. A stray Enter never picks an unseen record (rule 11, the app never guesses). Typing highlights the best match at once.
  - B. A field that has a record: Enter keeps it and moves on without re-picking.
  - C. Enter on the last picker of a form with its list open goes on to the form.
  - D. Enter before the first list answer keeps the value and moves on.
  - E. Grid: the list opens when the focus moves onto a picker cell, not when a switch restores it; Esc can always leave.
  - G. Line override: from the setting, else the preview, else unknown, shown as on.
- Tests changed for the new rule: picker, picker-click-add, grid-picker-click, bikri (grid keys), bikri-states, analytics-screen (mock default answers).
- Checks: typecheck clean; eslint and prettier clean; i18n:check passed; picker-opens-on-focus-grid, bikri-line-override-note, picker, bikri 91 passed (lead rerun); frontend's wider run 289 + 589 passed. Lead rerun: lot_seller_list 10, pickers_all 5, cargo fmt clean.
- Follow-up for the gate: other picker test files not run (Umar's targeted rule); a bare search mock that returns undefined would now crash `usePickerHits`. The full suite at the gate shows any.
