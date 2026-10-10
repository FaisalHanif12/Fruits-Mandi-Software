# frontend

What the user sees: React + TypeScript + Vite, shown by Tauri in WebView2 (Tech Stack 3, 5.3).
Urdu (right to left) by default, English (left to right) with `Alt+L`.

The frontend never calculates money. It sends what the user typed as text and shows what the
backend returns (CLAUDE.md rule 1).

## Commands

Run from the repository root (CLAUDE.md, Commands): `pnpm dev`, `pnpm lint`, `pnpm typecheck`,
`pnpm test`, `pnpm i18n:check`, `pnpm gen:types`, `pnpm e2e`.

## Folders

| Path                         | Holds                                                                 |
| ---------------------------- | --------------------------------------------------------------------- |
| `src/app/`                   | Shell, router, providers, layout, and the start screen of plan 01     |
| `src/features/<module>/`     | One folder per business module (Tech Stack 5.2)                       |
| `src/shared/`                | Design system, grid, keyboard, Urdu typing, i18n helpers, format, api |
| `src/i18n/ur`, `src/i18n/en` | Fluent files. `glossary.ftl` holds the locked words of PRD Section 3  |
| `src/styles/tokens.css`      | The only place for colours, spacing, font sizes and radii             |
| `src/bindings/`              | Generated from Rust by `pnpm gen:types`. Never edit by hand           |
| `public/fonts/`              | Bundled fonts with their OFL licences. No remote fonts                |
| `eslint-rules/`              | Local lint rules that block CLAUDE.md rules 1, 7 and 8                |
| `tests/unit/`                | Vitest component and unit tests                                       |
| `tests/e2e/`                 | Windows end-to-end suite (WebdriverIO + tauri-driver)                 |

## Lint rules of our own

| Rule                          | Blocks                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Where                                          | Rule in the docs                     |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- | ------------------------------------ |
| `mandi/no-paisa-arithmetic`   | `+ - * / %`, `+=`, `++`, unary minus, `Math.*` (also with `...spread`), `toFixed`, `toString`, and the conversions `Number()`, `String()`, `parseFloat`, `parseInt`, `as number`, `${amount}` on a value typed `Paisa`; Paisa (or Paisa[]) passed into any parameter not typed Paisa (`new Number(a)`, `JSON.stringify(a)`, `console.log(a)`); a function that would receive Paisa into a non-Paisa parameter (`amounts.map(Number)`, `forEach(console.log)`); `Object.values`, `Object.entries` and `JSON.stringify` on an object that holds Paisa in a property at any depth (`JSON.stringify(bill)`, plan 02, 01-M1) | Everywhere except `src/shared/format/money.ts` | CLAUDE.md rule 1, Code Standards F3  |
| `mandi/no-jsx-literals`       | Words between JSX tags; word strings shown through a branch (`on ? 'Yes' : 'No'`, `on && 'Saved'`), a `const`, or an array or object looked up by index (`words[n]`, `TITLES.a`), or a local function's return value (`{label()}`); word strings as Fluent variables (`getString('x', { v: 'Hello' })`, `vars={{ ... }}`) or written into `document.title`, `textContent`, `innerText`, `setAttribute('title', ...)`; word strings in any JSX attribute except code-only ones (`className`, `id`, `data-*`, `lang`, `role` ...) and props typed as a fixed set of strings (`variant`, `tone`)                           | Everywhere                                     | CLAUDE.md rule 7, Code Standards F31 |
| `mandi/no-physical-direction` | `ml- mr- pl- pr- left- right- text-left border-r rounded-l ...` in any string or template string, and `marginLeft`, `right`, `textAlign: 'left'`, `transformOrigin: 'left top'` (either branch) in any object literal                                                                                                                                                                                                                                                                                                                                                                                                   | `src/`                                         | CLAUDE.md rule 8, Code Standards F24 |
| `mandi/no-raw-design-values`  | Tailwind arbitrary values (`p-[13px]`), raw colours and sizes (`'#2e7d32'`, `'15px'`) in any string, and raw numbers for CSS sizes in any object literal                                                                                                                                                                                                                                                                                                                                                                                                                                                                | `src/`                                         | Code Standards F9                    |

A comment cannot switch a rule off: ESLint runs with `noInlineConfig` and reports any leftover
disable comment as an error, and Stylelint runs with `ignoreDisables`. Exceptions are made only
in the config files, as `files` overrides (tests/unit/lint-config.test.ts proves it).

