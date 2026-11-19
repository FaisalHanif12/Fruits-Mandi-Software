from ui import *


def cardbtn(ur, en, n=None, on=False):
    return (f'<div class="card" style="flex:1;min-width:150px;margin:0">{lab(ur, en)}{bd(n)}</div>')


def pages():
    out = []

    # ---- 1 open the Index
    home = frame("index", None, ("انڈکس", "Index"), body=
        '<div class="bigcap">One card for each list. Click a card to open it.</div>' +
        row(cardbtn("کھاتے", "Accounts", n=2), cardbtn("کھاتوں کی ترتیب", "Chart of accounts"), cardbtn("کمیشن اصول", "Commission rules")) +
        row(cardbtn("شہروں کی فہرست", "Cities"), cardbtn("قسم مال کی فہرست", "Items"), cardbtn("پیکنگ کی فہرست", "Packings")) +
        row(cardbtn("اڈا کی فہرست", "Addas"), cardbtn("مارکہ کی فہرست", "Marks"), cardbtn("سٹوروں کی فہرست", "Stores")) +
        row(cardbtn("تفصیل کی فہرست", "Narrations"), cardbtn("خرچہ مد کی فہرست", "Expense heads"), cardbtn("عملے کی فہرست", "Staff")),
        nmain=1, minh=330)
    out.append(page("Index", "@@", "Open the Index", "انڈکس", [("انڈکس", "Index")], home,
        steps=["1|In the dark menu on the right click [[انڈکس|Index]].", "2|Twelve cards appear. Click one to open that list. For example click [[کھاتے|Accounts]]."],
        see=["Cards for: Accounts, Chart of accounts, Commission rules, and the nine lists (cities, items, packings, addas, marks, stores, narrations, expense heads, staff).",
             "Arrow keys move between cards. Enter opens one."],
        tip="The Index is where names are made once. Every other screen only picks them from here.",
        differs=["Index = the lists and the accounts. **Sales** = the daily work. A list here never has a balance; only an account has one."]))

    # ---- 2 chart
    chart = frame("index", None, ("کھاتوں کی ترتیب", "Chart of accounts"), body=
        tree([(0, "01", "اثاثے", "Assets", ""), (1, "0101", "نقد", "Cash", ""), (2, "01010001", "روکڑ", "Cash", "50,000 Dr"),
              (0, "03", "پارٹیاں", "Parties", ""), (1, "03011", "بیوپاری", "Sellers", ""), (2, "030110001", "رحیم", "Rahim", "15,000 Dr"),
              (2, "030110002", "کریم", "Karim", "0"), (1, "0304", "گاہک", "Buyers", ""), (2, "03040001", "اکرم ٹول ٹیکس", "Akram", "0"),
              (0, "06", "سرمایہ", "Capital", ""), (2, "06010001", "ابتدائی سرمایہ", "Opening capital", "65,000 Cr")], n=2), minh=330)
    out.append(page("Index", "@@", "Chart of accounts", "کھاتوں کی ترتیب", [("انڈکس", "Index"), ("کھاتوں کی ترتیب", "Chart of accounts")], chart,
        steps=["1|On the Index click the card [[کھاتوں کی ترتیب|Chart of accounts]].", "2|Look at the tree: Head, then Control, then Account, each with its balance. Use the arrow keys to open and close a branch."],
        see=["Cash **50,000** (Dr = debit). Rahim **15,000 Dr** (his advance). Opening capital **65,000 Cr** (credit).",
             "System accounts are there too: labour, freight, store rent and munshiana payables, the income accounts, Naqdi Mandi, the two Pukhta Khata accounts, Own Goods, Opening Difference (0).",
             "After the sales of this book, come back: every balance here must agree with the trial balance."],
        wrong=["Open Accounts, pick **Cash**, and try to make it inactive: it is refused. System accounts can never be made inactive."],
        differs=["The chart shows every account with its balance. **Accounts** (next page) is the list you add to."]))

    # ---- 3 accounts list
    acc = frame("index", None, ("کھاتے", "Accounts"),
        actions=[btn("ابتدائی بقایا لکھیں", "Opening balances", n=3), btn("نیا کھاتہ", "New account", "F2", pri=True, n=2)],
        body=row(fld("قسم", "Type", "سب  (All)", kind="sel", n=1, w=150), fld("تلاش", "Search", "", ph="code or name")) +
        tbl([("کوڈ", "Code"), ("اردو نام", "Urdu name"), ("انگریزی نام", "English name"), ("ابتدائی بنام", "Opening debit"), ("موجودہ بقایا", "Balance")],
            [["030110001", "رحیم", "Rahim", "0", "15,000 Dr"], ["030110002", "کریم", "Karim", "0", "0"],
             ["03040001", "اکرم ٹول ٹیکس", "Akram Toll Tax", "0", "0"], ["03040002", "ارشد ملتانی", "Arshad Multani", "0", "0"]],
            total=["", "", "Total", "0", "15,000"]), minh=330)
    out.append(page("Index", "@@", "The accounts list", "کھاتے", [("انڈکس", "Index"), ("کھاتے", "Accounts")], acc,
        steps=["1|Use [[قسم|Type]] to show only sellers, buyers, money lent or others. Type in the search box to find one.",
               "2|Click [[نیا کھاتہ|New account]] (or press F2) to add an account. The next page does this.",
               "3|Click [[ابتدائی بقایا لکھیں|Opening balances]] to type opening amounts (a later page)."],
        see=["Columns: code, Urdu name, English name, city, opening date, opening debit, opening credit, credit allowed, current balance.",
             "The footer adds up the debit and the credit.", "Press Enter on a row to open that account."],
        wrong=["Search for {{akrm}} or {{اکرام}}: Akram is still found. Small spelling mistakes are forgiven."],
        tip="An account that has entries can never be deleted, only made inactive: open it and untick Active."))
    # ---- 4 new buyer
    form = frame("index", "accounts", ("نیا کھاتہ", "New account"),
                 actions=[btn("محفوظ کریں", "Save", "F10", pri=True, n=8)],
                 body=card("کھاتہ", "Account", row(fld("کھاتے کی قسم", "Account type", "گاہک  (Buyer)", kind="sel", n=1, grow=1),
                                                  fld("کوڈ", "Code", "03040003", n=2, grow=1)))
                 + card("نام", "Name", row(fld("اردو نام", "Urdu name", "طارق محمود", n=3, grow=1), fld("انگریزی نام", "English name", "Tariq Mehmood", n=4, grow=1)),
                        row(fld("نشان", "Tag", "", ph="K, BA ... (optional)", n=5, w=170), fld("فون", "Phone", "03001234567", n=6, grow=1), fld("شہر", "City", "", kind="sel", grow=1)))
                 + card("ابتدائی بقایا", "Opening", row(fld("ابتدائی تاریخ", "Opening date", "today"), fld("ابتدائی بنام", "Opening debit", ""), fld("ادھار کی اجازت", "Credit allowed", "1", kind="chk", n=7))),
                 nmain=None, minh=330)
    out.append(page("Index", "@@", "Add a new buyer", "نیا گاہک", [("انڈکس", "Index"), ("کھاتے", "Accounts"), ("نیا کھاتہ", "New account")], form,
                    steps=["1|In [[کھاتے کی قسم|Account type]] choose [[گاہک|Buyer]].",
                           "2|The [[کوڈ|Code]] is already filled: the next free one. Leave it.",
                           "3|Click the Urdu name box. Type the name in **English letters**. The app shows Urdu spellings under the box. Press **Space** or **Enter** to take the first one.",
                           "4|The English name fills by itself. Check it. You can change it.",
                           "5|Leave the tag empty for now (next page).",
                           "6|Type the phone number, 11 digits.",
                           "7|[[ادھار کی اجازت|Credit allowed]] is ticked. Leave it.",
                           "8|Click [[محفوظ کریں|Save]] or press F10."],
                    typed=[("Urdu name box", "{{tariq mehmood}}"), ("Phone", "{{03001234567}}")],
                    see=["Urdu name **طارق محمود**, English name **Tariq Mehmood** (filled by itself).", "Code **03040003**. After saving, his card shows the phone as 0300-1234567.",
                         "In any buyer box, typing {{tarq}} finds him. Press Alt+L: he shows as Tariq Mehmood."],
                    wrong=["English letters in the Urdu name: \"اردو نام اردو حروف میں لکھیں\".", "Urdu letters in the English name: \"انگریزی نام انگریزی حروف میں لکھیں\".", "Phone {{0300123}} (too short): refused.", "Opening debit {{500.50}}: \"رقم پورے روپوں میں لکھیں\"."],
                    differs=["{{Alt+U}} in a name box: **رومن** (English letters become Urdu), **اردو کی بورڈ** (a=ا, b=ب), **انگریزی** (nothing changes). Codes, numbers and phones never change."]))

    # ---- 5 tag and duplicate
    dup = frame("index", "x", ("نیا کھاتہ", "New account"),
        actions=[btn("محفوظ کریں", "Save", "F10", pri=True, n=5)],
        body=card("نام", "Name", row(fld("اردو نام", "Urdu name", "طارق محمود", grow=2), fld("انگریزی نام", "English name", "Tariq Mehmood", grow=2),
                  fld("نشان", "Tag", "K", n=1, w=90), fld("فون", "Phone", "03001234567", n=2, grow=2))) +
        note("amber", "اسی نام اور فون کا کھاتہ پہلے سے ہے: [[طارق محمود|Tariq Mehmood]] 03040003 (similar account)", n=3) +
        note("green", "With the tag **K** this is a different person. It is not a duplicate. Save.", n=4), nmain=None, minh=300)
    dup = dup.replace('<div class="sub"></div>', '')
    out.append(page("Index", "@@", "The tag (نشان) and the duplicate warning", "نشان", [("انڈکس", "Index"), ("کھاتے", "Accounts"), ("نیا کھاتہ", "New account")], dup,
        steps=["1|Open a new buyer. Type the same name {{tariq mehmood}} and the same phone {{03001234567}} as the buyer on the last page. **Leave the tag empty.**",
               "2|Look under the name: the similar account shows with its code and phone. This is the **duplicate warning**. Do not save.",
               "3|Now type the tag {{k}} in [[نشان|Tag]]. It turns into {{K}}. The warning goes: same name and phone with a different tag is another person.",
               "4|Save. Then try a third account with the same name, phone and tag {{K}}: the warning comes back. Do not save it."],
        see=["The picker now lists \"طارق محمود K\" and \"Tariq Mehmood K\".", "In a buyer box type {{tariq k}}: the one with the tag comes first."],
        wrong=["Tag {{K1}}, {{KKKKK}}, a space or Urdu letters: refused. A tag is 1 to 4 English capital letters and dots (K, BA, D.P)."],
        tip="The tag is also printed after the name on every slip and report."))

    # ---- 6 find an account
    pick = frame("bikri", "book", ("بکری بک", "Sale Book"), body=
        row(fld("گاہک کوڈ", "Buyer code", "1", n=1, w=190), fld("گاہک", "Buyer", "اکرم ٹول ٹیکس", n=2, grow=1)) +
        card("", "", '<div class="lab"><span class="u">اکرم ٹول ٹیکس</span><span class="e">Akram Toll Tax</span></div>' +
             '<div style="font-size:11px;margin-top:3px;direction:ltr;text-align:start">Balance 0 &nbsp;|&nbsp; Credit limit 0 &nbsp;|&nbsp; Phone &nbsp;|&nbsp; Last entry --</div>', n=3) +
        note("amber", "اس کوڈ کا کوئی گاہک نہیں   (No buyer has this code)   [F4 add]", n=4), nmain=None, nsub=None, minh=300)
    out.append(page("Index", "@@", "Find an account: code, short code, search", "تلاش", [("بکری", "Sales"), ("بکری بک", "Sale Book"), ("any account box", "")], pick,
        steps=["1|In any buyer box type just the number {{1}} and press **Enter**. This is the **short code**: 1 means 03040001.",
               "2|The name fills in and the **card** appears: balance, credit limit, phone, last entry. You typed nothing else.",
               "3|Type a code that does not exist, for example {{99}}, and press Enter.",
               "4|The message says no buyer has this code, with an F4 button to add one. F4 opens a small form and keeps what you typed."],
        see=["{{1}} gives Akram, {{2}} gives Arshad. In a **seller** box {{1}} gives Rahim. In a journal line {{1}} lists both the buyer and the seller with that number.",
             "Typing letters searches both Urdu and English names: {{akram}}, {{اکرم}}, {{akrm}} all find him."],
        differs=["A short code is taken when you press **Enter** or leave the box, never while typing (1 could be the start of 12)."]))

    # ---- 7 opening balances
    op = frame("index", None, ("ابتدائی بقایا", "Opening balances"), actions=[btn("محفوظ کریں", "Save", "F10", pri=True, n=4)], body=
        tbl([("کھاتہ", "Account"), ("ابتدائی بنام", "Opening debit"), ("ابتدائی جمع", "Opening credit")],
            [["Test Buyer", "1,000", ""], ["روکڑ  Cash", "65,000", ""], ["ابتدائی سرمایہ", "", "65,000"]], n=1) +
        totals([("کل بنام", "Total debit", "66,000"), ("کل جمع", "Total credit", "65,000"), ("ابتدائی فرق", "Opening difference", "1,000")], n=2) +
        note("amber", "The opening difference is not 0. The books still balance, but check your openings.", n=3), minh=300)
    out.append(page("Index", "@@", "Opening balances (fresh start)", "ابتدائی بقایا", [("انڈکس", "Index"), ("کھاتے", "Accounts"), ("ابتدائی بقایا لکھیں", "Opening balances")], op,
        steps=["1|Add a throw-away buyer called {{Test Buyer}} (Index, Accounts, New account). On its form type {{1,000}} in [[ابتدائی بنام|Opening debit]]. Save.",
               "2|Open [[ابتدائی بقایا|Opening balances]]. Look at the three totals.",
               "3|The opening difference shows **1,000**: one side has more than the other.",
               "4|Open the account again and clear the opening debit. Save. The difference is **0** again."],
        see=["A good set of openings has total debit = total credit and difference 0. The trial balance always balances anyway."],
        wrong=["Opening on an income account, the Opening Difference account, Naqdi Mandi, the two Pukhta accounts or Own Goods: refused.", "Both a debit and a credit on one account: refused.", "{{500.50}}: whole rupees only."],
        tip="Do this test on a fresh start. For the worked day the openings are already right (cash 65,000 against capital 65,000)."))

    # ---- 8 other lists
    lst = frame("index", None, ("قسم مال کی فہرست", "Item list"), actions=[btn("نئی قسم مال", "New item", "F2", pri=True, n=1)], body=
        row(fld("تلاش", "Search", "", ph="Type a code or a name", n=2, w=260), fld("غیر فعال بھی دکھائیں", "Show the inactive ones too", "", kind="chk", n=3)) +
        tbl([("کوڈ", "Code"), ("اردو نام", "Urdu name"), ("انگریزی نام", "English name"), ("گروپ", "Group")],
            [["0001", "کیلا", "Banana", "fruit"], ["0002", "سیب", "Apple", "fruit"], ["0003", "مولی", "Radish", "vegetable"]], n=4), minh=300)
    out.append(page("Index", "@@", "The other lists: cities, items, packings and more", "فہرستیں", [("انڈکس", "Index"), ("any list card", "")], lst,
        steps=["1|Open a list card (here the items). Click [[نئی قسم مال|New item]] (or F2). Type the name in English letters; the other name fills itself. Save.",
               "2|Use the search box to narrow the rows.", "3|Tick [[غیر فعال بھی دکھائیں|Show the inactive ones too]] to bring back hidden rows.", "4|Press Enter on a row to edit it. Untick Active to hide it."],
        typed=[("City", "{{multan}}"), ("Item", "{{orange}}  group **fruit**, sold by unit, default packing Box"), ("Packing", "{{crate}}"), ("Adda", "{{multan adda}}  city Multan"),
               ("Mark", "{{AR}}  owner Rahim"), ("Store", "{{main store}}"), ("Narration", "an Urdu and an English sentence"), ("Expense head", "{{tea}}"), ("Staff", "{{aslam}}  job Labourer")],
        see=["Each list: the new record with its code. You will need **Orange** (trips) and **Aslam** (money lent) later in this book."],
        wrong=["A record already used in an entry cannot be deleted. It can only be made inactive."],
        tip="A mark (مارکہ) is typed in one script and is not translated."))

    # ---- 9 rules
    rules = frame("index", None, ("کمیشن اصول", "Commission rules"), actions=[btn("اصول آزمائیں", "Test a rule", n=1), btn("نیا اصول", "New rule", "F2", pri=True, n=2)], body=
        tbl([("جانب", "Side"), ("قسم", "Charge"), ("کھاتہ", "Party"), ("قسم مال", "Item"), ("طریقہ", "Method"), ("شرح", "Value")],
            [["بیوپاری", "کمیشن", "any", "any", "فیصد %", "6"], ["گاہک", "کمیشن", "any", "any", "فیصد %", "6"], ["گاہک", "لاگا", "group: fruit", "any", "فی نگ", "8"]], n=3) +
        note("blue", "Which rule wins, most specific first: 1 typed on the line, 2 party + item, 3 party, 4 item, 5 item group, 6 global default.", n=4), minh=300)
    out.append(page("Index", "@@", "Commission rules: look and add", "کمیشن اصول", [("انڈکس", "Index"), ("کمیشن اصول", "Commission rules")], rules,
        steps=["1|Open the card [[کمیشن اصول|Commission rules]]. Look at the three starting rules.", "2|Click [[نیا اصول|New rule]] to add one (next page uses it).",
               "3|Read the table: seller commission 6%, buyer commission 6%, buyer [[لاگا|Laga]] 8 per nag on fruit.", "4|Read the blue note: the most specific rule wins."],
        typed=[("New rule (for the test)", "side Seller, charge Commission, party Rahim, item Banana, percent {{5}}, From date {{01-12-2026}}"),
               ("Second rule", "Seller, Commission, party Rahim, no item, percent {{4}}, From {{01-12-2026}}"),
               ("Third rule", "Seller, Commission, no party, item Apple, **per nag** {{30}}, From {{01-12-2026}}")],
        see=["Use a **future** From date so these test rules do not touch today's sales. Untick Active on them when you finish."],
        wrong=["Seller side must not offer Laga. Buyer side must not offer \"per lot\". Labour, munshiana and phone offer per nag and per lot only.", "Group works only when party and item are both empty."],
        differs=["A **rule** is for a rate that applies every time (for example Rahim always 5%). A rate **typed on one sale line** (for example 4%) is saved as manual (دستی) and changes only that line."]))

    # ---- 10 test a rule
    tr = frame("index", None, ("اصول آزمائیں", "Test a rule"), body=
        row(fld("بیوپاری", "Seller", "کریم", w=120), fld("گاہک", "Buyer", "اکرم", w=120), fld("قسم مال", "Item", "کیلا", w=110), fld("تاریخ", "Date", "01-12-2026", n=1, w=120),
            fld("تعداد", "Quantity", "20", w=70), fld("ریٹ", "Rate", "600", w=70), btn("آزمائیں", "Test", pri=True, n=2)) +
        tbl([("چارج", "Charge"), ("جیتنے والا اصول", "Winning rule"), ("سطح", "Level"), ("رقم", "Amount")],
            [["بیوپاری کمیشن", "default 6%", "6", "720"], ["گاہک کمیشن", "default 6%", "6", "720"], ["لاگا", "item group fruit, 8 per nag", "5", "160"]], n=3, hl={(0, 3), (1, 3), (2, 3)}), minh=300)
    out.append(page("Index", "@@", "Test a rule (nothing is saved)", "اصول آزمائیں", [("انڈکس", "Index"), ("کمیشن اصول", "Commission rules"), ("اصول آزمائیں", "Test a rule")], tr,
        steps=["1|Fill the boxes: seller, buyer, item, date, quantity, rate.", "2|Click [[آزمائیں|Test]].", "3|Read the table: each charge, the rule that won, its level and the amount."],
        typed=[("Test 1", "seller Karim, buyer Akram, item Banana, date today, qty {{20}}, rate {{600}}"),
               ("Test 2 (after adding the rules)", "date {{01-12-2026}}, seller **Rahim**, item Banana, 20 x 600"),
               ("Test 3", "date {{01-12-2026}}, seller **Rahim**, item **Apple**, 20 x 600"),
               ("Test 4", "date {{01-12-2026}}, seller **Karim**, item **Apple**, 20 x 600"),
               ("Test 5", "date {{01-12-2026}}, seller Karim, item Banana, 20 x 600")],
        see=["Test 1: seller commission **720**, buyer commission **720**, Laga **160** (level 5, item group).",
             "Test 2: **600** (Rahim + Banana 5%, level 2).  Test 3: **480** (Rahim, any item, 4%, level 3).",
             "Test 4: **600** (Apple, 30 per nag x 20, level 4).  Test 5: **720** (the default).",
             "Radish has no Laga. The screen also lists the levels it skipped."],
        tip="Nothing is saved by this screen. It is the safe way to check a rule."))
    return out
