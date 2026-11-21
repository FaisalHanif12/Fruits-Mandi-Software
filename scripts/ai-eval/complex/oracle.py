#!/usr/bin/env python3
"""The independent oracle of the complex-question list (plan 82, lane C).

Every expected figure of `questions.json` is worked out HERE, with SQL over the posted rows of a
database copy. It never calls the assistant, its tools or its prompts. Money is read as whole
rupees (paisa / 100). A line of a sale counts when `bikri_sales.is_active = 1`, its بکری is
`saved` and its lot is not removed (the rule of every report and view of the app); a journal figure
counts when the entry is `posted` (reversals are entries of their own, so they net out by
themselves); month-close entries are left out of income and expense (they only move it to capital).

Cross-checks (`crosscheck`) read the same figures a second way (the `analytics_facts` table that the
screens use, and the journal income accounts) and stop the build when two ways differ.
"""
import calendar
import datetime as dt
import sqlite3

D = dt.date


def rup(paisa):
    if paisa is None:
        return 0
    assert paisa % 100 == 0, f"not whole rupees: {paisa}"
    return paisa // 100


# ---- calendar: the app's period words (PRD FR-7.77) ------------------------------------------------


def add_months(day, n):
    """The same date n months back (n < 0) or forward, a month end clamped to the shorter month."""
    m0 = day.year * 12 + (day.month - 1) + n
    y, m = divmod(m0, 12)
    last = calendar.monthrange(y, m + 1)[1]
    return D(y, m + 1, min(day.day, last))


class Cal:
    def __init__(self, today):
        self.today = today

    def day(self, back=0):
        d = self.today - dt.timedelta(days=back)
        return d, d

    def week(self, back=0):
        monday = self.today - dt.timedelta(days=self.today.weekday())
        start = monday - dt.timedelta(days=7 * back)
        return start, start + dt.timedelta(days=6)

    def month(self, back=0):
        m0 = self.today.year * 12 + (self.today.month - 1) - back
        y, m = divmod(m0, 12)
        return D(y, m + 1, 1), D(y, m + 1, calendar.monthrange(y, m + 1)[1])

    def year(self, back=0):
        y = self.today.year - back
        return D(y, 1, 1), D(y, 12, 31)

    def last_days(self, n):
        return self.today - dt.timedelta(days=n), self.today

    def last_months(self, n):
        return add_months(self.today, -n), self.today

    def last_years(self, n):
        return add_months(self.today, -12 * n), self.today

    def named_month(self, month):
        """The latest month of that number that has started."""
        y = self.today.year if month <= self.today.month else self.today.year - 1
        return D(y, month, 1), D(y, month, calendar.monthrange(y, month)[1])


# ---- the books -------------------------------------------------------------------------------------

LINES_SQL = """
select s.entry_date, s.bikri_id, s.buyer_account_id, l.seller_account_id, l.item_id, s.qty,
       s.seller_amount_paisa, s.buyer_value_paisa, s.seller_comm_paisa, s.buyer_comm_paisa,
       s.laga_paisa, s.buyer_total_paisa, s.paid_paisa, s.rate_diff_paisa, s.chukta_name_en,
       s.chukta_person_key, l.id as lot_id
from bikri_sales s
join bikri_lots l on l.id = s.lot_id
join bikri_entries e on e.id = s.bikri_id
where s.is_active = 1 and e.status = 'saved' and l.removed_at is null
"""

FIELDS = (
    "date bikri buyer seller item qty gross bval sc bc laga btot paid rdiff chukta chukta_key lot"
).split()


