//! Money taken by a partner (his drawings account, 0603) and money lent (a person's account under
//! 0103): the returns never go above what was taken or lent, on any day (PRD FR-16.2, FR-16.5;
//! money agent 8.2b: the open amount is the debit balance and is never negative).
//!
//! The books of such an account are read as one net (debit less credit) for each day, oldest day
//! first, and the running balance at the end of each day is worked out here with checked maths.
//! Three questions are asked of it, by the save, the edit and cancel, the preview and the openings,
//! all through this one place (Code Standards B5):
//!
//! - how much can still be returned on a day ([`DayNets::open_on`]): the smaller of the balance at
//!   the end of that day and every later day-end balance, so a return dated before its take, or
//!   one that would leave a later day overdrawn, is refused;
//! - whether a change would leave some day overdrawn, or more overdrawn than it was, comparing the
//!   books before and after day by day ([`worst_excess`]).

use mandi_db::ledger::{self as ledger_repo, StoredEntry};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::error::{ErrorCode, ErrorValue, FieldError};
use mandi_domain::core::money::Paisa;
use mandi_domain::core::vouchers::VoucherType;

use crate::error::{Rejection, ServiceError, money_error};

/// The field a refusal of a return above what is open is blamed on.
pub(crate) const AMOUNT_FIELD: &str = "amount";

/// The controls whose accounts keep money to be returned: a partner's drawings (0603) and a
/// person's loan account (0103).
pub(crate) fn is_returnable_control(control_code: &str) -> bool {
    matches!(control_code, "0603" | "0103")
}

/// True for the voucher kinds that take money out to be returned: ذاتی خرچہ and ادھار رقم.
pub(crate) const fn is_take_kind(kind: VoucherType) -> bool {
    matches!(kind, VoucherType::PersonalExpense | VoucherType::MoneyLent)
}

/// The debit side less the credit side of an entry's lines on one account.
///
/// # Errors
/// `ERR_OVERFLOW` or a data error for an amount that is not whole rupees.
pub(crate) fn net_on(entry: &StoredEntry, account_id: i64) -> Result<Paisa, ServiceError> {
    let mut net = Paisa::ZERO;
    for line in entry.lines.iter().filter(|l| l.account_id == account_id) {
        let dr = Paisa::try_from_paisa(line.dr_paisa).map_err(money_error)?;
        let cr = Paisa::try_from_paisa(line.cr_paisa).map_err(money_error)?;
        net = net
            .checked_add(dr)
            .and_then(|v| v.checked_sub(cr))
            .map_err(money_error)?;
    }
    Ok(net)
}

/// The net (debit less credit) of one account for each day, oldest day first.
#[derive(Debug, Clone)]
pub(crate) struct DayNets {
    days: Vec<(String, Paisa)>,
}

impl DayNets {
    /// Reads the posted history of an account.
    ///
    /// # Errors
    /// `ServiceError::Db`, `ERR_OVERFLOW`.
    pub(crate) fn load(conn: &Connection, account_id: i64) -> Result<Self, ServiceError> {
        let mut days = Vec::new();
        for found in ledger_repo::day_totals(conn, account_id)? {
            let dr = Paisa::try_from_paisa(found.dr_paisa).map_err(money_error)?;
            let cr = Paisa::try_from_paisa(found.cr_paisa).map_err(money_error)?;
            days.push((found.day, dr.checked_sub(cr).map_err(money_error)?));
        }
        Ok(Self { days })
    }

    /// Adds `net` to the day (a new day is put in its place).
    ///
    /// # Errors
    /// `ERR_OVERFLOW`.
    pub(crate) fn with(mut self, day: &str, net: Paisa) -> Result<Self, ServiceError> {
        match self.days.binary_search_by(|(d, _)| d.as_str().cmp(day)) {
            Ok(index) => {
                if let Some(found) = self.days.get_mut(index) {
                    found.1 = found.1.checked_add(net).map_err(money_error)?;
                }
            }
            Err(index) => self.days.insert(index, (day.to_owned(), net)),
        }
        Ok(self)
    }

    /// Takes a posting out (an edit replaces it): its net on the account leaves its day.
    ///
    /// # Errors
    /// `ERR_OVERFLOW`.
    pub(crate) fn without(
        self,
        entry: &StoredEntry,
        account_id: i64,
    ) -> Result<Self, ServiceError> {
        let net = net_on(entry, account_id)?
            .checked_neg()
            .map_err(money_error)?;
        self.with(&entry.entry_date, net)
    }

    /// The balance now: the sum of every day.
    ///
    /// # Errors
    /// `ERR_OVERFLOW`.
    pub(crate) fn balance(&self) -> Result<Paisa, ServiceError> {
        self.days.iter().try_fold(Paisa::ZERO, |sum, (_, net)| {
            sum.checked_add(*net).map_err(money_error)
        })
    }

    /// The balance at the end of `day`: the sum of every day up to and including it.
    ///
    /// # Errors
    /// `ERR_OVERFLOW`.
    pub(crate) fn running_at(&self, day: &str) -> Result<Paisa, ServiceError> {
        self.days
            .iter()
            .take_while(|(d, _)| d.as_str() <= day)
            .try_fold(Paisa::ZERO, |sum, (_, net)| {
                sum.checked_add(*net).map_err(money_error)
            })
    }

