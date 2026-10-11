# Non-Functional Requirements (NFR)
# Mandi Software for Commission Agents (آڑھتی)

| | |
|---|---|
| Version | 1.2 (Umar's changes of 11-10-2026) |
| Date | 12 October 2026 |
| Status | Final for build |
| Client | Maher Adnan and Brothers, New Sabzi Mandi, Kot Radha Kishan |
| Depends on | `PRD_Mandi_Software_v1.md` (v1.2, the master) |
| Next documents | Tech Stack, Code Standards, Architecture, Money Calculations agent profile |

---

## 0. What this document is

The PRD says **what** the software does. This document says **how well** it must do it: how fast, how safe, how easy, how good the Urdu and the English are. Every requirement here has a number or a clear test, so we can prove it at the demo and before go-live.

Each requirement has an ID like `NFR-SPD-3` (area SPD = speed, item 3). The areas are:

| Code | Area |
|---|---|
| SCL | Scalability (growth of data and features) |
| SPD | Speed and performance |
| REL | Reliability (no crashes, no lost data) |
| BAK | Backup quality |
| SEC | Security |
| USE | Usability |
| URD | Urdu and English quality (both languages, typing, names) |
| MNT | Maintainability |
| INS | Installation and updates |
| OBS | Logs and diagnostics |

### 0.1 Notes where the direction needed adjusting

Umar asked to be corrected where needed. Four points:

1. **"Highly scalable" for a desktop app.** This app runs on one computer with one user at a time. It does not need to handle thousands of users like a website. For us, scalability means two things: the data can grow for 10+ years without slowing down, and new features can be added without breaking old ones. Section 1 sets targets for both.
2. **"Responsive" for a desktop app.** Responsive normally means "works on phones". Our app is desktop only, so responsive means: it looks right and stays usable on every laptop and monitor size the client may have (from a small 1366×768 laptop to a 1920×1080 monitor), at every zoom level, and when the window is resized. Section 6.4 sets this.
3. **"Decimal or non-decimal" for money.** Money must never be stored or calculated as normal decimal (floating point) numbers, because computers get small errors with them. Example: in floating point, 0.1 + 0.2 = 0.30000000000000004. Over thousands of sales, those errors become missing rupees, exactly like the old software's 2-rupee gap. Rules: every amount in this business is a **whole rupee** (PRD Section 22), and inside the software money is kept as a whole number of paisa (1 rupee = 100 paisa), so 1,050 rupees is stored as 105000. Details are in the Code Standards and the Money Calculations agent profile.
4. **"English typing should convert to Urdu."** No converter, free or paid, gets every Pakistani name right, because one English spelling can have several Urdu spellings ("Shahid" can be شاہد or شاھد; "Ali" is always علی). So the app never converts silently. It shows the most likely Urdu spelling first, with two more choices, the user accepts with `Space` or `Enter`, and every correction is learned (PRD FR-7.22, FR-7.26). Section 7.4 sets a measured accuracy target. This is faster and safer than silent conversion, which would save wrong names without anyone noticing.

### 0.2 Reference hardware

All speed targets are measured on this "slowest supported" machine, so they hold on the client's laptop:

| Part | Minimum (all targets must pass) | Recommended |
|---|---|---|
| Windows | Windows 10, 64-bit | Windows 11, 64-bit |
| Processor | Intel Core i3, 7th generation, or equal | Core i5 or better |
| Memory | 4 GB | 8 GB |
| Disk | 256 GB SSD (HDD supported, see NFR-SPD-12) | SSD |
| Screen | 1366 × 768 | 1920 × 1080 |
| Internet | None needed to work; any connection for cloud backup | Mobile data is enough |
| Power | | UPS (load shedding protection) |

---

## 1. Scalability (SCL)

### 1.1 How much data to expect

From the old system's backup screen (counts are approximate, the screen was blurry):

| Old table | Rows | Period | Per day |
|---|---|---|---|
| Sale lines (SDTL_B) | about 96,000 | 01-01-2018 to 10-10-2026 = about 3,200 days | 96,000 ÷ 3,200 ≈ **30 per day** |
| Journal lines (JDTL) | about 254,000 | same | 254,000 ÷ 3,200 ≈ **80 per day** |
| Accounts (CHART) | 1,572 | | |

So 10 years at today's pace is about 30 × 3,650 ≈ 110,000 sale lines. We plan for **more than 10 times that** (10 × 110,000 = 1,100,000, rounded up to 1,200,000), in case the business grows or the old history is imported.

| ID | Requirement | Target |
|---|---|---|
| NFR-SCL-1 | Data volume handled with no change in speed targets | 1,200,000 sale lines, 3,500,000 journal lines, 20,000 accounts (account numbering allows 8,999 sellers and 8,999 buyers, PRD FR-10.3, plus other accounts), 500 items, 20 years of dates |
| NFR-SCL-2 | Daily volume handled | 500 بکری and 5,000 sale lines in one day |
| NFR-SCL-3 | Lines in one بکری | Up to 50 lots and 500 sale lines, with no visible lag while typing |
| NFR-SCL-4 | Database size at NFR-SCL-1 | Under 2 GB on disk; compressed backup under 500 MB |
| NFR-SCL-5 | Partners | Up to 10 partners, any share split |
| NFR-SCL-6 | Commission rules | Up to 10,000 rules; finding the winning rule for a line under 1 ms |
| NFR-SCL-7 | Feature growth | A new module (for example crate tracking) can be added as a new folder in frontend and backend, without editing existing modules except to register it (see Tech Stack and Architecture) |
| NFR-SCL-8 | Database changes | Every change to the database design is a numbered migration that upgrades old data by itself. An app version must open data from any older version. |
| NFR-SCL-9 | Future multi-computer option | Not built now, but nothing in the design may block it later: all business logic sits in the backend behind clear commands, never in the screens |

How we test SCL-1: a script fills a test database with 1,200,000 realistic sale lines, then all speed tests in Section 2 run on it.

---

## 2. Speed and performance (SPD)

All targets are "95% of the time or better" (p95) on the reference hardware, with the NFR-SCL-1 test data, except where a target names a data size.

| ID | Action | Target |
|---|---|---|
| NFR-SPD-1 | Cold start (double-click to login screen) | Under 2 seconds |
| NFR-SPD-2 | Login to dashboard fully loaded | Under 1 second |
| NFR-SPD-3 | Open any screen from the menu | Under 300 ms |
| NFR-SPD-4 | Key press to character on screen, in any field or grid cell | Under 50 ms (feels instant) |
| NFR-SPD-5 | Arrow key move between cells or fields | Under 30 ms |
| NFR-SPD-6 | Picker search as you type (20,000 accounts), including small spelling mistakes and both names (PRD FR-7.40) | Results under 50 ms |
| NFR-SPD-7 | Live summary update after changing a sale line | Under 150 ms from the last key press (this includes a 50 ms pause that waits for the user to stop typing) |
| NFR-SPD-8 | Save a بکری with 30 sale lines (all postings, split, analytics) | Under 300 ms |
| NFR-SPD-9 | Save any voucher | Under 150 ms |
| NFR-SPD-10 | Any report for one day | Under 1 second |
| NFR-SPD-11 | Party ledger for 10 years | Under 2 seconds |
| NFR-SPD-12 | Any analytics view over 5 years | Under 1 second on SSD, under 3 seconds on HDD |
| NFR-SPD-13 | Trial balance for 10 years | Under 2 seconds |
| NFR-SPD-14 | Print preview of a 15-page report | Under 2 seconds |
| NFR-SPD-15 | Local backup (15-minute copy) | Snapshot under 5 seconds for data up to 300 MB (normal volume); the check after it (NFR-BAK-5) runs in the background. The user can keep working throughout. |
| NFR-SPD-16 | Memory use | Under 400 MB in normal use |
| NFR-SPD-17 | Long tasks (big export, restore) | Show progress, can run in the background, never freeze the screen |
| NFR-SPD-18 | Lists with thousands of rows | Scroll smoothly (only visible rows are drawn) |
| NFR-SPD-19 | Language switch (PRD FR-7.30), on any screen including a بکری with 30 unsaved lines | Whole screen redrawn in the other language and direction under 500 ms. No restart. Nothing typed is lost. |
| NFR-SPD-20 | English-to-Urdu suggestions while typing a word | Under 50 ms after each key press |
| NFR-SPD-21 | Party card after a code is complete (PRD FR-7.38), 10 years of data | Under 100 ms |
| NFR-SPD-22 | Attention list count (PRD FR-7.44) | Updated within 1 second after any save; the work never slows typing |
| NFR-SPD-23 | Rate hint and seller or truck memory (PRD FR-7.49 to FR-7.53) | Shown under 100 ms after the field is filled |

Everyday comparison for SPD-4: a calculator responds the moment you press a key. If typing in the app ever feels slower than that, it fails.

How we test: automated timing tests run before every release on the reference hardware. A release that misses any target is not shipped.

---

## 3. Reliability (REL)

### 3.1 Two key terms, with examples

- **RPO (how much work can be lost):** if the laptop is stolen at 4:00 PM and the last backup was at 3:45 PM, the work between 3:45 and 4:00 is lost. That gap is the RPO.
- **RTO (how long to get working again):** if the laptop dies, how long until the business is entering sales again on another computer.

### 3.2 Requirements

| ID | Requirement | Target |
|---|---|---|
| NFR-REL-1 | Saved work survives a power cut, Windows crash or app crash | 100%. A save is either complete or did not happen. Never half-saved. |
| NFR-REL-2 | Unsaved typing survives a power cut or crash | Draft kept, at most the last 5 seconds lost |
| NFR-REL-3 | RPO if the computer itself is lost or destroyed | At most 1 day (last cloud backup). At most 15 minutes if the local backup folder is on a second disk or USB that survives. |
| NFR-REL-4 | RTO on a new computer | Under 30 minutes: install, enter cloud key, restore, work |
| NFR-REL-5 | Crashes in normal use | Zero known crashes at release. Any crash found is a top-priority bug. |
| NFR-REL-6 | Error handling | No error ever closes the app. Every error shows a clear Urdu message with a reference number, and is written to the log. |
| NFR-REL-7 | Data checks | At start-up a quick check of the data file runs in the background (the login screen still appears within NFR-SPD-1); saving is held until it passes. A full check runs after each daily backup. If a problem is found, the app stops saving and offers to restore the latest good backup. |
| NFR-REL-8 | Books balance | The trial balance difference is 0 at all times. A daily automatic check confirms it and alerts if not. |
| NFR-REL-9 | Two copies of the app | Cannot happen. Opening the app a second time brings the first window to the front. |
| NFR-REL-10 | Disk full | Detected before saving. Clear message. Nothing half-saved. |
| NFR-REL-11 | Clock changed on the computer | Entry dates come from the user's chosen date. Audit times use the system clock and a warning shows if the clock moves backwards. |
| NFR-REL-12 | Long running | Runs all day (12+ hours) without slowing down or memory growth |
| NFR-REL-13 | Update safety | Before any update changes the data design, a full backup is taken. If the data upgrade fails, the data is restored from that backup by itself and the app shows how to reinstall the previous version (the installer of the previous version is kept on the computer). |

---

## 4. Backup quality (BAK)

The PRD says backups are automatic. This section says how good they must be.

### 4.1 The 3-2-1 rule

Keep **3** copies of the data, on **2** different kinds of storage, with **1** copy away from the shop. Example: the live data on the laptop, a copy on a USB drive, and a copy in the cloud. If the shop floods, the cloud copy survives. If the internet company loses the cloud copy, the USB survives.

### 4.2 Requirements

| ID | Requirement | Target |
|---|---|---|
| NFR-BAK-1 | Copies | 3 (live, local or USB, cloud) |
| NFR-BAK-2 | Local backup frequency | Every 15 minutes while data changes, plus on exit |
| NFR-BAK-3 | Cloud backup frequency | At least once a day (closing time and app exit) |
| NFR-BAK-4 | Consistency | Every backup is a complete, consistent snapshot, taken safely while the app is in use. Never a copy of a half-written file. |
| NFR-BAK-5 | Verification | Every backup is opened and checked (data check + record counts + checksum) right after it is made. A backup that fails the check is marked bad and an alert shows. |
| NFR-BAK-6 | Encryption | Cloud and USB copies are encrypted before they leave the computer. Without the key, nobody (including us or the cloud company) can read them. |
| NFR-BAK-7 | Key safety | The key is shown once at setup and printed. The app asks the Owner to confirm a test restore with the printed key in the first week. |
| NFR-BAK-8 | Retention | Local: last 48. USB: last 30. Cloud: 30 daily, 12 monthly, 1 per year kept forever. |
| NFR-BAK-9 | Offline behaviour | If there is no internet, cloud backups queue and retry every 10 minutes. No backup is ever skipped silently. |
| NFR-BAK-10 | Visibility | Status bar always shows the last good cloud backup time. Red if older than 24 hours. Dashboard alert if older than 48 hours. |
| NFR-BAK-11 | Restore drill | A "test restore" restores the latest cloud backup into a separate test copy and compares totals. Owner reminded monthly. |
| NFR-BAK-12 | Restore speed | Restoring 10 years of data takes under 5 minutes |
| NFR-BAK-13 | Readable copy | A daily pack of PDF and spreadsheet files (روزنامچہ, buyers, sellers, expenses, partner shares) is kept with each cloud backup, so data can be read even without the app |
| NFR-BAK-14 | Cost | At normal volume (data under 300 MB, compressed copies under 100 MB) the retained cloud copies fit in the provider's 10 GB free tier. At the NFR-SCL-1 maximum they would not, and a small monthly fee applies. The provider needs a payment card on file even for the free tier (see Tech Stack). |
| NFR-BAK-15 | No manual steps | Nobody has to remember anything. Backups happen whether or not the user clicks anything. |

---

## 5. Security (SEC)

The data is commercially sensitive: who owes how much, each partner's share, each seller's rates. The main risks are a staff member changing records, a stolen laptop, and a lost backup.

| ID | Requirement | Target |
|---|---|---|
| NFR-SEC-1 | Passwords | Stored only as a strong one-way hash (Argon2id). Minimum 6 characters. Never shown, never logged. |
| NFR-SEC-2 | Login lockout | As PRD FR-8.2: after 5 wrong tries wait 1 minute, after 5 more wait 5 minutes, then 15 minutes each time |
| NFR-SEC-3 | Idle lock | Locks after 15 minutes idle (setting). Unsaved work stays. |
| NFR-SEC-4 | Sign-in enforced in the backend | There is one user, the Owner, and no roles (PRD Section 2). Every command except first run, sign-in, recovery, app info and data status is refused by the backend unless the Owner is signed in and the app is not locked. Hiding a button is not enough. |
| NFR-SEC-5 | Audit trail | Every change to money, rules, shares, the password and settings is logged with who, when, old value, new value, reason. The log cannot be edited or deleted from the app. |
| NFR-SEC-6 | Tamper evidence | Each journal entry and each audit entry carries a keyed hash of its own content plus the previous entry's hash (a chain). The key is kept in Windows Credential Manager, not in the data file. If anyone changes journal or audit records outside the app without the key, the chain no longer matches and the app warns the Owner at the next check. Other tables are protected by the journal: every money table can be rebuilt and compared from the journal. |
| NFR-SEC-7 | Secrets | Cloud keys and the backup encryption key are stored in Windows Credential Manager, never in plain files or the database |
| NFR-SEC-8 | Data file protection | Recommended: Windows BitLocker on the laptop disk, so a stolen laptop does not expose the data. The app shows this as a setup tip. |
| NFR-SEC-9 | No network exposure | The app opens no network ports. The only outgoing connections are cloud backup and update checks. |
| NFR-SEC-10 | App lockdown | The screens can only call the backend commands they are allowed to. No access to files or the system outside what the app needs. |
| NFR-SEC-11 | Signed updates | Updates are digitally signed. The app refuses an update that is not signed by us. |
| NFR-SEC-12 | Recovery | Owner recovery code printed at setup. No "master password" exists anywhere. |
| NFR-SEC-13 | Exports | Excel and PDF exports need the signed-in Owner, like every other command (NFR-SEC-4) |

---

## 6. Usability (USE)

The client's main complaint was poor user experience. These targets make "easy to use" measurable.

### 6.1 Learnability

| ID | Requirement | Target |
|---|---|---|
| NFR-USE-1 | First بکری without training | A person who has never seen the app enters بکری 1 from the PRD worked example in under 10 minutes, using only on-screen hints |
| NFR-USE-2 | Trained speed | A trained munshi enters a 10-line بکری in under 90 seconds using only the keyboard |
| NFR-USE-3 | Learning time | A munshi who used the old software runs a full day alone after 1 day of training |
| NFR-USE-4 | Mistakes | In testing, users make at most 1 uncorrected error per 50 entries |
| NFR-USE-5 | Satisfaction | Munshi and owner rate the app 4 out of 5 or better on "easy to use" at the demo |

### 6.2 Efficiency

| ID | Requirement | Target |
|---|---|---|
| NFR-USE-6 | Keystrokes | A sale line (buyer code, qty, rate) takes at most 3 fields to type and 3 `Enter`s. Everything else fills itself. |
| NFR-USE-7 | Mouse not needed | Every task in the PRD can be done with keyboard only (arrows, Enter, Esc, F-keys) |
| NFR-USE-8 | Mouse also fine | Every task can also be done with mouse only. Keys are never the only way. |
| NFR-USE-9 | Clicks to common tasks | New بکری, وصولی, ادائیگی, پیشگی, خرچہ: 1 click or 1 key from anywhere (PRD Section 7.3: `Alt+B`, `Alt+V`, `Alt+A`, `Alt+P`, `Alt+K`) |
| NFR-USE-25 | Finding a party | Any party's details and balance appear with his code (or short code) and nothing else, on every screen (PRD FR-7.38, FR-7.39) |
| NFR-USE-26 | A repeat seller's lot | Seller code and quantity are the only fields typed when he sends the same item and packing as last time (PRD FR-7.51) |
| NFR-USE-27 | Nothing hidden | Every value the app fills by itself is visible on the screen where it is saved, and can be changed there. Checked in the screen review. |

### 6.3 Consistency and clarity

| ID | Requirement |
|---|---|
| NFR-USE-10 | The same action looks and works the same on every screen (same button place, same key, same word) |
| NFR-USE-11 | One design system (colours, spacing, fonts, components) used everywhere. No one-off styling. |
| NFR-USE-12 | Colours have one meaning each: green = money in or balanced, red = due or error, amber = warning. Colour is never the only signal (an icon or word goes with it). |
| NFR-USE-13 | Every error message says what went wrong and what to do, in plain words in the app language |
| NFR-USE-14 | No technical words on screen (no "null", "record", "transaction", "error code") |

### 6.4 Screen sizes ("responsive" for desktop)

| ID | Requirement | Target |
|---|---|---|
| NFR-USE-15 | Supported window sizes | The app content area works from 1024 × 600 up to 2560 × 1440 (in screen points after Windows scaling). Every screen is fully usable with no sideways scrolling anywhere in that range. |
| NFR-USE-16 | Window resize | Layout adjusts smoothly when the window is resized. The window cannot be made smaller than 1024 × 600. Nothing overlaps or gets cut off. |
| NFR-USE-17 | Windows display scaling | Supported combinations: 1366 × 768 at 100% and 125%; 1920 × 1080 at 100%, 125% and 150%. (1366 × 768 at 150% leaves less than 1024 points of width and is not supported; the app shows a tip to use 125%.) |
| NFR-USE-18 | In-app zoom | Text size 80% to 150% (`Ctrl +` / `Ctrl −`), remembered per user, but zooming in stops at the step where the content area would become narrower than 1024 points. Layout still aligned at every step. |
| NFR-USE-19 | Alignment | All fields, labels, columns and numbers line up on a common grid. Numbers are written left to right even inside the Urdu layout, and every number cell aligns its numbers to the **physical right edge** of the cell, with digits of equal width, so units, tens and hundreds line up under each other in every column. |

### 6.5 Accessibility

| ID | Requirement | Target |
|---|---|---|
| NFR-USE-20 | Text size | Numbers and Latin text: minimum 14 px in grids, 16 px elsewhere. Urdu Nastaliq text: minimum 18 px everywhere (Nastaliq looks smaller at the same size). |
| NFR-USE-21 | Contrast | Text contrast at least 4.5 : 1 (WCAG AA). Checked automatically. |
| NFR-USE-22 | Focus | The field or cell with keyboard focus is always clearly visible (thick outline) |
| NFR-USE-23 | Click targets | At least 32 × 32 px |
| NFR-USE-24 | Motion | No animation longer than 200 ms. No flashing. |
| NFR-USE-28 | Both languages | Every screen passes NFR-USE-15 to NFR-USE-24 in Urdu (right-to-left) and in English (left-to-right). Checked by screenshot tests of every screen in both languages. |

---

## 7. Urdu and English quality (URD)

The goal: a munshi reads the app as if it was written by an experienced Urdu-speaking accountant from the mandi. Not a translation. In English, a partner reads it as if it was written by a careful Pakistani accountant.

### 7.1 Language

| ID | Requirement |
|---|---|
| NFR-URD-1 | All screen text comes from translation files. No Urdu or English text is typed directly into code. |
| NFR-URD-2 | A locked glossary (from PRD Section 3, extended to 300+ terms) is the only source of business words. Terms like بکری، روکڑ، وصولی، اگراہی، پیشگی، لاگا، منشیانہ، صافی بکری are kept exactly. |
| NFR-URD-3 | No word-by-word machine translation in the final text. Machine translation may only produce first drafts of long help text, which a person then rewrites. |
| NFR-URD-4 | Review process: every new or changed Urdu string is reviewed by an Urdu-literate reviewer and then by the munshi before release. Each string records its reviewer. |
| NFR-URD-5 | Tone: short, polite, everyday mandi Urdu. Commands in the polite form (محفوظ کریں, not محفوظ کرو). |
| NFR-URD-6 | Grammar: correct gender and number agreement (for example "بکری محفوظ ہو گئی" but "واؤچر محفوظ ہو گیا"). Messages with numbers use correct forms. |
| NFR-URD-7 | Spelling: one fixed spelling per word across the app (for example always "واؤچر", never "ووچر" on one screen and "واؤچر" on another) |
| NFR-URD-8 | Acceptance: the munshi reads every screen at the demo. Zero words he does not understand. |

Everyday example of what we avoid: Google may translate "Save and New" as "محفوظ کریں اور نیا". A munshi would say "محفوظ کر کے نئی بکری". We use the second.

### 7.2 Script, fonts and layout

| ID | Requirement |
|---|---|
| NFR-URD-9 | Fonts are installed with the app (work offline): a Nastaliq font for labels, headings and printed bills; a Naskh font as an option for dense grids. Both under an open licence. |
| NFR-URD-10 | Text never breaks into separate letters, never shows boxes or question marks, and diacritics never overlap the line above or below |
| NFR-URD-11 | Line height for Nastaliq at least 1.8, so tall letters are not cut off in grid rows and printed tables |
| NFR-URD-12 | In Urdu the whole app is right-to-left: menus, grids, forms, dialogs, charts, prints. In English all of them are left-to-right (PRD FR-7.31). |
| NFR-URD-13 | Mixed text never jumbles: numbers, codes, truck numbers, phone numbers and Latin names inside Urdu text keep their order. Example: "(کیلا 16 @ 900)" always reads correctly, unlike the old report's "(575 . 1 کیلا)". |
| NFR-URD-14 | Digits: English digits (0-9) by default. Urdu digits (۰-۹) available as a setting for prints. |
| NFR-URD-15 | Amounts in words are grammatically correct Urdu, for every whole-rupee amount from 0 up to the largest allowed amount, 10,00,00,00,000 (دس ارب). Example: 19,316 = "انیس ہزار تین سو سولہ روپے"; 1,00,000 = "ایک لاکھ روپے". |
| NFR-URD-16 | Dates print as DD-MM-YYYY. Month names in the print language (جنوری، فروری ... or January, February ...). |

### 7.3 Typing and searching

| ID | Requirement |
|---|---|
| NFR-URD-17 | Built-in phonetic Urdu keyboard works without any Windows setting, with an on-screen key map available |
| NFR-URD-18 | Search treats look-alike letters as the same: Arabic ي and Urdu ی, Arabic ك and Urdu ک, ه and ہ, and with or without diacritics. Typing "اکرم" finds "اكرم". |
| NFR-URD-19 | Lists sort in correct Urdu alphabetical order (ا ب پ ت ٹ ...), not computer code order |
| NFR-URD-20 | Every name can be found by its English name, and by a near Roman spelling with small mistakes ("akram" and "akrm" find اکرم ٹول ٹیکس) |

### 7.4 English, names in two scripts, and English-to-Urdu typing

| ID | Requirement |
|---|---|
| NFR-URD-21 | English text is plain, professional English, with one fixed spelling per word (British spelling as used in Pakistan: Labour, Cheque). Mandi words use the PRD glossary's English column only. Reviewed by an English-literate reviewer and the Owner before release. |
| NFR-URD-22 | Every text key exists in both languages. The build fails if any key is missing, empty or left untranslated in either language. |
| NFR-URD-23 | Amounts in words are correct English for every whole-rupee amount from 0 to 10,00,00,00,000, in both groupings (PRD FR-19.7). Example: 19,316 = "Nineteen thousand three hundred sixteen rupees only". |
| NFR-URD-24 | English-to-Urdu accuracy (v1.2). The word list has at least 15,000 reviewed entries (PRD FR-7.27). On a fixed test list of 1,000 names typical of the mandi (people, firms, cities, items): for names in the word list, the correct Urdu spelling is the first suggestion for at least 95% and among the 3 suggestions for at least 99%; for names not in the word list (spelling rules only), the top-3 rate is reported on every build and must not drop below the last release. Every rules-only spelling is marked and confirmed before save (PRD FR-7.66), so a wrong spelling is never saved silently. The test list is kept in the repository and grows with every wrong conversion reported. |
| NFR-URD-25 | Urdu-to-English accuracy: on the same list, the suggested English name is accepted without change for at least 90%. |
| NFR-URD-26 | A learned word (PRD FR-7.26) is used from the very next word typed, and survives restart, update and restore. |
| NFR-URD-27 | The English font is installed with the app (works offline), with equal-width digits for number columns, under an open licence. |
| NFR-URD-28 | Mixed text in English screens and prints keeps its order: an Urdu note inside an English print reads right-to-left, and the numbers around it stay correct. |
| NFR-URD-29 | Urdu says exactly what English says (PRD FR-7.23). Each Urdu message is reviewed against its English message for the same meaning, the same numbers and the same business word from the glossary. Example: "ادھار رقم واپس ہو گئی" for "Money lent returned", never a looser phrase. Each pair records its reviewer (NFR-URD-4). |

---

## 8. Maintainability (MNT)

| ID | Requirement |
|---|---|
| NFR-MNT-1 | Code follows the Code Standards document. Automatic checks block code that does not. |
| NFR-MNT-2 | Each business module lives in its own folder in both frontend and backend, with the same name (Tech Stack Section 5.2) |
| NFR-MNT-3 | All money logic lives in one backend module and is covered by tests (see Money Calculations agent profile) |
| NFR-MNT-4 | Test coverage: 100% of money calculation code, at least 80% of backend code, all PRD end-to-end tests automated |
| NFR-MNT-5 | Every setting, rate and rule is data, not code. Changing a commission rate never needs a new app version. (Fixed business rules such as rounding are code, by design.) |
| NFR-MNT-6 | Documentation: each module has a short README; every public backend function has a doc comment |

---

## 9. Installation and updates (INS)

| ID | Requirement |
|---|---|
| NFR-INS-1 | One installer file. Installs in under 2 minutes with no internet needed. |
| NFR-INS-2 | First-run setup wizard in Urdu: firm details, Owner password, recovery code, partners, cloud backup key |
| NFR-INS-3 | Updates: the app checks for updates (when online), shows what is new in Urdu, and installs only when the Owner agrees. Never during work. |
| NFR-INS-4 | Updates keep all data and settings. A backup is taken first (NFR-REL-13). |
| NFR-INS-5 | Uninstall never deletes the data folder or backups |
| NFR-INS-6 | Move to a new computer: install, restore, done (NFR-REL-4) |

---

## 10. Logs and diagnostics (OBS)

| ID | Requirement |
|---|---|
| NFR-OBS-1 | The app keeps a log file of errors and important events, kept 30 days, maximum 50 MB |
| NFR-OBS-2 | Logs never contain passwords, keys or full account balances |
| NFR-OBS-3 | Each error shown to the user has a reference number that matches a log line |
| NFR-OBS-4 | A "مدد کے لیے معلومات" (support info) button collects the log, app version and system info into one file the client can send us |
| NFR-OBS-5 | Daily health check result (books balance, last backup, data check) visible to the Owner |

---

## 11. How each area is tested before release

| Area | How we test |
|---|---|
| SCL, SPD | Automated timing tests on the reference hardware with 1,200,000 test sale lines |
| REL | Pull the power cord during save (in a test machine), kill the app while typing, fill the disk, then check nothing is half-saved and drafts return |
| BAK | Automated backup and restore round trip; compare all totals; restore on a clean second computer |
| SEC | Sign-in tests (every command refused before sign-in and while locked), audit chain tamper test, check no plain secrets on disk |
| USE | 3 people who never saw the app try the PRD worked example; time them, count errors; test every supported size and scaling combination (NFR-USE-17) in both languages (NFR-USE-28) |
| URD | Urdu reviewer and munshi read every screen and print; English reviewer and Owner read every English screen and print; automatic tests for amounts in words (both languages), letter-variant search, missing text keys, and the 1,000-name English-to-Urdu list (NFR-URD-24, NFR-URD-25) |
| MNT | Code checks and coverage report in the build pipeline |

---

## 12. Change log

| Version | Date | Change |
|---|---|---|
| 1.0 | 10-10-2026 | Pre-build drafts and reviews: Umar's requirements, analysis of the old software video, research, client review comments and full cross-document reviews. |
| 1.1 | 10-10-2026 | **Final baseline for build.** From the start of development, every change raises the version (1.2, 1.3 ...) and adds a line here. |
| 1.2 | 12-10-2026 | PRD v1.2: one user, no roles (NFR-SEC-4, NFR-SEC-13, Section 11). Urdu accuracy: new NFR-URD-24 targets with a 15,000-word list and the spelling check; new NFR-URD-29. |
