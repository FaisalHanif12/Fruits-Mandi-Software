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
//! The events are light (a day, an entry, an amount). Only the events of the asked range are read
//! (a month page reads its month, a day page its day); what came before the range is one sum, his
//! کل رقم at the end of the day before. The events are sorted, the running total is worked out
//! with checked maths, and only the lines of the page are worded, from a handful of grouped reads
//! (Code Standards B22, B31, NFR-SPD-24). A note line sorts after the amounts of its day.
//!
//! Worked numbers (PRD D4-A33, the E2E-34 run, partner الف): -4,160 (his shop part of purchase
//! خ-1), +480 (his part of the commission 1,200), +3,384 (his 40% of the seller credit 8,460),
//! ending at -296.

use std::collections::{BTreeMap, HashMap};

use mandi_db::advances::Advances;
use mandi_db::partners::{self as repo, PartnerRecord};
use mandi_db::rusqlite::Connection;
use mandi_db::statement as read;
use mandi_db::ventures as venture_repo;
use mandi_domain::core::accounts::RecordRef;
use mandi_domain::core::advances::AdvanceEventKind;
use mandi_domain::core::dates::BusinessDate;
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::months::Month;
use mandi_domain::core::paging::SortDirection;
use mandi_domain::core::statement::{
    PartnerStatement, PartnerStatementDay, PartnerStatementMonth, PartnersStatementDayInput,
    PartnersStatementInput, PartnersStatementMonthInput, StatementColumns, StatementDetail,
    StatementHeading, StatementLine, StatementMonthDay, StatementTotals,
};
use mandi_domain::core::validate::{self, FieldErrors};

use crate::context::Context;
use crate::day;
use crate::error::{ServiceError, money_error};
use crate::page;
use crate::ventures::record_of;

/// Notes sort after the amounts of their day. Month close, then a write-off, then a closing.
const NOTE_LENT_SHARE: i64 = i64::MAX - 4;
const NOTE_MONTH_CLOSE: i64 = i64::MAX - 3;
const NOTE_WRITE_OFF: i64 = i64::MAX - 2;
const NOTE_TRIP_CLOSED: i64 = i64::MAX - 1;
const NOTE_GARDEN_CLOSED: i64 = i64::MAX;
/// Every event with an entry id below this is a real journal entry; from it up are the notes.
const FIRST_NOTE: i64 = NOTE_LENT_SHARE;

fn paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(money_error)
}

fn data(message: &str) -> ServiceError {
    ServiceError::Data(message.to_owned())
}

