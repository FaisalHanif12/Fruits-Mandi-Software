//! The large test data of NFR-SCL-1 for the D2 speed benchmarks. Owner: `robustness`.
//!
//! `ensure_volume` builds one data file once and keeps it in `backend/target/perf/nfr_scl1`
//! (the `MANDI_PERF_DIR` variable can point somewhere else). The next run opens it again.
//! The file is built through plain SQL, not through the services, because 20,000 accounts and
//! 3.5 million journal lines would take hours through one transaction per action. It is built
//! with the same tables, indexes and triggers as the app: every entry is first a draft, then
//! one `UPDATE` makes it posted, so the balance trigger checks every entry.
//!
//! What is in it (NFR-SCL-1, Section 1.1 of the NFR):
//!
//! - 20,000 accounts: 8,999 sellers (03011 + serial), 8,999 buyers (0304 + serial) and 2,002
//!   others (0103 + serial).
//! - 2,000 masters: 400 cities, 500 items, 100 packings, 300 addas, 100 stores, 200 expense
//!   heads, 400 staff.
//! - About 3.5 million journal lines in about 1.03 million posted entries over the 10 years
//!   11-10-2016 to 10-10-2026. The lines are skewed on purpose, as in a shop: the busiest buyer
//!   has about 55,000 lines, the busiest seller about 49,000, the cash account about 514,000
//!   and an ordinary buyer about 90 (the worst cases for a party card, a balance and a ledger).
//!
//! Not in it: the 1.2 million sale lines of NFR-SCL-1 (`bikri_sales`). No D1 or D2 query reads
//! that table. The bikri benchmarks of D3 add it.

#![allow(dead_code)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use chrono::{Duration as Days, NaiveDate};
use mandi_services::core::text::{search_en, search_phonetic, search_ur};
use mandi_services::db::rusqlite::{Connection, params};
use mandi_services::testing::TestEnv;
use mandi_services::{Secret, ServiceError, auth};

/// The password of the test Owner.
pub const OWNER_PASSWORD: &str = "secret1";

/// Sellers and buyers: serials 1 to 8,999 each (PRD FR-10.3).
pub const SELLERS: u32 = 8_999;
/// Buyers.
pub const BUYERS: u32 = 8_999;
/// Other accounts (control 0103).
pub const OTHERS: u32 = 2_002;
/// Posted journal entries.
pub const ENTRIES: u64 = 1_030_000;
/// Entries written in one transaction.
const BATCH: u64 = 20_000;
/// A change here rebuilds the cached file.
const VERSION: &str = "d2-v2";

/// Writes one line to the error output. (`eprintln!` is denied in real code, Code Standards G4.)
pub fn note(text: &str) {
    let _ = writeln!(std::io::stderr(), "{text}");
}

/// Stops the benchmark with a message. A bench has no way to return an error to criterion, and
/// `panic!` is denied (Code Standards B8).
pub fn fail(text: &str) -> ! {
    note(text);
    std::process::exit(1)
}

/// First day of the data.
pub fn first_day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2016, 10, 11).unwrap_or_default()
}

/// Last day of the data (the day of the PRD worked example).
pub fn last_day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 10).unwrap_or_default()
}

/// Small deterministic random numbers (splitmix64). The same seed gives the same data.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`. `n` must be above 0.
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }

    /// A number in `0..n` as usize.
    pub fn index(&mut self, n: usize) -> usize {
        usize::try_from(self.below(u64::try_from(n).unwrap_or(1))).unwrap_or(0)
    }
}

