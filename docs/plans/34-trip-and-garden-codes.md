# Plan 34: easy codes for market trips and gardens

Version 1.0, 27-10-2026. Lead: tech-lead. Asked by Umar on 27-10-2026 ("as we made the code easy 001 ... 006 or 007 for market trip and garden we can keep"). Staff already has 006, so the lead set **market trip 007** and **garden 008** (Umar told; a swap is one line). Builds on plan 32 (`mandi_core::codes`). Branch `feature/d5-early`.

## Requirements (PRD 1.8: FR-10.2 table, 12.10 trip no, 12.11 garden no, Section 21)

R1. A trip's number is a code by type 007: the first trip is 0070, then 0071 … 00710 (same builder, same no-padding rule, never reused). It replaces سفر-1, سفر-2 everywhere the trip number shows: trip list and screen, pickers, بکری lot (own goods), statement lines (FR-17.10), reports R33, R34, analytics J, prints and Excel.

R2. A garden's number is a code by type 008: the first garden is 0080, then 0081 … Its account under control 0306 gets the **same** code (0080), so the garden and its account read the same. It replaces باغ-1, باغ-2 everywhere: garden list and screen, pickers, بکری lot seller, statement lines, R36, R34, analytics I, prints and Excel.

R3. Short code = the running number in a trip or garden field (`0` → 0070 in a trip field, 0080 in a garden field), as plan 32 R5.

R4. Purchase numbers (خ-1), voucher numbers and پھل numbers do not change.

R5. A new numbered migration renumbers existing trips and gardens (and the gardens' accounts) in order. No amount, id, journal line or chain hash changes. Released migrations are never edited.

R6. No money change. Test texts and fixtures with سفر-1 or باغ-1 change to the new codes only. E2E-34, 34b, 40, 40b, 41 numbers unchanged.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Contract + logic: type numbers 007 and 008 in `codes`, trip and garden numbering, garden account code, short codes, migration, display text of the number in services/reports/print data, tests, fixtures, `pnpm gen:types`; tell `frontend` | backend |
| F1 | Every screen and print that shows a trip or garden number; pickers; fixtures and tests; Fluent where a label said سفر-1 style; screenshots ur and en | frontend (after plan 33 F1, or in parallel if no file overlap) |

Targeted checks only.

## Acceptance

R1 to R6 each shown by a test or screenshot; garden 0080 and its account 0080; first trip 0070; E2E-34, 40, 41 numbers unchanged; migration test on a file with old trip and garden numbers.
