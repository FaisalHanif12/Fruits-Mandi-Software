//! The tool `find`: people and things by code, short code, Urdu or English name, tag or city,
//! forgiving small mistakes (PRD FR-7.77, FR-7.40). It uses the smart search of the pickers.
//!
//! Up to 5 matches come back, each with its key, code, name and, for an account, its checked
//! balance (party card against R13, read now, not from the search index: Code Standards Q7). Phones,
//! addresses and father or firm names are never sent (PRD FR-7.83): the search hit's second line is
//! left out. No match gives an empty list; the search already includes the close spellings, so an
//! empty list means nothing is close.
//!
//! Roles: buyer, seller, any_party, pukhta, money_lent, separate_khata (accounts); item, city, adda,
//! collector, trip, garden (lists); partner (the partner section, so a locked section pauses the
//! question, PRD FR-7.82).

use mandi_services::assist;
use mandi_services::core::accounts::{AccountHit, AccountScope, AccountType, SearchAccountsInput};
use mandi_services::core::search::SearchScope;
use mandi_services::core::text;
use mandi_services::{coa, partners, search};
use serde::Deserialize;
use serde_json::Value;

use super::party::{card_name, checked_balance, type_word};
use super::{Draft, Env, Stop, ToolError, input, service_stop};
use crate::limits::MAX_OPTIONS;
use crate::results::{NameVal, Node, Val};

/// What is searched for. The names are the tool schema's enum, word for word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// A buyer account.
    Buyer,
    /// A seller account.
    Seller,
    /// Any account of a seller or a buyer.
    AnyParty,
    /// A پختہ کھاتہ person.
    Pukhta,
    /// An ادھار رقم person.
    MoneyLent,
    /// An الگ کھاتہ.
    SeparateKhata,
    /// A partner.
    Partner,
    /// A garden.
    Garden,
    /// A market trip.
    Trip,
    /// An item.
    Item,
    /// A city.
    City,
    /// An اڈا.
    Adda,
    /// A staff member who collects.
    Collector,
}

impl Role {
    fn word(self) -> &'static str {
        match self {
            Role::Buyer => "buyer",
            Role::Seller => "seller",
            Role::AnyParty => "any_party",
            Role::Pukhta => "pukhta",
            Role::MoneyLent => "money_lent",
            Role::SeparateKhata => "separate_khata",
            Role::Partner => "partner",
            Role::Garden => "garden",
            Role::Trip => "trip",
            Role::Item => "item",
            Role::City => "city",
            Role::Adda => "adda",
            Role::Collector => "collector",
        }
    }
}

/// The input of `find`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The word as the Owner wrote it.
    pub text: String,
    /// What is searched for.
    pub role: Role,
}

/// The most account hits asked of the search (the picker's own limit), before the role filter.
const ASK_ACCOUNTS: u32 = 50;
/// The most list hits asked of the search.
const ASK_LISTS: u32 = 50;
/// The code of the control of the الگ کھاتے (PRD FR-10.4).
const SEPARATE_KHATA_CONTROL: &str = "0305";

fn list_scope(role: Role) -> Option<(SearchScope, &'static str)> {
    match role {
        Role::Item => Some((SearchScope::Item, "i")),
        Role::City => Some((SearchScope::City, "ct")),
        Role::Adda => Some((SearchScope::Adda, "ad")),
        Role::Collector => Some((SearchScope::Staff, "st")),
        Role::Trip => Some((SearchScope::TripAll, "t")),
        Role::Garden => Some((SearchScope::GardenAll, "g")),
        _ => None,
    }
}

fn keep(role: Role, hit: &AccountHit) -> bool {
    match role {
        Role::Buyer => hit.account_type == AccountType::Buyer,
        Role::Seller => hit.account_type == AccountType::Seller,
        Role::AnyParty => matches!(hit.account_type, AccountType::Buyer | AccountType::Seller),
        Role::Pukhta => hit.is_pukhta_party,
        Role::MoneyLent => hit.account_type == AccountType::MoneyLent,
        _ => true,
    }
}

