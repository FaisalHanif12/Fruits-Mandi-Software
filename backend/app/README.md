# mandi-app

The Tauri 2 application: window, plugins and the thin commands (Code Standards B2, 5.6).

- May depend on: `mandi-services`, `mandi-reports`, `mandi-backup`.
- A command checks the role, parses the text input, calls one service and returns a typed result or a `UiError`.
- Commands: `src/commands/<module>.rs`. Register each one in `generate_handler!` in `src/lib.rs`, in `APP_COMMANDS` in `build.rs`, and in `capabilities/default.json`.
- `src/error.rs`: the `UiError` every command returns. Each one gets a reference `E-<yyyymmdd>-<counter>` (`src/refs.rs`) that is also written in the log line (NFR-OBS-3).
- `src/runner.rs`: `guard` and `run_blocking` wrap every command, so a panic becomes `ERR_INTERNAL` with a reference and never closes the app.
- `src/logging.rs`: daily log files in `<app_data_dir>/logs/`, 30 days and 50 MB, secret fields written as `[hidden]`.
- `src/state.rs`: `AppState` holds the open data file, or the start-up error with its reference. `app_data_status` tells the screen which.
- `src/webview2.rs` (Windows only) turns off the browser keys, zoom and context menu of WebView2 (Tech Stack 7).
- Printing (plan 14): `src/print_job.rs` is the action behind `reports_print` (check, jobs, print, then the print count and audit row). `src/print_types.rs` holds the plain job data. `src/print_windows.rs` (Windows only) opens one hidden window `print-<job>` per document, waits for the event `print-ready-<job>` from the frontend print route, and calls `src/webview2/print.rs` (the WebView2 `Print` and `PrintToPdf` calls and the printer list; its `unsafe` is allowed by `webview2.rs`). `capabilities/print.json` gives the `print-*` windows only the print data command and events. Printing cannot be run on a Mac: the code is type-checked against the Windows target and checked on real Windows (`windows-verify`).
- Binary: `mandi-app`, built as `backend/target/release/mandi-app.exe` on Windows.

Generate the TypeScript types (the root `pnpm gen:types` runs this):

```bash
cd backend && TS_RS_EXPORT_DIR="$PWD/../frontend/src/bindings" cargo test -p mandi-app export_bindings
```

Without `TS_RS_EXPORT_DIR` the same test writes to a temporary folder, so a plain `cargo test` never changes `frontend/`.
