//! Plan 40 T2: the dead amount (ڈوبی رقم, PRD FR-10.10, FR-19.9, R20 to R22, R24, R26, R32, R34, R35,
//! R38, E2E-44) in the reports, the dashboard, the party card and the pickers, on the PRD worked day.
//!
//! The expected numbers are the PRD's (E2E-44, 4.9) and the worked cases of the money agent of plan
//! 40. The tile ڈوبی رقم is also held against the R38 total, and کل اگراہی against R20 + R24 + R38
//! (FR-19.9), so the screens can never disagree with the books.
#![cfg(test)]

mod common;
mod reports_world;
#[allow(dead_code)]
#[path = "bikri_preview/support.rs"]
mod support;

use common::*;
use mandi_reports::dashboard::{self, AttentionKind, BackupFacts};
use mandi_reports::reports::{Cell, FilterKind, PaidFilter, ReportFilters, ReportId, RowKind};
use mandi_services::core::accounts::{
    AccountScope, AccountsRecentInput, BalanceSide, SearchAccountsInput,
};
use mandi_services::core::dead::{
    DeadAmountInput, DeadFilter, DeadMarkInput, DeadPartyType, DeadReason, DeadStatus,
    DeadUndoInput,
};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::lent::{LentPersonCreateInput, MoneyLentInput, MoneyLentReturnInput};
use mandi_services::{ServiceError, accounts, assist, dead, lent, search, vouchers};
use reports_world::*;
use support::World;

fn off() -> BackupFacts {
    BackupFacts {
        cloud_on: false,
        cloud_age_hours: None,
    }
}

fn tiles(w: &World) -> mandi_reports::dashboard::DashboardTiles {
    dashboard::get(w.env.ctx.db(), off()).unwrap().tiles
}

fn mark(w: &World, account: i64, date: &str, reason: DeadReason) -> i64 {
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: account,
            pukhta_person_key: None,
            entry_date: date.into(),
            reason,
            note: String::new(),
        },
    )
    .unwrap()
    .mark_id
}

fn recover(w: &World, account: i64, date: &str, amount: &str) {
    let mut input = receipt(account, amount);
    input.date = date.into();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
}

fn r38(w: &World, filter: DeadFilter, search: &str) -> mandi_services::core::dead::DeadAmountPage {
    mandi_reports::dead::page(
        w.env.ctx.db(),
        &DeadAmountInput {
            filter,
            search: search.into(),
            offset: 0,
            limit: 100,
        },
    )
    .unwrap()
}

/// The rows of R20 as (code, balance in rupees), and its total row.
fn r20(w: &World) -> (Vec<(String, i64)>, i64) {
    let page = report(w, ReportId::R20, ReportFilters::default(), 0, 100);
    let rows = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Normal)
        .map(|r| {
            let balance = r
                .cells
                .iter()
                .find_map(|c| match c {
                    Cell::Money { paisa } => Some(paisa.rupees()),
                    _ => None,
                })
                .unwrap_or(0);
            (code_of(&r.cells[0]), balance)
        })
        .collect();
    let total = page
        .totals
        .iter()
        .find_map(|c| match c {
            Cell::Money { paisa } => Some(paisa.rupees()),
            _ => None,
        })
        .unwrap_or(0);
    (rows, total)
}

