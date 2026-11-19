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

Found on Umar's own data, read-only, after he confirmed he typed the rate himself more than 5 times.

The auto-saved draft of his screen (local backup of 06-11-2026 10:08 UTC, table `drafts`) holds line 3 as qty 130, rate 900, and buyer id 9, code 0030: the shared one-time account, with no name. So the rate WAS typed. The line had the wrong buyer. The preview refused it with `ERR_PUKHTA_NAME_REQUIRED`, and `problems.ts` holds that code back silently until save. Result: no figures and no message, however many times the rate was typed. The buyer cell showed ہارون because that was the text he was typing, and the party card previews the highlighted record. `user_prefs recent.buyer` proves ہارون (id 27) was never picked on any screen. Replayed on the same data, a line with ہارون really on it gives 125,060 and the numbers of Section 3, so the backend was right. The real fault is in the picker, reproduced with a failing test on the real sale grid (6 failed before, 24 passed after): Downshift sets the highlight to row 0 on every change of the text, and the effect that clears it for an empty field runs once per open. So once a letter is typed and deleted, the empty-text list has row 0 highlighted, and Enter takes it: 0030, the most recent buyer. The cell itself looked chosen because the party card follows the highlighted record.

The earlier test work could not find it, because it always picked ہارون correctly: the fault is in which record Enter takes, not in the rate cell.

Fixes: (a) In a picker, with nothing typed, Enter takes only a row the user pointed at. With text typed, Enter waits for the answer to that text and takes its first row, or stays and says so. Text left in a cell with nothing chosen is kept and named on the line. (b) A line with a buyer, qty and rate and no figures always says why. (c) Plan 64 removes 0030 from the picker. (d) Hint look, what is missing, and the end-of-edit write stay as built.

Second bug found in the app log (06-11-2026, E-20261106-0099 and -0102): a preview for a lot with no complete line and no charges returned `ErrJournalUnbalanced` ("an entry needs lines", `services/src/bikri/shared.rs` `books_of`) instead of a preview with `ErrNoSaleLines`, so the screen kept its old answer. To be fixed in plan 65 T2.

Real findings of the 1,000-line work: a grid of more than 30 lines drew 4 px high (fixed); cursor moves rebuilt the draft (fixed); the backend cut a draft at 500 lines (fixed); the save fell from 378 ms to 120 ms. Timings in the run report.
