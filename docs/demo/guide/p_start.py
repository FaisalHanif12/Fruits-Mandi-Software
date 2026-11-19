from ui import *


def pages():
    out = []
    # ---- cover
    out.append("""<section class="page plain cover"><div class="head" style="background:#475569"><span>Start</span><span class="pn">@@</span></div>
<div class="bigtitle">Test Book</div><div class="sub2">Mandi Software, step by step.<br>Click here, type this, you must see that.</div>
<h2>How to read this book</h2>
<p>Every page is <b>one thing you can do in the app</b>. The page shows a <b>drawing of the screen</b>. The drawing is simple, but the buttons and fields are in the same places as in your app.</p>
<div class="legend">
<div style="background:#fde8e6;border-color:#f1a7a0"><b style="color:#d92d20">Red number</b><br>on the drawing = the step with the same number. Do the steps in order.</div>
<div style="background:#fffbea;border-color:#f0d36b"><b>Yellow box</b><br>the exact words and numbers to type.</div>
<div style="background:#ecf9f2;border-color:#9fdcbc"><b style="color:#0b5a3b">Green box</b><br>what you must see. If you see something else, the app is wrong.</div>
</div>
<div class="legend">
<div style="background:#fff1e6;border-color:#f6b98a"><b style="color:#8a3b00">Orange box</b><br>a wrong entry. The app must refuse it with a clear message. Do not save it.</div>
<div style="background:#f3effb;border-color:#c9b8ee"><b style="color:#4a2a8a">Purple box</b><br>"Not the same as": a look-alike screen. Do not mix them up.</div>
<div style="background:#eef3fb;border-color:#cfdcf0"><b style="color:#1d4ed8">Blue bar</b><br>"Open it": which menu to click to reach the screen.</div>
</div>
<h2>Words in the drawings</h2>
<p>Urdu words are written as the app shows them. The small English word under or beside it tells you what it means. In the steps, an Urdu word is followed by its English in brackets, for example <span class="u" style="font-size:13pt;font-weight:600">محفوظ کریں</span> (Save).</p>
<h2>The order of the book</h2>
<p>Index &rarr; Sales &rarr; Cash book &rarr; Gardens &rarr; Trips &rarr; Partners &rarr; Analytics &rarr; Reports &rarr; Backup &rarr; Settings &rarr; Dashboard. The same order as the menu of the app.</p>
<p style="color:#52606d;font-size:10pt;margin-top:6mm">Written by tech-lead, 26-10-2026, for Umar. Every number comes from the PRD (Sections 4, 18.4 and 27.2). If a screen shows a different number, write down the screen, what you typed and what you saw, and send it to tech-lead.</p>
</section>""")

    # ---- sign in
    sign = plain(
        card("لاگ ان", "Sign in",
             row(fld("صارف کا نام", "User name", "owner", n=1),
                 fld("پاس ورڈ", "Password", "••••••••••", n=2)),
             row(btn("داخل ہوں", "Sign in", pri=True, n=3), btn("باہر", "Exit"), fld("", "", ph=""), gap=8)) +
        '<div style="text-align:center;margin-top:8px;font-size:12px"><span class="u">اردو</span> / English</div>',
        minh=250)
    out.append(page(
        "Start", "@@", "Open the app and sign in", "لاگ ان", [("", "Start the app: pnpm dev")], sign,
        steps=["1|Click the first box and type the user name.", "2|Click the second box and type the password.",
               "3|Click [[داخل ہوں|Sign in]] (or press Enter)."],
        typed=[("User name", "{{owner}}"), ("Password", "{{mandi-dev-1}}")],
        see=["The dashboard opens (Part Dashboard at the end of this book).",
             "The first time on this Mac, a Keychain window asks about **audit-chain-key**. Type your Mac password and choose **Always Allow**. If you skip it, the app says \"something went wrong\"."],
        wrong=["Five wrong passwords: the app makes you wait 1 minute. Five more: 5 minutes. Then 15 minutes each time. It is safe, you only wait."],
        differs=["The app is started from the D5 folder with {{pnpm dev}}. It is running now.", "The sign-in screen has its own [[اردو|Urdu]] / English switch."]))

    # ---- set up the data
    out.append("""<section class="page plain"><div class="head" style="background:#475569"><span>Start</span><span class="pn">@@</span></div>
<h1><span>Get the test data ready (do this once)</span><span class="u h1u">تیاری</span></h1>
<p>The tests use the <b>worked day</b> of the PRD: Cash 50,000. Rahim has an open advance of 15,000. Partners A 40%, B 30%, C 30%. Sellers Rahim and Karim. Buyers Akram and Arshad. This is called the <b>fresh start</b>.</p>
<h2>Three ways to get data</h2>
<table class="gt"><tr><th style="width:24%">Way</th><th>When to use it</th></tr>
<tr><td><b>A. What is in the app now</b></td><td>You only want to look around. Check that Index shows Rahim, Karim, Akram, Arshad, and Cash shows 50,000. If not, use B.</td></tr>
<tr><td><b>B. Fresh start (best)</b></td><td>You want the exact PRD numbers. Follow the 5 steps below.</td></tr>
<tr><td><b>C. Empty app</b></td><td>You want to test the first-run wizard and type everything yourself. Takes about 30 minutes more.</td></tr></table>
<h2>Fresh start, 5 steps (open the Terminal app)</h2>
<pre class="cmd"># 1. Close the app window. In the Terminal that runs the app press Ctrl+C.
cd "/Users/faisalhanif/MyProfile/My Projects /Fruit Mandi Parallel/d5"
APP="$HOME/Library/Application Support/pk.mandisoftware.app"

# 2. Keep the old data under another name (nothing is lost).
mv "$APP" "$APP.old-$(date +%Y%m%d-%H%M)"

# 3. Make the opening position. First time takes a few minutes.
#    If a Keychain window asks, type your Mac password and choose Always Allow.
pnpm seed -- --reset

# 4. Put it where the app reads it.
mkdir -p "$APP/data"
cp dev-data/mandi.db "$APP/data/mandi.db"

# 5. Start the app again.
pnpm dev</pre>
<p style="font-size:10pt;color:#8a3b00">These commands were not run on your Mac by the lead. If the app says the data is damaged, put the <b>.old</b> folder back and tell tech-lead. Some tests (marked <b>Fresh start</b>) need this again from the beginning.</p>
<h2>Which date to type</h2>
<p>Leave the date box as it is. It already shows <b>today</b>. The PRD calls the days 10-10 and 11-10, but the numbers do not depend on the date. Using today makes the dashboard (it shows "today") agree with the PRD. Where a test needs an old date, the page tells you which date to type. A date on or before the closed-up-to date, or after today, is refused.</p>
</section>""")

    # ---- keys and the order of the tests
    out.append(plainpage("Start", "Keys, and the order of the tests", "ترتیب", """<h2>Keys you will use</h2>""" + gt(
        ["Key", "What it does", "Key", "What it does"],
        [["<span class='kbd'>Enter</span>", "next box, next cell", "<span class='kbd'>F10</span> or <span class='kbd'>Ctrl+S</span>", "save"],
         ["<span class='kbd'>Alt+L</span>", "Urdu or English", "<span class='kbd'>Ctrl+Enter</span>", "save, print, new entry"],
         ["<span class='kbd'>F4</span>", "add an account from a picker", "<span class='kbd'>Ctrl+K</span>", "search everything"],
         ["<span class='kbd'>Alt+U</span>", "typing mode in a name box", "<span class='kbd'>Ctrl+I</span>", "how was this worked out?"],
         ["<span class='kbd'>Ctrl+D</span>", "copy the cell above", "<span class='kbd'>Alt+Z</span>", "undo a save (10 seconds)"],
         ["<span class='kbd'>Esc</span>", "close, cancel, leave", "<span class='kbd'>Alt+B V A P K</span>", "new sale, receipt, payment, advance, expense"]]) +
        "<h2>The order of the tests (1 of 2)</h2><p>Do the runs in this order. <b>Run 1 and Run 2 use the same data.</b> Every line of <b>Run 3</b> starts from a fresh start (page 3).</p>" + gt(
        ["Step", "What you do", "Where in this book"],
        [["<b>Run 1</b>", "<b>The worked day</b>", ""],
         ["1", "Sign in. Look at the dashboard: cash 50,000.", "Start, Dashboard"],
         ["2", "Index: add an account, lists, rules.", "Index"],
         ["3", "Sale 1 (Rahim, bananas).", "Sales"],
         ["4", "Sale 2 (Karim, apples, a one-time buyer).", "Sales"],
         ["5", "Look: Pukhta register, Today's sales, Collection list, Unsold stock.", "Sales"],
         ["6", "Receipt 10,000 from Akram. Payment 4,316 to Rahim.", "Sales"],
         ["7", "Shop expense 3,000. Personal drawing 2,000.", "Cash book"],
         ["8", "<b>CHECKPOINT</b>: dashboard, partners, analytics, reports, trial balance 85,048.", "all parts"],
         ["9", "Edit sale 1 and edit it back. Draft, undo, backup, test restore.", "Sales, Backup"],
         ],
        ["14%", "62%", "24%"])))

    out.append(plainpage("Start", "The order of the tests (2 of 2)", "ترتیب", gt(
        ["Step", "What you do", "Where in this book"],
        [
         ["<b>Run 2</b>", "<b>More on the same data, in this order</b>", ""],
         ["10", "Partner B puts in capital 50,000. Partner statement.", "Partners"],
         ["11", "Partner A returns 2,000. Cash lent to Akram 3,000.", "Cash book, Sales"],
         ["12", "Money lent to Aslam (see the bell).", "Cash book"],
         ["13", "Cancel sale 2.", "Sales"],
         ["14", "Close the day, try to edit, reopen. Month close.", "Settings, Partners"],
         ["15", "Restore a backup.", "Backup"],
         ["<b>Run 3</b>", "<b>Fresh start for each</b>", ""],
         ["3a", "Part-sold sale, later sale, write-off, fruit taken.", "Sales, Cash book"],
         ["3b", "A garden.", "Gardens"],
         ["3c", "A market trip.", "Trips"],
         ["3d", "A separate khata.", "Cash book"],
         ["3e", "Advance variants.", "Sales, Cash book"],
         ["3f", "7% default rate, two one-time buyers, expense 3,384, opening balances.", "Index, Sales, Cash book"]],
        ["14%", "62%", "24%"]) + """
<h2>If a number is wrong</h2>
<p>Stop. Write down: the screen name, the language, what you typed, the number you expected, the number you saw, and the reference number if the message shows one. Send it to tech-lead. Do not change anything to make it match.</p>
<h2>Two things that are normal</h2>
<ul><li>The Mac shows the same numbers as Windows. It cannot prove real printing, PDF files, the installer or speed. Those are on the last page.</li>
<li>On the sign-in screen, five wrong passwords make you wait one minute. That is the safety rule, not a fault.</li></ul>"""))

    # ---- look-alike screens 1
    out.append(plainpage("Start", "Look-alike screens (1 of 2): do not mix them up", "فرق", """
<h2>Money in and out: one screen, three types</h2>""" + gt(
        ["", "[[وصولی|Receipt]]", "[[ادائیگی|Payment]]", "[[پیشگی|Advance]]"],
        [["Direction", "Money **in**", "Money **out**", "Money **out**, **before** his goods are sold"],
         ["Usual party", "A buyer", "A seller", "A seller"],
         ["Helper shown", "His open bills, oldest first. Buttons [[پورا بقایا|Full balance]] and [[پرانا بل|Oldest bill]].", "His payable postings. Button [[پورا بقایا|Full balance]].", "His open advances and the last advance."],
         ["Other use", "From a **seller**: he returns advance money.", "To a **buyer**: cash lent. A reason is required. It becomes his bill.", "Never a payment of what is owed."]],
        ["16%", "28%", "28%", "28%"]) + """
<h2>The money screens of the cash book</h2>""" + gt(
        ["Entry", "Who it touches", "Cash", "Split by shares?", "Partner's total"],
        [["[[دکان خرچہ|Shop expense]]", "The shop", "out", "**Yes**", "every partner, by his share"],
         ["[[ذاتی خرچہ|Personal drawing]]", "One partner", "out", "No", "that partner goes **down**"],
         ["ذاتی خرچہ واپسی (return)", "One partner", "in", "No", "that partner goes **up**"],
         ["[[پھل|Fruit]] for one partner", "One partner", "none", "No", "that partner only"],
         ["پھل for guests", "The shop's guests", "none", "**Equal parts**, not by share", "every partner, equal"],
         ["[[ادھار رقم|Money lent]]", "A person (staff or other)", "out", "No", "no change"],
         ["[[الگ کھاتہ|Separate khata]]", "A family or other khata", "out or in", "No", "no change. Never income or expense."],
         ["[[سرمایہ|Capital]]", "One partner", "in or out", "No", "that partner up or down"]],
        ["24%", "20%", "9%", "19%", "28%"]) + """
<h2>Garden or trip?</h2>""" + gt(
        ["", "[[باغ|Garden]]", "[[مارکیٹ سفر|Market trip]]"],
        [["What it is", "A garden rented for a season, rent paid in installments", "A day trip to a bigger market to buy goods"],
         ["Goods are sold with seller", "The **garden**", "[[اپنا مال|Own goods]]"],
         ["Ends when", "You close it (a confirm box)", "It closes **by itself** when nothing is left"],
         ["Sales money goes to", "The garden's account, split at closing", "The trip's partners, at once"]],
        ["22%", "39%", "39%"]) + """<p style="font-size:10pt">In both, the partners in it total exactly 100%. Money from the shop ([[دکان سے|From the shop]]) takes cash and the partner's total. Money from his pocket ([[اپنی جیب سے|Own pocket]]) posts nothing.</p>"""))

    # ---- look-alike screens 2
    out.append(plainpage("Start", "Look-alike screens (2 of 2)", "فرق", """
<h2>Edit, cancel, undo, write off, reopen</h2>""" + gt(
        ["Action", "What happens", "The day must be"],
        [["**Edit**", "The old posting is reversed and a new one made. Lines you did not change keep their values. The audit log keeps the old version.", "open"],
         ["**Cancel**", "Every posting is reversed. A reason is required. Receipts become credit and are matched again.", "open"],
         ["**Undo** (Alt+Z)", "Cancels what you just saved and opens a new entry with the same values. Only for 10 seconds.", "open"],
         ["**Write off**", "Takes the leftover quantity off stock, with a reason. No money posts.", "open"],
         ["**Reopen a day**", "Needs a reason. Reopens that day **and every later day**. Written to the audit log.", "closed"]],
        ["18%", "68%", "14%"]) + """
<h2>Things that take goods out of [[باقی مال|Unsold stock]]</h2>""" + gt(
        ["", "Later sale", "Write off", "Fruit taken"],
        [["Money", "New posting with the normal commission", "None", "Seller credited the value. **No commission.**"],
         ["Seller's advance", "Adjusted if the tick is on", "Not touched", "**Never** adjusted"]],
        ["18%", "32%", "22%", "28%"]) + """
<h2>Prints of one buyer</h2>""" + gt(
        ["Print", "What it is", "Who"],
        [["**Buyer slip** (from the sale)", "One buyer's slip of this sale, the client's layout", "A named buyer, cash lines too"],
         ["**R5** [[ادھاری تفصیل|Credit details]]", "One row per buyer, lines written \"(item qty . rate)\"", "**Credit buyers only**"],
         ["**R35** party day sheet", "The collector's slip: previous balance, bought, paid, balance. One slip, or all parties in one table.", "Buyers and sellers"],
         ["**R13** ledger", "Every entry of one party with a running balance", "One party, any dates"],
         ["**R14** statement", "Opening, bought, paid, balance. Option \"balance only\".", "One party"]],
        ["24%", "52%", "24%"]) + """
<h2>Other pairs</h2>""" + gt(
        ["Pair", "The difference"],
        [["[[خام بکری|Gross]] and [[صافی بکری|Net]]", "Gross is the sum of seller amounts. Net is gross minus all deductions of the lot."],
         ["Shared income and payables", "Commission, [[لاگا|Laga]], phone charge and rate difference are income, **shared** by partners. Labour, freight, store rent and munshiana are **payables**, not shared."],
         ["Reports and analytics", "Reports read the books (ledgers, registers) for a day and a party. Analytics read the stored value of each sale line: by item, seller, buyer, partner, garden, trip, fruit."],
         ["Day close and month close", "Day close locks the days. Month close moves a finished month's income and expenses into partner capital and never changes a partner's total."],
         ["Back up now, test restore, restore", "Back up now makes a copy. Test restore opens a copy in a **separate place** and checks it. Restore **replaces** your data (after a safety copy)."],
         ["Mac and Windows", "The Mac proves the numbers and the screens. Real printing, PDF, the installer and speed need the Windows computer."]],
        ["28%", "72%"])))
    return out
