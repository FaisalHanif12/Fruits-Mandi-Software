#!/usr/bin/env python3
"""Builds `questions.json` and `QUESTIONS.md`: the complex-question list of plan 82, lane C.

    python3 -I scripts/ai-eval/complex/build_questions.py --dev DEV.db --year YEAR.db

DEV.db is a COPY of the dev database (today 20-11-2026). YEAR.db is the rich year of the
robustness oracle (`export_the_year_snapshot`, today 05-11-2026). Every expected figure is worked out
by `oracle.py` (SQL over posted rows, cross-checked two ways). Nothing here is typed from the
assistant. A figure is whole rupees. `req` figures must all be in the answer; `opt` figures may be.
A figure is a number, or `["~", centre, tolerance]` (a split part that may differ by a rupee).
`readings` are the honest readings of the wording: the answer passes when all `req` figures of ONE
reading are shown.
"""
import argparse
import datetime as dt
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import oracle as O  # noqa: E402

D = dt.date
QUESTIONS = []


def _alts(v):
    """A figure is a number, an approximation ["~", centre, tolerance], or a list of alternatives."""
    if isinstance(v, list) and v and v[0] == "~":
        return [v]
    return v if isinstance(v, list) else [v]


def R(req, opt=None):
    return {"req": {k: _alts(v) for k, v in req.items()},
            "opt": {k: _alts(v) for k, v in (opt or {}).items()}}


def add(qid, world, lang, text, readings=None, names=(), names_any=(), outcomes=("answered",), zero=False,
        follow_of=None, group="", kind="", period="", source="", note="", ambiguous=False,
        must_not_show=(), fast=False, wrong_columns=None, direction=None, search_names=()):
    QUESTIONS.append({
        "id": qid, "world": world, "lang": lang, "question": text, "follow_of": follow_of,
        "group": group, "kind": kind, "period": period, "source": source, "note": note,
        "readings": readings or [], "names": list(names), "names_any": list(names_any),
        "outcomes": list(outcomes), "zero": zero, "ambiguous": ambiguous,
        "must_not_show": list(must_not_show), "fast": fast,
        "wrong_columns": dict(wrong_columns or {}),
        "direction": direction, "search_names": list(search_names),
    })


def per(rng):
    return f"{rng[0].isoformat()} to {rng[1].isoformat()}"


