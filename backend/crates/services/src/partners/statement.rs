//! The partner statement (R31 data, PRD FR-17.9, FR-17.10, plan 17): one dated line for every
//! amount that reaches a partner, with the running کل رقم (C22), made only from posted records.
//!
//! # Where the lines come from
//!
//! | Source | Heading | What it is |
//! |---|---|---|
//! | his share rows | `bikri`, `expense` | income and expense shares of a posting |
//! | his capital account (0602) | `capital` | opening, invested, taken back |
//! | his drawings account (0603) | `drawings` | money taken and returned, an opening debit |
//! | his gardens, trips and fruit account (0604) | `trip`, `garden`, `fruit` | his part paid from the shop's cash, money credited to him |
//! | the stored parts of a trip lot's credit | `trip` | his part of a seller credit (one line for each lot) |
//! | month close, a trip write-off, a trip closing | `capital`, `trip` | note lines with no amount |
//!
//! The sum of the amounts up to a day is exactly [`money::total_of`] on that day: the same
//! records, the same exclusion of month close. A test checks that the last running total equals
//! his کل رقم, on the worked day and on random postings.
//!
//! # How a page is made
//!
//! The events are light (a day, an entry, an amount). They are sorted, the running total is
//! worked out for all of them with checked maths, and only the lines of the page are worded, from
//! a handful of grouped reads (Code Standards B22, B31, NFR-SPD-24). A note line sorts after the
//! amounts of its day.
//!
//! Worked numbers (PRD D4-A33, the E2E-34 run, partner الف): -4,160 (his shop part of purchase
//! خ-1), +480 (his part of the commission 1,200), +3,384 (his 40% of the seller credit 8,460),
//! ending at -296.

use std::collections::{BTreeMap, HashMap};

use mandi_db::partners::{self as repo, PartnerRecord};
use mandi_db::rusqlite::Connection;
use mandi_db::statement as read;
use mandi_db::ventures as venture_repo;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::statement::{
    PartnerStatement, PartnersStatementInput, StatementDetail, StatementHeading, StatementLine,
    StatementTotals,
};
use mandi_domain::core::validate::{self, FieldErrors};

use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::page;
use crate::ventures::record_of;

/// Notes sort after the amounts of their day. Month close, then a write-off, then a closing.
const NOTE_MONTH_CLOSE: i64 = i64::MAX - 3;
const NOTE_WRITE_OFF: i64 = i64::MAX - 2;
const NOTE_TRIP_CLOSED: i64 = i64::MAX - 1;
const NOTE_GARDEN_CLOSED: i64 = i64::MAX;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// What kind of event a line is made from.
#[derive(Debug, Clone)]
enum Kind {
    Share,
    Capital,
    Drawings,
    Venture,
    /// His part of a seller credit of a trip lot.
    LotCredit {
        lot_id: i64,
        share_bps: i64,
    },
    /// A journal credit on his 0604 account from a بکری that the stored parts do not explain.
    Remainder,
    MonthClose,
    WriteOff {
        venture_id: i64,
    },
    TripClosed {
        venture_id: i64,
    },
    /// A garden closing that posted nothing for him (PRD FR-17.10, E64): a note with no amount.
    GardenClosed {
        venture_entry_id: i64,
        venture_id: i64,
    },
}

/// One event before it is worded: where it sorts, and its amount.
#[derive(Debug, Clone)]
struct Event {
    date: String,
    /// The entry it belongs to (the sort place of a note).
    entry_id: i64,
    /// Share rows first, then capital, drawings, and last the trip money.
    group: u8,
    id: i64,
    amount: Option<Paisa>,
    kind: Kind,
}

