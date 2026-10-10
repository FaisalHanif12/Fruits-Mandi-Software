# mandi-core

Pure helpers with no business rules and no database (Tech Stack 5.4).

Holds, as later plans add them: money (`Paisa`, `Bps`, `round_to_rupee`, `split_by_bps`, `split_by_qty`, parsing), dates, typed IDs, Urdu and English search keys, `translit`, amounts in words, and input validation.

- May depend on: no crate of ours.
- Never: database, files, network.
- `clippy::arithmetic_side_effects` is denied for the whole crate (Code Standards B11). Use checked maths.
