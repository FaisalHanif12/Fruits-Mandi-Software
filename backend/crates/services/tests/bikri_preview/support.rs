//! Helpers for the plan 12 tests: the worked day's parties, items and partners in a real
//! database, and builders for the typed draft of the بکری screen.

use mandi_services::auth::{self, Actor};
use mandi_services::bikri;
use mandi_services::core::accounts::{AccountInput, AccountType, OpeningLineInput, OpeningsInput};
use mandi_services::core::bikri::{
    BikriDraft, BikriPreview, ChargeInput, FreightPaidBy, HeaderInput, LineCalc, LineChargeInput,
    LineChargeMethod, LotChargeMethod, LotInput, LotPreview, SaleInput,
};
use mandi_services::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_services::core::masters::{ItemInput, MasterInput, PackingInput};
use mandi_services::core::money::Paisa;
use mandi_services::core::partners::{PartnerCreateInput, PartnersSetSharesInput, ShareInput};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, accounts, masters, partners};
use tempfile::TempDir;

use crate::common::{env, login_owner, set_up};

/// The worked day's world: parties, items, partners 40 / 30 / 30 and today 10-10-2026.
pub struct World {
    pub dir: TempDir,
    pub env: TestEnv,
    pub owner: Actor,
    /// رحیم, seller 030110001.
    pub rahim: i64,
    /// کریم, seller 030110002.
    pub karim: i64,
    /// اکرم ٹول ٹیکس, buyer 03040001.
    pub akram: i64,
    /// ارشد ملتانی, buyer 03040002.
    pub arshad: i64,
    /// نقدی منڈی, 03040000.
    pub naqdi: i64,
    /// پختہ کھاتہ (خریدار), 03049999.
    pub pukhta_buyer: i64,
    /// پختہ کھاتہ (بیوپاری), 030119999.
    pub pukhta_seller: i64,
    /// کیلا, fruit.
    pub banana: i64,
    /// سیب, fruit.
    pub apple: i64,
    /// مولی, vegetable.
    pub mooli: i64,
    /// A fruit sold by weight.
    pub weight_fruit: i64,
    /// پیٹی.
    pub box_: i64,
    pub partners: [i64; 3],
}

pub fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

pub fn id_of(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap_or_else(|| panic!("no account {code}"))
}

fn confirm_words(items: &[mandi_services::Rejection]) -> Option<Vec<String>> {
    if !items
        .iter()
        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed)
    {
        return None;
    }
    Some(
        items
            .iter()
            .filter_map(|i| match i.values.get("words") {
                Some(ErrorValue::Text(words)) => Some(words.split(' ').map(str::to_owned)),
                _ => None,
            })
            .flatten()
            .collect(),
    )
}

