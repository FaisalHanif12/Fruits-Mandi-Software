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
    /// Buyers who are inactive NOW (an account made inactive during the year). The ordinary days never need
    /// this (a buyer buys only while he is active); the vegetable pass sells on past days and must skip them.
    pub inactive_buyers: Vec<i64>,
    /// Give the lines of the last days a quality (plan 58 T4, [`grade_lines`]). `false` builds the same books with no
    /// quality at all, to prove that a quality changes no amount.
    pub grades: bool,
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
    build_with(true)
}

/// The same year with no quality on any line (plan 58 T4). Every amount of the books must be the same as in
/// [`build`]: the quality is shown on prints and ledgers and changes no figure (PRD FR-12.42).
pub fn build_without_grades() -> Year {
    build_with(false)
}

fn build_with(grades: bool) -> Year {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).expect("the Owner");
    let start = date(START);
    set_today(&env, START.0, START.1, START.2);
    let mut year = people_and_things(dir, env, owner);
    year.grades = grades;
    opening_position(&year, start);
    let mut rng = Rng::new(20_261_105);
    run_days(&mut year, &mut rng);
    vegetable_sales(&mut year);
    d7_last_rate_cases(&mut year);
    d7_richer_cases(&mut year);
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

/// What a party's card holds besides his name: phones and an address. These are the values the assistant must
/// never give (PRD FR-7.83); the privacy checks read them from the books.
type Contact<'a> = (&'a str, &'a str, &'a str);

