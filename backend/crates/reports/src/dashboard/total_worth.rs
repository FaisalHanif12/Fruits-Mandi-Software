//! The tile and the page کل مالیت (Total Worth, PRD FR-9.10, v1.61, plan 100 B): every rupee the
//! shop has or must take, less what it must give.
//!
//! **One account, one balance.** The working reads the ledger balance (Σ بنام − Σ جمع) of every
//! account of the heads 01 (assets), 02 (payables) and 03 (parties), but for the linked money
//! contra 0307. It puts each account's balance in the named lists, which are the very figures of
//! their own tiles and reports:
//!
//! * روکڑ: the cash tile (R12), which can be below 0 (PRD FR-6.7);
//! * گاہکوں کے ذمے: the rows of R20;
//! * the sellers: their payable and open advance netted ([`mandi_core::seller_net`]) and what
//!   they owe beyond them (R24), so each seller's have less give is exactly his balance;
//! * ڈوبی رقم: what each moved person still owes (R38);
//! * ادھار رقم: the open loans of R32, people in the dead amount left out;
//! * الگ کھاتے, the garden accounts, the payables of head 02 and buyers with a جمع balance, each
//!   by the side of its balance.
//!
//! Whatever part of a balance no named list explains (an old amount on an old closed account, a
//! جنرل line on a lent person, a dead person with a جمع balance, which R38 shows as 0) goes to the
//! "other" lists. So **have less give is exactly the sum of the balances**, always. That sum is the
//! books' equity (heads 04, 05, 06) without the linked partner accounts 0605: every posting of
//! 0605 has its contra on 0307, so the two together are 0 and move no money.
//!
//! Example (PRD 4.9, after Events 1 to 5): have 58,484 + 21,564 = 80,048; give 10,160 + 300 +
//! 4,000 + 100 = 14,560; net 65,488 = capital 65,000 + profit 2,488 − drawings 2,000.

use std::collections::{HashMap, HashSet};

use mandi_core::accounts::{BUYER_CONTROL, GARDEN_CONTROL_CODE, KHATA_CONTROL};
use mandi_core::money::Paisa;
use mandi_core::text::UrduCollator;
use mandi_db::Db;
use mandi_db::ledger::account_totals_in_controls;
use mandi_db::rusqlite::Connection;

use super::lent::LentFacts;
use super::narrow::cash_account;
use super::total_worth_model::{
    TotalWorthInput, TotalWorthPage, TotalWorthPart, TotalWorthPrintInput, TotalWorthRow,
    TotalWorthSide, TotalWorthSource, TotalWorthTile,
};
use super::{Facts, lent};
use crate::error::ReportsError;
use crate::reports::collections::dues::buyers_balances_now;
use crate::reports::collections::payables::{Seller, candidates};
use crate::reports::common::{add, in_snapshot, load_parties, paisa, today as today_of};
use crate::reports::filters::Typed;
use crate::reports::{
    Cell, Column, ColumnKind, PartyRef, Row, RowKind, TablePrintData, TableSpec, table_print_data,
};

/// The rows a page holds when no limit is sent, and the most one page holds.
const MAX_PAGE: usize = 100;
/// The controls of the heads that hold money: assets, payables and parties, without the linked
/// money contra 0307 (it is only the other side of the partners' 0605).
const CONTROLS_SQL: &str = "SELECT k.code, h.code FROM account_controls k
    JOIN account_heads h ON h.id = k.head_id
    WHERE h.code IN ('01', '02', '03') AND k.code <> '0307' ORDER BY k.code";
/// The head of the payables (مزدوری, کرایہ, کرایہ سٹور, منشیانہ).
const PAYABLE_HEAD: &str = "02";

/// One amount of the working: above 0, except the cash which keeps its sign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) source: TotalWorthSource,
    pub(crate) account_id: i64,
    pub(crate) amount: Paisa,
    /// The party when the list it comes from gives it already (the buyers and sellers).
    pub(crate) party: Option<PartyRef>,
}

/// The whole working.
#[derive(Debug, Clone, Default)]
pub(crate) struct Working {
    pub(crate) items: Vec<Item>,
}

impl Working {
    /// The two section totals and the net.
    pub(crate) fn tile(&self) -> Result<TotalWorthTile, ReportsError> {
        let (mut have, mut give) = (Paisa::ZERO, Paisa::ZERO);
        for item in &self.items {
            match item.source.side() {
                TotalWorthSide::Have => add(&mut have, item.amount)?,
                TotalWorthSide::Give => add(&mut give, item.amount)?,
            }
        }
        Ok(TotalWorthTile {
            have_paisa: have,
            give_paisa: give,
            net_paisa: have.checked_sub(give)?,
        })
    }

