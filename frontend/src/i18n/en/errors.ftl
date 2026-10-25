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
# A journal entry: one line, or the total of the entry, is over the largest amount allowed.
# $max: the largest amount in rupees.
# review: pending
err-entry-too-large = This entry is too large. One line, or the total of an entry, can be at most Rs { $max }.
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
# PRD 10.2, Code Standards 4.2 (Tag). The code name is the backend's (plan 17e T1).
# review: pending
err-tag-invalid = The tag must be 1 to 4 English capital letters or dots, with at least one letter. For example K, BA or D.P.

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
# $total: the total of the typed shares as a percent, for example 90.
err-shares-not-100 = The { -shirakat-dar } { -hissa } total is { $total }%, not 100%. Correct it in { -shirakat-dar } setup.

# PRD FR-17.2, E18, A31. $today: today, DD-MM-YYYY.
# review: pending
err-share-backdated = The share change cannot start in the past. Enter today ({ $today }) or a later date.

# PRD FR-17.1. A partner with a share cannot be made inactive.
# review: pending
err-partner-has-share = This { -shirakat-dar } still has a { -hissa }. Set his { -hissa } to 0 and save the shares first, then make him inactive.

# PRD FR-17.1. The share list must name every active partner once.
# review: pending
err-partner-not-listed = The share list must have every active partner once. Open the page again and try again.

# NFR-SCL-5. $max: the most partners.
# review: pending
err-too-many-partners = No more than { $max } partners can be added.

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
err-not-found = This was not found. Open the page again and choose again.
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

# plan 17b: an old client sent a mode, a bank or a cheque; the screens never do (PRD 15.1).
# review: pending
err-field-not-allowed = This field is no longer used. Leave it out.

# PRD FR-10.9, FR-6.5, FR-13.5, C20. Income and shop expense accounts, Opening Difference and Pukhta Khata cannot take one. A partner's drawings account and a person's money lent account take an opening debit only.
# review: pending
err-opening-not-allowed = This account cannot take this opening balance. Income and { -dukan-kharcha } accounts, { -naqdi-mandi }, { -pukhta-khata } and Opening Difference cannot take one. A partner's { -zati-kharcha } account and a { -adhar-raqam } account can take an opening debit only.

## Advances and closing days (plan 10).

# PRD FR-15.4. An advance goes to a seller, never to Pukhta Khata.
# review: pending
err-not-a-seller = This is not a { -beopari } account. An { -peshgi } can only be given to a { -beopari }, not to { -pukhta-khata }.
# PRD FR-10.2. Hidden from pickers.
# review: pending
err-account-inactive = This account is inactive and cannot be used in a new entry. Choose another account.
# PRD FR-10.4. A system account is used by the books, so it stays active.
# review: pending
err-system-account = This is a system account. The books use it, so it cannot be made inactive.
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

## The sale screen (plan 12). Warnings use the same kind of message.

# PRD FR-12.6. A line sells more than the lot has left.
# review: pending
err-qty-exceeds-lot = The quantity is more than the lot has left. Only { $available } { -nag } are left.
# PRD FR-11.10. The setting is off.
# review: pending
err-override-not-allowed = Changing the { -commission } or { -laga } on a line is switched off. The Owner can switch it on in Settings.
# PRD FR-12.4. $limit and $would_owe are amounts in rupees.
# review: pending
err-credit-limit-exceeded = This sale takes the buyer's balance to { $would_owe }, which is over his credit limit of { $limit }.
# review: pending
err-seller-required = Choose the seller of this lot.
# review: pending
err-buyer-required = Choose the buyer of this line.
# PRD FR-13.2.
# review: pending
err-pukhta-name-required = A name is required for { -pukhta-khata }.
# PRD FR-13.2.
# review: pending
err-pukhta-phone-required = A phone number is required for this { -pukhta-khata } person.
# PRD FR-12.8.
# review: pending
err-no-lots = Add at least one lot.
# PRD FR-12.8.
# review: pending
err-no-sale-lines = Add at least one sale line.
# PRD E23.
# review: pending
err-weight-required = Enter the weight in kilograms. This item is sold by weight.
# PRD FR-6.7. $cash_after is cash in hand after the entry, in rupees.
# review: pending
err-cash-below-zero-not-confirmed = After this entry cash in hand would be { $cash_after }. Confirm the entry to save it.
# review: pending
err-not-a-buyer = This is not a { -gahak } account. Choose a buyer.
# PRD 12.4, 10.2. The buyer's credit is off, or he is the nameless cash buyer.
# review: pending
err-credit-not-allowed = Credit is not allowed for this buyer. Change the line to cash.
# A warning (PRD FR-12.8): the rest goes to باقی مال.
# review: pending
err-qty-left-unsold = { $left } { -nag } are left. They will stay in { -baqi-maal }.
# A warning (PRD FR-12.7): the save is allowed. $net is in rupees.
# review: pending
err-lot-net-negative = The { -safi-bikri } of this lot is below zero ({ $net }). The seller owes this amount.
# A warning (PRD FR-7.55). $average is in rupees, $percent is the setting.
# review: pending
err-rate-far-from-average = This rate is more than { $percent }% away from the average of the last 7 days ({ $average }). Check it.

