### Backup and restore (plan 18, PRD Section 20, NFR-BAK-1 to NFR-BAK-15, NFR-INS-2).
### The screen, the list of copies, the restore, the test restore, the backup key and its wizard
### step, the status bar item, the settings, and the window-close overlay.

## The screen.

# review: pending
backup-title = Backup

# review: pending
backup-help = Copies of your data are made by themselves. Here you can see them, make one now, and get your data back.

# review: pending
backup-load-failed = The state of the backups could not be opened

# review: pending
backup-now = Back up now

# review: pending
backup-now-done = Backup finished

# review: pending
backup-test-now = Test restore

## The state of the backups.

# review: pending
backup-state-title = State of the backups

# review: pending
backup-state-local = Last copy on this computer

# review: pending
backup-state-usb = Last copy on the USB drive

# review: pending
backup-state-cloud = Last copy in the cloud

# review: pending
backup-state-queue = Cloud copies waiting for the internet

# review: pending
backup-state-last-test = Last test restore

# review: pending
backup-state-never = None yet

# review: pending
backup-state-off = Switched off

# review: pending
backup-state-usb-missing = The USB drive is not plugged in

# review: pending
backup-state-cloud-old = Older than one day

# review: pending
backup-state-cloud-fresh = Up to date

# review: pending
backup-state-running = A backup or restore is running. This page follows it.

# review: pending
backup-state-test-due = No test restore has worked in the last 30 days. A test restore proves that your copies can really bring your data back.

# The daily readable pack (PRD FR-20.5): the day book PDF waits for the reports, the spreadsheets for D5-Q1.
# review: pending
backup-state-pack-pending = The daily readable pack (the day book as a PDF, and spreadsheets) is not made yet.

## What "back up now" did.

# review: pending
backup-run-ok = copy made and checked

# review: pending
backup-run-queued = copy is waiting for the internet. It is sent by itself.

# review: pending
backup-run-failed = copy failed.

## Places.

# review: pending
backup-kind-local = This computer

# review: pending
backup-kind-usb = USB drive

# review: pending
backup-kind-cloud = Cloud

## The list of copies (PRD FR-20.8).

# review: pending
backup-list-title = Copies

# review: pending
backup-list-label = Backup copies

# review: pending
backup-filter-label = Show copies from

# review: pending
backup-filter-all = All

# review: pending
backup-filter-local = This computer

# review: pending
backup-filter-usb = USB drive

# review: pending
backup-filter-cloud = Cloud

# review: pending
backup-list-failed = The list of copies could not be opened

# review: pending
backup-list-empty = No copies yet

# review: pending
backup-list-empty-help = Press the button to make the first copy now.

# $shown and $total are whole numbers as text.
# review: pending
backup-list-count = Showing { $shown } of { $total } copies

# review: pending
backup-list-more = Show more

# review: pending
backup-list-how = Select a copy and press Enter, or double-click it, to restore it.

# review: pending
backup-restore-selected = Restore selected

# review: pending
backup-test-selected = Test selected

# review: pending
backup-list-cloud-unreachable = The cloud could not be reached, so its copies are not in this list.

# review: pending
backup-col-created = Made on

# review: pending
backup-col-place = Place

# review: pending
backup-col-size = Size

# review: pending
backup-col-status = Check

# review: pending
backup-col-locked = Locked with key

# review: pending
backup-col-name = File

# review: pending
backup-status-good = Good

# review: pending
backup-status-bad = Failed its check

# review: pending
backup-status-unchecked = Not checked yet

# review: pending
backup-locked-yes = Yes

# review: pending
backup-locked-no = No

# $value is a number as text, $unit is one of the units below.
# review: pending
backup-size-value = { $value } { $unit }

# review: pending
backup-size-bytes = bytes

# review: pending
backup-size-kb = KB

# review: pending
backup-size-mb = MB

# review: pending
backup-size-gb = GB

## Restore (PRD FR-20.9).

# review: pending
backup-restore-title = Restore this backup

# review: pending
backup-restore-help = Your books are replaced by the books in this copy. Look at what is in it first.

# review: pending
backup-restore-key-help = This copy is locked with the backup key, and this computer does not have the key. Type the key from your printed sheet.

