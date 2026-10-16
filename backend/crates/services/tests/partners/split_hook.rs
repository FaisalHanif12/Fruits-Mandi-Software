//! The share rows that `ledger::post` writes (PRD FR-6.5, FR-17.3, FR-17.4, C9 to C11), with the
//! worked numbers of plan 11 Section 3. Shares are 4000 / 3000 / 3000 bps unless a test says so.

use mandi_services::core::error::ErrorCode;
use mandi_services::core::ids::EntryId;
use mandi_services::db::partners as repo;
use mandi_services::domain::ledger::{PostingType, SourceType, build_entry};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::{ServiceError, partners};

use crate::common::set_today;
use crate::support::*;

#[test]
fn income_3948_gives_1580_1184_1184() {
    // PRD 4.4, E2E-4: 1,579 + 1,184 + 1,184 = 3,947, the leftover rupee goes to الف.
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let cash = id_of(&env, "01010001");
    let entry = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::General,
        vec![
            line(cash, Dr, 3_948),
            line(id_of(&env, "04010001"), Cr, 1_764),
            line(id_of(&env, "04010002"), Cr, 1_764),
            line(id_of(&env, "04010003"), Cr, 400),
            line(id_of(&env, "04010004"), Cr, 20),
        ],
    )
    .unwrap();
    assert_eq!(
        rows_of(&dir, entry),
        vec![
            row(a, "income", 3_948, 4_000, 1_580),
            row(b, "income", 3_948, 3_000, 1_184),
            row(c, "income", 3_948, 3_000, 1_184),
        ]
    );
}

#[test]
fn income_1540_gives_616_462_462() {
    // PRD 4.5, E2E-5: no leftover.
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    let cash = id_of(&env, "01010001");
    let entry = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::General,
        vec![
            line(cash, Dr, 1_540),
            line(id_of(&env, "04010001"), Cr, 720),
            line(id_of(&env, "04010002"), Cr, 720),
            line(id_of(&env, "04010003"), Cr, 80),
            line(id_of(&env, "04010004"), Cr, 20),
        ],
    )
    .unwrap();
    assert_eq!(amounts(&rows_of(&dir, entry)), vec![616, 462, 462]);
}

#[test]
fn a_shop_expense_of_3000_gives_minus_1200_900_900() {
    // PRD 4.8: electricity 3,000.
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let head = expense_head(&env, &owner);
    let entry = against_cash(&env, &owner, "2026-10-10", head, Dr, 3_000).unwrap();
    assert_eq!(
        rows_of(&dir, entry),
        vec![
            row(a, "expense", 3_000, 4_000, -1_200),
            row(b, "expense", 3_000, 3_000, -900),
            row(c, "expense", 3_000, 3_000, -900),
        ]
    );
}

#[test]
fn a_shop_expense_of_3384_gives_minus_1354_1015_1015() {
    // E2E-20: 1,353 + 1,015 + 1,015 = 3,383, the leftover goes to الف, then the sign is made
    // negative. The rows add up to exactly minus 3,384.
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    let head = expense_head(&env, &owner);
    let entry = against_cash(&env, &owner, "2026-10-10", head, Dr, 3_384).unwrap();
    let rows = rows_of(&dir, entry);
    assert_eq!(amounts(&rows), vec![-1_354, -1_015, -1_015]);
    assert_eq!(amounts(&rows).iter().sum::<i64>(), -3_384);
}

#[test]
fn a_negative_income_of_300_gives_minus_120_90_90() {
    // C11: the absolute amount is split (120 / 90 / 90), then negated. A negative rate
    // difference is a debit on the income account.
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let entry = against_cash(&env, &owner, "2026-10-10", id_of(&env, "04010005"), Dr, 300).unwrap();
    assert_eq!(
        rows_of(&dir, entry),
        vec![
            row(a, "income", -300, 4_000, -120),
            row(b, "income", -300, 3_000, -90),
            row(c, "income", -300, 3_000, -90),
        ]
    );
}

