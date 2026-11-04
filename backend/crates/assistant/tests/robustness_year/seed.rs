//! The rich test year (plan 52 item 12): 12 months of a commission agent's books, built only
//! through the app's own save functions (the same ones the screens call), so the journal balances
//! and every report reads it as it would read Umar's real data. Nothing is written with raw SQL.
//!
//! - Today is 05-11-2026. The books start on 06-11-2025.
//! - Six items: kela, saib, mausami, aam, angoor, aloo. Ten sellers. Fifteen buyers, among them
//!   "Faisal" (one) and two Akrams. Three partners A, B and C.
//! - A garden (Shahpur Garden, open), two market trips (one closed, one open), fruit taken for a
//!   partner's home and for the guests, shop and personal expenses, partner capital, money lent,
//!   advances to sellers, dues of every age (a prompt payer to a buyer who never paid and is marked
//!   as a dead amount), and unsold stock.
//! - The seed is fixed: the same books on every run.
//!
//! The expected answers are NOT here. They come from `oracle.rs`, which asks the app's own
//! reports and analytics after the books are built.
#![allow(dead_code)]

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use mandi_services::auth;
use mandi_services::core::accounts::{AccountType, OpeningLineInput, OpeningsInput};
use mandi_services::core::bikri::{
    BikriDraft, BikriSaveInput, BikriSaved, HeaderInput, LotInput, SaleInput,
};
use mandi_services::core::capital::CapitalInput;
use mandi_services::core::dead::{DeadMarkInput, DeadReason};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::expenses::{PersonalExpenseInput, PersonalReturnInput, ShopExpenseInput};
use mandi_services::core::fruit::{FruitInput, FruitLotsInput, FruitSource};
use mandi_services::core::gardens::{
    GardenEntryInput, GardenInput, GardenOtherSaleInput, GardenPaymentInput,
};
use mandi_services::core::lent::{
    LentPersonCreateInput, LentReturnMode, MoneyLentInput, MoneyLentReturnInput,
};
use mandi_services::core::partners::{PartnersSetSharesInput, ShareInput};
use mandi_services::core::purchases::{PurchaseInput, PurchaseWriteOffInput};
use mandi_services::core::trips::{OtherSaleInput, TripCostInput, TripInput};
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::core::vouchers::{
    AdvanceInput, DayCloseInput, OpenBillsInput, OpenPayablesInput, PaymentInput, ReceiptInput,
    RemovedPaymentFields,
};
use mandi_services::testing::TestEnv;
use mandi_services::{
    ServiceError, accounts, bikri, day, dead, fruit, gardens, lent, partners, purchases, trips,
    vouchers,
};
use tempfile::TempDir;

use crate::common::{env, login_owner, set_today, set_up};
use crate::world;

/// The first day of the books.
pub const START: (i32, u32, u32) = (2025, 11, 6);
/// Today, the last day of the books.
pub const TODAY: (i32, u32, u32) = (2026, 11, 5);

pub fn date(parts: (i32, u32, u32)) -> NaiveDate {
    NaiveDate::from_ymd_opt(parts.0, parts.1, parts.2).expect("a real date")
}

/// `DD-MM-YYYY`, the way a screen sends a date.
pub fn ddmmyyyy(d: NaiveDate) -> String {
    format!("{:02}-{:02}-{}", d.day(), d.month(), d.year())
}

// ---- a small fixed random source ---------------------------------------------------------------

/// SplitMix64: a fixed seed gives the same books on every run and every machine.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A number in `lo..=hi`.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        let span = u64::try_from(hi - lo + 1).unwrap_or(1).max(1);
        lo + i64::try_from(self.next() % span).unwrap_or(0)
    }
    /// True with the chance `percent` out of 100.
    pub fn chance(&mut self, percent: i64) -> bool {
        self.range(1, 100) <= percent
    }
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let at = usize::try_from(self.range(0, i64::try_from(items.len()).unwrap_or(1) - 1))
            .unwrap_or(0);
        &items[at]
    }
}

// ---- the people and things ---------------------------------------------------------------------

/// One item of the books: its ids and its price habit.
pub struct ItemInfo {
    pub key: &'static str,
    pub id: i64,
    pub packing: i64,
    /// A typical rate per نگ in rupees.
    pub base_rate: i64,
    /// The months it is in the market (1 to 12).
    pub months: &'static [u32],
    pub qty: (i64, i64),
}

pub struct SellerInfo {
    pub key: &'static str,
    pub id: i64,
    pub items: &'static [&'static str],
    /// Paid when a posting is at least this many days old.
    pub pay_after_days: i64,
}

pub struct BuyerInfo {
    pub key: &'static str,
    pub id: i64,
    /// False: cash only.
    pub credit: bool,
    /// Buys from this day.
    pub from: NaiveDate,
    /// Buys until this day.
    pub until: NaiveDate,
    /// Pays bills this many days old, or never.
    pub pay_after_days: Option<i64>,
    /// How much of the old bills he pays, in percent.
    pub pay_percent: i64,
    /// Stops paying after this day.
    pub stops_paying: Option<NaiveDate>,
}

