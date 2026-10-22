//! The worked day of the PRD (Section 4, Events 1 to 5), built through the real services the way
//! `golden_day.rs` builds it, and the small helpers the report tests share (plan 20). Read the
//! numbers in the PRD, not here: this file only makes the data.
#![allow(dead_code)]

use crate::common::*;
use crate::support;
use mandi_reports::reports::{
    Cell, PartyFilter, ReportFilters, ReportId, ReportPage, ReportRunInput, run,
};
use mandi_services::core::accounts::{BalanceSide, OpeningLineInput, OpeningsInput};
use mandi_services::core::bikri::{BikriDraft, BikriSaveInput, HeaderInput, PukhtaInput};
use mandi_services::core::error::ErrorCode;
use mandi_services::core::money::Paisa;
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, PaymentInput, ReceiptInput, VoucherMode,
};
use mandi_services::{accounts, bikri, day, ledger, vouchers};

pub fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

/// The opening position, the days up to 09-10-26 closed, and today 10-10-26 (as `golden_day`).
pub fn worked_day() -> support::World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = mandi_services::auth::require(&env.ctx).unwrap();
    let rahim = support::add_account(
        &env,
        &owner,
        mandi_services::core::accounts::AccountType::Seller,
        "رحیم",
        "Rahim",
        true,
        "",
    );
    let cash = support::id_of(&env, "01010001");
    let capital = support::id_of(&env, "06010001");
    set_today(&env, 2026, 8, 31);
    let line = |id: i64, dr: &str, cr: &str| OpeningLineInput {
        account_id: id,
        version: accounts::get(&env.ctx, id).unwrap().version,
        opening_date: "31-08-2026".into(),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: false,
    };
    accounts::openings_save(
        &env.ctx,
        &owner,
        &OpeningsInput {
            lines: vec![line(cash, "65,000", ""), line(capital, "", "65,000")],
        },
    )
    .unwrap();
    set_today(&env, 2026, 9, 1);
    vouchers::save_advance(
        &env.ctx,
        &owner,
        &AdvanceInput {
            confirm_cash_below_zero: false,
            date: "01-09-2026".into(),
            seller_account_id: rahim,
            amount: "15,000".into(),
            mode: VoucherMode::Cash,
            bank_account_id: None,
            cheque_no: String::new(),
            cheque_date: String::new(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .unwrap();
    set_today(&env, 2026, 10, 10);
    day::close(
        &env.ctx,
        &owner,
        &DayCloseInput {
            up_to: "09-10-2026".into(),
        },
    )
    .unwrap();
    support::finish_world(dir, env, owner, rahim, true)
}

pub fn save(w: &support::World, draft: &BikriDraft) -> mandi_services::core::bikri::BikriSaved {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap()
}

pub fn event2(w: &support::World) -> BikriDraft {
    let mut line = support::sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = support::lot(w, "lot-2", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    lot.sales = vec![line];
    BikriDraft {
        header: HeaderInput {
            truck_no: Some("LHR-0077".into()),
            ..support::header()
        },
        lots: vec![lot],
    }
}

pub fn receipt(account: i64, amount: &str) -> ReceiptInput {
    ReceiptInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: amount.into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        collector_staff_id: None,
        narration: String::new(),
    }
}

pub fn payment(account: i64, amount: &str) -> PaymentInput {
    PaymentInput {
        date: "10-10-2026".into(),
        account_id: account,
        person_key: None,
        amount: amount.into(),
        mode: VoucherMode::Cash,
        bank_account_id: None,
        cheque_no: String::new(),
        cheque_date: String::new(),
        reason: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    }
}

/// Event 5 of PRD 4.8 as plain journal postings, as `golden_day` does it (the خرچہ screen is
/// plan 17): electricity 3,000 as دکان خرچہ and الف's 2,000 as ذاتی خرچہ.
pub fn event5(w: &support::World) {
    use mandi_services::core::ids::AccountId;
    use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
    use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
    use mandi_services::ledger::PostingEvent;
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = ExpenseHeadInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: "بجلی".into(),
            name_en: "Electricity".into(),
            confirmed_rules_words: words.clone(),
        };
        match mandi_services::masters::save(&w.env.ctx, &w.owner, &MasterInput::ExpenseHead(input))
        {
            Ok(_) => break,
            Err(mandi_services::ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(mandi_services::core::error::ErrorValue::Text(text)) =
                        item.values.get("words")
                    {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    let expense = support::id_of(&w.env, "05010001");
    let drawings = support::id_of(&w.env, "06030001");
    let cash = support::id_of(&w.env, "01010001");
    let date = mandi_services::core::dates::BusinessDate::from_ymd(2026, 10, 10).unwrap();
    let post = |posting_type: PostingType, debit: i64, amount: i64, source_id: i64| {
        let entry = build_entry(vec![
            LedgerLine::new(AccountId::new(debit), Side::Dr, rupees(amount)),
            LedgerLine::new(AccountId::new(cash), Side::Cr, rupees(amount)),
        ])
        .unwrap();
        w.env
            .ctx
            .db()
            .write(|tx| {
                ledger::post(
                    &w.env.ctx,
                    tx,
                    w.owner.user_id,
                    &PostingEvent {
                        date,
                        posting_type,
                        source_type: SourceType::Voucher,
                        source_id,
                        narration: None,
                        reverses: None,
                        entry,
                    },
                )
            })
            .unwrap();
    };
    post(PostingType::ShopExpense, expense, 3_000, 9_001);
    post(PostingType::PersonalExpense, drawings, 2_000, 9_002);
}

/// Events 1 to 5 of the worked day.
pub fn full_day() -> support::World {
    let w = worked_day();
    save(&w, &support::event1(&w));
    save(&w, &event2(&w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "4,316")).unwrap();
    event5(&w);
    w
}

pub fn report(
    w: &support::World,
    id: ReportId,
    filters: ReportFilters,
    offset: i64,
    limit: i64,
) -> ReportPage {
    run(
        w.env.ctx.db(),
        &ReportRunInput {
            id,
            filters,
            offset,
            limit,
        },
    )
    .unwrap()
}

pub fn rejected(
    w: &support::World,
    id: ReportId,
    filters: ReportFilters,
    offset: i64,
    limit: i64,
) -> Vec<(ErrorCode, String)> {
    match run(
        w.env.ctx.db(),
        &ReportRunInput {
            id,
            filters,
            offset,
            limit,
        },
    ) {
        Err(mandi_reports::ReportsError::Rejected(items)) => {
            items.into_iter().map(|e| (e.code, e.field)).collect()
        }
        other => panic!("expected a rejection, got {other:?}"),
    }
}

pub fn party(account_id: i64) -> ReportFilters {
    ReportFilters {
        party: Some(PartyFilter {
            account_id,
            person_key: None,
        }),
        ..ReportFilters::default()
    }
}

/// Whole rupees of a money cell, or 0 for an empty one.
pub fn money(cell: &Cell) -> i64 {
    match cell {
        Cell::Money { paisa } => paisa.rupees(),
        Cell::Empty => 0,
        other => panic!("not money: {other:?}"),
    }
}

pub fn balance(cell: &Cell) -> (BalanceSide, i64) {
    match cell {
        Cell::Balance { paisa, side } => (*side, paisa.rupees()),
        other => panic!("not a balance: {other:?}"),
    }
}

pub fn code_of(cell: &Cell) -> String {
    match cell {
        Cell::Party { party } => party.code.clone(),
        other => panic!("not a party: {other:?}"),
    }
}