#[test]
fn income_and_cost_in_one_entry_are_split_apart_e2e_34() {
    // E2E-34: income 9,660 and the cost of the goods sold 7,800 in one entry. Each is split on
    // its own: income 3,864 / 2,898 / 2,898, cost minus 3,120 / 2,340 / 2,340, net 744 / 558 / 558.
    // The own-goods accounts (تجارتی فروخت, اپنا مال لاگت, اپنا مال سٹاک) come with plan 16b; the
    // hook looks only at the head of an account, so here an income account, a shop expense head
    // and the cash account stand in for them.
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let cost = expense_head(&env, &owner);
    let cash = id_of(&env, "01010001");
    let entry = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::BikriFirst,
        vec![
            line(cash, Dr, 9_660),
            line(cost, Dr, 7_800),
            line(id_of(&env, "04010001"), Cr, 540),
            line(id_of(&env, "04010002"), Cr, 540),
            line(id_of(&env, "04010003"), Cr, 120),
            line(id_of(&env, "04010005"), Cr, 8_460),
            line(cash, Cr, 7_800),
        ],
    )
    .unwrap();
    assert_eq!(
        rows_of(&dir, entry),
        vec![
            row(a, "income", 9_660, 4_000, 3_864),
            row(b, "income", 9_660, 3_000, 2_898),
            row(c, "income", 9_660, 3_000, 2_898),
            row(a, "expense", 7_800, 4_000, -3_120),
            row(b, "expense", 7_800, 3_000, -2_340),
            row(c, "expense", 7_800, 3_000, -2_340),
        ]
    );
    let net: Vec<i64> = [a, b, c]
        .iter()
        .map(|p| {
            rows_of(&dir, entry)
                .iter()
                .filter(|r| r.partner_id == *p)
                .map(|r| r.amount)
                .sum()
        })
        .collect();
    assert_eq!(net, vec![744, 558, 558]);
}

#[test]
fn an_entry_that_touches_no_income_or_expense_account_writes_no_rows() {
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    let entry = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::Receipt,
        vec![
            line(id_of(&env, "01010001"), Dr, 10_000),
            line(id_of(&env, "06010001"), Cr, 10_000),
        ],
    )
    .unwrap();
    assert!(rows_of(&dir, entry).is_empty());
}

#[test]
fn shares_not_100_refuse_the_post_and_write_nothing_e9() {
    // 4000 + 3000 + 2000 = 9,000 bps on the posting date.
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    env.ctx
        .db()
        .write(|tx| {
            // A later row on the same day wins ("the row saved last"): ج goes to 2000.
            repo::insert_share(
                tx,
                c,
                2_000,
                "2026-10-10",
                owner.user_id,
                &env.ctx.now_text(),
            )?;
            Ok::<_, ServiceError>(())
        })
        .unwrap();
    let before = (
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        count(&dir, "SELECT COUNT(*) FROM journal_lines"),
        count(&dir, "SELECT COUNT(*) FROM partner_share_rows"),
    );
    let income = id_of(&env, "04010001");
    let error = against_cash(&env, &owner, "2026-10-10", income, Cr, 3_948).unwrap_err();
    assert_eq!(
        first(&error, "total_bps"),
        (ErrorCode::ErrSharesNot100, None, Some(9_000))
    );
    let after = (
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        count(&dir, "SELECT COUNT(*) FROM journal_lines"),
        count(&dir, "SELECT COUNT(*) FROM partner_share_rows"),
    );
    assert_eq!(before, after, "nothing is written");
    let _ = (a, b);
    // An entry with no income or expense line still posts.
    against_cash(&env, &owner, "2026-10-10", id_of(&env, "06010001"), Cr, 500).unwrap();
}

#[test]
fn with_no_shares_at_all_an_income_post_is_refused_with_total_0() {
    let (dir, env, owner) = ready();
    let income = id_of(&env, "04010001");
    let error = against_cash(&env, &owner, "2026-10-10", income, Cr, 1_000).unwrap_err();
    assert_eq!(
        first(&error, "total_bps"),
        (ErrorCode::ErrSharesNot100, None, Some(0))
    );
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partner_share_rows"), 0);
    // A cash-only entry needs no shares.
    against_cash(
        &env,
        &owner,
        "2026-10-10",
        id_of(&env, "06010001"),
        Cr,
        1_000,
    )
    .unwrap();
}

