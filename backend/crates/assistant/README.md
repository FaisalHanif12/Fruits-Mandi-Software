# mandi-assistant: ask the app

PRD Section 7.7 (FR-7.72 to FR-7.88), Tech Stack 4.8, Code Standards 5.7 (Q1 to Q11, M14),
`docs/deliverables/D6-ai-assistant.md`, `docs/deliverables/D6-system-prompt.md`.

The model is the brain; this crate is its arms and its auditor. The model reaches the books only
through 8 read-only tools, writes its answer with references (`{{r2.balance}}`) instead of figures,
and the app checks every figure before it reaches the screen. The only thing written is the
question log (`assistant_questions`, append-only).

```
app (commands) -> assistant -> services + reports -> domain + db -> core
```

| File | What it does |
|---|---|
| `run.rs` | The loop of one question: request, model call, tools, checks, one rewrite, fill, log. `ask`, `pick`, `rerun`, `test_connection` |
| `quick.rs` | The 10 chips (PRD FR-7.80): fixed tool calls, no model, not counted |
| `client.rs` | The one HTTPS call to OpenRouter (`OpenRouterClient`) and the recorded client of the tests |
| `request.rs` | The request body: fixed prompt, last 3 questions with earlier results cut down, the provider block |
| `prompt.rs`, `prompt/` | The four fixed files, copied word for word from `D6-system-prompt.md` (a test keeps them equal) |
| `tools/` | The 8 tools: `find`, `party_card`, `report`, `analytics`, `tiles`, `period`, `calculate`, `estimate` |
| `results.rs` | `tool_result_json()`: the one place a result becomes what the model reads (16,000 bytes at most) |
| `check.rs` | Every rule of FR-7.76 on the text and the next questions; the word lists are `prompt/words.tsv` only |
| `fill.rs` | References into the figures, names, dates and tables of the answer |
| `service.rs`, `keys.rs`, `settings.rs`, `log.rs` | Status, on/off and model, the key in Credential Manager, the question log |
| `limits.rs` | Every limit as a named constant (Code Standards Q9) |

## Rules that hold

- No arithmetic on money here (`#![deny(clippy::arithmetic_side_effects)]`): sums, percents, averages
  and estimates come from `services::figures` and `services::forecast`.
- A figure is a reference to a value of this question's tool results. A digit, a number word, a
  month name, a name from the results, a typed check mark, or the words of the wrong side are
  refused (one rewrite, then plain tables). A figure whose two places differ ends the question.
- The key is read just before a call and never kept, logged, saved in a setting or sent to a screen.
- Every request goes to the Azure endpoint only, with zero data retention (`request::provider`).
- Phones, addresses, father or firm names, notes, reasons and the like never go to the model
  (PRD FR-7.83); the tests scan every request body.

## Tests

`cargo test -p mandi-assistant`. The model is a recorded transcript (`tests/fixtures/model/*.json`);
no commit test calls the network. `golden_questions.rs` is E2E-47 to E2E-63 on the PRD worked day,
`prompt.rs` and `golden_prompt.rs` are P1 to P7 (the 15 examples of the prompt run as golden
questions), `check_rules.rs` is one test for each rule of FR-7.76, `tools.rs` the tools on the
worked day, `loop_limits.rs` the limits, the failures, the chips and rereading an answer.

The 15 prompt examples (P5) use the keys of the prompt (`r1`, `a20`, `je4518`); each fixture lists
under `aliases` how those keys map to the ids of the worked day. No dummy rows are added to the books.
