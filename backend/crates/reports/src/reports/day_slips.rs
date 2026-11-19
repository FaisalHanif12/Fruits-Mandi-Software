//! The day's buyer parchi and the overall sale print (plan 76 point 3, PRD FR-19.12, FR-12.12).
//!
//! No new money maths: each slip is the R35 day slip of one party, built by the engine itself
//! (`build_in` with R35 and the party), and written in the print language by `build_print`. So a
//! parchi and a slip printed from R35 can never differ, and every figure is the posted ledger.
//!
//! * **Buyer parchi**: the slips of the buyers who bought on the day (of one بکری when asked, or of
//!   the buyers asked for), sorted by name in the Urdu order. Each slip has the buyer's lines of
//!   the day, today's total, what he paid, the previous balance and the new balance. The template
//!   lays several slips on one A4 page with cut lines.
//! * **Overall sale print**: the seller slips (each seller's lots sorted as PRD FR-12.12, the
//!   deductions on their own lines, net, previous and new balance) and the buyer slips (his
//!   purchases, paid on the spot, previous balance, deposited today, total now) of one بکری or of
//!   the whole day.
//!
//! The day of a mixed save holds both its later postings and its new بکری: the slips are by the
//! day of the sale lines, so a buyer's slip lists every line of the day.
//!
//! Example (E2E-43, 10-10-26): اکرم's slip: کیلا 20 @ 600 = 12,000, commission 720, لاگا 160, today
//! 12,880, paid 10,000, previous 0, new balance 2,880.

use mandi_core::auth::Language;
use mandi_core::dates::{BusinessDate, format_ddmmyyyy, parse_ddmmyyyy};
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::printing::Paper;
use mandi_db::Db;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::business::sheet::activity::{parties_for, sort_by_party};
use super::common::{in_snapshot, today as today_of};
use super::model::{PartyFilter, ReportFilters};
use super::print_data::{PrintOptions, ReportPrintData, build_print};
use crate::error::ReportsError;
use crate::print::{PrintFact, PrintFrame, PrintFrameInput, print_frame};

/// What a day's slips are made for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SlipSections {
    /// Only the sellers' slips.
    Sellers,
    /// Only the buyers' slips.
    Buyers,
    /// Both.
    Both,
}

/// Input of `reports_buyer_parchi_print_data` and `reports_sale_print_data`.
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
pub struct DaySlipsInput {
    /// The day, `DD-MM-YYYY` or a shortcut, or `YYYY-MM-DD`. Empty is today.
    #[serde(default)]
    pub date: String,
    /// Only the parties of this بکری's sale lines of the day, and only those lines on their slips.
    /// Null is the whole day. `bikri_ids` takes precedence when it is not empty.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub bikri_id: Option<i64>,
    /// Only these بکری: the new بکری of a mixed save and the old بکری of each carried lot, so the
    /// print covers everything that save sold (plan 76 point 3, FR-12.48). Null or empty: use
    /// `bikri_id`, else the whole day.
    #[serde(default)]
    #[ts(type = "Array<number> | null")]
    pub bikri_ids: Option<Vec<i64>>,
    /// Only these buyers (account ids). Null is every buyer of the scope. A چکتا buyer is asked
    /// for by his account (0030): every one of the day is meant.
    #[serde(default)]
    #[ts(type = "Array<number> | null")]
    pub buyer_ids: Option<Vec<i64>>,
    /// Which slips.
    #[serde(default = "both")]
    pub sections: SlipSections,
    /// The print language (PRD FR-7.36).
    pub language: Language,
    /// The paper; A4 when missing.
    #[serde(default)]
    pub paper: Option<Paper>,
}

fn both() -> SlipSections {
    SlipSections::Both
}

