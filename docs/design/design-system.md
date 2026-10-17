# Design system: "Calm ledger"

Version 1.0, 10-10-2026. Kept by `tech-lead`. Built by `frontend` into `frontend/src/styles/tokens.css` and `frontend/src/shared/ui/`.

**Design canvas:** https://claude.ai/artifact/EtYUy8QMjbEkx7yy9aeAiF (15 boards). The source of every board is in `docs/design/canvas/*.dc.html`, so it can be read without the link. Open a board file to see exact spacing and markup. The boards are drawings, not code: build them with our shared components and tokens.

| Board file | Screen |
|---|---|
| `Main.dc.html` | Foundations: colours, type, spacing, radius, fields, focus, buttons, number alignment |
| `LoginUr` / `LoginEn` | Login, with the lockout message (Urdu) and a wrong-password field error (English) |
| `WizardUr` / `WizardEn` | First-run wizard: recovery code step (Urdu), firm details step (English) |
| `ShellUr` / `ShellEn` | App shell and the empty dashboard |
| `UsersUr` / `UsersEn` | Settings: users list, add-user dialog (Urdu), success toast (English) |
| `FormUr` / `FormEn` | Sample entry form (پیشگی) with a field error, party card, and the "save changes?" prompt |
| `GridUr` / `GridEn` | Sample entry grid (بکری sale lines) with the live summary |
| `StatesUr` | The four states: loading, empty, error, data |
| `CompactUr` | The compact layout at 1024 x 600 |

The rules in CLAUDE.md, the NFR and Code Standards win over any drawing. If a board breaks a rule, follow the rule and tell the lead, who records it here.

## 1. Idea

A ledger on good paper. Warm off-white ground, white cards, navy ink for every action, and colour used only for meaning. Calm and dense enough for a munshi typing all day. Large, clear numbers for older owners.

## 2. Colour tokens (`--color-*`)

Light theme only. Every value below is the only place that colour may come from (Code Standards F9).

| Token | Value | Use | Contrast |
|---|---|---|---|
| `--color-bg` | `#F6F4EF` | Page ground | |
| `--color-surface` | `#FFFFFF` | Cards, fields, dialogs, grid body | |
| `--color-surface-sunken` | `#EFECE5` | Grid header, totals row, side menu of a module, status bar, key chips | |
| `--color-surface-readonly` | `#F8F7F3` | Calculated (read-only) grid cells | |
| `--color-row-current` | `#F1F5FB` | The row that has focus in a grid | |
| `--color-ink` | `#1C1F23` | Text | 15.1:1 on bg, 16.5:1 on surface |
| `--color-ink-muted` | `#565C64` | Hints, secondary text, header labels | 6.1:1 on bg, 5.7:1 on sunken |
| `--color-ink-placeholder` | `#6B6559` | Placeholder text, including the rate hint | 5.3:1 on row-current, 5.8:1 on surface |
| `--color-border` | `#D9D4C9` | Card and divider lines (decoration only) | |
| `--color-border-control` | `#8C8577` | Field, select and secondary-button borders | 3.7:1 on surface (non-text 3:1) |
| `--color-primary` | `#1D3D6E` | Primary button, selected tab and menu item, links | white on it 10.8:1 |
| `--color-primary-chip` | `#2E5288` | Key chip on a primary button | |
| `--color-primary-soft` | `#E6ECF5` | Selected row, selected tab ground, empty-state icon ground | primary on it 9.1:1 |
| `--color-nav` | `#16263F` | Main menu rail, toast ground, login brand panel | |
| `--color-nav-ink` | `#D6DDE8` | Menu text | 11.1:1 on nav |
| `--color-nav-active` | `#24406B` | Current menu item | white 10.4:1 |
| `--color-focus` | `#2563C9` | Focus outline only | 5.7:1 on surface |
| `--color-money-in` | `#17703F` | Money in, balanced, success, "day open" | 6.1:1 on surface |
| `--color-money-in-soft` | `#E5F3EA` | Success ground | 5.4:1 |
| `--color-due` | `#B3261E` | Due, owes, error | 6.5:1 on surface |
| `--color-due-soft` | `#FBEAE8` | Error field ground | 5.6:1 |
| `--color-warn` | `#8A5300` | Warning text and icon | 5.7:1 on warn-soft |
| `--color-warn-soft` | `#FDF1DC` | Warning ground | |
| `--color-warn-border` | `#E9C98F` | Warning box border | |

