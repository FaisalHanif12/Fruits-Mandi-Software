# assistant (plan 49, deliverable D6)

Ask the app: the question panel at the bottom of every screen and its Settings group (PRD 7.7,
FR-7.72 to FR-7.88, FR-5.5, the `F1` key of 7.3).

- `AssistantDock` is mounted once by `AppShell`, above the status bar, and never unmounts on a route
  change. `F1`, the status bar button (`AssistantButton`, with the day's count) or the close button
  open and close it. Two heights (`Ctrl+↑` and `Ctrl+↓` inside it), a slide of 150 ms, a focus trap
  while the focus is in it (`shared/keyboard/focusTrap.ts`), and `Esc` closes it and gives the focus
  back to where it came from. It draws over the lower part of the screen and never touches what is
  typed in the screen under it.
- Everything the panel shows is in `store.ts`, above the screens: the open state, the height, the box,
  the conversation. A language switch, a screen change or closing the panel loses nothing. The app
  locking or signing out clears it (FR-7.73).
- "New question" tells the backend first (`assistant_new_conversation`), then the panel forgets the
  conversation, so the next question goes out with `conversation_id: null`. Once a conversation has
  begun, the quick questions fold behind a button in both heights, so the answer has the room.
- The panel shows what the backend sends and nothing else (CLAUDE.md rule 1). `AnswerBlock` writes the
  parts: a `text` part as it is (never a number), a `figure` large through `<Num>` with ✓ when its two
  places agree and ⓘ for its working, a `name` through `displayName`. Tables use the report screens'
  own cell renderer (`features/reports/cells.ts`); the chart uses `shared/charts`.
- `ReplyBody` has one design for each outcome (answer, estimate, advice, which one, not found, need
  dates, need party, locked, outside the books, change refused, not understood, checks failed, records
  differ, limit reached, offline, error). The words of every state that is not an answer are in
  `i18n/<lang>/assistant.ftl`, each `# review: pending` until the Urdu reviewer reads it.
- `api.ts` is the only file that knows the command shapes and the `assistant://step` event. Tests mock
  it (`tests/helpers/assistant-shell.tsx`, with the backend's own fixtures in
  `tests/helpers/assistant-real.ts`).
- The key is only ever in the Settings paste field until it is sent with the Owner's password. It is
  not put into a store, the query cache, a log or a test.
- The buttons under an answer open a report or a view with the filters the model read it with
  (`addresses.ts`): the report address (`features/reports/search.ts`) and the analytics address
  (`features/analytics/search.ts`) learned those filters, and with none of them they read as before.
- The button of a refused change request opens the entry screen of the answer with the party chosen
  (`navigation.ts`, FR-7.79). The voucher, expense, lent, khata, capital, fruit and general screens
  read an optional `party` in their address (a whole picker value, `shared/accounts/address-party.ts`).
  It goes into the picker, every amount stays empty, nothing is saved, and with no `party` each screen
  opens as before.
- Development page: `#/dev/assistant` (see `tests/helpers/dev-bridge-assistant.ts` for its flags).
