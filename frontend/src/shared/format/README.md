# shared/format

Display-only formatting: paisa to text such as "12,000", grouping, print digits.
Nothing here calculates money (CLAUDE.md rule 1, Code Standards F3, M12).

| File          | Holds                                                                                                                                                                                                                                                                                                                                                |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `money.ts`    | The opaque `Paisa` type, `comparePaisa`, `formatPaisa(p, { grouping })` and `groupDigits`. The only file that looks inside `Paisa`. `formatPaisa` works on the digits as text and never divides, so it cannot make a decimal. A value that is not a safe whole number of rupees throws in development and is logged (shown as a dash) in production. |
| `grouping.ts` | The app-wide grouping setting (`international` 1,316,040 or `lakh` 13,16,040), default international (PRD A20).                                                                                                                                                                                                                                      |
| `digits.ts`   | Urdu digits (۰-۹) for Urdu prints only, off by default (NFR-URD-14). Screens always show English digits.                                                                                                                                                                                                                                             |

Examples: `formatPaisa(1_200_000, { grouping: 'international' })` gives `"12,000"`;
`formatPaisa(131_604_000, { grouping: 'lakh' })` gives `"13,16,040"`.
