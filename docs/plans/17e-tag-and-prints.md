# Plan 17e: the tag after a name, firm contacts and clear prints on the prints already built

Version 1.0, 23-10-2026. Lead: tech-lead. Brief: `docs/deliverables/D4R-refresh-v1.6.md` Section 2 "17e-tag-and-prints" (full text and worked checks). Branch `feature/refresh-v16`. Last of D4R (17b changes the receipt slip first). Use the `print-and-export` skill.

## 1. Goal

A buyer's tag (K) shows after his name on every screen and print ("عاشق ظفر K"), is found by search ("ashiq k") and is part of the duplicate check. The firm's up to 3 contact persons and trade mark print in one clear header on the five built prints (seller bill, buyer slips, پختہ slip, receipt slip, اگراہی فہرست), each with the same footer "Page x of y". R5 and the buyer slips write each line the client's way, "(کیلا 20 . 600)".

## 2. Requirements

PRD 10.2, FR-7.25, FR-7.28, FR-7.40, FR-10.7, E38, glossary (نشان), Section 21 Firm, FR-19.13, R5, FR-12.12, FR-15.2, FR-15.6, FR-19.3 to FR-19.7; Code Standards 4.2 (Tag), B26, F38, S7; Architecture 15.3 and 15.4; Tech Stack 4.7. Acceptance: D4R-A10, D4R-A11, D4R-A12.

## 3. Worked numbers

After Events 1 to 5: R5 for 10-10-26 has two rows, اکرم ٹول ٹیکس "(کیلا 20 . 600)", رقم 12,000, رقم بعد لاگا 12,880, and ارشد ملتانی "(کیلا 30 . 580)", 17,400, 18,684 (شاہد is نقدی, so not on R5); the seller bill still shows 29,400, 10,084, 19,316 and the words; a buyer with tag K prints as "اکرم ٹول ٹیکس K". Tag rules: saved "K", "BA", "D.P", "k" saved as "K"; refused "1A", "ABCDE", "ب", "K-".

## 4. Contract

`backend` T1 writes the exact types (`tag` on the account save, edit and every result type that carries names; firm settings commands with `firm_contacts` and `trade_mark`; the shared print header and footer data; R5 print data in the client layout); the lead copies them into Section 10. The column `accounts.tag` exists from migration 0006; contacts and trade mark are `settings` keys; no migration.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: types, stubs, `pnpm gen:types`, command tables | `backend` | 17d T2 |
| T2 | Logic: tag rules and storage, search keys and in-memory index with the tag, duplicate check on name + tag + phone, names carry the tag in every display result type, firm contacts and trade mark with audit, the shared print header and footer in `mandi-reports` used by the five built prints, R5 and buyer slips in the client's columns, print data snapshots in both languages with and without contacts, tests, fixtures | `backend` | T1 |
| F1 | نشان field on the account form after the English name, `displayName` adds the tag in one place, firm settings with up to 3 contacts and the trade mark, the shared print layout with the new header and footer, the R5 template, Fluent ur and en (including glossary نشان) | `frontend` | T1 |
| F2 | Wire to T2 fixtures, shape tests, keyboard and language-switch tests, screenshots, Windows spec for print checks (LOCAL-ONLY on Windows) | `frontend` | T2 |

## 6. Acceptance checklist

D4R-A10 (tag rules, `displayName`, search, duplicate check, no-tag accounts keep their keys), D4R-A11 (contacts and trade mark save with an audit entry and show in all five headers; none set means no empty box), D4R-A12 (five prints with header and footer, party with short code and name with tag, R5 columns, buyer slips still show a named buyer's نقدی lines, numbers unchanged).

## 7. Out of scope

The header and footer on every D5 report, analytics print and list print (D5 reuses this layout); R35 in the client's two layouts (D5).

## 8. Open questions

None. The team reading of R5 and the buyer slips (slips show a named buyer's نقدی lines; R5 lists credit buyers only) was left to the lead by Umar on 25-10-2026 and is decided as built here (`changes-v1.7.md` Section 7). It stays in the DONE REPORT list.

## 10. Contract as built in T1

(To be written by the lead from the backend T1 hand-off.)
