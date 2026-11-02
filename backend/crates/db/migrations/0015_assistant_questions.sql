-- Migration 15 (plan 48, D6 "ask the app", PRD FR-7.84, Architecture 15.3): `assistant_questions`, the
-- question log of the assistant.
--
-- One row for every question: a typed question to the model, a quick question (a chip), a
-- continuation after "کون سا؟" or after the partner code, an old answer read again, and a connection
-- test. It is written once, in its own save, after the answer. It is the source of the daily count
-- of 20 (PRD FR-7.81): the count of today's rows with `counted = 1`, where `asked_day` is the
-- Pakistan date.
--
-- What it holds: the question as typed, the tools the model called with their inputs, the answer
-- as the model wrote it (with its references, not the figures), the outcome, the model, the number
-- of model calls and the tokens. No figure shown to the Owner is stored: the tools and their
-- inputs are, so `assistant_rerun` can read an answer again on today's books. The OpenRouter key is
-- never here (NFR-SEC-15); it lives only in Credential Manager.
--
-- Append-only (Code Standards B17): triggers refuse UPDATE, DELETE and the replace of an existing
-- id. Nothing else in the file system or in the books changes; no existing row is touched.

CREATE TABLE assistant_questions (
    id INTEGER PRIMARY KEY,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    created_by INTEGER,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_by INTEGER,
    version INTEGER NOT NULL DEFAULT 1,
    -- When it was asked (UTC) and the Pakistan date it counts for.
    asked_at TEXT NOT NULL,
    asked_day TEXT NOT NULL CHECK (date(asked_day) IS asked_day),
    asked_by INTEGER REFERENCES users(id),
    -- The question as typed. For a chip, the chip's name (for example `cash_in_hand`).
    question TEXT NOT NULL CHECK (length(trim(question)) > 0),
    language TEXT NOT NULL CHECK (language IN ('ur', 'en')),
    conversation_id TEXT NOT NULL CHECK (length(conversation_id) > 0),
    -- The question a continuation belongs to.
    parent_id INTEGER REFERENCES assistant_questions(id),
    source TEXT NOT NULL CHECK (source IN ('chip', 'model', 'continue', 'rerun', 'test')),
    -- 1 only for a model question and a connection test (PRD FR-7.81, FR-7.85).
    counted INTEGER NOT NULL CHECK (counted IN (0, 1) AND (counted = 0 OR source IN ('model', 'test'))),
    -- Each source read, with its filters.
    sources_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(sources_json)),
    outcome TEXT NOT NULL CHECK (outcome IN (
        'answered', 'estimate', 'advice', 'which_one', 'need_dates', 'need_party', 'not_found',
        'outside_books', 'change_refused', 'not_understood', 'checks_failed', 'records_differ',
        'locked', 'limit_reached', 'offline', 'error')),
    -- Each tool called with its inputs, in order. For a chip, its fixed tool calls.
    tool_calls_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(tool_calls_json)),
    -- The answer as the model wrote it, with references. Null for a chip.
    answer_json TEXT CHECK (answer_json IS NULL OR json_valid(answer_json)),
    -- Null for a chip.
    model TEXT,
    model_calls INTEGER CHECK (model_calls IS NULL OR model_calls >= 0),
    tokens_in INTEGER CHECK (tokens_in IS NULL OR tokens_in >= 0),
    tokens_cached INTEGER CHECK (tokens_cached IS NULL OR tokens_cached >= 0),
    tokens_out INTEGER CHECK (tokens_out IS NULL OR tokens_out >= 0),
    tokens_reasoning INTEGER CHECK (tokens_reasoning IS NULL OR tokens_reasoning >= 0),
    finish_reason TEXT
) STRICT;

-- The daily count, the list newest first, and the continuations of a question.
CREATE INDEX idx_assistant_questions_day_counted ON assistant_questions (asked_day, counted);
CREATE INDEX idx_assistant_questions_asked_at ON assistant_questions (asked_at);
CREATE INDEX idx_assistant_questions_parent ON assistant_questions (parent_id);
CREATE INDEX idx_assistant_questions_asked_by ON assistant_questions (asked_by);

-- Append-only, as audit_log in migration 1.
CREATE TRIGGER assistant_questions_no_replace BEFORE INSERT ON assistant_questions
WHEN NEW.id > 0 AND EXISTS (SELECT 1 FROM assistant_questions WHERE id = NEW.id)
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER assistant_questions_no_update BEFORE UPDATE ON assistant_questions
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
CREATE TRIGGER assistant_questions_no_delete BEFORE DELETE ON assistant_questions
BEGIN SELECT RAISE(ABORT, 'ERR_APPEND_ONLY'); END;
