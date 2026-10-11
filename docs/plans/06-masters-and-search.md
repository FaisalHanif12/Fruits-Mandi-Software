# Plan 06: Masters, the shared Picker and the search index

Deliverable: D2, plan file 06. Branch: `feature/masters-search`.

## 1. Goal

Every master list of PRD 10.3 can be added to, edited and made inactive, with names in both scripts. A shared `Picker` finds records as you type, in both scripts, forgiving small spelling mistakes, with recent records first. The in-memory search index answers 20,000 accounts in under 50 ms. Every later picker (accounts in plan 08, items and packings in the بکری) is built on this one.

## 2. Requirements

PRD 10.3 (every master in its table), FR-7.10, FR-7.16, FR-7.35, FR-7.40, FR-7.41. NFR-SPD-6, NFR-SPD-18, NFR-URD-18 to NFR-URD-20. Code Standards 5.5 (B26, B27), F7, F40. Architecture 15.4 and 20. Tech Stack 4.7. Acceptance: D2-A5, D2-A6.

## 3. Worked examples (tests)

| Case | Result |
|---|---|
| Add city "Kot Radha Kishan" typed `kot radha kishan` | `name_ur` کوٹ رادھا کشن, `name_en` "Kot Radha Kishan", search keys filled by `core::text` in the same transaction |
| Add item کیلا as پھل, مولی as سبزی | `item_group` fruit, vegetable; both found by `kela` and `کیلا` |
| Picker search `akrm` over 20,000 accounts | finds اکرم ٹول ٹیکس within 50 ms (one mistake, 4 letters) |
| Picker search `اکرام` | finds اکرم (Urdu look-alike and one mistake) |
| Order of results | exact code, then starts-with, then recent and frequent for this user, then close matches |
| Deleting a used master | impossible: there is no delete; a used master can only be made inactive (E21) |
| An inactive master | hidden from pickers, kept in reports and lists with a flag |

## 4. Contract

### Backend

- Masters (Tech Stack 5.2 names, Architecture 15.4 columns exactly): `cities`, `items` (item_group fruit or vegetable, default_packing_id, sold_by unit or weight), `packings` (approx_grams), `addas` (city_id), `markas` (code, mark_text as typed in one script, owner_account_id), `stores`, `narrations` (text_ur, text_en, used_in_json), `expense_heads` (account_id), `staff` (role munshi, collector or labour: the job, not a login role; phone; `account_id` null, the link to his ادھار رقم account under control 0103, filled when he is first lent money in D4, shown read-only on the staff form), `bank_accounts` (bank_name, title, number as typed; account_id). Every named record has `name_ur` and `name_en` NOT NULL, validated by `core::validate` (script rules FR-7.28), passes the spelling check of plan 05 (FR-7.66, `confirmed_rules_words`), and the three search columns written by `core::text` only, in the same transaction (B26). Codes: a unique `code` per master, next number suggested.
- `expense_heads` and `bank_accounts` also create their account (control 0501 and 0102, next free serial, names from the master) in the same transaction, through one repository function that plan 07's account service later wraps; their `account_id` points at it. Their names get the system `is_system = 0`.
- Commands (async, sign-in check first; thin): `masters_list { kind, include_inactive: bool }` -> `Vec<MasterRow>` (a tagged union by kind, fields per kind); `masters_save { kind, input }` -> `MasterRow` (create when `id` is null, else edit; `version` checked for edits); `masters_set_active { kind, id, is_active }` -> `MasterRow`; `masters_next_code { kind }` -> text. Errors: the validate codes with field paths, `ERR_CODE_TAKEN` (field `code`), `ERR_NOT_FOUND`, `ERR_VERSION_CHANGED` (edit of a stale row). Audit row on every create, edit and change of active (S7).
- No delete command exists for any master (D3 soft states only). A test proves the table has no path to delete from the app.
- Search index (`services/src/search`, in memory, B27): built at start-up and after a restore, updated right after any save that changes a name, code, phone or active flag commits. Each entry holds the three search keys, code, kind and the display fields. Matching order (Tech Stack 4.7): exact code, exact serial (accounts only, plan 08), starts-with on any key, recent and frequent for the user (from `user_prefs` key `recent.<screen>`, last 10 record ids, plus use counts), then close matches by `strsim` Damerau-Levenshtein (at most 1 mistake up to 5 letters, 2 above) on the Urdu key, the English key and the phonetic key. Normalisation by `core::text` for both the record and the query (look-alike Urdu letters, ي ك ه ۃ, diacritics, zero-width). Sort of equal rank by `core` Urdu collation. Under 50 ms for 20,000 entries (NFR-SPD-6): add a criterion bench with 20,000 generated accounts, and a timing test with a generous bound.
- Commands: `search_masters { scope, text, limit }` -> `Vec<SearchHit { id, kind, code, name_ur, name_en, extra_ur, extra_en, is_active }>` (`scope` is a master kind now; plan 08 adds seller, buyer and account), `search_recent { scope }` -> hits (the last 10 used on this screen, then the rest), `search_touch { scope, id }` -> () (records a use for "recent first"; the pickers call it when one is chosen). Read-only commands never write except `search_touch`, which writes the user pref only (B29).
- Index rebuild after a simulated restore returns identical results (test).

