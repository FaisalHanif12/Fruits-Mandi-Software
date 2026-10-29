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
