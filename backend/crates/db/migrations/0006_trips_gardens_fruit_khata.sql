-- Migration 6 (plan 16b, PRD v1.7): the one migration of D4. It holds every database change of the
-- market trips (PRD 12.10), the gardens (12.11), the fruit taken (16.7), the partner capital (17.4),
-- the separate khata (16.6) and the tag after a name (10.2), so plans 17, D4R and D5 need no other
-- migration. The migration was never released: it replaces the earlier text of plan 16b (the firm's
-- own goods, v1.3) in place. A data file that already ran that text must be made again.
--
-- What it adds, in this file:
--   * the tables `ventures`, `venture_partners`, `venture_entries`, `venture_entry_parts`,
--     `purchases`, `purchase_cost_parts`, `lot_credit_parts` and `fruit_takes` (Architecture 15.5 and
--     15.6) with their checks, indexes and triggers;
--   * ADD COLUMN on existing tables, so no row is touched and no number changes:
--     `bikri_lots.purchase_id` and `qty_taken`, `bikri_sales.cost_paisa`,
--     `stock_moves.purchase_id` and `cost_paisa`, `analytics_facts.purchase_id`, `venture_id` and
--     `cost_paisa`, `partners.venture_account_id`, `accounts.tag`;
--   * the trigger `stock_moves_reversal_mirrors` again, now right for a move that has no lot and for
--     the cost it moves, and `bikri_sales_snapshot_is_final` again with `cost_paisa` frozen.
--
-- What the Rust hook of this migration does after this file (see `migrate.rs`, `trips_hook`):
--   * widens the CHECK lists of six tables in place (`widen_checks`): the posting types and source
--     types of `journal_entries`, the types of `vouchers`, the party types of `account_controls` and
--     `accounts` ('separate', 'garden'), the kinds of `stock_moves` ('taken') and `stock_moves.lot_id`
--     (a purchase move has no lot). SQLite cannot change a CHECK with ALTER TABLE. The documented way
--     to relax a CHECK (https://www.sqlite.org/lang_altertable.html, "Making Other Kinds Of Table
--     Schema Changes") is to edit the table text in `sqlite_schema` inside the transaction and raise
--     the schema version. That keeps every row, id, index, trigger and the keyed `chain_hash` of every
--     journal entry byte for byte as they were (a copy-and-rename of `journal_entries` would have to
--     recreate all of them), and the hook checks that each edit changed the text exactly once;
--   * seeds the controls 0305 (separate khata), 0306 (gardens) and 0604 (gardens, trips and fruit)
--     and the account 030119998 (Own Goods (Trips)), renames the English name of 01010001 to "Cash",
--     and makes one 0604 account for every partner that exists.
--
-- Differences from the data dictionary, all additions the code needs (reported to the lead):
--   * `venture_entries.cost_paisa` and `stock_moves.cost_paisa`: the cost of trip goods moved by a
--     sale in another shop, a write-off and their reversals (PRD C21). It is stored, never posted, and
--     never worked out again, so the undo of a write-off gives back exactly what it took.
--   * the table `purchase_cost_parts`: how each trip cost was spread over the purchases (PRD C24), so
--     a trip cost can be cancelled and each purchase's cost left stays exact;
--   * the table `lot_credit_parts`: each trip partner's part of the seller credit of a trip lot (PRD
--     C25), because the journal lines carry no lot.
--
-- Nothing is deleted. Posted rows are never touched (PRD FR-6.3).