#[test]
fn an_entry_whose_income_nets_to_zero_needs_no_shares_and_writes_no_rows() {
    let (dir, env, owner) = ready();
    let entry = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::General,
        vec![
            line(id_of(&env, "04010001"), Dr, 500),
            line(id_of(&env, "04010002"), Cr, 500),
        ],
    )
    .unwrap();
    assert!(rows_of(&dir, entry).is_empty());
}

#[test]
fn a_share_change_applies_from_its_day_and_saved_postings_keep_their_split() {
    // Plan 11 Section 3: the change on 20-10-26 to 50 / 30 / 20. A posting dated 10-10-26
    // keeps 40 / 30 / 30, one dated 20-10-26 splits 50 / 30 / 20 (FR-17.2, E18).
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let income = id_of(&env, "04010001");
    let first_post = against_cash(&env, &owner, "2026-10-10", income, Cr, 1_000).unwrap();
    set_shares(
        &env,
        &owner,
        "20-10-2026",
        &[(a, "50"), (b, "30"), (c, "20")],
    )
    .unwrap();
    set_today(&env, 2026, 10, 20);
    let later = against_cash(&env, &owner, "2026-10-20", income, Cr, 1_000).unwrap();
    assert_eq!(amounts(&rows_of(&dir, later)), vec![500, 300, 200]);
    assert_eq!(
        rows_of(&dir, later)
            .iter()
            .map(|r| r.bps)
            .collect::<Vec<_>>(),
        vec![5_000, 3_000, 2_000]
    );
    // The saved posting did not change, and an open earlier day still splits by its own date.
    assert_eq!(amounts(&rows_of(&dir, first_post)), vec![400, 300, 300]);
    let back_dated = against_cash(&env, &owner, "2026-10-10", income, Cr, 1_000).unwrap();
    assert_eq!(amounts(&rows_of(&dir, back_dated)), vec![400, 300, 300]);
    let today_shares: Vec<i32> = partners::list(&env.ctx)
        .unwrap()
        .iter()
        .map(|p| p.share_bps)
        .collect();
    assert_eq!(today_shares, vec![5_000, 3_000, 2_000]);
}

#[test]
fn a_reversal_copies_the_rows_with_the_opposite_sign_and_never_splits_again() {
    // FR-17.4. The shares change after the posting; a reversal that split again would use
    // 50 / 30 / 20 (income 1,974 / 1,184 / 790). It copies, so it gives exactly minus
    // 1,580 / 1,184 / 1,184 for the income and plus 1,200 / 900 / 900 for the shop expense of
    // 3,000 that the entry also holds.
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let cash = id_of(&env, "01010001");
    let head = expense_head(&env, &owner);
    let original = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::General,
        vec![
            line(cash, Dr, 3_948),
            line(head, Dr, 3_000),
            line(id_of(&env, "04010001"), Cr, 1_764),
            line(id_of(&env, "04010002"), Cr, 1_764),
            line(id_of(&env, "04010003"), Cr, 400),
            line(id_of(&env, "04010004"), Cr, 20),
            line(cash, Cr, 3_000),
        ],
    )
    .unwrap();
    set_shares(
        &env,
        &owner,
        "10-10-2026",
        &[(a, "50"), (b, "30"), (c, "20")],
    )
    .unwrap();
    let reversal = env
        .ctx
        .db()
        .write(|tx| {
            ledger::reverse(&env.ctx, tx, owner.user_id, original, "test").map(|p| p.entry_id)
        })
        .unwrap();
    let first_rows = rows_of(&dir, original);
    let back = rows_of(&dir, reversal);
    assert_eq!(
        amounts(&first_rows),
        vec![1_580, 1_184, 1_184, -1_200, -900, -900]
    );
    assert_eq!(
        back,
        vec![
            row(a, "income", -3_948, 4_000, -1_580),
            row(b, "income", -3_948, 3_000, -1_184),
            row(c, "income", -3_948, 3_000, -1_184),
            row(a, "expense", -3_000, 4_000, 1_200),
            row(b, "expense", -3_000, 3_000, 900),
            row(c, "expense", -3_000, 3_000, 900),
        ]
    );
    // The reversal has the SAME date as the original (FR-17.4) and its own entry id; the
    // original rows are untouched.
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(DISTINCT entry_date) FROM partner_share_rows"
        ),
        1,
        "the original and the reversal rows carry one date"
    );
    assert_ne!(original, reversal);
    assert_eq!(rows_of(&dir, original), first_rows);
    assert_eq!(
        count(
            &dir,
            "SELECT COUNT(*) FROM (SELECT partner_id FROM partner_share_rows
             GROUP BY partner_id HAVING SUM(amount_paisa) = 0)"
        ),
        3,
        "every partner nets to zero"
    );
}

