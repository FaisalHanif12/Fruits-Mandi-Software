# partners (plan 11)

The partners and their shares (PRD 17.1 to 17.3): the overview at `/partners` and the setup at
`/partners/setup`. The same setup form is the fourth step of the first-run wizard.

- `PartnerSetupForm` (exported through `index.ts`, with `usePartnerSetupStore`) is the one form for
  both homes. The host gives it the words of its Save button, what comes after a save, and
  optional buttons at the start of the action row (the wizard's "do this later").
- The shares are text as typed (`"40"`, `"33.33"`). The running total and the problem of each row
  come from the backend (`partners_check_shares`); the screen never adds a percent
  (CLAUDE.md rule 1). Save stays off until the backend says the total is exactly 100.
- What is typed lives in `setup-store.ts`, above the screens, so a language switch, a dialog or a
  wizard step loses nothing (F39). `setup-logic.ts` holds the text side that tests run without the
  screen.
- Partners are added one at a time in a dialog (names through `NamePair`, opening capital as
  text), because the check and the save need a real partner id for every row. "Save and add
  another" (Ctrl+Enter) keeps the dialog open, so the wizard needs no more clicks.
- The wizard signs the Owner in right after the setup (the setup does not), because every
  partners command needs a signed-in Owner. If that fails, the wizard ends at the sign-in screen
  and the partners are set up later.
- `api.ts` is the only file that knows the command shapes. Tests mock `api.ts`
  (`tests/helpers/partners.ts` is the fake backend).