#[test]
fn e2e_44_r20_and_r38_rows() {
    let w = full_day();
    // Before the move: R20 has both buyers, 2,880 + 18,684 = 21,564, and R38 is empty.
    let (rows, total) = r20(&w);
    assert_eq!(rows.len(), 2);
    assert_eq!(total, 21_564);
    assert_eq!(r38(&w, DeadFilter::Owed, "").row_count, 0);

    // 11-10-26: ارشد moves. R20 shows only اکرم (E2E-44) and its total is the tile.
    set_today(&w.env, 2026, 10, 11);
    let mark_id = mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    let (rows, total) = r20(&w);
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].1, total), (2_880, 2_880));
    assert_eq!(tiles(&w).buyers_dues_paisa.rupees(), 2_880);
    // R21 and R22 leave him out too: their total is the tile.
    for id in [ReportId::R21, ReportId::R22] {
        let page = report(&w, id, ReportFilters::default(), 0, 100);
        let sum: i64 = page
            .totals
            .iter()
            .filter_map(|c| match c {
                Cell::Money { paisa } => Some(paisa.rupees()),
                _ => None,
            })
            .next()
            .unwrap_or(0);
        assert_eq!(sum, 2_880, "{id:?}");
    }

    // R38 row: owed then 18,684, nothing recovered yet, still 18,684, one person.
    let page = r38(&w, DeadFilter::Owed, "");
    assert_eq!(page.row_count, 1);
    let row = &page.rows[0];
    assert_eq!(
        (
            row.mark_id,
            row.account_id,
            row.party_type,
            row.reason,
            row.entry_date.as_str(),
            row.owed_then_paisa.rupees(),
            row.recovered_paisa.rupees(),
            row.corrections_paisa,
            row.still_owed_paisa.rupees(),
            row.last_recovery_date.clone(),
            row.status
        ),
        (
            mark_id,
            w.arshad,
            DeadPartyType::Buyer,
            DeadReason::Died,
            "2026-10-11",
            18_684,
            0,
            0,
            18_684,
            None,
            DeadStatus::Owing
        )
    );
    assert_eq!(page.totals.still_owed_paisa, tiles(&w).dead_amount_paisa);

    // 12-10-26: a وصولی of 4,000. R38: owed then 18,684, recovered 4,000, still 14,684.
    set_today(&w.env, 2026, 10, 12);
    recover(&w, w.arshad, "12-10-2026", "4,000");
    let page = r38(&w, DeadFilter::Owed, "");
    let row = &page.rows[0];
    assert_eq!(
        (
            row.owed_then_paisa.rupees(),
            row.recovered_paisa.rupees(),
            row.corrections_paisa,
            row.still_owed_paisa.rupees(),
            row.last_recovery_date.as_deref()
        ),
        (18_684, 4_000, 0, 14_684, Some("2026-10-12"))
    );
    // The totals are over the whole filter and the tile equals the R38 total (FR-19.9).
    assert_eq!(
        (
            page.totals.owed_then_paisa.rupees(),
            page.totals.recovered_paisa.rupees(),
            page.totals.still_owed_paisa.rupees()
        ),
        (18_684, 4_000, 14_684)
    );
    let t = tiles(&w);
    assert_eq!(t.dead_amount_paisa.rupees(), 14_684);
    assert_eq!(t.dead_count, 1);
    // کل اگراہی = R20 + the "بیوپاری کے ذمے" total of R24 + R38 (FR-19.9).
    let (_, buyers) = r20(&w);
    assert_eq!(
        t.total_dues_paisa.rupees(),
        buyers + t.sellers_dues_paisa.rupees() + page.totals.still_owed_paisa.rupees()
    );
    assert_eq!(t.total_dues_paisa.rupees(), 17_564);

    // The same rows in the report engine (print and Excel): the filter `due` is the owed ones.
    let engine = report(
        &w,
        ReportId::R38,
        ReportFilters {
            status: Some(PaidFilter::Due),
            ..ReportFilters::default()
        },
        0,
        100,
    );
    assert_eq!(engine.total_count, 1);
    assert_eq!(code_of(&engine.rows[0].cells[0]), code_of_hit(&w, w.arshad));
    assert_eq!(money(&engine.totals[5]), 18_684);
    assert_eq!(money(&engine.totals[6]), 4_000);
    assert_eq!(money(&engine.totals[8]), 14_684);
    // The catalogue lists R38 with the status filter.
    let entry = mandi_reports::reports::catalog()
        .into_iter()
        .find(|e| e.id == ReportId::R38)
        .unwrap();
    assert_eq!(entry.filters, vec![FilterKind::Status]);

    // Search keeps the people it names, in either script, and the totals follow.
    assert_eq!(r38(&w, DeadFilter::Owed, "arshad").row_count, 1);
    assert_eq!(r38(&w, DeadFilter::Owed, "ارشد").row_count, 1);
    let none = r38(&w, DeadFilter::Owed, "zzz");
    assert_eq!(none.row_count, 0);
    assert_eq!(none.totals.still_owed_paisa.rupees(), 0);

    // Undone: he leaves the owed list, stays in "all" with a status, and the tile is 0 again.
    dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap();
    assert_eq!(r38(&w, DeadFilter::Owed, "").row_count, 0);
    let all = r38(&w, DeadFilter::All, "");
    assert_eq!(all.row_count, 1);
    assert_eq!(all.rows[0].status, DeadStatus::Undone);
    assert_eq!(all.rows[0].still_owed_paisa.rupees(), 0);
    assert_eq!(all.totals.still_owed_paisa.rupees(), 0);
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 0);
    let (rows, total) = r20(&w);
    assert_eq!((rows.len(), total), (2, 17_564));
}

fn code_of_hit(w: &World, account: i64) -> String {
    w.env
        .ctx
        .db()
        .read(|c| mandi_services::db::accounts::get(c, account))
        .unwrap()
        .unwrap()
        .code
}

