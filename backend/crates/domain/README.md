# mandi-domain

Business rules as pure functions (Code Standards B4). Never touches the database, files, clock or network.

- May depend on: `mandi-core` only.
- Money lives here, so every `+ - * /` on integers is a checked call (`clippy::arithmetic_side_effects` is denied for the whole crate, Code Standards B11).
- Later plans add commission rule resolution (PRD 11), بکری line and lot maths (PRD 12.5, 22), journal building (PRD 6, 12.6), partner split (PRD 17) and oldest-first matching.
- Never: database, files, clock, network (Code Standards B1).

## Modules

| Module | What it does |
|---|---|
| `rules` | Which charge rule wins (PRD 11.3): `RuleSet`, `resolve`, `charge_amount` with its working, the one `newer_key` order. |
| `ledger` | Journal entries as pure functions: `build_entry`, `build_reversal`, `build_opening` (PRD FR-6.2, FR-6.3, FR-10.9). |
| `auth::validate` | Sign-in name, password, display name, idle lock minutes, text size, last screen (PRD FR-8, NFR-SEC-1). |
| `auth::lockout` | The lockout steps: 5 wrong tries lock 60 seconds, 10 lock 300, 15, 20, 25 ... lock 900 (PRD FR-8.2). |
| `auth::recovery` | The Owner recovery code: 16 characters from an alphabet without look-alikes, shown as `XXXX-XXXX-XXXX-XXXX` (NFR-SEC-12). |
