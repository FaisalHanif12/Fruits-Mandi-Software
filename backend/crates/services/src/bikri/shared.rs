//! What the preview and the save share (Code Standards B5): the facts read for a draft, the one
//! working out of a draft, and what it does to the books and to cash. Both call [`work_out`], so
//! the screen and the saved numbers cannot differ.

use std::collections::{BTreeSet, HashMap, HashSet};

use mandi_db::bikri as repo;
use mandi_db::bikri::edit as edit_repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::partners as partner_repo;
use mandi_db::purchases as purchase_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{
    AccountFacts, CalcError, ItemFacts, ItemSales, Lookups, PartyKind, PostingCalc, PostingTarget,
    PurchaseFacts, StoredPosting, Switches, TripPartnerFacts, calc_posting, partner_share,
};
use mandi_domain::cash;
use mandi_domain::core::accounts::{
    BUYER_CONTROL, GARDEN_CONTROL_CODE, KHATA_CONTROL, NAQDI_MANDI_CODE, SELLER_CONTROL,
};
use mandi_domain::core::bikri::{BikriPreview, PreparedDraft, SplitPart};
use mandi_domain::core::cash::CashCheck;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::rules::{CreditLimitMode, ItemGroup, OverridesSettings};
use mandi_domain::core::ventures::TripRef;
use mandi_domain::ledger::bikri::{EntryLayout, SystemAccounts, build_bikri_entry_with_layout};
use mandi_domain::ledger::{BalancedEntry, LedgerError, MAX_LINE_PAISA};
use mandi_domain::partners::ShareKind;
use mandi_domain::rules::RuleSet;

use crate::cash as cash_service;
use crate::error::{Rejection, ServiceError, money_error};
use crate::partners::split_entry;
use crate::ventures;

/// The system accounts a بکری posts to (PRD FR-10.4).
const SELLER_COMMISSION_CODE: &str = "04010001";
const BUYER_COMMISSION_CODE: &str = "04010002";
const LAGA_CODE: &str = "04010003";
const PHONE_CODE: &str = "04010004";
const RATE_DIFF_CODE: &str = "04010005";
const LABOUR_CODE: &str = "02010001";
const FREIGHT_CODE: &str = "02010002";
const STORE_RENT_CODE: &str = "02010003";
const MUNSHIANA_CODE: &str = "02010004";
/// اپنا مال, the seller of a trip's goods (PRD FR-12.26).
pub(super) const OWN_GOODS_CODE: &str = "030119998";
/// The days of the rate check, ending on the posting day (PRD FR-7.55, an [A] reading).
const RATE_CHECK_DAYS: u32 = 7;

pub(super) fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

pub(super) fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// A failure of the calculation that is not the user's mistake.
pub(super) fn calc_error(error: CalcError) -> ServiceError {
    match error {
        CalcError::Money(money) => money_error(money),
        other => {
            tracing::error!(error = %other, "a بکری could not be worked out");
            ServiceError::Data(other.to_string())
        }
    }
}

/// The system accounts of PRD FR-10.4 that a بکری posts to, with their ids.
pub(super) fn system_accounts(conn: &Connection) -> Result<SystemAccounts, ServiceError> {
    let codes = [
        cash_service::CASH_ACCOUNT_CODE,
        SELLER_COMMISSION_CODE,
        BUYER_COMMISSION_CODE,
        LAGA_CODE,
        PHONE_CODE,
        RATE_DIFF_CODE,
        LABOUR_CODE,
        FREIGHT_CODE,
        STORE_RENT_CODE,
        MUNSHIANA_CODE,
    ];
    let ids = repo::ids_by_codes(conn, &codes)?;
    let id = |code: &str| {
        ids.get(code)
            .map(|id| AccountId::new(*id))
            .ok_or_else(|| ServiceError::Data(format!("the system account {code} is missing")))
    };
    Ok(SystemAccounts {
        cash: id(cash_service::CASH_ACCOUNT_CODE)?,
        seller_commission: id(SELLER_COMMISSION_CODE)?,
        buyer_commission: id(BUYER_COMMISSION_CODE)?,
        laga: id(LAGA_CODE)?,
        phone: id(PHONE_CODE)?,
        rate_diff: id(RATE_DIFF_CODE)?,
        labour: id(LABOUR_CODE)?,
        freight: id(FREIGHT_CODE)?,
        store_rent: id(STORE_RENT_CODE)?,
        munshiana: id(MUNSHIANA_CODE)?,
    })
}

