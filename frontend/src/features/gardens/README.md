# gardens (plan 17c)

باغات, the gardens (PRD 12.11, FR-5.2, FR-7.67, FR-12.29, FR-12.30, FR-12.34 to FR-12.38): the list
at `/gardens`, a new garden at `/gardens/new` and one garden at `/gardens/$id`. The main bar item
sits after روکڑ and before مارکیٹ سفر.

- A new garden asks in this order: the shares (which partners are in it and each one's %, with the
  backend's running total), then where each partner's money comes from (دکان سے or اپنی جیب سے),
  then the details (both names, the day, the main fruit, the rent, the end of the season, the first
  installment, a note). The form says in words that a part from a pocket posts nothing
  (FR-12.30). The shares and sources use `PartnerSharesEditor` of `shared/ui`; the names use
  `NamePair` with the spelling check (FR-7.66).
- One garden screen, one type list: قسط, باغ خرچ, دوسری دکان میں فروخت. `GardenEntryEditor` holds the
  day, the amount and the sources of the partners once, so changing the type keeps them and clears
  the rest (FR-7.67). The parts of a payment, the running balance after it and the cash after it
  come from the backend's preview (the same function as the save); the screen adds nothing
  (CLAUDE.md rule 1). `useEntrySaveFlow` does the save: the cash question (FR-6.7) comes first,
  one save runs at a time, a refusal shows at its field and nothing typed is lost.
- The history is one list from the backend (installments, costs, sales, fruit, the closing), oldest
  day first, with the running balance after each row (FR-12.37). The row with focus shows its parts
  or its closing credits and can be cancelled with a reason while the garden is open.
- Closing (FR-12.38): `CloseGardenCard` asks for the day, shows the backend's preview of the closing
  (money in, cost, result, each partner's part), and opens a confirm box with Back focused
  (FR-7.8). A garden with nothing in its account still closes, and the card says nothing is posted
  (E64). A closed garden shows a note instead of the form, and a button that undoes the closing
  by reversal while its day is open.
- Lists have page buttons (FR-19.11, plan 38): `useGardensList` and `useGarden` (the history) read one
  page at a time through `usePagedQuery`; the totals under the list are the backend's over the whole
  filter. `useAllGardens` reads every page for the garden filter box.
- What is typed is text in the screen's state, so a language switch loses nothing (F39). `draft.ts`
  holds the text side that tests run without the screen.
- `api.ts` is the only file that knows the command shapes. Tests mock `api.ts`
  (`tests/helpers/gardens.ts` over `gardens-data.ts` is the fake backend with the numbers of
  E2E-40; the development page `/dev/gardens` answers from the same data).
- A garden is offered as the seller of a lot in the sale book through the account scope
  `lot_seller`; the backend decides which accounts that scope answers (open gardens only).
- The overview of the partners shows the open gardens and trips under each card (`OpenVentures` in
  `features/partners`).
- The partner statement shows a garden's lines (`features/partners/statement-text.ts`): a payment
  from the shop's cash (`garden_paid`: the garden's number, the reason as typed, and "from the
  shop") and the closing (`garden_closed`: cost, sales, result, his %, his part and his money
  back). The statement text only joins the words the backend sent; it never works out an amount.
- Real answers: the fixtures copied from the backend (`tests/fixtures/gardens_*.json`) are read as
  the generated types in `tests/helpers/gardens-real.ts`. `gardens-real-shapes` proves the screens
  send exactly what the backend's own tests send; `gardens-real-answers` proves the screens show
  what it answers. The Windows run is `tests/e2e/specs/29-gardens.e2e.mjs` (LOCAL-ONLY on a Mac).

- **Goods for another shop (plan 90, PRD FR-12.41, FR-12.32, FR-12.36).** The split column of the item lines reads
  "دوسری دکان کے نگ" (`goods-line-col-other-qty`; the two help lines quote it by message reference). A sale in another
  shop, on a garden and on a trip, offers only the lines (`ArrivalRow`, `PurchaseRow`) with `qty_other_left` above 0,
  with that number in the option, through `other-shop.ts` (`offeredForOtherShop`; the chosen line stays, so an edit still
  shows its own line). When none has any, `OtherShopNone` says so and where to set it. Picking a line fills the shop
  box with the line's `shop_name` (`shopAfterPicking`: a typed name is never taken away). More نگ than the line has
  for another shop is refused under the qty field and F10 sends nothing (`qtyAboveOther`: a whole number is compared
  here, a sum such as `30+14` is read by the backend, whose preview says `qty_from_our_shop`). The backend still accepts
  such a sale (it moves the rest from our shop's part, plan 35 D1): this screen no longer lets it through. The choice
  lists read every page (`useAllArrivals`, `useTripStockPurchases`), not the one page the list below is on. The trip's
  purchases list is paged (`useTripPurchases`, `purchases_list` with `trip_id`) with the backend's totals row.

- **A busy garden stays organized (plan 96, PRD FR-12.11, FR-12.40, FR-12.41, FR-19.11, NFR-USE-16).** A garden has
  about 90 goods arrived entries, so the page is short and nothing in it moves as entries come and are sold:
  - the head of the garden holds the four key figures that are always in view (running balance and commission with
    their working, rent still due, money in; `KeyFigures`); the full figures and each partner's money fold under
    one heading (`GardenFiguresFold`, `FoldSection`, shut at first);
  - `GardenArrivalsSection` is the main list: a search box (number, item or other shop, `useArrivalsList` with
    `search`), 30 rows a page (`RECORDS_PAGE_SIZE`), the newest first (the list's own order; a click on a heading
    sorts the whole list by the backend), the backend's totals, and a height of its own (`--arrivals-region-h`)
    so the rows scroll inside it and the page grows no further;
  - the details of the entry with focus are `ArrivalDetail`: the buttons first, then the figures two to a line,
    the sales that took goods from it (the first four, the rest on a button). Beside the list from the normal
    window width (a docked panel, `--arrivals-panel-w`, so the list keeps 5 columns in 660 px; the commission,
    the day to sell and the day it arrived join the list at the wide step); on a narrow window they open in a
    window of their own with Enter or a double click, and a button there closes it before it opens its own box;
  - keys: ↑ ↓ move between entries and the panel follows; Enter goes to the first button of the panel (or opens the
    window); Esc goes back to the entry in the list (`useEscape('edit')`, PRD FR-7.18);
  - a saved goods arrived entry (`GardenEntryEditor.onArrivalsSaved`) clears the search, goes to the first page,
    selects the newest new entry and tints the new rows for 6 seconds (`ListGrid.markedRowIds`); the form is ready
    for the next entry where it was and the page does not scroll.
    The development page `/dev/gardens?many` answers with 90 entries (`tests/helpers/arrivals-many.ts`: search, sort,
    paging and totals like the backend's; `?few` has 6); `gardens-arrivals-many.test.tsx` covers it in both languages.
    The trips page keeps its own purchases list (`TripPurchasesSection`, `features/purchases`): it does not use these
    components.
