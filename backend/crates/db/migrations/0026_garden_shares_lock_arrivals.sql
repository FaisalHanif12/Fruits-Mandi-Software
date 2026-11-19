-- Migration 26 (plan 80 gate, Q-R4-1, PRD FR-12.29, Architecture 15.5): goods arrived (مال آیا) counts
-- as an entry of a garden for locking its shares.
--
-- Migration 6 locks the rows of `venture_partners` once the venture has a row in `venture_entries`.
-- Goods arrived is saved in `garden_arrivals`, not in `venture_entries`, and posts nothing, so a garden
-- with goods arrived but nothing sold could still have its shares changed. These three triggers add
-- the same refusal when the venture has a `garden_arrivals` row (a cancelled one counts too, like a
-- cancelled entry). Migration 6 is never edited: its triggers stay and these are new, with their own
-- names. They abort with the same text, so the services map them to the same error
-- (`ERR_VENTURE_SHARES_LOCKED`).
--
-- This migration adds three triggers and nothing else: no table, column, row, amount, id or chain
-- hash changes.
CREATE TRIGGER venture_partners_arrivals_lock_insert BEFORE INSERT ON venture_partners
WHEN EXISTS (SELECT 1 FROM garden_arrivals a WHERE a.venture_id = NEW.venture_id)
BEGIN SELECT RAISE(ABORT, 'ERR_VENTURE_SHARES_LOCKED'); END;

CREATE TRIGGER venture_partners_arrivals_lock_update BEFORE UPDATE ON venture_partners
WHEN (NEW.share_bps IS NOT OLD.share_bps OR NEW.partner_id IS NOT OLD.partner_id
      OR NEW.venture_id IS NOT OLD.venture_id)
  AND EXISTS (SELECT 1 FROM garden_arrivals a WHERE a.venture_id = OLD.venture_id)
BEGIN SELECT RAISE(ABORT, 'ERR_VENTURE_SHARES_LOCKED'); END;

CREATE TRIGGER venture_partners_arrivals_lock_delete BEFORE DELETE ON venture_partners
WHEN EXISTS (SELECT 1 FROM garden_arrivals a WHERE a.venture_id = OLD.venture_id)
BEGIN SELECT RAISE(ABORT, 'ERR_VENTURE_SHARES_LOCKED'); END;
