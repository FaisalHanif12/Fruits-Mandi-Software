//! The sums of one party on the day: the day columns and the true ledger balance (R35).

use std::collections::BTreeMap;

use super::*;

/// An entry that is one half of a same-day cancel or edit: the posting dated the same day as its
/// reversal, or the reversal itself. The two cancel each other.
pub(super) const SAME_DAY_PAIR: &str = "(e.reverses_entry_id IS NOT NULL AND EXISTS (
        SELECT 1 FROM journal_entries o
        WHERE o.id = e.reverses_entry_id AND o.entry_date = e.entry_date))
    OR EXISTS (SELECT 1 FROM journal_entries r
        WHERE r.reverses_entry_id = e.id AND r.status = 'posted' AND r.entry_date = e.entry_date)";

/// What one party (a buyer or seller account, or a person inside a چکتا کھاتہ account) has on the
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

/// What one party has on one day, from the three reads of [`activities`].
#[derive(Debug, Default, Clone, Copy)]
struct DayParts {
    prev_dr: i64,
    prev_cr: i64,
    /// Every posted line of the day.
    all_dr: i64,
    all_cr: i64,
    /// The lines of the day that are not half of a same-day cancel or edit.
    day_dr: i64,
    day_cr: i64,
}

/// The lines of the entries of one day, grouped by party: `?1` the day, `?2` an account or null, `?3`
/// a person or null. Public for the query plan test (Code Standards B18).
pub(crate) fn day_lines_sql() -> String {
    format!(
        "SELECT l.account_id, COALESCE(l.chukta_person_key, ''), k.code,
                COALESCE(SUM(l.dr_paisa), 0), COALESCE(SUM(l.cr_paisa), 0),
                COALESCE(SUM(CASE WHEN NOT ({SAME_DAY_PAIR}) THEN l.dr_paisa END), 0),
                COALESCE(SUM(CASE WHEN NOT ({SAME_DAY_PAIR}) THEN l.cr_paisa END), 0)
         FROM journal_entries e INDEXED BY idx_journal_entries_entry_date
         CROSS JOIN journal_lines l ON l.entry_id = e.id
         JOIN accounts a ON a.id = l.account_id
         JOIN account_controls k ON k.id = a.control_id
         WHERE k.code IN ('{BUYER_CONTROL}', '{SELLER_CONTROL}') AND e.status = 'posted'
           AND e.entry_date = ?1
           AND (?2 IS NULL OR l.account_id = ?2)
           AND (?3 IS NULL OR l.chukta_person_key = ?3)
         GROUP BY l.account_id, l.chukta_person_key, k.code"
    )
}

/// The parties' sums for the day, optionally for one account and one person (Code Standards B22,
/// NFR-SPD-24).
///
/// Three reads, none of them a pass over the journal joined with its entries:
///
/// 1. the balance before the day of every buyer and seller (`party_reads::totals_before`: the
///    all-time sums from the covering index less the lines of the few entries dated on or after the
///    day, 0.3 seconds for ten years, or the lines before the day when few entries are older);
/// 2. the lines of the entries of the day, found by the day index, grouped by party: all of them,
///    and those that are not half of a same-day cancel or edit (the day columns).
///
/// The balance at the end of the day is the balance before it plus every line of the day, which is
/// every posted line up to and including the day. The one grouped query over all the lines of the
/// buyers and sellers up to the day that did this before took 16 to 28 seconds on the NFR-SCL-1
/// volume, and the same query was run again for every page.
pub(super) fn activities(
    conn: &Connection,
    day: &str,
    account: Option<i64>,
    person_key: Option<&str>,
) -> Result<Vec<Activity>, ReportsError> {
    let controls = [BUYER_CONTROL, SELLER_CONTROL];
    let scope = mandi_db::party_reads::PartyScope {
        account_id: account,
        person_key,
    };
    // (account, person) -> the sums. A line with no person key has the empty one, as in the
    // balance read.
    let mut parts: BTreeMap<(i64, String), (String, DayParts)> = BTreeMap::new();
    // Two groups of one account that the read gives the same key (a line with no person key and a
    // line with an empty one) are never merged silently: that is an error, not a lost sum.
    let twice =
        || ReportsError::Data("two groups of lines of one party have the same key".to_owned());
    for before in mandi_db::party_reads::totals_before(conn, &controls, day, scope)? {
        let slot = parts
            .entry((before.account_id, before.person_key))
            .or_insert_with(|| (before.control_code.clone(), DayParts::default()));
        if slot.1.prev_dr != 0 || slot.1.prev_cr != 0 {
            return Err(twice());
        }
        slot.1.prev_dr = before.dr_paisa;
        slot.1.prev_cr = before.cr_paisa;
    }
    let sql = day_lines_sql();
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params![day, account, person_key], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, i64>(6)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (account_id, key, control, all_dr, all_cr, day_dr, day_cr) in rows {
        let slot = parts
            .entry((account_id, key))
            .or_insert_with(|| (control, DayParts::default()));
        if slot.1.all_dr != 0 || slot.1.all_cr != 0 {
            return Err(twice());
        }
        slot.1.all_dr = all_dr;
        slot.1.all_cr = all_cr;
        slot.1.day_dr = day_dr;
        slot.1.day_cr = day_cr;
    }
    let overflow = || ReportsError::Data("the sums of a party's day do not fit".to_owned());
    let mut out = Vec::with_capacity(parts.len());
    for ((account_id, key), (control, p)) in parts {
        let item = Activity {
            account_id,
            person_key: (!key.is_empty()).then_some(key),
            buyer: control == BUYER_CONTROL,
            prev_dr: paisa(p.prev_dr)?,
            prev_cr: paisa(p.prev_cr)?,
            day_dr: paisa(p.day_dr)?,
            day_cr: paisa(p.day_cr)?,
            end_dr: paisa(p.prev_dr.checked_add(p.all_dr).ok_or_else(overflow)?)?,
            end_cr: paisa(p.prev_cr.checked_add(p.all_cr).ok_or_else(overflow)?)?,
        };
        item.check()?;
        out.push(item);
    }
    Ok(out)
}

