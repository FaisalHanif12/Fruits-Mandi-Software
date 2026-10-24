//! Commission rules and which rule wins (PRD 11, plan 09).
//!
//! - Rules are data (NFR-MNT-5): the defaults of PRD 4.2 are rows of `charge_rules`. A change
//!   never touches a saved sale line, because each line keeps its own copy of the method, value
//!   and amount (PRD FR-11.6).
//! - One user action is one transaction: the rule, its audit row (Code Standards S7). The rules
//!   kept in memory for fast lookup are thrown away after the commit and built again on the next
//!   use, so a stale rule can never decide.
//! - The rule form refuses what PRD 11.2 and 11.3 do not allow. The three defaults (seller
//!   commission, buyer commission, لاگا on fruit) are edited only in Settings, so the Charges tab
//!   always shows the rate that is in force.

use std::sync::Arc;

use mandi_db::accounts as account_repo;
use mandi_db::rules::{self as repo, NewRule};
use mandi_db::rusqlite::Connection;
use mandi_domain::core::accounts::{BUYER_CONTROL, KHATA_CONTROL, SELLER_CONTROL};
use mandi_domain::core::dates::{BusinessDate, format_ddmmyyyy};
use mandi_domain::core::error::{ErrorCode, ErrorValue};
use mandi_domain::core::money::{Paisa, Qty, line_value};
use mandi_domain::core::rules::{
    ChargeSide, ChargeTest, ChargeType, ChargeValue, ChargesInput, ChargesSettings,
    CreditLimitMode, DEFAULT_RATE_CHECK_PERCENT, DefaultCharge, ItemGroup, OverridesSettings,
    RATE_CHECK_PERCENT_RANGE, RuleInput, RuleRow, RuleSaved, RuleTest, RuleTestInput, RulesFilter,
    RulesList, RulesListInput, RulesTotals, check_combination, is_default_scope, level_of,
    prepare_rule, rule_value,
};
use mandi_domain::core::validate;
use mandi_domain::rules::{Query, Rule, RuleSet, charge_amount, resolve};
use serde_json::{Value, json};

use crate::audit::Event;
use crate::auth::Actor;
use crate::context::Context;
use crate::day;
use crate::error::{Rejection, ServiceError, money_error};
use crate::store;

/// The settings key of the two switches of "Overrides" and "Checks".
const OVERRIDES_KEY: &str = "overrides";
/// The charges of the test screen, in the order it lists them (PRD 11.2).
const TEST_CHARGES: [(ChargeSide, ChargeType); 6] = [
    (ChargeSide::Seller, ChargeType::Commission),
    (ChargeSide::Seller, ChargeType::Labour),
    (ChargeSide::Seller, ChargeType::Munshiana),
    (ChargeSide::Seller, ChargeType::Phone),
    (ChargeSide::Buyer, ChargeType::Commission),
    (ChargeSide::Buyer, ChargeType::Laga),
];

fn data(message: String) -> ServiceError {
    ServiceError::Data(message)
}

fn not_allowed(field: &str) -> ServiceError {
    ServiceError::reject_field(ErrorCode::ErrRuleNotAllowed, field)
}

// ---- The rules in memory ----

fn domain_rule(record: &repo::RuleRecord) -> Result<Rule, ServiceError> {
    let parse = |text: &str| {
        BusinessDate::parse_iso(text)
            .ok_or_else(|| data(format!("rule {}: date {text}", record.id)))
    };
    Ok(Rule {
        id: record.id,
        side: record.side,
        charge: record.charge,
        account_id: record.account_id,
        item_id: record.item_id,
        item_group: record.item_group,
        value: ChargeValue::from_columns(record.method, record.value_bps, record.value_paisa)
            .ok_or_else(|| data(format!("rule {}: method and value do not match", record.id)))?,
        from_date: parse(&record.from_date)?,
        to_date: record.to_date.as_deref().map(parse).transpose()?,
        is_active: true,
    })
}

/// The active rules filed for lookup, read inside the caller's connection. A save of a sale
/// reads its rules this way, inside its own transaction, so the rules it uses are the rules it
/// commits with.
///
/// # Errors
/// `ServiceError::Db`; `ServiceError::Data` for a row that does not make sense.
pub fn rule_set_in(conn: &Connection) -> Result<RuleSet, ServiceError> {
    let records = repo::load_active(conn)?;
    let rules = records
        .iter()
        .map(domain_rule)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RuleSet::build(rules))
}

