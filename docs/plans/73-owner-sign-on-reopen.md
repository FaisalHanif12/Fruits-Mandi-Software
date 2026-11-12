# Plan 73: the Owner's sign when a closed day is reopened to fix an entry

| | |
|---|---|
| Asked by | Umar, 13-11-2026 |
| Branch | `feature/d8-demo-feedback` (Mac phase) |
| Requirements | PRD v1.36: FR-21.4 (the sign), FR-7.71 (the stamp shows it), Section 21 (Security); Architecture v1.25 15.3 |
| Requirement change | PRD v1.36 (this plan is Umar's approval) |

## 1. Goal

"دن کھولیں اور درست کریں" asks for the reason and the sign. Only the Owner's sign opens the day. Every edit saved in that fix shows the sign in its stamp and history.

Umar's words: "add the sign as well along with the reason. The sign will only be applicable adnan, which is owner. If someone writes anything else, that will not be open. Edited must be displayed along with sign, whatever we edited."

## 2. Decisions (defaults marked [A] in the PRD)

| # | Decision |
|---|---|
| S1 | The sign field "دستخط" sits below the reason in the reopen box, shared by every screen (`shared/edit`). Raw text, no Roman-to-Urdu conversion. |
| S2 | Accepted: the settings key `owner_sign`, default `adnan`. Compared after trimming, ignoring case. The Urdu `عدنان` is also accepted. |
| S3 | Wrong sign: `ERR_SIGN_WRONG` on `sign`, "دستخط درست نہیں". Nothing opens. An audit entry records the wrong try, without the typed text. |
| S4 | The sign is saved with the reason in the audit entry of the reopen and of every edit saved in that fix session (the fix token carries it). |
| S5 | Stamp and history: `EditStamp` and each `EditHistory` version gain `sign: string \| null`. Shown as "دستخط: adnan" after the name. An edit of an open day has no sign. |
| S6 | Settings, Security: the Owner can change his sign with his password; audit entry. |
| S7 | The date bug found by Umar on 13-11-2026 (the fix flow sent `YYYY-MM-DD`, `day_fix_open` takes `DD-MM-YYYY`) is fixed first, in the same shared flow. |

## 3. Contract

- `DayFixOpenInput` gains `sign: string`. Errors: `ERR_SIGN_REQUIRED` and `ERR_SIGN_WRONG`, both on `sign`.
- `EditStamp.sign` and `EditHistoryVersion.sign`: `string | null`.
- `settings_get_owner_sign` is not needed. `settings_set_owner_sign { password, sign }` refuses a wrong password with the existing code, and a sign that is empty or longer than 40 characters.
- No money, no table, no migration (the settings row is written on first change; the default is a constant).

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| T0 | PRD v1.36, Architecture v1.25 (done by the lead) | lead |
| T1 | Backend: input, check, audit entries carry the sign, stamp and history return it, the settings command; tests; `pnpm gen:types` | backend-paid |
| T2 | Frontend: the date fix (S7), the sign field in the reopen box, the error under it, the stamp and history show the sign, the Settings field; both languages; tests | frontend-paid |

## 5. Done when

A test reopens a closed day with sign `adnan` (and `ADNAN `, and `عدنان`) and it opens; with `munshi` it stays closed and nothing changes. The edit saved in that fix shows "دستخط: adnan" in its stamp and history. The date bug's failing test passes. Golden tests unchanged.
