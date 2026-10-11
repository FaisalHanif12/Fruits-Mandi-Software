-- 0001_init.sql: the whole database of Architecture Section 15 (plan 03 Section 4.1).
-- Every table is STRICT. Every table has the system columns of Architecture 15.2.
-- Every *_paisa column is a whole rupee: CHECK (x_paisa % 100 = 0) (Code Standards M1, B15).
-- Every *_date column is a real YYYY-MM-DD date. Times (*_at) are UTC text.
-- created_by and updated_by hold a user id but have no foreign key: they would need an
-- index on every table for no gain (B18). A released migration is never edited (B19).

-- Architecture 15.3. Roles: PRD Section 2.
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    username TEXT NOT NULL UNIQUE CHECK (length(trim(username)) > 0),
    password_hash TEXT NOT NULL,
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    role TEXT NOT NULL CHECK (role IN ('owner', 'munshi', 'partner')),
    partner_id INTEGER REFERENCES partners(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    failed_attempts INTEGER NOT NULL DEFAULT 0 CHECK (failed_attempts >= 0),
    locked_until_at TEXT,
    last_login_at TEXT
) STRICT;

-- Architecture 15.3.
CREATE TABLE settings (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    key TEXT NOT NULL UNIQUE CHECK (length(trim(key)) > 0),
    value_json TEXT NOT NULL CHECK (json_valid(value_json))
) STRICT;

-- Architecture 15.3. Append-only (B17). chain_hash is a keyed blake3 chain (NFR-SEC-6).
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    at TEXT NOT NULL,
    user_id INTEGER REFERENCES users(id),
    action TEXT NOT NULL CHECK (length(trim(action)) > 0),
    entity TEXT NOT NULL CHECK (length(trim(entity)) > 0),
    entity_id INTEGER,
    old_json TEXT CHECK (old_json IS NULL OR json_valid(old_json)),
    new_json TEXT CHECK (new_json IS NULL OR json_valid(new_json)),
    reason TEXT,
    chain_hash TEXT NOT NULL CHECK (length(chain_hash) = 64)
) STRICT;

-- Architecture 15.3. One row. closed_up_to_date NULL = no day closed yet (plan 03 4.2c).
CREATE TABLE day_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    closed_up_to_date TEXT CHECK (date(closed_up_to_date) IS closed_up_to_date)
) STRICT;

-- Architecture 15.3.
CREATE TABLE day_closes (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    action TEXT NOT NULL CHECK (action IN ('close', 'reopen')),
    by_user_id INTEGER NOT NULL REFERENCES users(id),
    at TEXT NOT NULL,
    reason TEXT,
    summary_json TEXT CHECK (summary_json IS NULL OR json_valid(summary_json))
) STRICT;

-- Architecture 15.3. month is YYYY-MM.
CREATE TABLE month_closes (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    month TEXT NOT NULL CHECK (month GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]' AND substr(month, 6, 2) BETWEEN '01' AND '12'),
    journal_entry_id INTEGER NOT NULL REFERENCES journal_entries(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    reversed_entry_id INTEGER REFERENCES journal_entries(id)
) STRICT;

-- Architecture 15.3. Unsaved work of a screen (NFR-REL-2).
CREATE TABLE drafts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    screen TEXT NOT NULL CHECK (length(trim(screen)) > 0),
    user_id INTEGER NOT NULL REFERENCES users(id),
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    saved_at TEXT NOT NULL
) STRICT;

-- Architecture 15.3.
CREATE TABLE backup_log (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    at TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('local', 'usb', 'cloud')),
    path_or_key TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0 CHECK (size_bytes >= 0),
    checksum TEXT,
    integrity_ok INTEGER NOT NULL CHECK (integrity_ok IN (0, 1)),
    row_counts_json TEXT CHECK (row_counts_json IS NULL OR json_valid(row_counts_json)),
    error TEXT
) STRICT;

-- Architecture 15.3.
CREATE TABLE user_prefs (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    user_id INTEGER NOT NULL REFERENCES users(id),
    key TEXT NOT NULL CHECK (length(trim(key)) > 0),
    value_json TEXT NOT NULL CHECK (json_valid(value_json)),
    UNIQUE (user_id, key)
) STRICT;

-- Architecture 15.3. Learned pairs only (PRD FR-7.26).
CREATE TABLE translit_pairs (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    roman TEXT NOT NULL CHECK (length(roman) > 0 AND roman = lower(roman)),
    urdu TEXT NOT NULL CHECK (length(urdu) > 0),
    kind TEXT NOT NULL CHECK (kind IN ('name', 'place', 'item', 'word')),
    use_count INTEGER NOT NULL DEFAULT 0 CHECK (use_count >= 0),
    last_used_at TEXT,
    UNIQUE (roman, urdu)
) STRICT;

-- Architecture 15.4. Level 1 of the chart (PRD 10.1).
CREATE TABLE account_heads (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL UNIQUE CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    nature TEXT NOT NULL CHECK (nature IN ('asset', 'payable', 'party', 'income', 'expense', 'capital'))
) STRICT;

-- Architecture 15.4. Level 2 of the chart (PRD 10.1).
CREATE TABLE account_controls (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    head_id INTEGER NOT NULL REFERENCES account_heads(id),
    code TEXT NOT NULL UNIQUE CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    party_type TEXT NOT NULL CHECK (party_type IN ('seller', 'buyer', 'other'))
) STRICT;

