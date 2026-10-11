# features/masters

انڈکس: the ten master lists of PRD 10.3 (plan 06). One generic screen, configured per master in
`configs.ts`: city, item, packing, adda, marka, store, narration, expense head, staff, bank account.

- `pages/MastersMenuPage`: the menu of the ten lists (`/index`).
- `pages/MasterListPage`: one list (`/index/<path>`): search (the backend search, both scripts),
  "show inactive", rows in `shared/grid/ListGrid`, four states.
- `pages/MasterFormPage` and `components/MasterForm`: add (`/index/<path>/new`) and change
  (`/index/<path>/<id>`). The code with the next one suggested, `NamePair` for the two names with
  the spelling check (PRD FR-7.66), the fields of that master, read-only facts from the backend.
- A master is never deleted. It is made inactive, after a confirm (PRD E21).
- `types.ts` is INTERIM until backend's generated bindings for the masters commands exist. Only
  `api.ts` knows the wire shape.
- To add a master: add its configuration, its Fluent messages (`masters.ftl`, ur and en) and its
  backend kind. Nothing else.
