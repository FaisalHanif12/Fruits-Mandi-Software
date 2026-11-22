# Morning summary, 22-11-2026 (for Umar)

## Done and pushed (branch fix/gardens-new-form)

Plans 81 to 91 and the small tweaks: garden form, assistant speed and accuracy, sales page, analytics measure, closed day with the sign, dropdowns, capital card, buyer peshgi, voucher edit, other-shop goods, seller paid on the spot (with the parchi lines), crate-only logo, R5 table lines, settings spacing, see the backup key again, backup and restore proven (PASS WITH MINOR), books-check bugs fixed, stale tests cleaned.

## Assistant (plan 82)

- 93 of 98 complex questions right on the whole list (35 answered by the app itself, all right, about 0.1 s); no wrong figure.
- Model answers: p50 7.0 s, p95 17.9 s, max 25.6 s, no timeout, none over 30 s. Target p95 under 15 s not yet met.
- Not run live on the newest code: 11 questions (Y03, Y04, Y11, M18, D25 are the open ones), because the Mac Keychain prompt for the OpenRouter key needs a click. Run them with the commands in backend/target/ai-eval-complex/report-final.md after clicking Allow.

## Needs your click or decision

1. Mac Keychain prompts: click Allow when the live check and the dev-copy backup check run (two tests).
2. Should a wrong sign have its own lockout after several tries (like the password)? **Decided by Umar, 22-11-2026: no lockout. A wrong sign opens nothing and says "دستخط درست نہیں" (already so, ERR_SIGN_WRONG).**
3. A cancelled بکری row in پچھلی بکری now shows its buyer total (it showed 0 before). Keep it or show 0? **Decided by Umar, 22-11-2026: keep the buyer total as now (the row stays marked منسوخ).**
4. Plan 91: an edit asks the seller-paid question again for an amount already confirmed; fine?
5. The OS app icon (installer) still has the fruit; change it to the crate too? **Decided by Umar, 22-11-2026: the installer icon stays as it is.**
6. Merge: open the pull request for fix/gardens-new-form and choose "Create a merge commit".