    /// Each source with rows and its total, in the order of the page.
    fn parts(&self) -> Result<Vec<TotalWorthPart>, ReportsError> {
        let mut parts = Vec::new();
        for source in TotalWorthSource::ALL {
            let (mut total, mut count) = (Paisa::ZERO, 0_i64);
            for item in self.items.iter().filter(|i| i.source == source) {
                add(&mut total, item.amount)?;
                count = count
                    .checked_add(1)
                    .ok_or_else(|| ReportsError::Data("the rows do not fit".to_owned()))?;
            }
            if count > 0 {
                parts.push(TotalWorthPart {
                    source,
                    side: source.side(),
                    total_paisa: total,
                    count,
                });
            }
        }
        Ok(parts)
    }
}

/// What the named lists already say about each account, signed: money to have above 0.
#[derive(Default)]
struct Explained {
    by_account: HashMap<i64, Paisa>,
    items: Vec<Item>,
}

impl Explained {
    /// Adds an amount of a named list. An amount of 0 is no row.
    fn put(
        &mut self,
        source: TotalWorthSource,
        account_id: i64,
        amount: Paisa,
        party: Option<PartyRef>,
    ) -> Result<(), ReportsError> {
        if amount == Paisa::ZERO && source != TotalWorthSource::Cash {
            return Ok(());
        }
        let signed = match source.side() {
            TotalWorthSide::Have => amount,
            TotalWorthSide::Give => amount.checked_neg()?,
        };
        add(
            self.by_account.entry(account_id).or_insert(Paisa::ZERO),
            signed,
        )?;
        self.items.push(Item {
            source,
            account_id,
            amount,
            party,
        });
        Ok(())
    }
}

/// The accounts whose جمع balance is not a buyer's credit: the old closed accounts of چکتا.
fn chukta_accounts(conn: &Connection) -> Result<HashSet<i64>, ReportsError> {
    let mut statement = conn.prepare("SELECT id FROM accounts WHERE is_chukta_khata = 1")?;
    let ids = statement
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<Result<HashSet<_>, _>>()?;
    Ok(ids)
}

/// Where the part of a balance that no named list explains goes. `left` is not 0.
fn unexplained_source(
    control: &str,
    head: &str,
    left: Paisa,
    plain_party: bool,
) -> TotalWorthSource {
    let owes_us = !left.is_negative();
    match control {
        BUYER_CONTROL if !owes_us && plain_party => TotalWorthSource::BuyersCredit,
        KHATA_CONTROL if owes_us => TotalWorthSource::KhataMinus,
        KHATA_CONTROL => TotalWorthSource::KhataPlus,
        GARDEN_CONTROL_CODE if owes_us => TotalWorthSource::GardenMinus,
        GARDEN_CONTROL_CODE => TotalWorthSource::GardenPlus,
        _ if head == PAYABLE_HEAD && !owes_us => TotalWorthSource::Payables,
        _ if owes_us => TotalWorthSource::OtherHave,
        _ => TotalWorthSource::OtherGive,
    }
}

/// The working as of now, from the lists the other tiles are made of. `facts` and `lent` must be
/// of the same snapshot as `conn`, and `cash` the balance of the cash account there.
pub(crate) fn working(
    conn: &Connection,
    facts: &Facts,
    lent: &LentFacts,
    cash: Paisa,
) -> Result<Working, ReportsError> {
    working_from(conn, &facts.buyers, &facts.sellers, lent, cash)
}