### Frontend

- Shared `Picker` (Downshift `useCombobox`, in `shared/ui`): search as you type (debounce to the key rate, previous request cancelled), rows show code, the name for the current language through `displayName`, the other script dimmed, and the extra line (father or firm, city, balance later); the keyboard rules of FR-7.16 (open list: up and down move, `Enter` picks, `Esc` closes; closed list: up and down move between fields; `Alt+↓` opens the full list with recent records first, FR-7.41); results in both scripts and a mistyped Roman word (FR-7.35, FR-7.40); the pickers never convert English letters to Urdu (F41); an optional "add new" action on `F4` hook (used by AccountPicker in plan 08); inactive records not offered; four states; virtualised for long lists (NFR-SPD-18); keeps its value when the language switches.
- A master list screen shell used by all ten masters (`features/masters`): grid list from `shared/grid` styles for read-only tables (virtualised, smooth with thousands of rows), filter box, show-inactive toggle, add and edit form with `NamePair`, code field with the suggested next code, make inactive with confirm, four states, keyboard only (`F2` new, `Enter` opens, `Esc` back). One generic screen configured per master, not ten hand-made ones. The انڈکس menu item opens the masters menu; their titles come from Fluent.
- Design: use the design system; the list and the form follow the FormUr/FormEn and GridUr/GridEn boards.
- Tests (ur and en): every picker key of FR-7.16; search results in both scripts against a fake; language switch keeps the picker value; keyboard-only add, edit and make-inactive for at least city, item, packing and expense head; the form's `NamePair` flow.

### Recorded during the build (12-10-2026)

- Every command takes one `input` object, like all earlier commands. `MasterInput` and `MasterRow` are unions tagged by `kind`: `masters_save { input: { kind: "city", ... } }`. `SearchScope` is its own enum with the ten master values; plan 08 adds seller, buyer and account.
- Migration 0001 has a UNIQUE index on `code` of every master that has one.
- New codes: `ERR_CODE_TAKEN`, `ERR_VERSION_CHANGED`, `ERR_FORMAT_INVALID`, `ERR_SERIES_FULL`.
- The criterion bench over 20,000 records (`backend/crates/services/benches/search.rs`) is written by `robustness` at the end gate; backend's timing test (debug build) gives about 4 to 5 ms per search and 69 ms in the worst case (a text that matches nothing).

## 5. Tasks

| # | Task | Owner |
|---|---|---|
| T1 | Contract types and stubs, `pnpm gen:types`, message frontend | backend |
| T2 | Migration additions if any, repositories, services, index, commands, tests, bench | backend |
| T3 | Picker, master list screen, ten master configurations, tests | frontend |
| T4 | Wire to the real commands | frontend |

No per-plan review (Umar, 11-10-2026).

## 6. Acceptance checklist

1. Every PRD 10.3 master can be added, edited and made inactive with both names, in a test and a screen test (D2-A5).
2. Picker speed over 20,000 accounts meets NFR-SPD-6 at its runner threshold, in both scripts and with small spelling mistakes (D2-A6; `pnpm perf`).
3. The index rebuilt after a simulated restore gives identical results.
4. `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm i18n:check` exit 0.

## 7. Out of scope

Accounts, code series and the party card (plans 07 and 08), the commission rules (09).

## 8. Open questions

None.
