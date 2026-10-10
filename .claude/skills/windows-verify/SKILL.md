---
name: windows-verify
description: Verifies the Mandi Software on real Windows with WebView2 (installer build, Windows tests, end-to-end keyboard runs, Urdu rendering, screenshots in both languages, printing and PDF, scaling and window sizes) through the Windows CI runner, plus a short manual checklist for Umar. Run before any UI, print or release task is called done.
argument-hint: "[feature or 'full']"
allowed-tools: Bash(gh workflow *) Bash(gh run *) Bash(git status *) Bash(git log *) Bash(git branch *)
---

# Verify on Windows

The client uses Windows. A Mac renders with WebKit, not WebView2, so Urdu shaping, fonts, keyboard focus and printing can look right on a Mac and still be wrong for the client (CLAUDE.md, Gotchas). This check closes that gap.

**What to verify:** $ARGUMENTS

## Step 1: Preconditions

- Run `git branch --show-current`, `git status --short` and `gh run list --limit 5` to see where things stand.
- All local checks are green: `pnpm lint`, `pnpm test`, `pnpm i18n:check`, and the golden tests. Do not use Windows CI to find problems a local run would find.
- The work is committed and pushed on its feature branch. If it is not, stop and ask `tech-lead`, who owns commits.

## Step 2: Run the Windows workflow

```bash
gh workflow run windows.yml --ref <branch>       # the Windows build-and-test workflow in .github/workflows/
# wait about 10 seconds, then get the new run's ID (filtering avoids picking up an older run)
gh run list --workflow windows.yml --branch <branch> --event workflow_dispatch --limit 1
gh run watch <run-id>
gh run view <run-id> --log-failed                # only if it failed
```

The workflow must do all of the following. If it is missing a step, report it as a finding for `tech-lead`.

1. Build the installer (MSI and NSIS) with the offline WebView2 installer bundled (NFR-INS-1).
2. Run all Rust tests on Windows, including the golden tests.
3. Run the end-to-end suite with WebdriverIO and `tauri-driver`, with an Edge driver that matches the installed WebView2 version:
   - the PRD worked day by keyboard only (Section 4);
   - E2E-17 (draft after closing the app);
   - E2E-22 to E2E-27 (language switch, typing names, short codes, memories, remaining qty, payables);
   - the tests for $ARGUMENTS.
4. Take screenshots of every changed screen in Urdu and English, at 1024 × 600, 1366 × 768 and 1920 × 1080, and at the scaling combinations in NFR-USE-17 (Code Standards F23).
5. Print every changed document to PDF in both languages and on each paper size it uses.
6. Upload as artifacts: the installer, the test report, the screenshots and the PDFs.

## Step 3: Look at the results

```bash
gh run download <run-id> --dir .verify/<run-id>
```

Open the screenshots and PDFs and look at them yourself. Use this checklist:

- [ ] **Urdu text** is joined Nastaliq. No broken letters, no boxes, no `?`. Tall letters are not clipped in grid rows or printed tables.
- [ ] **Direction:** the Urdu screens are right-to-left and the English screens left-to-right. Nothing overlaps or is cut off.
- [ ] **Numbers** keep their order inside Urdu text, for example `(16 @ 900)`. Money columns line up on the physical right edge.
- [ ] **Window sizes:** no sideways scroll at 1024 × 600, 1366 × 768 or 1920 × 1080, at each supported scaling.
- [ ] **Prints and PDFs:** the right paper size, headers repeated on every page, no row split across pages, "Page x of y" correct, amounts in words correct in the print language.
- [ ] **Tests:** every end-to-end test passed. Read the report, not just the green tick.

## Step 4: Manual check by Umar (only for release candidates or big UI changes)

Give Umar this short list to do on a Windows PC or Windows virtual machine with the installer from the artifacts:

1. Install on a clean Windows 10 or 11 machine with no internet. It installs in under 2 minutes (NFR-INS-1), and later starts to the login screen in under 2 seconds (NFR-SPD-1).
2. Press `Ctrl+R`, `F5`, `Ctrl+F`, `Ctrl+P` and `Ctrl +`. None of them should do a browser action, because WebView2's browser keys are switched off (Tech Stack 7). `Ctrl +` changes the app's own text size.
3. Set Windows display scaling to 100%, 125% and 150% at 1366 × 768 and 1920 × 1080 (NFR-USE-17). The screens stay usable. At 1366 × 768 and 150%, the app shows the tip to use 125%.
4. Print a seller bill on the real A4 printer and a buyer slip on the slip printer. Check them against the screen.
5. Switch to English with `Alt+L` in the middle of a بکری. Nothing typed is lost.

## Step 5: Report

Send this to `tech-lead` (or to Umar, if he ran this skill directly):

```
WINDOWS VERIFY <feature> - run <run-id> - VERDICT: PASS | FAIL
Installer: built / failed
Rust tests on Windows: <passed>/<total>
End-to-end: <passed>/<total>, failed: <names>
Screens checked (ur/en, sizes): <list>
Prints checked (paper, language): <list>
Findings:
  1. [BLOCKER|MAJOR|MINOR] <one line> - evidence: <artifact file> - owner: frontend | backend
```

Any rendering, keyboard or printing problem that the client would see is at least MAJOR.
