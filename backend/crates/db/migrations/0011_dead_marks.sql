-- Migration 11 (plan 40, PRD FR-10.10, v1.10): the dead amount (ڈوبی رقم).
--
-- A person who owes the shop may die or leave the business. He is moved to the dead amount: his
-- account is made inactive (a پختہ کھاتہ person only gets his person key marked, because the پختہ
-- accounts stay active) and what he still owes is shown on its own. NOTHING POSTS: his balance stays
-- on his account, so the trial balance, the partner split and every partner's کل رقم do not change
-- (PRD A65, FR-6.1). This table only sorts people into a group. It holds no balance: what a person
-- still owes is read from `journal_lines`. `owed_paisa` is the amount on the day of the move, kept
-- for the history and shown by R38 as "owed then".
--
-- What this file does:
--   1. `dead_marks`: one row for each move. `pukhta_person_key` is set only for a one-time person.
--   2. At most one ACTIVE mark for each account or person key. A plain unique index would let two
--      rows with a NULL key through (NULLs are distinct in SQLite), so the key is wrapped in
--      COALESCE.
--   3. Triggers: a mark is never deleted, and an undo changes only `is_active`, `undone_at`,
--      `undone_by`, `undo_reason` and `undone_after_entry_id` (PRD FR-6.3 in spirit: history is kept). An undone mark
--      never changes again; a person moved again later gets a new row.
--
-- No existing row is touched and no amount moves.

CREATE TABLE dead_marks (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    pukhta_person_key TEXT CHECK (pukhta_person_key IS NULL OR length(trim(pukhta_person_key)) > 0),
    entry_date TEXT NOT NULL CHECK (date(entry_date) IS entry_date),
    reason TEXT NOT NULL CHECK (reason IN ('died', 'left', 'other')),
    note TEXT NOT NULL DEFAULT '',
    owed_paisa INTEGER NOT NULL CHECK (owed_paisa > 0) CHECK (owed_paisa % 100 = 0),
    -- The highest POSTED journal entry id at the moment of the move. R38 counts what came after it:
    -- a recovery (a وصولی or an ادھار رقم واپسی) and a correction (an edit, cancel or undo of an
    -- earlier posting, PRD A67). An id is exact where a date is not (two entries of one day).
    after_entry_id INTEGER NOT NULL CHECK (after_entry_id >= 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    undone_at TEXT,
    undone_by INTEGER,
    undo_reason TEXT,
    -- The highest journal entry id when the move was undone, so the figures of an undone move stop
    -- there (what is posted after he is active again is ordinary business).
    undone_after_entry_id INTEGER,
    -- The business day of the undo, so a report for a day between the move and the undo still
    -- shows him in the dead amount.
    undone_date TEXT CHECK (undone_date IS NULL OR date(undone_date) IS undone_date),
    CHECK (reason <> 'other' OR length(trim(note)) > 0),
    CHECK ((is_active = 1 AND undone_at IS NULL AND undo_reason IS NULL
            AND undone_after_entry_id IS NULL AND undone_date IS NULL)
        OR (is_active = 0 AND undone_at IS NOT NULL AND undone_after_entry_id IS NOT NULL
            AND undone_date IS NOT NULL
            AND undo_reason IS NOT NULL AND length(trim(undo_reason)) > 0))
) STRICT;

CREATE UNIQUE INDEX idx_dead_marks_one_active
    ON dead_marks (account_id, COALESCE(pukhta_person_key, ''))
    WHERE is_active = 1;
CREATE INDEX idx_dead_marks_account ON dead_marks (account_id, is_active);
CREATE INDEX idx_dead_marks_date ON dead_marks (entry_date, id);

CREATE TRIGGER dead_marks_no_delete BEFORE DELETE ON dead_marks
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;

CREATE TRIGGER dead_marks_only_undo BEFORE UPDATE ON dead_marks
WHEN OLD.is_active = 0
  OR NEW.account_id IS NOT OLD.account_id
  OR NEW.pukhta_person_key IS NOT OLD.pukhta_person_key
  OR NEW.entry_date IS NOT OLD.entry_date
  OR NEW.reason IS NOT OLD.reason
  OR NEW.note IS NOT OLD.note
  OR NEW.owed_paisa IS NOT OLD.owed_paisa
  OR NEW.after_entry_id IS NOT OLD.after_entry_id
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
