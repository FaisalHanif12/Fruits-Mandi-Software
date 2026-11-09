# D8 evidence: Urdu PDFs and the Mac print window

Made on the Mac on 09-11-2026 with `node scripts/print/pdf-from-route.mjs` (plan 69 T3, plan 63 T4).
Headless Chrome opens the real print routes (`#/print?...`). The backend's real print data (the
contract fixtures) answers the route's own commands. The route says `print-ready`, and Chrome prints
the page to a PDF with the page size and margins of the print style.

| File | What it shows |
| --- | --- |
| `r1-seller-bill-ur.pdf` | The Urdu seller bill (R1) of بکری 1: 29,400, 19,316, 10-10-2026 |
| `r5-adhari-tafseel-ur.pdf` | ادھاری تفصیل (R5), one slip for each credit buyer: 12,000, 17,400 |
| `r35-party-slip-ur.pdf` | The party day slip (R35) of اکرم: 12,000, 12,880, paid 10,000 |
| `*.txt` | The text of each PDF, read with PDFKit (`scripts/print/pdf-text.swift`) |
| `*.png` | The first page of each PDF, to check the shapes of the digits by eye |
| `mac-print-box-*.png` | The print box on a Mac, with the hint for the system print window |
| `mac-settings-printing-no-printer-list-en.png` | Settings, Printing, with no printer choice when the list is empty |

Checks the script makes on every text (exit code 1 if one fails): no ۰-۹ and no ٠-٩, a date with
the year 2026, grouped numbers, the figures of the document, and "10,000" in the three together.
The text proves the characters, not the glyph shapes: the PNGs show plain 0-9 in Inter. The seller
bill and R5 of the worked day have no amount of 10,000; the party slip has it (paid 10,000).
The Windows WebView2 print of the same pages stays LOCAL-ONLY.