Rules:

- One meaning each (NFR-USE-12): green is money in or balanced, red is due or error, amber is warning. Colour is never the only signal: an icon or a word always goes with it. Example: a buyer balance shows "2,880 بنام" in red, with the word.
- A seller payable (جمع) is shown in ink, not green. It is money the firm owes, not money in.
- Focus blue is never used for anything else.

## 3. Type

Fonts are bundled (NFR-URD-9, NFR-URD-27), loaded with `font-display: block`.

| Family token | Fonts | Use |
|---|---|---|
| `--font-urdu` | Gulzar, then Noto Nastaliq Urdu | All Urdu words: labels, headings, menus, prints |
| `--font-urdu-dense` | Noto Naskh Arabic | Optional dense grids (user setting); Urdu digits on prints |
| `--font-latin` | Inter | English, all numbers, codes, phones, dates |

Five sizes per script (Code Standards F11):

| Step | Urdu (Nastaliq) | Latin | Use |
|---|---|---|---|
| `--text-1` | 18 px | 14 px | Grid cells only for Latin (NFR-USE-20); Urdu body, labels, hints |
| `--text-2` | 20 px | 16 px | Field text, card titles |
| `--text-3` | 24 px | 18 px | Section titles, dialog titles |
| `--text-4` | 28 px | 22 px | Page titles |
| `--text-5` | 34 px | 28 px | Brand, big totals |

- Nastaliq line height 1.9 (NFR-URD-11 asks at least 1.8). Latin line height 1.45.
- Every number uses `font-variant-numeric: tabular-nums` and the Latin font (NFR-USE-19). Number cells are `direction: ltr; text-align: end`, so they sit on the physical right edge in both languages. Headers of number columns line up with them.
- English labels on forms are 14 px semibold; the numbers in fields are 16 px.

## 4. Spacing, radius, size

- Spacing scale (F10): `--space-1` 4, `--space-2` 8, `--space-3` 12, `--space-4` 16, `--space-5` 24, `--space-6` 32, `--space-7` 48 px.
- Radius: `--radius-1` 4 px (fields, grid cells, key chips), `--radius-2` 8 px (buttons, cards, menu items, toasts), `--radius-3` 12 px (dialogs, page cards).
- Control height: `--control-h` 44 px for fields and buttons in both languages (Nastaliq at 18 px needs it). Compact toolbar buttons 40 px. Click targets are never under 32 x 32 (NFR-USE-23).
- Grid row height 44 px, header 44 px (Nastaliq 18 px x 1.9 fits).
- Layout steps (F22): compact 1024 to 1365 px wide (main menu becomes a 64 px icon rail with labels in tooltips and `aria-label`), normal 1366 to 1919 (224 px menu with labels), wide 1920 and up (content max width 1600 px, centred).

## 5. Focus

- Every focusable element: `outline: 3px solid var(--color-focus); outline-offset: 2px` (NFR-USE-22). Never removed.
- Grid cell with focus: `outline: 2px solid var(--color-focus); outline-offset: -2px`, and its row gets `--color-row-current`.
- A selected list row: `--color-primary-soft` ground plus the focus outline.

## 6. Components (in `shared/ui`)

