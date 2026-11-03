# Plan 56: restore on a new computer, from the first screen

Version 1.0, 04-11-2026. Lead: tech-lead. Asked by Umar on 04-11-2026. PRD v1.24 FR-20.11, NFR v1.14 NFR-INS-2 and NFR-INS-6, NFR-REL-4. Branch `feature/assistant`, folder `Fruit Mandi Parallel/d5`. It runs in parallel with plans 54 and 55, with its own teammates, and touches no assistant file.

## 1. Goal

A shop has two years of books. The laptop is lost. On a new laptop the Owner installs the app, presses پچھلا ڈیٹا واپس لائیں on the first screen, gives the cloud details and his printed backup key, and gets every book back exactly as in the last copy. He then signs in with his old password and carries on.

## 2. What exists today (checked 04-11-2026)

- **Restore exists only inside a signed-in app** (`backup_restore_preview`, `backup_restore`, `backup_test_restore`). It asks for the printed key when this computer does not have it.
- **The first-run wizard** (`setup_first_run`, `features/auth/pages/WizardPage.tsx`) has four steps for a new shop and **no restore choice**.
- **Keys kept in the Keychain or Credential Manager, never in the data file:** the backup key, the cloud keys, the OpenRouter key, and the journal and audit chain key (NFR-SEC-6, `backup/src/secrets.rs`).
  - On a new computer the chain key is not there.
  - So the tamper check must be planned for (Section 4), not discovered later.

## 3. The flow

1. **First screen of a new install, before anything is created:** two big buttons, نئی دکان شروع کریں and پچھلا ڈیٹا واپس لائیں, plus the اردو | English switch. "New shop" opens today's wizard unchanged.
2. **Where is the copy?** The choices are the cloud, or a USB drive or file.
   - **Cloud.** The Owner types the cloud address, the bucket, the folder (default `mandi`) and the two cloud keys. A "connect" step lists the copies, newest first, with date and size.
     - Wrong keys, a wrong bucket or no internet get a plain message that names the field.
   - **USB or file.** He picks the drive or a `.mbk` file.
3. **The copy and the key.**
   - The newest copy is chosen; he can pick another.
   - He types the backup key from the printed sheet, with the dashes. A wrong key says so and lets him try again.
4. **Check before restore.** The copy is decrypted into a temporary place and checked (NFR-BAK-5: it opens, its checksum, its integrity, its record counts and the books balance). Then the screen shows its date and totals: accounts, بکری entries, vouchers, posted entries, the newest entry date, total debit and credit.
5. **Restore.**
   - The data file is put in place and migrated if the copy is from an older version (NFR-SCL-8).
   - The backup key is saved in the Keychain or Credential Manager.
   - The cloud settings and keys are saved, so the daily copy goes on to the same place.
   - The chain key is handled as decided in Section 4.
   - The search index is rebuilt.
6. **Sign in** with the old user name and password, with the recovery code as the fallback (FR-8.5).
   - Then a short note: "OpenRouter key: please enter it again in Settings" (only if the assistant was on), and "make a new backup now" (a button).
7. **Failure at any point** leaves the new computer as a fresh install, with no data file and no keys half written. The first screen shows again with the reason.
8. **Speed.** Restoring two years of data takes under 5 minutes (NFR-BAK-12), and the whole flow under 30 minutes (NFR-REL-4).

## 4. The chain key (backend3 decides with evidence, then the lead approves)

The journal and audit chains are keyed hashes (NFR-SEC-6). On a new computer:
- **Option A.** The chain key travels inside the encrypted copy, locked by the backup key like the data. The restore puts it back in the Keychain or Credential Manager, and the chains verify as before.
  - It is never in plain form and never in the data file.
- **Option B.** Something else that keeps the append-only rules (B17) intact: rows are never rewritten.

backend3 reads what the backup already carries and proposes A or B with the trade-off, before building. The money agent is not needed: no money is calculated. The security rule stands: no key is ever in plain form, on disk or in a log.

