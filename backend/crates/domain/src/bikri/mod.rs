//! The بکری posting as one pure function (PRD 12.3 to 12.5, 22 C1 to C9 and C19, plan 12).
//!
//! [`calc_posting`] takes a typed draft that [`mandi_core::bikri::prepare_draft`] has read, the
//! facts of the accounts and items it names ([`Lookups`]), the active charge rules and the
//! Owner's switches, and gives back every line, lot and total with its working, the errors and
//! warnings, and the source of the journal. The live preview calls it now; the save of plan 13
//! calls the same function (Code Standards B5), so the screen and the books cannot differ.
//!
//! What it checks and where it points:
//!
//! | Check | Code | Field |
//! |---|---|---|
//! | no lot / no sale line in the whole بکری | `ERR_NO_LOTS`, `ERR_NO_SALE_LINES` | `lots` |
//! | seller / buyer missing, wrong kind, inactive | `ERR_SELLER_REQUIRED`, `ERR_BUYER_REQUIRED`, `ERR_NOT_A_SELLER`, `ERR_NOT_A_BUYER`, `ERR_ACCOUNT_INACTIVE` | the account field |
//! | sold more than received | `ERR_QTY_EXCEEDS_LOT` (`available`) | `lots[i].sales[j].qty` |
//! | one-time person | `ERR_PUKHTA_NAME_REQUIRED`, `ERR_PUKHTA_PHONE_REQUIRED` | `...pukhta.*` |
//! | credit on a cash-only buyer | `ERR_CREDIT_NOT_ALLOWED` | `...payment_type` |
//! | typed commission or لاگا while off | `ERR_OVERRIDE_NOT_ALLOWED` | `...seller_comm` etc. |
//! | credit limit | `ERR_CREDIT_LIMIT_EXCEEDED` (warning, or error when blocking) | `...buyer_account_id` |
//! | rate far from the average | `ERR_RATE_FAR_FROM_AVERAGE` (warning) | `...seller_rate` |
//! | lot not fully sold / net below 0 | `ERR_QTY_LEFT_UNSOLD`, `ERR_LOT_NET_NEGATIVE` (warnings) | `lots[i].qty_received`, `lots[i].seller_account_id` |
//!
//! A row is worked out only when its lot is complete (seller, item, packing, qty received) and
//! the row itself is; the summary and the journal use exactly those rows (money agent review,
//! point 4).
//!
//! ```
//! use mandi_domain::bikri::{Lookups, PostingKind, Switches, calc_posting};
//! use mandi_domain::core::bikri::{BikriDraft, HeaderInput, prepare_draft};
//! use mandi_domain::core::dates::BusinessDate;
//! use mandi_domain::rules::RuleSet;
//! // An empty draft has nothing to work out, and says so.
//! let draft = BikriDraft {
//!     header: HeaderInput {
//!         entry_date: "10-10-2026".to_owned(),
//!         bill_date: String::new(),
//!         book: None, truck_no: None, zamindar_account_id: None, adda_id: None,
//!         driver_name: None, driver_phone: None, total_freight: None, store_id: None, note: None,
//!     },
//!     lots: vec![],
//! };
//! let today = BusinessDate::from_ymd(2026, 10, 10).unwrap();
//! let prepared = prepare_draft(&draft, today, None);
//! let switches = Switches {
//!     allow_line_override: true,
//!     credit_limit_blocks: false,
//!     rate_check_percent: Some(30),
//! };
//! let calc = calc_posting(
//!     PostingKind::First, &prepared, &Lookups::default(), &RuleSet::default(), switches,
//! ).unwrap();
//! assert_eq!(calc.errors.len(), 1);
//! ```

mod checks;
mod line;
mod lookups;
mod lot;
mod process;
mod working;

use std::collections::HashMap;

use mandi_core::bikri::{LotPreview, PreparedDraft, SummaryTotals};
use mandi_core::dates::BusinessDate;
use mandi_core::error::{ErrorCode, FieldError};
use mandi_core::money::MoneyError;
use mandi_core::rules::{ChargeSide, ChargeType};
use thiserror::Error;

use self::checks::{Issues, Wanted};
use self::lot::{Acc, Scope};
use self::process::{Run, process_lot};
use crate::ledger::bikri::JournalSource;
use crate::rules::RuleSet;

pub use self::lookups::{AccountFacts, ItemFacts, ItemSales, Lookups, PartyKind, Switches};
pub use self::working::partner_share;

/// Which posting of a بکری this is (PRD FR-12.17). A later sale charges no lot charges and no
/// per-lot commission; plan 16 passes `Later`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostingKind {
    /// The first posting of a بکری.
    First,
    /// A later sale of goods left from the first posting.
    Later,
}