/// An account (as `world::add_account` makes it) with a collector, a staff member who collects from him, and
/// a contact (phone 1, phone 2, address).
fn account_of(
    env: &mandi_services::testing::TestEnv,
    owner: &auth::Actor,
    kind: AccountType,
    (ur, en): (&str, &str),
    (credit_allowed, credit_limit): (bool, &str),
    collector: Option<i64>,
    (phone1, phone2, address): Contact<'_>,
) -> i64 {
    let mut input = mandi_services::core::accounts::AccountInput {
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
        phone1: phone1.into(),
        phone2: phone2.into(),
        address: address.into(),
        linked_account_id: None,
        opening_date: String::new(),
        opening_dr: String::new(),
        opening_cr: String::new(),
        opening_is_advance: false,
        credit_allowed,
        credit_limit: credit_limit.into(),
        default_marka_id: None,
        collector_staff_id: collector,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    loop {
        match mandi_services::accounts::save(&env.ctx, owner, &input) {
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

fn people_and_things(dir: TempDir, env: TestEnv, owner: auth::Actor) -> Year {
    // The staff of the shop: a munshi, a collector and a labourer, for the lists of `find` (plan 55 R5).
    // The collector, Rasheed, collects from two buyers (Akram Toll Tax and Faisal): the collector filters
    // of the reports (plan 55 P4) have something to find.
    let mut rasheed: Option<i64> = None;
    for (name_ur, name_en, job) in [
        ("سلطان", "Sultan", "munshi"),
        ("رشید", "Rasheed", "collector"),
        ("نسیم", "Naseem", "labour"),
    ] {
        let row = confirming(|words| {
            mandi_services::masters::save(
                &env.ctx,
                &owner,
                &mandi_services::core::masters::MasterInput::Staff(
                    mandi_services::core::masters::StaffInput {
                        id: None,
                        version: None,
                        name_ur: name_ur.into(),
                        name_en: name_en.into(),
                        code: String::new(),
                        job: job.into(),
                        phone: if job == "collector" {
                            "0306-6666666".into()
                        } else {
                            String::new()
                        },
                        confirmed_rules_words: words.to_vec(),
                    },
                ),
            )
        })
        .expect("a staff member");
        if job == "collector" {
            rasheed = Some(row.id());
        }
    }
    // A few parties have a phone and an address (the values the assistant must never give).
    let contact_of = |en: &str| -> Contact<'static> {
        match en {
            "Rahim" => ("0301-1111111", "", "Gali 5, Sabzi Mandi, Kot Radha Kishan"),
            "Karim" => (
                "0302-2222222",
                "",
                "Plot 9, New Sabzi Mandi, Kot Radha Kishan",
            ),
            "Akram Toll Tax" => ("0303-3333333", "0304-4444444", "Shop 21, Toll Plaza Road"),
            "Faisal" => ("0305-5555555", "", "House 7, Mohalla Islam Pura"),
            _ => ("", "", ""),
        }
    };
    let seller = |ur: &str, en: &str| {
        account_of(
            &env,
            &owner,
            AccountType::Seller,
            (ur, en),
            (true, ""),
            None,
            contact_of(en),
        )
    };
    let buyer = |ur: &str, en: &str, credit: bool, limit: &str| {
        let collector = if matches!(en, "Akram Toll Tax" | "Faisal") {
            rasheed
        } else {
            None
        };
        account_of(
            &env,
            &owner,
            AccountType::Buyer,
            (ur, en),
            (credit, limit),
            collector,
            contact_of(en),
        )
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
    // Two stores, three marks (each with the seller who owns it) and three standard narrations, for the lists
    // of `find` (plan 55 R5). The mark text and the narration text are never given to the model (FR-7.83).
    use mandi_services::core::masters::{MarkaInput, MasterInput, NarrationInput, StoreInput};
    for (name_ur, name_en) in [("مین سٹور", "Main Store"), ("ٹھنڈا سٹور", "Cold Store")]
    {
        confirming(|words| {
            mandi_services::masters::save(
                &env.ctx,
                &owner,
                &MasterInput::Store(StoreInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: name_ur.into(),
                    name_en: name_en.into(),
                    confirmed_rules_words: words.to_vec(),
                }),
            )
        })
        .expect("a store");
    }
    for (mark_text, owner_account_id) in [("XQZ-77", rahim), ("AB-12", karim), ("KLM-5", naseer)] {
        mandi_services::masters::save(
            &env.ctx,
            &owner,
            &MasterInput::Marka(MarkaInput {
                id: None,
                version: None,
                code: String::new(),
                mark_text: mark_text.into(),
                owner_account_id: Some(owner_account_id),
            }),
        )
        .expect("a mark");
    }
    for (text_ur, text_en, used_in) in [
        (
            "سلطان نے بجلی کا بل ادا کیا 101",
            "Sultan settled the electricity bill ref ZQ-101",
            vec!["shop_expense"],
        ),
        (
            "نقد وصولی نمبر 102",
            "Cash receipt ref ZQ-102",
            vec!["receipt"],
        ),
        (
            "پیشگی ادا کی 103",
            "Advance paid ref ZQ-103",
            vec!["advance"],
        ),
    ] {
        mandi_services::masters::save(
            &env.ctx,
            &owner,
            &MasterInput::Narration(NarrationInput {
                id: None,
                version: None,
                text_ur: text_ur.into(),
                text_en: text_en.into(),
                used_in: used_in.into_iter().map(str::to_owned).collect(),
            }),
        )
        .expect("a narration");
    }

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
    // The other vegetables of the mandi (Umar, 06-11-2026): sold only by `vegetable_sales`, never by the
    // ordinary days, so the days generated before are the same as they were.
    let mooli = world::add_item(&env, &owner, "مولی", "Radish", "vegetable", "unit");
    let tamatar = world::add_item(&env, &owner, "ٹماٹر", "Tomato", "vegetable", "unit");
    let piyaz = world::add_item(&env, &owner, "پیاز", "Onion", "vegetable", "unit");
    let saag = world::add_item(&env, &owner, "ساگ", "Saag", "vegetable", "unit");
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
        ItemInfo {
            key: "mooli",
            id: mooli,
            packing: bag,
            base_rate: 700,
            months: &[1, 2, 3, 10, 11, 12],
            qty: (60, 160),
        },
        ItemInfo {
            key: "tamatar",
            id: tamatar,
            packing: box_,
            base_rate: 1_500,
            months: all,
            qty: (40, 120),
        },
        ItemInfo {
            key: "piyaz",
            id: piyaz,
            packing: bag,
            base_rate: 1_800,
            months: all,
            qty: (60, 180),
        },
        ItemInfo {
            key: "saag",
            id: saag,
            packing: box_,
            base_rate: 500,
            months: &[1, 2, 3, 11, 12],
            qty: (50, 140),
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
    let chukta_khata = world::id_of(&env, "0030");
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
        chukta_khata,
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
        inactive_buyers: Vec::new(),
        grades: true,
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
    let truck = draft.header.truck_no.clone();
    let saved = bikri::save(
        &y.w.env.ctx,
        &y.w.owner,
        &BikriSaveInput {
            draft,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_or_else(|e| panic!("a بکری of the year: {e:?}"));
    // A new بکری stores no truck (plan 71 took it off the header). The year stands for a shop with old data
    // (plan 72 P3: an old بکری keeps its truck), so the truck is put on the saved row by SQL, as the old
    // rows hold it. The trucks of the oracle facts and of the question bank follow.
    if let Some(truck) = truck {
        y.w.env
            .ctx
            .db()
            .write(|tx| -> Result<(), mandi_services::ServiceError> {
                tx.execute(
                    "UPDATE bikri_entries SET truck_no = ?1 WHERE id = ?2",
                    mandi_services::db::rusqlite::params![truck, saved.bikri_id],
                )?;
                Ok(())
            })
            .unwrap_or_else(|e| panic!("the truck of an old بکری: {e:?}"));
    }
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
    if y.grades {
        grade_lines(&mut lot.sales, day);
    }
    let _ = month;
    lot
}

/// The quality of the lines of one lot (plan 58 T4): for the lots of the last 10 days, and of every third day
/// before them, the line with the highest rate is `first`, the next `second`, the rest `third`; a tie goes by
/// line order. It uses no random number, so the books of [`build_without_grades`] are the same in every figure.
fn grade_lines(lines: &mut [SaleInput], day: NaiveDate) {
    let age = (date(TODAY) - day).num_days();
    if !(age < 10 || age % 3 == 0) {
        return;
    }
    let rate = |line: &SaleInput| line.seller_rate.parse::<i64>().unwrap_or(0);
    let mut order: Vec<(i64, usize)> = lines
        .iter()
        .enumerate()
        .map(|(at, line)| (rate(line), at))
        .collect();
    // The highest rate first; the same rate: the earlier line first.
    order.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let words = ["first", "second", "third"];
    let mut grade_of = vec!["third"; lines.len()];
    for (rank, (_, at)) in order.iter().enumerate() {
        if let (Some(slot), Some(word)) = (grade_of.get_mut(*at), words.get(rank)) {
            *slot = word;
        }
    }
    for (line, word) in lines.iter_mut().zip(grade_of) {
        line.grade = Some(word.to_owned());
    }
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
        .filter(|b| b.from <= day && day <= b.until && !y.inactive_buyers.contains(&b.id))
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

/// Vegetables across the whole year (Umar, 06-11-2026): the ordinary days sell one vegetable only (potato), so
/// this pass adds sales of mooli, aloo, tamatar, piyaz and saag, two بکری a month for each one in its season,
/// from the sellers in turn. It runs after the ordinary days with a random source of its own (so the days
/// before it are the same as they were) and before the day close; its dates are in the past and open. A
/// vegetable carries no لاگا (PRD 4.2: the لاگا rule is for the fruit group).
fn vegetable_sales(y: &mut Year) {
    let mut rng = Rng::new(20_261_106);
    let (start, today) = (date(START), date(TODAY));
    let veg: Vec<usize> = y
        .items
        .iter()
        .enumerate()
        .filter(|(_, it)| matches!(it.key, "mooli" | "aloo" | "tamatar" | "piyaz" | "saag"))
        .map(|(ix, _)| ix)
        .collect();
    // A buyer made inactive since cannot take a sale, even on a day when he bought (the account is checked now).
    y.inactive_buyers =
        y.w.env
            .ctx
            .db()
            .read(|c| {
                let mut st = c.prepare("SELECT id FROM accounts WHERE is_active = 0")?;
                let rows = st.query_map([], |r| r.get::<_, i64>(0))?;
                rows.collect::<Result<Vec<_>, mandi_services::db::rusqlite::Error>>()
            })
            .unwrap_or_default();
    let mut turn = 0usize;
    let (mut year, mut month) = (start.year(), start.month());
    while (year, month) <= (today.year(), today.month()) {
        for dom in [6u32, 20] {
            for (nth, &ix) in veg.iter().enumerate() {
                if !seasonal(&y.items[ix], month) {
                    continue;
                }
                // Different items on different days; never a Sunday.
                let spread = u32::try_from(nth).unwrap_or(0);
                let Some(mut day) = NaiveDate::from_ymd_opt(year, month, dom + spread) else {
                    continue;
                };
                if day.weekday() == Weekday::Sun {
                    day += Duration::days(1);
                }
                if day < start || day > today {
                    continue;
                }
                let seller_id = y.sellers[turn % y.sellers.len()].id;
                turn += 1;
                let item = &y.items[ix];
                let lot = lot_for(y, &mut rng, day, "lot-v", seller_id, item);
                let truck = format!("VEG-{}", rng.range(100, 999));
                save_bikri(
                    y,
                    BikriDraft {
                        header: header(day, &truck),
                        lots: vec![lot],
                    },
                );
            }
        }
        if month == 12 {
            (year, month) = (year + 1, 1);
        } else {
            month += 1;
        }
    }
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
        let bills = vouchers::open_bills(&y.w.env.ctx, &OpenBillsInput { account_id: b.id })
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

/// The newest sale line (plan 59 Section 11, lead ruling of 07-11-2026): a line is ordered by its own posting date,
/// then the بکری number, then its line number, never by the row id (an edit keeps the line number and writes a new
/// row; a later sale counts on its own date). Grapes are out of season in November, so these are the only grapes
/// lines of the week, and the newest one is known. All the buyers of these lines are CASH buyers (Rizwan Butt, Shahid
/// Raza and the cash buyer), so no buyer owes more and the dues of the books (R20, the tiles) are what they were
/// without them:
///
/// - `OLD` is a بکری of 03-11-2026 (number first) with 8 box left. On 05-11-2026 a LATER SALE sells them at 4,600:
///   a line of the oldest بکری with a newer row id than the others of that day.
/// - `E` is a بکری of 05-11-2026 (a lower number than A) whose lines (5,000 and 5,200) are EDITED last: its rows
///   have the newest ids of all.
/// - `A` is a بکری of 05-11-2026, the highest number, with FOUR lines of grapes (5,100, 4,900, 5,300, 5,000), edited
///   first (an edit gives every line a new row and a new, higher line number, in the same order). The newest line
///   by the rule is the last line of A at 5,000.
///
/// The row id would say 5,200 (E's last line), so a read that sorts by id shows a wrong rate. It runs before the
/// commission rules of [`end_of_year`], so these lines follow the default rates like every line of the year.
fn d7_last_rate_cases(y: &mut Year) {
    use mandi_services::core::bikri::{
        EditOpenInput, EditSaveInput, LaterSaleSaveInput, StockListInput,
    };
    set_today(&y.w.env, TODAY.0, TODAY.1, TODAY.2);
    let grapes = y
        .items
        .iter()
        .find(|it| it.key == "angoor")
        .map(|it| (it.id, it.packing))
        .expect("the grapes");
    let buyer = |key: &str| -> i64 {
        y.buyers
            .iter()
            .find(|b| b.key == key)
            .map(|b| b.id)
            .unwrap_or_else(|| panic!("no buyer {key}"))
    };
    // Cash buyers only: no line of these cases adds to what any buyer owes, so the dues of the books (R20, the tiles)
    // stay what they were without them.
    let (rizwan, shahid, naqdi) = (buyer("rizwan"), buyer("shahid"), y.w.naqdi);
    let karim = y.w.karim;
    let draft = |day: (i32, u32, u32), truck: &str, qty: &str, lines: &[(i64, &str, &str)]| {
        let mut lot = world::lot(&y.w, "lot-0", karim, grapes.0, qty);
        lot.packing_id = Some(grapes.1);
        lot.sales = lines
            .iter()
            .enumerate()
            .map(|(i, (who, qty, rate))| world::sale(&format!("s{i}"), *who, qty, rate))
            .collect();
        BikriDraft {
            header: header(date(day), truck),
            lots: vec![lot],
        }
    };
    let old_draft = draft(
        (2026, 11, 3),
        "LES-7001",
        "30",
        &[(rizwan, "12", "4800"), (shahid, "10", "4700")],
    );
    let e_draft = draft(
        (2026, 11, 5),
        "LES-7002",
        "20",
        &[(shahid, "8", "5000"), (rizwan, "12", "5200")],
    );
    let a_draft = draft(
        (2026, 11, 5),
        "LES-7003",
        "24",
        &[
            (rizwan, "6", "5100"),
            (shahid, "6", "4900"),
            (naqdi, "6", "5300"),
            (rizwan, "6", "5000"),
        ],
    );
    let old = save_bikri(y, old_draft);
    let e = save_bikri(y, e_draft);
    let a = save_bikri(y, a_draft);
    // An edit of a بکری of today: only the rate of its first line changes. It writes new rows for every line.
    let edit_first_line = |saved: &BikriSaved, rate: i64| {
        let opened = bikri::open_for_edit(
            &y.w.env.ctx,
            &EditOpenInput {
                bikri_id: saved.bikri_id,
                entry_id: saved.entry_id,
            },
        )
        .expect("open the بکری for an edit");
        let mut draft = opened.draft.clone();
        draft.lots[0].sales[0].seller_rate = rate.to_string();
        bikri::edit_save(
            &y.w.env.ctx,
            &y.w.owner,
            &EditSaveInput {
                entry_id: opened.entry_id,
                version: opened.version,
                draft,
                reason: Some("ریٹ کی درستی".into()),
                confirm_cash_below_zero: true,
            },
        )
        .expect("the edit of the first line");
    };
    // 1. A, the highest number, is edited first.
    edit_first_line(&a, 5_150);
    // 2. The later sale of the older بکری, on its own day (05-11-2026).
    let stock = bikri::stock_list(
        &y.w.env.ctx,
        &StockListInput {
            date: Some(ddmmyyyy(date((2026, 11, 3)))),
            item_id: Some(grapes.0),
            ..StockListInput::default()
        },
    )
    .expect("the stock of grapes");
    let lot_id = stock
        .rows
        .iter()
        .find(|r| r.bikri_id == old.bikri_id)
        .map(|r| r.lot_id)
        .expect("the grapes of the older بکری are left to sell");
    let mut line = world::sale("later", rizwan, "8", "4600");
    line.sale_id = None;
    bikri::later_sale_save(
        &y.w.env.ctx,
        &y.w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: ddmmyyyy(date(TODAY)),
            sales: vec![line],
            confirm_cash_below_zero: true,
        },
    )
    .expect("the later sale of the older بکری");
    // 3. E, a lower number, is edited LAST: its rows have the newest ids of the day.
    edit_first_line(&e, 5_050);
}

/// More of the shop for the D7 bank (plan 59 B2, lead, 07-11-2026). Added at the very end, ADD ONLY (no saved row of the
/// year is changed), through the app's own save functions, on days of the past that are still open:
///
/// - a city (Kasur) on three NEW accounts: two credit buyers (Naveed Traders, Rafique Brothers) and a چکتا party's own
///   account (Nadir, FR-13.11), each with a balance, so the dues by city are not empty;
/// - a day with cash AND credit lines of ONE buyer (14-10-2026): Naveed Traders takes 10 banana for cash and 12 apple on
///   credit, so "how much on the spot and how much stayed on credit" has both parts;
/// - a چکتا buyer of the shared چکتا کھاتہ (Aamir, with his phone) who pays in full on the spot, on 21-10-2026;
/// - a separate khata (الگ کھاتہ) with three entries;
/// - a sale line with ریٹ مختلف (22-10-2026: the buyer pays 4,350, the seller gets 4,300, and it is the newest apple
///   line of that day, so "the rate" must be the seller's 4,300);
/// - a LATER SALE of an older بکری's leftover (mosambi, bikri of 16-10-2026) made on 19-10-2026, a day that also has a
///   newer بکری of the same item saved before it: the newest mosambi line of 19-10 is the line of the newer بکری
///   (2,550), while the row id says the later sale (2,400).
///
/// The sellers have no advance, so no advance is matched. The new buyers are put in `y.buyers` so the oracle reads them.
fn d7_richer_cases(y: &mut Year) {
    use mandi_services::core::bikri::{ChuktaInput, PaymentType};
    use mandi_services::core::bikri::{LaterSaleSaveInput, StockListInput};
    use mandi_services::core::khata::{KhataCreateInput, KhataOutInput};
    use mandi_services::core::masters::{CityInput, MasterInput};
    set_today(&y.w.env, TODAY.0, TODAY.1, TODAY.2);
    let item = |key: &str| -> (i64, i64) {
        y.items
            .iter()
            .find(|it| it.key == key)
            .map(|it| (it.id, it.packing))
            .unwrap_or_else(|| panic!("no item {key}"))
    };
    let seller = |key: &str| -> i64 {
        y.sellers
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.id)
            .unwrap_or_else(|| panic!("no seller {key}"))
    };
    let (banana, apple, mosambi) = (item("kela"), item("saib"), item("mausami"));
    let (karim, bashir, saleem, jameel) = (
        seller("karim"),
        seller("bashir"),
        seller("saleem"),
        seller("jameel"),
    );

    // 1. A city and three new accounts in it.
    let city = confirming(|words| {
        mandi_services::masters::save(
            &y.w.env.ctx,
            &y.w.owner,
            &MasterInput::City(CityInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: "قصور".into(),
                name_en: "Kasur".into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    })
    .expect("the city of Kasur")
    .id();
    let new_account = |names: (&str, &str)| -> i64 {
        let mut input = mandi_services::core::accounts::AccountInput {
            id: None,
            version: None,
            account_type: AccountType::Buyer,
            control_id: None,
            code: String::new(),
            name_ur: names.0.into(),
            name_en: names.1.into(),
            confirmed_rules_words: Vec::new(),
            father_or_firm_ur: String::new(),
            father_or_firm_en: String::new(),
            city_id: Some(city),
            adda_id: None,
            phone1: String::new(),
            phone2: String::new(),
            address: String::new(),
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
            note: String::new(),
            tag: String::new(),
        };
        confirming(|words| {
            input.confirmed_rules_words = words.to_vec();
            mandi_services::accounts::save(&y.w.env.ctx, &y.w.owner, &input)
        })
        .unwrap_or_else(|e| panic!("the account {}: {e:?}", names.1))
        .id
    };
    let naveed = new_account(("نوید ٹریڈرز", "Naveed Traders"));
    let rafique = new_account(("رفیق برادرز", "Rafique Brothers"));
    let nadir = new_account(("نادر", "Nadir"));
    for (key, id) in [("naveed", naveed), ("rafique", rafique), ("nadir", nadir)] {
        y.buyers.push(BuyerInfo {
            key,
            id,
            credit: true,
            from: date((2026, 10, 14)),
            until: date(TODAY),
            pay_after_days: None,
            pay_percent: 0,
            stops_paying: None,
        });
    }

    // 2. A separate khata with three entries: 20,000 and 3,000 paid out, 5,000 received.
    let khata = mandi_services::khata::create(
        &y.w.env.ctx,
        &y.w.owner,
        &KhataCreateInput {
            name_ur: "خاندانی کھاتہ".into(),
            name_en: "Family Khata".into(),
            note: None,
            confirmed_rules_words: vec!["خاندانی".into(), "Family".into()],
        },
    )
    .unwrap_or_else(|e| panic!("the khata: {e:?}"))
    .account
    .id;
    let out = |date: &str, amount: &str, reason: &str| {
        vouchers::save_khata_out(
            &y.w.env.ctx,
            &y.w.owner,
            &KhataOutInput {
                date: date.into(),
                khata_account_id: Some(khata),
                amount: amount.into(),
                reason: reason.into(),
                narration: None,
                confirm_cash_below_zero: true,
            },
        )
        .unwrap_or_else(|e| panic!("a khata entry: {e:?}"));
    };
    out("08-10-2026", "20,000", "خاندان کا خرچ");
    vouchers::save_khata_in(
        &y.w.env.ctx,
        &y.w.owner,
        &mandi_services::core::khata::KhataInInput {
            date: "15-10-2026".into(),
            khata_account_id: Some(khata),
            amount: "5,000".into(),
            reason: "رقم واپس آئی".into(),
            narration: None,
            confirm_cash_below_zero: true,
        },
    )
    .unwrap_or_else(|e| panic!("a khata entry: {e:?}"));
    out("27-10-2026", "3,000", "خاندان کا خرچ");

    // 3. The drafts. A line to a cash buyer or a چکتا buyer is paid in full by itself (the backend sets
    // it, PRD FR-12.44); a credit line has nothing paid on the spot (`paid` empty), so `pay` only says
    // what the line is for the reader.
    let line = |key: &str, buyer: i64, qty: &str, rate: &str, _pay: PaymentType| {
        world::sale(key, buyer, qty, rate)
    };
    let lot = |w: &world::World,
               key: &str,
               seller: i64,
               it: (i64, i64),
               qty: &str,
               lines: Vec<SaleInput>| {
        let mut lot = world::lot(w, key, seller, it.0, qty);
        lot.packing_id = Some(it.1);
        lot.sales = lines;
        lot
    };
    // Naveed Traders pays his banana line in full on the spot (PRD FR-12.44): 10 × 1,900 = 19,000 + 6%
    // buyer commission 1,140 + لاگا 80 = 20,220. He may take credit, so the backend does not fill it:
    // it is typed, as the munshi would. His apple line stays on credit, so he has BOTH on this day.
    let mut naveed_cash = line("c0", naveed, "10", "1900", PaymentType::Cash);
    naveed_cash.paid = Some("20,220".into());
    let day14 = BikriDraft {
        header: header(date((2026, 10, 14)), "LES-7101"),
        lots: vec![
            lot(
                &y.w,
                "lot-0",
                karim,
                banana,
                "30",
                vec![
                    naveed_cash,
                    line("c1", rafique, "20", "1900", PaymentType::Credit),
                ],
            ),
            lot(
                &y.w,
                "lot-1",
                bashir,
                apple,
                "20",
                vec![
                    line("c2", naveed, "12", "4300", PaymentType::Credit),
                    line("c3", rafique, "8", "4300", PaymentType::Credit),
                ],
            ),
        ],
    };
    // A چکتا buyer always pays the whole bill on the spot (PRD FR-13.5), so his line is cash.
    let mut aamir = line("p0", y.w.chukta_khata, "8", "4300", PaymentType::Cash);
    aamir.chukta = Some(ChuktaInput {
        name_ur: "عامر".into(),
        name_en: "Aamir".into(),
        phone: "03001234567".into(),
    });
    let day21 = BikriDraft {
        header: header(date((2026, 10, 21)), "LES-7102"),
        lots: vec![lot(
            &y.w,
            "lot-0",
            bashir,
            apple,
            "24",
            vec![
                aamir,
                line("p1", nadir, "6", "4300", PaymentType::Credit),
                line("p2", rafique, "10", "4300", PaymentType::Credit),
            ],
        )],
    };
    // The line with ریٹ مختلف is the LAST apple line of 22-10-2026.
    let mut different = line("r1", rafique, "12", "4300", PaymentType::Credit);
    different.rate_differs = true;
    different.buyer_rate = Some("4350".into());
    let day22 = BikriDraft {
        header: header(date((2026, 10, 22)), "LES-7103"),
        lots: vec![lot(
            &y.w,
            "lot-0",
            karim,
            apple,
            "20",
            vec![
                line("r0", naveed, "8", "4250", PaymentType::Credit),
                different,
            ],
        )],
    };
    let old16 = BikriDraft {
        header: header(date((2026, 10, 16)), "LES-7104"),
        lots: vec![lot(
            &y.w,
            "lot-0",
            saleem,
            mosambi,
            "40",
            vec![line("o0", naveed, "20", "2500", PaymentType::Credit)],
        )],
    };
    let new19 = BikriDraft {
        header: header(date((2026, 10, 19)), "LES-7105"),
        lots: vec![lot(
            &y.w,
            "lot-0",
            jameel,
            mosambi,
            "30",
            vec![
                line("n0", naveed, "10", "2600", PaymentType::Credit),
                line("n1", rafique, "10", "2650", PaymentType::Credit),
                line("n2", naveed, "10", "2550", PaymentType::Credit),
            ],
        )],
    };
    save_bikri(y, day14);
    save_bikri(y, day21);
    save_bikri(y, day22);
    let old = save_bikri(y, old16);
    save_bikri(y, new19);
    // The later sale of the older بکری's 20 left, on the day of the newer one, saved AFTER it.
    let stock = bikri::stock_list(
        &y.w.env.ctx,
        &StockListInput {
            date: Some(ddmmyyyy(date((2026, 10, 16)))),
            item_id: Some(mosambi.0),
            ..StockListInput::default()
        },
    )
    .expect("the stock of mosambi");
    let lot_id = stock
        .rows
        .iter()
        .find(|r| r.bikri_id == old.bikri_id)
        .map(|r| r.lot_id)
        .expect("the mosambi of the older بکری are left to sell");
    let mut later = world::sale("later", rafique, "20", "2400");
    later.sale_id = None;
    bikri::later_sale_save(
        &y.w.env.ctx,
        &y.w.owner,
        &LaterSaleSaveInput {
            lot_id,
            date: ddmmyyyy(date((2026, 10, 19))),
            sales: vec![later],
            confirm_cash_below_zero: true,
        },
    )
    .expect("the later sale of the older mosambi");
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
    // The commission rules of PRD 11.3 (the worked examples), at the very end and dated today, so no
    // saved line changes: only what is sold from today on follows them. Seller commission:
    //   Rahim + banana 5% (party and item), Rahim any item 3% (party), Karim any item 4% (party),
    //   apple, any seller, 30 for each نگ (item). The rest follows the defaults and the group rule.
    use mandi_services::core::rules::{ChargeMethod, ChargeSide, ChargeType, RuleInput};
    let today = ddmmyyyy(date(TODAY));
    let rule =
        |account_id: Option<i64>, item_id: Option<i64>, method: ChargeMethod, value: &str| {
            mandi_services::rules::save(
                &y.w.env.ctx,
                &y.w.owner,
                &RuleInput {
                    id: None,
                    version: None,
                    side: ChargeSide::Seller,
                    charge_type: ChargeType::Commission,
                    account_id,
                    item_id,
                    item_group: None,
                    method,
                    value: value.into(),
                    from_date: today.clone(),
                    to_date: String::new(),
                    is_active: true,
                    note: String::new(),
                },
            )
            .expect("a commission rule");
        };
    rule(
        Some(y.w.rahim),
        Some(y.w.banana),
        ChargeMethod::Percent,
        "5",
    );
    rule(Some(y.w.rahim), None, ChargeMethod::Percent, "3");
    rule(Some(y.w.karim), None, ChargeMethod::Percent, "4");
    rule(None, Some(y.w.apple), ChargeMethod::PerUnit, "30");
}
