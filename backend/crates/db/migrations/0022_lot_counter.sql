-- Migration 22 (plan 71, PRD v1.34 FR-12.45, Architecture 15.6 v1.22; Umar 12-11-2026, decisions L1 to
-- L6): lot names. Every lot has a name made of its seller, its item and a counter: احسان، آم-1, then
-- احسان، آم-2 for his next mango lot. The counter belongs to one seller and one item and runs for the
-- seller's whole life. It never resets and a number is never reused, not even after a cancel or after
-- an edit moves a lot to another seller or item (L2, L5).
--
-- What this file does, in this order:
--   1. Adds `bikri_lots.lot_counter` (INTEGER NOT NULL, default 0 only so that SQLite accepts the
--      ALTER; no saved lot keeps 0).
--   2. Numbers every old lot (L6): per seller and item, in order of the day of its first posting (the
--      date of its بکری), then its row id. The first lot is 1.
--   3. Makes the table `lot_counters`: for each seller and item, the biggest number ever given. It
--      only goes up. It is what "never reused" rests on: a lot that an edit moves away from a seller
--      and item leaves its number behind, and the biggest number among the lots still there would
--      forget it.
--   4. A unique index: no two lots of one seller and item share a number.
--   5. Triggers (defence in depth, Code Standards 4.1 layer 4):
--        - `lot_counters` is never deleted from, its pair never changes and its number never goes down;
--        - a lot inserted with a counter that is not above the biggest ever given is refused;
--        - a lot inserted with counter 0 (an insert that forgot it) gets the next number by itself;
--        - the counter of a lot is final, except that an edit which moves the lot to another seller or
--          item gives it a number above every number ever given to the new seller and item (L5); a
--          counter may not change while the seller and item stay;
--        - `lot_counters` follows every new number.
--
-- No amount, journal entry, journal line, allocation, partner row, id or chain hash is touched: the
-- chain covers entries and lines, never a column of `bikri_lots`. `version` and `updated_at` of the old
-- lots are not changed (this is the meaning of an old row, not an edit by a person).

ALTER TABLE bikri_lots ADD COLUMN lot_counter INTEGER NOT NULL DEFAULT 0 CHECK (lot_counter >= 0);

-- L6. The correlated count is the position of the lot among the lots of its seller and item, ordered
-- by (day of its بکری, row id).
UPDATE bikri_lots SET lot_counter = (
    SELECT COUNT(*)
    FROM bikri_lots m
    JOIN bikri_entries me ON me.id = m.bikri_id
    JOIN bikri_entries own ON own.id = bikri_lots.bikri_id
    WHERE m.seller_account_id = bikri_lots.seller_account_id
      AND m.item_id = bikri_lots.item_id
      AND (me.entry_date < own.entry_date
           OR (me.entry_date = own.entry_date AND m.id <= bikri_lots.id))
);

CREATE TABLE lot_counters (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    seller_account_id INTEGER NOT NULL REFERENCES accounts(id),
    item_id INTEGER NOT NULL REFERENCES items(id),
    last_counter INTEGER NOT NULL CHECK (last_counter > 0),
    UNIQUE (seller_account_id, item_id)
) STRICT;

-- The seller is the first column of the unique pair; the item has its own index (Code Standards B18).
CREATE INDEX idx_lot_counters_item ON lot_counters (item_id);

INSERT INTO lot_counters (seller_account_id, item_id, last_counter)
SELECT seller_account_id, item_id, MAX(lot_counter)
FROM bikri_lots
GROUP BY seller_account_id, item_id;

CREATE UNIQUE INDEX idx_bikri_lots_counter ON bikri_lots (seller_account_id, item_id, lot_counter);

-- lot_counters: append-only in the sense that matters. A row is never deleted, its pair is final and
-- its number never goes down.
CREATE TRIGGER lot_counters_no_delete BEFORE DELETE ON lot_counters
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER lot_counters_pair_is_final BEFORE UPDATE OF seller_account_id, item_id ON lot_counters
WHEN NEW.seller_account_id IS NOT OLD.seller_account_id OR NEW.item_id IS NOT OLD.item_id
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER lot_counters_never_goes_back BEFORE UPDATE OF last_counter ON lot_counters
WHEN NEW.last_counter < OLD.last_counter
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- A new lot must carry a number above every number ever given to its seller and item (L2).
CREATE TRIGGER bikri_lots_counter_is_new BEFORE INSERT ON bikri_lots
WHEN NEW.lot_counter > 0
 AND EXISTS (SELECT 1 FROM lot_counters c
             WHERE c.seller_account_id = NEW.seller_account_id AND c.item_id = NEW.item_id
               AND c.last_counter >= NEW.lot_counter)
