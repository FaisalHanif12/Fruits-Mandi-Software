-- Migration 29 (plan 95 item 11, PRD FR-12.52 v1.57, Architecture 15.6): the پیشگی a seller takes with
-- a lot when he brings the goods. One column on `bikri_lots`.
--
-- `advance_paisa` is the amount typed on the lot. It is NOT in `deductions_paisa` or `net_paisa` (PRD
-- C6, C7): the lot's net and its seller credit (C8) are the same with or without it. The money itself
-- is its own journal entry (posting type 'advance', source the بکری), written in the same save as
-- the بکری: a seller debit flagged `is_advance` for each lot with an advance, and روکڑ credit for the
-- total. Keeping the amount on the lot lets an edit open the lot with it, and lets the daily books
-- check (NFR-SEC-6) compare each lot with the lines of that entry.
--
-- Whole rupees and never negative, as every money column. Every old lot gets 0: no journal line,
-- amount, id or chain hash changes. Only a lot that can take نقد has an advance: not trip goods, not a
-- garden's goods.
ALTER TABLE bikri_lots ADD COLUMN advance_paisa INTEGER NOT NULL DEFAULT 0
    CHECK (advance_paisa >= 0) CHECK (advance_paisa % 100 = 0) CHECK (advance_paisa <= 100000000000000)
    CHECK (advance_paisa = 0 OR (purchase_id IS NULL AND arrival_id IS NULL));
