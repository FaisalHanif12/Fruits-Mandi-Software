//! The بکری entry screen (PRD 12, plan 12): the live preview.
//!
//! Read-only: nothing here writes (Code Standards B29). The preview reads the facts a draft
//! names in one go (Code Standards B22), calls `mandi_domain::bikri::calc_posting`, builds the
//! journal entry with `build_bikri_entry`, splits the income with `partners::split_entry`, and
//! asks `cash::cash_after_entry` what the entry does to cash. The save of plan 13 calls the same
//! domain functions (Code Standards B5), so the preview and the saved numbers cannot differ.
//!
//! The read connection is not re-entrant, so the rules and switches are fetched before the one
//! read of the call (see the services README).

use std::collections::{BTreeSet, HashMap, HashSet};

use mandi_db::bikri as repo;
use mandi_db::ledger as ledger_repo;
use mandi_db::partners as partner_repo;
use mandi_db::rusqlite::Connection;
use mandi_domain::bikri::{
    AccountFacts, CalcError, ItemFacts, ItemSales, Lookups, PartyKind, PostingCalc, PostingKind,
    Switches, calc_posting, partner_share,
};
use mandi_domain::cash;
use mandi_domain::core::accounts::{BUYER_CONTROL, NAQDI_MANDI_CODE, SELLER_CONTROL};
use mandi_domain::core::bikri::{
    BikriDraft, BikriPreview, PreparedDraft, SplitPart, prepare_draft,
};
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::ids::AccountId;
use mandi_domain::core::money::Paisa;
use mandi_domain::core::rules::{CreditLimitMode, ItemGroup, OverridesSettings};
use mandi_domain::ledger::bikri::{SystemAccounts, build_bikri_entry};
use mandi_domain::ledger::{BalancedEntry, LedgerError, MAX_LINE_PAISA};
use mandi_domain::partners::ShareKind;

use crate::cash as cash_service;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::partners::split_entry;
use crate::rules;

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
/// اپنا مال, the firm as seller (PRD FR-12.26). Plan 16b makes it; until then a lot of it is
/// refused as a seller.
const OWN_GOODS_CODE: &str = "030119998";
/// The days of the rate check, ending on the posting day (PRD FR-7.55, an [A] reading).
const RATE_CHECK_DAYS: u32 = 7;

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

/// A failure of the calculation that is not the user's mistake.
fn calc_error(error: CalcError) -> ServiceError {
    match error {
        CalcError::Money(money) => money_error(money),
        other => {
            tracing::error!(error = %other, "a بکری could not be worked out");
            ServiceError::Data(other.to_string())
        }
    }
}

