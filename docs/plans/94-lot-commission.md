# Plan 94: Seller commission, buyer commission and لاگا in the lot row

| | |
|---|---|
| Status | Umar's request and decision, 22-11-2026 ("go with my choice"); starts after plan 92 T1 |
| Branch | `fix/vouchers-edit-sign` |
| PRD | FR-11.10 (v1.56 bullet), FR-11.6, FR-12.3, FR-12.4, FR-12.20, NFR-USE-6, C2 |

## Umar's decision

Each lot row has seller commission, buyer commission and لاگا. They start at what the rules give for that lot (6% by default; لاگا 8 per نگ on fruit). The lot's values apply to every sale line of the lot, a buyer with his own rule included. Any single line can still be changed below; a line changed by hand keeps its value when the lot's value changes.

## Money (money-calculations agent before and after)

- No new formula. Each line is still worked out on its own value: commission = round(value x bps / 10,000) (C2), لاگا per نگ = qty x value (C3). The lot field only gives the line its rate.
- Snapshot per line as today (FR-11.6): method and value; a value different from the rule is saved as دستی.
- Edit (FR-12.20): an unchanged line keeps its stored snapshot unless the lot value was changed in this edit.
- Worked: lot رحیم، کیلا-1, lot seller commission 5%: اکرم 12,000 gives 600, ارشد 17,400 gives 870 (total 1,470). Untouched lot: every number of E2E-1 to E2E-16b unchanged.

## Contract (to be fixed by backend in T1)

Lot input gains `seller_comm_rate`, `buyer_comm_rate`, `laga_rate` (text, optional; empty means the rule). The preview returns each lot's effective rates and their source (rule or lot). A line's own manual rate wins over the lot rate.

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: lot-level rates in the draft, applied to lines without their own manual rate, snapshot, edit, preview; tests with the worked numbers; golden tests unchanged; money agent twice | `backend` |
| T2 | Frontend: the three fields in the lot row (Enter does not stop on them), lines show the lot rate, a line override still works; ur and en; tests | `frontend` |

## Acceptance

Worked numbers pass; golden tests unchanged; a line override survives a lot change; 4 Enters per line; lead checks in WebKit.

## Contract as fixed by backend, 22-11-2026

- Input: `LotInput` gains optional `seller_comm_rate`, `buyer_comm_rate`, `laga_rate`, each the existing `LineChargeInput { method: "percent" | "per_unit", value: string }` (percent text "5", "2.5"; لاگا per نگ rupees with per_unit). Absent, null or blank = the rule decides. A line's own typed rate wins over the lot rate. With line override off in Settings: `ERR_OVERRIDE_NOT_ALLOWED` at `lots[i].seller_comm_rate` (and the other two).
- Output: `LotPreview` gains `seller_comm_rate`, `buyer_comm_rate`, `laga_rate`, each `LotRate | null` (null while the lot cannot be worked out). `LotRate { method, value_bps, value_paisa, source: typed | rule | default | none, rule_id }`. The buyer side and لاگا default are resolved without a buyer (item, group, default level), as Umar decided.
- Edit load: the lot rate is shown when every line of the lot was saved with the same manual value (the most common one if lines differ); lines that differ keep their own override.
- Saved lines unchanged in shape: each line its own snapshot, دستی when it follows a lot value.
- Carried lots and later sales: rules of the later day (FR-12.17); a lot rate typed on a carried lot applies to its new lines.
- Lead decision: the three new `None` fields are added to every `LotInput` literal in tests and benches by backend, including `robustness*` files, as a mechanical change with no logic change (no robustness teammate is running).
