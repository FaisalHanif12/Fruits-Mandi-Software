# D2. Accounts, names in two scripts, rules and the opening position

Version 1.0, 10-10-2026. Deliverable 2 of 5. Read `README.md` in this folder first.
Depends on: D1 done. Plan files: `05-names-two-scripts`, `06-masters-and-search`, `07-journal-and-accounts`, `08-party-card-and-short-codes`, `09-commission-rules`, `10-opening-position`.
Build order step: CLAUDE.md MVP step 4, plus the commission rules and the opening position that every بکری needs.

## Decisions

None open. If a question comes up, follow README Section 8.

## 1. What Umar can do at the end

He opens انڈکس and adds a buyer by typing `tariq mehmood` in English letters. The Urdu name field shows طارق محمود, the English name "Tariq Mehmood" fills by itself, and the account gets code 03040003. In any buyer field he types `3` and presses `Enter`: طارق محمود's party card appears, with his balance. Typing `tarq` finds him in both languages.

He adds cities, items (کیلا as پھل, مولی as سبزی), packings, اڈے and the other masters, each with names in both scripts. He opens commission rules: seller 6%, buyer 6%, and لاگا 8 per نگ on the پھل group are already there. "Test a rule" shows which rule wins for a seller, buyer and item, and why.

He types the opening balances of PRD 4.3, gives رحیم a 15,000 پیشگی on 01-09-26, and closes the days up to 09-10-26. The trial balance shows 65,000 on both sides, ابتدائی فرق 0, and cash 50,000. The Owner reopens a day with a reason, and the audit log shows it.

## 2. Plan files

### 05-names-two-scripts: typing names in Urdu and English

**Requirements:** PRD 7.4 (FR-7.21 to FR-7.28), FR-7.25; Settings "Language" (learned words list); NFR-URD-17, NFR-URD-20, NFR-URD-24 to NFR-URD-26, NFR-SPD-20; Architecture 19; Tech Stack 4.4; Code Standards 5.5 and 6.10.

**backend builds:**

- `core::translit`, pure: English letters to Urdu suggestions, and Urdu to English names. Learned pairs are passed in (CLAUDE.md gotcha).
- `services/translit`: loads and saves learned word pairs (FR-7.26). A learned word is used from the very next word and survives a restart (NFR-URD-26).
- The starting word list (FR-7.27): common Pakistani first names, family names, mandi items, cities. A data file, with each entry marked for Urdu review (NFR-URD-4).
- The fixed test list of 1,000 mandi names for the accuracy tests NFR-URD-24 and NFR-URD-25.
- Script checks (FR-7.28): an Urdu name holds Urdu letters, an English name holds English letters.

**frontend builds:**

- `RomanUrduInput`: the three ways to type Urdu without any Windows setting (FR-7.21): the built-in phonetic keyboard, the on-screen keyboard (NFR-URD-17), and English letters that become Urdu with a suggestion list (FR-7.22). Only Urdu text fields convert. Pickers never do (CLAUDE.md gotcha).
- `NamePair`: the Urdu name and the English name side by side. Filling one fills the other by itself; the user can change either (FR-7.24).
- The learned words list in Settings, where the Owner can see and remove learned pairs.

**Done when:** the 1,000-name tests meet NFR-URD-24 and NFR-URD-25, suggestions show within NFR-SPD-20, and `tariq mehmood` gives طارق محمود and "Tariq Mehmood".

### 06-masters-and-search: other masters, pickers and the search index

**Requirements:** PRD 10.3 (every master in its table), FR-7.10, FR-7.16, FR-7.35, FR-7.40, FR-7.41; NFR-SPD-6, NFR-SPD-18, NFR-URD-18 to NFR-URD-20; Code Standards 5.5; Architecture 20.

**backend builds:**

- Add, edit and make inactive for every master in PRD 10.3: city, item (with item group پھل or سبزی, default packing, sold by), packing, اڈا, mark, store, standard narration, expense head, staff, bank account. A used master cannot be deleted.
- The in-memory search index (CLAUDE.md gotcha): rebuilt at start-up and after a restore, updated on every save. It finds by code, Urdu name, English name and near Roman spellings with small mistakes.
- Recent records per user for "recent first" (FR-7.41).

**frontend builds:**

