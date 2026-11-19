# Plan 60: D8, the decisions of the run and the requirement documents

| | |
|---|---|
| Deliverable | D8 (`docs/deliverables/D8-demo-feedback.md` v2.0) |
| Branch | `feature/d8-demo-feedback`, from `feature/assistant-realtime` at 5bca5a61 |
| Owner | tech-lead |
| Phase | Mac phase: no pull request, no merge, no release. Commit and push after each plan. |

## 1. Goal

Write down, once, every decision of the D8 run, so all later plans (61 to 70) and all requirement
documents say the same thing. Then raise the PRD to v1.32 and update every affected document, in their
own commit, before any code of the run is committed.

## 2. Order of the run

| Plan | Item | Depends on |
|---|---|---|
| 60 | This file; PRD v1.32 and the other documents | |
| 61 | Bug 1: the sale line with no figures, the rate hint, 1,000 lines | |
| 62 | Bug 2: Backspace and Delete | |
| 63 | Bug 3: digits on prints | |
| 64 | Feature 5: پختہ کھاتہ removed, چکتا کھاتہ added | 60 |
| 65 | Feature 4: paid on the spot | 64 |
| 66 | Feature 6: party ledgers | 65 |
| 67 | Feature 7: find and edit a saved بکری | 65 |
| 68 | Feature 8: the R5 print button | 65 |
| 69 | Feature 9: printing on the Mac | 63 |
| 70 | Feature 10: harder assistant questions | 64, 65 |
| End | One gate: `reviewer` and `robustness` for the whole run, then the fix loop | all |

Feature 5 comes before Feature 4 on purpose: it renames and deletes across about 800 files, so it goes
first and alone, and Feature 4 is then built on the clean names.

## 3. The decisions

D8 Section 6 questions use their defaults: Q1 a buyer who paid in full still prints on ادھاری تفصیل with 0
left; Q2 Enter stops on the paid field (4 Enters); Q3 money paid on the spot settles its own bill first;
Q4 the screen is پارٹی کھاتے (Party Ledgers); Q5 the R5 button prints the whole day, with a choice.

### 3.1 Feature 4: paid on the spot (موقع پر جمع)

| # | Decision |
|---|---|
| P1 | `SaleInput.payment_type` is replaced by `paid` (text, whole rupees, may be empty). The ادھاری / نقدی toggle goes from the screen. |
| P2 | Empty means 0, except for a buyer who must pay in full (P3). |
| P3 | A buyer who may not take credit, نقدی منڈی and a چکتا buyer always pay the full amount. The backend sets paid = buyer total. The cell shows it and cannot be typed in. |
| P4 | Paid more than the line's buyer total is refused: `ERR_PAID_EXCEEDS_TOTAL`, with `max`, at `lots[i].sales[j].paid`. A negative or decimal value is refused as any money input. |
| P5 | Left on account = buyer total − paid. Worked out by the backend only. |
| P6 | Journal: the sale part does not change. The cash part has, for each line with paid above 0: روکڑ بنام paid, the same buyer account جمع paid. A line paid in full posts exactly what a نقدی line posts today; a line paid 0 posts exactly what an ادھاری line posts today. |
| P7 | Matching (Q3): the paid credit line settles that line's own bill first (`bill_allocations`, as a نقدی line is pinned today). What is left of the bill is a normal open bill, matched oldest first against the buyer's unmatched credits, as an ادھاری line is today. |
| P8 | The credit limit check uses what is left, not the buyer total. |
| P9 | Storage: new column `bikri_sales.paid_paisa` (`INTEGER NOT NULL DEFAULT 0`, multiple of 100, ≥ 0, not above `buyer_total_paisa`). The migration sets it to `buyer_total_paisa` on cash lines and 0 on credit lines, then freezes it in `bikri_sales_snapshot_is_final`. So no saved number changes. |
| P10 | `bikri_sales.payment_type` stays in the table and is still written: `cash` when paid = buyer total, else `credit`. Nothing decides money from it any more. |
| P11 | Edit: a line whose only change is `paid` keeps its stored rules and amounts (FR-12.20), as a change of quality does. |
| P12 | R3 (نقدی) lists every line with paid above 0, R4 (ادھاری) every line with something left. A part-paid line is on both. Both gain the columns paid and left. Their old columns and old rows do not change. |
| P13 | R5, R13, R14, R34, R35, dues, dashboard, assistant follow P5 and P6. R5 gains paid and left, and lists every buyer of the day (Q1). |
| P14 | Save confirmation of FR-6.7 is unaffected: paid money comes into روکڑ. |

Worked numbers: D8 Feature 4 table (21,360 / 10,000 / 0; totals 194,520 / 31,360 / 163,160; ہارون ends 115,060 بنام).

