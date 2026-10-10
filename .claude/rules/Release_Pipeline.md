# Release pipeline: push, test, merge, publish

How code gets from a feature branch to the client's computer with no manual deploy.
Related: Tech Stack 7, Architecture 17, NFR-INS-3, NFR-INS-4, NFR-SEC-9, Code Standards P1, P5 and 9.2.

## 1. The flow

| Step | What happens | Who |
|---|---|---|
| 1 | Work is pushed to a branch, for example `feature/bikri-live-summary` | `tech-lead` |
| 2 | **CI** runs on Windows: lint, all tests including the golden money tests, the i18n check, the generated types check, `cargo deny`, `pnpm audit` | GitHub |
| 3 | A pull request to `main` is opened with auto-merge on (`gh pr create`, then `gh pr merge --auto --squash`) | `tech-lead` |
| 4 | **Windows** also runs on the pull request: real installers, the keyboard-only end-to-end suite, screenshots, PDFs and the speed benchmarks | GitHub |
| 5 | Both checks are green, so GitHub merges it. If either is red, nothing merges. | GitHub |
| 6 | **Release** runs on `main`: CI and Windows again on the merged commit, then build, sign and publish `vX.Y.N` with `latest.json` | GitHub |
| 7 | The client's app checks for updates when online, shows "what's new" in Urdu, and installs only when the Owner agrees, never during work, after a backup | The app |

Example: you merge the live summary at 3 pm. By about 3:40 pm, release `v0.1.12` is on GitHub. The next time the shop computer is online, the app tells the Owner "نیا ورژن دستیاب ہے" with the notes. He presses update after closing the day.

"Auto deploy" here means auto publish. The app never installs silently. NFR-INS-3 requires the Owner's agreement. Changing that is a requirement change for Umar.

## 2. The three workflow files

| File | Runs on | Job name (used in the branch rules) |
|---|---|---|
| `.github/workflows/ci.yml` | every push to a branch, every pull request to `main`, and inside Release | `Checks (Windows)` |
| `.github/workflows/windows.yml` | every pull request to `main`, inside Release, and on demand (`windows-verify` skill) | `Windows E2E and speed` |
| `.github/workflows/release.yml` | every merge to `main` that changes app files | `Build, sign and publish` |

A merge that changes only `docs/`, `.claude/`, `.github/` or `.md` files makes no release. Example: fixing a typo in the PRD does not ask the Owner to update.

### How the version is set

The version is `major.minor` from `backend/app/tauri.conf.json` plus the Release run number.

Example: the file says `0.1.0` and this is Release run 12, so the app becomes `0.1.12`. The next run gives `0.1.13`. The number always goes up, so the updater always sees a new release as newer. At go-live, change the file to `1.0.0`; releases then start at `1.0.N`.

### Release notes

The release text, and the "what's new" the Owner sees, comes from `docs/release/whats-new.md`, Urdu first, then English. `tech-lead` updates it on the feature branch before merging any change the user can see. The Urdu text follows NFR-URD-4.

## 3. One-time setup

### 3.1 Updater signing keys (do this once, keep them forever)

On your Mac:

```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/mandi-updater.key
```

It asks for a password and prints the **public key**.

- The public key goes into `tauri.conf.json` (3.3).
- The private key file and its password go into GitHub secrets (3.2).
- Save both in your password manager, and keep one offline copy.

**If the private key is lost, installed copies can never take another update.** Each would need a manual reinstall.

### 3.2 GitHub secrets

```bash
gh secret set TAURI_SIGNING_PRIVATE_KEY --repo FaisalHanif12/Fruits-Mandi-Software < ~/.tauri/mandi-updater.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo FaisalHanif12/Fruits-Mandi-Software
```

Later, when the Windows code-signing certificate is bought (Tech Stack 7), add `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD`, and uncomment those lines in `release.yml`.

### 3.3 App settings the scaffold must have

`backend` adds these to `backend/app/tauri.conf.json` in build step 1 (the scaffold):

```json
{
  "productName": "Mandi Software",
  "version": "0.1.0",
  "identifier": "pk.mandisoftware.app",
  "bundle": {
    "targets": ["nsis", "msi"],
    "createUpdaterArtifacts": true,
    "windows": { "webviewInstallMode": { "type": "offlineInstaller" } }
  },
  "plugins": {
    "updater": {
      "pubkey": "<the public key from 3.1>",
      "endpoints": [
        "https://github.com/FaisalHanif12/Fruits-Mandi-Software/releases/latest/download/latest.json"
      ],
      "windows": { "installMode": "passive" }
    }
  }
}
```

The root `package.json` needs these scripts. CLAUDE.md lists them.

| Script | Used by |
|---|---|
| `tauri` (the Tauri CLI) | Windows, Release |
| `lint`, `typecheck`, `test`, `i18n:check`, `gen:types` | CI |
| `e2e` (WebdriverIO + `tauri-driver`, writes to `frontend/tests/e2e/output/`) | Windows |
| `perf` | Windows |

Also set `"packageManager": "pnpm@<exact version>"` so CI uses the same pnpm. The Node version is in `.nvmrc` (already in the repository). `cargo deny check` needs `backend/deny.toml`. The shared setup step (`.github/actions/setup`) fails with a clear message until `package.json`, `pnpm-lock.yaml`, `rust-toolchain.toml`, `backend/Cargo.toml` and `backend/app/tauri.conf.json` exist.

### 3.4 Repository settings on GitHub

1. **Settings, General, Pull Requests:** allow squash merging, tick **Allow auto-merge**, tick **Automatically delete head branches**.
2. **Settings, Rules, Rulesets, New branch ruleset**, named `main`:
   - Enforcement: Active. Target: the default branch. Bypass list: empty.
   - Tick: Restrict deletions, Block force pushes.
   - Tick: Require a pull request before merging. Required approvals: 0, because GitHub does not let you approve your own pull request. The checks are the gate.
   - Tick: Require status checks to pass, and **Require branches to be up to date before merging**. Add `Checks (Windows)` and `Windows E2E and speed`. They appear in the search box after they have run once on any pull request.
3. **Settings, Actions, General:** keep the default workflow permissions as read-only. Each workflow asks for what it needs.

### 3.5 First push

The repository is empty. Push the docs, `.claude/`, `CLAUDE.md` and `.github/` to `main` once, before you turn on the ruleset. This first push makes no release, because it holds only ignored paths. From then on, all work goes through pull requests.

CI cannot pass until the scaffold exists (build step 1). The scaffold's pull request is the first one that must go green.

### 3.6 First install on the client's computer

Open the repository's **Releases** page, download `Mandi Software_<version>_x64-setup.exe`, and run it on the shop computer. Until the code-signing certificate is bought, Windows SmartScreen shows a warning; choose "More info", then "Run anyway". Every later version arrives through the in-app update.

## 4. Things to know

- **The repository is public.** Anyone can read the code and the docs (including the client's name and the worked numbers) and download the installer. Client data is never in the repository. If you make it private, the app can no longer read `latest.json` from GitHub Releases. Updates must then be hosted elsewhere, for example a public Cloudflare R2 bucket kept separate from the backup bucket. That is one endpoint change and one upload step.
- **Speed tests run on GitHub's computers, not the reference hardware** (NFR Section 2). They catch big slowdowns. `robustness` sets the thresholds in `scripts/perf/` so a normal runner passes. The real targets are still checked on reference hardware before go-live.
- **A failed release changes nothing.** If any check fails after the merge, no release is published and the client keeps the last version. Fix it on a new branch; the next merge releases.
- **Re-run a release** from the Actions tab with "Run workflow" on Release. It gets a new run number, so a new version.
