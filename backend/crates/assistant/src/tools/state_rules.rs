//! `app_state{what:"rules"}`: which commission and charge rule wins, and the rules behind it
//! (PRD 11.1 to 11.3, plan 55 §11.1 R1).
//!
//! For a seller, a buyer, an item, an item group or none of them (the defaults), it looks up the
//! six allowed charges of PRD 11.2 on today's date with the very function a sale uses
//! (`domain::rules::resolve`, the one "test a rule" calls), and lists the active rules that can
//! apply to that subject, the most specific first. The party gives the seller side or the buyer side
//! its party level; the other side uses the item and the defaults. A value typed on the line is not
//! a rule and is not read here.
//!
//! The result is plain data with no second place. A percent is a `Val::Pct` (6% is `6.00`), an
//! amount for each نگ or each lot is money, a date is a date, and the words (`level`, `method`,
//! `charge`) are text the model reads and does not cite. A rule's `note` is typed by hand and is
//! never given (PRD FR-7.83).
//!
//! Example (worked day, no rule saved): `app_state{what:"rules"}` gives seller commission 6%,
//! buyer commission 6% and لاگا 8 for each نگ (the fruit group rule appears in `rules`), and no
//! rule for labour, munshiana and phone.

use mandi_services::assist;
use mandi_services::core::accounts::AccountType;
use mandi_services::core::masters::{MasterKind, MasterRow};
use mandi_services::core::money::Paisa;
use mandi_services::core::rules::{
    ChargeSide, ChargeType, ItemGroup, RuleLevel, RuleRow, RulesFilter,
};
use mandi_services::domain::rules::{Query, resolve};
use mandi_services::{masters, rules};
use serde_json::Value;

use super::party::type_word;
use super::records::named;
use super::state::on_screen;
use super::{Draft, Env, State, Stop, ToolError, service_stop};
use crate::answer::ScreenRoute;
use crate::results::{Money, NameVal, Node, Val};

/// The six charges PRD 11.2 allows, in the order of the "test a rule" box.
const CHARGES: [(ChargeSide, ChargeType); 6] = [
    (ChargeSide::Seller, ChargeType::Commission),
    (ChargeSide::Seller, ChargeType::Labour),
    (ChargeSide::Seller, ChargeType::Munshiana),
    (ChargeSide::Seller, ChargeType::Phone),
    (ChargeSide::Buyer, ChargeType::Commission),
    (ChargeSide::Buyer, ChargeType::Laga),
];

/// An item id no record has: with no item named, the item levels find nothing.
const NO_ITEM: i64 = 0;

/// Most rules listed.
const ROWS_CAP: usize = 12;

/// The key of one charge in `winners`.
fn charge_key(side: ChargeSide, charge: ChargeType) -> &'static str {
    match (side, charge) {
        (ChargeSide::Seller, ChargeType::Commission) => "seller_commission",
        (ChargeSide::Seller, ChargeType::Labour) => "seller_labour",
        (ChargeSide::Seller, ChargeType::Munshiana) => "seller_munshiana",
        (ChargeSide::Seller, ChargeType::Phone) => "seller_phone",
        (ChargeSide::Buyer, ChargeType::Commission) => "buyer_commission",
        (ChargeSide::Buyer, ChargeType::Laga) => "buyer_laga",
        _ => "other",
    }
}

/// The level of PRD 11.3 as a word.
fn level_word(level: RuleLevel) -> &'static str {
    match level {
        RuleLevel::LineOverride => "line",
        RuleLevel::PartyItem => "party_item",
        RuleLevel::Party => "party",
        RuleLevel::Item => "item",
        RuleLevel::ItemGroup => "item_group",
        RuleLevel::Default => "default",
    }
}

/// A group named by a word, in the words the Owner and the prompt use.
fn group_of_word(word: &str) -> Option<ItemGroup> {
    match word.trim().to_lowercase().as_str() {
        "fruit" | "fruits" | "phal" | "پھل" => Some(ItemGroup::Fruit),
        "vegetable" | "vegetables" | "sabzi" | "sabzian" | "سبزی" => Some(ItemGroup::Vegetable),
        _ => None,
    }
}

/// What the rules are asked for.
struct Subject {
    /// The party with his id and kind.
    party: Option<(i64, AccountType)>,
    /// The item's id.
    item_id: Option<i64>,
    /// The item's group, from the item or the word.
    group: Option<ItemGroup>,
}