/// The system accounts of PRD FR-10.4 that a بکری posts to, with their ids.
fn system_accounts(conn: &Connection) -> Result<SystemAccounts, ServiceError> {
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

/// What the database says about the accounts, items and masters of a draft.
fn load_lookups(
    conn: &Connection,
    prepared: &PreparedDraft,
    rate_check: bool,
) -> Result<Lookups, ServiceError> {
    let n = named(prepared);
    let account_ids: Vec<i64> = n.accounts.iter().copied().collect();
    let found = repo::accounts(conn, &account_ids)?;
    // A buyer with a credit limit needs his balance now (PRD FR-12.4): one query for all.
    let limited: Vec<i64> = found
        .iter()
        .filter(|a| a.control_code == BUYER_CONTROL && a.credit_limit_paisa.is_some())
        .map(|a| a.id)
        .collect();
    let totals = ledger_repo::totals_for_accounts(conn, &limited)?;
    let mut accounts = HashMap::with_capacity(found.len());
    for a in found {
        let balance = match totals.get(&a.id) {
            Some(t) => paisa(t.dr_paisa)?
                .checked_sub(paisa(t.cr_paisa)?)
                .map_err(money_error)?,
            None => Paisa::ZERO,
        };
        let kind = match a.control_code.as_str() {
            SELLER_CONTROL => PartyKind::Seller,
            BUYER_CONTROL => PartyKind::Buyer,
            _ => PartyKind::Other,
        };
        accounts.insert(
            a.id,
            AccountFacts {
                kind,
                is_pukhta: a.is_pukhta_khata,
                is_naqdi_mandi: a.code == NAQDI_MANDI_CODE,
                is_own_goods: a.code == OWN_GOODS_CODE,
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
            if let Some(hint) = crate::assist::saved_rate_hint(conn, item, packing, date)? {
                lookups.saved_hints.insert((item, packing), hint);
            }
        }
        if rate_check {
            // The posting day and the 6 days before it.
            let from = (1..RATE_CHECK_DAYS).try_fold(date, |d, _| d.previous_day());
            if let Some(from) = from {
                for &item in &n.items {
                    let (qty, amount) =
                        repo::item_sales_between(conn, item, &from.to_iso(), &date.to_iso())?;
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

fn switches(settings: &OverridesSettings) -> Switches {
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

/// Builds the entry of the worked rows, splits its income and works out what it does to cash.
/// Returns the split and the cash change.
fn books_of(
    conn: &Connection,
    prepared: &PreparedDraft,
    calc: &PostingCalc,
    errors: &mut Vec<FieldError>,
) -> Result<(Vec<SplitPart>, Paisa), ServiceError> {
    let (Some(date), false) = (prepared.header.entry_date, calc.journal.lots.is_empty()) else {
        return Ok((Vec::new(), Paisa::ZERO));
    };
    let accounts = system_accounts(conn)?;
    let entry = match build_bikri_entry(&calc.journal, &accounts) {
        Ok(entry) => entry,
        Err(LedgerError::TooLarge) => {
            let max = MAX_LINE_PAISA.checked_div(100).unwrap_or(i64::MAX);
            errors.push(
                FieldError::new(ErrorCode::ErrEntryTooLarge, "lots")
                    .with("max", ErrorValue::Number(max)),
            );
            return Ok((Vec::new(), Paisa::ZERO));
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
    let change = cash::net_change(&entry, accounts.cash).map_err(money_error)?;
    let split = split_of(conn, date, &entry, calc.summary.shared_income, errors)?;
    Ok((split, change))
}

/// Works out a whole typed بکری for the live summary (PRD 12.5, FR-7.5). Nothing is saved.
///
/// Example (PRD 4.4): Event 1 gives خام 29,400, کل اخراجات 10,084, صافی 19,316, buyer bills
/// 12,880 and 18,684, shared income 3,948 split 1,580 / 1,184 / 1,184, and cash 5,000 lower.
///
/// # Errors
/// The mistakes in the draft come back inside the preview (`errors`), not as an error here.
/// `ServiceError::Db`, `ServiceError::Data` or `ERR_OVERFLOW` for a failure the user cannot fix.
pub fn preview(ctx: &Context, draft: &BikriDraft) -> Result<BikriPreview, ServiceError> {
    let (closed_up_to, today) = day::window(ctx)?;
    let prepared = prepare_draft(draft, today, closed_up_to);
    // Fetched before the read below: the read connection is not re-entrant.
    let rule_set = rules::rule_set(ctx)?;
    let settings = rules::overrides(ctx)?;
    let switches = switches(&settings);
    ctx.db.read(|conn| {
        let lookups = load_lookups(conn, &prepared, switches.rate_check_percent.is_some())?;
        let calc = calc_posting(PostingKind::First, &prepared, &lookups, &rule_set, switches)
            .map_err(calc_error)?;
        let mut errors = calc.errors.clone();
        let (split, change) = books_of(conn, &prepared, &calc, &mut errors)?;
        let cash = cash_service::cash_after_entry(conn, change)?;
        Ok(BikriPreview {
            lots: calc.lots,
            summary: calc.summary,
            split,
            cash,
            allow_line_override: switches.allow_line_override,
            errors,
            warnings: calc.warnings,
        })
    })
}