/// Every event of a partner, unsorted.
fn events_of(conn: &Connection, partner: &PartnerRecord) -> Result<Vec<Event>, ServiceError> {
    let mut events = Vec::new();
    for row in read::share_events(conn, partner.id)? {
        events.push(Event {
            date: row.entry_date,
            entry_id: row.entry_id,
            group: 0,
            id: row.id,
            amount: Some(paisa(row.amount_paisa)?),
            kind: Kind::Share,
        });
    }
    for (account, group, kind) in [
        (partner.capital_account_id, 1, Kind::Capital),
        (partner.drawings_account_id, 2, Kind::Drawings),
    ] {
        for line in read::line_events(conn, account)? {
            events.push(Event {
                date: line.entry_date,
                entry_id: line.entry_id,
                group,
                id: line.id,
                amount: Some(
                    paisa(line.cr_paisa)?
                        .checked_sub(paisa(line.dr_paisa)?)
                        .map_err(money_error)?,
                ),
                kind: kind.clone(),
            });
        }
    }
    // The trip money. A بکری posting is explained by the stored parts of its lots (one line for
    // each lot); anything the parts do not explain is kept as one line, so the running total
    // always equals his کل رقم.
    let mut part_sums: HashMap<i64, Paisa> = HashMap::new();
    for part in read::lot_credit_events(conn, partner.id)? {
        let amount = paisa(part.amount_paisa)?;
        let slot = part_sums.entry(part.entry_id).or_insert(Paisa::ZERO);
        *slot = slot.checked_add(amount).map_err(money_error)?;
        events.push(Event {
            date: part.entry_date,
            entry_id: part.entry_id,
            group: 3,
            id: part.id,
            amount: Some(amount),
            kind: Kind::LotCredit {
                lot_id: part.lot_id,
                share_bps: part.share_bps,
            },
        });
    }
    if partner.venture_account_id != 0 {
        let mut bikri_net: BTreeMap<i64, (String, i64, Paisa)> = BTreeMap::new();
        for line in read::line_events(conn, partner.venture_account_id)? {
            let amount = paisa(line.cr_paisa)?
                .checked_sub(paisa(line.dr_paisa)?)
                .map_err(money_error)?;
            if line.source_type == "bikri" {
                let slot = bikri_net.entry(line.entry_id).or_insert((
                    line.entry_date.clone(),
                    line.id,
                    Paisa::ZERO,
                ));
                slot.2 = slot.2.checked_add(amount).map_err(money_error)?;
            } else {
                events.push(Event {
                    date: line.entry_date,
                    entry_id: line.entry_id,
                    group: 3,
                    id: line.id,
                    amount: Some(amount),
                    kind: Kind::Venture,
                });
            }
        }
        for (entry_id, (date, id, net)) in bikri_net {
            let explained = part_sums.get(&entry_id).copied().unwrap_or(Paisa::ZERO);
            let rest = net.checked_sub(explained).map_err(money_error)?;
            if rest != Paisa::ZERO {
                events.push(Event {
                    date,
                    entry_id,
                    group: 4,
                    id,
                    amount: Some(rest),
                    kind: Kind::Remainder,
                });
            }
        }
    }
    for note in read::month_close_notes(conn, partner.capital_account_id)? {
        events.push(Event {
            date: note.date,
            entry_id: NOTE_MONTH_CLOSE,
            group: 5,
            id: note.id,
            amount: None,
            kind: Kind::MonthClose,
        });
    }
    for note in read::write_off_notes(conn, partner.id)? {
        events.push(Event {
            date: note.date,
            entry_id: NOTE_WRITE_OFF,
            group: 5,
            id: note.id,
            amount: None,
            kind: Kind::WriteOff {
                venture_id: note.venture_id,
            },
        });
    }
    for note in read::closed_trip_notes(conn, partner.id)? {
        events.push(Event {
            date: note.date,
            entry_id: NOTE_TRIP_CLOSED,
            group: 5,
            id: note.id,
            amount: None,
            kind: Kind::TripClosed {
                venture_id: note.venture_id,
            },
        });
    }
    for note in read::unposted_garden_close_notes(conn, partner.id)? {
        events.push(Event {
            date: note.date,
            entry_id: NOTE_GARDEN_CLOSED,
            group: 5,
            id: note.id,
            amount: None,
            kind: Kind::GardenClosed {
                venture_entry_id: note.id,
                venture_id: note.venture_id,
            },
        });
    }
    events.sort_by(|a, b| {
        (a.date.as_str(), a.entry_id, a.group, a.id).cmp(&(
            b.date.as_str(),
            b.entry_id,
            b.group,
            b.id,
        ))
    });
    Ok(events)
}

