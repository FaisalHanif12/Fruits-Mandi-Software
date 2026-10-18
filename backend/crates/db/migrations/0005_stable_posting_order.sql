-- Migration 5 (plan 16, lead decision of 17-10-2026, money agent finding 1 of plan 15).
--
-- A stable order for postings. Matching settles the OLDEST bill first and adjusts the OLDEST
-- advance first (PRD FR-15.4, FR-15.8), and "oldest" has been (date, opening first, entry id).
-- An edit or a cancel writes its new posting as a NEW journal entry with a higher id, so a
-- same-day edit moved the posting behind a بکری saved later the same day. For advances that could
-- refuse an edited seller return or drop the adjustment of an edited پیشگی.
--
-- `order_key` fixes it. NULL means "the entry's own id" (every entry written so far, and every
-- new posting). An edit's new posting copies the key of the posting it replaces (its key, or its
-- id if it has none), so an edited posting keeps its place
-- in the day. Matching orders lines by (date, opening first, COALESCE(order_key, id), id, line_no).
-- The key is set when the entry is inserted as a draft; a posted entry never changes (trigger
-- `journal_entries_posted_is_final`). The journal hash chain covers it only when it is set, so the
-- chain of every entry written before this migration is unchanged.
--
-- Stock moves get a link from a reversal to the move it reverses, so an edit, a cancel and the undo
-- of a write-off can tell which moves are still in force. A move is reversed at most once.
--
-- A sale line says how it left force. `ended` is NULL while the line is in force (`is_active = 1`).
-- An edit replaces a line ('replaced', with `replaced_by_sale_id`) or drops it ('removed'); a cancel
-- ends every line of the بکری ('cancelled'). The register and the saved view of a cancelled بکری
-- need to tell a line that was cancelled from one that an earlier edit had already dropped
-- (Architecture 15.6: `is_active` is 0 when replaced by an edit or cancelled).
--
-- Three ALTER TABLE ADD COLUMN, one index and one trigger: no row is touched and no number changes.

ALTER TABLE journal_entries ADD COLUMN order_key INTEGER CHECK (order_key IS NULL OR order_key > 0);

ALTER TABLE stock_moves ADD COLUMN reverses_move_id INTEGER REFERENCES stock_moves(id);

ALTER TABLE bikri_sales ADD COLUMN ended TEXT
    CHECK (ended IS NULL OR ended IN ('replaced', 'removed', 'cancelled'))
    CHECK ((ended IS NULL) = (is_active = 1));

CREATE UNIQUE INDEX idx_stock_moves_reverses ON stock_moves (reverses_move_id) WHERE reverses_move_id IS NOT NULL;

-- A reversal move is the exact opposite of the move it names: the same lot, the opposite quantity,
-- and kind 'reversal'. Only a reversal names a move.
CREATE TRIGGER stock_moves_reversal_mirrors BEFORE INSERT ON stock_moves
WHEN NEW.reverses_move_id IS NOT NULL
  AND (NEW.kind <> 'reversal'
       OR NOT EXISTS (SELECT 1 FROM stock_moves o
                      WHERE o.id = NEW.reverses_move_id AND o.lot_id = NEW.lot_id AND o.qty = -NEW.qty
                        AND o.kind <> 'reversal'))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
