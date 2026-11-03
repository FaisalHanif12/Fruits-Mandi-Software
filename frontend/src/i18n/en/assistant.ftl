### Ask the app: the chat window above a round button at the end of every screen (plans 49 and 51, PRD 7.7, FR-7.72 to FR-7.88).
### Every Urdu line says what its English line says (NFR-URD-29). The words of an ANSWER are written by the
### model under the checks of FR-7.76; only the panel's own fixed words are here. Business words come from
### glossary.ftl.

## The chat window, its launcher and the status bar button (FR-5.5, FR-7.73).

# review: pending
assistant-button = Ask the app
# review: pending
assistant-button-count = Today <num>{ $used }</num> / <num>{ $limit }</num>
# review: pending
assistant-dock-label = Ask the app
# review: pending
assistant-dock-title = Ask the app
# review: pending
assistant-close = Close the chat window
# review: pending
assistant-larger = Make the window larger
# review: pending
assistant-smaller = Make the window smaller
# review: pending
assistant-state-ready = Ready
# review: pending
assistant-state-working = Preparing an answer
# review: pending
assistant-state-offline = No internet
# review: pending
assistant-state-off = Switched off
# review: pending
assistant-new-conversation = New question
# review: pending
assistant-conversation-label = Questions and answers
# review: pending
assistant-box-label = Type your question
# review: pending
assistant-box-placeholder = Type your question and press { $key }
# review: pending
assistant-send = Send
# review: pending
assistant-empty-title = Hello! Ask me anything about your books.
# review: pending
assistant-empty-help = For example: how much does Akram owe? How much { -commission } today? You can also start from the quick questions below.
# review: pending
assistant-off = Asking questions is off. You can turn it on in Settings. The quick questions work.
# review: pending
assistant-no-key = The OpenRouter key is not saved yet, so questions cannot be asked. Add the key in Settings. The quick questions work.
# review: pending
assistant-open-settings = Open Settings
# review: pending
assistant-status-failed = The state of asking questions could not be read
# review: pending
assistant-count-label = Questions asked today

## The working state: the first line at once, then one line per step (NFR-AI-3).

# review: pending
assistant-understanding = Understanding the question…
# review: pending
assistant-working-label = The answer is being prepared
# review: pending
assistant-step-generic = Preparing the answer…
# review: pending
assistant-step-finding = Looking for { $text }…
# review: pending
assistant-step-reading-card = Reading the card of { $name }…
# review: pending
assistant-step-reading-ledger = Reading the { -khata } of { $name }…
# review: pending
assistant-step-reading-report = Reading report { $report }…
# review: pending
assistant-step-reading-analytics = Reading { -tajziat }…
# review: pending
assistant-step-reading-tiles = Reading the dashboard figures…
# review: pending
assistant-step-period = Working out the dates of the period…
# review: pending
assistant-step-calculating = Calculating…
# review: pending
assistant-step-estimating = Working out the estimate…
# review: pending
assistant-step-checking = Checking the figures…
# review: pending
assistant-step-rewriting = Writing the answer again…

## The quick questions (FR-7.80). They need no internet and no model, and they are not counted.

# review: pending
assistant-chips-label = Quick questions
# review: pending
assistant-chip-sales = Today's { -bikri }
# review: pending
assistant-chip-commission = Today's { -commission }
# review: pending
assistant-chip-cash = Cash in hand
# review: pending
assistant-chip-dues = Buyers' dues
# review: pending
assistant-chip-total-dues = { -kul-agrahi }
# review: pending
assistant-chip-dead = { -doobi-raqam }
# review: pending
assistant-chip-payable = Sellers payable
# review: pending
assistant-chip-advances = Advances open
# review: pending
assistant-chip-stock = { -baqi-maal }
# review: pending
assistant-chip-party = A party's { -khata }
# review: pending
assistant-party-pick-label = Choose a party
# review: pending
assistant-party-pick-hint = Type the party's code or name
# review: pending
assistant-party-cancel = Never mind

## The answer block (FR-7.74, FR-7.75, FR-7.76).

# review: pending
assistant-you-asked = Your question
# review: pending
assistant-answer-label = Answer
# review: pending
assistant-checked = Checked
# review: pending
assistant-direction-more = more
# review: pending
assistant-direction-less = less
# review: pending
assistant-direction-same = the same
# review: pending
assistant-table-label = Rows of { $source }
# review: pending
assistant-table-more = First { $shown } of { $count } rows shown; the rest are in the full report
# review: pending
assistant-table-totals = { -meezan }
# review: pending
assistant-chart-label = Chart of the monthly amounts
# review: pending
assistant-chart-past = Past months
# review: pending
assistant-chart-ahead = Months ahead (estimate)
# review: pending
assistant-sources-line = Source: { $sources }, { $time }
# review: pending
assistant-open-report = Open the full report
# review: pending
assistant-open-report-of = { $source }: open the full report
# review: pending
assistant-print = Print
# review: pending
assistant-print-of = Print { $source }
# review: pending
assistant-next-label = Next questions
# review: pending
assistant-rerun-note = This old question was run again on today's books. The figures may have changed since then.