/// The facts that word the lines of one page, read in a few grouped queries.
#[derive(Default)]
struct Facts {
    entries: HashMap<i64, read::EntryInfo>,
    vouchers: HashMap<i64, read::VoucherInfo>,
    bikri: HashMap<i64, i64>,
    fruit: HashMap<i64, i64>,
    purchases: HashMap<i64, read::PurchaseInfo>,
    venture_entries: HashMap<i64, read::VentureEntryInfo>,
    ventures: HashMap<i64, (i64, String)>,
    lots: HashMap<i64, read::LotInfo>,
    sold: HashMap<(i64, i64), (i64, i64)>,
    credit_totals: HashMap<(i64, i64), i64>,
    part_shares: HashMap<i64, i64>,
    garden_closes: HashMap<i64, read::GardenCloseFigures>,
}

fn named(n: &read::Named) -> RecordRef {
    RecordRef {
        id: n.0,
        code: n.1.clone(),
        name_ur: n.2.clone(),
        name_en: n.3.clone(),
    }
}

fn sorted(mut ids: Vec<i64>) -> Vec<i64> {
    ids.sort_unstable();
    ids.dedup();
    ids
}

/// Reads the facts for the events of one page.
fn facts_for(conn: &Connection, partner_id: i64, page: &[&Event]) -> Result<Facts, ServiceError> {
    let entry_ids = sorted(
        page.iter()
            .filter(|e| e.entry_id < NOTE_MONTH_CLOSE)
            .map(|e| e.entry_id)
            .collect(),
    );
    let entries = read::entry_infos(conn, &entry_ids)?;
    let by_source = |source: &str| -> Vec<i64> {
        sorted(
            entries
                .values()
                .filter(|i| i.source_type == source)
                .map(|i| i.source_id)
                .collect(),
        )
    };
    let vouchers = read::voucher_infos(conn, &by_source("voucher"))?;
    let bikri = read::bikri_numbers(conn, &by_source("bikri"))?;
    let fruit = read::fruit_numbers(conn, &by_source("fruit"))?;
    let venture_entries = read::venture_entry_infos(conn, &by_source("venture"))?;
    let lot_ids = sorted(
        page.iter()
            .filter_map(|e| match e.kind {
                Kind::LotCredit { lot_id, .. } => Some(lot_id),
                _ => None,
            })
            .collect(),
    );
    let lots = read::lot_infos(conn, &lot_ids)?;
    let mut purchase_ids = by_source("purchase");
    purchase_ids.extend(lots.values().filter_map(|l| l.purchase_id));
    purchase_ids.extend(venture_entries.values().filter_map(|v| v.purchase_id));
    let purchases = read::purchase_infos(conn, &sorted(purchase_ids))?;
    let mut venture_ids: Vec<i64> = purchases.values().map(|p| p.venture_id).collect();
    venture_ids.extend(venture_entries.values().map(|v| v.venture_id));
    let mut note_closes = Vec::new();
    for event in page {
        match event.kind {
            Kind::WriteOff { venture_id } | Kind::TripClosed { venture_id } => {
                venture_ids.push(venture_id);
            }
            Kind::GardenClosed {
                venture_entry_id,
                venture_id,
            } => {
                venture_ids.push(venture_id);
                note_closes.push(venture_entry_id);
            }
            _ => {}
        }
    }
    let ventures = read::venture_numbers(conn, &sorted(venture_ids))?;
    // The lines of a lot were sold in the posting itself, or, for a reversal, in the posting it
    // took back (PRD C21: the cost is stored with the sale line).
    let posting_ids = sorted(
        page.iter()
            .filter(|e| matches!(e.kind, Kind::LotCredit { .. }))
            .filter_map(|e| entries.get(&e.entry_id))
            .map(|i| i.reverses_entry_id.unwrap_or(i.entry_id))
            .collect(),
    );
    let sold = read::lot_sold_figures(conn, &posting_ids)?;
    let credit_totals = read::lot_credit_totals(conn, &entry_ids)?;
    let part_entries = sorted(
        venture_entries
            .values()
            .filter(|v| v.kind == "other_sale")
            .map(|v| v.id)
            .collect(),
    );
    let part_shares = read::entry_part_shares(conn, partner_id, &part_entries)?;
    let mut close_entries: Vec<i64> = venture_entries
        .values()
        .filter(|v| v.kind == "close")
        .map(|v| v.id)
        .collect();
    close_entries.extend(note_closes);
    let close_entries = sorted(close_entries);
    let garden_closes = read::garden_close_figures(conn, partner_id, &close_entries)?;
    Ok(Facts {
        entries,
        vouchers,
        bikri,
        fruit,
        purchases,
        venture_entries,
        ventures,
        lots,
        sold,
        credit_totals,
        part_shares,
        garden_closes,
    })
}

