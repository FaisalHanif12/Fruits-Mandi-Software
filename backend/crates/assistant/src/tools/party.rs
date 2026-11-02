//! The tool `party_card`: one party's card now, and the checked balance used by `find` too (PRD
//! FR-7.38, FR-7.76, D6-system-prompt Section 5.2).
//!
//! A party's balance is kept in two places that must agree: the party card (one query on the
//! posted journal) and the closing balance of his ledger R13 as of now. Both are read in the same
//! moment; the balance is marked checked only when they are equal, and both places travel with it
//! (Code Standards Q7). A balance in `find` is read the same way, not from the search index.
//!
//! Example (worked day): اکرم ٹول ٹیکس, card 2,880 بنام and R13 closing 2,880 بنام: checked.

use mandi_reports::reports::{
    PartyFilter, ReportFilters, ReportId, ReportRunInput, run as run_report,
};
use mandi_services::assist;
use mandi_services::core::accounts::{AccountType, BalanceSide, PartyCard};
use mandi_services::core::money::Paisa;
use serde::Deserialize;
use serde_json::Value;

use super::{Draft, Env, Stop, ToolError, input, record, service_stop};
use crate::answer::AnswerSource;
use crate::results::{Money, Node, Side, Val};
use crate::source::AnswerRecord;

/// The input of `party_card`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// A find match (`r1.matches.0`) or a key from a result (`a20`).
    pub party: String,
}

/// The side of a balance as the results show it.
pub fn side_of(side: BalanceSide) -> Side {
    match side {
        BalanceSide::Dr => Side::Dr,
        BalanceSide::Cr => Side::Cr,
        BalanceSide::Zero => Side::Zero,
    }
}

/// The closing balance of R13 as of now, for one account: the second place of a party's balance.
///
/// # Errors
/// A service failure as a stop.
pub fn ledger_closing(env: &Env<'_>, account_id: i64) -> Result<(Paisa, Side), Stop> {
    let run = ReportRunInput {
        id: ReportId::R13,
        filters: ReportFilters {
            party: Some(PartyFilter {
                account_id,
                person_key: None,
            }),
            ..ReportFilters::default()
        },
        offset: 0,
        limit: 1,
        sort: None,
        group_by_month: false,
    };
    let page = run_report(env.ctx.db(), &run)
        .map_err(|e| Stop::Error(ToolError::new("ERR_INTERNAL", "party", &e.to_string())))?;
    let closing = page
        .summary
        .iter()
        .find(|line| line.label_id == "report-closing-balance")
        .map(|line| &line.cell);
    match closing {
        Some(mandi_reports::reports::Cell::Balance { paisa, side }) => Ok((*paisa, side_of(*side))),
        _ => Err(Stop::Error(ToolError::new(
            "ERR_INTERNAL",
            "party",
            "no closing balance",
        ))),
    }
}

/// A party's balance with its two places and whether they agree.
///
/// # Errors
/// A service failure as a stop.
pub fn checked_balance(env: &Env<'_>, card: &PartyCard) -> Result<Money, Stop> {
    let (ledger, ledger_side) = ledger_closing(env, card.id)?;
    let card_side = side_of(card.balance.side);
    let agree = ledger == card.balance.amount_paisa && ledger_side == card_side;
    Ok(Money {
        paisa: card.balance.amount_paisa,
        side: Some(card_side),
        checked: Some(agree),
        places: vec![("party_card", card.balance.amount_paisa), ("R13", ledger)],
    })
}

/// The kind of account as a word for the model.
pub fn type_word(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Seller => "seller",
        AccountType::Buyer => "buyer",
        AccountType::MoneyLent => "money_lent",
        AccountType::Other => "other",
    }
}

/// The name of a card as a value.
pub fn card_name(card: &PartyCard) -> crate::results::NameVal {
    crate::results::NameVal {
        name_ur: card.name_ur.clone(),
        name_en: card.name_en.clone(),
        tag: card.tag.clone(),
        code: Some(card.code.clone()),
    }
}

