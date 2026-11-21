#!/usr/bin/env python3
"""Scores the live (or offline) rows of the complex-question list against `questions.json`.

    python3 -I scripts/ai-eval/complex/score.py ROWS.jsonl [ROWS2.jsonl ...] [--report report.md] [--json out.json]

The expected figures are the oracle's (`oracle.py`), never the assistant's. For each question:

  RIGHT      every `req` figure of one honest reading is in the answer (parts or tables, with the
             right multiplicity), every required name is shown, and the outcome is an allowed one.
  ZERO-OK    a zero question answered with no positive figure.
  ASKED-OK   an ambiguous name answered with "which one" and the right names (never a silent guess).
  PARTIAL    some `req` figures of the best reading are shown, not all (the missing ones are listed).
  WRONG      the answer shows figures but not the required ones: a wrong figure, party or period.
  ASKED      the app asked or said "not found / need dates / not understood" where a figure was due.
  LOCKED-OK  a partner question that stopped at the partner code, with no partner figure shown.
  FAIL       an error, offline, time_limit, checks_failed, records_differ or no reply.
  NEEDS-MODEL (offline rows only) the app could not answer alone.
  LEAK       a figure that must stay behind the partner code is shown.
  WRONG-ROLE a named buyer was searched as a seller (or the other way round) and the question ended
             without his figure (`not_found` with the other role, or no right figure). A wrong role
             that was corrected in a later call keeps the verdict but is noted ("role retry").
  WRONG-DIRECTION the app's direction word on a difference (more / less) is not the true one.
  WRONG-COLUMN  the answer text shows a figure that is ANOTHER COLUMN of the same books for the same
             subject and period (the gross sale for a net sale, the buyer bill for a gross sale, the
             commission with laga for the commission without it, ...). `wrong_columns` of the question
             lists those figures. It fails even when the right figure is also shown: the note says so.

Figures are compared as whole rupees (a quantity or a count as itself, a percent as 5.45). A figure
may sit in the answer text (a `figure` part) or in a table the answer shows.
"""
import argparse
import collections
import json
import os
import statistics
import sys

FAIL_OUTCOMES = {"error", "offline", "time_limit", "checks_failed", "records_differ", "limit_reached"}
ASK_OUTCOMES = {"which_one", "need_dates", "need_party", "not_found", "not_understood", "outside_books"}


def collect(node, nums, names):
    """Walks the JSON of an answer. A cell with a `type` gives a number; a name gives its words."""
    if isinstance(node, dict):
        t = node.get("type")
        if t in ("money", "balance") and "paisa" in node:
            nums.append(node["paisa"] / 100)
        elif t == "number" and isinstance(node.get("value"), (int, float)):
            nums.append(node["value"])
        elif t == "percent" and "hundredths" in node:
            nums.append(node["hundredths"] / 100)
        for k, v in node.items():
            if k in ("name_en", "name_ur") and isinstance(v, str):
                names.add(v.strip().lower())
            elif k in ("sources", "how", "next", "read_at", "working"):
                continue
            else:
                collect(v, nums, names)
    elif isinstance(node, list):
        for x in node:
            collect(x, nums, names)


def extract(reply):
    """(figures of the answer text, figures of everything the answer shows, names shown)."""
    ans = (reply or {}).get("answer") or {}
    parts, everything, names = [], [], set()
    collect(ans.get("parts") or [], parts, names)
    collect(ans, everything, names)
    for o in (reply or {}).get("options") or []:
        collect(o, everything, names)
    return parts, everything, names


def has(figure, counter_list):
    """Is the figure in the list? `["~", c, tol]` is a centre and a tolerance."""
    if isinstance(figure, list) and figure and figure[0] == "~":
        c, tol = figure[1], figure[2]
        return any(abs(n - c) <= tol for n in counter_list)
    if isinstance(figure, float) and not float(figure).is_integer():
        return any(abs(n - figure) <= 0.011 for n in counter_list)
    return any(abs(n - figure) < 0.5 for n in counter_list)


