-- Migration 25 (plan 78 D2, PRD FR-12.11 v1.43, Architecture 15.5): a garden can have several items.
--
-- `venture_items` lists the items of a garden (its "main fruits"). A row is switched off when the
-- Owner takes an item out of the list, never deleted (Code Standards D3). `ventures.item_id` stays
-- and holds the FIRST item, so everything that reads it keeps working. A goods-arrived line still
-- picks its own item; the list only says what the garden grows.
--
-- It also adds a covering index for the goods filter of the commission page (plan 78 D4): the day,
-- the venture and the seller with the commission sums, so a goods filter over years is read from the
-- index alone (NFR-SPD-24). Indexes only; no row, amount, id or chain hash changes.
--
-- Rent relief (plan 78 D3, PRD FR-12.51): `ventures.rent_original_paisa` keeps the rent a garden was made with
-- once the Owner has changed it (null until then), so the relief is shown as original less now. A new
-- garden entry kind `refund` (رعایت واپسی: money the owner pays back) is allowed by the hook of this
-- migration, which widens the CHECK of `venture_entries.kind`.
--
-- This migration adds one table, a column and two indexes, and fills the table from the gardens that have an
-- item.
CREATE TABLE venture_items (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    venture_id INTEGER NOT NULL REFERENCES ventures(id),
    item_id INTEGER NOT NULL REFERENCES items(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    UNIQUE (venture_id, item_id)
) STRICT;

CREATE INDEX idx_venture_items_item ON venture_items (item_id);

CREATE INDEX idx_analytics_facts_goods_sums ON analytics_facts
    (entry_date, venture_id, seller_account_id, sign, seller_comm_paisa, buyer_comm_paisa, laga_paisa);

INSERT INTO venture_items (created_by, updated_by, venture_id, item_id)
SELECT NULL, NULL, id, item_id FROM ventures WHERE kind = 'garden' AND item_id IS NOT NULL ORDER BY id;

ALTER TABLE ventures ADD COLUMN rent_original_paisa INTEGER
    CHECK (rent_original_paisa % 100 = 0);