#[test]
fn r38_paging_has_totals_over_the_whole_filter_and_refuses_a_bad_page() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    mark(&w, w.arshad, "11-10-2026", DeadReason::Left);
    mark(&w, w.akram, "11-10-2026", DeadReason::Died);
    let page = mandi_reports::dead::page(
        w.env.ctx.db(),
        &DeadAmountInput {
            filter: DeadFilter::Owed,
            search: String::new(),
            offset: 1,
            limit: 1,
        },
    )
    .unwrap();
    assert_eq!((page.rows.len(), page.row_count), (1, 2));
    assert_eq!(page.totals.still_owed_paisa.rupees(), 21_564);
    for (offset, limit, field) in [(-1, 10, "offset"), (0, 0, "limit"), (0, 101, "limit")] {
        match mandi_reports::dead::page(
            w.env.ctx.db(),
            &DeadAmountInput {
                filter: DeadFilter::Owed,
                search: String::new(),
                offset,
                limit,
            },
        ) {
            Err(mandi_reports::ReportsError::Rejected(items)) => {
                assert_eq!(
                    (items[0].code, items[0].field.as_str()),
                    (ErrorCode::ErrOutOfRange, field)
                );
            }
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn a_cancelled_recovery_nets_out_and_a_cancelled_earlier_sale_is_a_correction() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    set_today(&w.env, 2026, 10, 12);
    let mut input = receipt(w.arshad, "4,000");
    input.date = "12-10-2026".into();
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();

    // Cancelling the recovery takes it off "recovered" (the lead, 30-10-2026, PRD FR-10.10): a
    // recovery of 4,000 that is cancelled shows recovered 0, no correction, and he owes 18,684
    // again.
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::vouchers::VoucherCancelInput {
            confirm_cash_below_zero: false,
            id: saved.voucher_id,
            version: vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().version,
            reason: "غلط".into(),
        },
    )
    .unwrap();
    let row = r38(&w, DeadFilter::Owed, "").rows.remove(0);
    assert_eq!(
        (
            row.recovered_paisa.rupees(),
            row.corrections_paisa,
            row.still_owed_paisa.rupees(),
            row.last_recovery_date
        ),
        (0, 0, 18_684, None)
    );
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 18_684);

    // A fresh recovery of 4,000, then his whole بکری 1 is cancelled (his line reverses with it;
    // the books stay balanced). He now has a credit of 4,000: still owed 0, status credit, and
    // any new recovery is refused (the limit is 0).
    recover(&w, w.arshad, "12-10-2026", "4,000");
    mandi_services::bikri::cancel(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::bikri::CancelInput {
            confirm_cash_below_zero: false,
            id: 1,
            reason: "غلط".into(),
            version: mandi_services::bikri::get(&w.env.ctx, 1).unwrap().version,
        },
    )
    .unwrap();
    let rows = r38(&w, DeadFilter::All, "").rows;
    let row = &rows[0];
    assert_eq!(
        (
            row.status,
            row.still_owed_paisa.rupees(),
            row.corrections_paisa / 100
        ),
        // recovered 4,000 (the cancelled first one nets to 0); the balance is 18,684 - 4,000 +
        // 4,000 - 4,000 - 18,684 = -4,000, so still owed 0 and the corrections are
        // 0 - 18,684 + 4,000.
        (DeadStatus::Credit, 0, -14_684)
    );
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 0);
    assert_eq!(tiles(&w).dead_count, 0);
    let refused = vouchers::save_receipt(&w.env.ctx, &w.owner, &{
        let mut i = receipt(w.arshad, "100");
        i.date = "12-10-2026".into();
        i
    })
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrReturnExceedsOpen);
    let trial = mandi_services::ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn r34_shows_the_four_dues_figures() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    set_today(&w.env, 2026, 10, 12);
    recover(&w, w.arshad, "12-10-2026", "4,000");
    let on = |day: &str| {
        report(
            &w,
            ReportId::R34,
            ReportFilters {
                from: day.into(),
                to: day.into(),
                ..ReportFilters::default()
            },
            0,
            200,
        )
    };
    let figure =
        |page: &mandi_reports::reports::ReportPage, label: &str| -> (Option<i64>, Option<i64>) {
            let row = page
                .rows
                .iter()
                .find(|r| matches!(&r.cells[1], Cell::Label { id } if id == label))
                .unwrap_or_else(|| panic!("no row {label}"));
            let count = match &row.cells[2] {
                Cell::Number { value } => Some(*value),
                _ => None,
            };
            let amount = match &row.cells[3] {
                Cell::Money { paisa } => Some(paisa.rupees()),
                _ => None,
            };
            (count, amount)
        };
    // 10-10-26 (before the move): buyers 21,564, dead 0. Earlier days have no sellers' dues or
    // total (the working is given for today or later only; nothing is guessed).
    let page = on("10-10-2026");
    assert_eq!(figure(&page, "report-r34-buyers-dues").1, Some(21_564));
    assert_eq!(figure(&page, "report-r34-dead-amount"), (Some(0), Some(0)));
    assert_eq!(figure(&page, "report-r34-sellers-dues").1, None);
    assert_eq!(figure(&page, "report-r34-total-dues").1, None);
    // 11-10-26 (he is moved): buyers 2,880, dead 18,684 for 1 person.
    let page = on("11-10-2026");
    assert_eq!(figure(&page, "report-r34-buyers-dues").1, Some(2_880));
    assert_eq!(
        figure(&page, "report-r34-dead-amount"),
        (Some(1), Some(18_684))
    );
    // 12-10-26 (today): all four, E2E-44: 2,880 + 0 + 14,684 = 17,564.
    let page = on("12-10-2026");
    assert_eq!(figure(&page, "report-r34-buyers-dues").1, Some(2_880));
    assert_eq!(figure(&page, "report-r34-sellers-dues").1, Some(0));
    assert_eq!(
        figure(&page, "report-r34-dead-amount"),
        (Some(1), Some(14_684))
    );
    assert_eq!(figure(&page, "report-r34-total-dues").1, Some(17_564));
}

