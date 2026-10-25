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

Written by the lead from the backend T1 hand-off of 26-10-2026 (`pnpm gen:types` wrote 511 files; `ErrorCode::ALL` is 130). Checks run by `backend`: fmt clean; clippy `--workspace --all-targets -D warnings` 0 warnings; `cargo check --workspace --tests` 0 errors; core lib 86; reports 10 lib + 30 `print_data` (7 snapshots, structure only); `golden_day` 85 (no number changed), `accounts` 25, `assist` 23, `print` 11, `settings_prefs_day` 11 and others, 0 failed; app `bindings` 5, `tauri_conf` 8, `ipc_auth` 18, `robustness_security` 11. Money review PASS (no money maths in `detail`, `serial`, the header, the frame or the collection types).

**Correction to this plan.** Section 1 and the D4R brief said five prints were already built. Four were: the seller bill, the buyer slips, the پختہ slip and the receipt slip. The اگراہی فہرست had no print. It becomes the 5th document `collection_list` through the same print pipeline (decision of the lead, 26-10-2026: FR-15.6 says it is printable, and one pipeline is the rule).

- **Tag.** `AccountInput.tag: string` (empty or missing means none). `tag?: string` (absent when none) on `RecordRef`, `AccountRow`, `AccountNode`, `AccountHit`, `PartyCard`, `SimilarAccount`, `PrintParty` and `PrintLine.name_tag`. New error `ERR_TAG_INVALID` at field `tag` (not raised until T2; the frontend adds it to `errors.ftl`). Until T2, a save with a non-empty `tag` answers `ERR_NOT_IMPLEMENTED` at `tag`. Rules: saved "K", "BA", "D.P"; "k" saved as "K"; refused "1A", "ABCDE", "ب", "K-". Same name and phone with a different tag is not a duplicate.
- **Firm.** `FirmSettings` and `SetFirmInput` gain `firm_contacts: FirmContact[]` (`{name_ur, name_en, phone}`, at most 3) and `trade_mark: string`, stored as `settings` keys `firm_contacts` and `trade_mark`, no migration. Until T2 a save with either non-empty answers `ERR_NOT_IMPLEMENTED` at `firm_contacts` or `trade_mark`. T2 will use `ERR_OUT_OF_RANGE` (0 to 3) on `firm_contacts`, the name and phone codes at `firm_contacts[i].name_ur|name_en|phone`, and `ERR_TEXT_TOO_LONG` on `trade_mark` (free text, 500). An audit entry is written on save.
- **One shared header and footer (nested, no duplicated field).** `PrintHeader {firm: PrintFirm, contacts: PrintContact[] {name, phone}, trade_mark: string|null (null = no box), title_id, facts: PrintFact[]}`; `PrintFooter {printed_label_id, printed_at, page_label_id, of_label_id}` (printed_at sits in the footer, with its label; language stays on `PrintData` and `PrintFrame`). `PrintData = {doc, language, dir, paper, urdu_digits, header, footer, number, cancelled, pages}`: `firm`, `title_id`, `facts` and `printed_at` MOVED into them. `PrintPage` gains `serial`; `PrintLine` gains `detail` (the client's "(کیلا 20 . 600)", built from the stored qty and rate text, no maths); `PrintParty` gains `tag?`. The frontend migrates every template.
- **New read command** `reports_print_header {language, title_id, facts}` gives `PrintFrame {language, dir, urdu_digits, header, footer}`. It is the ONE command Run B (D5) calls for every report and analytics print. Facts never carry money.
- **Collection list (5th document), stubs until T2.** `PrintDoc` gains `collection_list`; `PrintingSettings` and `SetPrintingInput` gain `paper_collection_list` (A4 when missing). `reports_collection_print_data {date, collector_staff_id?, city_id?, language}` gives `CollectionPrintData {doc, language, dir, paper, urdu_digits, header, footer, groups, total, buyer_count}`; `CollectionPrintGroup {collector|null, city|null, rows, total}`; `CollectionPrintRow {serial, code (short code), name, tag?, phone, dues, oldest_bill_date|null, oldest_days|null}`. `reports_print_collection {date, collector_staff_id?, city_id?, language, copies?, to}` gives `PrintOutcome` (PDF name `collection-list-<YYYYMMDD>-<lang>.pdf`). Both answer `ERR_NOT_IMPLEMENTED` until T2. `reports_print` with the docs `receipt_slip` or `collection_list` is refused by T2 with `ERR_FORMAT_INVALID` at `docs`.
- **Fluent ids the backend sends (the frontend defines them):** `print-collection-list-title`, `print-fact-date`, `print-fact-collector`, `print-fact-city`, `print-collection-total`; the footer reuses `print-printed-at`, `print-page-label`, `print-page-of`.
- **Changed commands:** `accounts_save`, `settings_get`, `settings_set_firm`, `settings_get_printing`, `settings_set_printing`, `reports_bikri_print_data`, `reports_voucher_print_data`. The 4 `reports_voucher_print_data_*` fixtures were regenerated (only the new header, footer and serial lines were added).

**Decisions of the lead (26-10-2026).**
1. The collection list total is the total OF THIS LIST (active buyers whose collection weekday is the day's, no پختہ, no نقدی منڈی). It is NOT the shop's total اگراہی and does not equal R20 or FR-19.9. The print labels it "total of this list". T2 builds the print from the SAME `CollectionList` value the screen reads, copies `group.total` and the list total (no re-sum), checks that the rows add up to the group total and the groups to the list total (a data error if not), and every row has dues above 0. Tests: a group total off by 100 paisa fails; the golden after Events 1 to 5 (rows 2,880 and 18,684, total 21,564); a second case with a buyer on another weekday (list total below R20); snapshots in both languages with Urdu digits on.
2. The print command makes two READ calls: `services::vouchers::collection_list`, then the pure builder in `mandi-reports` (reports cannot depend on services). This is an accepted deviation from Code Standards B2 ("one service call") because both are reads of the same value and nothing is written. The reviewer is told at the gate.
3. Template rules for `detail`: keep the spaces round the dot, no left-to-right isolate around the whole text, no wrapping. A weight line shows the نگ count with a rate per kilo ("(کیلا 4 . 590)" next to 2,950), which invites a wrong reading; until Umar decides (open list point g), the template shows the weight in kg on such a line or hides `detail` for it.
4. T2 must load `a.tag` in the db selects (every account reference carries `tag: None` today, including `collection_candidates`). `TrialRow` (services `ledger.rs`) is not exported with the tag yet: D5's R18 adds it.

**Mechanical edits in files owned by `robustness` (list them for it at the gate):** one line `tag: String::new(),` in `backend/crates/services/tests/robustness_{rules,search,journal,codes,balance,tamper,fault,clock}.rs`, and the earlier edits in `backend/app/tests/{ipc_auth,robustness_security,tauri_conf,bindings}.rs`.

**Merge note for Run B.** The shared existing print files `backend/crates/reports/src/print/{mod,build,pages,voucher,load,ids}.rs` were edited (small). All new code is in `print/header.rs`, `print/frame.rs` and `print/collection.rs`. Run B calls `reports_print_header`; it does not call `header()`, `footer()` or `pick()` (private to the module).