-- Architecture 15.5. A garden or a trip (PRD 12.10, 12.11). A trip's names are made from its number
-- and date by the backend. The rent and the account belong to a garden. A garden is closed by the
-- Owner; a trip closes by itself when it has a purchase and nothing is left (PRD FR-12.33).
CREATE TABLE ventures (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    kind TEXT NOT NULL CHECK (kind IN ('garden', 'trip')),
    number INTEGER NOT NULL CHECK (number > 0),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    item_id INTEGER REFERENCES items(id),
    rent_paisa INTEGER NOT NULL DEFAULT 0 CHECK (rent_paisa >= 0) CHECK (rent_paisa % 100 = 0),
    season_end_date TEXT CHECK (date(season_end_date) IS season_end_date),
    market TEXT,
    sent_person TEXT,
    account_id INTEGER REFERENCES accounts(id),
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    closed_date TEXT CHECK (date(closed_date) IS closed_date),
    close_entry_id INTEGER REFERENCES journal_entries(id),
    note TEXT,
    UNIQUE (kind, number),
    -- Only a garden has a rent, a season end and an account (PRD 12.11); a trip has none.
    CHECK (kind = 'garden' OR (rent_paisa = 0 AND season_end_date IS NULL AND account_id IS NULL)),
    CHECK (kind = 'trip' OR account_id IS NOT NULL),
    CHECK ((status = 'closed') = (closed_date IS NOT NULL))
) STRICT;

CREATE INDEX idx_ventures_kind_status ON ventures (kind, status);
CREATE INDEX idx_ventures_account ON ventures (account_id);
CREATE INDEX idx_ventures_item ON ventures (item_id);
CREATE INDEX idx_ventures_close_entry ON ventures (close_entry_id);
CREATE INDEX idx_ventures_date ON ventures (kind, entry_date);
CREATE INDEX idx_ventures_search_ur ON ventures (search_ur);
CREATE INDEX idx_ventures_search_en ON ventures (search_en);
CREATE INDEX idx_ventures_search_phonetic ON ventures (search_phonetic);

-- A venture is never deleted (Code Standards D3). The identity of a venture never changes; the
-- status with its closing, the version and the note can.
CREATE TRIGGER ventures_no_delete BEFORE DELETE ON ventures
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER ventures_identity_is_final BEFORE UPDATE ON ventures
WHEN NEW.id IS NOT OLD.id
   OR NEW.kind IS NOT OLD.kind
   OR NEW.number IS NOT OLD.number
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.rent_paisa IS NOT OLD.rent_paisa
   OR NEW.account_id IS NOT OLD.account_id
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- Architecture 15.5. The partners in a venture, each with his share and where his money comes from
-- (PRD FR-12.29). The rows of one venture total 10,000 bps (the service checks it: a row-by-row
-- trigger cannot). A partner who is not in the venture has no row.
CREATE TABLE venture_partners (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    venture_id INTEGER NOT NULL REFERENCES ventures(id),
    partner_id INTEGER NOT NULL REFERENCES partners(id),
    share_bps INTEGER NOT NULL CHECK (share_bps BETWEEN 1 AND 10000),
    source TEXT NOT NULL CHECK (source IN ('shop', 'pocket')),
    UNIQUE (venture_id, partner_id)
) STRICT;

CREATE INDEX idx_venture_partners_partner ON venture_partners (partner_id);