def reading_missing(reading, nums):
    """The labels of a reading whose figure is not in `nums`, with multiplicity for equal exact
    figures (three months with the same amount need three of them)."""
    pool = collections.Counter(round(n, 2) for n in nums)
    missing = []
    for label, alts in reading["req"].items():
        found = False
        for a in alts:
            if isinstance(a, list):  # approximation
                if has(a, nums):
                    found = True
                    break
                continue
            key = round(float(a), 2)
            if pool.get(key, 0) > 0:
                pool[key] -= 1
                found = True
                break
            # The same figure may be shown once for several labels in a table's total: accept
            # the figure when it is shown at least once.
        if not found:
            # a second chance without multiplicity
            if any(isinstance(a, (int, float)) and has(a, nums) for a in alts):
                found = True
        if not found:
            missing.append(label)
    return missing


def expected_numbers(q):
    out = []
    for r in q["readings"]:
        for d in (r["req"], r["opt"]):
            for alts in d.values():
                for a in alts:
                    out.append(a if not isinstance(a, list) else a[1])
    return out


ROLE_WORDS = {"buyer": {"buyer", "chukta"}, "seller": {"seller"}}


def role_errors(q, row):
    """(wrong, right) searches: a `find` of a named party in the role he does not have, or a not_found
    that asked the other role. `any`, `any_party` and every other role are not errors."""
    roles = q.get("roles") or {}
    wrong = []
    for call in row.get("tool_calls") or []:
        if not isinstance(call, dict) or call.get("tool") != "find":
            continue
        inp = call.get("input") or {}
        role, text = (inp.get("role") or ""), (inp.get("text") or "").strip().lower()
        for name, true_role in roles.items():
            n = name.lower()
            if text and (text in n or n in text) and role in ("buyer", "seller") and role != true_role:
                wrong.append(f"{name}: searched as {role}, he is a {true_role}")
    reply = row.get("reply") or {}
    fa = reply.get("found_as") or {}
    if row.get("outcome") == "not_found" and fa and fa.get("role") in ("buyer", "seller") and \
            reply.get("asked_role") in ("buyer", "seller") and reply.get("asked_role") != fa.get("role"):
        nm = ((fa.get("name") or {}).get("name_en") or "?")
        wrong.append(f"{nm}: asked as {reply.get('asked_role')}, he is a {fa.get('role')}")
    return sorted(set(wrong))