#[test]
fn r35_leaves_a_dead_row_out_of_the_total_and_adds_a_second_total_row() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    set_today(&w.env, 2026, 10, 12);
    recover(&w, w.arshad, "12-10-2026", "4,000");
    let page = report(
        &w,
        ReportId::R35,
        ReportFilters {
            as_of: "12-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        100,
    );
    // Rows: the buyers' heading (اکرم only), then the rows, then the second total row.
    let groups: Vec<&mandi_reports::reports::Row> = page
        .rows
        .iter()
        .filter(|r| r.kind == RowKind::Group)
        .collect();
    let remaining = |row: &mandi_reports::reports::Row| balance(&row.cells[5]);
    // The first group is the buyers' heading: اکرم 2,880 (R20). The second total row adds ارشد's
    // 14,684: 17,564.
    assert_eq!(remaining(groups[0]), (BalanceSide::Dr, 2_880));
    let with_dead = groups
        .iter()
        .find(|g| matches!(&g.cells[0], Cell::Label { id } if id == "report-r35-total-with-dead"))
        .expect("a second total row");
    assert_eq!(remaining(with_dead), (BalanceSide::Dr, 17_564));
    // His row is marked, and the collection line is still R20's total.
    let marked: Vec<_> = page
        .rows
        .iter()
        .filter_map(|r| match &r.cells[1] {
            Cell::Party { party } if party.dead.is_some() => Some(party.account_id),
            _ => None,
        })
        .collect();
    assert_eq!(marked, vec![w.arshad]);
    // On a day he has no entry he is not on the sheet at all.
    set_today(&w.env, 2026, 10, 13);
    let quiet = report(
        &w,
        ReportId::R35,
        ReportFilters {
            as_of: "13-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        100,
    );
    assert!(
        !quiet
            .rows
            .iter()
            .any(|r| matches!(&r.cells[1], Cell::Party { party } if party.account_id == w.arshad))
    );
}

#[test]
fn the_party_card_the_ledger_heading_and_the_pickers_say_dead() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    let mark_id = mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    // The party card.
    let card = assist::party_card(&w.env.ctx, w.arshad).unwrap();
    let info = card.dead.expect("the card says dead");
    assert_eq!(
        (info.mark_id, info.entry_date.as_str(), info.reason),
        (mark_id, "2026-10-11", DeadReason::Died)
    );
    assert!(!card.is_active);
    assert!(
        assist::party_card(&w.env.ctx, w.akram)
            .unwrap()
            .dead
            .is_none()
    );
    // The ledger heading (R13).
    let page = report(&w, ReportId::R13, party(w.arshad), 0, 100);
    assert_eq!(
        page.filters
            .party
            .as_ref()
            .and_then(|p| p.dead.as_ref())
            .map(|d| d.mark_id),
        Some(mark_id)
    );
    // The pickers: left out of a normal buyer field, listed (marked) when the field asks for dead
    // people, as the receipt picker does.
    let find = |include_dead: Option<bool>| -> Vec<(i64, bool, bool)> {
        search::search_accounts(
            &w.env.ctx,
            &w.owner,
            &SearchAccountsInput {
                include_dead,
                scope: AccountScope::Buyer,
                control_id: None,
                text: "arshad".into(),
                limit: 10,
                offset: 0,
            },
        )
        .unwrap()
        .into_iter()
        .map(|h| (h.id, h.is_active, h.is_dead))
        .collect()
    };
    assert!(find(None).is_empty());
    assert!(find(Some(false)).is_empty());
    assert_eq!(find(Some(true)), vec![(w.arshad, false, true)]);
    // The whole list of the field (empty text) and the recent list do the same.
    let whole = |include_dead: Option<bool>| -> bool {
        search::recent_accounts(
            &w.env.ctx,
            &w.owner,
            &AccountsRecentInput {
                include_dead,
                scope: AccountScope::Buyer,
                control_id: None,
            },
        )
        .unwrap()
        .iter()
        .any(|h| h.id == w.arshad)
    };
    assert!(!whole(None));
    assert!(whole(Some(true)));
    // A typed full code still resolves him as inactive, with `is_dead` for the receipt picker.
    let code = code_of_hit(&w, w.arshad);
    let resolved = search::resolve_code(
        &w.env.ctx,
        &mandi_services::core::accounts::ResolveCodeInput {
            scope: AccountScope::Buyer,
            control_id: None,
            text: code,
        },
    )
    .unwrap();
    assert_eq!(resolved.account.map(|a| a.is_dead), Some(true));

    // He cannot be made active by the account form or by `set_active`; only the undo does it.
    let refused = accounts::set_active(&w.env.ctx, &w.owner, w.arshad, true).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrAlreadyDead);
    assert!(
        !accounts::get(&w.env.ctx, w.arshad).unwrap().is_active,
        "still inactive"
    );
    dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "x".into(),
        },
    )
    .unwrap();
    assert!(accounts::get(&w.env.ctx, w.arshad).unwrap().is_active);
    assert!(
        assist::party_card(&w.env.ctx, w.arshad)
            .unwrap()
            .dead
            .is_none()
    );
    assert_eq!(find(None).len(), 1);
}