/// A line under construction.
struct Draft {
    heading: StatementHeading,
    detail: StatementDetail,
    number: Option<i64>,
    secondary: Option<i64>,
    subject: Option<RecordRef>,
    /// The packing of a fruit line (PRD FR-17.10).
    packing: Option<RecordRef>,
    reason: String,
    values: BTreeMap<String, ErrorValue>,
}

impl Draft {
    fn new(heading: StatementHeading, detail: StatementDetail) -> Draft {
        Draft {
            heading,
            detail,
            number: None,
            secondary: None,
            subject: None,
            packing: None,
            reason: String::new(),
            values: BTreeMap::new(),
        }
    }

    fn kind(mut self, kind: &str) -> Draft {
        self.values
            .insert("kind".to_owned(), ErrorValue::Text(kind.to_owned()));
        self
    }

    fn number(mut self, number: Option<i64>) -> Draft {
        self.number = number;
        self
    }

    fn number_value(mut self, name: &str, value: i64) -> Draft {
        self.values
            .insert(name.to_owned(), ErrorValue::Number(value));
        self
    }
}

fn is_negative(amount: Option<Paisa>) -> bool {
    amount.is_some_and(Paisa::is_negative)
}

/// The trip number of a purchase.
fn trip_number(facts: &Facts, purchase_id: Option<i64>) -> Option<i64> {
    let purchase = facts.purchases.get(&purchase_id?)?;
    facts.ventures.get(&purchase.venture_id).map(|v| v.0)
}

/// The voucher behind an entry, if it has one.
fn voucher_of<'a>(facts: &'a Facts, info: &read::EntryInfo) -> Option<&'a read::VoucherInfo> {
    if info.source_type == "voucher" {
        facts.vouchers.get(&info.source_id)
    } else {
        None
    }
}

