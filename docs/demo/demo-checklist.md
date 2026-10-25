# Demo checklist (about 45 minutes)

Written by `tech-lead`, 25-10-2026, for Umar. Source: PRD 27.1 (the demo script), 4.9 (what every screen shows), 18.4, 24 (assumptions), NFR-USE-1 to NFR-USE-5, NFR-SEC-8. Every number below is from the PRD or `worked_day.py`. If a screen shows a different number, the app is wrong, not this list.

Run the demo on the Windows computer with the installed app. On the Mac, use this list to rehearse; the Windows-only checks are marked LOCAL-ONLY at the end.

## 0. Before you start

- A fresh data folder with the opening position of PRD 4.3: روکڑ 50,000, رحیم with an open advance of 15,000, ابتدائی سرمایہ 65,000. `pnpm seed` makes it on a Mac. Trial balance 65,000 = 65,000.
- Partners الف 40%, ب 30%, ج 30%. Global commission 6% for sellers and buyers, لاگا 8 per نگ on fruit (the rules are set at setup).
- Accounts: رحیم 030110001, کریم 030110002, اکرم ٹول ٹیکس 03040001, ارشد ملتانی 03040002. Items کیلا and سیب, packing پیٹی, the two اڈا.
- The garden, trip and fruit demos (steps 6b) start from the same opening position, in a second data folder. They are separate runs (E2E-40, E2E-41, E2E-42), so their numbers do not add to the worked day.
- Printer and paper chosen in ترتیبات. The firm's contacts and trade mark entered (they print in every header).

## 1. The steps, with what the screen must show

| # | Step | Expected on screen |
|---|---|---|
| 1 | Sign in. Open the dashboard and the partner strip. | Cash 50,000. Three partner cards. |
| 2 | Open شراکت دار and کمیشن اصول. Use "اصول آزمائیں". | Shares 40 / 30 / 30 total 100%. Test: 20 × 600 for a 6% seller gives 720 and names the rule. |
| 3 | Make a new seller and a new buyer with the arrow keys and `Enter` only. Type the name in English letters. | Urdu name appears with the English name filled. Codes 030110003 and 03040003. |
| 4 | بکری 1 with short codes `1` and `2`. Show the party card, remaining qty, rate hint, the live summary, the partner split and the ⓘ on a commission. Switch to English with `Alt+L` halfway and back. Save with `Ctrl+Enter`. | خام 29,400. کل اخراجات 10,084. صافی 19,316. اکرم pays 12,880, ارشد 18,684. Shared income 3,948, split 1,580 / 1,184 / 1,184. ⓘ on اکرم's commission: "12,000 × 6 ÷ 100 = 720". After the switch nothing typed is lost. |
| 5 | بکری 2 with a one-time buyer through پختہ کھاتہ (شاہد, 10 پیٹی سیب at 1,200, نقدی). Enter the same truck again to show the warning. | Buyer pays 12,800. Seller صافی 10,160. Shared income 1,540, split 616 / 462 / 462. The register shows شاہد as paid. |
| 6 | On the voucher screen: وصولی 10,000 from اکرم (use "پورا بقایا" first to show it), then ادائیگی 4,316 to رحیم. On the خرچہ screen: بجلی 3,000 as دکان خرچہ (show the split before save), then 2,000 for الف as ذاتی خرچہ. | اکرم owes 2,880. رحیم 0 and his advance fully adjusted. Electricity split 1,200 / 900 / 900. الف's کل رقم goes 996 → −1,004. |
| 6a | Show the attention bell and آج کے کام. Print a buyer slip in Urdu and in English. | Bell lists what needs action. The slip shows the firm header, contacts, trade mark, the party with its tag and "Page 1 of 1". |
| 6b | Show a market trip, a garden, the partner statement with its dated lines, and الف's پھل. Print the party day sheet (R35) for all buyers. | See Section 2 below. R35 total remaining 21,564. |
| 7 | Dashboard. | Cash 58,484. Today's خام 41,400 (2 بکری). Commission 5,448. Shared income 5,488. Buyers' dues 21,564. Sellers payable 10,160. Advances open 0. Stock 0. Today's expenses 3,000. |
| 8 | تجزیات, views A, B, C and F. | View A: کیلا 50 نگ, 29,400, avg 588, total comm 3,928, 72.10%. سیب 10, 12,000, 1,200, 1,520, 27.90%. Total 60, 41,400, 5,448. View F: 2,196 / 1,200 / 996 and 1,646 / 900 / 746 twice; total 5,488 / 3,000 / 2,488. |
| 9 | Partner overview. | الف profit share 996, taken 2,000, خالص −1,004. ب and ج 746 each. |
| 10 | روزنامچہ and the trial balance. | Both balance. Trial balance 85,048 = 85,048. |
| 11 | Backup screen: take a backup now, show the list and the test restore. | A new row, checked OK. |
| 12 | Walk through the assumptions in Section 3 and note each answer. | |

## 2. Step 6b in detail (separate runs)

