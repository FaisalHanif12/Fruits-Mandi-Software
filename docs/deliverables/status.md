# Build status

Kept by `tech-lead`. Updated on each feature branch before its pull request.

## Current

- **PAUSED by Umar (16-10-2026) after plan 14, before plan 15.** D3+D4 run on branch `feature/bikri`. Plans 11, 12, 13 and 14 are built, committed and pushed (last commit `315a36a`). The agents are stopped on purpose; nothing is running except Umar's own test copy of the desktop app (`pnpm tauri dev`).
- **Resume from here:** start plan 15 (`docs/plans/15-vouchers.md`, written; its contract section "as built in T1" is added when `backend` finishes T1). Spawn a fresh `backend` (T1 contract and stubs, then T2) and, after T1, a fresh `frontend` (T3, then T4), as for plans 11 to 14. Then plans 16, 16b, 17, then the one end-of-run gate (`reviewer`, `robustness`), the fix loop and the DONE REPORT.
- **Notes for plan 15** (from the plan 13 and 14 hand-offs): use `matching::match_new_credits` or `matching::rematch`; every writer re-matches each affected key once; every cancel or edit of a پیشگی or a seller receipt calls `rematch_advances`; any code that posts posting type `opening` re-matches too. The receipt slip reuses `PrintData`/`PrintPage`: add a `PrintDoc` variant, a builder in `reports::print`, a `paper_receipt_slip` setting and its title and labels; vouchers have no `print_count` column, so they are counted in the audit row only (no migration). Migration numbers: 0004 is the bikri delete triggers (done); plan 16b's own-goods migration is 0005.
- **Notes for plan 16:** `bikri_lots` totals must be rewritten together (deductions, net, cash, commission, per-نگ charges); `first_entry_id` must point at the live first posting; a replaced posting writes replacement sale lines in the new posting (the `bikri_sales_snapshot_is_final` trigger blocks edits of old rows); lines with `is_active = 0` are not printed; decide what a cancelled بکری shows (saved totals stay visible in the saved view today); the credit-limit running balance must leave out the posting an edit replaces; `PostingKind::Later` charges no lot charges, freight, store rent or per-lot commission.
- **For the end gate (tell `robustness`):** robustness-owned files touched by builders as one-off mechanical changes: command tables in `backend/app/tests/{ipc_auth,robustness_security,tauri_conf,bindings}.rs`, `services/tests/robustness_balance.rs` (LATEST_VERSION 4), `services/tests/masters.rs` (allows DELETE FROM drafts), `services/benches/bikri.rs` and the new `bikri_save.rs`, `bikri_draft/`, `scripts/perf/thresholds.json` (3 new entries), `frontend/tests/robustness/app-security-config.test.ts` (capabilities default and print). The reviewer must also read `vouchers::post_advance` (the back-dated advance re-match fix was not shown to the money agent again), and the three optional low items of plan 14 applied after its second review.
- Baseline of the last full run on the quiet tree (plan 14): `pnpm test` frontend 2462 passed, Rust 1182 passed, 0 failed, 2 ignored; `pnpm lint`, `pnpm typecheck`, `pnpm i18n:check`, `cargo deny check`, `pnpm perf` exit 0 (perf can read OVER on the account list when the Mac is busy with other jobs: re-run alone).
- Umar's desktop test: username `owner`, password `mandi-dev-1` (the dev seed). His Oct 12 data was upgraded from schema 2 to 4; the copy before migrating is in `backups/pre-migration-2-4-20261016T054223Z.db`. Partners must be set at `/partners/setup` first (40/30/30).
- Earlier state (15-10-2026): the D1+D2 run is finished and its DONE REPORT is posted. Mac phase (README Section 10): push after every finished task; no pull request, no merge, no release, no Windows installer. GitHub holds 4 old stacked pull requests (#1, #2, #3, #5) and a squashed scaffold commit on `main` (#4); Umar decides the merge method for the Windows phase (squash-only today; rebase or merge commit would keep the history).

## Plan files

**Goal run (README 4a, v1.2):** plans are built by `backend` and `frontend`; `reviewer` and `robustness` run once at the end for the whole of D1 and D2 and cover every plan file 01 to 10. Plans 01 and 02 also had earlier per-plan verdicts.

| Plan | Branch | Pull requests | reviewer | robustness | State |
|---|---|---|---|---|---|
| 01-scaffold | feature/scaffold-repo | #1 | PASS WITH MINOR | PASS | gate passed; waiting for CI |
| 02-core | feature/core-money | #2 | PASS WITH MINOR | PASS | gate passed; committed locally |
| 03-database | feature/db-schema | #3 | PASS WITH MINOR | PASS WITH MINOR | built (`c114967`, fixes `7b1bdfe`, `f9ce265`) |
| 04-shell-and-login | feature/db-schema | | PASS WITH MINOR | PASS WITH MINOR | built (`76cc5ca`, fixes `7b1bdfe`, `6308b0e`, `ef08be8`); Windows E2E specs written, WAITING-CI |
| 05-names-two-scripts | feature/db-schema | | PASS WITH MINOR (D2-A2 accepted as a follow-up by Umar) | PASS WITH MINOR | built (`76cc5ca`); D2-A2 follow-up: word list and accuracy before go-live |
| 06-masters-and-search | feature/db-schema | | PASS WITH MINOR | PASS WITH MINOR | built (`76cc5ca`, fixes `6308b0e`) |
| 07-journal-and-accounts | feature/db-schema | | PASS WITH MINOR | PASS WITH MINOR | built (`9d585cd`, fixes `7b1bdfe`) |
| 08-party-card-and-short-codes | feature/db-schema | | PASS WITH MINOR | PASS WITH MINOR | built (`9d585cd`, fixes `7b1bdfe`, `f9ce265`) |
| 09-commission-rules | feature/db-schema | | PASS | PASS WITH MINOR | built (`c114967`) |
| 10-opening-position | feature/db-schema | | PASS WITH MINOR | PASS WITH MINOR | built (`c114967`, fixes `7b1bdfe`) |
| 11-partners | feature/bikri | | end of run | end of run | built (money review PASS WITH MINOR, fixes done; frontend 1743 and Rust 929 tests passed, golden_day E2E-19, cargo deny, lint, typecheck, i18n:check, perf PASS) |
| 12-bikri-entry | feature/bikri | | end of run | end of run | built (money review three passes, four defects fixed; Rust 1035 and frontend 1949 tests passed, golden_day, bikri_preview 34, assist 19, cargo deny, lint, typecheck, i18n:check, perf PASS on an idle machine) |
| 13-bikri-save | feature/bikri | | end of run | end of run | built (money review four passes, blocker and fixes done, the last back-dated advance fix not re-reviewed: reviewer to check `vouchers::post_advance`; Rust 1117 and frontend 2197 tests passed, golden_day 5, bikri_preview 34, bikri_save 55, assist 21, cargo deny, lint, typecheck, i18n:check, perf PASS) |
| 14-bikri-prints | feature/bikri | | end of run | end of run | built (money review two passes PASS; Rust 1182 and frontend 2462 tests passed, reports 35, print 11, snapshots of three documents in ur and en, cargo deny with one targeted exception, lint, typecheck, i18n:check, perf PASS; Windows print and PDF LOCAL-ONLY) |

## Design

- Canvas: https://claude.ai/artifact/EtYUy8QMjbEkx7yy9aeAiF. Rules: `docs/design/design-system.md`. Board sources: `docs/design/canvas/`.

## Setup facts

- Requirement documents are in `.claude/rules/` (Umar's choice). A path `docs/<Doc>.md` in any file means `.claude/rules/<Doc>.md`.
- Deliverable files were moved from `deliverables/` to `docs/deliverables/`, as README Section 1 expects.
- Updater key: made on 10-10-2026. Private key and password are in `~/.tauri/` on Umar's Mac. Public key in `docs/release/updater-pubkey.txt`. Both GitHub secrets are set.
- `main` ruleset is active (id 24841987): pull request, squash only, required checks `Checks (Windows)` and `Windows E2E and speed`, up to date.

- `docs/deliverables/progress.json` feeds the goal-progress band above the prompt (the `goal-progress` mod). The lead updates it as work moves: a plan goes `waiting`, `building`, `reviewing`, `passed` (both verdicts PASS or PASS WITH MINOR); an acceptance id is added only with real command output; a deliverable is added to `reports` when its DONE REPORT is posted. A new goal changes `goal.key` and `goal.scope`, and its progress starts from zero.

## Waiting for Windows

Accepted by Umar as LOCAL-ONLY on 15-10-2026. Each is run in the Windows phase (README Section 10). Local evidence is what exists today.

- D1-A2: CI and Windows checks pass, Release published with a signed `latest.json`. Local: the workflows are in `.github/workflows-disabled`.
- D3 (all LOCAL-ONLY, specs written and `node --check`ed in `frontend/tests/e2e/specs/` 12 to 15): D3-A2 wizard partners step on a clean install (spec 01); D3-A13 and E2E-17 draft offered and restored after the app is closed (spec 14); D3-A15 and D3-A16 real WebView2 print, PDF files, Urdu shaping and digits, A4/A5/80 mm (spec 15); D3-A17 keyboard-only 10-line بکری under 90 seconds and speed on the reference computer; Urdu shaping and fonts of the sale book and the saved view; whether WebView2 honours `size: 80mm 297mm` on the roll; the live check of Event 1 and Event 2 on the real app (Umar tests by hand on the Mac now).
- D1-A3: installer installs in under 2 minutes with the offline WebView2 installer. Local: `cargo test -p mandi-app --test tauri_conf` (7 passed, bundle settings).
- D1-A14: first-run wizard and the D1 story by keyboard only on Windows. Local: ten specs in `frontend/tests/e2e/specs` (written, `node --check` passed) and the unit keyboard tests.
- D1-A15: NFR-SPD-1, 2, 3, 5 and 19 measured through the Windows E2E suite. Local: runner factor 1.0 in `scripts/perf/README.md`; jsdom key latency p95 3.4 ms (typing) and 24.6 ms (arrow move, target 30).
- D1-A16: screenshots at the three sizes matched to the design on Windows, Urdu shaping in WebView2, fonts. Local: the contrast test and 240 headless Chrome captures (ur and en, 1024x600, 1366x768, 1920x1080).
- D2-A4: all three ways to type Urdu on Windows (spec 07). Local: unit tests of the three modes and `Alt+U`.
- D2-A11: quick add with `F4` and the unknown-code path on Windows (spec 08). Local: account-picker and global-search unit tests.
- D2-A16: every D2 screen against the design in both languages at the three sizes on Windows. Local: the full suite and the headless Chrome sweep above.
- Also for the Windows phase: NFR-REL-9 two copies of the app, install time (`pnpm perf` skips it on a Mac), the Radix CSP message (hash in the follow-ups above), WebView2 settings, printing.

## Blockers

- 10-10-2026: GitHub Actions does not start any job: "The job was not started because your account is locked due to a billing issue." Only Umar can fix this (github.com/settings/billing). Until then no check can pass, so nothing can merge and no release can run. Work continues locally on stacked branches.

## Follow-ups (MINOR)

- **Before go-live (D3+D4 run):** the `zip@4.6.1` exception in `backend/deny.toml` must be removed: `zip 4.x` is yanked and comes only through `tauri-plugin-updater`; move to a stable updater release that uses a non-yanked zip (the 3.x line is alpha on 16-10-2026) or build the updater without its `zip` feature (one real update test on Windows). Umar and the lead decide.
- Rate check starts ON at 30% (PRD does not say); the 7-day window is the posting day and the 6 before it; average by item, not packing, skipped for weight items [A].
- A credit sale is refused for نقدی منڈی and for a buyer with "ادھار کی اجازت" off (PRD 12.4, 10.2 read as a refusal).
- More than 50 lots or 500 sale lines in one بکری is refused (NFR-SCL-3 is a capacity target; the PRD gives no limit).
- Advance order: a seller credit adjusts only the advances before it (FR-15.8 "each later seller credit"); an advance given after a بکری does not retro-adjust it. FR-15.9 demo note: an advance of 30,000 against a credit of 24,316 leaves the نقد 5,000 under "بیوپاری کے ذمے".
- FR-12.8 check 1 read literally: only a whole بکری with no sale line is refused; a lot with no rows is allowed with the unsold warning.
- Preview speed: about 40 ms of every preview is the read of cash in hand (a SUM over the cash lines); a cached balance is the first fix if the reference computer is much slower, or skip the read when the entry cannot lower cash.
- PDFs go to `<app data>/prints/` with no save dialog; a PDF also raises `print_count`; copies apply to the printer only. A real print of document 1 counts even if document 2 fails.
- `partners_overview` has no `is_active`: an inactive partner with numbers shows as "0%" with no mark (plan 17 may add it). The picker cell editor flake is closed: jsdom zero-size rectangles made AG Grid close the editor (fixed in `tests/setup.ts`), not a production bug.
- Pre-existing accessibility defect: the Downshift `aria-labelledby` in `PickerBase` points to a missing label.
- English word spelling in amounts (hyphens, no "and") needs the English reviewer; the Urdu number words need the Urdu reviewer (NFR-URD-4).
- Urdu words waiting for review from plans 11 to 14 are all marked `# review: pending` in `partners.ftl`, `bikri.ftl`, `pukhta_khata.ftl`, `print.ftl`, the working IDs in `assist.ftl`, and the new messages in `errors.ftl`, `common.ftl`, `rules.ftl`, `settings.ftl`.

- Final recheck residuals (reviewer, MINOR): R1 the advance history shows the voucher number "پ-3" inside `Num` (Inter 16 px) in the Urdu app, so the Urdu prefix letter is under 18 px (`features/vouchers/components/AdvanceHistory.tsx:50`; frontend: a `Num` variant for a number with an Urdu prefix); R2 `scripts/seed/run.mjs` prints the Keychain hint on every computer, including Windows (print it only when `process.platform` is `darwin`; backend).
- The More columns tick on the account list is remembered per computer in `localStorage` (`useRememberedFlag`); `PrefKey` in the backend has only language, text size and last screen. FR-7.45 (remember filters per user) moves it to `user_prefs` later (D5, with the report and analytics filters).
- `pnpm seed` on a Mac may show a Keychain prompt; the hint is printed by `scripts/seed/run.mjs`. A sentence for the CLAUDE.md Commands section (Umar): "On a Mac, `pnpm seed` may show a Keychain prompt (enter the Mac password, choose Always Allow); `MANDI_SEED_TEST_KEY=1 pnpm seed` uses a throwaway dev key instead and its database cannot be verified by the real app."
- A Latin name inside the Urdu app is drawn in Gulzar (tagged `lang=en`, no type class of its own); no screen shows one today except typed notes. The mirror of the N1 rule can come with D3.
- Urdu words for Umar's reviewer from the final run: `accounts-more-columns` "مزید کالم" (is کالم the word for a list column?), plus the lines listed in the frontend hand-offs; all are `# review: pending`.
- D2-A2 (accepted as a follow-up by Umar, 15-10-2026; not a blocker for this run): the word list and accuracy targets of NFR-URD-24 and NFR-URD-25 (NFR v1.3) must pass before go-live; D5-A22 closes it with the old account names and the reviewer's confirmed rows, and removes the `#[ignore]` of the two gate tests (run with `--include-ignored`). Until then the numbers are reported on every build and must not drop. Measured on `cargo test -p mandi-core --test translit_accuracy -- --nocapture`: word list 12,942 entries, 8,212 known (15,000 needed); names with every word in the list (700): first suggestion 92.0% (gate 95), top 3 95.4% (gate 99); Urdu to English whole list 72.2% (gate 90), in-list 86.1%; rules-only names (300) top 3 47.0%; hold-out (749) 82.7% / 89.3%. The two gate tests are `#[ignore]`d with reasons and not weakened. The FR-7.66 confirm step stays exactly as built: no unconfirmed spelling is ever saved. Umar will provide the old account names (about 1,572) later; they and the Urdu reviewer's confirmed rows close the gap. The docs side (Umar) has updated NFR-URD-24 (NFR v1.3), D2-A2, Architecture 15.8, the Tech Stack word list columns (six) and the CLAUDE.md seed Keychain sentence in v1.5.
- Speed at the NFR-SCL-1 stress volume (20,000 accounts, 3.5M journal lines; 10 times the shop's real volume), Apple M1 Pro, after migration 0002: two rows are at the target, not under it with room: all 20,000 accounts (p95 289 to 319 ms, target 300) and the chart tree (236 to 273 ms, target 300). Both need every balance of the whole journal, one pass over 3.5M index entries, about 220 ms of CPU on the Mac, about 1.65 times that on the reference computer. At the real volume (about 350k lines) the pass is about 22 ms. The screens that matter are well under target: first page of 200 accounts 64 ms, party card 5 to 60 ms, save advance 12 to 17 ms. Team default: accepted for D1+D2; if Umar wants room at the stress volume, backend can read the journal in 2 to 4 slices on extra read connections (Code Standards B21 allows a second reader). A stored balance table is not built (FR-6.1, Architecture 15).
- Done in v1.5: Architecture 15.8 now lists the three indexes of migration 0002 (`idx_journal_lines_account_amounts`, `idx_journal_entries_unposted`, `idx_journal_lines_advance`); they add no table and no column.
- Radix Select writes one inline style that the production CSP (`style-src 'self'`) refuses; Windows will log a harmless refusal. The look is fixed in `globals.css`. To silence it, add the hash `'sha256-441zG27rExd4/il+NvIqyL8zFx5XmyNQtE381kSkUJk='` (from @radix-ui/react-select 2.3.8) to `style-src` in `backend/app/tauri.conf.json`. Check on Windows first.
- The opening balances grid has fixed column widths, so there is an empty strip on the right at 1366 wide (frontend, cosmetic).
- Review items 17 (spelling pairs by position) and 21 (a stronger plan 07 property test) stay open for D3. Review item 24 (the benches) is closed by committing them.
- 01-M1 and 01-M2: done in plan 02.
- 02: Windows WebView2 check of the Urdu field text position (design-system 8.4).
- 02: frontend `shared/` coverage (80%, Code Standards 9.4) is not measured yet; add `@vitest/coverage-v8` when `shared/` grows.

## Answered by Umar on 15-10-2026 (requirements v1.5, `changes-v1.5.md`)

| # | Answer | Where it is built |
|---|---|---|
| 1 | 6% by default for every seller, buyer and item. The munshi can type seller commission, buyer commission and لاگا on the بکری line by default (4% on one line). No "party + item group" level (the refusal is built). | The refusal is built (plan 09). The override setting starts on: changed now in D1+D2 code (`overrides_from`), the line input is D3 plan 12 (D3-A19). |
| 2 | "Newer wins" means the later start date; on the same start date, the one saved last. | Built (plan 09). Kept. |
| 3 | A removed per-lot commission moves, with its stored method and value, to the first remaining line of that lot in that posting, charged once. | D4 plan 16 (D4-A31). |
| 4 | A seller's opening credit is payable, his oldest payable posting. | D4 plan 15 (D4-A29). |
| 5 | A جنرل credit never adjusts advances; it is a payable posting. | Built (team default). Kept. D4-A29. |
| 6 | Cash below zero is allowed but must be confirmed: an amber note "روکڑ صفر سے کم ہو جائے گی" while typing, then a confirm box on save (focus on "نہیں، واپس جائیں"). The backend refuses the save without the confirmation (FR-6.7). | D3 and D4 plans 12, 15, 16, 16b, 17 (D3-A20, D4-A30). |
| 7 | A return-by date before the entry date is refused with `ERR_RETURN_BY_BEFORE_ENTRY`; the same day is allowed. | D4 plan 17 (D4-A30). |
| 8 | Opening balances as the team built them; also refused on اپنا مال; a partner's drawings account may take an opening debit (money taken, C20 and C22). | Built (plan 07). اپنا مال and the partner list in D4 (D4-A31). |
| 9 | System accounts are never made inactive. | Built (service rule and migration 0003). Kept. |

New Urdu words, to be built with plans 15 to 17 and marked `# review: pending`: "روکڑ صفر سے کم ہو جائے گی" (Cash in hand will go below zero) and "واپسی کی تاریخ اندراج کی تاریخ سے پہلے نہیں ہو سکتی" (The return-by date cannot be before the entry date).

## Team defaults on Umar's review list (12-10-2026)

- New glossary term `-peshgi` = پیشگی (feminine, the money given in advance; English "Advance"), next to `-peshgi-khata` = پیشگی کھاتہ (masculine, the screen and register). PRD Section 3 lists only پیشگی کھاتہ; NFR-URD-2 names پیشگی as a locked term. Grammar rule used in all messages: پیشگی takes feminine verbs, پیشگی کھاتہ masculine.

- English voucher letters R-1 (receipt), P-1 (payment), A-1 (advance); Urdu و-1, ا-1, پ-1 from PRD 15.1.
- Settings tab name "دستی تبدیلی اور جانچ" (Overrides and checks).
- Saved message "پیشگی کھاتہ { $number } محفوظ ہو گیا" (gender: the voucher, not the advance).

## Urdu words waiting for Umar's review (NFR-URD-4)

- Amounts in words: urdu_words.rs uses تیئس for 23 and اکاون for 51. Both spellings are in use; confirm the shop's spelling (تئیس / اکیاون are the other forms).
- Plan 02 words in amounts (`core::urdu_words`): the 0 to 99 list, "ایک روپیہ" for exactly one rupee, "ایک سو" for 100.
- Plan 02 error messages, all `# review: pending` in `frontend/src/i18n/ur/errors.ftl` (25 codes). New or reworded: `err-not-a-number` ("یہاں نمبر لکھیں، مثلاً 12,000۔ حروف اور غلط جگہ پر لگا ہوا کاما نہیں لکھ سکتے۔": check کاما, غلط جگہ پر لگا ہوا, مثلاً), `err-qty-not-whole`, `err-negative-not-allowed`, `err-control-chars` ("اس خانے میں چھپا ہوا حرف ہے، مثلاً ٹیب، نئی سطر یا تحریر کی سمت کا نشان۔ ...": check چھپا ہوا حرف, تحریر کی سمت کا نشان).
- Plan 03 messages in `errors.ftl` and `common.ftl`: `err-disk-full`, `err-data-check-failed`, `err-migration-failed`, `err-db-busy`, and the title `common-data-check-title` ("ایپ آپ کا ڈیٹا ابھی نہیں کھول سکی").
- Plan 03 seed: the control 0103 name ادھار رقم (v1.2); provisional Urdu names of the 6 account heads and 10 controls (marked in the seed code), for example اثاثے, واجب الادا, پارٹیاں, آمدن, اخراجات, سرمایہ.
- Plan 05 words: `translit.ftl` in ur and en (all `review: pending`), and the phonetic keyboard table in `frontend/src/shared/urdu/phonetic_keyboard.ts` (the letters beyond a ا b ب p پ t ت T ٹ h ہ H ھ were written by the team in the CRULP idea and need the Urdu reviewer). The seed list `core/data/translit_seed.tsv` (about 4,572 entries, all `review: pending`).
- Plan 04 v1.2: `err-already-signed-in` "آپ پہلے ہی لاگ ان ہیں۔" (You are already signed in.); `err-not-locked` "ایپ لاک نہیں ہے۔" (The app is not locked.).
- Plan 05 spelling check (FR-7.66, NFR-URD-29, each with its English): `translit-spelling-check` and `translit-spelling-title` "ہجے دیکھ لیں" (Check the spelling); `translit-spelling-body` "نشان زدہ لفظوں کے ہجے ایپ نے اپنے اصولوں سے بنائے ہیں، کیونکہ یہ لفظ اس کی لفظوں کی فہرست میں نہیں ہیں۔ ہجے ٹھیک ہوں تو تصدیق کریں۔ ٹھیک نہ ہوں تو واپس جا کر درست کریں۔"; `translit-spelling-confirm` "ہجے ٹھیک ہیں" (Spelling is right); `err-spelling-unconfirmed` "ان لفظوں کے ہجے کی تصدیق نہیں ہوئی: { $words }۔ ہجے دیکھیں، پھر تصدیق کریں یا درست کریں۔"; reworded `mode-full-urdu`, `words-col-count` "استعمال کتنی بار", `words-col-last` "آخری استعمال".
- `accountpicker-inactive` ("نئی اندراج": اندراج is masculine, fix asked: "نئے اندراج"), `working-per-unit-received`, `rules-overlap-help`, `rules-status-default` changed to عام.
- `err-nothing-to-close` "آج تک کے سب دن پہلے ہی بند ہیں، اس لیے بند کرنے کے لیے کوئی دن نہیں۔"; `err-not-a-seller`, `err-account-inactive`, `err-reason-required`, `err-trial-not-balanced`, `err-day-not-closed`, `err-month-closed`; `day-close-*` labels; `vouchers-source-opening`.
- Plans 09 and 10 errors and fields: `err-rule-not-allowed` ("اصول کی یہ ترتیب درست نہیں": is ترتیب the right word for "combination"?), `err-date-range`, `err-user-inactive` reworded, `err-opening-not-allowed` (names income, دکان خرچہ, نقدی منڈی, پختہ کھاتہ, ابتدائی فرق); all of `ur/rules.ftl` (122), `ur/vouchers.ftl`, `ur/day.ftl`, working messages in `ur/assist.ftl`. Notes, narrations and reasons now start in رومن mode in the Urdu app (FR-7.22).
- Plan 07 openings grid: `openings-*` lines in `ur/accounts.ftl`; `err-journal-unbalanced`, `err-code-reserved`, `err-opening-both-sides`, `err-advance-not-seller` (check the gender of پیشگی in "لگ سکتی ہے").
- Plans 07 and 08: all of `ur/accounts.ftl` (about 90, کھاتہ used for "account" as in PRD 10.1), `ur/assist.ftl` (about 20; read first "اس پارٹی کے ذمے فرم کی رقم ہے" = "This party owes the firm"), `layout-search-*`, `common-side-dr/cr`, `common-yes` ہاں, `common-no` نہیں, `err-series-full`, `err-code-taken`, `err-version-changed`, `err-format-invalid`.
- Plan 06 masters: all of `frontend/src/i18n/ur/masters.ftl` (about 110 messages, English beside in `en/masters.ftl`). Look first at `masters-inactive-description`, `masters-staff-account-hint`, `masters-field-code-hint`, `masters-item-packing` "طے شدہ { -packing }" (Default Packing), `masters-staff-job-collector` "{ -agrahi } والا" (Collector). Plain words not in the glossary: شہر, سٹور, تفصیل, عملہ, بینک, خرچہ مد. The narration list is called "تفصیل کی فہرست" / "Narration list" (PRD 10.3 says "Standard narration"): confirm.
- Plan 01 glossary: 51 terms marked `# review: pending` in `glossary.ftl`.
- Dev-only gallery words (never shipped): not on the list.

## Notes carried into later plans

- From the D1+D2 run review (12-10-2026): (17) the spelling check pairs Urdu and English words by position; a confirmed rules-only word is asked again on every edit when the two names have a different word count, and a different word order could learn a wrong pair. Rework before the Phase 2 import. (20) `ledger::post` does not refuse an inactive account; decide in D3 where the "active for new postings" check lives (a reversal must still work). (25) The frozen 1,000-name list has 30% names with a word outside the word list (the plan said 40%); 16 "team" rows in the word list have no recorded origin. The GeoNames credit goes on the D5 About screen.
- D3 and D4 (from plan 10): `ledger::reverse` switches off matching rows on reversed lines but cannot re-match (the matching engine is D3); every reversal caller in D3 re-runs matching in the same transaction, with a test that no seller shows too high an open advance afterwards. A voucher edit must update the voucher row, or `last_advance` must read through `current_entry_id`.

- D3 (بکری, plans 12 and 13), from backend's plan 09 money review: the working, prints and edits of a saved line come only from its snapshot columns (method, value, amount), never from resolving rules again; unchanged lines in an edit copy their snapshot; a typed value (method دستی) stores its percent or per نگ value too. Golden test E2E-13b (change the default, edit another line, the unchanged line keeps 1,044). Per-lot amounts only when `per_lot_due` (first line of the first posting). A restore must call `Context::invalidate_caches`.
- D3 (plan 12), from backend's plan 10 handoff: a charge with no winning rule for seller or buyer commission must be an error in the بکری, not a zero (the defaults always exist, so it means broken data). A proptest over random postings for the per-day close summaries was offered and not written; the deterministic side-flip test covers the drift risk. robustness may add it at the end gate.

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

- Umar answered the nine questions of the D1+D2 run on 15-10-2026 (see "Answered by Umar" above). What is still open keeps its team default and does not stop a run: D3-Q1 (`Enter` stops at buyer, qty and rate only), the two word list ideas below, D1-Q1, and the property test note in the notes (the sum check holds only when ابتدائی فرق is 0; random openings must balance or include it).
- (Done in Tech Stack 1.3, v1.5.) The word list file has six columns (roman, urdu, kind, weight, review, source).
- Plan 05 accuracy (12-10-2026, after removing the India filler; backend's run of `cargo test -p mandi-core --test translit_accuracy`): word list 12,942 entries, 8,212 counted as known (team 4,588, Wikidata Pakistan 3,312, GeoNames ur names 312); 4,730 place words with engine-proposed Urdu stay amber until reviewed. Gap to 15,000 known: 6,788. Frozen 1,000 names: names fully in the list (700) first suggestion 92.0%, top 3 95.4% (target 95/99: not met), English 86.1%; names with an unknown word (300) top 3 47.0%; whole list English 72.2% (target 90: not met); silent wrong 7.6% in-list. Hold-out (749): in-list 82.7/89.3, silent wrong 16.9%. Before any release, the rules-only floors in `translit_baseline.txt` were lowered (61.4 to 47.0) because the filler had made 30 names look known; the engine did not change. Closing the gap needs: the reviewer confirming the engine rows, and the shop's 1,572 old account names.
- Names in Urdu (research report `docs/research/translit-options.md`): the word list decides accuracy; no model reaches the targets on words it has never seen. Two ideas need your yes or no: (a) offer names already saved in your accounts as a fourth source of suggestions; (b) mark a word that has two common Urdu spellings (for example شاہد and شاھد) in amber like a rules-only word, so you choose instead of the app choosing silently. Also: please export the old software's account names (about 1,572) when you can; they are the best test and seed list (FR-7.27). Reviewing the roughly 10,500 new word-list entries needs about 26 hours of your Urdu reviewer's time.
- NFR-URD-24 and NFR-URD-25 (names typed in English letters). First honest measure of the engine on the frozen 1,000-name list (48% of its names hold a word the 4,572-entry starting list does not know): whole list first suggestion right 63.5%, right one in the top 3 76.9%, Urdu to English 65.5%; on names whose words are all in the starting list 94.8%, 97.1%, 95.0%. The requirement is 90%, 97% and 90% on a list of typical mandi names. Rules alone cannot know the spelling of a rare family name; the app learns each correction, and the real list of the client's account names (Phase 2 import, or a list from the shop) would lift it. The team is improving the engine in a time-boxed round and will report final numbers. Resolved by PRD v1.2 (NFR-URD-24 new targets, 15,000-word list, spelling check FR-7.66). Still useful: the old software's account names (about 1,572) when Umar exports them (FR-7.27).
- (Resolved in Architecture 15.6 v1.2.) Architecture 15.6 gave `bikri_lots` only an amount and a rule id for labour, munshiana and phone. PRD FR-11.6 says the method, value, rule and amount are saved on each lot too. The PRD wins, so migration 0001 now has `labour_method`, `labour_value_paisa` and the same for munshiana and phone. Architecture 15.6 needs a v1.2 line (requirement-change, Umar approves).

- D1-Q1: screenshot comparison tool. Recommended: `@wdio/visual-service`. Plain screenshots until answered.
