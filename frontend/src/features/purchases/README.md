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