| Component | States | Notes from the boards |
|---|---|---|
| `Button` | primary, secondary, danger (outline red), disabled, busy | Shows its key in a chip after the label (FR-7.4): "محفوظ کریں F10". One primary per screen, at the end of the action row. |
| `KeyChip` | on primary, on light | 22 px tall, Inter 12 px semibold |
| `Field` (label above, hint below) | normal, focus, error, read-only | Error: red border, red-soft ground, icon and message under it (FR-7.6). Read-only: `--color-bg` ground. |
| `MoneyInput`, `QtyInput`, `DateInput`, `CodeInput` | as Field | Latin font, tabular, text aligned to the physical right. The value is text; the backend parses it. |
| `Select` | as Field | |
| `LanguageSwitch` | ur active, en active | Segmented "اردو / English", 40 px, `aria-pressed`. `Alt+L`. |
| `MainNav` | full (224 px), rail (64 px) | Start side. Current item `--color-nav-active`. |
| `ModuleMenu` (بکری side menu) | | Sunken ground, 200 px, current item primary. |
| `TopBar` | | Search box with `Ctrl+K` chip, attention bell, language switch, user menu. |
| `StatusBar` | | 32 px: user, date, day status (green with a tick icon and the word), last backup with a dot and words. |
| `Toast` | success, error | Nav-coloured ground, white text, icon, at the bottom end corner, disappears by itself (FR-7.7). |
| `ConfirmDialog`, `SaveChangesPrompt` | | 12 px radius, scrim `rgba(22,38,63,.35)`. The save prompt has three buttons: back (Esc), leave without saving (danger outline), save (primary, focused) (FR-7.9). |
| `EmptyState` | | Round soft icon, one-line title, one-line help, one button with its key (FR-7.46). |
| `ErrorState` | | Red-soft icon, plain words, reference number in Latin, retry (NFR-REL-6, NFR-OBS-3). |
| `LoadingState` | | Skeleton rows shown only after 300 ms, so fast screens never flash. |
| `Num`, `Code` | | `<bdi>` isolation, tabular digits (F25). |
| `Stepper` | done, current, next | Done steps green with a tick, current step primary. |
| `Card` | | Surface, 1 px border, 12 px radius, 24 px padding. No coloured side borders. |

## 7. Motion

Only colour and opacity transitions, at most 150 ms. No animation over 200 ms, no flashing (NFR-USE-24). Respect `prefers-reduced-motion` by turning transitions off.

## 8. Changes from the boards

Where a board breaks a rule, the rule wins (recorded 10-10-2026, plan 01 review):

1. NFR-USE-20: Latin text is at least 16 px outside grids. English running text, labels and button labels use 16 px (`--text-body`), not the 14 or 15 px drawn on some English boards. 14 px Latin is used only inside grids. Urdu stays 18 px as drawn.
2. Key chips use Inter 16 px semibold, not 12 px, for the same reason. The chip grows to fit; buttons stay 44 px tall.
3. Extra token `--color-on-primary` (`#FFFFFF`): text and icons on the primary colour.
4. Urdu text inside fields is 18 px (`--text-field`), not 20 px. At 20 px, Gulzar touches the top border of a 44 px field. Latin field text stays 16 px. (Plan 02.)
5. The gap between label, field and hint is 8 px, not 4 px, so Nastaliq hints never touch the field border. (Plan 02.)
6. Urdu hints are 18 px, not the 16 px drawn on `StatesUr` (NFR-USE-20). (Plan 02.)
7. Added tokens (plan 02): `--color-scrim`, `--color-skeleton`, `--color-money-in-on-nav` `#5FBF8A` (6.7:1 on nav), `--color-due-on-nav` `#F2A29B` (7.5:1 on nav, error toast), `--shadow-overlay`, icon sizes 16 / 20 / 26, badge 56, `--spacing-0`, layout steps `normal:` 1366 and `wide:` 1920, `--page-max-w` 1600, `--dialog-w` 520, `--toast-max-w` 480, `--motion-fast` 150 ms.
8. The page sets `translate="no"`, so the browser translator never rewrites Urdu text (found in plan 02).
9. New shared pieces (plan 11, partners): `shared/ui/PercentInput` (percent text with up to 2 decimals, the "%" sign comes from Fluent) and the `Field` prop `hideLabel` (the label stays for screen readers). The share total box uses `--color-money-in` at 100 and `--color-due` otherwise, always with an icon and words. No token or colour was added. (Plan 11, 16-10-2026.)
10. The sale book (plan 12, `/bikri/book`, 16-10-2026). The screen has to be one screen at 1024 × 600, where the main menu and the side menu leave about 680 px. These are the choices that make it fit. No colour was added.
    - **The page heading is hidden at the compact step.** The side menu already says where the user is, so the `h1` is `sr-only` below 1366 px and shown from 1366 px (`sr-only normal:not-sr-only normal:type-title`). Screen readers always hear it. The help line is shown from 1366 px only.
    - **`Card` has a `compact` prop** (12 px padding and gap, instead of 20 and 16) for a card that sits above a grid. The header of the sale book uses it.
    - **The grid has a `fit` mode.** The columns share the free width of the grid by a weight per column type (`FIT_WEIGHTS`) down to a least width each (`FIT_MIN`: long name 150, name 100, long heading 96, heading 84, figure 72), so the grid never scrolls sideways at 1024 px. A header may wrap to two lines (`wrapHeaderText`, `autoHeaderHeight`). The sale book lots grid has 8 cells at 1024 px and adds the net and the balance from 1366 px. The sale lines grid has buyer, qty, rate, seller amount, buyer total and the payment type at 1024 px, adds the two commissions from 1366 px and the لاگا from 1920 px.
    - **Two new grid column types.** `picker` is a cell that opens the shared picker in a popup (280 px wide, token `--picker-cell-w`, `--spacing-picker-cell`) attached to the page body, so the grid cannot clip it. `choice` shows one word of a short list (ادھاری / نقدی); a click or Space moves to the next word, and it is not an Enter stop.
    - **A calculated cell has its ⓘ inside the cell**, a 32 × 32 target (`size-target`) that is out of the Tab order. `Ctrl+I` on the cell opens the same working. A rate cell with no typed value shows the last rate as grey text (`cell-hint`); an amber cell (`cell-warn`) is a warning and a red one (`cell-error`) is an error, and both are written out in the problem list as well (NFR-USE-12).
    - **The focus ring goes when focus leaves a grid**, so no cell looks active while the user types in a form field. It comes back with the next cell chosen.
    - **The party card follows the cursor.** At the wide step (1920 px and over) it is in a right column with the summary and the partner split. In the other steps it sits under the sale lines grid and the numbers sit under it, so the grids keep the whole width.
    - **The commission and لاگا of a line are in the line details strip**, not grid cells: three fields (seller commission, buyer commission, لاگا), each with a method toggle (% or per نگ), the rule tag and the amount with its ⓘ. At 1024 px there is no room for six more grid columns. The strip shows the line that has focus. When the Owner switches line overrides off the fields are read-only and a note says why.
    - **The rest of a lot is behind a "more" toggle** under the lots grid (مارکہ, store, weight, فون, کرایہ سٹور, who paid the freight, the advance tick, the meaning of the typed charges, and the one-time seller). It opens by itself for a one-time seller or for a problem on one of its fields.