## The two bands (FR-7.74, FR-7.86, FR-7.88).

# review: pending
assistant-band-estimate = Estimate, not certain
# review: pending
assistant-band-advice = Advice: the decision is yours
# review: pending
assistant-band-general = General advice
# review: pending
assistant-band-general-note = Practical advice for the shop, not from your books
# review: pending
assistant-estimate-same-pace = Same pace
# review: pending
assistant-estimate-same-growth = Same growth
# review: pending
assistant-estimate-same-season = Same season as last year
# review: pending
assistant-estimate-what-if = If the pace is <num>{ $factor }</num>
# review: pending
assistant-estimate-what-if-any = If the pace changes
# review: pending
assistant-chart-what-if = If the pace is { $factor }
# review: pending
assistant-estimate-year-so-far = So far this year (from your books)
# review: pending
assistant-estimate-year-months = <num>{ $count }</num> months of the books, { $from } to { $to }
# review: pending
assistant-estimate-year-month = One month of the books: { $from }
# review: pending
assistant-estimate-year-left = Estimate for the months left
# review: pending
assistant-estimate-year-total = Whole year
# review: pending
assistant-estimate-year-note = The current month is counted as a whole estimated month; its days so far are not added.
# review: pending
assistant-estimate-months = Based on <num>{ $count }</num> months, { $from } to { $to }
# review: pending
assistant-estimate-ahead = Months estimated: { $from } to { $to }
# review: pending
assistant-estimate-few-months = The books hold only <num>{ $count }</num> full months, so no estimate could be made.

## The states that are not an answer (FR-7.77, FR-7.79, FR-7.81, FR-7.82).

# review: pending
assistant-which-one = Which one?
# review: pending
assistant-which-one-more = There are more, type the code
# review: pending
assistant-not-found = Name not found
# review: pending
assistant-not-found-closest = The closest names:
# review: pending
assistant-option-balance-label = Balance
# review: pending
assistant-need-dates = The period is not clear. Ask again with the first and last date, for example from 01-10-2026 to 15-10-2026.
# review: pending
assistant-need-party = Whose? Type the name or the code.
# review: pending
assistant-outside-books = This question is outside the books
# review: pending
assistant-outside-books-nearest = The nearest report:
# review: pending
assistant-change-refused = I can only tell, not write
# review: pending
assistant-change-refused-help = The button below opens the entry screen. The amount stays empty; you write it yourself.
# review: pending
assistant-open-entry = Open the entry screen
# review: pending
assistant-open-screen = Open the screen
# review: pending
assistant-entry-neutral = The amount stays empty; you write it yourself
# review: pending
assistant-not-understood = I did not understand; please ask in other words
# review: pending
assistant-stopped-calls = I could not finish reading the books. Please ask in smaller parts, for example one party or one month.
# review: pending
assistant-stopped-tokens = The books I read were too large for one answer. Please ask in smaller parts, for example a shorter period.
# review: pending
assistant-stopped-tables = The records I did read are below.
# review: pending
assistant-checks-failed = I could not write a checked answer; the records are below
# review: pending
assistant-records-differ = A difference was found in the records; open the report
# review: pending
assistant-limit-reached = Today's <num>{ $limit }</num> questions are used up; ask again tomorrow
# review: pending
assistant-limit-chips = The quick questions still work.
# review: pending
assistant-offline = Questions need the internet; the quick questions below still work
# review: pending
assistant-error = No answer came; try again
# review: pending
assistant-error-retry = Ask again
# review: pending
assistant-quick-failed = The quick question could not be answered; try again

## Settings, the group "Ask the app" (FR-7.85).

