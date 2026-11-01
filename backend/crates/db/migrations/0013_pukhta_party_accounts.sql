-- Migration 13 (plan 47, PRD v1.15 FR-13.11, E81, A79; Architecture 15.4): a پختہ party's own account.
--
-- Until now `accounts.is_pukhta_khata = 1` has meant one thing everywhere: "the shared پختہ account,
-- where the person is typed on the line and kept under a person key" (accounts 0030 and 0031). A
-- rare party the shop wants to keep track of now gets his OWN account (پختہ کھاتہ گاہک or
-- پختہ کھاتہ بیوپاری, code 0032 on). He is a full buyer or seller account: his own ledger, balance,
-- matching and party card, and no person key. So he must NOT carry the old mark; he gets a new one.
--
-- What this file does:
--   1. `accounts.is_pukhta_party`: 1 for such an account, 0 for every other. It is a mark, not a
--      new party type: `party_type` stays `seller` or `buyer` and the control stays 03011 or 0304,
--      so every rule that works on a buyer or a seller works on him.
--   2. A CHECK that keeps the two marks apart: a party is never the shared account, never a system
--      account, and always a seller or a buyer.
--   3. A trigger that stops the mark being changed after the account is made (its code, its ledger
--      and its postings belong to the series).
--   4. A small partial index for the پختہ list filter.
--
-- Nothing else changes. Every existing row gets 0: no code, amount, journal line, id or chain hash
-- is touched, and the shared accounts 0030 and 0031 keep `is_pukhta_khata = 1`.

ALTER TABLE accounts ADD COLUMN is_pukhta_party INTEGER NOT NULL DEFAULT 0
    CHECK (is_pukhta_party IN (0, 1))
    CHECK (is_pukhta_party = 0 OR (is_pukhta_khata = 0 AND is_system = 0 AND party_type IN ('seller', 'buyer')));

CREATE TRIGGER accounts_pukhta_party_fixed BEFORE UPDATE OF is_pukhta_party ON accounts
WHEN NEW.is_pukhta_party IS NOT OLD.is_pukhta_party
BEGIN SELECT RAISE(ABORT, 'ERR_FORMAT_INVALID'); END;

CREATE INDEX idx_accounts_pukhta_party ON accounts (code) WHERE is_pukhta_party = 1;