/// The ids a draft names, each list without repeats.
#[derive(Default)]
struct Named {
    accounts: BTreeSet<i64>,
    items: BTreeSet<i64>,
    packings: BTreeSet<i64>,
    markas: BTreeSet<i64>,
    stores: BTreeSet<i64>,
    addas: BTreeSet<i64>,
    /// (item, packing) pairs, for the saved rate hint.
    pairs: BTreeSet<(i64, i64)>,
}

fn named(prepared: &PreparedDraft) -> Named {
    let mut n = Named::default();
    let h = &prepared.header;
    n.accounts.extend(h.zamindar_account_id);
    n.addas.extend(h.adda_id);
    n.stores.extend(h.store_id);
    for lot in &prepared.lots {
        n.accounts.extend(lot.seller_account_id);
        n.items.extend(lot.item_id);
        n.packings.extend(lot.packing_id);
        n.markas.extend(lot.marka_id);
        n.stores.extend(lot.store_id);
        if let Some(pair) = lot.item_id.zip(lot.packing_id) {
            n.pairs.insert(pair);
        }
        for sale in lot.sales.iter().filter(|s| !s.is_blank) {
            n.accounts.extend(sale.buyer_account_id);
        }
    }
    n
}

fn existing(
    conn: &Connection,
    table: repo::MasterTable,
    ids: &BTreeSet<i64>,
) -> Result<HashSet<i64>, ServiceError> {
    let ids: Vec<i64> = ids.iter().copied().collect();
    Ok(repo::existing_ids(conn, table, &ids)?.into_iter().collect())
}

/// What the purchases named by the own lots of a draft can give this posting (PRD FR-12.26): the
/// item and packing, and the نگ and cost left. In an edit, what the posting being replaced took is
/// added back, so the posting is worked out against what it could take if it were not saved yet.
fn load_purchases(
    conn: &Connection,
    prepared: &PreparedDraft,
    replaces: Option<i64>,
) -> Result<HashMap<i64, PurchaseFacts>, ServiceError> {
    let ids: BTreeSet<i64> = prepared.lots.iter().filter_map(|l| l.purchase_id).collect();
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let ids: Vec<i64> = ids.into_iter().collect();
    let mut back: HashMap<i64, (i64, i64)> = HashMap::new();
    if let Some(entry_id) = replaces {
        for taken in edit_repo::own_release_of_entry(conn, entry_id)? {
            back.insert(taken.purchase_id, (taken.qty, taken.cost_paisa));
        }
    }
    let mut out = HashMap::with_capacity(ids.len());
    for rec in purchase_repo::get_many(conn, &ids)? {
        let cancelled = rec.status == "cancelled";
        let (back_qty, back_cost) = back.get(&rec.id).copied().unwrap_or((0, 0));
        let qty_left = if cancelled {
            0
        } else {
            rec.qty
                .checked_sub(rec.qty_sold)
                .and_then(|v| v.checked_sub(rec.qty_written_off))
                .and_then(|v| v.checked_add(back_qty))
                .ok_or_else(|| data("the quantity of a purchase does not fit"))?
        };
        let cost_left = if cancelled {
            Paisa::ZERO
        } else {
            paisa(
                rec.total_cost_paisa
                    .checked_sub(rec.cost_released_paisa)
                    .and_then(|v| v.checked_add(back_cost))
                    .ok_or_else(|| data("the cost of a purchase does not fit"))?,
            )?
        };
        let trip = ventures::load(conn, rec.trip_id)?
            .ok_or_else(|| data("the trip of a purchase is missing"))?;
        let partners = trip_partners(&trip)?;
        out.insert(
            rec.id,
            PurchaseFacts {
                number: rec.number,
                entry_date: rec.entry_date.clone(),
                trip: TripRef {
                    id: rec.trip_id,
                    number: rec.trip_number,
                    name_ur: rec.trip_name_ur.clone(),
                    name_en: rec.trip_name_en.clone(),
                },
                partners,
                item_id: rec.item.id,
                packing_id: rec.packing.id,
                qty_left,
                cost_left,
                cancelled,
            },
        );
    }
    Ok(out)
}

