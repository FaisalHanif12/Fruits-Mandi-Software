### One message per backend error code: ERR_SOME_CODE becomes err-some-code.
### Each message says what went wrong and what to do (NFR-USE-13). No technical words (NFR-USE-14).

# review: pending
err-not-implemented = This part is not ready yet. It comes in a later version.
# review: pending
err-internal = Something went wrong. Please try again.
# Shown under an error when the backend sends a reference number (NFR-OBS-3).
# review: pending
err-reference = For help, give the reference number <code>{ $ref }</code>.

## Input checks (plan 02, contract 4.6; Code Standards 4.2). Numbers in { $max } arrive as
## digits and are grouped before they reach the message.

# review: pending
err-required = This field is empty. Please fill it in.
# Shared by money, quantity, percent and weight fields (a percent or a weight may have decimals),
# so it says "a number". A decimal in money or a quantity has its own message.
# review: pending
err-not-a-number = Enter a number, for example 12,000. Letters and misplaced commas cannot be used.
# PRD E31.
# review: pending
err-decimals-not-allowed = Enter the amount in whole rupees.
# A quantity typed with a decimal point (tech-lead decision, plan 02). Money uses err-decimals-not-allowed.
# review: pending
err-qty-not-whole = Write the quantity in whole { -nag }.
# review: pending
err-negative-not-allowed = A negative number is not allowed. Write it without a minus sign.
# $max: the largest amount in rupees.
# review: pending
err-amount-too-large = The amount is too large. The most you can enter is Rs { $max }.
# PRD E24.
# review: pending
err-must-be-more-than-zero = Enter a number more than zero.
# $max: how many digits may follow the decimal point (2 for a percent, 3 for a weight).
# review: pending
err-too-many-decimals = Enter at most { $max } digits after the decimal point.
# review: pending
err-percent-over-100 = A percent cannot be more than 100.
# review: pending
err-qty-too-large = The quantity is too large. The most you can enter is { $max } { -nag }.
# $max: kilograms.
# review: pending
err-weight-too-large = The weight is too large. The most you can enter is { $max } kg.
# review: pending
err-date-invalid = This date is not valid. Enter the day, month and year correctly.
# PRD FR-6.6. $closed_up_to: DD-MM-YYYY.
# review: pending
err-date-closed = This day is closed. Enter a date after { $closed_up_to }, or ask the Owner to reopen the day.
# PRD FR-6.6. $today: DD-MM-YYYY.
# review: pending
err-date-in-future = A future date is not allowed. Enter today ({ $today }) or an earlier date.
# PRD FR-7.39.
# review: pending
err-code-invalid = This code is not valid. Enter the full code, or a short code of 1 to 4 digits.
# PRD FR-7.28.
# review: pending
err-name-ur-script = Write the Urdu name in Urdu letters.
# PRD FR-7.28.
# review: pending
err-name-en-script = Write the English name in English letters.
# review: pending
err-name-too-long = The name is too long. Use at most { $max } characters.
# review: pending
err-phone-invalid = This phone number is not valid. Enter an 11-digit number that starts with 03.
# review: pending
err-text-too-long = The text is too long. Use at most { $max } characters.
# review: pending
err-control-chars = This field has a hidden character, such as a tab, a line break or a text-direction mark. Delete it and type the text again.
# Code Standards M7.
# review: pending
err-overflow = The amount has become too large. Check the amounts and try again.
# PRD FR-17.1, E9.
# review: pending
err-shares-not-100 = The { -shirakat-dar } { -hissa } total is not 100%. Ask the Owner to correct it in { -shirakat-dar } setup.

## The data file (plan 03, contract 4.6; NFR-REL-6, NFR-REL-7, NFR-REL-10, NFR-REL-13).
## The reference number follows each of these messages (err-reference), so they do not repeat it.

# NFR-REL-10. Nothing was written.
# review: pending
err-disk-full = The disk is full. Free some space, then try again. Nothing was saved.
# NFR-REL-7. Saving stays stopped until the check passes.
# review: pending
err-data-check-failed = The data file did not pass its check, so saving is stopped to protect your data. Close the app and open it again. If this shows again, contact support.
# NFR-REL-13. The copy made before the update was put back.
# review: pending
err-migration-failed = The data could not be updated for the new version. Your data is safe: the copy made before the update was put back. Install the previous version, or contact support.
# The data file was in use by another task. Nothing was written.
# review: pending
err-db-busy = The data file is busy. Try again in a moment.