const FIRST: [(&str, &str); 40] = [
    ("ارشد", "Arshad"),
    ("اکرم", "Akram"),
    ("شاہد", "Shahid"),
    ("ناصر", "Nasir"),
    ("رحیم", "Rahim"),
    ("کریم", "Karim"),
    ("طارق", "Tariq"),
    ("محمود", "Mehmood"),
    ("اسلم", "Aslam"),
    ("جاوید", "Javed"),
    ("اقبال", "Iqbal"),
    ("یوسف", "Yousaf"),
    ("عمران", "Imran"),
    ("فیصل", "Faisal"),
    ("سلیم", "Saleem"),
    ("نعیم", "Naeem"),
    ("خالد", "Khalid"),
    ("زاہد", "Zahid"),
    ("اشرف", "Ashraf"),
    ("افضل", "Afzal"),
    ("بشیر", "Bashir"),
    ("حنیف", "Hanif"),
    ("رفیق", "Rafiq"),
    ("شفیق", "Shafiq"),
    ("منیر", "Munir"),
    ("نذیر", "Nazir"),
    ("اعجاز", "Ijaz"),
    ("اعظم", "Azam"),
    ("ظفر", "Zafar"),
    ("غلام", "Ghulam"),
    ("حسین", "Hussain"),
    ("علی", "Ali"),
    ("عابد", "Abid"),
    ("عارف", "Arif"),
    ("کامران", "Kamran"),
    ("لطیف", "Latif"),
    ("مجید", "Majid"),
    ("نواز", "Nawaz"),
    ("وحید", "Waheed"),
    ("یاسین", "Yasin"),
];

const SECOND: [(&str, &str); 25] = [
    ("ملتانی", "Multani"),
    ("ٹول ٹیکس", "Toll Tax"),
    ("اینڈ سنز", "and Sons"),
    ("بٹ", "Butt"),
    ("چوہدری", "Chaudhry"),
    ("شیخ", "Sheikh"),
    ("راجپوت", "Rajput"),
    ("جٹ", "Jutt"),
    ("آرائیں", "Arain"),
    ("قریشی", "Qureshi"),
    ("انصاری", "Ansari"),
    ("گجر", "Gujjar"),
    ("کھوکھر", "Khokhar"),
    ("ڈوگر", "Dogar"),
    ("سیال", "Sial"),
    ("بھٹی", "Bhatti"),
    ("ورک", "Virk"),
    ("لودھی", "Lodhi"),
    ("ملک", "Malik"),
    ("خان", "Khan"),
    ("کاشف", "Kashif"),
    ("ٹریڈرز", "Traders"),
    ("فروٹ کمپنی", "Fruit Company"),
    ("برادرز", "Brothers"),
    ("اینڈ کو", "and Co"),
];

const PLACE: [(&str, &str); 20] = [
    ("قصور", "Kasur"),
    ("لاہور", "Lahore"),
    ("اوکاڑہ", "Okara"),
    ("پتوکی", "Pattoki"),
    ("چونیاں", "Chunian"),
    ("دیپالپور", "Depalpur"),
    ("ملتان", "Multan"),
    ("فیصل آباد", "Faisalabad"),
    ("سرگودھا", "Sargodha"),
    ("گوجرانوالہ", "Gujranwala"),
    ("شیخوپورہ", "Sheikhupura"),
    ("ساہیوال", "Sahiwal"),
    ("بہاولپور", "Bahawalpur"),
    ("رحیم یار خان", "Rahim Yar Khan"),
    ("جھنگ", "Jhang"),
    ("خانیوال", "Khanewal"),
    ("وہاڑی", "Vehari"),
    ("پاکپتن", "Pakpattan"),
    ("نارووال", "Narowal"),
    ("سیالکوٹ", "Sialkot"),
];

