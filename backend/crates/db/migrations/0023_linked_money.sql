-- Migration 23 (plan 76 points 5 and 10, PRD FR-16.4, FR-16.6, FR-17.9 v1.41, Architecture 15.4 v1.26;
-- Umar 14-11-2026, U1 to U3): the link of a separate khata and of a person who is lent money, and the
-- type of that person.
--
--   1. `accounts.link_kind` ('none', 'shop' or 'partner') and `accounts.link_partner_id`: the link of a
--      khata (control 0305) or of a person who is lent money (control 0103). A partner link names the
--      partner. The link belongs to the account, never to one entry, and it changes only while the
--      balance is 0 (the service refuses the rest). Every old account is 'none': it posts as before.
--      `link_since_entry_id` is the newest journal entry when the link was set, so an edit of an older
--      entry can tell that it was made before, or under another, link.
--   2. `accounts.person_type` ('staff', 'garden_owner', 'trip_person', 'other'): the type of a person
--      who is lent money, a label with no effect on any amount. An old person is 'staff' when a staff
--      record names his account, else 'other'. Null on every other account.
--   3. `partners.linked_account_id`: each partner's account for the money that reaches him through a
--      linked khata or loan (control 0605). The Rust hook after this file makes the controls 0307 and
--      0605, the system account 03070001 (the contra of every linked line, in head 03, so outside the
--      equity of FR-17.9) and one 0605 account for every partner that exists.
--
-- No amount, journal entry, journal line, id or chain hash is touched: no account is linked yet, so no
-- posting changes.

ALTER TABLE accounts ADD COLUMN link_kind TEXT NOT NULL DEFAULT 'none'
    CHECK (link_kind IN ('none', 'shop', 'partner'));
ALTER TABLE accounts ADD COLUMN link_partner_id INTEGER REFERENCES partners(id);
-- The newest journal entry when the link was last set (0 for an account that was never linked). An
-- entry with an id up to it was made before the current link: an edit of it is not mirrored when it
-- never was, and is refused when it was mirrored under an older link (the service reads this).
ALTER TABLE accounts ADD COLUMN link_since_entry_id INTEGER NOT NULL DEFAULT 0;
ALTER TABLE accounts ADD COLUMN person_type TEXT
    CHECK (person_type IS NULL OR person_type IN ('staff', 'garden_owner', 'trip_person', 'other'));
CREATE INDEX idx_accounts_link_partner ON accounts (link_partner_id) WHERE link_partner_id IS NOT NULL;

UPDATE accounts SET person_type = CASE
        WHEN EXISTS (SELECT 1 FROM staff s WHERE s.account_id = accounts.id) THEN 'staff'
        ELSE 'other' END
WHERE control_id = (SELECT id FROM account_controls WHERE code = '0103');

CREATE TRIGGER accounts_link_names_a_partner_only_when_partner_linked
BEFORE UPDATE OF link_kind, link_partner_id ON accounts
WHEN (NEW.link_kind = 'partner' AND NEW.link_partner_id IS NULL)
  OR (NEW.link_kind <> 'partner' AND NEW.link_partner_id IS NOT NULL)
BEGIN SELECT RAISE(ABORT, 'ERR_LINK_PARTNER_REQUIRED'); END;

ALTER TABLE partners ADD COLUMN linked_account_id INTEGER REFERENCES accounts(id);
CREATE INDEX idx_partners_linked_account ON partners (linked_account_id);