-- Architecture 15.4. Only one of opening debit or credit (PRD 10.2). collection_weekday: 0 to 6.
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    control_id INTEGER NOT NULL REFERENCES account_controls(id),
    code TEXT NOT NULL UNIQUE CHECK (length(code) > 0),
    serial INTEGER NOT NULL CHECK (serial BETWEEN 0 AND 9999),
    party_type TEXT NOT NULL CHECK (party_type IN ('seller', 'buyer', 'other')),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    father_or_firm_ur TEXT,
    father_or_firm_en TEXT,
    city_id INTEGER REFERENCES cities(id),
    adda_id INTEGER REFERENCES addas(id),
    phone1 TEXT,
    phone2 TEXT,
    address TEXT,
    linked_account_id INTEGER REFERENCES accounts(id),
    opening_date TEXT CHECK (date(opening_date) IS opening_date),
    opening_dr_paisa INTEGER NOT NULL DEFAULT 0 CHECK (opening_dr_paisa >= 0) CHECK (opening_dr_paisa % 100 = 0),
    opening_cr_paisa INTEGER NOT NULL DEFAULT 0 CHECK (opening_cr_paisa >= 0) CHECK (opening_cr_paisa % 100 = 0),
    opening_is_advance INTEGER NOT NULL DEFAULT 0 CHECK (opening_is_advance IN (0, 1)),
    opening_entry_id INTEGER REFERENCES journal_entries(id),
    credit_allowed INTEGER NOT NULL DEFAULT 1 CHECK (credit_allowed IN (0, 1)),
    credit_limit_paisa INTEGER CHECK (credit_limit_paisa IS NULL OR credit_limit_paisa >= 0) CHECK (credit_limit_paisa % 100 = 0),
    default_marka_id INTEGER REFERENCES markas(id),
    collector_staff_id INTEGER REFERENCES staff(id),
    collection_weekday INTEGER CHECK (collection_weekday IS NULL OR collection_weekday BETWEEN 0 AND 6),
    is_system INTEGER NOT NULL DEFAULT 0 CHECK (is_system IN (0, 1)),
    is_pukhta_khata INTEGER NOT NULL DEFAULT 0 CHECK (is_pukhta_khata IN (0, 1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    note TEXT,
    CHECK (opening_dr_paisa = 0 OR opening_cr_paisa = 0),
    CHECK (opening_is_advance = 0 OR opening_dr_paisa > 0)
) STRICT;

-- Architecture 15.4.
CREATE TABLE cities (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4.
CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    item_group TEXT NOT NULL CHECK (item_group IN ('fruit', 'vegetable')),
    default_packing_id INTEGER REFERENCES packings(id),
    sold_by TEXT NOT NULL DEFAULT 'unit' CHECK (sold_by IN ('unit', 'weight')),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4.
CREATE TABLE packings (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    approx_grams INTEGER CHECK (approx_grams IS NULL OR approx_grams > 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4.
CREATE TABLE addas (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    city_id INTEGER REFERENCES cities(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4. mark_text is kept as typed, one script (PRD FR-7.25).
CREATE TABLE markas (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL CHECK (length(code) > 0),
    mark_text TEXT NOT NULL CHECK (length(trim(mark_text)) > 0),
    owner_account_id INTEGER REFERENCES accounts(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4.
CREATE TABLE stores (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4.
CREATE TABLE narrations (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    text_ur TEXT NOT NULL CHECK (length(trim(text_ur)) > 0),
    text_en TEXT NOT NULL CHECK (length(trim(text_en)) > 0),
    used_in_json TEXT CHECK (used_in_json IS NULL OR json_valid(used_in_json)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4.
CREATE TABLE expense_heads (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    code TEXT NOT NULL CHECK (length(code) > 0),
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4.
CREATE TABLE staff (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    search_ur TEXT NOT NULL,
    search_en TEXT NOT NULL,
    search_phonetic TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('munshi', 'collector', 'labour')),
    phone TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.4. Bank name, title and number are kept as typed (PRD FR-7.25).
CREATE TABLE bank_accounts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    bank_name TEXT NOT NULL CHECK (length(trim(bank_name)) > 0),
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    number TEXT NOT NULL CHECK (length(trim(number)) > 0),
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.5. Allowed combinations: PRD 11.2. account_id NULL = any party, item_id NULL = any item.
CREATE TABLE charge_rules (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    side TEXT NOT NULL CHECK (side IN ('seller', 'buyer')),
    charge_type TEXT NOT NULL CHECK (charge_type IN ('commission', 'laga', 'labour', 'munshiana', 'phone')),
    account_id INTEGER REFERENCES accounts(id),
    item_id INTEGER REFERENCES items(id),
    item_group TEXT CHECK (item_group IN ('fruit', 'vegetable')),
    method TEXT NOT NULL CHECK (method IN ('percent', 'per_unit', 'per_lot')),
    value_bps INTEGER CHECK (value_bps IS NULL OR value_bps BETWEEN 0 AND 10000),
    value_paisa INTEGER CHECK (value_paisa IS NULL OR value_paisa >= 0) CHECK (value_paisa % 100 = 0),
    from_date TEXT NOT NULL CHECK (date(from_date) IS from_date),
    to_date TEXT CHECK (date(to_date) IS to_date),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    note TEXT,
    CHECK (item_id IS NULL OR item_group IS NULL),
    CHECK (to_date IS NULL OR to_date >= from_date),
    CHECK ((method = 'percent' AND value_bps IS NOT NULL AND value_paisa IS NULL)
        OR (method IN ('per_unit', 'per_lot') AND value_paisa IS NOT NULL AND value_bps IS NULL)),
    CHECK ((side = 'seller' AND charge_type = 'commission' AND method IN ('percent', 'per_unit', 'per_lot'))
        OR (side = 'seller' AND charge_type IN ('labour', 'munshiana', 'phone') AND method IN ('per_unit', 'per_lot'))
        OR (side = 'buyer' AND charge_type IN ('commission', 'laga') AND method IN ('percent', 'per_unit')))
) STRICT;

-- Architecture 15.5. The partner created first has the lowest id (PRD C10).
CREATE TABLE partners (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    name_ur TEXT NOT NULL CHECK (length(trim(name_ur)) > 0),
    name_en TEXT NOT NULL CHECK (length(trim(name_en)) > 0),
    user_id INTEGER REFERENCES users(id),
    capital_account_id INTEGER NOT NULL REFERENCES accounts(id),
    drawings_account_id INTEGER NOT NULL REFERENCES accounts(id),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1))
) STRICT;

-- Architecture 15.5. Share history (PRD FR-17.2).
CREATE TABLE partner_shares (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    partner_id INTEGER NOT NULL REFERENCES partners(id),
    share_bps INTEGER NOT NULL CHECK (share_bps BETWEEN 0 AND 10000),
    effective_from TEXT NOT NULL CHECK (date(effective_from) IS effective_from)
) STRICT;

-- Architecture 15.5. Append-only (B17). amount_paisa is signed; a part is never more than the whole (PRD C10, C11).
CREATE TABLE partner_share_rows (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    journal_entry_id INTEGER NOT NULL REFERENCES journal_entries(id),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    partner_id INTEGER NOT NULL REFERENCES partners(id),
    kind TEXT NOT NULL CHECK (kind IN ('income', 'expense')),
    base_paisa INTEGER NOT NULL CHECK (base_paisa % 100 = 0),
    share_bps INTEGER NOT NULL CHECK (share_bps BETWEEN 0 AND 10000),
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa % 100 = 0),
    CHECK (abs(amount_paisa) <= abs(base_paisa))
) STRICT;

-- Architecture 15.6. A cancel needs a reason (PRD FR-12.11).
CREATE TABLE bikri_entries (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    number INTEGER NOT NULL UNIQUE CHECK (number > 0),
    book TEXT,
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    bill_date TEXT NOT NULL CHECK (date(bill_date) IS bill_date),
    truck_no TEXT,
    zamindar_account_id INTEGER REFERENCES accounts(id),
    adda_id INTEGER REFERENCES addas(id),
    driver_name TEXT,
    driver_phone TEXT,
    total_freight_paisa INTEGER CHECK (total_freight_paisa IS NULL OR total_freight_paisa >= 0) CHECK (total_freight_paisa % 100 = 0),
    store_id INTEGER REFERENCES stores(id),
    note TEXT,
    status TEXT NOT NULL DEFAULT 'saved' CHECK (status IN ('saved', 'cancelled')),
    cancel_reason TEXT,
    cancelled_by INTEGER REFERENCES users(id),
    cancelled_at TEXT,
    print_count INTEGER NOT NULL DEFAULT 0 CHECK (print_count >= 0),
    CHECK (status <> 'cancelled' OR (cancel_reason IS NOT NULL AND length(trim(cancel_reason)) > 0))
) STRICT;

-- Architecture 15.6. Sold plus written off can never be more than received (PRD FR-12.6). net_paisa can be negative (PRD FR-12.7).
CREATE TABLE bikri_lots (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    bikri_id INTEGER NOT NULL REFERENCES bikri_entries(id),
    lot_no INTEGER NOT NULL CHECK (lot_no > 0),
    seller_account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_name_ur TEXT,
    pukhta_name_en TEXT,
    pukhta_phone TEXT,
    pukhta_person_key TEXT,
    item_id INTEGER NOT NULL REFERENCES items(id),
    packing_id INTEGER NOT NULL REFERENCES packings(id),
    marka_id INTEGER REFERENCES markas(id),
    store_id INTEGER REFERENCES stores(id),
    qty_received INTEGER NOT NULL CHECK (qty_received > 0),
    weight_received_grams INTEGER CHECK (weight_received_grams IS NULL OR weight_received_grams > 0),
    freight_paisa INTEGER NOT NULL DEFAULT 0 CHECK (freight_paisa >= 0) CHECK (freight_paisa % 100 = 0),
    freight_paid_by TEXT NOT NULL DEFAULT 'firm' CHECK (freight_paid_by IN ('firm', 'seller')),
    labour_paisa INTEGER NOT NULL DEFAULT 0 CHECK (labour_paisa >= 0) CHECK (labour_paisa % 100 = 0),
    labour_rule_id INTEGER REFERENCES charge_rules(id),
    munshiana_paisa INTEGER NOT NULL DEFAULT 0 CHECK (munshiana_paisa >= 0) CHECK (munshiana_paisa % 100 = 0),
    munshiana_rule_id INTEGER REFERENCES charge_rules(id),
    phone_paisa INTEGER NOT NULL DEFAULT 0 CHECK (phone_paisa >= 0) CHECK (phone_paisa % 100 = 0),
    phone_rule_id INTEGER REFERENCES charge_rules(id),
    cash_paid_paisa INTEGER NOT NULL DEFAULT 0 CHECK (cash_paid_paisa >= 0) CHECK (cash_paid_paisa % 100 = 0),
    store_rent_paisa INTEGER NOT NULL DEFAULT 0 CHECK (store_rent_paisa >= 0) CHECK (store_rent_paisa % 100 = 0),
    first_entry_id INTEGER REFERENCES journal_entries(id),
    qty_sold INTEGER NOT NULL DEFAULT 0 CHECK (qty_sold >= 0),
    qty_written_off INTEGER NOT NULL DEFAULT 0 CHECK (qty_written_off >= 0),
    gross_paisa INTEGER NOT NULL DEFAULT 0 CHECK (gross_paisa >= 0) CHECK (gross_paisa % 100 = 0),
    seller_comm_paisa INTEGER NOT NULL DEFAULT 0 CHECK (seller_comm_paisa >= 0) CHECK (seller_comm_paisa % 100 = 0),
    deductions_paisa INTEGER NOT NULL DEFAULT 0 CHECK (deductions_paisa >= 0) CHECK (deductions_paisa % 100 = 0),
    net_paisa INTEGER NOT NULL DEFAULT 0 CHECK (net_paisa % 100 = 0),
    UNIQUE (bikri_id, lot_no),
    CHECK (qty_sold + qty_written_off <= qty_received),
    CHECK (net_paisa = gross_paisa - deductions_paisa)
) STRICT;

-- Architecture 15.6. Each line keeps the rule snapshot (PRD FR-11.6). Method 'manual' is the hand-typed value of PRD FR-11.10. rate_diff_paisa can be negative.
CREATE TABLE bikri_sales (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    bikri_id INTEGER NOT NULL REFERENCES bikri_entries(id),
    lot_id INTEGER NOT NULL REFERENCES bikri_lots(id),
    line_no INTEGER NOT NULL CHECK (line_no > 0),
    journal_entry_id INTEGER NOT NULL REFERENCES journal_entries(id),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    buyer_account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_name_ur TEXT,
    pukhta_name_en TEXT,
    pukhta_phone TEXT,
    pukhta_person_key TEXT,
    qty INTEGER NOT NULL CHECK (qty > 0),
    weight_grams INTEGER CHECK (weight_grams IS NULL OR weight_grams > 0),
    seller_rate_paisa INTEGER NOT NULL CHECK (seller_rate_paisa > 0) CHECK (seller_rate_paisa % 100 = 0),
    seller_amount_paisa INTEGER NOT NULL CHECK (seller_amount_paisa >= 0) CHECK (seller_amount_paisa % 100 = 0),
    seller_comm_method TEXT NOT NULL CHECK (seller_comm_method IN ('percent', 'per_unit', 'per_lot', 'manual')),
    seller_comm_value_bps INTEGER CHECK (seller_comm_value_bps IS NULL OR seller_comm_value_bps BETWEEN 0 AND 10000),
    seller_comm_value_paisa INTEGER CHECK (seller_comm_value_paisa IS NULL OR seller_comm_value_paisa >= 0) CHECK (seller_comm_value_paisa % 100 = 0),
    seller_comm_rule_id INTEGER REFERENCES charge_rules(id),
    seller_comm_paisa INTEGER NOT NULL CHECK (seller_comm_paisa >= 0) CHECK (seller_comm_paisa % 100 = 0),
    buyer_rate_paisa INTEGER NOT NULL CHECK (buyer_rate_paisa > 0) CHECK (buyer_rate_paisa % 100 = 0),
    buyer_value_paisa INTEGER NOT NULL CHECK (buyer_value_paisa >= 0) CHECK (buyer_value_paisa % 100 = 0),
    buyer_comm_method TEXT NOT NULL CHECK (buyer_comm_method IN ('percent', 'per_unit', 'manual')),
    buyer_comm_value_bps INTEGER CHECK (buyer_comm_value_bps IS NULL OR buyer_comm_value_bps BETWEEN 0 AND 10000),
    buyer_comm_value_paisa INTEGER CHECK (buyer_comm_value_paisa IS NULL OR buyer_comm_value_paisa >= 0) CHECK (buyer_comm_value_paisa % 100 = 0),
    buyer_comm_rule_id INTEGER REFERENCES charge_rules(id),
    buyer_comm_paisa INTEGER NOT NULL CHECK (buyer_comm_paisa >= 0) CHECK (buyer_comm_paisa % 100 = 0),
    laga_method TEXT NOT NULL CHECK (laga_method IN ('percent', 'per_unit', 'manual')),
    laga_value_bps INTEGER CHECK (laga_value_bps IS NULL OR laga_value_bps BETWEEN 0 AND 10000),
    laga_value_paisa INTEGER CHECK (laga_value_paisa IS NULL OR laga_value_paisa >= 0) CHECK (laga_value_paisa % 100 = 0),
    laga_rule_id INTEGER REFERENCES charge_rules(id),
    laga_paisa INTEGER NOT NULL CHECK (laga_paisa >= 0) CHECK (laga_paisa % 100 = 0),
    buyer_total_paisa INTEGER NOT NULL CHECK (buyer_total_paisa >= 0) CHECK (buyer_total_paisa % 100 = 0),
    rate_diff_paisa INTEGER NOT NULL DEFAULT 0 CHECK (rate_diff_paisa % 100 = 0),
    payment_type TEXT NOT NULL CHECK (payment_type IN ('credit', 'cash')),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    replaced_by_sale_id INTEGER REFERENCES bikri_sales(id),
    note TEXT,
    CHECK ((seller_comm_method = 'percent' AND seller_comm_value_bps IS NOT NULL AND seller_comm_value_paisa IS NULL)
        OR (seller_comm_method IN ('per_unit', 'per_lot') AND seller_comm_value_paisa IS NOT NULL AND seller_comm_value_bps IS NULL)
        OR seller_comm_method = 'manual'),
    CHECK ((buyer_comm_method = 'percent' AND buyer_comm_value_bps IS NOT NULL AND buyer_comm_value_paisa IS NULL)
        OR (buyer_comm_method IN ('per_unit', 'per_lot') AND buyer_comm_value_paisa IS NOT NULL AND buyer_comm_value_bps IS NULL)
        OR buyer_comm_method = 'manual'),
    CHECK ((laga_method = 'percent' AND laga_value_bps IS NOT NULL AND laga_value_paisa IS NULL)
        OR (laga_method IN ('per_unit', 'per_lot') AND laga_value_paisa IS NOT NULL AND laga_value_bps IS NULL)
        OR laga_method = 'manual'),
    CHECK (buyer_total_paisa = buyer_value_paisa + buyer_comm_paisa + laga_paisa),
    CHECK (rate_diff_paisa = buyer_value_paisa - seller_amount_paisa),
    CHECK (replaced_by_sale_id IS NULL OR is_active = 0)
) STRICT;

-- Architecture 15.6. Append-only (B17). qty is signed. journal_entry_id is NULL for write-offs.
CREATE TABLE stock_moves (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    lot_id INTEGER NOT NULL REFERENCES bikri_lots(id),
    journal_entry_id INTEGER REFERENCES journal_entries(id),
    move_date TEXT NOT NULL CHECK (date(move_date) IS move_date),
    kind TEXT NOT NULL CHECK (kind IN ('received', 'sold', 'written_off', 'reversal')),
    qty INTEGER NOT NULL CHECK (qty <> 0),
    reason TEXT
) STRICT;

-- Architecture 15.6. Append-only (B17). Totals are SUM(sign * value). sign is -1 on reversal rows.
CREATE TABLE analytics_facts (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    journal_entry_id INTEGER NOT NULL REFERENCES journal_entries(id),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    year INTEGER NOT NULL,
    month INTEGER NOT NULL CHECK (month BETWEEN 1 AND 12),
    bikri_id INTEGER NOT NULL REFERENCES bikri_entries(id),
    lot_id INTEGER NOT NULL REFERENCES bikri_lots(id),
    sale_id INTEGER NOT NULL REFERENCES bikri_sales(id),
    item_id INTEGER NOT NULL REFERENCES items(id),
    seller_account_id INTEGER NOT NULL REFERENCES accounts(id),
    buyer_account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_person_key TEXT,
    city_id INTEGER REFERENCES cities(id),
    adda_id INTEGER REFERENCES addas(id),
    qty INTEGER NOT NULL CHECK (qty > 0),
    seller_amount_paisa INTEGER NOT NULL CHECK (seller_amount_paisa >= 0) CHECK (seller_amount_paisa % 100 = 0),
    seller_comm_paisa INTEGER NOT NULL CHECK (seller_comm_paisa >= 0) CHECK (seller_comm_paisa % 100 = 0),
    buyer_value_paisa INTEGER NOT NULL CHECK (buyer_value_paisa >= 0) CHECK (buyer_value_paisa % 100 = 0),
    buyer_comm_paisa INTEGER NOT NULL CHECK (buyer_comm_paisa >= 0) CHECK (buyer_comm_paisa % 100 = 0),
    laga_paisa INTEGER NOT NULL CHECK (laga_paisa >= 0) CHECK (laga_paisa % 100 = 0),
    sign INTEGER NOT NULL CHECK (sign IN (1, -1)),
    CHECK (year = CAST(substr(entry_date, 1, 4) AS INTEGER) AND month = CAST(substr(entry_date, 6, 2) AS INTEGER))
) STRICT;

-- Architecture 15.7. Bank and cheque rules: PRD 15.1. A cancel needs a reason.
CREATE TABLE vouchers (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    type TEXT NOT NULL CHECK (type IN ('receipt', 'payment', 'advance', 'shop_expense', 'personal_expense', 'general')),
    number INTEGER NOT NULL CHECK (number > 0),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    account_id INTEGER REFERENCES accounts(id),
    pukhta_person_key TEXT,
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa > 0) CHECK (amount_paisa % 100 = 0),
    mode TEXT NOT NULL DEFAULT 'cash' CHECK (mode IN ('cash', 'bank', 'cheque')),
    bank_account_id INTEGER REFERENCES bank_accounts(id),
    cheque_no TEXT,
    cheque_date TEXT CHECK (date(cheque_date) IS cheque_date),
    collector_staff_id INTEGER REFERENCES staff(id),
    partner_id INTEGER REFERENCES partners(id),
    expense_head_id INTEGER REFERENCES expense_heads(id),
    expected_goods TEXT,
    narration TEXT,
    status TEXT NOT NULL DEFAULT 'saved' CHECK (status IN ('saved', 'cancelled')),
    cancel_reason TEXT,
    current_entry_id INTEGER REFERENCES journal_entries(id),
    UNIQUE (type, number),
    CHECK (mode = 'cash' OR bank_account_id IS NOT NULL),
    CHECK (mode <> 'cheque' OR (cheque_no IS NOT NULL AND cheque_date IS NOT NULL)),
    CHECK (status <> 'cancelled' OR (cancel_reason IS NOT NULL AND length(trim(cancel_reason)) > 0))
) STRICT;

-- Architecture 15.7. Draft to posted only; the trigger checks the balance (B16). total_paisa must equal the debit total when posted.
CREATE TABLE journal_entries (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    posting_type TEXT NOT NULL CHECK (posting_type IN ('bikri_first', 'bikri_later', 'reversal', 'receipt', 'payment', 'advance', 'shop_expense', 'personal_expense', 'general', 'opening', 'month_close')),
    source_type TEXT NOT NULL CHECK (source_type IN ('bikri', 'voucher', 'account', 'month_close')),
    source_id INTEGER NOT NULL,
    reverses_entry_id INTEGER REFERENCES journal_entries(id),
    narration TEXT,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'posted')),
    total_paisa INTEGER NOT NULL DEFAULT 0 CHECK (total_paisa >= 0) CHECK (total_paisa % 100 = 0),
    chain_hash TEXT CHECK (chain_hash IS NULL OR length(chain_hash) = 64),
    CHECK ((posting_type = 'reversal') = (reverses_entry_id IS NOT NULL)),
    CHECK (total_paisa <= 100000000000000)
) STRICT;

-- Architecture 15.7. Append-only (B17). Both sides 0 or more, exactly one above 0 (PRD FR-6.2). An advance is a debit.
CREATE TABLE journal_lines (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    entry_id INTEGER NOT NULL REFERENCES journal_entries(id),
    line_no INTEGER NOT NULL CHECK (line_no > 0),
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_person_key TEXT,
    dr_paisa INTEGER NOT NULL DEFAULT 0 CHECK (dr_paisa % 100 = 0),
    cr_paisa INTEGER NOT NULL DEFAULT 0 CHECK (cr_paisa % 100 = 0),
    is_advance INTEGER NOT NULL DEFAULT 0 CHECK (is_advance IN (0, 1)),
    narration TEXT,
    UNIQUE (entry_id, line_no),
    CHECK (dr_paisa >= 0 AND cr_paisa >= 0 AND ((dr_paisa > 0) <> (cr_paisa > 0))),
    CHECK (dr_paisa <= 100000000000000 AND cr_paisa <= 100000000000000),
    CHECK (is_advance = 0 OR dr_paisa > 0)
) STRICT;

-- Architecture 15.7. Never deleted: an undo sets is_active = 0 with who and when (B17).
CREATE TABLE bill_allocations (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    bill_line_id INTEGER NOT NULL REFERENCES journal_lines(id),
    payment_line_id INTEGER NOT NULL REFERENCES journal_lines(id),
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_person_key TEXT,
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa > 0) CHECK (amount_paisa % 100 = 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    undone_at TEXT,
    undone_by INTEGER REFERENCES users(id),
    CHECK ((is_active = 1 AND undone_at IS NULL AND undone_by IS NULL) OR (is_active = 0 AND undone_at IS NOT NULL AND undone_by IS NOT NULL))
) STRICT;

-- Architecture 15.7. Never deleted: an undo sets is_active = 0 with who and when (B17).
CREATE TABLE advance_allocations (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    advance_line_id INTEGER NOT NULL REFERENCES journal_lines(id),
    credit_line_id INTEGER NOT NULL REFERENCES journal_lines(id),
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_person_key TEXT,
    amount_paisa INTEGER NOT NULL CHECK (amount_paisa > 0) CHECK (amount_paisa % 100 = 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    undone_at TEXT,
    undone_by INTEGER REFERENCES users(id),
    CHECK ((is_active = 1 AND undone_at IS NULL AND undone_by IS NULL) OR (is_active = 0 AND undone_at IS NOT NULL AND undone_by IS NOT NULL))
) STRICT;

-- Indexes of Architecture 15.8, then one index for every other foreign key (B18).
CREATE INDEX idx_journal_lines_account_entry ON journal_lines (account_id, entry_id);
CREATE INDEX idx_journal_lines_account_person ON journal_lines (account_id, pukhta_person_key);
CREATE INDEX idx_journal_entries_entry_date ON journal_entries (entry_date);
CREATE INDEX idx_journal_entries_source ON journal_entries (source_type, source_id);
-- An entry can be reversed only once, so "entry + reversal = 0" always holds (PRD FR-6.3).
CREATE UNIQUE INDEX idx_journal_entries_reverses ON journal_entries (reverses_entry_id) WHERE reverses_entry_id IS NOT NULL;
CREATE INDEX idx_bikri_sales_lot ON bikri_sales (lot_id);
CREATE INDEX idx_bikri_sales_buyer_date ON bikri_sales (buyer_account_id, entry_date);
CREATE INDEX idx_bikri_sales_entry ON bikri_sales (journal_entry_id);
CREATE INDEX idx_bikri_sales_bikri ON bikri_sales (bikri_id);
CREATE INDEX idx_bikri_sales_seller_comm_rule ON bikri_sales (seller_comm_rule_id);
CREATE INDEX idx_bikri_sales_buyer_comm_rule ON bikri_sales (buyer_comm_rule_id);
CREATE INDEX idx_bikri_sales_laga_rule ON bikri_sales (laga_rule_id);
CREATE INDEX idx_bikri_sales_replaced_by ON bikri_sales (replaced_by_sale_id);
CREATE INDEX idx_bikri_lots_bikri ON bikri_lots (bikri_id);
CREATE INDEX idx_bikri_lots_seller ON bikri_lots (seller_account_id, id);
CREATE INDEX idx_bikri_lots_item_packing ON bikri_lots (item_id, packing_id);
CREATE INDEX idx_bikri_lots_qty ON bikri_lots (qty_received, qty_sold, qty_written_off);
CREATE INDEX idx_bikri_lots_packing ON bikri_lots (packing_id);
CREATE INDEX idx_bikri_lots_marka ON bikri_lots (marka_id);
CREATE INDEX idx_bikri_lots_store ON bikri_lots (store_id);
CREATE INDEX idx_bikri_lots_labour_rule ON bikri_lots (labour_rule_id);
CREATE INDEX idx_bikri_lots_munshiana_rule ON bikri_lots (munshiana_rule_id);
CREATE INDEX idx_bikri_lots_phone_rule ON bikri_lots (phone_rule_id);
CREATE INDEX idx_bikri_lots_first_entry ON bikri_lots (first_entry_id);
CREATE INDEX idx_analytics_facts_date ON analytics_facts (entry_date);
CREATE INDEX idx_analytics_facts_item_date ON analytics_facts (item_id, entry_date);
CREATE INDEX idx_analytics_facts_seller_date ON analytics_facts (seller_account_id, entry_date);
CREATE INDEX idx_analytics_facts_buyer_date ON analytics_facts (buyer_account_id, entry_date);
CREATE INDEX idx_analytics_facts_entry ON analytics_facts (journal_entry_id);
CREATE INDEX idx_analytics_facts_bikri ON analytics_facts (bikri_id);
CREATE INDEX idx_analytics_facts_lot ON analytics_facts (lot_id);
CREATE INDEX idx_analytics_facts_sale ON analytics_facts (sale_id);
CREATE INDEX idx_analytics_facts_city ON analytics_facts (city_id);
CREATE INDEX idx_analytics_facts_adda ON analytics_facts (adda_id);
CREATE INDEX idx_partner_share_rows_partner_date ON partner_share_rows (partner_id, entry_date);
CREATE INDEX idx_partner_share_rows_entry ON partner_share_rows (journal_entry_id);
CREATE INDEX idx_bill_allocations_account ON bill_allocations (account_id, pukhta_person_key, is_active);
CREATE INDEX idx_bill_allocations_bill ON bill_allocations (bill_line_id);
CREATE INDEX idx_bill_allocations_payment ON bill_allocations (payment_line_id);
CREATE INDEX idx_bill_allocations_undone_by ON bill_allocations (undone_by);
CREATE INDEX idx_advance_allocations_account ON advance_allocations (account_id, pukhta_person_key, is_active);
CREATE INDEX idx_advance_allocations_advance ON advance_allocations (advance_line_id);
CREATE INDEX idx_advance_allocations_credit ON advance_allocations (credit_line_id);
CREATE INDEX idx_advance_allocations_undone_by ON advance_allocations (undone_by);
CREATE INDEX idx_bikri_entries_truck_date ON bikri_entries (truck_no, entry_date);
CREATE INDEX idx_bikri_entries_date ON bikri_entries (entry_date);
CREATE INDEX idx_bikri_entries_zamindar ON bikri_entries (zamindar_account_id);
CREATE INDEX idx_bikri_entries_adda ON bikri_entries (adda_id);
CREATE INDEX idx_bikri_entries_store ON bikri_entries (store_id);
CREATE INDEX idx_bikri_entries_cancelled_by ON bikri_entries (cancelled_by);
CREATE INDEX idx_accounts_type_serial ON accounts (party_type, serial);
CREATE INDEX idx_accounts_control ON accounts (control_id);
CREATE INDEX idx_accounts_city ON accounts (city_id);
CREATE INDEX idx_accounts_adda ON accounts (adda_id);
CREATE INDEX idx_accounts_linked ON accounts (linked_account_id);
CREATE INDEX idx_accounts_opening_entry ON accounts (opening_entry_id);
CREATE INDEX idx_accounts_marka ON accounts (default_marka_id);
CREATE INDEX idx_accounts_collector ON accounts (collector_staff_id);
CREATE INDEX idx_charge_rules_lookup ON charge_rules (side, charge_type, account_id, item_id, item_group, from_date);
CREATE INDEX idx_charge_rules_account ON charge_rules (account_id);
CREATE INDEX idx_charge_rules_item ON charge_rules (item_id);
CREATE INDEX idx_account_controls_head ON account_controls (head_id);
CREATE INDEX idx_items_packing ON items (default_packing_id);
CREATE INDEX idx_addas_city ON addas (city_id);
CREATE INDEX idx_markas_code ON markas (code);
CREATE INDEX idx_markas_owner ON markas (owner_account_id);
CREATE INDEX idx_expense_heads_account ON expense_heads (account_id);
CREATE INDEX idx_bank_accounts_account ON bank_accounts (account_id);
CREATE INDEX idx_partners_user ON partners (user_id);
CREATE INDEX idx_partners_capital ON partners (capital_account_id);
CREATE INDEX idx_partners_drawings ON partners (drawings_account_id);
CREATE INDEX idx_partner_shares_partner ON partner_shares (partner_id, effective_from);
CREATE INDEX idx_stock_moves_lot ON stock_moves (lot_id, move_date);
CREATE INDEX idx_stock_moves_entry ON stock_moves (journal_entry_id);
CREATE INDEX idx_vouchers_date ON vouchers (entry_date);
CREATE INDEX idx_vouchers_account ON vouchers (account_id);
CREATE INDEX idx_vouchers_bank ON vouchers (bank_account_id);
CREATE INDEX idx_vouchers_collector ON vouchers (collector_staff_id);
CREATE INDEX idx_vouchers_partner ON vouchers (partner_id);
CREATE INDEX idx_vouchers_expense_head ON vouchers (expense_head_id);
CREATE INDEX idx_vouchers_current_entry ON vouchers (current_entry_id);
CREATE INDEX idx_users_partner ON users (partner_id);
CREATE INDEX idx_audit_log_entity ON audit_log (entity, entity_id);
CREATE INDEX idx_audit_log_user ON audit_log (user_id);
CREATE INDEX idx_audit_log_at ON audit_log (at);
CREATE INDEX idx_day_closes_user ON day_closes (by_user_id);
CREATE INDEX idx_day_closes_date ON day_closes (entry_date);
CREATE INDEX idx_month_closes_entry ON month_closes (journal_entry_id);
CREATE INDEX idx_month_closes_reversed ON month_closes (reversed_entry_id);
CREATE INDEX idx_drafts_user_screen ON drafts (user_id, screen);
CREATE INDEX idx_accounts_search_ur ON accounts (search_ur);
CREATE INDEX idx_accounts_search_en ON accounts (search_en);
CREATE INDEX idx_accounts_search_phonetic ON accounts (search_phonetic);
CREATE INDEX idx_cities_search_ur ON cities (search_ur);
CREATE INDEX idx_cities_search_en ON cities (search_en);
CREATE INDEX idx_cities_search_phonetic ON cities (search_phonetic);
CREATE INDEX idx_cities_code ON cities (code);
CREATE INDEX idx_items_search_ur ON items (search_ur);
CREATE INDEX idx_items_search_en ON items (search_en);
CREATE INDEX idx_items_search_phonetic ON items (search_phonetic);
CREATE INDEX idx_items_code ON items (code);
CREATE INDEX idx_packings_search_ur ON packings (search_ur);
CREATE INDEX idx_packings_search_en ON packings (search_en);
CREATE INDEX idx_packings_search_phonetic ON packings (search_phonetic);
CREATE INDEX idx_packings_code ON packings (code);
CREATE INDEX idx_addas_search_ur ON addas (search_ur);
CREATE INDEX idx_addas_search_en ON addas (search_en);
CREATE INDEX idx_addas_search_phonetic ON addas (search_phonetic);
CREATE INDEX idx_addas_code ON addas (code);
CREATE INDEX idx_stores_search_ur ON stores (search_ur);
CREATE INDEX idx_stores_search_en ON stores (search_en);
CREATE INDEX idx_stores_search_phonetic ON stores (search_phonetic);
CREATE INDEX idx_stores_code ON stores (code);
CREATE INDEX idx_expense_heads_search_ur ON expense_heads (search_ur);
CREATE INDEX idx_expense_heads_search_en ON expense_heads (search_en);
CREATE INDEX idx_expense_heads_search_phonetic ON expense_heads (search_phonetic);
CREATE INDEX idx_expense_heads_code ON expense_heads (code);
CREATE INDEX idx_staff_search_ur ON staff (search_ur);
CREATE INDEX idx_staff_search_en ON staff (search_en);
CREATE INDEX idx_staff_search_phonetic ON staff (search_phonetic);

-- Triggers (Code Standards B16 and B17, PRD FR-6.2 and FR-6.3). Each message is an ERR_ code
-- that mandi-db turns into a DbError. Connections turn recursive_triggers ON, so an
-- INSERT OR REPLACE cannot delete a row without firing the delete triggers.

-- An entry is born a draft. It becomes posted only through the balance check below (B16).
CREATE TRIGGER journal_entries_insert_is_draft BEFORE INSERT ON journal_entries
WHEN NEW.status <> 'draft'
BEGIN SELECT RAISE(ABORT, 'ERR_JOURNAL_POSTED'); END;

-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER journal_entries_no_replace BEFORE INSERT ON journal_entries
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM journal_entries WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_JOURNAL_POSTED'); END;

-- A posted journal entry never changes (PRD FR-6.3).
CREATE TRIGGER journal_entries_posted_is_final BEFORE UPDATE ON journal_entries
WHEN OLD.status = 'posted'
BEGIN SELECT RAISE(ABORT, 'ERR_JOURNAL_POSTED'); END;

-- Draft to posted only if the entry has lines, the debits equal the credits, and total_paisa
-- equals the debit total (PRD FR-6.2).
CREATE TRIGGER journal_entries_balance_on_post BEFORE UPDATE OF status ON journal_entries
WHEN OLD.status = 'draft' AND NEW.status = 'posted'
BEGIN
  SELECT RAISE(ABORT, 'ERR_JOURNAL_UNBALANCED')
  WHERE (SELECT COUNT(*) FROM journal_lines WHERE entry_id = OLD.id) = 0
     OR (SELECT SUM(dr_paisa) FROM journal_lines WHERE entry_id = OLD.id)
        <> (SELECT SUM(cr_paisa) FROM journal_lines WHERE entry_id = OLD.id)
     OR NEW.total_paisa <> (SELECT SUM(dr_paisa) FROM journal_lines WHERE entry_id = OLD.id);
END;

-- A reversal is the exact mirror of a posted entry: same total, and every account and person
-- key nets to zero over the two entries (PRD FR-6.3, FR-12.19; the date rule stays in the
-- service because the month-close reversal date is not fixed by the PRD).
CREATE TRIGGER journal_entries_reversal_mirrors BEFORE UPDATE OF status ON journal_entries
WHEN OLD.status = 'draft' AND NEW.status = 'posted' AND NEW.reverses_entry_id IS NOT NULL
BEGIN
  SELECT RAISE(ABORT, 'ERR_JOURNAL_REVERSAL')
  WHERE NOT EXISTS (SELECT 1 FROM journal_entries o
                    WHERE o.id = NEW.reverses_entry_id AND o.status = 'posted' AND o.total_paisa = NEW.total_paisa)
     OR EXISTS (SELECT 1 FROM (SELECT account_id, pukhta_person_key AS person, SUM(dr_paisa - cr_paisa) AS net
                               FROM journal_lines WHERE entry_id IN (OLD.id, NEW.reverses_entry_id)
                               GROUP BY account_id, person)
                WHERE net <> 0);
END;
CREATE TRIGGER journal_entries_no_replace_reversal BEFORE INSERT ON journal_entries
WHEN NEW.reverses_entry_id IS NOT NULL AND EXISTS (SELECT 1 FROM journal_entries WHERE reverses_entry_id = NEW.reverses_entry_id)
BEGIN SELECT RAISE(ABORT, 'ERR_JOURNAL_REVERSAL'); END;

CREATE TRIGGER journal_entries_no_delete BEFORE DELETE ON journal_entries
BEGIN SELECT RAISE(ABORT, 'ERR_JOURNAL_POSTED'); END;

-- No line can be added to a posted entry.
CREATE TRIGGER journal_lines_no_insert_into_posted BEFORE INSERT ON journal_lines
WHEN (SELECT status FROM journal_entries WHERE id = NEW.entry_id) = 'posted'
BEGIN SELECT RAISE(ABORT, 'ERR_JOURNAL_POSTED'); END;

-- A line number is used once in an entry, also for INSERT OR REPLACE.
CREATE TRIGGER journal_lines_no_replace_number BEFORE INSERT ON journal_lines
WHEN EXISTS (SELECT 1 FROM journal_lines WHERE entry_id = NEW.entry_id AND line_no = NEW.line_no)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- Append-only tables (B17).
-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER journal_lines_no_replace BEFORE INSERT ON journal_lines
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM journal_lines WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER journal_lines_no_update BEFORE UPDATE ON journal_lines
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER journal_lines_no_delete BEFORE DELETE ON journal_lines
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER audit_log_no_replace BEFORE INSERT ON audit_log
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM audit_log WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER audit_log_no_update BEFORE UPDATE ON audit_log
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER audit_log_no_delete BEFORE DELETE ON audit_log
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER partner_share_rows_no_replace BEFORE INSERT ON partner_share_rows
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM partner_share_rows WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER partner_share_rows_no_update BEFORE UPDATE ON partner_share_rows
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER partner_share_rows_no_delete BEFORE DELETE ON partner_share_rows
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER stock_moves_no_replace BEFORE INSERT ON stock_moves
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM stock_moves WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER stock_moves_no_update BEFORE UPDATE ON stock_moves
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER stock_moves_no_delete BEFORE DELETE ON stock_moves
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER analytics_facts_no_replace BEFORE INSERT ON analytics_facts
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM analytics_facts WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER analytics_facts_no_update BEFORE UPDATE ON analytics_facts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER analytics_facts_no_delete BEFORE DELETE ON analytics_facts
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- Matching tables are never deleted (B17).
-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER bill_allocations_no_replace BEFORE INSERT ON bill_allocations
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM bill_allocations WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER bill_allocations_no_delete BEFORE DELETE ON bill_allocations
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
-- Only an undo is allowed: is_active goes from 1 to 0, with who and when. Nothing else changes.
CREATE TRIGGER bill_allocations_only_undo BEFORE UPDATE ON bill_allocations
WHEN NEW.id IS NOT OLD.id OR NEW.bill_line_id IS NOT OLD.bill_line_id OR NEW.payment_line_id IS NOT OLD.payment_line_id OR NEW.account_id IS NOT OLD.account_id OR NEW.pukhta_person_key IS NOT OLD.pukhta_person_key OR NEW.amount_paisa IS NOT OLD.amount_paisa
  OR (OLD.is_active = 0 AND (NEW.is_active IS NOT 0 OR NEW.undone_at IS NOT OLD.undone_at OR NEW.undone_by IS NOT OLD.undone_by))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
-- INSERT OR REPLACE of an existing id deletes the old row; refuse it even when
-- recursive_triggers is off (for example in the sqlite3 shell).
CREATE TRIGGER advance_allocations_no_replace BEFORE INSERT ON advance_allocations
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM advance_allocations WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER advance_allocations_no_delete BEFORE DELETE ON advance_allocations
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
-- Only an undo is allowed: is_active goes from 1 to 0, with who and when. Nothing else changes.
CREATE TRIGGER advance_allocations_only_undo BEFORE UPDATE ON advance_allocations
WHEN NEW.id IS NOT OLD.id OR NEW.advance_line_id IS NOT OLD.advance_line_id OR NEW.credit_line_id IS NOT OLD.credit_line_id OR NEW.account_id IS NOT OLD.account_id OR NEW.pukhta_person_key IS NOT OLD.pukhta_person_key OR NEW.amount_paisa IS NOT OLD.amount_paisa
  OR (OLD.is_active = 0 AND (NEW.is_active IS NOT 0 OR NEW.undone_at IS NOT OLD.undone_at OR NEW.undone_by IS NOT OLD.undone_by))
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- A matching row points at lines of the right direction on its own account and person key
-- (Architecture 15.7). The allocated amount against the line amount is checked by the service.
CREATE TRIGGER bill_allocations_lines_match BEFORE INSERT ON bill_allocations
WHEN NOT EXISTS (SELECT 1 FROM journal_lines b WHERE b.id = NEW.bill_line_id AND b.dr_paisa > 0
                   AND b.account_id = NEW.account_id AND b.pukhta_person_key IS NEW.pukhta_person_key)
  OR NOT EXISTS (SELECT 1 FROM journal_lines p WHERE p.id = NEW.payment_line_id AND p.cr_paisa > 0
                   AND p.account_id = NEW.account_id AND p.pukhta_person_key IS NEW.pukhta_person_key)
BEGIN SELECT RAISE(ABORT, 'ERR_ALLOCATION_LINES'); END;
CREATE TRIGGER advance_allocations_lines_match BEFORE INSERT ON advance_allocations
WHEN NOT EXISTS (SELECT 1 FROM journal_lines a WHERE a.id = NEW.advance_line_id AND a.dr_paisa > 0 AND a.is_advance = 1
                   AND a.account_id = NEW.account_id AND a.pukhta_person_key IS NEW.pukhta_person_key)
  OR NOT EXISTS (SELECT 1 FROM journal_lines c WHERE c.id = NEW.credit_line_id AND c.cr_paisa > 0
                   AND c.account_id = NEW.account_id AND c.pukhta_person_key IS NEW.pukhta_person_key)
BEGIN SELECT RAISE(ABORT, 'ERR_ALLOCATION_LINES'); END;

-- The active allocations of a line never add up to more than the line (B-1 of the money review).
CREATE TRIGGER bill_allocations_amount_fits BEFORE INSERT ON bill_allocations
WHEN (SELECT COALESCE(SUM(amount_paisa), 0) FROM bill_allocations WHERE bill_line_id = NEW.bill_line_id AND is_active = 1) + NEW.amount_paisa
       > (SELECT dr_paisa FROM journal_lines WHERE id = NEW.bill_line_id)
  OR (SELECT COALESCE(SUM(amount_paisa), 0) FROM bill_allocations WHERE payment_line_id = NEW.payment_line_id AND is_active = 1) + NEW.amount_paisa
       > (SELECT cr_paisa FROM journal_lines WHERE id = NEW.payment_line_id)
BEGIN SELECT RAISE(ABORT, 'ERR_ALLOCATION_AMOUNT'); END;
CREATE TRIGGER advance_allocations_amount_fits BEFORE INSERT ON advance_allocations
WHEN (SELECT COALESCE(SUM(amount_paisa), 0) FROM advance_allocations WHERE advance_line_id = NEW.advance_line_id AND is_active = 1) + NEW.amount_paisa
       > (SELECT dr_paisa FROM journal_lines WHERE id = NEW.advance_line_id)
  OR (SELECT COALESCE(SUM(amount_paisa), 0) FROM advance_allocations WHERE credit_line_id = NEW.credit_line_id AND is_active = 1) + NEW.amount_paisa
       > (SELECT cr_paisa FROM journal_lines WHERE id = NEW.credit_line_id)
BEGIN SELECT RAISE(ABORT, 'ERR_ALLOCATION_AMOUNT'); END;

-- A saved sale line keeps its stored values (PRD FR-11.6). An edit replaces the line: the old
-- one only gets is_active = 0 and replaced_by_sale_id (PRD FR-12.19).
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
   OR (OLD.is_active = 0 AND NEW.is_active = 1)
   OR (OLD.replaced_by_sale_id IS NOT NULL AND NEW.replaced_by_sale_id IS NOT OLD.replaced_by_sale_id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

-- System accounts cannot be deleted and their codes cannot change (PRD FR-10.4).
CREATE TRIGGER accounts_system_no_delete BEFORE DELETE ON accounts
WHEN OLD.is_system = 1
BEGIN SELECT RAISE(ABORT, 'ERR_SYSTEM_ACCOUNT'); END;
CREATE TRIGGER accounts_system_fixed BEFORE UPDATE ON accounts
WHEN OLD.is_system = 1 AND (NEW.code IS NOT OLD.code OR NEW.is_system IS NOT 1)
BEGIN SELECT RAISE(ABORT, 'ERR_SYSTEM_ACCOUNT'); END;

-- day_state keeps its one row.
CREATE TRIGGER day_state_no_delete BEFORE DELETE ON day_state
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
INSERT INTO day_state (id, closed_up_to_date) VALUES (1, NULL);
