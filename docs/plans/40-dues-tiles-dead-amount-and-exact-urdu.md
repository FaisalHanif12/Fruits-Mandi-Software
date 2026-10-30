# Plan 40: dues tiles, the dead amount (ڈوبی رقم) and exact Urdu words

Version 1.0, 30-10-2026. Lead: tech-lead. Asked by Umar on 30-10-2026 while testing. Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`.

Umar's words, in short:
- The dashboard should show sellers' dues, and one box with seller + buyer as the total اگراہی.
- Another box should show the dead amount. If a person takes goods and owes the shop, then dies or leaves the business, we try to recover the money. Recovered or not, we deactivate him. If it is not recovered, it is still in the total اگراہی, but shown apart as the dead amount, because the person is gone.
- "Use the exact Urdu word, like in English you are using buyers' dues but in Urdu you are using وصولی باقی, which means the whole. Go through all the pages."

His answers:
- 4 boxes.
- The word ڈوبی رقم.
- Anyone who owes can be moved.
- For the Urdu: change all, then give him a list.

## 1. Goal

The owner sees at a glance:
- what buyers owe;
- what sellers owe;
- the total to collect;
- how much of it is dead.

A person who is gone is moved out of the daily lists, but his debt is never forgotten. Every Urdu label says exactly what its English says.

## 2. Requirements

PRD v1.10:
- 9.1 tiles;
- FR-7.23;
- FR-10.10;
- FR-19.9, R20, R24, R35, new R38;
- E71 to E73, A65;
- E2E-44.

Also Architecture v1.9 15.4 (`dead_marks`), Code Standards v1.11 9.2 (E2E-44 in `golden_day.rs`) and NFR-URD-29.

## 3. Worked numbers (E2E-44)

Start: after Events 1 to 5. That gives cash 58,484; اکرم 2,880; ارشد 18,684; no seller owes; trial balance 85,048.

| Step | گاہکوں کے ذمے | بیوپاریوں کے ذمے | ڈوبی رقم | کل اگراہی | Cash |
|---|---|---|---|---|---|
| End of 10-10-26 | 2,880 + 18,684 = 21,564 | 0 | 0 | 21,564 | 58,484 |
| 11-10-26: ارشد moved (died) | 2,880 | 0 | 18,684 (1) | 2,880 + 0 + 18,684 = 21,564 | 58,484 |
| 12-10-26: وصولی 4,000 on ارشد | 2,880 | 0 | 18,684 − 4,000 = 14,684 | 17,564 | 62,484 |

- The trial balance stays balanced at every step (85,048 = 85,048).
- Partner totals do not change.
- A new بکری line for ارشد is refused (`ERR_ACCOUNT_INACTIVE`).
- R38 shows him with owed then 18,684, recovered 4,000 and still owed 14,684.

## 4. Contract between backend and frontend

Backend writes the exact Rust types in T1 and records any change here through me. Money is always paisa (`number`). Dates are `YYYY-MM-DD` on output, and text as typed on input.

| Command | Input | Output | Errors (field) |
|---|---|---|---|
| `masters_dead_preview` | `{ account_id, pukhta_person_key: string \| null }` | `{ owed_paisa, allowed: bool, reason_not_allowed: string \| null }` | `ERR_NOT_FOUND` |
| `masters_dead_mark` | `{ account_id, pukhta_person_key: string \| null, entry_date: string, reason: "died" \| "left" \| "other", note: string }` | `{ mark_id, owed_paisa }` | `ERR_NOTHING_OWED` (account_id), `ERR_DEAD_NOT_ALLOWED` (account_id: partner, الگ کھاتہ, garden, system), `ERR_ALREADY_DEAD` (account_id), `ERR_NOTE_REQUIRED` (note, when reason is other), the open-day date errors (entry_date) |
| `masters_dead_undo` | `{ mark_id, reason: string }` | `{ ok }` | `ERR_REASON_REQUIRED` (reason), `ERR_NOT_FOUND` |
| `reports_dead_amount` (R38) | `{ filter: "owed" \| "all", search: string, offset, limit }` (B31 page) | `{ rows: [{ mark_id, account_id, code, name_ur, name_en, tag, party_type, pukhta_person_key, pukhta_name_ur, pukhta_name_en, reason, note, entry_date, owed_then_paisa, recovered_paisa, still_owed_paisa, last_recovery_date }], row_count, totals: { owed_then_paisa, recovered_paisa, still_owed_paisa } }` | |

Existing outputs that grow (exact names chosen by backend in T1):
- **Dashboard tiles:** `buyers_dues_paisa` (now without dead people), `sellers_dues_paisa`, `total_dues_paisa`, `dead_amount_paisa`, `dead_count`.
- **Party card and ledger heading:** `dead: { mark_id, entry_date, reason, note } | null`.
- **Account search:** `is_dead: bool`. Dead accounts are returned only to the receipt picker and the ادھار رقم واپسی picker.

New migration: `dead_marks` (Architecture 15.4), at most one active mark per account or person key.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Contract: migration, Rust types, command stubs that return `ERR_NOT_IMPLEMENTED`, the new fields on the tiles, the party card and search, `pnpm gen:types`, fixtures. Tell `frontend`. | backend | |
| T2 | Logic. Call the money-calculations agent before writing and again after. Cover: mark, undo, R38 paging and totals, the tiles, and leaving dead people out of R20, R21, R22, R24 (بیوپاری کے ذمے), R32, R35, analytics C "dues now", the attention list, aging and the collection round. A وصولی and an ادھار رقم واپسی on a dead account are allowed with normal matching; every other posting on it is refused. Audit entries. Golden test E2E-44 in `golden_day.rs`. Tests for each error. | backend | T1 |
| F1 | Screens against the contract: the 4 dues tiles plus the renamed ones (9.1, R38 opens from the dead tile); the "ڈوبی رقم میں ڈالیں" dialog (balance shown, date, reason, note) on the account form, the party card and the R20, R24 and R32 rows; "دوبارہ فعال کریں" with a reason; the R38 screen and its print and Excel (print-and-export skill); the "ڈوبی" tag in the two pickers, the party card and the ledger heading. Four states, keyboard, ur and en, screenshots. | frontend | T1 |
| F2 | Exact Urdu. Check every message pair in `frontend/src/i18n/ur` against `en`. Where the Urdu is looser, wider or different in meaning, change the Urdu to say exactly what the English says (FR-7.23, NFR-URD-29). Glossary mandi words (بکری, روکڑ, وصولی, اگراہی, لاگا ...) stay. Where the English itself is the wrong one, do not change it: list it. Mark changed messages `# review: pending`. Write the full list (message ID, English, old Urdu, new Urdu, why) to `frontend/tests/e2e/output/plan40/urdu-changes.md` for Umar. | frontend | |
| F3 | Wire F1 to the real commands; the E2E-44 path works in the app. | frontend | T2, F1 |