# review: pending
assistant-settings-title = Ask the app
# review: pending
assistant-settings-help = Ask questions about your books. The question and the records needed to answer it go to OpenRouter. Phone numbers, addresses, notes and reasons never go.
# review: pending
assistant-settings-enabled = Asking questions is on
# review: pending
assistant-settings-enabled-needs-key = Save the key first, then turn it on.
# review: pending
assistant-settings-key = OpenRouter key
# review: pending
assistant-settings-key-paste = Paste the key here
# review: pending
assistant-settings-key-save = Save the key
# review: pending
assistant-settings-key-saved = Key saved: <num>••••{ $last4 }</num>
# review: pending
assistant-settings-key-none = No key is saved yet
# review: pending
assistant-settings-key-replace = Replace
# review: pending
assistant-settings-key-remove = Remove
# review: pending
assistant-settings-key-password = Type your password
# review: pending
assistant-settings-key-password-help = Your password is needed to save, replace or remove the key.
# review: pending
assistant-settings-key-confirm-replace = Replace
# review: pending
assistant-settings-key-confirm-remove = Remove
# review: pending
assistant-settings-key-cancel = Never mind
# review: pending
assistant-settings-model = Model
# review: pending
assistant-settings-model-help = A new model must be checked before it is used, and it must have an Azure endpoint that keeps no data.
# review: pending
assistant-settings-model-save = Save the model
# review: pending
assistant-settings-test = Test the connection
# review: pending
assistant-settings-test-help = Sends one fixed question to see whether it works. It also counts as a question.
# review: pending
assistant-settings-test-needs-key = Save the key first to test the connection.
# review: pending
assistant-settings-test-working = Testing…
# review: pending
assistant-settings-test-ok = The connection works: <num>{ $ms }</num> ms
# review: pending
assistant-settings-test-failed = The connection could not be made: { $error }
# review: pending
assistant-settings-count = Today's count: <num>{ $used }</num> / <num>{ $limit }</num>
# review: pending
assistant-settings-limit-fixed = The daily limit is fixed; it is not a setting.
# review: pending
assistant-settings-failed = The settings for asking questions could not be read
# review: pending
assistant-settings-save-failed = This change could not be saved

## The question log in Settings (FR-7.84).

# review: pending
assistant-log-title = Question log
# review: pending
assistant-log-help = Every question is listed here. Press a question to run it again on today's books. The log cannot be changed or cleared.
# review: pending
assistant-log-empty = No question has been asked yet
# review: pending
assistant-log-empty-help = When you ask a question it will show here.
# review: pending
assistant-log-failed = The question log could not be opened
# review: pending
assistant-log-col-time = Time
# review: pending
assistant-log-col-question = Question
# review: pending
assistant-log-col-kind = Kind
# review: pending
assistant-log-col-outcome = Outcome
# review: pending
assistant-log-col-tools = Tools used
# review: pending
assistant-log-col-counted = In the count
# review: pending
assistant-log-kind-model = Model question
# review: pending
assistant-log-kind-quick = Quick question
# review: pending
assistant-log-kind-pick = After a choice
# review: pending
assistant-log-kind-rerun = Read again
# review: pending
assistant-log-kind-test = Connection test
# review: pending
assistant-log-counted-yes = Counted
# review: pending
assistant-log-counted-no = Not counted
# review: pending
assistant-log-open = Read again
# review: pending
assistant-log-open-row = Read again: { $question }
# review: pending
assistant-log-no-tools = No tools
# review: pending
assistant-log-col-took = How long
# review: pending
assistant-log-took = <num>{ $total }</num> { $total_unit } (model <num>{ $model }</num> { $model_unit }, tools <num>{ $tools }</num> { $tools_unit })
# review: pending
assistant-log-took-total = <num>{ $total }</num> { $total_unit }
# review: pending
assistant-log-took-none = Not recorded
# review: pending
assistant-unit-s = s
# review: pending
assistant-unit-ms = ms

## The outcome of a question, in the log (FR-7.84).

# review: pending
assistant-outcome-answered = Answered
# review: pending
assistant-outcome-estimate = Estimate given
# review: pending
assistant-outcome-advice = Advice given
# review: pending
assistant-outcome-which-one = Asked which one
# review: pending
assistant-outcome-need-dates = Asked for dates
# review: pending
assistant-outcome-need-party = Asked for a party
# review: pending
assistant-outcome-not-found = Name not found
# review: pending
assistant-outcome-outside-books = Outside the books
# review: pending
assistant-outcome-change-refused = Change refused
# review: pending
assistant-outcome-not-understood = Not understood
# review: pending
assistant-outcome-checks-failed = Checks failed
# review: pending
assistant-outcome-records-differ = Records differ
# review: pending
assistant-outcome-locked = Behind the code
# review: pending
assistant-outcome-limit-reached = Limit reached
# review: pending
assistant-outcome-offline = No internet
# review: pending
assistant-outcome-error = Error

## Sources, buttons and the estimate band (FR-7.74, FR-7.86).

# review: pending
assistant-source-party-card = Party card
# review: pending
assistant-source-analytics = { -tajziat } { $title }
# review: pending
assistant-sources-separator = , 
# review: pending
assistant-open-dashboard = Open the dashboard
# review: pending
assistant-open-entry-party = Open the entry screen for { $name }
# review: pending
assistant-estimate-per-month = Each month
# review: pending
assistant-estimate-step = Change each month
# review: pending
assistant-estimate-total = Total
# review: pending
assistant-estimate-target = Target
# review: pending
assistant-estimate-target-reached = The target is reached in { $month }
# review: pending
assistant-estimate-target-not = The target is not reached in these months

