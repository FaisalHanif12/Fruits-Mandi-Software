# Plan 42: buyer and seller day slips, the day total sheet, and the print header people

Version 1.0, 31-10-2026. Lead: tech-lead. Asked by Umar on 31-10-2026 while testing بکری. Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`.

Umar's words, in short:
- "Buyers: on the current day, whatever items he has bought must be mentioned, with the total of that day. Commission is added, then the total gross. If he has paid cash, it is deducted. What remains is added to his previous amount. For example, he owes 15,000, today's total is 5,000 including commission, he pays 2,000: 3,000 remains and is added to his total."
- "The same for the sellers."
- "Then the total sale of the day, for the seller and the buyer, with the commission mentioned separately, the total sale, the net cash received that day, and the cash that went out."
- His answers on the mock-ups:
  - The header carries the RH mark, the owners' names and the munshi's name.
  - Commission and لاگا go on separate lines.
  - The day total sheet lists each buyer's and each seller's total in a short table.

## 1. Requirements

PRD v1.12:
- FR-19.12, the one-party slips;
- FR-19.13, the header;
- R34, the day total sheet;
- R35;
- Section 21, Firm: up to 6 people with a role;
- E2E-36 and E2E-43 (new figures).

Architecture v1.10 15.3: `firm_contacts`.

The all-parties table of R35, R5 and the بکری prints (seller bill, buyer slips) do not change.

## 2. Worked numbers (after Events 1 to 5, 10-10-26)

**Buyer slip, اکرم:**

| Line | Amount |
|---|---|
| کیلا, پیٹی, 20 @ 600 | 12,000 |
| مال کی رقم | 12,000 |
| کمیشن | 720 |
| لاگا | 160 |
| آج کا کل | 12,880 |
| آج ادا | 10,000 |
| آج کا باقی | 2,880 |
| سابقہ رقم | 0 |
| نیا بقایا | 2,880 |

**Seller slip, رحیم:**

| Line | Amount |
|---|---|
| کیلا, پیٹی, 20 @ 600 | 12,000 |
| کیلا, پیٹی, 30 @ 580 | 17,400 |
| کیلا کل (50 نگ) | 29,400 |
| خام بکری | 29,400 |
| کمیشن | 1,764 |
| کرایہ | 3,000 |
| مزدوری | 250 |
| منشیانہ | 50 |
| فون | 20 |
| آج کی صافی بکری | 24,316 |
| آج ادا (5,000 نقد + 4,316 ادائیگی) | 9,316 |
| آج کا باقی | 15,000 جمع |
| سابقہ رقم | 15,000 بنام |
| نیا بقایا | 0 |

**R34 day total sheet:**
- 2 بکری; خام 41,400; 60 نگ.
- Commission: seller 2,484, buyer 2,484, لاگا 480, total 5,448.
- Buyers: billed 44,364, cash received 22,800, left 21,564.
- Sellers: net sale 34,476, paid out 9,316.
- Cash: 50,000 + 22,800 − 14,316 = 58,484.

Buyers' table:

| Buyer | سابقہ | آج کا کل | آج ادا | نیا بقایا |
|---|---|---|---|---|
| اکرم | 0 | 12,880 | 10,000 | 2,880 |
| ارشد | 0 | 18,684 | 0 | 18,684 |
| شاہد | 0 | 12,800 | 12,800 | 0 |
| Total | 0 | 44,364 | 22,800 | 21,564 |

Sellers' table:

| Seller | سابقہ | آج کی صافی بکری | آج ادا | نیا بقایا |
|---|---|---|---|---|
| رحیم | 15,000 بنام | 24,316 | 9,316 | 0 |
| کریم | 0 | 10,160 | 0 | 10,160 جمع |

Rule for every row: نیا بقایا = سابقہ + آج کا کل (or صافی بکری) + other entries − paid, each with its side. The backend checks the identity before it prints. A mismatch stops the print with an error, never a wrong slip.

## 2b. Header people

- Settings → Firm holds up to 6 people.
- Each person has a name in ur and en, a role (مالک owner or منشی munshi) and a phone (optional).
- Every print header shows the trade mark box, then the owners, then the munshi.
- Existing contacts are read as role munshi, so nothing breaks.

## 3. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: the print data types for the buyer slip and the seller slip (goods rows with item-total rows, then the labelled lines), the R34 additions (commission lines, buyers/sellers summary, the two tables), the firm people with role. Write the stubs, run `pnpm gen:types`, add fixtures. Tell `frontend`. | backend | |
| T2 | Logic: build both slips and the R34 additions from posted records, with the identity check. Call the money-calculations agent before and after. Golden tests E2E-36 and E2E-43 get the new figures (Code Standards 9.2 files). Excel for R34. | backend | T1 |
| F1 | The two slip templates (A4, A5 and 80 mm; ur and en; plan 39 alignment; numbers centred), the R34 print and screen with the two tables, the header with the trade mark, owners and munshi, the Settings → Firm people editor (role, up to 6). Screenshots ur and en. | frontend | T1 |
| F2 | Wire to the real data; print preview of both slips and R34 with the worked numbers. | frontend | T2, F1 |

Targeted checks only (Umar's rule).

## 4. Acceptance

- The two slips and R34 print the worked numbers of Section 2 exactly, in ur and en.
- Every earlier golden number is unchanged.
- The header shows RH, the owners and the munshi.

## 5. T1 as built and lead decisions (31-10-2026)

- Contract: the existing ReportPage and ReportPrintData are used. New: FirmRole, FirmContact (role, optional phone, max 6), PrintContact.role, ReportTable, ReportPage.tables, RowKind::ItemTotal, PrintReportTable and ReportPrintData.tables. Fixtures: `report_r35_slip_buyer_akram*`, `report_r35_slip_seller_rahim*`, `report_r34_day_sheet*`.
- A line that can be below 0 is a balance cell with its side: the seller's net sale, other entries, today's left, previous and new.
- R34 gets an "other entries" line on each side when it is not 0. `report-r34-commission` stays the total, and three rows come before it.
- Q1 to Q3, accepted as backend's defaults and recorded as PRD A69 to A71 for Umar:
  - The tables list only parties with an entry that day. Dead people follow the R35 table rule.
  - The seller sums cover 03011 sellers only.
  - The seller slip shows the ledger balance.
- Minor points, accepted:
  - "other entries" shows as a net with its side;
  - an item total is per item, across packings;
  - a slip for اپنا مال is refused;
  - a weight item shows its stored amount.
- PRD figures added: the sellers' table total (previous 15,000 بنام, new 10,160 جمع), the buyers' table total, and اکرم's total نگ 20.

## 6. F1 as built and lead decisions (31-10-2026)

- F1 is built on the T1 contract and fixtures: both slips (A4, A5, 80 mm), R34 print and screen with the two tables, the header (trade mark, owners, munshi) and Settings → Firm (up to 6 people, role, optional phone). Text in `print.ftl`, `reports.ftl`, `settings.ftl`, marked review pending.
- English words approved: Value of Goods, Today's Total, Other Entries, Paid Today, Left Today, New Balance, Net Sale Today, Buyers' List, Sellers' List. Role words مالک / منشی, Owner / Munshi.
- 80 mm roll: offered for a one-party slip only (PRD Section 21, paper per document). Backend makes sure the R35 report print accepts `mm80` for a one-party slip and refuses it for the all-parties table and R34.
- R34 on A4: aim for 2 pages for one day. The label and value lines (sale, commission, cash, other money) go in two columns side by side on A4. The two short tables stay full width. Nothing is removed, no word or order changes. A5 keeps one column.
- At 1024 x 600 a vertical scroll to reach "New Balance" is fine (NFR-USE-15 forbids only sideways scroll).
- Old real fixtures (`reports_run_*.json` without `tables`, header and settings fixtures without `role`) are regenerated by backend in T2. Frontend rewrites the old slip tests and removes unused Fluent ids in F2.

## 7. T2 as built and lead decisions (31-10-2026)

- T2 HANDOFF accepted: slips and R34 built from posted records with the identity check; R34 Excel holds both tables; mm80 for a one-party R35 only (else ERR_OUT_OF_RANGE on "paper"); old fixtures regenerated. Money agent two rounds, all points applied.
- H1 accepted: a one-time seller's name and phone are fixed in an edit when another entry rests on the lot (ERR_EDIT_NOT_ALLOWED, lot_seller_locked). PRD A74.
- A same-day پیشگی to a seller is in "other entries", not "paid today". PRD A72.
- A الگ کھاتہ lot's credit is outside the seller sums: already PRD A70.
- Weight items: the slip row shows kg, rate per kg and the stored amount. PRD A73. Backend adds the weight to the row; frontend shows it in F2.
- backup_commands fixture (schema 10 vs 11 after plan 40): backend regenerates it.
- R34 speed (NFR-SPD-10) is measured at the gate by robustness.
