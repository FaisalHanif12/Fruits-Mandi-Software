#!/usr/bin/env python3
"""The scorer on made-up replies (no model, no database): a perfect answer is RIGHT, a missing figure
is PARTIAL, a wrong figure is WRONG, a zero question with a figure is WRONG, an ambiguous name that
is not asked about is WRONG, and a hidden figure that shows is a LEAK."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import score  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
QS = json.load(open(os.path.join(HERE, "questions.json"), encoding="utf-8"))


def money(v):
    return {"type": "figure", "cell": {"type": "money", "paisa": int(round(v * 100))}, "dir": None, "checked": True}


def pct(v):
    return {"type": "figure", "cell": {"type": "percent", "hundredths": int(round(v * 100))}, "dir": None, "checked": False}


def reply(q, figures, names=(), outcome="answered"):
    parts = []
    for n in names:
        parts.append({"type": "name", "name": {"name_en": n, "name_ur": "x", "tag": None, "code": None}})
    for f in figures:
        if isinstance(f, list):
            f = f[1]
        parts.append(pct(f) if isinstance(f, float) and not float(f).is_integer() else money(f))
    return {"id": q["id"], "outcome": outcome, "millis": 1000, "model_calls": 0, "finish_reason": "local",
            "reply": {"outcome": outcome, "answer": {"parts": parts, "tables": []}}}


bad = 0
for q in QS:
    if q["ambiguous"]:
        row = reply(q, [], [])
        row["outcome"] = row["reply"]["outcome"] = "which_one"
        row["reply"]["options"] = [{"name": {"name_en": n, "name_ur": "x"}} for n in q["names_any"]]
        v = score.score_one(q, row)["verdict"]
        assert v == "ASKED-OK", (q["id"], v)
        row = reply(q, [100], q["names_any"])
        assert score.score_one(q, row)["verdict"] == "WRONG", q["id"]
        continue
    if q["zero"]:
        assert score.score_one(q, reply(q, [], q["names"]))["verdict"] == "ZERO-OK", q["id"]
        assert score.score_one(q, reply(q, [12345], q["names"]))["verdict"] == "WRONG", q["id"]
        continue
    r0 = q["readings"][0]
    figs = [a[0] for a in r0["req"].values()]
    names = q["names"] or [n for n in q["names_any"][:1]]
    out = q["outcomes"][0]
    v = score.score_one(q, reply(q, figs, names, out))
    assert v["verdict"] == "RIGHT", (q["id"], v)
    if len(figs) > 1:
        v = score.score_one(q, reply(q, figs[:-1], names, out))
        assert v["verdict"] in ("PARTIAL", "RIGHT"), (q["id"], v)   # RIGHT only if another reading is met
    v = score.score_one(q, reply(q, [987654321], names, out))
    assert v["verdict"] in ("WRONG",), (q["id"], v)
    for r in q["readings"][1:]:
        figs2 = [a[0] for a in r["req"].values()]
        v = score.score_one(q, reply(q, figs2, names, out))
        assert v["verdict"] == "RIGHT", (q["id"], "reading 2", v)
d11 = next(q for q in QS if q["id"] == "D11")
assert score.score_one(d11, reply(d11, [200000, 100000]))["verdict"] == "LEAK"
assert score.score_one(d11, reply(d11, [200000]))["verdict"] == "RIGHT"
print("selftest ok:", len(QS), "questions")

# A figure of another column of the same books fails the answer, even beside the right figure.
m01 = next(q for q in QS if q["id"] == "M01")
assert m01["wrong_columns"], "M01 lists its wrong columns"
right = m01["readings"][0]["req"]["net_sale"][0]
gross = m01["wrong_columns"]["gross"]
assert score.score_one(m01, reply(m01, [right]))["verdict"] == "RIGHT"
v = score.score_one(m01, reply(m01, [gross]))
assert v["verdict"] == "WRONG-COLUMN" and "not shown" in v["note"], v
v = score.score_one(m01, reply(m01, [right, gross]))
assert v["verdict"] == "WRONG-COLUMN" and "also shown" in v["note"], v
print("selftest wrong-column ok")

# A buyer searched as a seller and not found is WRONG-ROLE; the same retried and answered keeps RIGHT.
m22 = next(q for q in QS if q["id"] == "M22")
assert m22["roles"].get("Haroon") == "buyer", m22["roles"]
row = reply(m22, [], [], "not_found")
row["tool_calls"] = [{"tool": "find", "input": {"role": "seller", "text": "Haroon"}}]
r = score.score_one(m22, row)
errs = score.role_errors(m22, row)
assert errs and "searched as seller" in errs[0], errs
print("selftest role ok")
