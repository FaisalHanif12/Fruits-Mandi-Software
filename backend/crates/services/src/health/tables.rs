//! The money tables compared with the posted journal (NFR-SEC-6 v1.21, NFR-OBS-5, plan 80 Q-T3-3).
//!
//! The journal and the audit log are in keyed hash chains. The other money tables are protected
//! by the journal: each can be rebuilt from it and compared. This module reads the sums with SQL,
//! one query for each comparison and none inside a loop (Code Standards B22), in ONE read snapshot,
//! so a save that commits meanwhile cannot show in some of them and not in others. The money is
//! worked out and compared in `mandi_domain::books_check` with checked `Paisa` operations.
//!
//! 1. Sale lines and lots of each بکری posting against that entry's journal lines (PRD FR-12.9).
//! 2. Each lot's stored gross, deductions and net against its live sale lines and charges
//!    (PRD C6, C7).
//! 3. The partner share rows of each entry against the split of its income and shop expense
//!    lines (PRD FR-6.5, C9 to C11).
//! 4. The stock moves of each lot, purchase and garden arrival against its stored quantities
//!    (PRD C16).
//! 5. The analytics facts against the live sale lines (PRD FR-18.6, FR-18.7).
//!
//! It reads only. A reason names the table and the record and holds no amount (NFR-OBS-2).

use std::collections::{HashMap, HashSet};

use mandi_db::rusqlite::{Connection, Row};
use mandi_domain::books_check::{
    BuyerSums, Difference, LineSums, LotPosting, LotRow, PersonLines, PersonSale, PostingEntry,
    PostingInputs, SaleSums, ShareHistory, ShareStates, StoredShare, SystemAccounts, TripPart,
    bases_of, compare_persons, compare_postings, lot_difference, reversal_rows_match,
    share_rows_match, weight_line_difference,
};
use mandi_domain::core::ids::{AccountId, EntryId, PartnerId};
use mandi_domain::core::money::{Bps, Paisa};
use mandi_domain::partners::ShareKind;

use crate::error::ServiceError;

/// How many posting entries one chunk of the postings comparison holds.
const POSTINGS_PER_CHUNK: usize = 20_000;

/// The most reasons one run keeps. The count of all of them is kept apart.
const MAX_REASONS: usize = 50;

/// What the comparisons found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    /// How many differences there are.
    pub count: usize,
    /// The first of them, at most [`MAX_REASONS`], each naming a table and a record.
    pub reasons: Vec<Difference>,
}

impl Found {
    fn add(&mut self, difference: Difference) {
        self.count = self.count.saturating_add(1);
        if self.reasons.len() < MAX_REASONS {
            self.reasons.push(difference);
        }
    }

    fn add_all(&mut self, differences: impl IntoIterator<Item = Difference>) {
        for difference in differences {
            self.add(difference);
        }
    }
}

/// A fixed text: the cause may carry an amount, and no amount goes to a log (NFR-OBS-2).
fn data_error(_cause: impl std::fmt::Display) -> ServiceError {
    ServiceError::Data("the books check: a table value does not fit".to_owned())
}

/// A stored money value. A value that is not a whole rupee is a difference of the table, and the
/// comparison goes on with 0 so the rest is still checked.
fn money(found: &mut Found, table: &'static str, record: &str, value: i64) -> Paisa {
    match Paisa::try_from_paisa(value) {
        Ok(paisa) => paisa,
        Err(_) => {
            found.add(Difference::new(
                table,
                format!("{record}: a value is not a whole rupee"),
            ));
            Paisa::ZERO
        }
    }
}

/// Runs the five comparisons in one read snapshot.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` when a system account is missing or a sum does not
/// fit. A difference is NOT an error: it is the answer.
pub(super) fn compare(ctx: &crate::context::Context) -> Result<Found, ServiceError> {
    ctx.db.read_snapshot(compare_connection)
}

/// Runs the five comparisons on one connection. The caller makes sure the statements see one
/// moment of the data (a read transaction), as [`compare`] does. A test may call it on a raw
/// connection to the data file, to see that no book it builds is flagged.
///
/// # Errors
/// As [`compare`].
pub fn compare_connection(conn: &Connection) -> Result<Found, ServiceError> {
    let mut found = Found::default();
    postings(conn, &mut found)?;
    chukta_persons(conn, &mut found)?;
    weight_lines(conn, &mut found)?;
    lots(conn, &mut found)?;
    share_rows(conn, &mut found)?;
    stock(conn, &mut found)?;
    facts(conn, &mut found)?;
    Ok(found)
}

fn account_id(row: &Row<'_>, at: usize) -> Result<AccountId, mandi_db::rusqlite::Error> {
    Ok(AccountId::new(row.get(at)?))
}

fn entry_id(row: &Row<'_>, at: usize) -> Result<EntryId, mandi_db::rusqlite::Error> {
    Ok(EntryId::new(row.get(at)?))
}

