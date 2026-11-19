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
- Restore on a new computer (PRD FR-20.11, plan 56): `first_run_list`, `first_run_preview`, `first_run_restore` (modules `first_run`, `first_run_source`, `first_run_install`, types in `first_run_api`). They run before sign-in and only while no Owner exists (`ERR_ALREADY_SET_UP`). Nothing is kept between calls. A copy is fetched, unlocked, checked, upgraded and its two hash chains verified in a scratch folder; only then are the keys saved, the data file replaced in one step and the settings adjusted. A failure undoes everything: the computer stays a fresh install.
- The chain key (`chain`): the key of the journal and audit hash chains (NFR-SEC-6) travels INSIDE the encrypted bundle (`Embedded.chain_key`), never in the `.json` next to a copy and never in a plain local copy. A restore verifies the chains with it and saves it on the computer. A copy without it is accepted only where the computer's own key verifies the chains; otherwise `ERR_BACKUP_CHAIN_KEY_MISSING`. The signed-in restore does the same.
- Tests for plan 56: `tests/restore_new_computer.rs` (the flows), `tests/restore_new_computer_faults.rs` (the failures, and the scans that no key is in a plain file or a log), helpers in `tests/new_computer/`; the IPC test and the `backup_first_run_*` fixtures are written by `backend/app/tests/backup_first_run_commands.rs`.
