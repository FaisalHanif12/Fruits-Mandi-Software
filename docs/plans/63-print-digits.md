# Plan 63: Bug 3, Urdu prints show Urdu digits

Deliverable D8, item 3. Requirements: PRD FR-7.32, FR-19.6, Section 21 Printing (v1.32); NFR-URD-14; Tech Stack 4.5. Decision D1 in plan 60.

## 1. Goal

Every number on every print and PDF, in Urdu and English, uses 0 to 9 with grouping: 2024, 10,000, 06-11-2026. Amounts in words stay Urdu words. The Printing setting "Urdu digits" goes away.

## 2. What is known

`backend/crates/reports/src/print/format.rs` turns digits into ۰-۹ when the print uses Urdu digits; `core/src/printing.rs` holds the setting; `frontend/src/shared/format/digits.ts` is the frontend half. Find why the Urdu prints show them with the setting off: the setting's value, its default, a font, or `digits.ts`. Write the real cause here.

## 3. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Failing tests: print data and rendered print text of R1, R5, R35 in Urdu hold no ۰ to ۹ or ٠ to ٩ and hold "10,000" and "2026". | backend, frontend |
| T2 | Remove the digit conversion and the setting: `to_urdu_digits`, the setting key and its screen control, `digits.ts`, their Fluent text. An old stored setting value is ignored safely. Parsing of Urdu digits typed by a user (Code Standards 4.2) stays. | backend, frontend |
| T3 | Regenerate fixtures that carried the field; no number changes. | backend |
| T4 | Urdu PDFs of R1, R5 and R35 made on the Mac from the same templates, with their text extracted. This uses the Mac path of plan 69 or headless Chrome. Golden test E2E-69 comes with it. | frontend, lead |

## 4. Acceptance (D8-A6)

The text of the Urdu PDFs of R1, R5 and R35 holds no ۰ to ۹ or ٠ to ٩, and holds "10,000" and "2026".

## 5. Real cause

Only the setting `urdu_digits` could make Urdu digit shapes. It drove `to_urdu_digits` in `reports/src/print/format.rs`, and `counter(page, persian)` with the `[data-digits=urdu]` style in the print sheet. The fonts, Fluent and `Intl` all give 0 to 9, in Chrome and in WebKit. The default is off. `backend` found the stored value in the development database on this Mac: `settings.printing` holds `"urdu_digits":true`, written on 26-10-2026 and kept when the data was copied. So the Urdu digits came from that stored setting. The fix removes the setting and both producers, so no stored value can bring the digits back.
