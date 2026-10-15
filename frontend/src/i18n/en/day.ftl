# Closing and reopening days (plan 10, PRD FR-21.1, FR-21.2).

# review: pending
day-title = Close the day

# review: pending
day-today = Today's date

# review: pending
day-closed-up-to = Closed up to

# review: pending
day-none-closed = No day is closed yet

# review: pending
day-load-failed = The state of the day could not be opened

## Close.

# review: pending
day-close-title = Close the day

# review: pending
day-close-help = After a day is closed, its entries cannot be changed unless the Owner reopens that day.

# review: pending
day-close-up-to = Close up to this date

# review: pending
day-close-days = Days that will close

# review: pending
day-close-from = First open day

# review: pending
day-close-entries = Entries in these days

# review: pending
day-close-trial-dr = { -trial-balance }: total { -banaam }

# review: pending
day-close-trial-cr = { -trial-balance }: total { -jama }

# review: pending
day-close-trial = Difference in the { -trial-balance }

# review: pending
day-close-cash = { -rokar } balance

# review: pending
day-close-nothing = No day is open up to this date.

# review: pending
day-close-button = Close the day

# review: pending
day-close-preview-failed = What closing would do could not be opened

# review: pending
day-close-confirm-title = Close the day?

# $date: the date to close up to. $days: how many days close.
# review: pending
day-close-confirm-description = { $days ->
    [one] { $days } day up to { $date } will be closed. Its entries cannot be changed until you reopen that day.
   *[other] { $days } days up to { $date } will be closed. Their entries cannot be changed until you reopen those days.
}

# review: pending
day-closed = Days up to { $date } are closed

## Reopen.

# review: pending
day-reopen-title = Reopen a day

# review: pending
day-reopen-help = Reopening a date reopens every later date too. A reason is required and is written to the audit log.

# review: pending
day-reopen-date = Date to reopen

# review: pending
day-reopen-reason = Reason

# review: pending
day-reopen-button = Reopen

# review: pending
day-reopen-confirm-title = Reopen the day?

# review: pending
day-reopen-confirm-description = { $date } and every later date will be reopened. The reason will be written to the audit log.

# review: pending
day-reopened = { $date } and the days after it are reopened

# review: pending
day-reopen-none-title = No day is closed yet

# review: pending
day-reopen-none-help = When you close a day, you can reopen it here.