/// The active rules, kept in memory (a live preview calls this on every change). The same
/// answers as [`rule_set_in`]; thrown away after any change to a rule.
///
/// # Errors
/// As [`rule_set_in`].
pub fn rule_set(ctx: &Context) -> Result<Arc<RuleSet>, ServiceError> {
    let mut slot = ctx.rules_lock();
    if let Some(set) = slot.as_ref() {
        return Ok(Arc::clone(set));
    }
    let set = Arc::new(ctx.db.read(rule_set_in)?);
    *slot = Some(Arc::clone(&set));
    Ok(set)
}

/// Throws the rules in memory away, so the next lookup reads the database. Called after the
/// commit of any change to a rule.
fn invalidate(ctx: &Context) {
    *ctx.rules_lock() = None;
}

// ---- Reading ----

fn mark_defaults(_conn: &Connection, rows: &mut [RuleRow]) -> Result<(), ServiceError> {
    // A default is a row of the default chain, known by its scope (PRD 4.2): the rule form must
    // not edit or switch off any row of it, old or new.
    for row in rows {
        row.is_default = is_default_scope(
            row.side,
            row.charge_type,
            row.account.as_ref().map(|a| a.id),
            row.item.as_ref().map(|i| i.id),
            row.item_group,
        );
    }
    Ok(())
}

fn row_of(conn: &Connection, id: i64) -> Result<RuleRow, ServiceError> {
    let mut rows = [repo::get(conn, id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "id"))?];
    mark_defaults(conn, &mut rows)?;
    let [row] = rows;
    Ok(row)
}

/// The rules that match a filter: by side and charge, then level (most specific first), then
/// newest first.
///
/// # Errors
/// `ServiceError::Db`.
pub fn list(ctx: &Context, filter: &RulesFilter) -> Result<Vec<RuleRow>, ServiceError> {
    ctx.db.read(|conn| {
        let mut rows = repo::list(conn, filter)?;
        mark_defaults(conn, &mut rows)?;
        Ok(rows)
    })
}

/// One page of the rules that match a filter, in the order of [`list`], with the counts over every
/// matching rule (PRD FR-19.11, B31). The order depends on the level of each rule, which is worked
/// out for all of them (at most 10,000, NFR-SCL-6) before the page is cut.
///
/// # Errors
/// `ServiceError::Db`.
pub fn list_page(ctx: &Context, input: &RulesListInput) -> Result<RulesList, ServiceError> {
    let rows = list(ctx, &input.filter)?;
    let count =
        |n: usize| i64::try_from(n).map_err(|_| ServiceError::Data("too many rows".to_owned()));
    let total_count = count(rows.len())?;
    let active_count = count(rows.iter().filter(|r| r.is_active).count())?;
    Ok(RulesList {
        rows: crate::page::slice(rows, input.offset, input.limit),
        total_count,
        totals: RulesTotals { active_count },
    })
}

/// One rule, for the edit form.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`.
pub fn get(ctx: &Context, id: i64) -> Result<RuleRow, ServiceError> {
    ctx.db.read(|conn| row_of(conn, id))
}

// ---- Writing ----

fn new_rule(prepared: &mandi_domain::core::rules::PreparedRule) -> NewRule {
    let (value_bps, value_paisa) = prepared.value.columns();
    NewRule {
        side: prepared.side,
        charge: prepared.charge,
        account_id: prepared.account_id,
        item_id: prepared.item_id,
        item_group: prepared.item_group,
        method: prepared.value.method(),
        value_bps,
        value_paisa,
        from_date: prepared.from_date.to_iso(),
        to_date: prepared.to_date.map(BusinessDate::to_iso),
        is_active: prepared.is_active,
        note: prepared.note.clone(),
    }
}

fn audit_json(rule: &NewRule) -> Value {
    json!({
        "side": rule.side.as_str(),
        "charge_type": rule.charge.as_str(),
        "account_id": rule.account_id,
        "item_id": rule.item_id,
        "item_group": rule.item_group.map(ItemGroup::as_str),
        "method": rule.method.as_str(),
        "value_bps": rule.value_bps,
        "value_paisa": rule.value_paisa,
        "from_date": rule.from_date,
        "to_date": rule.to_date,
        "is_active": rule.is_active,
        "note": rule.note,
    })
}

