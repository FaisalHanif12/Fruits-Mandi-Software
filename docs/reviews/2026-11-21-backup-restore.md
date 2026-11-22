# Backup and restore check, 21-11-2026

Verdict: PASS WITH MINOR (robustness, assistant-eval). No data loss, no half save, no bug found.

| Check | Result |
|---|---|
| Existing backup suite (cargo test -p mandi-backup) | 125 passed, 0 failed, 5 ignored with reasons |
| First-screen restore (USB folder, file, cloud stand-in), robustness_backup_first_screen.rs | 25 of 25 |
| Copy of Umar's dev books: local and encrypted USB backup, restore on a new computer and over the same computer, wrong key, damaged file, safety copy, chains verify | 24 of 25 in run 1; the one fail was a test artefact, fixed; rerun waits for a Keychain click |
| Frontend first-run wizard (both choices, cloud fields, check page, failure returns) | 149 of 149 |
| Older-version copy runs the migrations | pass (existing tests) |
| Real Cloudflare upload | skipped: no real keys; proven on the object store stand-in only |

Minor: R4-9 (known): partner share rows are in no hash chain or restore check; needs the backup key to exploit. Real R2 network behaviour is not proven until real keys are set.

## Dev-copy check with the real Keychain keys, 22-11-2026

`MANDI_BACKUP_DEV_KEYCHAIN=1 cargo test -p mandi-backup --test robustness_backup_dev_copy -- --nocapture` on a `.backup` copy of the dev database (52 tables, 1,123 rows, 20 entries, 401 audit rows, trial balance 4,192,650 = 4,192,650): **PASS, every row A0 to F2.** Both hash chains verify with the app's real key; local and USB copies made and checked; the USB copy is encrypted with no plain marker; a wrong key and a damaged file are refused with nothing changed; restore on a new computer and over the same computer equal the copy in every table, balance, chain and setting; the chain key travels with the copy; the preview names the work that would be lost; a safety copy is taken first; the test restore reports good and damaged copies correctly. The real Cloudflare upload is still not proven (no keys).
