-- Migration 19 (plan 58, PRD FR-12.42, Architecture 15.6 v1.19): the quality of a sale line.
--
-- The goods of one lot can differ in quality, and each quality sells at its own rate. The munshi
-- sells them as separate lines and may mark each line `first`, `second` or `third` (اول, دوم,
-- سوم), or leave it empty. The quality changes no figure: it is only shown on the bills and the
-- ledgers. One nullable column; every old line keeps NULL, so no book, amount, id or chain hash
-- changes.
ALTER TABLE bikri_sales ADD COLUMN grade TEXT CHECK (grade IN ('first', 'second', 'third'));

-- The quality is part of the stored snapshot of a line (PRD FR-12.42, FR-11.6): a saved line is
-- never changed, only replaced by an edit, and an edit is audited. So the trigger of migration 6
-- is made again with `grade` among the frozen columns. Nothing else in it changes.
DROP TRIGGER bikri_sales_snapshot_is_final;
CREATE TRIGGER bikri_sales_snapshot_is_final BEFORE UPDATE ON bikri_sales
WHEN NEW.id IS NOT OLD.id
   OR NEW.bikri_id IS NOT OLD.bikri_id
   OR NEW.lot_id IS NOT OLD.lot_id
   OR NEW.line_no IS NOT OLD.line_no
   OR NEW.journal_entry_id IS NOT OLD.journal_entry_id
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.buyer_account_id IS NOT OLD.buyer_account_id
   OR NEW.pukhta_name_ur IS NOT OLD.pukhta_name_ur
   OR NEW.pukhta_name_en IS NOT OLD.pukhta_name_en
   OR NEW.pukhta_phone IS NOT OLD.pukhta_phone
   OR NEW.pukhta_person_key IS NOT OLD.pukhta_person_key
   OR NEW.qty IS NOT OLD.qty
   OR NEW.weight_grams IS NOT OLD.weight_grams
   OR NEW.seller_rate_paisa IS NOT OLD.seller_rate_paisa
   OR NEW.seller_amount_paisa IS NOT OLD.seller_amount_paisa
   OR NEW.seller_comm_method IS NOT OLD.seller_comm_method
   OR NEW.seller_comm_value_bps IS NOT OLD.seller_comm_value_bps
   OR NEW.seller_comm_value_paisa IS NOT OLD.seller_comm_value_paisa
   OR NEW.seller_comm_rule_id IS NOT OLD.seller_comm_rule_id
   OR NEW.seller_comm_paisa IS NOT OLD.seller_comm_paisa
   OR NEW.buyer_rate_paisa IS NOT OLD.buyer_rate_paisa
   OR NEW.buyer_value_paisa IS NOT OLD.buyer_value_paisa
   OR NEW.buyer_comm_method IS NOT OLD.buyer_comm_method
   OR NEW.buyer_comm_value_bps IS NOT OLD.buyer_comm_value_bps
   OR NEW.buyer_comm_value_paisa IS NOT OLD.buyer_comm_value_paisa
   OR NEW.buyer_comm_rule_id IS NOT OLD.buyer_comm_rule_id
   OR NEW.buyer_comm_paisa IS NOT OLD.buyer_comm_paisa
   OR NEW.laga_method IS NOT OLD.laga_method
   OR NEW.laga_value_bps IS NOT OLD.laga_value_bps
   OR NEW.laga_value_paisa IS NOT OLD.laga_value_paisa
   OR NEW.laga_rule_id IS NOT OLD.laga_rule_id
   OR NEW.laga_paisa IS NOT OLD.laga_paisa
   OR NEW.buyer_total_paisa IS NOT OLD.buyer_total_paisa
   OR NEW.rate_diff_paisa IS NOT OLD.rate_diff_paisa
   OR NEW.payment_type IS NOT OLD.payment_type
   OR NEW.cost_paisa IS NOT OLD.cost_paisa
   OR NEW.grade IS NOT OLD.grade
   OR (OLD.is_active = 0 AND NEW.is_active = 1)
   OR (OLD.replaced_by_sale_id IS NOT NULL AND NEW.replaced_by_sale_id IS NOT OLD.replaced_by_sale_id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
