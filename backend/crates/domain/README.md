# mandi-domain

Business rules as pure functions: inputs in, results out (Code Standards B4).

Holds, as later plans add them: commission rule resolution (PRD 11), بکری line and lot maths (PRD 12.5, 22), journal building (PRD 6, 12.6), partner split (PRD 17), oldest-first matching.

- May depend on: `mandi-core` only.
- Never: database, files, clock, network (Code Standards B1).
- `clippy::arithmetic_side_effects` is denied for the whole crate (Code Standards B11).
