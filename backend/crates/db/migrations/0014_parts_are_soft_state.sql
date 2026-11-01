-- Migration 14 (plan 44 follow-up, CLAUDE.md rule 4: business records are never DELETEd; PRD FR-7.70):
-- the parts of a trip or garden entry become SOFT STATE.
--
-- Until now the edit of an entry (migration 12) DELETED the rows of `venture_entry_parts` (the split of
-- one entry between the partners) and `purchase_cost_parts` (how one trip cost was spread over the
-- purchases, C24) inside the entry's edit window and inserted the new ones. From now on an edit marks
-- the old rows `is_active = 0`, inserts the new ones, and links each old row to the row that replaced
-- it (`replaced_by`, empty if nothing replaced it, for example a purchase that no longer gets a share
-- of a cost). Nothing is ever deleted; every reader of the two tables reads `is_active = 1` only. The
-- journal is not touched: an edit still reverses the old posting and posts the new one (PRD FR-6.3),
-- and the audit log still keeps the old and the new values.
--
-- What this file does, for each of the two tables:
--   1. Rebuilds the table with the two new columns. The old rows keep their ids and values and
--      become active. SQLite cannot drop an inline UNIQUE, so the table is rebuilt (the old one is
--      renamed away, the new one made, the rows copied, the old one dropped).
--   2. Replaces `UNIQUE (entry_id, partner_id)` and `UNIQUE (entry_id, purchase_id)` by a unique index
--      over the ACTIVE rows only, so an entry has at most one active row for a partner (a purchase),
--      and any number of retired ones.
--   3. Replaces the update trigger. An update is allowed only inside the edit window of the row's
--      entry and only to retire a row (`is_active` 1 to 0, nothing else changes), or to link a retired
--      row once to the active row that replaced it (same entry, same partner or purchase). Nothing
--      else, and never a row coming back to life.
--   4. Replaces the delete trigger: a row is never deleted, window or not.
--
-- No existing row, amount, id or chain hash changes. Nothing else points at these two tables by id.

-- ---- venture_entry_parts -----------------------------------------------------------------------

ALTER TABLE venture_entry_parts RENAME TO venture_entry_parts_old;

CREATE TABLE venture_entry_parts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entry_id INTEGER NOT NULL REFERENCES venture_entries(id),
    partner_id INTEGER NOT NULL REFERENCES partners(id),
    share_bps INTEGER NOT NULL CHECK (share_bps BETWEEN 1 AND 10000),
    source TEXT NOT NULL CHECK (source IN ('shop', 'pocket')),
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa % 100 = 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    replaced_by INTEGER REFERENCES venture_entry_parts(id),
    CHECK (replaced_by IS NULL OR is_active = 0)
) STRICT;

INSERT INTO venture_entry_parts
    (id, created_at, created_by, updated_at, updated_by, version, entry_id, partner_id, share_bps,
     source, amount_paisa, is_active, replaced_by)
SELECT id, created_at, created_by, updated_at, updated_by, version, entry_id, partner_id, share_bps,
       source, amount_paisa, 1, NULL
FROM venture_entry_parts_old;

-- The old table takes its index and its two triggers with it.
DROP TABLE venture_entry_parts_old;

CREATE INDEX idx_venture_entry_parts_partner ON venture_entry_parts (partner_id);
CREATE UNIQUE INDEX uq_venture_entry_parts_active ON venture_entry_parts (entry_id, partner_id)
    WHERE is_active = 1;
CREATE INDEX idx_venture_entry_parts_entry ON venture_entry_parts (entry_id, is_active);
CREATE INDEX idx_venture_entry_parts_replaced_by ON venture_entry_parts (replaced_by);

CREATE TRIGGER venture_entry_parts_no_update BEFORE UPDATE ON venture_entry_parts
WHEN NOT (
    EXISTS (SELECT 1 FROM edit_windows w
            WHERE w.entity = 'venture_entry' AND w.entity_id = OLD.entry_id)
    AND NEW.id IS OLD.id AND NEW.entry_id IS OLD.entry_id AND NEW.partner_id IS OLD.partner_id
    AND NEW.share_bps IS OLD.share_bps AND NEW.source IS OLD.source
    AND NEW.amount_paisa IS OLD.amount_paisa
    AND NEW.created_at IS OLD.created_at AND NEW.created_by IS OLD.created_by
    AND (
        -- retire
        (OLD.is_active = 1 AND NEW.is_active = 0 AND NEW.replaced_by IS NULL)
        -- link a retired row once to the active row of the same entry and partner
        OR (OLD.is_active = 0 AND NEW.is_active = 0 AND OLD.replaced_by IS NULL
            AND NEW.replaced_by IS NOT NULL
            AND NEW.replaced_by IN (SELECT n.id FROM venture_entry_parts n
                                    WHERE n.entry_id = OLD.entry_id AND n.partner_id = OLD.partner_id
                                      AND n.is_active = 1))
    )
)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER venture_entry_parts_no_delete BEFORE DELETE ON venture_entry_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- ---- purchase_cost_parts -----------------------------------------------------------------------

ALTER TABLE purchase_cost_parts RENAME TO purchase_cost_parts_old;

CREATE TABLE purchase_cost_parts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entry_id INTEGER NOT NULL REFERENCES venture_entries(id),
    purchase_id INTEGER NOT NULL REFERENCES purchases(id),
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa >= 0) CHECK (amount_paisa % 100 = 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    replaced_by INTEGER REFERENCES purchase_cost_parts(id),
    CHECK (replaced_by IS NULL OR is_active = 0)
) STRICT;

INSERT INTO purchase_cost_parts
    (id, created_at, created_by, updated_at, updated_by, version, entry_id, purchase_id, amount_paisa,
     is_active, replaced_by)
SELECT id, created_at, created_by, updated_at, updated_by, version, entry_id, purchase_id, amount_paisa,
       1, NULL
FROM purchase_cost_parts_old;

DROP TABLE purchase_cost_parts_old;

CREATE INDEX idx_purchase_cost_parts_purchase ON purchase_cost_parts (purchase_id);
CREATE UNIQUE INDEX uq_purchase_cost_parts_active ON purchase_cost_parts (entry_id, purchase_id)
    WHERE is_active = 1;
CREATE INDEX idx_purchase_cost_parts_entry ON purchase_cost_parts (entry_id, is_active);
CREATE INDEX idx_purchase_cost_parts_replaced_by ON purchase_cost_parts (replaced_by);

CREATE TRIGGER purchase_cost_parts_no_update BEFORE UPDATE ON purchase_cost_parts
WHEN NOT (
    EXISTS (SELECT 1 FROM edit_windows w
            WHERE w.entity = 'venture_entry' AND w.entity_id = OLD.entry_id)
    AND NEW.id IS OLD.id AND NEW.entry_id IS OLD.entry_id AND NEW.purchase_id IS OLD.purchase_id
    AND NEW.amount_paisa IS OLD.amount_paisa
    AND NEW.created_at IS OLD.created_at AND NEW.created_by IS OLD.created_by
    AND (
        (OLD.is_active = 1 AND NEW.is_active = 0 AND NEW.replaced_by IS NULL)
        OR (OLD.is_active = 0 AND NEW.is_active = 0 AND OLD.replaced_by IS NULL
            AND NEW.replaced_by IS NOT NULL
            AND NEW.replaced_by IN (SELECT n.id FROM purchase_cost_parts n
                                    WHERE n.entry_id = OLD.entry_id
                                      AND n.purchase_id = OLD.purchase_id AND n.is_active = 1))
    )
)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER purchase_cost_parts_no_delete BEFORE DELETE ON purchase_cost_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
