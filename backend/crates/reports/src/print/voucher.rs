//! The slip of a voucher (plan 15, PRD FR-15.2): a وصولی, an ادائیگی or a پیشگی on one page, with
//! the party, the amount and the amount in words. There is no line about how the money moved:
//! all money is روکڑ (PRD 15.1, v1.6).
//!
//! The same generic [`PrintData`] as the بکری documents: the template lays out what it receives.
//! Every number is a stored value of the voucher. Titles and labels are Fluent message IDs; the
//! only sentence built here is the amount in words (PRD FR-19.7).

use chrono::{DateTime, Utc};
use mandi_core::accounts::{BUYER_CONTROL, short_code_of};
use mandi_core::auth::Language;
use mandi_core::english_words::english_words;
use mandi_core::money::Grouping;
use mandi_core::printing::{PrintDoc, PrintSetup};
use mandi_core::urdu_words::urdu_words;
use mandi_core::vouchers::{VoucherRow, VoucherType};
use mandi_db::Db;
use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers as voucher_repo;

use super::format::NumberStyle;
use super::header::{footer, header, pick};
use super::ids::*;
use super::load::{FirmNames, load_firm, load_grouping, load_setup};
use super::{
    AmountRole, PrintAmount, PrintData, PrintFact, PrintPage, PrintParty, PrintWords, TextDir,
    VoucherPrintDataInput,
};
use crate::error::ReportsError;

/// Reads the voucher and the settings, and builds its slip.
///
/// # Errors
/// `ReportsError::NotFound` if the voucher does not exist; a database or data error otherwise.
pub fn voucher_print_data(
    db: &Db,
    input: &VoucherPrintDataInput,
) -> Result<PrintData, ReportsError> {
    let loaded = db.read(|conn| {
        let row = voucher_repo::get(conn, input.voucher_id)?.ok_or(ReportsError::NotFound)?;
        Ok::<_, ReportsError>(Loaded {
            party_kind: party_kind(conn, &row)?,
            collector: match row.collector_staff_id {
                Some(id) => voucher_repo::staff_names(conn, id)?,
                None => None,
            },
            row,
            firm: load_firm(conn)?,
            grouping: load_grouping(conn)?,
            setup: load_setup(conn)?,
        })
    })?;
    build_slip(&loaded, input.language, db.clock().now_utc())
}

/// What the slip is about, besides the voucher itself.
struct Loaded {
    row: VoucherRow,
    /// `buyer`, `seller` or `other`, with whether the account is a پختہ کھاتہ one.
    party_kind: (PartyKind, bool),
    collector: Option<(String, String)>,
    firm: FirmNames,
    grouping: Grouping,
    setup: PrintSetup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartyKind {
    Buyer,
    Seller,
    Other,
}

fn party_kind(conn: &Connection, row: &VoucherRow) -> Result<(PartyKind, bool), ReportsError> {
    let Some(account) = &row.account else {
        return Ok((PartyKind::Other, false));
    };
    let facts = account_repo::facts(conn, account.id)?
        .ok_or_else(|| ReportsError::Data("a voucher names no account".to_owned()))?;
    let kind = match facts.control_code.as_str() {
        BUYER_CONTROL => PartyKind::Buyer,
        mandi_core::accounts::SELLER_CONTROL => PartyKind::Seller,
        _ => PartyKind::Other,
    };
    Ok((kind, facts.is_pukhta_khata))
}

fn build_slip(
    loaded: &Loaded,
    language: Language,
    now_utc: DateTime<Utc>,
) -> Result<PrintData, ReportsError> {
    let row = &loaded.row;
    let style = NumberStyle::new(language, loaded.grouping, loaded.setup.urdu_digits);
    let (title_id, amount_label) = match row.voucher_type {
        VoucherType::Receipt => (TITLE_RECEIPT_SLIP, SLIP_AMOUNT_RECEIVED),
        VoucherType::Payment => (TITLE_PAYMENT_SLIP, SLIP_AMOUNT_PAID),
        VoucherType::Advance => (TITLE_ADVANCE_SLIP, SLIP_AMOUNT_ADVANCE),
        other => {
            return Err(ReportsError::Data(format!(
                "the slip of a {} voucher is not built",
                other.as_str()
            )));
        }
    };
    // The party: the one-time person by his own names, else the account (PRD FR-13.2).
    let (kind, is_pukhta) = loaded.party_kind;
    let account = row
        .account
        .as_ref()
        .ok_or_else(|| ReportsError::Data("a voucher names no account".to_owned()))?;
    let party = match (&row.person, is_pukhta) {
        (Some(person), true) => PrintParty {
            role_id: match kind {
                PartyKind::Seller => PARTY_ONETIME_SELLER,
                _ => PARTY_ONETIME_BUYER,
            }
            .to_owned(),
            code: None,
            short_code: None,
            name: pick(language, &person.name_ur, &person.name_en),
            tag: None,
            phone: Some(person.phone.trim().to_owned()).filter(|p| !p.is_empty()),
        },
        _ => PrintParty {
            role_id: match kind {
                PartyKind::Buyer => PARTY_BUYER,
                PartyKind::Seller => PARTY_SELLER,
                PartyKind::Other => PARTY_ACCOUNT,
            }
            .to_owned(),
            code: Some(account.code.clone()),
            short_code: short_code_of(&account.code),
            name: pick(language, &account.name_ur, &account.name_en),
            tag: account.tag.clone(),
            phone: None,
        },
    };

    let mut facts = vec![
        PrintFact {
            label_id: FACT_VOUCHER_NUMBER.to_owned(),
            value: style.count(row.number),
        },
        PrintFact {
            label_id: FACT_DATE.to_owned(),
            value: style.date(&row.entry_date)?,
        },
    ];
    if !row.reason.is_empty() {
        facts.push(PrintFact {
            label_id: FACT_REASON.to_owned(),
            value: row.reason.clone(),
        });
    }
    if let Some((name_ur, name_en)) = &loaded.collector {
        facts.push(PrintFact {
            label_id: FACT_COLLECTOR.to_owned(),
            value: pick(language, name_ur, name_en),
        });
    }
    if !row.narration.is_empty() {
        facts.push(PrintFact {
            label_id: FACT_NARRATION.to_owned(),
            value: row.narration.clone(),
        });
    }

    // PRD FR-15.2: the amount, and the amount in words.
    let words = match language {
        Language::Ur => urdu_words(row.amount_paisa)?,
        Language::En => english_words(row.amount_paisa, loaded.grouping)?,
    };
    let page = PrintPage {
        serial: style.count(1),
        party,
        lines: Vec::new(),
        amounts: vec![PrintAmount {
            label_id: amount_label.to_owned(),
            amount: style.money(row.amount_paisa)?,
            role: AmountRole::Total,
        }],
        // All money is روکڑ (PRD 15.1, v1.6): a slip has no line about how it was paid, and an
        // old bank or cheque row prints the same way.
        payment_id: None,
        words: Some(PrintWords {
            label_id: IN_WORDS.to_owned(),
            text: words,
        }),
    };
    Ok(PrintData {
        doc: PrintDoc::ReceiptSlip,
        language,
        dir: TextDir::of(language),
        paper: loaded.setup.paper_of(PrintDoc::ReceiptSlip),
        urdu_digits: style.urdu_digits(),
        header: header(language, &loaded.firm, title_id, facts),
        footer: footer(style.printed_at(now_utc)),
        number: style.count(row.number),
        cancelled: row.status == "cancelled",
        pages: vec![page],
    })
}