-- The shares of a venture are fixed after its first entry (PRD FR-12.29, ERR_VENTURE_SHARES_LOCKED).
-- The source of a partner can change for one entry (it is saved on that entry's parts), and this table
-- keeps the default for the next entry, so only `share_bps` and the rows themselves are locked.
CREATE TRIGGER venture_partners_shares_locked_insert BEFORE INSERT ON venture_partners
WHEN EXISTS (SELECT 1 FROM venture_entries e WHERE e.venture_id = NEW.venture_id)
BEGIN SELECT RAISE(ABORT, 'ERR_VENTURE_SHARES_LOCKED'); END;
CREATE TRIGGER venture_partners_shares_locked_update BEFORE UPDATE ON venture_partners
WHEN (NEW.share_bps IS NOT OLD.share_bps OR NEW.partner_id IS NOT OLD.partner_id
      OR NEW.venture_id IS NOT OLD.venture_id)
  AND EXISTS (SELECT 1 FROM venture_entries e WHERE e.venture_id = OLD.venture_id)
BEGIN SELECT RAISE(ABORT, 'ERR_VENTURE_SHARES_LOCKED'); END;
CREATE TRIGGER venture_partners_shares_locked_delete BEFORE DELETE ON venture_partners
WHEN EXISTS (SELECT 1 FROM venture_entries e WHERE e.venture_id = OLD.venture_id)
BEGIN SELECT RAISE(ABORT, 'ERR_VENTURE_SHARES_LOCKED'); END;

-- Architecture 15.6, PRD 12.10. One row per purchase of a trip. qty_sold, qty_written_off and
-- cost_released_paisa are kept up to date in the same save as the sale or write-off that changes them
-- (PRD FR-12.25, C21). trip_cost_paisa is the part of the trip costs spread over it (C24) and
-- total_cost_paisa is paid now plus that part. There is no supplier account, no credit and no bank:
-- a purchase is paid at once (PRD FR-12.24, A53), and the supplier is only text.
CREATE TABLE purchases (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    number INTEGER NOT NULL UNIQUE CHECK (number > 0),
    venture_id INTEGER NOT NULL REFERENCES ventures(id),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    supplier_text TEXT,
    item_id INTEGER NOT NULL REFERENCES items(id),
    packing_id INTEGER NOT NULL REFERENCES packings(id),
    qty INTEGER NOT NULL CHECK (qty > 0),
    weight_grams INTEGER CHECK (weight_grams IS NULL OR weight_grams > 0),
    rate_paisa INTEGER NOT NULL CHECK (rate_paisa > 0) CHECK (rate_paisa % 100 = 0),
    value_paisa INTEGER NOT NULL CHECK (value_paisa > 0) CHECK (value_paisa % 100 = 0),
    freight_paisa INTEGER NOT NULL DEFAULT 0 CHECK (freight_paisa >= 0) CHECK (freight_paisa % 100 = 0),
    labour_paisa INTEGER NOT NULL DEFAULT 0 CHECK (labour_paisa >= 0) CHECK (labour_paisa % 100 = 0),
    other_cost_paisa INTEGER NOT NULL DEFAULT 0 CHECK (other_cost_paisa >= 0) CHECK (other_cost_paisa % 100 = 0),
    paid_now_paisa INTEGER NOT NULL CHECK (paid_now_paisa > 0) CHECK (paid_now_paisa % 100 = 0),
    trip_cost_paisa INTEGER NOT NULL DEFAULT 0 CHECK (trip_cost_paisa >= 0) CHECK (trip_cost_paisa % 100 = 0),
    total_cost_paisa INTEGER NOT NULL CHECK (total_cost_paisa > 0) CHECK (total_cost_paisa % 100 = 0),
    journal_entry_id INTEGER REFERENCES journal_entries(id),
    status TEXT NOT NULL DEFAULT 'saved' CHECK (status IN ('saved', 'cancelled')),
    cancel_reason TEXT,
    note TEXT,
    qty_sold INTEGER NOT NULL DEFAULT 0 CHECK (qty_sold >= 0),
    qty_written_off INTEGER NOT NULL DEFAULT 0 CHECK (qty_written_off >= 0),
    cost_released_paisa INTEGER NOT NULL DEFAULT 0 CHECK (cost_released_paisa >= 0) CHECK (cost_released_paisa % 100 = 0),
    -- PRD FR-12.24: paid now is the value plus what was paid at once for freight, labour and the rest.
    CHECK (paid_now_paisa = value_paisa + freight_paisa + labour_paisa + other_cost_paisa),
    -- PRD C24: the total cost is paid now plus the part of the trip costs.
    CHECK (total_cost_paisa = paid_now_paisa + trip_cost_paisa),
    -- PRD C1: by نگ the value is qty x rate; a weight item is rounded from the kilograms.
    CHECK (weight_grams IS NOT NULL OR value_paisa = qty * rate_paisa),
    -- Sold plus written off is never more than bought, and the cost released is never more than the
    -- cost (PRD C21, Code Standards 9.3).
    CHECK (qty_sold + qty_written_off <= qty),
    CHECK (cost_released_paisa <= total_cost_paisa),
    CHECK (status <> 'cancelled' OR (cancel_reason IS NOT NULL AND length(trim(cancel_reason)) > 0))
) STRICT;

CREATE INDEX idx_purchases_venture ON purchases (venture_id, entry_date);
CREATE INDEX idx_purchases_date ON purchases (entry_date);
CREATE INDEX idx_purchases_item ON purchases (item_id);
CREATE INDEX idx_purchases_packing ON purchases (packing_id);
CREATE INDEX idx_purchases_entry ON purchases (journal_entry_id);
CREATE INDEX idx_purchases_qty ON purchases (qty, qty_sold, qty_written_off);

-- A purchase is never deleted (Code Standards D3). A cancel sets status and reason.
CREATE TRIGGER purchases_no_delete BEFORE DELETE ON purchases
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- What was bought and paid never changes (PRD FR-6.3). Only the running figures (sold, written off,
-- cost released), the part of the trip costs with the total it makes, the status with its reason, the
-- entry (set once, right after posting) and the system columns can change.
CREATE TRIGGER purchases_record_is_final BEFORE UPDATE ON purchases
WHEN NEW.id IS NOT OLD.id
   OR NEW.number IS NOT OLD.number
   OR NEW.venture_id IS NOT OLD.venture_id
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.supplier_text IS NOT OLD.supplier_text
   OR NEW.item_id IS NOT OLD.item_id
   OR NEW.packing_id IS NOT OLD.packing_id
   OR NEW.qty IS NOT OLD.qty
   OR NEW.weight_grams IS NOT OLD.weight_grams
   OR NEW.rate_paisa IS NOT OLD.rate_paisa
   OR NEW.value_paisa IS NOT OLD.value_paisa
   OR NEW.freight_paisa IS NOT OLD.freight_paisa
   OR NEW.labour_paisa IS NOT OLD.labour_paisa
   OR NEW.other_cost_paisa IS NOT OLD.other_cost_paisa
   OR NEW.paid_now_paisa IS NOT OLD.paid_now_paisa
   OR (OLD.journal_entry_id IS NOT NULL AND NEW.journal_entry_id IS NOT OLD.journal_entry_id)
   OR (OLD.status = 'cancelled' AND NEW.status <> 'cancelled')
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- Architecture 15.5. Payments, sales in another shop and closings of a venture. `kind` purchase,
-- cost, other_sale belong to a trip; installment, cost, other_sale, close to a garden. A trip
-- purchase has one row here (with the partner parts) and one in `purchases`; both carry the same
-- journal entry (Architecture 15.7). `cost_paisa` is the cost of the goods sold in another shop
-- (PRD C21), stored and never posted. Nothing is edited: a cancel sets the status.
CREATE TABLE venture_entries (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    venture_id INTEGER NOT NULL REFERENCES ventures(id),
    kind TEXT NOT NULL CHECK (kind IN ('installment', 'cost', 'purchase', 'other_sale', 'close')),
    number INTEGER NOT NULL CHECK (number > 0),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa >= 0) CHECK (amount_paisa % 100 = 0),
    reason TEXT,
    purchase_id INTEGER REFERENCES purchases(id),
    qty INTEGER CHECK (qty IS NULL OR qty > 0),
    shop_name TEXT,
    cost_paisa INTEGER NOT NULL DEFAULT 0 CHECK (cost_paisa >= 0) CHECK (cost_paisa % 100 = 0),
    journal_entry_id INTEGER REFERENCES journal_entries(id),
    status TEXT NOT NULL DEFAULT 'saved' CHECK (status IN ('saved', 'cancelled')),
    cancel_reason TEXT,
    current_entry_id INTEGER REFERENCES journal_entries(id),
    UNIQUE (kind, number),
    -- Every payment and every sale brings money; only a garden's closing can be 0 (PRD E64).
    CHECK (kind = 'close' OR amount_paisa > 0),
    CHECK (kind <> 'purchase' OR purchase_id IS NOT NULL),
    CHECK (kind <> 'other_sale' OR qty IS NOT NULL),
    -- A payment of a garden or a trip says why it was made (PRD FR-12.31, FR-12.35).
    CHECK (kind NOT IN ('installment', 'cost') OR (reason IS NOT NULL AND length(trim(reason)) > 0)),
    CHECK (status <> 'cancelled' OR (cancel_reason IS NOT NULL AND length(trim(cancel_reason)) > 0))
) STRICT;