CSS files are checked by Stylelint: `stylelint-use-logical` (rule 8); our own
`mandi/no-physical-shorthand` (`stylelint-rules/`), which rejects margin, padding, scroll-margin,
scroll-padding, border-width/style/color with 3 or 4 values, inset with 2 or more, and
border-radius with more than one, because those list left and right separately; the keywords
`left` and `right` in background-position, transform-origin, object-position, mask-position,
perspective-origin, and the `background` and `mask` shorthands (`to right` gradients too, 01-M2); and outside `tokens.css`
no hex colours, no `px`/`rem`/`em` units, and font size, line height, weight, family and radius
only through `var(--token)` (F9). eslint-plugin-react is not used because it does not support
ESLint 10 yet; the two React checks we need are the local rules above.

## Money type

`Paisa` (`src/shared/format/money.ts`) is opaque: it is not a number type. TypeScript itself
rejects `const n: number = amount`, `amount * 2`, `amount + 1`, `Math.round(amount)` and passing it
to a `number` or `string` parameter (`tests/unit/paisa-types.test.ts` compiles fixtures that must
fail). Only `money.ts` looks inside: `comparePaisa` for sorting now, `formatPaisa` in plan 02.
Tests build amounts with `tests/helpers/paisa.ts`. The lint rule stays as a second layer for what
TypeScript allows (`Number(amount)`, `${amount}`, `-amount`).

## Design tokens

`src/styles/tokens.css` follows `docs/design/design-system.md` ("Calm ledger"): `--color-bg`,
`--color-primary`, `--color-ink`, `--color-border`, `--color-border-control`, `--color-warn` ...,
`--space-1` to `--space-7`, `--radius-1` to `--radius-3` (4, 8, 12 px), `--control-h` 44 px,
Nastaliq line height 1.9 and Latin 1.45. `--text-1` to `--text-5` have one value per script and
follow the `lang` of each element, so `text-1` is 18 px in Urdu and 14 px in English.

Where a board breaks a rule, the rule wins (design system, top note). Two places, reported to
tech-lead: English running text and button labels use 16 px, not 14 or 15 px, and key chips use
16 px, not 12 px, because NFR-USE-20 asks for at least 16 px Latin text outside grids.
`--color-on-primary` (white text on primary) is one extra name for the white the boards use there.

## Design system components (plan 02)

`src/shared/ui/` holds every piece a screen may use (Code Standards F7). Each has a component test
in Urdu and English (`tests/unit/ui-*.test.tsx`).

| Component                                          | Notes                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Button`                                           | primary, secondary, danger; `disabled`; `busy` (cannot be pressed twice); key chip                                                                                                                                                                                                                                                                                                                                                                                        |
| `Field`                                            | label above, control, then hint or error (red, icon and words). Passes `id`, `aria-describedby`, `invalid` and `readOnly` to its control through a function child                                                                                                                                                                                                                                                                                                         |
| `TextInput`                                        | plain text; `lang` for an English name on an Urdu screen                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `MoneyInput`, `QtyInput`                           | value is text; commas the field adds for display (FR-7.20) are never part of the value: the user types 12000, sees 12,000, and the value is "12000". A comma the user types or pastes is sent and shown exactly as typed: "1,200" is sent as "1,200", and "850,50" is sent as "850,50" for the backend to refuse (plan 02 4.7, R02-6). Anything that is not plain digits (`20+25`, Urdu digits, `15000.50`) is shown and sent as typed, for the backend to read or refuse |
| `DateInput`, `CodeInput`                           | Latin, left to right, physical right edge; never grouped                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `Select`                                           | Radix Select for short fixed lists; long lists use pickers                                                                                                                                                                                                                                                                                                                                                                                                                |
| `Card`, `EmptyState`, `ErrorState`, `LoadingState` | the four states (F8). The skeleton waits 300 ms                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `Num`, `Code`                                      | `<bdi dir="ltr">`, tabular. `<Num paisa={p} />` formats money with the app grouping                                                                                                                                                                                                                                                                                                                                                                                       |
| `Toaster`, `showToast`                             | one toast area in the providers; bottom end corner; 5 s                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `ConfirmDialog`                                    | Radix AlertDialog; action focused, but Back focused for a danger action; focus goes back to the opener                                                                                                                                                                                                                                                                                                                                                                    |
| `ErrorMessage`                                     | one backend error in the app language; number values such as `max` are grouped                                                                                                                                                                                                                                                                                                                                                                                            |

Parts that render in a portal (dialog, select list, toast) set `lang` and `dir` again from the
Fluent bundle around them (`useUiLanguage`), so fonts and direction stay right.

`index.html` has `translate="no"`: a browser translator once rewrote the Urdu title and the `lang`
attribute during a check in Chromium.

**Development gallery:** `pnpm dev`, then `http://localhost:5173/#/dev/gallery`. It shows every
component in Urdu and English side by side. Its words live in `src/app/dev/i18n/` and never ship;
the route exists only when `import.meta.env.DEV` is true, so `vite build` leaves it out.