- The masters screens, each with `NamePair`, a list with smooth scrolling for thousands of rows (NFR-SPD-18), and the four states.
- The shared `Picker`: search as you type (FR-7.10, FR-7.40), the keyboard rules of FR-7.16, `Alt+↓` for recent (FR-7.41), results in both languages (FR-7.35). Every later picker is built on it.

**Done when:** a picker over 20,000 test accounts meets NFR-SPD-6, including small spelling mistakes and both scripts.

### 07-journal-and-accounts: the journal, the chart of accounts and accounts

**Requirements:** PRD 6 (FR-6.1 to FR-6.6), 10.1 and 10.2 (FR-10.1 to FR-10.3, FR-10.5 to FR-10.7, FR-10.9), FR-7.63, Settings "Codes"; NFR-REL-8, NFR-REL-11, NFR-SEC-5, NFR-SEC-6, NFR-SCL-1; Architecture 6, 13 and 15.4; Code Standards 3, 5.3 and 7. E2E-23.

`backend` calls the money-calculations agent before writing the journal engine and again to review it.

**backend builds:**

- The journal engine: an entry is saved as draft, checked (بنام equals جمع, FR-6.2), then posted. Posted rows never change (FR-6.3). Corrections are reversals. Each entry carries a keyed hash (NFR-SEC-6). Every posting date passes the open-day check of FR-6.6.
- Posting events (PRD 6.1) as the one way any module writes to the books. Every screen's totals are read from posted records (FR-6.1).
- The books-balance check used by every later save and by the daily check (NFR-REL-8). Trial balance and ledger queries as service functions. (Their report screens come in D5.)
- Chart of accounts in three levels: head, control, account (FR-10.1). Account codes = control code + 4-digit serial (FR-10.2), with the serial rules of FR-10.3.
- Accounts: every field of PRD 10.2, including the credit limit and the "opening debit is an advance" mark used later by FR-15.8. Inactive instead of delete once used (FR-10.6). Duplicate warning and similar names while typing (FR-10.7, FR-7.63).
- Opening balances post as one opening entry against ابتدائی فرق 06019999 (FR-10.9).
- Audit rows for every account, opening and code change (NFR-SEC-5).

**frontend builds:**

- Chart of Account Head, Control and Tree screens (FR-10.1).
- The account list with the columns of FR-10.5, and the account form with `NamePair`, city picker and opening balance fields.
- An opening balances grid (with the `entry-grid` skill) for typing many openings at once.

`backend/crates/services/tests/assist.rs` starts here, with the backend part of E2E-23 (Code Standards 9.2). The screen part runs in the Windows suite.

**Done when:** E2E-23 passes (the new buyer gets code 03040003 when two buyers already exist), and a property test shows that any mix of opening entries keeps the trial balance equal.

### 08-party-card-and-short-codes: a code brings the whole party

**Requirements:** FR-7.38, FR-7.39, FR-7.42, FR-7.64, FR-10.8, FR-5.4 (accounts part); NFR-SPD-21, NFR-USE-25; Architecture 20.

**backend builds:**

- The party card data: name in both scripts, city, phone, balance from the database (never from the index), credit limit and how much is used. Under NFR-SPD-21 for 10 years of data. Oldest open bill and payables are added in D4 when vouchers exist.
- Short codes (FR-7.39): in a seller field the serial alone is enough; in a buyer field the same. A short code is taken on `Enter` or on leaving the field, not while typing (`1` can be the start of `12`).
- Unknown code (FR-7.42) with an offer to add.
- Global search results for accounts (FR-5.4). Later deliverables add their own record types to it.

**frontend builds:**

- `AccountPicker`, filtered by account type, on top of the shared `Picker`.
- `PartyCard`, shown as soon as a code is complete.
- Quick add with `F4` or "+" in any picker (FR-10.8), and adding a missing city or اڈا from the account form (FR-7.64), without leaving the screen.
- The global search box in the top bar (`Ctrl+K`), for accounts.

**Done when:** typing `1` then `Enter` in a buyer field brings 03040001 with his card, and the unknown-code path adds a new account and returns to the field.

### 09-commission-rules: which rule wins

**Requirements:** PRD 11 (FR-11.1 to FR-11.10), 4.2 defaults, Settings "Charges" and "Overrides"; NFR-SCL-6, NFR-MNT-5, NFR-SEC-5; Architecture 8; the money-calculations agent.

