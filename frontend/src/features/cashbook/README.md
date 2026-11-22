# cashbook (plan 17, task F1)

The روکڑ area: the cash book and three entry screens. PRD 16.1 to 16.6, FR-7.67, FR-7.68, FR-7.62,
FR-6.7, FR-16.2, FR-16.4, FR-16.5, FR-16.6, FR-16.8, FR-16.9, FR-19.11. One cash account: there is no
mode, bank, cheque or cash count anywhere here. The only "mode" is the salary deduction of a
returned loan (`cash` or `salary`).

| Route                  | Screen                                                                       | Page                     |
| ---------------------- | ---------------------------------------------------------------------------- | ------------------------ |
| `/rokar`               | روکڑ بک, paged, opening and closing cash, backend totals                     | `pages/CashbookPage.tsx` |
| `/rokar/expense?type=` | the ONE خرچہ screen: دکان خرچہ, ذاتی خرچہ, ذاتی خرچہ واپسی. `Alt+K` opens it | `pages/ExpensePage.tsx`  |
| `/rokar/lent?type=`    | ادھار رقم: money lent and returned (cash or salary)                          | `pages/LentPage.tsx`     |
| `/rokar/khata?type=`   | الگ کھاتہ: paid out (minus) and received (plus)                              | `pages/KhataPage.tsx`    |

The tabs of the area are `RokarTabs` (`app/layout/ModuleMenu.tsx`), in the title row of each screen (plan 93). Its last item opens the voucher
screen of the بکری area (FR-15.10). The پھل tab (plan 17d) is one more entry in `expense-tabs.ts` and
one more route; the tab strip shows itself when there are two.

## How an entry screen works

- `components/EntryFrame.tsx` is the frame all three share: title, type list (`SegmentedControl`),
  New (`F2`) and Save (`F10`, `Ctrl+S`), `Esc` back to the cash book, the leave guard, the "save
  changes?" prompt and the cash question (`CashConfirmDialog`, PRD FR-6.7).
- Everything typed is text in the screen's state (`*-draft.ts`), so a language switch loses nothing
  (F39). The draft files also hold what a type change keeps (FR-7.67) and the field paths.
- The preview comes from the backend while typing (`usePreviewQuery`, 50 ms): `vouchers_preview_expense`
  for the خرچہ screen, the generic `vouchers_preview` for the other two. The screens show each
  partner's share, a partner's total now and after, balances, warnings and the cash after the entry.
  They work out no money (CLAUDE.md rule 1). Problems that only say "empty" wait until the user tries
  to save (`entry-support.ts`).
- The save is `useEntrySaveFlow` (`shared/hooks`): the cash question first, one save at a time, the
  backend's refusal at its fields.
- Money entered by a button ("Fill the form" of repeat last month) is put into the visible field and
  read again by the backend on save (CLAUDE.md rule 11).

## Choices to know

- Changing the type clears the fields of the other type and keeps date, amount and narration
  (FR-7.67). On ادھار رقم the person stays, and on الگ کھاتہ everything stays, because the types
  share all their fields. After a save the partner, person or khata stays chosen so his list stays in
  view; only typed text counts as unsaved work.
- The FR-7.68 note ("is this a partner's own spending?") has a button; the backend finds the partner
  (either script). Nothing switches by itself.
- Lists are `PagedList` (loading, error with retry, empty message, rows; count and totals from the
  backend). Edit and cancel of the new kinds are not on these lists yet: they need the backend's
  `vouchers_edit` and `vouchers_cancel` for the new kinds (T2).

## Files

`api.ts` (the only file that knows the command shapes; tests mock it), `hooks.ts`, `types.ts`,
`screen-type.ts` (types and addresses, read without loading a screen), `list-columns.ts`,
`expense-draft.ts`, `lent-draft.ts`, `khata-draft.ts`, `entry-support.ts`, `expense-tabs.ts`,
`components/`, `pages/`. Test doubles: `tests/helpers/cashbook-books.ts` (plain books),
`cashbook.ts` (mocks), `dev-bridge-rokar.ts` (the page `/dev/rokar`).

## Plan 76: linked khatas, person types and the dues of garden owners and others (PRD FR-16.6, FR-16.13)

- A الگ کھاتہ and a person who is lent money have a **link**: nobody (a khata only, the firm's own), the shop (every
  movement is split between the partners by the shares of its day) or one partner. `link-draft.ts` holds the form
  value (`LinkDraft`) and builds the link the backend takes; `components/LinkChoice.tsx` is the one pair of fields
  (kind, and the partner for a partner link), used by the add dialog, the khata card and the person card.
  The backend splits and posts; the screen only sends the link.
