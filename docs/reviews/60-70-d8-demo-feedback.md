# Review: D8, client demo feedback (plans 60 to 70)

Branch `feature/d8-demo-feedback`, from `feature/assistant-realtime` at 5bca5a61. One end gate for the whole run, as a goal run.

## Reviewer (code-reviewer)

First pass: FAIL on one MAJOR finding, plus five MINOR findings.

| # | Finding | What was done |
|---|---|---|
| 1 | MAJOR. Feature 10 not finished: H1, H2, H5 and H6 had no test, and there was no live run (D8-A13) | Offline work committed in cc16fe83. Live run: 7 of 9 right, $0.0036. H5, H6 and H1's speed are being fixed, then re-run |
| 2 | No money review of plan 67 on record | Done. The money agent found real faults: a seller-only edit re-read the buyer's rules, and the reverse. Umar decided the rule (PRD FR-12.20, 10-11-2026). Fixed in 719b113c, then money agent PASS |
| 3 | E2E-15 lost its check of the cancelled status | Asserted on the day's list (719b113c) |
| 4 | Golden tests not where Code Standards 9.2 says | Table corrected, Code Standards v1.22 |
| 5 | پختہ words left in the demo checklist and the word list | Removed; the 1,000-name accuracy floors hold |
| 6 | Stale notes on R9 and Mac printing | Fixed |

The reviewer recomputed the worked numbers of plans 61, 65, 66 and 67 by hand, and every one matched. No golden number changed.

## Robustness (robustness-tester)

First pass: FAIL on one MAJOR speed finding.

| # | Finding | What was done |
|---|---|---|
| 1 | MAJOR. At 1,000 lines the edit preview cost 97 to 104 ms of CPU, about 165 ms on the reference machine, against a 100 ms budget. Cause: three rule queries per saved line | Levels read in one query: 13 to 15 ms, about 22 ms on the reference machine (cc16fe83) |
| 2 | MINOR. The later party-ledger page has a thin margin at full volume: 256 of 300 ms | Follow-up |
| 3 | MINOR. `chukta_search` matched any phone that contained a typed digit | Fixed: it needs 3 or more digits in a run of the phone |

Re-verdict: PASS WITH MINOR.

What held: 74 hostile inputs on the paid cell; an 80-step random run with the books balanced at every step; migrations 0020 and 0021 with injected failures and 40 process kills; a read-only copy of Umar's own backup migrated with nothing moved; about 70 hostile چکتا names; the 1,000-line limits; party ledgers at NFR-SCL-1 volume; a keyboard fuzz with no wrong pick; 13,008 hostile command calls with 0 internal errors.

## Follow-ups

- The later party-ledger page margin (robustness 2).
- Windows, LOCAL-ONLY: Urdu shaping of the new headings in WebView2, a keyboard run of the 4-Enter path, and the Windows print path.
- Hand checks for Umar: one real Mac print dialog, Print and then Save as PDF. The Mac cannot tell Print from Cancel, so a cancelled dialog still counts as printed.
- The Urdu reviewer must check every new Fluent message marked `review: pending`, and `whats-new.md`.