/// One party's sums of the day as plain numbers, for the test that holds [`activities`] to the
/// grouped query it replaced.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayActivity {
    /// The account.
    pub account_id: i64,
    /// The person of a چکتا کھاتہ account.
    pub person_key: Option<String>,
    /// True for a buyer.
    pub buyer: bool,
    /// Σ debits and credits before the day.
    pub prev: (i64, i64),
    /// Σ debits and credits of the day, a same-day cancel or edit left out.
    pub day: (i64, i64),
    /// Σ debits and credits up to and including the day.
    pub end: (i64, i64),
}

/// [`activities`] for the whole book as plain numbers.
///
/// # Errors
/// As [`activities`].
#[doc(hidden)]
pub fn day_activities(conn: &Connection, day: &str) -> Result<Vec<DayActivity>, ReportsError> {
    Ok(activities(conn, day, None, None)?
        .into_iter()
        .map(|a| DayActivity {
            account_id: a.account_id,
            person_key: a.person_key,
            buyer: a.buyer,
            prev: (a.prev_dr.as_paisa(), a.prev_cr.as_paisa()),
            day: (a.day_dr.as_paisa(), a.day_cr.as_paisa()),
            end: (a.end_dr.as_paisa(), a.end_cr.as_paisa()),
        })
        .collect())
}

/// The parties of the activities as the report names them: the account's names with its tag, and
/// a چکتا buyer with his own names and key.
pub(super) fn parties_of(
    conn: &Connection,
    items: &[Activity],
) -> Result<HashMap<(i64, Option<String>), PartyRef>, ReportsError> {
    let keys: Vec<(i64, Option<String>)> = items
        .iter()
        .map(|a| (a.account_id, a.person_key.clone()))
        .collect();
    parties_for(conn, &keys)
}

/// The parties of some (account, person) keys as the report names them: the account's names with
/// its tag, and a چکتا buyer with his own names, phone and key.
pub(crate) fn parties_for(
    conn: &Connection,
    keys: &[(i64, Option<String>)],
) -> Result<HashMap<(i64, Option<String>), PartyRef>, ReportsError> {
    let ids: Vec<i64> = keys.iter().map(|k| k.0).collect();
    let accounts = load_parties(conn, &ids)?;
    let chukta_ids: Vec<i64> = keys.iter().filter(|k| k.1.is_some()).map(|k| k.0).collect();
    let people = chukta_names(conn, &chukta_ids)?;
    let mut out = HashMap::new();
    for (account_id, person_key) in keys {
        let base = accounts
            .get(account_id)
            .cloned()
            .ok_or_else(|| ReportsError::Data(format!("account {account_id} is missing")))?;
        let party = match person_key {
            Some(key) => {
                let person = people.get(key);
                chukta_party(
                    base,
                    Some(key.clone()),
                    person.map(|p| p.name_ur.as_str()),
                    person.map(|p| p.name_en.as_str()),
                )
            }
            None => base,
        };
        out.insert((*account_id, person_key.clone()), party);
    }
    Ok(out)
}

/// The signed total of a side as a balance cell.
pub(super) fn net_cell(net: Paisa) -> Result<Cell, ReportsError> {
    Ok(balance_cell(Balance::from_net(net)?))
}

/// An item with the keys it is sorted by: the Urdu name, the tag and the (account, person) key.
type Sortable<T> = (Vec<u8>, Option<String>, (i64, Option<String>), T);

/// The items in the order of their party's Urdu name (the alphabetical order of Urdu); two people
/// with the same name are told apart by the tag, then by the account and the person (the order of
/// the all-parties table of R35 and of the tables of R34).
pub(crate) fn sort_by_party<T>(
    items: Vec<T>,
    parties: &HashMap<(i64, Option<String>), PartyRef>,
    key_of: impl Fn(&T) -> (i64, Option<String>),
) -> Vec<T> {
    let collator = UrduCollator::new().ok();
    let sort_key = |name: &str| -> Vec<u8> {
        collator
            .as_ref()
            .map_or_else(|| name.as_bytes().to_vec(), |c| c.sort_key(name))
    };
    let mut keyed: Vec<Sortable<T>> = items
        .into_iter()
        .map(|item| {
            let key = key_of(&item);
            let party = parties.get(&key);
            let name = party.map(|p| p.name_ur.clone()).unwrap_or_default();
            (
                sort_key(&name),
                party.and_then(|p| p.tag.clone()),
                key,
                item,
            )
        })
        .collect();
    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.0.cmp(&b.2.0))
            .then(a.2.1.cmp(&b.2.1))
    });
    keyed.into_iter().map(|k| k.3).collect()
}