#[test]
fn the_bell_leaves_a_dead_buyer_out_of_the_old_dues() {
    let w = full_day();
    set_today(&w.env, 2026, 11, 15);
    let old = |w: &World| {
        dashboard::attention(w.env.ctx.db(), off())
            .unwrap()
            .items
            .iter()
            .find(|i| i.kind == AttentionKind::DuesOld)
            .map(|i| i.count)
    };
    // 35 days later both buyers' oldest bills are older than 30 days.
    assert_eq!(old(&w), Some(2));
    mark(&w, w.arshad, "15-11-2026", DeadReason::Left);
    assert_eq!(old(&w), Some(1));
}

fn aslam(w: &World) -> i64 {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..5 {
        match lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: Some("0300-1234567".into()),
                staff_id: None,
                confirmed_rules_words: words.clone(),
            },
        ) {
            Ok(p) => return p.account.id,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(t)) =
                        item.values.get("words")
                    {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling was never accepted");
}

#[test]
fn a_dead_person_with_a_loan_leaves_the_lent_lists_and_can_still_return() {
    let w = full_day();
    let person = aslam(&w);
    vouchers::save_money_lent(
        &w.env.ctx,
        &w.owner,
        &MoneyLentInput {
            date: "10-10-2026".into(),
            person_account_id: Some(person),
            amount: "5,000".into(),
            reason: "گھریلو ضرورت".into(),
            return_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    assert_eq!(tiles(&w).lent_open_paisa.rupees(), 5_000);
    let total_before = tiles(&w).total_dues_paisa.rupees();
    let r32_rows = |w: &World| {
        report(w, ReportId::R32, ReportFilters::default(), 0, 100)
            .rows
            .iter()
            .filter(|r| r.kind == RowKind::Normal)
            .count()
    };
    assert_eq!(r32_rows(&w), 1);

    set_today(&w.env, 2026, 10, 11);
    let preview = dead::preview(
        &w.env.ctx,
        &mandi_services::core::dead::DeadPreviewInput {
            account_id: person,
            pukhta_person_key: None,
        },
    )
    .unwrap();
    assert_eq!(preview.owed_paisa.rupees(), 5_000);
    mark(&w, person, "11-10-2026", DeadReason::Left);
    // ادھار رقم باقی goes down by his loan and کل اگراہی goes up by it (A66).
    let t = tiles(&w);
    assert_eq!(t.lent_open_paisa.rupees(), 0);
    assert_eq!(t.dead_amount_paisa.rupees(), 5_000);
    assert_eq!(t.total_dues_paisa.rupees(), total_before + 5_000);
    assert_eq!(r32_rows(&w), 0);
    // A new loan to him is refused; a return is allowed up to what he owes.
    let lend = |amount: &str| {
        vouchers::save_money_lent(
            &w.env.ctx,
            &w.owner,
            &MoneyLentInput {
                date: "11-10-2026".into(),
                person_account_id: Some(person),
                amount: amount.into(),
                reason: "x".into(),
                return_by: None,
                narration: None,
                confirm_cash_below_zero: false,
            },
        )
    };
    assert_eq!(
        lend("100").unwrap_err().code(),
        ErrorCode::ErrAccountInactive
    );
    let give_back = |amount: &str| {
        vouchers::save_money_lent_return(
            &w.env.ctx,
            &w.owner,
            &MoneyLentReturnInput {
                date: "11-10-2026".into(),
                person_account_id: Some(person),
                amount: amount.into(),
                mode: mandi_services::core::lent::LentReturnMode::Cash,
                salary_head_id: None,
                narration: None,
            },
        )
    };
    assert_eq!(
        give_back("5,001").unwrap_err().code(),
        ErrorCode::ErrReturnExceedsOpen
    );
    give_back("2,000").unwrap();
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 3_000);
    let row = r38(&w, DeadFilter::Owed, "").rows.remove(0);
    assert_eq!(
        (
            row.party_type,
            row.recovered_paisa.rupees(),
            row.still_owed_paisa.rupees()
        ),
        (DeadPartyType::MoneyLent, 2_000, 3_000)
    );
}

#[test]
fn a_dead_seller_leaves_r24_and_r26_and_a_dead_one_time_person_is_not_made_regular() {
    use mandi_services::core::bikri::{LineChargeMethod, PaymentType};
    // E2E-28: the advance tick is off; رحیم is paid 19,316, so his ledger is 15,000 بنام and his
    // advance of 15,000 is open. Before the move he is a row of R26.
    let w = worked_day();
    let mut draft = support::event1(&w);
    draft.lots[0].adjust_advance = false;
    save(&w, &draft);
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "19,316")).unwrap();
    let rows = |w: &World, id: ReportId| {
        report(w, id, ReportFilters::default(), 0, 100)
            .rows
            .iter()
            .filter(|r| r.kind == RowKind::Normal)
            .count()
    };
    assert_eq!(rows(&w, ReportId::R26), 1);
    assert_eq!(rows(&w, ReportId::R24), 1);
    set_today(&w.env, 2026, 10, 11);
    mark(&w, w.rahim, "11-10-2026", DeadReason::Left);
    // He is in neither the advances nor the payables, and the open advance tile is 0; his whole
    // balance is the dead amount (A66).
    assert_eq!(rows(&w, ReportId::R26), 0);
    assert_eq!(rows(&w, ReportId::R24), 0);
    let t = tiles(&w);
    assert_eq!(
        (t.advances_open_paisa.rupees(), t.dead_amount_paisa.rupees()),
        (0, 15_000)
    );
    // A report as of a day before the move still has him (PRD FR-10.10: only those moved by that
    // day are left out).
    let before = report(
        &w,
        ReportId::R26,
        ReportFilters {
            as_of: "10-10-2026".into(),
            ..ReportFilters::default()
        },
        0,
        100,
    );
    assert_eq!(before.rows.len(), 1);

    // A one-time buyer who owes is moved by his key and is not made regular (FR-13.8).
    let w = worked_day();
    let mut line = support::sale("s-1", w.pukhta_buyer, "2", "1000");
    line.pukhta = Some(mandi_services::core::bikri::PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    line.payment_type = Some(PaymentType::Credit);
    line.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
    line.seller_comm = support::typed(LineChargeMethod::Percent, "0");
    line.laga = support::typed(LineChargeMethod::PerUnit, "0");
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "2");
    lot.sales = vec![line];
    save(&w, &support::draft(vec![lot]));
    let key = mandi_services::core::text::person_key("شاہد", "0300-0000000");
    set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.pukhta_buyer,
            pukhta_person_key: Some(key.clone()),
            entry_date: "11-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();
    let request = serde_json::json!({
        "person_key": key,
        "side": "buyer",
        "account_input": {
            "id": null, "version": null, "account_type": "buyer", "control_id": null,
            "code": "", "name_ur": "شاہد", "name_en": "Shahid", "confirmed_rules_words": [],
            "father_or_firm_ur": "", "father_or_firm_en": "", "city_id": null, "adda_id": null,
            "phone1": "03000000000", "phone2": "", "address": "", "linked_account_id": null,
            "opening_date": "", "opening_dr": "", "opening_cr": "", "opening_is_advance": false,
            "credit_allowed": true, "credit_limit": "", "default_marka_id": null,
            "collector_staff_id": null, "collection_weekday": null, "is_active": true, "note": ""
        }
    });
    let input: mandi_services::core::vouchers::MakeRegularInput =
        serde_json::from_value(request).unwrap();
    let refused = vouchers::make_regular(&w.env.ctx, &w.owner, &input).unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrPersonInDeadAmount);
    // The party card of the one-time account is not a person card: no dead mark on it.
    assert!(
        assist::party_card(&w.env.ctx, w.pukhta_buyer)
            .unwrap()
            .dead
            .is_none()
    );
    assert_eq!(tiles(&w).dead_count, 1);
    let _ = BalanceSide::Dr;
}

