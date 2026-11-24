# Plan 97: بکری fixes, garden and trip cards, ادھاری تفصیل names, other shop in پھل

| | |
|---|---|
| Status | Umar's report, 24-11-2026 (screenshots 78 to 88) |
| Branch | `fix/rokar-garden` (on top of plan 96, commit cea56a32) |
| PRD | FR-12.52, FR-12.6, FR-12.23, R5, FR-16.10, FR-16.11, FR-7.69 |

## 1. پھل from another shop (Umar answered 24-11-2026, PRD v1.59 FR-16.11)

- The آڑھتی box lists only seller accounts marked آڑھتی دکان. A new shop is added on the spot (names, optional phone), saved as a seller account with that mark (FR-7.69). Marked accounts are not offered as a lot's seller in بکری; they show in seller payables and the ادائیگی picker as usual.
- New input کل کٹوتی (whole rupees, 0 or more, less than the value). Subtracted: the agent is credited value − کل کٹوتی; the partner, or the partners for guests (C27), are charged the same net. 0 posts exactly as today.
- Worked numbers: 2 پیٹی × 1,400 = 2,800; کل کٹوتی 200; net 2,800 − 200 = 2,600. Journal: الف 0604 بنام 2,600, agent جمع 2,600. Guests with 3 partners: 2,600 → 867 / 867 / 866 by C27 (split_by_qty, leftover to the first created).
- Backend: a numbered migration (the next free number) with the account mark and the fruit take's deduction column (`*_paisa`, CHECK multiple of 100, ≥ 0); the lead records the names in Architecture 15.4 and 15.6 after the hand-off. Edit and cancel follow FR-16.12. Money agent review required.

## 2. Sold-out lot: no new sale line (FR-12.52)

- When a lot's sold qty equals its received qty, neither the + button nor `↓` on the last sale line (nor `Enter` on the last cell) adds a line for that lot.
- Instead the message "اس بیوپاری کا سارا مال بک چکا ہے" shows for 5 seconds, and again on each new try.
- Same for a garden lot and a trip lot.
- Today `↓` still adds a line (screenshot 80: هارون 20 on a lot with 40 of 40 sold, "ایک لائن شامل نہیں").

## 3. Seller card title

- The summary card title is the seller's name only (with tag), for example "احسان". The words "موقع پر ادائیگی" are removed from the title. The row "موقع پر ادائیگی" inside the card stays.

## 4. Every warning hides after 5 seconds (FR-12.52, changed from 10 s)

- Every warning and error message on the بکری screen hides 5 seconds after it appears: the list of lot warnings above the lots (for example "لاٹ 3: باغ کے مال کا یہ لاٹ اسی بکری میں پورا بکنا چاہیے..."), field bubbles, the sold-out message and save errors.
- It shows again, for another 5 seconds, each time the user repeats the action that caused it (save, a change to that cell or lot, the same key).
- The red mark on the cell stays until the problem is fixed. Saving is still refused while the problem exists.
- Today the lot warning stays on screen (screenshot 83).

## 5. Garden and trip cards in the summary

When a بکری holds a garden lot or a trip lot, the summary has one card for each garden and each trip, in the same grid as the seller cards:
- Garden card: the garden's name; پچھلا بقایا (the garden account's balance before this بکری, with its side); آج کی صافی بکری (its seller credit in this بکری); نیا بقایا. No موقع پر ادائیگی (نقد is hidden on a garden lot).
- Trip card: the trip's number; خام بکری; کٹوتی; آج کی صافی بکری, with the line "سفر کے شراکت داروں کو ان کے حصے کے مطابق" (to the trip's partners by their share). No partner figures (FR-17.11 allows the one entry's split, but the card does not need it).
- Figures come from the backend preview, the same function as the save (B5).

## 6. ادھاری تفصیل: names of the other buyers (R5)

- The single row "دوسرے گاہک جنہوں نے آج نہیں خریدا" becomes one row per buyer who owes but did not buy today, by name (with code and tag), sorted by Urdu name, with پچھلا بقایا and کل ادھار. A small heading above them keeps the words "دوسرے گاہک جنہوں نے آج نہیں خریدا".
- The totals rows stay: آج کے گاہک, then the grand total کل ادھار، سب گاہک, from the backend.
- A buyer whose balance is جمع is shown with its side and left out of the total, as today. A person in the dead amount follows the R35 table rule. چکتا buyers stay out.
- Many rows: still compact, page breaks keep rows whole.

## Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Preview data for garden and trip cards (item 5); R5 rows of other buyers (item 6); regenerate bindings | backend-97 | |
| T2 | Items 2, 3, 4 on the بکری screen | frontend-97 | |
| T3 | Item 5 cards and item 6 print, after T1 | frontend-97 | T1 |
| T4 | Item 1 after Umar answers | backend-97, frontend-97 | Umar |
| T5 | Finish plan 96 tests (backup fixtures schema 29, sign fixtures, 28 stale frontend tests) | backend-96f, frontend-96f | |

## Acceptance

- `↓`, `Enter` on the last cell and + on a sold-out lot add no line and show the message for 5 s, again on each try (component test with keys).
- The seller card title is the name only.
- Every بکری warning hides after 5 s and returns on a repeated action (fake timers test).
- A بکری with a garden lot and a trip lot shows their cards with backend figures; golden tests unchanged.
- R5 lists each other buyer by name; its totals equal today's totals; the PDF in Urdu has digits 0 to 9 only.
- Golden tests unchanged; `pnpm i18n:check`, typecheck, lint green.

## Out of scope

- Any change to how a lot's money posts.