/// The partners of a trip with their shares and 0604 accounts, for the split of the seller credit
/// of an own lot (PRD FR-12.26, C25). The shares were checked to total 100% when the trip was made.
fn trip_partners(trip: &ventures::Venture) -> Result<Vec<TripPartnerFacts>, ServiceError> {
    let parts = ventures::credit_partners(trip)?;
    trip.partners
        .iter()
        .zip(parts)
        .map(|(p, partner)| {
            Ok(TripPartnerFacts {
                partner,
                name_ur: p.name_ur.clone(),
                name_en: p.name_en.clone(),
            })
        })
        .collect()
}

/// An own lot with no item or packing takes them from its purchase; the screen shows them as
/// filled values (PRD FR-12.26, NFR-USE-27). A different one is refused by the domain.
fn fill_own_lots(prepared: &mut PreparedDraft, purchases: &HashMap<i64, PurchaseFacts>) {
    for lot in &mut prepared.lots {
        if let Some(facts) = lot.purchase_id.and_then(|id| purchases.get(&id)) {
            lot.item_id = lot.item_id.or(Some(facts.item_id));
            lot.packing_id = lot.packing_id.or(Some(facts.packing_id));
        }
    }
}

/// What the database says about the accounts, items and masters of a draft.
fn load_lookups(
    conn: &Connection,
    prepared: &PreparedDraft,
    rate_check: bool,
    replaces: Option<i64>,
) -> Result<Lookups, ServiceError> {
    let n = named(prepared);
    let account_ids: Vec<i64> = n.accounts.iter().copied().collect();
    let found = repo::accounts(conn, &account_ids)?;
    // A buyer with a credit limit needs his balance now (PRD FR-12.4): one query for all. In an
    // edit the balance leaves out the posting being replaced, which the new one takes over.
    let limited: Vec<i64> = found
        .iter()
        .filter(|a| a.control_code == BUYER_CONTROL && a.credit_limit_paisa.is_some())
        .map(|a| a.id)
        .collect();
    let totals = ledger_repo::totals_for_accounts(conn, &limited)?;
    let own = match replaces {
        Some(entry_id) => ledger_repo::entry_totals_for_accounts(conn, entry_id, &limited)?,
        None => HashMap::new(),
    };
    let mut accounts = HashMap::with_capacity(found.len());
    for a in found {
        let balance = match totals.get(&a.id) {
            Some(t) => {
                let (own_dr, own_cr) = own.get(&a.id).map_or((0, 0), |o| (o.dr_paisa, o.cr_paisa));
                let dr = t.dr_paisa.checked_sub(own_dr);
                let cr = t.cr_paisa.checked_sub(own_cr);
                match (dr, cr) {
                    (Some(dr), Some(cr)) => {
                        paisa(dr)?.checked_sub(paisa(cr)?).map_err(money_error)?
                    }
                    _ => return Err(data("a balance does not fit")),
                }
            }
            None => Paisa::ZERO,
        };
        let kind = match a.control_code.as_str() {
            SELLER_CONTROL => PartyKind::Seller,
            BUYER_CONTROL => PartyKind::Buyer,
            KHATA_CONTROL => PartyKind::Khata,
            GARDEN_CONTROL_CODE => PartyKind::Garden,
            _ => PartyKind::Other,
        };
        // A garden takes lots only while it is open (PRD FR-12.38).
        let garden_closed =
            kind == PartyKind::Garden && mandi_db::gardens::account_is_closed(conn, a.id)?;
        accounts.insert(
            a.id,
            AccountFacts {
                kind,
                is_pukhta: a.is_pukhta_khata,
                is_naqdi_mandi: a.code == NAQDI_MANDI_CODE,
                is_own_goods: a.code == OWN_GOODS_CODE,
                garden_closed,
                is_active: a.is_active,
                credit_allowed: a.credit_allowed,
                credit_limit: a.credit_limit_paisa.map(paisa).transpose()?,
                balance,
            },
        );
    }
    let item_ids: Vec<i64> = n.items.iter().copied().collect();
    let items = repo::items(conn, &item_ids)?
        .into_iter()
        .map(|i| {
            let group = ItemGroup::parse(&i.item_group)
                .ok_or_else(|| data("an item has a group that is not fruit or vegetable"))?;
            Ok((
                i.id,
                ItemFacts {
                    group,
                    by_weight: i.sold_by == "weight",
                    is_active: i.is_active,
                },
            ))
        })
        .collect::<Result<HashMap<_, _>, ServiceError>>()?;

    let mut lookups = Lookups {
        accounts,
        items,
        packings: existing(conn, repo::MasterTable::Packings, &n.packings)?,
        markas: existing(conn, repo::MasterTable::Markas, &n.markas)?,
        stores: existing(conn, repo::MasterTable::Stores, &n.stores)?,
        addas: existing(conn, repo::MasterTable::Addas, &n.addas)?,
        ..Lookups::default()
    };
    // The saved rates need the posting day (PRD FR-7.53, FR-7.55). Without a valid day the
    // header already has its error and no row is worked out.
    if let Some(date) = prepared.header.entry_date {
        for &(item, packing) in &n.pairs {
            if let Some(hint) =
                crate::assist::saved_rate_hint_except(conn, item, packing, date, replaces)?
            {
                lookups.saved_hints.insert((item, packing), hint);
            }
        }
        if rate_check {
            // The posting day and the 6 days before it.
            let from = (1..RATE_CHECK_DAYS).try_fold(date, |d, _| d.previous_day());
            if let Some(from) = from {
                for &item in &n.items {
                    let (qty, amount) = repo::item_sales_between_except(
                        conn,
                        item,
                        &from.to_iso(),
                        &date.to_iso(),
                        replaces,
                    )?;
                    lookups.item_sales.insert(
                        item,
                        ItemSales {
                            qty,
                            amount: paisa(amount)?,
                        },
                    );
                }
            }
        }
    }
    Ok(lookups)
}

