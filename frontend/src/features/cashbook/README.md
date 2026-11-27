# cashbook (plan 17, task F1)

The روکڑ area: the cash book and three entry screens. PRD 16.1 to 16.6, FR-7.67, FR-7.68, FR-7.62,
FR-6.7, FR-16.2, FR-16.4, FR-16.5, FR-16.6, FR-16.8, FR-16.9, FR-19.11. One cash account: there is no
mode, bank, cheque or cash count anywhere here. The only "mode" is the salary deduction of a
returned loan (`cash` or `salary`).

| Route                  | Screen                                                                       | Page                     |
| ---------------------- | ---------------------------------------------------------------------------- | ------------------------ |
| `/rokar`               | روکڑ بک: three cards, named rows, type filter, summary, backend totals       | `pages/CashbookPage.tsx` |
| `/rokar/expense?type=` | the ONE خرچہ screen: دکان خرچہ, ذاتی خرچہ, ذاتی خرچہ واپسی. `Alt+K` opens it | `pages/ExpensePage.tsx`  |
| `/rokar/lent?type=`    | ادھار رقم: money lent and returned (cash or salary)                          | `pages/LentPage.tsx`     |
| `/rokar/khata?type=`   | الگ کھاتہ: paid out (minus) and received (plus)                              | `pages/KhataPage.tsx`    |

The tabs of the area are `RokarTabs` (`app/layout/ModuleMenu.tsx`), in the title row of each screen (plan 93): روکڑ,
وصولی, ادائیگی, پیشگی, خرچہ, ادھار رقم, الگ کھاتہ, جنرل (plan 95, PRD FR-5.3 v1.57). وصولی, ادائیگی and پیشگی open the
one voucher screen (`/bikri/voucher?type=`, FR-15.10) with the روکڑ tabs on it. The پھل tab (plan 17d) is one more entry in `expense-tabs.ts` and
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

## Plan 99: a simple, exact روکڑ (PRD FR-16.14 v1.60, FR-10.9)

- Two cards (`components/CashCards.tsx`): کل سرمایہ and آخر میں روکڑ, both the backend's (شروع میں روکڑ was
  removed in v1.61, plan 100).
- Every row is named for what it is (`row-text.ts`): the backend sends the `kind` and the records (`refs`:
  parties with a count of all, partner, garden, trip, expense head), and the words come from Fluent, for
  example "وصولی 1، اکرم", "بکری 2، موقع پر جمع، شاہد" and "باغ خرچ 1، پومپ والا باغ". A sale's cash is two
  rows with one entry, so a row's key is the entry and the kind.
- The type filter (سب and the backend's groups) sends the group's kinds as `Cashbook.summary` names them. With
  a type chosen the backend sends no running cash, and the screen says so. The switch "Show old and cancelled
  entries too" sends `show_reversed`; the rows then say they are an old entry or a reversing one.
- `components/CashSummary.tsx`: in and out by type over the window (it does not narrow with the filter); a type
  in it filters the book. A button in the filter row shows it; it starts shut so the book stays in view, and the
  screen remembers how it was left.
- The one-time fix of old opening balances (`components/OpeningsToCashView.tsx`): an amber note on the page only
  while `accounts_openings_to_cash_preview` has rows. The view lists each opening, cash and the opening
  difference before and after and the days that reopen; the save asks the reason and the Owner's sign
  (`ReasonDialog`), then the cash question when cash ends below zero (FR-6.7), and calls
  `accounts_openings_to_cash`.
- The capital card of plan 87 and its print are gone. Tests: `plan99-rokar.test.tsx`.

## Plan 100: the lists under the forms (PRD FR-19.11) and two cards on روکڑ بک

- Under دکان خرچہ, ادھار رقم, الگ کھاتہ and جنرل, `components/EntryListSection.tsx` lists 30 days at a time,
  newest first (a date-descending sort until the user picks a column), with pages, arrows, go-to-date and
  Edit and Cancel on the focused row. Without a person or khata it lists every entry; with one, only his.
  The kinds and columns are in `entry-list-columns.ts`. A save reads the list again (`VOUCHERS_KEY`).
- The روکڑ بک has two cards: کل سرمایہ and آخر میں روکڑ.
