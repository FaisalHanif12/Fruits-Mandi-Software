# app/layout

The app shell: AppShell, MainNav, ModuleMenu (the tabs of an area), TopBar (search, attention bell, language switch, user
menu) and StatusBar (Tech Stack 5.3, PRD 5).

- **The tabs of an area (plan 93, PRD FR-5.3 and FR-7.17, v1.56).** The second menu of بکری and روکڑ is not a side
  panel: it is a row of tabs in the title row of each screen, so the screen has the width the panel took and no
  extra row of height. `ModuleMenu.tsx` keeps the items in order (`BIKRI_MENU`, `ROKAR_MENU`) and builds the row with
  the one shared component `shared/ui/SectionTabs`; `AuthGate` gives the row for the open address to `AppShell`
  (`sectionTabs`), which hands it to the screen through `SectionTabsProvider`. A screen does not build tabs: it puts
  `TitleOrTabs` where its title is (`ListPage`, `EntryFrame`, the sale book and the voucher screen do), and outside a
  section it keeps its own title. In a section the open tab IS the title and the `<h1>` stays for screen readers. The
  row is always one line: tabs that do not fit go behind "More", the open one stays in the row. Keys: ← → between
  tabs in the reading direction, Enter opens, ↓ into the screen; only the open tab is a Tab stop. A screen that is
  a state of one entry (a saved بکری read only, an edit, a later sale) keeps its own title and has no tabs.

- `AttentionBell` (plan 19, PRD FR-7.44): the number the backend counts (`assist_attention`) and the
  list under it, one line for each kind, each opening the right screen; a new version of the app is
  one more line at the top, only a note. Four states: loading, an error with a retry, nothing to do,
  and the lines. The lines are `features/dashboard/components/AttentionLines`, the same as the
  dashboard's "آج کے کام".
- The user menu has "change password", "show the guide" (opens the first-run guide again, `app/guide`,
  PRD FR-7.12) and "lock the app". Log out is not there: it is the last item of the main menu, at the
  bottom under a thin line (PRD FR-5.2, FR-5.6, plan 75). It asks first when a screen has unsaved work:
  `AuthGate` calls `confirmLeaveAll()` (`app/leave-guards.ts`), and each `LeaveGuard` on screen that
  is dirty shows its own "Save changes?" box. The idle lock never asks and keeps the work (FR-8.4).
- Plan 75: the main menu starts with the app's mark (`shared/ui/BrandMark`, the same as on the sign-in
  panel); at the smallest window (`shortest:`, under 641 px high) the mark gives way and the items
  are 40 px, so the twelve items fit 1024 x 600. The area menus have an icon on each item and a white
  card for the current one. The bell and the user are quiet buttons with no box until the pointer is on
  them.
- `app/use-last-screen.ts` starts the app where it was left and keeps the screen that is open
  (PRD FR-7.45); `app/refresh-after-save.ts` reads the dashboard and the bell again after any save.