#[test]
fn an_undone_move_is_frozen_when_he_is_active_again() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    let mark_id = mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    set_today(&w.env, 2026, 10, 12);
    recover(&w, w.arshad, "12-10-2026", "4,000");
    dead::undo(
        &w.env.ctx,
        &w.owner,
        &DeadUndoInput {
            mark_id,
            reason: "غلطی".into(),
        },
    )
    .unwrap();
    let frozen = |w: &World| {
        let row = r38(w, DeadFilter::All, "").rows.remove(0);
        (
            row.status,
            row.recovered_paisa.rupees(),
            row.corrections_paisa,
            row.still_owed_paisa.rupees(),
        )
    };
    assert_eq!(frozen(&w), (DeadStatus::Undone, 4_000, 0, 0));
    // He is an ordinary buyer again: a receipt of 1,000 and a new bikri line are ordinary business
    // and do not move the figures of the old move.
    recover(&w, w.arshad, "12-10-2026", "1,000");
    assert_eq!(frozen(&w), (DeadStatus::Undone, 4_000, 0, 0));
    assert_eq!(
        r38(&w, DeadFilter::All, "").totals.recovered_paisa.rupees(),
        4_000
    );
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 0);
}

#[test]
fn a_recovery_before_the_move_on_the_same_day_is_not_recovered_one_after_it_is() {
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    // A receipt of 100 on 11-10, posted before the move: it is in what he owed then.
    recover(&w, w.arshad, "11-10-2026", "100");
    mark(&w, w.arshad, "11-10-2026", DeadReason::Left);
    // A receipt of 100 on the same day, posted after the move: a recovery.
    recover(&w, w.arshad, "11-10-2026", "100");
    let row = r38(&w, DeadFilter::Owed, "").rows.remove(0);
    assert_eq!(
        (
            row.owed_then_paisa.rupees(),
            row.recovered_paisa.rupees(),
            row.corrections_paisa,
            row.still_owed_paisa.rupees()
        ),
        (18_584, 100, 0, 18_484)
    );
}