CREATE INDEX idx_venture_entries_venture ON venture_entries (venture_id, entry_date);
CREATE INDEX idx_venture_entries_entry ON venture_entries (journal_entry_id);
CREATE INDEX idx_venture_entries_purchase ON venture_entries (purchase_id);
CREATE INDEX idx_venture_entries_current_entry ON venture_entries (current_entry_id);

CREATE TRIGGER venture_entries_no_delete BEFORE DELETE ON venture_entries
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER venture_entries_record_is_final BEFORE UPDATE ON venture_entries
WHEN NEW.id IS NOT OLD.id
   OR NEW.venture_id IS NOT OLD.venture_id
   OR NEW.kind IS NOT OLD.kind
   OR NEW.number IS NOT OLD.number
   OR NEW.entry_date IS NOT OLD.entry_date
   OR NEW.amount_paisa IS NOT OLD.amount_paisa
   OR NEW.reason IS NOT OLD.reason
   OR NEW.purchase_id IS NOT OLD.purchase_id
   OR NEW.qty IS NOT OLD.qty
   OR NEW.shop_name IS NOT OLD.shop_name
   OR NEW.cost_paisa IS NOT OLD.cost_paisa
   OR (OLD.journal_entry_id IS NOT NULL AND NEW.journal_entry_id IS NOT OLD.journal_entry_id)
   OR (OLD.status = 'cancelled' AND NEW.status <> 'cancelled')
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- Architecture 15.5. One part per partner of an entry: his % of the payment (C23), of the sales money
-- (C25) or of a garden's closing (C26), and where it came from. The parts add up to the entry's amount
-- (the service checks it). A pocket part has no journal line. Signed only for a closing that is a loss.
CREATE TABLE venture_entry_parts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entry_id INTEGER NOT NULL REFERENCES venture_entries(id),
    partner_id INTEGER NOT NULL REFERENCES partners(id),
    share_bps INTEGER NOT NULL CHECK (share_bps BETWEEN 1 AND 10000),
    source TEXT NOT NULL CHECK (source IN ('shop', 'pocket')),
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa % 100 = 0),
    UNIQUE (entry_id, partner_id)
) STRICT;

