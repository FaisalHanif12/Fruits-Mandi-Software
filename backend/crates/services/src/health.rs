//! The daily books check (plan 22 early, NFR-REL-8, NFR-OBS-5, NFR-SEC-6): the trial balance
//! difference must be 0, and the keyed hash chains of the journal and of the audit log must
//! still match. It reads posted records only and changes nothing in the books.
//!
//! The result of the latest run is kept in the `settings` row `health_last` (a system record, not
//! a business record) so the Owner sees it without waiting for a new run. A run that finds a
//! problem also writes one audit row (`check_failed` on `health`), so the problem is in the
//! audit viewer too. No secret and no balance is written: only the two totals' difference, the
//! chain states and counts.
//!
//! Example (a healthy book): debit 85,048 and credit 85,048, difference 0, both chains `ok`.

use mandi_db::audit::VerifyError;
use serde_json::{Value, json};

use crate::audit::Event;
use crate::context::Context;
use crate::error::ServiceError;
use crate::ledger;
use crate::store;
use mandi_domain::core::money::Paisa;

/// The `settings` key of the latest result.
pub const KEY_HEALTH_LAST: &str = "health_last";

/// How a keyed hash chain stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainState {
    /// Every hash matches.
    Ok,
    /// A row was changed or removed outside the app. The text names the first entry.
    Broken(String),
}

impl ChainState {
    /// True for a chain that matches.
    pub fn is_ok(&self) -> bool {
        matches!(self, ChainState::Ok)
    }
}

/// The result of the books check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BooksCheck {
    /// Σ of the debit balances of all accounts (the debit total of the trial balance).
    pub debit: Paisa,
    /// Σ of the credit balances of all accounts.
    pub credit: Paisa,
    /// `debit − credit`. 0 when the books balance (it can be negative if they do not).
    pub difference: Paisa,
    /// The journal chain (NFR-SEC-6).
    pub journal_chain: ChainState,
    /// The audit chain (NFR-SEC-6).
    pub audit_chain: ChainState,
    /// Posted journal entries.
    pub journal_entries: i64,
    /// Audit rows.
    pub audit_rows: i64,
}

impl BooksCheck {
    /// True when the books balance and both chains match.
    pub fn is_ok(&self) -> bool {
        self.difference == Paisa::ZERO && self.journal_chain.is_ok() && self.audit_chain.is_ok()
    }

    /// The JSON kept as the latest result: counts, the difference and the chain states; no
    /// balance of any account.
    pub fn summary_json(&self) -> Value {
        let chain = |state: &ChainState| match state {
            ChainState::Ok => json!({ "ok": true }),
            ChainState::Broken(at) => json!({ "ok": false, "at": at }),
        };
        json!({
            "balanced": self.difference == Paisa::ZERO,
            "difference_paisa": self.difference.as_paisa(),
            "journal_chain": chain(&self.journal_chain),
            "audit_chain": chain(&self.audit_chain),
            "journal_entries": self.journal_entries,
            "audit_rows": self.audit_rows,
        })
    }
}

fn line_paisa(value: i64) -> Result<Paisa, ServiceError> {
    Paisa::try_from_paisa(value).map_err(data_error)
}

/// A fixed text on purpose: the cause may carry an amount, and no amount goes to a log
/// (NFR-OBS-2).
fn data_error(_cause: impl std::fmt::Display) -> ServiceError {
    ServiceError::Data(
        "the books check: a line total is not a whole rupee or does not fit".to_owned(),
    )
}