fn row_json(row: &RuleRow) -> Value {
    json!({
        "side": row.side.as_str(),
        "charge_type": row.charge_type.as_str(),
        "account_id": row.account.as_ref().map(|a| a.id),
        "item_id": row.item.as_ref().map(|i| i.id),
        "item_group": row.item_group.map(ItemGroup::as_str),
        "method": row.method.as_str(),
        "value_bps": row.value_bps,
        "value_paisa": row.value_paisa.map(Paisa::as_paisa),
        "from_date": row.from_date,
        "to_date": row.to_date,
        "is_active": row.is_active,
        "note": row.note,
    })
}

/// A party must be on the rule's side (PRD 11.3): a seller on the seller side, a buyer on the
/// buyer side. The پختہ کھاتہ accounts count (PRD FR-11.7). A separate khata can be the party of a
/// seller-side rule, because it can be the seller of a lot (PRD FR-16.7).
fn check_party(conn: &Connection, side: ChargeSide, account_id: i64) -> Result<(), ServiceError> {
    let facts = account_repo::facts(conn, account_id)?
        .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "account_id"))?;
    let wanted = match side {
        ChargeSide::Seller => SELLER_CONTROL,
        ChargeSide::Buyer => BUYER_CONTROL,
    };
    if facts.control_code == wanted
        || (side == ChargeSide::Seller && facts.control_code == KHATA_CONTROL)
    {
        Ok(())
    } else {
        Err(not_allowed("account_id"))
    }
}

/// The three defaults are changed only in Settings (a second default-level rule would hide the
/// one the Charges tab shows). A new rule with the scope of a default is refused.
fn refuse_second_default(
    side: ChargeSide,
    charge: ChargeType,
    account: Option<i64>,
    item: Option<i64>,
    group: Option<ItemGroup>,
) -> Result<(), ServiceError> {
    match (level_of(account, item, group), side, charge, group) {
        (Some(mandi_domain::core::rules::RuleLevel::Default), _, ChargeType::Commission, _) => {
            Err(not_allowed("charge_type"))
        }
        (
            Some(mandi_domain::core::rules::RuleLevel::ItemGroup),
            ChargeSide::Buyer,
            ChargeType::Laga,
            Some(ItemGroup::Fruit),
        ) => Err(not_allowed("item_group")),
        _ => Ok(()),
    }
}

/// Adds or edits a rule (PRD 11.1, plan 09). Returns the rule and the other active rules of the
/// same scope whose dates overlap (PRD FR-11.5).
///
/// # Errors
/// The field errors of `mandi_core::rules::prepare_rule`; `ERR_RULE_NOT_ALLOWED` on `account_id`
/// (a party on the wrong side), on `charge_type` or `item_group` (a second default) or on `id`
/// (a default row); `ERR_NOT_FOUND` on `account_id`, `item_id` or `id`; `ERR_VERSION_CHANGED`.
pub fn save(ctx: &Context, actor: &Actor, input: &RuleInput) -> Result<RuleSaved, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let prepared = prepare_rule(input, today)?;
    let saved = ctx.db.write(|tx| {
        let now = ctx.now_text();
        if let Some(account) = prepared.account_id {
            check_party(tx, prepared.side, account)?;
        }
        if let Some(item) = prepared.item_id
            && repo::item_group_of(tx, item)?.is_none()
        {
            return Err(ServiceError::reject_field(
                ErrorCode::ErrNotFound,
                "item_id",
            ));
        }
        let rule = new_rule(&prepared);
        let (id, old) = match prepared.id {
            None => {
                refuse_second_default(
                    prepared.side,
                    prepared.charge,
                    prepared.account_id,
                    prepared.item_id,
                    prepared.item_group,
                )?;
                (repo::insert(tx, &rule, actor.user_id, &now)?, None)
            }
            Some(id) => {
                let old = row_of(tx, id)?;
                if old.is_default {
                    return Err(not_allowed("id"));
                }
                // The version of the row read must still be the stored one.
                if prepared.version != Some(old.version) {
                    return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
                }
                refuse_second_default(
                    prepared.side,
                    prepared.charge,
                    prepared.account_id,
                    prepared.item_id,
                    prepared.item_group,
                )?;
                if !repo::update(tx, id, old.version, &rule, actor.user_id, &now)? {
                    return Err(ServiceError::reject(ErrorCode::ErrVersionChanged));
                }
                (id, Some(old))
            }
        };
        let overlaps = if rule.is_active {
            repo::overlaps(tx, &rule, Some(id))?
        } else {
            Vec::new()
        };
        let mut event = Event::new(
            Some(actor.user_id),
            if old.is_some() { "edit" } else { "create" },
            "charge_rule",
        )
        .entity_id(id)
        .new_value(audit_json(&rule));
        if let Some(old) = &old {
            event = event.old(row_json(old));
        }
        ctx.db.auditor().write(tx, &event.into_entry())?;
        Ok(RuleSaved {
            row: row_of(tx, id)?,
            overlaps,
        })
    })?;
    invalidate(ctx);
    Ok(saved)
}

