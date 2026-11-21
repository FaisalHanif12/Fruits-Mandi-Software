//! The world one random run lives in: the worked day's parties and partners, an expense head, three
//! khatas (no link, shop link, partner link), two loan people (shop link, partner link), the other
//! agent's shop, one market trip with a purchase, one garden. Plus what the run has saved so far, so
//! a step can pick an earlier entry to edit or cancel.

use std::collections::BTreeSet;

use mandi_services::core::accounts::{AccountInput, AccountType};
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::gardens::GardenInput;
use mandi_services::core::khata::{KhataCreateInput, KhataCreateRequest};
use mandi_services::core::lent::{LentPersonCreateInput, LentPersonCreateRequest};
use mandi_services::core::links::{Link, LinkKind, PersonType};
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput};
use mandi_services::core::purchases::PurchaseInput;
use mandi_services::core::trips::TripInput;
use mandi_services::core::ventures::{MoneySource, VenturePartnerInput};
use mandi_services::{ServiceError, accounts, gardens, khata, lent, masters, purchases, trips};

use crate::common::set_today;
use crate::support::{self, World};

/// The random part of a run's world.
#[derive(Debug, Clone)]
pub struct Setup {
    /// Weights of the partners in the trip and in the garden (three each).
    pub trip_weights: [u32; 3],
    pub trip_pockets: [bool; 3],
    pub garden_weights: [u32; 3],
    pub garden_pockets: [bool; 3],
    /// The first purchase of the trip: نگ and rate.
    pub purchase: (i64, i64),
    pub rent: i64,
}

/// Everything a step needs.
pub struct U {
    pub w: World,
    pub head_id: i64,
    pub head_account: i64,
    /// The other agent's shop (a seller), the source of fruit from another shop.
    pub agent: i64,
    /// Two khata-less loan people: shop link, partner link (ب).
    pub persons: [i64; 2],
    /// Three khatas: no link, shop link, partner link (الف).
    pub khatas: [i64; 3],
    pub orange: i64,
    pub trip: i64,
    pub purchases: Vec<i64>,
    pub garden: i64,
    pub garden_account: i64,
    /// New sellers and buyers made with an opening balance during the run.
    pub extra_sellers: Vec<i64>,
    pub extra_buyers: Vec<i64>,
    pub shares_changed: bool,
    pub made: u32,
    /// The harness's own idea of the closed-up-to day, kept apart from the service so it can check
    /// it: -1 = no day of the run closed, 0 = 10-10-2026, 1 = 11-10-2026 (the 12th never closes).
    pub closed_level: i8,
    /// The journal entries dated on or before the closed-up-to day (an opening of a new account
    /// apart, PRD FR-10.11), as they were when the day was last closed or reopened. A closed day
    /// never changes except through a reopen (PRD FR-6.6, FR-6.3).
    pub closed_ids: BTreeSet<i64>,
    /// The بکری that had a later sale, a carried posting or an edit since their save. Only an
    /// untouched بکری can be undone (PRD FR-7.47, plan 80 R2-2).
    pub touched: BTreeSet<i64>,
    /// One refusal that the step has just proved to be the right answer, so the run does not take
    /// it for a bug (see `steps::exec`). Set by the step itself, only for the exact case.
    pub allowed: Option<ErrorCode>,
    /// The second garden of the goods-arrived lock step is made (once in a run).
    pub garden2_done: bool,
    /// The day (`YYYY-MM-DD`) of the posting the last successful edit of a بکری changed.
    pub last_edit_day: Option<String>,
}

/// Runs a service call that may ask the spelling question of the names it saves (PRD FR-7.66), and
/// confirms the words it names.
pub fn confirming<T>(mut attempt: impl FnMut(&[String]) -> Result<T, ServiceError>) -> T {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..6 {
        match attempt(&words) {
            Ok(done) => return done,
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        words.extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("{other:?}"),
        }
    }
    panic!("the spelling question did not end");
}

/// Shares in bps that total exactly 10,000, from weights.
pub fn shares_of(weights: &[u32]) -> Vec<i32> {
    let total: u32 = weights.iter().sum();
    let mut out: Vec<i32> = weights
        .iter()
        .map(|w| i32::try_from(u64::from(*w) * 10_000 / u64::from(total)).unwrap_or(0))
        .collect();
    let used: i32 = out.iter().sum();
    if let Some(first) = out.first_mut() {
        *first += 10_000 - used;
    }
    out
}

fn percent(bps: i32) -> String {
    format!("{}.{:02}", bps / 100, bps % 100)
}

fn listed(w: &World, weights: &[u32; 3], pockets: &[bool; 3]) -> Vec<VenturePartnerInput> {
    let shares = shares_of(weights);
    w.partners
        .iter()
        .enumerate()
        .filter(|(i, _)| shares.get(*i).copied().unwrap_or(0) > 0)
        .map(|(i, id)| VenturePartnerInput {
            partner_id: *id,
            share: percent(shares.get(i).copied().unwrap_or(0)),
            source: if pockets[i] {
                MoneySource::Pocket
            } else {
                MoneySource::Shop
            },
        })
        .collect()
}

