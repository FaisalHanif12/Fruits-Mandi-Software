//! The full NFR-SCL-1 dataset of lane T2: the plain volume of `benches/support` (20,000 accounts,
//! 3.5 million journal lines, 10 years) plus the 1.2 million sale lines it lacks, with everything
//! a real file holds for them. Built once, kept in `<MANDI_PERF_DIR>/../nfr_scl1_gate`.
//!
//! What the layer adds (all through the real tables, indexes and triggers: every journal entry is
//! a draft first and one `UPDATE` posts it, so the balance trigger checks each one):
//!
//! - 41,000 بکری: 40,500 spread over the 3,650 days (about 11 a day, about 30 lines each, 3 lots)
//!   and 500 on the last day (NFR-SCL-2: 500 بکری and 5,000 lines in one day), 1,200,000 sale
//!   lines in all, about 122,500 lots, each بکری posted as one journal entry with the lines of
//!   PRD FR-12.9 (a buyer line for each sale line, a seller line for each lot, the income and
//!   payable lines, the cash lines for money paid on the spot);
//! - 1,200,000 `analytics_facts`, the partner share rows, the stock moves;
//! - 15% of the lines paid in full on the spot, 2% part paid, 1.5% چکتا buyers;
//! - 3,000 seller advances and 3,000 money lent entries with their returns, 110,000 vouchers,
//!   100 people in the dead amount;
//! - the matching of every buyer and seller (`matching::rematch`), as a real file keeps it.
//!
//! So the journal grows to about 5 million lines. That is more than the 3.5 million of NFR-SCL-1,
//! on purpose: every number measured on it is a little worse than on the exact volume.

use std::path::{Path, PathBuf};
use std::time::Instant;

use mandi_services::core::text::{search_en, search_phonetic, search_ur};
use mandi_services::db::DbError;
use mandi_services::db::rusqlite::{Connection, params};
use mandi_services::{ServiceError, matching};

use crate::bikri_draft::add_partners;
use crate::support::{self, Rng, fail, note};

/// A change here builds the layer again.
const GATE_VERSION: &str = "gate-t2-v3";
/// بکری spread over the 10 years.
pub const REGULAR_BIKRI: u64 = 40_500;
/// بکری on the busiest day (the last day).
pub const BUSY_BIKRI: u64 = 500;
/// Sale lines on the busiest day: 500 بکری of 10 lines, the first 250 of 11.
pub const BUSY_LINES: u64 = 5_250;
/// بکری written in one transaction.
const BATCH: u64 = 200;

/// The data the timing tests work on.
#[derive(Debug, Clone)]
pub struct Gate {
    /// The folder with `data/mandi.db`.
    pub dir: PathBuf,
    /// The busiest buyer of the plain volume (0020).
    pub busy_buyer: i64,
    /// The busiest seller (0010).
    pub busy_seller: i64,
    /// An ordinary buyer.
    pub median_buyer: i64,
}

fn db(e: mandi_services::db::rusqlite::Error) -> ServiceError {
    ServiceError::from(DbError::from(e))
}

fn gate_dir(base: &Path) -> PathBuf {
    let regular = regular_count();
    if regular == REGULAR_BIKRI {
        base.with_file_name("nfr_scl1_gate")
    } else {
        base.with_file_name("nfr_scl1_gate_dev")
    }
}

/// `GATE_T2_BIKRI` makes a small layer for trying the generator; the default is the real one.
fn regular_count() -> u64 {
    std::env::var("GATE_T2_BIKRI")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(REGULAR_BIKRI)
}

fn busy_count() -> u64 {
    if regular_count() == REGULAR_BIKRI {
        BUSY_BIKRI
    } else {
        (regular_count() / 80).max(2)
    }
}