### 3.2 Feature 5: چکتا کھاتہ

| # | Decision |
|---|---|
| C1 | The system account 0030 stays, in control 0304, type number 003, and is renamed چکتا کھاتہ / Chukta Khata. It holds no balance after any sale, because every line on it is paid in full. E2E-10 keeps its row with the new word and 0. |
| C2 | A new table `chukta_buyers` (system columns, `name_ur`, `name_en`, search columns, `phone` nullable, `is_active`, unique on the person key) is the list of چکتا buyers. |
| C3 | The buyer picker of the بکری lists چکتا buyers beside ordinary buyers, each marked چکتا. Its add row offers both "new buyer" and "new چکتا buyer". The short form asks name (both scripts, FR-7.24) and an optional phone. The new record is saved at once and chosen at once (FR-7.69). |
| C4 | The picker does not offer account 0030 itself. Other pickers do not list چکتا buyers: they are not parties. |
| C5 | A sale line to a چکتا buyer is `buyer_account_id` = 0030 plus the chukta person (the same plumbing as the old person: names, phone and key on the line and the journal lines, renamed). Paid is forced to the full amount (P3). Normal commission and لاگا rules apply. |
| C6 | No personal ledger: no party card balance (the card shows name, phone and the words "چکتا: پورا نقد"), not in R13, R14, R20 to R24, R26, party ledgers, voucher pickers, the attention list or the dead amount. |
| C7 | They do stay in the day's sale book, R2, R3, R5, R34, the R35 all-parties table and analytics, each by name, as E2E-36 and E2E-43 need. |
| C8 | Removed for good: account 0031 and one-time sellers, پختہ party accounts (0032 up, FR-13.11), the register R9 and its menu item, the person picker on وصولی and ادائیگی (FR-13.7), make regular (FR-13.8), the پختہ filter of the accounts list, dead-amount marks by person, the پختہ lines of R24 and R26. R9 is retired; other report numbers do not move. |
| C9 | Old data (development databases only; no shop uses the app yet). A new migration: renames the account 0030; renames `pukhta_*` columns to `chukta_*` and `is_pukhta_khata` to `is_chukta_khata`; fills `chukta_buyers` from the distinct people of old lines on 0030; makes 0031 inactive and renames it to say it is an old closed account; old پختہ party accounts stay as plain sellers or buyers, and `is_pukhta_party` is renamed `was_one_time_party` (never read again). The seller-side `bikri_lots.pukhta_*` columns are renamed too and stay null on new lots. No posted entry, amount or chain hash is touched. Old open amounts on 0030 stay in the books and the trial balance and simply no longer show in dues lists. |
| C9a | The one-time slip is gone. Buyer slips include چکتا buyers (FR-7.57, FR-12.12). The word پکا کھاتہ stays on the assistant's never-use list. |
| C10 | Old migrations 0001 to 0019 are never edited, so the word `pukhta` stays in them and in history (old plans, reviews, change logs). Everywhere else it is gone. That is what the repository search must show. |
| C11 | The code renames: `pukhta` becomes `chukta` where the buyer-side plumbing lives on (`chukta_person_key` and so on); everything seller-side and party-account is deleted, not renamed. |

Worked number: D8 Feature 5 (شاہد, 12,000, 720, 80, paid 12,800). Every number of the worked day stays.

### 3.3 Other items

| # | Decision |
|---|---|
| B1 | Rate hint (FR-7.53): it must not look like a typed value: smaller, italic, marked with a word or sign such as "پچھلا 900" and with no number alignment of a value. Plan 61 fixes the exact look. |
| B2 | Over-quantity: the existing refusal of FR-12.6 stays. In addition the cursor cannot leave that line's quantity with Enter until it is fixed. |
| B3 | An incomplete line (buyer, qty or rate missing, once something is typed on it) says in words what is missing, in the row. A blank row stays silent. |
| B4 | NFR-SCL-3 becomes 50 lots and 1,000 sale lines. NFR-SPD-7 and NFR-SPD-4 hold at that size. |
| D1 | Digits: Urdu prints use 0 to 9 only. The Printing setting "Urdu digits" and all code behind it are removed. NFR-URD-14 and FR-7.32 change. Amounts in words stay Urdu words. |
| K1 | Backspace and Delete (FR-7.14, FR-7.15, new FR-7.91): only delete. On a grid cell that is not being edited they clear an editable cell. They never leave a screen. |
| L1 | Party ledgers: a new module `party_ledgers`, FR-12.43. Level 3 and its print reuse the R35 day slip (FR-19.12), so the figures cannot differ. A person in the dead amount is listed with his balance, marked ڈوبی. |
| E1 | Find: `bikri_find` also takes a buyer (FR-12.13). Edit: the buyer and seller of a line or lot can change where FR-7.70 allows; locks stay as written (a later sale resting on a lot). |
| R1 | The R5 button sits next to the seller-bill print on the بکری page (FR-12.12). |
| M1 | Mac printing, Mac phase only: on the Mac, Print and the printing of PDFs go through the native print dialog of the web view, whose PDF button saves a PDF. Same templates. No new `unsafe`: CLAUDE.md rule 5 stands, and direct one-click PDF on the Mac is left out. The Windows path does not change. FR-12.12 and FR-19.3 get a Mac-phase note. |
| A1 | The assistant's words change پختہ to چکتا and learn موقع پر جمع; R9 leaves its catalogue. D6-system-prompt.md is updated here and the four prompt files are copied word for word in plan 64. `MAX_PREFIX_TOKENS` stays 11,600. |