## Drafts (plan 13, PRD FR-12.10).

# $max_bytes: the most the app keeps for one unfinished entry. The number is not shown.
# review: pending
err-draft-invalid = Your unfinished work could not be kept for now. Save the entry now. If this happens again, tell us.

## Printing (plan 14).

# PRD FR-12.12. Printing needs Windows; on other computers the preview shows how it will print.
# review: pending
err-print-windows-only = Printing works only on Windows. On this computer, use the preview to see how the document will print.

# $printer is the printer name in Settings.
# review: pending
err-printer-not-found = The printer { $printer } was not found. Check that it is on and connected, or choose another printer in Settings.

# review: pending
err-print-failed = The document could not be printed. Check the printer and try again. If it keeps happening, ask for help.

## Vouchers (plan 15, PRD 15.1 to 15.4).

# PRD FR-15.11. Money paid to a buyer is cash lent to him.
# review: pending
err-payment-to-buyer-needs-reason = Money paid to a { -gahak } is { -naqd-udhar }. Please write the reason.

# PRD FR-15.12, FR-16.2. $open is the most that can be returned, in rupees.
# review: pending
err-return-exceeds-open = This is more than can be returned. The most that can be returned is { $open }.

# PRD FR-15.3.
# review: pending
err-voucher-cancelled = This voucher is cancelled and cannot be changed.

# PRD FR-13.5, FR-13.7.
# review: pending
err-person-required = For { -pukhta-khata }, type the name or phone of the person and pick him from the list.

# A warning (PRD FR-15.7): the save is allowed. $payable is what is payable, $over how much the
# amount is more than it, both in rupees.
# review: pending
err-payment-above-payable = This is { $over } more than what is payable to the { -beopari } ({ $payable }). You can still save it. The extra shows as owed by the { -beopari }; it is not an { -peshgi }.

## Later sales, edit, cancel and write-off (plan 16).

# review: pending
err-bikri-cancelled = This { -bikri } is cancelled and cannot be changed.

# review: pending
err-lot-not-found = This lot was not found. Open the list of { -baqi-maal } and choose again.

# review: pending
err-nothing-to-sell = Nothing is left in this lot to sell.

# $available: how many { -nag } are left in the lot.
# review: pending
err-write-off-exceeds-balance = You cannot write off more than is left. Only { $available } { -nag } are left.

# review: pending
err-date-before-received = The day cannot be before the day the goods were received.

# review: pending
err-qty-below-sold = The quantity received cannot be less than the quantity already sold from this lot.

# PRD FR-12.18 to FR-12.20. The edit rules of a saved { -bikri }. $rule is a fixed name from the
# backend, one case each: the day, a lot, its order, its seller or item, a line, a later sale's lot.
# Every case says what to do. The field points at the lot or the line.
# review: pending
err-edit-not-allowed = { $rule ->
    [day_changed] The day of a saved entry cannot be changed in an edit. Write the old day back.
    [lot_removed] A saved lot cannot be removed, because its sales and its goods rest on it. Put the lot back. To take the whole sale back, cancel it.
    [lot_not_in_posting] This lot is not part of the entry being edited. Close the edit and open the entry again.
    [lot_order_changed] Saved lots keep their order, and new lots come after them. Move the new lot to the end.
    [lot_seller_locked] A later entry or sale already rests on this lot, so its seller cannot be changed. Choose the old seller again.
    [lot_item_locked] A later entry or sale already rests on this lot, so its item cannot be changed. Choose the old item again.
    [line_not_in_posting] This line is not part of the entry being edited. Close the edit and open the entry again.
    [posting_lots_locked] In a later sale only its own lines can be changed, not the lot. Change the lot in the first entry.
   *[other] This change is not allowed in this entry. Close the edit and open the entry again.
}

## Trip goods and trips (plan 16b, PRD 12.10, FR-12.24, FR-12.26, FR-12.29, FR-12.31).