class Books:
    def __init__(self, path, today):
        self.path = path
        self.cal = Cal(today)
        self.c = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
        self.c.row_factory = sqlite3.Row
        self._lines = [dict(zip(FIELDS, tuple(r))) for r in self.c.execute(LINES_SQL)]
        for r in self._lines:
            r["date"] = D.fromisoformat(r["date"])

    # -- names -----------------------------------------------------------------------------------
    def account(self, name_en=None, code=None):
        if code:
            row = self.c.execute("select id from accounts where code=?", (code,)).fetchone()
        else:
            rows = self.c.execute(
                "select id from accounts where lower(name_en)=lower(?) and party_type in "
                "('seller','buyer')", (name_en,)).fetchall()
            assert len(rows) == 1, f"{name_en}: {len(rows)} accounts"
            row = rows[0]
        return row[0]

    def name_of(self, account_id):
        r = self.c.execute("select name_en, name_ur, tag, code from accounts where id=?",
                           (account_id,)).fetchone()
        return dict(r)

    def item(self, name_en):
        rows = self.c.execute("select id from items where lower(name_en)=lower(?)",
                              (name_en,)).fetchall()
        assert len(rows) == 1, name_en
        return rows[0][0]

    # -- sales lines -----------------------------------------------------------------------------
    def lines(self, rng=None, buyer=None, seller=None, item=None):
        out = []
        buyers = None if buyer is None else (buyer if isinstance(buyer, (list, tuple, set)) else [buyer])
        sellers = None if seller is None else (seller if isinstance(seller, (list, tuple, set)) else [seller])
        items = None if item is None else (item if isinstance(item, (list, tuple, set)) else [item])
        for r in self._lines:
            if rng and not (rng[0] <= r["date"] <= rng[1]):
                continue
            if buyers is not None and r["buyer"] not in buyers:
                continue
            if sellers is not None and r["seller"] not in sellers:
                continue
            if items is not None and r["item"] not in items:
                continue
            out.append(r)
        return out

    @staticmethod
    def sums(lines):
        """Whole rupees. `comm` is the shop's commission on the lines: seller + buyer + laga.
        `buyer_comm` is what a buyer gives: buyer commission + laga (view C `total comm`)."""
        t = {k: 0 for k in ("gross", "bval", "sc", "bc", "laga", "btot", "paid", "qty", "rdiff")}
        for r in lines:
            for k in t:
                t[k] += r[k]
        out = {k: (v if k == "qty" else rup(v)) for k, v in t.items()}
        out["n"] = len(lines)
        out["comm"] = out["sc"] + out["bc"] + out["laga"]
        out["buyer_comm"] = out["bc"] + out["laga"]
        out["left"] = out["btot"] - out["paid"]
        return out

    def by_day(self, lines):
        days = {}
        for r in lines:
            days.setdefault(r["date"], []).append(r)
        return {d: self.sums(v) for d, v in sorted(days.items())}

    def by_month(self, lines):
        months = {}
        for r in lines:
            months.setdefault((r["date"].year, r["date"].month), []).append(r)
        return {m: self.sums(v) for m, v in sorted(months.items())}

    def by_party(self, lines, key):
        groups = {}
        for r in lines:
            groups.setdefault(r[key], []).append(r)
        return {k: self.sums(v) for k, v in groups.items()}

    # -- the journal -----------------------------------------------------------------------------
    def journal(self, code_like, rng=None, types=None, exclude=("month_close",)):
        """Σ(debit − credit) in whole rupees over posted entries on accounts whose code matches."""
        sql = ("select coalesce(sum(jl.dr_paisa - jl.cr_paisa),0) from journal_lines jl "
               "join journal_entries je on je.id=jl.entry_id join accounts a on a.id=jl.account_id "
               "where je.status='posted' and a.code like ?")
        args = [code_like]
        if rng:
            sql += " and je.entry_date between ? and ?"
            args += [rng[0].isoformat(), rng[1].isoformat()]
        if types:
            sql += " and je.posting_type in (%s)" % ",".join("?" * len(types))
            args += list(types)
        if exclude:
            sql += " and je.posting_type not in (%s)" % ",".join("?" * len(exclude))
            args += list(exclude)
        return rup(self.c.execute(sql, args).fetchone()[0])

    def journal_by_account(self, code_like, rng=None, exclude=("month_close", "opening")):
        sql = ("select a.id, a.code, a.name_en, a.name_ur, sum(jl.dr_paisa - jl.cr_paisa) "
               "from journal_lines jl join journal_entries je on je.id=jl.entry_id "
               "join accounts a on a.id=jl.account_id where je.status='posted' and a.code like ?")
        args = [code_like]
        if rng:
            sql += " and je.entry_date between ? and ?"
            args += [rng[0].isoformat(), rng[1].isoformat()]
        if exclude:
            sql += " and je.posting_type not in (%s)" % ",".join("?" * len(exclude))
            args += list(exclude)
        sql += " group by a.id order by a.code"
        return [
            {"id": r[0], "code": r[1], "name_en": r[2], "name_ur": r[3], "amount": rup(r[4])}
            for r in self.c.execute(sql, args)
        ]

    def balance(self, account_id, upto=None):
        sql = ("select coalesce(sum(jl.dr_paisa - jl.cr_paisa),0) from journal_lines jl "
               "join journal_entries je on je.id=jl.entry_id where je.status='posted' "
               "and jl.account_id=?")
        args = [account_id]
        if upto:
            sql += " and je.entry_date <= ?"
            args.append(upto.isoformat())
        return rup(self.c.execute(sql, args).fetchone()[0])

    def shop_expense(self, rng=None):
        """دکان خرچہ: every head under 0501, net of reversals (a positive number is money spent)."""
        return self.journal("0501%", rng)

    def income(self, rng=None):
        """Every income account (head 04), credit less debit, month-close entries left out."""
        return -self.journal("04%", rng)

    def profit(self, rng=None):
        return self.income(rng) - self.shop_expense(rng)

    # -- gardens and trips -----------------------------------------------------------------------
    def venture(self, kind, number):
        r = self.c.execute("select * from ventures where kind=? and number=?",
                           (kind, number)).fetchone()
        return dict(r)

    def venture_paid(self, venture_id, kinds=("installment", "cost")):
        sql = ("select coalesce(sum(amount_paisa),0) from venture_entries where venture_id=? "
               "and status='saved' and kind in (%s)" % ",".join("?" * len(kinds)))
        return rup(self.c.execute(sql, (venture_id, *kinds)).fetchone()[0])

    def venture_parts(self, venture_id, kinds=("installment", "cost")):
        sql = ("select p.partner_id, pt.name_en, p.source, sum(p.amount_paisa) from "
               "venture_entry_parts p join venture_entries e on e.id=p.entry_id "
               "join partners pt on pt.id=p.partner_id where e.venture_id=? and e.status='saved' "
               "and p.is_active=1 and e.kind in (%s) group by p.partner_id, p.source" %
               ",".join("?" * len(kinds)))
        return [
            {"partner": r[1], "source": r[2], "amount": rup(r[3])}
            for r in self.c.execute(sql, (venture_id, *kinds))
        ]

    # -- the second way (cross-check) -------------------------------------------------------------
    def facts_sums(self, rng=None):
        """Σ sign × value of `analytics_facts`, the table the analytics screens read."""
        sql = ("select coalesce(sum(sign*seller_amount_paisa),0), coalesce(sum(sign*seller_comm_paisa),0), "
               "coalesce(sum(sign*buyer_comm_paisa),0), coalesce(sum(sign*laga_paisa),0), "
               "coalesce(sum(sign*qty),0) from analytics_facts")
        args = []
        if rng:
            sql += " where entry_date between ? and ?"
            args = [rng[0].isoformat(), rng[1].isoformat()]
        g, sc, bc, lg, qty = self.c.execute(sql, args).fetchone()
        return {"gross": rup(g), "sc": rup(sc), "bc": rup(bc), "laga": rup(lg), "qty": qty}

    def crosscheck(self):
        """Both ways must agree on the whole books, and on every month."""
        mine = self.sums(self.lines())
        theirs = self.facts_sums()
        for k in ("gross", "sc", "bc", "laga", "qty"):
            assert mine[k] == theirs[k], f"{self.path}: {k}: sales lines {mine[k]} != facts {theirs[k]}"
        for (y, m), s in self.by_month(self.lines()).items():
            rng = (D(y, m, 1), D(y, m, calendar.monthrange(y, m)[1]))
            f = self.facts_sums(rng)
            for k in ("gross", "sc", "bc", "laga"):
                assert s[k] == f[k], f"{y}-{m:02d} {k}: {s[k]} != {f[k]}"
        # The journal's income accounts hold the same commission and laga as the sale lines
        # (04010001 seller, 04010002 buyer, 04010003 laga).
        assert -self.journal("04010001") == mine["sc"], "journal seller commission"
        assert -self.journal("04010002") == mine["bc"], "journal buyer commission"
        assert -self.journal("04010003") == mine["laga"], "journal laga"
        return True


