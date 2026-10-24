# shared/screen-state (plan 19, PRD FR-7.45)

What a screen remembers for the Owner: the filters of a report, an analytics view or a list, and the
screen he left the app on. The backend keeps the JSON text a screen wrote and reads none of it
(`prefs_screen_get`, `prefs_screen_set`; the name is lower-case letters, digits and `_ - .`, at most 40).

- `useScreenMemory(screen)`: `ready` (the saved text arrived or could not be read), `failed`, `saved`
  (parsed JSON, or null) and `remember(value)`. A save waits 500 ms for the Owner to pause, is sent at
  once when the screen closes, and is not sent when the value is the one that was read (keys in any
  order). A failed read or save is never shown: the screen starts as new.
- `Remembered`: draws a screen only when its saved text is here, so it never flashes its first state
  and jumps. The screen reads `saved` once, when it opens, and owns its choices after that.
- `remember.ts`: small readers (`choiceOr`, `idOrNull`, `pickedOrNull`, `pickedText`...) so a screen
  keeps only what it can read back, and leaves out what it cannot (an old shape, a record that is gone).
- Nothing about money is kept. A screen keeps choices, never amounts.

Used by: every report (`report.<id>`), analytics (`analytics.filters`), today's بکری, باقی مال, the
پختہ کھاتہ register, the partner statement, the last screen (`app.last-screen`) and the first-run
guide's seen mark (`app.guide`). In tests the commands are answered by `tests/setup.ts`: nothing is
kept, and the guide counts as seen.