- **Garden (E2E-40).** Shahpur Garden, rent 50,000, الف 60% from the shop, ب 40% from his pocket. Installments 30,000, then 20,000 and labour 5,000. A بکری lot of 100 پیٹی کیلا at 600 to ارشد: he pays 64,400, the garden's account gets 56,400, commission 8,000 split 3,200 / 2,400 / 2,400. Other shop 10,000. Running balance 11,400. Close it: 66,400 splits 39,840 / 26,560 and the garden account is 0. Partner totals 10,040 / 28,960 / 2,400. Trial balance 106,400.
- **Trip (E2E-41).** الف 50% from the shop, ب 50% from his pocket. 80 پیٹی مالٹا at 1,200 (96,000) and a truck cost of 4,000. A lot of اپنا مال sells all 80 to ارشد at 1,400: he pays 119,360, commission 14,080 split 5,632 / 4,224 / 4,224, the trip gets 105,280, result 5,280, the trip closes. Partner totals 8,272 / 56,864 / 4,224. Trial balance 134,360.
- **Fruit (E2E-42).** After the partly sold بکری (E2E-16): الف takes 2 پیٹی کیلا at 580 (1,160 off his total; رحیم payable 2,750; lot balance 3). Guest fruit 3 پیٹی سیب at 1,000 from حاجی نذیر: 1,000 each, حاجی نذیر payable 3,000. Cash stays 45,000. Totals −736 / 68 / 68. Trial balance 77,610.
- **R35 (E2E-43).** اکرم: previous 0, bought 12,880, paid on the spot 0, paid that day 10,000, remaining 2,880. ارشد 18,684. شاہد 0. All buyers 21,564 = the R20 total.

## 3. Assumptions to ask the client (PRD 24)

Ask each, write the answer next to it, and tell `tech-lead`. The default in brackets is what the app does now.

1. Real commission rates per seller and item (6%).
2. Any لاگا on vegetables (none).
3. Usual amounts for مزدوری, منشیانہ and فون, so rules can fill them (typed on each lot).
4. When the buyer rate differs from the seller rate, and who keeps the difference (the firm; shared by partners).
5. Real partner names and shares (الف 40, ب 30, ج 30).
6. Old تاریخ بل: what is it used for (same as تاریخ).
7. Does the credit limit warn or block (warn).
8. Printer and paper: A4 bills, A5 or 80 mm slips.
9. Lakh style numbers, 13,16,040 (English digits, 1,316,040).
10. Should buyer slips always be in Urdu (app language, with a choice on each print).
11. Does the munshi prefer the phonetic keyboard to English-to-Urdu typing (English-to-Urdu).
12. Attention limits: 2 days for lots, 30 days for dues, 7 days for payables.
13. A loan taken off a salary: is it posted as تنخواہ expense (yes).
14. Are trip goods ever bought on credit (no, paid at once). Is the truck fare ever known before anything is bought.
15. Fruit for guests: equal between partners, or a fixed share (equal).
16. Does the money of a sale in another shop ever stay with a partner (it comes into روکڑ).
17. Are the shares of a garden or trip ever changed after its first entry (no).
18. A42: are the کیلا lines with no rate sold by weight, or is a lump amount typed (by weight, rate per kg).
19. Should the buyer side also be 0 for the family khata's own goods (no, buyer side is charged).
20. Should a partner's name in "ادا کیا" block a دکان خرچہ (it only suggests ذاتی خرچہ).
21. Buyer slips and R5 (team decision of 25-10-2026): slips show a named buyer's cash lines, R5 ادھاری تفصیل lists credit buyers only. Is this right.
22. Print All order: slips follow the Urdu name order, so Arshad comes before Akram. Is that right.
23. On a weight line, should R5 show the weight in the detail.

## 4. Measures to take with the munshi and the Owner

- **NFR-USE-1.** Three people who have never seen the app enter بکری 1 with only the on-screen hints. Target: under 10 minutes each. Write the time and the mistakes.
- **NFR-USE-2.** A trained munshi enters a 10-line بکری with the keyboard only. Target: under 90 seconds.
- **NFR-USE-3.** A munshi who used the old software runs a full day alone after one day of training.
- **NFR-USE-4.** At most one uncorrected mistake in 50 entries.
- **NFR-USE-5.** The munshi and the Owner rate "easy to use" 4 out of 5 or better.
- **NFR-URD-8.** The munshi reads every screen. Zero words he does not understand. Use `docs/demo/urdu-review.md`.

## 5. The shop laptop

- Turn on Windows BitLocker for the disk (NFR-SEC-8). A stolen laptop then shows nothing.
- Print the backup key and the recovery code at setup and keep them away from the laptop. Do a test restore in the first week (NFR-BAK-7).
- Plug a UPS in. Load shedding must not cut a save.

## 6. LOCAL-ONLY: only on the Windows computer

Not checked on the Mac. Check them before go-live:

- The installer (MSI and NSIS), the offline WebView2, the update check.
- Urdu Nastaliq shaping in WebView2 and the real printer: the bill, slips, reports in A4, A5 and 80 mm, and the PDF.
- The full Windows end-to-end suite (WebdriverIO and `tauri-driver`), including the fruit and print specs.
- A real cloud upload to Cloudflare R2 (D5-Q2) and the Windows code-signing certificate (D5-Q3).
- The speed targets of NFR Section 2 on the reference laptop (Core i3, 4 GB, 1366 × 768).
