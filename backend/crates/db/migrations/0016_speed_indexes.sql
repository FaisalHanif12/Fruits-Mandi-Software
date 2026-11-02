-- Migration 16 (plan 48 P48-S3, NFR-SPD-2, NFR-SPD-10, NFR-AI-3, Code Standards B18): two partial
-- indexes so the dashboard, the day summary R34 and the buyers' dues R20 read in under a second on
-- the NFR-SCL-1 volume (3.5 million journal lines).
--
-- The two indexes are `idx_journal_entries_opening` and `idx_bill_allocations_open_sum`. Their rows
-- for Architecture 15.8 go to Umar with the next document version; this run changes no requirement
-- document.
--
-- This file adds indexes only. It adds no table, no column and no stored total (PRD FR-6.1), and
-- no row, amount, id or chain hash changes.
--
-- 1. The opening entries. The attention list asks for a seller's opening credit, whatever its
--    date. Without an index that is a scan of every entry (1.2 seconds for a million). An opening
--    is rare (one for each account that was given an opening balance), so the partial index is
--    tiny. The predicate is the literal `posting_type = 'opening'`, so a query that says the same
--    uses it.
CREATE INDEX idx_journal_entries_opening ON journal_entries (id) WHERE posting_type = 'opening';

-- 2. What the active matching rows take from each bill. R20 and the dashboard need it for every
--    debit line of every buyer: `SUM(amount_paisa) GROUP BY bill_line_id` over the rows with
--    `is_active = 1`. With the amount in the index the sum is read from the index alone, with no
--    lookup of the row (0.28 seconds down to 0.07 for 300,000 bills). The predicate is the
--    literal `is_active = 1`, so the queries that say the same use it; a switched-off row (an undo)
--    is not in it.
CREATE INDEX idx_bill_allocations_open_sum ON bill_allocations (bill_line_id, amount_paisa) WHERE is_active = 1;
