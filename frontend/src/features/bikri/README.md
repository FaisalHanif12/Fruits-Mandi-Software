# bikri (plan 12)

The sale book, `/bikri/book` (PRD 12.1 to 12.5, E2E-26): the truck details, the lots, the sale
lines of the selected lot, the live summary (per lot and for the whole sale), the live partner
split, the cash note and the party card of the party the cursor is on. Nothing is saved here yet.
Saving, drafts, edit and cancel are plan 13.

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

## Files

| File | What it holds |
|---|---|
| `pages/BikriBookPage.tsx` | The screen: four states, layout at the three window sizes, which party the card shows |
| `components/` | `HeaderCard`, `LotsSection`, `LotMore`, `SalesSection` (+ `ActiveLine`), `LineDetails`, `PukhtaFields`, `SummaryPanel`, `SplitPanel`, `CashNote`, `ProblemList` |
| `columns.ts` | The columns of the two grids and the Enter stops (lot: 8 cells; line: buyer, qty, rate) |
| `draft-store.ts`, `draft.ts`, `union-store.ts` | The stores, the lines of all lots as one list for the whole-sale view, and the draft built from them as the backend takes it |
| `preview-view.ts`, `grid-views.ts`, `problems.ts` | What the answer shows in each cell, and where each problem goes |
| `hooks/` | `useBikriPreview`, `useDraftFills`, `useTruckMemory`, `useSplitFreight` |
| `api.ts` | The only file that knows the command shapes |

## Tests

`tests/unit/bikri-contract.test.tsx` (the draft the screen sends equals the backend's request
files for Event 1 and E2E-26, and the screen shows the backend's answer),
`bikri-whole-view.test.tsx` and `bikri-union-store.test.ts` (the whole-sale view and its store),
`bikri-strip.test.tsx` (the numbers strip), `bikri-backend-cases.test.tsx` (a one-time seller's نقد, shares not 100%, weight items, no sale
lines, credit not allowed), `bikri.test.tsx` (Event 1 by keyboard, the numbers, ⓘ, E2E-26),
`bikri-states.test.tsx` (four states, a late answer, the override of a commission, the cash note,
problems at their cells, the one-time buyer, the freight split, a language switch in the middle of
an entry), `bikri-preview-hook.test.tsx` (the 50 ms), `entry-grid-pieces.test.tsx`,
`cash-confirm-dialog.test.tsx`, `suggest-input.test.tsx`. The fake backend is
`tests/helpers/bikri.ts`, with the files the backend wrote: `tests/fixtures/bikri_preview_event1`
and `bikri_preview_e2e26`, each as `.json` (the answer) and `_request.json` (what the screen
sends). They are copied from `backend/crates/services/tests/fixtures/`, never edited. The Windows spec is
`tests/e2e/specs/12-bikri-entry.e2e.mjs`.
