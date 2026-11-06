//! The tool `app_state`: small read-only reads of the app's own state (PRD FR-7.87, plan 55 §11.1,
//! R1 to R4). One tool, four reads, so the fixed part of the prompt stays small:
//!
//! | `what` | Gives | Same as |
//! |---|---|---|
//! | `rules` | the commission and charge rule that wins for a party, an item, an item group or the defaults, and the rules behind it (PRD 11.3) | the rule list and the "test a rule" box |
//! | `attention` | the attention list "آج کے کام" (PRD FR-7.44) | the bell and the dashboard list |
//! | `days` | the day state: closed up to, whether today is open, the last month close | the status bar and day close |
//! | `audit` | the audit rows of one بکری or one account (or, with neither, the newest of the whole log), with the fields that changed | the audit viewer (PRD FR-21.3) |
//!
//! Nothing is written (Code Standards B29). Nothing of PRD FR-7.83's never-send list goes into a
//! result: no phone, address, father or firm name, note, reason, typed text or driver. A partner
//! figure obeys the partner lock (PRD FR-7.82). The rules and audit reads are in `state_rules.rs`
//! and `state_audit.rs`.
//!
//! Example (worked day): `app_state{what:"days"}` gives today 10-10-26, closed up to 09-10-26,
//! today open, and no month close yet.

use mandi_reports::dashboard::{AttentionKind, AttentionList, AttentionTarget, attention};
use mandi_services::day;
use mandi_services::month::{self, MonthState};
use serde::Deserialize;
use serde_json::Value;

use super::report::reports_stop;
use super::{Draft, Env, State, Stop, ToolError, input, service_stop};
use crate::answer::{AnswerSource, ScreenRoute};
use crate::results::{Money, NameVal, Node, Val};

/// Which read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum What {
    /// The rule that wins, and the rules behind it.
    Rules,
    /// The attention list.
    Attention,
    /// The day state.
    Days,
    /// The audit rows of one entry.
    Audit,
}

/// The input of `app_state`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Which read.
    pub what: What,
    /// A seller or a buyer (a find match or a key): for `rules`, his rules; for `audit`, an account's
    /// rows.
    #[serde(default)]
    pub party: Option<String>,
    /// An item (a find match or a key), or the word `fruit` or `vegetable` for a group: `rules` only.
    #[serde(default)]
    pub item: Option<String>,
    /// A بکری number: `audit` only.
    #[serde(default)]
    pub bikri: Option<i64>,
}

/// A filter that does not belong to this read is an error, as it is in `report`.
fn only(field: &str, given: bool, what: &str) -> Result<(), ToolError> {
    if given {
        Err(ToolError::new(
            "ERR_OUT_OF_RANGE",
            field,
            &format!("this read does not take {field}; {what}"),
        ))
    } else {
        Ok(())
    }
}

/// The tool.
pub fn run(env: &Env<'_>, state: &State, args: &Value) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let (party, item, bikri) = (&parsed.party, &parsed.item, &parsed.bikri);
    match parsed.what {
        What::Rules => {
            only("bikri", bikri.is_some(), "party and item are what it takes")?;
            super::state_rules::run(env, state, args, party.as_deref(), item.as_deref())
        }
        What::Audit => {
            only("item", item.is_some(), "party or bikri is what it takes")?;
            super::state_audit::run(env, state, args, party.as_deref(), *bikri)
        }
        What::Attention => {
            let what = "it takes no filter";
            only("party", party.is_some(), what)?;
            only("item", item.is_some(), what)?;
            only("bikri", bikri.is_some(), what)?;
            attention_read(env, args)
        }
        What::Days => {
            let what = "it takes no filter";
            only("party", party.is_some(), what)?;
            only("item", item.is_some(), what)?;
            only("bikri", bikri.is_some(), what)?;
            days_read(env, args)
        }
    }
}

/// The draft with the button of the screen it is about (the id of the result is filled in later).
pub(super) fn on_screen(mut draft: Draft, route: ScreenRoute) -> Draft {
    draft.source = Some(AnswerSource::Screen {
        result_id: String::new(),
        route,
    });
    draft
}

fn count(n: i64) -> Node {
    Node::Val(Val::Count(n))
}

fn plain(paisa: mandi_services::core::money::Paisa) -> Node {
    Node::Val(Val::Money(Money::plain(paisa)))
}

// ---- days -------------------------------------------------------------------------------------

/// `2026-09` as a month value.
fn month_val(text: &str) -> Node {
    let mut parts = text.split('-');
    match (
        parts.next().and_then(|y| y.parse::<i32>().ok()),
        parts.next().and_then(|m| m.parse::<u32>().ok()),
    ) {
        (Some(year), Some(month)) => Node::Val(Val::Month { year, month }),
        _ => Node::Null,
    }
}