/// Words one amount event.
#[allow(clippy::too_many_lines)]
fn draft_of(
    conn: &Connection,
    partner: &PartnerRecord,
    facts: &Facts,
    event: &Event,
) -> Result<Draft, ServiceError> {
    use StatementDetail as D;
    use StatementHeading as H;
    let info = facts.entries.get(&event.entry_id);
    // A reversal is worded like the posting it takes back; its amount is the opposite.
    let effective = info.map(|i| i.original_type.as_deref().unwrap_or(&i.posting_type));
    let voucher = info.and_then(|i| voucher_of(facts, i));
    let negative = is_negative(event.amount);
    Ok(match &event.kind {
        Kind::Share => {
            let info = info.ok_or_else(|| data("a share row has no entry"))?;
            match (info.source_type.as_str(), voucher) {
                ("bikri", _) => Draft::new(H::Bikri, D::IncomeShare)
                    .number(facts.bikri.get(&info.source_id).copied()),
                (_, Some(v)) => {
                    let (detail, kind) = match v.voucher_type.as_str() {
                        "shop_expense" => (D::ExpenseShare, "shop_expense"),
                        "money_lent_return" => (D::ExpenseShare, "salary"),
                        "general" if negative => (D::ExpenseShare, "general"),
                        "general" => (D::IncomeShare, "general"),
                        _ => (D::ExpenseShare, "other"),
                    };
                    let mut draft = Draft::new(H::Expense, detail)
                        .number(Some(v.number))
                        .kind(kind);
                    draft.subject = v.head.as_ref().map(named);
                    draft.reason.clone_from(&v.reason);
                    draft
                }
                _ => Draft::new(H::Expense, D::ExpenseShare),
            }
        }
        Kind::Capital => {
            let opening = effective == Some("opening");
            if opening {
                Draft::new(H::Capital, D::CapitalOpening)
            } else {
                let (detail, kind) = match voucher.map(|v| v.voucher_type.as_str()) {
                    Some("capital_in") => (D::CapitalIn, "capital_in"),
                    Some("capital_out") => (D::CapitalOut, "capital_out"),
                    _ if negative => (D::CapitalOut, "general"),
                    _ => (D::CapitalIn, "general"),
                };
                Draft::new(H::Capital, detail)
                    .number(voucher.map(|v| v.number))
                    .kind(kind)
            }
        }
        Kind::Drawings => {
            if effective == Some("opening") {
                Draft::new(H::Drawings, D::DrawingsOpening)
            } else {
                let (detail, kind) = match voucher.map(|v| v.voucher_type.as_str()) {
                    Some("personal_expense") => (D::Taken, "personal_expense"),
                    Some("personal_return") => (D::Returned, "personal_return"),
                    _ if negative => (D::Taken, "general"),
                    _ => (D::Returned, "general"),
                };
                let mut draft = Draft::new(H::Drawings, detail)
                    .number(voucher.map(|v| v.number))
                    .kind(kind);
                if let Some(v) = voucher {
                    draft.reason.clone_from(&v.reason);
                }
                draft
            }
        }
        Kind::Venture => venture_draft(facts, info, effective, voucher, negative)?,
        Kind::LotCredit { lot_id, share_bps } => {
            let info = info.ok_or_else(|| data("a trip credit has no entry"))?;
            let lot = facts
                .lots
                .get(lot_id)
                .ok_or_else(|| data("a trip credit names no lot"))?;
            let sign: i64 = if info.reverses_entry_id.is_some() {
                -1
            } else {
                1
            };
            let posting = info.reverses_entry_id.unwrap_or(info.entry_id);
            let (qty, cost) = facts
                .sold
                .get(&(posting, *lot_id))
                .copied()
                .unwrap_or((0, 0));
            let sales = facts
                .credit_totals
                .get(&(info.entry_id, *lot_id))
                .copied()
                .unwrap_or(0);
            let signed = |value: i64| -> Result<i64, ServiceError> {
                value
                    .checked_mul(sign)
                    .ok_or_else(|| data("a figure does not fit"))
            };
            let (qty, cost) = (signed(qty)?, signed(cost)?);
            let result = sales
                .checked_sub(cost)
                .ok_or_else(|| data("a figure does not fit"))?;
            let mut draft = Draft::new(H::Trip, D::TripCredit)
                .number(trip_number(facts, lot.purchase_id))
                .number_value("qty", qty)
                .number_value("sales_paisa", sales)
                .number_value("cost_paisa", cost)
                .number_value("result_paisa", result)
                .number_value("share_bps", *share_bps);
            draft.secondary = Some(lot.bikri_number);
            draft.subject = Some(named(&lot.item));
            draft
        }
        Kind::Remainder => Draft::new(H::Trip, D::TripCredit).kind("other"),
        Kind::MonthClose => Draft::new(H::Capital, D::MonthClose),
        Kind::WriteOff { venture_id } => {
            Draft::new(H::Trip, D::TripWriteOff).number(facts.ventures.get(venture_id).map(|v| v.0))
        }
        Kind::TripClosed { venture_id } => trip_closed_draft(conn, partner, facts, *venture_id)?,
        // A closing that posted nothing for him: the same words and figures, with no amount.
        Kind::GardenClosed {
            venture_entry_id,
            venture_id,
        } => {
            let mut draft = Draft::new(H::Garden, D::GardenClosed)
                .number(facts.ventures.get(venture_id).map(|v| v.0));
            if let Some(figures) = facts.garden_closes.get(venture_entry_id) {
                draft = garden_close_values(draft, figures, 1)?;
            }
            draft
        }
    })
}

