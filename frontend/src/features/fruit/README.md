# fruit (plan 17d, task F1)

The پھل screen: fruit a partner takes home, or fruit served to the shop's guests, recorded by item and
quantity. PRD 16.7, FR-16.10 to FR-16.12, FR-7.53, FR-7.9, FR-19.11, C16, C27, E67, A58 to A60.

| Route          | Screen                                                              | Page                  |
| -------------- | ------------------------------------------------------------------- | --------------------- |
| `/rokar/fruit` | the پھل tab next to خرچہ: the form, what the save will do, the list | `pages/FruitPage.tsx` |

It is one more entry in `features/cashbook/expense-tabs.ts` (the tab strip shows itself when there are
two tabs) and one route in `app/router.tsx`. The side menu of the روکڑ area keeps خرچہ current
(`rokarMenuOf`).

## How it works

- `components/FruitForm.tsx`: for whom (a partner, or the guests), the day, the source, then either the
  lot of our shop (a picker over `fruit_lots`, `lot-source.ts`) or the other agent, item and packing,
  the quantity, the weight (only for an item sold by weight), the rate and a note. The item and packing
  of fruit from our shop come from the lot and are shown, not typed.
- The rate is typed. The last rate shows as grey placeholder text and as words under the field, and is
  never put into it (PRD FR-7.53, F43). The backend is asked as soon as the lot (or the item) is known,
  so the hint shows before the rate is typed; what is only empty is told when the user tries to save.
- `components/FruitPreviewCard.tsx`: the value with its working, who is credited, the lot's balance
  before and after, and each partner's part with his total now and after. For the guests the value is
  shared equally, not by the commission shares (C27), and the working says how. All of it is the
  backend's preview, the same function as the save (B5). The screen works out no money.
- `components/FruitListSection.tsx`: the entries page by page with filters (the days, the partner or the
  guests, the item) and the backend's totals over the whole filter. The buttons above the list edit or
  cancel the row that has focus; both work by reversal and only while its day is open (`can_change`).
  The print and Excel of this list come with R37 (D5).
- Edit: the form opens with the entry as `fruit_get` gives it now (id and version), the saved rate in the
  visible field, and asks why (a required `reason`, sent with `fruit_edit`). Cancel: `CancelFruitDialog`
  asks why and sends `fruit_cancel`.
- Everything typed is text in the screen's state (`draft.ts`), so a language switch loses nothing (F39).
  The frame (keys, leave guard, new-entry prompt) is `EntryFrame` of the cashbook module. No cash moves
  on this screen, so the cash question never comes.

## Other places that show fruit

- The statement of a partner: heading `fruit`, detail `fruit_taken` (`features/partners/statement-text.ts`
  and `partners.ftl`): "2 Box Orange @ 1,400 = 2,800", or his equal part of the guests' fruit.
- باقی مال and the lot cards show `qty_taken` next to written-off; the balance is the backend's (C16).

## Files

`api.ts` (the only file that knows the command shapes; tests mock it), `hooks.ts`, `types.ts`,
`draft.ts`, `lot-source.ts`, `list-columns.ts`, `components/`, `pages/`. Test doubles:
`tests/helpers/fruit-data.ts` (the books, with the numbers of E2E-42), `fruit.ts` (mocks), and the
page `/dev/rokar?screen=fruit` (`tests/helpers/dev-bridge-rokar.ts`).