/// Makes a rule active or inactive. Asking for the state it has changes nothing. A default
/// cannot be switched off. Returns the overlap warnings when it is switched on.
///
/// # Errors
/// `ERR_NOT_FOUND` on `id`; `ERR_RULE_NOT_ALLOWED` on `id` for a default.
pub fn set_active(
    ctx: &Context,
    actor: &Actor,
    id: i64,
    is_active: bool,
) -> Result<RuleSaved, ServiceError> {
    let saved = ctx.db.write(|tx| {
        let old = row_of(tx, id)?;
        if old.is_default && !is_active {
            return Err(not_allowed("id"));
        }
        if old.is_active != is_active {
            repo::set_active(tx, id, is_active, actor.user_id, &ctx.now_text())?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "edit", "charge_rule")
                    .entity_id(id)
                    .old(json!({ "is_active": old.is_active }))
                    .new_value(json!({ "is_active": is_active }))
                    .into_entry(),
            )?;
        }
        let row = row_of(tx, id)?;
        let overlaps = if row.is_active {
            repo::overlaps(tx, &new_rule_from_row(&row), Some(id))?
        } else {
            Vec::new()
        };
        Ok(RuleSaved { row, overlaps })
    })?;
    invalidate(ctx);
    Ok(saved)
}

fn new_rule_from_row(row: &RuleRow) -> NewRule {
    NewRule {
        side: row.side,
        charge: row.charge_type,
        account_id: row.account.as_ref().map(|a| a.id),
        item_id: row.item.as_ref().map(|i| i.id),
        item_group: row.item_group,
        method: row.method,
        value_bps: row.value_bps,
        value_paisa: row.value_paisa.map(Paisa::as_paisa),
        from_date: row.from_date.clone(),
        to_date: row.to_date.clone(),
        is_active: row.is_active,
        note: if row.note.is_empty() {
            None
        } else {
            Some(row.note.clone())
        },
    }
}

// ---- Test a rule ----

/// "Test a rule" (PRD FR-11.8): for a seller, a buyer, an item and a day, the winner of each
/// charge, its level, the levels skipped, and the amount on a quantity and a rate, with the
/// working. Uses the same lookup and the same amount function as a sale will. Nothing is saved.
///
/// # Errors
/// The field errors of the date, quantity and rate; `ERR_NOT_FOUND` on `seller_id`, `buyer_id`
/// or `item_id`; `ERR_FORMAT_INVALID` on `seller_id` or `buyer_id` for an account of the wrong
/// kind.
pub fn test(ctx: &Context, input: &RuleTestInput) -> Result<RuleTest, ServiceError> {
    let (_, today) = day::window(ctx)?;
    let mut errors = validate::FieldErrors::default();
    let date = errors.take(validate::date("date", &input.date, today));
    // `validate::qty` adds the `max` value the message needs (finding 13).
    let qty: Option<Qty> = errors.take(validate::qty("qty", &input.qty));
    let rate = errors.take(validate::rate("rate", &input.rate));
    errors.finish()?;
    let (Some(date), Some(qty), Some(rate)) = (date, qty, rate) else {
        return Err(ServiceError::reject(ErrorCode::ErrInternal));
    };
    let value = line_value(qty, rate).map_err(money_error)?;
    let (seller, buyer, group) = ctx.db.read(|conn| {
        let kind = |id: i64, field: &str, control: &str| -> Result<(), ServiceError> {
            let facts = account_repo::facts(conn, id)?
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, field))?;
            if facts.control_code == control {
                Ok(())
            } else {
                Err(ServiceError::reject_field(
                    ErrorCode::ErrFormatInvalid,
                    field,
                ))
            }
        };
        kind(input.seller_id, "seller_id", SELLER_CONTROL)?;
        kind(input.buyer_id, "buyer_id", BUYER_CONTROL)?;
        let group = repo::item_group_of(conn, input.item_id)?
            .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrNotFound, "item_id"))?;
        Ok::<_, ServiceError>((input.seller_id, input.buyer_id, group))
    })?;
    let set = rule_set(ctx)?;
    let mut charges = Vec::with_capacity(TEST_CHARGES.len());
    for (side, charge) in TEST_CHARGES {
        let found = resolve(
            &set,
            &Query {
                side,
                charge,
                party_id: Some(match side {
                    ChargeSide::Seller => seller,
                    ChargeSide::Buyer => buyer,
                }),
                item_id: input.item_id,
                item_group: Some(group),
                date,
                line_override: None,
            },
        );
        let charged =
            charge_amount(found.winner.as_ref(), charge, value, qty, true).map_err(money_error)?;
        charges.push(ChargeTest {
            side,
            charge_type: charge,
            winner: found.winner.as_ref().map(mandi_domain::rules::Winner::view),
            skipped: found.skipped,
            amount_paisa: charged.amount,
            working: charged.working,
        });
    }
    Ok(RuleTest {
        line_value_paisa: value,
        charges,
    })
}

