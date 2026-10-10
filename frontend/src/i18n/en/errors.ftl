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
