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

The side menu of the area is `RokarMenu` (`app/layout/ModuleMenu.tsx`). Its last item opens the voucher
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
