-- Balances read the index only (NFR-SPD-2, NFR-SPD-3, NFR-SPD-9, NFR-SPD-13, NFR-SPD-21).
--
-- PRD FR-6.1: no screen keeps a balance of its own, so every balance is a SUM over posted journal
-- lines. This migration adds indexes only. It adds no table and no stored total, and the two
-- indexes of Architecture 15.8 stay as they are.

-- A balance is Σ dr_paisa and Σ cr_paisa of one account. With both amounts in the index, SQLite
-- adds them up from the index alone and never reads a line row (a covering index). The entry id
-- comes before the amounts so the "posted" check below can use it.
CREATE INDEX idx_journal_lines_account_amounts ON journal_lines (account_id, entry_id, dr_paisa, cr_paisa);

-- "Is this line posted?" For each line, a lookup of its entry used to be the cost of every
-- balance. A line is posted unless its entry is a draft, and a draft exists only inside the
-- transaction that is posting it (B16), so this index is empty whenever a balance is read.
-- A balance leaves out the lines of these entries instead of looking at every entry.
CREATE INDEX idx_journal_entries_unposted ON journal_entries (id) WHERE status <> 'posted';

-- A seller's advances (PRD FR-15.8) are a few lines among his thousands. The partial index lists
-- only the lines flagged as an advance, so the open advance on the party card does not read
-- the others.
CREATE INDEX idx_journal_lines_advance ON journal_lines (account_id, entry_id) WHERE is_advance = 1;
