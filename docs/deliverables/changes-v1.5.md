# Requirement changes v1.5 (read before the D3+D4 run)

Written for `tech-lead`, 15-10-2026. Umar approved every change below. They are his answers to the open questions in `status.md` after the D1+D2 run, plus the document follow-ups from the D1+D2 DONE REPORT. They are in the working tree, not committed yet.

## 1. Short answer

- **No golden test number changed.** The worked day still balances at 85,048 on both sides. E2E-1 to E2E-39 are the same.
- **Most answers confirm what the team already built.** Five need code, and each one is now an acceptance item: D3-A19, D3-A20, D4-A29, D4-A30 and D4-A31.
- **No new table, column or posting type.** The one code default that changes is the line override setting (answer 1).
- **The D2-A2 gates move to go-live** (D5-A22). Until then, the accuracy numbers are reported on every build and must not drop.
- **Commit the documents on their own first:** one commit, `docs: requirements v1.5`, with only the files in Section 5.

## 2. Umar's answers

| # | Question (status.md) | Umar's answer | PRD | Built today? | Who builds it | Proved by |
|---|---|---|---|---|---|---|
| 1 | A rule with a party and an item group | No party + group level. The default is 6% for every seller, buyer and item, and the commission agent changes it on the بکری itself: seller commission, buyer commission and لاگا can be typed on a line **by default**. Example: 4% for this seller on this one line. | 11.1, 11.3, FR-11.10, 12.4, Section 21 | The refusal of a group with a party is built. The override setting reads `false` when missing (`services/src/rules.rs`, `overrides_from`); it must start `true`. | Plan 12 | D3-A19 |
| 2 | What "newer wins" means | The later start date wins; on the same start date, the one saved last. Example: "7% from 01-11" saved first, then "5% from 01-10": October 5%, November 7%. | FR-11.5 | Yes (plan 09) | Keep it | Existing rules tests |
| 3 | Per-lot commission when its line is removed | It moves, with its stored method and value, to the first remaining line of that lot in that posting, so the lot is charged once. It is the one exception to FR-12.20. Example: 500 on line 1 of 2; line 1 removed; line 2 carries 500. | FR-11.9 | No | Plan 16 | D4-A31 |
| 4 | A seller's opening credit | It is payable, his oldest payable posting. Example: کریم opens with 10,160 جمع; an ادائیگی of 4,000 leaves 6,160 payable, and no پیشگی note shows. | FR-7.59, money agent 8.2a | No | Plan 15 | D4-A29 |
| 5 | Does a جنرل credit adjust advances? | No. Only بکری credits and the seller's وصولی adjust advances. The PRD now matches Architecture 15.7 and the money agent. So that the payable working still equals the balance, a جنرل credit on a seller is a payable posting (FR-7.59, money agent 8.2a). Example: رحیم, 15,000 advance open, جنرل credit 1,000: advance stays 15,000, 1,000 payable. | 16.4, money agent 8.2 | Yes (the team default) | Keep it | D4-A29 |
| 6 | An advance larger than the cash in hand | Umar (15-10-2026): cash can really go below zero, so it is allowed, but every entry that takes cash below 0 must be confirmed. An amber note "روکڑ صفر سے کم ہو جائے گی" shows while typing; on save a confirm box asks "کیا یہ اندراج درست ہے؟" with "ہاں، محفوظ کریں" and "نہیں، واپس جائیں" (focus on No). The backend refuses the save without the confirmation. Example: روکڑ 20,000, پیشگی 25,000 cash: −5,000, asked; Yes saves, No goes back. | FR-6.7, E58 | No | Plans 12, 15, 16, 17 | D3-A20, D4-A30 |
| 7 | A return-by date before the entry date | Refused with `ERR_RETURN_BY_BEFORE_ENTRY`; the same day is allowed. Message: "واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی". | FR-16.9, E59 | No | Plan 17 | D4-A30 |
| 8 | Opening balances | Agree with the team: at most one opening balance per account (one debit or one credit); refused on income, expense, ابتدائی فرق, نقدی منڈی and the two پختہ کھاتہ accounts; also refused on اپنا مال (it never holds a balance). A partner's drawings account can take an opening debit, which counts as money taken (C20, C22). | FR-10.9 | Yes, except اپنا مال (seeded in D4) and the drawings count in the partner list | Plan 17 | D4-A31 |
| 9 | System accounts made inactive | Agree: never. | FR-10.4 | Yes (migration 0003) | Keep it | Existing tests |

