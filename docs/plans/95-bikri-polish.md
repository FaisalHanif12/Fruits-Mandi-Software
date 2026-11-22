# Plan 95: بکری polish, روکڑ tabs and print fixes

| | |
|---|---|
| Status | Umar's request, 23-11-2026 (screenshots 50 to 65), his answers the same day |
| Branch | `fix/bikri-polish` (on top of plans 92 and 93) |
| PRD | v1.57: FR-12.52 (new), FR-5.3, FR-12.23, FR-12.42, FR-7.69, FR-6.7, FR-15.8, R5, FR-19.15, FR-19.11, FR-19.8, FR-19.12 |

## Items

| # | Item | Owner |
|---|---|---|
| 1 | Lot warnings above the lots (net below zero) hide after 10 s; show again on a repeated action; the red figure stays | frontend-A |
| 2 | Sold-out lot: + of its sale lines adds nothing and shows "اس بیوپاری کا سارا مال بک چکا ہے" for a few seconds, again on each press (uses the backend lot balance) | frontend-A |
| 3 | درجہ list opens by itself when the cursor reaches it; leaving with no choice keeps it empty | frontend-A |
| 4 | Short forms from بکری pickers fully by keyboard: Enter/↓ next field, Enter on last or F10 saves and chooses, Tab stays inside, only Esc closes without saving (screenshot: نیا چکتا گاہک closed on Tab/↓ without saving) | frontend-A |
| 5 | No نقدی منڈی in the sale-line buyer picker; old lines still show it | backend (search scope) + frontend-A if the picker filters |
| 6 | Icon beside the seller name in each lot row: selects the lot and shows its sale lines; if fully sold, shows its summary | frontend-A |
| 7 | Lot column and summary read نقد ادائیگی; clicking the cell types in place, the page never jumps | frontend-A |
| 8 | Seller summary: the موقع پر ادائیگی box goes; the five lines stay, موقع پر ادائیگی = sum of his lots' نقد ادائیگی; the warning above today's credit works on that sum | backend (preview lines from lot نقد) + frontend-A |
| 9 | Money check: after نقد ادائیگی, the rest of the sale credit goes on his balance (جمع adds, بنام reduces); money agent verifies with tests; نقد ادائیگی shows on seller statements and slips | backend |
| 10 | Summary cards 2 or 3 to a row | frontend-A |
| 11 | پیشگی in the lot's مزید باتیں: a real advance, own posting in the same save (seller بنام is_advance, روکڑ جمع), cash check, matching, printed on the seller bill and day slip, reversed by edit and cancel; not taken off this sale unless the پیشگی tick is on | backend + frontend-A (field) + frontend-B (prints) |
| 12 | R5 ادھاری تفصیل without چکتا buyers | backend (R5 data) |
| 13 | Whole sale print buyers table: value, کمیشن, لاگا, bill, نقد, ادھار | backend (print data) + frontend-B (template) |
| 14 | وصولی, ادائیگی, پیشگی lists: 30-day window (was 7) | frontend-B |
| 15 | روکڑ tabs: combined "وصولی / ادائیگی / پیشگی کھاتہ" tab goes; three tabs وصولی, ادائیگی, پیشگی | frontend-B |
| 16 | بکری tabs: بکری بک, پارٹی کھاتے, پچھلی بکری, باقی مال; مزید only when narrow; voucher screens show the روکڑ tabs | frontend-B |

## Worked numbers (item 9 and 11)

- عدنان: previous 9,260 جمع, lot net (credit before cash) 52,900, نقد ادائیگی 30,000: sale rest 22,900 جمع added, new 32,160 جمع (screenshot 58). Previous 30,000 بنام, credit 20,000, نقد 5,000: rest 15,000 reduces it, new 15,000 بنام.
- پیشگی 20,000 with نقد 30,000 on net 52,900, tick off: cash out 50,000; payable for the sale 22,900; open advance 20,000; ledger 22,900 − 20,000 = 2,900 جمع. Tick on: the advance is adjusted from this sale's credit, payable 2,900, advance open 0.
- Golden tests unchanged (no lot پیشگی in them).

## File ownership

- frontend-A: `features/bikri/components/*` lots grid, sale lines, summary, LotMore, short forms in `shared/ui` pickers; not the toolbar tabs.
- frontend-B: `app/layout/ModuleMenu.tsx`, `SectionTabs`, cashbook/vouchers routes and lists, print templates (`features/bikri/print`, reports print).
- backend: services, domain, core, reports; `pnpm gen:types` only by backend.

## Acceptance (lead checks in WebKit at 1024, 1366, 1480, 1920, ur and en, and the money numbers above)

Every item above works; golden tests unchanged; money agent PASS on items 8, 9, 11; typecheck, lint, i18n:check, clippy, the touched tests pass.

## Lead decisions during the build

- [A] The "above today's credit" warning and confirm (FR-12.23 v1.53, FR-6.7) counts نقد ادائیگی plus the lot پیشگی when "پیشگی اس بکری سے کاٹیں" is ticked, because the advance is then taken off today's credit too and his balance can grow. With the tick off, the advance is not counted (it stays an open advance). This follows the existing rule: warn whenever his balance will grow.
- Lot پیشگی model (backend with the money agent): its own journal entry, posting type `advance`, source the بکری; one seller بنام line (is_advance) per lot and one روکڑ جمع; same transaction; column `bikri_lots.advance_paisa` (migration 0029); never in deductions, net or credit.
- Item 17 (Umar, 23-11-2026): R5 ادھاری تفصیل drops its نقد column and نقد total; the نقد column stays on the whole sale print buyers table. آج کا ادھار is still bill less paid on the spot. Owner: backend-95 (R5 data), frontend-95b (template, tests).