/// The slips of one day, written in the print language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct DaySlipsPrintData {
    /// The header and the footer (PRD FR-19.13).
    pub frame: PrintFrame,
    /// The paper class.
    pub paper: Paper,
    /// The day, `YYYY-MM-DD`.
    pub date: String,
    /// The sellers' slips, sorted by name.
    pub sellers: Vec<ReportPrintData>,
    /// The buyers' slips, sorted by name.
    pub buyers: Vec<ReportPrintData>,
}

fn reject(code: ErrorCode, field: &str) -> ReportsError {
    ReportsError::Rejected(vec![FieldError::new(code, field)])
}

/// A buyer of a slip: the account and, for a چکتا buyer, his person key.
type BuyerKey = (i64, Option<String>);

/// The parties that sold or bought on a day, by the net of the analytics facts (a cancelled line
/// nets to 0), in the order of their ids.
fn keys_of(
    conn: &mandi_db::rusqlite::Connection,
    iso: &str,
    bikri: &[i64],
) -> Result<(Vec<BuyerKey>, Vec<i64>), ReportsError> {
    let mut buyers: Vec<(i64, Option<String>)> = Vec::new();
    let mut sellers: Vec<i64> = Vec::new();
    // No بکری: the whole day. Else only the sale lines of those بکری.
    let marks = vec!["?"; bikri.len()].join(", ");
    let only = if bikri.is_empty() {
        String::new()
    } else {
        format!(" AND bikri_id IN ({marks})")
    };
    let mut args: Vec<mandi_db::rusqlite::types::Value> =
        vec![mandi_db::rusqlite::types::Value::Text(iso.to_owned())];
    args.extend(
        bikri
            .iter()
            .map(|id| mandi_db::rusqlite::types::Value::Integer(*id)),
    );
    let mut statement = conn.prepare(&format!(
        "SELECT buyer_account_id, chukta_person_key, seller_account_id
         FROM analytics_facts WHERE entry_date = ?{only}
         GROUP BY buyer_account_id, chukta_person_key, seller_account_id, sale_id
         HAVING SUM(sign) <> 0"
    ))?;
    let rows = statement.query_map(mandi_db::rusqlite::params_from_iter(args.iter()), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, i64>(2)?,
        ))
    })?;
    for row in rows {
        let (buyer, person, seller) = row?;
        let key = (buyer, person);
        if !buyers.contains(&key) {
            buyers.push(key);
        }
        if !sellers.contains(&seller) {
            sellers.push(seller);
        }
    }
    Ok((buyers, sellers))
}

/// True for an account a seller slip can be made for: a real seller (control 03011), not the
/// firm's own goods, a garden or a khata.
fn is_seller_account(conn: &mandi_db::rusqlite::Connection, id: i64) -> Result<bool, ReportsError> {
    let found = conn
        .query_row(
            "SELECT k.code, a.code, a.is_chukta_khata FROM accounts a
             JOIN account_controls k ON k.id = a.control_id WHERE a.id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            },
        )
        .map(Some)
        .or_else(|e| match e {
            mandi_db::rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })?;
    Ok(found.is_some_and(|(control, code, chukta)| {
        control == "03011" && code != mandi_core::accounts::OWN_GOODS_CODE && chukta == 0
    }))
}

fn slip_of(
    conn: &mandi_db::rusqlite::Connection,
    db: &Db,
    day: BusinessDate,
    account: i64,
    person: Option<&str>,
    only_bikri: &[i64],
    options: &PrintOptions,
) -> Result<ReportPrintData, ReportsError> {
    let filters = ReportFilters {
        as_of: format_ddmmyyyy(day),
        party: Some(PartyFilter {
            account_id: account,
            person_key: person.map(str::to_owned),
        }),
        // A seller's lots print in the order of the print language (PRD FR-12.12).
        order_language: Some(options.language),
        ..ReportFilters::default()
    };
    let page = super::build_slip_in(conn, db, &filters, only_bikri)?;
    build_print(&page, options)
}