pub(super) fn switches(settings: &OverridesSettings) -> Switches {
    Switches {
        allow_line_override: settings.allow_line_override,
        credit_limit_blocks: settings.credit_limit_mode == CreditLimitMode::Block,
        rate_check_percent: settings
            .rate_check_enabled
            .then_some(settings.rate_check_percent),
    }
}

/// The partner split of the income of the entry, through the one function the save will use
/// (`partners::split_entry`, PRD FR-17.4, plan 11). The income is never added up here.
fn split_of(
    conn: &Connection,
    date: BusinessDate,
    entry: &BalancedEntry,
    shared_income: Paisa,
    errors: &mut Vec<FieldError>,
) -> Result<Vec<SplitPart>, ServiceError> {
    let rows = match split_entry(conn, date, entry) {
        Ok(rows) => rows,
        // Shares that do not total 100% stop the save, but the numbers are still shown (money
        // agent point 11).
        Err(ServiceError::Rejected(items))
            if items.iter().all(|i| i.code == ErrorCode::ErrSharesNot100) =>
        {
            for item in items {
                errors.push(FieldError {
                    code: item.code,
                    field: "split".to_owned(),
                    values: item.values,
                });
            }
            return Ok(Vec::new());
        }
        Err(other) => return Err(other),
    };
    if rows.iter().any(|r| r.kind != ShareKind::Income) {
        return Err(data("a بکری made a share row that is not income"));
    }
    let total = rows
        .iter()
        .try_fold(Paisa::ZERO, |sum, r| sum.checked_add(r.amount))
        .map_err(money_error)?;
    // The summary and the split are worked out from the same lines, so they must agree (money
    // agent point 4). If they do not, something is wrong and nothing is shown.
    if total != shared_income {
        tracing::error!(
            split = total.as_paisa(),
            shared = shared_income.as_paisa(),
            "the partner split differs from the shared income of the بکری"
        );
        return Err(data("the partner split differs from the shared income"));
    }
    let names: HashMap<i64, (String, String)> = partner_repo::list(conn)?
        .into_iter()
        .map(|p| (p.id, (p.name_ur, p.name_en)))
        .collect();
    Ok(rows
        .iter()
        .map(|row| {
            let id = row.partner_id.get();
            let (name_ur, name_en) = names.get(&id).cloned().unwrap_or_default();
            SplitPart {
                partner_id: id,
                name_ur,
                name_en,
                share_bps: row.share_bps.get(),
                income_paisa: row.amount,
                working: partner_share(row.base, row.share_bps.get(), row.amount),
            }
        })
        .collect())
}