11. New shared pieces (plan 12): `shared/ui/SuggestInput` (a text field that suggests what was used before, such as a truck number; it takes any text, Enter takes what is typed unless an arrow moved onto a suggestion) and `shared/ui/CashConfirmDialog` (PRD FR-6.7: "Yes, save" and "No, go back"; focus starts on No, Esc is No, and the screen may take focus back to the amount field). `CalcInfoAt` shows a working next to a grid cell; `PickerBase` and `AccountPicker` take `initialText`, `startOpen` and `onEnterPicked` for cells. New token: `--picker-cell-w` 280 px. (Plan 12, 16-10-2026.)
12. The sale book against the real backend (plan 12, T4, 16-10-2026). Three behaviours changed after the backend's answers were real.
    - **A blank sale line already starts at what the lot has left.** The backend's E2E-26 file sends the blank second line with qty "30". The fill happens at once, before a buyer is chosen, and is an Enter stop like any filled value. A line with no buyer and no rate is still blank, so the preview and the save ignore it.
    - **An editable cell can show a value the backend worked out, with its ⓘ.** The نقد cell of a one-time seller (PRD FR-13.10) shows his whole net while it is empty. It looks like a typed number, has the ⓘ with the backend's working, and a value typed there is kept as typed. `Ctrl+I` works on any cell marked `explain`, not only on calculated cells. The grid column flags are `showsCalc` and `explain`.
    - **The problem "shares do not total 100%" has a link under it** ("Partner setup", the shared `LinkButton`) that opens `/partners/setup`, because the shares are fixed on another screen.
