//! Helpers for the partner tests: a signed-in Owner, partners with shares, posting an entry
//! through `ledger::post`, and reading the share rows back with plain SQL.

use mandi_services::auth::{self, Actor};
use mandi_services::core::dates::BusinessDate;
use mandi_services::core::error::{ErrorCode, ErrorValue};
use mandi_services::core::ids::AccountId;
use mandi_services::core::masters::{ExpenseHeadInput, MasterInput, MasterRow};
use mandi_services::core::money::Paisa;
use mandi_services::core::partners::{
    PartnerCreateInput, PartnerRow, PartnersSetSharesInput, ShareInput,
};
use mandi_services::domain::ledger::{LedgerLine, PostingType, Side, SourceType, build_entry};
use mandi_services::ledger::{self, PostingEvent};
use mandi_services::testing::TestEnv;
use mandi_services::{ServiceError, masters, partners};
use tempfile::TempDir;

use crate::common::{env, login_owner, raw, set_up};

pub use mandi_services::domain::ledger::Side::{Cr, Dr};

/// A set-up database with the Owner signed in. Today is 10-10-2026.
pub fn ready() -> (TempDir, TestEnv, Actor) {
    let (dir, env) = env();
    set_up(&env);
    login_owner(&env);
    let owner = auth::require(&env.ctx).unwrap();
    (dir, env, owner)
}

pub fn rupees(value: i64) -> Paisa {
    Paisa::from_rupees(value).unwrap()
}

pub fn date(text: &str) -> BusinessDate {
    BusinessDate::parse_iso(text).unwrap()
}

/// The id of an account by its code.
pub fn id_of(env: &TestEnv, code: &str) -> i64 {
    env.ctx
        .db()
        .read(|c| mandi_services::db::accounts::id_by_code(c, code))
        .unwrap()
        .unwrap()
}

pub fn count(dir: &TempDir, sql: &str) -> i64 {
    raw(dir).query_row(sql, [], |r| r.get(0)).unwrap()
}

/// Adds a partner. Asks again with the words the spelling check named, as the Owner would
/// confirm them on the screen (PRD FR-7.66).
pub fn new_partner(
    env: &TestEnv,
    owner: &Actor,
    name_ur: &str,
    name_en: &str,
    opening: Option<&str>,
) -> Result<PartnerRow, ServiceError> {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..4 {
        let input = PartnerCreateInput {
            name_ur: name_ur.to_owned(),
            name_en: name_en.to_owned(),
            opening_capital: opening.map(str::to_owned),
            confirmed_rules_words: words.clone(),
        };
        match partners::create(&env.ctx, owner, &input) {
            Err(ServiceError::Rejected(items))
                if !items.is_empty()
                    && items
                        .iter()
                        .all(|i| i.code == ErrorCode::ErrSpellingUnconfirmed) =>
            {
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        words.extend(found.split(' ').map(str::to_owned));
                    }
                }
            }
            other => return other,
        }
    }
    Err(ServiceError::reject(ErrorCode::ErrInternal))
}

/// Sets the shares from a `DD-MM-YYYY` day: `(partner id, percent text)` for each partner.
pub fn set_shares(
    env: &TestEnv,
    owner: &Actor,
    from: &str,
    shares: &[(i64, &str)],
) -> Result<Vec<PartnerRow>, ServiceError> {
    partners::set_shares(
        &env.ctx,
        owner,
        &PartnersSetSharesInput {
            effective_from: from.to_owned(),
            shares: shares
                .iter()
                .map(|&(partner_id, share)| ShareInput {
                    partner_id,
                    share: share.to_owned(),
                })
                .collect(),
        },
    )
}

/// Partners الف, ب and ج with 40 / 30 / 30 from today (PRD 4.2). Returns their ids.
pub fn three_partners(env: &TestEnv, owner: &Actor) -> [i64; 3] {
    let a = new_partner(env, owner, "الف", "Partner A", None)
        .unwrap()
        .id;
    let b = new_partner(env, owner, "ب", "Partner B", None).unwrap().id;
    let c = new_partner(env, owner, "ج", "Partner C", None).unwrap().id;
    set_shares(env, owner, "10-10-2026", &[(a, "40"), (b, "30"), (c, "30")]).unwrap();
    [a, b, c]
}

/// A shop expense head (control 0501) and the id of its account.
pub fn expense_head(env: &TestEnv, owner: &Actor) -> i64 {
    let mut words: Vec<String> = Vec::new();
    for _ in 0..4 {
        let saved = masters::save(
            &env.ctx,
            owner,
            &MasterInput::ExpenseHead(ExpenseHeadInput {
                id: None,
                version: None,
                code: String::new(),
                name_ur: "بجلی".to_owned(),
                name_en: "Electricity".to_owned(),
                confirmed_rules_words: words.clone(),
            }),
        );
        match saved {
            Ok(MasterRow::ExpenseHead(row)) => return row.account_id,
            Ok(_) => break,
            Err(ServiceError::Rejected(items)) => {
                for item in items {
                    if let Some(ErrorValue::Text(found)) = item.values.get("words") {
                        words.extend(found.split(' ').map(str::to_owned));
                    }
                }
            }
            Err(other) => panic!("expense head: {other:?}"),
        }
    }
    panic!("the expense head was not saved");
}

