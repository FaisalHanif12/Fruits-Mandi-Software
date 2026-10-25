//! Print data of the collection list (اگراہی فہرست, PRD FR-15.6, FR-7.61, FR-19.13; plan 17e).
//!
//! The list is the buyers planned for one day, with their dues, by collector and then by city. It
//! is the fifth built print, so it has the same header and footer as the other four and the same
//! rules: every number is formatted for print here, every title and label is a Fluent message ID,
//! and names are in the print language.
//!
//! Plan 17e T1 holds the types only. The builder (`collection_print_data`) comes in T2, from the
//! same list the screen reads (`assist_collection_list`), so the print and the screen can never
//! disagree. The blank column for the collector's note is layout: the template draws it, there is
//! no data for it.
//!
//! Example (after Events 1 to 5, a collector with three buyers): one group, three rows, the
//! total of their dues, and the buyers' count.

use chrono::{DateTime, Utc};
use mandi_core::accounts::short_code_of;
use mandi_core::auth::Language;
use mandi_core::money::{Grouping, Paisa};
use mandi_core::printing::{CollectionFilterNames, Paper, PrintDoc, PrintSetup};
use mandi_core::vouchers::CollectionList;
use mandi_db::Db;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::format::NumberStyle;
use super::header::{PrintFooter, PrintHeader, footer, header, pick};
use super::ids::{COLLECTION_TOTAL, FACT_CITY, FACT_COLLECTOR, FACT_DATE, TITLE_COLLECTION_LIST};
use super::load::{FirmNames, load_firm, load_grouping, load_setup};
use super::{AmountRole, PrintAmount, PrintFact, TextDir};
use crate::error::ReportsError;

/// Input of `reports_collection_print_data` (plan 17e): the same day and filters as
/// `assist_collection_list`, and the print language.
#[derive(Debug, Clone, Deserialize, TS)]
pub struct CollectionPrintDataInput {
    /// The day, `DD-MM-YYYY` or a shortcut (PRD FR-7.19). Buyers whose collection day is this
    /// weekday are on the list.
    pub date: String,
    /// Only this collector (a staff member). Missing means every collector.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub collector_staff_id: Option<i64>,
    /// Only this city. Missing means every city.
    #[serde(default)]
    #[ts(type = "number | null")]
    pub city_id: Option<i64>,
    /// The print language (PRD FR-7.36): names, digits and dates follow it.
    pub language: Language,
}

/// One buyer on the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionPrintRow {
    /// The row number in the whole list, 1, 2, 3 ..., formatted for print.
    pub serial: String,
    /// The short code of the buyer (PRD FR-7.39): his serial, for example `1` for 03040001.
    pub code: String,
    /// His name in the print language.
    pub name: String,
    /// His tag (نشان), printed after the name. Absent if none (PRD 10.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tag: Option<String>,
    /// His phone as typed (PRD FR-7.25). Empty if none.
    pub phone: String,
    /// What he owes (his debit balance), in whole rupees, formatted for print.
    pub dues: String,
    /// The day of his oldest open bill, `DD-MM-YYYY`. Null if he has none.
    pub oldest_bill_date: Option<String>,
    /// Its age in days, formatted for print. Null if he has no open bill.
    pub oldest_days: Option<String>,
}

/// The buyers of one collector in one city.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionPrintGroup {
    /// The collector's name in the print language. Null for buyers with no collector.
    pub collector: Option<String>,
    /// The city's name in the print language. Null for buyers with no city.
    pub city: Option<String>,
    /// The buyers, by code.
    pub rows: Vec<CollectionPrintRow>,
    /// Σ of their dues, formatted for print.
    pub total: String,
}

/// Everything a template needs to print the collection list of one day (plan 17e).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
pub struct CollectionPrintData {
    /// Always `collection_list`.
    pub doc: PrintDoc,
    /// The print language.
    pub language: Language,
    /// The writing direction, from the language.
    pub dir: TextDir,
    /// The paper class, from the Printing settings (starts as A4).
    pub paper: Paper,
    /// True when Urdu digits are used on this print.
    pub urdu_digits: bool,
    /// The shared header (PRD FR-19.13). Its facts are the day, and the collector and the city
    /// when the list is filtered by them.
    pub header: PrintHeader,
    /// The shared footer (PRD FR-19.13).
    pub footer: PrintFooter,
    /// By collector, then city.
    pub groups: Vec<CollectionPrintGroup>,
    /// Σ of all dues on the list: the totals row (PRD FR-19.5).
    pub total: PrintAmount,
    /// How many buyers are on the list, formatted for print.
    pub buyer_count: String,
}

// ---- The builder (plan 17e T2) ----

/// Everything besides the list that decides what is printed.
#[derive(Debug, Clone)]
pub struct CollectionBuildOptions {
    /// The print language.
    pub language: Language,
    /// The number grouping setting (PRD FR-19.7).
    pub grouping: Grouping,
    /// The Printing settings: the paper and the Urdu digits switch.
    pub setup: PrintSetup,
    /// The firm header.
    pub firm: FirmNames,
    /// The time of the print.
    pub now_utc: DateTime<Utc>,
    /// The names of the collector and the city the list is filtered by.
    pub filters: CollectionFilterNames,
}