/// The fields of the `subject` the result shows: the party, the item and the group.
type SubjectFields = Vec<(&'static str, Node)>;

fn bad(field: &str, detail: &str) -> Stop {
    Stop::Error(ToolError::new("ERR_FORMAT_INVALID", field, detail))
}

fn read_subject(
    env: &Env<'_>,
    state: &State,
    party: Option<&str>,
    item: Option<&str>,
) -> Result<(Subject, SubjectFields), Stop> {
    let mut shown = Vec::new();
    let mut subject = Subject {
        party: None,
        item_id: None,
        group: None,
    };
    if let Some(text) = party {
        let (id, record) = named(env, state, text, "party", &["a"])?;
        let card = assist::party_card(env.ctx, id).map_err(service_stop)?;
        if !matches!(card.account_type, AccountType::Seller | AccountType::Buyer) {
            return Err(bad(
                "party",
                "rules are for a seller or a buyer; this account is neither",
            ));
        }
        subject.party = Some((id, card.account_type));
        shown.push((
            "party",
            Node::obj(vec![
                (
                    "name",
                    Node::Val(Val::Name(NameVal {
                        name_ur: record.name_ur,
                        name_en: record.name_en,
                        tag: record.tag,
                        code: record.code,
                    })),
                ),
                ("type", Node::Str(type_word(card.account_type).to_owned())),
            ]),
        ));
    }
    if let Some(text) = item {
        if let Some(group) = group_of_word(text) {
            subject.group = Some(group);
        } else {
            let (id, record) = named(env, state, text, "item", &["i"])?;
            let group = masters::list(env.ctx, MasterKind::Item, true)
                .map_err(service_stop)?
                .into_iter()
                .find_map(|row| match row {
                    MasterRow::Item(item) if item.id == id => ItemGroup::parse(&item.item_group),
                    _ => None,
                });
            subject.item_id = Some(id);
            subject.group = group;
            shown.push((
                "item",
                Node::Val(Val::Name(NameVal {
                    name_ur: record.name_ur,
                    name_en: record.name_en,
                    tag: None,
                    code: record.code,
                })),
            ));
        }
    }
    if let Some(group) = subject.group {
        shown.push(("group", Node::Str(group.as_str().to_owned())));
    }
    Ok((subject, shown))
}

fn plain(paisa: Paisa) -> Node {
    Node::Val(Val::Money(Money::plain(paisa)))
}

/// A percent (hundredths of a percent, as basis points are) or an amount.
fn value_node(bps: Option<i32>, paisa: Option<Paisa>) -> Node {
    match (bps, paisa) {
        (Some(bps), _) => Node::Val(Val::Pct(i64::from(bps))),
        (None, Some(paisa)) => plain(paisa),
        (None, None) => Node::Null,
    }
}

fn date_node(iso: &str) -> Node {
    Node::Val(Val::Date(iso.to_owned()))
}

fn record_name(name: &mandi_services::core::accounts::RecordRef) -> Node {
    Node::Val(Val::Name(NameVal {
        name_ur: name.name_ur.clone(),
        name_en: name.name_en.clone(),
        tag: name.tag.clone(),
        code: (!name.code.is_empty()).then(|| name.code.clone()),
    }))
}

/// The winner of one charge.
fn winner_node(
    env: &Env<'_>,
    set: &mandi_services::domain::rules::RuleSet,
    subject: &Subject,
    all: &[RuleRow],
    side: ChargeSide,
    charge: ChargeType,
) -> Node {
    // The party gives only the side of his own kind a party level.
    let party_id = subject.party.and_then(|(id, kind)| match (side, kind) {
        (ChargeSide::Seller, AccountType::Seller) | (ChargeSide::Buyer, AccountType::Buyer) => {
            Some(id)
        }
        _ => None,
    });
    let resolution = resolve(
        set,
        &Query {
            side,
            charge,
            party_id,
            item_id: subject.item_id.unwrap_or(NO_ITEM),
            item_group: subject.group,
            date: env.today,
            line_override: None,
        },
    );
    let side_word = Node::Str(side.as_str().to_owned());
    let charge_word = Node::Str(charge.as_str().to_owned());
    let Some(winner) = resolution.winner else {
        return Node::obj(vec![
            ("side", side_word),
            ("charge", charge_word),
            ("found", Node::Bool(false)),
        ]);
    };
    let view = winner.view();
    let row = view.rule_id.and_then(|id| all.iter().find(|r| r.id == id));
    Node::obj(vec![
        ("side", side_word),
        ("charge", charge_word),
        ("found", Node::Bool(true)),
        ("level", Node::Str(level_word(view.level).to_owned())),
        ("method", Node::Str(view.method.as_str().to_owned())),
        ("value", value_node(view.value_bps, view.value_paisa)),
        ("from", row.map_or(Node::Null, |r| date_node(&r.from_date))),
        (
            "to",
            row.and_then(|r| r.to_date.as_deref())
                .map_or(Node::Null, date_node),
        ),
        (
            "rule",
            view.rule_id
                .map_or(Node::Null, |id| Node::Str(format!("ru{id}"))),
        ),
    ])
}

/// True when an active rule can apply to the subject (the most it can be is as specific as the
/// subject). With a party named and no item, all of his rules are listed; with nothing named, the
/// defaults and the group rules are.
fn relevant(row: &RuleRow, subject: &Subject) -> bool {
    let account_ok = match &row.account {
        None => true,
        Some(account) => subject.party.is_some_and(|(id, _)| id == account.id),
    };
    let item_ok = match &row.item {
        None => true,
        Some(item) => {
            subject.item_id == Some(item.id)
                || (subject.party.is_some() && subject.item_id.is_none() && row.account.is_some())
        }
    };
    let group_ok = match row.item_group {
        None => true,
        Some(group) => {
            subject.group == Some(group) || (subject.party.is_none() && subject.item_id.is_none())
        }
    };
    row.is_active && account_ok && item_ok && group_ok
}

fn rule_node(row: &RuleRow) -> Node {
    Node::obj(vec![
        ("side", Node::Str(row.side.as_str().to_owned())),
        ("charge", Node::Str(row.charge_type.as_str().to_owned())),
        ("level", Node::Str(level_word(row.level).to_owned())),
        (
            "party",
            row.account.as_ref().map_or(Node::Null, record_name),
        ),
        ("item", row.item.as_ref().map_or(Node::Null, record_name)),
        (
            "group",
            row.item_group
                .map_or(Node::Null, |g| Node::Str(g.as_str().to_owned())),
        ),
        ("method", Node::Str(row.method.as_str().to_owned())),
        ("value", value_node(row.value_bps, row.value_paisa)),
        ("from", date_node(&row.from_date)),
        ("to", row.to_date.as_deref().map_or(Node::Null, date_node)),
    ])
}

/// The tool, for `what: "rules"`.
pub(super) fn run(
    env: &Env<'_>,
    state: &State,
    args: &Value,
    party: Option<&str>,
    item: Option<&str>,
) -> Result<Draft, Stop> {
    let (subject, shown) = read_subject(env, state, party, item)?;
    let set = rules::rule_set(env.ctx).map_err(service_stop)?;
    let all = rules::list(
        env.ctx,
        &RulesFilter {
            side: None,
            charge_type: None,
            account_id: None,
            item_id: None,
            include_inactive: false,
        },
    )
    .map_err(service_stop)?;
    let winners = CHARGES
        .iter()
        .map(|(side, charge)| {
            (
                charge_key(*side, *charge).to_owned(),
                winner_node(env, &set, &subject, &all, *side, *charge),
            )
        })
        .collect();
    let applicable: Vec<&RuleRow> = all.iter().filter(|r| relevant(r, &subject)).collect();
    let total = applicable.len();
    let rows: Vec<(String, Node)> = applicable
        .into_iter()
        .take(ROWS_CAP)
        .map(|r| (format!("ru{}", r.id), rule_node(r)))
        .collect();
    let more = total > rows.len();
    let fields = vec![
        ("what", Node::Str("rules".to_owned())),
        ("as_of", date_node(&env.today.to_iso())),
        ("subject", Node::obj(shown)),
        ("winners", Node::Keyed(winners)),
        ("rules", Node::Keyed(rows)),
        (
            "count",
            Node::Val(Val::Count(i64::try_from(total).unwrap_or(i64::MAX))),
        ),
        ("more", Node::Bool(more)),
        (
            "note",
            Node::Str(
                "winners are the rule that decides today for this subject, most specific level \
                 first; a value typed on a sale line beats every rule; when `found` is false no \
                 rule applies and the amount is typed on the lot or is 0"
                    .to_owned(),
            ),
        ),
    ];
    Ok(on_screen(
        Draft::new("app_state", args.clone(), Node::obj(fields)),
        ScreenRoute::CommissionRules,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_word_is_known_in_the_words_the_owner_uses() {
        assert_eq!(group_of_word("fruit"), Some(ItemGroup::Fruit));
        assert_eq!(group_of_word(" Phal "), Some(ItemGroup::Fruit));
        assert_eq!(group_of_word("سبزی"), Some(ItemGroup::Vegetable));
        assert_eq!(group_of_word("a12"), None);
    }

    #[test]
    fn every_allowed_charge_has_its_own_key() {
        let mut keys: Vec<&str> = CHARGES.iter().map(|(s, c)| charge_key(*s, *c)).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), CHARGES.len());
        assert!(!keys.contains(&"other"));
    }
}