/// A journal entry with the positions of the lines that matter for matching.
pub(crate) type BuiltEntry = (BalancedEntry, EntryLayout);

/// A draft worked out against the database: everything the preview shows and the save writes.
#[derive(Debug)]
pub(crate) struct Worked {
    /// The typed draft, read.
    pub prepared: PreparedDraft,
    /// The one calculation (`calc_posting`).
    pub calc: PostingCalc,
    /// Every error that stops a save: those of the calculation and of the books (shares).
    pub errors: Vec<FieldError>,
    /// The live partner split of the income.
    pub split: Vec<SplitPart>,
    /// The journal entry of the worked rows and where its matching lines are. `None` when
    /// nothing could be worked out (then `errors` says why).
    pub entry: Option<BuiltEntry>,
    /// What the entry does to cash in hand (PRD FR-6.7).
    pub cash: CashCheck,
    /// The Owner's switches used.
    pub switches: Switches,
}

impl Worked {
    /// What the live preview sends. The save sends the same value back with its result.
    pub(crate) fn preview(&self) -> BikriPreview {
        BikriPreview {
            lots: self.calc.lots.clone(),
            summary: self.calc.summary.clone(),
            split: self.split.clone(),
            cash: self.cash,
            allow_line_override: self.switches.allow_line_override,
            errors: self.errors.clone(),
            warnings: self.calc.warnings.clone(),
        }
    }
}