# review: pending
backup-restore-key-label = Backup key

# review: pending
backup-restore-key-hint = Copy it exactly as printed, with the dashes.

# review: pending
backup-restore-key-continue = Continue

# review: pending
backup-restore-preview-failed = This copy could not be read

# review: pending
backup-restore-made-on = This copy was made on <num>{ $date }</num>.

# review: pending
backup-restore-totals-label = Books in this copy and books now

# review: pending
backup-restore-lost-none = Nothing will be lost: there is nothing newer than this copy.

# $journal, $bikri and $vouchers are the records newer than the copy, as numbers.
# review: pending
backup-restore-lost = Everything entered after this copy will be lost: <num>{ $journal }</num> posted entries, <num>{ $bikri }</num> { -bikri } entries and <num>{ $vouchers }</num> vouchers.

# review: pending
backup-restore-upgrade = This copy is from an older version of the app. It is upgraded by itself after the restore.

# review: pending
backup-restore-safety = Before anything is replaced, a safety copy of your present data is made and checked. You are signed out after the restore and sign in again.

# review: pending
backup-restore-button = Restore

# review: pending
backup-restore-confirm-title = Restore this backup?

# $lost is the number of records newer than the copy; the choice only tells none from some.
# review: pending
backup-restore-confirm-description =
    { $lost ->
        [0] A safety copy of your present data is made first. Then all your books are replaced by the copy made on <num>{ $date }</num>. You are signed out.
       *[other] A safety copy of your present data is made first. Then all your books are replaced by the copy made on <num>{ $date }</num>. These will be lost: <num>{ $journal }</num> posted entries, <num>{ $bikri }</num> { -bikri } entries and <num>{ $vouchers }</num> vouchers. You are signed out.
    }

# review: pending
backup-restored = Restored from the copy of { $date }. Please sign in again.

## The books of a copy next to the books now.

# review: pending
backup-totals-what = Books

# review: pending
backup-totals-in-backup = In this copy

# review: pending
backup-totals-now = Now

# review: pending
backup-totals-restored = Restored test copy

# review: pending
backup-totals-live = Live data

# review: pending
backup-totals-accounts = Accounts

# review: pending
backup-totals-bikri = { -bikri } entries

# review: pending
backup-totals-vouchers = Vouchers

# review: pending
backup-totals-journal = Posted entries

# review: pending
backup-totals-last-entry = Date of the newest entry

# review: pending
backup-totals-debit = Total { -banaam }

# review: pending
backup-totals-credit = Total { -jama }

# review: pending
backup-totals-cash = Cash in hand

# review: pending
backup-totals-none = None

## Test restore (PRD NFR-BAK-11).

# review: pending
backup-test-title = Test restore

# review: pending
backup-test-help = The copy is restored into a separate place and checked. Your real data is not touched.

# review: pending
backup-test-running = Restoring the copy into a test place and checking it. Please wait.

# review: pending
backup-test-again = Test again

# review: pending
backup-test-failed = The test restore could not run

# review: pending
backup-test-close = Close

# review: pending
backup-test-ok = This copy is good. It restores correctly and its books balance.

# review: pending
backup-test-not-ok = This copy failed a check. Do not rely on it. Make a new copy now.

# review: pending
backup-test-totals-label = Books in the test copy and live books

# review: pending
backup-check-opens = The copy opens

# review: pending
backup-check-checksum = The copy is not damaged

# review: pending
backup-check-integrity = The data file is sound

# review: pending
backup-check-counts = The number of records is right

# review: pending
backup-check-balanced = The books balance

# review: pending
backup-check-passed = Passed

# review: pending
backup-check-failed = Failed

## The backup key (PRD FR-20.10, NFR-BAK-7).

# review: pending
backup-key-title = Backup key

# review: pending
backup-key-help = This key locks the copies on the USB drive and in the cloud. Without it nobody can open them, not even us. It is shown only once. Print it and keep it in a safe place, away from this computer.

# review: pending
backup-key-failed = The backup key could not be made

# review: pending
backup-key-already = A backup key is in use already, so a new one cannot be made. The key is not shown again.

# review: pending
backup-key-continue = Continue

# review: pending
backup-key-made-on = Made on <num>{ $date }</num>