const ITEMS: [(&str, &str); 25] = [
    ("کیلا", "Banana"),
    ("سیب", "Apple"),
    ("انگور", "Grapes"),
    ("آم", "Mango"),
    ("مالٹا", "Orange"),
    ("کینو", "Kinnow"),
    ("انار", "Pomegranate"),
    ("امرود", "Guava"),
    ("آڑو", "Peach"),
    ("خربوزہ", "Melon"),
    ("تربوز", "Watermelon"),
    ("مولی", "Radish"),
    ("گاجر", "Carrot"),
    ("آلو", "Potato"),
    ("پیاز", "Onion"),
    ("ٹماٹر", "Tomato"),
    ("بھنڈی", "Okra"),
    ("کریلا", "Bitter Gourd"),
    ("توری", "Ridge Gourd"),
    ("گوبھی", "Cauliflower"),
    ("مٹر", "Peas"),
    ("پالک", "Spinach"),
    ("ساگ", "Mustard Greens"),
    ("لہسن", "Garlic"),
    ("ادرک", "Ginger"),
];

/// The names of account number `i` (0 to 19,999). Different `i` give different names.
pub fn account_names(i: u32) -> (String, String) {
    let i = usize::try_from(i).unwrap_or(0);
    let first = FIRST.get(i % 40).copied().unwrap_or_default();
    let second = SECOND.get((i / 40) % 25).copied().unwrap_or_default();
    let place = PLACE.get((i / 1000) % 20).copied().unwrap_or_default();
    (
        format!("{} {} {}", first.0, second.0, place.0),
        format!("{} {} {}", first.1, second.1, place.1),
    )
}

/// The path of the cached data folder.
pub fn volume_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MANDI_PERF_DIR") {
        return PathBuf::from(dir);
    }
    // CARGO_MANIFEST_DIR is backend/crates/services; the target folder is backend/target.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("perf")
        .join("nfr_scl1")
}

fn marker(dir: &Path) -> PathBuf {
    dir.join("volume.ok")
}

fn id_of(conn: &Connection, sql: &str, code: &str) -> i64 {
    conn.query_row(sql, [code], |r| r.get(0)).unwrap_or(0)
}

/// What the generator made, for the report.
#[derive(Debug, Clone)]
pub struct Volume {
    /// The folder with `data/mandi.db`.
    pub dir: PathBuf,
    /// The busiest buyer (about 55,000 lines).
    pub busy_buyer: i64,
    /// The busiest seller (about 49,000 lines).
    pub busy_seller: i64,
    /// An ordinary buyer.
    pub median_buyer: i64,
    /// Journal lines.
    pub lines: i64,
}

