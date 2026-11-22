"""python3 -I scripts/ai-eval/complex/merge_final.py, then score.py on results-final-merged.jsonl.

Merges the latest row of every question: earlier live rows, then the free offline pass of the new
commit for rows the app now answers by itself, then the final live rows. Writes the merged file."""
import json, os, sys  # noqa: F401
HERE = os.path.dirname(os.path.abspath(__file__))
O = os.path.join(HERE, "..", "..", "..", "backend", "target", "ai-eval-complex")

def rows(name):
    p = os.path.join(O, name)
    if not os.path.exists(p):
        return []
    return [json.loads(l) for l in open(p) if l.strip()]

merged, source = {}, {}
for name in ["results-dev-live.jsonl", "results-year-live.jsonl", "results-dev-live-rerun1.jsonl", "results-year-live-rerun1.jsonl",
             "results-dev-live-rerun2.jsonl", "results-year-live-rerun2.jsonl"]:
    for r in rows(name):
        if r["id"].startswith("_"):
            continue
        merged[r["id"]] = r; source[r["id"]] = name
local_now = 0
for name in ["results-dev-offline-final.jsonl", "results-year-offline-final.jsonl"]:
    for r in rows(name):
        if r["id"].startswith("_"):
            continue
        # a row the app now answers itself (no model call) replaces the older one
        if r.get("outcome") != "offline" and r.get("model_calls", 0) in (0, None):
            merged[r["id"]] = r; source[r["id"]] = name; local_now += 1
for name in ["results-dev-live-final.jsonl", "results-year-live-final.jsonl"]:
    for r in rows(name):
        if r["id"].startswith("_"):
            continue
        merged[r["id"]] = r; source[r["id"]] = name
out = os.path.join(O, "results-final-merged.jsonl")
with open(out, "w") as f:
    for k in sorted(merged):
        f.write(json.dumps(merged[k], ensure_ascii=False) + "\n")
json.dump(source, open(os.path.join(O, "final-sources.json"), "w"), indent=1)
print(len(merged), "rows;", local_now, "local from the offline pass of the new commit;",
      sum(1 for s in source.values() if "final" in s and "live-final" in s), "live final")
