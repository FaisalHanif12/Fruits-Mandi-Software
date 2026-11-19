-- Migration 17 (plan 51 P51-B2, D6 "ask the app", PRD FR-7.84, NFR-SPD-17, NFR-OBS-1): how long a
-- question took.
--
-- Umar found every answer slow, and the log could not say why: `asked_at` and `created_at` are both
-- set when the row is written, so a row has no duration. These four nullable columns of
-- `assistant_questions` say where the time of one run went:
--   total_millis  the whole run, from the start to the reply;
--   model_millis  of that, the time spent waiting for the model (all its calls);
--   tool_millis   of that, the app's own work in the tools;
--   calls_json    each model call of the run as JSON: its time, its tokens, whether it failed
--                 (a timeout shows here) and its tools with their times. Counts, times and the
--                 fixed names of tools only: no question, name, figure or key.
--
-- Additions only. Every existing row keeps its values and gets NULL in the new columns (the
-- question log of an older version has no times). No row is touched, so the append-only triggers
-- of migration 15 (a row is never changed, never deleted and never replaced) stay as they are and
-- still hold for the new columns. No other table, index or trigger changes, and no amount, id or chain hash moves.

ALTER TABLE assistant_questions ADD COLUMN total_millis INTEGER
    CHECK (total_millis IS NULL OR total_millis >= 0);
ALTER TABLE assistant_questions ADD COLUMN model_millis INTEGER
    CHECK (model_millis IS NULL OR model_millis >= 0);
ALTER TABLE assistant_questions ADD COLUMN tool_millis INTEGER
    CHECK (tool_millis IS NULL OR tool_millis >= 0);
ALTER TABLE assistant_questions ADD COLUMN calls_json TEXT
    CHECK (calls_json IS NULL OR json_valid(calls_json));
