-- Migration 24 (plan 77, robustness findings 2 and 3, NFR-SPD-24, Code Standards B18): three covering
-- indexes so the commission page of the whole shop and the sale views of analytics (A, B, C, E, L)
-- read ten years in a few seconds instead of minutes on the NFR-SCL-1 volume (1.2 million sale
-- facts, 520,000 partner share rows).
--
-- Their rows for Architecture 15.8 go to Umar with the next document version; this run changes no
-- requirement document.
--
-- This file adds indexes only. It adds no table, no column and no stored total (PRD FR-6.1), and no
-- row, amount, id or chain hash changes.
--
-- 1. The income share rows by day. The page sums each partner's income shares for each day of the
--    range: `WHERE kind = 'income' AND entry_date BETWEEN ... GROUP BY entry_date, partner_id`. No
--    index started with `entry_date` or `kind`, so the query scanned every share row of the shop (1.3
--    seconds on the volume above). With the kind first (a constant in this query), then the day, the
--    partner and the amount, the days of the range are read from the index alone, already in the
--    order of the grouping: no lookup of the row and no sort (0.14 seconds).
CREATE INDEX idx_partner_share_rows_kind_date ON partner_share_rows (kind, entry_date, partner_id, amount_paisa);

-- 2. The commission of the shop by day. The page sums `sign × value` of the three commission columns
--    for each day of the range. With the date index alone, each fact needs a lookup of its row (5
--    seconds for ten years); with the sign and the three columns in the index the sums are read
--    from the index alone, in day order. A query for one party keeps using his own index (account,
--    date). The queries that only need the date keep finding it through `idx_analytics_facts_date`.
CREATE INDEX idx_analytics_facts_date_sums ON analytics_facts (entry_date, sign, seller_comm_paisa, buyer_comm_paisa, laga_paisa);

-- 3. The sale facts of analytics. Views A, B, C, E and L group the facts of a range by item, seller,
--    buyer, person and بکری and sum six columns by sign. With the date index alone each fact needs a
--    lookup of its row in a 1.5 GB file (10 to 14 seconds for ten years while the file is not in
--    memory); with the dimensions and the sums in the index the range is read from the index alone
--    (about 3 seconds on the same machine, the rest being the grouping). A query that also filters
--    on the city, the اڈا, a trip, a garden's purchases or a quality needs a column that is not in
--    it, so it keeps its own indexes. About 65 MB for 1.2 million facts.
CREATE INDEX idx_analytics_facts_views ON analytics_facts (entry_date, item_id, seller_account_id, buyer_account_id, chukta_person_key, bikri_id, sign, qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa, laga_paisa);
