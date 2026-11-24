# vouchers (plans 10 and 15)

The one voucher screen with a type list: وصولی (receipt), ادائیگی (payment) and پیشگی (advance).
PRD FR-7.67, FR-7.68, FR-15.1 to FR-15.12, FR-6.7, FR-13.5, FR-13.8, FR-16.8 (shape), FR-19.2.
It is one screen, not three: the three tabs of the روکڑ area (plan 95, PRD FR-5.3 v1.57; the screen and the اگراہی فہرست
show the روکڑ tabs) and `Alt+V`, `Alt+A`, `Alt+P` open it with a type
already chosen, and the old addresses (`/bikri/receipt`, `/bikri/payment`, `/bikri/advance`) go to
`/bikri/voucher?type=...`. `&id=12` opens a saved voucher (edit while its day is open, else read
only).

## What the screen does

- **Form** (`components/VoucherForm.tsx`): type list, date, account (`AccountPicker` with
  `PartyCard`), amount, collector (receipt),
  reason (money given to a buyer is cash lent: a payment to him, or an advance to him), expected goods
  (an advance to a seller), narration. Current and new balance come from the preview.
- **Advance to a buyer** (plan 88, PRD 15.4, FR-15.11): the advance tab's account field takes a seller
  or a buyer (scope `seller_buyer`), each row marked with his kind. A seller goes on as before. A buyer
  is cash lent: the reason is asked and required, his open bills show under the form, and the input
  carries `reason` and no expected goods. The backend saves it as a payment to him, so the message
  says "Cash Lent" with a number of the payment series (`savedKindOf`). The advance list is asked
  with `advance_view` and has its own window source `{ kind: 'advances' }`; it holds the advances
  and the cash lent, with an account type column. Opening a cash lent row from that list opens it
  on the payment tab, which edits a payment.
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
- **List** (`components/VoucherListSection.tsx`) of the type, one window of 30 days at a time (`useListWindow` with
  `days: VOUCHER_DAYS_WINDOW`, plan 95 item 14, PRD FR-19.11 v1.57; the buttons say "30 days"), with Edit, Cancel and Print, and a
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

Plan 17e: the collection list can be printed (`print/CollectionListTemplate.tsx`, the 5th document,
`print/CollectionPrintView.tsx` for the preview and the hidden window, `print/CollectionPrintBox.tsx`
for the language, printer or PDF, copies and preview). The day and the filters are the collection
screen's own. The total on paper is the total OF THIS LIST (not the shop's total اگراہی). The Print
button (and F9) of the collection screen opens the box. The hidden print window is
`print/CollectionPrintPage.tsx`, the address `#/print?collection=<YYYY-MM-DD>&doc=collection_list&
language=<ur|en>[&collector=<id>][&city=<id>]&job=<job>` (`print/collection-params.ts` reads it; the day
goes to the command as `DD-MM-YYYY`). It may call `reports_collection_print_data` only. An empty day
shows the words for nothing to print.

- **Editing a saved voucher is clear (plan 89, PRD FR-7.70, FR-7.71).** The amber band (`EditBand`) of a وصولی,
  ادائیگی or پیشگی says what CAN be changed (`edit-band-can-voucher`: the amount, the account and the narration;
  only the day is fixed), not only that the day cannot move. In edit mode `VoucherForm` puts the day and the account
  in the first row and the amount in the second; the party card is drawn by the screen
  in the aside of `FormAside` (the picker has `showCard={false}` and says whose card it is through `onCardAccount`),
  so a tall card never pushes the amount down; the cursor starts in the amount. After the save the voucher stays on
  the screen (it is read again, `VoucherEditor` is keyed by id and version): its stamp (time, Owner, the reason; never the
  sign), the mark "Before ..." on the amount with the old and the new figure in its tip, and
  the "Edited" word of its list row. A voucher of a closed day that was reopened for the fix closes again after the
  save (`finishFixSession`) and opens read only with the same stamp and mark (`SavedVoucherView`). Tests:
  `vouchers-edit-clear.test.tsx` (the three types and the closed-day path), `vouchers-edit-cancel.test.tsx`.

- **Saving a change asks the reason and the Owner's sign (plan 92, PRD FR-15.3, FR-21.4, FR-7.71).** F10, Ctrl+S and
  the button on a saved وصولی, ادائیگی or پیشگی open one box (`ReasonDialog` with its sign field, opened by
  `useSaveFlow`: `askSign`, `confirmSign`) on any day, today included. Both fields are required; the focus starts in
  the reason, Enter goes to the sign, Enter or F10 there saves, Esc closes the box with nothing saved and the change
  kept. The backend checks the sign: a wrong one is told under the sign in the box and nothing is saved (no
  lockout). The cash question (FR-6.7) comes after the box. When the day was reopened to fix this entry the reopen
  asked both already: its reason goes with the edit, the sign goes empty (the backend took it at the reopen), and no
  second box opens; if the backend still wants the sign the box opens. A refusal is never silent: the box shows the
  problems of the sign and of the reason, and `shownFieldPaths(draft)` (draft.ts) lists only the fields the form
  really draws, so every other problem shows in the notice (the reason of a payment to a seller was listed as shown
  although it has no field, so the refusal showed nowhere). Tests: `voucher-edit-sign.test.tsx`, helper
  `tests/helpers/edit-sign-box.ts`.