/// Opens the full volume, building the plain volume and the layer first if they are missing.
pub fn gate_volume() -> Gate {
    if std::env::var("MANDI_PERF_DIR").is_err() {
        fail(
            "set MANDI_PERF_DIR to a folder under backend/target-gate-t2 (the data is about 3 GB)",
        );
    }
    let base = support::ensure_volume();
    let dir = gate_dir(&base.dir);
    let marker = dir.join("gate_t2.ok");
    let expected = format!("{GATE_VERSION} {} {}", regular_count(), busy_count());
    if std::fs::read_to_string(&marker).unwrap_or_default().trim() != expected {
        let started = Instant::now();
        note(&format!(
            "gate t2: building the sale layer in {}",
            dir.display()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let to = dir.join("data");
        std::fs::create_dir_all(&to).unwrap_or_else(|e| fail(&format!("no folder: {e}")));
        std::fs::copy(base.dir.join("data").join("mandi.db"), to.join("mandi.db"))
            .unwrap_or_else(|e| fail(&format!("cannot copy the plain volume: {e}")));
        // The plain volume's own marker, so `MANDI_PERF_DIR=<this folder>` makes every existing
        // criterion bench run on this data and never build a plain one again.
        let _ = std::fs::copy(base.dir.join("volume.ok"), dir.join("volume.ok"));
        build_layer(&dir);
        let _ = std::fs::write(&marker, &expected);
        note(&format!("gate t2: layer built in {:?}", started.elapsed()));
    }
    Gate {
        dir,
        busy_buyer: base.busy_buyer,
        busy_seller: base.busy_seller,
        median_buyer: base.median_buyer,
    }
}

struct Ids {
    cash: i64,
    comm_seller: i64,
    comm_buyer: i64,
    laga: i64,
    phone: i64,
    labour: i64,
    freight: i64,
    munshiana: i64,
    chukta: i64,
    sellers: Vec<i64>,
    buyers: Vec<i64>,
    others: Vec<i64>,
    items: Vec<i64>,
    packings: Vec<i64>,
    cities: Vec<i64>,
    partners: Vec<i64>,
}

fn ids_of(conn: &Connection, sql: &str) -> Result<Vec<i64>, ServiceError> {
    let mut statement = conn.prepare(sql).map_err(db)?;
    let rows = statement
        .query_map([], |r| r.get::<_, i64>(0))
        .map_err(db)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(db)
}

fn id_of(conn: &Connection, code: &str) -> i64 {
    conn.query_row("SELECT id FROM accounts WHERE code = ?1", [code], |r| {
        r.get(0)
    })
    .unwrap_or_else(|e| fail(&format!("no account {code}: {e}")))
}

fn lookups(conn: &Connection) -> Result<Ids, ServiceError> {
    Ok(Ids {
        cash: id_of(conn, "01010001"),
        comm_seller: id_of(conn, "04010001"),
        comm_buyer: id_of(conn, "04010002"),
        laga: id_of(conn, "04010003"),
        phone: id_of(conn, "04010004"),
        labour: id_of(conn, "02010001"),
        freight: id_of(conn, "02010002"),
        munshiana: id_of(conn, "02010004"),
        chukta: id_of(conn, "0030"),
        sellers: ids_of(
            conn,
            "SELECT id FROM accounts WHERE code GLOB '001[0-9]*' AND is_system = 0 ORDER BY serial",
        )?,
        buyers: ids_of(
            conn,
            "SELECT id FROM accounts WHERE code GLOB '002[0-9]*' AND is_system = 0 ORDER BY serial",
        )?,
        others: ids_of(
            conn,
            "SELECT id FROM accounts WHERE code GLOB '0103*' AND is_system = 0 ORDER BY serial",
        )?,
        items: ids_of(conn, "SELECT id FROM items ORDER BY id")?,
        packings: ids_of(conn, "SELECT id FROM packings ORDER BY id")?,
        cities: ids_of(conn, "SELECT id FROM cities ORDER BY id")?,
        partners: ids_of(conn, "SELECT id FROM partners ORDER BY id")?,
    })
}

/// A 64-character hash. The timing tests never verify the chain (the base volume does the same).
const FAKE_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

const CHUKTA_PEOPLE: usize = 50;

/// Whole rupees of `value` (paisa) times 6%, half up, back in paisa.
fn six_percent(value: i64) -> i64 {
    (value / 100 * 6 + 50) / 100 * 100
}

struct Line {
    buyer: i64,
    chukta: Option<usize>,
    lot: usize,
    qty: i64,
    rate: i64,
    value: i64,
    s_comm: i64,
    b_comm: i64,
    laga: i64,
    total: i64,
    paid: i64,
}

#[derive(Default, Clone)]
struct Lot {
    seller: i64,
    item: i64,
    packing: i64,
    qty: i64,
    gross: i64,
    s_comm: i64,
}

const LOT_FREIGHT: i64 = 300_000;
const LOT_LABOUR: i64 = 25_000;
const LOT_MUNSHIANA: i64 = 5_000;
const LOT_PHONE: i64 = 2_000;

fn build_layer(dir: &Path) {
    let started = Instant::now();
    let (env, actor) = support::open(dir);
    add_partners(&env, &actor);
    let ids = env
        .ctx
        .db()
        .read(lookups)
        .unwrap_or_else(|e| fail(&format!("cannot read the ids: {e}")));
    let regular = regular_count();
    let busy = busy_count();
    let days = (support::last_day() - support::first_day())
        .num_days()
        .max(1);
    let day_text: Vec<String> = (0..=days)
        .map(|d| {
            (support::first_day() + chrono::Duration::days(d))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect();
    let last_date = day_text.last().cloned().unwrap_or_default();

    env.ctx
        .db()
        .write(|tx| insert_chukta_people(tx))
        .unwrap_or_else(|e| fail(&format!("cannot add the chukta buyers: {e}")));

    let total = regular + busy;
    let mut rng = Rng(0x00D2_0000_6A7E_0001);
    let mut next = 0_u64;
    let mut sale_lines = 0_u64;
    while next < total {
        let upto = (next + BATCH).min(total);
        let lines_in_batch = env
            .ctx
            .db()
            .write(|tx| -> Result<u64, ServiceError> {
                let mut lines = 0_u64;
                for k in next..upto {
                    let (date, n_lines, n_lots) = if k < regular {
                        let day =
                            usize::try_from(k.saturating_mul(3_650) / regular.max(1)).unwrap_or(0);
                        (
                            day_text.get(day).map_or("2020-01-01", String::as_str),
                            29 + usize::try_from(k % 2).unwrap_or(0),
                            3_usize,
                        )
                    } else {
                        let j = k - regular;
                        (
                            last_date.as_str(),
                            10 + usize::from(j < (busy / 2).max(1)),
                            2_usize,
                        )
                    };
                    insert_bikri(tx, &ids, &mut rng, k + 1, date, n_lines, n_lots)?;
                    lines += u64::try_from(n_lines).unwrap_or(0);
                }
                Ok(lines)
            })
            .unwrap_or_else(|e| fail(&format!("cannot write the بکری {next}: {e}")));
        sale_lines += lines_in_batch;
        next = upto;
        if next % 4_000 < BATCH {
            note(&format!(
                "gate t2: {next} of {total} بکری, {sale_lines} sale lines after {:?}",
                started.elapsed()
            ));
        }
    }
    note(&format!(
        "gate t2: {total} بکری, {sale_lines} sale lines in {:?}",
        started.elapsed()
    ));

    env.ctx
        .db()
        .write(|tx| other_entries(tx, &ids, &day_text, &mut rng))
        .unwrap_or_else(|e| fail(&format!("cannot write the vouchers: {e}")));
    note(&format!(
        "gate t2: advances, loans, vouchers in {:?}",
        started.elapsed()
    ));

    // The matching of every buyer and seller, as a real file keeps it.
    let now = env.ctx.now_text();
    for chunk in ids.buyers.chunks(250) {
        env.ctx
            .db()
            .write(|tx| {
                for id in chunk {
                    matching::rematch(tx, actor.user_id, &now, *id, None)?;
                }
                Ok::<(), ServiceError>(())
            })
            .unwrap_or_else(|e| fail(&format!("the matching was not written: {e}")));
    }
    // The advances of the sellers are not matched: the plain volume's old seller credits have no
    // lots, and `rematch_advances` refuses such a credit ("1 credit lines of the seller for 0
    // lots"). So the advance pages see every advance as open, the larger case.
    note(&format!("gate t2: matching in {:?}", started.elapsed()));

    env.ctx
        .db()
        .write(|tx| finishing_touches(tx))
        .unwrap_or_else(|e| fail(&format!("cannot finish: {e}")));
    drop(env);
    if let Ok(conn) = Connection::open(dir.join("data").join("mandi.db")) {
        let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
        let _ = conn.execute_batch("ANALYZE");
        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap_or(-1) };
        note(&format!(
            "gate t2: {} بکری, {} lots, {} sale lines, {} facts, {} journal lines, {} bill allocations, {} vouchers; file {} MB",
            count("SELECT COUNT(*) FROM bikri_entries"),
            count("SELECT COUNT(*) FROM bikri_lots"),
            count("SELECT COUNT(*) FROM bikri_sales"),
            count("SELECT COUNT(*) FROM analytics_facts"),
            count("SELECT COUNT(*) FROM journal_lines"),
            count("SELECT COUNT(*) FROM bill_allocations"),
            count("SELECT COUNT(*) FROM vouchers"),
            std::fs::metadata(dir.join("data").join("mandi.db"))
                .map(|m| m.len() / 1_000_000)
                .unwrap_or(0),
        ));
    }
}

fn insert_chukta_people(tx: &Connection) -> Result<(), ServiceError> {
    let mut insert = tx
        .prepare_cached(
            "INSERT INTO chukta_buyers (person_key, name_ur, name_en, search_ur, search_en, search_phonetic, phone)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(db)?;
    for i in 0..CHUKTA_PEOPLE {
        let (ur, en) = support::account_names(u32::try_from(i * 7 + 3).unwrap_or(0));
        insert
            .execute(params![
                format!("ck-{i:03}"),
                ur,
                en,
                search_ur(&ur),
                search_en(&en),
                search_phonetic(&en),
                format!("0301{i:07}"),
            ])
            .map_err(db)?;
    }
    Ok(())
}

/// One بکری with its lots, lines, facts, stock moves, partner rows and one posted journal entry.
fn insert_bikri(
    tx: &Connection,
    ids: &Ids,
    rng: &mut Rng,
    number: u64,
    date: &str,
    n_lines: usize,
    n_lots: usize,
) -> Result<(), ServiceError> {
    let number_i = i64::try_from(number).unwrap_or(0);
    let pick = |rng: &mut Rng, list: &[i64]| -> i64 {
        list.get(rng.index(list.len())).copied().unwrap_or(0)
    };
    // The lots.
    let mut lots: Vec<Lot> = (0..n_lots)
        .map(|_| Lot {
            seller: if rng.below(100) < 3 {
                ids.sellers.first().copied().unwrap_or(0)
            } else {
                pick(rng, &ids.sellers)
            },
            item: pick(rng, &ids.items),
            packing: pick(rng, &ids.packings),
            ..Lot::default()
        })
        .collect();
    // The lines.
    let mut lines: Vec<Line> = Vec::with_capacity(n_lines);
    for j in 0..n_lines {
        let qty = 1 + i64::try_from(rng.below(8)).unwrap_or(0);
        let rate = (200 + i64::try_from(rng.below(700)).unwrap_or(0)) * 100;
        let value = qty * rate;
        let s_comm = six_percent(value);
        let b_comm = six_percent(value);
        let laga = qty * 800;
        let total = value + b_comm + laga;
        let kind = rng.below(1_000);
        let (chukta, paid) = if kind < 150 {
            (None, total)
        } else if kind < 170 {
            (None, total / 200 * 100)
        } else if kind < 185 {
            (Some(rng.index(CHUKTA_PEOPLE)), total)
        } else {
            (None, 0)
        };
        let buyer = match chukta {
            Some(_) => ids.chukta,
            None if rng.below(100) < 3 => ids.buyers.first().copied().unwrap_or(0),
            None => pick(rng, &ids.buyers),
        };
        let lot = j % n_lots;
        if let Some(l) = lots.get_mut(lot) {
            l.qty += qty;
            l.gross += value;
            l.s_comm += s_comm;
        }
        lines.push(Line {
            buyer,
            chukta,
            lot,
            qty,
            rate,
            value,
            s_comm,
            b_comm,
            laga,
            total,
            paid,
        });
    }

    tx.prepare_cached(
        "INSERT INTO bikri_entries (number, entry_date, bill_date, created_by, updated_by)
         VALUES (?1, ?2, ?2, 1, 1)",
    )
    .map_err(db)?
    .execute(params![number_i, date])
    .map_err(db)?;
    let bikri_id = tx.last_insert_rowid();
    tx.prepare_cached(
        "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
         VALUES (?1, 'bikri_first', 'bikri', ?2, 1, 1)",
    )
    .map_err(db)?
    .execute(params![date, bikri_id])
    .map_err(db)?;
    let entry_id = tx.last_insert_rowid();

    // The lots.
    let mut lot_ids: Vec<i64> = Vec::with_capacity(n_lots);
    for (index, lot) in lots.iter().enumerate() {
        let deductions = LOT_FREIGHT + lot.s_comm + LOT_LABOUR + LOT_MUNSHIANA + LOT_PHONE;
        tx.prepare_cached(
            "INSERT INTO bikri_lots (bikri_id, lot_no, seller_account_id, item_id, packing_id,
                 qty_received, freight_paisa, labour_paisa, munshiana_paisa, phone_paisa,
                 first_entry_id, qty_sold, gross_paisa, seller_comm_paisa, deductions_paisa,
                 net_paisa, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?6, ?12, ?13, ?14, ?15, 1, 1)",
        )
        .map_err(db)?
        .execute(params![
            bikri_id,
            i64::try_from(index + 1).unwrap_or(1),
            lot.seller,
            lot.item,
            lot.packing,
            lot.qty,
            LOT_FREIGHT,
            LOT_LABOUR,
            LOT_MUNSHIANA,
            LOT_PHONE,
            entry_id,
            lot.gross,
            lot.s_comm,
            deductions,
            lot.gross - deductions,
        ])
        .map_err(db)?;
        let lot_id = tx.last_insert_rowid();
        lot_ids.push(lot_id);
        tx.prepare_cached(
            "INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 1)",
        )
        .map_err(db)?
        .execute(params![lot_id, entry_id, date, "received", lot.qty])
        .map_err(db)?;
        tx.prepare_cached(
            "INSERT INTO stock_moves (lot_id, journal_entry_id, move_date, kind, qty, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 1)",
        )
        .map_err(db)?
        .execute(params![lot_id, entry_id, date, "sold", -lot.qty])
        .map_err(db)?;
    }

    // The lines and their facts.
    let year: i64 = date.get(0..4).and_then(|s| s.parse().ok()).unwrap_or(2020);
    let month: i64 = date.get(5..7).and_then(|s| s.parse().ok()).unwrap_or(1);
    for (index, line) in lines.iter().enumerate() {
        let lot = lots.get(line.lot).cloned().unwrap_or_default();
        let lot_id = lot_ids.get(line.lot).copied().unwrap_or(0);
        let key = line.chukta.map(|i| format!("ck-{i:03}"));
        let (name_ur, name_en, phone) = match line.chukta {
            Some(i) => {
                let (ur, en) = support::account_names(u32::try_from(i * 7 + 3).unwrap_or(0));
                (Some(ur), Some(en), Some(format!("0301{i:07}")))
            }
            None => (None, None, None),
        };
        tx.prepare_cached(
            "INSERT INTO bikri_sales (bikri_id, lot_id, line_no, journal_entry_id, entry_date,
                 buyer_account_id, chukta_name_ur, chukta_name_en, chukta_phone, chukta_person_key,
                 qty, seller_rate_paisa, seller_amount_paisa, seller_comm_method,
                 seller_comm_value_bps, seller_comm_paisa, buyer_rate_paisa, buyer_value_paisa,
                 buyer_comm_method, buyer_comm_value_bps, buyer_comm_paisa, laga_method,
                 laga_value_paisa, laga_paisa, buyer_total_paisa, rate_diff_paisa, payment_type,
                 paid_paisa, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 'percent', 600, ?14,
                 ?12, ?13, 'percent', 600, ?15, 'per_unit', 800, ?16, ?17, 0, ?18, ?19, 1, 1)",
        )
        .map_err(db)?
        .execute(params![
            bikri_id,
            lot_id,
            i64::try_from(index + 1).unwrap_or(1),
            entry_id,
            date,
            line.buyer,
            name_ur,
            name_en,
            phone,
            key,
            line.qty,
            line.rate,
            line.value,
            line.s_comm,
            line.b_comm,
            line.laga,
            line.total,
            if line.paid == line.total {
                "cash"
            } else {
                "credit"
            },
            line.paid,
        ])
        .map_err(db)?;
        let sale_id = tx.last_insert_rowid();
        let city = ids
            .cities
            .get(usize::try_from(line.buyer).unwrap_or(0) % ids.cities.len().max(1))
            .copied();
        tx.prepare_cached(
            "INSERT INTO analytics_facts (journal_entry_id, entry_date, year, month, bikri_id, lot_id,
                 sale_id, item_id, seller_account_id, buyer_account_id, chukta_person_key, city_id,
                 qty, seller_amount_paisa, seller_comm_paisa, buyer_value_paisa, buyer_comm_paisa,
                 laga_paisa, sign, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?14, ?16, ?17, 1, 1, 1)",
        )
        .map_err(db)?
        .execute(params![
            entry_id,
            date,
            year,
            month,
            bikri_id,
            lot_id,
            sale_id,
            lot.item,
            lot.seller,
            line.buyer,
            key,
            city,
            line.qty,
            line.value,
            line.s_comm,
            line.b_comm,
            line.laga,
        ])
        .map_err(db)?;
    }

    // The journal entry (PRD FR-12.9).
    let mut journal: Vec<(i64, i64, i64, Option<String>)> = Vec::new();
    for line in &lines {
        let key = line.chukta.map(|i| format!("ck-{i:03}"));
        journal.push((line.buyer, line.total, 0, key.clone()));
        if line.paid > 0 {
            journal.push((ids.cash, line.paid, 0, None));
            journal.push((line.buyer, 0, line.paid, key));
        }
    }
    let (mut s_comm, mut b_comm, mut laga) = (0_i64, 0_i64, 0_i64);
    for line in &lines {
        s_comm += line.s_comm;
        b_comm += line.b_comm;
        laga += line.laga;
    }
    for lot in &lots {
        let credit = lot.gross - lot.s_comm - LOT_FREIGHT - LOT_LABOUR - LOT_MUNSHIANA - LOT_PHONE;
        if credit >= 0 {
            journal.push((lot.seller, 0, credit, None));
        } else {
            journal.push((lot.seller, -credit, 0, None));
        }
    }
    let n = i64::try_from(n_lots).unwrap_or(1);
    for (account, amount) in [
        (ids.comm_seller, s_comm),
        (ids.comm_buyer, b_comm),
        (ids.laga, laga),
        (ids.phone, LOT_PHONE * n),
        (ids.labour, LOT_LABOUR * n),
        (ids.freight, LOT_FREIGHT * n),
        (ids.munshiana, LOT_MUNSHIANA * n),
    ] {
        journal.push((account, 0, amount, None));
    }
    let mut total = 0_i64;
    {
        let mut insert = tx
            .prepare_cached(
                "INSERT INTO journal_lines (entry_id, line_no, account_id, chukta_person_key, dr_paisa, cr_paisa, created_by, updated_by)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 1)",
            )
            .map_err(db)?;
        let mut line_no = 0_i64;
        for (account, dr, cr, key) in &journal {
            if *dr == 0 && *cr == 0 {
                continue;
            }
            line_no += 1;
            total += dr;
            insert
                .execute(params![entry_id, line_no, account, key, dr, cr])
                .map_err(db)?;
        }
    }
    tx.prepare_cached(
        "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
    )
    .map_err(db)?
    .execute(params![total, FAKE_HASH, entry_id])
    .map_err(db)?;

    // The partner split of the shared income (PRD C9, C10): floor to whole rupees, the leftover
    // rupees to the largest share (the first partner).
    let shared = s_comm + b_comm + laga + LOT_PHONE * n;
    let rupees = shared / 100;
    let bps = [4_000_i64, 3_000, 3_000];
    let mut parts: Vec<i64> = bps.iter().map(|b| rupees * b / 10_000 * 100).collect();
    let leftover = shared - parts.iter().sum::<i64>();
    if let Some(first) = parts.first_mut() {
        *first += leftover;
    }
    for ((partner, share), amount) in ids.partners.iter().zip(bps).zip(parts) {
        tx.prepare_cached(
            "INSERT INTO partner_share_rows (journal_entry_id, entry_date, partner_id, kind, base_paisa, share_bps, amount_paisa, created_by, updated_by)
             VALUES (?1, ?2, ?3, 'income', ?4, ?5, ?6, 1, 1)",
        )
        .map_err(db)?
        .execute(params![entry_id, date, partner, shared, share, amount])
        .map_err(db)?;
    }
    let _ = number_i;
    Ok(())
}

/// Seller advances, money lent and its returns, vouchers (receipts and payments) and people in
/// the dead amount.
fn other_entries(
    tx: &Connection,
    ids: &Ids,
    days: &[String],
    rng: &mut Rng,
) -> Result<(), ServiceError> {
    let day =
        |rng: &mut Rng| -> String { days.get(rng.index(days.len())).cloned().unwrap_or_default() };
    let entry = |kind: &str, date: &str, source_id: i64| -> Result<i64, ServiceError> {
        tx.prepare_cached(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
             VALUES (?1, ?2, 'voucher', ?3, 1, 1)",
        )
        .map_err(db)?
        .execute(params![date, kind, source_id])
        .map_err(db)?;
        Ok(tx.last_insert_rowid())
    };
    let two_lines = |entry_id: i64,
                     dr: i64,
                     cr: i64,
                     amount: i64,
                     advance: bool|
     -> Result<(), ServiceError> {
        tx.prepare_cached(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, is_advance, created_by, updated_by)
             VALUES (?1, 1, ?2, ?4, 0, ?5, 1, 1)",
        )
        .map_err(db)?
        .execute(params![entry_id, dr, cr, amount, advance])
        .map_err(db)?;
        tx.prepare_cached(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
             VALUES (?1, 2, ?2, 0, ?3, 1, 1)",
        )
        .map_err(db)?
        .execute(params![entry_id, cr, amount])
        .map_err(db)?;
        tx.prepare_cached(
            "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
        )
        .map_err(db)?
        .execute(params![amount, FAKE_HASH, entry_id])
        .map_err(db)?;
        Ok(())
    };
    let mut voucher = tx
        .prepare_cached(
            "INSERT INTO vouchers (type, number, entry_date, account_id, amount_paisa, mode, reason,
                 return_by_date, narration, current_entry_id, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, 'cash', ?6, ?7, ?8, ?9, 1, 1)",
        )
        .map_err(db)?;

    // Seller advances: a debit on the seller marked as an advance, cash paid out.
    for n in 1..=3_000_i64 {
        let date = day(rng);
        let seller = ids
            .sellers
            .get(rng.index(ids.sellers.len()))
            .copied()
            .unwrap_or(0);
        let amount = (1 + i64::try_from(rng.below(300)).unwrap_or(0)) * 10_000;
        let e = entry("advance", &date, n)?;
        two_lines(e, seller, ids.cash, amount, true)?;
        voucher
            .execute(params![
                "advance",
                n,
                date,
                seller,
                amount,
                Option::<String>::None,
                Option::<String>::None,
                "advance",
                e
            ])
            .map_err(db)?;
    }
    // Money lent to the people of control 0103, and some returns.
    let mut lent_number = 0_i64;
    let mut return_number = 0_i64;
    for _ in 0..3_000 {
        let date = day(rng);
        let person = ids
            .others
            .get(rng.index(ids.others.len()))
            .copied()
            .unwrap_or(0);
        let amount = (1 + i64::try_from(rng.below(100)).unwrap_or(0)) * 100_000;
        lent_number += 1;
        let e = entry("money_lent", &date, lent_number)?;
        two_lines(e, person, ids.cash, amount, false)?;
        let return_by = days
            .iter()
            .skip_while(|d| d.as_str() < date.as_str())
            .nth(30)
            .cloned();
        voucher
            .execute(params![
                "money_lent",
                lent_number,
                date,
                person,
                amount,
                "loan",
                return_by,
                "money lent",
                e
            ])
            .map_err(db)?;
        if rng.below(100) < 50 {
            return_number += 1;
            let back = amount / 200 * 100;
            let e = entry("money_lent_return", &date, return_number)?;
            two_lines(e, ids.cash, person, back, false)?;
            voucher
                .execute(params![
                    "money_lent_return",
                    return_number,
                    date,
                    person,
                    back,
                    Option::<String>::None,
                    Option::<String>::None,
                    "returned",
                    e
                ])
                .map_err(db)?;
        }
    }
    // 110,000 receipts and payments in the voucher list (10 years at about 30 a day), as
    // `speed_pages` makes them, linked to no entry.
    let mut plain = tx
        .prepare_cached(
            "INSERT INTO vouchers (type, number, entry_date, account_id, amount_paisa, mode, narration, status, cancel_reason, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, 'cash', ?6, ?7, ?8, 1, 1)",
        )
        .map_err(db)?;
    for i in 0..110_000_i64 {
        let receipt = i % 2 == 0;
        let account = if receipt {
            ids.buyers
                .get(rng.index(ids.buyers.len()))
                .copied()
                .unwrap_or(0)
        } else {
            ids.sellers
                .get(rng.index(ids.sellers.len()))
                .copied()
                .unwrap_or(0)
        };
        let cancelled = i % 50 == 7;
        plain
            .execute(params![
                if receipt { "receipt" } else { "payment" },
                i / 2 + 1,
                days.get(usize::try_from(i % 3_650).unwrap_or(0))
                    .cloned()
                    .unwrap_or_default(),
                account,
                (1 + i % 900) * 100,
                format!("voucher {i}"),
                if cancelled { "cancelled" } else { "saved" },
                if cancelled { Some("wrong") } else { None },
            ])
            .map_err(db)?;
    }
    Ok(())
}