/// Runs the books check now.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` if a total does not fit or a line is not a whole
/// rupee. A broken chain or a difference is NOT an error: it is the answer. (A check that cannot
/// run is stored as a failed result by the caller, see [`store_failed`].)
pub fn check_books(ctx: &Context) -> Result<BooksCheck, ServiceError> {
    let (debit, credit, lines) = ctx.db.read(|conn| {
        let trial = ledger::trial_balance_unchecked(conn, None)?;
        // The trial balance walks the accounts of the tree. A line of an account that has no
        // row would be left out of it, so the sum of every posted line is worked out too.
        let mut line_dr = Paisa::ZERO;
        let mut line_cr = Paisa::ZERO;
        for totals in ledger::balances_by_account(conn, None)?.values() {
            line_dr = line_dr
                .checked_add(line_paisa(totals.dr_paisa)?)
                .map_err(data_error)?;
            line_cr = line_cr
                .checked_add(line_paisa(totals.cr_paisa)?)
                .map_err(data_error)?;
        }
        let lines = line_dr.checked_sub(line_cr).map_err(data_error)?;
        Ok::<_, ServiceError>((trial.total_dr, trial.total_cr, lines))
    })?;
    let from_accounts = debit.checked_sub(credit).map_err(data_error)?;
    // Either way of counting that finds a gap is the answer: the books do not balance.
    let difference = if lines == Paisa::ZERO {
        from_accounts
    } else {
        lines
    };
    let journal_chain = match ledger::verify_journal_chain(ctx) {
        Ok(()) => ChainState::Ok,
        Err(ServiceError::Data(found)) => ChainState::Broken(found),
        Err(other) => return Err(other),
    };
    let key = ctx.db.auditor().key()?;
    let audit_chain =
        ctx.db.read(
            |conn| match mandi_db::audit::verify_audit_chain(conn, &key) {
                Ok(()) => Ok(ChainState::Ok),
                Err(VerifyError::Broken(found)) => Ok(ChainState::Broken(found.to_string())),
                Err(VerifyError::Db(db)) => Err(ServiceError::Db(db)),
            },
        )?;
    let (journal_entries, audit_rows) = ctx.db.read(|conn| {
        let entries: i64 = conn.query_row(
            "SELECT COUNT(*) FROM journal_entries WHERE status = 'posted'",
            [],
            |r| r.get(0),
        )?;
        let rows: i64 = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |r| r.get(0))?;
        Ok::<_, ServiceError>((entries, rows))
    })?;
    Ok(BooksCheck {
        debit,
        credit,
        difference,
        journal_chain,
        audit_chain,
        journal_entries,
        audit_rows,
    })
}

/// Keeps the latest result (the whole health report as JSON, built by the caller) and, when
/// the check found a problem, writes one audit row about it.
///
/// # Errors
/// `ServiceError::Db`.
pub fn store_last(
    ctx: &Context,
    report: &Value,
    books: &BooksCheck,
    user_id: Option<i64>,
) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        store::set_setting(tx, KEY_HEALTH_LAST, report, user_id)?;
        if !books.is_ok() {
            ctx.db.auditor().write(
                tx,
                &Event::new(user_id, "check_failed", "health")
                    .new_value(books.summary_json())
                    .into_entry(),
            )?;
        }
        Ok(())
    })
}

/// Keeps a result for a check that could not run, and writes one audit row about it. The row
/// holds no reason text and no amount, only that the check did not run. This replaces an older
/// "ok", so the Owner never sees a stale good result.
///
/// # Errors
/// `ServiceError::Db`.
pub fn store_failed(
    ctx: &Context,
    report: &Value,
    user_id: Option<i64>,
) -> Result<(), ServiceError> {
    ctx.db.write(|tx| {
        store::set_setting(tx, KEY_HEALTH_LAST, report, user_id)?;
        ctx.db.auditor().write(
            tx,
            &Event::new(user_id, "check_failed", "health")
                .new_value(json!({ "ran": false }))
                .into_entry(),
        )?;
        Ok(())
    })
}

/// The latest stored result, if a check has run.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` if the stored text is not JSON.
pub fn last(ctx: &Context) -> Result<Option<Value>, ServiceError> {
    ctx.db.read(|conn| store::setting(conn, KEY_HEALTH_LAST))
}
