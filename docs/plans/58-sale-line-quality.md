# Plan 58: the quality of a sale line (درجہ)

Version 1.0, 06-11-2026. Lead: tech-lead. Asked by Umar on 06-11-2026. PRD v1.30 FR-12.42 and 12.4; Architecture v1.19 15.6 (migration 0019). Branch `feature/assistant`, folder `Fruit Mandi Parallel/d5`. It runs after plan 57 on the بکری files, and in parallel with the assistant runs.

## 1. Goal

The goods of one lot can differ in quality, and each quality sells at its own rate. The munshi already sells them as separate lines of the lot. Now he can mark each line with its quality, so the bills show why the rates differ.

## 2. Rules (Umar's words, settled)

- **Optional.** Blank by default. The munshi adds it only when he wants to.
- **A dropdown with fixed names.** The names are اول، دوم، سوم (English: First, Second, Third). No other word can be typed.
- **No money change.** The line is still qty × rate, with its own commission and لاگا (C1 to C5).
  - An edit that changes only the quality keeps the line's stored rules and amounts (FR-12.20).
- **Keyboard.** `Enter` still goes buyer, qty, rate, then the next line (NFR-USE-6). The quality cell is reached with the arrow keys or a click, and Space or Alt+↓ opens the dropdown.
- **Prints.** The quality appears after the item on every print with sale lines, for the buyer and for the seller:
  - the seller bill (R1);
  - the buyer slips;
  - R5: "(سیب اول 10 . 1200)";
  - the party day slip R35, for example "10, پیٹی سیب اول, 1,200, ...";
  - the ledgers with items (R13, R14);
  - the sales register (R2).
  A blank quality prints nothing, and the old prints read exactly as before.
- **Assistant.** It understands awwal / doyam / soyam, اول / دوم / سوم and first / second / third quality. Example questions:
  - "awwal seb aaj kitne ka bika";
  - "doyam kele ka aaj ka rate";
  - "ehsan ke seb kis kis darje mein bike".
  It answers from the sale lines, which carry the quality, and every line it shows carries it. There is no new analytics view and no rate hint per quality (Umar: "rest ignore").

## 3. Worked numbers

Ehsan's 20 پیٹی سیب, seller commission 6%:

| Line | Buyer | Quality | Qty | Rate | Value | Seller commission |
|---|---|---|---|---|---|---|
| 1 | اکرم | اول | 10 | 1,200 | 12,000 | 720 |
| 2 | اکرم | دوم | 6 | 800 | 4,800 | 288 |
| 3 | ارشد | سوم | 4 | 500 | 2,000 | 120 |
| | | | **20** | | **18,800** | **1,128** |

- The figures are the same as with no quality at all.
- R5 for اکرم reads "(سیب اول 10 . 1200) 12,000 ..." and "(سیب دوم 6 . 800) 4,800 ...".

## 4. Contract

- **Migration 0019:** `bikri_sales.grade TEXT NULL CHECK (grade IN ('first','second','third'))`. Old rows stay null.
- **Sale line input:** `grade: "first" | "second" | "third" | null`. The backend parses it like any enum, and any other value is ERR_OUT_OF_RANGE on `lots[i].sales[j].grade`.
- **Preview, save, edit, later sale and the saved بکری view:** each carries the grade through. Its snapshot is kept on the line.
- **Print data:** each sale line row gets `grade`, and the print templates put the Fluent word after the item.
- **Report rows** that show sale lines (R2, R5, R13, R14, R35 slip, R1) carry `grade`. The assistant's report tool shows it as a column word.
- `pnpm gen:types`.

## 5. Tasks

| # | Task | Owner | Blocked by |
|---|---|---|---|
| T1 | Migration 0019, the input, the domain pass-through, save, edit, later sale, print data and report rows, with tests. Money agent: confirm that no amount changes and that a quality-only edit keeps the snapshot. | backend3 | plan 57 T1 |
| T2 | The dropdown cell in the sale lines grid, with the keyboard rule above, and the quality in the saved view and on every print template. Fluent words in the glossary, in both languages. Screenshots of the prints in both languages. | frontend3 | plan 57 T2, T1 types |
| T3 | Assistant: the quality words in the lexicon, a filter word on the line reads, and the prompt line (lead). Tests in three languages. | backend2 (lexicon and reads), lead (prompt) | T1 |
| T4 | Robustness: quality lines in the year books, bank questions, and checks that a print shows the quality and that no figure changes. | robustness | T1 |

## 6. Acceptance

- The worked example saves, and the figures equal the no-quality case.
- R5, R1, R35 and R13 show the quality in Urdu and English. A line with no quality prints as before.
- `Enter` takes exactly 3 stops per line.
- Every golden test passes with no number changed. fmt, clippy, typecheck, lint and i18n:check are clean.
- The assistant answers "awwal seb aaj kitne ka bika" with a checked figure.

## 7. Umar's choice (06-11-2026): the layout

The sale lines grid has no room for one more column at 1024 or 1366 wide without a sideways scroll. That scroll breaks NFR-USE-15.
- Umar chose: quality is a column right after the rate.
- The ادھاری / نقدی toggle moves into the line details box under the grid, at every size. It still fills itself from the buyer's credit setting, and it keeps its key and mouse path.
- The saved view and the prints show credit or cash as before.