// ---- Settings: the three defaults ----

/// The three defaults, as (name, side, charge, group).
const DEFAULTS: [(&str, ChargeSide, ChargeType, Option<ItemGroup>); 3] = [
    (
        "seller_commission",
        ChargeSide::Seller,
        ChargeType::Commission,
        None,
    ),
    (
        "buyer_commission",
        ChargeSide::Buyer,
        ChargeType::Commission,
        None,
    ),
    (
        "laga_fruit",
        ChargeSide::Buyer,
        ChargeType::Laga,
        Some(ItemGroup::Fruit),
    ),
];

fn default_charge(row: RuleRow) -> DefaultCharge {
    DefaultCharge {
        rule_id: row.id,
        method: row.method,
        value_text: row.value_text,
        version: row.version,
        from_date: row.from_date,
    }
}

/// The default in force on a day, marked as a default.
fn default_on(conn: &Connection, which: usize, date: &str) -> Result<RuleRow, ServiceError> {
    let (name, side, charge, group) = DEFAULTS
        .get(which)
        .copied()
        .ok_or_else(|| data("no such default".to_owned()))?;
    let mut rows = [repo::default_in_force(conn, side, charge, group, date)?
        .ok_or_else(|| data(format!("no default {name} is in force on {date}")))?];
    mark_defaults(conn, &mut rows)?;
    let [row] = rows;
    Ok(row)
}

fn charges_in(conn: &Connection, today: &str) -> Result<ChargesSettings, ServiceError> {
    Ok(ChargesSettings {
        seller_commission: default_charge(default_on(conn, 0, today)?),
        buyer_commission: default_charge(default_on(conn, 1, today)?),
        laga_fruit: default_charge(default_on(conn, 2, today)?),
    })
}

fn today_iso(ctx: &Context) -> String {
    ctx.clock().local_today().format("%Y-%m-%d").to_string()
}

/// The three global defaults in force today (PRD 4.2, Section 21 "Charges"), each with the day it
/// started.
///
/// # Errors
/// `ServiceError::Data` if a default is missing.
pub fn charges(ctx: &Context) -> Result<ChargesSettings, ServiceError> {
    let today = today_iso(ctx);
    ctx.db.read(|conn| charges_in(conn, &today))
}

/// The day before an ISO date.
fn day_before(date: &str) -> Result<String, ServiceError> {
    BusinessDate::parse_iso(date)
        .and_then(BusinessDate::previous_day)
        .map(BusinessDate::to_iso)
        .ok_or_else(|| data(format!("the day before {date}")))
}

