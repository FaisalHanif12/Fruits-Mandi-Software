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