CREATE INDEX idx_venture_entry_parts_partner ON venture_entry_parts (partner_id);

CREATE TRIGGER venture_entry_parts_no_update BEFORE UPDATE ON venture_entry_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER venture_entry_parts_no_delete BEFORE DELETE ON venture_entry_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- How a trip cost was spread over the purchases that still had goods left (PRD FR-12.31, C24). It
-- lets a trip cost be cancelled and keeps each purchase's trip_cost_paisa exact. Append-only.
CREATE TABLE purchase_cost_parts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entry_id INTEGER NOT NULL REFERENCES venture_entries(id),
    purchase_id INTEGER NOT NULL REFERENCES purchases(id),
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa >= 0) CHECK (amount_paisa % 100 = 0),
    UNIQUE (entry_id, purchase_id)
) STRICT;

CREATE INDEX idx_purchase_cost_parts_purchase ON purchase_cost_parts (purchase_id);

CREATE TRIGGER purchase_cost_parts_no_update BEFORE UPDATE ON purchase_cost_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER purchase_cost_parts_no_delete BEFORE DELETE ON purchase_cost_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- The seller credit of a trip lot goes to the trip's partners in the same entry as the sale (PRD
-- FR-12.26, C25). The journal lines carry no lot, so each partner's part is also stored here, one row
-- for each (posting, lot, partner), in the same save. An edit or a cancel reverses the posting and
-- writes the opposite rows for its reversal entry, so the sum of a lot's rows is what stands now. Money
-- back, sales money and the result of a trip are read from these rows (PRD FR-12.27) and never worked
-- out again.
CREATE TABLE lot_credit_parts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    lot_id INTEGER NOT NULL REFERENCES bikri_lots(id),
    journal_entry_id INTEGER NOT NULL REFERENCES journal_entries(id),
    partner_id INTEGER NOT NULL REFERENCES partners(id),
    share_bps INTEGER NOT NULL CHECK (share_bps BETWEEN 1 AND 10000),
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa % 100 = 0),
    UNIQUE (journal_entry_id, lot_id, partner_id)
) STRICT;

