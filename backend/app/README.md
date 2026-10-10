# mandi-app

The Tauri 2 application: window, plugins and the thin commands (Code Standards B2, 5.6).

- May depend on: `mandi-services`, `mandi-reports`, `mandi-backup`.
- A command checks the role, parses the text input, calls one service and returns a typed result or a `UiError`.
- Commands: `src/commands/<module>.rs`. Register each one in `generate_handler!` in `src/lib.rs`, in `APP_COMMANDS` in `build.rs`, and in `capabilities/default.json`.
- `src/webview2.rs` (Windows only) turns off the browser keys, zoom and context menu of WebView2 (Tech Stack 7).
- Binary: `mandi-app`, built as `backend/target/release/mandi-app.exe` on Windows.

Generate the TypeScript types (the root `pnpm gen:types` runs this):

```bash
cd backend && TS_RS_EXPORT_DIR="$PWD/../frontend/src/bindings" cargo test -p mandi-app export_bindings
```

Without `TS_RS_EXPORT_DIR` the same test writes to a temporary folder, so a plain `cargo test` never changes `frontend/`.