# PRD FR-12.26. $qty_left: how many { -nag } of the quantity taken are not sold on the lines.
# review: pending
err-own-lot-not-fully-sold = This lot of { -apna-maal } must be sold in full in this { -bikri }. { $qty_left } { -nag } are not sold on the lines yet. Sell them on the lines, or take fewer { -nag } from the purchase. What you do not take stays in the purchase.

# PRD FR-12.26. The account offered only as the seller of a lot.
# review: pending
err-own-goods-account-restricted = { -apna-maal } can only be the seller of a lot in the { -bikri }. Choose another account.

# PRD FR-12.24. $qty_sold and $qty_written_off: how many { -nag }.
# review: pending
err-purchase-has-sales = This purchase cannot be cancelled, because { $qty_sold } { -nag } were sold and { $qty_written_off } { -nag } were written off. Cancel the sales or undo the write-off first.

# review: pending
err-purchase-cancelled = This purchase is cancelled. Choose another purchase.

# A warning, not a refusal (PRD FR-12.26).
# review: pending
err-own-lot-freight-in-cost = The freight of the purchase is already in its cost. A freight typed here is counted a second time. Leave it empty unless that is what you mean.

# PRD FR-12.31, FR-12.33: a closed trip takes no new purchase or cost.
# review: pending
err-trip-closed = This { -market-safar } is closed. It takes no new purchase or cost.

# PRD FR-12.31: a trip cost is spread over the purchases that still have goods left.
# review: pending
err-trip-no-stock = No purchase of this { -market-safar } has goods left, so there is nothing to spread the cost over. Record the purchase first.

# PRD FR-12.29: the shares are fixed after the first entry.
# review: pending
err-venture-shares-locked = This { -market-safar } already has an entry, so the { -hissa } of its partners can no longer be changed.

# PRD FR-12.24. $trip_date: the day of the trip.
# review: pending
err-purchase-before-trip = The day is before the day of the { -market-safar } ({ $trip_date }). Choose the day of the trip or a later day.

# PRD FR-12.31. $qty_sold and $qty_written_off: how many { -nag } of the purchases it was spread over.
# review: pending
err-trip-cost-has-sales = This cost cannot be cancelled, because { $qty_sold } { -nag } of the purchases it was spread over were sold and { $qty_written_off } { -nag } were written off. Cancel the sales or undo the write-off first.

# PRD A63: a partner cannot be made inactive while he is in an open trip.
# review: pending
err-partner-in-open-venture = This { -shirakat-dar } is in an open { -market-safar }, so he cannot be made inactive. Let the { -market-safar } close first.

# PRD FR-12.31: cancel the cost first.
# review: pending
err-purchase-carries-trip-cost = This purchase carries a part of a { -market-safar } cost, so it cannot be cancelled. Cancel that cost first.

# review: pending
err-trip-entry-cancelled = This entry is cancelled already.

## Backup and restore (plan 18, PRD Section 20). The codes of `BackupCode` that core has not got yet.
## The seven that core has already (disk full, internal, data check, busy, required, out of range,
## migration) share the messages above.

# review: pending
err-backup-not-found = This copy was not found. It may have been removed. Refresh the list and try again.

# review: pending
err-backup-key-not-set = There is no backup key on this computer yet. Make the key first.

# review: pending
err-backup-key-already-set = A backup key is in use already, so a new one cannot be made.

# review: pending
err-backup-key-format = The key is not written the right way. It has six groups of four letters and numbers, with dashes between them.

# review: pending
err-backup-key-wrong = This is not the right key. Check the printed sheet and try again. Nothing was changed.

# review: pending
err-backup-key-required = This copy is locked with the backup key. Type the key from your printed sheet.

# review: pending
err-backup-corrupt = This copy is damaged and cannot be used. Nothing was changed. Use another copy.

# review: pending
err-backup-check-failed = The copy was made but did not pass its check, so it cannot be trusted. Make a new copy.

# review: pending
err-backup-cloud-not-configured = The cloud copy is not set up. Fill in the cloud details in the backup settings.

# $reason is offline, refused or not_found.
# review: pending
err-backup-cloud-failed =
    { $reason ->
        [offline] The cloud could not be reached. Check the internet. The copy waits and is sent again by itself.
        [refused] The cloud refused the copy. Check the cloud keys in the backup settings.
        [not_found] The cloud bucket or the copy was not found. Check the bucket name in the backup settings.
       *[other] The cloud copy failed. Try again later.
    }

