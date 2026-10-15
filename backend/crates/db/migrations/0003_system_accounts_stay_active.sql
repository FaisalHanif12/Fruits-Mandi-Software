-- A system account cannot be made inactive (PRD FR-10.4; Code Standards 4.1 layer 4).
--
-- The app posts to the system accounts by itself (cash, the income accounts, نقدی منڈی, the two
-- پختہ کھاتہ accounts, ابتدائی فرق ...) and a picker would no longer offer an inactive one.
-- `services::accounts` already refuses with ERR_SYSTEM_ACCOUNT; this is the last line of defence
-- if a bug lets such an update through. `accounts_system_fixed` (0001) covers the code and
-- `is_system`, and `accounts_system_no_delete` the delete; this covers `is_active`.
--
-- A trigger only: no table, column or index changes and no row is touched, so every balance and
-- every record is what it was. Only the change from active to inactive is stopped. Making a
-- system account active again stays allowed, and so do all other edits of it (the opening
-- balance is typed on the account form, PRD 4.3).
CREATE TRIGGER accounts_system_stays_active BEFORE UPDATE OF is_active ON accounts
WHEN OLD.is_system = 1 AND OLD.is_active = 1 AND NEW.is_active = 0
BEGIN SELECT RAISE(ABORT, 'ERR_SYSTEM_ACCOUNT'); END;