/// Opens the cached data, building it first if it is missing or out of date.
pub fn ensure_volume() -> Volume {
    let dir = volume_dir();
    let expected = format!("{VERSION} {ENTRIES}");
    let current = std::fs::read_to_string(marker(&dir)).unwrap_or_default();
    if current.trim() != expected {
        note(&format!(
            "perf data: building the NFR-SCL-1 volume in {}",
            dir.display()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        build(&dir);
        let _ = std::fs::write(marker(&dir), &expected);
        // `MANDI_PERF_BUILD_ONLY=1 cargo bench --bench search` makes the data and stops.
        if std::env::var("MANDI_PERF_BUILD_ONLY").is_ok() {
            note("perf data: built; stopping because MANDI_PERF_BUILD_ONLY is set");
            std::process::exit(0);
        }
    }
    let conn = Connection::open(dir.join("data").join("mandi.db"));
    let (busy_buyer, busy_seller, median_buyer, lines) = match conn {
        Ok(conn) => (
            id_of(&conn, "SELECT id FROM accounts WHERE code = ?1", "03040001"),
            id_of(
                &conn,
                "SELECT id FROM accounts WHERE code = ?1",
                "030110001",
            ),
            id_of(&conn, "SELECT id FROM accounts WHERE code = ?1", "03044500"),
            conn.query_row("SELECT COUNT(*) FROM journal_lines", [], |r| r.get(0))
                .unwrap_or(0),
        ),
        Err(_) => (0, 0, 0, 0),
    };
    Volume {
        dir,
        busy_buyer,
        busy_seller,
        median_buyer,
        lines,
    }
}

/// Writes what is in the data file to the error output, so a benchmark log shows the volume
/// it was measured on.
pub fn describe(volume: &Volume) {
    let path = volume.dir.join("data").join("mandi.db");
    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if let Ok(conn) = Connection::open(path) {
        let lines_of = |id: i64| -> i64 {
            conn.query_row(
                "SELECT COUNT(*) FROM journal_lines WHERE account_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap_or(0)
        };
        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap_or(0) };
        let cash = id_of(&conn, "SELECT id FROM accounts WHERE code = ?1", "01010001");
        note(&format!(
            "perf data: {} accounts, {} entries, {} journal lines, file {} MB; lines on the busiest buyer {}, busiest seller {}, an ordinary buyer {}, the cash account {}",
            count("SELECT COUNT(*) FROM accounts"),
            count("SELECT COUNT(*) FROM journal_entries"),
            volume.lines,
            size / 1_000_000,
            lines_of(volume.busy_buyer),
            lines_of(volume.busy_seller),
            lines_of(volume.median_buyer),
            lines_of(cash),
        ));
    }
}

/// Opens the volume as a test environment. The Owner is created on the first call. Reads only
/// (use [`copy_of_volume`] for a bench that writes).
pub fn open(dir: &Path) -> (TestEnv, auth::Actor) {
    let env =
        TestEnv::new(dir).unwrap_or_else(|e| fail(&format!("cannot open the perf data: {e}")));
    let actor = sign_in(&env).unwrap_or_else(|e| fail(&format!("cannot sign in: {e}")));
    (env, actor)
}

fn sign_in(env: &TestEnv) -> Result<auth::Actor, ServiceError> {
    let status = auth::status(&env.ctx)?;
    if status.first_run {
        auth::first_run(&env.ctx, &first_run_input())?;
    }
    auth::login(&env.ctx, "owner", &Secret::new(OWNER_PASSWORD))?;
    auth::require(&env.ctx)
}

fn first_run_input() -> auth::FirstRunRaw {
    use mandi_services::settings::Firm;
    auth::FirstRunRaw {
        firm: Firm {
            name_ur: "ماہر عدنان اینڈ برادرز".to_owned(),
            name_en: "Maher Adnan and Brothers".to_owned(),
            mandi_ur: "نئی سبزی منڈی کوٹ رادھا کشن".to_owned(),
            mandi_en: "New Sabzi Mandi Kot Radha Kishan".to_owned(),
            address: "Kot Radha Kishan".to_owned(),
            phone: "0300-1234567".to_owned(),
        },
        owner_username: "Owner".to_owned(),
        owner_display_name: "Maher Adnan".to_owned(),
        password: Secret::new(OWNER_PASSWORD),
        password_repeat: Secret::new(OWNER_PASSWORD),
    }
}

/// A private copy of the volume in a temporary folder, for a bench that writes.
pub fn copy_of_volume(volume: &Volume) -> tempfile::TempDir {
    let copy = tempfile::tempdir().unwrap_or_else(|e| fail(&format!("no temp folder: {e}")));
    let to = copy.path().join("data");
    std::fs::create_dir_all(&to).unwrap_or_else(|e| fail(&format!("no data folder: {e}")));
    std::fs::copy(
        volume.dir.join("data").join("mandi.db"),
        to.join("mandi.db"),
    )
    .unwrap_or_else(|e| fail(&format!("cannot copy the volume: {e}")));
    copy
}

fn build(dir: &Path) {
    let started = Instant::now();
    {
        // The Owner, the system accounts and the schema come from the real start-up path.
        let env =
            TestEnv::new(dir).unwrap_or_else(|e| fail(&format!("cannot create the data: {e}")));
        sign_in(&env).unwrap_or_else(|e| fail(&format!("cannot sign in: {e}")));
        env.ctx
            .db()
            .write(|tx| -> Result<(), ServiceError> {
                insert_masters(tx)?;
                insert_accounts(tx)?;
                Ok(())
            })
            .unwrap_or_else(|e| fail(&format!("cannot write accounts: {e}")));
        let ids = account_ids(&env);
        let mut rng = Rng(0x00D2_0000_5EED_0001);
        let days = (last_day() - first_day()).num_days().max(1);
        let day_text: Vec<String> = (0..=days)
            .map(|d| (first_day() + Days::days(d)).format("%Y-%m-%d").to_string())
            .collect();
        let mut written = 0_u64;
        while written < ENTRIES {
            let batch = BATCH.min(ENTRIES - written);
            env.ctx
                .db()
                .write(|tx| -> Result<(), ServiceError> {
                    insert_entries(tx, &ids, &day_text, &mut rng, written, batch)?;
                    Ok(())
                })
                .unwrap_or_else(|e| fail(&format!("cannot write entries: {e}")));
            written += batch;
            if written.is_multiple_of(200_000) {
                note(&format!(
                    "perf data: {written} of {ENTRIES} entries after {:?}",
                    started.elapsed()
                ));
            }
        }
    }
    // Fold the write-ahead log into the file, so a copy of the file alone is complete.
    if let Ok(conn) = Connection::open(dir.join("data").join("mandi.db")) {
        let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
        let _ = conn.execute_batch("ANALYZE");
    }
    note(&format!("perf data: built in {:?}", started.elapsed()));
}

struct Ids {
    sellers: Vec<i64>,
    buyers: Vec<i64>,
    cash: i64,
    commission: i64,
    labour: i64,
}

fn account_ids(env: &TestEnv) -> Ids {
    env.ctx
        .db()
        .read(|conn| -> Result<Ids, ServiceError> {
            let list = |like: &str| -> Result<Vec<i64>, ServiceError> {
                let mut statement = conn
                    .prepare(
                        "SELECT id FROM accounts WHERE code LIKE ?1
                         AND CAST(substr(code, -4) AS INTEGER) BETWEEN 1 AND 8999 ORDER BY code",
                    )
                    .map_err(mandi_services::db::DbError::from)?;
                let rows = statement
                    .query_map([like], |r| r.get::<_, i64>(0))
                    .map_err(mandi_services::db::DbError::from)?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| ServiceError::from(mandi_services::db::DbError::from(e)))
            };
            Ok(Ids {
                sellers: list("03011____")?,
                buyers: list("0304____")?,
                cash: id_of(conn, "SELECT id FROM accounts WHERE code = ?1", "01010001"),
                commission: id_of(conn, "SELECT id FROM accounts WHERE code = ?1", "04010001"),
                labour: id_of(conn, "SELECT id FROM accounts WHERE code = ?1", "02010001"),
            })
        })
        .unwrap_or_else(|e| fail(&format!("cannot read account ids: {e}")))
}

fn control_id(tx: &Connection, code: &str) -> i64 {
    id_of(tx, "SELECT id FROM account_controls WHERE code = ?1", code)
}

fn insert_accounts(tx: &Connection) -> Result<(), ServiceError> {
    let mut insert = tx
        .prepare_cached(
            "INSERT INTO accounts (control_id, code, serial, party_type, name_ur, name_en,
                 search_ur, search_en, search_phonetic, phone1, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, 1)",
        )
        .map_err(mandi_services::db::DbError::from)?;
    let kinds = [
        ("03011", "seller", SELLERS, 0_u32),
        ("0304", "buyer", BUYERS, SELLERS),
        ("0103", "other", OTHERS, SELLERS + BUYERS),
    ];
    for (control_code, party, count, shift) in kinds {
        let control = control_id(tx, control_code);
        for serial in 1..=count {
            let (ur, en) = account_names(serial.saturating_add(shift).saturating_sub(1));
            let phone = (serial % 20 == 0).then(|| format!("0300{serial:07}"));
            insert
                .execute(params![
                    control,
                    format!("{control_code}{serial:04}"),
                    serial,
                    party,
                    ur,
                    en,
                    search_ur(&ur),
                    search_en(&en),
                    search_phonetic(&en),
                    phone,
                ])
                .map_err(mandi_services::db::DbError::from)?;
        }
    }
    Ok(())
}

