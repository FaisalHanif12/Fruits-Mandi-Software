# app/layout

The app shell: AppShell, MainNav, ModuleMenu, TopBar (search, attention bell, language switch, user
menu) and StatusBar (Tech Stack 5.3, PRD 5).

- `AttentionBell` (plan 19, PRD FR-7.44): the number the backend counts (`assist_attention`) and the
  list under it, one line for each kind, each opening the right screen; a new version of the app is
  one more line at the top, only a note. Four states: loading, an error with a retry, nothing to do,
  and the lines. The lines are `features/dashboard/components/AttentionLines`, the same as the
  dashboard's "آج کے کام".
- The user menu has "show the guide", which opens the first-run guide again (`app/guide`, PRD FR-7.12).
- `app/use-last-screen.ts` starts the app where it was left and keeps the screen that is open
  (PRD FR-7.45); `app/refresh-after-save.ts` reads the dashboard and the bell again after any save.
