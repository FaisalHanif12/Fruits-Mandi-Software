# Deliverables: how the whole build runs

Version 1.5, 15-10-2026 (D3 and D4 run together, Umar's answers in `changes-v1.5.md`; Mac phase: no CI/CD and no Windows installer until a Windows computer is ready, Section 10; PRD v1.4: one خرچہ screen, one voucher screen; v1.3: own goods, partner capital, print any day and party; v1.1: one review gate per goal run). Owner: Umar. Kept by `tech-lead`.

The whole Mandi Software is built in **five deliverables**. Each one is a file in this folder. You start one deliverable with one `/goal` line. The lead runs the agent team until that deliverable is finished and proven, then stops and reports.

| # | File | What you can do at the end | Plan files |
|---|---|---|---|
| D1 | `D1-foundation.md` | Install the app, run the first-run wizard, log in, move around the empty app in Urdu and English | 01 to 04 |
| D2 | `D2-accounts-names-rules.md` | Add sellers, buyers and all masters with names in two scripts, set commission rules, post the opening position (opening balances, the first پیشگی, day close and reopen) | 05 to 10 |
| D3 | `D3-bikri.md` | Enter, save and print a full بکری with the live summary, journal, پختہ کھاتہ and partner split | 11 to 14 |
| D4 | `D4-money-in-out.md` | Run a full day: وصولی, ادائیگی, later sales, edit and cancel, the firm's own goods, partner capital, expenses, جنرل, cash book, full day close | 15, 16, 16b, 17 |
| D5 | `D5-reports-backup-golive.md` | Back up and restore, see the dashboard, every report and analytics view, and hand over version 1.0 | 18 to 23 |

Why five: each deliverable ends with something the client could use or see, and each one needs the one before it. D3 cannot save a بکری without the accounts, rules and opening position from D2, and D2 cannot store anything without the database from D1.

These files live in `docs/deliverables/`. Their order replaces the MVP build order list in CLAUDE.md, which now points here.

**Mac phase now.** D3, D4 and D5 are built and tested on the Mac. GitHub CI/CD, pull requests, releases and the Windows installer wait for the Windows phase. Section 10 says what that changes. Where it disagrees with Sections 1 and 4 to 7, Section 10 wins.

**Scope.** PRD Section 26 "Must" and "Should" items are both in scope. "Could" (Phase 2) and "Won't" are not. Every Must and Should item, every E2E test and every NFR group sits in exactly one deliverable. A later deliverable may extend an earlier screen, and says so.

## 1. Before the first deliverable (Umar, once)

1. **Plugin for screen quality.** In Claude Code, run `/plugin install frontend-design@claude-plugins-official`.
2. **Updater keys.** Follow `docs/Release_Pipeline.md` 3.1 and 3.2: make the key, and add the two GitHub secrets. Put the **public** key (one line) in `docs/release/updater-pubkey.txt`. `backend` copies it into `tauri.conf.json`.
3. **First push.** Push the folder to `main` as it is (docs, `.claude/`, `CLAUDE.md`, `.github/`). This makes no release.
4. **Answer the open decisions** listed at the top of D1, D3 and D5. Write each answer in that file, under "Decisions". Each file says what the team does while a decision is still open. If an answer changes a requirement document (D5-Q1 does), run `/requirement-change` yourself before starting that deliverable. The lead cannot run that skill, and the goal forbids it from editing requirement documents.
5. **Repository rules.** After the first D1 pull request has run its checks once, add the `main` ruleset from `docs/Release_Pipeline.md` 3.4, so the check names can be picked. Section 4, point 6 says how the lead merges before and after that.

## 2. How to start a deliverable

1. Open a terminal in the project folder and run `claude --agent tech-lead`. Use a fresh session for each deliverable.
2. Switch to auto mode, so the long run does not stop at every permission prompt.
3. Paste the deliverable's `/goal` line from Section 3.
4. Leave it running. Type `/goal` at any time to see its status. The lead posts short progress lines after each feature.
5. At the end, the lead posts a **DONE REPORT** (or a **BLOCKED** report). Read it, answer any questions, then start the next deliverable.

Notes:

- Agent teams need an interactive session. Do not run a goal with `claude -p`; teammates are not spawned there.
- If you hit a usage limit, the goal pauses. Continue with `claude --continue`. The goal comes back, but teammates do not. The lead then reads `docs/deliverables/status.md` and the plan files, and spawns the team again.
- The goal is judged only from what the lead writes in the conversation. That is why the lead must paste real command output into the DONE REPORT, not just say "done".

## 3. The goal lines

Each line is under the 4,000-character limit. Copy it exactly.

**D1 and D2 together (finished 15-10-2026, DONE REPORT on `3d4a47e`).** Kept for the record.

```
/goal You are tech-lead. This resumes the D1+D2 goal run that stopped on 13-10-2026 in the end gate. Work locally only: no GitHub Actions, no pull request, no merge and no release in this run. First read docs/deliverables/changes-v1.3-v1.4.md, then README.md, D1-foundation.md, D2-accounts-names-rules.md and status.md fully. Commit Umar's document changes on their own first (changes file Section 4). The v1.3 and v1.4 changes need no D1 or D2 code; they are built in D3 to D5. Run git status and the full check set to see which end-gate fixes in the working tree are finished. Then, with backend and frontend (plus the money-calculations agent for money code), finish the open fixes listed in status.md (the balance speed fix with migration 0002, frontend's hand-off, the pnpm perf rerun) on the run branch and commit them. Then have reviewer and robustness recheck the whole run against every D1 and D2 acceptance item and attack list; send every finding to its owner, fix and recheck until both give PASS or PASS WITH MINOR. The open questions in status.md stay open with the team defaults already recorded: list them in the DONE REPORT and do not stop for them. The goal is met only when you have posted a message headed "D1+D2 DONE REPORT" in the README Section 7 format, showing: every acceptance item D1-A1 to D1-A16 and D2-A1 to D2-A17 ticked with the command you ran and its real local output, except an item that only the Windows workflow can prove, which is marked LOCAL-ONLY with the local evidence you have and listed under Follow-ups (items that name the final main are checked on the run branch); pnpm lint, pnpm typecheck, pnpm test, pnpm i18n:check and cargo test -p mandi-services --test golden_day exit 0 on the branch, with E2E-19 and E2E-23 passing; the final reviewer and robustness verdicts for the whole run; every fix committed on the run branch. Also stop if you have posted "D1+D2 BLOCKED" as README Section 8 describes. Never change a golden test number or a requirement document to reach the goal.
```

**D3 and D4 together (the next run, from 15-10-2026).** Use this line instead of the D3 and D4 lines below. Start it in a fresh session (`claude --agent tech-lead`), so no old goal or teammate is carried over.

```
/goal You are tech-lead. This is one goal run for D3 and D4 together (README Section 4a), in the Mac phase (README Section 10): with no GitHub Actions, no pull request, no merge, no release and no Windows installer. Create the run branch feature/bikri from feature/db-schema. Read docs/deliverables/changes-v1.5.md first, then README.md, D3-bikri.md, D4-money-in-out.md and status.md fully. Commit all uncommitted document changes on their own first, then the uncommitted override-default code change as its own commit (changes file Section 5). After every finished task with green tests, commit it and push the run branch. Then deliver D3 and D4 exactly as written: plan files 11, 12, 13, 14, 15, 16, 16b and 17 in that order, running the agent team as the files say. Open questions that have a team default do not stop the run: list them in the DONE REPORT. The goal is met only when you have posted a message headed "D3+D4 DONE REPORT" in the README Section 7 format, showing: every acceptance item D3-A1 to D3-A20 and D4-A1 to D4-A31 ticked with the command you ran and its real local output, except an item that only Windows can prove, which is marked LOCAL-ONLY with the local evidence you have, listed under Follow-ups and added to "Waiting for Windows" in status.md (items that name the final main are checked on the run branch); cargo test -p mandi-services --test golden_day passes with E2E-1 to E2E-10, E2E-12 to E2E-16b, E2E-13b, E2E-19 to E2E-21, E2E-28 to E2E-32, E2E-30b, E2E-34, E2E-34b, E2E-35, E2E-37, E2E-38 and the preview numbers of E2E-39, including the trial balance of 85,048 on both sides, and the backend parts of E2E-22 to E2E-27 pass in assist.rs, with no expected number changed; pnpm lint, pnpm typecheck, pnpm test, pnpm i18n:check, pnpm perf and cargo deny check exit 0 on the run branch, and the full test suite, including every D1 and D2 test, passes; reviewer and robustness gave PASS or PASS WITH MINOR in the one end-of-run gate (Section 4a), and the money-calculations agent reviewed all money code; every plan file and fix is committed and pushed on the run branch. Also stop if you have posted "D3+D4 BLOCKED" as README Section 8 describes. Never change a golden test number or a requirement document to reach the goal.
```

**D1**

```
/goal You are tech-lead. Read docs/deliverables/README.md and docs/deliverables/D1-foundation.md fully, then deliver D1 exactly as written, running the agent team as the files say. The goal is met only when you have posted a message headed "D1 DONE REPORT" in the README Section 7 format, showing: every acceptance item D1-A1 to D1-A16 ticked, each with the command you ran and its real output; pnpm lint, pnpm test and pnpm i18n:check exit 0 on the final main; reviewer and robustness gave PASS or PASS WITH MINOR in the end-of-run gate (Section 4a); the run's pull request is merged and the last Release run on main is green with its version number. Also stop if you have posted "D1 BLOCKED" as README Section 8 describes. Never change a golden test number or a requirement document to reach the goal.
```

**D2**

```
/goal You are tech-lead. Read docs/deliverables/README.md and docs/deliverables/D2-accounts-names-rules.md fully, then deliver D2 exactly as written, running the agent team as the files say. The goal is met only when you have posted a message headed "D2 DONE REPORT" in the README Section 7 format, showing: every acceptance item D2-A1 to D2-A17 ticked, each with the command you ran and its real output; pnpm lint, pnpm test and pnpm i18n:check exit 0 on the final main; E2E-19 and E2E-23 pass; the full test suite, including every earlier test, passes on the final main; reviewer and robustness gave PASS or PASS WITH MINOR in the end-of-run gate (Section 4a); the run's pull request is merged and the last Release run on main is green with its version number. Also stop if you have posted "D2 BLOCKED" as README Section 8 describes. Never change a golden test number or a requirement document to reach the goal.
```

**D3**

```
/goal You are tech-lead. Read docs/deliverables/README.md and docs/deliverables/D3-bikri.md fully, then deliver D3 exactly as written, running the agent team as the files say. Work in the Mac phase (README Section 10): with no GitHub Actions, no pull request, no merge, no release and no Windows installer. The goal is met only when you have posted a message headed "D3 DONE REPORT" in the README Section 7 format, showing: every acceptance item D3-A1 to D3-A20 ticked, each with the command you ran and its real local output, except an item that only Windows can prove, which is marked LOCAL-ONLY with the local evidence you have and listed under Follow-ups (items that name the final main are checked on the run branch); cargo test -p mandi-services --test golden_day passes with E2E-1 to E2E-5 and E2E-28 (its بکری part) included and no expected number changed; pnpm lint, pnpm typecheck, pnpm test and pnpm i18n:check exit 0 on the run branch; the full test suite, including every earlier test, passes on the run branch; reviewer and robustness gave PASS or PASS WITH MINOR in the end-of-run gate (Section 4a), and the money-calculations agent reviewed all money code; every plan file and fix is committed and pushed on the run branch. Also stop if you have posted "D3 BLOCKED" as README Section 8 describes. Never change a golden test number or a requirement document to reach the goal.
```

**D4**

```
/goal You are tech-lead. Read docs/deliverables/README.md and docs/deliverables/D4-money-in-out.md fully, then deliver D4 exactly as written, running the agent team as the files say. Work in the Mac phase (README Section 10): with no GitHub Actions, no pull request, no merge, no release and no Windows installer. The goal is met only when you have posted a message headed "D4 DONE REPORT" in the README Section 7 format, showing: every acceptance item D4-A1 to D4-A31 ticked, each with the command you ran and its real local output, except an item that only Windows can prove, which is marked LOCAL-ONLY with the local evidence you have and listed under Follow-ups (items that name the final main are checked on the run branch); cargo test -p mandi-services --test golden_day passes with E2E-1 to E2E-10, E2E-12 to E2E-16b, E2E-13b, E2E-19 to E2E-21, E2E-28 to E2E-32, E2E-30b, E2E-34, E2E-34b, E2E-35, E2E-37, E2E-38 and the preview numbers of E2E-39, including the trial balance of 85,048 on both sides, and the backend parts of E2E-22 to E2E-27 pass in assist.rs, with no expected number changed; pnpm lint, pnpm typecheck, pnpm test and pnpm i18n:check exit 0 on the run branch; the full test suite, including every earlier test, passes on the run branch; reviewer and robustness gave PASS or PASS WITH MINOR in the end-of-run gate (Section 4a), and the money-calculations agent reviewed all money code; every plan file and fix is committed and pushed on the run branch. Also stop if you have posted "D4 BLOCKED" as README Section 8 describes. Never change a golden test number or a requirement document to reach the goal.
```

**D5**

```
/goal You are tech-lead. Read docs/deliverables/README.md and docs/deliverables/D5-reports-backup-golive.md fully, then deliver D5 exactly as written, running the agent team as the files say. Work in the Mac phase (README Section 10): with no GitHub Actions, no pull request, no merge, no release and no Windows installer. The release of version 1.0, the installer and the Windows checks belong to the Windows phase, not to this run. The goal is met only when you have posted a message headed "D5 DONE REPORT" in the README Section 7 format, showing: every acceptance item D5-A1 to D5-A22 ticked, each with the command you ran and its real local output, except an item that only Windows can prove, which is marked LOCAL-ONLY with the local evidence you have and listed under Follow-ups (items that name the final main are checked on the run branch); every E2E test from E2E-1 to E2E-39 passes in the Rust golden tests with no expected number changed, and every Windows suite spec is written and syntax-checked (its run is LOCAL-ONLY); every NFR Section 2 target is measured on the Mac in scripts/perf and shown beside the NFR target, as README Section 10 defines; pnpm lint, pnpm typecheck, pnpm test and pnpm i18n:check exit 0 on the run branch; tauri.conf.json says 1.0.0 on the run branch; reviewer and robustness gave PASS or PASS WITH MINOR in the end-of-run gate (Section 4a) and on the final whole-app review; every plan file and fix is committed and pushed on the run branch, and status.md lists every LOCAL-ONLY item for the Windows phase. Also stop if you have posted "D5 BLOCKED" as README Section 8 describes. Never change a golden test number or a requirement document to reach the goal.
```

## 4. How the team works inside a deliverable

The lead follows `.claude/agents/tech-lead.md` for every plan file. In short:

1. **One plan file per feature.** The deliverable names them (for example `docs/plans/04-shell-and-login.md`). The lead writes each one with the contract, the worked numbers and the acceptance checklist, before anyone codes.
2. **Order.** Plan files run one after another, in number order. The team shares one working folder and one checkout, so two plan files on two branches at once would collide. The speed comes from inside a plan file: once T1 is done, `backend` and `frontend` work at the same time in their own folders.
3. **Tasks T1 to T7** as in tech-lead.md: `backend` writes the contract stubs first, then `backend` and `frontend` build in parallel, then `reviewer` and `robustness` check in parallel, then the fix loop. In a goal run (4a), T5 to T7 run once, at the end of the run.
4. **Skills.** Teammates use the project skills for their kind of work:

   | Work | Skill |
   |---|---|
   | A new or changed command | `tauri-command` |
   | Any screen, dialog or form | `bilingual-screen` |
   | Any grid where the user types | `entry-grid` |
   | Any user-facing words | `urdu-english-text` |
   | Any print, PDF or Excel | `print-and-export` |
   | Before a UI or print feature is called done | `windows-verify` |
   | Any money code | the `money-calculations` agent, before writing and again to review |

5. **One branch and one pull request per goal run** (Code Standards P3). The lead commits each built plan file on that branch, one Conventional Commit per plan step, and opens the pull request after the final gate.
6. **Merge.** Before the `main` ruleset exists, the lead never uses `gh pr merge --auto` (with no rules it would merge at once). It runs `gh pr checks <number> --watch`, and only when every check is green, `gh pr merge --squash`. Once `gh api repos/FaisalHanif12/Fruits-Mandi-Software/rulesets` shows the active `main` ruleset, the lead uses `gh pr merge --auto --squash` as tech-lead.md says. Each merge to `main` publishes a release. That is expected during the build.
7. **Status file.** The lead keeps `docs/deliverables/status.md`: plan files done, pull requests, verdicts, follow-ups, open questions. It updates it on each feature branch before the pull request, so it reaches `main` with the feature. (There are no docs-only pull requests.) This is how a resumed session knows where it stands.
8. **Progress lines.** After each plan file, the lead posts three lines in the conversation: what was built or merged, the evidence, what is next.
9. **Teammates.** As tech-lead.md says, the lead shuts the teammates down after each plan file and spawns fresh ones for the next, so each starts with a clean context. For the end-of-run gate (Section 6), it spawns a fresh `reviewer` and `robustness`.

### 4a. Goal runs (Umar, 11-10-2026)

Reviewing every plan file on its own made D1 too slow. So one goal covers a whole run (for example D1 and D2 together), and works like this:

1. The lead and `backend` and `frontend` build every plan file of the run, one after another (T1 to T4). A plan file is **built** when the builders' tests, lint and golden tests are green and the lead has checked its acceptance list with evidence. The lead marks it "built" in `status.md` and commits it.
2. When every plan file is built, the lead spawns a fresh `reviewer` and `robustness` once. They review the whole run: every acceptance item and attack list of its deliverables, and regressions against earlier work.
3. Every finding goes to the lead. The lead assigns it to its owner (`backend` or `frontend`), gets it fixed, and asks for a recheck. This repeats until both say PASS or PASS WITH MINOR.
4. Then the lead runs the end checks of Section 6 on the run's branch, opens the one pull request, and posts the DONE REPORT. While GitHub Actions is unavailable, the lead does not wait on `gh pr checks --watch`; the pull request stays open and the report says so.

### Worked example: the login screen (plan file 04)

- **Lead:** writes `docs/plans/04-shell-and-login.md`. The contract names `auth_login`, `auth_logout`, `auth_unlock`, `auth_status`, the error codes (`ERR_WRONG_PASSWORD`, `ERR_LOCKED_WAIT`) and the lockout numbers from FR-8.2. The lead runs `/design` for the login screen, both languages (Section 5), and saves the result in `docs/design/`.
- **backend:** T1 stubs and `pnpm gen:types`; then T2: Argon2id hashing, lockout timing, the audit rows for every login, the sign-in check. Integration test: 5 wrong tries, then a 1-minute wait, written to the audit log.
- **frontend:** T3 builds the screen against the stubs, from the design, with `bilingual-screen` and `urdu-english-text`: the language switch on the login screen, keyboard only, the four states. T4 wires it to the real commands.
- **reviewer:** checks FR-8.1 to FR-8.6, NFR-SEC-1 to NFR-SEC-3, both languages, no hard-coded words, no `left` or `right` in CSS, and the design match.
- **robustness:** 1,000 wrong passwords in a row, a very long password, Urdu letters in the password, the clock moved back during a lockout, the app killed during login. Measures login to dashboard against NFR-SPD-2.
- **Lead:** sends each finding to its owner, gets a recheck, then commits and updates `status.md` (in a goal run, the findings come at the end of the run).

## 5. Design: state of the art, made for the mandi

Every deliverable with screens starts with a design step, run by the lead before `frontend` builds.

1. **D1 sets the design system.** The lead runs the built-in `/design` skill with the brief below, for the login, the first-run wizard, the app shell and an empty dashboard, a sample form and a sample entry grid, each in Urdu and English. The lead writes the result to `docs/design/design-system.md`: colours with their one meaning each, type scale, spacing grid, focus style, components and their states. The design canvas link goes in that file too.
2. **D2 to D5 design their own screens** with `/design`, inside that design system. Each deliverable lists its screens.
3. **`frontend` builds from the design**, with the `frontend-design` plugin and the `bilingual-screen` skill, and keeps it inside our rules. If a design idea breaks a rule below, the rule wins and the lead notes the change in `docs/design/`.
4. **Check.** Screenshots from the Windows workflow, in both languages and at 1024 × 600, 1366 × 768 and 1920 × 1080, are compared with the design. `reviewer` checks the match.

If `/design` is not available in the session (it needs a claude.ai login with artifacts), the lead writes the design brief into the plan file and `frontend` designs with the `frontend-design` plugin. The rest stays the same.

**The design brief** (paste it into every `/design` call, with the screen list):

> A Windows desktop app for a fruit and vegetable commission agent in Pakistan. A munshi types sales all day with the keyboard. Older owners read the numbers. Urdu (Nastaliq, right to left) is the default; English (left to right) is one switch away, with the same layout mirrored.
> Make it modern, calm and premium: the clarity of the best finance tools, not a generic admin template. Strong type hierarchy, generous but dense-enough spacing, numbers that are easy to scan, and one clear main action per screen.
> Hard rules: works from 1024 × 600 with no sideways scroll; Nastaliq line height at least 1.8; text contrast at least 4.5 : 1; click targets at least 32 × 32; the focused field always clearly outlined; colours have one meaning each (green money in or balanced, red due or error, amber warning); no animation longer than 200 ms; numbers right-aligned with equal-width digits; every button shows its key (for example "محفوظ کریں F10").
> Show each screen in Urdu and in English, with its empty, loading, error and filled states.

These rules come from NFR-USE-10 to NFR-USE-24, NFR-URD-9 to NFR-URD-13 and PRD 7.1.

## 6. The gates

**After each plan file:** in a goal run (4a), the builders' checks and the lead's acceptance check only. All golden tests pass and the Code Standards Section 11 checklist is complete. The `reviewer` and `robustness` gate runs once, at the end of the run.

**Speed on the runner.** GitHub's Windows runner is not the reference hardware of NFR 0.2. In D1, `robustness` measures a runner factor once and records it in `scripts/perf/README.md`, with each NFR target and its runner threshold side by side. For example, if the runner is 1.5 times slower, the 300 ms target of NFR-SPD-3 becomes 450 ms on the runner. "Met" in any acceptance item means met at that runner threshold, measured the way the NFR names (benchmark, E2E timing or the long run). The run on reference hardware is on Umar's go-live checklist (D5 plan 23).

**At the end of each deliverable,** before the DONE REPORT:

1. All plan files of the run are built and committed on the run's branch.
2. Run the full local set: `pnpm lint`, `pnpm test`, `pnpm i18n:check`, `cargo test -p mandi-services --test golden_day` (once it exists), `cargo deny check`.
3. Run the `windows-verify` skill with `full` on the run's branch (when GitHub Actions is available).
4. Spawn a fresh `reviewer` for a **run review**: every acceptance item of every deliverable in the run, regressions against all earlier deliverables, consistency across this deliverable's screens.
5. Spawn a fresh `robustness` for the **attack lists** of every deliverable in the run.
6. Every finding goes to the lead, who assigns the fix to its owner. Recheck. Then open the pull request and write the DONE REPORT.

## 7. DONE REPORT format

```
D<n> DONE REPORT
Final main: <commit sha>    Release: v<version> (run <id>, green)

Plan files
| Plan | Pull requests | reviewer | robustness |
| 0x-<name> | #12, #13 | PASS | PASS WITH MINOR |

Acceptance
- [x] D<n>-A1 <item>. Evidence: `<command>` -> <the real result line>
- [x] D<n>-A2 ...

Checks on final main
- pnpm lint -> exit 0
- pnpm test -> <n> passed, 0 failed
- pnpm i18n:check -> <result>
- cargo test -p mandi-services --test golden_day -> <n> passed
- Windows run <id>: Checks (Windows) pass; Windows E2E and speed pass (<n>/<n>)

Design: <canvas link>; screenshots compared (ur and en, three sizes)
Follow-ups (MINOR, and WAITING-CI items): <list or "none">
Open questions for Umar: <list or "none">
```

In a goal run, the first line names the run (for example `D1+D2 DONE REPORT`), "Final main" becomes the run's branch and its last commit, the plan table has one row per plan file with "built" and its commit, the reviewer and robustness verdicts are given once for the whole run, and "Checks on final main" become checks on the run's branch.

Every "Evidence" is a real command and the real line it printed. "Looks fine" is not evidence.

## 8. When to stop early (BLOCKED)

Stop and post a BLOCKED report only when **all** of these are true:

- the question can only be answered by Umar (two documents disagree, a rule is missing, or a money number cannot be proven from the PRD);
- the PRD has no `[A]` default for it (an `[A]` item is built as stated, never a reason to stop);
- no other plan file in this deliverable can move forward without the answer.

If other work can continue, list the question under "Open questions" in `status.md`, do the other work, and ask at the end.

Also stop if the same check fails after **three** fix rounds with no progress. Paste the failing output.

```
D<n> BLOCKED
Question: <one sentence>
Why the documents do not answer it: <sections read>
Options: 1) <option, with a worked example>  2) <option, with a worked example>
Done so far: <plan files built or merged>   Waiting: <plan files>
```

## 9. Rules that hold in every deliverable

- CLAUDE.md is the rulebook. Its 12 rules are never broken to finish faster.
- Golden test numbers never change. If code and test disagree, the code is wrong.
- Requirement documents change only through the `requirement-change` skill, after Umar approves. A goal is never a reason to edit them.
- Each deliverable keeps every earlier deliverable working. A regression is a FAIL.
- Words on screen come from Fluent, in both languages. Each DONE REPORT lists the screens with new Urdu text, so the munshi can review them (NFR-URD-4).

## 10. Mac phase and Windows phase (Umar, 15-10-2026)

There is no Windows computer yet. So the app is built and tested on the Mac first. Windows comes later, as its own step, when the laptop is ready. Nothing in the requirement documents changes: the app is still a Windows app. Only the order of the work changes.

**Mac phase (now: D3, D4 and D5).**

- No GitHub CI/CD. The workflows stay in `.github/workflows-disabled/`, so a push runs nothing on GitHub. No pull request, no merge to `main` and no release.
- **Push after every task (Umar, 15-10-2026).** Each time a task is finished and its tests are green (a plan step T1 to T4, or one fix), the lead commits it and pushes the run branch (`git push -u origin feature/bikri`). GitHub then holds a backup of the work, so a dead laptop never loses more than one task. Example: backend finishes the override default with its tests green; the lead commits `feat(rules): commission override on by default` and pushes.
- No Windows installer. `pnpm build` and `pnpm e2e` are not run.
- Each run works on one branch and commits every plan file and fix there. Nothing is merged to `main`, so a new run starts its branch from the last run's branch. Example: D3 starts `feature/bikri` from `feature/db-schema`.
- Every check that runs on the Mac is run: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check`, `golden_day`, `cargo deny check` and `pnpm perf`.
- An acceptance item that only Windows can prove is marked **LOCAL-ONLY**. Examples: a Windows E2E run, a PDF from the Windows run, Urdu shaping in WebView2, the installer and its install time, the update test, a `windows-verify` report. The lead gives the local evidence it has (the spec written and syntax-checked, unit tests, print data snapshots, headless Chrome screenshots) and lists the item under Follow-ups and in `status.md` under "Waiting for Windows".
- Screens are compared with the design in headless Chrome on the Mac, in both languages and at the three sizes of Section 5.
- Speed is measured on the Mac with the runner factor 1.0 already recorded in `scripts/perf/README.md`.
- In the DONE REPORT, "Final main" is the run branch and its last commit, "Release" is "none (Mac phase)", and the Windows run line says "Windows phase".
- Section 1 steps 2, 3 and 5 (updater keys, first push, repository rules) wait for the Windows phase.

**Windows phase (when the Windows laptop is ready).** It runs as its own goal after D5. Its goal line is written then. In short:

1. Umar does Section 1 steps 2, 3 and 5.
2. The workflows move back from `.github/workflows-disabled/` to `.github/workflows/`.
3. The last run branch is pushed, one pull request is opened, and `Checks (Windows)` and `Windows E2E and speed` run.
4. Every LOCAL-ONLY item in `status.md` is run on Windows: the Windows E2E suite, Urdu shaping and printing in WebView2, screenshots, the installer and its install time, the update test, and speed on the reference computer.
5. What fails is fixed, the pull request is merged, and version 1.0 is published (D5 plan 23 and `docs/Release_Pipeline.md`).