fn insert_masters(tx: &Connection) -> Result<(), ServiceError> {
    let db = |e| ServiceError::from(mandi_services::db::DbError::from(e));
    let named = |i: usize| {
        let first = FIRST.get(i % 40).copied().unwrap_or_default();
        let place = PLACE.get((i / 40) % 20).copied().unwrap_or_default();
        (
            format!("{} {}", first.0, place.0),
            format!("{} {}", first.1, place.1),
        )
    };
    for (table, count) in [
        ("cities", 400_usize),
        ("addas", 300),
        ("stores", 100),
        ("packings", 100),
    ] {
        let mut insert = tx
            .prepare_cached(&format!(
                "INSERT INTO {table} (code, name_ur, name_en, search_ur, search_en, search_phonetic)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
            ))
            .map_err(db)?;
        for i in 0..count {
            let (ur, en) = named(i + 7);
            insert
                .execute(params![
                    format!("{:03}", i + 1),
                    ur,
                    en,
                    search_ur(&ur),
                    search_en(&en),
                    search_phonetic(&en),
                ])
                .map_err(db)?;
        }
    }
    let mut insert = tx
        .prepare_cached(
            "INSERT INTO items (code, name_ur, name_en, search_ur, search_en, search_phonetic, item_group)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(db)?;
    for i in 0..500_usize {
        let item = ITEMS.get(i % 25).copied().unwrap_or_default();
        let grade = i / 25 + 1;
        let (ur, en) = (
            format!("{} نمبر {grade}", item.0),
            format!("{} No {grade}", item.1),
        );
        insert
            .execute(params![
                format!("{:03}", i + 1),
                ur,
                en,
                search_ur(&ur),
                search_en(&en),
                search_phonetic(&en),
                if i % 25 < 11 { "fruit" } else { "vegetable" },
            ])
            .map_err(db)?;
    }
    let cash = id_of(tx, "SELECT id FROM accounts WHERE code = ?1", "01010001");
    let mut insert = tx
        .prepare_cached(
            "INSERT INTO expense_heads (code, name_ur, name_en, search_ur, search_en, search_phonetic, account_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(db)?;
    for i in 0..200_usize {
        let (ur, en) = named(i + 3);
        insert
            .execute(params![
                format!("{:03}", i + 1),
                ur,
                en,
                search_ur(&ur),
                search_en(&en),
                search_phonetic(&en),
                cash,
            ])
            .map_err(db)?;
    }
    let mut insert = tx
        .prepare_cached(
            "INSERT INTO staff (name_ur, name_en, search_ur, search_en, search_phonetic, role)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(db)?;
    for i in 0..400_usize {
        let (ur, en) = named(i + 11);
        insert
            .execute(params![
                ur,
                en,
                search_ur(&ur),
                search_en(&en),
                search_phonetic(&en),
                ["munshi", "collector", "labour"]
                    .get(i % 3)
                    .copied()
                    .unwrap_or("munshi"),
            ])
            .map_err(db)?;
    }
    Ok(())
}

/// A 64-character hash. The benchmarks never verify the chain, so the same text is used.
const FAKE_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn insert_entries(
    tx: &Connection,
    ids: &Ids,
    days: &[String],
    rng: &mut Rng,
    first: u64,
    count: u64,
) -> Result<(), ServiceError> {
    let db = |e| ServiceError::from(mandi_services::db::DbError::from(e));
    let mut entry = tx
        .prepare_cached(
            "INSERT INTO journal_entries (entry_date, posting_type, source_type, source_id, created_by, updated_by)
             VALUES (?1, ?2, 'voucher', ?3, 1, 1)",
        )
        .map_err(db)?;
    let mut line = tx
        .prepare_cached(
            "INSERT INTO journal_lines (entry_id, line_no, account_id, dr_paisa, cr_paisa, created_by, updated_by)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 1)",
        )
        .map_err(db)?;
    let mut post = tx
        .prepare_cached(
            "UPDATE journal_entries SET status = 'posted', total_paisa = ?1, chain_hash = ?2 WHERE id = ?3",
        )
        .map_err(db)?;
    let last_day_index = days.len().saturating_sub(1) as u64;
    let hot_buyer = ids.buyers.first().copied().unwrap_or(0);
    let hot_seller = ids.sellers.first().copied().unwrap_or(0);
    for n in first..first.saturating_add(count) {
        let day = usize::try_from(n.saturating_mul(last_day_index) / ENTRIES).unwrap_or(0);
        let date = days.get(day).map(String::as_str).unwrap_or("2026-10-10");
        let buyer = if rng.below(100) < 6 {
            hot_buyer
        } else {
            ids.buyers
                .get(rng.index(ids.buyers.len()))
                .copied()
                .unwrap_or(hot_buyer)
        };
        let seller = if rng.below(100) < 6 {
            hot_seller
        } else {
            ids.sellers
                .get(rng.index(ids.sellers.len()))
                .copied()
                .unwrap_or(hot_seller)
        };
        // Whole rupees, as paisa.
        let total = i64::try_from(1_000 + rng.below(199_000)).unwrap_or(1_000) * 100;
        let fee = (total / 100 * 6 / 100 + 1) * 100;
        let kind = rng.below(100);
        let (posting, lines): (&str, Vec<(i64, i64, i64)>) = if kind < 50 {
            // A sale: buyer owes, seller is credited, commission and labour are income and payable.
            let labour = 25_000;
            (
                "bikri_first",
                vec![
                    (buyer, total + fee, 0),
                    (seller, 0, total - labour),
                    (ids.commission, 0, fee),
                    (ids.labour, 0, labour),
                ],
            )
        } else if kind < 70 {
            ("receipt", vec![(ids.cash, total, 0), (buyer, 0, total)])
        } else if kind < 90 {
            ("payment", vec![(seller, total, 0), (ids.cash, 0, total)])
        } else {
            // A general entry with several lines.
            let a = total / 3 / 100 * 100;
            (
                "general",
                vec![
                    (buyer, total, 0),
                    (ids.cash, total, 0),
                    (seller, 0, a),
                    (ids.commission, 0, a),
                    (ids.labour, 0, total - a - a),
                    (buyer, 0, total),
                ],
            )
        };
        // The sale above is out of balance by `labour` if total < labour; totals start at
        // 100,000 paisa so the credit of the seller stays positive.
        let sum_dr: i64 = lines.iter().map(|l| l.1).sum();
        let sum_cr: i64 = lines.iter().map(|l| l.2).sum();
        let (lines, sum_dr) = if sum_dr == sum_cr {
            (lines, sum_dr)
        } else {
            // Put the difference on the seller or on cash so the entry balances.
            let mut fixed = lines;
            let diff = sum_dr - sum_cr;
            fixed.push((ids.cash, 0, diff.max(0)));
            fixed.push((ids.cash, (-diff).max(0), 0));
            fixed.retain(|l| l.1 > 0 || l.2 > 0);
            let dr: i64 = fixed.iter().map(|l| l.1).sum();
            (fixed, dr)
        };
        let entry_id = {
            entry
                .execute(params![date, posting, i64::try_from(n).unwrap_or(0) + 1])
                .map_err(db)?;
            tx.last_insert_rowid()
        };
        for (index, (account, dr, cr)) in lines.iter().enumerate() {
            line.execute(params![
                entry_id,
                i64::try_from(index + 1).unwrap_or(1),
                account,
                dr,
                cr
            ])
            .map_err(db)?;
        }
        post.execute(params![sum_dr, FAKE_HASH, entry_id])
            .map_err(db)?;
    }
    Ok(())
}
