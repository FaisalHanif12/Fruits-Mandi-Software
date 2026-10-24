# shared/print (plan 14)

The one layout of every printed page of the app: the seller bill, the buyer slips, the پختہ slip,
later the receipt slip (plan 15) and the reports (D5). PRD FR-19.3 to FR-19.7, FR-7.36, NFR-URD-11
to NFR-URD-16, NFR-URD-28, NFR-SPD-14. Read the `print-and-export` skill first.

## The rule

The backend sends ONE shape, `PrintData` (plan 14 contract 5): every value is text already made
by the backend, every label is a Fluent message id, the amount in words is a sentence. **The layout
lays out what it gets and calculates nothing** (CLAUDE.md rule 1). There is no `+`, `*`, rounding or
summing in this folder or in `features/bikri/print/`.

## What is here

| File                                                         | What it is                                                                                                                                                                                                                                   |
| ------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `print.css`                                                  | All print styles. ONE literal `@page` rule per paper class (`a4`, `a5`, `r80`), with the paper size, the margins and the page footer. Table header repeats, rows never split, numbers on the physical right edge, a new sheet for each party |
| `PrintDocument.tsx`                                          | The root: print language (`lang`, `dir`), paper class, digit style. Gives the pages a Fluent bundle of the PRINT language                                                                                                                    |
| `PrintProvider.tsx`                                          | A Fluent bundle of the print language that leaves the app language alone (PRD FR-7.36)                                                                                                                                                       |
| `PrintSheet.tsx`                                             | One party: header, who it is for, goods, amounts, how he pays, amount in words                                                                                                                                                               |
| `PrintGoods.tsx`                                             | The goods: a table, or two rows of text for each line on the 80 mm roll                                                                                                                                                                      |
| `PrintSummary.tsx`, `PrintHeader.tsx`, `PrintPartyBlock.tsx` | The parts of a sheet                                                                                                                                                                                                                         |
| `footer.ts`                                                  | The page footer: "Printed 10-10-2026 17:40 · Page 1 of 2", as a CSS `content` value                                                                                                                                                          |
| `wait-ready.ts`                                              | Waits for the fonts and the layout, and gives the page height for the 80 mm roll                                                                                                                                                             |
| `docs.ts`, `types.ts`                                        | The documents (seller bill, buyer slips, پختہ slip, receipt slip) and their names; the types from the generated bindings                                                                                                                     |

## Things that were found the hard way

- **The footer is set through the CSS object model.** The production Content Security Policy
  (`style-src 'self'`) refuses an inline `<style>`, so the text of the page margin box is the
  custom property `--print-footer` set on the root element (`footer.ts`). `print.css` reads it with
  `content: var(--print-footer, "")`. The page and the page count are CSS counters
  (`counter(page)`, `counter(pages)`), so "Page 2 of 4" is right on every sheet.
- **The `page` property is on `html`, not on the document.** A box whose page name differs from the
  box beside it forces a page break, and the empty toast area beside the document made a blank
  second sheet.
- **The date in the footer is wrapped in a left-to-right isolate.** A page margin box has no
  `<bdi>`, so "10-10-2026" is otherwise written "2026-10-10" inside an Urdu line.
- **Numbers take the LATIN print size of the text around them** (`--print-num-size`), also in an
  Urdu print: Inter at step 3 is 10 pt beside Gulzar at 12 pt, and they look alike.
- **Paper is white.** `html` and `body` get a white ground and no minimum height in print (the app
  page ground would print tinted, and a body as tall as the window made a blank second sheet).
- **The 80 mm roll is one tall page.** All slips of "print all" are on it, with a dashed cut line
  between them and no page break; `height_px` (from the print route) is the height of all of them.
  Its footer is the print time only. A4 and A5 start each party on a new sheet.
- **`height_px` is the height of the document, not `scrollHeight`**, which is never less than the
  height of the hidden window. The window (`data-window`) lays the page out at the width of the text
  of the paper with no padding or frame, so the height is the height it prints at (checked in Chrome
  by `print-shots.mjs`: the same number on screen and in print).
- Chrome shows what WebView2 shows for all of the above, but Urdu shaping and printing must still
  be checked on Windows (CLAUDE.md, Gotchas): the spec `tests/e2e/specs/15-bikri-print.e2e.mjs`.

## Looking at it

`pnpm --filter frontend dev`, then `node frontend/tests/e2e/print-shots.mjs`. It writes the PDF and
the PNG of every document in both languages on A4, A5 and the roll to
`tests/e2e/output/screenshots/plan14/`, and tells the page counts. The page `#/dev/print?doc=
seller_bill&language=ur&paper=a4` (dev only) shows one document in the browser; `&digits`,
`&cancelled`, `&long=15` and `&lines=60` show the other cases.

## The receipt slip (plan 15)

The fourth document, `receipt_slip` (PRD FR-15.2), is one voucher on one page: the same sheet, with
the voucher number (`print-fact-voucher-number`), the party, the amount and the amount in words.
There is no line about how it was paid: all money is روکڑ (PRD 15.1). The template is
`features/vouchers/print/ReceiptSlipTemplate.tsx`, the hidden route is
`?voucher=<id>&doc=receipt_slip&language=..&job=..`, and its paper is `paper_receipt_slip` in the
Printing tab. The data is `reports_voucher_print_data`; the layout still calculates nothing.