Targeted checks only (Umar's rule): the touched tests, `pnpm typecheck`, `pnpm lint`, `pnpm i18n:check`, and `cargo test -p mandi-services --test golden_day`.

## 6. Acceptance

- E2E-44 passes with the numbers in Section 3. Every earlier golden number is unchanged.
- The tiles show the 4 dues boxes in ur and en. Buyers' dues equals the R20 میزان. Total dues equals R20 + بیوپاری کے ذمے + R38.
- Move, recover and undo work with the keyboard only, in both languages, with an audit entry each.
- The Urdu change list covers every changed message, and `i18n:check` passes.

## 7. Out of scope

Writing the dead amount off as a loss (A65, a later question for Umar). Changes to English text.

## 8. Decisions after the document review (30-10-2026)

An independent review of PRD 1.10 found 28 points. All are applied in the PRD (FR-10.10 rewritten), Code Standards 4.2 and 9.2, and Architecture 15.8 and diagram 6. The ones that change the build:

- D1. **پختہ person.** The پختہ accounts stay active. Only the person key is marked. A new line or lot with that key is refused with `ERR_PERSON_IN_DEAD_AMOUNT`.
- D2. **Recovery limit.** A recovery more than the dead amount is refused with `ERR_RETURN_EXCEEDS_OPEN`, with the amount still owed.
- D3. **Whole balance [A], A66.** A seller's open advance and payables, and a person's open loans, are netted into his dead amount. They leave پیشگی باقی, بیوپاریوں کو ادائیگی باقی and ادھار رقم باقی, and they join کل اگراہی.
- D4. **Corrections [A], A67.** An edit, cancel or undo of an earlier posting, or of a recovery, is still allowed while its day is open. R38 shows it as a correction.
- D5. **Linked account [A], A68.** It is not moved with him.
- D6. **Date of the move.** It defaults to today. It must be an open date and not before his last posting.
- D7. **More lists leave a dead person out:** R21, R22, R26, the dashboard's oldest dues list, FR-7.44 (credit limit, dues, payables, money lent), the collection round and the اگراہی فہرست. "Make regular" (FR-13.8) is not offered for a dead person.
- D8. **R35** leaves a dead person's row out of the grand total and adds it in a second total row. **R34** shows all four dues figures.
- D9. **Urdu titles:** R20 is گاہکوں کے ذمے رپورٹ and R24 is بیوپاریوں کو ادائیگی باقی.
- D10. **E2E-44 is split.** The tiles, refusals and trial balance go in `golden_day.rs`. The R20 and R38 rows go in `reports/tests/ledger_items.rs`.

Contract additions: `masters_dead_mark` also returns `ERR_PERSON_IN_DEAD_AMOUNT` (pukhta_person_key, when already marked), and `ERR_DATE_BEFORE_LAST_POSTING` (entry_date). The receipt and ادھار رقم واپسی saves return `ERR_RETURN_EXCEEDS_OPEN` (amount) for a dead account.

## 9. Contract as built in T1 (backend, 30-10-2026, accepted by the lead)

1. `masters_dead_preview.reason_not_allowed` is an error code or null, not text (Code Standards: no user text in Rust). The codes are ERR_NOTHING_OWED, ERR_DEAD_NOT_ALLOWED, ERR_ALREADY_DEAD and ERR_PERSON_IN_DEAD_AMOUNT.
2. The preview also returns `last_posting_date` (D6) and `linked`, which is `{ account_id, code, name_ur, name_en, tag, balance }` or null (D5: the screen shows the linked balance).
3. R38 rows add `status` ("owing", "recovered" or "undone") and `corrections_paisa` (signed).
   - still owed = owed then − recovered + corrections.
   - The totals include corrections.
   - The "all" filter also lists recovered and undone moves.
4. Six new error codes need ur and en messages in errors.ftl: ERR_NOTHING_OWED, ERR_DEAD_NOT_ALLOWED, ERR_ALREADY_DEAD, ERR_NOTE_REQUIRED, ERR_PERSON_IN_DEAD_AMOUNT and ERR_DATE_BEFORE_LAST_POSTING.
5. Field names:
   - tiles: `buyers_dues_paisa` (now without dead people), `sellers_dues_paisa`, `total_dues_paisa`, `dead_amount_paisa`, `dead_count`;
   - party card: `dead: DeadInfo | null`;
   - ledger heading: `ReportParty.dead`;
   - search: `AccountHit.is_dead`.
6. ReportId "R38" is added in T2, together with its catalog entry.
7. The migration is `0011_dead_marks.sql`. The unique index is on (account_id, COALESCE(pukhta_person_key, '')) where is_active = 1. This keeps the meaning of Architecture 15.8: SQLite treats two NULLs as different, so a plain index would allow two active marks on one account.
8. The command tables in `ipc_auth.rs` and `robustness_security.rs` gained the three new commands. These are table rows only, in a file owned by robustness; robustness to be told at the gate.
9. `search_accounts` input gains `include_dead: bool` (default false). Only the receipt picker and the ادھار رقم واپسی picker pass true (lead decision, 30-10-2026).
10. `masters_set_active(true)` on an account with an active dead mark is refused with ERR_ALREADY_DEAD. Only `masters_dead_undo` makes him active again (lead decision).

## 10. Lead answers to backend's T2 questions (30-10-2026)

- Recoveries are netted. A cancelled or edited recovery changes "recovered" itself; "corrections" are only reversals of other postings. PRD FR-10.10 and A67 are reworded.
- Known limits, accepted:
  - A dead person whose balance turns to credit shows only in R38 (status "credit").
  - An edit of an earlier ادائیگی, پیشگی or جنرل on a dead account is refused (cancel works).
  - R34 shows sellers' and total dues for today only, as R24 does.
  - R35's second total adds only the dead rows with an entry that day.
- Speed (one extra query per posting, the dead tile, R34) is measured by robustness at the gate.
- Docs: Architecture 15.4 and 15.8 (3 more columns, the COALESCE index); Code Standards 9.2 (the E2E-44 report rows are in `services/tests/reports_dead.rs`).
- `search_pukhta_people` input gains `include_dead` (default false), and each hit gains `is_dead`. Only the receipt screen sends true. `reports_find_people` still lists everybody, so a dead person's ledger can be opened.