fn working_from(
    conn: &Connection,
    buyers: &[crate::reports::collections::dues::Buyer],
    sellers: &[Seller],
    lent: &LentFacts,
    cash: Paisa,
) -> Result<Working, ReportsError> {
    let mut explained = Explained::default();
    let cash_id = cash_account(conn)?;
    explained.put(TotalWorthSource::Cash, cash_id, cash, None)?;
    for buyer in buyers {
        explained.put(
            TotalWorthSource::BuyersDues,
            buyer.party.account_id,
            buyer.balance,
            Some(buyer.party.clone()),
        )?;
    }
    for seller in sellers {
        let Some(w) = seller.working else { continue };
        let net = w.net()?;
        let id = seller.party.account_id;
        let party = || Some(seller.party.clone());
        explained.put(TotalWorthSource::SellersDues, id, w.owes_firm, party())?;
        explained.put(
            TotalWorthSource::OpenAdvances,
            id,
            net.uncovered_advance,
            party(),
        )?;
        explained.put(TotalWorthSource::SellersPayable, id, net.payable, party())?;
    }
    for (account_id, still_owed) in crate::dead::still_owed_by_account(conn)? {
        explained.put(TotalWorthSource::DeadAmount, account_id, still_owed, None)?;
    }
    for person in &lent.people {
        explained.put(
            TotalWorthSource::MoneyLent,
            person.account_id,
            person.open,
            None,
        )?;
    }

    // The balance of every account that holds money, and what of it no list explains.
    let controls = {
        let mut statement = conn.prepare(CONTROLS_SQL)?;
        statement
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?
            .collect::<Result<Vec<_>, _>>()?
    };
    let chukta = chukta_accounts(conn)?;
    let dead = mandi_db::dead::dead_set(conn, None)?;
    let Explained {
        mut by_account,
        mut items,
    } = explained;
    for (control, head) in &controls {
        // One control at a time: the buyers' and sellers' sums are the kept ones of R20 and R24.
        for (account_id, totals) in account_totals_in_controls(conn, &[control.as_str()], None)? {
            let balance = paisa(totals.dr_paisa)?.checked_sub(paisa(totals.cr_paisa)?)?;
            let named = by_account.remove(&account_id).unwrap_or(Paisa::ZERO);
            let left = balance.checked_sub(named)?;
            if left == Paisa::ZERO {
                continue;
            }
            let plain_party = !chukta.contains(&account_id) && !dead.has_account(account_id);
            let source = unexplained_source(control, head, left, plain_party);
            let amount = match source.side() {
                TotalWorthSide::Have => left,
                TotalWorthSide::Give => left.checked_neg()?,
            };
            items.push(Item {
                source,
                account_id,
                amount,
                party: None,
            });
        }
    }
    // An account with no line at all has a balance of 0, so no list may say anything about it.
    if let Some((account_id, _)) = by_account
        .iter()
        .find(|(id, amount)| **amount != Paisa::ZERO && **id != cash_id)
    {
        return Err(ReportsError::Data(format!(
            "account {account_id} is in a list but has no lines"
        )));
    }
    Ok(Working { items })
}

/// The working read in its own snapshot, as the dashboard reads it.
fn load(db: &Db) -> Result<(String, Working, HashMap<i64, PartyRef>), ReportsError> {
    db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let today_iso = today.to_iso();
            let buyers = buyers_balances_now(conn, today)?;
            let typed = Typed {
                today: Some(today),
                ..Typed::default()
            };
            let (sellers, _) = candidates(conn, &typed, &today_iso, true)?;
            let lent = lent::facts(conn, today)?;
            let cash = super::narrow::cash_of(conn)?;
            let working = working_from(conn, &buyers, &sellers, &lent, cash)?;
            let ids: Vec<i64> = working
                .items
                .iter()
                .filter(|i| i.party.is_none())
                .map(|i| i.account_id)
                .collect();
            let parties = load_parties(conn, &ids)?;
            Ok((today_iso, working, parties))
        })
    })
}

/// The rows of one side, by source in the order of the page, then by Urdu name.
fn rows_of(
    working: &Working,
    parties: &HashMap<i64, PartyRef>,
    side: TotalWorthSide,
) -> Result<Vec<TotalWorthRow>, ReportsError> {
    let mut rows = Vec::new();
    for item in working.items.iter().filter(|i| i.source.side() == side) {
        let party = match &item.party {
            Some(party) => party.clone(),
            None => parties.get(&item.account_id).cloned().ok_or_else(|| {
                ReportsError::Data(format!("account {} is missing", item.account_id))
            })?,
        };
        rows.push(TotalWorthRow {
            source: item.source,
            party,
            amount_paisa: item.amount,
        });
    }
    let collator = UrduCollator::new().ok();
    rows.sort_by(|a, b| {
        a.source.cmp(&b.source).then_with(|| match &collator {
            Some(c) => c.compare(&a.party.name_ur, &b.party.name_ur),
            None => a.party.name_ur.cmp(&b.party.name_ur),
        })
    });
    Ok(rows)
}

/// One page of the rows of one side, with every source's total and the net.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `limit`; a database or money error.
pub fn total_worth(db: &Db, input: &TotalWorthInput) -> Result<TotalWorthPage, ReportsError> {
    let limit = match usize::try_from(input.limit).unwrap_or(usize::MAX) {
        0 => MAX_PAGE,
        n if n <= MAX_PAGE => n,
        _ => {
            return Err(ReportsError::Rejected(vec![
                mandi_core::error::FieldError::new(
                    mandi_core::error::ErrorCode::ErrOutOfRange,
                    "limit",
                ),
            ]));
        }
    };
    let (today, working, parties) = load(db)?;
    let rows = rows_of(&working, &parties, input.side)?;
    let total_count = i64::try_from(rows.len())
        .map_err(|_| ReportsError::Data("the rows do not fit".to_owned()))?;
    let offset = usize::try_from(input.offset).unwrap_or(usize::MAX);
    let rows = rows.into_iter().skip(offset).take(limit).collect();
    Ok(TotalWorthPage {
        today,
        side: input.side,
        rows,
        total_count,
        offset: input.offset,
        limit: u32::try_from(limit).unwrap_or(u32::MAX),
        parts: working.parts()?,
        totals: working.tile()?,
    })
}

