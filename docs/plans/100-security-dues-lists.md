# Plan 100: net seller tiles, total worth tile, cancel with code and sign, audit, lists, parchi, pickers, notifications

| | |
|---|---|
| Status | Umar's request of 28-11-2026 (screenshots 107 to 132), his answers of 28-11-2026 |
| Branch | `fix/rokar-garden` |
| PRD | v1.61: 9.1, FR-9.1, FR-9.6, FR-9.8 (tile retired), new FR-9.10, FR-7.41, FR-7.44, FR-7.47, FR-7.70, FR-7.71, FR-12.22, FR-15.3, FR-16.2, FR-16.8, FR-16.14, FR-17.11, FR-19.11, FR-19.15, FR-21.3, Section 21; E2E-28 |
| NFR | v1.26: NFR-SEC-5, NFR-SEC-14 |

## 1. Goal

Make the seller figures agree everywhere, give the Owner one tile with the shop's whole money position, protect every cancel with the partner code, the sign and a reason, record every security event in a readable audit log, list every entry type month by month, open personal expense without the code, and polish the parchi, pickers, notifications and the code card.

## 2. Umar's answers (28-11-2026)

1. Seller tiles: **net everywhere**. He approves the E2E-28 tile change (19,316 to 4,316; advance tile 15,000 to 0).
2. New tile: **show both, net big** (money to have, money to give, net).
3. The 10-second undo after a save (FR-7.47) stays free. Every other cancel asks code + sign + reason.
4. روکڑ: **remove** the card شروع میں روکڑ.
5. Personal expense (ذاتی خرچہ and its واپسی) needs no partner code (stated in his message).

## 3. Requirements and lead decisions

### A. Net seller figures (FR-9.1, 9.1, FR-9.6, FR-7.59, R24 stays)

Per seller (dead amount left out), with ledger balance L = open payable P − open advance A (R24 identity):

| Tile / place | Value |
|---|---|
| بیوپاریوں کو ادائیگی باقی | Σ max(L, 0) |
| پیشگی باقی | Σ over sellers with L < 0 of min(A, −L) |
| بیوپاریوں کے ذمے | Σ over sellers with L < 0 of (−L − min(A, −L)) |

So payable − advance − dues = Σ L, the same as party ledgers, the seller dues page and the party card. The seller dues page (FR-9.6) side totals and the side filter use the same split. The پیشگی باقی page lists only sellers with an uncovered advance. ادائیگی "پورا بقایا" fills max(L, 0); the payable postings list (FR-7.59) stays as the working, with a note line "پیشگی باقی: 100,000، خالص دینا: 17,040". Attention list and notifications use the net. R24 keeps its three columns. Matching (FR-15.8) and the advance tick do not change.

**Covered advance (Umar, 28-11-2026, follow-up).** An advance that the seller's own credit already covers is not shown as پیشگی باقی anywhere. Per seller, covered = min(A, P) and uncovered = A − covered (= min(A, −L) when L < 0, else 0). Everywhere that shows a seller's open advance (party card, پیشگی voucher card, پیشگی باقی page, attention list, notifications) shows only the uncovered part as پیشگی باقی; when covered > 0 it adds one line "پیشگی 100,000: بقایا میں سے کٹ چکی" (the advance is already taken out of his balance). R26 keeps given, returned, adjusted, open, and adds two columns: covered by his balance, and uncovered. Matching and the advance tick do not change.

Worked numbers (Umar's data, فیصل): P 117,040, A 100,000, L 17,040 جمع. Payable tile 17,040; advance tile 0; dues tile 0. E2E-28: P 19,316, A 15,000, L 4,316 → payable 4,316, advance 0. After the ادائیگی of 19,316: L −15,000 → payable 0, advance 15,000, dues 0 (unchanged from today).

### B. New tile کل مالیت (Total Worth), replacing آج کی نقدی (new FR-9.10)

- Money to have: روکڑ + buyers' dues + sellers' dues + dead amount + open money lent + open advances (section A) + الگ کھاتے with a minus balance.
- Money to give: sellers payable (section A) + head 02 payables (مزدوری, کرایہ, کرایہ سٹور, منشیانہ) + buyers with a جمع balance + الگ کھاتے with a plus balance.
- Net = have − give, shown big on the tile, with have and give as a small second line.
- Its page: two sections with one row per source (روکڑ one line, the rest one row per party with code and name, page by page, FR-19.11), each with its total from the backend, then the net. A row opens that party's history. Every total equals its own tile (buyers' dues = R20, and so on). Print, PDF, Excel.
- The cash sales page (FR-9.8) stays, reached from the dashboard list of pages; only its tile goes.
- Worked check (after Events 1 to 5): have 58,484 + 21,564 + 0 + 0 + 0 + 0 = 80,048; give 10,160 + (300 + 4,000 + 100) = 14,560; net 65,488 = capital 65,000 + profit 2,488 − drawings 2,000. (Equity check, must hold in a test.)