**backend builds:**

- Rules with the fields of 11.1 and the charges of 11.2. Every rate is data, never code (NFR-MNT-5).
- Rule resolution, most specific first, at six levels (PRD 11.3): line override, party + item, party, item, item group, global default. Under 1 ms with 10,000 rules (NFR-SCL-6).
- The seeded defaults of PRD 4.2: seller 6%, buyer 6%, لاگا 8 per نگ as an item-group rule on پھل, and no default for مزدوری, منشیانہ, فون, کرایہ or کرایہ سٹور.
- "Test a rule": for a chosen seller, buyer, item and date, it returns the winning rule for each charge and the levels it skipped.
- Audit rows for every rule change.

**frontend builds:** the rules list and form, the "test a rule" panel, and the Charges and Overrides settings.

**Worked check the tests must contain:** seller رحیم, buyer اکرم, item کیلا (پھل): seller 6%, buyer 6%, لاگا 8 per نگ. The same with مولی (سبزی): no لاگا. Add a rule "اکرم buys at 7%" and the test shows the party level winning over the global default.

**Done when:** each of the six levels has a test that shows it winning over the levels below it.

### 10-opening-position: the first voucher, open days and the golden setup

**Requirements:** PRD 4.3; PRD 15.1 and FR-15.1 (for پیشگی); 15.4 (giving an advance; the adjustment comes in D3 and D4); FR-7.60; FR-21.1 and FR-21.2 (day close and reopen, first version); FR-6.6; NFR-SPD-9; Code Standards 9.2. E2E-19.

**backend builds:**

- The voucher framework every voucher type will use: numbering per type (پ-1, پ-2 ...), the common fields of 15.1, the running list of FR-15.1, current and new balance.
- The پیشگی voucher: seller بنام, cash or bank جمع, as an advance record. Under NFR-SPD-9.
- Day close: closes the oldest open day, or several in order, and moves the "closed up to" date forward (FR-21.1). Its summary in D2 shows what exists now: trial balance difference (must be 0) and cash in hand. D4 adds the بکری and stock lines, and D5 starts the cloud backup.
- Reopen: Owner only, with a reason; reopening a date reopens every later date; written to the audit log (FR-21.2).
- `pnpm seed` builds the PRD 4.3 opening position through the real services, exactly as 4.3 says, plus the masters and parties the worked day needs (رحیم, کریم, اکرم, ارشد, کیلا, سیب, پیٹی, سوات اڈا ...), with names in both scripts.
- `backend/crates/services/tests/golden_day.rs` starts here, with the 4.3 setup and E2E-19. Golden tests set a fixed "today" through the injected clock (the domain never reads the clock, Code Standards B1): 31-08-26 and 01-09-26 for the setup, 10-10-26 for the worked day, and later 11-10-26 for E2E-16b.

**frontend builds:** the پیشگی screen in the بکری side menu, with the advance history on the seller's card (FR-7.60); the day close and reopen screens. (The same پیشگی screen is linked from the روکڑ area in D4, when that area exists; FR-15.10.)

**Done when:** E2E-19 passes: trial balance 65,000 = 65,000; ابتدائی فرق 0; advances open 15,000; cash 50,000.

## 3. Design for D2

The lead runs `/design` inside `docs/design/design-system.md` for:

1. The account form with `NamePair` while English letters turn into Urdu, and the suggestion list.
2. The on-screen Urdu keyboard.
3. A picker open with results in both scripts, and the party card next to a field.
4. The chart of accounts tree.
5. The commission rules list, the rule form and the "test a rule" panel.
6. The پیشگی voucher list screen and the day close summary.

The party card and the picker appear on almost every later screen, so they get the most care: fast to read, clear balance colour, nothing hidden (NFR-USE-27).

## 4. Team flow for D2

Plan files run in order (README Section 4), and the order is forced by what each needs:

- 05 first: every later name field uses `NamePair`.
- 06 next: the shared `Picker` and the masters (cities, items) that accounts and rules use.
- 07: the journal and accounts.
- 08: `AccountPicker` and the party card, on top of 06 and 07.
- 09: the rules, which need `AccountPicker` for the party levels and items for the item levels (CLAUDE.md rule 9: no one-off pickers).
- 10 last: it needs accounts, pickers, the journal and the voucher framework.
- `backend` uses the money-calculations agent for 07, 09 and 10.

