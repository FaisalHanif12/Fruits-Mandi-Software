# backup (plan 18)

The بیک اپ screen (PRD Section 20, FR-20.8, FR-20.9, NFR-BAK-1 to NFR-BAK-15), route `/backup`.

- `pages/BackupPage.tsx`: the state of the backups, "back up now" (and what each place did), the
  backup key, the list of copies, restore, test restore and the settings. Four states: loading, an
  error with a retry, and the screen (the list has its own empty and error states).
- Restore (`RestoreDialog`): the copy's date, its books next to the books now, what would be lost,
  then a confirm box that starts on Back. A locked copy on a computer with no key asks for the
  printed key first. After a restore the backend signs the Owner out and the sign-in screen opens.
- Test restore (`TestRestoreDialog`): restores into a separate place, shows each check and the books
  next to the live books.
- The backup key (`BackupKeyCard`): shown once, printed (F9; the sheet is `[data-key-sheet]`, drawn
  on paper only, `backup.css`), typed back from the paper. It is the fourth step of the first-run
  wizard (after the recovery code, before the partners) and sits on the backup screen when there
  is no key or it is not confirmed. While it is open it lives in `key-store.ts`.
- `components/BackupStatusItem.tsx`: the last good cloud backup in the status bar, red over 24 hours
  (NFR-BAK-10). `components/BackupExitOverlay.tsx`: the copy at exit (the event `backup-exit`).
- `api.ts` is the only file that knows the wire shapes. The state is read about once a minute, every
  two seconds while a copy or a restore runs.
- Nothing here works out money, a time or a size's meaning: every figure is the backend's.

Look at it without the app: `pnpm --filter frontend exec vite --port <free port>` and open
`/#/dev/backup` (flags: `?new`, `?old`, `?key`, `?cloud`, `?empty`, `?many`, `?bad`, `?down`, `?lost`).