## The working under ⓘ (FR-7.43, FR-7.75). The titles are the ones the backend names.

# review: pending
assistant-working-average-of = Average of
# review: pending
assistant-working-sum = Sum of the amounts
# review: pending
assistant-working-difference = Difference of the amounts
# review: pending
assistant-working-average = Average of the amounts
# review: pending
assistant-working-percent-of = Percent
# review: pending
assistant-working-change-percent = Change in percent
# review: pending
assistant-working-percent-more = An amount and a percent more: the amount times (100% + the percent)
# review: pending
assistant-working-percent-less = An amount and a percent less: the amount times (100% − the percent)
# review: pending
assistant-working-same-pace = Same pace: the average of the past months, times the months ahead
# review: pending
assistant-working-year-parts = The last line: the months of the books so far + the estimate for the months left
# review: pending
assistant-working-what-if-pace = Same pace, changed by the factor
# review: pending
assistant-working-same-growth = Same growth: the average change each month, added for each month ahead
# review: pending
assistant-working-same-season = Same season as last year: the same months of last year

## What an estimate is about, what a figure is, and where its two places are (FR-7.75, FR-7.76).

# review: pending
assistant-measure-sales = { -kham-bikri }
# review: pending
assistant-measure-commission-seller = Seller { -commission }
# review: pending
assistant-measure-commission-buyer = Buyer { -commission }
# review: pending
assistant-measure-commission-laga = { -laga }
# review: pending
assistant-measure-commission-all = Total { -commission } (including { -laga })
# review: pending
assistant-measure-expenses = { -dukan-kharcha }
# review: pending
assistant-measure-profit = Profit
# review: pending
assistant-measure-receipts = { -wasooli }
# review: pending
assistant-measure-garden-sales = { -bagh } sales
# review: pending
assistant-measure-trip-sales = { -safar } sales
# review: pending
assistant-figure-party-balance = The party's balance
# review: pending
assistant-figure-buyers-dues = Buyers' dues
# review: pending
assistant-figure-sellers-dues = Sellers' dues
# review: pending
assistant-figure-dead-amount = { -doobi-raqam }
# review: pending
assistant-figure-total-dues = { -kul-agrahi }
# review: pending
assistant-figure-cash = Cash in hand
# review: pending
assistant-figure-day-sales = The day's { -bikri }
# review: pending
assistant-figure-day-commission = The day's { -commission }
# review: pending
assistant-figure-partner-total = The partner's { -kul-raqam }
# review: pending
assistant-place-party-card = Party card
# review: pending
assistant-place-r13-closing = Closing balance of R13
# review: pending
assistant-place-tile-buyers-dues = Dashboard: buyers' dues
# review: pending
assistant-place-r20-total = Total of R20
# review: pending
assistant-place-tile-sellers-dues = Dashboard: sellers' dues
# review: pending
assistant-place-r24-total = Total of R24
# review: pending
assistant-place-tile-dead-amount = Dashboard: { -doobi-raqam }
# review: pending
assistant-place-r38-total = Total of R38
# review: pending
assistant-place-tile-total-dues = Dashboard: { -kul-agrahi }
# review: pending
assistant-place-tile-cash = Dashboard: cash in hand
# review: pending
assistant-place-r12-closing = Closing balance of R12
# review: pending
assistant-place-tile-day = Dashboard: today's figures
# review: pending
assistant-place-r34 = R34 day summary
# review: pending
assistant-place-view-a-total = Total of { -tajziat } A
# review: pending
assistant-place-partner-overview = Partner overview
# review: pending
assistant-place-r31-closing = Closing total of R31

## Why a model call failed (D6 Section 2, FailureKind). One reviewed message for each.

# review: pending
assistant-failure-offline = No internet connection was found.
# review: pending
assistant-failure-dropped = The connection dropped. Please ask again.
# review: pending
assistant-failure-timeout = The answer took too long. Please ask again.
# review: pending
assistant-failure-bad-key = OpenRouter did not accept the key. Check the key in Settings.
# review: pending
assistant-failure-no-credit = The OpenRouter account has no credit left.
# review: pending
assistant-failure-no-endpoint = No Azure server that keeps no data accepted this request. Check that the model has Azure servers on OpenRouter, and that the app sends no setting Azure does not take. The model is in Settings.
# review: pending
assistant-failure-busy = OpenRouter is busy. Try again in a little while.
# review: pending
assistant-failure-server = OpenRouter had an error.
# review: pending
assistant-failure-bad-reply = The model's reply could not be read.
# review: pending
assistant-failure-other = An unknown error.