- `AddAccountDialog` (the `F4` form) asks the link of a new khata, and the type (staff, garden owner,
  market-trip person, other) and the link of a new person. `KhataLinkCard` (in the khata list) shows the link in
  words and changes it (`khata_set_link`); the button is off while the khata's balance is not 0, with the words
  `cashbook-link-locked` (the backend refuses it too: `ERR_LINK_LOCKED`). `PersonDetailsCard` (in the ادھار رقم
  list) shows his type and link and changes them (`lent_person_update`); while he owes, the link is fixed
  (`ERR_LINK_FIXED_WHILE_OWED`). The money lent form has a "For" field: it shows the person's link, is fixed while
  he owes, and sends the link only when he owes nothing (`linkToSend`). A salary return for a partner-linked
  person is refused by the backend (`ERR_SALARY_NOT_ALLOWED_FOR_LINKED`) and shown at its field.
- The dashboard tile "Garden owners' and others' dues" opens `/others-dues` (`pages/OthersDuesPage.tsx`):
  the money given to people who are not staff and what came back, one month at a time (`useListWindow`, source
  `lent_dues`), with the filters person type and link (the shop, or one partner), the people with something open
  and the entries of the month. Every figure and total is `lent_dues`'s. Enter opens the person on the ادھار رقم
  screen. Tests: `others-dues-page.test.tsx`, `khata-link.test.tsx`, `lent-link.test.tsx`.
- The lists under the ادھار رقم, partner takes (خرچہ) and capital forms show **one month at a time**
  (`ListWindowBar`, PRD FR-19.11): the hooks take the month's `from` and `to`.
- **The capital card (سرمایہ کہاں ہے, plan 87, PRD FR-16.14)** is a folded section of `/rokar`
  (`components/CapitalWhereCard.tsx`, pieces in `components/capital/`). It reads nothing until it is opened and is
  behind the partner code (`ERR_PARTNER_LOCKED` shows the code box). It reads like a statement: `CapitalHeadline`
  (the partners' total capital, which is the backend's `net_paisa`, with ✓ "both sides are equal" and the day it is
  as of), two `CapitalBlock` tables (`have_lines`: to receive and in hand; `owe_lines`: to pay) with the backend's
  subtotals, `CapitalNet`, `CapitalWhose` (each partner's کل رقم, his share in basis points written as a percent and
  a bar, the opening capital, the open advances) and `OpeningDifferenceNotice`, the amber warning for the opening
  difference. **Every amount of the blocks and the net is unsigned; the side is the block, never a minus sign.**
  Cash below zero comes from the backend as the cash line of the paying block and is written "cash short" with a
  note; a net that is short (`net_side` is `owe`) carries the word "short". A line that is one account (what is owed
  for labour, freight ...) is named by the account. A partner's total below 0 is written without its sign, in the
  due colour, with the words "owes the firm" (`Num withoutSign`). The net, the headline and the whose total are
  unsigned from the backend, each with its side (`net_side`, `whose_lines_side`; `owe` is written "short").
  - The warning shows what the difference is made of (one line for each group of `difference.groups`, each with the
    backend's net and side) and, folded, every account that makes it (each opens its ledger). The button "put the
    capital into روکڑ" shows only when `difference.placeable_partners` is not empty. It opens `PlaceCapitalDialog`
    (the `ReasonDialog` of the reopen-and-fix flow with its sign field) and calls `partners_place_opening_capital`
    (`partnersPlaceOpeningCapital` in `api.ts`, kept here because only this card calls it) with those partners. A
    wrong sign is told in the box and nothing is read again; a yes closes the box, shows a toast, and reads the card,
    the cash book, the partners and the day again (`useReadAgain`).
  - Every line opens the page the backend names for it (`line.link`): `capital-links.ts` maps a `CapitalLink` and the
    record it names to a destination (`targetOfLink`, plain data, tested) and `useOpenCapitalTarget` navigates. The
    cash line brings the cash rows into view (the card is on that page).
  - The "show it as of" box is `GoToDate`; the card is read as of that day (`as_of` is sent as typed, `DD-MM-YYYY`,
    null is today). Print opens the preview of the card in place of the book (`TablePrintPreview`; the hidden print
    window has the kind `capital`, `app/print-kinds`), with the same day. Excel is not offered for this card.
  - `capital-types.ts` re-exports the generated bindings. The print is `cashbook_capital_print_data` (the card laid out as a table by the backend, ids in `print.ftl` as `print-capital-*`) and `cashbook_capital_print`. Tests: `cashbook-capital-where.test.tsx`,
    `cashbook-capital-links.test.ts`, `plan78-api-contract.test.ts` and `plan78-real-answers.test.tsx` (the backend's real fixtures: worked day, before and after the fix, as of a day, print data, refusals); the hand-made data of the four states is
    `tests/helpers/capital-card.ts`; the development page `/dev/rokar?screen=book&umar` shows Umar's data before the
    fix and the fix with the sign "adnan".
