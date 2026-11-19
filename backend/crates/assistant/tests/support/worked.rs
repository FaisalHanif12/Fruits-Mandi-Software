//! The PRD worked day for the assistant tests (PRD Section 4 and 27): the opening position, the
//! parties, items and partners, and Events 1 to 5. A copy of what `services/tests/golden_day.rs`
//! does, so the assistant tests do not depend on a test binary of another crate.
#![allow(dead_code)]

use mandi_services::core::accounts::{AccountInput, AccountType, OpeningLineInput, OpeningsInput};
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, ChuktaInput, HeaderInput, SaleInput,
};
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

/// Event 2 of PRD 4.5: کریم's 10 سیب sold to the چکتا buyer شاہد for cash.
pub fn event2(w: &world::World) -> BikriDraft {
    let mut line = world::sale("s-1", w.chukta_khata, "10", "1200");
    line.chukta = Some(ChuktaInput {
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

/// The books of the client's demo (D8 Feature 4 and 10), today 08-11-2026. Every figure below is worked by
/// hand from these lines: value = qty x rate, buyer commission 6% of it, لاگا 8 a نگ.
///
/// | Date | Seller, item | Buyer, qty at rate | Buyer total | Paid | Left |
/// |---|---|---|---|---|---|
/// | 20-11-25 | Rahim, banana | Haroon 10 at 500 | 5,380 | 0 | 5,380 |
/// | 15-12-25 | Rahim, apple | Haroon 20 at 800 | 17,120 | 6,000 | 11,120 |
/// | 15-03-26 | Rahim, banana | Haroon 30 at 600 | 19,320 | 19,320 | 0 |
/// | 12-09-26 | Rahim, apple | Anas 5 at 1,000 | 5,340 | 0 | 5,340 |
/// | 01-11-26 | Rahim, banana | Akram 10 at 500 | 5,380 | 5,380 | 0 |
/// | 03-11-26 | Ehsan, mango 200 | Mani 20 at 1,000 | 21,360 | 21,360 | 0 |
/// |  |  | Anas 50 at 900 | 48,100 | 10,000 | 38,100 |
/// |  |  | Haroon 130 at 900 | 125,060 | 0 | 125,060 |
/// | 04-11-26 | Rahim, banana | Anas 10 at 500 | 5,380 | 5,380 | 0 |
/// | 05-11-26 | Rahim, apple | Akram 10 at 1,000 | 10,680 | 3,000 | 7,680 |
/// | 06-11-26 | Rahim, banana | Akram 5 at 500 | 2,690 | 0 | 2,690 |
/// | 08-11-26 | Karim, apple | Shahid (walk-in) 10 at 1,200 | 12,800 | 12,800 | 0 |
pub struct D8 {
    pub w: world::World,
    pub ehsan: i64,
    pub anas: i64,
    pub haroon: i64,
    pub mango: i64,
}

fn paid(mut s: SaleInput, amount: &str) -> SaleInput {
    s.paid = Some(amount.into());
    s
}

pub fn d8() -> D8 {
    let (dir, env) = crate::common::env();
    crate::common::set_up(&env);
    crate::common::login_owner(&env);
    let owner = mandi_services::auth::require(&env.ctx).unwrap();
    let rahim = world::add_account(&env, &owner, AccountType::Seller, "رحیم", "Rahim", true, "");
    crate::common::set_today(&env, 2025, 11, 1);
    let w = world::finish_world(dir, env, owner, rahim, false);
    mandi_services::partners::set_shares(
        &w.env.ctx,
        &w.owner,
        &mandi_services::core::partners::PartnersSetSharesInput {
            effective_from: "01-11-2025".into(),
            shares: [
                (w.partners[0], "40"),
                (w.partners[1], "30"),
                (w.partners[2], "30"),
            ]
            .iter()
            .map(
                |&(partner_id, share)| mandi_services::core::partners::ShareInput {
                    partner_id,
                    share: share.to_owned(),
                },
            )
            .collect(),
        },
    )
    .unwrap();
    let ehsan = world::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "احسان",
        "Ehsan",
        true,
        "",
    );
    let buyer = |ur: &str, en: &str| {
        world::add_account(&w.env, &w.owner, AccountType::Buyer, ur, en, true, "")
    };
    let (mani, anas, haroon) = (
        buyer("مانی", "Mani"),
        buyer("انس", "Anas"),
        buyer("ہارون", "Haroon"),
    );
    let mango = world::add_item(&w.env, &w.owner, "آم", "Mango", "fruit", "unit");
    let d = D8 {
        w,
        ehsan,
        anas,
        haroon,
        mango,
    };
    let save = |date: (i32, u32, u32), seller: i64, item: i64, qty: &str, lines: Vec<SaleInput>| {
        crate::common::set_today(&d.w.env, date.0, date.1, date.2);
        let mut lot = world::bare_lot(&d.w, "lot-1", seller, item, qty);
        lot.sales = lines;
        mandi_services::bikri::save(
            &d.w.env.ctx,
            &d.w.owner,
            &BikriSaveInput {
                draft: BikriDraft {
                    header: HeaderInput {
                        entry_date: format!("{:02}-{:02}-{}", date.2, date.1, date.0),
                        ..world::header()
                    },
                    lots: vec![lot],
                },
                confirm_cash_below_zero: true,
            },
        )
        .unwrap();
    };
    let (rahim, banana, apple) = (d.w.rahim, d.w.banana, d.w.apple);
    save(
        (2025, 11, 20),
        rahim,
        banana,
        "10",
        vec![world::sale("s", haroon, "10", "500")],
    );
    save(
        (2025, 12, 15),
        rahim,
        apple,
        "20",
        vec![paid(world::sale("s", haroon, "20", "800"), "6,000")],
    );
    save(
        (2026, 3, 15),
        rahim,
        banana,
        "30",
        vec![paid(world::sale("s", haroon, "30", "600"), "19,320")],
    );
    save(
        (2026, 9, 12),
        rahim,
        apple,
        "5",
        vec![world::sale("s", anas, "5", "1,000")],
    );
    save(
        (2026, 11, 1),
        rahim,
        banana,
        "10",
        vec![paid(world::sale("s", d.w.akram, "10", "500"), "5,380")],
    );
    save(
        (2026, 11, 3),
        ehsan,
        mango,
        "200",
        vec![
            paid(world::sale("l1", mani, "20", "1,000"), "21,360"),
            paid(world::sale("l2", anas, "50", "900"), "10,000"),
            world::sale("l3", haroon, "130", "900"),
        ],
    );
    save(
        (2026, 11, 4),
        rahim,
        banana,
        "10",
        vec![paid(world::sale("s", anas, "10", "500"), "5,380")],
    );
    save(
        (2026, 11, 5),
        rahim,
        apple,
        "10",
        vec![paid(world::sale("s", d.w.akram, "10", "1,000"), "3,000")],
    );
    save(
        (2026, 11, 6),
        rahim,
        banana,
        "5",
        vec![world::sale("s", d.w.akram, "5", "500")],
    );
    // Today: a walk-in buyer pays the whole bill at once (PRD Section 13): Shahid, 10 at 1,200 = 12,000,
    // commission 720 + 720 and لاگا 80, 12,800 in all, paid 12,800.
    let mut shahid = world::sale("s", d.w.chukta_khata, "10", "1,200");
    shahid.chukta = Some(ChuktaInput {
        name_ur: "شاہد".into(),
        name_en: "Shahid".into(),
        phone: String::new(),
    });
    let karim = d.w.karim;
    save((2026, 11, 8), karim, apple, "10", vec![shahid]);
    crate::common::set_today(&d.w.env, 2026, 11, 8);
    d
}
