# purchases (plan 16b)

The خریداری of a market trip (PRD 12.10, FR-12.24 to FR-12.28). A purchase has no screen of its
own: it is made, listed and written off on the trip screen (`features/trips`, Tech Stack 5.2).

- Exports (through `index.ts`): the fields of the purchase form (`PurchaseFieldsForm`), its draft and
  input (`buildPurchase`), its live numbers (`PurchaseNumbersCard`, `usePurchasePreview`), the card
  of one purchase (`PurchaseDetail`), the write-off and cancel boxes, and the columns of the list.
- A purchase is paid at once: no supplier account, credit, bank or cheque. Each partner's part and
  where it comes from (shop or pocket) are the backend's (C23); a pocket part posts nothing.
- The write-off posts nothing (FR-12.28): the cost only shows as a loss in the trip's result. The
  same two boxes serve the trip screen and the list of باقی مال.
- The sale book picks a purchase for a lot of اپنا مال from `usePurchasesList` (open trips with
  goods left); the panel is `features/bikri/components/OwnLotPanel.tsx`.
- Every figure is the backend's with its working (`CalcInfo`); nothing is worked out here.

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