BEGIN SELECT RAISE(ABORT, 'ERR_LOT_COUNTER_USED'); END;

-- A new lot with a number follows it into lot_counters.
CREATE TRIGGER bikri_lots_counter_follows AFTER INSERT ON bikri_lots
WHEN NEW.lot_counter > 0
BEGIN
    INSERT INTO lot_counters (seller_account_id, item_id, last_counter)
    VALUES (NEW.seller_account_id, NEW.item_id, NEW.lot_counter)
    ON CONFLICT (seller_account_id, item_id) DO UPDATE
        SET last_counter = MAX(last_counter, excluded.last_counter),
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
            version = version + 1;
END;

-- A lot inserted with counter 0 (the insert forgot the number) takes the next number by itself. The
-- app never relies on this: it writes the number the preview showed (PRD FR-12.45).
CREATE TRIGGER bikri_lots_counter_given AFTER INSERT ON bikri_lots
WHEN NEW.lot_counter = 0
BEGIN
    INSERT INTO lot_counters (seller_account_id, item_id, last_counter)
    VALUES (NEW.seller_account_id, NEW.item_id, 1)
    ON CONFLICT (seller_account_id, item_id) DO UPDATE
        SET last_counter = last_counter + 1,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
            version = version + 1;
    UPDATE bikri_lots
       SET lot_counter = (SELECT c.last_counter FROM lot_counters c
                          WHERE c.seller_account_id = NEW.seller_account_id AND c.item_id = NEW.item_id)
     WHERE id = NEW.id;
END;

-- The counter of a saved lot is final. It changes only when an edit moves the lot to another seller or
-- item (L5): the counter may change then, and only then.
CREATE TRIGGER bikri_lots_counter_is_final BEFORE UPDATE OF lot_counter ON bikri_lots
WHEN OLD.lot_counter > 0
 AND NEW.lot_counter IS NOT OLD.lot_counter
 AND NEW.seller_account_id IS OLD.seller_account_id
 AND NEW.item_id IS OLD.item_id
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- A lot that is moved to another seller or item takes a number above every number ever given to the new
-- seller and item (L2, L5). The number may be the same figure as before: 1 of one seller and item is
-- not 1 of another. The old seller's and item's number is not given back, because `lot_counters` of the
-- old pair only goes up.
CREATE TRIGGER bikri_lots_moved_takes_a_new_number BEFORE UPDATE OF seller_account_id, item_id, lot_counter ON bikri_lots
WHEN OLD.lot_counter > 0
 AND (NEW.seller_account_id IS NOT OLD.seller_account_id OR NEW.item_id IS NOT OLD.item_id)
 AND EXISTS (SELECT 1 FROM lot_counters c
             WHERE c.seller_account_id = NEW.seller_account_id AND c.item_id = NEW.item_id
               AND c.last_counter >= NEW.lot_counter)
BEGIN SELECT RAISE(ABORT, 'ERR_LOT_COUNTER_USED'); END;

CREATE TRIGGER bikri_lots_moved_follows AFTER UPDATE OF seller_account_id, item_id, lot_counter ON bikri_lots
WHEN NEW.lot_counter > 0
 AND (NEW.seller_account_id IS NOT OLD.seller_account_id OR NEW.item_id IS NOT OLD.item_id
      OR NEW.lot_counter IS NOT OLD.lot_counter)
BEGIN
    INSERT INTO lot_counters (seller_account_id, item_id, last_counter)
    VALUES (NEW.seller_account_id, NEW.item_id, NEW.lot_counter)
    ON CONFLICT (seller_account_id, item_id) DO UPDATE
        SET last_counter = MAX(last_counter, excluded.last_counter),
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
            version = version + 1;
END;
