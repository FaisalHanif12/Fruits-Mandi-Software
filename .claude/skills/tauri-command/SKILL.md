---
name: tauri-command
description: Adds or changes one Tauri command end to end in the Mandi Software, from the Rust types and command to the generated TypeScript types and the frontend api.ts call. Use whenever a screen needs to read or save data through the backend, or when a command's input, output or errors change.
---

# Add or change a Tauri command

A command is the only door between a screen and the data. Every command has the same shape, so a reviewer can check any of them in a minute.

**Example used below:** `vouchers_save_receipt`, which saves one وصولی (PRD 15.2).

Full code templates are in [templates.md](templates.md). Read them before writing code.

## Before you start

1. Find the command in the feature's plan file (`docs/plans/<nn>-<feature>.md`). Its name, input fields, output fields and error codes are the **contract**. If the command is not in the plan, stop and ask `tech-lead` to add it first.
2. Name it `<module>_<verb>[_<noun>]` (Tech Stack Section 6), for example `vouchers_save_receipt` or `bikri_preview`. The module name must be one from Tech Stack 5.2.
3. There is one user, the Owner, and no roles (PRD Section 2). Every command needs the signed-in Owner, except first run, sign-in, recovery, app info and data status.

## Steps (backend owner)

Do them in this order. Each step has a template in [templates.md](templates.md).

1. **Input type** in `backend/app/src/commands/<module>.rs` (or the module's types file).
   - Every value the user typed arrives as `String`: money, quantity, rate, percent, date, code. Never `i64` or `f64` from the screen.
   - IDs of things the user picked from a list arrive as typed IDs.
   - `#[derive(Deserialize, TS)]`, `#[serde(rename_all = "camelCase")]`, `#[ts(export)]`.
2. **Output type.** Money fields are `Paisa` with `#[ts(type = "number")]`. Include `working` records for every calculated amount the screen shows (Code Standards B30). Include everything the screen needs, so the frontend never has to calculate anything.
3. **Parse step.** `XxxDraft::parse(input) -> Result<XxxDraft, UiError>` uses only `core::validate` and `core::money` parsers. It collects **all** field errors and returns them together, each with its `field` path (Code Standards 4.3).
4. **Service function** in `backend/crates/services/src/<module>/`: one function, one transaction, takes the parsed draft and the current user. Business rules live in `domain`.
5. **Command function.** Thin, in this order: clone what it needs from `AppState`, call `require_login()` as the first check, then `spawn_blocking` with parse and the service call. No business logic.
6. **Register it** in two places:
   - the `generate_handler![...]` list in `backend/app/src/lib.rs`;
   - the app command list in `backend/app/build.rs` and the matching allow permission in `backend/app/capabilities/main.json`, so the screens can call only our listed commands (Code Standards S4). After a build, check the generated permission name under `backend/app/gen/schemas/` and use exactly that name.
   - Capabilities apply per window label. If another window calls the command (for example the hidden print window), its label must be listed in that capability too.
7. **Error codes.** Add every new `ERR_...` code to the error enum, and tell `frontend` the list, so they add the messages to `errors.ftl` in both languages.
8. **Generate the types:** `pnpm gen:types`. Commit the generated files with the change. Never edit `frontend/src/bindings/` by hand.
9. **Tests** (templates in [templates.md](templates.md)):
   - Parse: one test per rejected input in Code Standards 4.2 that applies (decimals, negative, empty, too large, wrong script, closed date).
   - Service: an integration test on a temporary database. Data written, journal balanced, audit row written, everything in PRD 6.2 for this action updated.
   - Sign-in: before sign-in and while locked, the command is refused (`ERR_NOT_LOGGED_IN`, `ERR_SESSION_LOCKED`) and writes nothing.
10. **Tell `frontend`:** the command name, the type names, the error codes, and whether it is a stub or real.

### Stubs for the contract (task T1)

When the plan is new, first add the types and a command that returns `ERR_NOT_IMPLEMENTED` with no side effects. Generate the types and message `frontend`. They build the screen while you write the real service.

## Steps (frontend owner)

1. **Wrapper** in `frontend/src/features/<module>/api.ts`. It is the only place that calls `invoke`. It takes typed input and returns the typed output, or throws an `ApiError` holding the error list.
2. **Hook** in `frontend/src/features/<module>/hooks/`: `useQuery` for reads and `useMutation` for saves. After a save, invalidate exactly the queries that PRD 6.2 says this action updates. Example: a وصولی invalidates the buyer ledger, the cash book, the open bills, the اگراہی list and the dashboard.
3. **Errors.** Map each error to its field with the shared error helper, and show the message from `errors.ftl` under that field or grid cell. Never show the raw code.
4. **Money and quantities go to the backend as the text the user typed.** Display returned `Paisa` only through `formatPaisa`.
5. **Live preview** (screens with live totals): call the preview command 50 ms after the last key press, and cancel the previous call (Code Standards F35).

## Changing an existing command

- Any change to the input, output or error codes is a contract change. Agree it with the other side and ask `tech-lead` to record it in the plan file **before** changing code.
- Prefer adding a new optional field over renaming or removing one.
- Run `pnpm gen:types`. The TypeScript build will show every screen that needs updating. Fix them all in the same change.

## Done when

- [ ] Name follows `<module>_<verb>[_<noun>]` and matches the plan.
- [ ] Sign-in check is the first line. Parse collects all errors with field paths.
- [ ] No business logic in the command. One service call, one transaction.
- [ ] Registered in `lib.rs`, `build.rs` and the capability file.
- [ ] Types generated; no hand-edited bindings.
- [ ] Parse, service and sign-in tests pass.
- [ ] Every new error code has a message in `ur` and `en` (`pnpm i18n:check` passes).
- [ ] Frontend calls it only through `api.ts`, sends text, and shows backend values only.
