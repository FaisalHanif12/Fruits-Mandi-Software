# Plan 92: Saving an edited وصولی, ادائیگی or پیشگی asks the reason and the sign; the sign can be changed in Settings

| | |
|---|---|
| Status | Umar's request, 22-11-2026 (screenshots: ادائیگی 1 of 16-11-2026, amount 100,000 changed to 110,000, F10 saves nothing) |
| Branch | `fix/vouchers-edit-sign` |
| PRD | FR-15.3 (v1.55 bullet), FR-7.70, FR-7.71 (v1.55 note), FR-21.4 (the sign), Section 21 Security (v1.55) |

## Root cause of "it does not save"

16-11-2026 is not today, so the backend refuses the edit with `ERR_REASON_REQUIRED` on the field `reason` (`services/src/vouchers/edit.rs`). The screen shows the edit reason's error only for the field `edit_reason` (`VoucherForm.tsx:301`), so the refusal is shown nowhere: the user presses F10 and nothing happens. A refusal must never be silent.

## Umar's decisions

1. Pressing save (F10, Ctrl+S, the button) on an edited وصولی, ادائیگی or پیشگی opens a box that asks the reason and the sign (دستخط), on any day, today included. Both are required. A wrong sign shows "دستخط درست نہیں" under the sign and saves nothing (no lockout, Umar 22-11-2026). The right sign saves the edit.
2. After the save the entry shows "تبدیل شدہ" with the time, Owner, the reason and the sign (FR-7.71), in its header and its list row.
3. Settings, Security: the sign is changed with the Owner's password **and the current sign**, then the new sign typed twice.

## Contract (backend to frontend)

- `vouchers_edit` input gains `sign: String`. For these three types: `ERR_REASON_REQUIRED` on `reason` when empty (any day), `ERR_SIGN_REQUIRED` on `sign` when empty, `ERR_SIGN_WRONG` on `sign` when it is not the Owner's sign (same check as FR-21.4: "adnan" by default, case and end spaces ignored, عدنان accepted). A wrong sign writes an audit row like FR-21.4 and saves nothing. The audit entry of the edit holds the reason and the sign; the stamp read (the one the screen already uses for FR-7.71) returns the sign.
- On a closed day the existing reopen-and-fix flow (FR-21.4) already asks the reason and sign: the voucher screen must ask them once, not twice.
- `settings_set_owner_sign` input gains `current_sign: String`: `ERR_SIGN_REQUIRED` / `ERR_SIGN_WRONG` on `current_sign`, checked after the password. The new sign is checked as today.
- No money changes: the edit posts exactly as today (reversal and new posting, FR-6.3).

## Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Backend: the `sign` field and its checks on the voucher edit, audit and stamp with the sign, `current_sign` on the sign change; tests that fail before; `pnpm gen:types` | `backend` |
| T2 | Frontend: on save of an edited voucher, a box with reason and sign (focus on reason, Enter moves, F10 saves, Esc cancels with nothing saved), errors under their fields; the inline edit-reason field goes into the box; every refusal shown (no silent F10); the stamp with time, reason and sign after save; Settings sign card asks password, current sign, new sign twice; ur and en; tests | `frontend` |

## Acceptance

- Edit ادائیگی 1 of 16-11-2026 from 100,000 to 110,000, F10: the box asks reason and sign; sign "munshi" says the sign is wrong and nothing changes; sign "adnan" saves; احسان's balance shows 110,780 جمع; the header shows "تبدیل شدہ" with time, reason and sign; the audit row has both.
- Empty reason or sign: told in the box, nothing saved.
- Settings: wrong password or wrong current sign changes nothing; right ones change the sign; the new sign opens the next edit.
- Golden tests unchanged; clippy, typecheck, lint, i18n:check and the touched tests pass.

## T2b: the blank band under Settings > فرم (Umar, 22-11-2026, his screenshots)

Root cause, reproduced by the lead in headless WebKit at 1480x830: a print-header person row has the role Radix Select. Radix keeps a hidden native `<select>`, which takes `main` (relative in AppShell) as its containing block instead of the panel's scroll box, so `main` gets 502 px of overflow: the title and tabs scroll away and a blank band shows. With no person there is no bug, which is why the earlier check (no person) passed. Making the scroll box `relative` gives 0 overflow. Owner: `frontend`. Check the other inner scroll areas the same way. The lead verifies with the WebKit script (one person added, main overflow must be 0) before handing to Umar.
