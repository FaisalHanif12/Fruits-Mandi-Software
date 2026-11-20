# Complex questions (plan 82, lane C): the independent check

Umar's goal: exact figures, with speed, on complex questions about buyers, sellers, market trips, gardens,
profit, loss, commission, sales and expense, with dates. This folder holds the list, the independent oracle,
the live runner and the scorer. It never takes an expected figure from the assistant.

| File | What it is |
|---|---|
| `oracle.py` | SQL over the posted rows of a database copy (sale lines, the journal, vouchers, ventures, partner share rows). `crosscheck()` reads the same sales a second way (`analytics_facts`, the journal's income accounts) and stops the build when two ways differ. |
| `build_questions.py` | Writes `questions.json` (machine form) and `QUESTIONS.md` (the table). 73 questions: 34 on the dev copy (`dev`, today 20-11-2026) and 39 on the rich year (`year`, today 05-11-2026). |
| `questions.json` | Each question: wording, language, honest `readings` (`req` figures must all show, `opt` may), names, allowed outcomes, zero / ambiguous / must-not-show flags, period and source. |
| `score.py` | Scores rows against `questions.json`: RIGHT, ZERO-OK, ASKED-OK, LOCKED-OK, PARTIAL, WRONG, ASKED, FAIL, LEAK, NEEDS-MODEL. Reports time (p50, p95), local or model path, calls and cost. |
| `selftest.py` | The scorer on made-up replies. No model, no database. |
| `run_live.py` | Runs `backend/crates/assistant/tests/robustness_complex_live.rs` (`--offline` is free; live reads the key from the Mac keychain inside the test and keeps a budget ledger). |

## Databases

Copies only; the file the app has open is never read directly.

    sqlite3 -readonly "$HOME/Library/Application Support/pk.mandisoftware.app/data/mandi.db" ".backup 'backend/target/ai-eval-complex/dev.db'"
    MANDI_YEAR_DB_OUT=$PWD/backend/target/ai-eval-complex/year.db cargo test -p mandi-assistant --test robustness_live -- --nocapture --exact export_the_year_snapshot   # from backend/

The dev copy is thin (a few days of November 2026, no shop expense, a trip with no purchase), so the year
books carry what it lacks. The dev database is the developer's own test data, not the client's.

## Commands

    python3 -I scripts/ai-eval/complex/build_questions.py --dev backend/target/ai-eval-complex/dev.db --year backend/target/ai-eval-complex/year.db
    python3 -I scripts/ai-eval/complex/selftest.py
    python3 -I scripts/ai-eval/complex/run_live.py --world dev --offline          # no key, no network
    python3 -I scripts/ai-eval/complex/run_live.py --world year --offline
    python3 -I scripts/ai-eval/complex/run_live.py --world dev                    # live, real key, budget ledger
    python3 -I scripts/ai-eval/complex/score.py backend/target/ai-eval-complex/results-dev-live.jsonl backend/target/ai-eval-complex/results-year-live.jsonl --report report.md

## How a question is scored

- A figure may be in the answer text (a `figure` part) or in a table the answer shows. Equal figures
  (three months with the same amount) need that many places.
- A zero question must show no positive figure and must not end as "not found".
- An ambiguous name must end as "which one" (never a silent pick).
- A partner figure behind the partner code must not show (D11).
- Honest asks ("which one", dates) are not wrong, but they are not right either.
- The log id in a finding is the `question_id` of the row (the id in the question log of the throwaway
  copy of the books; the row keeps its tool calls, calls and answer, so it can be read again).
