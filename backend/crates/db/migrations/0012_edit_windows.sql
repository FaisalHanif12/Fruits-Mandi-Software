-- Migration 12 (plan 44, PRD FR-7.70, FR-7.71; Architecture 15.3): the edit window.
--
-- The trip and garden entries (a purchase, a trip cost, a sale in another shop, a garden installment,
-- cost or sale, a garden arrival, a garden's rent) were read-only since migration 6: a trigger
-- refused every change of what was bought, paid or received. From plan 44 the Owner may correct
-- them while nothing of their goods has moved, so an edit must be able to change ONE entry inside its
-- own transaction.
--
-- How: `edit_windows` holds a row for the entry being edited. The edit service inserts the row, changes
-- the entry, its parts and its spread, and deletes the row, all before COMMIT. While the row exists the
-- triggers below let a change of THAT entry through; every other entry, and every change outside an edit
-- save, is refused exactly as before. The table is empty at all times between two saves. The journal is
-- not touched by this file: an edit still reverses the old posting and posts the new one (PRD FR-6.3),
-- and the audit log keeps the old and the new values.
--
-- `entity` says whose id it is:
--   purchase        `purchases.id`
--   venture_entry   `venture_entries.id` (its row, its parts and the cost parts spread from it)
--   garden_arrival  `garden_arrivals.id`
--   venture         `ventures.id` (the rent of a garden)
--
-- No existing row, amount, id or chain hash changes.

CREATE TABLE edit_windows (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entity TEXT NOT NULL CHECK (entity IN ('purchase', 'venture_entry', 'garden_arrival', 'venture')),
    entity_id INTEGER NOT NULL,
    UNIQUE (entity, entity_id)
) STRICT;

-- A purchase: what was bought and paid can change inside its window. Cancelled stays cancelled.
DROP TRIGGER purchases_record_is_final;
CREATE TRIGGER purchases_record_is_final BEFORE UPDATE ON purchases
WHEN (NEW.id IS NOT OLD.id
   OR NEW.number IS NOT OLD.number
   OR NEW.venture_id IS NOT OLD.venture_id
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.supplier_text IS NOT OLD.supplier_text
   OR NEW.item_id IS NOT OLD.item_id
   OR NEW.packing_id IS NOT OLD.packing_id
   OR NEW.qty IS NOT OLD.qty
   OR NEW.weight_grams IS NOT OLD.weight_grams
   OR NEW.rate_paisa IS NOT OLD.rate_paisa
   OR NEW.value_paisa IS NOT OLD.value_paisa
   OR NEW.freight_paisa IS NOT OLD.freight_paisa
   OR NEW.labour_paisa IS NOT OLD.labour_paisa
   OR NEW.other_cost_paisa IS NOT OLD.other_cost_paisa
   OR NEW.paid_now_paisa IS NOT OLD.paid_now_paisa
   OR (OLD.journal_entry_id IS NOT NULL AND NEW.journal_entry_id IS NOT OLD.journal_entry_id)
   OR (OLD.status = 'cancelled' AND NEW.status <> 'cancelled'))
  AND NOT (NEW.id IS OLD.id AND NEW.number IS OLD.number AND NEW.venture_id IS OLD.venture_id
           AND NEW.entry_date IS OLD.entry_date
           AND OLD.status <> 'cancelled'
           AND EXISTS (SELECT 1 FROM edit_windows w WHERE w.entity = 'purchase' AND w.entity_id = OLD.id))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- A venture entry: the amount, the reason, the quantity, the shop, the cost and the journal entry
-- can change inside the entry's window. Its venture, kind, number, date and purchase never do.
DROP TRIGGER venture_entries_record_is_final;
CREATE TRIGGER venture_entries_record_is_final BEFORE UPDATE ON venture_entries
WHEN (NEW.id IS NOT OLD.id
   OR NEW.venture_id IS NOT OLD.venture_id
   OR NEW.kind IS NOT OLD.kind
   OR NEW.number IS NOT OLD.number
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.amount_paisa IS NOT OLD.amount_paisa
   OR NEW.reason IS NOT OLD.reason
   OR NEW.purchase_id IS NOT OLD.purchase_id
   OR NEW.arrival_id IS NOT OLD.arrival_id
   OR NEW.moved_qty IS NOT OLD.moved_qty
   OR NEW.qty IS NOT OLD.qty
   OR NEW.shop_name IS NOT OLD.shop_name
   OR NEW.cost_paisa IS NOT OLD.cost_paisa
   OR (OLD.journal_entry_id IS NOT NULL AND NEW.journal_entry_id IS NOT OLD.journal_entry_id)
   OR (OLD.status = 'cancelled' AND NEW.status <> 'cancelled'))
  AND NOT (NEW.id IS OLD.id AND NEW.venture_id IS OLD.venture_id AND NEW.kind IS OLD.kind
           AND NEW.number IS OLD.number AND NEW.entry_date IS OLD.entry_date
           AND NEW.purchase_id IS OLD.purchase_id AND NEW.arrival_id IS OLD.arrival_id
           AND OLD.status <> 'cancelled'
           AND EXISTS (SELECT 1 FROM edit_windows w
                       WHERE w.entity = 'venture_entry' AND w.entity_id = OLD.id))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- The parts of an entry: replaced as a set inside the entry's window, never one by one outside it.
DROP TRIGGER venture_entry_parts_no_update;
CREATE TRIGGER venture_entry_parts_no_update BEFORE UPDATE ON venture_entry_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
DROP TRIGGER venture_entry_parts_no_delete;
CREATE TRIGGER venture_entry_parts_no_delete BEFORE DELETE ON venture_entry_parts
WHEN NOT EXISTS (SELECT 1 FROM edit_windows w
                 WHERE w.entity = 'venture_entry' AND w.entity_id = OLD.entry_id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- How a trip cost was spread over the purchases: spread again inside the cost's window.
DROP TRIGGER purchase_cost_parts_no_update;
CREATE TRIGGER purchase_cost_parts_no_update BEFORE UPDATE ON purchase_cost_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
DROP TRIGGER purchase_cost_parts_no_delete;
CREATE TRIGGER purchase_cost_parts_no_delete BEFORE DELETE ON purchase_cost_parts
WHEN NOT EXISTS (SELECT 1 FROM edit_windows w
                 WHERE w.entity = 'venture_entry' AND w.entity_id = OLD.entry_id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- A garden arrival: what arrived (item, packing, quantity, weight, note) can change inside its window.
DROP TRIGGER garden_arrivals_record_is_final;
CREATE TRIGGER garden_arrivals_record_is_final BEFORE UPDATE ON garden_arrivals
WHEN (NEW.id IS NOT OLD.id
   OR NEW.number IS NOT OLD.number
   OR NEW.venture_id IS NOT OLD.venture_id
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.item_id IS NOT OLD.item_id
   OR NEW.packing_id IS NOT OLD.packing_id
   OR NEW.qty IS NOT OLD.qty
   OR NEW.weight_grams IS NOT OLD.weight_grams
   OR NEW.note IS NOT OLD.note
   OR (OLD.status = 'cancelled' AND NEW.status <> 'cancelled'))
  AND NOT (NEW.id IS OLD.id AND NEW.number IS OLD.number AND NEW.venture_id IS OLD.venture_id
           AND NEW.entry_date IS OLD.entry_date
           AND OLD.status <> 'cancelled'
           AND EXISTS (SELECT 1 FROM edit_windows w
                       WHERE w.entity = 'garden_arrival' AND w.entity_id = OLD.id))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- A venture: the rent of a garden can change inside the venture's window. The identity never does.
DROP TRIGGER ventures_identity_is_final;
CREATE TRIGGER ventures_identity_is_final BEFORE UPDATE ON ventures
WHEN NEW.id IS NOT OLD.id
   OR NEW.kind IS NOT OLD.kind
   OR NEW.number IS NOT OLD.number
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.account_id IS NOT OLD.account_id
   OR (NEW.rent_paisa IS NOT OLD.rent_paisa
       AND NOT (OLD.kind = 'garden'
                AND EXISTS (SELECT 1 FROM edit_windows w
                            WHERE w.entity = 'venture' AND w.entity_id = OLD.id)))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
