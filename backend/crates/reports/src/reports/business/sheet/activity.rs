//! The sums of one party on the day: the day columns and the true ledger balance (R35).

use super::*;

/// An entry that is one half of a same-day cancel or edit: the posting dated the same day as its
/// reversal, or the reversal itself. The two cancel each other.
pub(super) const SAME_DAY_PAIR: &str = "(e.reverses_entry_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM journal_entries o
        WHERE o.id = e.reverses_entry_id AND o.entry_date = e.entry_date))
    OR EXISTS (SELECT 1 FROM journal_entries r
        WHERE r.reverses_entry_id = e.id AND r.status = 'posted' AND r.entry_date = e.entry_date)";

/// What one party (a buyer or seller account, or a person inside a پختہ کھاتہ account) has on the
/// day.
#[derive(Debug, Clone)]
pub(super) struct Activity {
    pub(super) account_id: i64,
    pub(super) person_key: Option<String>,
    pub(super) buyer: bool,
    pub(super) prev_dr: Paisa,
    pub(super) prev_cr: Paisa,
    pub(super) day_dr: Paisa,
    pub(super) day_cr: Paisa,
    /// Every posted line up to and including the day: the true ledger balance.
    pub(super) end_dr: Paisa,
    pub(super) end_cr: Paisa,
}

impl Activity {
    /// The balance at the end of the day, signed (a debit balance is positive): the posted lines
    /// up to the day, the same sums as the trial balance and R20.
    pub(super) fn end_net(&self) -> Result<Paisa, ReportsError> {
        Ok(self.end_dr.checked_sub(self.end_cr)?)
    }

    /// `previous + debit − credit = remaining` must hold; if it does not, the reversal rules
    /// have changed under this report and its numbers cannot be trusted.
    pub(super) fn check(&self) -> Result<(), ReportsError> {
        let by_day = self
            .prev_net()?
            .checked_add(self.day_dr)?
            .checked_sub(self.day_cr)?;
        if by_day != self.end_net()? {
            return Err(ReportsError::Data(
                "the day of a party does not add up to his ledger balance".to_owned(),
            ));
        }
        Ok(())
    }

    pub(super) fn prev_net(&self) -> Result<Paisa, ReportsError> {
        Ok(self.prev_dr.checked_sub(self.prev_cr)?)
    }

    /// True when the party has a balance or an entry that day.
    pub(super) fn worth_a_row(&self) -> Result<bool, ReportsError> {
        Ok(self.prev_net()? != Paisa::ZERO
            || self.day_dr != Paisa::ZERO
            || self.day_cr != Paisa::ZERO
            || self.end_net()? != Paisa::ZERO)
    }
}

/// The parties' sums for the day, optionally for one account and one person. One grouped query
/// over the posted lines of the buyer and seller accounts up to the day (Code Standards B22).
pub(super) fn activities(
    conn: &Connection,
    day: &str,
    account: Option<i64>,
    person_key: Option<&str>,
) -> Result<Vec<Activity>, ReportsError> {
    let sql = format!(
        "SELECT l.account_id, l.pukhta_person_key, k.code,
                COALESCE(SUM(CASE WHEN e.entry_date < ?1 THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date < ?1 THEN l.cr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date = ?1 AND NOT ({SAME_DAY_PAIR})
                                  THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN e.entry_date = ?1 AND NOT ({SAME_DAY_PAIR})
                                  THEN l.cr_paisa END), 0),
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0)
         FROM journal_lines l
         JOIN journal_entries e ON e.id = l.entry_id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ('{BUYER_CONTROL}', '{SELLER_CONTROL}') AND e.status = 'posted'
           AND e.entry_date <= ?1
           AND (?2 IS NULL OR l.account_id = ?2)
           AND (?3 IS NULL OR l.pukhta_person_key = ?3)
         GROUP BY l.account_id, l.pukhta_person_key, k.code
         ORDER BY l.account_id, l.pukhta_person_key"
    );
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params![day, account, person_key], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, i64>(7)?,
                r.get::<_, i64>(8)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let mut out = Vec::with_capacity(rows.len());
    for (account_id, person_key, control, prev_dr, prev_cr, day_dr, day_cr, end_dr, end_cr) in rows
    {
        let item = Activity {
            account_id,
            person_key,
            buyer: control == BUYER_CONTROL,
            prev_dr: paisa(prev_dr)?,
            prev_cr: paisa(prev_cr)?,
            day_dr: paisa(day_dr)?,
            day_cr: paisa(day_cr)?,
            end_dr: paisa(end_dr)?,
            end_cr: paisa(end_cr)?,
        };
        item.check()?;
        out.push(item);
    }
    Ok(out)
}

/// The parties of the activities as the report names them: the account's names with its tag, and
/// a one-time person with his own names and key.
pub(super) fn parties_of(
    conn: &Connection,
    items: &[Activity],
) -> Result<HashMap<(i64, Option<String>), PartyRef>, ReportsError> {
    let ids: Vec<i64> = items.iter().map(|a| a.account_id).collect();
    let accounts = load_parties(conn, &ids)?;
    let pukhta_ids: Vec<i64> = items
        .iter()
        .filter(|a| a.person_key.is_some())
        .map(|a| a.account_id)
        .collect();
    let people = pukhta_names(conn, &pukhta_ids)?;
    let mut out = HashMap::new();
    for item in items {
        let base = accounts
            .get(&item.account_id)
            .cloned()
            .ok_or_else(|| ReportsError::Data(format!("account {} is missing", item.account_id)))?;
        let party = match &item.person_key {
            Some(key) => {
                let person = people.get(key);
                pukhta_party(
                    base,
                    Some(key.clone()),
                    person.map(|p| p.name_ur.as_str()),
                    person.map(|p| p.name_en.as_str()),
                )
            }
            None => base,
        };
        out.insert((item.account_id, item.person_key.clone()), party);
    }
    Ok(out)
}

/// The signed total of a side as a balance cell.
pub(super) fn net_cell(net: Paisa) -> Result<Cell, ReportsError> {
    Ok(balance_cell(Balance::from_net(net)?))
}
