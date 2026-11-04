# restore: bring back my data on a new computer

Plan 56, PRD FR-20.11, NFR-INS-2, NFR-INS-6, NFR-REL-4.

What it is: the very first screen of a new install, and the pages that restore a backup copy on a
computer that has no data file yet.

## The flow

1. `StartPage`: two big choices, new shop (the existing wizard) and bring back my data.
2. `RestorePage` with four steps:
   - `WhereStep`: the cloud (the same fields as Settings, Backup), a USB drive or folder, or one file.
   - `CopyStep`: the copies, newest first and already chosen, and the backup key from the printed sheet.
   - `CheckStep`: the date, the checks and the books of the copy. Nothing is on this computer yet.
   - `RunStep`: progress, then done and "go to sign in".
3. The sign-in screen as it always was. Its recovery code is the way in if the password is forgotten.
4. `AfterRestoreCard`: once, after the first sign-in: make a new backup, retype the OpenRouter key if
   the assistant was on, type the cloud keys again if the copy came from a file or folder.

## Rules

- Everything typed lives in `store.ts`, above the pages, so a language switch loses nothing. The three
  keys (cloud access key, cloud secret, backup key) are masked when typed and dropped from the store
  the moment the restore ends, fails or is left. The two cloud keys can never be shown. The backup
  key has an eye (off by default), because the Owner copies 24 characters from paper.
- The actions are in `flow.ts`. One request at a time. A refusal is shown next to its field (the field
  names are in `field-paths.ts`). Only a failure of the restore itself goes back to the first screen.
  When he edits a field, the message about that field goes at once (it was about the old text); the
  messages of the other fields stay. A cloud address the backend refuses says why, with its `reason`
  value (`https`: it must start with https:// and have a host; `path`: write only the address, the
  bucket name has its own field). The words are `err-out-of-range` in `errors.ftl`, shared with Settings.
- `api.ts` wraps the three backend commands `backup_first_run_list`, `backup_first_run_preview` and
  `backup_first_run_restore`. Their types are the generated bindings.
- The pages work out no money. The totals are the backend's, drawn by the backup module's
  `TotalsTable`.

## Looking at it without a backend

Run the Vite dev server and open `/dev/restore` (see `src/app/dev/RestoreFirstRunPage.tsx` and
`tests/helpers/dev-bridge-restore.ts` for the flags, for example `?at=copy&many`).
