//! The PRD worked day for the assistant tests (PRD Section 4 and 27): the opening position, the
//! parties, items and partners, and Events 1 to 5. A copy of what `services/tests/golden_day.rs`
//! does, so the assistant tests do not depend on a test binary of another crate.
#![allow(dead_code)]

use mandi_services::core::accounts::{AccountInput, AccountType, OpeningLineInput, OpeningsInput};
use mandi_services::core::bikri::{BikriDraft, BikriSaveInput, HeaderInput, PukhtaInput};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{PersonalExpenseInput, ShopExpenseInput};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, PaymentInput, ReceiptInput, RemovedPaymentFields,
};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, auth, bikri, day, masters, partners, vouchers};
use tempfile::TempDir;

use crate::common::{env, login_owner, set_today, set_up};
use crate::world;

/// Details of رحیم that are in the books and must never reach the model (PRD FR-7.83, E2E-54).
pub const PRIVATE_PHONE: &str = "0300-1111111";
pub const PRIVATE_ADDRESS: &str = "گلی نمبر 5 ملتان";
pub const PRIVATE_NOTE: &str = "خاص نوٹ رحیم";

fn account_id(env: &TestEnv, code: &str) -> i64 {
    world::id_of(env, code)
}

fn seller_rahim(env: &TestEnv, owner: &auth::Actor) -> i64 {
    let input = AccountInput {
        id: None,
        version: None,
        account_type: AccountType::Seller,
        is_pukhta_party: false,
        control_id: None,
        code: String::new(),
        name_ur: "رحیم".into(),
        name_en: "Rahim".into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: "حاجی صاحب".into(),
        father_or_firm_en: "Haji Sahib".into(),
        city_id: None,
        adda_id: None,
        phone1: PRIVATE_PHONE.into(),
        phone2: String::new(),
        address: PRIVATE_ADDRESS.into(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: PRIVATE_NOTE.into(),
        tag: String::new(),
    };
    accounts::save(&env.ctx, owner, &input).unwrap().id
}

/// The worked day after PRD 4.3: openings, the advance to رحیم, the days up to 09-10-26 closed,
/// today 10-10-26, and the parties, items and partners 40 / 30 / 30 on top.
pub fn worked_day() -> world::World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = seller_rahim(&env, &owner);
    let cash = account_id(&env, "01010001");
    let capital = account_id(&env, "06010001");
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
            removed: RemovedPaymentFields::default(),
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
    world::finish_world(dir, env, owner, rahim, true)
}

/// Event 2 of PRD 4.5: کریم's 10 سیب sold to the one-time buyer شاہد for cash.
pub fn event2(w: &world::World) -> BikriDraft {
    let mut line = world::sale("s-1", w.pukhta_buyer, "10", "1200");
    line.pukhta = Some(PukhtaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: "0300-0000000".into(),
    });
    let mut lot = world::lot(w, "lot-2", w.karim, w.apple, "10");
    lot.freight = Some("1,000".into());
    lot.cash_paid = None;
    lot.sales = vec![line];
    BikriDraft {
        header: HeaderInput {
            truck_no: Some("LHR-0077".into()),
            ..world::header()
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
        removed: RemovedPaymentFields::default(),
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
        removed: RemovedPaymentFields::default(),
        reason: String::new(),
        narration: String::new(),
        confirm_cash_below_zero: false,
    }
}

