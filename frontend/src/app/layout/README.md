# app/layout

The app shell: AppShell, MainNav, ModuleMenu, TopBar (search, attention bell, language switch, user
menu) and StatusBar (Tech Stack 5.3, PRD 5).

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