### C. Cancel needs partner code + sign + reason (FR-7.70, FR-12.22, FR-15.3, FR-7.47)

- Every cancel command (بکری, vouchers of every type, خرچہ, ادھار رقم, الگ کھاتہ, سرمایہ, جنرل, پھل, trip purchase and cost and other-shop sale, garden entry and arrival and closing undo, write-off undo) takes `partner_code`, `sign`, `reason` (all required). Backend checks them inside the same service call; nothing changes on a wrong value.
- Wrong code: counts in the partner-code lockout (FR-17.11) and is audited. Wrong sign: `ERR_SIGN_WRONG`, audited. No code set yet: `ERR_PARTNER_CODE_NOT_SET` (the box links to Settings, Security).
- The undo of FR-7.47 (within 10 seconds, same session) needs none of them.
- One shared frontend dialog `ConfirmCancelDialog` (reason, sign, code) used by every منسوخ button.

### D. Audit log (FR-21.3, NFR-SEC-5)

- New audited events: failed login (already), wrong partner code, wrong sign (reopen, edit, cancel, backup key view), lockout started, a save refused because the day is closed, cancel with its reason. Never the typed secret, only that it was wrong (FR-7.71 v1.58).
- The page: every action shown in words from Fluent (no raw id like `day_fix_close`); a filter "حفاظتی واقعات" (security events) and a filter by kind (edit, cancel, reopen, wrong code, wrong sign, wrong password, closed day); each row opens a detail with time, user, screen, record, reason, old and new values. The pager no longer overlaps the rows.
- Rows are not partner figures, so the page shows them without the code; only old and new partner amounts stay masked while locked.

### E. Personal expense without the code (FR-17.11, FR-16.8, NFR-SEC-14)

On the خرچہ screen the ذاتی خرچہ and واپسی list (`partner_takes`) and the partner's کل رقم before and after are open without the code. The kharch page partner part (FR-9.5) and view M partner part too. Everything else behind the code stays.

### F. Lists of every entry type (FR-19.11)

The list under the form on خرچہ (دکان خرچہ), ادھار رقم, الگ کھاتہ and جنرل shows a 30-day window newest first with pages, arrows and go-to-date, like وصولی, ادائیگی and پیشگی. With no party chosen it lists every entry; with a party chosen, only his. Right after a save the entry is there. Edit and cancel from a row.

### G. روکڑ (FR-16.14)

Remove the card شروع میں روکڑ. کل سرمایہ and آخر میں روکڑ stay.

### H. Settings and prints (Section 21, FR-19.13, FR-19.15)

- Firm phones: a list of up to 4 phones (was one), printed in every header after the firm name.
- Print header people: up to 8 (was 6), munshis still at most 3.
- Buyer parchi: white background (no tint); the centre holds the firm name with the owners' phones under it; the side holds the munshi's name with his phone; content moved a little down; 3 to 4 slips on one A4 page, a long slip at most half a page (2 per page), never one per page.

### I. Pickers: most used first (FR-7.41)

Every picker (accounts, items, packings, expense heads, staff, cities, people, khatas, gardens, trips ...). Opening with empty text: the most used of that field kind first (use count, ties by last use), at most 10, then the rest by code. While typing, among matches of the same rank, the most used first. Counts kept per field kind in `user_prefs`, updated on save (no migration).

### J. Notifications (FR-7.44)

- The bell opens a large panel (about half the window wide, most of its height) with three sections:
  1. **بیوپاریوں کو ادائیگی**: top 5 sellers by net payable (section A), with amount and days; total; button "سب دیکھیں" opens the seller dues page on "دکان کو دینا ہے".
  2. **گاہکوں سے وصولی**: top 5 buyers by amount and days; total; button opens the buyer dues page.
  3. **عام یاد دہانیاں**: head 02 payables still to pay from بکری (مزدوری, کرایہ, منشیانہ, کرایہ سٹور, each with its amount and a button that opens ادائیگی with that account chosen), drafts, unclosed days, overdue money lent and partner takes, old backup, failed health check, missing names, goods waiting.
- Items come from the books, so they stay until settled. Each item has "بعد میں" (later): it hides until the next بکری is saved (kept in `user_prefs`). The bell count shows items not hidden.

### K. Partner code card (FR-17.11)

The code card is centred on the page, horizontally and vertically, as a modern card.

## 4. Worked numbers

Sections A and B above. Money agent reviews A, B and C before and after.

## 5. Contract (backend writes the exact types in T1 and reports; lead records here)

