### Settings: firm, language and text size, auto lock (plan 04, PRD Section 21).

# review: pending
settings-title = Settings

# review: pending
settings-tabs = Settings sections

# review: pending
settings-tab-firm = Firm

# review: pending
settings-tab-language = Language and text

# review: pending
settings-tab-lock = Auto lock

# review: pending
settings-saved = Settings saved

# review: pending
settings-load-failed = Settings could not open

# review: pending
settings-firm-name-ur = Firm name (Urdu)

# review: pending
settings-firm-name-en = Firm name (English)

# review: pending
settings-firm-mandi-ur = Mandi (Urdu)

# review: pending
settings-firm-mandi-en = Mandi (English)

# review: pending
settings-firm-phone = Phone

# review: pending
settings-firm-address = Address

# review: pending
settings-language-default = Default language

# review: pending
settings-language-default-help = This language is used for the sign-in screen and the daily backup pack. You change your own language from the top bar.

# review: pending
settings-text-size = Text size

# review: pending
settings-text-size-help = This size is kept for you.

# review: pending
settings-text-size-value = Current size: <num>{ $size }%</num>

# review: pending
settings-text-bigger = Bigger

# review: pending
settings-text-smaller = Smaller

# review: pending
settings-text-normal = Normal size

# review: pending
settings-lock-minutes = Auto lock after (minutes)

# review: pending
settings-lock-hint = The app locks after this many minutes with no key pressed. Your work stays as it was.

# review: pending
settings-tab-printing = Printing

## Plan 22 (D5 early track): the Day group, number grouping, the books check, updates and the support file (PRD 21, FR-21.3, NFR-INS-3, NFR-OBS-4, NFR-OBS-5).

# review: pending
settings-tab-day = Day and check

# review: pending
settings-tab-audit = Audit log

# review: pending
settings-tab-app = App and updates

# review: pending
settings-day-title = Closing time and automatic close

# review: pending
settings-day-closing-time = Closing time

# review: pending
settings-day-closing-time-hint = Hours and minutes in 24-hour time, for example 21:00, Pakistan time. The daily cloud and USB backup runs at this time. It is the same time as on the backup screen.

# review: pending
settings-day-auto-close = Close the day by itself at closing time

# review: pending
settings-day-auto-close-note = This is only remembered for now: the app does not close the day by itself yet. Close the day yourself, as before.

# review: pending
settings-grouping-title = Number grouping

# review: pending
settings-grouping-help = How big amounts are written on the screen and on prints. In the lakh style the digits after the first three go in pairs.

# review: pending
settings-grouping-international = International (<num>1,316,040</num>)

# review: pending
settings-grouping-lakh = Lakh (<num>13,16,040</num>)

# review: pending
settings-check-title = Books check

# review: pending
settings-check-help = Checks that the books balance, that nobody has changed the records outside the app, that the data file is healthy and that the backups are recent. It also runs by itself about once a day.

# review: pending
settings-check-run = Check now

# review: pending
settings-check-running = Checking...

# review: pending
settings-check-none = Nothing has been checked yet. Press "Check now".

# review: pending
settings-check-failed = The check could not be read.

# review: pending
settings-check-last = Last checked: { $time }

# review: pending
settings-check-ok = Everything is in order

# review: pending
settings-check-bad = Something needs your attention

# review: pending
settings-check-books = Books

# review: pending
settings-check-books-balanced = The { -banaam } and { -jama } totals are equal (<num>{ $entries }</num> entries checked).

# review: pending
settings-check-books-difference = The { -banaam } and { -jama } totals differ by { $difference }. Contact support.

# review: pending
settings-check-books-not-run = The books could not be checked. Try again. If it happens again, contact support.

# review: pending
settings-check-journal-ok = No posted entry has been changed outside the app.

# $entry is the first entry that does not match, or the word none.
# review: pending
settings-check-journal-bad = The posted entry { $entry } does not match its seal. Someone may have changed the records outside the app. Contact support.

# review: pending
settings-check-journal-bad-unknown = A posted entry does not match its seal. Someone may have changed the records outside the app. Contact support.

# review: pending
settings-check-audit-ok = The audit log has not been changed.

# $entry is the first entry that does not match, or the word none.
# review: pending
settings-check-audit-bad = The audit entry { $entry } does not match its seal. Someone may have changed the records outside the app. Contact support.

# review: pending
settings-check-audit-bad-unknown = An audit entry does not match its seal. Someone may have changed the records outside the app. Contact support.

# review: pending
settings-check-data = Data file

# review: pending
settings-check-data-ok = The data file passed its check at start-up.

# review: pending
settings-check-data-checking = The data file is still being checked.

# review: pending
settings-check-data-failed = The data file check failed. Do not enter new work. Restore the last good backup and contact support.

# review: pending
settings-check-data-reference = Reference: <code>{ $reference }</code>

# review: pending
settings-check-backup = Backups

# review: pending
settings-check-backup-unknown = The state of the backups is not known.

# review: pending
settings-check-backup-off = Cloud backup is not set up yet.

# review: pending
settings-check-backup-never = No cloud copy has been made yet.

# review: pending
settings-check-backup-green = The last cloud copy is { $hours } hours old.

# review: pending
settings-check-backup-red = The last cloud copy is { $hours } hours old: more than a day.

# review: pending
settings-check-backup-alert = The last cloud copy is { $hours } hours old: more than two days.

# review: pending
settings-check-backup-queue =
    { $count ->
        [one] { $count } copy is waiting to be sent.
       *[other] { $count } copies are waiting to be sent.
    }

# review: pending
settings-check-backup-errors =
    { $count ->
        [one] { $count } backup problem. Look at the backup screen.
       *[other] { $count } backup problems. Look at the backup screen.
    }

# review: pending
settings-app-title = Updates

# review: pending
settings-app-version = Version { $version }

# review: pending
settings-update-help = The app looks for a new version when it can reach the internet. It installs one only when you agree, never in the middle of work, and only after a full backup.

# review: pending
settings-update-checking = Looking for a new version...

# review: pending
settings-update-up-to-date = The app is up to date.

# review: pending
settings-update-available = A new version is available: { $version }

# review: pending
settings-update-date = Released: { $date }

# review: pending
settings-update-notes = What is new

# review: pending
settings-update-no-notes = No notes came with this version.

# review: pending
settings-update-check-again = Look again

# review: pending
settings-update-install = Update now

# review: pending
settings-update-has-drafts = There are unsaved entries ({ $drafts }). Save them or throw them away before you update.

# review: pending
settings-update-confirm-title = Update to version { $version }?

# review: pending
settings-update-confirm-body = The app makes a full backup first, then installs the new version and closes. Open it again when it has closed. Update after you close the day, not in the middle of work. There are no unsaved entries now.

# review: pending
settings-update-confirm-yes = Yes, update

# review: pending
settings-update-installing = Updating. The app makes a safety copy, installs the new version and closes by itself. Do not switch off the computer.

# review: pending
settings-update-done = Version { $version } is installed. The safety copy is { $file }. The app closes now.

# review: pending
settings-support-title = Information for support

# review: pending
settings-support-help = Collects the log, the app version and the check results into one file that you can send to us. It has no password, no key and no balances.

# review: pending
settings-support-save = Save support file

# review: pending
settings-support-saved = The support file is saved as { $path }. It holds { $lines } log lines.

# review: pending
settings-support-path-refused = Choose a full file name ending in .txt, in a folder that exists.

# The name of the file type in the save box.
# review: pending
settings-support-file-type = Text file
