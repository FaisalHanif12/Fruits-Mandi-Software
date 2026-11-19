-- Migration 21 (plan 65, PRD v1.32 FR-12.44, FR-12.9, FR-15.4; Architecture 15.6 v1.20; D8 Feature 4,
-- decisions P1 to P14 of plan 60): money paid on the spot (موقع پر جمع).
--
-- A money field at the end of each buyer line replaces the ادھاری / نقدی toggle: how much the buyer paid
-- right there, from 0 up to the line's buyer total. The amount is stored with the line, in a new
-- column `bikri_sales.paid_paisa` (whole rupees, never negative, never above `buyer_total_paisa`).
--
-- What this file does, in this order:
--   1. Adds the column, with the CHECKs, default 0. SQLite tests an added CHECK against every old row:
--      0 is never above the buyer total, so every old row passes.
--   2. Gives every old line the meaning it had: a line saved as `cash` was paid in full, so its
--      `paid_paisa` is its buyer total; a line saved as `credit` stays at 0. No `is_active` filter, so
--      replaced and cancelled lines say the same as when they were in force. `version` and
--      `updated_at` are not touched (this is the meaning of an old row, not an edit by a person).
--   3. Makes `bikri_sales_snapshot_is_final` again with `paid_paisa` among the frozen columns (PRD
--      FR-12.20, FR-11.6: a saved line is never changed, only replaced by an edit, which is audited).
--      The trigger is written with the `chukta_*` names of migration 20.
--
-- `bikri_sales.payment_type` stays in the table and is still written (`cash` only when paid is the
-- whole buyer total, else `credit`), but nothing decides money from it any more. It is not tied to
-- `paid_paisa` in a CHECK, so an old row stays valid as it is.
--
-- No journal line, journal entry, bill or advance allocation, amount, id or chain hash is touched: the
-- chain covers values of entries and lines, never a column of `bikri_sales`. Every old balance, due
-- and total stays exactly as it was.

ALTER TABLE bikri_sales ADD COLUMN paid_paisa INTEGER NOT NULL DEFAULT 0
    CHECK (paid_paisa >= 0)
    CHECK (paid_paisa % 100 = 0)
    CHECK (paid_paisa <= buyer_total_paisa);

UPDATE bikri_sales SET paid_paisa = buyer_total_paisa WHERE payment_type = 'cash';

DROP TRIGGER bikri_sales_snapshot_is_final;
CREATE TRIGGER bikri_sales_snapshot_is_final BEFORE UPDATE ON bikri_sales
WHEN NEW.id IS NOT OLD.id
   OR NEW.bikri_id IS NOT OLD.bikri_id
   OR NEW.lot_id IS NOT OLD.lot_id
   OR NEW.line_no IS NOT OLD.line_no
   OR NEW.journal_entry_id IS NOT OLD.journal_entry_id
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.buyer_account_id IS NOT OLD.buyer_account_id
   OR NEW.chukta_name_ur IS NOT OLD.chukta_name_ur
   OR NEW.chukta_name_en IS NOT OLD.chukta_name_en
   OR NEW.chukta_phone IS NOT OLD.chukta_phone
   OR NEW.chukta_person_key IS NOT OLD.chukta_person_key
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
   OR NEW.paid_paisa IS NOT OLD.paid_paisa
   OR (OLD.is_active = 0 AND NEW.is_active = 1)
   OR (OLD.replaced_by_sale_id IS NOT NULL AND NEW.replaced_by_sale_id IS NOT OLD.replaced_by_sale_id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