/// The ten system accounts of a بکری posting, by code (PRD FR-10.4).
fn system_accounts(conn: &Connection) -> Result<SystemAccounts, ServiceError> {
    let mut by_code: HashMap<String, i64> = HashMap::new();
    let mut statement = conn.prepare(
        "SELECT code, id FROM accounts WHERE code IN ('01010001', '04010001', '04010002',
             '04010003', '04010004', '04010005', '02010001', '02010002', '02010003', '02010004')",
    )?;
    let rows = statement.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
    for row in rows {
        let (code, id) = row?;
        by_code.insert(code, id);
    }
    let get = |code: &str| -> Result<AccountId, ServiceError> {
        by_code
            .get(code)
            .map(|id| AccountId::new(*id))
            .ok_or_else(|| {
                ServiceError::Data("the books check: a system account is missing".to_owned())
            })
    };
    Ok(SystemAccounts {
        cash: get("01010001")?,
        seller_commission: get("04010001")?,
        buyer_commission: get("04010002")?,
        laga: get("04010003")?,
        phone: get("04010004")?,
        rate_diff: get("04010005")?,
        labour: get("02010001")?,
        freight: get("02010002")?,
        store_rent: get("02010003")?,
        munshiana: get("02010004")?,
    })
}

/// Comparison 1: the sale lines and lots of each posting against its journal lines.
fn postings(conn: &Connection, found: &mut Found) -> Result<(), ServiceError> {
    let system = system_accounts(conn)?;
    // A posting is live when no entry reverses it.
    let reversed: HashSet<i64> = {
        let mut statement = conn.prepare(
            "SELECT reverses_entry_id FROM journal_entries
             WHERE reverses_entry_id IS NOT NULL AND status = 'posted'",
        )?;
        let rows = statement.query_map([], |r| r.get::<_, i64>(0))?;
        rows.collect::<Result<_, _>>()?
    };
    let entries: Vec<PostingEntry> = {
        let mut statement = conn.prepare(
            "SELECT id FROM journal_entries
             WHERE status = 'posted' AND posting_type IN ('bikri_first', 'bikri_later')
             ORDER BY id",
        )?;
        let rows = statement.query_map([], |r| r.get::<_, i64>(0))?;
        let mut out = Vec::new();
        for row in rows {
            let id = row?;
            out.push(PostingEntry {
                entry_id: EntryId::new(id),
                live: !reversed.contains(&id),
            });
        }
        out
    };
    let live: HashSet<EntryId> = entries
        .iter()
        .filter(|e| e.live)
        .map(|e| e.entry_id)
        .collect();
    // The linked money mirror: each partner's 0605 account and the contra 03070001 (PRD FR-16.6).
    let mirror: Vec<AccountId> = {
        let mut statement = conn.prepare(
            "SELECT linked_account_id FROM partners WHERE linked_account_id IS NOT NULL
             UNION SELECT id FROM accounts WHERE code = '03070001'",
        )?;
        let rows = statement.query_map([], |r| r.get::<_, i64>(0))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(AccountId::new(row?));
        }
        out
    };

    // The postings are compared a chunk of entries at a time (a range of entry ids), so a book of
    // years never has its whole journal in memory at once (NFR-SPD-16). Each chunk is one query for
    // each table, not one per row (Code Standards B22). The ranges join up, so a sale line that
    // names an entry outside every posting is in some chunk and is found.
    let context = PostingContext {
        system,
        mirror,
        live: &live,
    };
    let mut start = 0_i64;
    let mut at = 0_usize;
    loop {
        let end_at = at.saturating_add(POSTINGS_PER_CHUNK).min(entries.len());
        let last = end_at >= entries.len();
        let high = if last {
            i64::MAX
        } else {
            entries
                .get(end_at.saturating_sub(1))
                .map_or(i64::MAX, |e| e.entry_id.get())
        };
        let chunk = entries.get(at..end_at).unwrap_or_default();
        postings_chunk(conn, found, &context, chunk, (start, high))?;
        if last {
            return Ok(());
        }
        start = high.saturating_add(1);
        at = end_at;
    }
}

/// What every chunk of the postings comparison shares.
struct PostingContext<'a> {
    system: SystemAccounts,
    mirror: Vec<AccountId>,
    live: &'a HashSet<EntryId>,
}

