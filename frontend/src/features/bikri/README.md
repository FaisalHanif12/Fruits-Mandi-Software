# bikri (plans 12 and 13)

The sale book, `/bikri/book` (PRD 12.1 to 12.6, E2E-26): the truck details, the lots, the sale
lines of the selected lot, the live summary (per lot and for the whole sale), the live partner
split, the cash note and the party card of the party the cursor is on. Plan 13 adds saving (F10),
the draft, the saved بکری opened read-only (`/bikri/book?id=12`), the list "آج کی بکری"
(`/bikri/today`), first / previous / next / last, find, and the one-time people (see
`../pukhta_khata`). Later sales, edit, cancel and undo are plan 16 (see the section below); printing is plan 14.

## Rules this module keeps

- **The frontend works out no money and no quantity.** The draft goes to `bikri_preview` as the
  text the user typed (`"20+5"`, `"5,000"`), 50 ms after the last key. Every number on the screen
  is a number that came back, shown with `formatPaisa`. The quantity a new line starts with
  (`remaining_qty`) and the grey rate hint (`rate_hint`) also come from the backend.
- **The preview is the only path.** `useBikriPreview` keeps each answer under the draft it was
  asked for, so an answer for older text never replaces the answer for newer text (F35). While the
  next answer is on its way the last one stays on screen.
- **A fill is a visible value.** The seller of a lot, the item, packing and مارکہ from his last lot,
  the اڈا, زمیندار and driver from the truck, and the quantity of a new line are put in the cell
  like typed text and are Enter stops (`useDraftFills`, `HeaderCard`). The quantity of a new line
  is filled at once, before the buyer: a line with no buyer and no rate is blank and ignored. The
  rate is never filled: the last rate is grey text only, and `Ctrl+D` copies the rate of the line
  above. A one-time seller's نقد (PRD FR-13.10) is shown in its cell while empty, from the
  backend's `totals.cash_paid`, with its working; a typed value is kept as typed.
