# vouchers (plans 10 and 15)

The one voucher screen with a type list: وصولی (receipt), ادائیگی (payment) and پیشگی (advance).
PRD FR-7.67, FR-7.68, FR-15.1 to FR-15.12, FR-6.7, FR-13.5, FR-13.8, FR-16.8 (shape), FR-19.2.
It is one screen, not three: the side menu items and `Alt+V`, `Alt+A`, `Alt+P` open it with a type
already chosen, and the old addresses (`/bikri/receipt`, `/bikri/payment`, `/bikri/advance`) go to
`/bikri/voucher?type=...`. `&id=12` opens a saved voucher (edit while its day is open, else read
only).

## What the screen does

- **Form** (`components/VoucherForm.tsx`): type list, date, account (`AccountPicker` with
  `PartyCard`), the one-time person (the merged list of PRD FR-7.29), amount, collector (receipt),
  reason (payment to a buyer is cash lent), expected goods
  (advance), narration. Current and new balance come from the preview.
- **Waiting list** (PRD FR-15.1, `components/WaitingList.tsx`, `useSaveList.ts`): `F2` puts the form in
  the list and starts the next voucher (type and date stay); `F10` (or `Ctrl+Enter`)
  saves the list one voucher at a time, each with its own save command and its own transaction,
  in the order of the list; the form joins the list if it is filled. A saved voucher leaves the
  list (it is in the list of the day under the form, with the backend's total). One the backend
  refuses stays with its reason and can be saved again or removed. Cash below zero asks per
  voucher: the first try never carries the confirmation; if the backend refuses with
  `ERR_CASH_BELOW_ZERO_NOT_CONFIRMED`, the run stops there and asks with the cash after THAT entry
  (a fresh `vouchers_preview`). Yes saves it with the confirmation and goes on; No keeps it in the
  list ("not saved") and goes on with the rest. Each waiting voucher shows its amount and balance
  after from its own `vouchers_preview` (`useRowPreviews`). The list holds vouchers of any type; a
  lone voucher with an empty list is saved straight from the form (refusals at its fields, slip
  printing). The list is not offered while a saved voucher is being changed (F2 starts a new one).
  The waiting list has no sum of its own: the contract has no backend total for waiting vouchers
  (the day's total under the form is the backend's `total_paisa`).
- **Notes** (`components/VoucherNotes.tsx`): cash will go below zero, more than is payable, the
  right-type note of FR-7.68 (a button; it never switches by itself).
- **Helper panel** under the form, chosen by `helperTarget` in `draft.ts`: open bills with the
  marks of this amount (receipt, cash lent), payables (payment to a seller), open advances (advance
  and a receipt from a seller). "پورا بقایا" and "پرانا بل" fill the amount with the text the
  backend sent.
  The backend sends at most the oldest 200 open bills or payable postings (`total_count`,
  `truncated`); when it cut the list a one-line note says "the oldest 200 of N are shown", and the
  totals and "full balance" are still for all of them.
- **List** (`components/VoucherListSection.tsx`) of the type, with Edit, Cancel and Print, and a
  pinned total under the amount column: the backend's `total_paisa` (`vouchers_list` returns
  `{ rows, total_paisa }`; a cancelled voucher is in the rows and left out of the total).
- **Collection round** (`pages/CollectionPage.tsx`, `/bikri/collection`): the اگراہی فہرست
  (`assist_collection_list`) as a grid; the user types the amounts received and saves them together
  (`vouchers_save_receipts_batch`). A buyer left at 0 is skipped. A refused row keeps its amount and
  its problem when the round is rebuilt.
- **Receipt slip** (`print/`): the hidden route, the template and the print box (PRD FR-15.2).

## Rules kept

- Everything typed is text. The screen never adds, multiplies, rounds or splits money. The balances,
  the marks and the cash after the entry come from `vouchers_preview` (the same function as the
  save). Only `isPositivePaisa` and `comparePaisa` touch a `Paisa`.
- What is typed lives in the screen's state (`VoucherDraft`), so a language switch loses nothing
  (F39). A type change keeps date, amount and narration (`changeType`). All money is روکڑ: there is no
  mode, bank or cheque field and none is sent (PRD 15.1, plan 17b).
- Every save sends `confirm_cash_below_zero`; the confirm box only asks (`useSaveFlow.ts`).
- The voucher number is made in Fluent (`vouchers-number-<type>`): و-1 / ا-1 / پ-1 in Urdu,
  R-1 / P-1 / A-1 in English. The English letters are for Umar to confirm.

## Files

`screen-type.ts` (types and the address, read without loading the screen), `types.ts` (bindings and
`VoucherDraft`), `draft.ts` (pure helpers, unit tested in `tests/unit/voucher-draft.test.ts`),
`api.ts` (the only file that knows the command shapes; tests mock it), `hooks.ts` (queries and the
preview hook), `useSaveFlow.ts`, `useEditSeed.ts`, `collection.ts`, `list-columns.ts`.