/// Saves the three global defaults. A default is dated like any rule (PRD 11.3: the rule date is
/// the posting date). A change made today ends the row in force on the day before and starts a
/// new row today; a second change on the same day updates that day's row. So a back-dated
/// posting on an open day, and "Test a rule" on a past date, use the default in force on that
/// date. Saved sale lines keep what they stored (PRD FR-11.6). A default that did not change
/// writes nothing. All in one transaction, with an audit row for each row written.
///
/// # Errors
/// The percent and money codes on `<default>.value`; `ERR_RULE_NOT_ALLOWED` on
/// `<default>.method`; `ERR_VERSION_CHANGED` on `<default>.version`.
pub fn set_charges(
    ctx: &Context,
    actor: &Actor,
    input: &ChargesInput,
) -> Result<ChargesSettings, ServiceError> {
    // Check every value first, so all mistakes come back together.
    let wanted = [
        &input.seller_commission,
        &input.buyer_commission,
        &input.laga_fruit,
    ];
    let mut errors = validate::FieldErrors::default();
    let mut values: Vec<Option<ChargeValue>> = Vec::with_capacity(3);
    for ((name, side, charge, _), one) in DEFAULTS.iter().zip(wanted) {
        let combination = check_combination(&format!("{name}."), *side, *charge, one.method);
        values.push(match combination {
            Err(e) => {
                errors.push(e);
                None
            }
            Ok(()) => errors.take(rule_value(&format!("{name}.value"), one.method, &one.value)),
        });
    }
    errors.finish()?;
    let today = today_iso(ctx);
    let yesterday = day_before(&today)?;
    let saved = ctx.db.write(|tx| {
        let now = ctx.now_text();
        for (which, (one, value)) in wanted.iter().zip(&values).enumerate() {
            let Some(value) = value else {
                return Err(ServiceError::reject(ErrorCode::ErrInternal));
            };
            let name = DEFAULTS.get(which).map_or("", |d| d.0);
            // A default that starts after today can exist only if the computer's clock was moved
            // back (NFR-REL-11). Changing the earlier row would overlap it, so it is refused.
            let (_, side, charge, group) = DEFAULTS
                .get(which)
                .copied()
                .ok_or_else(|| data("no such default".to_owned()))?;
            if let Some(later) = repo::default_starting_after(tx, side, charge, group, &today)? {
                let shown = BusinessDate::parse_iso(&later)
                    .map(format_ddmmyyyy)
                    .unwrap_or(later);
                return Err(ServiceError::Rejected(vec![Rejection {
                    code: ErrorCode::ErrDateInFuture,
                    field: Some(format!("{name}.value")),
                    values: std::collections::BTreeMap::from([(
                        "today".to_owned(),
                        ErrorValue::Text(shown),
                    )]),
                }]));
            }
            let old = default_on(tx, which, &today)?;
            if old.version != one.version {
                return Err(ServiceError::reject_field(
                    ErrorCode::ErrVersionChanged,
                    &format!("{name}.version"),
                ));
            }
            let (value_bps, value_paisa) = value.columns();
            if old.method == value.method()
                && old.value_bps == value_bps
                && old.value_paisa.map(Paisa::as_paisa) == value_paisa
            {
                continue;
            }
            let mut rule = new_rule_from_row(&old);
            rule.method = value.method();
            rule.value_bps = value_bps;
            rule.value_paisa = value_paisa;
            if old.from_date == today {
                // A second change on the same day updates that day's row.
                if !repo::update(tx, old.id, old.version, &rule, actor.user_id, &now)? {
                    return Err(ServiceError::reject_field(
                        ErrorCode::ErrVersionChanged,
                        &format!("{name}.version"),
                    ));
                }
                ctx.db.auditor().write(
                    tx,
                    &Event::new(Some(actor.user_id), "edit", "charge_rule")
                        .entity_id(old.id)
                        .old(row_json(&old))
                        .new_value(audit_json(&rule))
                        .into_entry(),
                )?;
            } else {
                // End the row in force the day before, then start the new one today. The order
                // matters: the table refuses two defaults that cover the same day.
                if !repo::set_to_date(tx, old.id, old.version, &yesterday, actor.user_id, &now)? {
                    return Err(ServiceError::reject_field(
                        ErrorCode::ErrVersionChanged,
                        &format!("{name}.version"),
                    ));
                }
                ctx.db.auditor().write(
                    tx,
                    &Event::new(Some(actor.user_id), "edit", "charge_rule")
                        .entity_id(old.id)
                        .old(json!({ "to_date": old.to_date }))
                        .new_value(json!({ "to_date": yesterday }))
                        .into_entry(),
                )?;
                rule.from_date.clone_from(&today);
                rule.to_date = None;
                // Its version is above anything a stale form could still hold (the old row is
                // now at version + 1).
                let new_id = repo::insert_with_version(
                    tx,
                    &rule,
                    old.version.saturating_add(2),
                    actor.user_id,
                    &now,
                )?;
                ctx.db.auditor().write(
                    tx,
                    &Event::new(Some(actor.user_id), "create", "charge_rule")
                        .entity_id(new_id)
                        .old(row_json(&old))
                        .new_value(audit_json(&rule))
                        .into_entry(),
                )?;
            }
        }
        charges_in(tx, &today)
    })?;
    invalidate(ctx);
    Ok(saved)
}

