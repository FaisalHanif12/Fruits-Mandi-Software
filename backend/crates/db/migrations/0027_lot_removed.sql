-- Migration 27 (plan 80 gate, Q-R2-1, PRD FR-7.70, FR-12.19, FR-6.3, Code Standards D3): a saved lot
-- can be removed in an edit of the first posting of its بکری, unless a later sale, a write-off or fruit
-- taken rests on it.
--
-- A business record is never deleted, so the lot stays as a row. `removed_at` is NULL while the lot is
-- in force and the time of the edit that removed it after that. The edit has already reversed the
-- posting (journal, partner shares, stock moves, analytics facts, trip and garden goods) and ended the
-- lot's sale lines as `removed`; the lot's own columns stay as the record of what was removed, as the
-- lots of a cancelled بکری do. Every read that lists or adds up lots leaves a removed lot out.
--
-- This file adds one column and three guard triggers: no row, amount, id or chain hash changes.
ALTER TABLE bikri_lots ADD COLUMN removed_at TEXT
    CHECK (removed_at IS NULL OR removed_at <> '');

-- Guards for a removed lot (plan 80 Q-R2-1, the money review): the services already refuse to sell,
-- write off or take fruit from a removed lot (`lot_state` does not find it), and these triggers refuse
-- it in the database too, so a bug can never post a seller credit on goods that are no longer there.
-- A lot is removed once: `removed_at`, once set, never changes.
CREATE TRIGGER bikri_lots_removed_is_final BEFORE UPDATE OF removed_at ON bikri_lots
WHEN OLD.removed_at IS NOT NULL AND NEW.removed_at IS NOT OLD.removed_at
BEGIN SELECT RAISE(ABORT, 'ERR_LOT_REMOVED'); END;

CREATE TRIGGER bikri_lots_removed_takes_no_goods BEFORE UPDATE OF qty_sold, qty_written_off, qty_taken ON bikri_lots
WHEN OLD.removed_at IS NOT NULL AND (NEW.qty_sold IS NOT OLD.qty_sold
     OR NEW.qty_written_off IS NOT OLD.qty_written_off OR NEW.qty_taken IS NOT OLD.qty_taken)
BEGIN SELECT RAISE(ABORT, 'ERR_LOT_REMOVED'); END;

CREATE TRIGGER bikri_sales_no_sale_on_a_removed_lot BEFORE INSERT ON bikri_sales
WHEN EXISTS (SELECT 1 FROM bikri_lots l WHERE l.id = NEW.lot_id AND l.removed_at IS NOT NULL)
BEGIN SELECT RAISE(ABORT, 'ERR_LOT_REMOVED'); END;