def score_one(q, row):
    reply = row.get("reply") or {}
    outcome = row.get("outcome") or ("ask_error" if row.get("ask_error") else "none")
    parts, everything, names = extract(reply)
    local = row.get("finish_reason") == "local"
    info = {
        "id": q["id"], "question": q["question"], "outcome": outcome, "local": bool(local),
        "calls": row.get("model_calls") or 0, "secs": (row.get("millis") or 0) / 1000.0,
        "qid": row.get("question_id"), "finish": row.get("finish_reason"), "cost": row.get("cost") or 0.0,
        "missing": [], "extras": [], "note": "",
    }
    # names
    name_missing = [n for n in q["names"] if n.lower() not in names and not any(n.lower() in x for x in names)]
    if q["names_any"] and not any(n.lower() in x for n in q["names_any"] for x in names):
        name_missing.append("any of " + "/".join(q["names_any"]))

    if row.get("offline") and outcome == "offline":
        info["verdict"] = "NEEDS-MODEL"
        return info
    if outcome in FAIL_OUTCOMES or row.get("ask_error"):
        info["verdict"] = "FAIL"
        info["note"] = (reply.get("failure") or {}).get("reason", "") if isinstance(reply.get("failure"), dict) else ""
        return info

    # a figure that must stay hidden
    leak = [n for n in q["must_not_show"] if has(n, everything)]
    if leak:
        info["verdict"] = "LEAK"
        info["missing"] = [f"shown but must be hidden: {leak}"]
        return info
    if outcome == "locked":
        ok = "locked" in q["outcomes"]
        info["verdict"] = "LOCKED-OK" if ok else "ASKED"
        return info

    if q["ambiguous"]:
        if outcome == "which_one":
            opt_names = set()
            for o in reply.get("options") or []:
                n = o.get("name") or {}
                opt_names.add((n.get("name_en") or "").lower())
            need = [n for n in q["names_any"] if n.lower() not in opt_names]
            info["verdict"] = "ASKED-OK" if not need else "ASKED"
            info["missing"] = [f"option missing: {n}" for n in need]
            return info
        info["verdict"] = "WRONG"
        info["note"] = "answered a name that fits two accounts without asking which"
        return info

    if outcome not in q["outcomes"]:
        if outcome in ASK_OUTCOMES:
            info["verdict"] = "ASKED"
            info["note"] = f"outcome {outcome}, wanted {q['outcomes']}"
        else:
            info["verdict"] = "WRONG"
            info["note"] = f"outcome {outcome}, wanted {q['outcomes']}"
        return info

    if q["zero"]:
        positives = [n for n in parts if n > 0.5]
        info["verdict"] = "ZERO-OK" if not positives else "WRONG"
        if positives:
            info["extras"] = positives[:8]
            info["note"] = "a zero question shows a positive figure"
        return info

    # the direction word the app puts on a difference must be the true one (more or less)
    want = q.get("direction")
    if want:
        dirs = {p.get("dir") for p in (reply.get("answer") or {}).get("parts", []) if p.get("type") == "figure" and p.get("dir")}
        if dirs and dirs != {want}:
            info["verdict"] = "WRONG-DIRECTION"
            info["missing"] = [f"direction shown {sorted(dirs)}, true direction {want} (the first period named against the second)"]
            return info

    # a figure of another column of the same books, in the text of the answer
    wc = {label: v for label, v in (q.get("wrong_columns") or {}).items() if has(v, parts)}
    allowed = {a for r in q["readings"] for d in (r["req"], r["opt"]) for alts in d.values() for a in alts
               if not isinstance(a, list)}
    wc = {label: v for label, v in wc.items() if v not in allowed}
    if wc:
        miss = [m for r in q["readings"] for m in [reading_missing(r, everything)]]
        right_too = any(not m for m in miss)
        info["verdict"] = "WRONG-COLUMN"
        info["missing"] = [f"shows another column: {label}={v:g}" for label, v in wc.items()]
        info["note"] = "the asked figure is also shown" if right_too else "the asked figure is not shown"
        info["extras"] = sorted({n for n in parts if n > 0.5})[:10]
        return info

    # figures
    best = None
    for r in q["readings"]:
        miss = reading_missing(r, everything)
        if best is None or len(miss) < len(best[1]):
            best = (r, miss)
    exp = expected_numbers(q)
    info["extras"] = sorted({n for n in parts if n > 0.5 and not has_any(n, exp)})[:10]
    if best is None:
        info["verdict"] = "RIGHT" if not name_missing else "PARTIAL"
        return info
    miss = best[1]
    info["missing"] = miss + [f"name: {n}" for n in name_missing]
    in_parts = reading_missing(best[0], parts)
    if not miss and not name_missing:
        info["verdict"] = "RIGHT"
        if in_parts:
            info["note"] = "figures only in a table, not in the text: " + ",".join(in_parts)
    elif len(miss) < len(best[0]["req"]) and (everything):
        info["verdict"] = "PARTIAL"
    else:
        opt_hit = [lab for lab, alts in best[0]["opt"].items() if any(not isinstance(a, list) and has(a, everything) for a in alts)]
        if opt_hit:
            info["verdict"] = "PARTIAL"
            info["note"] = "only the total of the period is shown (" + ",".join(opt_hit) + ")"
        else:
            info["verdict"] = "WRONG" if parts or everything else "ASKED"
    return info


def has_any(n, exp):
    return any(abs(n - e) < 0.5 for e in exp)