// ---- Settings: overrides and checks ----

/// Reads the stored switches. A missing value means the starting position of PRD FR-11.10 v1.5
/// and FR-7.55: a typed commission or لاگا is allowed, a credit limit warns, and the rate check
/// is on at 30%. A value stored as `false` stays `false` (the Owner turned it off), and a stored
/// percent in range is kept as stored.
fn overrides_from(value: Option<Value>) -> OverridesSettings {
    let flag = |key: &str| {
        value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(Value::as_bool)
            .unwrap_or(true)
    };
    let mode = match value
        .as_ref()
        .and_then(|v| v.get("credit_limit_mode"))
        .and_then(Value::as_str)
    {
        Some("block") => CreditLimitMode::Block,
        _ => CreditLimitMode::Warn,
    };
    let percent = value
        .as_ref()
        .and_then(|v| v.get("rate_check_percent"))
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        // The range is checked when the Owner saves it; a value outside it can only come from
        // outside the app, so the starting position is used (money agent review, N1).
        .filter(|n| RATE_CHECK_PERCENT_RANGE.contains(n))
        .unwrap_or(DEFAULT_RATE_CHECK_PERCENT);
    OverridesSettings {
        allow_line_override: flag("allow_line_override"),
        credit_limit_mode: mode,
        rate_check_enabled: flag("rate_check_enabled"),
        rate_check_percent: percent,
    }
}

fn overrides_json(settings: &OverridesSettings) -> Value {
    json!({
        "allow_line_override": settings.allow_line_override,
        "credit_limit_mode": match settings.credit_limit_mode {
            CreditLimitMode::Warn => "warn",
            CreditLimitMode::Block => "block",
        },
        "rate_check_enabled": settings.rate_check_enabled,
        "rate_check_percent": settings.rate_check_percent,
    })
}

/// The switches of Section 21 "Overrides" and "Checks": a typed commission or لاگا on a line is
/// allowed, and a credit limit warns, until the Owner changes them (PRD FR-11.10 v1.5, A18).
///
/// # Errors
/// `ServiceError::Db`.
pub fn overrides(ctx: &Context) -> Result<OverridesSettings, ServiceError> {
    ctx.db
        .read(|conn| Ok(overrides_from(store::setting(conn, OVERRIDES_KEY)?)))
}

/// Saves the switches, with an audit row (old and new value). Asking for the values it has
/// writes nothing.
///
/// # Errors
/// `ERR_OUT_OF_RANGE` on `rate_check_percent` (values `min`, `max`) if it is not 1 to 1,000;
/// `ServiceError::Db`.
pub fn set_overrides(
    ctx: &Context,
    actor: &Actor,
    input: &OverridesSettings,
) -> Result<OverridesSettings, ServiceError> {
    if !RATE_CHECK_PERCENT_RANGE.contains(&input.rate_check_percent) {
        let bound = |n: u32| ErrorValue::Number(i64::from(n));
        return Err(ServiceError::Rejected(vec![Rejection {
            code: ErrorCode::ErrOutOfRange,
            field: Some("rate_check_percent".to_owned()),
            values: std::collections::BTreeMap::from([
                ("min".to_owned(), bound(*RATE_CHECK_PERCENT_RANGE.start())),
                ("max".to_owned(), bound(*RATE_CHECK_PERCENT_RANGE.end())),
            ]),
        }]));
    }
    ctx.db.write(|tx| {
        let old = overrides_from(store::setting(tx, OVERRIDES_KEY)?);
        if old != *input {
            store::set_setting(
                tx,
                OVERRIDES_KEY,
                &overrides_json(input),
                Some(actor.user_id),
            )?;
            ctx.db.auditor().write(
                tx,
                &Event::new(Some(actor.user_id), "edit", "settings")
                    .old(json!({ "overrides": overrides_json(&old) }))
                    .new_value(json!({ "overrides": overrides_json(input) }))
                    .into_entry(),
            )?;
        }
        Ok(*input)
    })
}