pub fn line(account_id: i64, side: Side, amount: i64) -> LedgerLine {
    LedgerLine::new(AccountId::new(account_id), side, rupees(amount))
}

/// Posts an entry dated `day` (`YYYY-MM-DD`) through the real `ledger::post`.
pub fn post(
    env: &TestEnv,
    owner: &Actor,
    day: &str,
    posting_type: PostingType,
    lines: Vec<LedgerLine>,
) -> Result<i64, ServiceError> {
    let entry = build_entry(lines).unwrap();
    let source_type = match posting_type {
        PostingType::Opening => SourceType::Account,
        PostingType::MonthClose => SourceType::MonthClose,
        _ => SourceType::Voucher,
    };
    env.ctx.db().write(|tx| {
        ledger::post(
            &env.ctx,
            tx,
            owner.user_id,
            &PostingEvent {
                date: date(day),
                posting_type,
                source_type,
                source_id: 1,
                narration: None,
                reverses: None,
                entry,
            },
        )
        .map(|posted| posted.entry_id)
    })
}

/// A general entry with the cash account on the other side of one line: `account` gets
/// `side` for `amount`, cash gets the opposite.
pub fn against_cash(
    env: &TestEnv,
    owner: &Actor,
    day: &str,
    account: i64,
    side: Side,
    amount: i64,
) -> Result<i64, ServiceError> {
    let cash = id_of(env, "01010001");
    let other = if side == Side::Dr { Cr } else { Dr };
    post(
        env,
        owner,
        day,
        PostingType::General,
        vec![line(account, side, amount), line(cash, other, amount)],
    )
}

/// One stored share row, in rupees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub partner_id: i64,
    pub kind: String,
    pub base: i64,
    pub bps: i64,
    pub amount: i64,
}

pub fn row(partner_id: i64, kind: &str, base: i64, bps: i64, amount: i64) -> Row {
    Row {
        partner_id,
        kind: kind.to_owned(),
        base,
        bps,
        amount,
    }
}

/// The share rows of one entry, in the order they were written.
pub fn rows_of(dir: &TempDir, entry_id: i64) -> Vec<Row> {
    let conn = raw(dir);
    let mut statement = conn
        .prepare(
            "SELECT partner_id, kind, base_paisa, share_bps, amount_paisa FROM partner_share_rows
             WHERE journal_entry_id = ?1 ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([entry_id], |r| {
            Ok(Row {
                partner_id: r.get(0)?,
                kind: r.get(1)?,
                base: r.get::<_, i64>(2)? / 100,
                bps: r.get(3)?,
                amount: r.get::<_, i64>(4)? / 100,
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// The amounts of rows, in rupees.
pub fn amounts(rows: &[Row]) -> Vec<i64> {
    rows.iter().map(|r| r.amount).collect()
}

/// The first rejection of an error, as its code, field and number value `key`.
pub fn first(error: &ServiceError, key: &str) -> (ErrorCode, Option<String>, Option<i64>) {
    match error {
        ServiceError::Rejected(items) => {
            let item = items.first().unwrap();
            let value = match item.values.get(key) {
                Some(ErrorValue::Number(n)) => Some(*n),
                _ => None,
            };
            (item.code, item.field.clone(), value)
        }
        other => (other.code(), None, None),
    }
}

/// Every rejection of an error as `(code, field)`.
pub fn all(error: &ServiceError) -> Vec<(ErrorCode, String)> {
    match error {
        ServiceError::Rejected(items) => items
            .iter()
            .map(|i| (i.code, i.field.clone().unwrap_or_default()))
            .collect(),
        other => panic!("not a rejection: {other:?}"),
    }
}

/// Reverses a posted entry through the real `ledger::reverse` and returns the reversal's id.
pub fn reverse(env: &TestEnv, owner: &Actor, entry_id: i64) -> i64 {
    env.ctx
        .db()
        .write(|tx| {
            ledger::reverse(&env.ctx, tx, owner.user_id, entry_id, "test").map(|p| p.entry_id)
        })
        .unwrap()
}

/// Panics unless the error is a `ServiceError::Data` whose text contains `needle`. A test that
/// only checks "some Data error" could pass for the wrong reason.
pub fn assert_data(error: &ServiceError, needle: &str) {
    match error {
        ServiceError::Data(text) => assert!(text.contains(needle), "{text:?} lacks {needle:?}"),
        other => panic!("not a data error: {other:?}"),
    }
}