# review: pending
err-backup-usb-not-present = The USB drive is not plugged in. Plug it in, or switch the USB copy off in the backup settings.

# review: pending
err-backup-usb-failed = The copy to the USB drive failed. Check the drive and the folder.

# review: pending
err-backup-local-failed = The copy on this computer failed. Check the folder in the backup settings and the free space on the disk.

# review: pending
err-backup-busy = A backup or restore is running already. Wait until it is done.

# review: pending
err-backup-not-confirmed = Please confirm before the restore starts. Nothing was changed.

# review: pending
err-backup-newer-than-app = This copy is from a newer version of the app than the one on this computer. Update the app first.

# review: pending
err-backup-not-started = The backup service is not running yet. Wait a moment and try again.

# review: pending
err-backup-restore-failed = The restore could not be finished. Close the app and open it again. If your books look wrong, contact support: a safety copy was made before the restore.

## Plan 17: the cash book area (PRD 16, FR-16.9, FR-17.8, 16.4).

# PRD FR-16.9. The same day is allowed.
# review: pending
err-return-by-before-entry = The return-by date cannot be before the entry date.

# review: pending
err-partner-required = Choose the { -shirakat-dar }.

# review: pending
err-expense-head-required = Choose the expense head.

# review: pending
err-lent-person-required = Choose the person.

# review: pending
err-khata-required = Choose the { -alag-khata }.

# PRD 16.4. Nothing is saved. $dr and $cr are the two totals in rupees.
# review: pending
err-general-not-balanced = The { -banaam } total is { $dr } and the { -jama } total is { $cr }. They must be equal before the entry can be saved.

# review: pending
err-general-too-few-lines = A { -general } entry needs at least two lines.

# PRD FR-17.8. A warning only: the entry can still be saved. $balance is the capital balance and $over is the part above it, in rupees.
# review: pending
err-capital-exceeds-balance = This is more than the partner's { -sarmaya } balance of { $balance }, by { $over }. You can still save it.

## Plan 22: updates and the support file (PRD NFR-INS-3, NFR-INS-4, NFR-SEC-11, NFR-OBS-4).

# review: pending
err-update-check-failed = The app could not look for a new version. Check the internet connection and try again. Your work is not affected.

# review: pending
err-update-not-signed = This update is not signed by us, so it was not installed. Nothing was changed and no copy was made. Do not use this update.

# $drafts is the number of unsaved entries.
# review: pending
err-update-during-work = There are unsaved entries ({ $drafts }). Save them or throw them away, then update. Nothing was installed.

# review: pending
err-update-not-confirmed = The update was not confirmed, so nothing was installed. Press the update button and answer the question.

# review: pending
err-update-backup-failed = The safety copy before the update could not be made, so nothing was installed. Look at the backup screen and try again.

# review: pending
err-update-changed = A different version is on offer now. Look for the new version again and read what is new before you update.

# review: pending
err-update-none = There is no newer version. The app is up to date.

## Plan 17c: gardens (PRD 12.11, FR-12.34 to FR-12.38).

# PRD FR-16.12: the fruit entry was cancelled already (it cannot be changed or cancelled again).
# review: pending
err-fruit-cancelled = This fruit entry was cancelled already, so it cannot be changed.

# PRD 16.11, FR-12.22: fruit was taken from a lot of this sale, so the sale cannot be cancelled or undone.
# $qty_taken: the نگ taken from its lots.
# review: pending
err-bikri-has-fruit = { $qty_taken } { -nag } of { -phal } were taken from a lot of this { -bikri }, so it cannot be cancelled. Cancel the { -phal } entry first.

# PRD FR-12.38, E65. Also shown when an entry of a closed garden is cancelled, or a { -bikri } with a lot of it is edited or cancelled.
# review: pending
err-garden-closed = This { -bagh } is closed, so it takes no new entry, no lot and no fruit. Undo its closing first.

# PRD FR-12.38. The garden is open, so there is no closing to undo.
# review: pending
err-garden-not-closed = This { -bagh } is open, so there is no closing to undo.

# PRD FR-12.34. $garden_date: the day of the garden.
# review: pending
err-date-before-garden = The day is before the day of the { -bagh } ({ $garden_date }). Choose the day of the garden or a later day.

# Plan 17c. $last_date: the day of the latest entry or lot of the garden.
# review: pending
err-close-before-last-entry = The { -bagh } cannot be closed before its latest entry or lot ({ $last_date }). Choose that day or a later day.

# Plan 17c. The entry of the garden was cancelled already.
# review: pending
err-garden-entry-cancelled = This entry of the { -bagh } is cancelled already.