fn save(w: &world::World, draft: &BikriDraft) {
    bikri::save(
        &w.env.ctx,
        &w.owner,
        &BikriSaveInput {
            draft: draft.clone(),
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// Events 1 to 4 of the PRD: بکری 1, بکری 2, the وصولی of 10,000 and the ادائیگی of 4,316.
pub fn events_1_to_4(w: &world::World) {
    save(w, &world::event1(w));
    save(w, &event2(w));
    vouchers::save_receipt(&w.env.ctx, &w.owner, &receipt(w.akram, "10,000")).unwrap();
    vouchers::save_payment(&w.env.ctx, &w.owner, &payment(w.rahim, "4,316")).unwrap();
}

/// Makes an expense head, confirming the spellings the engine is unsure of, and returns its id.
pub fn expense_head(w: &world::World, name_ur: &str, name_en: &str) -> i64 {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = ExpenseHeadInput {
            id: None,
            version: None,
            code: String::new(),
            name_ur: name_ur.into(),
            name_en: name_en.into(),
            confirmed_rules_words: words.clone(),
        };
        match masters::save(&w.env.ctx, &w.owner, &MasterInput::ExpenseHead(input)) {
            Ok(_) => break,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(text)) = item.values.get("words") {
                        words.extend(text.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    masters::expense_heads(&w.env.ctx)
        .unwrap()
        .rows
        .iter()
        .find(|r| r.head.name_en == name_en)
        .unwrap()
        .id
}

/// Event 5 of the PRD: electricity 3,000 as دکان خرچہ and الف's 2,000 as ذاتی خرچہ.
pub fn event5(w: &world::World) {
    let head = expense_head(w, "بجلی", "Electricity");
    vouchers::save_shop_expense(
        &w.env.ctx,
        &w.owner,
        &ShopExpenseInput {
            date: "10-10-2026".into(),
            expense_head_id: Some(head),
            amount: "3,000".into(),
            paid_to: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
    vouchers::save_personal_expense(
        &w.env.ctx,
        &w.owner,
        &PersonalExpenseInput {
            date: "10-10-2026".into(),
            partner_id: Some(w.partners[0]),
            amount: "2,000".into(),
            reason: "گھر کا خرچ".into(),
            return_by: None,
            taken_by: None,
            narration: None,
            confirm_cash_below_zero: false,
        },
    )
    .unwrap();
}

/// Events 1 to 5: the whole worked day.
pub fn events_1_to_5(w: &world::World) {
    events_1_to_4(w);
    event5(w);
}

/// A throwaway directory handle kept alive by the world.
pub fn keep(_dir: &TempDir) {}

/// A world with a month of sales for each `(month, banana?, qty, rate)` of 2026 (banana if true,
/// else apple), so that the commission of each month (seller 6% + buyer 6% + لاگا 8 per نگ on
/// fruit) is a known figure, and today 10-10-26 (PRD E2E-59). One sale of رحیم's goods to اکرم on
/// the 15th of the month, on credit, no charges.
pub fn months_world(months: &[(u32, bool, i64, i64)]) -> world::World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = seller_rahim(&env, &owner);
    set_today(&env, 2026, 4, 1);
    let w = world::finish_world(dir, env, owner, rahim, false);
    partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &PartnersSetSharesInput {
            effective_from: "01-04-2026".into(),
            shares: [
                (w.partners[0], "40"),
                (w.partners[1], "30"),
                (w.partners[2], "30"),
            ]
            .iter()
            .map(|&(partner_id, share)| ShareInput {
                partner_id,
                share: share.to_owned(),
            })
            .collect(),
        },
    )
    .unwrap();
    for &(month, banana, qty, rate) in months {
        set_today(&w.env, 2026, month, 15);
        let item = if banana { w.banana } else { w.apple };
        let mut lot = world::bare_lot(&w, "lot-1", w.rahim, item, &qty.to_string());
        lot.sales = vec![world::sale(
            "s-1",
            w.akram,
            &qty.to_string(),
            &rate.to_string(),
        )];
        let draft = BikriDraft {
            header: HeaderInput {
                entry_date: format!("15-{month:02}-2026"),
                ..world::header()
            },
            lots: vec![lot],
        };
        save(&w, &draft);
    }
    set_today(&w.env, 2026, 10, 10);
    w
}

/// The E2E-59 history: one banana sale a month, April to September (see [`months_world`]).
pub fn history_world(months: &[(u32, i64, i64)]) -> world::World {
    let entries: Vec<(u32, bool, i64, i64)> =
        months.iter().map(|&(m, q, r)| (m, true, q, r)).collect();
    months_world(&entries)
}

/// E2E-16: Event 1 with only 45 of the 50 sold (اکرم 20 at 600, ارشد 25 at 580), today 13-10-26.
pub fn part_sold_world() -> world::World {
    let w = worked_day();
    let mut draft = world::event1(&w);
    if let Some(lot) = draft.lots.first_mut()
        && let Some(second) = lot.sales.get_mut(1)
    {
        second.qty = "25".into();
        second.seller_rate = "580".into();
    }
    save(&w, &draft);
    set_today(&w.env, 2026, 10, 13);
    w
}