pub struct Year {
    pub w: world::World,
    pub items: Vec<ItemInfo>,
    pub sellers: Vec<SellerInfo>,
    pub buyers: Vec<BuyerInfo>,
    pub partner_ids: [i64; 3],
    pub heads: Heads,
    pub garden_id: i64,
    pub garden_account: i64,
    pub trip_one: i64,
    pub trip_two: i64,
    pub own_goods: i64,
    pub aslam: i64,
    pub haji_nazeer: i64,
    pub bikri_count: usize,
    /// (trip, item key, purchase id) of every trip purchase.
    pub purchase_ids: Vec<(i64, &'static str, i64)>,
}

pub struct Heads {
    pub rent: i64,
    pub electricity: i64,
    pub salaries: i64,
    pub tea: i64,
    pub guests: i64,
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

/// Calls `f` with the words confirmed so far, and confirms the spellings the engine is unsure of
/// until the call goes through (the way the screen asks the user).
fn confirming<T>(
    mut f: impl FnMut(&[String]) -> Result<T, ServiceError>,
) -> Result<T, ServiceError> {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..8 {
        match f(&words) {
            Err(ServiceError::Rejected(items)) if confirm_words(&items).is_some() => {
                words.extend(confirm_words(&items).unwrap_or_default());
            }
            other => return other,
        }
    }
    f(&words)
}

fn shop(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Shop,
    }
}

fn pocket(partner_id: i64, share: &str) -> VenturePartnerInput {
    VenturePartnerInput {
        partner_id,
        share: share.into(),
        source: MoneySource::Pocket,
    }
}

// ---- building ----------------------------------------------------------------------------------

/// Builds the whole year. Takes about half a minute in a debug build.
pub fn build() -> Year {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).expect("the Owner");
    let start = date(START);
    set_today(&env, START.0, START.1, START.2);
    let mut year = people_and_things(dir, env, owner);
    opening_position(&year, start);
    let mut rng = Rng::new(20_261_105);
    run_days(&mut year, &mut rng);
    end_of_year(&year);
    year
}

/// The thin books of a shop that has only just begun (like the Owner's own data in plan 52):
/// the same people, items and partners, the opening position, and no sale, no voucher, no
/// expense. Today is the same day as in [`build`].
pub fn build_thin() -> Year {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).expect("the Owner");
    set_today(&env, START.0, START.1, START.2);
    let year = people_and_things(dir, env, owner);
    opening_position(&year, date(START));
    set_today(&year.w.env, TODAY.0, TODAY.1, TODAY.2);
    year
}

