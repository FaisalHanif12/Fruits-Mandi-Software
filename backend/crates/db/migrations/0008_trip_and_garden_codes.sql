-- Migration 8 (plan 34, PRD 1.8): easy codes for market trips and gardens.
--
-- A trip's number is a code of type 007 (the first trip is 0070, then 0071 ...) and a garden's number
-- is a code of type 008 (the first garden is 0080, then 0081 ...). The garden's account under control
-- 0306 gets the same code as its garden, so the garden and its account read the same. The trip and
-- garden numbers themselves (1, 2, 3 ...) do not change; the code is the type number plus the number
-- less one, with no padding, and the code is worked out when it is shown (`mandi_core::codes`).
--
-- What this file does:
--   1. Renumbers the account of every garden: code `008` + (number - 1), serial (number - 1).
--   2. The Rust hook of this migration (see `migrate.rs`, `trip_names_hook`) then gives every trip
--      its new name, the code and the day ("0070 10-10-2026" in both scripts), with its search keys.
--      A trip's names are made by the app from its number and its date; nobody types them.
--
-- Nothing else changes. No amount, id, journal line, balance or chain hash is touched. Gardens keep
-- the names they were typed with. Old audit rows and journal narrations keep the text they were
-- written with (history, append-only).

UPDATE accounts
SET code = '008' || ((SELECT v.number FROM ventures v WHERE v.account_id = accounts.id AND v.kind = 'garden') - 1),
    serial = (SELECT v.number FROM ventures v WHERE v.account_id = accounts.id AND v.kind = 'garden') - 1
WHERE id IN (SELECT account_id FROM ventures WHERE kind = 'garden' AND account_id IS NOT NULL);