/// Builds the entry of the worked rows, splits its income and works out what it does to cash.
/// Returns the split, the cash change and the entry with its layout.
fn books_of(
    conn: &Connection,
    prepared: &PreparedDraft,
    calc: &PostingCalc,
    accounts: &SystemAccounts,
    errors: &mut Vec<FieldError>,
) -> Result<(Vec<SplitPart>, Paisa, Option<BuiltEntry>), ServiceError> {
    let (Some(date), false) = (prepared.header.entry_date, calc.journal.lots.is_empty()) else {
        return Ok((Vec::new(), Paisa::ZERO, None));
    };
    let built = match build_bikri_entry_with_layout(&calc.journal, accounts) {
        Ok(built) => built,
        Err(LedgerError::TooLarge) => {
            let max = MAX_LINE_PAISA.checked_div(100).unwrap_or(i64::MAX);
            errors.push(
                FieldError::new(ErrorCode::ErrEntryTooLarge, "lots")
                    .with("max", ErrorValue::Number(max)),
            );
            return Ok((Vec::new(), Paisa::ZERO, None));
        }
        Err(LedgerError::Overflow) => return Err(ServiceError::reject(ErrorCode::ErrOverflow)),
        Err(other) => {
            // The calculation built rows that do not balance: a bug signal (Code Standards B16).
            tracing::error!(error = %other, "the journal of a بکری does not balance");
            return Err(ServiceError::Rejected(vec![Rejection {
                code: ErrorCode::ErrJournalUnbalanced,
                field: None,
                values: std::collections::BTreeMap::new(),
            }]));
        }
    };
    let change = cash::net_change(&built.0, accounts.cash).map_err(money_error)?;
    let split = split_of(conn, date, &built.0, calc.summary.shared_income, errors)?;
    Ok((split, change, Some(built)))
}

/// The posting an edit takes the place of: its entry, and what it did to cash in hand.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Replaced {
    /// The journal entry being replaced.
    pub entry_id: i64,
    /// What that entry did to cash (debit minus credit on روکڑ). The edit's cash question is about
    /// the difference between the new entry and this (PRD FR-6.7).
    pub cash_change: Paisa,
}

/// What a draft is worked out for: a new بکری, a later sale or an edit, with what is stored about
/// the posting (plan 16). The preview and the save of each pass the same basis (Code Standards
/// B5).
#[derive(Debug, Clone)]
pub(crate) struct Basis {
    /// What `calc_posting` works out.
    pub target: PostingTarget,
    /// What is stored about the posting (empty for a new بکری).
    pub stored: StoredPosting,
    /// The posting an edit replaces.
    pub replaces: Option<Replaced>,
}

impl Basis {
    /// A new بکری.
    pub(crate) fn new_bikri() -> Self {
        Basis {
            target: PostingTarget::New,
            stored: StoredPosting::none(),
            replaces: None,
        }
    }
}

/// Works out a typed draft against the database: reads what it names, calls `calc_posting`,
/// builds the journal entry, splits the income and asks what the entry does to cash. Used by
/// the preview (inside a read) and by the save (inside its write transaction), so there is one
/// path (Code Standards B5). Nothing is written here.
///
/// # Errors
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot
/// fix. The mistakes in the draft come back in `Worked::errors`.
pub(crate) fn work_out(
    conn: &Connection,
    mut prepared: PreparedDraft,
    rules: &RuleSet,
    switches: Switches,
    basis: &Basis,
) -> Result<Worked, ServiceError> {
    let purchases = load_purchases(conn, &prepared, basis.replaces.map(|r| r.entry_id))?;
    fill_own_lots(&mut prepared, &purchases);
    let mut lookups = load_lookups(
        conn,
        &prepared,
        switches.rate_check_percent.is_some(),
        basis.replaces.map(|r| r.entry_id),
    )?;
    lookups.purchases = purchases;
    let calc = calc_posting(
        basis.target,
        &prepared,
        &lookups,
        rules,
        switches,
        &basis.stored,
    )
    .map_err(calc_error)?;
    let mut errors = calc.errors.clone();
    let accounts = system_accounts(conn)?;
    let (split, change, entry) = books_of(conn, &prepared, &calc, &accounts, &mut errors)?;
    // An edit's cash question is about the difference between the new entry and the one it
    // replaces: cash now still holds the old posting (PRD FR-6.7).
    let change = match basis.replaces {
        Some(old) => change.checked_sub(old.cash_change).map_err(money_error)?,
        None => change,
    };
    let cash = cash_service::cash_after_entry(conn, change)?;
    Ok(Worked {
        prepared,
        calc,
        errors,
        split,
        entry,
        cash,
        switches,
    })
}