fn make_khata(w: &World, ur: &str, en: &str, link: Option<Link>) -> i64 {
    confirming(|words| {
        khata::create_linked(
            &w.env.ctx,
            &w.owner,
            &KhataCreateRequest {
                base: KhataCreateInput {
                    name_ur: ur.into(),
                    name_en: en.into(),
                    note: None,
                    confirmed_rules_words: words.to_vec(),
                },
                link,
            },
        )
    })
    .account
    .id
}

fn make_person(w: &World, ur: &str, en: &str, link: Option<Link>) -> i64 {
    confirming(|words| {
        lent::person_create_linked(
            &w.env.ctx,
            &w.owner,
            &LentPersonCreateRequest {
                base: LentPersonCreateInput {
                    name_ur: ur.into(),
                    name_en: en.into(),
                    phone: None,
                    staff_id: None,
                    confirmed_rules_words: words.to_vec(),
                },
                person_type: Some(PersonType::GardenOwner),
                link,
            },
        )
    })
    .account
    .id
}

/// The world of a run.
pub fn build(setup: &Setup) -> U {
    let w = support::world_with(200_000, true);
    set_today(&w.env, 2026, 10, 12);
    let (head_id, head_account) = {
        confirming(|words| {
            masters::save(
                &w.env.ctx,
                &w.owner,
                &MasterInput::ExpenseHead(ExpenseHeadInput {
                    id: None,
                    version: None,
                    code: String::new(),
                    name_ur: "بجلی".into(),
                    name_en: "Electricity".into(),
                    confirmed_rules_words: words.to_vec(),
                }),
            )
        });
        let row = masters::expense_heads(&w.env.ctx)
            .unwrap()
            .rows
            .into_iter()
            .find(|r| r.head.name_en == "Electricity")
            .unwrap();
        (row.id, row.account_id)
    };
    confirming(|words| {
        masters::save(
            &w.env.ctx,
            &w.owner,
            &MasterInput::ExpenseHead(ExpenseHeadInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: "تنخواہ".into(),
                name_en: "Salaries".into(),
                confirmed_rules_words: words.to_vec(),
            }),
        )
    });
    let agent = support::add_account(
        &w.env,
        &w.owner,
        AccountType::Seller,
        "حاجی نذیر",
        "Haji Nazir",
        true,
        "",
    );
    let shop = Link {
        kind: LinkKind::Shop,
        partner_id: None,
    };
    let partner = |i: usize| Link {
        kind: LinkKind::Partner,
        partner_id: Some(w.partners[i]),
    };
    let khatas = [
        make_khata(&w, "خاندانی کھاتہ", "Family Khata", None),
        make_khata(&w, "گھر کھاتہ", "Home Khata", Some(shop)),
        make_khata(&w, "ذاتی کھاتہ", "Personal Khata", Some(partner(0))),
    ];
    let persons = [
        make_person(&w, "نواز", "Nawaz", Some(shop)),
        make_person(&w, "اشرف", "Ashraf", Some(partner(1))),
    ];
    let orange = support::add_item(&w.env, &w.owner, "مالٹا", "Orange", "fruit", "unit");
    let trip = trips::create(
        &w.env.ctx,
        &w.owner,
        &TripInput {
            date: "10-10-2026".into(),
            market: None,
            sent_person: None,
            note: None,
            partners: listed(&w, &setup.trip_weights, &setup.trip_pockets),
        },
    )
    .unwrap()
    .id;
    let first = purchases::save(
        &w.env.ctx,
        &w.owner,
        &PurchaseInput {
            // Half of the goods are set aside for another shop (PRD FR-12.41, v1.52): a sale in
            // another shop takes only from that part.
            other_qty: (setup.purchase.0 >= 2).then(|| (setup.purchase.0 / 2).to_string()),
            shop_name: (setup.purchase.0 >= 2).then(|| "Haji Traders".to_owned()),
            sell_date: None,
            trip_id: Some(trip),
            date: "10-10-2026".into(),
            supplier: None,
            item_id: Some(orange),
            packing_id: Some(w.box_),
            qty: setup.purchase.0.to_string(),
            weight: None,
            rate: setup.purchase.1.to_string(),
            freight: None,
            labour: None,
            other_cost: None,
            note: None,
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .id;
    let garden = gardens::create(
        &w.env.ctx,
        &w.owner,
        &GardenInput {
            name_ur: "شاہ پور باغ".into(),
            name_en: "Shahpur Garden".into(),
            confirmed_rules_words: vec![
                "شاہ".into(),
                "پور".into(),
                "باغ".into(),
                "Shahpur".into(),
                "Garden".into(),
            ],
            date: "10-10-2026".into(),
            item_id: Some(w.banana),
            item_ids: Vec::new(),
            rent: setup.rent.to_string(),
            season_end: None,
            note: None,
            partners: listed(&w, &setup.garden_weights, &setup.garden_pockets),
            paid_now: None,
            paid_now_reason: None,
            parts: Vec::new(),
            confirm_cash_below_zero: true,
        },
    )
    .unwrap()
    .row;
    U {
        w,
        head_id,
        head_account,
        agent,
        persons,
        khatas,
        orange,
        trip,
        purchases: vec![first],
        garden: garden.id,
        garden_account: garden.account.id,
        extra_sellers: Vec::new(),
        extra_buyers: Vec::new(),
        shares_changed: false,
        made: 0,
        closed_level: -1,
        closed_ids: BTreeSet::new(),
        touched: BTreeSet::new(),
        allowed: None,
        garden2_done: false,
        last_edit_day: None,
    }
}

/// A second garden, with no entry, no goods arrived and no posted line, on the open 12th (PRD
/// FR-12.29). Its shares come from `weights`. Gives its id.
pub fn make_second_garden(u: &U, weights: &[u32; 3]) -> i64 {
    let w = &u.w;
    confirming(|words| {
        gardens::create(
            &w.env.ctx,
            &w.owner,
            &GardenInput {
                name_ur: "گل باغ".into(),
                name_en: "Gul Bagh".into(),
                confirmed_rules_words: words.to_vec(),
                date: "12-10-2026".into(),
                item_id: Some(w.banana),
                item_ids: Vec::new(),
                rent: "9000".into(),
                season_end: None,
                note: None,
                partners: listed(w, weights, &[false; 3]),
                paid_now: None,
                paid_now_reason: None,
                parts: Vec::new(),
                confirm_cash_below_zero: true,
            },
        )
    })
    .row
    .id
}

/// The partners of a garden as the share inputs of a change, from `weights`, every one paid from
/// the shop's cash.
pub fn partner_inputs(u: &U, weights: &[u32; 3]) -> Vec<VenturePartnerInput> {
    listed(&u.w, weights, &[false; 3])
}

impl U {
    /// Every seller account a receipt, payment or advance can name.
    pub fn sellers(&self) -> Vec<i64> {
        let mut v = vec![self.w.rahim, self.w.karim, self.agent];
        v.extend(&self.extra_sellers);
        v
    }

    /// Every buyer account.
    pub fn buyers(&self) -> Vec<i64> {
        let mut v = vec![self.w.akram, self.w.arshad];
        v.extend(&self.extra_buyers);
        v
    }

    /// Every party a receipt or a payment can name.
    pub fn parties(&self) -> Vec<i64> {
        let mut v = self.buyers();
        v.extend(self.sellers());
        v
    }
}

/// A new seller or buyer with an opening balance, on its own (PRD FR-10.9, FR-10.11).
pub fn new_account_with_opening(
    u: &mut U,
    seller: bool,
    debit: bool,
    amount: i64,
    advance: bool,
) -> Result<i64, ServiceError> {
    u.made += 1;
    let n = u.made;
    const NAMES: [(&str, &str); 10] = [
        ("احمد", "Ahmad"),
        ("بلال", "Bilal"),
        ("کامران", "Kamran"),
        ("دلاور", "Dilawar"),
        ("ہارون", "Haroon"),
        ("فیضان", "Faizan"),
        ("غلام", "Ghulam"),
        ("حسن", "Hasan"),
        ("اکبر", "Akbar"),
        ("جمیل", "Jamil"),
    ];
    let (ur, en) = NAMES[usize::try_from(n).unwrap_or(1) % NAMES.len()];
    let (ur, en) = (ur, en.to_owned());
    let mut input = AccountInput {
        id: None,
        version: None,
        account_type: if seller {
            AccountType::Seller
        } else {
            AccountType::Buyer
        },
        control_id: None,
        code: String::new(),
        name_ur: ur.into(),
        name_en: en,
        confirmed_rules_words: Vec::new(),
        father_or_firm_ur: String::new(),
        father_or_firm_en: String::new(),
        city_id: None,
        adda_id: None,
        phone1: String::new(),
        phone2: String::new(),
        address: String::new(),
        linked_account_id: None,
        opening_date: "10-10-2026".into(),
        opening_dr: if debit {
            amount.to_string()
        } else {
            String::new()
        },
        opening_cr: if debit {
            String::new()
        } else {
            amount.to_string()
        },
        opening_is_advance: advance && seller && debit,
        credit_allowed: true,
        credit_limit: String::new(),
        default_marka_id: None,
        collector_staff_id: None,
        is_active: true,
        note: String::new(),
        tag: String::new(),
    };
    for _ in 0..6 {
        match accounts::save(&u.w.env.ctx, &u.w.owner, &input) {
            Ok(row) => {
                if seller {
                    u.extra_sellers.push(row.id);
                } else {
                    u.extra_buyers.push(row.id);
                }
                return Ok(row.id);
            }
            Err(ServiceError::Rejected(items))
                if items
                    .iter()
                    .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(t)) = item.values.get("words") {
                        input
                            .confirmed_rules_words
                            .extend(t.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => return Err(other),
        }
    }
    panic!("the spelling question did not end");
}