CREATE INDEX idx_lot_credit_parts_lot ON lot_credit_parts (lot_id);
CREATE INDEX idx_lot_credit_parts_partner ON lot_credit_parts (partner_id);

CREATE TRIGGER lot_credit_parts_no_update BEFORE UPDATE ON lot_credit_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER lot_credit_parts_no_delete BEFORE DELETE ON lot_credit_parts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- Architecture 15.6, PRD 16.7. Fruit taken by a partner (partner_id) or for the shop's guests (null).
-- Built in D4R plan 17d; the table is here so that plan needs no migration. The credit goes to the
-- lot's seller or to the other agent's account (PRD FR-16.11).
CREATE TABLE fruit_takes (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    number INTEGER NOT NULL UNIQUE CHECK (number > 0),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    partner_id INTEGER REFERENCES partners(id),
    source TEXT NOT NULL CHECK (source IN ('shop', 'other_shop')),
    lot_id INTEGER REFERENCES bikri_lots(id),
    credit_account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_person_key TEXT,
    item_id INTEGER NOT NULL REFERENCES items(id),
    packing_id INTEGER NOT NULL REFERENCES packings(id),
    qty INTEGER NOT NULL CHECK (qty > 0),
    weight_grams INTEGER CHECK (weight_grams IS NULL OR weight_grams > 0),
    rate_paisa INTEGER NOT NULL CHECK (rate_paisa > 0) CHECK (rate_paisa % 100 = 0),
    value_paisa INTEGER NOT NULL CHECK (value_paisa > 0) CHECK (value_paisa % 100 = 0),
    note TEXT,
    journal_entry_id INTEGER REFERENCES journal_entries(id),
    status TEXT NOT NULL DEFAULT 'saved' CHECK (status IN ('saved', 'cancelled')),
    cancel_reason TEXT,
    current_entry_id INTEGER REFERENCES journal_entries(id),
    CHECK (source <> 'shop' OR lot_id IS NOT NULL),
    CHECK (status <> 'cancelled' OR (cancel_reason IS NOT NULL AND length(trim(cancel_reason)) > 0))
) STRICT;

CREATE INDEX idx_fruit_takes_date ON fruit_takes (entry_date);
CREATE INDEX idx_fruit_takes_partner ON fruit_takes (partner_id, entry_date);
CREATE INDEX idx_fruit_takes_lot ON fruit_takes (lot_id);
CREATE INDEX idx_fruit_takes_account ON fruit_takes (credit_account_id);
CREATE INDEX idx_fruit_takes_item ON fruit_takes (item_id);
CREATE INDEX idx_fruit_takes_packing ON fruit_takes (packing_id);
CREATE INDEX idx_fruit_takes_entry ON fruit_takes (journal_entry_id);
CREATE INDEX idx_fruit_takes_current_entry ON fruit_takes (current_entry_id);

CREATE TRIGGER fruit_takes_no_delete BEFORE DELETE ON fruit_takes
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- Trip goods in the lots, the sale lines, the stock moves and the facts (Architecture 15.6).
ALTER TABLE bikri_lots ADD COLUMN purchase_id INTEGER REFERENCES purchases(id);
CREATE INDEX idx_bikri_lots_purchase ON bikri_lots (purchase_id);

-- Fruit taken from a lot (PRD FR-16.11, C16): received - sold - written off - taken is the balance.
ALTER TABLE bikri_lots ADD COLUMN qty_taken INTEGER NOT NULL DEFAULT 0
    CHECK (qty_taken >= 0)
    CHECK (qty_sold + qty_written_off + qty_taken <= qty_received);