/// Cities on the buyers, a person type on the borrowers, and 100 people in the dead amount.
fn finishing_touches(tx: &Connection) -> Result<(), ServiceError> {
    tx.execute_batch(
        "UPDATE accounts SET person_type = CASE id % 3 WHEN 0 THEN 'staff' WHEN 1 THEN 'garden_owner' ELSE 'other' END
         WHERE code GLOB '0103*' AND is_system = 0;
         UPDATE accounts SET city_id = (SELECT MIN(id) FROM cities) + (id % (SELECT COUNT(*) FROM cities))
         WHERE party_type IN ('seller', 'buyer') AND is_system = 0;",
    )
    .map_err(db)?;
    // The 100 buyers who owe the most in the dead amount (PRD FR-10.10): the mark holds what they
    // owed, and the account is no longer active.
    tx.execute_batch(
        "INSERT INTO dead_marks (account_id, entry_date, reason, note, owed_paisa, after_entry_id, is_active, created_by, updated_by)
         SELECT account_id, '2026-09-01', 'died', '', bal, (SELECT MAX(id) FROM journal_entries WHERE entry_date <= '2026-09-01'), 1, 1, 1
         FROM (SELECT l.account_id, SUM(l.dr_paisa) - SUM(l.cr_paisa) AS bal
               FROM journal_lines l JOIN accounts a ON a.id = l.account_id
               WHERE a.party_type = 'buyer' AND a.is_system = 0 AND a.id % 90 = 5
               GROUP BY l.account_id HAVING bal > 0 ORDER BY bal LIMIT 100);
         UPDATE accounts SET is_active = 0 WHERE id IN (SELECT account_id FROM dead_marks);",
    )
    .map_err(db)?;
    Ok(())
}
