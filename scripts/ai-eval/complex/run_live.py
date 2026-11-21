#!/usr/bin/env python3
"""Runs the complex-question list through `robustness_complex_live` (plan 82, lane C).

    python3 -I scripts/ai-eval/complex/run_live.py --world dev  [--offline] [--only D01,D02] [--budget 1.00]
    python3 -I scripts/ai-eval/complex/run_live.py --world year [--local 0]

- The databases are COPIES (`backend/target/ai-eval-complex/dev.db` and `year.db`); the test copies each
  again for every question. The live file of the app is never opened.
- `--offline`: no key, no network, no model. The app's own instant answers and reads run for real; what
  needs the model ends as `offline`. Free.
- Live: the key is read by the app's own key store (the Mac keychain) inside the test; this script never
  sees it. The money spent is kept in `ledger.json` next to the rows, so several runs add up against
  `--budget` (dollars, default 1.00 for the whole lane).
- It prints nothing secret and passes no secret.
"""
import argparse
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
OUT = os.path.join(ROOT, "backend", "target", "ai-eval-complex")
WORLDS = {"dev": ("dev.db", "2026-11-20"), "year": ("year.db", "2026-11-05")}


def spent_so_far():
    path = os.path.join(OUT, "ledger.json")
    if not os.path.exists(path):
        return 0.0
    return sum(e.get("spent", 0.0) for e in json.load(open(path)))


def add_ledger(world, spent, note):
    path = os.path.join(OUT, "ledger.json")
    entries = json.load(open(path)) if os.path.exists(path) else []
    entries.append({"world": world, "spent": round(spent, 6), "note": note})
    json.dump(entries, open(path, "w"), indent=1)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--world", required=True, choices=["dev", "year"])
    ap.add_argument("--offline", action="store_true")
    ap.add_argument("--only", default="")
    ap.add_argument("--budget", type=float, default=1.00)
    ap.add_argument("--local", default="1", choices=["0", "1"])
    ap.add_argument("--tag", default="")
    args = ap.parse_args()
    db, today = WORLDS[args.world]
    tag = ("-offline" if args.offline else "-live") + (f"-{args.tag}" if args.tag else "")
    out = os.path.join(OUT, f"results-{args.world}{tag}.jsonl")
    remaining = max(0.0, args.budget - spent_so_far())
    if not args.offline and remaining < 0.02:
        print(f"BUDGET STOP: ${spent_so_far():.4f} of ${args.budget:.2f} is spent")
        return 4
    env = dict(os.environ)
    env.update({
        "CARGO_BUILD_JOBS": "2",
        "MANDI_CX_DB": os.path.join(OUT, db),
        "MANDI_CX_TODAY": today,
        "MANDI_CX_QUESTIONS": os.path.join(HERE, "questions.json"),
        "MANDI_CX_WORLD": args.world,
        "MANDI_CX_ONLY": args.only,
        "MANDI_CX_OUT": out,
        "MANDI_CX_BUDGET": f"{remaining:.4f}",
        "MANDI_CX_LOCAL": args.local,
    })
    if args.offline:
        env["MANDI_CX_OFFLINE"] = "1"
        name, extra = "offline_complex_questions", []
    else:
        name, extra = "live_complex_questions", ["--ignored"]
    cmd = ["cargo", "test", "-j", "2", "-p", "mandi-assistant", "--test", "robustness_complex_live", "--",
           *extra, "--nocapture", "--exact", name]
    print("run:", " ".join(cmd), "->", out, flush=True)
    code = subprocess.call(cmd, cwd=os.path.join(ROOT, "backend"), env=env)
    if os.path.exists(out) and not args.offline:
        cost = sum(json.loads(l).get("cost", 0.0) for l in open(out) if l.strip())
        add_ledger(args.world, cost, f"{tag} only={args.only or 'all'}")
        print(f"spent this run ${cost:.4f}; total ${spent_so_far():.4f} of ${args.budget:.2f}")
    return code


if __name__ == "__main__":
    sys.exit(main())