/// What kind of event a line is made from.
#[derive(Debug, Clone)]
enum Kind {
    /// A share row. `expense` is true for a share of a shop expense.
    Share {
        expense: bool,
    },
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
    /// A line on his 0605 account: money that reached him through a linked khata or loan (plan 76
    /// points 5 and 10).
    Linked,
    /// His part of an advance to a seller, going out, coming back or covered by the seller's
    /// credit (plan 78 B4, PRD FR-17.13).
    Advance {
        change: AdvanceEventKind,
        covered: bool,
        seller: RecordRef,
        amount_paisa: i64,
        share_bps: i32,
    },
    /// A loan linked to the shop was given or returned: his % of it, as a note (no amount).
    LentShare {
        voucher_number: i64,
        given: bool,
        amount_paisa: i64,
        part_paisa: Option<i64>,
        person: read::Named,
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

/// The events of a partner dated inside `days`, in the order of the statement. A month page asks
/// for its month only, so it never reads ten years of share rows to show thirty days (plan 76,
/// NFR-SPD-24); a whole-history page passes [`read::Days::ALL`].
fn events_of(
    conn: &Connection,
    partner: &PartnerRecord,
    days: read::Days<'_>,
    advances: &Advances,
) -> Result<Vec<Event>, ServiceError> {
    let mut events = Vec::new();
    for row in read::share_events(conn, partner.id, days)? {
        events.push(Event {
            date: row.entry_date,
            entry_id: row.entry_id,
            group: 0,
            id: row.id,
            amount: Some(paisa(row.amount_paisa)?),
            kind: Kind::Share {
                expense: row.kind == "expense",
            },
        });
    }
    for (account, group, kind) in [
        (partner.capital_account_id, 1, Kind::Capital),
        (partner.drawings_account_id, 2, Kind::Drawings),
    ] {
        for line in read::line_events(conn, account, days)? {
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
    for part in read::lot_credit_events(conn, partner.id, days)? {
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
        for line in read::line_events(conn, partner.venture_account_id, days)? {
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
    // Plan 76 points 5 and 10: his 0605 account, the money of a khata or loan linked to him or to
    // the shop. One line for each journal line, so the running total always equals his کل رقم.
    if partner.linked_account_id != 0 {
        for line in read::line_events(conn, partner.linked_account_id, days)? {
            events.push(Event {
                date: line.entry_date,
                entry_id: line.entry_id,
                group: 2,
                id: line.id,
                amount: Some(
                    paisa(line.cr_paisa)?
                        .checked_sub(paisa(line.dr_paisa)?)
                        .map_err(money_error)?,
                ),
                kind: Kind::Linked,
            });
        }
    }
    // PRD FR-17.13 (plan 78 B4): his parts of the advances to sellers, out and back, worked out from
    // the advance matching. One line for each, so the running total always equals his کل رقم. The
    // events are read once by the request (R1-4), not for each partner or period.
    for e in advances
        .events
        .iter()
        .filter(|e| e.partner_id == partner.id)
        .filter(|e| days.from.is_none_or(|d| e.date.as_str() >= d))
        .filter(|e| days.to.is_none_or(|d| e.date.as_str() <= d))
    {
        let out = e.kind == AdvanceEventKind::Out;
        let Some(seller) = advances.sellers.get(&e.line_id) else {
            return Err(data("an advance has no seller"));
        };
        events.push(Event {
            date: e.date.clone(),
            entry_id: e.entry_id,
            group: 2,
            id: e.allocation_id.unwrap_or(e.line_id),
            amount: Some(if out {
                e.part.checked_neg().map_err(money_error)?
            } else {
                e.part
            }),
            kind: Kind::Advance {
                change: e.kind,
                covered: e.covered,
                seller: seller.clone(),
                amount_paisa: e.amount.as_paisa(),
                share_bps: e.share_bps,
            },
        });
    }
    // A loan that is the shop's changes no total; his % of it is shown as a note (PRD FR-16.13).
    let notes = read::shop_loan_notes(conn, days)?;
    if !notes.is_empty() {
        let mut shares_by_day: HashMap<String, Vec<(i64, mandi_domain::core::money::Bps)>> =
            HashMap::new();
        for note in notes {
            let part = loan_part(conn, &mut shares_by_day, partner.id, &note)?;
            events.push(Event {
                date: note.entry_date,
                entry_id: NOTE_LENT_SHARE,
                group: 5,
                id: note.voucher_id,
                amount: None,
                kind: Kind::LentShare {
                    voucher_number: note.number,
                    given: note.given,
                    amount_paisa: note.amount_paisa,
                    part_paisa: part,
                    person: note.person,
                },
            });
        }
    }
    for note in read::month_close_notes(conn, partner.capital_account_id, days)? {
        events.push(Event {
            date: note.date,
            entry_id: NOTE_MONTH_CLOSE,
            group: 5,
            id: note.id,
            amount: None,
            kind: Kind::MonthClose,
        });
    }
    for note in read::write_off_notes(conn, partner.id, days)? {
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
    for note in read::closed_trip_notes(conn, partner.id, days)? {
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
    for note in read::unposted_garden_close_notes(conn, partner.id, days)? {
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

/// His part of a loan of the shop: `split_by_bps` of the amount by the shares of the loan's day
/// (PRD C10). None when no share was set that day, or when he had none that day (the note then has
/// no part to show). Any other split error is returned, never swallowed: shares that do not total 100% or an overflow
/// would be a wrong figure on a statement (plan 80 gate R1-6, Principle 4).
fn loan_part(
    conn: &Connection,
    shares_by_day: &mut HashMap<String, Vec<(i64, mandi_domain::core::money::Bps)>>,
    partner_id: i64,
    note: &read::ShopLoanNote,
) -> Result<Option<i64>, ServiceError> {
    use mandi_domain::core::money::{Bps, split_by_bps};
    if !shares_by_day.contains_key(&note.entry_date) {
        let shares = repo::shares_on(conn, &note.entry_date)?
            .into_iter()
            .map(|(id, bps)| {
                i32::try_from(bps)
                    .ok()
                    .and_then(|b| Bps::new(b).ok())
                    .map(|b| (id, b))
                    .ok_or_else(|| data("a stored share is not 0 to 10,000"))
            })
            .collect::<Result<Vec<_>, ServiceError>>()?;
        shares_by_day.insert(note.entry_date.clone(), shares);
    }
    let shares = shares_by_day
        .get(&note.entry_date)
        .map_or(&[][..], Vec::as_slice);
    if shares.is_empty() {
        return Ok(None);
    }
    let parts = split_by_bps(paisa(note.amount_paisa)?, shares).map_err(money_error)?;
    Ok(parts
        .into_iter()
        .find(|(id, _)| *id == partner_id)
        .map(|(_, part)| part.as_paisa()))
}

/// His part of the result of trip goods, as the real money (plan 80 gate Q-R1-2, PRD FR-17.12): his
/// credit of this sale (C25: for a بکری lot his stored `lot_credit_parts` row, for a sale in another
/// shop his line on his 0604 account, signed like the line, so a reversal is negative) less his part
/// of the cost of the goods sold (C21), `split_by_bps(cost, the shares of the trip)`. So the note adds up with the credit he
/// sees on his line. The shares of a trip cannot change after its first entry (FR-12.29).
///
/// Example (E2E-34, 40 / 30 / 30): sale 8,460, cost 7,800. His credit 3,384, his part of the cost
/// 3,120, so his part is 264 (198 and 198 for the others, 660 in all). A reversal gives −264.
///
/// The cost part is the shares' split of one sale's cost, not what he paid for that sale. Each split
/// rounds down and gives its leftover to the largest share, so the parts of the sales can differ
/// from his real payments (C23, each payment split on its own) by up to about one rupee for each
/// sale and each payment (and up to the number of partners less one in one split, for the largest
/// share). The note carries no amount, so no total (C22) and no line moves.
///
/// None when the line has no trip or no credit to compare. A trip with no shares, a split that
/// does not total 100% and an overflow are errors, never a missing figure.
fn trip_result_part(
    facts: &Facts,
    partner_id: i64,
    venture_id: Option<i64>,
    credit: Option<Paisa>,
    cost: i64,
) -> Result<Option<i64>, ServiceError> {
    use mandi_domain::core::money::split_by_bps;
    let (Some(venture_id), Some(credit)) = (venture_id, credit) else {
        return Ok(None);
    };
    let shares = facts
        .trip_shares
        .get(&venture_id)
        .filter(|shares| !shares.is_empty())
        .ok_or_else(|| data("a trip has no shares"))?;
    let parts = split_by_bps(paisa(cost)?, shares).map_err(money_error)?;
    // A credit for a partner who is not in the shares of his trip is a data error, not a cost of 0.
    let (_, his_cost) = parts
        .into_iter()
        .find(|(id, _)| *id == partner_id)
        .ok_or_else(|| {
            data("a trip credit belongs to a partner who is not in the trip's shares")
        })?;
    let part = credit.checked_sub(his_cost).map_err(money_error)?;
    Ok(Some(part.as_paisa()))
}

/// The facts that word the lines of one page, read in a few grouped queries.
#[derive(Default)]
struct Facts {
    entries: HashMap<i64, read::EntryInfo>,
    vouchers: HashMap<i64, read::VoucherInfo>,
    bikri: HashMap<i64, i64>,
    fruit: HashMap<i64, read::FruitInfo>,
    purchases: HashMap<i64, read::PurchaseInfo>,
    venture_entries: HashMap<i64, read::VentureEntryInfo>,
    ventures: HashMap<i64, (i64, String)>,
    lots: HashMap<i64, read::LotInfo>,
    sold: HashMap<(i64, i64), (i64, i64)>,
    credit_totals: HashMap<(i64, i64), i64>,
    part_shares: HashMap<i64, i64>,
    garden_closes: HashMap<i64, read::GardenCloseFigures>,
    /// The khata or person on each entry of a line of his 0605 account.
    subjects: HashMap<i64, read::Named>,
    /// The shares of each trip on the page, read once for all its lines (plan 76 point 11).
    trip_shares: HashMap<i64, Vec<(i64, mandi_domain::core::money::Bps)>>,
}

fn named(n: &read::Named) -> RecordRef {
    RecordRef {
        id: n.0,
        code: n.1.clone(),
        name_ur: n.2.clone(),
        name_en: n.3.clone(),
        tag: None,
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
            .filter(|e| e.entry_id < FIRST_NOTE)
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
    let fruit = read::fruit_infos(conn, &by_source("fruit"))?;
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
    let venture_list = sorted(venture_ids);
    let ventures = read::venture_numbers(conn, &venture_list)?;
    let mut trip_shares = HashMap::new();
    for (venture_id, shares) in venture_repo::partners_of_many(conn, &venture_list)? {
        let parsed = shares
            .into_iter()
            .map(|s| {
                i32::try_from(s.share_bps)
                    .ok()
                    .and_then(|b| mandi_domain::core::money::Bps::new(b).ok())
                    .map(|b| (s.partner_id, b))
                    .ok_or_else(|| data("a stored share is not 0 to 10,000"))
            })
            .collect::<Result<Vec<_>, ServiceError>>()?;
        trip_shares.insert(venture_id, parsed);
    }
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
    let linked_entries = sorted(
        page.iter()
            .filter(|e| matches!(e.kind, Kind::Linked))
            .map(|e| e.entry_id)
            .collect(),
    );
    let subjects = read::linked_subjects(conn, &linked_entries)?;
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
        subjects,
        trip_shares,
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

    fn text_value(mut self, name: &str, text: &str) -> Draft {
        self.values
            .insert(name.to_owned(), ErrorValue::Text(text.to_owned()));
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

/// The code of the garden or trip a line belongs to (plan 34): `0080`, `0070`. Null for the lines
/// of any other heading, whose number is a بکری or a voucher.
fn venture_code(heading: StatementHeading, number: Option<i64>) -> Option<String> {
    use mandi_domain::core::codes::{CodeType, number_code};
    let code_type = match heading {
        StatementHeading::Garden => CodeType::Garden,
        StatementHeading::Trip => CodeType::Trip,
        _ => return None,
    };
    number.map(|n| number_code(code_type, n))
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
        Kind::Share { .. } => {
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
        Kind::Venture => venture_draft(
            partner.id,
            facts,
            (info, effective, voucher),
            (negative, event.amount),
        )?,
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
            let venture_id = lot
                .purchase_id
                .and_then(|id| facts.purchases.get(&id))
                .map(|p| p.venture_id);
            let mut draft = Draft::new(H::Trip, D::TripCredit)
                .number(trip_number(facts, lot.purchase_id))
                .number_value("qty", qty)
                .number_value("sales_paisa", sales)
                .number_value("cost_paisa", cost)
                .number_value("result_paisa", result)
                .number_value("share_bps", *share_bps);
            // His part of the result (plan 76 point 11, FR-17.12): shown in the trip note.
            if let Some(part) = trip_result_part(facts, partner.id, venture_id, event.amount, cost)?
            {
                draft = draft.number_value("part_paisa", part);
            }
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
        Kind::Linked => {
            let info = info.ok_or_else(|| data("a linked line has no entry"))?;
            let (detail, kind) = match voucher.map(|v| v.voucher_type.as_str()) {
                Some("khata_out") => (D::KhataPaid, "khata_out"),
                Some("khata_in") => (D::KhataReceived, "khata_in"),
                Some("money_lent") => (D::LentGiven, "money_lent"),
                Some("money_lent_return") => (D::LentReturned, "money_lent_return"),
                _ => (D::LinkedOther, "other"),
            };
            let mut draft = Draft::new(H::Linked, detail).kind(kind);
            draft.number = match voucher {
                Some(v) => Some(v.number),
                None => facts.bikri.get(&info.source_id).copied(),
            };
            draft.subject = facts.subjects.get(&event.entry_id).map(named);
            if let Some(v) = voucher {
                draft.reason.clone_from(&v.reason);
            }
            draft
        }
        Kind::Advance {
            change,
            covered,
            seller,
            amount_paisa,
            share_bps,
        } => {
            let detail = match (change, *covered) {
                (AdvanceEventKind::Out, _) => D::AdvanceOut,
                (AdvanceEventKind::Back, false) => D::AdvanceBack,
                (AdvanceEventKind::Back, true) => D::AdvanceCovered,
            };
            let mut draft = Draft::new(H::Advance, detail)
                .number_value("amount_paisa", *amount_paisa)
                .number_value("share_bps", i64::from(*share_bps));
            draft.subject = Some(seller.clone());
            draft
        }
        Kind::LentShare {
            voucher_number,
            given,
            amount_paisa,
            part_paisa,
            person,
        } => {
            let mut draft = Draft::new(H::Linked, D::LentShareNote)
                .number(Some(*voucher_number))
                .kind(if *given { "given" } else { "returned" })
                .number_value("amount_paisa", *amount_paisa);
            if let Some(part) = part_paisa {
                draft = draft.number_value("part_paisa", *part);
            }
            draft.subject = Some(named(person));
            draft
        }
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
    let draft = draft
        .number_value("cost_paisa", signed(figures.cost_paisa)?)
        .number_value("sales_paisa", signed(figures.sales_paisa)?)
        .number_value("result_paisa", signed(result)?)
        .number_value("share_bps", figures.share_bps)
        .number_value("part_paisa", signed(part)?)
        .number_value("back_paisa", signed(figures.back_paisa)?);
    // PRD FR-12.51: where the owner gave relief, the closing line also says the rent now and the
    // rent the garden was made with. Its figures are the garden's (not his part), so they are not
    // signed for a reversal.
    match figures.rent_original_paisa {
        Some(original) if original > figures.rent_paisa => Ok(draft
            .number_value("rent_paisa", figures.rent_paisa)
            .number_value("rent_original_paisa", original)
            .number_value(
                "relief_paisa",
                original
                    .checked_sub(figures.rent_paisa)
                    .ok_or_else(|| data("a figure does not fit"))?,
            )),
        _ => Ok(draft),
    }
}

/// A line from the journal of his gardens, trips and fruit account that is not a بکری.
fn venture_draft(
    partner_id: i64,
    facts: &Facts,
    (info, effective, voucher): (
        Option<&read::EntryInfo>,
        Option<&str>,
        Option<&read::VoucherInfo>,
    ),
    (negative, credit): (bool, Option<Paisa>),
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
            // His part of the result (plan 76 point 11, FR-17.12).
            if let Some(part) =
                trip_result_part(facts, partner_id, Some(entry.venture_id), credit, cost)?
            {
                draft = draft.number_value("part_paisa", part);
            }
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
            let fruit = facts.fruit.get(&info.source_id);
            let mut draft = Draft::new(H::Fruit, D::FruitTaken).number(fruit.map(|f| f.number));
            // The details are those of the entry as it stands. An edit updates the entry, so an older
            // posting of an edited entry (and its reversal) gets the number only.
            let effective = info.reverses_entry_id.unwrap_or(info.entry_id);
            if let Some(f) = fruit.filter(|f| f.current_entry_id == Some(effective)) {
                let signed = |value: i64| -> Result<i64, ServiceError> {
                    value
                        .checked_mul(sign)
                        .ok_or_else(|| data("a figure does not fit"))
                };
                draft.subject = Some(named(&f.item));
                draft.packing = Some(named(&f.packing));
                // The line says "qty @ rate = value", so `value_paisa` is the value before the deduction
                // (qty × rate, C1) and the deduction is its own argument; the amount of the line is
                // the net the partner was charged (PRD FR-16.11 v1.59).
                let gross = f
                    .value_paisa
                    .checked_add(f.deduction_paisa)
                    .ok_or_else(|| data("a figure does not fit"))?;
                draft = draft
                    .number_value("qty", signed(f.qty)?)
                    .number_value("rate_paisa", f.rate_paisa)
                    .number_value("value_paisa", signed(gross)?)
                    .number_value("deduction_paisa", signed(f.deduction_paisa)?)
                    .text_value("for", if f.guests { "guests" } else { "home" });
            }
            draft
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

/// His کل رقم at the end of the day before `first`: the opening of a range. It is the sum of every
/// event before `first`, read as one aggregate instead of ten years of events (NFR-SPD-24). Zero when
/// there is no earlier day.
fn opening_before(
    conn: &Connection,
    partner: &PartnerRecord,
    first: &str,
    advances: &Advances,
) -> Result<Paisa, ServiceError> {
    let date = BusinessDate::parse_iso(first).ok_or_else(|| data("a range starts on no day"))?;
    match date.previous_day() {
        Some(before) => super::money::total_with(conn, partner, Some(&before.to_iso()), advances),
        None => Ok(Paisa::ZERO),
    }
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
    // The sort is checked first: an unknown column is refused (plan 38 R4). The lines are drawn
    // for the page only, so a line can be sorted only by what every event has.
    if let Some(sort) = &input.sort {
        sort.check(&["entry_date", "amount_paisa", "running_paisa"])
            .map_err(|e| ServiceError::reject_field(e.code, &e.field))?;
    }
    ctx.db.read(|conn| {
        let partner = repo::get(conn, input.partner_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "partner_id"))?;
        // Only the events of the range are read. What came before it is one sum, his کل رقم at the
        // end of the day before (the same figure the events before it would add up to).
        let days = read::Days {
            from: from.as_deref(),
            to: to.as_deref(),
        };
        // The advance events are read and replayed once for the whole request (R1-4).
        let advances = super::money::read_advances(conn)?;
        let events = events_of(conn, &partner, days, &advances)?;
        let mut month_acc = crate::page::MonthAcc::new(2);
        let opening = match from.as_deref() {
            Some(first) => opening_before(conn, &partner, first, &advances)?,
            None => Paisa::ZERO,
        };
        // The running total at every event, in checked Paisa.
        let mut running = opening;
        let mut in_range: Vec<(&Event, Paisa)> = Vec::new();
        let mut totals = StatementTotals {
            plus_paisa: Paisa::ZERO,
            minus_paisa: Paisa::ZERO,
        };
        for event in &events {
            if let Some(amount) = event.amount {
                running = running.checked_add(amount).map_err(money_error)?;
            }
            let (plus, minus) = match event.amount {
                Some(a) if a.is_negative() => (0, a.checked_neg().map_err(money_error)?.as_paisa()),
                Some(a) => (a.as_paisa(), 0),
                None => (0, 0),
            };
            month_acc.add(&event.date, &[plus, minus])?;
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
        // The running total was worked out in the natural order, so each event keeps its true
        // figure. A sort reorders the whole range after that, before the page is cut.
        if let Some(sort) = &input.sort {
            sort_events(&mut in_range, &sort.column, sort.direction);
        }
        let in_date_order = input.sort.as_ref().is_none_or(|s| s.column == "entry_date");
        let newest_first = input
            .sort
            .as_ref()
            .is_some_and(|s| s.column == "entry_date" && s.direction == SortDirection::Desc);
        let months = if input.group_by_month && in_date_order {
            month_acc.finish(&[("plus_paisa", true), ("minus_paisa", true)], newest_first)
        } else {
            Vec::new()
        };
        let page_rows = page::slice(in_range, input.offset, input.limit);
        let page_events: Vec<&Event> = page_rows.iter().map(|(e, _)| *e).collect();
        let facts = facts_for(conn, partner.id, &page_events)?;
        let mut lines = Vec::with_capacity(page_rows.len());
        for (event, running) in &page_rows {
            let draft = draft_of(conn, &partner, &facts, event)?;
            let entry_id = (event.entry_id < FIRST_NOTE).then_some(event.entry_id);
            lines.push(StatementLine {
                entry_date: event.date.clone(),
                heading: draft.heading,
                detail: draft.detail,
                entry_id: entry_id
                    .or_else(|| (matches!(event.kind, Kind::MonthClose)).then_some(event.id)),
                number: draft.number,
                venture_code: venture_code(draft.heading, draft.number),
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
            months,
        })
    })
}

/// Sorts the events of the range by a column (`entry_date`, `amount_paisa` or `running_paisa`). A
/// stable sort over the natural order, so a sort by the day mirrors the order of events when
/// descending, and equal amounts keep the order by day.
fn sort_events(rows: &mut Vec<(&Event, Paisa)>, column: &str, direction: SortDirection) {
    use std::cmp::Ordering;
    type Keyed<'a> = (usize, (&'a Event, Paisa));
    let mut keyed: Vec<Keyed<'_>> = rows.drain(..).enumerate().collect();
    let cmp: fn(&Keyed<'_>, &Keyed<'_>) -> Ordering = match column {
        "entry_date" => |a, b| a.0.cmp(&b.0),
        "amount_paisa" => |a, b| a.1.0.amount.cmp(&b.1.0.amount),
        "running_paisa" => |a, b| a.1.1.cmp(&b.1.1),
        _ => |_, _| Ordering::Equal,
    };
    keyed.sort_by(|a, b| direction.apply(cmp(a, b)));
    rows.extend(keyed.into_iter().map(|(_, row)| row));
}

// ---- One month at a time, and one day in detail (plan 76 point 11) ----

/// The column of the month page an amount event belongs to. A note line has none.
#[derive(Clone, Copy)]
enum Column {
    Commission,
    Expense,
    GardensTrips,
    Fruit,
    TakenReturned,
    Capital,
    Advances,
}

fn column_of(event: &Event, sources: &HashMap<i64, String>) -> Option<Column> {
    event.amount?;
    Some(match &event.kind {
        Kind::Share { expense: true } => Column::Expense,
        Kind::Share { expense: false } => Column::Commission,
        Kind::Capital => Column::Capital,
        Kind::Drawings | Kind::Linked => Column::TakenReturned,
        Kind::Advance { .. } => Column::Advances,
        // A trip lot credit and the rest of a بکری posting are already lines of his 0604 account:
        // they are never added a second time.
        Kind::LotCredit { .. } | Kind::Remainder => Column::GardensTrips,
        Kind::Venture => {
            if sources.get(&event.entry_id).is_some_and(|s| s == "fruit") {
                Column::Fruit
            } else {
                Column::GardensTrips
            }
        }
        _ => return None,
    })
}

#[derive(Default, Clone, Copy)]
struct Cols {
    commission: Paisa,
    expense: Paisa,
    gardens_trips: Paisa,
    fruit: Paisa,
    taken_returned: Paisa,
    capital: Paisa,
    advances: Paisa,
}

impl Cols {
    fn add(&mut self, column: Column, amount: Paisa) -> Result<(), ServiceError> {
        let slot = match column {
            Column::Commission => &mut self.commission,
            Column::Expense => &mut self.expense,
            Column::GardensTrips => &mut self.gardens_trips,
            Column::Fruit => &mut self.fruit,
            Column::TakenReturned => &mut self.taken_returned,
            Column::Capital => &mut self.capital,
            Column::Advances => &mut self.advances,
        };
        *slot = slot.checked_add(amount).map_err(money_error)?;
        Ok(())
    }

    fn merge(&mut self, other: &Cols) -> Result<(), ServiceError> {
        for (a, b) in [
            (&mut self.commission, other.commission),
            (&mut self.expense, other.expense),
            (&mut self.gardens_trips, other.gardens_trips),
            (&mut self.fruit, other.fruit),
            (&mut self.taken_returned, other.taken_returned),
            (&mut self.capital, other.capital),
            (&mut self.advances, other.advances),
        ] {
            *a = a.checked_add(b).map_err(money_error)?;
        }
        Ok(())
    }

    fn shown(&self) -> Result<StatementColumns, ServiceError> {
        let mut net = Paisa::ZERO;
        for part in [
            self.commission,
            self.expense,
            self.gardens_trips,
            self.fruit,
            self.taken_returned,
            self.capital,
            self.advances,
        ] {
            net = net.checked_add(part).map_err(money_error)?;
        }
        Ok(StatementColumns {
            commission_paisa: self.commission,
            expense_paisa: self.expense,
            gardens_trips_paisa: self.gardens_trips,
            fruit_paisa: self.fruit,
            taken_returned_paisa: self.taken_returned,
            capital_paisa: self.capital,
            advances_paisa: self.advances,
            net_paisa: net,
        })
    }
}

/// A partner's statement for one month: his کل رقم at the start, one row for each day with a
/// movement (by what it was), the month's totals and his کل رقم at the end. Read only. The
/// closing is checked against `total_of` at the end of the month (two ways to the same figure), so
/// a month that does not add up is an error and never a printed wrong figure.
///
/// Example (partner ب, October 2026, after Events 1 to 5 and E2E-35): opening 0; 10-10-26
/// commission 1,646, expense -900, کل رقم 746; 11-10-26 capital +50,000, کل رقم 50,746.
///
/// # Errors
/// `ERR_NOT_FOUND` on `partner_id`; `ERR_FORMAT_INVALID` and `ERR_DATE_IN_FUTURE` on `month`.
pub fn statement_month(
    ctx: &Context,
    input: &PartnersStatementMonthInput,
) -> Result<PartnerStatementMonth, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let current = Month::of(today);
    let month = match input
        .month
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        None => current,
        Some(text) => Month::parse(text)
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrFormatInvalid, "month"))?,
    };
    if month > current {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrDateInFuture,
            "month",
        ));
    }
    let (Some(first), Some(last)) = (month.first_day(), month.last_day()) else {
        return Err(ServiceError::reject_field(
            ErrorCode::ErrFormatInvalid,
            "month",
        ));
    };
    let (first_iso, last_iso) = (first.to_iso(), last.to_iso());
    ctx.db.read(|conn| {
        let partner = repo::get(conn, input.partner_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "partner_id"))?;
        // Only the events of the month are read; the figure before it is one sum (NFR-SPD-24).
        let range = read::Days {
            from: Some(&first_iso),
            to: Some(&last_iso),
        };
        // The advance events are read and replayed once for the whole request (R1-4).
        let advances = super::money::read_advances(conn)?;
        let events = events_of(conn, &partner, range, &advances)?;
        let first_month = first_event_date_with(conn, &partner, &advances)?
            .and_then(|d| d.get(..7).map(str::to_owned))
            .unwrap_or_else(|| current.key());
        // Which 0604 lines are fruit: the source of their entries, read for the month only.
        let venture_entries = sorted(
            events
                .iter()
                .filter(|e| matches!(e.kind, Kind::Venture) && e.entry_id < FIRST_NOTE)
                .map(|e| e.entry_id)
                .collect(),
        );
        let sources: HashMap<i64, String> = read::entry_infos(conn, &venture_entries)?
            .into_iter()
            .map(|(id, info)| (id, info.source_type))
            .collect();
        let opening = opening_before(conn, &partner, &first_iso, &advances)?;
        let mut running = opening;
        let mut days: BTreeMap<String, (Cols, bool, Paisa)> = BTreeMap::new();
        for event in &events {
            if let Some(amount) = event.amount {
                running = running.checked_add(amount).map_err(money_error)?;
            }
            let day = days
                .entry(event.date.clone())
                .or_insert((Cols::default(), false, running));
            match (column_of(event, &sources), event.amount) {
                (Some(column), Some(amount)) => day.0.add(column, amount)?,
                _ => day.1 = true,
            }
            day.2 = running;
        }
        let mut totals = Cols::default();
        let mut rows = Vec::with_capacity(days.len());
        let mut closing = opening;
        for (date, (cols, notes, end)) in &days {
            let shown = cols.shown()?;
            // The day's columns must be the day's change of the running total.
            if closing.checked_add(shown.net_paisa).map_err(money_error)? != *end {
                return Err(data(
                    "the columns of a day do not add up to his running total",
                ));
            }
            closing = *end;
            totals.merge(cols)?;
            rows.push(StatementMonthDay {
                date: date.clone(),
                columns: shown,
                running_paisa: *end,
                has_notes: *notes,
            });
        }
        // Two ways to the same figure: what the sums of his records say moved in the month. The
        // lines above and these sums are different reads, so a month that does not add up is
        // an error and never a printed wrong figure.
        let moved = super::money::movement_with(conn, &partner, &first_iso, &last_iso, &advances)?;
        if opening.checked_add(moved).map_err(money_error)? != closing {
            return Err(data("the closing of a month is not his total at its end"));
        }
        Ok(PartnerStatementMonth {
            partner: record_of(partner.id, &partner.name_ur, &partner.name_en),
            month: month.key(),
            first_month,
            current_month: current.key(),
            opening_total_paisa: opening,
            days: rows,
            totals: totals.shown()?,
            closing_total_paisa: closing,
        })
    })
}

/// Every movement of one day of a partner's statement with its source, and his کل رقم before and
/// after (plan 76 point 11). A share line carries `base_paisa` (the posting's shared income or
/// expense) and `share_bps`, so the card can say "his 30% of 3,948". Read only.
///
/// # Errors
/// `ERR_NOT_FOUND` on `partner_id`; `ERR_DATE_INVALID` on `date`.
pub fn statement_day(
    ctx: &Context,
    input: &PartnersStatementDayInput,
) -> Result<PartnerStatementDay, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let text = input.date.trim();
    let date = BusinessDate::parse_iso(text)
        .or_else(|| mandi_domain::core::dates::parse_ddmmyyyy(text, today).ok())
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrDateInvalid, "date"))?;
    let typed = mandi_domain::core::dates::format_ddmmyyyy(date);
    let mut page = statement(
        ctx,
        &PartnersStatementInput {
            partner_id: input.partner_id,
            from: Some(typed.clone()),
            to: Some(typed),
            offset: 0,
            limit: DAY_LINES,
            sort: None,
            group_by_month: false,
        },
    )?;
    // What the share lines were worked from.
    let entry_ids: Vec<i64> = page
        .lines
        .iter()
        .filter(|l| {
            matches!(
                l.detail,
                StatementDetail::IncomeShare | StatementDetail::ExpenseShare
            )
        })
        .filter_map(|l| l.entry_id)
        .collect();
    if !entry_ids.is_empty() {
        let figures = ctx.db.read(|conn| {
            Ok::<_, ServiceError>(read::share_figures(conn, input.partner_id, &entry_ids)?)
        })?;
        for line in &mut page.lines {
            let kind = match line.detail {
                StatementDetail::IncomeShare => "income",
                StatementDetail::ExpenseShare => "expense",
                _ => continue,
            };
            if let Some((base, bps)) = line
                .entry_id
                .and_then(|id| figures.get(&(id, kind.to_owned())))
            {
                line.values
                    .insert("base_paisa".to_owned(), ErrorValue::Number(*base));
                line.values
                    .insert("share_bps".to_owned(), ErrorValue::Number(*bps));
            }
        }
    }
    Ok(PartnerStatementDay {
        partner: page.partner,
        date: date.to_iso(),
        opening_total_paisa: page.opening_total_paisa,
        lines: page.lines,
        after_total_paisa: page.closing_total_paisa,
    })
}

/// The most lines of one day's detail: every movement of a day, not a page of them.
const DAY_LINES: u32 = 5_000;

/// The day of his first movement, or null when he has none (the back button of his month page).
///
/// It is the earliest of the first day of each kind of event, each read by an index or from a
/// short list, so a partner with ten years of share rows is not read row by row (NFR-SPD-24). A
/// بکری line on his 0604 account with no stored part and a net of 0 is no event and could make
/// this day earlier than his first line; that cannot happen in saved data (every such line has a
/// stored part), so it is left as it is.
///
/// # Errors
/// `ServiceError::Db`.
pub fn first_event_date(
    conn: &Connection,
    partner: &PartnerRecord,
) -> Result<Option<String>, ServiceError> {
    first_event_date_with(conn, partner, &super::money::read_advances(conn)?)
}

/// [`first_event_date`] with the advance events already read by the request (R1-4).
///
/// # Errors
/// `ServiceError::Db`.
fn first_event_date_with(
    conn: &Connection,
    partner: &PartnerRecord,
    advances: &Advances,
) -> Result<Option<String>, ServiceError> {
    let mut first: Option<String> = read::first_share_day(conn, partner.id)?;
    let mut keep = |day: Option<String>| {
        if let Some(day) = day
            && first.as_ref().is_none_or(|f| day < *f)
        {
            first = Some(day);
        }
    };
    let all = read::Days::ALL;
    for account in [
        partner.capital_account_id,
        partner.drawings_account_id,
        partner.venture_account_id,
        partner.linked_account_id,
    ] {
        if account != 0 {
            keep(read::first_line_day(conn, account)?);
        }
    }
    keep(read::first_lot_credit_day(conn, partner.id)?);
    // His first part of an advance to a seller (PRD FR-17.13, plan 78 B4).
    keep(
        advances
            .events
            .iter()
            .filter(|e| e.partner_id == partner.id)
            .map(|e| e.date.clone())
            .min(),
    );
    for note in read::shop_loan_notes(conn, all)? {
        keep(Some(note.entry_date));
    }
    for note in read::month_close_notes(conn, partner.capital_account_id, all)? {
        keep(Some(note.date));
    }
    for note in read::write_off_notes(conn, partner.id, all)? {
        keep(Some(note.date));
    }
    for note in read::closed_trip_notes(conn, partner.id, all)? {
        keep(Some(note.date));
    }
    for note in read::unposted_garden_close_notes(conn, partner.id, all)? {
        keep(Some(note.date));
    }
    Ok(first)
}

#[cfg(test)]
mod tests {
    //! Plan 80 gate R1-6 and Q-R1-2: the notes of the statement show the real money and never hide a
    //! split error.

    use mandi_domain::core::money::Bps;

    use super::*;

    fn rs(rupees: i64) -> Paisa {
        Paisa::from_rupees(rupees).unwrap()
    }

    fn shares(list: &[(i64, i32)]) -> Vec<(i64, Bps)> {
        list.iter()
            .map(|&(id, bps)| (id, Bps::new(bps).unwrap()))
            .collect()
    }

    fn facts_with(trip: i64, list: &[(i64, i32)]) -> Facts {
        let mut facts = Facts::default();
        facts.trip_shares.insert(trip, shares(list));
        facts
    }

    #[test]
    fn the_trip_part_is_his_credit_less_his_part_of_the_cost() {
        // E2E-34 at 40 / 30 / 30: sale 8,460, cost 7,800. His credit 3,384, his part of the cost
        // 3,120, so his part is 264 (198 and 198 for the others, 660 in all).
        let facts = facts_with(1, &[(1, 4_000), (2, 3_000), (3, 3_000)]);
        let part = |partner, credit| {
            trip_result_part(
                &facts,
                partner,
                Some(1),
                Some(rs(credit)),
                rs(7_800).as_paisa(),
            )
            .unwrap()
        };
        assert_eq!(part(1, 3_384), Some(rs(264).as_paisa()));
        assert_eq!(part(2, 2_538), Some(rs(198).as_paisa()));
        assert_eq!(part(3, 2_538), Some(rs(198).as_paisa()));
        // A reversal is the opposite: -3,384 and a cost of -7,800 give -264.
        let back =
            trip_result_part(&facts, 1, Some(1), Some(rs(-3_384)), rs(-7_800).as_paisa()).unwrap();
        assert_eq!(back, Some(rs(-264).as_paisa()));
    }

    #[test]
    fn the_trip_part_follows_the_credit_he_sees_where_split_of_the_result_differs() {
        // The money agent's smallest case, 60 / 40: sale 3 rupees, cost 1 rupee. The credits are 2 and
        // 1; the cost parts 1 and 0; the parts of the result 1 and 1 (they add up to the result 2). The
        // old `split_by_bps(result = 2)` gave 2 and 0, which did not match his lines.
        let facts = facts_with(1, &[(1, 6_000), (2, 4_000)]);
        let part = |partner, credit| {
            trip_result_part(&facts, partner, Some(1), Some(rs(credit)), rs(1).as_paisa()).unwrap()
        };
        assert_eq!(part(1, 2), Some(rs(1).as_paisa()));
        assert_eq!(part(2, 1), Some(rs(1).as_paisa()));
    }

    #[test]
    fn the_trip_part_says_nothing_without_a_trip_or_a_credit_and_fails_on_bad_data() {
        let facts = facts_with(1, &[(1, 6_000), (2, 4_000)]);
        assert_eq!(
            trip_result_part(&facts, 1, None, Some(rs(1)), 0).unwrap(),
            None
        );
        assert_eq!(trip_result_part(&facts, 1, Some(1), None, 0).unwrap(), None);
        // A trip with no shares in the facts, or shares that do not total 100%, is an error and never
        // a missing figure (R1-6).
        assert!(trip_result_part(&facts, 1, Some(9), Some(rs(1)), 100).is_err());
        let broken = facts_with(1, &[(1, 5_000), (2, 4_000)]);
        assert!(trip_result_part(&broken, 1, Some(1), Some(rs(1)), 100).is_err());
    }

    fn note(date: &str) -> read::ShopLoanNote {
        read::ShopLoanNote {
            voucher_id: 1,
            number: 1,
            entry_date: date.to_owned(),
            given: true,
            amount_paisa: rs(10_000).as_paisa(),
            person: (1, "00601".into(), "ا".into(), "A".into()),
        }
    }

    #[test]
    fn a_loan_part_is_none_only_when_no_share_was_set_that_day() {
        let conn = Connection::open_in_memory().unwrap();
        let mut by_day = HashMap::new();
        // No share that day: no part to show.
        by_day.insert("2026-10-10".to_owned(), Vec::new());
        assert_eq!(
            loan_part(&conn, &mut by_day, 1, &note("2026-10-10")).unwrap(),
            None
        );
        // Shares that total 100%: his 40% of 10,000.
        by_day.insert(
            "2026-10-11".to_owned(),
            shares(&[(1, 4_000), (2, 3_000), (3, 3_000)]),
        );
        assert_eq!(
            loan_part(&conn, &mut by_day, 1, &note("2026-10-11")).unwrap(),
            Some(rs(4_000).as_paisa())
        );
        // Shares that do not total 100% were swallowed as None before the fix; now it is an error.
        by_day.insert("2026-10-12".to_owned(), shares(&[(1, 4_000), (2, 3_000)]));
        assert!(loan_part(&conn, &mut by_day, 1, &note("2026-10-12")).is_err());
    }
}