def build_dev(path):
    b = O.Books(path, D(2026, 11, 20))
    b.crosscheck()
    cal = b.cal
    mani, haroon, ans = (b.account(n) for n in ("Mani", "Haroon", "Ans"))
    faisal, ehsan, adnan = (b.account(n) for n in ("Faisal", "Ehsan", "adnan"))
    S = lambda rng=None, **f: b.sums(b.lines(rng, **f))
    month, last_month = cal.month(0), cal.month(1)
    allr = None
    w = "dev"

    # D01 several buyers together, all time: what each gave the shop
    s = {n: S(buyer=a) for n, a in (("Mani", mani), ("Haroon", haroon), ("Ans", ans))}
    tot = sum(v["buyer_comm"] for v in s.values())
    tot2 = sum(v["bc"] for v in s.values())
    add("D01", w, "roman", "mani, haroon aur ans ne abhi tak shop ko kitni commission di ?",
        [R({"mani": s["Mani"]["buyer_comm"], "haroon": s["Haroon"]["buyer_comm"], "ans": s["Ans"]["buyer_comm"]},
           {"total": tot}),
         R({"mani": s["Mani"]["bc"], "haroon": s["Haroon"]["bc"], "ans": s["Ans"]["bc"]}, {"total": tot2})],
        names=["Mani", "Haroon", "Ans"], group="several-buyers", kind="commission", period="up to now",
        source="commission page / view C (buyer commission + laga)")

    # D02 a period with nothing in it: zero, never "not found"
    add("D02", w, "roman", "mani ne pichle mahine kitni khareedari ki aur kitni commission di ?",
        zero=True, names=["Mani"], group="empty-period", kind="zero", period=per(last_month),
        source="view C", note="October has only opening balances: an honest 0")

    # D03 every buyer, this month: purchase and commission each
    req = {}
    for n, a in (("mani", mani), ("haroon", haroon), ("ans", ans)):
        v = S(month, buyer=a)
        req[f"{n}.value"] = v["bval"]
        req[f"{n}.comm"] = v["buyer_comm"]
    req2 = {}
    for n, a in (("mani", mani), ("haroon", haroon), ("ans", ans)):
        v = S(month, buyer=a)
        req2[f"{n}.value"] = v["bval"]
        req2[f"{n}.comm"] = v["bc"]
    add("D03", w, "roman", "is mahine har gahaq ne alag alag kitni bikri li aur kitni commission di ?",
        [R(req), R(req2)], names=["Mani", "Haroon", "Ans"], group="buyers-list", kind="sale+commission",
        period=per(month), source="view C")

    # D04 the top buyer by commission
    v = S(month, buyer=haroon)
    add("D04", w, "roman", "is mahine sab se zyada commission kis gahaq ne di ?",
        [R({"haroon": v["buyer_comm"]}), R({"haroon": v["bc"]})], names=["Haroon"],
        group="ranked-buyers", kind="top-1", period=per(month), source="view C ranked by commission",
        wrong_columns={"commission_on_his_lines_both_sides": v["comm"]},
        note="the seller's commission on his lines is not commission HE gave")

    # D05 several sellers together: their sale
    req = {n: S(seller=a)["gross"] for n, a in (("faisal", faisal), ("ehsan", ehsan), ("adnan", adnan))}
    add("D05", w, "roman", "faisal, ehsan aur adnan beopariyon ki abhi tak ki bikri kitni hui ?",
        [R(req, {"total": sum(req.values())})], names=["Faisal", "Ehsan", "adnan"],
        group="several-sellers", kind="sale", period="up to now", source="view B (gross sale)",
        note="adnan is also a partner: asking which one is honest only if no sale figure is shown")

    # D06 what the shop earned on one seller's goods
    v = S(seller=ehsan)
    add("D06", w, "roman", "ehsan ke maal se shop ko abhi tak kitni commission mili ?",
        [R({"comm_on_goods": v["comm"]}), R({"seller_comm": v["sc"]})], names=["Ehsan"],
        group="seller-commission", kind="commission", period="up to now",
        source="view B / commission page seller filter")

    # D07 two weeks
    tw, lw = cal.week(0), cal.week(1)
    a, c = S(tw)["gross"], S(lw)["gross"]
    add("D07", w, "roman", "is hafte ki bikri aur pichle hafte ki bikri ka farq batao",
        [R({"this_week": a, "diff": a - c}, {"last_week": c}),
         R({"this_week": S(tw)["btot"]})],
        group="compare-periods", kind="sale", period=f"{per(tw)} vs {per(lw)}", source="view A compare",
        note="direction: this week more", direction="up")

    # D08 last 7 days
    r7 = cal.last_days(7)
    add("D08", w, "roman", "pichle 7 din mein shop ko kitni commission mili ?",
        [R({"commission": S(r7)["comm"]})], group="last-n-days", kind="commission", period=per(r7),
        source="view A", note="last 7 days = 13-11 to 20-11")

    # D09 two named days
    d1, d2 = D(2026, 11, 6), D(2026, 11, 16)
    add("D09", w, "roman", "6 november aur 16 november ki commission alag alag batao",
        [R({"6 nov": S((d1, d1))["comm"], "16 nov": S((d2, d2))["comm"]})],
        group="named-days", kind="commission", period="06-11 and 16-11", source="view A by day")

    # D10 month by month this year
    ms = b.by_month(b.lines(cal.year(0)))
    add("D10", w, "roman", "is saal mahine ke hisab se kitni commission bani ?",
        [R({f"{y}-{m:02d}": v["comm"] for (y, m), v in ms.items()})], group="month-by-month",
        kind="commission", period=per(cal.year(0)), source="view G / A grouped by month",
        note="only November has sales; October may show 0")

    # D11 garden by partner: the partner code is set in this copy
    g = b.garden(1)
    add("D11", w, "roman", "pump wala bagh mein abhi tak kitna paisa laga aur partner ke hisab se kitna ?",
        [R({"paid": g["paid"]})], outcomes=["answered", "locked"], group="garden", kind="garden-paid",
        period="up to now", source="R36 garden",
        must_not_show=[100000], note="partner code is set: the partner parts (100,000 each) must stay behind it")

    # D12 garden balance
    add("D12", w, "roman", "bagh ka hisab batao: kitna paisa diya, kitni bikri aayi aur ab balance kya hai ?",
        [R({"paid": g["paid"]}, {"rent": g["rent"], "rent_due": g["rent_due"], "money_in": g["money_in"]})],
        group="garden", kind="garden-balance", period="up to now", source="R36 garden",
        note=f"running balance is -{g['paid']} (money in {g['money_in']} less paid {g['paid']})")

    # D13 a trip with nothing bought
    add("D13", w, "roman", "pehle safar ka nateeja kya raha, kitna nafa ya nuksan ?", zero=True,
        group="trip", kind="trip-result", period="up to now", source="R33 trips",
        note="the one trip has no purchase: an honest 0 or 'nothing bought'")

    # D14 / D15 expenses: nothing was spent
    add("D14", w, "roman", "pichle saal se is mahine tak shop ne kharche par kitna paisa lagaya ?", zero=True,
        group="expenses", kind="zero", period="2025-01-01 to 2026-11-30", source="R29",
        note=f"shop expenses in the books: {b.shop_expense()}")
    add("D15", w, "roman", "bijli ka kharcha is mahine kitna hua aur pichle mahine kitna ?", zero=True,
        names=["Bijli"], group="expenses", kind="zero", period="this and last month", source="R29 head",
        names_any=[])

    # D16 profit this month
    add("D16", w, "roman", "is mahine shop ka nafa kitna hai ?", [R({"profit": b.profit(month)})],
        group="profit", kind="profit", period=per(month), source="R30 (income less shop expenses)",
        note=f"expenses {b.shop_expense(month)}")

    # D17 a buyer's balance and his month
    v = S(month, buyer=mani)
    add("D17", w, "roman", "mani ka kitna bakaya hai aur is mahine usne kitna maal liya ?",
        [R({"balance": b.balance(mani), "purchase": v["bval"]}), R({"balance": b.balance(mani), "purchase": v["btot"]})],
        names=["Mani"], group="balance+sale", kind="balance+sale", period=per(month),
        source="party card / R13 and view C")

    # D18 buyers with the biggest balance
    bal = sorted(((b.balance(a), n) for a, n in ((mani, "mani"), (haroon, "haroon"), (ans, "ans"))), reverse=True)
    add("D18", w, "roman", "kin gahakon ke zimme sab se zyada paisa hai, top 3 batao",
        [R({n: v for v, n in bal})], names=[n.capitalize() for _, n in bal], group="ranked-dues",
        kind="top-3 dues", period="today", source="R20",
        note="order: " + " > ".join(f"{n} {v}" for v, n in bal))

    # D19 a buyer over the last 3 months
    r3 = cal.last_months(3)
    v = S(r3, buyer=ans)
    add("D19", w, "roman", "ans ne pichle 3 mahine mein kitna maal liya aur kitni commission di ?",
        [R({"purchase": v["bval"], "comm": v["buyer_comm"]}), R({"purchase": v["bval"], "comm": v["bc"]})],
        names=["Ans"], group="last-n-months", kind="sale+commission", period=per(r3), source="view C")

    # D20 Urdu letters, one buyer
    v = S(buyer=mani)
    add("D20", w, "urdu", "مانی نے اب تک دکان کو کتنا کمیشن دیا ہے؟",
        [R({"comm": v["buyer_comm"]}), R({"comm": v["bc"]})], names=["Mani"], group="one-buyer",
        kind="commission", period="up to now", source="commission page / view C")

    # D21 Urdu letters, two buyers
    h, a_ = S(month, buyer=haroon), S(month, buyer=ans)
    add("D21", w, "urdu", "ہارون اور انس نے اس مہینے کتنی خریداری کی اور کتنا کمیشن دیا؟",
        [R({"haroon.value": h["bval"], "haroon.comm": h["buyer_comm"], "ans.value": a_["bval"], "ans.comm": a_["buyer_comm"]}),
         R({"haroon.value": h["bval"], "haroon.comm": h["bc"], "ans.value": a_["bval"], "ans.comm": a_["bc"]})],
        names=["Haroon", "Ans"], group="several-buyers", kind="sale+commission", period=per(month), source="view C")

    # D22 English, two buyers together
    add("D22", w, "english", "How much commission did the shop earn from Haroon and Ans together this month?",
        [R({"together": h["buyer_comm"] + a_["buyer_comm"]}), R({"together": h["bc"] + a_["bc"]})],
        search_names=["Haroon", "Ans"], group="several-buyers", kind="commission-sum", period=per(month), source="view C + calculator")

    # D23 English, the top seller and the shop's commission on his goods
    top = max(((S(month, seller=s_)["gross"], s_) for s_ in (faisal, ehsan, adnan)))
    v = S(month, seller=top[1])
    add("D23", w, "english",
        "Which seller sent the most goods this month and how much commission did the shop get on his goods?",
        [R({"comm": v["comm"]}, {"gross": v["gross"], "qty": v["qty"]}),
         R({"comm": v["sc"]}, {"gross": v["gross"], "qty": v["qty"]})],
        names=[b.name_of(top[1])["name_en"]], group="ranked-sellers", kind="top-1", period=per(month),
        source="view B", note="'most goods' may mean quantity (nag) or sale: both are honest")

    # D24 English, two months
    add("D24", w, "english", "Compare this month's commission with last month and tell me the difference",
        [R({"this": S(month)["comm"], "last": 0, "diff": S(month)["comm"]})], group="compare-periods",
        kind="commission", period=f"{per(month)} vs {per(last_month)}", source="view A compare", direction="up",
        note="last month has none: the difference is the whole figure, direction up")

    # D25 English, a percent
    mc, tot = S(month, buyer=mani), S(month)
    add("D25", w, "english", "What share of the shop's total commission came from Mani this month, in percent?",
        [R({"pct": round(mc["buyer_comm"] * 100 / tot["comm"], 2)}),
         R({"pct": round(mc["bc"] * 100 / tot["comm"], 2)}),
         R({"pct": round(mc["comm"] * 100 / tot["comm"], 2)})],
        names=["Mani"], group="percent", kind="calc-percent", period=per(month), source="calculator",
        note="three honest readings: his buyer commission+laga, commission only, or all commission on his lines")

    # D26 chain of three
    f, e = S(seller=faisal)["gross"], S(seller=ehsan)["gross"]
    add("D26a", w, "roman", "faisal ki abhi tak ki bikri kitni hai ?", [R({"faisal": f})], names=["Faisal"],
        group="follow-up", kind="sale", period="up to now", source="view B")
    add("D26b", w, "roman", "aur ehsan ki ?", [R({"ehsan": e})], names=["Ehsan"], follow_of="D26a",
        group="follow-up", kind="sale", period="up to now", source="view B")
    add("D26c", w, "roman", "dono ki mila kar kitni hui ?", [R({"sum": f + e})], follow_of="D26b",
        group="follow-up", kind="calc-sum", period="up to now", source="calculator")

    # D27 two buyers, two periods
    t1 = S(month, buyer=mani)["buyer_comm"] + S(month, buyer=ans)["buyer_comm"]
    t2 = S(month, buyer=mani)["bc"] + S(month, buyer=ans)["bc"]
    add("D27", w, "roman", "mani aur ans se is mahine aur pichle mahine kitni commission mili ?",
        search_names=["Mani", "Ans"],
        readings=[R({"this_month": t1}), R({"this_month": t2}),
         R({"mani": S(month, buyer=mani)["buyer_comm"], "ans": S(month, buyer=ans)["buyer_comm"]}),
         R({"mani": S(month, buyer=mani)["bc"], "ans": S(month, buyer=ans)["bc"]})], group="two-parties-two-periods",
        kind="commission", period=f"{per(month)} and {per(last_month)}", source="view C")

    # D28 chukta buyers
    v = S(month, buyer=b.account(code="0030"))
    add("D28", w, "roman", "chukta khate walon ne is mahine kitna maal khareeda ?",
        [R({"purchase": v["bval"]}), R({"purchase": v["btot"]})], group="chukta", kind="sale",
        period=per(month), source="view C (chukta rows)",
        note="chukta buyers have no ledger; a figure or an honest 'no ledger' are both fine")

    # D29 a seller's day with his buyers
    day = D(2026, 11, 6)
    rows = b.by_party(b.lines((day, day), seller=ehsan), "buyer")
    req = {b.name_of(k)["name_en"].lower(): v["bval"] for k, v in rows.items()}
    add("D29", w, "roman", "6 november ko ehsan ne kitna maal bheja aur kis kis gahaq ne kitna liya ?",
        [R({"ehsan.gross": S((day, day), seller=ehsan)["gross"], **req}),
         R({"ehsan.gross": S((day, day), seller=ehsan)["gross"],
            **{k.replace(".gross", "") + ".qty": v for k, v in
               {b.name_of(kk)["name_en"].lower(): vv["qty"] for kk, vv in rows.items()}.items()}})],
        names=["Ehsan", "Mani", "Ans", "Haroon"],
        group="seller-day", kind="sale", period=str(day), source="R35 / view B")

    # D30 an item
    malta = b.item("Malta")
    v = S(item=malta)
    add("D30", w, "roman", "malta ki abhi tak ki bikri aur commission kitni hai ?",
        [R({"gross": v["gross"], "comm": v["comm"]}), R({"gross": v["gross"], "comm": v["buyer_comm"]})],
        names_any=["Malta"], group="item", kind="sale+commission", period="up to now", source="view A")

    # D31 payable and receivable together
    add("D31", w, "roman", "ehsan ko abhi tak kitna paisa dena hai aur ans se kitna lena hai ?",
        [R({"ehsan_payable": abs(b.balance(ehsan)), "ans_due": b.balance(ans)})], names=["Ehsan", "Ans"],
        group="balances", kind="balance", period="today", source="party card / R13",
        note="Ehsan: the shop owes him (جمع); Ans owes the shop (بنام)")

    # D32 sale, commission, expense together
    add("D32", w, "roman", "pichle saal se is mahine tak shop ki bikri, commission aur kharcha teeno batao",
        [R({"sale": S()["gross"], "commission": S()["comm"]}, {"expense": b.shop_expense()})],
        group="sale+commission+expense", kind="three-figures", period="2025-01-01 to 2026-11-30",
        source="view A/G + R29")
    # ---- measure words: the columns of the same sale differ, and only the asked one is right -------
    allr_ = S()
    net_all = b.net()
    cols = {"gross": allr_["gross"], "net": net_all, "buyer_total": allr_["btot"]}
    dues_all = sum(b.balance(a) for a in (mani, haroon, ans))
    only = lambda keep: {k: v for k, v in cols.items() if k != keep}
    add("M01", w, "roman", "abhi tak ki safi bikri kitni hai ?", [R({"net_sale": net_all})], group="measure-words",
        kind="net-sale", period="up to now", source="R6 total / lot net (صافی بکری)",
        wrong_columns=only("net"), note="net = gross less commission, freight, labour, munshiana, phone, cash paid")
    add("M02", w, "roman", "abhi tak ki khaam bikri kitni hui ?", [R({"gross_sale": allr_["gross"]})],
        group="measure-words", kind="gross-sale", period="up to now", source="view A / R2",
        wrong_columns=only("gross"))
    add("M03", w, "roman", "abhi tak gahakon ko kul kitne ke bill bane (commission aur laga mila kar) ?",
        [R({"buyer_bills": allr_["btot"]})], group="measure-words", kind="buyer-total", period="up to now",
        source="R4/R5 (رقم گاہک)",
        wrong_columns={**only("buyer_total"), "commission_and_laga": allr_["buyer_comm"], "dues_outstanding": dues_all})
    add("M04", w, "urdu", "اب تک کی صافی بکری کتنی ہے؟", [R({"net_sale": net_all})], group="measure-words",
        kind="net-sale", period="up to now", source="R6 total", wrong_columns=only("net"))
    add("M05", w, "urdu", "اب تک کی خام بکری کتنی ہوئی؟", [R({"gross_sale": allr_["gross"]})],
        group="measure-words", kind="gross-sale", period="up to now", source="view A", wrong_columns=only("gross"))
    add("M06", w, "english", "What is the net sale so far?", [R({"net_sale": net_all})], group="measure-words",
        kind="net-sale", period="up to now", source="R6 total", wrong_columns=only("net"))
    add("M07", w, "english", "What is the total gross sale so far?", [R({"gross_sale": allr_["gross"]})],
        group="measure-words", kind="gross-sale", period="up to now", source="view A", wrong_columns=only("gross"))
    add("M08", w, "english", "What is the total of all buyer bills so far?", [R({"buyer_bills": allr_["btot"]})],
        group="measure-words", kind="buyer-total", period="up to now", source="R4/R5",
        wrong_columns={**only("buyer_total"), "commission_and_laga": allr_["buyer_comm"], "dues_outstanding": dues_all})

    e = S(seller=ehsan)
    e_net = b.net(ehsan)
    ec = {"gross": e["gross"], "net": e_net, "buyer_total": e["btot"]}
    add("M09", w, "roman", "ehsan ki safi bikri kitni hai ?", [R({"net_sale": e_net})], names=["Ehsan"],
        group="measure-words", kind="net-sale", period="up to now", source="R6 / lot net",
        wrong_columns={k: v for k, v in ec.items() if k != "net"})
    add("M10", w, "roman", "ehsan ki khaam bikri kitni hai ?", [R({"gross_sale": e["gross"]})], names=["Ehsan"],
        group="measure-words", kind="gross-sale", period="up to now", source="view B",
        wrong_columns={k: v for k, v in ec.items() if k != "gross"})
    m_ = S(buyer=mani)
    add("M11", w, "roman", "mani ke bill ki kul raqam kitni hai (commission aur laga samet) ?",
        [R({"buyer_bill": m_["btot"]})], names=["Mani"], group="measure-words", kind="buyer-total",
        period="up to now", source="R13 / R4",
        wrong_columns={"value": m_["bval"], "commission+laga": m_["buyer_comm"], "balance_outstanding": b.balance(mani)})
    add("M12", w, "roman", "mani ne kitne ke maal ki khareedari ki (commission aur laga ke baghair) ?",
        [R({"purchase_value": m_["bval"]})], names=["Mani"], group="measure-words", kind="buyer-value",
        period="up to now", source="view C purchase value",
        wrong_columns={"buyer_total": m_["btot"], "commission+laga": m_["buyer_comm"]})
    add("M13", w, "roman", "mani ne commission aur laga alag alag kitna diya ?",
        [R({"commission": m_["bc"], "laga": m_["laga"]}, {"together": m_["buyer_comm"]})], names=["Mani"],
        group="measure-words", kind="commission-and-laga", period="up to now", source="view C columns",
        wrong_columns={"value": m_["bval"], "buyer_total": m_["btot"]})
    add("M14", w, "roman", "shop ne abhi tak sirf laga kitna kamaya ?", [R({"laga": allr_["laga"]})],
        group="measure-words", kind="laga-only", period="up to now", source="view A laga column",
        wrong_columns={"all_commission": allr_["comm"], "buyer_side": allr_["buyer_comm"],
                       "commission_without_laga": allr_["sc"] + allr_["bc"]})
    add("M15", w, "english", "How much commission, not counting laga, has the shop earned so far?",
        [R({"commission_without_laga": allr_["sc"] + allr_["bc"]}, {"laga": allr_["laga"]}),
         R({"seller_commission": allr_["sc"], "buyer_commission": allr_["bc"]})],
        group="measure-words", kind="commission-without-laga", period="up to now",
        source="view A seller + buyer commission", wrong_columns={"all_commission": allr_["comm"]})
    add("M16", w, "roman", "ehsan ne shop ko kitni commission di aur shop ne uske maal par kul kitna kamaya ?",
        [R({"given_by_seller": e["sc"], "earned_on_goods": e["comm"]})], names=["Ehsan"],
        group="measure-words", kind="commission-given-vs-earned", period="up to now", source="view B",
        wrong_columns={"gross": e["gross"], "net": e_net, "buyer_total": e["btot"]})
    add("M21", w, "urdu", "احسان کی صافی بکری کتنی ہے؟", [R({"net_sale": e_net})], names=["Ehsan"],
        group="measure-words", kind="net-sale", period="up to now", source="R6 / lot net",
        wrong_columns={k: v for k, v in ec.items() if k != "net"})
    h_ = S(buyer=haroon)
    add("M22", w, "english", "How much commission, not counting laga, did the shop earn from Haroon so far?",
        [R({"commission_without_laga": h_["bc"]}, {"laga": h_["laga"]})], names=["Haroon"], group="measure-words",
        kind="commission-without-laga", period="up to now", source="view C buyer commission column",
        wrong_columns={"commission+laga": h_["buyer_comm"], "value": h_["bval"], "buyer_total": h_["btot"]})
    add("M23", w, "english",
        "How much commission did Ehsan give the shop, and how much did the shop earn in all on his goods?",
        [R({"given_by_seller": e["sc"], "earned_on_goods": e["comm"]})], names=["Ehsan"], group="measure-words",
        kind="commission-given-vs-earned", period="up to now", source="view B",
        wrong_columns={"gross": e["gross"], "net": e_net, "buyer_total": e["btot"]})
    add("M24", w, "urdu", "ہارون کے بلوں کی کل رقم کتنی ہے؟", [R({"buyer_bill": h_["btot"]})], names=["Haroon"],
        group="measure-words", kind="buyer-total", period="up to now", source="R13 / R4",
        wrong_columns={"value": h_["bval"], "commission+laga": h_["buyer_comm"], "balance_outstanding": b.balance(haroon)})
    add("M25", w, "roman", "gahakon se hum ne abhi tak kitna commission aur kitna laga alag alag liya ?",
        [R({"buyer_commission": allr_["bc"], "laga": allr_["laga"]}, {"together": allr_["buyer_comm"]})],
        group="measure-words", kind="commission-and-laga", period="up to now", source="view A buyer side",
        wrong_columns={"seller_commission": allr_["sc"], "all_commission": allr_["comm"], "gross": allr_["gross"]})
    return b