#[test]
fn a_reversal_works_even_when_the_shares_do_not_total_100_any_more() {
    let (dir, env, owner) = ready();
    let [a, ..] = three_partners(&env, &owner);
    let income = id_of(&env, "04010001");
    let original = against_cash(&env, &owner, "2026-10-10", income, Cr, 1_000).unwrap();
    env.ctx
        .db()
        .write(|tx| {
            repo::insert_share(
                tx,
                a,
                1_000,
                "2026-10-10",
                owner.user_id,
                &env.ctx.now_text(),
            )?;
            Ok::<_, ServiceError>(())
        })
        .unwrap();
    // A new income post on that day is refused (70% in total) ...
    let error = against_cash(&env, &owner, "2026-10-10", income, Cr, 1_000).unwrap_err();
    assert_eq!(
        first(&error, "total_bps"),
        (ErrorCode::ErrSharesNot100, None, Some(7_000))
    );
    // ... but the reversal of the earlier one copies its rows.
    let reversal = env
        .ctx
        .db()
        .write(|tx| {
            ledger::reverse(&env.ctx, tx, owner.user_id, original, "test").map(|p| p.entry_id)
        })
        .unwrap();
    assert_eq!(amounts(&rows_of(&dir, reversal)), vec![-400, -300, -300]);
}

#[test]
fn month_close_and_opening_write_no_rows() {
    // FR-17.6: month close only moves amounts. FR-10.9: an opening has no income or expense line.
    let (dir, env, owner) = ready();
    // No shares exist, so a split would be refused: these must not even try.
    let month_close = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::MonthClose,
        vec![
            line(id_of(&env, "04010001"), Dr, 1_000),
            line(id_of(&env, "06010001"), Cr, 1_000),
        ],
    )
    .unwrap();
    assert!(rows_of(&dir, month_close).is_empty());
    let opening = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::Opening,
        vec![
            line(id_of(&env, "01010001"), Dr, 65_000),
            line(id_of(&env, "06010001"), Cr, 65_000),
        ],
    )
    .unwrap();
    assert!(rows_of(&dir, opening).is_empty());
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partner_share_rows"), 0);
}

#[test]
fn an_opening_with_an_income_line_is_a_bug_and_is_refused_loudly() {
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    let before = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let error = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::Opening,
        vec![
            line(id_of(&env, "01010001"), Dr, 100),
            line(id_of(&env, "04010001"), Cr, 100),
        ],
    )
    .unwrap_err();
    assert!(matches!(error, ServiceError::Data(_)), "{error:?}");
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), before);
}

#[test]
fn the_opening_capital_of_a_new_partner_writes_no_rows() {
    let (dir, env, owner) = ready();
    new_partner(&env, &owner, "الف", "Partner A", Some("50,000")).unwrap();
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM partner_share_rows"), 0);
}

#[test]
fn an_event_whose_reverses_and_posting_type_disagree_is_refused_before_anything_is_written() {
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    let original = against_cash(
        &env,
        &owner,
        "2026-10-10",
        id_of(&env, "04010001"),
        Cr,
        1_000,
    )
    .unwrap();
    let before = count(&dir, "SELECT COUNT(*) FROM partner_share_rows");
    for (posting_type, reverses) in [
        (PostingType::General, Some(EntryId::new(original))),
        (PostingType::Reversal, None),
    ] {
        let entry = build_entry(vec![
            line(id_of(&env, "01010001"), Dr, 1_000),
            line(id_of(&env, "04010001"), Cr, 1_000),
        ])
        .unwrap();
        let error = env
            .ctx
            .db()
            .write(|tx| {
                ledger::post(
                    &env.ctx,
                    tx,
                    owner.user_id,
                    &PostingEvent {
                        date: date("2026-10-10"),
                        posting_type,
                        source_type: SourceType::Voucher,
                        source_id: 1,
                        narration: None,
                        reverses,
                        entry,
                    },
                )
            })
            .unwrap_err();
        assert!(matches!(error, ServiceError::Data(_)), "{error:?}");
    }
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM partner_share_rows"),
        before
    );
}