def pct(xs, p):
    if not xs:
        return 0.0
    xs = sorted(xs)
    return xs[min(len(xs) - 1, max(0, round((len(xs) - 1) * p)))]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("rows", nargs="+")
    ap.add_argument("--questions", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "questions.json"))
    ap.add_argument("--report")
    ap.add_argument("--json")
    args = ap.parse_args()
    qs = {q["id"]: q for q in json.load(open(args.questions, encoding="utf-8"))}
    rows = {}
    for path in args.rows:
        for line in open(path, encoding="utf-8"):
            line = line.strip()
            if line:
                r = json.loads(line)
                rows[r["id"]] = r
    results = []
    for qid, q in qs.items():
        if qid in rows:
            res = score_one(q, rows[qid])
            errs = role_errors(q, rows[qid])
            if errs:
                if res["verdict"] in ("RIGHT", "ZERO-OK", "ASKED-OK", "LOCKED-OK"):
                    res["note"] = ("role retry: " + "; ".join(errs) + ". " + res["note"]).strip()
                else:
                    res["verdict"] = "WRONG-ROLE" if res["verdict"] in ("ASKED", "WRONG", "PARTIAL", "FAIL") else res["verdict"]
                    res["missing"] = res["missing"] + errs
            results.append(res)
    good = {"RIGHT", "ZERO-OK", "ASKED-OK", "LOCKED-OK"}
    n = len(results)
    right = sum(1 for r in results if r["verdict"] in good)
    secs = [r["secs"] for r in results if r["verdict"] != "NEEDS-MODEL"]
    local = [r["secs"] for r in results if r["local"] and r["verdict"] != "NEEDS-MODEL"]
    model = [r["secs"] for r in results if not r["local"] and r["verdict"] != "NEEDS-MODEL"]
    lines = ["# Complex-question live check (plan 82, lane C)", "",
             "| id | outcome | path | calls | secs | verdict | missing / note | extras (figures in the text not asked) | log id |",
             "|---|---|---|---|---|---|---|---|---|"]
    for r in results:
        row = dict(r)
        row.update(path="local" if r["local"] else "model",
                   note="; ".join(r["missing"] + ([r["note"]] if r["note"] else [])),
                   extras=", ".join(f"{x:g}" for x in r["extras"]))
        lines.append("| {id} | {outcome} | {path} | {calls} | {secs:.1f} | {verdict} | {note} | {extras} | {qid} |".format(**row))
    counts = collections.Counter(r["verdict"] for r in results)
    lines += ["", "## Summary", "", f"- questions scored: {n}",
              f"- right (RIGHT, ZERO-OK, ASKED-OK, LOCKED-OK): {right} of {n}",
              "- verdicts: " + ", ".join(f"{k} {v}" for k, v in sorted(counts.items()))]
    if secs:
        lines += [f"- time, all answered: p50 {pct(secs, .5):.1f} s, p95 {pct(secs, .95):.1f} s, max {max(secs):.1f} s",
                  f"- local (no model call): {len(local)}; p95 {pct(local, .95):.2f} s",
                  f"- model path: {len(model)}; p50 {pct(model, .5):.1f} s, p95 {pct(model, .95):.1f} s, max {max(model or [0]):.1f} s",
                  f"- over 15 s: {sum(1 for s in secs if s > 15)}; over 30 s: {sum(1 for s in secs if s > 30)}",
                  f"- model calls: " + ", ".join(f"{k} calls x{v}" for k, v in sorted(collections.Counter(r['calls'] for r in results).items())),
                  f"- cost counted: ${sum(r['cost'] for r in results):.4f}"]
    order = {"LEAK": 0, "WRONG-COLUMN": 1, "WRONG-DIRECTION": 1, "WRONG": 2, "WRONG-ROLE": 3, "FAIL": 4, "PARTIAL": 5, "ASKED": 6}
    bad = sorted((r for r in results if r["verdict"] in order), key=lambda r: (order[r["verdict"]], r["id"]))
    if bad:
        lines += ["", "## Findings (worst first)", ""]
        for r in bad:
            q = qs[r["id"]]
            lines.append(f"- **{r['verdict']}** {r['id']} (log id {r['qid']}, {r['secs']:.1f} s, {r['calls']} calls, outcome {r['outcome']}): "
                         f"{q['question']} | " + "; ".join(r["missing"] + ([r["note"]] if r["note"] else [])))
    complex_secs = [r["secs"] for r in results if not r["local"] and r["verdict"] != "NEEDS-MODEL"]
    if complex_secs:
        lines += ["", "## Acceptance (plan 82)", "",
                  f"- complex (model path) p95 {pct(complex_secs, .95):.1f} s (target under 15 s), longest {max(complex_secs):.1f} s (target under 30 s)",
                  f"- no time_limit or error outcome: {'yes' if not any(r['outcome'] in ('time_limit', 'error') for r in results) else 'NO'}",
                  f"- simple (local) under 1 s: {'yes' if all(r['secs'] < 1.0 for r in results if r['local'] and r['verdict'] != 'NEEDS-MODEL') else 'NO'}",
                  f"- right on the list: {right} of {n} ({100 * right / max(1, n):.0f}%)"]
    text = "\n".join(lines) + "\n"
    if args.report:
        open(args.report, "w", encoding="utf-8").write(text)
    if args.json:
        json.dump(results, open(args.json, "w", encoding="utf-8"), ensure_ascii=False, indent=1)
    print(text)


if __name__ == "__main__":
    main()
