-- Migration 9 (plan 33 v1.1, PRD 1.8 FR-12.39, FR-12.40): the date to sell on a trip purchase, and the
-- goods that arrived from a garden (مال آیا).
--
-- A purchase gets a date to sell (`sell_date`). It starts as the purchase day and is never before it.
-- A new بکری opens with a ready lot for every stock record with goods left whose date to sell is on or
-- before its day. Existing purchases get their own entry date, so nothing moves.
--
-- A garden arrival is a stock record of a garden (PRD FR-12.40): the garden, the day it arrived, the
-- day to sell, the item, the packing and the quantity, with the running figures `qty_sold` and
-- `qty_written_off` kept in the same save as the sale or write-off that changes them. It posts NO
-- journal entry, writes no partner row and has no cost: it is only goods. A garden lot of a بکری can
-- be linked to it (`bikri_lots.arrival_id`); the lot's sold goods come off the arrival and the rest
-- stays in it for the next new بکری. The stock moves of an arrival carry `stock_moves.arrival_id`.
--
-- What this file does:
--   1. `purchases.sell_date` (nullable in the file because SQLite cannot add a NOT NULL column
--      without a default; the code always writes it, and every read takes `COALESCE(sell_date,
--      entry_date)`), filled with the entry date, with a trigger that refuses a date before the
--      purchase day.
--   2. The table `garden_arrivals` with its checks, indexes and triggers (never deleted; what arrived
--      never changes; only the running figures, the date to sell and the cancel can change).
--   3. `bikri_lots.arrival_id` and `stock_moves.arrival_id` (ADD COLUMN, so no row is touched), and
--      the two stock-move triggers again so a move may belong to an arrival.
--
-- Nothing is deleted and no posted row is touched (PRD FR-6.3).

ALTER TABLE purchases ADD COLUMN sell_date TEXT
    CHECK (sell_date IS NULL OR date(sell_date) IS sell_date);
UPDATE purchases SET sell_date = entry_date;

CREATE TRIGGER purchases_sell_date_not_early_insert BEFORE INSERT ON purchases
WHEN NEW.sell_date IS NOT NULL AND NEW.sell_date < NEW.entry_date
BEGIN SELECT RAISE(ABORT, 'ERR_SELL_DATE_BEFORE_ENTRY'); END;
CREATE TRIGGER purchases_sell_date_not_early_update BEFORE UPDATE OF sell_date ON purchases
WHEN NEW.sell_date IS NOT NULL AND NEW.sell_date < NEW.entry_date
BEGIN SELECT RAISE(ABORT, 'ERR_SELL_DATE_BEFORE_ENTRY'); END;

CREATE INDEX idx_purchases_sell_date ON purchases (sell_date);

-- Architecture 15.6 (added by plan 33 v1.1). The weight is in grams for a weight item.
CREATE TABLE garden_arrivals (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    number INTEGER NOT NULL UNIQUE CHECK (number > 0),
    venture_id INTEGER NOT NULL REFERENCES ventures(id),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    sell_date TEXT NOT NULL CHECK (date(sell_date) IS sell_date),
    item_id INTEGER NOT NULL REFERENCES items(id),
    packing_id INTEGER NOT NULL REFERENCES packings(id),
    qty INTEGER NOT NULL CHECK (qty > 0),
    weight_grams INTEGER CHECK (weight_grams IS NULL OR weight_grams > 0),
    note TEXT,
    status TEXT NOT NULL DEFAULT 'saved' CHECK (status IN ('saved', 'cancelled')),
    cancel_reason TEXT,
    qty_sold INTEGER NOT NULL DEFAULT 0 CHECK (qty_sold >= 0),
    qty_written_off INTEGER NOT NULL DEFAULT 0 CHECK (qty_written_off >= 0),
    -- The date to sell is never before the day the goods arrived (PRD FR-12.40).
    CHECK (sell_date >= entry_date),
    -- Sold plus written off is never more than arrived (Code Standards 9.3).
    CHECK (qty_sold + qty_written_off <= qty),
    CHECK (status <> 'cancelled' OR (cancel_reason IS NOT NULL AND length(trim(cancel_reason)) > 0))
) STRICT;

CREATE INDEX idx_garden_arrivals_venture ON garden_arrivals (venture_id, entry_date);
CREATE INDEX idx_garden_arrivals_date ON garden_arrivals (entry_date);
CREATE INDEX idx_garden_arrivals_sell_date ON garden_arrivals (sell_date);
CREATE INDEX idx_garden_arrivals_item ON garden_arrivals (item_id);
CREATE INDEX idx_garden_arrivals_packing ON garden_arrivals (packing_id);
CREATE INDEX idx_garden_arrivals_qty ON garden_arrivals (qty, qty_sold, qty_written_off);

-- An arrival is never deleted (Code Standards D3). A cancel sets status and reason.
CREATE TRIGGER garden_arrivals_no_delete BEFORE DELETE ON garden_arrivals
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER garden_arrivals_no_replace BEFORE INSERT ON garden_arrivals
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM garden_arrivals WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- What arrived never changes. Only the running figures (sold, written off), the date to sell, the
-- status with its reason and the system columns can change. A cancel cannot be taken back.
CREATE TRIGGER garden_arrivals_record_is_final BEFORE UPDATE ON garden_arrivals
WHEN NEW.id IS NOT OLD.id
   OR NEW.number IS NOT OLD.number
   OR NEW.venture_id IS NOT OLD.venture_id
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.item_id IS NOT OLD.item_id
   OR NEW.packing_id IS NOT OLD.packing_id
   OR NEW.qty IS NOT OLD.qty
   OR NEW.weight_grams IS NOT OLD.weight_grams
   OR NEW.note IS NOT OLD.note
   OR (OLD.status = 'cancelled' AND NEW.status <> 'cancelled')
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

ALTER TABLE bikri_lots ADD COLUMN arrival_id INTEGER REFERENCES garden_arrivals(id);
CREATE INDEX idx_bikri_lots_arrival ON bikri_lots (arrival_id);

ALTER TABLE stock_moves ADD COLUMN arrival_id INTEGER REFERENCES garden_arrivals(id);
CREATE INDEX idx_stock_moves_arrival ON stock_moves (arrival_id, move_date);

-- A move belongs to a lot, a purchase or an arrival.
DROP TRIGGER stock_moves_has_an_owner;
CREATE TRIGGER stock_moves_has_an_owner BEFORE INSERT ON stock_moves
WHEN NEW.lot_id IS NULL AND NEW.purchase_id IS NULL AND NEW.arrival_id IS NULL
BEGIN SELECT RAISE(ABORT, 'ERR_STOCK_MOVE_OWNER'); END;

-- A reversal move is the exact opposite of the move it names: the same lot, purchase and arrival
-- (each may be empty), the opposite quantity and cost, and kind 'reversal'.
DROP TRIGGER stock_moves_reversal_mirrors;
CREATE TRIGGER stock_moves_reversal_mirrors BEFORE INSERT ON stock_moves
WHEN NEW.reverses_move_id IS NOT NULL
  AND (NEW.kind <> 'reversal'
       OR NOT EXISTS (SELECT 1 FROM stock_moves o
                      WHERE o.id = NEW.reverses_move_id AND o.lot_id IS NEW.lot_id
                        AND o.purchase_id IS NEW.purchase_id AND o.arrival_id IS NEW.arrival_id
                        AND o.qty = -NEW.qty
                        AND o.cost_paisa = -NEW.cost_paisa
                        AND o.kind <> 'reversal'))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
