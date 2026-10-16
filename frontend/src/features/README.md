# features

One folder per business module, named as in Tech Stack 5.2 (`bikri`, `vouchers`, `masters` ...).
Every module has the same inside: `pages/`, `components/`, `hooks/`, `api.ts`, `schemas.ts`,
`routes.tsx`, `index.ts`.

A module imports only from `shared/` and from another module's `index.ts`, never from its inner
files (Tech Stack 5.3). The first module arrives in plan 04.
