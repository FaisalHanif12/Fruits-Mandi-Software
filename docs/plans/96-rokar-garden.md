# Plan 96: روکڑ بک table blank with real data; garden page organized for many entries

| | |
|---|---|
| Status | Umar's report, 24-11-2026 (screenshots 72 to 76) |
| Branch | `fix/rokar-garden` (on top of plan 95) |
| PRD | FR-16.14, FR-19.11 (v1.57 30-day window), FR-12.11, FR-12.37, FR-12.40, FR-12.41, NFR-USE-16, F8, F34 |

## 1. P0: روکڑ بک shows no table (Umar's real data)

With Umar's books the روکڑ tab shows the two cash cards and "11 میں سے 1 تا 11" but the table area is empty. The dev page (fake data, 4 rows) draws fine, so the lead missed it. Steps:
- backend: run the real `cashbook` book command (the one the روکڑ tab calls, with the default 30-day window ending today) against a COPY of Umar's database (copy at the lead's scratchpad `realcopy/mandi.db`, never the live file), and save the exact JSON response as a fixture.
- frontend: load that exact response into the روکڑ بک screen (dev page and a test), find why the table does not draw (render error, a new row kind, the window grouping, a height of 0, the no-scrollbar or relative change), fix the cause, and add a test with this real response that fails before.
- Also check "شروع میں روکڑ" for that window against the backend.
- The lead checks in WebKit with the real response before Umar sees it.

## 2. Garden page stays organized with ~90 مال آیا entries

A busy garden has about 90 goods-arrived entries. Today the page is one long scroll: the header, the entry form, the arrivals table (paged), the selected arrival's detail card with its sales table and buttons, then the garden figures. Make it organized:
- The arrivals table is the main list: 30 rows per page (FR-19.11), compact rows, newest first, sorted and searchable by item or number; its totals from the backend.
- Selecting an arrival shows its detail in place under its row or in a side panel (no second long card down the page); its sales list inside it pages too; the buttons (تاریخ فروخت بدلیں, تقسیم بدلیں, ضائع کریں, ضائع واپس لیں, مال آیا منسوخ کریں, تبدیل کریں) in one tidy row.
- The garden figures (باغ کے اعداد) and partners collapse into a short card with the key numbers visible (چلتا بقایا, ٹھیکہ باقی, کمیشن) and the rest on open.
- The entry form keeps its place; saving an entry keeps the page where it was.
- Same pattern for a market trip with many purchases (FR-12.41) if it shares the components.
- Four states, both languages, 1024 to 1920, keyboard.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Real cashbook response from the copy, saved as a fixture | backend |
| T2 | روکڑ بک blank table: find, fix, test with the real response | frontend-a |
| T3 | Garden page organized (and trips if shared) | frontend-b |