/// Reads the settings and builds the print data of a collection list. The list is the one the
/// screen reads (`services::vouchers::collection_list`), passed in by the caller, so the print and
/// the screen can never disagree (plan 17e, accepted deviation from Code Standards B2: two reads
/// of the same value, nothing is written).
///
/// # Errors
/// As [`build_collection`], and a database or data error.
pub fn collection_print_data(
    db: &Db,
    list: &CollectionList,
    filters: &CollectionFilterNames,
    language: Language,
) -> Result<CollectionPrintData, ReportsError> {
    let (firm, grouping, setup) = db.read(|conn| {
        Ok::<_, ReportsError>((load_firm(conn)?, load_grouping(conn)?, load_setup(conn)?))
    })?;
    build_collection(
        list,
        &CollectionBuildOptions {
            language,
            grouping,
            setup,
            firm,
            now_utc: db.clock().now_utc(),
            filters: filters.clone(),
        },
    )
}

/// The sum of these amounts, checked. An overflow is an error and never wraps.
fn checked_sum(amounts: impl Iterator<Item = Paisa>) -> Result<Paisa, ReportsError> {
    let mut sum = Paisa::ZERO;
    for amount in amounts {
        sum = sum.checked_add(amount)?;
    }
    Ok(sum)
}

/// Builds the print data of a collection list. Pure.
///
/// Nothing is worked out again (Code Standards B20): each buyer's dues and each total are the
/// stored values the screen shows, formatted for print. Before anything is formatted the list is
/// checked: every buyer owes more than 0, the buyers of a group add up to the group total, and the
/// groups add up to the list total (all with checked additions). A list that does not add up is
/// not printed.
///
/// The total is the total OF THIS LIST: only the active buyers whose collection day is the day's.
/// It is not the shop's total اگراہی (PRD FR-19.9).
///
/// # Errors
/// `ReportsError::Data` for a list that does not add up or a stored date that is not a date;
/// `ReportsError::Money` if an amount does not fit or is not whole rupees.
pub fn build_collection(
    list: &CollectionList,
    options: &CollectionBuildOptions,
) -> Result<CollectionPrintData, ReportsError> {
    for group in &list.groups {
        if group.buyers.iter().any(|b| b.dues_paisa <= Paisa::ZERO) {
            return Err(ReportsError::Data(
                "a buyer on the collection list owes nothing".to_owned(),
            ));
        }
        if checked_sum(group.buyers.iter().map(|b| b.dues_paisa))? != group.total_paisa {
            return Err(ReportsError::Data(
                "the buyers of a collection group do not add up to its total".to_owned(),
            ));
        }
    }
    if checked_sum(list.groups.iter().map(|g| g.total_paisa))? != list.total_paisa {
        return Err(ReportsError::Data(
            "the groups of the collection list do not add up to its total".to_owned(),
        ));
    }

    let language = options.language;
    let style = NumberStyle::new(language, options.grouping, options.setup.urdu_digits);
    let mut serial: i64 = 0;
    let mut groups = Vec::with_capacity(list.groups.len());
    for group in &list.groups {
        let mut rows = Vec::with_capacity(group.buyers.len());
        for buyer in &group.buyers {
            serial = serial.saturating_add(1);
            rows.push(CollectionPrintRow {
                serial: style.count(serial),
                // A buyer always has a short code; his full code is the fallback, never a blank.
                code: short_code_of(&buyer.account.code)
                    .unwrap_or_else(|| buyer.account.code.clone()),
                name: pick(language, &buyer.account.name_ur, &buyer.account.name_en),
                tag: buyer.account.tag.clone(),
                phone: buyer.phone.clone(),
                dues: style.money(buyer.dues_paisa)?,
                oldest_bill_date: buyer
                    .oldest_bill_date
                    .as_deref()
                    .map(|day| style.date(day))
                    .transpose()?,
                oldest_days: buyer.oldest_days.map(|days| style.count(days)),
            });
        }
        groups.push(CollectionPrintGroup {
            collector: group
                .collector
                .as_ref()
                .map(|c| pick(language, &c.name_ur, &c.name_en)),
            city: group
                .city
                .as_ref()
                .map(|c| pick(language, &c.name_ur, &c.name_en)),
            rows,
            total: style.money(group.total_paisa)?,
        });
    }

    let mut facts = vec![PrintFact {
        label_id: FACT_DATE.to_owned(),
        value: style.date(&list.date)?,
    }];
    if let Some((name_ur, name_en)) = &options.filters.collector {
        facts.push(PrintFact {
            label_id: FACT_COLLECTOR.to_owned(),
            value: pick(language, name_ur, name_en),
        });
    }
    if let Some((name_ur, name_en)) = &options.filters.city {
        facts.push(PrintFact {
            label_id: FACT_CITY.to_owned(),
            value: pick(language, name_ur, name_en),
        });
    }
    Ok(CollectionPrintData {
        doc: PrintDoc::CollectionList,
        language,
        dir: TextDir::of(language),
        paper: options.setup.paper_of(PrintDoc::CollectionList),
        urdu_digits: style.urdu_digits(),
        header: header(language, &options.firm, TITLE_COLLECTION_LIST, facts),
        footer: footer(style.printed_at(options.now_utc)),
        groups,
        total: PrintAmount {
            label_id: COLLECTION_TOTAL.to_owned(),
            amount: style.money(list.total_paisa)?,
            role: AmountRole::Total,
        },
        buyer_count: style.count(serial),
    })
}
