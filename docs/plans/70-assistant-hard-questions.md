# Plan 70: Feature 10, harder real-time questions for the assistant

Deliverable D8, item 10. Requirements: PRD 7.7 (v1.32), NFR-AI-1 to NFR-AI-5, Code Standards Q1 to Q11, D6-system-prompt. Decision A1 in plan 60.

## 1. Goal

The assistant answers hard real-time questions correctly and fast, and knows چکتا buyers and money paid on the spot. Total live spend on D8 is at most $0.50.

## 2. The questions (D8 Feature 10)

H1 to H9, as in D8. H4: the books decide the side. H8: refused as a change request. H9: no made-up figure.

## 3. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Tools and reads expose paid and left and چکتا totals, through the same functions as the screens (Q3). The instant-answer layer (FR-7.90) learns the new words. | backend |
| T2 | H1 to H9 with recorded transcripts: `golden_questions`. Offline first, fixes offline. | backend |
| T3 | `scripts/ai-eval/run.mjs` gets `--budget 0.50`: it counts the real cost of every call from the usage block and stops before passing the budget. | backend |
| T4 | Live: run H1 to H9 once, fix offline, re-run only the failures. Saved as recorded transcripts. | lead |

The key: only in the app's own key store. On a Mac the keychain may ask for the Mac password; the live step waits for Umar if it does. Never in a file, a message, a log or a commit.

## 4. Acceptance (D8-A13)

H1 to H9 run live. Failures fixed and re-run. Results with model, tokens and cost. Total under $0.50.
