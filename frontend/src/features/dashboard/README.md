# dashboard (plan 19, PRD Section 9)

The screen after sign-in, and the attention list that the bell and the dashboard share.

## What is here

- `pages/DashboardPage.tsx`: greeting, a note when a new version is ready, quick actions, the tiles,
  the partner strip, today's بکری, the oldest buyer dues, today's tasks, the two mini charts.
- `components/`: one file for each part (`DashboardTiles`, `StatusTiles`, `PartnerStrip`,
  `TodayBikriList`, `OldestDues`, `QuickActions`, `DashboardCharts`, `AttentionLines`, `KindIcon`,
  `UpdateNote`). Every part opens the screen behind it (FR-9.1).
- `api.ts`: `dashboard_get` and `assist_attention`, both read only. `hooks.ts`: the two queries.
- `destinations.ts`: where a tile, a line or a name leads, as plain data (`Destination`), and the one
  hook that opens it (`useOpenDestination`). Tests say where a click goes by comparing destinations.
- `attention.ts`: the Fluent message of each kind (`attention-<kind>`, and `attention-<kind>-aged`
  when the backend also names an age). `charts.ts`: what the two charts draw.

## Rules the screen keeps

- Every figure is the backend's. The dashboard adds up nothing: the tiles, the lists and the charts
  are written with `Num` and `formatPaisa`. Heights in a chart are numbers for the drawing only.
- One call (`dashboard_get`) fills everything but the greeting and the two status tiles. When it
  fails the page shows one error with a retry; the greeting, the quick actions and the backup and
  books check tiles (they ask for themselves) are still there.
- A tile or a figure that is below zero, or past its date, also says so in words (NFR-USE-12).
- The dashboard and the bell are read again after every save: `app/refresh-after-save.ts` watches
  the mutation cache (300 ms after the last save), and the bell also asks once a minute (NFR-SPD-22).
- The gardens tile (PRD 9.1, FR-12.37) is in `DashboardTiles`: the running balance of the open gardens
  and how many are open, from the same `dashboard_get` call as the other tiles (`gardens_open`,
  `gardens_balance_paisa`). It opens `/gardens`. Under the partners, `components/OpenGardens.tsx` lists
  each open garden (name, running balance, rent due) from `Dashboard.gardens`; a name opens that
  garden. With no open garden the tile says so and the section is not drawn.
- The new-version note comes from `useStartupUpdateCheck` (settings), asked once when the app starts.
  It is only a note, here and in the bell; installing stays in Settings.

## Tests and screenshots

`dashboard-screen.test.tsx`, `dashboard-bell.test.tsx`, `dashboard-words.test.ts`,
`dashboard-charts.test.ts`, `dashboard-api-contract.test.ts`; Windows spec `48-dashboard.e2e.mjs`;
screenshot snippet `tests/e2e/shots-plan26.playwright.txt` on the development page `/dev/dashboard`
(flags in `tests/helpers/dev-bridge-dashboard.ts`).