/// What a calculation cannot go on without. None of these is the user's mistake.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CalcError {
    /// An overflow or a bug in a money step.
    #[error("money: {0}")]
    Money(#[from] MoneyError),
    /// No commission rule at all for a side: the data is broken, a commission is never 0
    /// (plan 12 contract).
    #[error("no {charge:?} rule for the {side:?} side")]
    NoRule {
        /// The side.
        side: ChargeSide,
        /// The charge.
        charge: ChargeType,
    },
    /// Data that does not make sense.
    #[error("inconsistent data: {0}")]
    Inconsistent(&'static str),
}

/// What one call works on.
pub(crate) struct Env<'a> {
    pub kind: PostingKind,
    pub date: BusinessDate,
    pub lookups: &'a Lookups,
    pub rules: &'a RuleSet,
    pub switches: Switches,
}

/// The result of working out a draft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostingCalc {
    /// The lots, in the order of the draft.
    pub lots: Vec<LotPreview>,
    /// The lines of PRD 12.5 for the whole بکری.
    pub summary: SummaryTotals,
    /// What stops a save.
    pub errors: Vec<FieldError>,
    /// What only warns.
    pub warnings: Vec<FieldError>,
    /// The journal of the rows that were worked out (see `ledger::bikri::build_bikri_entry`).
    pub journal: JournalSource,
}

fn header_checks(issues: &mut Issues, draft: &PreparedDraft, lookups: &Lookups) {
    let h = &draft.header;
    checks::account(
        issues,
        lookups,
        h.zamindar_account_id,
        Wanted::Seller,
        "header.zamindar_account_id",
        None,
    );
    checks::master_exists(issues, &lookups.addas, h.adda_id, "header.adda_id");
    checks::master_exists(issues, &lookups.stores, h.store_id, "header.store_id");
}

/// Works out a draft: the one function of the preview and (plan 13) the save.
///
/// # Errors
/// [`CalcError`] only for overflow or broken data; every mistake of the user comes back in
/// `PostingCalc::errors` with its field.
pub fn calc_posting(
    kind: PostingKind,
    draft: &PreparedDraft,
    lookups: &Lookups,
    rules: &RuleSet,
    switches: Switches,
) -> Result<PostingCalc, CalcError> {
    let mut issues = Issues::default();
    issues.errors.extend(draft.issues.iter().cloned());
    header_checks(&mut issues, draft, lookups);
    if draft.lots.is_empty() {
        issues.error(ErrorCode::ErrNoLots, "lots");
    } else if draft
        .lots
        .iter()
        .all(|lot| lot.sales.iter().all(|row| row.is_blank))
    {
        // PRD FR-12.8 check 1: at least one lot and one sale line in the whole بکری. A lot with
        // no row of its own is allowed (PRD E3); it waits in باقی مال (tech-lead decision).
        issues.error(ErrorCode::ErrNoSaleLines, "lots");
    }
    let env = draft.header.entry_date.map(|date| Env {
        kind,
        date,
        lookups,
        rules,
        switches,
    });
    let date_iso = draft
        .header
        .entry_date
        .map(BusinessDate::to_iso)
        .unwrap_or_default();
    let mut run = Run {
        owed: HashMap::new(),
        last_rates: HashMap::new(),
    };
    let mut whole = Acc::default();
    let mut previews = Vec::with_capacity(draft.lots.len());
    let mut journal = JournalSource::default();
    for (index, lot) in draft.lots.iter().enumerate() {
        let done = process_lot(
            env.as_ref(),
            lookups,
            &mut run,
            &date_iso,
            index,
            lot,
            &mut issues,
        )?;
        if let Some(acc) = &done.acc {
            whole.merge(acc)?;
        }
        if let Some(j) = done.journal {
            journal.lots.push(j);
        }
        previews.push(done.preview);
    }
    // The warnings of each row also sit on the row, for the screen.
    for (i, lot) in previews.iter_mut().enumerate() {
        for (j, line) in lot.lines.iter_mut().enumerate() {
            let prefix = format!("lots[{i}].sales[{j}].");
            line.warnings = issues
                .warnings
                .iter()
                .filter(|w| w.field.starts_with(&prefix))
                .cloned()
                .collect();
        }
    }
    let summary = whole.totals(&Scope::Whole)?;
    Ok(PostingCalc {
        lots: previews,
        summary,
        errors: issues.errors,
        warnings: issues.warnings,
        journal,
    })
}

#[cfg(test)]
mod tests;