fn people_and_things(dir: TempDir, env: TestEnv, owner: auth::Actor) -> Year {
    let seller = |ur: &str, en: &str| {
        world::add_account(&env, &owner, AccountType::Seller, ur, en, true, "")
    };
    let buyer = |ur: &str, en: &str, credit: bool, limit: &str| {
        world::add_account(&env, &owner, AccountType::Buyer, ur, en, credit, limit)
    };
    let rahim = seller("رحیم", "Rahim");
    let karim = seller("کریم", "Karim");
    let naseer = seller("نصیر", "Naseer");
    let jameel = seller("جمیل", "Jameel");
    let saleem = seller("سلیم", "Saleem");
    let bashir = seller("بشیر", "Bashir");
    let tahir = seller("طاہر", "Tahir");
    let ghulam = seller("غلام رسول", "Ghulam Rasool");
    let nazeer = seller("حاجی نذیر", "Haji Nazeer");
    let sarfraz = seller("سرفراز", "Sarfraz");

    let start = date(START);
    let d = |days: i64| start + Duration::days(days);
    let buyer_rows: Vec<(&str, &str, &str, bool, &str, NaiveDate, NaiveDate)> = vec![
        (
            "akram_toll_tax",
            "اکرم ٹول ٹیکس",
            "Akram Toll Tax",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "akram_baloch",
            "اکرم بلوچ",
            "Akram Baloch",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "arshad",
            "ارشد ملتانی",
            "Arshad Multani",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        ("faisal", "فیصل", "Faisal", true, "", d(0), date(TODAY)),
        (
            "shahid",
            "شاہد رضا",
            "Shahid Raza",
            false,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "nadeem",
            "ندیم ٹریڈرز",
            "Nadeem Traders",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "imran",
            "عمران فروٹ",
            "Imran Fruit",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "waqas",
            "وقاص برادرز",
            "Waqas Brothers",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "zahid",
            "زاہد محمود",
            "Zahid Mehmood",
            true,
            "",
            d(0),
            date((2026, 7, 31)),
        ),
        (
            "kashif",
            "کاشف علی",
            "Kashif Ali",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "tariq",
            "طارق محمود",
            "Tariq Mehmood",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "bilal",
            "بلال احمد",
            "Bilal Ahmed",
            true,
            "",
            d(0),
            date((2026, 1, 31)),
        ),
        (
            "usman",
            "عثمان غنی",
            "Usman Ghani",
            true,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "rizwan",
            "رضوان بٹ",
            "Rizwan Butt",
            false,
            "",
            d(0),
            date(TODAY),
        ),
        (
            "danish",
            "دانش خان",
            "Danish Khan",
            true,
            "",
            d(0),
            date(TODAY),
        ),
    ];
    // How each pays: after how many days, how much of it, and whether he stops.
    let habits: [(Option<i64>, i64, Option<NaiveDate>); 15] = [
        (Some(7), 100, None),
        (Some(30), 100, None),
        (Some(45), 60, None),
        (Some(14), 100, None),
        (None, 0, None),
        (Some(21), 100, None),
        (Some(60), 100, None),
        (Some(30), 60, None),
        (Some(14), 100, Some(date((2026, 4, 30)))),
        (Some(10), 100, None),
        (Some(90), 100, None),
        (None, 0, None),
        (Some(35), 100, None),
        (None, 0, None),
        (Some(3), 100, None),
    ];
    let buyers: Vec<BuyerInfo> = buyer_rows
        .iter()
        .zip(habits)
        .map(|(row, habit)| {
            let (key, ur, en, credit, limit, from, until) = *row;
            BuyerInfo {
                key,
                id: buyer(ur, en, credit, limit),
                credit,
                from,
                until,
                pay_after_days: habit.0,
                pay_percent: habit.1,
                stops_paying: habit.2,
            }
        })
        .collect();

    let kela = world::add_item(&env, &owner, "کیلا", "Banana", "fruit", "unit");
    let saib = world::add_item(&env, &owner, "سیب", "Apple", "fruit", "unit");
    let mausami = world::add_item(&env, &owner, "موسمی", "Mosambi", "fruit", "unit");
    let aam = world::add_item(&env, &owner, "آم", "Mango", "fruit", "unit");
    let angoor = world::add_item(&env, &owner, "انگور", "Grapes", "fruit", "unit");
    let aloo = world::add_item(&env, &owner, "آلو", "Potato", "vegetable", "unit");
    let box_ = world::add_packing(&env, &owner, "پیٹی", "Box");
    let bag = world::add_packing(&env, &owner, "بوری", "Bag");

    let all: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let items = vec![
        ItemInfo {
            key: "kela",
            id: kela,
            packing: box_,
            base_rate: 1_900,
            months: all,
            qty: (30, 110),
        },
        ItemInfo {
            key: "saib",
            id: saib,
            packing: box_,
            base_rate: 4_200,
            months: &[1, 2, 3, 4, 5, 9, 10, 11, 12],
            qty: (25, 90),
        },
        ItemInfo {
            key: "mausami",
            id: mausami,
            packing: box_,
            base_rate: 2_400,
            months: &[1, 2, 3, 4, 10, 11, 12],
            qty: (30, 100),
        },
        ItemInfo {
            key: "aam",
            id: aam,
            packing: box_,
            base_rate: 5_400,
            months: &[5, 6, 7, 8],
            qty: (25, 80),
        },
        ItemInfo {
            key: "angoor",
            id: angoor,
            packing: box_,
            base_rate: 3_600,
            months: &[6, 7, 8, 9, 10],
            qty: (25, 70),
        },
        ItemInfo {
            key: "aloo",
            id: aloo,
            packing: bag,
            base_rate: 1_100,
            months: all,
            qty: (80, 220),
        },
    ];
    let sellers = vec![
        SellerInfo {
            key: "rahim",
            id: rahim,
            items: &["kela", "saib"],
            pay_after_days: 14,
        },
        SellerInfo {
            key: "karim",
            id: karim,
            items: &["saib", "mausami"],
            pay_after_days: 21,
        },
        SellerInfo {
            key: "naseer",
            id: naseer,
            items: &["mausami", "kela"],
            pay_after_days: 30,
        },
        SellerInfo {
            key: "jameel",
            id: jameel,
            items: &["aam", "angoor"],
            pay_after_days: 14,
        },
        SellerInfo {
            key: "saleem",
            id: saleem,
            items: &["aloo"],
            pay_after_days: 7,
        },
        SellerInfo {
            key: "bashir",
            id: bashir,
            items: &["kela"],
            pay_after_days: 45,
        },
        SellerInfo {
            key: "tahir",
            id: tahir,
            items: &["saib", "mausami"],
            pay_after_days: 30,
        },
        SellerInfo {
            key: "ghulam_rasool",
            id: ghulam,
            items: &["angoor", "aam"],
            pay_after_days: 21,
        },
        SellerInfo {
            key: "haji_nazeer",
            id: nazeer,
            items: &["saib", "kela"],
            pay_after_days: 60,
        },
        SellerInfo {
            key: "sarfraz",
            id: sarfraz,
            items: &["aloo", "kela"],
            pay_after_days: 10,
        },
    ];

    let a = world::new_partner(&env, &owner, "الف", "Partner A");
    let b = world::new_partner(&env, &owner, "ب", "Partner B");
    // The third partner is Usman (Umar's own question "usman shirakat daar ka kitna hai abhi").
    let c = world::new_partner(&env, &owner, "عثمان", "Usman");
    partners::set_shares(
        &env.ctx,
        &owner,
        &PartnersSetSharesInput {
            effective_from: ddmmyyyy(date(START)),
            shares: [(a, "40"), (b, "30"), (c, "30")]
                .iter()
                .map(|&(partner_id, share)| ShareInput {
                    partner_id,
                    share: share.to_owned(),
                })
                .collect(),
        },
    )
    .expect("partner shares");
    let naqdi = world::id_of(&env, "03040000");
    let pukhta_buyer = world::id_of(&env, "0030");
    let pukhta_seller = world::id_of(&env, "0031");
    let own_goods = world::id_of(&env, "030119998");
    let w = world::World {
        dir,
        env,
        owner,
        rahim,
        karim,
        akram: buyers[0].id,
        arshad: buyers[2].id,
        naqdi,
        pukhta_buyer,
        pukhta_seller,
        banana: kela,
        apple: saib,
        mooli: aloo,
        weight_fruit: angoor,
        box_,
        partners: [a, b, c],
    };
    let head = |ur: &str, en: &str| crate::worked::expense_head(&w, ur, en);
    let heads = Heads {
        rent: head("کرایہ دکان", "Shop Rent"),
        electricity: head("بجلی", "Electricity"),
        salaries: head("تنخواہ", "Salaries"),
        tea: head("چائے", "Tea"),
        guests: head("مہمان نوازی", "Guests Food and Drinks"),
    };
    let aslam = confirming(|words| {
        lent::person_create(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateInput {
                name_ur: "اسلم".into(),
                name_en: "Aslam".into(),
                phone: None,
                staff_id: None,
                confirmed_rules_words: words.to_vec(),
            },
        )
    })
    .expect("the staff member")
    .account
    .id;
    Year {
        w,
        items,
        sellers,
        buyers,
        partner_ids: [a, b, c],
        heads,
        garden_id: 0,
        garden_account: 0,
        trip_one: 0,
        trip_two: 0,
        own_goods,
        aslam,
        haji_nazeer: nazeer,
        bikri_count: 0,
        purchase_ids: Vec::new(),
    }
}

fn opening_position(y: &Year, start: NaiveDate) {
    let cash = world::id_of(&y.w.env, "01010001");
    let capital = world::id_of(&y.w.env, "06010001");
    let line = |id: i64, dr: &str, cr: &str| OpeningLineInput {
        account_id: id,
        version: accounts::get(&y.w.env.ctx, id).expect("account").version,
        opening_date: ddmmyyyy(start),
        opening_dr: dr.into(),
        opening_cr: cr.into(),
        opening_is_advance: false,
    };
    accounts::openings_save(
        &y.w.env.ctx,
        &y.w.owner,
        &OpeningsInput {
            lines: vec![line(cash, "2,500,000", ""), line(capital, "", "2,500,000")],
        },
    )
    .expect("the opening position");
}

// ---- the days ----------------------------------------------------------------------------------

fn seasonal(item: &ItemInfo, month: u32) -> bool {
    item.months.contains(&month)
}

fn rate_for(item: &ItemInfo, month: u32, rng: &mut Rng) -> i64 {
    // A swing through the year (±20%) plus a little noise, in whole tens of rupees.
    let swing = [95, 92, 90, 94, 100, 108, 115, 112, 106, 100, 97, 96]
        [usize::try_from(month - 1).unwrap_or(0)];
    let noise = rng.range(94, 106);
    ((item.base_rate * swing / 100 * noise / 100) / 10) * 10
}

fn header(day: NaiveDate, truck: &str) -> HeaderInput {
    HeaderInput {
        entry_date: ddmmyyyy(day),
        truck_no: Some(truck.to_owned()),
        ..world::header()
    }
}

fn save_bikri(y: &mut Year, draft: BikriDraft) -> BikriSaved {
    let saved = bikri::save(
        &y.w.env.ctx,
        &y.w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_or_else(|e| panic!("a بکری of the year: {e:?}"));
    y.bikri_count += 1;
    saved
}

/// A lot of one item from a seller, with 2 to 4 sale lines to buyers who are active on `day`.
fn lot_for(
    y: &Year,
    rng: &mut Rng,
    day: NaiveDate,
    key: &str,
    seller_id: i64,
    item: &ItemInfo,
) -> LotInput {
    let month = day.month();
    let qty = rng.range(item.qty.0, item.qty.1);
    let mut lot = world::lot(&y.w, key, seller_id, item.id, &qty.to_string());
    lot.packing_id = Some(item.packing);
    lot.freight = Some(rng.range(10, 40).saturating_mul(100).to_string());
    lot.cash_paid = if rng.chance(40) {
        Some(rng.range(10, 80).saturating_mul(100).to_string())
    } else {
        None
    };
    let leave_some = rng.chance(8);
    lot.sales = lines_for(y, rng, day, key, item, qty, leave_some);
    let _ = month;
    lot
}

fn lines_for(
    y: &Year,
    rng: &mut Rng,
    day: NaiveDate,
    key: &str,
    item: &ItemInfo,
    qty: i64,
    leave_some: bool,
) -> Vec<SaleInput> {
    let active: Vec<&BuyerInfo> = y
        .buyers
        .iter()
        .filter(|b| b.from <= day && day <= b.until)
        .collect();
    let to_sell = if leave_some {
        qty - rng.range(3, (qty / 4).max(4))
    } else {
        qty
    };
    let parts = rng.range(2, 4).min(to_sell.max(1));
    let mut left = to_sell;
    let base = rate_for(item, day.month(), rng);
    let mut lines = Vec::new();
    for i in 0..parts {
        let take = if i == parts - 1 {
            left
        } else {
            (to_sell / parts).max(1).min(left)
        };
        if take <= 0 {
            break;
        }
        left -= take;
        let buyer = if rng.chance(7) {
            y.w.naqdi
        } else {
            rng.pick(&active).id
        };
        let rate = (base * rng.range(97, 103) / 100 / 10) * 10;
        lines.push(world::sale(
            &format!("{key}-s{i}"),
            buyer,
            &take.to_string(),
            &rate.to_string(),
        ));
    }
    lines
}

fn run_days(y: &mut Year, rng: &mut Rng) {
    let start = date(START);
    let today = date(TODAY);
    let garden_days: Vec<NaiveDate> = [
        (2026, 6, 4),
        (2026, 6, 12),
        (2026, 6, 20),
        (2026, 6, 29),
        (2026, 7, 8),
        (2026, 7, 21),
        (2026, 8, 3),
    ]
    .map(date)
    .to_vec();
    let mut day_no = 0;
    let mut cursor = start;
    while cursor <= today {
        set_today(&y.w.env, cursor.year(), cursor.month(), cursor.day());
        let weekday = cursor.weekday();
        // Special events first (they use their own lots), then the ordinary market day.
        special_events(y, rng, cursor, &garden_days);
        // The market day.
        let bikris = match weekday {
            Weekday::Sun => i64::from(rng.chance(15)),
            _ => {
                let roll = rng.range(1, 100);
                if roll <= 18 {
                    0
                } else if roll <= 72 {
                    1
                } else {
                    2
                }
            }
        };
        for _ in 0..bikris {
            ordinary_bikri(y, rng, cursor);
        }
        money_events(y, rng, cursor);
        day_no += 1;
        cursor += Duration::days(1);
    }
    let _ = day_no;
}

fn ordinary_bikri(y: &mut Year, rng: &mut Rng, day: NaiveDate) {
    let n_lots = if rng.chance(30) { 2 } else { 1 };
    let mut lots = Vec::new();
    for i in 0..n_lots {
        // A seller who has an item in season.
        let mut tries = 0;
        let (seller_id, item_ix) = loop {
            tries += 1;
            let at = usize::try_from(rng.range(0, i64::try_from(y.sellers.len()).unwrap_or(1) - 1))
                .unwrap_or(0);
            let s = &y.sellers[at];
            let key = *rng.pick(s.items);
            if let Some(ix) = y
                .items
                .iter()
                .position(|it| it.key == key && seasonal(it, day.month()))
            {
                break (s.id, ix);
            }
            if tries > 40 {
                // A seller of kela is always possible.
                break (y.w.rahim, 0);
            }
        };
        let item = &y.items[item_ix];
        let lot = lot_for(y, rng, day, &format!("lot-{i}"), seller_id, item);
        lots.push(lot);
    }
    let truck = format!("LES-{}", rng.range(1000, 9999));
    save_bikri(
        y,
        BikriDraft {
            header: header(day, &truck),
            lots,
        },
    );
}

fn money_events(y: &mut Year, rng: &mut Rng, day: NaiveDate) {
    let ctx = &y.w.env.ctx;
    let owner = &y.w.owner;
    let dd = ddmmyyyy(day);
    let [a, b, c] = y.partner_ids;
    let shop_expense = |head: i64, amount: i64| {
        vouchers::save_shop_expense(
            ctx,
            owner,
            &ShopExpenseInput {
                date: dd.clone(),
                expense_head_id: Some(head),
                amount: amount.to_string(),
                paid_to: None,
                narration: None,
                confirm_cash_below_zero: true,
            },
        )
        .unwrap_or_else(|e| panic!("a shop expense on {dd}: {e:?}"));
    };
    let personal = |partner: i64, amount: i64, reason: &str| {
        vouchers::save_personal_expense(
            ctx,
            owner,
            &PersonalExpenseInput {
                date: dd.clone(),
                partner_id: Some(partner),
                amount: amount.to_string(),
                reason: reason.to_owned(),
                return_by: None,
                taken_by: None,
                narration: None,
                confirm_cash_below_zero: true,
            },
        )
        .unwrap_or_else(|e| panic!("a personal expense on {dd}: {e:?}"));
    };
    // Expenses.
    if day.day() == 1 {
        shop_expense(y.heads.rent, 40_000);
        shop_expense(y.heads.salaries, 90_000);
    }
    if day.day() == 8 {
        shop_expense(y.heads.electricity, rng.range(18, 45) * 1_000);
    }
    if day.weekday() == Weekday::Mon {
        shop_expense(y.heads.tea, rng.range(15, 30) * 100);
    }
    if day.day() == 20 {
        shop_expense(y.heads.guests, rng.range(40, 90) * 100);
    }
    // Partners take money and give some back.
    if day.day() == 10 {
        personal(a, 25_000, "گھر کا خرچ");
    }
    if day.day() == 12 && day.month().is_multiple_of(2) {
        personal(b, 15_000, "بچوں کی فیس");
    }
    if day.day() == 14 && (day.month() == 3 || day.month() == 8) {
        personal(c, 8_000, "علاج");
    }
    if day.day() == 25 && day.month().is_multiple_of(3) {
        vouchers::save_personal_return(
            ctx,
            owner,
            &PersonalReturnInput {
                date: dd.clone(),
                partner_id: Some(a),
                amount: "10,000".into(),
                narration: None,
            },
        )
        .unwrap_or_else(|e| panic!("a personal return on {dd}: {e:?}"));
    }
    // Capital.
    let capital = |partner: i64, amount: &str, out: bool| {
        let input = CapitalInput {
            date: dd.clone(),
            partner_id: Some(partner),
            amount: amount.to_owned(),
            narration: None,
            confirm_cash_below_zero: true,
        };
        let saved = if out {
            vouchers::save_capital_out(ctx, owner, &input)
        } else {
            vouchers::save_capital_in(ctx, owner, &input)
        };
        saved.unwrap_or_else(|e| panic!("capital on {dd}: {e:?}"));
    };
    match (day.year(), day.month(), day.day()) {
        (2025, 12, 10) => capital(c, "300,000", false),
        (2026, 1, 15) => capital(b, "500,000", false),
        (2026, 8, 20) => capital(c, "100,000", true),
        _ => {}
    }
    // Money lent to Aslam.
    let aslam = y.aslam;
    match (day.year(), day.month(), day.day()) {
        (2026, 4, 5) => lend(y, &dd, "10,000", Some("05-05-2026"), "گھریلو ضرورت"),
        (2026, 10, 10) => lend(y, &dd, "5,000", Some("30-11-2026"), "علاج"),
        (2026, 5, 20) => {
            vouchers::save_money_lent_return(
                &y.w.env.ctx,
                &y.w.owner,
                &MoneyLentReturnInput {
                    date: dd.clone(),
                    person_account_id: Some(aslam),
                    amount: "3,000".into(),
                    mode: LentReturnMode::Cash,
                    salary_head_id: None,
                    narration: None,
                },
            )
            .expect("a loan returned in cash");
        }
        (2026, 6, 30) => {
            vouchers::save_money_lent_return(
                &y.w.env.ctx,
                &y.w.owner,
                &MoneyLentReturnInput {
                    date: dd.clone(),
                    person_account_id: Some(aslam),
                    amount: "2,000".into(),
                    mode: LentReturnMode::Salary,
                    salary_head_id: Some(y.heads.salaries),
                    narration: None,
                },
            )
            .expect("a loan returned from salary");
        }
        _ => {}
    }
    // Advances to sellers: Tahir every second month, Rahim in December.
    if day.day() == 3 && !day.month().is_multiple_of(2) {
        advance(y, &dd, y.sellers[6].id, "30,000");
    }
    if day.year() == 2025 && day.month() == 12 && day.day() == 5 {
        advance(y, &dd, y.w.rahim, "15,000");
    }
    // Collections on Saturdays, payments on Mondays.
    if day.weekday() == Weekday::Sat {
        collect(y, day);
    }
    if day.weekday() == Weekday::Mon {
        pay_sellers(y, day);
    }
}

fn lend(y: &Year, dd: &str, amount: &str, return_by: Option<&str>, reason: &str) {
    vouchers::save_money_lent(
        &y.w.env.ctx,
        &y.w.owner,
        &MoneyLentInput {
            date: dd.to_owned(),
            person_account_id: Some(y.aslam),
            amount: amount.to_owned(),
            reason: reason.to_owned(),
            return_by: return_by.map(str::to_owned),
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .expect("money lent");
}

fn advance(y: &Year, dd: &str, seller: i64, amount: &str) {
    vouchers::save_advance(
        &y.w.env.ctx,
        &y.w.owner,
        &AdvanceInput {
            confirm_cash_below_zero: true,
            date: dd.to_owned(),
            seller_account_id: seller,
            amount: amount.to_owned(),
            removed: RemovedPaymentFields::default(),
            expected_goods: String::new(),
            narration: String::new(),
        },
    )
    .expect("an advance");
}

/// Saturday: each credit buyer pays his bills that are old enough, as much as his habit says.
fn collect(y: &Year, day: NaiveDate) {
    for b in &y.buyers {
        let Some(after) = b.pay_after_days else {
            continue;
        };
        if b.stops_paying.is_some_and(|s| day > s) {
            continue;
        }
        let bills = vouchers::open_bills(
            &y.w.env.ctx,
            &OpenBillsInput {
                account_id: b.id,
                person_key: None,
            },
        )
        .expect("open bills");
        let due: i64 = bills
            .bills
            .iter()
            .filter(|bill| bill.days >= after)
            .map(|bill| bill.open_paisa.rupees())
            .sum();
        let amount = due * b.pay_percent / 100;
        if amount <= 0 {
            continue;
        }
        vouchers::save_receipt(
            &y.w.env.ctx,
            &y.w.owner,
            &ReceiptInput {
                date: ddmmyyyy(day),
                account_id: b.id,
                person_key: None,
                amount: amount.to_string(),
                removed: RemovedPaymentFields::default(),
                collector_staff_id: None,
                narration: String::new(),
            },
        )
        .unwrap_or_else(|e| panic!("a receipt from {} on {day}: {e:?}", b.key));
    }
}

/// Monday: each seller is paid what is open and old enough, by the app's own working of payables.
fn pay_sellers(y: &Year, day: NaiveDate) {
    for s in &y.sellers {
        let open = vouchers::open_payables(
            &y.w.env.ctx,
            &OpenPayablesInput {
                seller_id: s.id,
                person_key: None,
                include_settled: false,
            },
        )
        .expect("open payables");
        let due: i64 = open
            .postings
            .iter()
            .filter(|p| p.days >= s.pay_after_days)
            .map(|p| p.still_open_paisa.rupees())
            .sum();
        if due <= 0 {
            continue;
        }
        vouchers::save_payment(
            &y.w.env.ctx,
            &y.w.owner,
            &PaymentInput {
                date: ddmmyyyy(day),
                account_id: s.id,
                person_key: None,
                amount: due.to_string(),
                removed: RemovedPaymentFields::default(),
                reason: String::new(),
                narration: String::new(),
                confirm_cash_below_zero: true,
            },
        )
        .unwrap_or_else(|e| panic!("a payment to {} on {day}: {e:?}", s.key));
    }
}

// ---- the garden, the trips, the fruit ----------------------------------------------------------

fn special_events(y: &mut Year, rng: &mut Rng, day: NaiveDate, garden_days: &[NaiveDate]) {
    let ymd = (day.year(), day.month(), day.day());
    let dd = ddmmyyyy(day);
    let [a, b, _] = y.partner_ids;
    match ymd {
        (2026, 3, 12) => make_garden(y, &dd, a, b),
        (2026, 4, 20) => garden_payment(y, &dd, true, "100,000", "قسط"),
        (2026, 5, 5) => garden_payment(y, &dd, false, "12,000", "سپرے"),
        (2026, 6, 10) => garden_payment(y, &dd, true, "100,000", "قسط"),
        (2026, 6, 20) => garden_payment(y, &dd, false, "8,000", "مزدوری"),
        (2026, 7, 15) => {
            gardens::entry_save(
                &y.w.env.ctx,
                &y.w.owner,
                &GardenEntryInput::OtherSale(GardenOtherSaleInput {
                    arrival_id: None,
                    garden_id: Some(y.garden_id),
                    date: dd.clone(),
                    shop: Some("Haji Traders".into()),
                    qty: "30".into(),
                    amount: "120,000".into(),
                    note: None,
                }),
            )
            .expect("a garden sale in another shop");
        }
        (2026, 8, 14) => trip_one(y, &dd),
        (2026, 8, 15) => trip_one_sales(y, rng, day),
        (2026, 8, 17) => {
            let t = trip_one_purchase(y, "saib");
            purchases::write_off(
                &y.w.env.ctx,
                &y.w.owner,
                &PurchaseWriteOffInput {
                    purchase_id: t,
                    qty: "5".into(),
                    reason: "خراب".into(),
                },
            )
            .expect("a write-off");
        }
        (2026, 9, 1) => {
            // Bilal never paid: the Owner marks his old dues as a dead amount.
            let bilal = y.buyers.iter().find(|b| b.key == "bilal").map(|b| b.id);
            if let Some(account_id) = bilal {
                dead::mark(
                    &y.w.env.ctx,
                    &y.w.owner,
                    &DeadMarkInput {
                        account_id,
                        pukhta_person_key: None,
                        entry_date: dd.clone(),
                        reason: DeadReason::Left,
                        note: "left the market".into(),
                    },
                )
                .expect("a dead amount");
            }
        }
        (2026, 10, 22) => trip_two(y, &dd),
        (2026, 10, 23) => trip_two_sale(y, rng, day),
        _ => {}
    }
    if garden_days.contains(&day) {
        garden_sale(y, rng, day);
    }
    // Fruit taken: A takes bananas home on the 18th, the guests' apples come on the 25th.
    if day.day() == 18 {
        fruit_home(y, &dd);
    }
    if day.day() == 25 {
        fruit_guests(y, &dd);
    }
}

fn make_garden(y: &mut Year, dd: &str, a: i64, b: i64) {
    let item = y.items.iter().find(|i| i.key == "aam").map(|i| i.id);
    let row = confirming(|words| {
        gardens::create(
            &y.w.env.ctx,
            &y.w.owner,
            &GardenInput {
                name_ur: "شاہ پور باغ".into(),
                name_en: "Shahpur Garden".into(),
                confirmed_rules_words: words.to_vec(),
                date: dd.to_owned(),
                item_id: item,
                rent: "400,000".into(),
                season_end: None,
                note: None,
                partners: vec![shop(a, "60"), pocket(b, "40")],
                paid_now: Some("150,000".into()),
                paid_now_reason: Some("پہلی قسط".into()),
                parts: Vec::new(),
                confirm_cash_below_zero: true,
            },
        )
    })
    .expect("the garden")
    .row;
    y.garden_id = row.id;
    y.garden_account = row.account.id;
}

fn garden_payment(y: &Year, dd: &str, installment: bool, amount: &str, reason: &str) {
    let payment = GardenPaymentInput {
        garden_id: Some(y.garden_id),
        date: dd.to_owned(),
        amount: amount.to_owned(),
        reason: reason.to_owned(),
        parts: Vec::new(),
        confirm_cash_below_zero: true,
    };
    gardens::entry_save(
        &y.w.env.ctx,
        &y.w.owner,
        &if installment {
            GardenEntryInput::Installment(payment)
        } else {
            GardenEntryInput::Cost(payment)
        },
    )
    .expect("a garden payment");
}

/// One بکری with only the garden's mango lot.
fn garden_sale(y: &mut Year, rng: &mut Rng, day: NaiveDate) {
    let item_ix = y.items.iter().position(|i| i.key == "aam").unwrap_or(0);
    let item = &y.items[item_ix];
    let qty = rng.range(40, 80);
    let mut lot = world::bare_lot(&y.w, "garden", y.garden_account, item.id, &qty.to_string());
    lot.packing_id = Some(item.packing);
    lot.arrival_id = None;
    lot.sales = lines_for(y, rng, day, "garden", item, qty, false);
    let truck = format!("MUL-{}", rng.range(100, 999));
    save_bikri(
        y,
        BikriDraft {
            header: header(day, &truck),
            lots: vec![lot],
        },
    );
}

fn trip_input(date: &str, market: &str, a: i64, b: i64) -> TripInput {
    TripInput {
        date: date.to_owned(),
        market: Some(market.to_owned()),
        sent_person: Some("Aslam".into()),
        note: None,
        partners: vec![shop(a, "50"), pocket(b, "50")],
    }
}

fn buy(
    y: &mut Year,
    trip: i64,
    dd: &str,
    item: &'static str,
    qty: &str,
    rate: &str,
    freight: &str,
) {
    let it = y.items.iter().find(|i| i.key == item).expect("an item");
    let id = purchases::save(
        &y.w.env.ctx,
        &y.w.owner,
        &PurchaseInput {
            other_qty: None,
            shop_name: None,
            sell_date: None,
            trip_id: Some(trip),
            date: dd.to_owned(),
            supplier: Some("Lahore Mandi".into()),
            item_id: Some(it.id),
            packing_id: Some(it.packing),
            qty: qty.to_owned(),
            weight: None,
            rate: rate.to_owned(),
            freight: Some(freight.to_owned()),
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .expect("a trip purchase")
    .id;
    y.purchase_ids.push((trip, item, id));
}

fn trip_one(y: &mut Year, dd: &str) {
    let [a, b, _] = y.partner_ids;
    let t = trips::create(&y.w.env.ctx, &y.w.owner, &trip_input(dd, "Lahore", a, b))
        .expect("trip one")
        .id;
    y.trip_one = t;
    buy(y, t, dd, "mausami", "60", "2,100", "3,000");
    buy(y, t, dd, "saib", "40", "3,800", "2,000");
    trips::cost_save(
        &y.w.env.ctx,
        &y.w.owner,
        &TripCostInput {
            trip_id: Some(t),
            date: dd.to_owned(),
            amount: "4,000".into(),
            reason: "ٹرک".into(),
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .expect("a trip cost");
}

/// The purchase of this item on trip one, by the app's own list.
fn trip_one_purchase(y: &Year, item: &str) -> i64 {
    purchase_of(y, y.trip_one, item)
}

fn purchase_of(y: &Year, trip: i64, item: &str) -> i64 {
    y.purchase_ids
        .iter()
        .find(|(t, k, _)| *t == trip && *k == item)
        .map(|(_, _, id)| *id)
        .expect("the purchase")
}

fn own_lot(
    y: &Year,
    rng: &mut Rng,
    day: NaiveDate,
    purchase_id: i64,
    item: &ItemInfo,
    qty: i64,
    rate: i64,
) -> LotInput {
    let buyers: Vec<&BuyerInfo> = y
        .buyers
        .iter()
        .filter(|b| b.from <= day && day <= b.until)
        .collect();
    let first = rng.pick(&buyers).id;
    let second = rng.pick(&buyers).id;
    let q1 = qty / 2;
    let mut lot = LotInput {
        seller_account_id: Some(y.own_goods),
        purchase_id: Some(purchase_id),
        item_id: None,
        packing_id: None,
        adjust_advance: false,
        ..world::bare_lot(&y.w, "own", y.w.rahim, item.id, &qty.to_string())
    };
    lot.sales = vec![
        world::sale("own-s0", first, &q1.to_string(), &rate.to_string()),
        world::sale("own-s1", second, &(qty - q1).to_string(), &rate.to_string()),
    ];
    lot
}

fn trip_one_sales(y: &mut Year, rng: &mut Rng, day: NaiveDate) {
    // All 60 mosambi in one بکری, and 25 of the apples; 10 apples go to another shop; 5 spoil.
    let m = trip_one_purchase(y, "mausami");
    let s = trip_one_purchase(y, "saib");
    let mausami = y.items.iter().position(|i| i.key == "mausami").unwrap_or(0);
    let saib = y.items.iter().position(|i| i.key == "saib").unwrap_or(0);
    let lot_m = own_lot(y, rng, day, m, &y.items[mausami], 60, 2_600);
    let lot_s = own_lot(y, rng, day, s, &y.items[saib], 25, 4_300);
    let truck = format!("LHR-{}", rng.range(100, 999));
    save_bikri(
        y,
        BikriDraft {
            header: header(day, &truck),
            lots: vec![lot_m, lot_s],
        },
    );
    trips::other_sale_save(
        &y.w.env.ctx,
        &y.w.owner,
        &OtherSaleInput {
            trip_id: Some(y.trip_one),
            date: ddmmyyyy(day),
            shop: Some("Rafiq Fruit".into()),
            purchase_id: Some(s),
            qty: "10".into(),
            amount: "41,000".into(),
            note: None,
        },
    )
    .expect("a trip sale in another shop");
}

fn trip_two(y: &mut Year, dd: &str) {
    let [a, b, _] = y.partner_ids;
    let t = trips::create(
        &y.w.env.ctx,
        &y.w.owner,
        &trip_input(dd, "Faisalabad", a, b),
    )
    .expect("trip two")
    .id;
    y.trip_two = t;
    buy(y, t, dd, "mausami", "50", "2,300", "2,500");
}

fn trip_two_sale(y: &mut Year, rng: &mut Rng, day: NaiveDate) {
    let m = purchase_of(y, y.trip_two, "mausami");
    let mausami = y.items.iter().position(|i| i.key == "mausami").unwrap_or(0);
    let lot = own_lot(y, rng, day, m, &y.items[mausami], 20, 2_700);
    let truck = format!("FSD-{}", rng.range(100, 999));
    save_bikri(
        y,
        BikriDraft {
            header: header(day, &truck),
            lots: vec![lot],
        },
    );
}

fn fruit_home(y: &Year, dd: &str) {
    let [a, _, _] = y.partner_ids;
    let banana = y.w.banana;
    let lots = fruit::lots(
        &y.w.env.ctx,
        &FruitLotsInput {
            search: None,
            offset: 0,
            limit: 100,
        },
    )
    .expect("lots with a balance");
    let Some(lot) = lots
        .rows
        .iter()
        .find(|l| l.item.id == banana && l.balance >= 2)
    else {
        return;
    };
    fruit::save(
        &y.w.env.ctx,
        &y.w.owner,
        &FruitInput {
            for_guests: false,
            partner_id: Some(a),
            date: dd.to_owned(),
            source: FruitSource::Shop,
            lot_id: Some(lot.lot_id),
            agent_account_id: None,
            item_id: None,
            packing_id: None,
            qty: "2".into(),
            weight: None,
            rate: "1,800".into(),
            note: None,
            editing_id: None,
        },
    )
    .expect("fruit taken home");
}

fn fruit_guests(y: &Year, dd: &str) {
    fruit::save(
        &y.w.env.ctx,
        &y.w.owner,
        &FruitInput {
            for_guests: true,
            partner_id: None,
            date: dd.to_owned(),
            source: FruitSource::OtherShop,
            lot_id: None,
            agent_account_id: Some(y.haji_nazeer),
            item_id: Some(y.w.apple),
            packing_id: Some(y.w.box_),
            qty: "3".into(),
            weight: None,
            rate: "4,000".into(),
            note: None,
            editing_id: None,
        },
    )
    .expect("fruit for the guests");
}

fn end_of_year(y: &Year) {
    // The days up to the end of last month are closed, as the Owner does.
    set_today(&y.w.env, TODAY.0, TODAY.1, TODAY.2);
    day::close(
        &y.w.env.ctx,
        &y.w.owner,
        &DayCloseInput {
            up_to: "31-10-2026".into(),
        },
    )
    .expect("close the days up to 31-10-2026");
}
