# Build status

Kept by `tech-lead`. Updated on each feature branch before its pull request.

## Current

- Goal run: D1 and D2 together (README Section 4a, requirements v1.2 committed as `b0902a6`). Build every plan 01 to 10, mark each "built" and commit it; then one fresh `reviewer` and `robustness` for the whole run; then one pull request and the D1+D2 DONE REPORT.
- Branch: `feature/db-schema` carries plans 03 onward (stacked on `feature/core-money`, PR #2, on `feature/scaffold-repo`, PR #1). GitHub Actions is off; Windows-workflow items are WAITING-CI.
- Now (12-10-2026): plans 04 to 06 committed as built (`76cc5ca`), waiting for the end gate. Word list honest: 12,942 entries, 8,212 known, below 15,000 (open question for Umar). Backend on plan 07 (T1 contract, then T2). Frontend on the openings grid, then plans 09 and 10 screens (the plan 09 rules folder is uncommitted work in progress).

## Plan files

**Goal run (README 4a, v1.2):** plans are built by `backend` and `frontend`; `reviewer` and `robustness` run once at the end for the whole of D1 and D2 and cover every plan file 01 to 10. Plans 01 and 02 also had earlier per-plan verdicts.

| Plan | Branch | Pull requests | reviewer | robustness | State |
|---|---|---|---|---|---|
| 01-scaffold | feature/scaffold-repo | #1 | PASS WITH MINOR | PASS | gate passed; waiting for CI |
| 02-core | feature/core-money | #2 | PASS WITH MINOR | PASS | gate passed; committed locally |
| 03-database | feature/db-schema | #3 | at the end | at the end | built, local checks green |
| 04-shell-and-login | feature/db-schema | | at the end | at the end | built (`76cc5ca`); end gate |
| 05-names-two-scripts | feature/db-schema | | at the end | at the end | built (`76cc5ca`); end gate |
| 06-masters-and-search | feature/db-schema | | at the end | at the end | built (`76cc5ca`); end gate |
| 07-journal-and-accounts | feature/db-schema | | at the end | at the end | built on both sides (backend 612 Rust tests, money agent PASS WITH MINOR, E2E-23 backend part in assist.rs); frontend T3+T4 wired to the bindings (tree, list, form, openings grid; 1340 frontend tests); backend after the word-list fix. Openings grid contract added 12-10 |
| 08-party-card-and-short-codes | feature/db-schema | | at the end | at the end | building: frontend T3 done (AccountPicker, PartyCard, quick add, global search); backend T1/T2 waiting |
| 09-commission-rules | feature/db-schema | | at the end | at the end | building: frontend T3 done (rules list, form, test a rule, settings tabs); backend after plan 08 |
| 10-opening-position | feature/db-schema | | at the end | at the end | building: frontend T3 done (پیشگی screen with running list, day close and reopen); backend after plan 09 |

## Design

- Canvas: https://claude.ai/artifact/EtYUy8QMjbEkx7yy9aeAiF. Rules: `docs/design/design-system.md`. Board sources: `docs/design/canvas/`.

## Setup facts

- Requirement documents are in `.claude/rules/` (Umar's choice). A path `docs/<Doc>.md` in any file means `.claude/rules/<Doc>.md`.
- Deliverable files were moved from `deliverables/` to `docs/deliverables/`, as README Section 1 expects.
- Updater key: made on 10-10-2026. Private key and password are in `~/.tauri/` on Umar's Mac. Public key in `docs/release/updater-pubkey.txt`. Both GitHub secrets are set.
- `main` ruleset is active (id 24841987): pull request, squash only, required checks `Checks (Windows)` and `Windows E2E and speed`, up to date.

- `docs/deliverables/progress.json` feeds the goal-progress band above the prompt (the `goal-progress` mod). The lead updates it as work moves: a plan goes `waiting`, `building`, `reviewing`, `passed` (both verdicts PASS or PASS WITH MINOR); an acceptance id is added only with real command output; a deliverable is added to `reports` when its DONE REPORT is posted. A new goal changes `goal.key` and `goal.scope`, and its progress starts from zero.

## Blockers

- 10-10-2026: GitHub Actions does not start any job: "The job was not started because your account is locked due to a billing issue." Only Umar can fix this (github.com/settings/billing). Until then no check can pass, so nothing can merge and no release can run. Work continues locally on stacked branches.

## Follow-ups (MINOR)

- 01-M1 and 01-M2: done in plan 02.
- 02: Windows WebView2 check of the Urdu field text position (design-system 8.4).
- 02: frontend `shared/` coverage (80%, Code Standards 9.4) is not measured yet; add `@vitest/coverage-v8` when `shared/` grows.

## Open question added 12-10-2026 (plan 07)

- Opening balances: the team refuses an opening on income and shop expense accounts, ابتدائی فرق, نقدی منڈی and the two پختہ کھاتہ accounts (`ERR_OPENING_NOT_ALLOWED`). The PRD does not say. Is that right? And should a partner's drawings account (0603) take an opening debit, which would count as money he took (C20)? Today it is allowed.

## Open question added 12-10-2026 (plan 09)

- Commission rules with a party and an item group (for example "رحیم, all fruit: 5%"). PRD 11.1 allows a group whenever the item is blank, but the six levels of PRD 11.3 have no "party + group" level. The team refuses a group when a party is chosen (`ERR_RULE_NOT_ALLOWED`). Should there be a party + group level, and where in the order?

## Team defaults on Umar's review list (12-10-2026)

- English voucher letters R-1 (receipt), P-1 (payment), A-1 (advance); Urdu و-1, ا-1, پ-1 from PRD 15.1.
- Settings tab name "دستی تبدیلی اور جانچ" (Overrides and checks).
- Saved message "پیشگی کھاتہ { $number } محفوظ ہو گیا" (gender: the voucher, not the advance).

## Urdu words waiting for Umar's review (NFR-URD-4)

- Plan 02 words in amounts (`core::urdu_words`): the 0 to 99 list, "ایک روپیہ" for exactly one rupee, "ایک سو" for 100.
- Plan 02 error messages, all `# review: pending` in `frontend/src/i18n/ur/errors.ftl` (25 codes). New or reworded: `err-not-a-number` ("یہاں نمبر لکھیں، مثلاً 12,000۔ حروف اور غلط جگہ پر لگا ہوا کاما نہیں لکھ سکتے۔": check کاما, غلط جگہ پر لگا ہوا, مثلاً), `err-qty-not-whole`, `err-negative-not-allowed`, `err-control-chars` ("اس خانے میں چھپا ہوا حرف ہے، مثلاً ٹیب، نئی سطر یا تحریر کی سمت کا نشان۔ ...": check چھپا ہوا حرف, تحریر کی سمت کا نشان).
- Plan 03 messages in `errors.ftl` and `common.ftl`: `err-disk-full`, `err-data-check-failed`, `err-migration-failed`, `err-db-busy`, and the title `common-data-check-title` ("ایپ آپ کا ڈیٹا ابھی نہیں کھول سکی").
- Plan 03 seed: the control 0103 name ادھار رقم (v1.2); provisional Urdu names of the 6 account heads and 10 controls (marked in the seed code), for example اثاثے, واجب الادا, پارٹیاں, آمدن, اخراجات, سرمایہ.
- Plan 05 words: `translit.ftl` in ur and en (all `review: pending`), and the phonetic keyboard table in `frontend/src/shared/urdu/phonetic_keyboard.ts` (the letters beyond a ا b ب p پ t ت T ٹ h ہ H ھ were written by the team in the CRULP idea and need the Urdu reviewer). The seed list `core/data/translit_seed.tsv` (about 4,572 entries, all `review: pending`).
- Plan 04 v1.2: `err-already-signed-in` "آپ پہلے ہی لاگ ان ہیں۔" (You are already signed in.); `err-not-locked` "ایپ لاک نہیں ہے۔" (The app is not locked.).
- Plan 05 spelling check (FR-7.66, NFR-URD-29, each with its English): `translit-spelling-check` and `translit-spelling-title` "ہجے دیکھ لیں" (Check the spelling); `translit-spelling-body` "نشان زدہ لفظوں کے ہجے ایپ نے اپنے اصولوں سے بنائے ہیں، کیونکہ یہ لفظ اس کی لفظوں کی فہرست میں نہیں ہیں۔ ہجے ٹھیک ہوں تو تصدیق کریں۔ ٹھیک نہ ہوں تو واپس جا کر درست کریں۔"; `translit-spelling-confirm` "ہجے ٹھیک ہیں" (Spelling is right); `err-spelling-unconfirmed` "ان لفظوں کے ہجے کی تصدیق نہیں ہوئی: { $words }۔ ہجے دیکھیں، پھر تصدیق کریں یا درست کریں۔"; reworded `mode-full-urdu`, `words-col-count` "استعمال کتنی بار", `words-col-last` "آخری استعمال".
- Plans 09 and 10 errors and fields: `err-rule-not-allowed` ("اصول کی یہ ترتیب درست نہیں": is ترتیب the right word for "combination"?), `err-date-range`, `err-user-inactive` reworded, `err-opening-not-allowed` (names income, دکان خرچہ, نقدی منڈی, پختہ کھاتہ, ابتدائی فرق); all of `ur/rules.ftl` (122), `ur/vouchers.ftl`, `ur/day.ftl`, working messages in `ur/assist.ftl`. Notes, narrations and reasons now start in رومن mode in the Urdu app (FR-7.22).
- Plan 07 openings grid: `openings-*` lines in `ur/accounts.ftl`; `err-journal-unbalanced`, `err-code-reserved`, `err-opening-both-sides`, `err-advance-not-seller` (check the gender of پیشگی in "لگ سکتی ہے").
- Plans 07 and 08: all of `ur/accounts.ftl` (about 90, کھاتہ used for "account" as in PRD 10.1), `ur/assist.ftl` (about 20; read first "اس پارٹی کے ذمے فرم کی رقم ہے" = "This party owes the firm"), `layout-search-*`, `common-side-dr/cr`, `common-yes` ہاں, `common-no` نہیں, `err-series-full`, `err-code-taken`, `err-version-changed`, `err-format-invalid`.
- Plan 06 masters: all of `frontend/src/i18n/ur/masters.ftl` (about 110 messages, English beside in `en/masters.ftl`). Look first at `masters-inactive-description`, `masters-staff-account-hint`, `masters-field-code-hint`, `masters-item-packing` "طے شدہ { -packing }" (Default Packing), `masters-staff-job-collector` "{ -agrahi } والا" (Collector). Plain words not in the glossary: شہر, سٹور, تفصیل, عملہ, بینک, خرچہ مد. The narration list is called "تفصیل کی فہرست" / "Narration list" (PRD 10.3 says "Standard narration"): confirm.
- Plan 01 glossary: 51 terms marked `# review: pending` in `glossary.ftl`.
- Dev-only gallery words (never shipped): not on the list.

## Notes carried into later plans

- D5 About screen (plan 22): show the GeoNames credit "This product contains data from GeoNames (https://www.geonames.org), licensed under CC BY 4.0." (word list source, `backend/crates/core/data/SOURCES.md`).

- Plan 07 (journal and accounts, from backend's plan 03 handoff): (a) write lot gross, deductions and net in ONE UPDATE; (b) write the posted `chain_hash` and `total_paisa` in the same UPDATE that sets status to posted; (c) Pukhta Khata defaults to cash (FR-13.4) while 12.4 follows credit_allowed, so the service must special-case it; (d) the domain checks computed values against the 1e14 paisa cap and returns ERR_AMOUNT_TOO_LARGE, not a raw CHECK failure; (e) the reversal date rule (same date) is a service rule; the database only checks the mirror and the total.
- Known audit limit: removing the LAST audit rows keeps the chain valid. Anchoring the tail hash outside the file belongs to the backup plan (NFR-SEC-6, D5).
- Money agent leftovers for plans 11 to 17: vouchers keep money columns editable (O-7), person key against the Pukhta account (B-6), a C6 check on lot deductions.
- Services rules for plans 06 and 07 (backend, plan 04 handoff): undo old matching rows (`is_active = 0`) before inserting new ones; write quantity and lot charges in ONE update; the reversal date rule (same date as the original) is a service rule, the database checks only the mirror; Pukhta Khata defaults to cash in the service.
- Plan 11 (edit of a بکری): PRD FR-12.20 v1.2 settles it: when only the received qty is corrected, per-نگ charges keep method and value and the amount is worked out again.
- Service duties for plans 11 to 16 (backend, v1.2 migration handoff): a lot's `adjust_advance = 1` is checked by the service (the E2E-28 golden test asserts zero advance_allocations rows for an unticked lot); allocations never point at lines of reversed entries (reverse first, then re-match); a payment to a buyer needs a reason field error from the service (the database trigger is the last line only).
- Plan 05 spelling check: a confirmed rules-only word is learned only when the other name has the same number of words (backend `spelling.rs`); otherwise the save goes through and nothing is learned.
- Codes from the role removal: `ERR_ALREADY_SIGNED_IN` (sign-in while signed in) and `ERR_NOT_LOCKED` (unlock when not locked) replace the two non-role uses of `ERR_NOT_ALLOWED`.
- `UiError.ref` is always present; the frontend does not show it for field errors (plan 03 4.2c).

## Open questions for Umar

- Doc fix for Tech Stack 4.4: the word list file now has six columns (roman, urdu, kind, weight, review, source), not four.
- Plan 05 accuracy (12-10-2026, after removing the India filler; backend's run of `cargo test -p mandi-core --test translit_accuracy`): word list 12,942 entries, 8,212 counted as known (team 4,588, Wikidata Pakistan 3,312, GeoNames ur names 312); 4,730 place words with engine-proposed Urdu stay amber until reviewed. Gap to 15,000 known: 6,788. Frozen 1,000 names: names fully in the list (700) first suggestion 92.0%, top 3 95.4% (target 95/99: not met), English 86.1%; names with an unknown word (300) top 3 47.0%; whole list English 72.2% (target 90: not met); silent wrong 7.6% in-list. Hold-out (749): in-list 82.7/89.3, silent wrong 16.9%. Before any release, the rules-only floors in `translit_baseline.txt` were lowered (61.4 to 47.0) because the filler had made 30 names look known; the engine did not change. Closing the gap needs: the reviewer confirming the engine rows, and the shop's 1,572 old account names.
- Advance matching from a جنرل entry: PRD 16.4 and FR-15.8 say جنرل lines on seller accounts take part in advance matching, but Architecture 15.7 v1.2 and the money agent (8.2, 8.2a) list only بکری credits and seller receipts, and the payable working can break if a جنرل credit adjusts an advance (FR-13.8 "make regular" is also a جنرل). Built now: the Architecture version (a جنرل credit never adjusts an advance). Which is right? If the PRD, the trigger, Architecture and the agent text change together.
- `return_by_date` earlier than the entry date: the PRD is silent. Nothing enforced now. Should a return-by date before the date of the money taken be refused?
- Names in Urdu (research report `docs/research/translit-options.md`): the word list decides accuracy; no model reaches the targets on words it has never seen. Two ideas need your yes or no: (a) offer names already saved in your accounts as a fourth source of suggestions; (b) mark a word that has two common Urdu spellings (for example شاہد and شاھد) in amber like a rules-only word, so you choose instead of the app choosing silently. Also: please export the old software's account names (about 1,572) when you can; they are the best test and seed list (FR-7.27). Reviewing the roughly 10,500 new word-list entries needs about 26 hours of your Urdu reviewer's time.
- NFR-URD-24 and NFR-URD-25 (names typed in English letters). First honest measure of the engine on the frozen 1,000-name list (48% of its names hold a word the 4,572-entry starting list does not know): whole list first suggestion right 63.5%, right one in the top 3 76.9%, Urdu to English 65.5%; on names whose words are all in the starting list 94.8%, 97.1%, 95.0%. The requirement is 90%, 97% and 90% on a list of typical mandi names. Rules alone cannot know the spelling of a rare family name; the app learns each correction, and the real list of the client's account names (Phase 2 import, or a list from the shop) would lift it. The team is improving the engine in a time-boxed round and will report final numbers. Resolved by PRD v1.2 (NFR-URD-24 new targets, 15,000-word list, spelling check FR-7.66). Still useful: the old software's account names (about 1,572) when Umar exports them (FR-7.27).
- (Resolved in Architecture 15.6 v1.2.) Architecture 15.6 gave `bikri_lots` only an amount and a rule id for labour, munshiana and phone. PRD FR-11.6 says the method, value, rule and amount are saved on each lot too. The PRD wins, so migration 0001 now has `labour_method`, `labour_value_paisa` and the same for munshiana and phone. Architecture 15.6 needs a v1.2 line (requirement-change, Umar approves).

- D1-Q1: screenshot comparison tool. Recommended: `@wdio/visual-service`. Plain screenshots until answered.
