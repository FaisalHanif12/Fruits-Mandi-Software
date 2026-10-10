# Mandi Software

A Windows desktop app for a commission agent (آڑھتی) in a fruit and vegetable mandi.
Urdu first (right to left), with a one-key switch to English. Works offline on one computer, with automatic cloud backup.

Stack: Rust + Tauri 2, React + TypeScript, SQLite.

## Where things are

| Path | What it holds |
|---|---|
| `.claude/CLAUDE.md` | How to work on this project. Start here. |
| `.claude/rules/` | Requirements: PRD, NFR, Tech Stack, Code Standards, Architecture, Release Pipeline |
| `.claude/agents/`, `.claude/skills/` | The Claude Code agent team and its skills |
| `.github/workflows/` | CI, Windows tests and Release |
| `backend/` | Rust workspace: Tauri app and business crates |
| `frontend/` | React + TypeScript screens |

## How code reaches the shop

1. Work happens on a branch, for example `feature/bikri-live-summary`.
2. A pull request to `main` runs **CI** (`Checks (Windows)`) and **Windows** (`Windows E2E and speed`).
3. When both are green, GitHub merges it.
4. **Release** tests the merged commit again, then builds, signs and publishes `vX.Y.N`.
5. The app on the shop computer offers the update. It installs only when the Owner agrees.

Full details: `.claude/rules/Release_Pipeline.md`.

## Secrets

This repository is public. Never commit keys, passwords, `.env` files, databases or client data.
`.gitignore` blocks the common cases. Secrets for the pipeline live in GitHub repository secrets.
Secrets for the app live in Windows Credential Manager on the shop computer.
