# shared/navigation

The Back button and Esc (plan 78 F1, PRD FR-7.18). One place for both, so they always do the same.

- `page-trail.ts`: a short trail, in memory, of the pages the user opened one from another. Each stop is an
  address (path and search), so Back returns to the page in the state it was left (its month, its filters).
  Nothing is saved to disk. A way in from a menu starts the trail again (`promiseNavigation('fresh', path)`).
- `back-parents.ts`: the page each child page goes back to when the trail has nothing (opened by an address,
  after a restart, or from search). Pages reached from a menu have no parent.
- `use-page-trail.ts`: mounted once by the signed-in shell (`app/AuthGate.tsx`). Records the page that was left.
- `use-back.ts`: `useBack()` gives `{ available, go }` for the Back button of the top bar and for Esc.
- The button is `shared/ui/BackButton.tsx`. It stands at the start of the top bar (`app/layout/TopBar.tsx`),
  so no page has its own.

A page never writes its own Back button or navigates to its parent on Esc. A page that has an inner step
(a preview over a table) registers Esc at level `leave` with `active` true only while that step is open; with
the table showing, Esc is the shell's.

Example: the dashboard opens "Buyers' dues" (`/dues`). The trail is `['/']`. Back goes to `/`. If the user
opened `/dues` by its address, the trail is empty and Back goes to its parent, `/`.
