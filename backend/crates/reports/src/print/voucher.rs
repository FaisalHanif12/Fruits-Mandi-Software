//! The slip of a voucher (plan 15, PRD FR-15.2): a وصولی, an ادائیگی or a پیشگی on one page, with
//! the party, the amount, the way the money moved and the amount in words.
//!
//! The same generic [`PrintData`] as the بکری documents: the template lays out what it receives.
//! Every number is a stored value of the voucher. Titles and labels are Fluent message IDs; the
//! only sentence built here is the amount in words (PRD FR-19.7).

use chrono::{DateTime, Utc};
use mandi_core::accounts::BUYER_CONTROL;
use mandi_core::auth::Language;
use mandi_core::english_words::english_words;
use mandi_core::money::Grouping;
use mandi_core::printing::{PrintDoc, PrintSetup};
use mandi_core::urdu_words::urdu_words;
use mandi_core::vouchers::{VoucherMode, VoucherRow, VoucherType};
use mandi_db::Db;
use mandi_db::accounts as account_repo;
use mandi_db::rusqlite::Connection;
use mandi_db::vouchers as voucher_repo;

use super::format::NumberStyle;
use super::ids::*;
use super::load::{FirmNames, load_firm, load_grouping, load_setup};
use super::{
    AmountRole, PrintAmount, PrintData, PrintFact, PrintFirm, PrintPage, PrintParty, PrintWords,
    TextDir, VoucherPrintDataInput,
};
use crate::error::ReportsError;

/// The name in the print language; the other script if that one is empty.
fn pick(language: Language, name_ur: &str, name_en: &str) -> String {
    let (first, second) = match language {
        Language::Ur => (name_ur, name_en),
        Language::En => (name_en, name_ur),
    };
    if first.trim().is_empty() {
        second.to_owned()
    } else {
        first.to_owned()
    }
}

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
            bank: match row.bank_account_id {
                Some(id) => voucher_repo::bank_account_label(conn, id)?,
                None => None,
            },
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
    bank: Option<(String, String, String)>,
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
            name: pick(language, &person.name_ur, &person.name_en),
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
            name: pick(language, &account.name_ur, &account.name_en),
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
    // The bank and the cheque are as typed (PRD FR-7.25).
    if let Some((bank, title, number)) = &loaded.bank {
        facts.push(PrintFact {
            label_id: FACT_BANK.to_owned(),
            value: format!("{bank} - {title} - {number}"),
        });
    }
    if !row.cheque_no.is_empty() {
        let date = match &row.cheque_date {
            Some(date) => format!(" / {}", style.date(date)?),
            None => String::new(),
        };
        facts.push(PrintFact {
            label_id: FACT_CHEQUE.to_owned(),
            value: format!("{}{date}", row.cheque_no),
        });
    }
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
        party,
        lines: Vec::new(),
        amounts: vec![PrintAmount {
            label_id: amount_label.to_owned(),
            amount: style.money(row.amount_paisa)?,
            role: AmountRole::Total,
        }],
        payment_id: Some(
            match row.mode {
                VoucherMode::Cash => MODE_CASH,
                VoucherMode::Bank => MODE_BANK,
                VoucherMode::Cheque => MODE_CHEQUE,
            }
            .to_owned(),
        ),
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
        title_id: title_id.to_owned(),
        firm: PrintFirm {
            name: pick(language, &loaded.firm.name_ur, &loaded.firm.name_en),
            mandi: pick(language, &loaded.firm.mandi_ur, &loaded.firm.mandi_en),
            address: loaded.firm.address.clone(),
            phone: loaded.firm.phone.clone(),
        },
        number: style.count(row.number),
        facts,
        printed_at: style.printed_at(now_utc),
        cancelled: row.status == "cancelled",
        pages: vec![page],
    })
}
