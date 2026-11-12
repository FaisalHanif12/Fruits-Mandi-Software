# shared/edit: the one edit kit (plan 44)

PRD FR-7.70 (edit every entry the same way), FR-7.71 (the edit stamp), FR-21.4 (reopen and fix).

Every saved entry uses these pieces, never its own copy:

| Piece                                              | Where                    | What it does                                                                                                                                                                                                               |
| -------------------------------------------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `EntryActions`                                     | `shared/ui`              | The two buttons, تبدیل کریں and منسوخ کریں, from the flags on the row (`can_edit`, `can_cancel`, `locked_reason`, `needs_reopen`). On a closed day Edit reads "دن کھولیں اور درست کریں". A locked entry says why in words. |
| `EditBand`                                         | `shared/ui`              | The amber band: "You are changing Sale 5 (10-10-26)", the day that cannot change, the days reopened for the fix.                                                                                                           |
| `EditReviewCard`                                   | `shared/ui`              | What was changed and its effect, from the backend's `EditReview`. Partner figures behind the code show the locked box.                                                                                                     |
| `useEditReview`                                    | here                     | Asks the review 50 ms after the last key.                                                                                                                                                                                  |
| `EditReasonDialog`                                 | `shared/ui`              | The reason, asked when `reason_required`.                                                                                                                                                                                  |
| `EditStampLine`, `EditedMark`, `EditHistoryDialog` | `shared/ui`              | The stamp in a header, on a row, and the history of an entry.                                                                                                                                                              |
| `useFixDayFlow`, `FixDayDialog`, `FixDayHost`      | here, `shared/ui`, `app` | Reopen the days with a reason, edit, close them again after the save or when the user leaves.                                                                                                                              |
| `statusWithEdited`, `editedTip`                    | here                     | The "Edited" word and tip of a list row.                                                                                                                                                                                   |

Rules:

- Nothing here works out money or decides whether a day is closed. The backend sends the flags and the review.
- The field names of a change come from Fluent: `edit-field-<key>` (key with dashes). A key the app has no words for is shown as "other field".
- A screen with a new entry kind adds its kind to `EditBand` and its flags to the row; it does not build a new band, button pair or reason box.
- **The date of the reopen.** Screens hold a day as the backend sent it (`YYYY-MM-DD`); `day_fix_open` reads `DD-MM-YYYY`. `useFixDay.open` writes it as typed, once, for every screen (Umar, 13-11-2026). A new caller gives `useFixDayFlow.start` its day as it has it, and adds a test that reads the date given to `dayFixOpen` (`tests/helpers/fix-day.ts`).
- **The Owner's sign (plan 73, PRD FR-21.4, FR-7.71).** The reopen box asks for the reason and below it the sign (دستخط): raw text, required, sent as typed. `ERR_SIGN_WRONG` and `ERR_SIGN_REQUIRED` are told under the field and nothing opens. An edit made through a reopened day has `sign` on its stamp and its history versions; `signOf` and `whenWhoMessage` (`sign.ts`) are the one place that reads it, and every "when and by whom" line uses them (stamp, row tip, history, field marks). Settings, Security changes the sign with the Owner's password (`OwnerSignCard`).