/// The numbers of a partner's closing line (PRD FR-17.10), from stored rows only: the total cost,
/// the sales money (the closing parts together), the result, his %, his part of the result (his
/// money back less what he paid) and his money back. `sign` is -1 for the line of a reversal.
fn garden_close_values(
    draft: Draft,
    figures: &read::GardenCloseFigures,
    sign: i64,
) -> Result<Draft, ServiceError> {
    let signed = |value: i64| -> Result<i64, ServiceError> {
        value
            .checked_mul(sign)
            .ok_or_else(|| data("a figure does not fit"))
    };
    let result = figures
        .sales_paisa
        .checked_sub(figures.cost_paisa)
        .ok_or_else(|| data("a figure does not fit"))?;
    let part = figures
        .back_paisa
        .checked_sub(figures.paid_paisa)
        .ok_or_else(|| data("a figure does not fit"))?;
    Ok(draft
        .number_value("cost_paisa", signed(figures.cost_paisa)?)
        .number_value("sales_paisa", signed(figures.sales_paisa)?)
        .number_value("result_paisa", signed(result)?)
        .number_value("share_bps", figures.share_bps)
        .number_value("part_paisa", signed(part)?)
        .number_value("back_paisa", signed(figures.back_paisa)?))
}

/// A line from the journal of his gardens, trips and fruit account that is not a بکری.
fn venture_draft(
    facts: &Facts,
    info: Option<&read::EntryInfo>,
    effective: Option<&str>,
    voucher: Option<&read::VoucherInfo>,
    negative: bool,
) -> Result<Draft, ServiceError> {
    use StatementDetail as D;
    use StatementHeading as H;
    let info = info.ok_or_else(|| data("a trip line has no entry"))?;
    let sign: i64 = if info.reverses_entry_id.is_some() {
        -1
    } else {
        1
    };
    Ok(match effective {
        Some("purchase") => {
            let purchase = facts.purchases.get(&info.source_id);
            let mut draft = Draft::new(H::Trip, D::TripPaid)
                .number(trip_number(facts, Some(info.source_id)))
                .kind("purchase");
            draft.secondary = purchase.map(|p| p.number);
            draft
        }
        Some("trip_cost") => {
            let entry = facts.venture_entries.get(&info.source_id);
            let mut draft = Draft::new(H::Trip, D::TripPaid).kind("cost");
            draft.number = entry
                .and_then(|e| facts.ventures.get(&e.venture_id))
                .map(|v| v.0);
            draft.secondary = entry.map(|e| e.number);
            if let Some(e) = entry {
                draft.reason.clone_from(&e.reason);
            }
            draft
        }
        Some("venture_receipt") => {
            let entry = facts
                .venture_entries
                .get(&info.source_id)
                .ok_or_else(|| data("a trip sale has no entry"))?;
            let signed = |value: i64| -> Result<i64, ServiceError> {
                value
                    .checked_mul(sign)
                    .ok_or_else(|| data("a figure does not fit"))
            };
            let (sales, cost) = (signed(entry.amount_paisa)?, signed(entry.cost_paisa)?);
            let result = sales
                .checked_sub(cost)
                .ok_or_else(|| data("a figure does not fit"))?;
            let mut draft = Draft::new(H::Trip, D::TripCredit)
                .number_value("qty", signed(entry.qty.unwrap_or(0))?)
                .number_value("sales_paisa", sales)
                .number_value("cost_paisa", cost)
                .number_value("result_paisa", result)
                .number_value(
                    "share_bps",
                    facts.part_shares.get(&entry.id).copied().unwrap_or(0),
                );
            draft.number = facts.ventures.get(&entry.venture_id).map(|v| v.0);
            draft.subject = entry
                .purchase_id
                .and_then(|id| facts.purchases.get(&id))
                .map(|p| named(&p.item));
            draft
        }
        Some("garden_payment") => {
            let entry = facts.venture_entries.get(&info.source_id);
            let mut draft = Draft::new(H::Garden, D::GardenPaid);
            draft.number = entry
                .and_then(|e| facts.ventures.get(&e.venture_id))
                .map(|v| v.0);
            draft.secondary = entry.map(|e| e.number);
            if let Some(e) = entry {
                draft.reason.clone_from(&e.reason);
            }
            draft
        }
        Some("garden_close") => {
            let entry = facts.venture_entries.get(&info.source_id);
            let mut draft = Draft::new(H::Garden, D::GardenClosed);
            draft.number = entry
                .and_then(|e| facts.ventures.get(&e.venture_id))
                .map(|v| v.0);
            // The numbers of his closing line (PRD FR-17.10), from stored rows only: the total
            // cost, the sales money (the closing parts together), the result, his %, his part of
            // the result (his money back less what he paid) and his money back. A reversal line
            // shows them with the opposite sign.
            if let Some(figures) = entry.and_then(|e| facts.garden_closes.get(&e.id)) {
                draft = garden_close_values(draft, figures, sign)?;
            }
            draft
        }
        Some("fruit_taken") => {
            Draft::new(H::Fruit, D::FruitTaken).number(facts.fruit.get(&info.source_id).copied())
        }
        other => {
            // A جنرل (or anything else) on the account: worded by its sign.
            let detail = if negative { D::TripPaid } else { D::TripCredit };
            let kind = voucher
                .map(|v| v.voucher_type.as_str())
                .or(other)
                .unwrap_or("other");
            Draft::new(H::Trip, detail).kind(if kind == "general" {
                "general"
            } else {
                "other"
            })
        }
    })
}

