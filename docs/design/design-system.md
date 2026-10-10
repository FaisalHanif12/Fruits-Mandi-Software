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