**Token additions in plan 02** (reported to tech-lead): `--color-scrim`, `--color-skeleton`,
`--color-money-in-on-nav`, `--color-due-on-nav`, `--shadow-overlay`, icon and badge sizes,
`--spacing-0` (Tailwind's `inset-0` needs it after the reset), the layout steps as `normal:` and
`wide:` variants, `--page-max-w`, `--dialog-w`, `--toast-max-w`, `--motion-fast`. Urdu field text
is 18 px (`--text-field`), as the boards draw it: at 20 px Gulzar reaches the field border.

## Version notes

- **TypeScript 6.0.3, not 7.0.2.** TypeScript 7 is the newest stable release, but typescript-eslint
  8.71.1 (the newest) supports TypeScript below 6.1 only. Type-aware lint rules are required
  (Code Standards F1, F3), so TypeScript stays on 6.0.3 until typescript-eslint supports 7.
- Every version is pinned exactly in `package.json` (Tech Stack, version policy).
- `tauri-plugin-single-instance` has no JavaScript package. The other four plugins of Tech Stack 2
  have theirs here.

## Decisions accepted by tech-lead (plan 01)

- **`pnpm gen:types` is a Node script** (`scripts/gen-types.mjs`). It has the same effect as
  `cd backend && TS_RS_EXPORT_DIR="$PWD/../frontend/src/bindings" cargo test -p mandi-app export_bindings`,
  then Prettier on the bindings. The shell form `VAR=value command` fails under cmd.exe, which pnpm
  uses for scripts on Windows. It removes old binding files first, so a type deleted in Rust
  disappears, and it adds the `Paisa` import to bindings that use the money type
  (`scripts/bindings-paisa.mjs`, plan 02).
- **pnpm settings in `pnpm-workspace.yaml`:** `allowBuilds` denies the install scripts of
  edgedriver, geckodriver and esbuild (none is needed). `minimumReleaseAgeExclude` lets six exact
  versions, which were the newest stable on 10-10-2026, past pnpm's hold on very new releases.
  `auditConfig.ignoreGhsas` ignores GHSA-vfj7-8cjw-p6xm (braces, through stylelint only, dev-only,
  no patched version yet); remove it when a fix ships.
- **Glossary format.** Each PRD Section 3 word is a Fluent term (`-bikri`, with `.gender` in Urdu)
  that other messages reference, plus a message `glossary-bikri = { -bikri }` for showing the word
  alone. Rows with several words are split, for example بیوپاری / زمیندار / بزگر become
  `-beopari`, `-zamindar`, `-buzgar`. Every message is marked `# review: pending` (NFR-URD-4).
- **Prettier** checks the frontend and the root scripts, `scripts/perf/` included, from the root
  `.prettierignore`. Always run it from the repository root, so that ignore file applies.
- **AG Grid styling (plan 04):** the CSS-file theming (`theme: "legacy"`), with the grid and theme
  CSS imported through Vite and restyled from our tokens. Nothing injects styles at run time, so the
  Content Security Policy keeps `style-src 'self'`.

## Window sizes and the Mac

On a Mac, Tauri uses WebKit, not WebView2. Urdu shaping, fonts and printing are checked on
Windows (CLAUDE.md, Gotchas). The Windows suite saves screenshots in both languages at
1024 x 600, 1366 x 768 and 1920 x 1080 to `tests/e2e/output/screenshots/`. The test resizes
until the content area is exactly that size and fails with a clear message if the screen is too
small. It never saves a screenshot under the wrong size. The runner uses
`backend/target/release/mandi-app.exe` (or `Mandi Software.exe` if the Tauri CLI renames it) and
prints which one it tested.