/// The record of a card, for the buttons.
pub fn card_record(card: &PartyCard) -> AnswerRecord {
    AnswerRecord {
        id: card.id,
        code: Some(card.code.clone()),
        name_ur: card.name_ur.clone(),
        name_en: card.name_en.clone(),
        tag: card.tag.clone(),
    }
}

/// The tool.
pub fn run(env: &Env<'_>, state: &super::State, args: &Value) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let rec = record(env, state, &parsed.party, "party")?;
    if rec.prefix != "a" {
        return Err(ToolError::new("ERR_FORMAT_INVALID", "party", "a party is an account").into());
    }
    let card = assist::party_card(env.ctx, rec.id).map_err(service_stop)?;
    let balance = checked_balance(env, &card)?;
    let mut fields = vec![
        (
            "party",
            Node::obj(vec![
                ("key", Node::Str(format!("a{}", card.id))),
                ("code", Node::Str(card.code.clone())),
                ("type", Node::Str(type_word(card.account_type).to_owned())),
                ("name", Node::Val(Val::Name(card_name(&card)))),
            ]),
        ),
        ("balance", Node::Val(Val::Money(balance))),
        (
            "last_entry",
            card.last_entry_date
                .clone()
                .map_or(Node::Null, |d| Node::Val(Val::Date(d))),
        ),
    ];
    if let (Some(limit), used) = (card.credit_limit, card.credit_used) {
        fields.push((
            "credit",
            Node::obj(vec![
                ("limit", Node::Val(Val::Money(Money::plain(limit)))),
                (
                    "used",
                    used.map_or(Node::Null, |u| Node::Val(Val::Money(Money::plain(u)))),
                ),
            ]),
        ));
    }
    fields.push((
        "open_advance",
        card.advance_open
            .map_or(Node::Null, |a| Node::Val(Val::Money(Money::plain(a)))),
    ));
    fields.push(("dead", Node::Bool(card.dead.is_some())));
    fields.push(("active", Node::Bool(card.is_active)));
    let warnings = card
        .warnings
        .iter()
        .map(|w| Node::Str(format!("{:?}", w.kind)))
        .collect();
    fields.push(("warnings", Node::List(warnings)));
    // A seller's open payable and a buyer's oldest open bill, the same working as the screens.
    extras(env, &card, &mut fields)?;
    let mut draft = Draft::new("party_card", args.clone(), Node::obj(fields));
    draft.source = Some(AnswerSource::PartyCard {
        result_id: String::new(),
        party: card_record(&card),
    });
    Ok(draft)
}

fn extras(
    env: &Env<'_>,
    card: &PartyCard,
    fields: &mut Vec<(&'static str, Node)>,
) -> Result<(), Stop> {
    use mandi_services::core::vouchers::{OpenBillsInput, OpenPayablesInput};
    use mandi_services::vouchers;
    match card.account_type {
        AccountType::Buyer => {
            let open = vouchers::open_bills(
                env.ctx,
                &OpenBillsInput {
                    account_id: card.id,
                    person_key: None,
                },
            )
            .map_err(service_stop)?;
            let oldest = open
                .bills
                .iter()
                .min_by(|a, b| a.entry_date.cmp(&b.entry_date));
            if let Some(bill) = oldest {
                fields.push((
                    "oldest_bill",
                    Node::obj(vec![
                        ("date", Node::Val(Val::Date(bill.entry_date.clone()))),
                        ("days", Node::Val(Val::Count(bill.days))),
                        ("open", Node::Val(Val::Money(Money::plain(bill.open_paisa)))),
                    ]),
                ));
            }
        }
        AccountType::Seller => {
            let open = vouchers::open_payables(
                env.ctx,
                &OpenPayablesInput {
                    seller_id: card.id,
                    person_key: None,
                    include_settled: false,
                },
            )
            .map_err(service_stop)?;
            fields.push((
                "payable",
                Node::Val(Val::Money(Money::plain(open.open_paisa))),
            ));
        }
        _ => {}
    }
    Ok(())
}
