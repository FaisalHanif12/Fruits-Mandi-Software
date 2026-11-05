# Plan 57: بکری, a line over the lot's quantity is not worked out, and every lot shows its seller

Version 1.0, 06-11-2026. Lead: tech-lead. Asked by Umar on 06-11-2026, from his own test of the بکری screen. Branch `feature/assistant`, folder `Fruit Mandi Parallel/d5`. It runs in parallel with the assistant live runs (plan 55) and touches no assistant file.

PRD: FR-12.6 (v1.28 sentence below), FR-12.8 check 3, FR-7.5, FR-7.6, FR-12.1, FR-7.24 (names through displayName). Code Standards B5 (preview and save share one function), M12, F38.

## 1. Goal

1. **A line that sells more than the lot has gets an error at once and no figures.**
   - Umar: the lot has 50 نگ and a buyer line is typed as 70. Today the screen shows the error at the top, but it still works out 84,000, the commissions, the buyer's bill and the lot's صافی بکری.
   - Wanted: the error shows under that quantity cell straight away, for example "اتنی تعداد دستیاب نہیں: صرف 50 نگ باقی ہیں". That line shows no amount, commission or total, and the lot and the whole بکری summaries leave it out until the quantity is fixed. Save stays blocked, as today (FR-12.8 check 3).
2. **Every lot label names its seller.** For example "لاٹ 1: احسان" and "Lot 1: Ehsan":
   - on the switch above the sale lines;
   - in the "لاٹ 1 کی بکری" title;
   - in the summary strip;
   - on the "مزید باتیں" button.
   One seller can own two lots, as Ehsan does in Umar's test (سیب and کیلا), so the label also shows the item: "لاٹ 1: احسان، سیب". The name comes through `displayName` (with its tag), and a پختہ کھاتہ lot shows the typed name.

## 2. Worked numbers

The lot holds 50 پیٹی سیب. Line 1 is مانی, 70 نگ at 1,200.
- **Today:** 84,000 / 5,040 / 89,600 are shown.
- **After:**
  - Line 1 shows no figures.
  - The error sits on its quantity cell.
  - The lot summary shows خام بکری 0 and "the sum waits for the quantity" (`bikri-summary-incomplete` style).
  - The lot's صافی بکری is not worked out.
- **The user changes 70 to 50:** 60,000 / 3,600 (6%) / 3,600 / لاگا as the rules say, and the figures appear again.
- **Two lines on a 50 lot, 30 and 30:** line 1 (30) is worked out, and line 2 (30) is over by 10 and gets the error and no figures. The line that pushes the lot over is the one refused, in line order.

## 3. Contract (backend3 proposes, frontend3 builds; the lead records it here)

- `bikri_preview` marks a refused line, so the screen knows not to show figures for it. The line's computed amounts are absent (`null`), never 0.
- Lot and بکری totals leave the refused line out.
- The existing error `ERR_QTY_EXCEEDS_LOT` with `available` stays, and points at `lots[i].sales[j].qty`.
- The same domain function serves the preview and the save (B5). The save refuses as today.
- `pnpm gen:types` if a type changes.

## 4. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Preview: a refused line gets no figures, and the totals leave it out. Tests on §2. The money agent before and after. Golden tests unchanged. | backend3 | |
| T2 | Screen: the error on the cell, no figures on a refused line, and the seller (and item) on every lot label. Both languages, keyboard, screenshots. | frontend3 | T1 contract for part 1; part 2 can start now |
| T3 | Robustness: attack the preview with lines over the lot, many lines, a later sale over the stock left, and an own-goods lot. | robustness | T1, T2 |

## 5. Acceptance

- Umar's case (50 in, 70 typed) shows the cell error at once and no 84,000 anywhere on the screen. Save is refused.
- Correcting the quantity brings the figures back, with no reload.
- The lot labels read "لاٹ 1: احسان، سیب" in Urdu and "Lot 1: Ehsan, Apple" in English.
- golden_day and every other golden test pass with no number changed. fmt, clippy, typecheck, lint and i18n:check are clean.

## 6. Out of scope

Changing how a lot's received quantity is typed, and later sales from باقی مال (they already check the stock left).