New Urdu words (both with English): "روکڑ صفر سے کم ہو جائے گی" (Cash in hand will go below zero); "اس اندراج کے بعد روکڑ { $amount } ہو جائے گی۔ کیا یہ اندراج درست ہے؟" (After this entry cash in hand will be { $amount }. Is this entry correct?); "ہاں، محفوظ کریں" (Yes, save); "نہیں، واپس جائیں" (No, go back); and "واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی" (The return-by date cannot be before the entry date). Mark them `# review: pending` like the others.

## 3. The document follow-ups from the D1+D2 DONE REPORT

- **D2-A2:** NFR-URD-24 and NFR-URD-25 (NFR v1.3) now say the 15,000 entries and the 95%, 99% and 90% gates must be met before go-live. D2-A2 is marked as the accepted follow-up. D5-A22 closes it with the old account names and the reviewer's confirmed rows. Keep the two gate tests `#[ignore]`d with their reasons until then; D5 removes the `#[ignore]` and D5-A22 runs them with `--include-ignored`.
- **Architecture 15.8:** lists the three indexes of migration 0002.
- **Tech Stack 4.4:** the word list file has six columns (roman, urdu, kind, weight, review, source). Wikidata (CC0) and GeoNames (CC BY 4.0) are named; GeoNames is credited in the app information in D5.
- **CLAUDE.md:** the Keychain sentence for `pnpm seed` on a Mac, and "versions 1.3 to 1.5".

## 4. Document versions now

| Document | Version |
|---|---|
| PRD | 1.5 |
| Architecture | 1.5 |
| Code Standards | 1.4 (only its Depends on line changed) |
| NFR | 1.3 |
| Tech Stack | 1.3 |
| Money-calculations agent | 1.4 |
| Deliverables | README 1.5, D2 1.2, D3 1.3, D4 1.4, D5 1.4 |

`python3 .claude/skills/requirement-change/scripts/check_docs.py` passes.

## 5. Files in the documents commit

The first part was committed as `c66884d`. Umar's later document changes (the cash confirm box, and push after every task) are the second documents commit: every uncommitted file under `.claude/` and `docs/deliverables/`, including `README.md`, `.claude/CLAUDE.md` and `.claude/agents/tech-lead.md`. The uncommitted override-default code (`backend/crates/core/src/rules.rs`, `services/src/rules.rs`, `services/tests/rules.rs`) is not a document: commit it on its own after the documents, once its tests pass.

First part, already in `c66884d`:

- `.claude/rules/PRD_Mandi_Software_v1.md`, `Architecture_v1.md`, `NFR_Mandi_Software_v1.md`, `Tech_Stack_v1.md`, and the `Code_Standards_v1.md` header line (Depends on PRD v1.5; its version stays 1.4)
- `.claude/agents/money-calculations.md`, `.claude/CLAUDE.md`
- `docs/deliverables/README.md`, `D2-accounts-names-rules.md`, `D3-bikri.md`, `D4-money-in-out.md`, `D5-reports-backup-golive.md`, and this file

## 6. Still open, with the team default (do not stop for these)

- D3-Q1: `Enter` stops at buyer, qty and rate only (the recommended default in D3).
- The word list ideas: names already saved as a fourth suggestion source, and amber for words with two spellings. Not built until Umar says yes.
- D1-Q1: the screenshot tool for Windows (`@wdio/visual-service` recommended). Windows phase.
- The property test "Σ کل رقم + ابتدائی سرمایہ = equity" holds only when ابتدائی فرق is 0. When `robustness` adds random openings (for example a drawings opening), it makes them balance or includes ابتدائی فرق in the check.
- The Urdu words waiting for review in `status.md`. Suggested for `accounts-more-columns`: keep "مزید کالم".