#[test]
fn an_edit_of_a_recovery_is_checked_against_what_he_owes_with_its_own_reversal_taken_off() {
    use mandi_services::core::vouchers::{VoucherEditInput, VoucherEntry};
    let w = full_day();
    set_today(&w.env, 2026, 10, 11);
    mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    set_today(&w.env, 2026, 10, 12);
    let mut input = receipt(w.arshad, "4,000");
    input.date = "12-10-2026".into();
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    let edit = |amount: &str| {
        let mut entry = receipt(w.arshad, amount);
        entry.date = "12-10-2026".into();
        let version = vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().version;
        vouchers::edit(
            &w.env.ctx,
            &w.owner,
            &VoucherEditInput {
                id: saved.voucher_id,
                version,
                entry: VoucherEntry::Receipt(entry),
                reason: "درستی".into(),
                confirm_cash_below_zero: false,
            },
        )
    };
    // 4,000 -> 18,685 is more than the 18,684 he owed before the receipt: refused, nothing changes.
    assert_eq!(
        edit("18,685").unwrap_err().code(),
        ErrorCode::ErrReturnExceedsOpen
    );
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 14_684);
    // 4,000 -> 18,684 is exactly all of it: allowed.
    edit("18,684").unwrap();
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 0);
    let trial = mandi_services::ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn an_edit_cannot_add_a_dead_one_time_person_who_was_not_in_the_posting() {
    use mandi_services::core::bikri::{
        BikriSaveInput, EditOpenInput, EditSaveInput, LineChargeMethod, PaymentType, PukhtaInput,
    };
    let w = worked_day();
    let line = |key: &str, qty: &str, name_ur: &str, name_en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "1000");
        l.pukhta = Some(PukhtaInput {
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    // بکری 1 has شاہد; بکری 2 has ناصر, who then dies.
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![line("s-1", "1", "شاہد", "Shahid", "0300-0000000")];
    let first = mandi_services::bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: support::draft(vec![lot]),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    save(&w, &{
        let mut lot = support::bare_lot(&w, "lot-2", w.rahim, w.banana, "2");
        lot.sales = vec![line("s-2", "2", "ناصر", "Nasir", "0300-1111111")];
        support::draft(vec![lot])
    });
    let nasir = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.pukhta_buyer,
            pukhta_person_key: Some(nasir),
            entry_date: "11-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();

    // Editing بکری 1 to add a NEW line for ناصر is refused: he was not in that posting.
    let opened = mandi_services::bikri::open_for_edit(
        &w.env.ctx,
        &EditOpenInput {
            bikri_id: first.bikri_id,
            entry_id: first.entry_id,
        },
    )
    .unwrap();
    let mut draft = opened.draft.clone();
    draft.lots[0]
        .sales
        .push(line("s-3", "2", "ناصر", "Nasir", "0300-1111111"));
    let refused = mandi_services::bikri::edit_save(
        &w.env.ctx,
        &w.owner,
        &EditSaveInput {
            entry_id: opened.entry_id,
            version: opened.version,
            draft,
            reason: Some("x".into()),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap_err();
    assert_eq!(refused.code(), ErrorCode::ErrPersonInDeadAmount);
    // The books are as they were: ناصر still owes his 2,000 and the dead amount is that.
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 2_000);
    let trial = mandi_services::ledger::trial_balance(&w.env.ctx, None).unwrap();
    assert_eq!(trial.total_dr, trial.total_cr);
}

#[test]
fn an_edited_recovery_counts_its_new_amount_and_one_saved_before_the_move_is_a_correction() {
    use mandi_services::core::vouchers::{VoucherCancelInput, VoucherEditInput, VoucherEntry};
    let w = full_day();
    // A receipt of 500 on 11-10, saved BEFORE the move: it is in what he owed then (18,184).
    set_today(&w.env, 2026, 10, 11);
    let mut before = receipt(w.arshad, "500");
    before.date = "11-10-2026".into();
    let early = vouchers::save_receipt(&w.env.ctx, &w.owner, &before).unwrap();
    mark(&w, w.arshad, "11-10-2026", DeadReason::Died);
    // A recovery after the move, edited from 4,000 to 3,000: recovered 3,000, no correction.
    set_today(&w.env, 2026, 10, 12);
    let mut input = receipt(w.arshad, "4,000");
    input.date = "12-10-2026".into();
    let saved = vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    let mut smaller = receipt(w.arshad, "3,000");
    smaller.date = "12-10-2026".into();
    vouchers::edit(
        &w.env.ctx,
        &w.owner,
        &VoucherEditInput {
            id: saved.voucher_id,
            version: vouchers::get(&w.env.ctx, saved.voucher_id).unwrap().version,
            entry: VoucherEntry::Receipt(smaller),
            reason: "درستی".into(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    let row = r38(&w, DeadFilter::Owed, "").rows.remove(0);
    assert_eq!(
        (
            row.owed_then_paisa.rupees(),
            row.recovered_paisa.rupees(),
            row.corrections_paisa,
            row.still_owed_paisa.rupees(),
            row.last_recovery_date.as_deref()
        ),
        (18_184, 3_000, 0, 15_184, Some("2026-10-12"))
    );
    // Cancelling the receipt saved BEFORE the move is a correction (+500), not a negative
    // recovery: recovered stays 3,000 and he owes 500 more.
    vouchers::cancel(
        &w.env.ctx,
        &w.owner,
        &VoucherCancelInput {
            confirm_cash_below_zero: false,
            id: early.voucher_id,
            version: vouchers::get(&w.env.ctx, early.voucher_id).unwrap().version,
            reason: "غلط".into(),
        },
    )
    .unwrap();
    let row = r38(&w, DeadFilter::Owed, "").rows.remove(0);
    assert_eq!(
        (
            row.recovered_paisa.rupees(),
            row.corrections_paisa / 100,
            row.still_owed_paisa.rupees()
        ),
        (3_000, 500, 15_684)
    );
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 15_684);
}

#[test]
fn a_dead_one_time_person_is_found_only_when_dead_people_are_asked_for_and_can_be_recovered() {
    use mandi_services::core::bikri::{
        LineChargeMethod, PaymentType, PukhtaInput, PukhtaPeopleInput, PukhtaSide,
    };
    let w = worked_day();
    let line = |key: &str, qty: &str, name_ur: &str, name_en: &str, phone: &str| {
        let mut l = support::sale(key, w.pukhta_buyer, qty, "1000");
        l.pukhta = Some(PukhtaInput {
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            phone: phone.into(),
        });
        l.payment_type = Some(PaymentType::Credit);
        l.buyer_comm = support::typed(LineChargeMethod::Percent, "0");
        l.seller_comm = support::typed(LineChargeMethod::Percent, "0");
        l.laga = support::typed(LineChargeMethod::PerUnit, "0");
        l
    };
    let mut lot = support::bare_lot(&w, "lot-1", w.rahim, w.banana, "3");
    lot.sales = vec![
        line("s-1", "1", "شاہد", "Shahid", "0300-0000000"),
        line("s-2", "2", "ناصر", "Nasir", "0300-1111111"),
    ];
    save(&w, &support::draft(vec![lot]));
    let nasir = mandi_services::core::text::person_key("ناصر", "0300-1111111");
    set_today(&w.env, 2026, 10, 11);
    dead::mark(
        &w.env.ctx,
        &w.owner,
        &DeadMarkInput {
            account_id: w.pukhta_buyer,
            pukhta_person_key: Some(nasir.clone()),
            entry_date: "11-10-2026".into(),
            reason: DeadReason::Died,
            note: String::new(),
        },
    )
    .unwrap();

    let find = |text: &str, include_dead: Option<bool>| -> Vec<(String, bool)> {
        mandi_services::bikri::search_people(
            &w.env.ctx,
            &PukhtaPeopleInput {
                text: text.into(),
                side: PukhtaSide::Buyer,
                include_dead,
            },
        )
        .unwrap()
        .into_iter()
        .map(|p| (p.name_en, p.is_dead))
        .collect()
    };
    // Hidden unless the field asks for dead people.
    assert!(find("nasir", None).is_empty());
    assert!(find("nasir", Some(false)).is_empty());
    assert_eq!(find("nasir", Some(true)), vec![("Nasir".to_owned(), true)]);
    // Shahid is found either way, and is not dead; the whole list marks only Nasir.
    assert_eq!(find("shahid", None), vec![("Shahid".to_owned(), false)]);
    assert_eq!(
        find("shahid", Some(true)),
        vec![("Shahid".to_owned(), false)]
    );
    let mut all = find("", Some(true));
    all.sort();
    assert_eq!(
        all,
        vec![("Nasir".to_owned(), true), ("Shahid".to_owned(), false)]
    );
    assert_eq!(find("", None), vec![("Shahid".to_owned(), false)]);

    // A receipt on him (the recovery) is allowed within what he owes (2,000) and refused above.
    let mut input = receipt(w.pukhta_buyer, "2,001");
    input.date = "11-10-2026".into();
    input.person_key = Some(nasir.clone());
    assert_eq!(
        vouchers::save_receipt(&w.env.ctx, &w.owner, &input)
            .unwrap_err()
            .code(),
        ErrorCode::ErrReturnExceedsOpen
    );
    input.amount = "500".into();
    vouchers::save_receipt(&w.env.ctx, &w.owner, &input).unwrap();
    assert_eq!(tiles(&w).dead_amount_paisa.rupees(), 1_500);
    // The seller side has its own account: nobody is dead there.
    assert!(
        mandi_services::bikri::search_people(
            &w.env.ctx,
            &PukhtaPeopleInput {
                text: String::new(),
                side: PukhtaSide::Seller,
                include_dead: Some(true),
            },
        )
        .unwrap()
        .iter()
        .all(|p| !p.is_dead)
    );
}