fn account_matches(
    env: &Env<'_>,
    role: Role,
    scope: AccountScope,
    control_id: Option<i64>,
    query: &str,
) -> Result<(Vec<Node>, bool), Stop> {
    let hits = search::search_accounts(
        env.ctx,
        env.actor,
        &SearchAccountsInput {
            scope,
            control_id,
            text: query.to_owned(),
            limit: ASK_ACCOUNTS,
            offset: 0,
            include_dead: Some(true),
        },
    )
    .map_err(service_stop)?;
    let hits: Vec<AccountHit> = hits.into_iter().filter(|h| keep(role, h)).collect();
    let more = hits.len() > MAX_OPTIONS;
    let mut matches = Vec::new();
    for hit in hits.into_iter().take(MAX_OPTIONS) {
        let card = assist::party_card(env.ctx, hit.id).map_err(service_stop)?;
        let balance = checked_balance(env, &card)?;
        matches.push(Node::obj(vec![
            ("key", Node::Str(format!("a{}", card.id))),
            ("code", Node::Str(card.code.clone())),
            ("type", Node::Str(type_word(card.account_type).to_owned())),
            ("name", Node::Val(Val::Name(card_name(&card)))),
            (
                "city",
                Node::Val(Val::Name(NameVal {
                    name_ur: card.city_ur.clone(),
                    name_en: card.city_en.clone(),
                    tag: None,
                    code: None,
                })),
            ),
            ("dead", Node::Bool(card.dead.is_some())),
            ("active", Node::Bool(card.is_active)),
            ("balance", Node::Val(Val::Money(balance))),
        ]));
    }
    Ok((matches, more))
}

fn list_matches(
    env: &Env<'_>,
    scope: SearchScope,
    prefix: &str,
    query: &str,
) -> Result<(Vec<Node>, bool), Stop> {
    let hits = search::search(env.ctx, env.actor, scope, query, ASK_LISTS).map_err(service_stop)?;
    let more = hits.len() > MAX_OPTIONS;
    let matches = hits
        .into_iter()
        .take(MAX_OPTIONS)
        .map(|hit| {
            Node::obj(vec![
                ("key", Node::Str(format!("{prefix}{}", hit.id))),
                ("code", Node::Str(hit.code.clone())),
                (
                    "name",
                    Node::Val(Val::Name(NameVal {
                        name_ur: hit.name_ur,
                        name_en: hit.name_en,
                        tag: None,
                        code: Some(hit.code),
                    })),
                ),
                ("active", Node::Bool(hit.is_active)),
            ])
        })
        .collect();
    Ok((matches, more))
}

/// Partners are few (at most 10): the typed word is matched against both names.
fn partner_matches(env: &Env<'_>, query: &str) -> Result<(Vec<Node>, bool), Stop> {
    if env.partner_locked {
        return Err(Stop::Locked);
    }
    let rows = partners::list(env.ctx).map_err(service_stop)?;
    let ur = text::search_ur(query);
    let en = text::search_en(query);
    let hit = |row: &mandi_services::core::partners::PartnerRow| {
        (!ur.is_empty() && text::search_ur(&row.name_ur).contains(&ur))
            || (!en.is_empty() && text::search_en(&row.name_en).contains(&en))
    };
    let found: Vec<_> = rows.iter().filter(|row| hit(row)).collect();
    let more = found.len() > MAX_OPTIONS;
    let matches = found
        .into_iter()
        .take(MAX_OPTIONS)
        .map(|row| {
            Node::obj(vec![
                ("key", Node::Str(format!("s{}", row.id))),
                (
                    "name",
                    Node::Val(Val::Name(NameVal {
                        name_ur: row.name_ur.clone(),
                        name_en: row.name_en.clone(),
                        tag: None,
                        code: None,
                    })),
                ),
                ("active", Node::Bool(row.is_active)),
            ])
        })
        .collect();
    Ok((matches, more))
}

/// The tool.
pub fn run(env: &Env<'_>, _state: &super::State, args: &Value) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let query = parsed.text.trim();
    if query.is_empty() {
        return Err(ToolError::new("ERR_REQUIRED", "text", "nothing to find").into());
    }
    let role = parsed.role;
    let (matches, more) = match role {
        Role::Buyer => account_matches(env, role, AccountScope::Buyer, None, query)?,
        Role::Seller => account_matches(env, role, AccountScope::Seller, None, query)?,
        Role::AnyParty | Role::Pukhta | Role::MoneyLent => {
            account_matches(env, role, AccountScope::Any, None, query)?
        }
        Role::SeparateKhata => {
            let control = coa::controls(env.ctx)
                .map_err(service_stop)?
                .into_iter()
                .find(|c| c.code == SEPARATE_KHATA_CONTROL);
            match control {
                Some(c) => account_matches(env, role, AccountScope::Control, Some(c.id), query)?,
                None => (Vec::new(), false),
            }
        }
        Role::Partner => partner_matches(env, query)?,
        Role::Item | Role::City | Role::Adda | Role::Collector | Role::Trip | Role::Garden => {
            let Some((scope, prefix)) = list_scope(role) else {
                return Err(ToolError::new("ERR_OUT_OF_RANGE", "role", "unknown role").into());
            };
            list_matches(env, scope, prefix, query)?
        }
    };
    let node = Node::obj(vec![
        ("text", Node::Str(query.to_owned())),
        ("role", Node::Str(role.word().to_owned())),
        ("matches", Node::List(matches)),
        ("more", Node::Bool(more)),
    ]);
    Ok(Draft::new("find", args.clone(), node))
}
