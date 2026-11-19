# Plan 41: add any missing list entry on the spot

Version 1.0, 31-10-2026. Lead: tech-lead. Asked by Umar on 31-10-2026 while testing the خرچہ screen. Branch `feature/d5-early`, folder `Fruit Mandi Parallel/d5`. Owner: `frontend`. Backend is called in only if a save command lacks something the form needs.

Umar's words, in short: "There is no expense showing, so it is really weird. I go to another page like indexing, find the expense head list, add the expense and then come back. It is a simple step. We can add it here, it will save in the expense head list, and next time it should be shown in the drop down. I'm not talking only about the head list: any kind of drop down, like packing, items, whether it's in بکری, خرچہ, روکڑ. Our core task is to make the software easy for the users."

## 1. Goal

A missing record is added from the list where it is needed, with a short form, and is chosen at once. The user never leaves his entry.

## 2. Requirements

PRD v1.11:
- FR-7.69 (new);
- FR-7.10, FR-7.42, FR-7.64 and FR-10.8 (updated);
- E74, E75.

It also keeps to these:
- FR-7.24 (both names);
- FR-7.63 and FR-10.7 (similar names);
- FR-10.3 (code by itself);
- NFR-USE-7, NFR-USE-8 (keyboard and mouse);
- F7 and F40 (shared pickers only);
- F39 (nothing typed is lost).

This replaces plan 32 R8 and its decision D1, where the no-match button opened the full Index form and the user had to pick the new record himself.

## 3. What to build

R1. **One shared quick-add.** It lives in the shared picker base, so every entry picker gets it with no screen code. Every list ends with a "+ نیا ... شامل کریں" row. When text is typed, the row names it: "+ «بجلی» کو نئی خرچہ مد بنائیں". It is reached by a click, by `↓` then `Enter`, or by `F4` in the picker.

R2. **An empty list says so in words.** For example "ابھی کوئی خرچہ مد نہیں" / "No expense heads yet", with the add row under it. It never shows the empty box seen in Umar's screenshot of 31-10-2026.

R3. **The short form, per list** (PRD FR-7.69 table):

| List | Short form |
|---|---|
| Every list | Both names (NamePair), with the typed text in the name |
| Accounts | The tag |
| Seller, buyer | City and phone (both optional) |
| Item | Item group, and sold by (starts at نگ) |
| Packing | Approx weight (optional) |
| اڈا | City (optional) |
| مارکہ | The mark text. The owner is the lot's seller when it is opened from a lot |
| Narration | Urdu and English text |
| Staff | Job, and phone (optional) |
| Expense head, city, store | Names only |
| ادھار رقم person, الگ کھاتہ | Their existing F4 forms |

- The code is given by itself.
- Similar records show while the name is typed, and can be picked instead.
- A link "مکمل فارم" (Full form) opens the full Index form over the screen.
- `F10` saves and `Esc` closes. While the form is open, F10, Ctrl+Enter and Esc act only inside it (as plan 32 D5).

R4. **After save:**
- The new record is chosen in the field and the cursor carries on.
- The list caches are refreshed, so the record shows on every screen.
- The search index picks it up through the existing save path.

R5. **Where it appears:** every picker on an entry screen. That means:
- بکری (header, lots, lines);
- the voucher screen;
- خرچہ, پھل, ادھار رقم, الگ کھاتہ, سرمایہ, جنرل;
- trips and purchases, gardens and arrivals;
- commission rules;
- the account form (city, اڈا);
- partner setup, where it has pickers.

It does not appear on filter boxes: report and analytics filters, list filters, find, and the rule test (as plan 32 D5).

R6. Nothing changes in money, postings or saved data shapes.

## 4. Tasks

| # | Task | Owner |
|---|---|---|
| F1 | List every picker on every entry screen and what master it reads. Check each save command returns the new record (masters_save returns MasterRow; accounts_save). If one does not, tell me before building. | frontend |
| F2 | R1 to R5 in the shared picker base and one shared QuickAddDialog per list kind; remove the plan 32 "go to Index" button path (keep the full form behind "مکمل فارم"). Fluent ur and en (exact Urdu, plan 40 rule). | frontend |
| F3 | Tests: each list kind adds and chooses; empty list text; similar-name pick; Esc keeps the field; keyboard-only path; mouse path; language switch with the form open keeps what was typed; filter boxes have no add row. Screenshots ur and en at 1366 and 1024, including the خرچہ screen from Umar's picture. | frontend |

Targeted checks only (Umar's rule). The full `pnpm typecheck`, `pnpm lint` and `pnpm i18n:check` must be clean.

## 5. Acceptance

- On the خرچہ screen with no heads, typing بجلی and pressing Enter on the add row, then F10, saves the head and chooses it. The next خرچہ shows بجلی in the list.
- The same works for an item and a packing on a بکری lot, and for a city on the account form.
- Nothing typed on the screen underneath is lost.

## 6. Out of scope

Editing a record from the picker (it stays in the Index); new master fields.
