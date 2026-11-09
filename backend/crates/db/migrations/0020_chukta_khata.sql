-- Migration 20 (plan 64, PRD v1.32 Section 13, FR-13.9, FR-10.4; Architecture 15.2, 15.4; D8 Feature 5,
-- decisions C1, C2, C9 and C11 of plan 60): چکتا کھاتہ takes the place of پختہ کھاتہ.
--
-- The one-time accounts are removed from the app. What stays is the buyer side: the system account
-- 0030, now چکتا کھاتہ, holds the lines of walk-in buyers who pay the whole bill on the spot. The people
-- of those lines are listed in a new table, `chukta_buyers`, so a buyer is found again by his name.
--
-- What this file does, in this order:
--   1. Renames every `pukhta_*` column to `chukta_*` (SQLite ALTER TABLE ... RENAME COLUMN, which also
--      rewrites the triggers and indexes that use the column, so `bikri_sales_snapshot_is_final`, the
--      balance and matching triggers and the person indexes keep working on the new names).
--      `accounts.is_pukhta_khata` becomes `is_chukta_khata`. `accounts.is_pukhta_party` becomes
--      `was_one_time_party`: a column of the old mark that no code reads again, so those accounts stay
--      as plain sellers or buyers (decision C9).
--      The seller-side `bikri_lots.chukta_*` columns stay in the table, empty on every new lot.
--   2. Drops the trigger and the partial index of the old party mark, and makes the trigger again under
--      a new name, so the old mark still cannot be changed.
--   3. Makes the account 0031 (the old shared seller account) inactive. A system account cannot be made
--      inactive (migration 3), so that one trigger is dropped for this statement and made again below,
--      exactly as migration 3 wrote it.
--   4. Retires every dead-amount mark that was made on a one-time person: nothing marks a person any more.
--   5. Creates `chukta_buyers`, with its indexes and triggers.
--   The Rust hook of this migration (`migrate.rs`, `chukta_hook`) then gives 0030 and 0031 their new
--   names with their search keys (only `core::text` builds those, Code Standards B26) and fills
--   `chukta_buyers` from the distinct people of the old lines on 0030.
--
-- No journal line, journal entry, amount, id or chain hash is touched: the hash covers values and
-- never a column name. Old open amounts on 0030 and 0031 stay in the books and in the trial balance;
-- they simply no longer show in any dues list. Migrations 1 to 19 are never edited, so the old word
-- stays in them.

-- 1. The columns.
ALTER TABLE accounts RENAME COLUMN is_pukhta_khata TO is_chukta_khata;
ALTER TABLE accounts RENAME COLUMN is_pukhta_party TO was_one_time_party;

ALTER TABLE bikri_lots RENAME COLUMN pukhta_name_ur TO chukta_name_ur;
ALTER TABLE bikri_lots RENAME COLUMN pukhta_name_en TO chukta_name_en;
ALTER TABLE bikri_lots RENAME COLUMN pukhta_phone TO chukta_phone;
ALTER TABLE bikri_lots RENAME COLUMN pukhta_person_key TO chukta_person_key;

ALTER TABLE bikri_sales RENAME COLUMN pukhta_name_ur TO chukta_name_ur;
ALTER TABLE bikri_sales RENAME COLUMN pukhta_name_en TO chukta_name_en;
ALTER TABLE bikri_sales RENAME COLUMN pukhta_phone TO chukta_phone;
ALTER TABLE bikri_sales RENAME COLUMN pukhta_person_key TO chukta_person_key;

ALTER TABLE analytics_facts RENAME COLUMN pukhta_person_key TO chukta_person_key;
ALTER TABLE vouchers RENAME COLUMN pukhta_person_key TO chukta_person_key;
ALTER TABLE fruit_takes RENAME COLUMN pukhta_person_key TO chukta_person_key;
ALTER TABLE journal_lines RENAME COLUMN pukhta_person_key TO chukta_person_key;
ALTER TABLE bill_allocations RENAME COLUMN pukhta_person_key TO chukta_person_key;
ALTER TABLE advance_allocations RENAME COLUMN pukhta_person_key TO chukta_person_key;
ALTER TABLE dead_marks RENAME COLUMN pukhta_person_key TO chukta_person_key;

-- 2. The old party mark is fixed for good and nothing lists by it any more.
DROP TRIGGER accounts_pukhta_party_fixed;
DROP INDEX idx_accounts_pukhta_party;
CREATE TRIGGER accounts_old_one_time_mark_fixed BEFORE UPDATE OF was_one_time_party ON accounts
WHEN NEW.was_one_time_party IS NOT OLD.was_one_time_party
BEGIN SELECT RAISE(ABORT, 'ERR_SYSTEM_ACCOUNT'); END;

-- 3. The old shared seller account is closed. Nothing posts to it any more. Its balance, if any, stays.
DROP TRIGGER accounts_system_stays_active;
UPDATE accounts SET is_active = 0 WHERE code = '0031' AND is_chukta_khata = 1;
CREATE TRIGGER accounts_system_stays_active BEFORE UPDATE OF is_active ON accounts
WHEN OLD.is_system = 1 AND OLD.is_active = 1 AND NEW.is_active = 0
BEGIN SELECT RAISE(ABORT, 'ERR_SYSTEM_ACCOUNT'); END;

-- 4. A dead-amount mark on a one-time person ends here. The row stays, as history (a mark is never
-- deleted), and is shown as undone with the reason below. The business day is today.
UPDATE dead_marks
SET is_active = 0,
    undone_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    undo_reason = 'one-time accounts removed',
    undone_after_entry_id = (SELECT COALESCE(MAX(id), 0) FROM journal_entries),
    undone_date = date('now')
WHERE is_active = 1 AND chukta_person_key IS NOT NULL;

-- 5. The list of چکتا buyers (Architecture 15.4). One row for each person. `person_key` is the
-- normalised Urdu name and the phone digits (`core::text::person_key`), so one person is one row. A row
-- is never deleted; `is_active` hides it from the picker.
CREATE TABLE chukta_buyers (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    person_key TEXT NOT NULL CHECK (length(trim(person_key)) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    phone TEXT CHECK (phone IS NULL OR length(trim(phone)) > 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

CREATE UNIQUE INDEX idx_chukta_buyers_key ON chukta_buyers (person_key);
CREATE INDEX idx_chukta_buyers_search_ur ON chukta_buyers (search_ur);
CREATE INDEX idx_chukta_buyers_search_en ON chukta_buyers (search_en);
CREATE INDEX idx_chukta_buyers_search_phonetic ON chukta_buyers (search_phonetic);

CREATE TRIGGER chukta_buyers_no_delete BEFORE DELETE ON chukta_buyers
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- The key ties a person's lines together, so it never changes once a row exists.
CREATE TRIGGER chukta_buyers_key_fixed BEFORE UPDATE OF person_key ON chukta_buyers
WHEN NEW.person_key IS NOT OLD.person_key
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
