# Plan 71: lot names with a counter, sorted seller prints, and a simple بکری header

| | |
|---|---|
| Asked by | Umar, 12-11-2026, while testing D8 |
| Branch | `feature/d8-demo-feedback` |
| Requirement change | PRD v1.34 (this plan is Umar's approval) |

## 1. Umar's decisions (12-11-2026)

| # | Decision |
|---|---|
| L1 | Every lot has a name made of the seller, the item and a counter: احسان، آم-1, then احسان، آم-2 for his next mango lot, and احسان، کیلا-1 for his first banana lot. English: "Ehsan, Mango-1". |
| L2 | The counter belongs to one seller and one item, and runs for the seller's whole life: it never resets, and a number is never reused, even after a cancel. |
| L3 | The first lot is numbered too (-1). |
| L4 | The number is given when the lot is first saved, like the بکری number. Before the save, the screen shows the number the lot will get (the backend works it out from the saved lots and the lots above it in the same draft). |
| L5 | An edit that changes a lot's seller or item gives the lot the next number of its new seller and item. The old number stays unused. |
| L6 | Old lots get numbers by a migration: per seller and item, in order of first posting date, then lot id. No amount, entry or chain hash changes. |
| L7 | Every print of a seller's lots shows them sorted, whatever order they were typed in: the seller bill (R1) and the seller's day slip (R35): by item (Urdu order of the item name, English order in an English print), then by counter. Example: typed آم-3, کیلا-1, آم-2 prints آم-2, آم-3, کیلا-1. |
| L8 | The lot name is shown wherever a lot is named today: the lot labels of the بکری screen (FR-12.6), باقی مال, later sale, edit, R2, R6, R10, R27, party ledgers day detail, and the assistant. |
| L9 | The assistant: "ehsan ke 200 peti aam" with more than one mango lot of Ehsan asks کون سا؟ with a button per lot (name, date, received qty). A question that names the lot ("ehsan aam-2", "ehsan-2 ke aam") answers it at once. |
| H1 | The بکری header keeps only the date and the بکری number. Truck number, زمیندار, اڈا and everything behind مزید (book, bill date, driver, driver phone, total freight, store, note) leave the screen. |
| H2 | The features that lived only on those fields are retired: truck memory and the same-truck warning (FR-7.49, FR-7.50), the header freight split over lots (C17), the header زمیندار as the first lot's seller, the header store as a default, and report R8 (اڈا رپورٹ). Freight, store and seller stay on each lot. |
| H3 | Old saved بکری keep their truck number, اڈا and the rest in the database; an old print shows them when they are there. New بکری have none. Bill date is the entry date. |
| H4 | The date and the بکری number sit in the top row of the page, beside the title and the buttons, so the whole header card is gone and its space goes to the lots and lines. Nothing may overlap or scroll sideways at 1024, 1366 and 1920, in both languages. |

## 2. Contract (backend proposes the exact names; the lead records them here)

- `LotPreview` and `SavedLot` gain `lot_counter: number` (null in a preview for a lot that cannot be worked out yet) and the lot name parts the screen needs to show "احسان، آم-2" through `displayName` (the seller and item names already travel; the counter is new).
- `bikri_lots.lot_counter` (INTEGER NOT NULL after the migration; unique per seller and item among lots ever saved) in migration `0022_lot_counter.sql`.
- Print data of R1 and R35 sorted as L7 (backend sorts; the frontend draws in the order given).
- `assist_truck_memory` and the same-truck check retired; R8 out of `reports::catalog` and the assistant's catalogue (D6 prompt files copied word for word; prefix stays under 11,600).
- Header input: the removed fields may still arrive from an old draft; the backend ignores them for a new بکری (no refusal).

## 3. Tasks

| # | Task | Owner |
|---|---|---|
| T0 | PRD v1.34 and affected documents (Architecture data dictionary, Tech Stack, Code Standards golden list if any, D6 prompt for R8 and lot names) | docs helper, lead commits |
| T1 | Migration, counter at save and in preview, edit rule L5, sorted R1 and R35, lot name in every read and print, retire truck memory and R8, assistant L9; tests; `pnpm gen:types` | backend |
| T2 | Header card out, date and number in the top row, lot names in every label, removed calls (truck memory, freight split, header seller and store defaults), R8 out of the report menu; both languages; screenshots at three sizes | frontend |

## 4. Acceptance

- Lots of one بکری typed آم, کیلا, آم for Ehsan show احسان، آم-1, احسان، کیلا-1, احسان، آم-2 before and after save; his next بکری's mango lot is آم-3.
- Seller bill and seller day slip of ten lots typed in mixed order print sorted (L7).
- Old data migrates with every number, entry and hash unchanged; golden tests pass with no number changed.
- The header shows only date and number in the top row; no sideways scroll at the three sizes.
- The assistant asks which lot when needed, and answers a named lot.
