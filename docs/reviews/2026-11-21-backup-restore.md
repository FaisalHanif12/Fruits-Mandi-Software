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
