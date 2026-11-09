# Plan 61: Bug 1, a complete sale line shows no figures

Deliverable D8, item 1. Requirements: PRD FR-7.5, FR-7.14, FR-7.52, FR-7.53, FR-12.6; NFR-SCL-3 (v1.32: 1,000 lines), NFR-SPD-4, NFR-SPD-5, NFR-SPD-7. Decisions B1 to B4 in plan 60.

## 1. Goal

Every complete line shows its figures and counts in the summary. Umar's exact case is fixed. A line that cannot be worked out says why.

## 2. Lead's first reading of the cause (to be proved, not assumed)

The rate cell of a blank line shows the backend's last-rate hint. `EntryGrid` serves that hint from `hintFromCalc`. If the value the grid compares against when a cell edit ends is the hint, then typing the same number as the hint (900 after a line at 900) ends with "no change" and nothing is written to the store. The cell stays empty and grey, the line stays blank, and the backend ignores it. Line 2 worked because its rate (900) differed from the hint (1,000). This explains "typed many times". `frontend` proves or replaces this with a failing test and writes the real cause in Section 6.

## 3. Worked numbers (D8 Bug 1)

Lot: 200 received, کرایہ 10,000, مزدوری 1,000. Lines: مانی 20 @ 1,000; انس 50 @ 900; ہارون 130 @ 900.
Buyer totals 21,360 / 48,100 / 125,060 = 194,520. خام بکری 182,000. کل اخراجات 10,000 + 10,920 + 1,000 = 21,920. صافی بکری 160,080. Shared income 10,920 + 10,920 + 1,600 = 23,440: 9,376 / 7,032 / 7,032.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | A test on the real grid and the real draft that types Umar's exact case, the rate equal to the hint. It fails first. | frontend |
| T2 | The fix. The summary then shows the Section 3 numbers from a real preview fixture. | frontend |
| T3 | Hint look (B1). Over-quantity: Enter does not leave the qty cell (B2). Incomplete lines say what is missing (B3), Fluent in both languages. | frontend |
| T4 | Speed at 1,000 lines: typing, moving and the live summary. Find what is slow on both sides and fix it. Timings before and after. A backend bench for `bikri_preview` at 1,000 lines and a grid timing test. | frontend, backend |
| T5 | Golden test E2E-64 in `golden_day` with the Section 3 numbers. | backend |

## 5. Acceptance (D8-A2, A3, A4)

- Umar's case, rate typed, fails before the fix and passes after. Numbers of Section 3 shown.
- Exactly sold lot completes with no warning. Over-quantity stops at once. Incomplete line says what is missing. The hint no longer looks like a value.
- 1,000 lines: key press under 50 ms, arrow move under 30 ms, live summary under 150 ms after the last key where the machine allows; real timings shown either way.

## 6. Real cause (written by the lead after proof)

To be filled.