/// The day state: closed up to, whether today is open (PRD FR-6.6), the last month close (PRD
/// FR-17.6) and how many finished months are not closed yet.
fn days_read(env: &Env<'_>, args: &Value) -> Result<Draft, Stop> {
    let status = day::status(env.ctx).map_err(service_stop)?;
    let months = month::list(env.ctx).map_err(service_stop)?;
    let today = env.today.to_iso();
    // Dates are `YYYY-MM-DD`, so text order is date order.
    let today_open = status
        .closed_up_to
        .as_deref()
        .is_none_or(|closed| today.as_str() > closed);
    // The list is newest first.
    let last_close = months.rows.iter().find(|r| r.state == MonthState::Closed);
    let to_close = months
        .rows
        .iter()
        .filter(|r| matches!(r.state, MonthState::Ready | MonthState::DaysOpen))
        .count();
    let node = Node::obj(vec![
        ("what", Node::Str("days".to_owned())),
        ("today", Node::Val(Val::Date(today))),
        (
            "closed_up_to",
            status
                .closed_up_to
                .map_or(Node::Null, |d| Node::Val(Val::Date(d))),
        ),
        ("today_open", Node::Bool(today_open)),
        (
            "last_month_close",
            last_close.map_or(Node::Null, |row| {
                Node::obj(vec![
                    ("month", month_val(&row.month)),
                    (
                        "closed_on",
                        row.closed_on
                            .clone()
                            .map_or(Node::Null, |d| Node::Val(Val::Date(d))),
                    ),
                ])
            }),
        ),
        (
            "months_to_close",
            count(i64::try_from(to_close).unwrap_or(i64::MAX)),
        ),
    ]);
    Ok(on_screen(
        Draft::new("app_state", args.clone(), node),
        ScreenRoute::DayClose,
    ))
}

// ---- attention --------------------------------------------------------------------------------

/// The stable key of a kind of the list: the word the model reads and cites by.
fn kind_key(kind: AttentionKind) -> &'static str {
    match kind {
        AttentionKind::Drafts => "drafts",
        AttentionKind::OpenDays => "open_days",
        AttentionKind::StockWaiting => "stock_waiting",
        AttentionKind::OverCreditLimit => "over_credit_limit",
        AttentionKind::DuesOld => "dues_old",
        AttentionKind::PayablesOld => "payables_old",
        AttentionKind::BackupOld => "backup_old",
        AttentionKind::LoansOverdue => "loans_overdue",
        AttentionKind::TakesOverdue => "takes_overdue",
        AttentionKind::NamesMissing => "names_missing",
    }
}

/// The most urgent record of an item, as a node the model can pass on: an account is `a20` and a
/// partner `s1`, so `party_card` and `report` take it; a بکری is `bk12`.
fn first_node(first: &mandi_reports::dashboard::AttentionFirst) -> Node {
    let prefix = match first.target {
        AttentionTarget::Account => "a",
        AttentionTarget::Partner => "s",
        AttentionTarget::Bikri => "bk",
        AttentionTarget::Purchase => "pu",
        AttentionTarget::Arrival => "ar",
    };
    let name = match (&first.name_ur, &first.name_en) {
        (Some(ur), Some(en)) => Node::Val(Val::Name(NameVal {
            name_ur: ur.clone(),
            name_en: en.clone(),
            tag: first.tag.clone(),
            code: None,
        })),
        _ => Node::Null,
    };
    // One person of a پختہ کھاتہ account shares the account with others: the account key would
    // open all of them, so he is given by name only.
    let key = if first.person_key.is_some() {
        Node::Null
    } else {
        Node::Str(format!("{prefix}{}", first.id))
    };
    Node::obj(vec![("key", key), ("name", name)])
}

fn item_node(item: &mandi_reports::dashboard::AttentionItem) -> Node {
    Node::obj(vec![
        ("count", count(item.count)),
        ("amount", item.amount_paisa.map_or(Node::Null, plain)),
        ("qty", item.qty.map_or(Node::Null, count)),
        ("days", item.days.map_or(Node::Null, count)),
        ("first", item.first.as_ref().map_or(Node::Null, first_node)),
    ])
}

/// The attention list (PRD FR-7.44), the very list of the bell and the dashboard. The money of
/// what partners took past its return-by day is behind the partner code and is left out while the
/// section is locked (PRD FR-7.82).
fn attention_read(env: &Env<'_>, args: &Value) -> Result<Draft, Stop> {
    let mut list: AttentionList = attention(env.ctx.db(), env.backup).map_err(reports_stop)?;
    if env.partner_locked {
        list.hide_partners();
    }
    let has_takes = list
        .items
        .iter()
        .any(|i| i.kind == AttentionKind::TakesOverdue);
    let rows = list
        .items
        .iter()
        .map(|item| (kind_key(item.kind).to_owned(), item_node(item)))
        .collect();
    let node = Node::obj(vec![
        ("what", Node::Str("attention".to_owned())),
        ("total", count(list.total_count)),
        ("items", Node::Keyed(rows)),
    ]);
    // The dashboard shows this list as "آج کے کام".
    let mut draft = on_screen(
        Draft::new("app_state", args.clone(), node),
        ScreenRoute::Dashboard,
    );
    draft.partner_parts_locked = env.partner_locked && has_takes;
    draft.partner_data = !env.partner_locked && has_takes;
    Ok(draft)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_month_text_becomes_a_month_value() {
        assert_eq!(
            month_val("2026-09"),
            Node::Val(Val::Month {
                year: 2026,
                month: 9
            })
        );
        assert_eq!(month_val("nonsense"), Node::Null);
    }

    #[test]
    fn every_attention_kind_has_its_own_key() {
        let mut keys: Vec<&str> = AttentionKind::ALL.iter().map(|k| kind_key(*k)).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), AttentionKind::ALL.len());
    }
}
