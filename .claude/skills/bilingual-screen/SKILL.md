---
name: bilingual-screen
description: Builds or changes any screen of the Mandi Software so it works the same in Urdu (right-to-left) and English (left-to-right), at every supported window size, with the four states, keyboard support and no lost work on a language switch. Use for every new page, dialog, form, list or dashboard part, and when reviewing one.
---

# Build a screen that works in Urdu and English

Every screen exists twice: Urdu right-to-left (the default) and English left-to-right. **It is one screen with one layout**, not two. The direction comes from the app; the screen never checks the language to decide its layout (Code Standards F37).

Related skills: `entry-grid` for grids where the user types data, `urdu-english-text` for every word on the screen, `tauri-command` for the data it loads and saves.

## Before you start

- Read the plan file and the PRD section for the screen: its fields, buttons, keys and what it updates.
- Pick the screen pattern:
  - **Entry screen:** header fields, a grid or form, a live summary, and an action bar. Examples: بکری, وصولی.
  - **List screen:** filters, a table, and row actions. Examples: آج کی بکری, باقی مال.
  - **Report screen:** a filter panel, an on-screen view, and print, PDF and Excel buttons.
- Reuse the pattern components from `shared/ui`. If a pattern piece is missing, add it there, not in the feature.

## Build steps

1. **Route and page.** Add a lazy route in the module's `routes.tsx` (Code Standards F36). The page composes shared components only. Data comes through the module's hooks (`tauri-command` skill).
2. **Layout with logical properties only.**
   - Tailwind: `ms-` `me-` `ps-` `pe-` `start-` `end-` `text-start` `text-end` `border-s` `border-e` `rounded-s` `rounded-e`.
   - CSS: `margin-inline-start`, `inset-inline-end`, and so on.
   - Never `left`, `right`, `ml-`, `mr-`, `pl-`, `pr-`, `text-left` or `text-right`. Lint blocks them.
   - Flex and grid rows flip by themselves with `dir`. Do not reverse them by hand.
   - Use the 12-column layout grid and spacing tokens from `shared/ui` (Code Standards F9, F10, F15).
3. **Text.** Every word comes from Fluent (`urdu-english-text` skill). Use `<Localized id="...">` or `useLocalization().l10n.getString(...)`. There are no string literals in components.
4. **Names.** Always `displayName(record, language)`. Never read `name_ur` or `name_en` in a component (F38). Free text such as notes and مارکہ is shown exactly as typed (PRD FR-7.25).
5. **Numbers, codes, phones, truck numbers.** Always through `<Num>` or `<Code>`, which isolate their direction, so `(16 @ 900)` never flips inside Urdu (F25). Money only through `formatPaisa` (inside `<Num>`).
6. **Typography.** Use the type tokens only. Urdu Nastaliq is at least 18 px with line height at least 1.8. Numbers and Latin text are at least 14 px in grids and 16 px elsewhere (NFR-USE-20, NFR-URD-11). Number columns use tabular digits, centred under their heading (F17).
7. **Icons.** Icons that point a direction (arrows, chevrons, "back", "next") mirror in right-to-left using the `rtl:` variant. Other icons never mirror (F26).
8. **Popovers, tooltips and menus.** Place them with logical sides through the shared helper (for example "end" side), not a fixed "right". Radix gets the direction from `DirectionProvider`.
9. **Charts.** Only through the shared chart wrapper, which places axes and legends for the current direction.
10. **The four states** (F8):
    - Loading: a skeleton.
    - Empty: says why and what to do, with a button. Example: "آج کوئی بکری نہیں۔ نئی بکری کے لیے Alt+B دبائیں" (FR-7.46).
    - Error: a plain message from `errors.ftl` with a retry button.
    - Data.
11. **Unsaved work.**
    - Keep everything the user typed in the screen's store or form state, above the page component. A language switch rebuilds grids and must lose nothing (FR-7.33, F39).
    - Register the screen with the draft auto-save (every 5 seconds, FR-12.10) and the "محفوظ کریں؟" leave guard (FR-7.9).
12. **Keyboard.**
    - Register the screen's fields in reading order with `shared/keyboard`.
    - Register its shortcuts in `shortcuts.ts`, and show each key on its button, for example "محفوظ کریں F10" (FR-7.4).
    - Arrow keys follow what the user sees (FR-7.13).
    - Every action also has a visible button.
13. **Window sizes.** The screen must work from a 1024 × 600 content area to 2560 × 1440 with no sideways scroll (NFR-USE-15).
    - Use the three layout steps: compact (1024 to 1365), normal (1366 to 1919), wide (1920 and above) (F22).
    - In compact, the بکری summary moves under the grids, and less-used columns go behind "مزید" (F22).
    - Use flexible layouts (`minmax`, flex). No fixed page widths (F21).
14. **Text size.** The screen stays aligned at every text-size step from 80% to 150% (NFR-USE-18).

## Common mistakes

| Mistake | What happens | Do this instead |
|---|---|---|
| `ml-4` on a label | The gap is on the wrong side in Urdu | `ms-4` |
| `translateX(8px)` for a slide animation | Slides the wrong way in Urdu | Flip the sign with `rtl:` or use logical keyframes |
| A number inside an Urdu sentence without `<Num>` | `16 @ 900` shows as `900 @ 16` | Wrap it in `<Num>` |
| A popover with `side="right"` | It opens over the field in Urdu | Use the logical side helper |
| Typed values kept only inside an AG Grid cell | Lost on a language switch | Keep them in the screen store (`entry-grid` skill) |
| `if (language === 'ur')` to swap columns | Two layouts drift apart | Logical properties; the direction does the rest |
| A hard-coded "Rs." or "روپے" | Wrong in one of the languages | A Fluent message with a variable |

## Tests (Code Standards 9.1, "New screen")

- **Component tests** for the four states.
- **Keyboard-only test:** the screen's main task done with keys only (F30).
- **Language switch mid-entry:** type values, switch with `Alt+L`, check every value, the selected row and the focus are unchanged, then switch back (F39, E2E-22 style).
- **Screenshot tests** in Urdu and English (F14, F37, NFR-USE-28), at 1366 × 768 and 1920 × 1080, and at the smallest supported size. Screenshots must come from WebView2 on Windows, because a Mac renders with WebKit. The Tech Stack does not name the screenshot tool yet. Until it does, ask `tech-lead` before adding one.

## Self-check before handing off

Open the screen in both languages and go through this list:

- [ ] Everything mirrors correctly. Nothing overlaps or gets cut off at 1024 × 600 or at 150% text size.
- [ ] No English word in the Urdu screen and no Urdu word in the English screen, except names and free text exactly as typed.
- [ ] Every number, code and phone keeps its order. Number columns are centred under their headings, with the total centred under them (F17).
- [ ] Urdu letters are never broken or boxed, and tall letters are not clipped in rows.
- [ ] Loading, empty, error and data all look designed.
- [ ] The main task can be done with keys only, and with the mouse only.
- [ ] A language switch in the middle of typing loses nothing.
- [ ] Checked on Windows with the `windows-verify` skill.
