# The alignment check (plan 39)

Every table, grid and card of labels and values stands lined up, in Urdu and in English, at 1366 x 768 and
1024 x 600, on every screen and print. This folder holds the check that proves it.

| File | What it is |
|---|---|
| `measure.page.txt` | The check itself: one function that runs **inside a page**. It reads the layout the browser made (the rectangles of the text) and compares, for each column, the heading with its cells and its total (2 px allowed). It is a snippet, not a module, so the linter does not read it (like the `shots-plan*.playwright.txt` files). |
| `screens.mjs` | The screens and prints it visits: the development pages `/dev/...`, answered by the stand-in backend. |
| `run.mjs` | Runs the check on every screen, language and size in a headless Chrome and takes the pictures. |
| `selftest.mjs` | Proves the check: it must name each fault in small pages that are lined up wrongly on purpose, and pass a page that is right. |
| `summarize.mjs` | Turns the result files into the table of `FINDINGS.md`. |
| `make_sheets.py` | Puts the picture before and the picture after side by side, one sheet for each screen. |
| `../specs/54-table-alignment.e2e.mjs` | The same check on the real app on Windows, after the worked day. |

## The rules it checks (plan 39)

| Rule | What it measures |
|---|---|
| R1 | A column of figures (money, quantity, rate, weight, code, date, number of a record, days): the heading, the figures and the total are **centred**; the figures are written left to right. |
| R2 | A column of words: the words and the heading start at the same edge (right in Urdu, left in English). |
| R3 | A tick, a chip, an icon or a button: centred with its heading. |
| R4 | A heading is never cut off, never broken inside a word, has at most two lines (four in a print of more than eight columns), and all headings of a table have one height, in the middle of the row. |
| R5 | One width for each kind of column. Proved by `tests/unit/column-width-tokens.test.ts` and `tests/unit/table-column-sizes.test.ts`. |
| R6 | A total is under its column; the word "Total" is in the first column of words. |
| R7 | One row height, the words in the middle of the row, and kept off the cell border. |
| R8 | A card of labels and values: the labels on the start edge, the figures in one column on the right edge. |

A column's kind is read from its content (and from `data-align` or the grid's `cell-num`, `cell-center`,
`cell-name` classes), so a column that forgot its class is still caught.

## How to run it

```bash
pnpm dev                                    # the Vite server on http://localhost:5173
node tests/e2e/alignment/selftest.mjs       # the check proves itself
node tests/e2e/alignment/run.mjs --phase after                 # every screen, both languages, both sizes
node tests/e2e/alignment/run.mjs --phase after --only report-r2$,dashboard --sizes 1366x768 --no-shots
node tests/e2e/alignment/summarize.mjs after                   # the findings as a table
python3 tests/e2e/alignment/make_sheets.py                     # before and after sheets
```

It needs `playwright-core` (set `PLAYWRIGHT_CORE_DIR`, or it is found in `~/.npm/_npx`) and a Chrome (set
`CHROME_PATH`, or it uses the one Playwright cached). The project does not carry Playwright: WebdriverIO is
the Windows suite, and it runs `54-table-alignment.e2e.mjs` with the same `measure.page.txt`. The pictures
and `alignment-<width>.json` go to `tests/e2e/output/screenshots/plan39/<phase>/`. The exit code is 1 while
any rule fails.

## A new table

Use `shared/ui/Table` (give every figure column its `size`), or `ListGrid` / `EntryGrid`, or the shared
print layout. A card of labels and values puts its figures in `value-col`. Then run the check on the screen.
