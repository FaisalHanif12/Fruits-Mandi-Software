# Plan 98: partner's personal expense not shown (خرچہ screen list, dashboard kharch page)

| | |
|---|---|
| Status | Umar's report, 24-11-2026 (screenshots 89 to 92) |
| Branch | `fix/rokar-garden` |
| PRD | FR-16.2, FR-16.8, FR-9.5, FR-17.11, analytics view M |

## 1. خرچہ screen, type ذاتی خرچہ: the list under the form is empty

- Umar saved ذاتی خرچہ 1 for عدنان, 2,000, on 24-11-2026 (it shows in روکڑ بک). On the خرچہ screen, type ذاتی خرچہ, the card "شراکت دار کی لی ہوئی رقم" only says "شراکت دار چنیں" and lists nothing.
- Lead decision (Umar: "when partner selected then make sure that specific personal expense must be display"): with no partner chosen, the card lists every partner's money taken and returned (date, number, partner, amount, reason, taken by, return by, returned, still open), newest first, 30-day window and pages (FR-19.11). With a partner chosen, it lists only that partner's, as FR-16.2. Right after a save, the new entry is in the list at once.
- Same for type ذاتی خرچہ واپسی.

## 2. Dashboard آج کا خرچہ page: partner filter shows nothing for the partner

- Umar opened the kharch page (FR-9.5), period month, partner عدنان. Only the shop part shows (بجلی 40,000). عدنان's ذاتی خرچہ 2,000 is not shown.
- Required (FR-9.5): the partner part shows each partner's own kharch (ذاتی خرچہ) with a total; with a partner chosen, only that partner's row and his entries on a click. The partner part is behind the partner code (FR-17.11): while locked it shows the "کوڈ درج کریں" box in its place, never nothing. After the code, the figures show. Totals from the backend.
- Find the cause (missing section, a filter that drops personal expenses, the lock with no box, or the backend query) and fix it. Same check for analytics view M.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Find both causes; fix frontend; tell the lead at once if the backend data is wrong or missing | frontend-98 |

## Acceptance

- With Umar-like data (a ذاتی خرچہ of 2,000 for عدنان today): the خرچہ screen card lists it with no partner chosen and with عدنان chosen; the kharch page with partner عدنان shows 2,000 (after the code; the code box before it); view M the same.
- Tests that fail before and pass after, both languages.

## Causes and decisions (24-11-2026)

- Bug 1: the list was empty by design when no partner was chosen, and `partner_takes` needs a partner. Decision A: `partner_takes.partner_id` optional (null = every partner, same guard), `TakeRow.partner`, returns oldest-first per partner, merged by date, totals over all. Backend: backend-96f; frontend: frontend-98.
- Bug 2: the backend partner part is right; on the page the partner section sits below the tall shop chart and table, out of view. Decision: with a partner chosen, the partner section comes first and the shop section collapses to its total line; with no partner, the partner heading is visible at 1366x768 without scrolling.

## Contract (backend-96f, 24-11-2026)

- `partner_takes` input: `partner_id: number | null` (null = every partner with takes in the window, inactive partners included; lead decision). Same partner-code guard.
- `TakeRow.partner: RecordRef` (new). `PartnerTakes.partner: RecordRef | null` (null for the all-partners list).
- `WindowSource` `partner_takes` has an optional `partner_id`, so `windows_bounds` works for the all-partners list.
- Natural order oldest day first; the screen sends `sort {column: "entry_date", direction: "desc"}` for newest first. Within a day, the partner created first comes first. Returns settle each partner's own oldest take; totals over all rows. Money agent: correct.

## Result (24-11-2026)

- frontend-98 hand-off accepted by the lead (shots checked: expense list with every partner and the partner column; kharch page with partner first and the shop part folded to its total). Tests: 9 files, 265 passed (ur and en); the new kharch tests fail without the fix. Fixtures partner_takes_all*.json added.
- backend-96f: `partner_takes` all-partners mode; `cash_kinds takes` 3 passed; money agent: correct.