## 5. Contract (backend3 proposes it, frontend3 builds against it, the lead records it here)

The commands run **before** sign-in, because no Owner exists yet on the new computer. They are allowed only while no data file exists (the first-run state), like `setup_first_run` (NFR-SEC-4 exceptions):
- `restore_first_run_list(source)` gives the copies.
- `restore_first_run_preview(source, copy, backup_key)` gives the checks and totals.
- `restore_first_run(source, copy, backup_key)` gives done, then sign in.
- `source` is either `{cloud: {endpoint, bucket, region, prefix, key_id, secret}}` or `{file: path}`.
- Every error has a code and a field.
- Progress goes as events, the same as the existing restore.

## 6. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| R1 | Read the backup, restore and secrets code. Propose the contract (Section 5) and the chain-key option (Section 4) to the lead. | backend3 | |
| R2 | Build the commands. Tests: back up a shop with the worked day to a local S3 stand-in (or the object store's local or memory backend) and to a file. Then, as a new computer (empty data folder, empty test key store), restore it, and compare every table's row count and checksum, the trial balance, both chains, the settings, the prefs and the learned words. Sign in with the old password and with the recovery code. Make a new backup to the same place. Also: a wrong key, a damaged copy, a wrong bucket, no internet, a half-finished restore, and a copy from an older version. | backend3 | R1 approved |
| R3 | The first screen with two choices, and the restore pages (where, cloud fields or file, the copy list, the key, the checks and totals, progress, done, sign in). Both languages, keyboard, the four states, 1024x600 and 1366x768. | frontend3 | R1 contract |
| R4 | Attack tests: a first-run restore command must be refused once a shop exists, and no key in plain form on disk or in a log. A 2-year data copy for the 5-minute target. | robustness | R2 |
| R5 | The lead's own end-to-end check on the Mac. Back up a test shop to a test folder in Umar's bucket, using his own keys only through the app. In a fresh app profile, restore from the first screen, and check that the dashboard and the reports match. | lead | R2, R3 |

## 7. Out of scope
- Moving data between two computers that both stay in use (NFR-SCL-9).
- Restoring part of the books.

## 8. Lead decisions on backend3's R1 proposal (04-11-2026)

- **Chain key: Option A.**
  - The key travels inside the encrypted `.mbk` bundle (`Embedded.chain_key`), locked by the backup key. It is never in the `.json` sidecar or a plain local copy.
  - Restore verifies both chains on the temporary copy with that key before it installs anything.
  - The same applies to the signed-in restore.
  - NFR-SEC-6 is updated (NFR v1.14).
- **Q1:** backend3 may change `db/src/audit.rs` (key to and from hex, install, set) and the KeyStore impls. backend is told.
- **Q2:** the command names are `backup_first_run_list`, `backup_first_run_preview` and `backup_first_run_restore` (Tech Stack 5.2), in a new file `commands/backup_first_run.rs`.
- **Q3:** a copy with no chain key, and no matching key on this computer, is refused with ERR_BACKUP_CHAIN_KEY_MISSING. No copy is in the field yet.
- **Q4:** the key error field is `backup_key` in these commands.
- **Q5:** a false check (chains, balance, integrity) refuses, and the Owner picks an older copy. There is no override.
- **First-run state:** no Owner exists (`users` count is 0), as for `setup_first_run`. NFR-SEC-4 now lists restore on a new computer as allowed before sign-in, only in that state.
- **Settings from the old computer:** a `local_dir` that does not exist goes back to the default. `usb_folder` and `cloud_enabled` are kept.
- **OpenRouter key:** after sign-in, the frontend shows "enter it again" when the assistant is on and no key is stored.
- **Progress:** the new event `backup-restore-progress` with its phases. The preview and the restore each download the copy; there is no cache that holds secrets.
- **Copies made before this change** carry no chain key. Umar's first cloud copies are among them, so after this is built, he makes one new copy.
