# Plan 75: Modern polish, logout in the sidebar, and a guide card on every report

Branch: `feature/d8-demo-feedback`. Asked by Umar on 13-11-2026, with four screenshots.

## 1. Goal

Make the parts of the app that look plain or "old desktop" look modern and refined, without changing the overall design, which Umar likes. Four things:

1. The dark brand panel on the login screen (logged out) looks unfinished: the app name breaks into two lines in the wrong place, the firm and the mandi wrap badly, and the panel is empty below them. Make it polished and modern.
2. Log out moves from the user menu in the top bar to the bottom of the main sidebar, where the dark space is empty today.
3. A short design pass over the rest of the app: only the parts that look average or typical get a modern touch. The layout, the colours' meaning and the keyboard stay the same.
4. The reports page has many cards and Umar cannot tell where to go to print what. When a report card is hovered (or reached with the keyboard), a larger detail card opens beside it with everything about that report: what it shows, its columns, its filters, what can be printed, and the paper and languages. Then the Owner knows where to go before he opens it.

## 2. Requirements

- PRD FR-5.2, FR-5.6 (changed: log out leaves the user menu and sits at the bottom of the main bar, v1.38), FR-7.3, FR-7.4, 8.1 (login screen), 19.1, 19.2, FR-19.3, FR-19.13, new FR-19.14 (the guide card, v1.38).
- NFR-USE-10 to NFR-USE-24, NFR-USE-28 (both languages), NFR-URD-9 to NFR-URD-13, NFR-SPD-3 (a screen opens under 300 ms), NFR-USE-24 (no animation over 200 ms).
- Code Standards F7, F9 to F14, F17, F24 to F29, F31 to F33, F37, G8.
- No money, no posting, no golden test is touched.

## 3. Worked numbers

None. Nothing in this plan calculates or shows a new figure.

## 4. Contract between backend and frontend

One change, to the report catalogue that `reports_catalog` already returns.

`ReportCatalogEntry` gains one field:

| Field | Type | Meaning |
|---|---|---|
| `column_title_ids` | `string[]` | The Fluent message IDs of the report's column headings, in the order the report shows them. They come from the same definition the report itself uses when it runs, so the guide card can never list a column the report does not have. A report whose columns depend on a filter (for example a choice of view) lists the columns of its default view. |

- No new command, no new error code, no new table or migration.
- Every ID in `column_title_ids` must exist in both `ur/reports.ftl` and `en/reports.ftl` (the existing column headings). A backend test checks every ID against the embedded Fluent files.
- The frontend reads the filters, `party_required` and `column_title_ids` from the entry. It names no report the backend does not send.
- Decision (lead, after T1): most heading IDs are in `reports.ftl`, but R31 uses three from `partners.ftl` and R37 three from `fruit.ftl`. All exist in both languages. The backend test checks every ID against the whole embedded Fluent bundle, and the frontend localises the IDs through the full bundle, not `reports.ftl` alone.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Add `column_title_ids` to the catalogue entry from the report's own column definitions; test that every ID is in both Fluent files; `pnpm gen:types` | backend | |
| T2 | Login brand panel polish (Section 7.1) | frontend | |
| T3 | Log out at the bottom of the main sidebar; remove it from the user menu (Section 7.2) | frontend | |
| T4 | Design audit: screenshots of every main screen in both languages; send the lead a list of at most 10 parts that look average, each with the change proposed. Build only what the lead approves | frontend | |
| T5 | Build the approved polish items | frontend | T4 approval |
| T6 | Shared `HoverDetails` component (`shared/ui/HoverDetails.tsx`, new file) on the `radix-ui` HoverCard, opening on hover and on keyboard focus | frontend-reports | |
| T7 | Report guide card on the reports home page (Section 7.3), with its Fluent text in both languages | frontend-reports | T1, T6 |
| T8 | Review and robustness for the whole plan | reviewer, robustness | Umar says continue |
| T9 | Fix loop, commit, push | tech-lead | T8 |

### File ownership (two teammates never edit the same file)

| Teammate | Owns in this plan |
|---|---|
| `backend` | `backend/**`, `frontend/src/bindings/` through `pnpm gen:types` only |
| `frontend` | `frontend/src/app/**`, `frontend/src/features/auth/**`, `frontend/src/styles/**`, `frontend/src/shared/**` except `shared/ui/HoverDetails.tsx`, every Fluent file except `reports.ftl`, and any other screen folder named in an approved T4 item |
| `frontend-reports` | `frontend/src/features/reports/**`, `frontend/src/shared/ui/HoverDetails.tsx` (new) and its test, `frontend/src/i18n/ur/reports.ftl`, `frontend/src/i18n/en/reports.ftl` |

