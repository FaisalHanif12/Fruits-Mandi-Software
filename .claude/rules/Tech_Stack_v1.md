# Tech Stack
# Mandi Software for Commission Agents (آڑھتی)

| | |
|---|---|
| Version | 1.8 (Umar's changes of 11-10-2026, 15-10-2026, 19-10-2026, 22-10-2026, 27-10-2026, 30-10-2026 and 01-11-2026) |
| Date | 1 November 2026 |
| Status | Final for build |
| Depends on | PRD v1.16 (the master), NFR v1.8 |
| Used by | Claude Code during the build; Code Standards; Architecture |

---

## 0. What this document is

It names every technology we use, why we chose it, and how the code is organised on disk. It has three parts:

1. **High level:** the big pieces (front end, back end, database, backup, printing).
2. **Low level:** the exact libraries, including everything needed for professional Urdu and English, typing in two scripts, and smart assistance.
3. **Repository:** the folder structure (with separate `frontend/` and `backend/` folders) and naming rules.

Version policy: we use the latest **stable** version of each tool at project start, and pin exact versions in the lock files (`Cargo.lock`, `pnpm-lock.yaml`). No beta or release-candidate versions. Upgrades are planned, tested, and done one tool at a time.

---

## 1. High level

```
+--------------------------------------------------------------+
|                Windows desktop app (one installer)            |
|                                                              |
|  FRONTEND (what the user sees)                               |
|  React + TypeScript, right-to-left, Urdu                     |
|  Screens, grids, forms, charts, keyboard navigation          |
|  Never calculates money. Only shows what the backend sends.  |
|                         |  Tauri commands (typed)             |
|                         v                                     |
|  BACKEND (the brain)                                         |
|  Rust                                                        |
|  Validation, money calculations, posting, partner split,     |
|  rules engine, reports, analytics, backup, audit             |
|                         |                                     |
|                         v                                     |
|  DATABASE: SQLite (one file on the computer)                 |
+--------------------------------------------------------------+
         |                         |                  |
   Local backup folder        USB drive         Cloud storage
   (every 15 min)             (daily)           (daily, encrypted)
```

| Layer | Choice | Why | Rejected and why |
|---|---|---|---|
| App shell | **Tauri 2** | Small installer, uses Windows' built-in browser engine (WebView2, Chromium based) which renders Urdu Nastaliq correctly, strong security model | Electron: 10 times bigger, more memory. .NET WPF: good, but weaker Urdu text rendering and we lose the web UI ecosystem. |
| Back end | **Rust** | No crashes from memory bugs, very fast, strict types catch money mistakes at compile time | Node.js: floating point numbers by default, easier to make money mistakes |
| Front end | **React + TypeScript** | Largest ecosystem for grids, forms, charts, accessible components; Claude Code writes it very well | Svelte, Vue: fine, but smaller ecosystem for data grids and RTL |
| Database | **SQLite** | One file, no server to crash, extremely well tested, full transactions, safe live backups | PostgreSQL / MySQL: a server process for one user on one PC is extra risk with no benefit |
| Cloud backup | **Cloudflare R2** (S3-compatible) | 10 GB free, no download fees, simple keys that do not expire. A payment card must be on file even for the free tier. | Google Drive: login tokens can expire for private apps. Backblaze B2 is the second choice (also S3-compatible, 10 GB free). |
| Printing and PDF | **HTML print templates printed by WebView2**, called from Rust through Tauri's `with_webview` and the `webview2-com` crate (WebView2 `Print` with printer, copies and paper size; `PrintToPdf` for PDF files. These are the Win32 COM names; .NET calls them `PrintAsync` and `PrintToPdfAsync`.) | Same engine as the screen, so Urdu prints exactly as it looks. Lets Settings choose printer, paper and copies, which plain `window.print()` cannot. | Rust PDF libraries: weak Nastaliq shaping |

### 1.1 One rule above all

**The backend owns all money.** The frontend sends what the user typed (as text) and shows what the backend returns. It never adds, multiplies, rounds or splits money. This is how the screen and the saved data can never disagree (PRD FR-6.1).

Example: the user types rate 600 and qty 20. The frontend sends `{"qty":"20","rate":"600"}`. The backend replies with value 1200000 paisa, commission 72000 paisa, and so on. The frontend only formats 1200000 paisa as "12,000".

---

## 2. Low level: back end (Rust)

| Need | Library | Notes |
|---|---|---|
| Language | Rust stable, edition 2024 | Pinned in `rust-toolchain.toml` |
| App framework | `tauri` 2 | Commands, events, windows |
| Tauri plugins | `tauri-plugin-single-instance`, `tauri-plugin-updater`, `tauri-plugin-dialog`, `tauri-plugin-window-state`, `tauri-plugin-process` | Only these. No general file-system or shell plugin. |
| Database driver | `rusqlite` with `bundled`, `backup` and `collation` features | SQLite compiled into the app, same version everywhere. `collation` registers our Urdu sort order inside SQL. |
| Migrations | `rusqlite_migration` | Numbered SQL files, run on start-up after a backup |
| Money | **Own module `money`** (no external crate) | `Paisa(i64)` and `Bps(i32)` types. See Money Calculations agent profile. |
| Big intermediate maths | Rust `i128` | For `amount × rate` before dividing, so nothing overflows |
| Dates | `chrono` (dates only, no time zones in business data) | Business dates are plain `YYYY-MM-DD` |
| Errors | `thiserror` | One error type per crate, mapped to error codes at the edge; the frontend shows them in the app language |
| Serialisation | `serde`, `serde_json` | |
| TypeScript types from Rust | `ts-rs` | Generates `frontend/src/bindings/*.ts`. Never edited by hand. Money types are exported as `number` (`#[ts(type = "number")]`), because they travel as JSON numbers; ts-rs would otherwise export `i64` as `bigint`. |
| Validation helpers | Own `validate` module | Parsing money text, quantities, codes, phones, Urdu names |
| Passwords | `argon2` (Argon2id) | |
| Secrets | `keyring` | Windows Credential Manager |
| Logging | `tracing`, `tracing-subscriber`, `tracing-appender` | Daily files; our own small clean-up keeps 30 days and at most 50 MB in total (NFR-OBS-1) |
| Compression | `zstd` | Backups |
| Encryption | `age` | Backups (cloud and USB) |
| Cloud upload | `object_store` (S3 mode) | Works with R2 and B2 |
| Checksums | `blake3` | Backup verification; keyed hash chain over journal and audit entries (`blake3::keyed_hash`, key in Credential Manager, NFR-SEC-6) |
| HTTP client for the assistant | `reqwest` (rustls TLS, JSON; no default features) | Already in the build through `object_store` and the updater, so no new code is downloaded. Used only by `mandi-assistant`, for its OpenRouter calls (Section 4.8, NFR-SEC-9). |
| Excel export | `rust_xlsxwriter` | Urdu text supported. Column headings come from the same Fluent files as the screens (see 4.2). |
| Urdu and English text in the backend | `fluent-bundle` | Reads the same `.ftl` files as the frontend, embedded at build time, for Excel headings and the backup pack, in the language asked for |
| English-to-Urdu typing | **Own module `translit`** in the `core` crate | Word list + learned pairs + spelling rules, fully offline (Section 4.4) |
| Smart search | **Own in-memory index** in `services/src/search`, with `strsim` for spelling distance | All accounts held in memory (20,000 accounts is a few MB). Built from the database at start-up and after a restore; updated right after any save that changes a name, phone or city commits (Section 4.7) |
| Urdu sorting | `icu_collator` (ICU4X) with Urdu locale | Correct ا ب پ ت ٹ order |
| Unicode clean-up | `unicode-normalization` | Plus our own Urdu letter-variant mapping (Section 4.3) |
| Testing | built-in `cargo test`, `proptest` (property tests for money), `insta` (snapshot tests for reports) | |
| Benchmarks | `criterion` | For NFR speed targets |
| Code checks | `rustfmt`, `clippy` (warnings are errors), `cargo-deny` (licences and security advisories) | |

---

## 3. Low level: front end (TypeScript)

| Need | Library | Notes |
|---|---|---|
| Language | TypeScript, `strict: true` | No `any` |
| UI library | React (latest stable) | |
| Build tool | Vite | |
| Package manager | pnpm | |
| Routing | TanStack Router | Type-safe routes for every screen |
| Server state (backend data) | TanStack Query | Caching, refresh after save |
| Local UI state | Zustand | Small stores only (current lot, side menu open) |
| Forms | React Hook Form + Zod | Zod checks are for quick on-screen hints only. The backend validates again and is the final word. |
| Data grids | **AG Grid Community** | Built-in RTL mode, keyboard navigation like a spreadsheet, draws only visible rows (fast with thousands of rows), MIT licence. The RTL setting is fixed when a grid is created, so a language switch re-creates grids (Section 4.3). |
| UI components | Radix UI primitives + our own design system, styled with Tailwind CSS | Radix gives accessible dialogs, menus, popovers and tabs with RTL support. We style them ourselves so the look is ours. |
| Search pickers | Downshift (`useCombobox`) | Accessible type-to-search picker for accounts, items and other masters (Radix has no combobox) |
| Styling | Tailwind CSS with design tokens in CSS variables | Logical properties (`ps-`, `pe-`, `ms-`, `me-`) for RTL |
| Icons | Lucide | |
| Charts | Apache ECharts | Fast, draws Urdu labels correctly. It has no automatic RTL mode, so our chart wrapper places legends and axes for RTL or LTR itself, from the current language. |
| Translations | Project Fluent (`@fluent/bundle`, `@fluent/react`) | Handles Urdu grammar (gender, number) and English plurals properly. Both bundles are loaded at start, so a switch needs no loading. |
| Testing | Vitest + Testing Library | |
| End-to-end testing | WebdriverIO + `tauri-driver` on Windows | Runs the PRD worked day on the real app |
| Code checks | ESLint (typescript-eslint strict), Prettier | |

---

## 4. Low level: Urdu and English stack

Professional Urdu, a complete English app, and typing names in two scripts need more than a translation file. Every piece below is free and open source, and works offline.

### 4.1 Fonts (bundled in the app, no internet needed)

| Use | Font | Licence | Why |
|---|---|---|---|
| Labels, headings, menus, printed bills | **Gulzar** (Nastaliq) | SIL Open Font License | Modern, readable Nastaliq, designed for screens |
| Fallback Nastaliq | Noto Nastaliq Urdu | SIL Open Font License | Very complete character coverage |
| Dense grids (optional, user setting) | Noto Naskh Arabic | SIL Open Font License | Shorter line height, more rows fit on screen |
| Numbers, Latin text and the whole English app | Inter, with tabular figures | SIL Open Font License | Digits all the same width, so money columns line up. When the optional Urdu digits (۰-۹) are used on prints, Noto Naskh Arabic supplies them. |

Fonts are loaded from the app's own files with `font-display: block`, so text never flashes in a wrong font.

### 4.2 Translation system

- Project Fluent `.ftl` files: `frontend/src/i18n/ur/*.ftl` and `en/*.ftl`, one file per module (`bikri.ftl`, `vouchers.ftl` ...).
- Message IDs are English, kebab-case, prefixed with the module: `bikri-header-truck-no = ٹرک نمبر`.
- Grammar is handled inside the message, not in code. Example:

```ftl
common-saved-toast = { $kind ->
    [bikri] بکری { $number } محفوظ ہو گئی
   *[voucher] واؤچر { $number } محفوظ ہو گیا
}
```

- `glossary.ftl` holds the locked business terms (PRD Section 3). Other files reference them, so a term is spelled once.
- Backend error messages are codes (`ERR_QTY_EXCEEDS_LOT`). The frontend turns each code into the app language through Fluent.
- `ur/` and `en/` always have the same files and the same message IDs. The script `pnpm i18n:check` fails the build if a message is missing, empty or identical to the other language where it should not be (NFR-URD-22). English mandi words come from `en/glossary.ftl`, which mirrors the PRD glossary's English column.
- The backend needs text in three places only: Excel headings, the readable backup pack, and amounts in words. For the first two it reads the same `.ftl` files with `fluent-bundle` (the files are embedded into the backend at build time from `frontend/src/i18n/`), in the language asked for, so every word still has one source. It never writes Urdu or English sentences by hand in Rust code.

### 4.3 Right-to-left, mixed text and search

| Need | How |
|---|---|
| Direction follows the language | One `language` value in the app store drives everything: `<html dir lang>` (`rtl`/`ur` or `ltr`/`en`), Radix `DirectionProvider`, the Fluent bundle, the chart wrapper and the grids. Styles use only logical properties (`ps-`, `me-`, `start`, `end`), so one stylesheet serves both directions. |
| Switching without losing work | Unsaved entry data lives in Zustand stores and React Hook Form state that sit above the screens, never inside a grid. On a switch, AG Grid instances are re-created (`enableRtl` is fixed at creation) from that data, and the focused row and column are restored. Target: NFR-SPD-19. |
| Numbers and codes inside Urdu text never flip | Every number, code, phone and truck number is wrapped in `<bdi>` (or `unicode-bidi: isolate`). Shared `<Num>` and `<Code>` components do this, so nobody forgets. |
| Money and number columns | `font-variant-numeric: tabular-nums`; in a table, grid or print column each number cell, its heading and its total are centred (`direction: ltr; text-align: center`, NFR-USE-19, v1.7); in a label and value card the value is on the physical right edge (`text-align: end`). Shared grid column types, `shared/ui/Table`, the print tables and the `<Num>` component do this. |
| Urdu letter variants | Search keys are normalised in Rust before saving and before searching: ي→ی, ى→ی, ك→ک, ه (Arabic heh)→ہ, ۃ→ہ, while ھ (do-chashmi heh) stays as it is, remove diacritics (zabar, zer, pesh, shadd), unify spaces and zero-width joiners. Example: "اكرم" and "اکرم" both match. |
| Sorting | ICU4X collator with the Urdu (`ur`) locale in Rust |
| Search in both scripts | Every named record has a required English name. Each record keeps three search keys, built in Rust on save: the normalised Urdu name, the lower-case English name, and a phonetic key (the English name with vowels and doubled letters removed, so "arshd" and "arshad" give the same key). Details in Section 4.7. |

### 4.4 Typing names in two scripts (PRD Section 7.4)

**Three typing modes per field**, switched with `Alt+U`, shown as a badge inside the field ("رومن", "اردو", "EN"):

| Mode | How it works |
|---|---|
| رومن سے اردو (default for Urdu text fields) | The shared `RomanUrduInput` component sends the current word to the backend command `translit_suggest` on each key press and shows the top 3 suggestions. `Space`, `Enter` or leaving the field accepts the highlighted one (the first unless `↑` `↓` moved it); `Enter` then moves on. Never used in pickers or search boxes. |
| اردو کی بورڈ | Our own phonetic keyboard in `frontend/src/shared/urdu/phonetic_keyboard.ts`, based on the widely used CRULP phonetic layout (`a`→ا, `b`→ب, `p`→پ, `t`→ت, `T`→ٹ, `h`→ہ, `H`→ھ). An on-screen key map opens with one click. |
| انگریزی | No conversion. Used for English name fields. |

Codes, numbers and phones always take English digits and never convert. Urdu letters typed from a Windows Urdu keyboard pass through unchanged in every mode.

**The `translit` engine** (Rust, `backend/crates/core/src/translit/`) is pure: no database. `services/src/translit` keeps the learned pairs in memory (loaded from the `translit_pairs` table at start-up, updated on each `translit_learn`) and passes them in. For each word the engine looks, in this order:

1. **Learned pairs** from the `translit_pairs` table (PRD FR-7.26), ranked by how often each was chosen.
2. **Word list**: `backend/crates/core/data/translit_seed.tsv` (six columns: roman, urdu, kind, weight, review, source; `review` is `pending` until the Urdu reviewer confirms the row, and `source` says where it came from, for example `team`, `wikidata`, `geonames` or `engine`), embedded at build time. At least 15,000 reviewed entries (NFR-URD-24): Pakistani first names, family and firm names, cities and towns of Punjab and Khyber Pakhtunkhwa, fruits, vegetables, packings and glossary words (PRD FR-7.27). Several Urdu spellings per Roman word are allowed (shahid → شاہد, شاھد), ordered by weight.
3. **Spelling rules** for words not found: longest-match rules on letter groups (`kh`→خ, `gh`→غ, `sh`→ش, `ch`→چ, `th`→تھ, `ph`→پھ, `aa`→ا, `ee`→ی, `oo`→و, final `a`→ا or ہ ...) produce candidates, which are scored by a small letter-pair model built from the word list at build time.

Candidates from the three sources are merged in that order (learned pairs first), duplicates are removed, and the engine returns at most 3. Whole phrases are converted word by word ("arshad multani" → ارشد ملتانی). Each suggestion carries its source (`learned`, `word_list` or `rules`), so `RomanUrduInput` and `NamePair` can mark rules-only words with "ہجے دیکھ لیں" and the save asks for confirmation (PRD FR-7.66).

**Urdu to English** (PRD FR-7.24) uses the same data the other way: a reverse lookup in learned pairs and the word list, else letter rules (ش→sh, خ→kh ...), then capital first letters.

**Accuracy is measured**, not hoped for: `backend/crates/core/tests/translit_names.tsv` holds the 1,000-name test list of NFR-URD-24 and NFR-URD-25, and a test reports the top-1 and top-3 rates on every build.

**Options we looked at:** IndicXlit (AI4Bharat, MIT licence, supports Urdu) and RomanAlfaz (MIT licence, rules plus a dictionary) both run in Python with a model or a large runtime, which a small offline Windows app should not carry. We follow the same idea (word list, rules, ranking) in Rust. Open word lists may be used to grow our list only after their licence is checked and credited. In use now: Wikidata (CC0, no credit needed) and GeoNames (CC BY 4.0, credited in the app information screen in D5).

### 4.5 Numbers, words and dates in both languages

- Amount in Urdu words: our own Rust module `urdu_words` in the `core` crate, using Pakistani grouping (ہزار، لاکھ، کروڑ، ارب). Covers every whole-rupee amount from 0 to the largest allowed amount, 10,00,00,00,000 (دس ارب), tested with known samples and boundaries. Example: 1,316,040 → "تیرہ لاکھ سولہ ہزار چالیس روپے".
- Amount in English words: our own module `english_words` in the `core` crate, with two groupings (thousand, million, billion; or thousand, lakh, crore), chosen by the number grouping setting (PRD FR-19.7). Same range and the same kind of tests. Example: 19,316 → "Nineteen thousand three hundred sixteen rupees only".
- Dates: stored as `YYYY-MM-DD`, shown as `DD-MM-YYYY` in both languages. Month names come from Fluent in the current or print language.
- Digits are English digits (0-9) in both languages. Optional Urdu digits (۰-۹) for Urdu prints only, by setting.

### 4.6 Machine translation (drafts only)

If long help text is needed, IndicTrans2 (MIT licence, supports Urdu) can produce a first draft, which a person rewrites (NFR-URD-3). It is not part of the app. Meta's NLLB is not used because its licence does not allow commercial use.

### 4.7 Smart assistance (PRD Section 7.6)

| Piece | Where | How |
|---|---|---|
| Party card | Command `assist_party_card(account_id)`; component `shared/ui/PartyCard` | One query on indexed balances and the matching tables; target NFR-SPD-21 |
| Short codes and smart search | Command `search_accounts(text, scope)`; component `shared/ui/AccountPicker` | Rust index in memory: code, serial, three name keys (the tag included), father or firm, city, phone. Order: exact code, exact running number (the short code, PRD FR-7.39; in a seller or buyer field, that type only; in an any-account field, the seller and the buyer with that running number), code starts-with, name starts-with, recent and frequent (from `user_prefs`), then close matches by `strsim` Damerau-Levenshtein (at most 1 mistake up to 5 letters, 2 above). Target NFR-SPD-6. With empty text (a click on the picker, PRD FR-7.10) it returns every record of the field's type, recent first, then by code, a page at a time. |
| How was this worked out | Every preview result from the backend carries a `working` object for each calculated amount (formula, inputs, rule ID); component `shared/ui/CalcInfo` | The frontend never rebuilds the working; it only shows what the backend sent |
| Attention list | Command `assist_attention()`; components `AttentionBell` (top bar) and the dashboard list | Counts from indexed queries, refreshed after each save event |
| Memories and hints | Commands `assist_truck_memory`, `assist_seller_memory`, `assist_last_rate`, `assist_open_bills`, `assist_open_payables` | Read-only queries; nothing is saved by them. The rate hint for a lot is returned by `bikri_preview`, which sees the unsaved lines of the screen, and falls back to `assist_last_rate` for saved data (PRD FR-7.53). Open payables use the read-time oldest-first working of PRD FR-7.59 (Money agent 8.2). |
| User preferences | Table `user_prefs`; commands `settings_get_prefs`, `settings_set_pref` | Language, text size, last screen, filters, print-after-save list, recent records |

### 4.8 Ask the app (PRD Section 7.7)

The model is the brain: it understands the question, decides which tools to call, reads the results, asks for sums and estimates, and writes the answer. The app is its arms and its auditor: every tool is our own read-only Rust code calling the same functions the screens call, and no answer is shown until our code has checked it. So the assistant adds no new way to compute a figure on the books, and the model never types a number that reaches the screen.

**The flow, for one question:**

1. The frontend sends the typed text and the panel's conversation id to the command `assistant_ask`.
2. `mandi-assistant` checks sign-in, the setting, the key and the daily count (PRD FR-7.81), and opens one read snapshot for the whole question (NFR-AI-6).
3. It sends the first model call: the fixed instructions (first, so they are cached), the last 3 questions of the conversation with their tool results and answers, today's date, the app language and the question, with the tools of PRD FR-7.87.
4. The model replies with tool calls. For each one the app checks the inputs like a command input (Code Standards Q1), runs the tool, builds its result with `tool_result_json()` (Q7) and gives it an id (`r1`, `r2` ..., running on through the conversation). A call to a partner source while the section is locked pauses the question at once, with no model call (PRD FR-7.82); after the code the paused calls run and their results are sent. The panel shows a step line for each tool in the app's own words. The results go back to the model in the next call.
5. Steps 3 and 4 repeat until the model gives its final answer, at most 4 calls and 8 tool calls in all (NFR-AI-5). A continuation after "کون سا؟" or the partner code goes on with what is left of these limits.
6. The app checks the final answer (`check.rs`, PRD FR-7.76). If a check fails, it sends the model one more call with the reasons, and checks again. If that fails too, the panel shows the tool results as plain tables.
7. It fills every reference with the exact value (money with its side when it has one, a difference with its direction word), builds the tables, charts, buttons and source line, and returns the answer to the panel.
8. It writes the question log row in its own save, after the answer (PRD FR-7.84).

| Piece | Where | How |
|---|---|---|
| Model and request | `crates/assistant/src/client.rs` | `POST https://openrouter.ai/api/v1/chat/completions`. The model comes from the setting, `openai/gpt-6-luna` by default. Every call sets `stream: false`, the `tools` of PRD FR-7.87 as strict function schemas, `tool_choice: "auto"`, `parallel_tool_calls: true`, `response_format: {type: "json_schema", json_schema: {name: "answer", strict: true, schema: <the answer schema>}}` for the final answer, `reasoning: {effort: "medium"}` [A, lowered to "low" if NFR-AI-2 still passes], `max_tokens` = what is left of the 3,000 output tokens of the question (at most 1,500 a call), a fixed `seed`, and `provider: {only: ["azure"], allow_fallbacks: false, zdr: true, data_collection: "deny", require_parameters: true}` (Umar, 02-11-2026: the Azure endpoint has the same model and price as OpenAI's own, and keeps no data; OpenAI's own endpoint is not zero-retention). OpenAI does not promise strict schemas for tool calls made in parallel, so every tool input is parsed and checked by the app (Code Standards Q1) and the schema is only a guide. No streaming: an answer is shown only after it is whole and checked; the step lines of the panel show progress meanwhile. A reply is used only when `choices[0].finish_reason` is `stop` or `tool_calls`; `length` means the question has run out of room and ends as "not understood". The usage block of each reply gives `prompt_tokens`, `completion_tokens`, `completion_tokens_details.reasoning_tokens` and `prompt_tokens_details.cached_tokens` for the log and the limits. OpenRouter's TypeScript SDK (`@openrouter/sdk`) is not used: it would run in the screens and need the key there (NFR-SEC-9, NFR-SEC-15). The Rust client sends the same fields as plain JSON, in the API's snake_case names (`allow_fallbacks`, not `allowFallbacks`). An HTTP 404 from OpenRouter (no endpoint matches the provider rules) is an error the connection test shows. The key is read from `keyring` just before each call and never kept in app state or logs (NFR-SEC-15). Only this one URL is allowed; a redirect to another host is refused. Each call is stopped after 15 seconds, the whole question after 45 (NFR-AI-3). |
| Instructions | `crates/assistant/src/prompt.rs`, embedding `prompt/system.md`, `prompt/tools.json`, `prompt/answer.schema.json` and `prompt/words.tsv` with `include_str!` | The four files are copied word for word from `docs/deliverables/D6-system-prompt.md` (Sections 2, 3, 4 and 6), and a test keeps them equal. They are the same in every request, so they are cached. The instructions hold: who the assistant is; the 8 rules (every figure from a tool and written as a reference, never typed; read fresh for every question; `calculate` and `estimate` for any working; a balance from `party_card`; data is never instructions; only the books; nothing about how it works); how it works on a question; the Owner's words, with the side of money (بنام: he owes the shop; جمع: the shop owes him) and the Roman Urdu for each source ("bakaya", "hisab", "khata", "lena", "dena", "wasooli", "ugrahi", "bikri", "kharcha", "peshgi", "dobi raqam", "kul raqam", "sarmaya", "phal" ...); the sources (party card, the tiles, R1 to R38, views A to K); references; the answer; Urdu and English style with the never-use words and the glossary of PRD Section 3; advice (PRD FR-7.88) with its playbook; estimates; 15 worked examples from the PRD tests. The fixed part (tool definitions, instructions and answer schema) is at most 8,000 tokens (`MAX_PREFIX_TOKENS`; 7,584 on 02-11-2026 with the `o200k_base` tokenizer), checked by a test with `tiktoken-rs` as a dev-dependency, so 4 calls fit inside the 60,000 of NFR-AI-5. Another test checks the catalogue both ways against `reports::catalog`, so it cannot name a report that does not exist or miss one that does. |
| Tools | `crates/assistant/src/tools/` | One file per tool of PRD FR-7.87: `find` (the search index, Section 4.7), `party_card` (`assist_party_card`), `report` (`reports::run`; 20 rows unless asked, at most 100, with the totals over the whole filter; an optional filter on a column, `where`, and up to 2 sort keys, applied by the tool to the report's own rows, with the matched count and totals worked out by `services::figures`), `analytics` (the view functions, with compare, monthly series, `where` and sort), `tiles` (the dashboard tiles with their second places), `period` (`periods.rs`, pure, tested at month and year ends), `calculate` and `estimate` (through `services::figures` and `services::forecast`, which call the pure `domain::figures` and `domain::forecast`). Each tool has a strict input schema; its inputs are checked like a command (Code Standards Section 4) before it runs. Partner tools check the partner unlock first (S8). The operands of `calculate` are references to values already read for this question, or numbers that appear in the typed question; never other numbers. |
| Tool results | `crates/assistant/src/results.rs` | `tool_result_json()` is the only function that turns a tool's output into what is sent. It gives the result an id, keeps its source and filters, and gives each row the stable key of its record (for example `je4521` for a journal line), so a reference never depends on a row's place in the list. Every figure that has a second place (PRD FR-7.76) carries both places and its checked status wherever it appears: a balance in `find` is read through `assist_party_card` on the same read snapshot, not from the search index. It leaves out every field PRD FR-7.83 says never goes. `request_body()` does the same to the conversation it resends, and drops an earlier partner result when the section is locked. Money goes as whole rupees with its side (بنام or جمع), never as paisa. Every value is a small typed object (money with its side, checked status and both places; a difference with its direction; a percent; a count; a date; a month; a name; a label), so a reference always ends on a value the app can fill (`D6-system-prompt.md` Section 5). A result is at most 16,000 bytes (rows dropped from the end, `rows_cut` set, totals unchanged); R11 to R13 give their latest rows first. In the conversation it resends, `request_body()` cuts an earlier question's results down to their totals, summary and the rows its answer referred to. |
| The answer | `crates/assistant/src/answer.rs` | The strict answer type (`prompt/answer.schema.json`): `kind` (answer, estimate, advice, which_one, need_dates, need_party, not_found, outside_books, change_request, not_understood); `text` (the answer, only for answer, estimate and advice, empty for the other kinds, whose words the panel has; with references written as `{{r2.balance}}`, `{{r1.matches.0.name}}`, `{{r3.rows.je4521.debit}}`, `{{c1.result}}`, `{{e1.ahead.ah2.same_growth}}`, `{{q.n1}}` for a number the app read from the question); `show` (up to 3 tables or charts, each naming a tool result); `next` (up to 3 suggested questions, checked like the text); `entry` (for a change request: the entry screen and the party reference); `choose` (for which_one and not_found: the `find` result); `nearest` (for outside_books: the closest source and party). The log's outcome adds what the app decides: change_refused, checks_failed, records_differ, locked, limit_reached, offline, error. |
| Checks | `crates/assistant/src/check.rs` | Runs every rule of PRD FR-7.76 on the text and the next questions, and returns each failure with its reason, for the one rewrite (a figure whose two places differ is final, with no rewrite): every reference resolves; no character of the Unicode digit class (Nd) outside a reference; no word of the number-word list (`NUMBER_WORDS`, Urdu and English) and no month name outside a reference; no name of a party, partner, item, garden, trip, city, اڈا or staff member returned by this question's tools outside a reference (glossary words and seeded system accounts are not checked); no ✓ typed; no reference to a result of an earlier question; no reference to a figure whose two places differ; around each balance, up to the nearest punctuation mark, new line, point mark or other money figure on each side (a name or date never ends it), none of the other side's words (`words.tsv`); a reference to an estimate needs `kind` estimate or advice; the script of the app language; no word of the never-use list; at most 130 words (`MAX_ANSWER_WORDS`), counted after filling, each reference as one word; every table and chart names a tool result of this question. Then it fills each reference: money through the shared formatter with its side when it has one, a difference or change with its direction word (زیادہ or کم), names through `displayName` (with the tag), dates in DD-MM-YY, numbers from the question in a plain style with no ✓. The numbers of the question are read by `domain::figures::question_numbers` (digits, and lakh, hazar and crore after a number: "3 lakh" is 3,00,000), each with its unit (months, days, years, percent, amount or plain), and a number the model passes for months ahead, last N or a بکری number must match one with the right unit. |
| Calculator and estimator | `domain/src/figures.rs`, `domain/src/forecast.rs`, wrapped by `services/src/figures.rs` and `services/src/forecast.rs` | Pure functions with checked integer maths on `Paisa` (Code Standards M14), reviewed by the money-calculations agent. `figures`: sum, difference, average, percent of (in bps), change in percent, and the numbers read from a question. `forecast`: the three methods of PRD FR-7.86 and the month a target is reached, up to 12 months ahead, on a monthly series read by `services::forecast` through the analytics view functions. `mandi-assistant` itself does no arithmetic on money (clippy `arithmetic_side_effects` denied in the crate). |
| Quick questions | `crates/assistant/src/quick.rs` | The chips of PRD FR-7.80: fixed tool calls and fixed Fluent words, with no model. |
| Question log | `db/src/assistant.rs`, table `assistant_questions` (Architecture 15.3) | Append-only. It is the source of the daily count. |
| Screens | `frontend/src/features/assistant/` | `AssistantDock` sits in the app shell above the status bar. It holds the question box (raw text, no Roman-to-Urdu conversion, like global search), the quick-question chips, the step lines while the model works (the first, "سوال سمجھ رہا ہوں…", shown by the panel the moment a question is sent), and `AnswerBlock`: the answer text with each filled figure drawn by `<Num>` and its ✓ and ⓘ, the tables (the report screens' own cell renderer), the charts (the shared chart wrapper), the estimate band and the advice band, the buttons, the source line and the next-question chips; plus the "کون سا؟" buttons and `PartnerLockedBox` (already shared). The key `F1` is registered in `shared/keyboard/shortcuts.ts`. |
| Tests | `crates/assistant/tests/golden_questions.rs` with `tests/fixtures/model/*.json` | Each golden question has a recorded transcript: the model's reply to each call (tool calls, then the answer). No commit test calls the network. Live check before a release: `scripts/ai-eval/` with the question list `questions.tsv` (NFR-AI-2). |

---

## 5. Repository structure

### 5.1 Top level

```
mandi-software/
├── frontend/                 Everything the user sees (React + TypeScript)
├── backend/                  Everything else (Rust: Tauri app + business crates)
├── docs/                     PRD, NFR, Tech Stack, Code Standards, Architecture
├── scripts/                  Seed data, performance data generator, release helpers
├── .github/workflows/        Build, test and release pipelines (Windows runner)
├── CLAUDE.md                 Instructions for Claude Code (written at build start)
├── package.json              pnpm workspace root (scripts only)
├── pnpm-workspace.yaml
├── rust-toolchain.toml
├── .editorconfig
└── README.md
```

### 5.2 Module names are the same everywhere

Each business module has **one name**, used unchanged in the frontend folder, the backend folders, Tauri command names, database table prefixes and Fluent file names. Lower case, English letters, words joined by `_`.

| Module | PRD section | Frontend folder | Backend folders | Commands start with | Fluent file |
|---|---|---|---|---|---|
| auth | 8 | `features/auth` | `services/src/auth`, `db/src/auth` | `auth_` | `auth.ftl` |
| dashboard | 9 | `features/dashboard` | `services/src/dashboard` | `dashboard_` | `dashboard.ftl` |
| masters | 10 | `features/masters` | `domain/src/masters`, `services/src/masters`, `db/src/masters` | `masters_` | `masters.ftl` |
| rules | 11 | `features/rules` | `domain/src/rules`, ... | `rules_` | `rules.ftl` |
| bikri | 12 | `features/bikri` | `domain/src/bikri`, ... | `bikri_` | `bikri.ftl` |
| pukhta_khata | 13 | `features/pukhta_khata` | ... | `pukhta_khata_` | `pukhta_khata.ftl` |
| vouchers | 15, 16 | `features/vouchers` | ... | `vouchers_` | `vouchers.ftl` |
| cashbook | 16 | `features/cashbook` | ... | `cashbook_` | `cashbook.ftl` |
| partners | 17 | `features/partners` | ... | `partners_` | `partners.ftl` |
| trips | 12.10 | `features/trips` | `services/src/trips`, `db/src/trips` | `trips_` | `trips.ftl` |
| purchases | 12.10 | `features/purchases` (inside the مارکیٹ سفر section) | `core/src/purchases`, `services/src/purchases`, `db/src/purchases` | `purchases_` | `purchases.ftl` |
| gardens | 12.11 | `features/gardens` | `services/src/gardens`, `db/src/gardens` | `gardens_` | `gardens.ftl` |
| ventures | 12.10, 12.11 | (no screen of its own: the shares, sources and splits that gardens and trips share) | `domain/src/ventures`, `services/src/ventures`, `db/src/ventures` | `ventures_` | `ventures.ftl` |
| fruit | 16.7 | `features/fruit` (a tab of the خرچہ area) | `services/src/fruit`, `db/src/fruit` | `fruit_` | `fruit.ftl` |
| analytics | 18 | `features/analytics` | ... | `analytics_` | `analytics.ftl` |
| reports | 19 | `features/reports` | `reports/src/...` | `reports_` | `reports.ftl` |
| backup | 20 | `features/backup` | `backup/src` | `backup_` | `backup.ftl` |
| settings | 21 | `features/settings` | ... | `settings_` | `settings.ftl` |
| audit | 21 | `features/audit` | ... | `audit_` | `audit.ftl` |
| ledger | 6, 12.6 | (no screen of its own) | `domain/src/ledger`, `services/src/ledger`, `db/src/ledger` | `ledger_` | `ledger.ftl` |
| translit | 7.4 | `shared/urdu` (no screen of its own) | `core/src/translit`, `services/src/translit`, `db/src/translit` | `translit_` | `translit.ftl` |
| search | 5.4, 7.6 | `shared/ui/AccountPicker`, global search in `app/layout` | `services/src/search` | `search_` | `search.ftl` |
| assist | 7.6 | `shared/ui` (PartyCard, CalcInfo, AttentionBell) | `services/src/assist`, `db/src/assist` | `assist_` | `assist.ftl` |
| assistant | 7.7 | `features/assistant` (the question panel, drawn by the app shell) | `crates/assistant` (`mandi-assistant`), `db/src/assistant` | `assistant_` | `assistant.ftl` |

Example: everything about بکری is found by searching for `bikri`: `frontend/src/features/bikri/`, `backend/crates/domain/src/bikri/`, command `bikri_save`, table `bikri_lots`, file `bikri.ftl`.

### 5.3 Frontend folder

```
frontend/
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── public/
│   └── fonts/                Gulzar, Noto Nastaliq Urdu, Noto Naskh Arabic, Inter
├── src/
│   ├── main.tsx              Entry point
│   ├── app/                  App shell (one place)
│   │   ├── App.tsx
│   │   ├── router.tsx        All routes
│   │   ├── providers.tsx     Query, i18n, direction, theme
│   │   └── layout/           AppShell, MainNav, SideMenu, TopBar (search, AttentionBell, LanguageSwitch, user menu), StatusBar
│   ├── features/             One folder per module (names from 5.2)
│   │   └── bikri/            Same inner layout for every module:
│   │       ├── pages/        Full screens (BikriBookPage.tsx, TodayBikriPage.tsx, StockPage.tsx)
│   │       ├── components/   Pieces used only by this module (LotsGrid.tsx, SalesGrid.tsx, BikriSummary.tsx)
│   │       ├── hooks/        useBikriPreview.ts, useSaveBikri.ts
│   │       ├── api.ts        Typed wrappers around this module's Tauri commands
│   │       ├── schemas.ts    Zod schemas for on-screen hints
│   │       ├── routes.tsx    This module's routes, registered in app/router.tsx
│   │       └── index.ts      Public exports of the module
│   ├── shared/               Used by many modules. No business logic.
│   │   ├── ui/               Design system: Button, Field, MoneyInput, QtyInput, DateInput,
│   │   │                     AccountPicker, PartyCard, CalcInfo, RomanUrduInput, NamePair, Dialog, Toast,
│   │   │                     Card, Tile, Tabs, Badge, EmptyState, Num, Code
│   │   ├── grid/             AG Grid setup: RTL, keyboard rules, column types (money, qty, account, date)
│   │   ├── keyboard/         Arrow navigation engine, shortcut registry, focus management
│   │   ├── urdu/             Phonetic keyboard, typing-mode badge, bidi helpers
│   │   ├── i18n/             useLanguage(), displayName() (picks the name for the current language), direction helpers
│   │   ├── format/           Display-only formatting: paisa to "12,000", dates, numbers
│   │   ├── api/              invoke() client, error-code to message mapping (current language)
│   │   └── hooks/            Generic hooks (useDebounce, useHotkey)
│   ├── i18n/
│   │   ├── ur/               glossary.ftl, common.ftl, errors.ftl, one file per module
│   │   ├── en/               Same files in English
│   │   └── index.ts
│   ├── styles/
│   │   ├── tokens.css        Colours, spacing, radius, font sizes (the only place for these values)
│   │   └── globals.css
│   └── bindings/             TypeScript types generated from Rust (do not edit)
└── tests/                    Component tests and test helpers
```

Rule: a module may import from `shared/` and from another module's `index.ts` only. Never from another module's inner files.

### 5.4 Backend folder (Rust workspace)

```
backend/
├── Cargo.toml                Workspace definition
├── app/                      The Tauri application (thin layer)
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json       frontendDist = "../../frontend/dist"
│   ├── capabilities/         Which commands the screens may call
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── lib.rs            Builds the app, registers plugins and commands
│       ├── state.rs          Shared app state (database pool, current user)
│       ├── error.rs          Turns crate errors into error codes for the frontend
│       └── commands/         One file per module: bikri.rs, vouchers.rs, ...
│                             Each command: check sign-in -> call a service -> return a typed result
└── crates/
    ├── core/                 No business rules, no database. Pure helpers.
    │   ├── data/             translit_seed.tsv (word list, embedded at build time)
    │   ├── tests/            translit_names.tsv accuracy list (NFR-URD-24)
    │   └── src/
    │       ├── money.rs      Paisa, Bps, rounding, parsing, formatting
    │       ├── dates.rs
    │       ├── ids.rs        Typed IDs (AccountId, BikriId ...)
    │       ├── text.rs       Urdu normalisation, English lower-casing and phonetic key for search
    │       ├── translit/     English-to-Urdu and Urdu-to-English suggestions (Section 4.4)
    │       ├── urdu_words.rs Amount in Urdu words
    │       ├── english_words.rs Amount in English words
    │       └── validate.rs   Shared input checks
    ├── domain/               Business rules as pure functions. No database, no files.
    │   └── src/
    │       ├── rules/        Commission rule resolution (PRD 11)
    │       ├── bikri/        Line, lot and بکری calculations (PRD 12.5, 22)
    │       ├── ledger/       Building balanced journal entries (PRD 6, 12.6)
    │       ├── partners/     Partner split (PRD 17, C10, C11)
    │       ├── allocation/   Oldest-first matching of receipts and advances
    │       └── ...           One folder per module that has rules
    ├── db/                   SQLite: connection, migrations, repositories
    │   ├── migrations/       0001_init.sql, 0002_..., never edited after release
    │   └── src/              One folder per module: queries for that module's tables
    ├── services/             Use cases: one database transaction per user action
    │   ├── src/              One folder per module: bikri::save(), vouchers::save_receipt() ...
    │   └── tests/            Golden end-to-end tests (PRD Section 27)
    ├── reports/              Report queries, print data, Excel export
    ├── backup/               Local, USB and cloud backup, restore, verification
    └── assistant/            Ask the app (Section 4.8): tools, results, checks, run loop, OpenRouter client
```

Crate names: `mandi-app`, `mandi-core`, `mandi-domain`, `mandi-db`, `mandi-services`, `mandi-reports`, `mandi-backup`, `mandi-assistant`.

Dependency direction (never the other way): `app` → `services` → `domain` + `db` → `core`. `domain` never depends on `db`. `mandi-assistant` sits between `app` and the rest: `app` → `assistant` → `services` + `reports`; nothing depends on `assistant` except `app` (v1.8). This keeps business rules testable without a database.

### 5.5 Tauri setup notes

- `tauri.conf.json` lives in `backend/app/`. The Tauri CLI finds it by searching the folders below the project root (the search depth can be raised with `TAURI_CLI_CONFIG_DEPTH` if ever needed).
- `build.beforeDevCommand` = `pnpm --filter frontend dev`, `build.devUrl` = `http://localhost:5173`, `build.beforeBuildCommand` = `pnpm --filter frontend build`, `build.frontendDist` = `../../frontend/dist`.
- Root `package.json` scripts: `dev`, `build`, `test`, `lint`, `gen:types` (runs ts-rs), `seed`, `perf`.

---

## 6. Naming conventions

| Thing | Convention | Example |
|---|---|---|
| Module names | lower snake_case, English transliteration, same everywhere | `bikri`, `pukhta_khata` |
| Rust files and modules | snake_case | `partner_split.rs` |
| Rust types, enums, traits | PascalCase | `BikriLot`, `ChargeMethod` |
| Rust functions and variables | snake_case, verb first for functions | `calculate_lot_net()`, `seller_comm` |
| Rust constants | SCREAMING_SNAKE_CASE | `MAX_LINES_PER_BIKRI` |
| Tauri commands | `<module>_<verb>[_<noun>]` | `bikri_preview`, `bikri_save`, `vouchers_save_receipt` |
| React components | PascalCase file and name | `SalesGrid.tsx` |
| Hooks | `use` + PascalCase | `useSaveBikri.ts` |
| Other TS files | snake_case | `phonetic_keyboard.ts`, `api.ts` |
| Name columns | Every named record has `name_ur` and `name_en`; never one `name` column | `accounts.name_ur`, `accounts.name_en` |
| TS types and interfaces | PascalCase | `SaleLineInput` |
| TS variables and functions | camelCase | `buyerTotal`, `formatPaisa()` |
| CSS tokens | kebab-case with group prefix | `--color-money-in`, `--space-3` |
| Database tables | snake_case, plural. A module with several tables starts them with the module name (`journal_` for the ledger); master lists use their plain noun | `bikri_entries`, `bikri_lots`, `journal_lines`, `translit_pairs`; masters: `accounts`, `cities` |
| Database columns | snake_case; money columns end in `_paisa`; rates end in `_bps`; dates end in `_date`; times end in `_at` | `seller_amount_paisa`, `comm_bps`, `bill_date`, `created_at` |
| Migrations | 4-digit number + short name | `0007_add_rule_item_group.sql` |
| Fluent message IDs | kebab-case, module prefix | `bikri-summary-net` |
| Error codes | `ERR_` + SCREAMING_SNAKE_CASE | `ERR_JOURNAL_UNBALANCED` |
| Git branches | `feature/<module>-<short>`, `fix/<module>-<short>` | `feature/bikri-lots-grid` |
| Commits | Conventional Commits | `feat(bikri): add live summary` |

Urdu words in code: we use simple English transliteration for business terms that have no exact English word (`bikri`, `laga`, `munshiana`, `pukhta_khata`, `peshgi`), and English for general terms (`seller`, `buyer`, `commission`, `freight`). A table of these lives in `docs/` and the glossary.

---

## 7. Development, build and release

| Topic | Choice |
|---|---|
| Developer machine | Umar's Mac works for writing code and running the app. Important: on a Mac, Tauri uses Apple's WebKit, not Windows' WebView2, so Urdu rendering and printing must be checked on Windows. Use a Windows PC or a Windows virtual machine for UI checks. |
| Windows builds | GitHub Actions on a Windows runner builds the installer (MSI and NSIS). Building a Windows installer directly on a Mac is not reliable. |
| Tests in CI | Every push: Rust format, clippy, all Rust tests (including golden money tests), frontend lint, type check, component tests. Before release: Windows end-to-end tests and speed tests. |
| WebView2 settings | On start, through `with_webview`: browser accelerator keys off (`AreBrowserAcceleratorKeysEnabled = false`), browser zoom off (`IsZoomControlEnabled = false`), default context menu off. Our own keys from PRD Section 7.3 replace them. |
| Release pipeline | Three GitHub Actions workflows: `ci.yml` (every push), `windows.yml` (every pull request to `main`), `release.yml` (every merge to `main`). A merge is allowed only when both checks pass. Release tests again, then builds, signs and publishes with `tauri-apps/tauri-action`. Version is `major.minor` from `tauri.conf.json` plus the run number, for example `0.1.12`. Full setup in `docs/Release_Pipeline.md`. |
| Update hosting | GitHub Releases of the public repository. The app reads `releases/latest/download/latest.json` and installs the NSIS setup file. If the repository becomes private, hosting must move (for example a public R2 bucket separate from backups). |
| Update signing | Tauri updater keys (free). Every release is signed. |
| Windows code signing | Recommended: a Windows code-signing certificate, so Windows SmartScreen does not warn on install. Costs money yearly; decide before go-live. |
| WebView2 | Offline WebView2 installer bundled in the installer (NFR-INS-1) |
| Data location | Tauri's `app_data_dir()` for the bundle identifier `pk.mandisoftware.app`, which on Windows is `%APPDATA%\pk.mandisoftware.app\`. Data file `data\mandi.db`; backups in `backups\` by default, changeable to a second disk. |

---

## 8. Change log

| Version | Date | Change |
|---|---|---|
| 1.0 | 10-10-2026 | Pre-build drafts and reviews: Umar's requirements, analysis of the old software video, research, client review comments and full cross-document reviews. |
| 1.1 | 10-10-2026 | **Final baseline for build.** From the start of development, every change raises the version (1.2, 1.3 ...) and adds a line here. |
| 1.2 | 12-10-2026 | PRD v1.2: sign-in check instead of roles (5.4); word list of at least 15,000 entries and suggestion source for the spelling check (4.4). |
| 1.3 | 15-10-2026 | Approved by Umar (PRD v1.5): the word list file has six columns (4.4) and its open sources are named and credited (4.4). |
| 1.4 | 19-10-2026 | Approved by Umar (PRD v1.6): module names trips, purchases, gardens, ventures and fruit (5.2). |
| 1.5 | 22-10-2026 | Approved by Umar (PRD v1.7): the tag is in the search index (4.7). |
| 1.6 | 27-10-2026 | Approved by Umar (PRD v1.8): codes by type, short code = running number, empty-text search for a click (4.7). |
| 1.7 | 30-10-2026 | Approved by Umar (PRD v1.9): numbers centred under their column heading (4.3). |
| 1.8 | 01-11-2026 | Approved by Umar (PRD v1.16): ask the app (4.8), the `reqwest` client for its one OpenRouter call (2), the module and crate `assistant` (5.2, 5.4). Amended 02-11-2026 before release (Umar): every request goes only to the zero-retention Azure endpoint of the model, with no fallback (4.8). Further amended 02-11-2026 (Umar): the instructions, tools and answer schema are four fixed files copied from `D6-system-prompt.md`, measured at 7,584 tokens; advice; `where` and sort on reports and views; typed result values; the partner pause (4.8). |