/// The Fluent id of a source, for example `total-worth-source-buyers-dues`.
fn source_label(source: TotalWorthSource) -> String {
    let key = match source {
        TotalWorthSource::Cash => "cash",
        TotalWorthSource::BuyersDues => "buyers-dues",
        TotalWorthSource::SellersDues => "sellers-dues",
        TotalWorthSource::DeadAmount => "dead-amount",
        TotalWorthSource::MoneyLent => "money-lent",
        TotalWorthSource::OpenAdvances => "open-advances",
        TotalWorthSource::KhataMinus => "khata-minus",
        TotalWorthSource::GardenMinus => "garden-minus",
        TotalWorthSource::OtherHave => "other-have",
        TotalWorthSource::SellersPayable => "sellers-payable",
        TotalWorthSource::Payables => "payables",
        TotalWorthSource::BuyersCredit => "buyers-credit",
        TotalWorthSource::KhataPlus => "khata-plus",
        TotalWorthSource::GardenPlus => "garden-plus",
        TotalWorthSource::OtherGive => "other-give",
    };
    format!("total-worth-source-{key}")
}

/// The table of the whole page, for the print and the Excel sheet: money to have, each source with
/// its total and its parties, then money to give the same way; the totals row is the net, with the
/// two section totals under it.
///
/// # Errors
/// A database or money error.
pub fn total_worth_spec(db: &Db) -> Result<TableSpec, ReportsError> {
    let (_, working, parties) = load(db)?;
    let totals = working.tile()?;
    let parts = working.parts()?;
    let columns = vec![
        Column {
            key: "party".to_owned(),
            title_id: "print-total-worth-col-party".to_owned(),
            kind: ColumnKind::Party,
        },
        Column {
            key: "amount".to_owned(),
            title_id: "print-total-worth-col-amount".to_owned(),
            kind: ColumnKind::Money,
        },
    ];
    let line = |id: String, paisa: Paisa, kind: RowKind| Row {
        cells: vec![Cell::Label { id }, Cell::Money { paisa }],
        source: None,
        kind,
    };
    let mut rows = Vec::new();
    let mut count = 0_i64;
    for (side, side_id, side_total) in [
        (
            TotalWorthSide::Have,
            "total-worth-side-have",
            totals.have_paisa,
        ),
        (
            TotalWorthSide::Give,
            "total-worth-side-give",
            totals.give_paisa,
        ),
    ] {
        rows.push(line(side_id.to_owned(), side_total, RowKind::Group));
        let side_rows = rows_of(&working, &parties, side)?;
        for part in parts.iter().filter(|p| p.side == side) {
            rows.push(line(
                source_label(part.source),
                part.total_paisa,
                RowKind::Group,
            ));
            for row in side_rows.iter().filter(|r| r.source == part.source) {
                count = count
                    .checked_add(1)
                    .ok_or_else(|| ReportsError::Data("the rows do not fit".to_owned()))?;
                rows.push(Row {
                    cells: vec![
                        Cell::Party {
                            party: row.party.clone(),
                        },
                        Cell::Money {
                            paisa: row.amount_paisa,
                        },
                    ],
                    source: None,
                    kind: RowKind::Normal,
                });
            }
        }
    }
    Ok(TableSpec {
        title_id: "print-total-worth-title".to_owned(),
        facts: Vec::new(),
        column_names: vec![None; columns.len()],
        columns,
        rows,
        totals_label_id: "total-worth-net".to_owned(),
        totals: vec![
            Cell::Empty,
            Cell::Money {
                paisa: totals.net_paisa,
            },
        ],
        total_count: count,
        words_of: Some(totals.net_paisa).filter(|net| *net > Paisa::ZERO),
        plain_sale: false,
        extra_totals: Vec::new(),
    })
}

/// The print of the same table, in the print language.
///
/// # Errors
/// As [`total_worth_spec`]; a stored amount that cannot be written in words.
pub fn total_worth_print_data(
    db: &Db,
    input: &TotalWorthPrintInput,
) -> Result<TablePrintData, ReportsError> {
    let spec = total_worth_spec(db)?;
    table_print_data(db, input.language, input.paper, spec)
}