#[test]
fn the_split_a_preview_asks_for_is_the_split_the_save_writes() {
    // Code Standards B5: plan 13's preview calls `split_entry`, and the save calls it through
    // `ledger::post`. The rows must be the same, in the same order.
    let (dir, env, owner) = ready();
    let [a, b, c] = three_partners(&env, &owner);
    let head = expense_head(&env, &owner);
    let build = || {
        build_entry(vec![
            line(id_of(&env, "01010001"), Dr, 9_660),
            line(head, Dr, 7_800),
            line(id_of(&env, "04010001"), Cr, 540),
            line(id_of(&env, "04010002"), Cr, 540),
            line(id_of(&env, "04010003"), Cr, 120),
            line(id_of(&env, "04010005"), Cr, 8_460),
            line(id_of(&env, "01010001"), Cr, 7_800),
        ])
        .unwrap()
    };
    let entry = build();
    let previewed = env
        .ctx
        .db()
        .read(|conn| partners::split_entry(conn, date("2026-10-10"), &entry))
        .unwrap();
    assert_eq!(previewed.len(), 6);
    assert_eq!(
        count(&dir, "SELECT COUNT(*) FROM partner_share_rows"),
        0,
        "a preview writes nothing"
    );
    let saved = env
        .ctx
        .db()
        .write(|tx| {
            ledger::post(
                &env.ctx,
                tx,
                owner.user_id,
                &PostingEvent {
                    date: date("2026-10-10"),
                    posting_type: PostingType::BikriFirst,
                    source_type: SourceType::Bikri,
                    source_id: 1,
                    narration: None,
                    reverses: None,
                    entry,
                },
            )
            .map(|p| p.entry_id)
        })
        .unwrap();
    let stored = rows_of(&dir, saved);
    // The whole snapshot row: partner, kind, base, share and amount.
    let from_preview: Vec<(i64, String, i64, i64, i64)> = previewed
        .iter()
        .map(|r| {
            (
                r.partner_id.get(),
                r.kind.as_str().to_owned(),
                r.base.rupees(),
                i64::from(r.share_bps.get()),
                r.amount.rupees(),
            )
        })
        .collect();
    let from_save: Vec<(i64, String, i64, i64, i64)> = stored
        .iter()
        .map(|r| (r.partner_id, r.kind.clone(), r.base, r.bps, r.amount))
        .collect();
    assert_eq!(from_preview, from_save);
    assert_eq!(
        from_save.iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![a, b, c, a, b, c]
    );
    // With wrong shares the preview gives the same refusal the save gives.
    let early = build();
    let error = env
        .ctx
        .db()
        .read(|conn| partners::split_entry(conn, date("2026-09-01"), &early))
        .unwrap_err();
    assert_eq!(
        first(&error, "total_bps"),
        (ErrorCode::ErrSharesNot100, None, Some(0)),
        "before the first share nothing totals 100%"
    );
}

#[test]
fn a_reversal_dated_differently_from_its_original_is_refused_when_it_has_rows_to_copy() {
    // PRD FR-17.4: the copied rows carry the date of the original; the monthly sums rely on it.
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    let income = id_of(&env, "04010001");
    let cash = id_of(&env, "01010001");
    let original = against_cash(&env, &owner, "2026-10-10", income, Cr, 1_000).unwrap();
    set_today(&env, 2026, 10, 11);
    let lines = vec![line(income, Cr, 1_000), line(cash, Dr, 1_000)];
    let before = (
        count(&dir, "SELECT COUNT(*) FROM journal_entries"),
        count(&dir, "SELECT COUNT(*) FROM partner_share_rows"),
    );
    let error = reversal_event(&env, &owner, "2026-10-11", original, lines.clone()).unwrap_err();
    assert_data(&error, "must have the date of the entry it reverses");
    assert_eq!(
        (
            count(&dir, "SELECT COUNT(*) FROM journal_entries"),
            count(&dir, "SELECT COUNT(*) FROM partner_share_rows"),
        ),
        before,
        "nothing was written"
    );
    // Control: the same reversal on the original's date is accepted and copies the rows.
    let reversal = reversal_event(&env, &owner, "2026-10-10", original, lines).unwrap();
    assert_eq!(amounts(&rows_of(&dir, reversal)), vec![-400, -300, -300]);
}