/// The closing note of a trip (PRD FR-12.33, FR-17.10): its cost, its sales money, its result, the
/// partner's % and his net. His net is his money back less his money in (money agent, plan 16b
/// finding 3), not `split_by_bps` of the result.
fn trip_closed_draft(
    conn: &Connection,
    partner: &PartnerRecord,
    facts: &Facts,
    venture_id: i64,
) -> Result<Draft, ServiceError> {
    let totals = read::trip_totals(conn, venture_id)?;
    let cost = paisa(totals.cost_paisa)?;
    let sales = paisa(totals.sales_paisa)?
        .checked_add(paisa(totals.other_shop_paisa)?)
        .map_err(money_error)?;
    let result = sales
        .checked_sub(paisa(totals.released_paisa)?)
        .map_err(money_error)?;
    let shares = venture_repo::partners_of(conn, venture_id)?;
    let share = shares
        .iter()
        .find(|s| s.partner_id == partner.id)
        .map_or(0, |s| s.share_bps);
    let money = venture_repo::partner_money(conn, venture_id)?;
    let mine = money.get(&partner.id).copied().unwrap_or_default();
    let lot_back = venture_repo::lot_credits_by_partner(conn, venture_id)?
        .get(&partner.id)
        .copied()
        .unwrap_or(0);
    let back = paisa(mine.back_other_paisa)?
        .checked_add(paisa(lot_back)?)
        .map_err(money_error)?;
    let money_in = paisa(mine.in_shop_paisa)?
        .checked_add(paisa(mine.in_pocket_paisa)?)
        .map_err(money_error)?;
    let net = back.checked_sub(money_in).map_err(money_error)?;
    Ok(
        Draft::new(StatementHeading::Trip, StatementDetail::TripClosed)
            .number(facts.ventures.get(&venture_id).map(|v| v.0))
            .number_value("cost_paisa", cost.as_paisa())
            .number_value("sales_paisa", sales.as_paisa())
            .number_value("result_paisa", result.as_paisa())
            .number_value("share_bps", share)
            .number_value("net_paisa", net.as_paisa()),
    )
}

