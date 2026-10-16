# pukhta_khata (plan 13)

The one-time buyers and sellers (PRD Section 13): the register screen and the one merged list of
known people in the sale book.

- **Register** (`/bikri/pukhta`, `pages/PukhtaRegisterPage.tsx`, `columns.ts`): every line of a
  one-time buyer and every lot of a one-time seller (`pukhta_register`), with filters day from and
  to (empty means no limit), side, name, phone, paid or due. The name and the phone are searched as
  typed and never converted (F41). A row opens its بکری read-only. A cancelled line is marked in
  words and is neither paid nor due. The نگ column of FR-13.6 does not fit beside the other columns at
  any step, so the "More columns" tick adds it (kept per computer, `pukhta.more_columns`) and one
  column steps aside where the room is short: the date (compact), the phone (normal), the
  commission (wide). Plan in `columns.ts`, helper `shared/grid/moreColumns.ts`.
- **Merged list** (PRD FR-7.29): in the name and the phone of a lot or a line, the one-time people
  already recorded come first (`search_pukhta_people`, both names, phone, last day), then the Urdu
  spellings of the typed word. `usePukhtaPeople` asks the backend; `PukhtaPersonRow` draws a row;
  the rows go into `NamePair` (`knownRecords`) and `SuggestInput`. Taking a person fills both names
  and the phone exactly as recorded; none is highlighted until an arrow moves onto it, so Enter on a
  new name still moves on, and a row is never taken by Space or by leaving the field.
- The frontend sorts and ranks nothing: the backend gives the people newest first and the register
  rows in order.