- `frontend-reports` needs `HoverDetails` exported from `shared/ui/index.ts`. `frontend` adds that one export line when `frontend-reports` asks. Nobody else edits `index.ts`.
- If `frontend` changes a shared component the reports page uses (for example `Card`), it tells `frontend-reports` before, so the guide card is checked on top of it.

## 6. Design rules for every item

- Keep the overall design: the colours, the layout grid, the type scale and the places of things. Refine, do not redesign.
- Every value from `styles/tokens.css`. A new token goes in `tokens.css` first (F9). No raw colours or pixel values in components. No arbitrary Tailwind values.
- Logical properties only (`ms-`, `pe-`, `start`, `end`). Every change is checked in Urdu (right-to-left) and English (left-to-right).
- Urdu text at least 18 px, Latin at least 14 px in grids and 16 px elsewhere (NFR-USE-20). Nastaliq line height at least 1.8. Contrast at least 4.5 : 1.
- Motion: at most 200 ms, no flashing, and nothing moves when the system asks for reduced motion.
- Focus outline always visible. Click targets at least 32 by 32.
- No text in code. New words go in Fluent, in both languages, glossary words only.
- Nothing scrolls sideways from 1024 by 600 to 2560 by 1440.
- Entry grids, keyboard behaviour (PRD 7.2, 7.3) and the print templates are out of this plan.

## 7. What each part must do

### 7.1 Login brand panel (T2)

