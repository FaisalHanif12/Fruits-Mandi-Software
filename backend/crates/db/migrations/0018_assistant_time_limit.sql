-- Migration 18 (plan 54 S5, plan 55 Section 3, PRD FR-7.84): the question log accepts the outcome
-- `time_limit`.
--
-- A question that reaches the hard limit of 25 seconds ends with what was read, shown as plain
-- tables, and the log keeps it as `time_limit` (not as `error`, and never as "not understood").
-- SQLite cannot widen a CHECK in place, so the table is made again with the longer list of
-- outcomes and every row is copied as it is: same ids, same values, same order. Nothing else in
-- the file changes: no book, amount, id or chain hash.
--
-- The order matters because a continuation points at its parent question (a self link). The old
-- table is renamed first, so the new table can take the name, point at itself, and be filled
-- parents first (by id); the old table, which nothing else points at, is dropped last. The old
-- triggers and indexes are dropped first (their names are made again below with the same rules),
-- and dropping a table fires no trigger.

DROP TRIGGER assistant_questions_no_replace;
DROP TRIGGER assistant_questions_no_update;
DROP TRIGGER assistant_questions_no_delete;
DROP INDEX idx_assistant_questions_day_counted;
DROP INDEX idx_assistant_questions_asked_at;
DROP INDEX idx_assistant_questions_parent;
DROP INDEX idx_assistant_questions_asked_by;

ALTER TABLE assistant_questions RENAME TO assistant_questions_old;

CREATE TABLE assistant_questions (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    asked_at TEXT NOT NULL,
    asked_day TEXT NOT NULL CHECK (date(asked_day) IS asked_day),
    asked_by INTEGER REFERENCES users(id),
    question TEXT NOT NULL CHECK (length(trim(question)) > 0),
    language TEXT NOT NULL CHECK (language IN ('ur', 'en')),
    conversation_id TEXT NOT NULL CHECK (length(conversation_id) > 0),
    parent_id INTEGER REFERENCES assistant_questions(id),
    source TEXT NOT NULL CHECK (source IN ('chip', 'model', 'continue', 'rerun', 'test')),
    counted INTEGER NOT NULL CHECK (counted IN (0, 1) AND (counted = 0 OR source IN ('model', 'test'))),
    sources_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(sources_json)),
    outcome TEXT NOT NULL CHECK (outcome IN (
        'answered', 'estimate', 'advice', 'which_one', 'need_dates', 'need_party', 'not_found',
        'outside_books', 'change_refused', 'not_understood', 'checks_failed', 'records_differ',
        'locked', 'limit_reached', 'offline', 'error', 'time_limit')),
    tool_calls_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(tool_calls_json)),
    answer_json TEXT CHECK (answer_json IS NULL OR json_valid(answer_json)),
    model TEXT,
    model_calls INTEGER CHECK (model_calls IS NULL OR model_calls >= 0),
    tokens_in INTEGER CHECK (tokens_in IS NULL OR tokens_in >= 0),
    tokens_cached INTEGER CHECK (tokens_cached IS NULL OR tokens_cached >= 0),
    tokens_out INTEGER CHECK (tokens_out IS NULL OR tokens_out >= 0),
    tokens_reasoning INTEGER CHECK (tokens_reasoning IS NULL OR tokens_reasoning >= 0),
    finish_reason TEXT,
    total_millis INTEGER CHECK (total_millis IS NULL OR total_millis >= 0),
    model_millis INTEGER CHECK (model_millis IS NULL OR model_millis >= 0),
    tool_millis INTEGER CHECK (tool_millis IS NULL OR tool_millis >= 0),
    calls_json TEXT CHECK (calls_json IS NULL OR json_valid(calls_json))
) STRICT;

INSERT INTO assistant_questions (
    id, created_at, created_by, updated_at, updated_by, version, asked_at, asked_day, asked_by,
    question, language, conversation_id, parent_id, source, counted, sources_json, outcome,
    tool_calls_json, answer_json, model, model_calls, tokens_in, tokens_cached, tokens_out,
    tokens_reasoning, finish_reason, total_millis, model_millis, tool_millis, calls_json)
SELECT
    id, created_at, created_by, updated_at, updated_by, version, asked_at, asked_day, asked_by,
    question, language, conversation_id, parent_id, source, counted, sources_json, outcome,
    tool_calls_json, answer_json, model, model_calls, tokens_in, tokens_cached, tokens_out,
    tokens_reasoning, finish_reason, total_millis, model_millis, tool_millis, calls_json
FROM assistant_questions_old ORDER BY id;

DROP TABLE assistant_questions_old;

CREATE INDEX idx_assistant_questions_day_counted ON assistant_questions (asked_day, counted);
CREATE INDEX idx_assistant_questions_asked_at ON assistant_questions (asked_at);
CREATE INDEX idx_assistant_questions_parent ON assistant_questions (parent_id);
CREATE INDEX idx_assistant_questions_asked_by ON assistant_questions (asked_by);

CREATE TRIGGER assistant_questions_no_replace BEFORE INSERT ON assistant_questions
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM assistant_questions WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER assistant_questions_no_update BEFORE UPDATE ON assistant_questions
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER assistant_questions_no_delete BEFORE DELETE ON assistant_questions
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