- `dashboard_get`: tile values per section A; new tile `total_worth { have, give, net }`; `today_cash_sales` tile removed from the grid (command stays for the page).
- New `dashboard_total_worth(page, section)` and its print and print data: rows of have and give, totals, net.
- Every `*_cancel` input gains `partner_code: string, sign: string, reason: string`. Errors: `ERR_PARTNER_CODE_WRONG` (field `partner_code`), `ERR_PARTNER_LOCKED_WAIT`, `ERR_PARTNER_CODE_NOT_SET`, `ERR_SIGN_WRONG` (field `sign`), `ERR_REASON_REQUIRED` (field `reason`). `vouchers_undo` (FR-7.47) unchanged.
- `audit_list` input gains `kind` filter and `security_only: bool`; rows carry `action` as a stable key for Fluent.
- `partner_takes`, kharch partner part, view M partner part: no partner guard.
- List commands for shop expense, money lent, khata and general: optional party, window and page, as the voucher lists.
- `settings` firm: `phones: string[]` (max 4); header people max 8.
- `assist_attention` returns three groups (sellers, buyers, general) with top 5 and totals; `assist_snooze(item_key)`; snoozes clear on `bikri_save`.
- Search commands: most-used ordering; `search_record_use(kind, id)` or done inside saves.

## 6. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract types, stubs, `pnpm gen:types`, report to lead and frontend | backend | |
| T2 | A, B, C, D, E, F (backend), H settings, I, J backend, with tests; money agent before and after; E2E-28 tile number per Umar | backend | T1 |
| T3 | Screens: A, B page, C dialog, D page, E, F lists, G, H settings and parchi, I, J panel, K | frontend | T1 |
| T4 | Wire to real commands; both languages; four states | frontend | T2, T3 |
| T7 | Lead: check acceptance, commit, push | lead | T4 |

Review and robustness run at the end of the goal run, after Umar says continue.

## 7. Acceptance checklist

- Umar's data: payable tile 17,040, advance tile 0, dues tile 0; seller page and party ledger 17,040 جمع.
- E2E-28 new tile numbers; every other golden test unchanged.
- کل مالیت worked check of section B holds in a test (net = equity).
- Every cancel refuses without code, sign or reason, and with a wrong one; nothing changes; the wrong tries are in the audit log; the 10-second undo still works with nothing.
- Audit page: words, no raw ids; security filter lists wrong code, wrong sign, wrong password and closed-day tries.
- ذاتی خرچہ list and preview open without the code.
- خرچہ, ادھار رقم, الگ کھاتہ, جنرل lists show Umar's entries of 24 and 26-11 in the month window.
- روکڑ has two cards.
- Settings saves 4 phones; they print in headers and the parchi; parchi white, 3 to 4 per A4.
- Pickers: a party used 3 times is first on open.
- Bell panel: three sections, top 5 each, buttons work, "بعد میں" hides until the next بکری save.
- Code card centred at 1024, 1366, 1920, both languages.
- `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `cargo test --workspace` green.

## 8. Out of scope

No change to matching, posting, the advance tick or any journal. No new table unless the backend shows one is needed (then the lead decides).

## 9. Open questions

None.

## 10. Contract decisions (T1, 28-11-2026)

- Cancel inputs: the old fields + `partner_code` + `sign` (wrapper types in `core::cancel`). Each command calls a new guarded service function that checks them inside the same call. The old functions stay for undo and tests. The wait error is the existing `ERR_LOCKED_WAIT`. Month-close reverse and dead-amount undo are not in the cancel guard.
- Undo (`bikri_undo`, `bikri_mixed_undo`): allowed only in the same session, within 15 s of the save commit (the screen shows 10 s, FR-7.47). Otherwise `ERR_UNDO_EXPIRED`.
- کل مالیت also has the sources garden_plus/garden_minus and other_have/other_give, so net = Σ heads 01, 02, 03 (without the contra 0307) = equity, also with an open garden. The worked day is unchanged: 80,048 / 14,560 / 65,488.
- `PrintFirm.phones` replaces `phone`. The audit `kind` and `security_only` filters are inside `AuditFilters`. The bell's عام section keeps over-credit-limit and records-differ (FR-7.44).

## 11. Known items for the gate (robustness)

- `frontend/tests/robustness/past-sales-fuzz`: its cancel step needs the sign and the partner code (use `frontend/tests/helpers/cancel-secrets.ts`).
- `scripts/ai-eval/oracle-keys.txt`: rewrite with `MANDI_ORACLE_MANIFEST`, because R26 has two new columns (crates/assistant `robustness_live` "oracle-keys").
- D6 v1.16 changed the prompt (R26 columns): NFR-AI-2 must run again before release.
- backend updated `backend/app/tests/robustness_security.rs` and `robustness_gate_t4_ipc.rs` for plan 100 behaviour; robustness to check them.

## 12. Partner totals follow the covered advance (Umar, 28-11-2026)

FR-17.13: a partner's advance amount is his share of the seller's uncovered advance (A − min(A, P)), at each date. When coverage grows, the newly covered part comes back to the partners on that date; when it shrinks, it goes out again. Split by C28. The FR-17.9 check uses the uncovered advances. No PRD golden number changes. The money agent reviews it before and after.
