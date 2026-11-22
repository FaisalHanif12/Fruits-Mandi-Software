# trips (plan 16b)

مارکیٹ سفر, the market trips (PRD 12.10, FR-5.2, FR-7.67, FR-12.24 to FR-12.33): the list at
`/trips`, a new trip at `/trips/new` and one trip at `/trips/$id` (`?purchase=7` chooses a
purchase). The main bar item sits after روکڑ; D4R adds باغات before it.

- One trip screen, one type list: خریداری, سفر خرچ, دوسری دکان میں فروخت. `TripEntryEditor` holds the
  day, the amount, the note and the sources of the partners once, so changing the type keeps them
  and clears the rest (FR-7.67). The fields of a purchase are in `features/purchases`.
- Where the money comes from is a choice for each partner on every entry, filled from the trip
  and changeable (A54). The parts, the spread of a cost, the credits and the cash after the entry
  all come from the backend's preview (the same function as the save); the screen adds nothing
  (CLAUDE.md rule 1). A shop part comes off the partner's total, a pocket part is only recorded.
- `useEntrySaveFlow` (in `shared/hooks`, also used by the cash book area) is the one save for the three types: the cash question (FR-6.7) comes first,
  one save runs at a time, a refusal shows at its field and nothing typed is lost.
- Lists have page buttons (FR-19.11, plan 38): `useTripsList` and `useTrip` (the history) read one
  page at a time through `usePagedQuery`; the totals under a list are the backend's over the whole
  filter. `useAllTrips` reads every page for the trip filter box of analytics.
- What is typed is text in the screen's state, so a language switch loses nothing (F39). `draft.ts`
  holds the text side that tests run without the screen.
- `api.ts` is the only file that knows the command shapes. Tests mock `api.ts`
  (`tests/helpers/trips.ts` over `trips-data.ts` is the fake backend; the development page
  `/dev/trips` answers from the same data).
- The pieces that gardens will share (the shares editor, the sources of one entry, the parts and
  the credits) are in `shared/ui` (`PartnerSharesEditor`, `EntrySources`, `EntryPartsCard`,
  `ShareAmounts`); their words are in `ventures.ftl`.

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