# ---- more helpers: shares, gardens, trips, partners, estimates -------------------------------------


def split_by_bps(amount, shares):
    """PRD C10: every part rounded down, the leftover rupees one by one to the largest share (tie:
    the first listed). `shares` is a list of (key, bps) in the order the partners were created."""
    parts = {k: amount * bps // 10_000 for k, bps in shares}
    left = amount - sum(parts.values())
    order = sorted(range(len(shares)), key=lambda i: (-shares[i][1], i))
    i = 0
    while left > 0:
        parts[shares[order[i % len(order)]][0]] += 1
        left -= 1
        i += 1
    return parts


def avg_round(total, n):
    """Half up, the shared `avg_rate` rule for a per-month figure of an estimate (money agent 9.1)."""
    return (total * 2 + n) // (2 * n)


def months_between(books, a, b):
    """The calendar months (year, month) from a to b, both included."""
    out, y, m = [], a.year, a.month
    while (y, m) <= (b.year, b.month):
        out.append((y, m))
        m += 1
        if m == 13:
            y, m = y + 1, 1
    return out


def month_range(y, m):
    return (D(y, m, 1), D(y, m, calendar.monthrange(y, m)[1]))


def _venture_purchases(self, venture_id):
    return [r[0] for r in self.c.execute("select id from purchases where venture_id=? and status='saved'", (venture_id,))]


def _own_lot_credit(self, purchase_ids):
    """Seller credit of the lines of own-goods lots (C8, no lot charges on an own lot)."""
    ids = set(purchase_ids)
    rows = self.c.execute("select id from bikri_lots where purchase_id is not null").fetchall()
    own = {r[0] for r in rows if self.c.execute("select purchase_id from bikri_lots where id=?", (r[0],)).fetchone()[0] in ids}
    credit = 0
    sold = 0
    for r in self._lines:
        if r["lot"] in own:
            credit += r["gross"] - r["sc"]
            sold += r["qty"]
    return rup(credit), sold


def _trip(self, number):
    v = self.venture("trip", number)
    pids = self.venture_purchases(v["id"])
    paid = self.venture_paid(v["id"], ("purchase", "cost"))
    other = self.c.execute("select coalesce(sum(amount_paisa),0), coalesce(sum(qty),0) from venture_entries "
                           "where venture_id=? and kind='other_sale' and status='saved'", (v["id"],)).fetchone()
    credit, sold_here = self.own_lot_credit(pids)
    bought = self.c.execute("select coalesce(sum(qty),0), coalesce(sum(value_paisa),0), coalesce(sum(qty_sold),0), "
                            "coalesce(sum(qty_written_off),0) from purchases where venture_id=? and status='saved'",
                            (v["id"],)).fetchone()
    money_in = credit + rup(other[0])
    return {
        "id": v["id"], "status": v["status"], "paid": paid, "money_in": money_in,
        "own_shop_credit": credit, "other_shop": rup(other[0]),
        "bought_qty": bought[0], "bought_value": rup(bought[1]), "sold_qty": bought[2],
        "written_off_qty": bought[3], "result": money_in - paid,
        "shares": [(r[0], r[1]) for r in self.c.execute(
            "select partner_id, share_bps from venture_partners where venture_id=? order by partner_id", (v["id"],))],
    }


def _garden(self, number):
    v = self.venture("garden", number)
    inst = self.venture_paid(v["id"], ("installment",))
    cost = self.venture_paid(v["id"], ("cost",))
    other = rup(self.c.execute("select coalesce(sum(amount_paisa),0) from venture_entries where venture_id=? "
                               "and kind='other_sale' and status='saved'", (v["id"],)).fetchone()[0])
    shop_lines = self.sums(self.lines(seller=v["account_id"]))
    # the garden's account holds its sales money after commission and the money from other shops
    money_in = -self.balance(v["account_id"])
    return {
        "id": v["id"], "status": v["status"], "rent": rup(v["rent_paisa"]), "installments": inst, "costs": cost,
        "paid": inst + cost, "rent_due": rup(v["rent_paisa"]) - inst, "other_shop": other,
        "shop_gross": shop_lines["gross"], "shop_comm": shop_lines["comm"], "money_in": money_in,
        "balance": money_in - inst - cost,
        "shares": [(r[0], r[1], r[2]) for r in self.c.execute(
            "select partner_id, share_bps, source from venture_partners where venture_id=? order by partner_id", (v["id"],))],
    }


def _personal_taken(self, rng=None):
    """ذاتی خرچہ taken by each partner (vouchers of the type, saved), whole rupees."""
    sql = ("select pt.name_en, sum(v.amount_paisa) from vouchers v join partners pt on pt.id=v.partner_id "
           "where v.type='personal_expense' and v.status='saved'")
    args = []
    if rng:
        sql += " and v.entry_date between ? and ?"
        args = [rng[0].isoformat(), rng[1].isoformat()]
    sql += " group by pt.id order by pt.id"
    return {r[0]: rup(r[1]) for r in self.c.execute(sql, args)}


def _partner_profit(self, rng):
    """Each partner's income share less expense share in the period (partner_share_rows)."""
    sql = ("select pt.name_en, r.kind, sum(r.amount_paisa) from partner_share_rows r "
           "join partners pt on pt.id=r.partner_id where r.entry_date between ? and ? group by pt.id, r.kind order by pt.id")
    out = {}
    for name, kind, amt in self.c.execute(sql, (rng[0].isoformat(), rng[1].isoformat())):
        out.setdefault(name, {"income": 0, "expense": 0})[kind] = rup(amt)
    for v in out.values():
        v["profit"] = v["income"] + v["expense"]
    return out


Books.venture_purchases = _venture_purchases
Books.own_lot_credit = _own_lot_credit
Books.trip = _trip
Books.garden = _garden
Books.personal_taken = _personal_taken
Books.partner_profit = _partner_profit


# ---- the measure columns of a sale: gross, net, buyer bill, commission with and without laga -------

LOTS_SQL = """
select l.id, l.seller_account_id, l.item_id, l.gross_paisa, l.net_paisa, l.seller_comm_paisa,
       l.freight_paisa, l.freight_paid_by, l.labour_paisa, l.munshiana_paisa, l.phone_paisa,
       l.cash_paid_paisa, l.store_rent_paisa, e.entry_date
from bikri_lots l join bikri_entries e on e.id = l.bikri_id
where e.status = 'saved' and l.removed_at is null
"""


def _lot_rows(self):
    rows = [dict(r) for r in self.c.execute(LOTS_SQL)]
    for r in rows:
        # PRD C6 and C7: net = gross less the deductions (freight only when the firm paid it).
        freight = r["freight_paisa"] if r["freight_paid_by"] == "firm" else 0
        ded = (r["seller_comm_paisa"] + freight + r["labour_paisa"] + r["munshiana_paisa"] +
               r["phone_paisa"] + r["cash_paid_paisa"] + r["store_rent_paisa"])
        assert r["gross_paisa"] - ded == r["net_paisa"], f"lot {r['id']}: net is not gross less deductions"
    return rows


def _net(self, seller=None):
    """صافی بکری (net sale): the lots' net, gross less every deduction (PRD C7), whole rupees."""
    sellers = None if seller is None else (seller if isinstance(seller, (list, tuple, set)) else [seller])
    return rup(sum(r["net_paisa"] for r in self.lot_rows() if sellers is None or r["seller_account_id"] in sellers))


Books.lot_rows = _lot_rows
Books.net = _net