#[test]
fn a_reversal_of_an_entry_that_does_not_exist_is_refused_and_writes_nothing() {
    let (dir, env, owner) = ready();
    three_partners(&env, &owner);
    let cash = id_of(&env, "01010001");
    let before = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let lines = vec![line(cash, Dr, 100), line(id_of(&env, "06010001"), Cr, 100)];
    let error = reversal_event(&env, &owner, "2026-10-10", 99_999, lines).unwrap_err();
    assert_data(&error, "the entry to reverse does not exist");
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), before);
}

/// Posts a reversal event by hand: the mirror of `original_lines`, dated `day`.
fn reversal_event(
    env: &mandi_services::testing::TestEnv,
    owner: &mandi_services::auth::Actor,
    day: &str,
    original: i64,
    original_lines: Vec<mandi_services::domain::ledger::LedgerLine>,
) -> Result<i64, ServiceError> {
    let mirror =
        mandi_services::domain::ledger::build_reversal(&build_entry(original_lines).unwrap());
    env.ctx.db().write(|tx| {
        ledger::post(
            &env.ctx,
            tx,
            owner.user_id,
            &PostingEvent {
                date: date(day),
                posting_type: PostingType::Reversal,
                source_type: SourceType::Voucher,
                source_id: 1,
                narration: None,
                reverses: Some(EntryId::new(original)),
                entry: mirror,
            },
        )
        .map(|posted| posted.entry_id)
    })
}

#[test]
fn every_reversal_has_the_date_of_its_original_also_one_with_no_rows_to_copy() {
    // `taken_between` relies on this date, and a take (a debit on a drawings account) has no
    // share rows at all.
    let (dir, env, owner) = ready();
    let [a, ..] = three_partners(&env, &owner);
    let drawings = partners::get(&env.ctx, a).unwrap().row.drawings_account_id;
    let cash = id_of(&env, "01010001");
    let take = against_cash(&env, &owner, "2026-10-10", drawings, Dr, 2_000).unwrap();
    assert!(rows_of(&dir, take).is_empty());
    set_today(&env, 2026, 10, 11);
    let lines = vec![line(drawings, Dr, 2_000), line(cash, Cr, 2_000)];
    let before = count(&dir, "SELECT COUNT(*) FROM journal_entries");
    let error = reversal_event(&env, &owner, "2026-10-11", take, lines.clone()).unwrap_err();
    assert_data(&error, "must have the date of the entry it reverses");
    assert_eq!(count(&dir, "SELECT COUNT(*) FROM journal_entries"), before);
    // The same reversal on the original's date is accepted.
    reversal_event(&env, &owner, "2026-10-10", take, lines).unwrap();
}

#[test]
fn the_reversal_of_a_month_close_may_have_another_date_and_copies_no_rows() {
    // PRD FR-17.6, FR-21.2: a month close writes no rows, and undoing it is the one reversal
    // that may be dated on another day.
    let (dir, env, owner) = ready();
    // The shares are set, so a split (instead of no rows) would show up as rows.
    three_partners(&env, &owner);
    let lines = vec![
        line(id_of(&env, "04010001"), Dr, 1_000),
        line(id_of(&env, "06010001"), Cr, 1_000),
    ];
    let close = post(
        &env,
        &owner,
        "2026-10-10",
        PostingType::MonthClose,
        lines.clone(),
    )
    .unwrap();
    set_today(&env, 2026, 10, 11);
    let undone = reversal_event(&env, &owner, "2026-10-11", close, lines).unwrap();
    assert!(rows_of(&dir, close).is_empty());
    assert!(rows_of(&dir, undone).is_empty());
}
