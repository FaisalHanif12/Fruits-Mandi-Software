# features/masters

انڈکس: the nine master lists of PRD 10.3 (plan 06). One generic screen, configured per master in
`configs.ts`: city, item, packing, adda, marka, store, narration, expense head, staff. There is no bank master: all money is روکڑ (PRD 10.3, FR-10.4).

- `pages/MastersMenuPage`: the menu of the nine lists (`/index`).
- `pages/MasterListPage`: one list (`/index/<path>`): search (the backend search, both scripts),
  "show inactive", rows in `shared/grid/ListGrid`, four states.
- `pages/MasterFormPage` and `components/MasterForm`: add (`/index/<path>/new`) and change
  (`/index/<path>/<id>`). The code with the next one suggested, `NamePair` for the two names with
  the spelling check (PRD FR-7.66), the fields of that master, read-only facts from the backend.
- The account list (`account-columns.ts`, `pages/AccountListPage`): the opening debit and credit show at
  the wide step (1920 and up). Below it a "More columns" tick adds them; at the normal step city and
  credit allowed step aside, so the list never scrolls sideways (`tests/unit/list-widths.test.ts`
  adds the widths up). The tick is kept by `shared/hooks/useRememberedFlag` on this computer, until
  `user_prefs` has a `filters.<screen>` key.
- A master is never deleted. It is made inactive, after a confirm (PRD E21).
- `types.ts` is INTERIM until backend's generated bindings for the masters commands exist. Only
  `api.ts` knows the wire shape.
- To add a master: add its configuration, its Fluent messages (`masters.ftl`, ur and en) and its
  backend kind. Nothing else.

The master lists and the account list load page by page (plan 17b). A typed filter is matched
against the loaded rows, so while it is typed the rest of the pages is read (`useLoadRest`); the
openings screen reads every page too, because it types against every active account.

The tag (نشان, plan 17e, PRD 10.2): the account form has a plain field for it after the names
(`shared/ui/TagInput`: English capital letters and dots, made capital when the user leaves the field,
never converted to Urdu). It is sent as typed; the backend makes it capital and checks it
(`ERR_TAG_INVALID` at `tag`). The tag is shown after the name only through `displayName`, so the
account list, the chart tree and the similar-accounts panel carry it without any code of their own.
A record a screen keeps from a picker is made by `pickedOf` (`shared/accounts/picked.ts`), which
keeps the tag; a screen never builds the record by hand.

The duplicate check (PRD FR-10.7, E38): the form sends the typed names, phone AND tag to
`accounts_similar`, and the backend marks the account whose name, tag and phone all equal them
(`is_duplicate`). `SimilarAccounts` shows that row with a word chip and changes the title of the panel; the
frontend never decides it. It only warns, and the save is never blocked. A contact person of the firm
(`features/settings/components/FirmTab`) has the same spelling check as the firm's names.

The accounts list and the codes (plan 32, R1 to R6): the list shows only sellers, buyers and the
two پختہ کھاتہ accounts, with the filter All, Seller, Buyer, Pukhta (`AccountListKind`); the
chart still shows every account. A code has no fixed length: `0010`, `00110`, `0020`. The screens
read it as text and never count its digits, and the new-account form offers only Seller and Buyer.
Staff have a code of their own now (`0060`, `0061`).

The Index form from a picker (plan 32, R8): the "add a new …" button of an empty picker opens
`components/IndexAddDialog` above the screen. It is the same `AccountForm` or `MasterForm`, given
`embedded` (the type, the typed text and a way out), in a `FormDialog` that answers Esc through
the Esc order. The screen under it is never closed, so an entry in progress keeps every value.
`app/AddRecordHost` gives the dialog to the pickers (`shared/ui/add-record.ts`); the pickers inside
the dialog get none, so there is no second Index form on top of the first.