/// The postings of one range of entry ids.
fn postings_chunk(
    conn: &Connection,
    found: &mut Found,
    context: &PostingContext<'_>,
    entries: &[PostingEntry],
    range: (i64, i64),
) -> Result<(), ServiceError> {
    let live = context.live;
    let mut lines = Vec::new();
    {
        let mut statement = conn.prepare(
            "SELECT l.entry_id, l.account_id, SUM(l.dr_paisa), SUM(l.cr_paisa)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE e.status = 'posted' AND e.posting_type IN ('bikri_first', 'bikri_later')
             AND l.entry_id BETWEEN ?1 AND ?2
             GROUP BY l.entry_id, l.account_id",
        )?;
        let rows = statement.query_map([range.0, range.1], |r| {
            Ok((
                entry_id(r, 0)?,
                account_id(r, 1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (entry, account, debit, credit) = row?;
            let record = format!("journal entry {}", entry.get());
            lines.push(LineSums {
                entry_id: entry,
                account_id: account,
                debit: money(found, "journal_lines", &record, debit),
                credit: money(found, "journal_lines", &record, credit),
            });
        }
    }

    let mut buyers = Vec::new();
    let mut sales = Vec::new();
    {
        let mut statement = conn.prepare(
            "SELECT journal_entry_id, buyer_account_id, SUM(buyer_total_paisa), SUM(paid_paisa),
                    SUM(seller_comm_paisa), SUM(buyer_comm_paisa), SUM(laga_paisa),
                    SUM(rate_diff_paisa)
             FROM bikri_sales WHERE journal_entry_id BETWEEN ?1 AND ?2
             GROUP BY journal_entry_id, buyer_account_id",
        )?;
        let rows = statement.query_map([range.0, range.1], |r| {
            Ok((
                entry_id(r, 0)?,
                account_id(r, 1)?,
                [
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                ],
            ))
        })?;
        for row in rows {
            let (entry, account, [total, paid, seller, buyer, laga, diff]) = row?;
            let record = format!("journal entry {}", entry.get());
            let mut m = |value: i64| money(found, "bikri_sales", &record, value);
            buyers.push(BuyerSums {
                entry_id: entry,
                account_id: account,
                buyer_total: m(total),
                paid: m(paid),
            });
            sales.push(SaleSums {
                entry_id: entry,
                seller_comm: m(seller),
                buyer_comm: m(buyer),
                laga: m(laga),
                rate_diff: m(diff),
            });
        }
    }

    // The lots of the live postings only: the lot row is edited in place by later edits.
    let mut lots = Vec::new();
    {
        let mut statement = conn.prepare(
            "SELECT s.journal_entry_id, s.lot_id, l.seller_account_id, l.purchase_id IS NOT NULL,
                    l.first_entry_id IS s.journal_entry_id,
                    SUM(s.seller_amount_paisa), SUM(s.seller_comm_paisa),
                    l.freight_paisa, l.freight_paid_by = 'firm', l.labour_paisa,
                    l.munshiana_paisa, l.store_rent_paisa, l.phone_paisa, l.cash_paid_paisa
             FROM bikri_sales s JOIN bikri_lots l ON l.id = s.lot_id
             WHERE s.journal_entry_id BETWEEN ?1 AND ?2
             GROUP BY s.journal_entry_id, s.lot_id",
        )?;
        let rows = statement.query_map([range.0, range.1], |r| {
            Ok((
                entry_id(r, 0)?,
                r.get::<_, i64>(1)?,
                account_id(r, 2)?,
                r.get::<_, bool>(3)?,
                r.get::<_, bool>(4)?,
                [
                    r.get::<_, i64>(5)?,
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                ],
                r.get::<_, bool>(8)?,
                [
                    r.get::<_, i64>(9)?,
                    r.get::<_, i64>(10)?,
                    r.get::<_, i64>(11)?,
                    r.get::<_, i64>(12)?,
                    r.get::<_, i64>(13)?,
                ],
            ))
        })?;
        for row in rows {
            let (entry, lot_id, seller, trip, first, [amount, comm, freight], by_firm, charges) =
                row?;
            if !live.contains(&entry) {
                continue;
            }
            let record = format!("lot {lot_id}");
            let mut m = |value: i64| money(found, "bikri_lots", &record, value);
            lots.push(LotPosting {
                entry_id: entry,
                lot_id,
                seller_account_id: seller,
                trip,
                first,
                seller_amount: m(amount),
                seller_comm: m(comm),
                freight: m(freight),
                freight_by_firm: by_firm,
                labour: m(charges[0]),
                munshiana: m(charges[1]),
                store_rent: m(charges[2]),
                phone: m(charges[3]),
                cash_paid: m(charges[4]),
            });
        }
    }

    // A lot of a first posting that has no line in it (not sold yet) still posts its charges.
    {
        let mut statement = conn.prepare(
            "SELECT l.first_entry_id, l.id, l.seller_account_id, l.purchase_id IS NOT NULL,
                    l.freight_paisa, l.freight_paid_by = 'firm', l.labour_paisa,
                    l.munshiana_paisa, l.store_rent_paisa, l.phone_paisa, l.cash_paid_paisa
             FROM bikri_lots l
             WHERE l.first_entry_id BETWEEN ?1 AND ?2 AND l.removed_at IS NULL
               AND NOT EXISTS (SELECT 1 FROM bikri_sales s
                               WHERE s.lot_id = l.id AND s.journal_entry_id = l.first_entry_id)",
        )?;
        let rows = statement.query_map([range.0, range.1], |r| {
            Ok((
                entry_id(r, 0)?,
                r.get::<_, i64>(1)?,
                account_id(r, 2)?,
                r.get::<_, bool>(3)?,
                [r.get::<_, i64>(4)?],
                r.get::<_, bool>(5)?,
                [
                    r.get::<_, i64>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, i64>(8)?,
                    r.get::<_, i64>(9)?,
                    r.get::<_, i64>(10)?,
                ],
            ))
        })?;
        for row in rows {
            let (entry, lot_id, seller, trip, [freight], by_firm, charges) = row?;
            if !live.contains(&entry) {
                continue;
            }
            let record = format!("lot {lot_id}");
            let mut m = |value: i64| money(found, "bikri_lots", &record, value);
            lots.push(LotPosting {
                entry_id: entry,
                lot_id,
                seller_account_id: seller,
                trip,
                first: true,
                seller_amount: Paisa::ZERO,
                seller_comm: Paisa::ZERO,
                freight: m(freight),
                freight_by_firm: by_firm,
                labour: m(charges[0]),
                munshiana: m(charges[1]),
                store_rent: m(charges[2]),
                phone: m(charges[3]),
                cash_paid: m(charges[4]),
            });
        }
    }

    let mut parts = Vec::new();
    {
        let mut statement = conn.prepare(
            "SELECT c.journal_entry_id, c.lot_id, p.venture_account_id, c.amount_paisa
             FROM lot_credit_parts c LEFT JOIN partners p ON p.id = c.partner_id
             WHERE c.journal_entry_id BETWEEN ?1 AND ?2",
        )?;
        let rows = statement.query_map([range.0, range.1], |r| {
            Ok((
                entry_id(r, 0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (entry, lot_id, account, amount) = row?;
            if !live.contains(&entry) {
                continue;
            }
            let record = format!("lot {lot_id}");
            let Some(account) = account else {
                found.add(Difference::new(
                    "lot_credit_parts",
                    format!("{record}: a part belongs to a partner with no trip account"),
                ));
                continue;
            };
            parts.push(TripPart {
                entry_id: entry,
                lot_id,
                account_id: AccountId::new(account),
                amount: money(found, "lot_credit_parts", &record, amount),
            });
        }
    }

    let differences = compare_postings(&PostingInputs {
        entries: &entries,
        buyers: &buyers,
        sales: &sales,
        lots: &lots,
        parts: &parts,
        system: context.system,
        lines: &lines,
        mirror: &context.mirror,
    })
    .map_err(data_error)?;
    found.add_all(differences);
    Ok(())
}

/// The sale lines and journal lines of the چکتا کھاتہ, person by person: a line moved from one
/// person to another keeps the account total, so [`postings`] does not see it (PRD FR-13.5).
fn chukta_persons(conn: &Connection, found: &mut Found) -> Result<(), ServiceError> {
    let account: Option<i64> = conn
        .query_row("SELECT id FROM accounts WHERE code = '0030'", [], |r| {
            r.get(0)
        })
        .map(Some)
        .or_else(|e| match e {
            mandi_db::rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })?;
    let Some(account) = account else {
        return Ok(());
    };
    let mut sales = Vec::new();
    {
        let mut statement = conn.prepare(
            "SELECT journal_entry_id, COALESCE(chukta_person_key, ''), SUM(buyer_total_paisa),
                    SUM(paid_paisa)
             FROM bikri_sales WHERE buyer_account_id = ?1
             GROUP BY journal_entry_id, COALESCE(chukta_person_key, '')",
        )?;
        let rows = statement.query_map([account], |r| {
            Ok((
                entry_id(r, 0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (entry, person, total, paid) = row?;
            let record = format!("journal entry {}", entry.get());
            sales.push(PersonSale {
                entry_id: entry,
                person,
                buyer_total: money(found, "bikri_sales", &record, total),
                paid: money(found, "bikri_sales", &record, paid),
            });
        }
    }
    let mut lines = Vec::new();
    {
        let mut statement = conn.prepare(
            "SELECT l.entry_id, COALESCE(l.chukta_person_key, ''), SUM(l.dr_paisa), SUM(l.cr_paisa)
             FROM journal_lines l JOIN journal_entries e ON e.id = l.entry_id
             WHERE l.account_id = ?1 AND e.status = 'posted'
               AND e.posting_type IN ('bikri_first', 'bikri_later')
             GROUP BY l.entry_id, COALESCE(l.chukta_person_key, '')",
        )?;
        let rows = statement.query_map([account], |r| {
            Ok((
                entry_id(r, 0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (entry, person, debit, credit) = row?;
            let record = format!("journal entry {}", entry.get());
            lines.push(PersonLines {
                entry_id: entry,
                person,
                debit: money(found, "journal_lines", &record, debit),
                credit: money(found, "journal_lines", &record, credit),
            });
        }
    }
    found.add_all(compare_persons(&sales, &lines).map_err(data_error)?);
    Ok(())
}

/// A sale line sold by weight: its amounts are the weight times the rate (PRD C1). The table checks
/// `qty × rate` only for a line sold by نگ.
fn weight_lines(conn: &Connection, found: &mut Found) -> Result<(), ServiceError> {
    let mut statement = conn.prepare(
        "SELECT id, weight_grams, seller_rate_paisa, seller_amount_paisa, buyer_rate_paisa,
                buyer_value_paisa
         FROM bikri_sales WHERE weight_grams IS NOT NULL",
    )?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            [
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, i64>(5)?,
            ],
        ))
    })?;
    for row in rows {
        let (id, grams, [seller_rate, seller_amount, buyer_rate, buyer_value]) = row?;
        let record = format!("sale line {id}");
        let mut m = |value: i64| money(found, "bikri_sales", &record, value);
        let (a, b, c, d) = (
            m(seller_rate),
            m(seller_amount),
            m(buyer_rate),
            m(buyer_value),
        );
        let what = weight_line_difference(grams, a, b, c, d);
        match what {
            Ok(None) => {}
            Ok(Some(what)) => found.add(Difference::new(
                "bikri_sales",
                format!("{record}: the {what} is not its weight times its rate"),
            )),
            Err(_) => found.add(Difference::new(
                "bikri_sales",
                format!("{record}: its weight and rate do not make an amount"),
            )),
        }
    }
    Ok(())
}

/// Comparison 2: the stored totals of each lot against its live sale lines and charges.
fn lots(conn: &Connection, found: &mut Found) -> Result<(), ServiceError> {
    let mut statement = conn.prepare(
        "SELECT l.id, l.qty_sold, l.gross_paisa, l.seller_comm_paisa, l.deductions_paisa,
                l.net_paisa, l.freight_paisa, l.freight_paid_by = 'firm', l.labour_paisa,
                l.munshiana_paisa, l.cash_paid_paisa, l.store_rent_paisa, l.phone_paisa,
                COALESCE(a.qty, 0), COALESCE(a.gross, 0), COALESCE(a.comm, 0),
                e.status = 'cancelled', l.first_entry_id IS NULL
         FROM bikri_lots l
         LEFT JOIN bikri_entries e ON e.id = l.bikri_id
         LEFT JOIN (SELECT lot_id, SUM(qty) AS qty, SUM(seller_amount_paisa) AS gross,
                           SUM(seller_comm_paisa) AS comm
                    FROM bikri_sales WHERE is_active = 1 GROUP BY lot_id) a ON a.lot_id = l.id
         WHERE l.removed_at IS NULL",
    )?;
    let rows = statement.query_map([], |r| {
        let mut values = [0_i64; 13];
        for (at, slot) in values.iter_mut().enumerate() {
            // Column 7 is the freight flag (0 or 1).
            *slot = r.get::<_, i64>(at)?;
        }
        let sums = [
            r.get::<_, i64>(13)?,
            r.get::<_, i64>(14)?,
            r.get::<_, i64>(15)?,
        ];
        Ok((
            values,
            sums,
            r.get::<_, Option<bool>>(16)?.unwrap_or(false),
            r.get::<_, bool>(17)?,
        ))
    })?;
    for row in rows {
        let (v, sums, cancelled, no_first) = row?;
        let by_firm = v[7] != 0;
        let lot_id = v[0];
        let record = format!("lot {lot_id}");
        if no_first && !cancelled {
            // A saved lot always names the posting that carries its charges.
            found.add(Difference::new(
                "bikri_lots",
                format!("lot {lot_id}: it has no first posting"),
            ));
        }
        let mut m = |value: i64| money(found, "bikri_lots", &record, value);
        let lot = LotRow {
            cancelled,
            lines_qty: sums[0],
            lines_gross: m(sums[1]),
            lines_comm: m(sums[2]),
            qty_sold: v[1],
            gross: m(v[2]),
            seller_comm: m(v[3]),
            deductions: m(v[4]),
            net: m(v[5]),
            freight: m(v[6]),
            freight_by_firm: by_firm,
            labour: m(v[8]),
            munshiana: m(v[9]),
            cash_paid: m(v[10]),
            store_rent: m(v[11]),
            phone: m(v[12]),
        };
        if let Some(what) = lot_difference(&lot).map_err(data_error)? {
            found.add(Difference::new(
                "bikri_lots",
                format!("lot {lot_id}: the stored {what} differs from its sale lines"),
            ));
        }
    }
    Ok(())
}

/// One posted journal entry, as the share comparison needs it.
struct EntryRow {
    date: String,
    posting_type: String,
    reverses: Option<i64>,
}

/// How many entries one chunk of the share comparison holds (a range of entry ids).
const SHARE_ENTRIES_PER_CHUNK: usize = 50_000;

/// A stored share row with the entry it belongs to and its date.
type StoredRow = (i64, String, StoredShare);

/// Reads the share rows a statement returns, in the order of `partner_share_rows`. A row that cannot
/// be read (an unknown kind, a share outside 0 to 100%, an amount that is not a whole rupee) is a
/// difference.
fn read_share_rows(
    conn: &Connection,
    sql: &str,
    params: &[i64],
    found: &mut Found,
) -> Result<Vec<StoredRow>, ServiceError> {
    const TABLE: &str = "partner_share_rows";
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map(mandi_db::rusqlite::params_from_iter(params.iter()), |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, i64>(5)?,
            r.get::<_, i64>(6)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (entry, date, partner, kind, base, bps, amount) = row?;
        let record = format!("journal entry {entry}");
        let Some(kind) = ShareKind::parse(&kind) else {
            found.add(Difference::new(
                TABLE,
                format!("{record}: a row has an unknown kind"),
            ));
            continue;
        };
        let Some(share_bps) = i32::try_from(bps).ok().and_then(|b| Bps::new(b).ok()) else {
            found.add(Difference::new(
                TABLE,
                format!("{record}: a row has a share outside 0 to 100%"),
            ));
            continue;
        };
        out.push((
            entry,
            date,
            StoredShare {
                partner_id: PartnerId::new(partner),
                kind,
                base: money(found, TABLE, &record, base),
                share_bps,
                amount: money(found, TABLE, &record, amount),
            },
        ));
    }
    Ok(out)
}

const SHARE_ROWS_SQL: &str = "SELECT journal_entry_id, entry_date, partner_id, kind, base_paisa,
        share_bps, amount_paisa FROM partner_share_rows";

/// Comparison 3: the partner share rows of each entry against the split of its lines. The entries
/// are taken a range of ids at a time (NFR-SPD-16); the rows of an entry that has a reversal are
/// read once, because the reversal must be their mirror.
fn share_rows(conn: &Connection, found: &mut Found) -> Result<(), ServiceError> {
    let history: Vec<ShareHistory> = {
        let mut statement = conn.prepare(
            "SELECT id, partner_id, share_bps, effective_from, created_at FROM partner_shares",
        )?;
        let rows = statement.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, partner, bps, from, at) = row?;
            let Some(share_bps) = i32::try_from(bps).ok().and_then(|b| Bps::new(b).ok()) else {
                found.add(Difference::new(
                    "partner_shares",
                    format!("row {id}: a share outside 0 to 100%"),
                ));
                continue;
            };
            out.push(ShareHistory {
                id,
                partner_id: PartnerId::new(partner),
                share_bps,
                effective_from: from,
                created_at: at,
            });
        }
        out
    };
    let mut originals: HashMap<i64, Vec<StoredShare>> = HashMap::new();
    for (entry, _, row) in read_share_rows(
        conn,
        &format!(
            "{SHARE_ROWS_SQL} WHERE journal_entry_id IN (SELECT reverses_entry_id FROM journal_entries
                 WHERE reverses_entry_id IS NOT NULL AND status = 'posted')"
        ),
        &[],
        found,
    )? {
        originals.entry(entry).or_default().push(row);
    }
    let ids = ids_of(
        conn,
        "SELECT id FROM journal_entries WHERE status = 'posted' ORDER BY id",
    )?;
    let mut states = ShareStates::default();
    let mut start = 0_i64;
    let mut at = 0_usize;
    loop {
        let end_at = at.saturating_add(SHARE_ENTRIES_PER_CHUNK).min(ids.len());
        let last = end_at >= ids.len();
        let high = if last {
            i64::MAX
        } else {
            ids.get(end_at.saturating_sub(1))
                .copied()
                .unwrap_or(i64::MAX)
        };
        share_chunk(
            conn,
            found,
            &ShareContext {
                history: &history,
                originals: &originals,
            },
            &mut states,
            (start, high),
        )?;
        if last {
            return Ok(());
        }
        start = high.saturating_add(1);
        at = end_at;
    }
}

/// What every chunk of the share comparison shares.
struct ShareContext<'a> {
    history: &'a [ShareHistory],
    originals: &'a HashMap<i64, Vec<StoredShare>>,
}

/// The entries of one range of ids against their share rows.
fn share_chunk(
    conn: &Connection,
    found: &mut Found,
    context: &ShareContext<'_>,
    states: &mut ShareStates,
    range: (i64, i64),
) -> Result<(), ServiceError> {
    const TABLE: &str = "partner_share_rows";
    let mut entries: HashMap<i64, EntryRow> = HashMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT id, entry_date, posting_type, reverses_entry_id FROM journal_entries
             WHERE status = 'posted' AND id BETWEEN ?1 AND ?2",
        )?;
        let rows = statement.query_map([range.0, range.1], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                EntryRow {
                    date: r.get(1)?,
                    posting_type: r.get(2)?,
                    reverses: r.get(3)?,
                },
            ))
        })?;
        for row in rows {
            let (id, entry) = row?;
            entries.insert(id, entry);
        }
    }

    // Income and shop expense lines: the (debit, credit) sums of head 04 and of head 05.
    let mut sums: HashMap<i64, [(Paisa, Paisa); 2]> = HashMap::new();
    {
        let mut statement = conn.prepare(
            "SELECT l.entry_id, h.code, SUM(l.dr_paisa), SUM(l.cr_paisa)
             FROM journal_lines l
             JOIN accounts a ON a.id = l.account_id
             JOIN account_controls c ON c.id = a.control_id
             JOIN account_heads h ON h.id = c.head_id
             WHERE h.code IN ('04', '05') AND l.entry_id BETWEEN ?1 AND ?2
             GROUP BY l.entry_id, h.code",
        )?;
        let rows = statement.query_map([range.0, range.1], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (entry, head, debit, credit) = row?;
            let record = format!("journal entry {entry}");
            let pair = (
                money(found, "journal_lines", &record, debit),
                money(found, "journal_lines", &record, credit),
            );
            let slot = sums.entry(entry).or_insert([(Paisa::ZERO, Paisa::ZERO); 2]);
            if head == "04" {
                slot[0] = pair;
            } else {
                slot[1] = pair;
            }
        }
    }

    // The stored rows, by entry.
    let mut stored: HashMap<i64, Vec<StoredShare>> = HashMap::new();
    let mut row_dates: HashMap<i64, HashSet<String>> = HashMap::new();
    for (entry, date, row) in read_share_rows(
        conn,
        &format!("{SHARE_ROWS_SQL} WHERE journal_entry_id BETWEEN ?1 AND ?2"),
        &[range.0, range.1],
        found,
    )? {
        stored.entry(entry).or_default().push(row);
        row_dates.entry(entry).or_default().insert(date);
    }

    let none: Vec<StoredShare> = Vec::new();
    let mut ids: Vec<i64> = entries.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        let Some(entry) = entries.get(&id) else {
            continue;
        };
        let have = stored.get(&id).unwrap_or(&none);
        let ok = if entry.posting_type == "month_close" {
            // A month close only moves amounts between accounts (PRD FR-17.6).
            have.is_empty()
        } else if let Some(original) = entry.reverses {
            let of_original = context.originals.get(&original).unwrap_or(&none);
            reversal_rows_match(of_original, have).map_err(data_error)?
        } else {
            let pairs = sums
                .get(&id)
                .copied()
                .unwrap_or([(Paisa::ZERO, Paisa::ZERO); 2]);
            let bases = bases_of(pairs[0], pairs[1]).map_err(data_error)?;
            share_rows_match(states.of(context.history, &entry.date), bases, have)
        };
        let date_ok = row_dates
            .get(&id)
            .is_none_or(|dates| dates.iter().all(|d| *d == entry.date));
        if !ok || !date_ok {
            found.add(Difference::new(
                TABLE,
                format!(
                    "journal entry {id}: the share rows differ from its income and expense lines"
                ),
            ));
        }
    }
    // A share row of an entry that is not a posted entry.
    let mut orphans: Vec<i64> = stored
        .keys()
        .copied()
        .filter(|id| !entries.contains_key(id))
        .collect();
    orphans.sort_unstable();
    for id in orphans {
        found.add(Difference::new(
            TABLE,
            format!("journal entry {id}: share rows of an entry that is not posted"),
        ));
    }
    Ok(())
}

/// Reads the ids a statement returns.
fn ids_of(conn: &Connection, sql: &str) -> Result<Vec<i64>, ServiceError> {
    let mut statement = conn.prepare(sql)?;
    let rows = statement.query_map([], |r| r.get::<_, i64>(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The quantities the stock moves of an owner (a lot, a purchase, a garden arrival) add up to, by
/// what each move is: a reversal counts as the kind of the move it reverses. The goods an owner
/// received are counted from the moves that name no lot, because a trip or garden lot has its own
/// "received" move, which is not the purchase's.
fn buckets(owner: &str, receipts_of_owner_only: bool) -> String {
    let received = if receipts_of_owner_only {
        "m.lot_id IS NULL"
    } else {
        "1 = 1"
    };
    format!(
        "SELECT m.{owner} AS owner,
                SUM(CASE WHEN COALESCE(o.kind, m.kind) = 'received' AND {received} THEN m.qty ELSE 0 END) AS rec,
                SUM(CASE WHEN COALESCE(o.kind, m.kind) = 'sold' THEN m.qty ELSE 0 END) AS sold,
                SUM(CASE WHEN COALESCE(o.kind, m.kind) = 'written_off' THEN m.qty ELSE 0 END) AS wo,
                SUM(CASE WHEN COALESCE(o.kind, m.kind) = 'taken' THEN m.qty ELSE 0 END) AS taken
         FROM stock_moves m LEFT JOIN stock_moves o ON o.id = m.reverses_move_id
         WHERE m.{owner} IS NOT NULL GROUP BY m.{owner}"
    )
}

/// Comparison 4: the stock moves of each lot, purchase and garden arrival against its quantities.
/// A cancel reverses the moves and ends the lines but keeps the columns of what it cancelled
/// (plan 16), so a cancelled owner must show nothing left in its moves, and no quantity sold, written
/// off or taken.
fn stock(conn: &Connection, found: &mut Found) -> Result<(), ServiceError> {
    let lots = ids_of(
        conn,
        &format!(
            "SELECT l.id FROM bikri_lots l
             LEFT JOIN bikri_entries e ON e.id = l.bikri_id
             LEFT JOIN ({}) b ON b.owner = l.id
             WHERE l.removed_at IS NULL AND CASE WHEN e.status = 'cancelled'
                 THEN COALESCE(b.rec, 0) <> 0 OR COALESCE(b.sold, 0) <> 0
                   OR COALESCE(b.wo, 0) <> 0 OR COALESCE(b.taken, 0) <> 0 OR l.qty_taken <> 0
                 ELSE COALESCE(b.rec, 0) <> l.qty_received
                   OR -COALESCE(b.sold, 0) <> l.qty_sold
                   OR -COALESCE(b.wo, 0) <> l.qty_written_off
                   OR -COALESCE(b.taken, 0) <> l.qty_taken END",
            buckets("lot_id", false)
        ),
    )?;
    for id in lots {
        found.add(Difference::new(
            "stock_moves",
            format!("lot {id}: the stock moves differ from its quantities"),
        ));
    }
    for (table, owner, label) in [
        ("purchases", "purchase_id", "purchase"),
        ("garden_arrivals", "arrival_id", "garden arrival"),
    ] {
        let ids = ids_of(
            conn,
            &format!(
                "SELECT p.id FROM {table} p LEFT JOIN ({}) b ON b.owner = p.id
                 WHERE CASE WHEN p.status = 'cancelled'
                     THEN COALESCE(b.rec, 0) <> 0 OR COALESCE(b.sold, 0) <> 0
                       OR COALESCE(b.wo, 0) <> 0 OR p.qty_sold <> 0 OR p.qty_written_off <> 0
                     ELSE COALESCE(b.rec, 0) <> p.qty
                       OR -COALESCE(b.sold, 0) <> p.qty_sold
                       OR -COALESCE(b.wo, 0) <> p.qty_written_off END",
                buckets(owner, true)
            ),
        )?;
        for id in ids {
            found.add(Difference::new(
                "stock_moves",
                format!("{label} {id}: the stock moves differ from its quantities"),
            ));
        }
    }
    // A move of a lot, purchase or arrival that does not exist.
    let orphans = ids_of(
        conn,
        "SELECT m.id FROM stock_moves m
         WHERE (m.lot_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM bikri_lots l WHERE l.id = m.lot_id))
            OR (m.purchase_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM purchases p WHERE p.id = m.purchase_id))
            OR (m.arrival_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM garden_arrivals a WHERE a.id = m.arrival_id))",
    )?;
    for id in orphans {
        found.add(Difference::new(
            "stock_moves",
            format!("move {id}: it belongs to a record that does not exist"),
        ));
    }
    Ok(())
}

/// Comparison 5: the analytics facts against the sale lines. A sale line's facts add up (sign
/// times value) to the line's value while it is active and to 0 once it ended: an edit or a cancel
/// copies each fact with the opposite sign (PRD FR-18.7). The first fact of every line, the one
/// with sign +1, always holds the line's own values, active or not. The views group by item,
/// seller, trip and garden, so those must be the lot's too, while the line is active.
fn facts(conn: &Connection, found: &mut Found) -> Result<(), ServiceError> {
    const TABLE: &str = "analytics_facts";
    let sales = ids_of(
        conn,
        "SELECT s.id FROM bikri_sales s
         LEFT JOIN (SELECT sale_id,
                           SUM(sign * qty) AS qty,
                           SUM(sign * seller_amount_paisa) AS seller_amount,
                           SUM(sign * seller_comm_paisa) AS seller_comm,
                           SUM(sign * buyer_value_paisa) AS buyer_value,
                           SUM(sign * buyer_comm_paisa) AS buyer_comm,
                           SUM(sign * laga_paisa) AS laga,
                           SUM(sign * cost_paisa) AS cost,
                           SUM(CASE WHEN sign = 1 THEN qty ELSE 0 END) AS first_qty,
                           SUM(CASE WHEN sign = 1 THEN seller_amount_paisa ELSE 0 END) AS first_seller_amount,
                           SUM(CASE WHEN sign = 1 THEN seller_comm_paisa ELSE 0 END) AS first_seller_comm,
                           SUM(CASE WHEN sign = 1 THEN buyer_value_paisa ELSE 0 END) AS first_buyer_value,
                           SUM(CASE WHEN sign = 1 THEN buyer_comm_paisa ELSE 0 END) AS first_buyer_comm,
                           SUM(CASE WHEN sign = 1 THEN laga_paisa ELSE 0 END) AS first_laga,
                           SUM(CASE WHEN sign = 1 THEN cost_paisa ELSE 0 END) AS first_cost
                    FROM analytics_facts GROUP BY sale_id) f ON f.sale_id = s.id
         WHERE COALESCE(f.qty, 0) <> CASE WHEN s.is_active = 1 THEN s.qty ELSE 0 END
            OR COALESCE(f.seller_amount, 0) <> CASE WHEN s.is_active = 1 THEN s.seller_amount_paisa ELSE 0 END
            OR COALESCE(f.seller_comm, 0) <> CASE WHEN s.is_active = 1 THEN s.seller_comm_paisa ELSE 0 END
            OR COALESCE(f.buyer_value, 0) <> CASE WHEN s.is_active = 1 THEN s.buyer_value_paisa ELSE 0 END
            OR COALESCE(f.buyer_comm, 0) <> CASE WHEN s.is_active = 1 THEN s.buyer_comm_paisa ELSE 0 END
            OR COALESCE(f.laga, 0) <> CASE WHEN s.is_active = 1 THEN s.laga_paisa ELSE 0 END
            OR COALESCE(f.cost, 0) <> CASE WHEN s.is_active = 1 THEN s.cost_paisa ELSE 0 END
            OR COALESCE(f.first_qty, 0) <> s.qty
            OR COALESCE(f.first_seller_amount, 0) <> s.seller_amount_paisa
            OR COALESCE(f.first_seller_comm, 0) <> s.seller_comm_paisa
            OR COALESCE(f.first_buyer_value, 0) <> s.buyer_value_paisa
            OR COALESCE(f.first_buyer_comm, 0) <> s.buyer_comm_paisa
            OR COALESCE(f.first_laga, 0) <> s.laga_paisa
            OR COALESCE(f.first_cost, 0) <> s.cost_paisa",
    )?;
    for id in sales {
        found.add(Difference::new(
            TABLE,
            format!("sale line {id}: its facts differ from the line"),
        ));
    }
    // A fact that names another بکری, lot, buyer or day than its sale line, or no sale line.
    let strays = ids_of(
        conn,
        "SELECT f.id FROM analytics_facts f
         WHERE NOT EXISTS (SELECT 1 FROM bikri_sales s WHERE s.id = f.sale_id
                             AND s.bikri_id = f.bikri_id AND s.lot_id = f.lot_id
                             AND s.buyer_account_id = f.buyer_account_id
                             AND s.chukta_person_key IS f.chukta_person_key
                             AND s.entry_date = f.entry_date)",
    )?;
    for id in strays {
        found.add(Difference::new(
            TABLE,
            format!("fact {id}: it does not match a sale line"),
        ));
    }
    // The item, seller, trip purchase and trip or garden of an active line's fact are its lot's.
    let moved = ids_of(
        conn,
        "SELECT f.id FROM analytics_facts f
         JOIN bikri_sales s ON s.id = f.sale_id AND s.is_active = 1
         JOIN bikri_lots l ON l.id = s.lot_id
         WHERE f.item_id <> l.item_id
            OR f.seller_account_id <> l.seller_account_id
            OR f.purchase_id IS NOT l.purchase_id
            OR f.venture_id IS NOT (CASE WHEN l.purchase_id IS NOT NULL
                   THEN (SELECT p.venture_id FROM purchases p WHERE p.id = l.purchase_id)
                   ELSE (SELECT g.id FROM ventures g WHERE g.account_id = l.seller_account_id) END)",
    )?;
    for id in moved {
        found.add(Difference::new(
            TABLE,
            format!("fact {id}: its item, seller, trip or garden is not its lot's"),
        ));
    }
    // A reversal copies the fact it takes back, so every fact of a line has the same dimensions.
    let mixed = ids_of(
        conn,
        "SELECT sale_id FROM analytics_facts GROUP BY sale_id
         HAVING COUNT(DISTINCT item_id) > 1
             OR COUNT(DISTINCT seller_account_id) > 1
             OR COUNT(DISTINCT IFNULL(purchase_id, 0)) > 1
             OR COUNT(DISTINCT IFNULL(venture_id, 0)) > 1
             OR COUNT(DISTINCT year * 100 + month) > 1",
    )?;
    for id in mixed {
        found.add(Difference::new(
            TABLE,
            format!("sale line {id}: its facts do not agree on item, seller, trip or month"),
        ));
    }
    Ok(())
}