## 5. Review focus for D2

- The journal engine is the heart of the books. `reviewer` and the money agent check it line by line against PRD 6, Code Standards 3 and Architecture 6.
- Names: every new screen shows names only through `displayName(record, language)`; codes, phones and amounts never change on a language switch (FR-7.25, FR-7.32).
- The six rule levels against PRD 11.3, with a test for each.
- Short codes taken only on `Enter` or leaving the field.

## 6. Robustness attack list for D2

1. Names: mixed scripts, zero-width characters, Arabic look-alike letters, very long names, emoji, names that differ only in look-alike letters. Duplicate checks still catch them.
2. Translit: the 1,000-name accuracy (NFR-URD-24, NFR-URD-25); speed per key press (NFR-SPD-20); a learned word survives a crash right after learning.
3. Search: 20,000 accounts and 2,000 masters; picker speed (NFR-SPD-6); index rebuilt after a simulated restore gives the same results.
4. Journal: random sequences of opening entries, advances and reversals; the trial balance difference is always 0. An unbalanced entry is refused and nothing is written. Posting on a closed date is refused.
5. Codes: two accounts created at the same moment never get the same code; serials 0000 and 9000 to 9999 are refused for normal accounts (FR-10.3).
6. Rules: 10,000 rules; resolution under 1 ms (NFR-SCL-6); a rule with an invalid percent or a negative per-نگ value is refused.
7. Day close: closing out of order is refused; a reopen by a Munshi is refused; a reopen writes the audit row.

## 7. Acceptance (D2-A1 to D2-A16)

| ID | Item | Evidence expected |
|---|---|---|
| D2-A1 | `tariq mehmood` gives طارق محمود and "Tariq Mehmood"; the other script fills by itself | Test output and a screenshot |
| D2-A2 | 1,000-name accuracy meets NFR-URD-24 and NFR-URD-25 | Accuracy test output |
| D2-A3 | Learned words work from the next word and survive restart (FR-7.26, NFR-URD-26) | Test output |
| D2-A4 | All three ways to type Urdu work with no Windows setting (FR-7.21) | Windows E2E names |
| D2-A5 | Every PRD 10.3 master can be added, edited and made inactive, with names in both scripts | Integration and E2E test names |
| D2-A6 | Picker speed over 20,000 accounts meets NFR-SPD-6 at its runner threshold, in both scripts and with small spelling mistakes | `pnpm perf` output |
| D2-A7 | Journal: unbalanced entries refused; posted rows unchangeable; reversals; keyed hash checked | Test output |
| D2-A8 | Chart of accounts in three levels; codes and serial rules of FR-10.2 and FR-10.3 | Test output |
| D2-A9 | E2E-23 passes | Test name and result |
| D2-A10 | Short code `1` + `Enter` in a buyer field brings 03040001 and his card; card under NFR-SPD-21 | Test output and timing |
| D2-A11 | Quick add with `F4` and the unknown-code path add an account without leaving the screen | Windows E2E names |
| D2-A12 | Seeded rules match PRD 4.2; each of the six rule levels wins over the ones below it | Test output |
| D2-A13 | Rule resolution under 1 ms with 10,000 rules (NFR-SCL-6) | Benchmark output |
| D2-A14 | E2E-19 passes: 65,000 = 65,000; ابتدائی فرق 0; advances open 15,000; cash 50,000 | `cargo test -p mandi-services --test golden_day` output |
| D2-A15 | Day close in order; reopen by Owner only, with reason, in the audit log | Test output |
| D2-A16 | Every D2 screen matches the design in both languages at the three sizes; the full test suite, including every D1 test, passes on the final main | `windows-verify` report; full test run |

## 8. Not in D2

- Partners and the income split: D3.
- One-time people in پختہ کھاتہ fields (FR-7.29): D3.
- Advance adjustment when a بکری is saved (FR-15.8): D3.
- Oldest open bill and payables on the party card, وصولی and ادائیگی: D4.
- The day close summary lines for بکری and stock: D4. Cloud backup at day close: D5.
- Report screens for the trial balance, ledger and account index: D5.
- Party report buttons on the card (FR-7.65): D5.