Today (Umar's screenshot): "منڈی سافٹ ویئر" breaks after "سافٹ", the firm and "نئی سبزی منڈی کوٹ رادھا کشن" wrap into ragged lines, and two thirds of the panel is empty.

- The app name stays on one line at every supported window size (smaller step of the type scale if needed, never a broken word).
- A clear hierarchy: a small mark or monogram (drawn with tokens, no new image file unless it is an SVG in the repository), the app name, then the firm name, then the mandi name as a quieter line. Lines wrap only between words and look balanced.
- The empty space is used with restraint: for example a soft gradient or a quiet pattern made with tokens, and one or two short lines about the app (works offline, books always balance, daily backup) from Fluent. Nothing that looks like an advert.
- The version stays at the bottom, quieter, with `<Num>`.
- Both languages: in English the panel mirrors.

### 7.2 Log out in the sidebar (T3)

- A "لاگ آؤٹ" / "Log out" button at the bottom of the main sidebar (`MainNav`), in the empty dark space shown in Umar's screenshot. Same icon as today (Lucide `LogOut`, mirrored in RTL as a direction icon, F26).
- It looks like it belongs to the sidebar: the sidebar's own item style, separated from the menu above by a thin line, not a bright button.
- It does exactly what log out does today, through the same function. If there is unsaved work, it asks first as today (FR-7.9).
- It is reachable with the keyboard like the other sidebar items (PRD FR-7.17). If the sidebar can collapse to icons, it shows the icon with its name on hover.
- The user menu keeps text size and change password. Log out leaves it, so there is one place for it (NFR-USE-10).

### 7.3 Report guide card (T6, T7)

- Hover on a report card, or move to it with the keyboard (focus), and after a short wait (about 300 ms on hover, at once on focus) a larger card opens beside it. Moving away closes it. `Esc` closes it. It never hides the card it belongs to and never goes off screen; it flips side near the edge, in both directions.
- It holds, from the catalogue and Fluent:
  1. The report's name with its ID (for example R5) and its group.
  2. What it shows, in two or three short lines (a new `report-rNN-detail` message per report, both languages, plain mandi words; the existing one-line `-help` stays on the small card).
  3. Its columns, from `column_title_ids`, as small chips or a short list.
  4. Its filters, from `filters`, with "party is required" when `party_required` is true.
  5. What you can do with it: open on screen, print, save as PDF, Excel (FR-19.3), and the special print where there is one (R1 the seller bill, R5 Print All, R35 one party's slip or the all-parties table).
  6. When to use it: one line, for example "for the collector's round, print R35 for the day".
  7. A button "رپورٹ کھولیں" / "Open report" that opens it, same as clicking the card.
- The small cards themselves get a modern touch at the same time (clearer hover state, the report ID as a quiet badge, the icon less heavy), and the page gets a short line at the top that says hover a card to see what it holds.
- Mouse and keyboard both work. On a touch screen a long press opens it.
- Opening the reports page stays under 300 ms; the guide card opens with no backend call (everything is already in the catalogue).

## 7.4 Approved design pass items (T4, lead, 13-11-2026)

All nine items of `frontend`'s audit are approved, with the conditions below. Screenshots: `frontend/tests/e2e/output/screenshots/plan75/audit/`.

1. Dashboard tiles: icon chip, calmer label, larger figure, hover tint. Colours and meanings unchanged.
2. Main sidebar top: the login mark plus the name; softer active pill.
3. Area side menu: small icon per item, lighter active state, no wrapping.
4. Status words as quiet chips, the word always kept, with an icon (never colour only).
5. Filter areas in a quiet toolbar card (Past Sales, analytics).
6. Long forms in titled cards, name pairs side by side from 640 px. Condition: the keyboard order (Enter, arrows) follows the new visual reading order in both directions, and every existing keyboard test of those forms still passes.
7. Summary figures as a strip of stat cards (cash book and similar).
8. Read-only tables: lighter header, row hover, softer total row, zero amounts in muted ink. Number cells, headings and totals stay centred (NFR-USE-19). Entry grids untouched.
9. Top bar controls: one height and radius, ghost bell, avatar chip.

Word: "Log out" in English everywhere (the sidebar item and the lock box), لاگ آؤٹ in Urdu, one spelling (NFR-URD-7, NFR-URD-21).

Ownership for these items: `frontend` may edit `features/dashboard`, `features/bikri` (the Past Sales files only), `features/cashbook`, `features/analytics` and `features/settings`. `features/reports/**` stays with `frontend-reports`. A change to `shared/grid` or `shared/ui/Table.tsx` reaches the report tables, so `frontend` tells `frontend-reports` before it lands.

## 7.5 Guide card decisions (lead, after T7)

- Part order: name, what it shows, what you can do (prints), when to use it, columns, filters, Open button. Prints come before columns so "where do I print this" stays in view; in Urdu at 1366 by 768 the card can be taller than the window, so it scrolls inside itself with the Open button fixed at its bottom.
- The card stays inside the main area (it never covers the menus, top bar or status bar): about 370 px wide in the middle column at 1366, up to 520 px elsewhere.
- It opens at once only when a key moved the focus (arrows, Tab). The page's first-card autofocus and "go to group" focus quietly (`focusQuietly`) and open nothing.
- A test checks that the column chips of every report equal its `column_title_ids`, in order.

## 7.6 Part 2: the same modern touch in every section, and a new app logo (Umar, 13-11-2026)

Umar was pleased with the hover, the icons and the cards, and asked for the same throughout the app (انڈکس and every other section), and for a professional app logo, relevant to the shop, in place of the green square with a cream circle shown in the Mac dock.

### Design patterns that may be applied without asking

These are the patterns Umar has now seen and approved. A builder applies them to every section it owns:

- Tiles and summary figures as stat cards with a soft icon chip (the T5 item 1 and 7 look).
- Status words as chips, word always kept, with an icon.
- Filters in a quiet filter card (`FilterCard`), ticks in one row.
- Read-only tables in the lighter list theme (header, row hover, soft totals, muted zeros).
- Area menus and page headers with a small Lucide icon.
- Cards with a clear hover and focus state; list rows that open something show it on hover.
- Empty states with a soft icon and one helpful line and button (FR-7.46).
- Long forms grouped in titled cards, name pairs side by side.
- A guide card on hover or focus (`HoverDetails`) only where it explains a choice the Owner must make, for example the type list of a shared entry screen or a master list's add row. Not on every button.

Anything else (a new layout, a new component pattern, a colour) is proposed to the lead first.

### Hard rules (as Section 6, plus)

- Never hide a figure or a column that is visible today at the same window size (NFR-USE-27).
- Never change Enter or arrow order, field order or which cells are editable (PRD 7.2, 7.3).
- Entry grids and print templates keep their look.
- Shared components (`shared/**`) are edited only by `frontend`. `frontend-reports` asks `frontend` for a shared change.
- Test load: run only the tests of the files you touched while working. The lead runs the full suites once at the end.

### Sections and owners

| Teammate | Sections (`frontend/src/features/...`) |
|---|---|
| `frontend` | `masters` (انڈکس: accounts, items, packings, cities, اڈے, marks, stores, narrations, expense heads, staff, chart of accounts), `rules`, `settings` (every tab), `backup`, `restore`, `audit`, `day`, `month`, `partners`, `party_ledgers`, `dashboard`, `auth` (wizard), `assistant` (look only, no logic), `bikri` (screens other than the entry grid and prints), plus all of `shared/**`, `app/**`, `styles/**` |
| `frontend-reports` | `cashbook` (روکڑ بک, خرچہ, ادھار رقم, الگ کھاتہ, جنرل), `vouchers`, `fruit`, `gardens`, `trips`, `purchases`, `analytics`, `reports`, and their Fluent files |

### The app logo

- Concept: a professional mark for a fruit and vegetable commission shop: for example a mandi crate (پیٹی) with a fruit and a leaf, or a balance scale with produce, on the app's deep navy with a warm gold or fresh green accent. Simple, readable at 16 px, no text inside (the same icon serves both languages), no clip art.
- `frontend-reports` draws three options as SVG in `frontend/brand/` (new folder, theirs), renders each at 1024, 128 and 32 px, and on a light and a dark dock background, and sends the lead the renders. The lead picks one.
- The chosen mark: a 1024 by 1024 PNG with the Mac rounded-square shape and safe margin, plus the SVG source, both in `frontend/brand/`.
- `backend` generates every app icon from it with `pnpm tauri icon frontend/brand/app-icon-1024.png` into `backend/app/icons/` (icns, ico, png sizes in `tauri.conf.json`). It never draws the icon by hand.
- `frontend` changes `BrandMark` to the same mark (inline SVG, from tokens), so the sidebar, the login panel and the dock show one logo.
- The dock shows the new icon after the app is restarted.

## 7.7 Part 3: پچھلی بکری days closed by default (Umar, 13-11-2026, PRD FR-12.46 v1.39)

Confirmed with Umar before building. Owner: `frontend` (the Past Sales files).

- Today's section stays fully open.
- Each day of the 7-day window is one closed line: a small arrow (chevron, mirrored in RTL), the date, the بکری count pill ("بکری 3") and the خام بکری pill. The count, not the range of numbers (Umar).
- The arrow, or `Enter` / `Space` on the line, opens the day under its line with its بکری rows exactly as today, and closes it. The arrow turns down when open. Several days can be open.
- A day with no بکری: one quiet line "کوئی بکری نہیں", no arrow.
- The میزان row of the window stays.
- Moving the window (← →, Page Up, Page Down): new days start closed.
- ↑ ↓ move between the lines. ← → still move the window (FR-12.46).
- تاریخ پر جائیں: that day opens and is highlighted.
- A search or filter: days with a match open by themselves.
- پوری بکری دکھائیں: opens every day; unticking closes them.
- After an edit: the edited day is open and its row selected.
- Print and Excel: every بکری of the window, whatever is open on screen.
- Open state lives in the screen's state, not inside a grid, so a language switch keeps it (F39).
- Modern look: soft card line per day, hover highlight, open and close within 200 ms with reduced motion respected, numbers centred (NFR-USE-19), both directions, no sideways scroll.
- Speed: opening a day draws rows already loaded for the window; no extra backend call unless the window's page did not hold them (then one indexed call, under 300 ms, NFR-SPD-24).
- Tests: closed by default; arrow, Enter and Space open and close; empty day has no arrow; window move closes; go-to-date opens; search opens matching days; tick opens all; edit return keeps the day open; language switch keeps open days; screenshots ur and en at 1366 and 1024, closed and one day open.

## 7.8 Fix: sign-in shows "کچھ غلط ہو گیا" with the right password (Umar, 14-11-2026)

Cause, from the log (E-20261114-0001 to 0003, and 32 times since 26-10-2026): `audit key store: Platform failure: The user name or passphrase you entered is not correct.` Sign-in writes an audit entry (FR-8.6), which reads the audit chain key from the Mac Keychain (NFR-SEC-6). Every dev rebuild gives the app a new ad-hoc signature, so the Keychain asks again for the Mac password; a dismissed, hidden or mistyped Keychain box comes back as this error, and the screen shows the generic error. Windows Credential Manager does not ask like this, so it is a Mac phase problem, but it must not show again.

Fix (owner `backend`, Fluent text by `frontend`):
1. Read the audit chain key from the key store once per app run and keep it in process memory for the rest of the run (never on disk, never logged, cleared on exit). One Keychain approval then covers the whole run, not every sign-in.
2. A Keychain refusal is its own error code `ERR_KEYSTORE_DENIED` (not `ERR_INTERNAL`), shown on the sign-in screen in both languages with what to do: "Mac Keychain did not allow the app. Enter your Mac password in the Keychain box and choose Always Allow, then sign in again." Nothing is saved and the typed user name stays.
3. Dev builds keep one stable code signature, so "Always Allow" stays valid across rebuilds: a cargo runner signs the debug binary with a local "Mandi Dev" identity when it exists (plain run when it does not), and a one-time script `scripts/dev/make-dev-signing-identity.sh` makes that identity (Umar runs it once, with his Mac password).
4. Test: a key store that refuses gives `ERR_KEYSTORE_DENIED`, and a second sign-in in the same run reads no key from the store.

## 8. Acceptance checklist

- [ ] T1: `column_title_ids` on every catalogue entry; each ID found in both Fluent files (test); `cargo test -p mandi-reports` green; bindings regenerated, not hand-edited.
- [ ] Login panel: app name on one line at 1024, 1366 and 1920 wide, in both languages; screenshots before and after.
- [ ] Log out at the bottom of the sidebar, keyboard reachable, asks on unsaved work, gone from the user menu; component test.
- [ ] Every approved T4 item built, with before and after screenshots in both languages.
- [ ] Guide card opens on hover and on focus, closes on leave and `Esc`, flips at the edges, never scrolls sideways; shows the seven parts above for every report; component tests for hover, focus, `Esc` and an RTL and LTR snapshot.
- [ ] `pnpm lint`, `pnpm typecheck`, `pnpm i18n:check`, frontend tests and the screenshot tests green.
- [ ] No golden test, money code or print template changed.
- [ ] Code Standards Section 11 checklist.

## 9. Out of scope

- Entry grids, keyboard rules, the print templates, the dashboard figures, the assistant panel's logic.
- Any new colour meaning (green, red and amber keep theirs, NFR-USE-12).
- A redesign of the whole app.

## 10. Open questions for Umar

None blocking. Lead decisions (written here, easy to change):

- D1: log out is moved, not copied: it leaves the user menu so there is one place for it.
- D2: the guide card opens on keyboard focus as well as hover, so the keyboard-only path works (NFR-USE-7).
- D3: reviewer and robustness run only after Umar has looked at the result and says continue.

## 11. Follow-ups (not in this run unless time allows)

1. Optional `icon` on Card, ListPage and Fact (a soft chip before the title or label).
2. A guide card on the type buttons of the خرچہ, ادھار رقم and الگ کھاتہ screens (SegmentedControl option details).
3. `Balance inherit`, so ReportSummary and the balance now and after of entries can be stat cards.
4. The NumberRuns first and last run gap (4 px on a cell that starts with a code): the cause of the Urdu R2 and R6 alignment findings on report-r3, r5, r33, r34, r35 and analytics-c, h, j. With `frontend` after Past Sales and BrandMark if time allows.
5. report-r13-party R7: a ledger line wraps on two lines by design; the checker should allow it.
6. The purchase detail (23 plain facts) as stat cards.
7. Titled form cards and side-by-side name pairs on the روکڑ forms, only with an unchanged keyboard order.
8. FilterCard on more lists (vouchers list, the cash book's other lists).
9. Print alignment findings on the r34 and r37 prints (prints were out of scope).
10. A test helper for old-data analytics rows, so a test does not drop the `analytics_facts_no_update` trigger in its temporary database (analytics adda test, plan 75 follow-up fix).
11. A stat strip on the partners overview and the day close cards.
12. Empty states with a soft icon on the master lists, and a HoverDetails guide on the masters add row.
13. Titled-card grouping of the rules form and the setup wizard.
14. Use the new optional props (Card, ListPage and Fact `icon`, SegmentedControl `details`, Balance `inherit`) on the روکڑ, voucher and report screens.
15. The fixture copy script covers only the services fixtures; extend it to backend/crates/backup/tests/fixtures and backend/app/tests/fixtures.
