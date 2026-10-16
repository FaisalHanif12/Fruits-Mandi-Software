-- Migration 4 (plan 13, lead decision of 16-10-2026): the بکری tables cannot lose a row, and a
-- بکری keeps its number. Triggers only: no table, column or index changes and no row is touched.
--
-- Business records are never deleted (CLAUDE.md rule 4, Code Standards D3): a cancelled بکری keeps
-- its row, its lots and its lines. The number of a بکری is given once at its first save and never
-- used again (PRD FR-12.2); the save takes the biggest number plus one, so a deleted row or a
-- changed number could give a number twice. These triggers are defence in depth: the services
-- never delete or renumber, and the database now refuses it too (Code Standards 4.1 layer 4).
--
-- What stays allowed, for the plans that edit and cancel (plan 16): `status`, `cancel_reason`,
-- `cancelled_by`, `cancelled_at` and `print_count` of a بکری, the running totals and
-- `qty_written_off` of a lot, and `is_active` (1 to 0) and `replaced_by_sale_id` of a sale line
-- (its own trigger `bikri_sales_snapshot_is_final` guards the rest).
--
-- Connections run with `recursive_triggers = ON`, so `INSERT OR REPLACE` cannot remove a row
-- without these triggers firing; the `no_replace` ones refuse it even when that setting is off
-- (for example in the sqlite3 shell).

CREATE TRIGGER bikri_entries_no_delete BEFORE DELETE ON bikri_entries
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER bikri_lots_no_delete BEFORE DELETE ON bikri_lots
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER bikri_sales_no_delete BEFORE DELETE ON bikri_sales
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- The number is final once the row exists.
CREATE TRIGGER bikri_entries_number_is_final BEFORE UPDATE OF number ON bikri_entries
WHEN NEW.number IS NOT OLD.number
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- INSERT OR REPLACE of an existing id or number deletes the old row; refuse it even when
-- recursive_triggers is off.
CREATE TRIGGER bikri_entries_no_replace BEFORE INSERT ON bikri_entries
WHEN (NEW.id > 0 AND EXISTS (SELECT 1 FROM bikri_entries WHERE id = NEW.id))
  OR EXISTS (SELECT 1 FROM bikri_entries WHERE number = NEW.number)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER bikri_lots_no_replace BEFORE INSERT ON bikri_lots
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM bikri_lots WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER bikri_sales_no_replace BEFORE INSERT ON bikri_sales
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM bikri_sales WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
