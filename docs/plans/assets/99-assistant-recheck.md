# Plan 99: the old/new round on the 8 NFR-AI-2 misses (not run yet)

Tried on 27-11-2026 from 19:38 to 20:38: both programs waited at the keychain prompt with the Mac
locked, then the one-hour limit stopped them. No model call was made and no result file was written.

Why: the NFR-AI-2 live run of 27-11-2026 had 8 misses with no old-version result to compare with.
Questions: Q032, Q055, Q065, Q069, Q083, Q086, Q090, Q100 (`scripts/ai-eval/questions-110.tsv`).
Both builds use the same seeded worlds and the same per-world clock. About 16 live questions.

What to look for:
- Q055 on new must show -1,004 (it showed 996; fixed, not yet re-asked live).
- Q083 and Q086 on new should put Arshad Multani (18,684) before Akram (2,880): backend-99r's R20
  tie-break (equal days, larger balance first) is in `backend/crates/reports/src/reports/view.rs`.
- A question where NEW is worse than OLD (wrong figure, wrong source, worse outcome) is a
  regression to fix and re-ask. Equally bad in OLD: a follow-up.

## Before running

- Mac unlocked and awake, someone at it.
- macOS asks once for EACH test program: "robustness_live wants to use your confidential
  information stored in pk.mandisoftware.app". Enter the Mac password and click Always Allow.
  Without it the program waits and does nothing. The key is never printed.

## 1. Set the paths

```bash
S=/private/tmp/claude-501/-Users-faisalhanif-MyProfile-My-Projects--Fruit-Mandi-Project/5089e34e-19f5-4b81-84c9-6e274b920ba1/scratchpad
R="/Users/faisalhanif/MyProfile/My Projects /Fruit Mandi Project"
export CARGO_TARGET_DIR=/private/tmp/claude-501/-Users-faisalhanif-MyProfile-My-Projects--Fruit-Mandi-Project/14aca70d-2ffc-4c44-a120-e7ac5c79f7d2/scratchpad/target-99ai
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3
```

## 2. The two programs

Old: `$S/before-bin/robustness_live` is already built. It is the committed old assistant
(HEAD 783c990d; the one difference is the plan 97 `deduction` left-out column, needed to compile).

New: `$S/new-bin/robustness_live` was built on 27-11-2026 at about 19:30. Rebuild it only if the code
has changed since (a rebuild means a new keychain prompt). Always delete before copying: macOS kills
a program that was overwritten in place.

```bash
cd "$R/backend"
cargo test -p mandi-assistant --test robustness_live --no-run
f=$(ls -t "$CARGO_TARGET_DIR"/debug/deps/robustness_live-* | grep -v '\.d$' | head -1)
mkdir -p "$S/new-bin" && rm -f "$S/new-bin/robustness_live" && cp "$f" "$S/new-bin/robustness_live"
```

If the scratchpad was cleared (reboot), build the old one from a separate worktree, with its own
target folder, then remove the worktree:

```bash
git -C "$R" worktree add "$S/old-tree" 783c990d
(cd "$S/old-tree/backend" && CARGO_TARGET_DIR="$S/target-old" cargo test -p mandi-assistant --test robustness_live --no-run)
f=$(ls -t "$S"/target-old/debug/deps/robustness_live-* | grep -v '\.d$' | head -1)
mkdir -p "$S/before-bin" && cp "$f" "$S/before-bin/robustness_live"
git -C "$R" worktree remove "$S/old-tree"
```

## 3. Ask the 8 questions (one build after the other)

```bash
mkdir -p "$S/nfr" && cd "$S/nfr"
for v in old:before-bin new:new-bin; do
  name=${v%%:*}; bin=${v##*:}
  MANDI_EVAL_QUESTIONS="$R/scripts/ai-eval/questions-110.tsv" \
  MANDI_EVAL_OUT="$S/nfr/base-$name.json" \
  MANDI_EVAL_ONLY="Q032,Q055,Q065,Q069,Q083,Q086,Q090,Q100" \
  MANDI_EVAL_THREADS=2 \
  caffeinate -i "$S/$bin/robustness_live" live_100_questions --ignored --exact
done
```

## 4. Read and score

```bash
python3 "$S/nfr/compare8.py" "$S/nfr/base-old.json" "$S/nfr/base-new.json"
cd "$R"
for name in old new; do
  node scripts/ai-eval/run.mjs --results "$S/nfr/base-$name.json" \
    --only Q032,Q055,Q065,Q069,Q083,Q086,Q090,Q100 \
    --out-dir "$S/nfr/score-$name" --review-out "$S/nfr/score-$name/urdu-review.tsv" \
    --summary-out "$S/nfr/score-$name/summary.json" --detail-out "$S/nfr/score-$name/misses.txt"
done
```

Always pass the four output options: without them the scorer rewrites
`scripts/ai-eval/urdu-review.tsv` in the repository.

## Also still open

- Q071 now goes to the model (it used to get a balance for a percent); never asked live.
- The full NFR-AI-2 run and the Urdu review before release. 27-11-2026 result: 101 of 109 (92.7%),
  target 97. Evidence: `$S/nfr/after-live-merged.json`, `$S/nfr/score/report-merged.txt`.
- The replay of the 43 logged questions: `$S/replay/table-merged.md`.
