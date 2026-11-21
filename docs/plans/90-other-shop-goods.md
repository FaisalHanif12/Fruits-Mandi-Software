# Plan 90: Goods for another shop, clearer and filled by themselves (gardens and trips)

| | |
|---|---|
| Status | Umar's request, 21-11-2026 (screenshots: a garden's مال آیا grid and دوسری دکان میں فروخت form) |
| Branch | `fix/gardens-new-form` |
| PRD | FR-12.41 (split between our shop and another shop), FR-12.32, FR-12.36, FR-12.40, FR-19.11 |

## Changes (gardens AND market trips, they share the entry screens)

1. **Column heading.** In the مال آیا grid (and a trip purchase's item lines), the column "دوسری دکان" reads "دوسری دکان کے نگ" (English "Nag for the other shop"); its hint and help line say the same.
2. **The goods list of a sale in another shop.** "مال کہاں سے" lists only the lines that still have goods set aside for another shop (other-shop qty − already sold there > 0), with that number: "آ-2: آم، پیٹی (دوسری دکان کے لیے 10 باقی)". A line with 0 for the other shop is not offered. If none has any, the list says so in words ("دوسری دکان کے لیے کوئی مال باقی نہیں") with a hint to set the split on مال آیا. The qty sold is checked against that number (FR-12.41: never below what each part has sold).
3. **Shop name filled.** Picking a line puts the shop name typed on that arrival or purchase line into "دکان" (editable, like any value the app fills, NFR-USE-27). If the line has no shop name, the field stays empty.
4. **Large lists (question answered, no change):** the gardens and trips lists already load page by page (FR-19.11: 25, 50 or 100 a page, "صفحہ x از y", grouped by month, with the period and status filters, totals from the backend over the whole filter); a garden's or trip's own entries follow the same rule. Check that each garden and trip entry list (installments, costs, goods arrived, purchases, sales in other shops) also pages and has its totals from the backend; if one loads everything, page it.

## Owner

`frontend-capital` (frontend). If the read for item 2 or 3 lacks the other-shop remaining qty or the shop name, it tells the lead, and a backend teammate adds it.

## Acceptance

- Heading reads دوسری دکان کے نگ in ur (and en); the sale list offers only lines with other-shop goods left, with the number; the shop name fills; the qty is refused above what is left for the other shop; garden and trip both; the entry lists page; tests that fail before, ur and en; typecheck, lint, i18n:check pass.
