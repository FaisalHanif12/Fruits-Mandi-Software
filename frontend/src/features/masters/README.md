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
