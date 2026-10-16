# rules (commission rules, plan 09)

The screens of PRD 11: the rule list, the rule form, "Test a rule", and the settings tabs for the
global defaults and the overrides (the settings tabs live in `features/settings`).

- The form offers only the charges and methods of PRD 11.2 (`allowed.ts`). The backend refuses any
  other and is the final word.
- A rule's value is text, as written (`6`, `2.5`, `8`). The frontend never works out an amount.
  "Test a rule" shows the amounts and the working the backend sends (`shared/ui/CalcInfo`).
- The types come from the generated bindings (`types.ts`), and `api.ts` is the only file that knows
  the command shapes. The working of an amount is typed (money, percent in basis points, count,
  level); `shared/ui/CalcInfo` writes each value as text.