### 3.4 Golden tests

Allowed changes only: wording where پختہ becomes چکتا in E2E-5, E2E-10, E2E-15, E2E-36, E2E-43; E2E-21
replaced. No expected number changes. New tests E2E-64 to E2E-69 are added for D8.

| New | Check | Expected |
|---|---|---|
| E2E-21 (replaced) | Two چکتا buyers in one day, شاہد 1,000 and ناصر 2,000, each line paid in full | Neither has a balance or a due; 0030 ends at 0; both are in the day's sale book |
| E2E-64 | Image 1 of D8: 200 of 200 sold in three lines with the rate of line 3 equal to the hint | Line totals 21,360 / 48,100 / 125,060; خام بکری 182,000; کل اخراجات 21,920; صافی بکری 160,080; shared income 23,440; split 9,376 / 7,032 / 7,032 |
| E2E-65 | E2E-64 with paid 21,360 / 10,000 / 0, and ہارون holding 10,000 جمع before the sale | Cash up 31,360; left 0 / 38,100 / 125,060; ہارون after the sale 115,060 بنام |
| E2E-66 | Old data: a line saved as cash and a line saved as credit before migration | paid = total and paid = 0; every balance as before |
| E2E-67 | Party ledgers after Events 1 to 5 | Buyers اکرم 2,880 بنام, ارشد 18,684 بنام; اکرم on 10-10-26: previous 0, charged 12,880, paid 10,000, balance 2,880 |
| E2E-68 | بکری 1 line 1 moved from اکرم to طارق after Events 1 to 5, then back | اکرم 10,000 جمع; طارق 12,880 بنام; commission and shares unchanged; trial balance 95,048; back gives 85,048 |
| E2E-69 | Urdu PDFs of R1, R5, R35 | No ۰ to ۹ or ٠ to ٩; "10,000" and "2026" present |

Check of 95,048 (E2E-68): بنام = 58,484 + 12,880 (طارق) + 18,684 + 3,000 + 2,000 = 95,048. جمع = 85,048 + 10,000 (اکرم) = 95,048.

## 4. Documents to change (D8 Section 5, rule 2)

| Document | Version | What |
|---|---|---|
| `.claude/rules/PRD_Mandi_Software_v1.md` | 1.32 | All of Section 3 above. Glossary, Sections 4.5, 7, 10, 12, 13, 15, 19, 21, 23, 24, 26, 27, 29 and every other mention. |
| `.claude/rules/NFR_Mandi_Software_v1.md` | next | NFR-SCL-3, NFR-USE-6 (4 fields, 4 Enters when a line may take credit), NFR-URD-14 |
| `.claude/rules/Code_Standards_v1.md` | next | Golden list, the پختہ mentions, the field rule for paid |
| `.claude/rules/Architecture_v1.md` | next | Data dictionary (`bikri_sales.paid_paisa`, `chukta_buyers`, renamed columns), diagrams 9 and 11, change log |
| `.claude/rules/Tech_Stack_v1.md` | next | Module table (`pukhta_khata` out, `party_ledgers` in), digits |
| `.claude/agents/money-calculations.md` | next | The cash part and matching of paid on the spot, چکتا in place of پختہ |
| `docs/deliverables/D6-system-prompt.md` | next | چکتا, موقع پر جمع, R9 out, within the token cap |
| `.claude/CLAUDE.md`, `.claude/skills/print-and-export/SKILL.md` | | Wording |
| `docs/deliverables/status.md`, `progress.json` | | D8 entry |

D8 is Umar's approval for these changes (D8 Section 5, rule 2).

## 5. Report items for Umar

The defaults above, M1 (no one-click PDF on the Mac), P12 (a part-paid line is on both R3 and R4), C9.
