#!/usr/bin/env python3
"""Recalculates the PRD Section 4 worked day and the money end-to-end tests.

Use it whenever a rate or default changes, so no number in the documents is
worked out by hand. The defaults reproduce PRD v1.2, including E2E-28 to E2E-33. Change a rate with a flag:

    python3 worked_day.py                    # current PRD numbers
    python3 worked_day.py --buyer-pct 5      # what changes if buyer commission is 5%
    python3 worked_day.py --json             # machine-readable output

All money is whole rupees. Rounding is half up to the nearest rupee, once,
exactly like core::money::round_to_rupee. Splits round each share down and give
the leftover to the largest share (tie: the partner listed first), like
split_by_bps.
"""
import argparse
import json


def pct(amount, percent):
    """round_to_rupee(amount x percent / 100), half up. percent may have 2 decimals."""
    bps = round(percent * 100)
    return (amount * bps + 5000) // 10000


def split(total, shares_bps):
    """split_by_bps: floor each share, leftover to the largest share (tie: first)."""
    sign = -1 if total < 0 else 1
    base = abs(total)
    parts = [base * s // 10000 for s in shares_bps]
    leftover = base - sum(parts)
    largest = max(range(len(shares_bps)), key=lambda i: (shares_bps[i], -i))
    parts[largest] += leftover
    return [sign * p for p in parts]


def lot(lines, a, freight, naqd, labour_per_nag, munshiana, phone, received):
    """One lot, first posting. lines = [(buyer, qty, rate)]. Returns a dict of results."""
    out = {"lines": []}
    for buyer, qty, rate in lines:
        value = qty * rate
        line = {
            "buyer": buyer, "qty": qty, "rate": rate, "value": value,
            "seller_comm": pct(value, a.seller_pct),
            "buyer_comm": pct(value, a.buyer_pct),
            "laga": qty * a.laga_per_nag,
        }
        line["buyer_pays"] = value + line["buyer_comm"] + line["laga"]
        out["lines"].append(line)
    s = lambda k: sum(l[k] for l in out["lines"])
    labour = received * labour_per_nag
    out.update(
        gross=s("value"), seller_comm=s("seller_comm"), buyer_comm=s("buyer_comm"),
        laga=s("laga"), buyer_pays=s("buyer_pays"), freight=freight, labour=labour,
        munshiana=munshiana, phone=phone, naqd=naqd,
    )
    out["deductions"] = freight + out["seller_comm"] + labour + munshiana + naqd + phone
    out["net"] = out["gross"] - out["deductions"]
    out["seller_credit"] = out["gross"] - out["seller_comm"] - freight - labour - munshiana - phone
    out["shared_income"] = out["seller_comm"] + out["buyer_comm"] + out["laga"] + phone
    out["split"] = split(out["shared_income"], a.shares)
    return out


def run(a):
    r = {}
    # Section 4.3 opening position
    cash = 65000 - 15000
    rahim_advance = 15000

    # Section 4.4: بکری 1 (all lines ادھاری)
    b1 = lot([("Akram", 20, 600), ("Arshad", 30, 580)], a, freight=3000, naqd=5000,
             labour_per_nag=a.labour_per_nag, munshiana=a.munshiana, phone=a.phone, received=50)
    b1["journal_sale_part"] = b1["buyer_pays"]
    b1["journal_whole"] = b1["buyer_pays"] + b1["naqd"]
    r["bikri1"] = b1

    # Section 4.5: بکری 2 (one-time buyer, نقدی)
    b2 = lot([("Shahid (pukhta)", 10, 1200)], a, freight=1000, naqd=0,
             labour_per_nag=a.labour_per_nag, munshiana=a.munshiana, phone=a.phone, received=10)
    b2["journal_whole"] = 2 * b2["buyer_pays"]  # sale part + cash part
    r["bikri2"] = b2

    akram = b1["lines"][0]["buyer_pays"]
    arshad = b1["lines"][1]["buyer_pays"]

    # 4.6 receipt, 4.7 payment, 4.8 expenses
    receipt = 10000
    rahim_payable = b1["seller_credit"] - rahim_advance - b1["naqd"]
    shop_expense, drawing = 3000, 2000
    cash = cash - b1["naqd"] + b2["buyer_pays"] + receipt - rahim_payable - shop_expense - drawing

    income = [x + y for x, y in zip(b1["split"], b2["split"])]
    expense = split(shop_expense, a.shares)
    shared = b1["shared_income"] + b2["shared_income"]
    commission = (b1["seller_comm"] + b1["buyer_comm"] + b1["laga"]) + (b2["seller_comm"] + b2["buyer_comm"] + b2["laga"])
    r["end_of_day"] = {
        "cash": cash,
        "akram_due": akram - receipt,
        "arshad_due": arshad,
        "buyers_dues": akram - receipt + arshad,
        "rahim_payable_paid": rahim_payable,
        "karim_payable": b2["seller_credit"],
        "todays_commission": commission,
        "shared_income": shared,
        "partner_income": income,
        "partner_expense": [-e for e in expense],
        "partner_profit": [i - e for i, e in zip(income, expense)],
        "firm_profit": shared - shop_expense,
        "todays_gross_sale": b1["gross"] + b2["gross"],
        "shop_expense_split_e2e_20_3384": [-x for x in split(3384, a.shares)],
    }

    seller_comm_inc = b1["seller_comm"] + b2["seller_comm"]
    buyer_comm_inc = b1["buyer_comm"] + b2["buyer_comm"]
    laga_inc = b1["laga"] + b2["laga"]
    dr = cash + (akram - receipt) + arshad + shop_expense + drawing
    cr = (b2["seller_credit"] + seller_comm_inc + buyer_comm_inc + laga_inc + (b1["phone"] + b2["phone"])
          + (b1["labour"] + b2["labour"]) + (b1["freight"] + b2["freight"])
          + (b1["munshiana"] + b2["munshiana"]) + 65000)
    r["trial_balance"] = {"debit": dr, "credit": cr, "balanced": dr == cr}
    r["trial_balance_rows"] = {
        "Dr cash": cash, "Dr Akram": akram - receipt, "Dr Arshad": arshad,
        "Dr shop expense (electricity)": shop_expense, "Dr partner A drawings": drawing,
        "Cr Rahim": 0, "Cr Karim": b2["seller_credit"], "Cr pukhta khata (buyer)": 0, "Cr opening difference": 0,
        "Cr seller commission income": seller_comm_inc, "Cr buyer commission income": buyer_comm_inc,
        "Cr laga income": laga_inc, "Cr phone income": b1["phone"] + b2["phone"],
        "Cr labour payable": b1["labour"] + b2["labour"], "Cr freight payable": b1["freight"] + b2["freight"],
        "Cr munshiana payable": b1["munshiana"] + b2["munshiana"], "Cr opening capital": 65000,
    }

    total = commission
    pct2 = lambda part: (part * 10000 + total // 2) // total
    banana = b1["seller_comm"] + b1["buyer_comm"] + b1["laga"]
    apple = b2["seller_comm"] + b2["buyer_comm"] + b2["laga"]
    r["analytics_view_a"] = {
        "banana_total_comm": banana, "apple_total_comm": apple,
        "banana_pct": pct2(banana) / 100, "apple_pct": pct2(apple) / 100,
        "banana_avg_rate": (b1["gross"] + 50 // 2) // 50,  # avg_rate = round_to_rupee(gross / qty), qty 50
        "apple_avg_rate": (b2["gross"] + 10 // 2) // 10,
    }

    # E2E-13: edit Akram's rate 600 -> 610 (lot charges unchanged)
    e13 = lot([("Akram", 20, 610), ("Arshad", 30, 580)], a, 3000, 5000, a.labour_per_nag, a.munshiana, a.phone, 50)
    r["e2e_13"] = {k: e13[k] for k in ("gross", "seller_comm", "deductions", "net", "shared_income", "split")}
    r["e2e_13"]["akram_pays"] = e13["lines"][0]["buyer_pays"]

    # E2E-13b: default buyer commission changed before the same edit; only the changed line uses it
    changed = 12200
    new_bc = pct(changed, a.e13b_buyer_pct)
    unchanged_bc = b1["lines"][1]["buyer_comm"]
    sh = e13["seller_comm"] + new_bc + unchanged_bc + b1["laga"] + a.phone
    r["e2e_13b"] = {"akram_buyer_comm": new_bc, "akram_pays": changed + new_bc + 20 * a.laga_per_nag,
                    "shared_income": sh, "split": split(sh, a.shares)}

    # E2E-16 and 16b: 45 of 50 sold, then 5 more later at 560 (no lot charges again)
    e16 = lot([("Akram", 20, 600), ("Arshad", 25, 580)], a, 3000, 5000, a.labour_per_nag, a.munshiana, a.phone, 50)
    r["e2e_16"] = {k: e16[k] for k in ("gross", "seller_comm", "buyer_comm", "laga", "deductions", "net", "shared_income", "split")}
    v = 5 * 560
    sc, bc, lg = pct(v, a.seller_pct), pct(v, a.buyer_pct), 5 * a.laga_per_nag
    r["e2e_16b"] = {"value": v, "seller_comm": sc, "buyer_comm": bc, "laga": lg, "arshad_pays": v + bc + lg,
                    "seller_credit": v - sc, "shared_income": sc + bc + lg, "split": split(sc + bc + lg, a.shares),
                    "lot_gross_total": e16["gross"] + v,
                    "lot_deductions_total": e16["deductions"] + sc,
                    "lot_net_total": e16["gross"] + v - (e16["deductions"] + sc)}

    eod = r["end_of_day"]
    profit = eod["partner_profit"]

    # E2E-28 (PRD v1.2, FR-15.8): بکری 1 saved with "پیشگی سے کاٹیں" unticked.
    # The advance stays open. Payable = seller credit - نقد. Ledger = advance + نقد - credit.
    payable_28 = b1["seller_credit"] - b1["naqd"]
    ledger_28 = rahim_advance + b1["naqd"] - b1["seller_credit"]  # negative = جمع
    r["e2e_28"] = {"rahim_payable": payable_28, "advance_open": rahim_advance,
                   "rahim_ledger_cr": -ledger_28,
                   "rahim_after_paying_payable_dr": ledger_28 + payable_28}

    # E2E-29 (FR-16.3): partner A takes 2,000 on 10-10 (Event 5), returns 2,000 on 11-10.
    taken = drawing
    r["e2e_29"] = {"a_profit_share": profit[0], "a_net_10_10": profit[0] - taken,
                   "a_net_taken_after_return": taken - 2000, "a_net_after_return": profit[0],
                   "cash_11_10": cash + 2000}

    # E2E-30 (FR-16.5): staff Aslam is lent 5,000 on 10-10, returns 3,000 cash on 11-10,
    # and 2,000 is deducted from his salary on 12-10 (posts shop expense تنخواہ, split).
    lent, ret, sal = 5000, 3000, 2000
    r["e2e_30"] = {"cash_10_10": cash - lent, "open_11_10": lent - ret, "cash_11_10": cash - lent + ret,
                   "open_12_10": lent - ret - sal, "cash_12_10": cash - lent + ret,
                   "salary_split": [-x for x in split(sal, a.shares)]}

    # E2E-31 (FR-15.8): separate run. Rahim returns 10,000 of his advance in cash on 10-10,
    # before بکری 1. Then بکری 1 adjusts only the 5,000 still open.
    adv_ret = 10000
    open_31 = rahim_advance - adv_ret
    payable_31 = b1["seller_credit"] - b1["naqd"] - open_31
    r["e2e_31"] = {"advance_open_after_return": open_31, "cash_after_return": 65000 - 15000 + adv_ret,
                   "advance_adjusted_by_bikri1": open_31, "rahim_payable": payable_31,
                   "rahim_ledger_cr": b1["seller_credit"] - (rahim_advance - adv_ret) - b1["naqd"]}

    # E2E-32 (FR-15.11): after Events 1 to 5, Akram borrows 3,000 cash (ادائیگی to a buyer).
    loan = 3000
    r["e2e_32"] = {"akram_due": eod["akram_due"] + loan, "cash": cash - loan,
                   "shop_total_ograhi": eod["buyers_dues"] + loan}

    # E2E-33 (FR-19.8): Akram's ledger with items, after Events 1 to 5.
    l0 = b1["lines"][0]
    r["e2e_33"] = {"line_value": l0["value"], "line_comm": l0["buyer_comm"], "line_laga": l0["laga"],
                   "line_total_dr": l0["buyer_pays"], "receipt_cr": receipt,
                   "closing_dr": l0["buyer_pays"] - receipt, "r20_total": eod["buyers_dues"]}
    return r


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--seller-pct", type=float, default=6)
    p.add_argument("--buyer-pct", type=float, default=6)
    p.add_argument("--laga-per-nag", type=int, default=8)
    p.add_argument("--labour-per-nag", type=int, default=5, help="typed on each lot in the worked day")
    p.add_argument("--munshiana", type=int, default=50, help="typed on each lot in the worked day")
    p.add_argument("--phone", type=int, default=20, help="typed on each lot in the worked day")
    p.add_argument("--e13b-buyer-pct", type=float, default=7)
    p.add_argument("--shares", type=int, nargs="+", default=[4000, 3000, 3000], help="partner shares in bps")
    p.add_argument("--json", action="store_true")
    a = p.parse_args()
    if sum(a.shares) != 10000:
        raise SystemExit("Partner shares must total 10000 bps (100%).")
    r = run(a)
    if a.json:
        print(json.dumps(r, ensure_ascii=False, indent=2))
        return
    for section, values in r.items():
        print(f"\n== {section}")
        for k, v in values.items():
            if k == "lines":
                for line in v:
                    print(f"   line {line['buyer']}: value {line['value']:,}, seller comm {line['seller_comm']:,}, "
                          f"buyer comm {line['buyer_comm']:,}, laga {line['laga']:,}, pays {line['buyer_pays']:,}")
            elif isinstance(v, list):
                print(f"   {k}: " + " / ".join(f"{x:,}" for x in v))
            elif isinstance(v, (int,)) and not isinstance(v, bool):
                print(f"   {k}: {v:,}")
            else:
                print(f"   {k}: {v}")
    if not r["trial_balance"]["balanced"]:
        raise SystemExit("\nTRIAL BALANCE DOES NOT BALANCE. Stop and find the error.")


if __name__ == "__main__":
    main()