## Sign-in and settings (plan 04, contract 4. NFR-SEC-1, NFR-SEC-2, NFR-SEC-4, PRD FR-8.2).
## The reference number follows these messages when the backend sends one (err-reference).

# $min: the shortest password, in characters.
# review: pending
err-password-too-short = The password must be at least { $min } characters long.
# review: pending
err-passwords-differ = The two passwords are not the same. Type them again.
# review: pending
err-username-invalid = The user name must be 3 to 32 characters. Use only small English letters, digits, a dot, an underscore or a hyphen.
# The same message for a wrong password and an unknown name, so nobody learns which names exist.
# $tries_left: tries left before the sign-in waits.
# review: pending
err-wrong-password = The user name or password is wrong. Tries left: { $tries_left }
# PRD FR-8.2. $seconds: seconds left to wait. The sign-in screen counts down by itself.
# review: pending
err-locked-wait = Too many wrong tries. Wait { $seconds } seconds, then try again.
# review: pending
err-user-inactive = This account cannot sign in. Use the recovery code.
# review: pending
err-already-signed-in = You are already signed in.
# review: pending
err-not-locked = The app is not locked.
# review: pending
err-recovery-code-wrong = The user name or the recovery code is wrong. Check and type them again.
# review: pending
err-already-set-up = First-time setup is already done. Please sign in.
# $min and $max: the allowed range.
# review: pending
err-out-of-range = This value must be between { $min } and { $max }.
# review: pending
err-not-logged-in = Please sign in first.
# PRD FR-8.4.
# review: pending
err-session-locked = The app is locked. Type your password to unlock it.
# review: pending
err-not-found = This item was not found. Open the list again.
# PRD FR-7.66, E49.
# review: pending
err-spelling-unconfirmed = The spelling of these words is not confirmed: { $words }. Check the spelling, then confirm it or fix it.

## The master lists (plan 06).

# review: pending
err-code-taken = This code is already used by another one. Enter a different code.
# review: pending
err-version-changed = This was changed after you opened it. Open it again.
# review: pending
err-format-invalid = What is written here is not in the right form. Check it and write it again.
# review: pending
err-series-full = Every code of this series is used. Use another series.

## Accounts, the chart and opening balances (plan 07).

# PRD FR-6.2. Nothing was saved.
# review: pending
err-journal-unbalanced = The Debit and the Credit of this entry are not equal, so nothing was saved. Check the amounts and try again.
# PRD FR-10.3.
# review: pending
err-code-reserved = This code is kept for the app's own accounts. Enter a different code.
# PRD FR-10.2.
# review: pending
err-opening-both-sides = Enter an opening Debit or an opening Credit, not both.
# PRD FR-10.2, FR-15.8.
# review: pending
err-advance-not-seller = An opening Advance can only be set on the opening Debit of a Seller.

# PRD FR-10.9, FR-6.5, FR-13.5. Income and shop expense accounts, Opening Difference and Pukhta Khata cannot take one.
# review: pending
err-opening-not-allowed = This account cannot have an opening balance. Income and { -dukan-kharcha } accounts, { -naqdi-mandi }, { -pukhta-khata } and Opening Difference cannot take one.

## Advances and closing days (plan 10).

# PRD FR-15.4. An advance goes to a seller, never to Pukhta Khata.
# review: pending
err-not-a-seller = This is not a { -beopari } account. An { -peshgi } can only be given to a { -beopari }, not to { -pukhta-khata }.
# PRD FR-10.2. Hidden from pickers.
# review: pending
err-account-inactive = This account is inactive and cannot be used in a new entry. Choose another account.
# PRD FR-21.2.
# review: pending
err-reason-required = A reason is required.
# NFR-REL-8. Nothing was closed.
# review: pending
err-trial-not-balanced = The { -trial-balance } has a difference, so the day cannot be closed. Check the { -trial-balance } and fix the difference first.
# PRD FR-21.2.
# review: pending
err-day-not-closed = This day is not closed, so there is nothing to reopen. Enter the date of a closed day.
# PRD FR-17.6.
# review: pending
err-month-closed = The month of this date is closed. Reverse the month close before reopening the day.

# PRD FR-21.1. Every day up to today is closed already.
# review: pending
err-nothing-to-close = Every day up to today is already closed, so there is no day to close.

## Commission rules (plan 09).

# PRD 11.2, 11.3. The field says which part.
# review: pending
err-rule-not-allowed = This combination is not allowed for a rule. Choose a charge and a method from the lists, and a group only when no party or { -qism-maal } is chosen. The default rates are changed in Settings, Charges.
# review: pending
err-date-range = The last date is before the first date. Enter a last date on or after the first date.
