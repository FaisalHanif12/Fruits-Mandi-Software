# Plan 87: سرمایہ کہاں ہے, a clear and professional card, and the capital put into روکڑ

| | |
|---|---|
| Status | Umar's request and answers, 20-11-2026 |
| Branch | `fix/gardens-new-form` (one pull request for plans 81 to 87) |
| PRD | FR-16.14 (where the capital is, v1.43, and v1.50 below), 17.1 (where is this money), FR-10.9, FR-21.4 (sign), FR-17.11 (partner code), FR-6.7 |

## What is wrong today (Umar's data, 20-11-2026)

- Adnan's and Usman's opening capital (500,000 each, 29-10-2026) was posted against ابتدائی فرق (06019999), before PRD 17.1 v1.43 offered روکڑ. So روکڑ shows −161,740 and the card shows "رقم کا مقام درج نہیں −935,000".
- 935,000 = 1,000,000 (capital not placed) − 65,000 (party openings: buyers Mani 30,000, Ans 15,000, adnan 60,000 = 105,000 owed to the shop; sellers Faisal 10,000, Haroon 10,000, Ehsan 20,000 = 40,000 owed by the shop).
- The card uses "منفی:" labels and minus signs, ends on a negative total (−96,530), and nothing on it can be opened.

## Decisions (Umar)

1. Rebuild the card (below).
2. A one-click fix with the sign: re-post each partner's opening capital with روکڑ on the other side.

## The new card

1. Headline: "شراکت داروں کا کل سرمایہ" with the figure, and a ✓ "دونوں طرف برابر" when the two sides are equal (always, from the backend).
2. Where the money is, in two blocks with no minus signs: **لینا ہے / پاس ہے** (cash, buyers' dues, the dead amount on its own line, advances with sellers, sellers' dues beyond advances, money lent, الگ کھاتے, any other non-zero line) and **دینا ہے** (owed to sellers, labour, freight, munshi, stores, garden money held until closing, linked money already in partner totals), each with a subtotal, then **صافی**. When cash is below zero it shows in the دینا ہے side as "روکڑ کم" with a note, never as a minus figure.
3. Whose money: each partner's کل رقم with his % of the total and a small bar, ابتدائی سرمایہ, the advances still open (FR-17.13).
4. ابتدائی فرق when not zero: an amber warning, not a row: what it is in plain words, its breakdown by account (each partner opening capital not placed, each party opening), and, when a partner opening capital sits on ابتدائی فرق, the button "سرمایہ روکڑ میں درج کریں".
5. Every line opens its page (روکڑ بک, buyer dues, seller dues, R38, R26, R32, الگ کھاتے, the payables ledger R13, the partner statement).
6. An "as of" date (default today), print and PDF (FR-19.13 header), Urdu and English, the four states, keyboard. Behind the partner code as today.

## The fix (one action, one transaction)

`partners_place_opening_capital(partner_ids, reason, sign)`: for each partner whose opening capital's other side is ابتدائی فرق, reverse that opening entry and post it again with روکڑ بنام on the same opening date (FR-10.9 change of an opening; FR-6.3 reversal, nothing deleted). The sign is checked as FR-21.4 (wrong sign: nothing changes, audited without the text). The opening date is usually closed: the command reopens and closes again inside the same flow, like FR-21.4, or posts the change as FR-10.11 allows only if that is allowed for a changed opening (it is not today; use reopen-and-close). Audit: old and new posting, reason, sign. Cash below zero cannot arise (cash goes up).

Expected on Umar's data after the fix: روکڑ 838,260; ابتدائی فرق 65,000 جمع shown as "پرانے کھاتوں کا فرق 65,000" with its breakdown; partners' totals unchanged (419,235 each); trial balance balanced.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: the card read reshaped (blocks, subtotals, headline, warning breakdown by account, as-of date, links), print data; the fix command with sign, reopen-and-close, audit; tests (Umar's numbers on a copy of his data, the worked day, the sides always equal, the fix moves 1,000,000 into روکڑ and leaves partner totals and the trial balance unchanged, wrong sign changes nothing) | `backend-capital` |
| T2 | Frontend: the new card and its print, the warning with the button and the sign box (reuse the sign box of FR-21.4), links, as-of date, ur and en, four states, keyboard; tests | `frontend-capital` |

## Acceptance

- The card reads clearly with no minus figures and no "منفی:"; both sides equal; each line opens its page.
- On a copy of Umar's data: before the fix the warning names 1,000,000 of partner capital and 65,000 of party openings; after the fix روکڑ is 838,260 and partner totals are 419,235 each.
- Golden tests unchanged; money-calculations agent review of the fix; every test fails before and passes after; clippy, typecheck, lint, i18n:check pass.
