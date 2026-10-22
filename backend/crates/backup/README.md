# mandi-backup

Local, USB and cloud backup, restore and verification (PRD 20, NFR 4, Architecture 14, plan 18).

- May depend on: `mandi-db` and `mandi-core`. The tests also use `mandi-services` (a dev-dependency) to build the PRD worked day.
- `BackupService` is the one object: `run` (copies), `tick` (what is due now), `retry_queue`, `restore_preview`, `restore`, `test_restore`, `status`, `list`, `key_setup`, `key_confirm`, `settings_set`.
- A copy is one snapshot (SQLite online backup, one step) checked with the full integrity check, counts, books and a blake3 checksum. Local copies are plain `.db` files. USB and cloud copies are `.mbk` bundles: zstd, then age with a passphrase (the backup key, 24 symbols, shown once).
- Every file is written as `.partial` and renamed when whole. A cloud copy goes into the outbox first and leaves it only when the upload is checked by downloading it again.
- Secrets (the key, the cloud keys) live in the Credential Manager (`secrets`), never in the data file.
- The cloud is a trait (`cloud::CloudStore`): `ObjectStoreCloud` over `object_store` (S3, Cloudflare R2) and `LocalFolderCloud`, a folder used by tests until D5-Q2 is answered.
- The readable daily pack (`daily_pack`): the روزنامچہ PDF comes from a `PackSource` the app gives once the report engine exists; the spreadsheets wait for D5-Q1 (Excel). Until then `PendingPack`.
- Tests: `tests/round_trip.rs` is E2E-18; `tests/faults.rs` holds the fault and schedule tests; `tests/fixtures/` holds the request/response pairs of the `backup_*` commands (written and checked by `backend/app/tests/backup_commands.rs`).