13. The whole-sale view and the numbers strip (plan 12, 16-10-2026, decisions of the lead).
    - **The "پوری بکری" view of the lines.** A switch on the heading of the sale lines ("Lot 2 | Whole sale", the same `SegmentedControl` as the summary's) shows the lines of all lots in one grid with a lot column first (its number and the seller, for example "2 Rahim"). The grid is editable like the grid of one lot. The two switches are one: pressing either changes both, the lines and the summary. Nothing may scroll sideways, not even inside a grid, so the whole grid keeps only the columns that fit beside the lot column: in compact windows buyer, qty, rate, seller amount and buyer total (the payment type is left out); in normal windows the columns of the compact step with the payment type; in wide windows the columns of the normal step (the commissions) and not the لاگا. The line details and the summary hold what is left out. The lot column is a `label` cell of 100 px at least (72 px in wide windows); a long seller name is cut and read in full in the tooltip. The least width of the payment column (`choice`) went from 104 to 84 px (its heading wraps), which the one-lot grid needed at 1920 px with a classic scrollbar. `tests/unit/bikri-column-widths.test.ts` keeps the sum of the least widths of every grid inside the room it has at each step, with a 17 px scrollbar. Enter on the last stop of a lot's last line goes on to the first line of the next lot (a new line for a lot is made in its own view or with `↓` on the last line).
    - **No second copy of the data.** The lines stay in the stores of the lots. `stores.all` only lists their rows in lot order and hands every change (typed text, a picked record, a patch, a removal, focus) to the store of the lot that owns the row, and focusing a line selects its lot, so the line details, the party card and the problems follow the cursor in both views. The cursor stays on the same cell when the view changes.
    - **The numbers strip.** In compact and normal windows (not in wide, where the summary has its own column) a strip is pinned to the bottom of the scrolled area with two rows: the selected lot and the whole sale, each with خام بکری, کل اخراجات, صافی بکری and the buyers' bills (net in red when below zero), and the cash note when cash would go below zero. The numbers are the backend's, with no arithmetic. It holds no button, link or field, so it is not a tab stop and the arrow keys pass over it; it is a named region, not a live region, so a screen reader does not read it out on each key. It uses the card look (border, surface, overlay shadow) and sits 8 px above the status bar. A lot that cannot be worked out yet says so in words. At 1024 px the strip is about 90 px tall in Urdu (one line for each group) and 115 px in English (the last number of each group wraps to a second line), a line more with the cash note. Token added: `--strip-clearance` 160 px, the room `main` keeps under a focused cell (`scroll-pb-strip`), enough for the tallest case.
14. Saving, drafts and the saved بکری (plan 13, 16-10-2026). No colour or token was added.
    - **One toolbar above the sale book, new or saved** (`BikriToolbar`): first, previous, next and last (icon buttons with their words in `sr-only`; the arrows that point a way are mirrored in Urdu with `rtl:rotate-180`), `Find F3`, and at the end the action of the screen: the primary `Save F10` on a new بکری, the primary `New Alt+B` on a saved one. The truck is still the first field the cursor is in. A short note under the toolbar says "there is no next بکری" and goes by itself after 4 seconds. The last saved بکری has a "last saved: number 5, open" line in the toolbar, so the way back to it is always on screen.
    - **A saved بکری has the layout of the sale book, without grids.** Header facts (`Fact`, the new shared piece: a label above and a value, no box), a card for each lot with its sale lines in the read-only `Table`, one summary card (the lot or the whole, the same two columns as the live summary and no ⓘ because nothing is worked out), the partners' share, and the entries it made in the books. A cancelled بکری has a red banner with a ban icon and the words. No input, no picker and no grid exists on this screen, so nothing can be typed or scrolled sideways.
    - **The start-up offer "Finish the last بکری?"** is the three-button `SaveChangesPrompt` with its own words (`words` prop): restore (primary, focused), discard (danger) and "not now" (Esc). The cash box, the leave prompt and this offer are the three Radix alert dialogs of the screen; none stacks on another.
15. The list screens beside the بکری side menu (plan 13). The room of a list there is the content area minus the 200 px side menu: 692 px at the narrowest compact window, 874 at the narrowest normal, 1332 at the narrowest wide (`tests/unit/list-widths.test.ts`). New list column type `datetime` (170 px, DD-MM-YYYY HH:MM, on the physical right edge). The list of بکری has number, sellers, خام and صافی and status at 1024 px; adds day and time at 1366 px; adds truck, items and buyer total at 1920 px. The نگ column of PRD FR-12.7 and FR-13.6 comes with a "More columns" tick (`common-more-columns`, kept per computer), as on the account list: it is added at every step and one less needed column steps aside where the room is short (`shared/grid/moreColumns.ts`, plans in `list-columns.ts` and `pukhta_khata/columns.ts`). The phone column of the register is `number` (140 px), because eleven digits do not fit a 120 px code.
16. The one merged list of known one-time people (plan 13, PRD FR-7.29). The list that opens under a name or phone field has two kinds of row: the people (two lines: his name in the app language and his phone, then the other script and the day of his last line) and, in the Urdu name field, the Urdu spellings of the word. The people come first. The first spelling is the highlighted row; a person is highlighted only after an arrow moves onto him, so Enter on a new name moves on and never takes an old person. The list is the same box in `RomanUrduInput` (`extras`) and `SuggestInput` (rows with `content` and `onPick`). A new shared option: `Select` inside `Field` with `hideLabel`, for the lot picker of the saved summary.
17. Prints (plan 14, 16-10-2026). The seller bill, the buyer slips and the پختہ slip are laid out in `frontend/src/shared/print/` (the layout and `print.css`) and `features/bikri/print/` (the three templates). They are paper, not screen, so they have their own scale; the screen type scale (F11) never applies to a printed page. No colour was added.
    - **Print type scale, in points** (tokens `--print-latin-1` to `-5` and `--print-urdu-1` to `-5`; the step in use is `--print-text-1` to `-5`, which follows the `lang` of the sheet like the screen steps): Latin 8, 9, 10, 12, 16 pt; Urdu 10, 11, 12, 15, 20 pt. Body text is step 3 (Inter 10 pt, Gulzar 12 pt), labels step 2, the firm and party names and the title step 4, the 80 mm total is step 4 and the cancelled stamp step 5. Nastaliq keeps its line height of 1.9 (NFR-URD-11). **Numbers are always Inter, so they take the LATIN step of the text around them**, also in an Urdu print (Inter 10 pt beside Gulzar 12 pt look alike). With the Urdu digits setting on, Naskh comes first (`--font-urdu-digits`).
    - **Lengths on paper are millimetres** (`--print-gap-1` to `-5`: 1, 2, 3, 5, 8 mm; `--print-rule` 0.25 mm and `--print-rule-strong` 0.5 mm; sheet widths `--print-w-a4`, `-a5`, `-mm80`). Spacing in pixels is for screens.
    - **One literal `@page` rule for each paper class**: `a4` (A4, margin 14 mm top and bottom, 10 mm sides), `a5` (A5, 10 and 8 mm), `r80` (80 × 297 mm, margin 4 and 3 mm). The backend gives WebView2 the same sizes and margins. The margin box `@bottom-center` holds the page footer, "Printed 10-10-2026 17:40 · Page 1 of 2" (on the roll only "Printed 10-10-2026 17:40", because the roll is one page), from the custom property `--print-footer` set on the root element with the CSS object model (an inline `<style>` is refused by the Content Security Policy). The `page` property is on `html`, never on the document, because a box with another page name forces a page break.
    - **The sheet.** Header (the firm at the start, the title at the end, a heavy rule under it), the facts (number, date, bill date when it differs, truck), a bordered box for who the page is for (role, name, code, phone), the goods table, the amounts at the end side with the amount column as wide as the goods amount column, so the figures stand under the figures (75 mm at most), how the buyer pays in a bordered box (solid = paid, dashed = payment due; a border, not a colour, so a black and white printer loses nothing), and the amount in words. A subtotal has a rule above it and the total a rule above and a double rule under it. A cancelled بکری has the word across the page, red, with a border. Numbers are `<Num>` and `<Code>`: left to right, tabular digits, on the physical right edge of their cell, and the heading of a figure column has the same edge.
    - **A4 and A5:** each party starts on a new sheet (`break-before: page`); the header row of the goods table repeats on every page; no row is cut (`break-inside: avoid`); the amounts, how he pays and the words are kept together. The goods table has date (only when a line is of another day than the header's), buyer (seller bill only), item, packing, qty, weight (only when a line has one), rate, amount.
    - **The 80 mm roll** has no columns: each line is two rows of text, the item and packing with the date on the first, "20 @ 600" (always left to right) and the amount on the second. All slips of "print all" are on ONE tall page with a dashed cut line between them and no page break; the print route reports the height of all of them (`height_px`, the height of the document itself, not `scrollHeight`, which is never less than the window). The hidden window (`data-window`) lays the page out at the width of the text of the paper (A4 190 mm, A5 132 mm, roll 74 mm) with no padding, frame or ground, so that height is the height it prints at.
    - **Paper is white.** In print, `html` and `body` get the white ground and no minimum height. On screen (the in-app preview and the hidden window) a sheet is as wide as its paper, so a line breaks where it will break on paper.
    - **The print box** (`F9`, and a "Print F9" button in the toolbar of the sale book, new or saved): a dialog with two columns (print language and Send to; documents in a row; buyer and copies), the three buttons Close, Preview and Print (primary). It fits a 1024 × 600 window without scrolling. The preview shows the documents in the dialog, in the print language, in the same sheets the hidden window prints; the app keeps its own language. The Printing tab of Settings has the paper of each document in three columns, the printer and the copies, the Urdu digits, and the "print after save" list.
18. The voucher screen (plan 15, 17-10-2026): وصولی, ادائیگی and پیشگی on one screen (PRD FR-7.67). No colour was added.
    - **The type list is a `SegmentedControl` with three choices** above the form (its label is `sr-only`). It can be disabled: a saved voucher that is being edited keeps its type. Changing the type keeps date, amount, mode, bank, cheque and narration and clears the rest. The side menu marks the item of the current type (`moduleMenuOf` reads the address).
    - **The form is one bordered box, the helper panel is under it.** The box is a two-column grid at every step; the account and the one-time person fields take both columns and the party card opens under the account field (not in a side column), so nothing scrolls sideways at 1024 px. The helper panel (`PanelShell`: title, the list, a "not available" line when its query fails) shows the open bills, the payables or the open advances of the chosen account. A row that the amount typed settles carries a mark in words (full, part) and a tick icon, never a colour alone (NFR-USE-12).
    - **`Notice` is the shared note** (amber warn, neutral info, red error; icon, words, and an optional button). The cash note (`CashNote`, moved from the sale book to `shared/ui`), "more than is payable" and the right-type note of FR-7.68 use it. The cash note and the confirm box show the same number, from the backend.
    - **`ReasonDialog`** (new, Radix AlertDialog) asks for a reason before something that is hard to undo: cancelling a voucher. Focus starts in the reason field; an empty reason is told at the field and nothing is sent; the dialog stays open on a refusal and keeps what was typed; `note` shows the cash after a cancel that takes cash below zero, and the next press confirms.
    - **The list under the form** is a `ListGrid` with number, date (from 1366 px), account, amount, mode (from 1920 px) and state. A pinned total row stands under the amount column with the word "Total" (`glossary-meezan`) in the account column; the figure is the backend's `total_paisa` for the saved vouchers of the list (a cancelled voucher is in the rows and not in the total). Its toolbar has Edit, Cancel and Print for the selected row. Past 30 rows it gets its own scroll region of `--list-region-h` 360 px (token `--spacing-list-region`), so a long list never pushes the form out of the window. The list columns are kept inside the room beside the side menu by `tests/unit/list-widths.test.ts`.
    - **The waiting list** (PRD FR-15.1) is a compact `Card` with a `Table` between the form and the helper panel, shown only while it has a voucher: type, account (code and name), amount, balance after, state, and a remove button. The state is "waiting" (muted), the refusal in words (red, `role="alert"`) or "not saved: you went back at the cash question" (amber). The amount and balance come from each voucher's own preview; there is no sum row, because the contract has no backend total for waiting vouchers. F2 and the button "Add to list" put the form in it; the same button reads "New entry" while a saved voucher is changed.
    - **The collection round is an `EntryGrid` with `fixedRows` and `fit`**: one row per buyer from the backend, only the received column is editable, and the least width of that column is `FIT_MIN.longHeading` so its heading fits. A refused row keeps its amount and the problem under it.
    - **The one-time person field is a `SuggestInput` with `highlightFirst`**: the first known person is highlighted when the list opens, so Enter takes him (a typed name that is not taken means nothing here, PRD FR-13.7). "Make regular" uses `QuickAddAccount` with the names and phone of the person to start with (`initial`) and its own saving (`saveWith`).
    - **The receipt slip** is a print document like the others (item 17), on A4, A5 and the 80 mm roll. The slip's "how he paid" box is solid (the money was received), and shows the mode, the bank and the cheque. The voucher number is the Fluent message of the print (`print-fact-voucher-number`), not "Sale No.". The print box and the Printing tab have a fourth paper choice, `paper_receipt_slip`.
    - **Global search** shows a voucher hit with its type word and number. A kind the app does not know shows only its number, so a newer backend never breaks the list.
19. Goods left, later sales, edit, cancel and undo (plan 16, 18-10-2026). No colour was added; one token was used again (`--spacing-3` for the corner mark).
    - **باقی مال is a list with a card under it** (`/bikri/stock`). The list shows the columns that fit beside the 200 px side menu (`STOCK_COLUMNS`): number, seller, item and packing in one cell and the balance at 1024 px; the day received from 1366 px; received, sold, written off and days waiting at 1920 px. Every figure of FR-12.14 is in the card of the lot that has focus (`LotFacts`), with the three actions: sell what is left (Enter on the row does the same), write off, undo a write-off. The total under the balance column is the backend's `balance_total` as text. The tick "also lots that are fully written off" lists a lot with nothing left that has a write-off to undo.
    - **`ReasonDialog` grew four options**: `extra` (fields above the reason, such as the quantity of a write-off or the choice of write-off to undo), `extraFields` (the field paths `extra` shows its own problems for, so a refusal is not told twice), `confirmVariant` (primary for an edit, danger for a cancel) and `confirmDisabled` (a cancel on a closed day), and `initialReason` (the box opened again after the cash question keeps what was typed).
    - **A later sale and the edit of a later entry are one form of sale lines** (`LinesForm`): the lot is read-only in a card (`LotFacts`), only the lines are typed, with the grid, line details, party card, summary, partner split and cash note of the sale book. The lot / whole switch of the grid and of the summary is not drawn (`SalesSection` `single`, `SummaryPanel` without `onWholeChange`). A note in the card says the lot's freight and other charges are not charged again.
    - **Edit mode is the sale book itself** with a different source: the same header, lots, lines and keys. The day field is read-only. The saved lots cannot be removed with Ctrl+Delete (`EntryGrid` prop `canRemoveRow`); a new lot or line can. A changed cell has a soft primary ground and a corner mark on its start side (`cell-changed`, prop `cellChanged`, tooltip "Changed. It was: 600"); the corner is a shape, so colour is never the only signal (NFR-USE-12). Under the lines, the card "What you changed" (`EditChangesCard`) lists each change in a sentence (old text next to new) and, under it, the selected lot's gross, deductions, net and buyer total before (stored) and after (preview), both the backend's. The comparison is made by `edit-changes.ts` on text only.
    - **The reason of an edit**: the save of an entry dated before today opens `ReasonDialog` (primary button) and sends the text as `reason`. An entry dated today saves at once. A refusal `ERR_REASON_REQUIRED` opens the same box.
    - **Closed day**: `ClosedDayNotice` (a red `Notice` with the day and a link to the day screen) shows on an edit and in the cancel box; the save and the cancel buttons are dimmed.
    - **Cancel** is a `ReasonDialog` that names the entries it reverses (kind, day, amount) and shows the cash line (now and after); if cash would go below zero the `CashConfirmDialog` follows the reason, and "No" returns to the reason with its text.
    - **The toast can hold a button** (`showToast` option `action`: label, `altText`, optional `shortcut`; `durationMs`). The key of the button works only while the toast is on screen (the button component registers it). The message after a save of a new بکری uses it for "Undo Alt+Z" for 10 seconds (PRD FR-7.47). Undo asks the cash question first when needed, then opens the sale book on a new entry with the returned values (through the same door as a draft restored at start-up). If another entry has been started, it is not replaced: a message tells him to finish or clear it first.
