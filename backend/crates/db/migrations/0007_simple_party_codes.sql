-- Migration 7 (plan 32, PRD 1.8): simple codes.
--
-- A seller, a buyer, an item, a packing and a staff member get a code that is a 3-digit type number
-- plus a running number from 0 with no padding (R1, R2):
--
--   seller 001 -> 0010, 0011 ... 0019, 00110 ...   buyer 002 -> 0020, 0021 ...
--   پختہ کھاتہ 003 -> 0030 (buyer account), 0031 (seller account)
--   item 004 -> 0040 ...   packing 005 -> 0050 ...   staff 006 -> 0060 ...
--
-- What this file does, in this order:
--   1. Staff get a `code` column (they had none), with a unique index.
--   2. Existing records are renumbered in the order they have now (by their old code, then id):
--      the sellers and buyers that are not system accounts, the two پختہ کھاتہ system accounts, the
--      items, the packings and the staff. Every other code is untouched (R4): cash, bank, payables,
--      income, expense, capital, drawings, 0604, نقدی منڈی, اپنا مال, money-lent people, khatas,
--      gardens, cities, addas, marks, stores, narrations and expense heads.
--   3. `accounts.serial` of a renumbered account becomes its running number. The Rust hook of this
--      migration (see `migrate.rs`, `codes_hook`) then lifts the upper limit of that column, so the
--      running number has no upper limit (R2). SQLite cannot change a CHECK with ALTER TABLE; the hook
--      uses the same documented way as migration 6.
--
-- Nothing else changes. No journal line, no amount, no id, no balance and no chain hash is touched:
-- the journal and the audit log hold account ids and never codes (the audit rows of old edits keep
-- the code text they were written with, as history). A record keeps its id, so every picker,
-- recent list and rule that points at it still does.
--
-- Items and packings are renumbered in two steps (a temporary code first), because their code is
-- unique and an old code typed by hand could equal a new one half way through.

-- The new numbers are worked out first and kept in a temporary table, because an UPDATE that reads
-- the table it is changing could see rows it has already changed.
-- table_kind: seller, buyer, item, packing, staff. n: the running number (0 for the first).
CREATE TEMP TABLE new_codes (kind TEXT NOT NULL, id INTEGER NOT NULL, n INTEGER NOT NULL);

-- Sellers: control 03011, not a system account (this leaves out پختہ کھاتہ and اپنا مال).
INSERT INTO new_codes
SELECT 'seller', a.id, ROW_NUMBER() OVER (ORDER BY a.serial, a.id) - 1
FROM accounts a JOIN account_controls c ON c.id = a.control_id
WHERE c.code = '03011' AND a.is_system = 0;

-- Buyers: control 0304, not a system account (this leaves out نقدی منڈی and پختہ کھاتہ).
INSERT INTO new_codes
SELECT 'buyer', a.id, ROW_NUMBER() OVER (ORDER BY a.serial, a.id) - 1
FROM accounts a JOIN account_controls c ON c.id = a.control_id
WHERE c.code = '0304' AND a.is_system = 0;

-- Items and packings keep the order of their list (by code as a number, then id). Staff had no code:
-- they follow the order they were added.
INSERT INTO new_codes
SELECT 'item', id, ROW_NUMBER() OVER (ORDER BY CAST(code AS INTEGER), code, id) - 1 FROM items;
INSERT INTO new_codes
SELECT 'packing', id, ROW_NUMBER() OVER (ORDER BY CAST(code AS INTEGER), code, id) - 1 FROM packings;
INSERT INTO new_codes
SELECT 'staff', id, ROW_NUMBER() OVER (ORDER BY id) - 1 FROM staff;

-- 1. Staff code.
ALTER TABLE staff ADD COLUMN code TEXT;

-- 2a. The codes of system accounts cannot change (PRD FR-10.4). The two پختہ کھاتہ accounts are
-- system accounts, so the trigger is dropped for this migration and made again below, exactly as
-- migration 1 made it.
DROP TRIGGER accounts_system_fixed;

-- No new account code can equal an old one: every old account code starts with 01 to 06.
UPDATE accounts
SET code = '001' || (SELECT n FROM new_codes WHERE kind = 'seller' AND id = accounts.id),
    serial = (SELECT n FROM new_codes WHERE kind = 'seller' AND id = accounts.id)
WHERE id IN (SELECT id FROM new_codes WHERE kind = 'seller');

UPDATE accounts
SET code = '002' || (SELECT n FROM new_codes WHERE kind = 'buyer' AND id = accounts.id),
    serial = (SELECT n FROM new_codes WHERE kind = 'buyer' AND id = accounts.id)
WHERE id IN (SELECT id FROM new_codes WHERE kind = 'buyer');

-- The two پختہ کھاتہ accounts: buyers' 0030 (running number 0), sellers' 0031 (running number 1).
UPDATE accounts SET code = '0030', serial = 0 WHERE code = '03049999' AND is_pukhta_khata = 1;
UPDATE accounts SET code = '0031', serial = 1 WHERE code = '030119999' AND is_pukhta_khata = 1;

CREATE TRIGGER accounts_system_fixed BEFORE UPDATE ON accounts
WHEN OLD.is_system = 1 AND (NEW.code IS NOT OLD.code OR NEW.is_system IS NOT 1)
BEGIN SELECT RAISE(ABORT, 'ERR_SYSTEM_ACCOUNT'); END;

-- 2b. Items and packings in two steps (a temporary code first, see the header), then staff.
UPDATE items SET code = 'tmp-' || id;
UPDATE items
SET code = '004' || (SELECT n FROM new_codes WHERE kind = 'item' AND id = items.id);

UPDATE packings SET code = 'tmp-' || id;
UPDATE packings
SET code = '005' || (SELECT n FROM new_codes WHERE kind = 'packing' AND id = packings.id);

UPDATE staff
SET code = '006' || (SELECT n FROM new_codes WHERE kind = 'staff' AND id = staff.id);

CREATE UNIQUE INDEX idx_staff_code ON staff (code);

DROP TABLE new_codes;