pub fn add_account(
    env: &TestEnv,
    owner: &Actor,
    kind: AccountType,
    ur: &str,
    en: &str,
    credit_allowed: bool,
    credit_limit: &str,
) -> i64 {
    let mut input = AccountInput {
        id: None,
        version: None,
        account_type: kind,
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed,
        credit_limit: credit_limit.into(),
        default_marka_id: None,
        collector_staff_id: None,
        collection_weekday: None,
        is_active: true,
        note: String::new(),
    };
    loop {
        match accounts::save(&env.ctx, owner, &input) {
            Ok(row) => return row.id,
            Err(ServiceError::Rejected(items)) if confirm_words(&items).is_some() => {
                input
                    .confirmed_rules_words
                    .extend(confirm_words(&items).unwrap_or_default());
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

pub fn add_item(
    env: &TestEnv,
    owner: &Actor,
    ur: &str,
    en: &str,
    group: &str,
    sold_by: &str,
) -> i64 {
    let mut input = ItemInput {
        id: None,
        version: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        item_group: group.into(),
        default_packing_id: None,
        sold_by: sold_by.into(),
        confirmed_rules_words: Vec::new(),
    };
    loop {
        match masters::save(&env.ctx, owner, &MasterInput::Item(input.clone())) {
            Ok(row) => return row.id(),
            Err(ServiceError::Rejected(items)) if confirm_words(&items).is_some() => {
                input
                    .confirmed_rules_words
                    .extend(confirm_words(&items).unwrap_or_default());
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn add_packing(env: &TestEnv, owner: &Actor, ur: &str, en: &str) -> i64 {
    let mut input = PackingInput {
        id: None,
        version: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en.into(),
        approx_kg: String::new(),
        confirmed_rules_words: Vec::new(),
    };
    loop {
        match masters::save(&env.ctx, owner, &MasterInput::Packing(input.clone())) {
            Ok(row) => return row.id(),
            Err(ServiceError::Rejected(items)) if confirm_words(&items).is_some() => {
                input
                    .confirmed_rules_words
                    .extend(confirm_words(&items).unwrap_or_default());
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

fn new_partner(env: &TestEnv, owner: &Actor, ur: &str, en: &str) -> i64 {
    let mut words: Vec<String> = Vec::new();
    loop {
        let input = PartnerCreateInput {
            name_ur: ur.to_owned(),
            name_en: en.to_owned(),
            opening_capital: None,
            confirmed_rules_words: words.clone(),
        };
        match partners::create(&env.ctx, owner, &input) {
            Ok(row) => return row.id,
            Err(ServiceError::Rejected(items)) if confirm_words(&items).is_some() => {
                words.extend(confirm_words(&items).unwrap_or_default());
            }
            Err(other) => panic!("{other:?}"),
        }
    }
}

/// The worked day: today is 10-10-2026, nothing is closed, partners A / B / C have 40 / 30 / 30,
/// cash in hand is 50,000 (PRD 4.3 after the advance; the advance itself is not needed here).
pub fn world() -> World {
    world_with(50_000, true)
}

/// The same world with another cash in hand, and shares that are set or not (PRD FR-17.1).
pub fn world_with(cash: i64, shares: bool) -> World {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    let rahim = add_account(&env, &owner, AccountType::Seller, "رحیم", "Rahim", true, "");
    let karim = add_account(&env, &owner, AccountType::Seller, "کریم", "Karim", true, "");
    let akram = add_account(
        &env,
        &owner,
        AccountType::Buyer,
        "اکرم ٹول ٹیکس",
        "Akram Toll Tax",
        true,
        "",
    );
    let arshad = add_account(
        &env,
        &owner,
        AccountType::Buyer,
        "ارشد ملتانی",
        "Arshad Multani",
        true,
        "",
    );
    let banana = add_item(&env, &owner, "کیلا", "Banana", "fruit", "unit");
    let apple = add_item(&env, &owner, "سیب", "Apple", "fruit", "unit");
    let mooli = add_item(&env, &owner, "مولی", "Radish", "vegetable", "unit");
    let weight_fruit = add_item(&env, &owner, "انگور", "Grapes", "fruit", "weight");
    let box_ = add_packing(&env, &owner, "پیٹی", "Box");
    let a = new_partner(&env, &owner, "الف", "Partner A");
    let b = new_partner(&env, &owner, "ب", "Partner B");
    let c = new_partner(&env, &owner, "ج", "Partner C");
    if shares {
        partners::set_shares(
            &env.ctx,
            &owner,
            &PartnersSetSharesInput {
                effective_from: "10-10-2026".into(),
                shares: [(a, "40"), (b, "30"), (c, "30")]
                    .iter()
                    .map(|&(partner_id, share)| ShareInput {
                        partner_id,
                        share: share.to_owned(),
                    })
                    .collect(),
            },
        )
        .unwrap();
    }
    let naqdi = id_of(&env, "03040000");
    let pukhta_buyer = id_of(&env, "03049999");
    let pukhta_seller = id_of(&env, "030119999");
    let w = World {
        dir,
        env,
        owner,
        rahim,
        karim,
        akram,
        arshad,
        naqdi,
        pukhta_buyer,
        pukhta_seller,
        banana,
        apple,
        mooli,
        weight_fruit,
        box_,
        partners: [a, b, c],
    };
    if cash != 0 {
        set_cash(&w, cash);
    }
    w
}

/// Puts cash in hand at `amount` rupees with an opening dated today (PRD FR-10.9). Call it once.
pub fn set_cash(w: &World, amount: i64) {
    let cash = id_of(&w.env, "01010001");
    let capital = id_of(&w.env, "06010001");
    let line = |id: i64, dr: &str, cr: &str| OpeningLineInput {
        account_id: id,
        version: accounts::get(&w.env.ctx, id).unwrap().version,
        opening_date: "10-10-2026".into(),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: false,
    };
    let text = amount.to_string();
    accounts::openings_save(
        &w.env.ctx,
        &w.owner,
        &OpeningsInput {
            lines: vec![line(cash, &text, ""), line(capital, "", &text)],
        },
    )
    .unwrap();
}

// ---- Draft builders ----

pub fn header() -> HeaderInput {
    HeaderInput {
        entry_date: "10-10-2026".into(),
        bill_date: String::new(),
        book: None,
        truck_no: Some("LES-1234".into()),
        zamindar_account_id: None,
        adda_id: None,
        driver_name: None,
        driver_phone: None,
        total_freight: None,
        store_id: None,
        note: None,
    }
}

pub fn manual(value: &str) -> Option<ChargeInput> {
    Some(ChargeInput {
        method: LotChargeMethod::Manual,
        value: value.into(),
    })
}

pub fn per_unit(value: &str) -> Option<ChargeInput> {
    Some(ChargeInput {
        method: LotChargeMethod::PerUnit,
        value: value.into(),
    })
}

/// A lot of PRD 4.4: freight 3,000 paid by the firm, labour 5 per نگ, munshiana 50, phone 20,
/// نقد 5,000. Lines are added with [`with_sales`].
pub fn lot(w: &World, key: &str, seller: i64, item: i64, qty_received: &str) -> LotInput {
    LotInput {
        key: key.into(),
        seller_account_id: Some(seller),
        pukhta: None,
        item_id: Some(item),
        packing_id: Some(w.box_),
        marka_id: None,
        store_id: None,
        qty_received: qty_received.into(),
        weight_received: None,
        freight: Some("3,000".into()),
        freight_paid_by: FreightPaidBy::Firm,
        labour: per_unit("5"),
        munshiana: manual("50"),
        phone: manual("20"),
        cash_paid: Some("5,000".into()),
        store_rent: None,
        adjust_advance: true,
        sales: Vec::new(),
    }
}

/// A lot with no charges at all: every charge empty, so the rules decide.
pub fn bare_lot(w: &World, key: &str, seller: i64, item: i64, qty_received: &str) -> LotInput {
    LotInput {
        freight: None,
        labour: None,
        munshiana: None,
        phone: None,
        cash_paid: None,
        ..lot(w, key, seller, item, qty_received)
    }
}

pub fn sale(key: &str, buyer: i64, qty: &str, rate: &str) -> SaleInput {
    SaleInput {
        key: key.into(),
        buyer_account_id: Some(buyer),
        pukhta: None,
        qty: qty.into(),
        weight: None,
        seller_rate: rate.into(),
        rate_differs: false,
        buyer_rate: None,
        payment_type: None,
        seller_comm: None,
        buyer_comm: None,
        laga: None,
        note: None,
    }
}

/// A row with only the quantity the screen filled in: no buyer, no rate.
pub fn blank_sale(key: &str, qty: &str) -> SaleInput {
    SaleInput {
        buyer_account_id: None,
        ..sale(key, 0, qty, "")
    }
}

pub fn typed(method: LineChargeMethod, value: &str) -> Option<LineChargeInput> {
    Some(LineChargeInput {
        method,
        value: value.into(),
    })
}

pub fn with_sales(mut lot: LotInput, sales: Vec<SaleInput>) -> LotInput {
    lot.sales = sales;
    lot
}

pub fn draft(lots: Vec<LotInput>) -> BikriDraft {
    BikriDraft {
        header: header(),
        lots,
    }
}

/// PRD 4.4, Event 1: رحیم's 50 کیلا, اکرم 20 × 600 and ارشد 30 × 580.
pub fn event1(w: &World) -> BikriDraft {
    draft(vec![with_sales(
        lot(w, "lot-1", w.rahim, w.banana, "50"),
        vec![
            sale("s-1", w.akram, "20", "600"),
            sale("s-2", w.arshad, "30", "580"),
        ],
    )])
}

pub fn preview(w: &World, draft: &BikriDraft) -> BikriPreview {
    bikri::preview(&w.env.ctx, draft).unwrap()
}

// ---- Reading a preview ----

/// A row's numbers; panics if the row is not complete.
pub fn calc(lot: &LotPreview, line: usize) -> &LineCalc {
    lot.lines
        .get(line)
        .and_then(|l| l.calc.as_ref())
        .unwrap_or_else(|| panic!("line {line} has no numbers: {:?}", lot.lines.get(line)))
}

/// Rupees of a money value.
pub fn r(value: Paisa) -> i64 {
    value.rupees()
}

pub fn has(list: &[FieldError], code: ErrorCode, field: &str) -> bool {
    list.iter().any(|e| e.code == code && e.field == field)
}

// ---- Saved data, inserted with plain SQL (the save itself comes in plan 13) ----

/// One saved sale line to put into the database, with a lot and a بکری of its own.
pub struct SavedSale<'a> {
    pub number: i64,
    pub date: &'a str,
    pub truck: Option<&'a str>,
    pub seller: i64,
    pub buyer: i64,
    pub item: i64,
    pub packing: i64,
    pub marka: Option<i64>,
    pub qty: i64,
    /// The seller rate in whole rupees.
    pub rate: i64,
    pub adda: Option<i64>,
    pub zamindar: Option<i64>,
    pub driver: Option<&'a str>,
}

impl<'a> SavedSale<'a> {
    pub fn new(
        number: i64,
        date: &'a str,
        seller: i64,
        buyer: i64,
        item: i64,
        packing: i64,
    ) -> Self {
        Self {
            number,
            date,
            truck: None,
            seller,
            buyer,
            item,
            packing,
            marka: None,
            qty: 10,
            rate: 600,
            adda: None,
            zamindar: None,
            driver: None,
        }
    }
}

/// Inserts a saved بکری of one lot and one line, as the save of plan 13 will, with the numbers
/// of the 6% / 6% / 8 per نگ defaults. Returns (bikri id, lot id, sale id).
pub fn insert_saved(w: &World, s: &SavedSale<'_>) -> (i64, i64, i64) {
    let conn = crate::common::raw(&w.dir);
    let paisa = |rupees: i64| rupees * 100;
    let amount = s.qty * s.rate;
    let comm = (amount * 6 + 50) / 100;
    let laga = s.qty * 8;
    let total = amount + comm + laga;
    conn.execute(
        "INSERT INTO bikri_entries (number, entry_date, bill_date, truck_no, zamindar_account_id, adda_id, driver_name)
         VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6)",
        mandi_services::db::rusqlite::params![s.number, s.date, s.truck, s.zamindar, s.adda, s.driver],
    )
    .unwrap();
    let bikri_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO bikri_lots (bikri_id, lot_no, seller_account_id, item_id, packing_id, marka_id, qty_received)
         VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6)",
        mandi_services::db::rusqlite::params![bikri_id, s.seller, s.item, s.packing, s.marka, s.qty],
    )
    .unwrap();
    let lot_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, status, total_paisa)
         VALUES (?1, 'bikri_first', 'bikri', ?2, 'draft', 0)",
        mandi_services::db::rusqlite::params![s.date, bikri_id],
    )
    .unwrap();
    let entry_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date, buyer_account_id, qty,
            seller_rate_paisa, seller_amount_paisa, seller_comm_method, seller_comm_value_bps, seller_comm_paisa,
            buyer_rate_paisa, buyer_value_paisa, buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa,
            laga_method, laga_value_paisa, laga_paisa, buyer_total_paisa, rate_diff_paisa, payment_type)
         VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6, ?7, ?8, 'percent', 600, ?9, ?7, ?8, 'percent', 600, ?9,
            'per_unit', 800, ?10, ?11, 0, 'credit')",
        mandi_services::db::rusqlite::params![
            bikri_id, lot_id, entry_id, s.date, s.buyer, s.qty,
            paisa(s.rate), paisa(amount), paisa(comm), paisa(laga), paisa(total)
        ],
    )
    .unwrap();
    let sale_id = conn.last_insert_rowid();
    let year: i64 = s.date[0..4].parse().unwrap();
    let month: i64 = s.date[5..7].parse().unwrap();
    conn.execute(
        "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id, sale_id, item_id,
            seller_account_id, buyer_account_id, qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa,
            buyer_comm_paisa, laga_paisa, sign)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?12, ?13, ?14, 1)",
        mandi_services::db::rusqlite::params![
            entry_id, s.date, year, month, bikri_id, lot_id, sale_id, s.item, s.seller, s.buyer, s.qty,
            paisa(amount), paisa(comm), paisa(laga)
        ],
    )
    .unwrap();
    (bikri_id, lot_id, sale_id)
}