def build_year(path):
    b = O.Books(path, D(2026, 11, 5))
    b.crosscheck()
    cal = b.cal
    w = "year"
    S = lambda rng=None, **f: b.sums(b.lines(rng, **f))
    nm = lambda a: b.name_of(a)["name_en"]
    acct = b.account
    month, last_month = cal.month(0), cal.month(1)
    ytd = cal.year(0)
    start = (D(2025, 1, 1), D(2026, 11, 30))
    heads = lambda rng: {r["name_en"]: r["amount"] for r in b.journal_by_account("0501%", rng)}

    # Y01 expenses since last year
    add("Y01", w, "roman", "pichle saal se is mahine tak shop ne kharche par kitna paisa lagaya ?",
        [R({"total": b.shop_expense(start)})], group="expenses", kind="expense-range", period=per(start),
        source="R29 / view M", note="Umar's own example. The voucher-only reading is 2,091,500: a salary deduction of 2,000 is also a shop expense")

    # Y02 by head, last 6 months
    r6 = cal.last_months(6)
    h = heads(r6)
    add("Y02", w, "roman", "pichle 6 mahine ka kharcha head ke hisab se alag alag batao",
        [R({k.lower(): v for k, v in h.items()}, {"total": b.shop_expense(r6)})], group="expenses",
        kind="expense-by-head", period=per(r6), source="R29 by head / view M")

    # Y03 electricity month by month
    rows = {}
    for (y, m) in O.months_between(b, D(2025, 11, 1), D(2026, 11, 1)):
        rng = O.month_range(y, m)
        rows[f"{y}-{m:02d}"] = sum(r["amount"] for r in b.journal_by_account("05010002", rng))
    req = {k: v for k, v in rows.items() if v != 0}
    add("Y03", w, "roman", "bijli ka kharcha mahine ke hisab se pichle saal se ab tak kitna raha ?",
        [R(req, {"whole_period_total": sum(rows.values())})], group="expenses", kind="head-by-month", period="2025-11 to 2026-11", source="view M by month",
        note="12 months; a month with 0 may be left out")

    # Y04 English, two heads, two periods
    lh, th = heads(last_month), heads(month)
    add("Y04", w, "english", "How much did the shop spend on salaries and tea last month and this month so far?",
        [R({"salaries_last": lh["Salaries"], "tea_last": lh["Tea"], "salaries_this": th["Salaries"],
            "tea_this": th["Tea"]})], group="expenses", kind="two-heads-two-periods",
        period=f"{per(last_month)} / {per(month)}", source="R29 by head")

    # Y05 commission last month and this month
    add("Y05", w, "roman", "pichle mahine kul commission kitni bani aur is mahine ab tak kitni ?",
        [R({"last": S(last_month)["comm"], "this": S(month)["comm"]})], group="compare-periods",
        kind="commission", period=f"{per(last_month)} / {per(month)}", source="view A")

    # Y06 top 3 buyers by commission, last 3 months
    r3 = cal.last_months(3)
    by = b.by_party(b.lines(r3), "buyer")
    rank = sorted(by.items(), key=lambda kv: -kv[1]["buyer_comm"])[:3]
    rank2 = sorted(by.items(), key=lambda kv: -kv[1]["bc"])[:3]
    add("Y06", w, "roman", "pichle 3 mahine mein sab se zyada commission dene wale 3 gahaq kaun se hain ?",
        [R({nm(k): v["buyer_comm"] for k, v in rank}), R({nm(k): v["bc"] for k, v in rank2})],
        names=[nm(k) for k, _ in rank], group="ranked-buyers", kind="top-3", period=per(r3), source="view C ranked",
        note="order: " + " > ".join(f"{nm(k)} {v['buyer_comm']}" for k, v in rank))

    # Y07 top 5 sellers by sale since last year
    by = b.by_party(b.lines(), "seller")
    rank = sorted(((k, v) for k, v in by.items() if nm(k) not in ("Own Goods (Trips)", "Old Closed Account")),
                  key=lambda kv: -kv[1]["gross"])[:5]
    add("Y07", w, "roman", "pichle saal se is mahine tak sab se zyada bikri karne wale 5 beopari kaun hain ?",
        [R({nm(k): v["gross"] for k, v in rank})], names=[nm(k) for k, _ in rank], group="ranked-sellers",
        kind="top-5", period="up to now", source="view B ranked",
        note="order: " + " > ".join(f"{nm(k)} {v['gross']}" for k, v in rank))

    # Y08 two sellers: sale and commission this year
    rahim, karim = acct("Rahim"), acct("Karim")
    req_a, req_b = {}, {}
    for n, a in (("rahim", rahim), ("karim", karim)):
        v = S(ytd, seller=a)
        req_a[f"{n}.sale"], req_a[f"{n}.comm"] = v["gross"], v["sc"]
        req_b[f"{n}.sale"], req_b[f"{n}.comm"] = v["gross"], v["comm"]
    add("Y08", w, "roman", "rahim aur karim ne is saal shop ko kitni commission di aur kitni bikri ki ?",
        [R(req_a), R(req_b)], names=["Rahim", "Karim"], group="several-sellers", kind="sale+commission",
        period=per(ytd), source="view B", note="commission given by the seller, or the shop's commission on his goods")

    # Y09 a full name
    r3 = cal.last_months(3)
    ab = acct("Akram Baloch")
    v = S(r3, buyer=ab)
    add("Y09", w, "roman", "akram baloch ne pichle 3 mahine mein kitna maal liya aur kitni commission di ?",
        [R({"purchase": v["bval"], "comm": v["buyer_comm"]}), R({"purchase": v["bval"], "comm": v["bc"]})],
        names=["Akram Baloch"], group="one-buyer", kind="sale+commission", period=per(r3), source="view C")

    # Y10 two Akrams: the honest answer is a question
    add("Y10", w, "roman", "akram ne is mahine kitni commission di ?", outcomes=["which_one", "answered"],
        names_any=["Akram Toll Tax", "Akram Baloch"], ambiguous=True, group="ambiguous-name",
        kind="which-one", period=per(month), source="find",
        note="two buyers are called Akram: asking 'which one' is right; a figure for one of them without asking is a wrong guess",
        readings=[R({"toll_tax": S(month, buyer=acct("Akram Toll Tax"))["buyer_comm"],
                     "baloch": S(month, buyer=ab)["buyer_comm"]})])

    # Y11 a buyer month by month this year
    faisal = acct("Faisal")
    ms = b.by_month(b.lines(ytd, buyer=faisal))
    add("Y11", w, "roman", "faisal gahaq ne is saal mahine ke hisab se kitna maal liya ?",
        [R({f"{y}-{m:02d}": v["bval"] for (y, m), v in ms.items()}, {"year_total": sum(v["bval"] for v in ms.values())})],
        names=["Faisal"], group="month-by-month",
        kind="buyer-purchase", period=per(ytd), source="view C by month")

    # Y12 two months sale
    a, c = S(last_month)["gross"], S(month)["gross"]
    add("Y12", w, "roman", "pichle mahine aur is mahine ki bikri ka farq batao",
        [R({"last": a, "this": c, "diff": a - c}), R({"diff": a - c})], group="compare-periods", kind="sale",
        period=f"{per(last_month)} / {per(month)}", source="view A compare", note="direction: less",
        direction="down")

    # Y13 trip result
    t1, t2 = b.trip(1), b.trip(2)
    add("Y13", w, "roman", "pehle safar ka nateeja kya raha, kitna nafa ya nuksan hua ?",
        [R({"result": t1["result"]})], group="trip", kind="trip-result", period="up to now",
        source="R33 trips", note=f"money back {t1['money_in']} less paid {t1['paid']}; closed")

    # Y14 trip money and each partner
    half = t1["result"] // 2
    add("Y14", w, "roman",
        "pehle safar mein kitna paisa laga, kitna wapas aya aur har partner ka hissa kitna raha ?",
        [R({"paid": t1["paid"], "money_in": t1["money_in"], "result": t1["result"]},
           {"each_partner": ["~", half, 3]})], group="trip", kind="trip-partners", period="up to now",
        source="R33 trips + partner statement", note="two partners at 50 / 50: each ~half the result (a rupee may move)")

    # Y15 profit since last year
    add("Y15", w, "roman", "pichle saal se is mahine tak shop ka nafa nuksan kitna raha ?",
        [R({"profit": b.profit(start)}, {"income": b.income(start), "expense": b.shop_expense(start)})],
        group="profit", kind="profit", period=per(start), source="R30",
        note=f"income less shop expenses; commission-only reading would be {S()['comm'] - b.shop_expense(start)}")

    # Y16 last month profit and the partners
    pp = b.partner_profit(last_month)
    add("Y16", w, "roman", "pichle mahine ka nafa batao aur har partner ka hissa kitna bana ?",
        [R({"profit": b.profit(last_month)}, {k: v["profit"] for k, v in pp.items()})],
        group="profit", kind="profit-partners", period=per(last_month), source="R30 / R31",
        note="no partner code is set in the year books")

    # Y17 partners' personal expenses this year
    pt = b.personal_taken(ytd)
    net = {r["name_en"].replace("Drawings - ", ""): r["amount"] for r in b.journal_by_account("0603%", ytd)}
    add("Y17", w, "roman", "is saal partnaron ne zaati kharche mein kitna kitna paisa liya ?",
        [R({k.lower(): v for k, v in pt.items()}), R({k.lower(): v for k, v in net.items()})],
        group="partner-kharch", kind="partner-kharch", period=per(ytd), source="R29 partner part / view M",
        note="taken (the kharch he spent), or net of what he returned")

    # Y18 expenses two months
    a, c = b.shop_expense(last_month), b.shop_expense(month)
    add("Y18", w, "roman", "is mahine aur pichle mahine ka kharcha compare karo",
        [R({"last": a, "this": c, "diff": a - c}), R({"last": a, "this": c}), R({"this": c, "diff": a - c})],
        group="compare-periods", kind="expense",
        period=f"{per(last_month)} / {per(month)}", source="R29 / view M compare", note="direction: less",
        direction="down")

    # Y19 an estimate
    full = [O.month_range(2026, m) for m in (8, 9, 10)]
    pace = O.avg_round(sum(S(r)["comm"] for r in full), 3)
    add("Y19", w, "roman", "agar commission isi raftar se chalta raha to agle 3 mahine mein kitna hoga ?",
        [R({"per_month": pace, "three_months": pace * 3})], outcomes=["estimate", "answered"],
        group="estimate", kind="estimate", period="Nov 2026 to Jan 2027", source="estimator, same pace",
        note="average of August, September, October commission, rounded once, times 3")

    # Y20 a seller, an item, a range
    bn = b.item("Banana")
    r6 = cal.last_months(6)
    v = S(r6, seller=rahim, item=bn)
    add("Y20", w, "roman", "rahim ne kele ki bikri pichle 6 mahine mein kitni ki ?",
        [R({"sale": v["gross"]})], names=["Rahim"], group="seller-item", kind="sale", period=per(r6),
        source="view B/A with item", note=f"{v['n']} lines")

    # Y21 chain of three
    r_, k_ = S(ytd, seller=rahim), S(ytd, seller=karim)
    add("Y21a", w, "roman", "rahim ki is saal ki bikri kitni hai ?", [R({"rahim": r_["gross"]})], names=["Rahim"],
        group="follow-up", kind="sale", period=per(ytd), source="view B")
    add("Y21b", w, "roman", "aur karim ki ?", [R({"karim": k_["gross"]})], names=["Karim"], follow_of="Y21a",
        group="follow-up", kind="sale", period=per(ytd), source="view B")
    add("Y21c", w, "roman", "dono ki commission mila kar kitni hai ?",
        [R({"sum": r_["comm"] + k_["comm"]}), R({"sum": r_["sc"] + k_["sc"]})], follow_of="Y21b",
        group="follow-up", kind="calc-sum", period=per(ytd), source="calculator")

    # Y22 English, months
    req = {}
    for m in range(5, 11):
        req[f"2026-{m:02d}"] = S(O.month_range(2026, m))["comm"]
    add("Y22", w, "english", "How much commission did the shop earn in each month from May to October 2026?",
        [R(req, {"six_month_total": sum(req.values())})], group="month-by-month", kind="commission", period="2026-05 to 2026-10", source="view A by month")

    # Y23 Urdu letters, heads last month
    h = heads(last_month)
    add("Y23", w, "urdu", "پچھلے مہینے دکان کا خرچہ مدوں کے حساب سے کتنا ہوا؟",
        [R({k.lower(): v for k, v in h.items()}, {"total": b.shop_expense(last_month)})], group="expenses",
        kind="expense-by-head", period=per(last_month), source="R29 by head")

    # Y24 Urdu letters, one seller
    v = S(ytd, seller=rahim)
    add("Y24", w, "urdu", "رحیم نے اس سال دکان کو کتنا کمیشن دیا؟",
        [R({"comm": v["sc"]}), R({"comm": v["comm"]})], names=["Rahim"], group="one-seller",
        kind="commission", period=per(ytd), source="view B / commission page")

    # Y25 a range inside a month
    rg = (D(2026, 10, 10), D(2026, 10, 20))
    v = S(rg)
    add("Y25", w, "roman", "10 october se 20 october ke beech shop ki bikri aur commission kitni rahi ?",
        [R({"sale": v["gross"], "commission": v["comm"]})], group="range", kind="sale+commission",
        period=per(rg), source="view A")

    # Y26 the best month
    ms = b.by_month(b.lines(start))
    full_months = {k: v for k, v in ms.items() if k != (2026, 11)}
    best = max(full_months.items(), key=lambda kv: kv[1]["comm"])
    add("Y26", w, "roman", "pichle saal se is mahine tak kis mahine mein sab se zyada commission aaya ?",
        [R({"best_month_commission": best[1]["comm"]})], group="best-month", kind="top-month",
        period=per(start), source="view A by month", note=f"{best[0][0]}-{best[0][1]:02d}")

    # Y27 the open trip so far
    add("Y27", w, "roman", "doosre safar mein abhi tak kitna maal khareeda aur kitna bik chuka ?",
        [R({"paid": t2["paid"], "bought_qty": t2["bought_qty"], "sold_qty": t2["sold_qty"]},
           {"money_in": t2["money_in"]})], group="trip", kind="trip-so-far", period="up to now",
        source="R33 trips", note=f"open trip: money back so far {t2['money_in']}")

    # Y28 commission from sellers and from buyers
    v = S(last_month)
    add("Y28", w, "roman", "pichle mahine commission bikri walon se kitni li aur khareedaron se kitni ?",
        [R({"from_sellers": v["sc"], "from_buyers": v["bc"] + v["laga"]}, {"total": v["comm"]}),
         R({"from_sellers": v["sc"], "from_buyers": v["bc"]}, {"laga": v["laga"]})],
        group="commission-sides", kind="commission", period=per(last_month), source="view A (seller / buyer)")

    # Y29 three figures this year
    v = S(ytd)
    add("Y29", w, "roman", "is saal shop ki commission, kharcha aur nafa teeno batao",
        [R({"commission": v["comm"], "expense": b.shop_expense(ytd), "profit": b.profit(ytd)})],
        group="sale+commission+expense", kind="three-figures", period=per(ytd), source="R29/R30/view A")

    # Y30 English, a seller and his quantity
    v = S(last_month, seller=karim)
    add("Y30", w, "english", "How much did Karim sell last month and how many boxes was that?",
        [R({"sale": v["gross"], "boxes": v["qty"]})], names=["Karim"], group="one-seller", kind="sale+qty",
        period=per(last_month), source="view B")

    # Y31 English chain on expenses
    el, tea = (sum(r["amount"] for r in b.journal_by_account(c, ytd)) for c in ("05010002", "05010004"))
    add("Y31a", w, "english", "How much did the shop spend on electricity this year?", [R({"electricity": el})],
        group="follow-up", kind="expense-head", period=per(ytd), source="R29 head")
    add("Y31b", w, "english", "and on tea?", [R({"tea": tea})], follow_of="Y31a", group="follow-up",
        kind="expense-head", period=per(ytd), source="R29 head")
    add("Y31c", w, "english", "what is the total of both?", [R({"sum": el + tea})], follow_of="Y31b",
        group="follow-up", kind="calc-sum", period=per(ytd), source="calculator")

    # Y32 garden
    g = b.garden(1)
    add("Y32", w, "roman", "shahpur bagh ka hisab batao: kitna paisa lagaya, kitni bikri aayi aur ab balance kya hai ?",
        [R({"paid": g["paid"], "balance": g["balance"]}, {"money_in": g["money_in"], "rent": g["rent"],
                                                      "rent_due": g["rent_due"]}),
         R({"installments": g["installments"], "costs": g["costs"], "balance": g["balance"]},
           {"money_in": g["money_in"], "rent": g["rent"]})],
        group="garden", kind="garden-balance", period="up to now", source="R36 garden",
        note=f"installments {g['installments']} + costs {g['costs']}; money in {g['money_in']} (after commission)")

    # Y33 top 5 buyers by commission this year
    by = b.by_party(b.lines(ytd), "buyer")
    rank = sorted(((k, v) for k, v in by.items() if nm(k) not in ("Chukta Khata",)),
                  key=lambda kv: -kv[1]["buyer_comm"])[:5]
    add("Y33", w, "roman", "is saal sab se zyada commission dene wale 5 gahaq kaun hain aur kitni kitni ?",
        [R({nm(k): v["buyer_comm"] for k, v in rank}),
         R({nm(k): v["bc"] for k, v in sorted(by.items(), key=lambda kv: -kv[1]["bc"])[:5] if nm(k) != "Chukta Khata"})],
        names=[nm(k) for k, _ in rank], group="ranked-buyers",
        kind="top-5", period=per(ytd), source="view C ranked",
        note="order: " + " > ".join(f"{nm(k)} {v['buyer_comm']}" for k, v in rank))

    # Y34 a seller, this year and last year
    y25 = cal.year(1)
    add("Y34", w, "roman", "rahim ki bikri is saal pichle saal ke muqable mein kitni zyada ya kam hai ?",
        [R({"this_year": S(ytd, seller=rahim)["gross"], "last_year": S(y25, seller=rahim)["gross"]}),
         R({"diff": S(ytd, seller=rahim)["gross"] - S(y25, seller=rahim)["gross"]})],
        names=["Rahim"], group="compare-periods", kind="sale", period=f"{per(ytd)} / {per(y25)}", direction="up",
        source="view B compare", note="the books start on 06-11-2025: last year is only 6 Nov to 31 Dec")

    # Y35 a buyer over three months
    v = S(r3, buyer=faisal)
    add("Y35", w, "roman", "faisal gahaq ne pichle 3 mahine mein kitni commission di aur kitna maal liya ?",
        [R({"purchase": v["bval"], "comm": v["buyer_comm"]}), R({"purchase": v["bval"], "comm": v["bc"]})],
        names=["Faisal"], group="one-buyer", kind="sale+commission", period=per(r3), source="view C")
    # ---- measure words on the rich year ---------------------------------------------------------
    r_all = S(seller=rahim)
    r_net = b.net(rahim)
    add("M17", w, "roman", "rahim ki safi bikri abhi tak kitni hai ?", [R({"net_sale": r_net})], names=["Rahim"],
        group="measure-words", kind="net-sale", period="up to now", source="R6 / lot net",
        wrong_columns={"gross": r_all["gross"], "buyer_total": r_all["btot"]})
    o = S(last_month)
    add("M18", w, "roman", "pichle mahine ki khaam bikri aur gahakon ke kul bill alag alag batao",
        [R({"gross_sale": o["gross"], "buyer_bills": o["btot"]})], group="measure-words", kind="gross-and-bills",
        period=per(last_month), source="view A / R4",
        wrong_columns={"all_commission": o["comm"], "buyer_value_not_gross": o["bval"]}
        if o["bval"] != o["gross"] else {"all_commission": o["comm"]},
        note=f"last month the rate difference makes buyer value {o['bval']} against gross {o['gross']}")
    y_ = S(ytd)
    add("M19", w, "urdu", "اس سال کمیشن اور لاگا الگ الگ بتاؤ",
        [R({"commission_without_laga": y_["sc"] + y_["bc"], "laga": y_["laga"]}, {"together": y_["comm"]})],
        group="measure-words", kind="commission-and-laga", period=per(ytd), source="view A columns",
        wrong_columns={"gross": y_["gross"]})
    add("M20", w, "english", "What is the total of all buyer bills last month, with commission and laga included?",
        [R({"buyer_bills": o["btot"]})], group="measure-words", kind="buyer-total", period=per(last_month),
        source="R4", wrong_columns={"gross": o["gross"], "all_commission": o["comm"]})
    return b


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dev", required=True)
    ap.add_argument("--year", required=True)
    ap.add_argument("--out", default=os.path.dirname(os.path.abspath(__file__)))
    args = ap.parse_args()
    dev_b = build_dev(args.dev)
    year_b = build_year(args.year)
    # The role each named party really has in the books (a buyer is searched as a buyer, a seller
    # as a seller): the scorer fails an answer that searched him in the other role.
    for q in QUESTIONS:
        books = dev_b if q["world"] == "dev" else year_b
        roles = {}
        for name in q["names"] + q["names_any"] + q["search_names"]:
            rows = books.c.execute("select distinct party_type from accounts where lower(name_en)=lower(?) "
                                   "and party_type in ('seller','buyer')", (name,)).fetchall()
            if len(rows) == 1:
                roles[name] = rows[0][0]
        q["roles"] = roles
    ids = [q["id"] for q in QUESTIONS]
    assert len(ids) == len(set(ids)), "duplicate ids"
    with open(os.path.join(args.out, "questions.json"), "w", encoding="utf-8") as f:
        json.dump(QUESTIONS, f, ensure_ascii=False, indent=1)
        f.write("\n")
    lines = ["# Complex questions with expected figures (plan 82, lane C)", "",
             "Worked out by `oracle.py` (SQL over posted rows). Dev = copy of the dev database, today 20-11-2026. "
             "Year = the rich year of the robustness oracle, today 05-11-2026. Whole rupees.", ""]
    for world in ("dev", "year"):
        lines += [f"## World: {world}", "", "| id | lang | question | period | expected (reading 1) | other readings |", "|---|---|---|---|---|---|"]
        for q in QUESTIONS:
            if q["world"] != world:
                continue
            def fmt(r):
                return "; ".join(f"{k}={'/'.join(map(str, v))}" for k, v in r["req"].items())
            rs = q["readings"]
            exp = fmt(rs[0]) if rs else ("0 (honest zero)" if q["zero"] else "-")
            other = " || ".join(fmt(r) for r in rs[1:])
            if q["ambiguous"]:
                exp = "ask 'which one' (" + ", ".join(q["names_any"]) + ")"
            if q["follow_of"]:
                exp = f"(follow-up of {q['follow_of']}) " + exp
            lines.append(f"| {q['id']} | {q['lang']} | {q['question']} | {q['period']} | {exp} | {other} |")
        lines.append("")
    with open(os.path.join(args.out, "QUESTIONS.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(len(QUESTIONS), "questions")


if __name__ == "__main__":
    main()