/// The seller bills of a scope (plan 78 A1, A3): the R35 slip of each seller who sold in it, each
/// titled with his name ("{ $name } کا بل"). The same slips as the sellers of [`day_slips`]; only
/// the sellers are read. Read only.
///
/// # Errors
/// As [`day_slips`].
pub fn seller_bills(db: &Db, input: &DaySlipsInput) -> Result<DaySlipsPrintData, ReportsError> {
    let mut input = input.clone();
    input.sections = SlipSections::Sellers;
    let mut data = day_slips(db, &input, "print-seller-bills-title")?;
    for slip in &mut data.sellers {
        slip.named_title_id = Some(super::ids::SELLER_BILL_TITLE.to_owned());
    }
    Ok(data)
}

/// The slips of a day in the print language (plan 76 point 3). Read only.
///
/// # Errors
/// `ERR_DATE_INVALID` and `ERR_DATE_IN_FUTURE` on `date`; a database or money error.
pub fn day_slips(
    db: &Db,
    input: &DaySlipsInput,
    title_id: &str,
) -> Result<DaySlipsPrintData, ReportsError> {
    let grouping = db.read(crate::print::load::load_grouping)?;
    let options = PrintOptions {
        language: input.language,
        grouping,
        paper: input.paper.unwrap_or(Paper::A4),
    };
    let (iso, day, sellers, buyers) = db.read(|conn| {
        in_snapshot(conn, |conn| {
            let today = today_of(db.clock().now_utc())?;
            let text = input.date.trim();
            let day = if text.is_empty() {
                today
            } else {
                BusinessDate::parse_iso(text)
                    .or_else(|| parse_ddmmyyyy(text, today).ok())
                    .ok_or_else(|| reject(ErrorCode::ErrDateInvalid, "date"))?
            };
            if day > today {
                return Err(reject(ErrorCode::ErrDateInFuture, "date"));
            }
            let iso = day.to_iso();
            // The بکری asked for: `bikri_ids`, else the one `bikri_id`, else the whole day.
            let only: Vec<i64> = match (&input.bikri_ids, input.bikri_id) {
                (Some(ids), _) if !ids.is_empty() => ids.clone(),
                (_, Some(id)) => vec![id],
                _ => Vec::new(),
            };
            let (mut buyer_keys, seller_ids) = keys_of(conn, &iso, &only)?;
            if let Some(ids) = &input.buyer_ids {
                buyer_keys.retain(|(id, _)| ids.contains(id));
            }
            let want_buyers = input.sections != SlipSections::Sellers;
            let want_sellers = input.sections != SlipSections::Buyers;
            let mut buyers = Vec::new();
            if want_buyers {
                let parties = parties_for(conn, &buyer_keys)?;
                for key in sort_by_party(buyer_keys, &parties, |k| k.clone()) {
                    buyers.push(slip_of(
                        conn,
                        db,
                        day,
                        key.0,
                        key.1.as_deref(),
                        &only,
                        &options,
                    )?);
                }
            }
            let mut sellers = Vec::new();
            if want_sellers {
                let mut keys: Vec<(i64, Option<String>)> = Vec::new();
                for id in seller_ids {
                    if is_seller_account(conn, id)? {
                        keys.push((id, None));
                    }
                }
                let parties = parties_for(conn, &keys)?;
                for key in sort_by_party(keys, &parties, |k| k.clone()) {
                    sellers.push(slip_of(conn, db, day, key.0, None, &only, &options)?);
                }
            }
            Ok((iso, day, sellers, buyers))
        })
    })?;
    let value = format_ddmmyyyy(day);
    let frame = print_frame(
        db,
        &PrintFrameInput {
            language: input.language,
            title_id: title_id.to_owned(),
            facts: vec![PrintFact {
                label_id: "print-fact-date".to_owned(),
                value,
            }],
        },
    )?;
    Ok(DaySlipsPrintData {
        frame,
        paper: options.paper,
        date: iso,
        sellers,
        buyers,
    })
}
