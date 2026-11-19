-- Migration 10 (plan 35, PRD FR-12.41): the split between our shop and another shop.
--
-- Every line of goods, a trip purchase and a garden arrival, says how many of its نگ are for another
-- shop (`other_qty`; the rest are for our shop) and the name of that shop (`shop_name`, as typed).
-- `qty_sold_other` is the part of `qty_sold` that was sold in other shops, so what our shop sold is
-- `qty_sold - qty_sold_other`. A write-off is one number on the line; the parts are worked out from
-- the counters (`mandi_core::split::StockParts`). No money is stored here or changed.
--
-- What this file does:
--   1. `purchases` and `garden_arrivals` get `other_qty`, `qty_sold_other` and `shop_name`. A purchase
--      that was sold in other shops before this file gets the qty of those sales (the saved
--      `other_sale` entries of the purchase) as its `other_qty` and as `qty_sold_other`, so every
--      number it had stays true; every other row starts as all for our shop.
--   2. Triggers keep the split true: the part for another shop is not above the qty, other shops did
--      not sell more than their part, and our shop did not sell more than its part.
--   3. `venture_entries.arrival_id`: a garden's sale in another shop can name the arrival it takes
--      its goods from (PRD FR-12.41). `venture_entries.moved_qty`: the نگ a sale in another shop
--      moved from our shop's part to the part for another shop, because that part did not have
--      enough (plan 35 D1); a cancel moves them back. The trigger that freezes an entry freezes
--      both.
--
-- Nothing is deleted and no posted row is touched (PRD FR-6.3).

ALTER TABLE purchases ADD COLUMN other_qty INTEGER NOT NULL DEFAULT 0 CHECK (other_qty >= 0);
ALTER TABLE purchases ADD COLUMN qty_sold_other INTEGER NOT NULL DEFAULT 0 CHECK (qty_sold_other >= 0);
ALTER TABLE purchases ADD COLUMN shop_name TEXT;

UPDATE purchases
SET other_qty = COALESCE((SELECT SUM(e.qty) FROM venture_entries e
                          WHERE e.purchase_id = purchases.id AND e.kind = 'other_sale'
                            AND e.status = 'saved'), 0),
    qty_sold_other = COALESCE((SELECT SUM(e.qty) FROM venture_entries e
                               WHERE e.purchase_id = purchases.id AND e.kind = 'other_sale'
                                 AND e.status = 'saved'), 0);

CREATE TRIGGER purchases_split_fits_insert BEFORE INSERT ON purchases
WHEN NEW.other_qty > NEW.qty
  OR NEW.qty_sold_other > NEW.other_qty
  OR NEW.qty_sold_other > NEW.qty_sold
  OR NEW.qty_sold - NEW.qty_sold_other > NEW.qty - NEW.other_qty
BEGIN SELECT RAISE(ABORT, 'ERR_SPLIT_BROKEN'); END;
CREATE TRIGGER purchases_split_fits_update BEFORE UPDATE ON purchases
WHEN NEW.other_qty > NEW.qty
  OR NEW.qty_sold_other > NEW.other_qty
  OR NEW.qty_sold_other > NEW.qty_sold
  OR NEW.qty_sold - NEW.qty_sold_other > NEW.qty - NEW.other_qty
BEGIN SELECT RAISE(ABORT, 'ERR_SPLIT_BROKEN'); END;

ALTER TABLE garden_arrivals ADD COLUMN other_qty INTEGER NOT NULL DEFAULT 0 CHECK (other_qty >= 0);
ALTER TABLE garden_arrivals ADD COLUMN qty_sold_other INTEGER NOT NULL DEFAULT 0 CHECK (qty_sold_other >= 0);
ALTER TABLE garden_arrivals ADD COLUMN shop_name TEXT;

CREATE TRIGGER garden_arrivals_split_fits_insert BEFORE INSERT ON garden_arrivals
WHEN NEW.other_qty > NEW.qty
  OR NEW.qty_sold_other > NEW.other_qty
  OR NEW.qty_sold_other > NEW.qty_sold
  OR NEW.qty_sold - NEW.qty_sold_other > NEW.qty - NEW.other_qty
BEGIN SELECT RAISE(ABORT, 'ERR_SPLIT_BROKEN'); END;
CREATE TRIGGER garden_arrivals_split_fits_update BEFORE UPDATE ON garden_arrivals
WHEN NEW.other_qty > NEW.qty
  OR NEW.qty_sold_other > NEW.other_qty
  OR NEW.qty_sold_other > NEW.qty_sold
  OR NEW.qty_sold - NEW.qty_sold_other > NEW.qty - NEW.other_qty
BEGIN SELECT RAISE(ABORT, 'ERR_SPLIT_BROKEN'); END;

ALTER TABLE venture_entries ADD COLUMN arrival_id INTEGER REFERENCES garden_arrivals(id);
ALTER TABLE venture_entries ADD COLUMN moved_qty INTEGER NOT NULL DEFAULT 0 CHECK (moved_qty >= 0);
CREATE INDEX idx_venture_entries_arrival ON venture_entries (arrival_id);

DROP TRIGGER venture_entries_record_is_final;
CREATE TRIGGER venture_entries_record_is_final BEFORE UPDATE ON venture_entries
WHEN NEW.id IS NOT OLD.id
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
   OR (OLD.status = 'cancelled' AND NEW.status <> 'cancelled')
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