-- The cost of the trip goods sold on this line (PRD C21), stored at save and never worked out again.
-- It is shown with the result and never posted (PRD FR-12.26).
ALTER TABLE bikri_sales ADD COLUMN cost_paisa INTEGER NOT NULL DEFAULT 0
    CHECK (cost_paisa >= 0) CHECK (cost_paisa % 100 = 0);

-- A purchase move has no lot; a sale of an own lot names both its lot and its purchase. The `lot_id`
-- NOT NULL is removed by the hook; the trigger below keeps "a lot or a purchase" true. `cost_paisa`
-- is the cost of trip goods this move releases (positive for a write-off or a sale in another shop,
-- negative for the reversal of one); a lot move has 0.
ALTER TABLE stock_moves ADD COLUMN purchase_id INTEGER REFERENCES purchases(id);
ALTER TABLE stock_moves ADD COLUMN cost_paisa INTEGER NOT NULL DEFAULT 0 CHECK (cost_paisa % 100 = 0);
CREATE INDEX idx_stock_moves_purchase ON stock_moves (purchase_id, move_date);

ALTER TABLE analytics_facts ADD COLUMN purchase_id INTEGER REFERENCES purchases(id);
ALTER TABLE analytics_facts ADD COLUMN venture_id INTEGER REFERENCES ventures(id);
ALTER TABLE analytics_facts ADD COLUMN cost_paisa INTEGER NOT NULL DEFAULT 0
    CHECK (cost_paisa >= 0) CHECK (cost_paisa % 100 = 0);
CREATE INDEX idx_analytics_facts_purchase ON analytics_facts (purchase_id);
CREATE INDEX idx_analytics_facts_venture ON analytics_facts (venture_id);

-- Each partner's garden, trip and fruit account under control 0604 (PRD FR-10.4, C22). The hook makes
-- one for every partner that exists; the partner save makes it for each new partner.
ALTER TABLE partners ADD COLUMN venture_account_id INTEGER REFERENCES accounts(id);
CREATE INDEX idx_partners_venture_account ON partners (venture_account_id);

-- The tag after a name (PRD 10.2, v1.7): 1 to 4 English capital letters and dots, null for none. The
-- service validates and capitalises it; D4R plan 17e builds its use and the search keys.
ALTER TABLE accounts ADD COLUMN tag TEXT
    CHECK (tag IS NULL OR (length(tag) BETWEEN 1 AND 4 AND tag NOT GLOB '*[^A-Z.]*' AND tag GLOB '*[A-Z]*'));

CREATE TRIGGER stock_moves_has_an_owner BEFORE INSERT ON stock_moves
WHEN NEW.lot_id IS NULL AND NEW.purchase_id IS NULL
BEGIN SELECT RAISE(ABORT, 'ERR_STOCK_MOVE_OWNER'); END;

-- A reversal move is the exact opposite of the move it names: the same lot and purchase (either may
-- be empty), the opposite quantity and cost, and kind 'reversal'. Only a reversal names a move.
-- (Migration 5 compared lot ids with `=`, which is not true for two empty lot ids.)
DROP TRIGGER stock_moves_reversal_mirrors;
CREATE TRIGGER stock_moves_reversal_mirrors BEFORE INSERT ON stock_moves
WHEN NEW.reverses_move_id IS NOT NULL
  AND (NEW.kind <> 'reversal'
       OR NOT EXISTS (SELECT 1 FROM stock_moves o
                      WHERE o.id = NEW.reverses_move_id AND o.lot_id IS NEW.lot_id
                        AND o.purchase_id IS NEW.purchase_id AND o.qty = -NEW.qty
                        AND o.cost_paisa = -NEW.cost_paisa
                        AND o.kind <> 'reversal'))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- The cost of the trip goods a line sold is part of its stored snapshot too (PRD FR-12.26, C21): the
-- trigger of migration 1 again, with `cost_paisa` among the frozen columns.
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
   OR (OLD.is_active = 0 AND NEW.is_active = 1)
   OR (OLD.replaced_by_sale_id IS NOT NULL AND NEW.replaced_by_sale_id IS NOT OLD.replaced_by_sale_id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