# review: pending
backup-key-printed = I have printed the key and put it in a safe place

# review: pending
backup-key-print = Print

# review: pending
backup-key-type-label = Type the key from the printed sheet

# review: pending
backup-key-type-hint = This proves that you have the key. Copy it exactly as printed, with the dashes.

# review: pending
backup-key-confirm = Confirm the key

# review: pending
backup-key-confirmed = The backup key is confirmed.

# review: pending
backup-key-later = Do this later

# review: pending
backup-key-later-help = The key is not kept if you leave now. You can make a new one on the Backup screen.

# review: pending
backup-key-sheet-title = Backup key

# review: pending
backup-key-sheet-note = Keep this sheet in a safe place, away from the computer. Anyone who has this key can open the copies of your data. If it is lost, the copies on the USB drive and in the cloud cannot be opened.

# review: pending
backup-key-make = Make the backup key

# review: pending
backup-key-missing = There is no backup key yet. Without it, copies cannot be locked and sent to the USB drive or the cloud.

# review: pending
backup-key-confirm-due = You made the backup key more than 7 days ago and have not confirmed it. Type it from your printed sheet.

# review: pending
backup-key-unconfirmed = You have not confirmed the backup key yet. Type it from your printed sheet to prove that you have it.

# review: pending
backup-key-lost = I lost the printed sheet

# review: pending
backup-key-lost-help = A new key can be made only while no copy on the USB drive or in the cloud is locked with the old one.

# review: pending
backup-wizard-step-key = Backup key

# review: pending
backup-wizard-key-title = Print the backup key and keep it safe

## The status bar (PRD FR-5.5, NFR-BAK-10).

# review: pending
backup-bar-ok = Cloud backup: <num>{ $date }</num>

# review: pending
backup-bar-ok-queue = Cloud backup: <num>{ $date }</num>, <num>{ $queued }</num> waiting

# review: pending
backup-bar-old = Cloud backup is old: <num>{ $date }</num>

# review: pending
backup-bar-never = No cloud backup yet

# review: pending
backup-bar-check-failed = A backup failed its check

# review: pending
backup-bar-unknown = Backup: state unknown

## The copy at exit (PRD FR-20.3).

# review: pending
backup-exit-title = Saving a backup before closing

# review: pending
backup-exit-help = The app closes by itself as soon as the backup is done. Please wait.

## Settings (PRD FR-20.2 to FR-20.4, Section 21).

# review: pending
backup-set-title = Backup settings

# review: pending
backup-settings-failed = The backup settings could not be opened

# review: pending
backup-settings-saved = Backup settings saved

# review: pending
backup-set-local-dir = Folder for the copies on this computer

# review: pending
backup-set-local-dir-hint = Leave it empty to use <code>{ $folder }</code>. Use a full path. A folder on a second disk keeps your copies if this disk fails.

# review: pending
backup-set-closing-time = Daily copy time

# review: pending
backup-set-closing-time-hint = Hours and minutes, for example 21:00. The USB and cloud copies are made at this time every day.

# review: pending
backup-set-usb-on = Make a copy on a USB drive

# review: pending
backup-set-usb-folder = Folder on the USB drive

# review: pending
backup-set-usb-folder-hint = The full path, for example E:\MandiBackup

# review: pending
backup-set-cloud-on = Make a copy in the cloud

# review: pending
backup-set-endpoint = Cloud address

# review: pending
backup-set-endpoint-hint = For example https://your-account.r2.cloudflarestorage.com

# review: pending
backup-set-bucket = Bucket name

# review: pending
backup-set-region = Region

# review: pending
backup-set-region-hint = Cloudflare R2 uses auto.

# review: pending
backup-set-prefix = Folder inside the bucket

# review: pending
backup-set-prefix-hint = A name for all the copies of this shop, for example mandi.

# review: pending
backup-set-key-id = Cloud access key

# review: pending
backup-set-secret = Cloud secret key

# review: pending
backup-set-key-stored = A key is stored. Type a new one to replace it.

# review: pending
backup-set-key-none = No key is stored.

# review: pending
backup-set-key-will-remove = The stored key will be removed when you save.

# review: pending
backup-set-key-remove = Remove the stored key
