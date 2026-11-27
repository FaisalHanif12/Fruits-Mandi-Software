## ادھاری تفصیل on screen: how much the shop has to take from its buyers (R5, PRD R5 v1.43, plan 80 Q-R5-2).
## The column words and the words of the totals are the print's own (print.ftl), so the screen and the paper say the same.

# review: pending
adhari-help = How much the shop has to take from its { -gahakon } on the chosen day: one row for each { -gahak } who bought, with his goods, the day's bill, today's credit, what he owed before and the total credit now. Under the table: the { -gahakon } who owed but bought nothing today, and the total credit of all { -gahakon }. The totals are over every { -gahak }, not only the rows on screen.

# review: pending
adhari-label = Buyers of the day

# review: pending
adhari-failed = The sheet could not be read

# review: pending
adhari-empty = No { -gahak } bought on this day

# review: pending
adhari-empty-help = Choose another day, or enter a { -bikri } first.

# review: pending
adhari-goto = Go to day

# review: pending
adhari-goto-hint = Type a day, for example 10-10-2026. A short form works too: 10 is the 10th of this month.

# review: pending
adhari-goto-placeholder = DD-MM-YYYY

# review: pending
adhari-today = Today

# The buyers who owed at the start of the day and bought nothing today, as one total.
# review: pending
adhari-others-line = { print-adhari-sheet-others } (<count>{ $count }</count>): { -sabiqa-raqam } <prev></prev>, { print-adhari-sheet-col-total } <due></due>

# Everything every buyer owes the shop.
# review: pending
adhari-all-line = { print-adhari-sheet-all }: <due></due>

# review: pending
adhari-dead-line = { -doobi-raqam } (listed apart, not in the total above): { -sabiqa-raqam } <prev></prev>, { -taza-banam } <bill></bill>, { print-adhari-sheet-col-total } <due></due>

# The buyers whose total credit ends in { -jama } (the shop owes them): listed apart so the total above stays what the buyers owe.
# review: pending
adhari-credit-line = { -gahak } in { -jama } (listed apart, not in the total above): { -sabiqa-raqam } <prev></prev>, { -taza-banam } <bill></bill>, { print-adhari-sheet-col-total } <due></due>