- **The whole-sale view has no second copy.** The switch on the lines area (the same as the
  summary's) shows the lines of all lots in one grid with a lot column. `stores.all`
  (`union-store.ts`) lists the rows of the lots' stores and hands each change to the lot that owns
  the row. Screens move the cursor into the lines with `stores.focusSaleLine`, which knows which
  grid is on screen.
- **The numbers strip** (`NumbersStrip`) is pinned to the bottom in compact and normal windows with
  the numbers of the selected lot and of the whole sale and the cash note. It has nothing the
  keyboard can reach.
- **Unsaved data lives above the grids** (`draft-store.ts`: the header store, the lots store and
  one sale lines store per lot). A language switch re-creates the grids and loses nothing (F39).

## Saving, drafts and the saved view (plan 13)

- **F10 and Ctrl+S save; Ctrl+Enter saves and, from plan 14, prints.** `useSaveFlow` sends the
  draft as typed to `bikri_save` with `confirm_cash_below_zero` always set. If the live answer says
  cash goes below zero the `CashConfirmDialog` opens first (focus on No; No puts the cursor on the
  نقد cell). If the backend refuses for the cash instead (the answer on screen was older), the box
  opens with the cash worked out for the draft as it is now. One save at a time. A refusal shows each
  problem at its cell by field path and in the problem list, keeps everything typed, and goes away
  when the text typed changes (not when the cursor moves or the language switches).
- **After a save** a message "بکری 5 محفوظ ہو گئی", then a new empty بکری opens with the cursor on
  the truck number (the editor is remade with a new key). The id of the بکری saved last is kept in
  the page state (`lastSaved`): it is the way back to it ("last saved ... open") and the hook that
  edit (plan 16) opens with. `after-save.ts` is the hook for printing: plan 14 calls
  `registerPrintAfterSave`.
- **The draft** (`draft-snapshot.ts`, `hooks/useDraftAutosave.ts`, `draft-offer.tsx`): every 5
  seconds the draft stores go to `drafts_save` as JSON text (nothing for an empty draft, nothing
  when the text did not change, and the draft is thrown away when the user empties the screen). A
  save stops it and throws away a draft that was on its way. At start-up the app offers "Finish the
  last بکری?": restore opens the sale book with it, discard throws it away, Esc keeps it for the
  next start. A snapshot that cannot be read is thrown away without a word.
- **Leaving with typed work** asks first (`LeaveGuard`: save, leave without saving, back), also
  for the shell menu, the "Partner setup" link and the arrows. "Leave" throws the draft away.
- **The saved view** (`components/saved/`) is the layout of the sale book without grids: the
  header facts, a card for each lot with its lines, the summary (one lot or the whole, from the
  stored sums `SavedTotals`), the partners' share and the entries made in the books. It shows only
  stored values and never calls the preview. A cancelled بکری says so in words, in red.
- **The toolbar** (`BikriToolbar`) is the same on the new and the saved بکری: first / previous /
  next / last (`bikri_navigate`, by number), find (`F3`, `bikri_find`), and at the end Save or New.
- **The list** (`pages/TodayBikriPage.tsx`, `list-columns.ts`): filters day, seller, item, status;
  the columns come with the wider layout steps. The نگ column of FR-12.7 (sold / received) does not fit
  beside the other nine at any step (200 px side menu: 1332 px at 1920 px), so the "More columns"
  tick adds it (kept per computer, `bikri.list_more_columns`). Where the room is short one column
  steps aside: خام (compact), the day and time (normal and wide). Plan in `list-columns.ts`,
  helper `shared/grid/moreColumns.ts`.

## Files

| File                                                                                  | What it holds                                                                                                                                       |
| ------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pages/BikriBookPage.tsx`                                                             | The screen: four states, layout at the three window sizes, which party the card shows                                                               |
| `components/`                                                                         | `HeaderCard`, `LotsSection`, `LotMore`, `SalesSection` (+ `ActiveLine`), `LineDetails`, `PukhtaFields`, `SummaryPanel`, `SplitPanel`, `ProblemList` |
| `columns.ts`                                                                          | The columns of the two grids and the Enter stops (lot: 8 cells; line: buyer, qty, rate)                                                             |
| `draft-store.ts`, `draft.ts`, `union-store.ts`                                        | The stores, the lines of all lots as one list for the whole-sale view, and the draft built from them as the backend takes it                        |
| `preview-view.ts`, `grid-views.ts`, `problems.ts`                                     | What the answer shows in each cell, and where each problem goes                                                                                     |
| `hooks/`                                                                              | `useBikriPreview`, `useDraftFills`, `useTruckMemory`, `useSplitFreight`                                                                             |
| `api.ts`                                                                              | The only file that knows the command shapes                                                                                                         |
| `draft-snapshot.ts`, `draft-offer.tsx`, `after-save.ts`                               | The draft as text, the start-up offer, the printing hook                                                                                            |
| `hooks/useSaveFlow.tsx`, `useSaveBikri.ts`, `useDraftAutosave.ts`, `useBikriReads.ts` | The save flow, the call and what it refreshes, the 5 second draft, the reads                                                                        |
| `components/BikriToolbar.tsx`, `BikriFindDialog.tsx`, `components/saved/`             | The arrows and find, and the saved view                                                                                                             |
| `pages/TodayBikriPage.tsx`, `list-columns.ts`                                         | The list of بکری                                                                                                                                    |

## Prints (plan 14)

The seller bill (A4), the buyer slips (A5 or the 80 mm roll, one for each buyer) and the پختہ slip,
in Urdu or English (PRD FR-12.12). Layout and styles are in `shared/print`; this folder only picks
the columns.

| File                                                                                                                      | What it is                                                                                                                                                                                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `print/SellerBillTemplate.tsx`, `BuyerSlipTemplate.tsx`, `PukhtaSlipTemplate.tsx`, `BikriPrintDocument.tsx`, `columns.ts` | The three templates and which goods columns each has. They calculate nothing                                                                                                                                                                                                                    |
| `print/BikriPrintPage.tsx`, `params.ts`                                                                                   | The hidden print window's route `#/print?bikri=..&doc=..&language=..&job=..[&buyer=..]`. It reads `reports_bikri_print_data` (the only command it may call), draws the document, waits for the fonts, and says `print-ready-<job>` with `{ height_px }` or `print-error-<job>` with `{ codes }` |
| `print/BikriPrintView.tsx`, `PrintPreview.tsx`                                                                            | One document on screen (the window, and the in-app preview where printing is not available)                                                                                                                                                                                                     |
| `print/PrintBox.tsx`, `PrintAction.tsx`                                                                                   | The print box (F9 and the "Print" button of the toolbar): language, documents, one buyer, copies, printer or PDF                                                                                                                                                                                |
| `print/usePrintAfterSave.tsx`, `after-save.ts`                                                                            | `Ctrl+Enter`: save, print the ticked documents, open a new بکری. A failed print never undoes the save                                                                                                                                                                                           |

## Tests

`tests/unit/bikri-contract.test.tsx` (the draft the screen sends equals the backend's request
files for Event 1 and E2E-26, and the screen shows the backend's answer),
`bikri-whole-view.test.tsx` and `bikri-union-store.test.ts` (the whole-sale view and its store),
`bikri-strip.test.tsx` (the numbers strip), `bikri-backend-cases.test.tsx` (a one-time seller's نقد, shares not 100%, weight items, no sale
lines, credit not allowed), `bikri.test.tsx` (Event 1 by keyboard, the numbers, ⓘ, E2E-26),
`bikri-save.test.tsx` (the save flow, cash box both ways, refusals, the leave guard, Event 1 to F10),
`bikri-event2.test.tsx` (Event 2 by keyboard to F10), `bikri-autosave.test.tsx` and
`bikri-draft-snapshot.test.ts` and `bikri-draft-offer.test.tsx` (the draft), `bikri-saved.test.tsx`
(the saved view, arrows, find), `bikri-list.test.tsx` (the list), `pukhta.test.tsx` and
`list-extras.test.tsx` (the one merged list and the register),
`bikri-states.test.tsx` (four states, a late answer, the override of a commission, the cash note,
problems at their cells, the one-time buyer, the freight split, a language switch in the middle of
an entry), `bikri-preview-hook.test.tsx` (the 50 ms), `entry-grid-pieces.test.tsx`,
`cash-confirm-dialog.test.tsx`, `suggest-input.test.tsx`. The fake backend is
`tests/helpers/bikri.ts`, with the files the backend wrote: `tests/fixtures/bikri_preview_event1`
and `bikri_preview_e2e26`, each as `.json` (the answer) and `_request.json` (what the screen
sends). They are copied from `backend/crates/services/tests/fixtures/`, never edited. The Windows spec is
`tests/e2e/specs/12-bikri-entry.e2e.mjs`; plan 13 adds `13-bikri-save.e2e.mjs` (Events 1 and 2 saved
by keyboard) and `14-bikri-draft.e2e.mjs` (E2E-17).

The prints: `print-layout.test.tsx` (the three templates in both languages, numbers isolated, paper
class, digits), `print-foundation.test.ts` (the footer text, the print stylesheet, the Fluent ids,
the commands), `print-route.test.tsx` (the hidden window calls one command and emits its events),
`print-box.test.tsx`, `print-after-save.test.tsx`, `print-settings.test.tsx` (the Printing tab),
`print-speed.test.tsx` (15 pages, NFR-SPD-14). Canned data: `tests/helpers/print-data.ts`, which
follows the generated binding types. The Windows spec is `15-bikri-print.e2e.mjs` (PDFs in both
languages); `tests/e2e/print-shots.mjs` makes the Chrome PDFs and pictures.

## Goods left, later sale, edit, cancel, undo (plan 16)

- **باقی مال** (`pages/StockPage.tsx`, `stock-columns.ts`, `components/stock/`): the list of
  lots with goods left, the card of the lot under it, and the write-off boxes. Enter on a lot starts
  a later sale (`/bikri/later?lot=40`, `pages/LaterSalePage.tsx`).
- **One form for sale lines** (`components/LinesForm.tsx`): used by the later sale and by the edit
  of a later entry (`components/LaterPostingEdit.tsx`). The preview and the save are the
  backend's later-sale or edit commands (`preview-source.ts`, `hooks/useSaveBikri.ts`); the hooks
  `useBikriPreview` and `useSaveFlow` are the sale book's, given another source.
- **Edit mode of the sale book**: `/bikri/book?id=12&edit=57` opens `EditBikri` in
  `pages/BikriBookPage.tsx`. `hooks/useEditSession.ts` loads the entry (`bikri_open_for_edit`),
  reads the saved بکری for names, puts the backend's draft into the stores
  (`draft-load.ts`: ids to records, keys to row ids, the ids of saved lots and lines in hidden
  cells `LOT.lotId` and `SALE.saleId`) and keeps a copy of every loaded cell (`edit-changes.ts`).
  The changed cells are marked and listed (`edit-view.ts`, `edit-fields.ts`,
  `components/EditChangesCard.tsx`). The session is not under the `['bikri']` query key, so a save
  never remakes the stores under the user's hands. An edit keeps no draft and never touches the
  draft of a new بکری.
- **Cancel** (`components/CancelBikriFlow.tsx`): `bikri_cancel_preview` for the entries and cash,
  the reason, the cash question, `bikri_cancel`. **Undo** (`hooks/useUndoSave.tsx`): the button on
  the message after a save and Alt+Z for 10 seconds.
- **Tests**: `bikri-stock.test.tsx`, `bikri-later-sale.test.tsx`, `bikri-edit-cancel.test.tsx` (edit,
  cancel and undo), `bikri-draft-load.test.ts` (a draft from the backend in the stores and the
  comparison), the toast button in `ui-overlays.test.tsx`. Canned answers: `tests/helpers/bikri-plan16.ts`
  (the double) and `bikri-plan16-data.ts`. `bikri-plan16-contract.test.tsx` runs the screens on the
  backend's own files (`tests/fixtures/bikri_stock_list`, `bikri_write_off_*`, `bikri_later_sale_*`,
  `bikri_open_for_edit_event1`, `bikri_edit_*_akram_610`, `bikri_cancel_*_event2`,
  `bikri_undo_event1`, each with its `_request.json`; read through `tests/helpers/bikri-plan16-real.ts`).
  It checks that the draft an edit loads goes back unchanged, that a changed rate gives exactly the
  request file, and that the later sale, cancel, write-off and undo send what the backend's tests send.
  The Windows specs are `18-bikri-later-sale`, `19-bikri-edit`, `20-bikri-cancel`, `21-bikri-undo`.

## Trip goods in the sale book (plan 16b, PRD FR-12.26, C21, C25)

- A lot whose seller is اپنا مال (`isOwnLot`, code 030119998; offered only by the lots grid's seller
  column, scope `lot_seller`) shows `components/OwnLotPanel.tsx` under the lots grid: the choice of
  the purchase, its facts, and what the backend worked out for the lot (`LotPreview.own`): the
  commission, the seller credit with each trip partner's part, the cost and the result, each with
  its working. A saved lot shows the same in `SavedLotCard` (`SavedLot.own` and `purchase`).
- Keys: after the seller is taken focus goes to the purchase choice; choosing a purchase fills the
  item, packing and quantity in the visible cells and `Select`'s `afterChoose` moves focus to the
  quantity (Radix would otherwise give it back to the choice). Enter then never stops at the نقد
  cell, which does not apply.
- A new own lot sends no cash and no advance tick (`cash_paid` null, `adjust_advance` false); a
  saved one (an edit) sends back what the backend gave, so an edit that changes nothing sends the
  draft as it came (`hooks/useDraftFills.ts` keeps a saved lot's cash).
- The goods left list shows the trip's purchases apart (`OwnStockSection`, `own-stock-columns.ts`):
  the card has every figure, the write-off and its undo (nothing is posted), and a way to open the
  trip.
- **Tests**: `bikri-own-real-answers.test.tsx` (the panel, the saved view of E2E-34 and E2E-41, the
  goods left list, write-off and undo and an edit, on the backend's `bikri_*_own_lot_*` and
  `bikri_stock_list_own_rows` files, read through `tests/helpers/bikri-own-real.ts`),
  `bikri-own-keyboard.test.tsx` (the whole entry by keyboard with a language switch in the middle),
  `bikri-own-real-shapes.test.ts` (canned data has the real shape; every working the backend can
  send has words in both languages). The development page shows it all with `?own`
  (`/dev/bikri?own`, `&id=1`, `&id=1&edit=4`, `&stock`). The Windows spec is `23-trip-goods-in-bikri`.