    /// What can be returned by an entry dated `day`: never below 0, and never more than the
    /// balance at the end of `day` or at the end of any later day (a return on `day` lowers all of
    /// them). With no lines after `day` this is the balance as of `day`, and with no returns dated
    /// later it is the balance now (PRD FR-16.2, FR-16.5).
    ///
    /// # Errors
    /// `ERR_OVERFLOW`.
    pub(crate) fn open_on(&self, day: &str) -> Result<Paisa, ServiceError> {
        let mut running = Paisa::ZERO;
        let mut up_to_day = Paisa::ZERO;
        let mut later_low: Option<Paisa> = None;
        for (d, net) in &self.days {
            running = running.checked_add(*net).map_err(money_error)?;
            if d.as_str() <= day {
                up_to_day = running;
            } else {
                later_low = Some(later_low.map_or(running, |low| low.min(running)));
            }
        }
        let open = later_low.map_or(up_to_day, |low| low.min(up_to_day));
        Ok(open.max(Paisa::ZERO))
    }
}

/// The largest part below 0 that a change leaves on a day, if on some day the balance at the end
/// of the day is now below 0 and lower than it was. The books are compared DAY BY DAY, so a deeper
/// hole on an earlier day cannot hide a new one on a later day. A history that was already
/// overdrawn (a جنرل credit above the open amount is allowed) does not block a change that makes
/// no day worse.
///
/// # Errors
/// `ERR_OVERFLOW`.
pub(crate) fn worst_excess(
    before: &DayNets,
    after: &DayNets,
) -> Result<Option<Paisa>, ServiceError> {
    let mut worst: Option<Paisa> = None;
    for (day, _) in before.days.iter().chain(after.days.iter()) {
        let was = before.running_at(day)?;
        let now = after.running_at(day)?;
        if now.is_negative() && now < was {
            let over = now.checked_neg().map_err(money_error)?;
            worst = Some(worst.map_or(over, |w| w.max(over)));
        }
    }
    Ok(worst)
}

/// The refusal: returns would be `over` above what was taken or lent, 0 open (PRD FR-16.2,
/// FR-16.5).
///
pub(crate) fn field_error(over: Paisa, field: &str) -> FieldError {
    FieldError::new(ErrorCode::ErrReturnExceedsOpen, field)
        .with("open_paisa", ErrorValue::Number(0))
        .with("over_paisa", ErrorValue::Number(over.as_paisa()))
}

/// The same refusal as an error of a save.
pub(crate) fn rejection(over: Paisa, field: &str) -> ServiceError {
    ServiceError::Rejected(vec![Rejection::from(field_error(over, field))])
}

/// The day-end balances of an account now. Read BEFORE a change, to be compared by
/// [`check_after`].
///
/// # Errors
/// `ServiceError::Db`, `ERR_OVERFLOW`.
pub(crate) fn floor_of(conn: &Connection, account_id: i64) -> Result<(i64, DayNets), ServiceError> {
    Ok((account_id, DayNets::load(conn, account_id)?))
}

/// [`floor_of`] for a take or a loan kind; `None` for every other kind (nothing to check).
///
/// # Errors
/// `ServiceError::Db`, `ERR_OVERFLOW`.
pub(crate) fn floor_before(
    conn: &Connection,
    kind: VoucherType,
    account_id: Option<i64>,
) -> Result<Option<(i64, DayNets)>, ServiceError> {
    match (is_take_kind(kind), account_id) {
        (true, Some(id)) => Ok(Some(floor_of(conn, id)?)),
        _ => Ok(None),
    }
}

/// After an edit, a cancel or an opening change: no day may end lower, and below 0, than it did
/// before. The refusal is blamed on `field`.
///
/// # Errors
/// `ERR_RETURN_EXCEEDS_OPEN` (values `open_paisa` 0 and `over_paisa`).
pub(crate) fn check_after(
    conn: &Connection,
    before: Option<(i64, DayNets)>,
    field: &str,
) -> Result<(), ServiceError> {
    let Some((account_id, was)) = before else {
        return Ok(());
    };
    let now = DayNets::load(conn, account_id)?;
    match worst_excess(&was, &now)? {
        Some(over) => Err(rejection(over, field)),
        None => Ok(()),
    }
}

/// What the check after a change WOULD say, for a preview: the posting being replaced (`old`: its
/// account and entry) leaves the books and the new one (`new`: its account, day and debit amount)
/// comes in. The decision is the same as [`check_after`] (Code Standards B5). `None` when nothing
/// is overdrawn.
///
/// # Errors
/// `ServiceError::Db`, `ERR_OVERFLOW`.
pub(crate) fn what_if(
    conn: &Connection,
    old: Option<(i64, &StoredEntry)>,
    new: Option<(i64, &str, Paisa)>,
    field: &str,
) -> Result<Option<FieldError>, ServiceError> {
    let mut accounts: Vec<i64> = Vec::new();
    for id in [old.map(|o| o.0), new.map(|n| n.0)].into_iter().flatten() {
        if !accounts.contains(&id) {
            accounts.push(id);
        }
    }
    for id in accounts {
        let before = DayNets::load(conn, id)?;
        let mut after = before.clone();
        if let Some((old_id, entry)) = old
            && old_id == id
        {
            after = after.without(entry, id)?;
        }
        if let Some((new_id, day, amount)) = new
            && new_id == id
        {
            after = after.with(day, amount)?;
        }
        if let Some(over) = worst_excess(&before, &after)? {
            return Ok(Some(field_error(over, field)));
        }
    }
    Ok(None)
}

/// [`what_if`] for the voucher previews: only a take or a loan kind is checked, and the refusal is
/// blamed on `amount`.
///
/// # Errors
/// `ServiceError::Db`, `ERR_OVERFLOW`.
pub(crate) fn preview_refusal(
    conn: &Connection,
    kind: VoucherType,
    old: Option<(i64, &StoredEntry)>,
    new: Option<(i64, &str, Paisa)>,
) -> Result<Option<FieldError>, ServiceError> {
    if is_take_kind(kind) {
        what_if(conn, old, new, AMOUNT_FIELD)
    } else {
        Ok(None)
    }
}