/// A partner's statement for a range, one page (PRD FR-17.9, FR-17.10, FR-19.11). Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `partner_id`; the date codes on `from` and `to`.
pub fn statement(
    ctx: &Context,
    input: &PartnersStatementInput,
) -> Result<PartnerStatement, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = FieldErrors::default();
    let mut bound = |field: &str, text: Option<&String>| -> Option<String> {
        let text = text.map(|t| t.trim()).filter(|t| !t.is_empty())?;
        errors
            .take(validate::date(field, text, today))
            .map(BusinessDate::to_iso)
    };
    let from = bound("from", input.from.as_ref());
    let to = bound("to", input.to.as_ref());
    errors.finish()?;
    ctx.db.read(|conn| {
        let partner = repo::get(conn, input.partner_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "partner_id"))?;
        let events = events_of(conn, &partner)?;
        // The running total at every event, in checked Paisa.
        let mut opening = Paisa::ZERO;
        let mut running = Paisa::ZERO;
        let mut in_range: Vec<(&Event, Paisa)> = Vec::new();
        let mut totals = StatementTotals {
            plus_paisa: Paisa::ZERO,
            minus_paisa: Paisa::ZERO,
        };
        for event in &events {
            if let Some(amount) = event.amount {
                running = running.checked_add(amount).map_err(money_error)?;
            }
            if from.as_deref().is_some_and(|f| event.date.as_str() < f) {
                opening = running;
                continue;
            }
            if to.as_deref().is_some_and(|t| event.date.as_str() > t) {
                continue;
            }
            match event.amount {
                Some(a) if a.is_negative() => {
                    totals.minus_paisa = totals
                        .minus_paisa
                        .checked_add(a.checked_neg().map_err(money_error)?)
                        .map_err(money_error)?;
                }
                Some(a) => {
                    totals.plus_paisa = totals.plus_paisa.checked_add(a).map_err(money_error)?;
                }
                None => {}
            }
            in_range.push((event, running));
        }
        let closing = in_range.last().map_or(opening, |(_, r)| *r);
        let total_count = i64::try_from(in_range.len()).unwrap_or(i64::MAX);
        let page_rows = page::slice(in_range, input.offset, input.limit);
        let page_events: Vec<&Event> = page_rows.iter().map(|(e, _)| *e).collect();
        let facts = facts_for(conn, partner.id, &page_events)?;
        let mut lines = Vec::with_capacity(page_rows.len());
        for (event, running) in &page_rows {
            let draft = draft_of(conn, &partner, &facts, event)?;
            let entry_id = (event.entry_id < NOTE_MONTH_CLOSE).then_some(event.entry_id);
            lines.push(StatementLine {
                entry_date: event.date.clone(),
                heading: draft.heading,
                detail: draft.detail,
                entry_id: entry_id
                    .or_else(|| (matches!(event.kind, Kind::MonthClose)).then_some(event.id)),
                number: draft.number,
                secondary_number: draft.secondary,
                subject: draft.subject,
                packing: draft.packing,
                reason: draft.reason,
                amount_paisa: event.amount,
                running_paisa: *running,
                values: draft.values,
            });
        }
        Ok(PartnerStatement {
            partner: record_of(partner.id, &partner.name_ur, &partner.name_en),
            opening_total_paisa: opening,
            closing_total_paisa: closing,
            lines,
            total_count,
            totals,
        })
    })
}
