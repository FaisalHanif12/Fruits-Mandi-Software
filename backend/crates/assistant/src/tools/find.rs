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

/// The words of a name as normalised keys, in both scripts: each word, and the whole name.
fn name_words(name_ur: &str, name_en: &str) -> Vec<String> {
    let (ur, en) = (text::search_ur(name_ur), text::search_en(name_en));
    let mut out: Vec<String> = ur
        .split(' ')
        .chain(en.split(' '))
        .map(str::to_owned)
        .collect();
    out.push(ur);
    out.push(en);
    out.retain(|w| !w.is_empty());
    out
}

/// Partners are few (at most 10). The typed word is matched against both names in two tiers: a
/// partner whose name, or one word of it, IS the typed word comes first and alone ("A" is Partner
/// A, "alif" is الف); only when none is, a partner whose name merely contains it (plan 52 item 3).
/// Without the tiers "a" would match every "Partner".
fn partner_matches(env: &Env<'_>, query: &str) -> Result<(Vec<Node>, bool), Stop> {
    if env.partner_locked {
        return Err(Stop::Locked);
    }
    let rows = partners::list(env.ctx).map_err(service_stop)?;
    let ur = text::search_ur(query);
    let en = text::search_en(query);
    let typed = [ur.as_str(), en.as_str()];
    let is_word = |row: &mandi_services::core::partners::PartnerRow| {
        name_words(&row.name_ur, &row.name_en)
            .iter()
            .any(|w| typed.iter().any(|t| !t.is_empty() && w == t))
    };
    let contains = |row: &mandi_services::core::partners::PartnerRow| {
        (!ur.is_empty() && text::search_ur(&row.name_ur).contains(&ur))
            || (!en.is_empty() && text::search_en(&row.name_en).contains(&en))
    };
    let mut found: Vec<_> = rows.iter().filter(|row| is_word(row)).collect();
    if found.is_empty() {
        found = rows.iter().filter(|row| contains(row)).collect();
    }
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

/// The matches of one word in one role.
fn matches_for(env: &Env<'_>, role: Role, query: &str) -> Result<(Vec<Node>, bool), Stop> {
    Ok(match role {
        Role::Buyer => account_matches(env, role, AccountScope::Buyer, None, query)?,
        Role::Seller => account_matches(env, role, AccountScope::Seller, None, query)?,
        // A buyer or a seller: both lists are asked for, so income, expense and other accounts
        // cannot fill the 50 hits of one search before the type is filtered (money review F6b).
        Role::AnyParty => {
            let (mut buyers, buyers_more) =
                account_matches(env, role, AccountScope::Buyer, None, query)?;
            let (sellers, sellers_more) =
                account_matches(env, role, AccountScope::Seller, None, query)?;
            buyers.extend(sellers);
            let more = buyers_more || sellers_more || buyers.len() > MAX_OPTIONS;
            buyers.truncate(MAX_OPTIONS);
            (buyers, more)
        }
        Role::Pukhta | Role::MoneyLent => {
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
    })
}

/// True when the Urdu name of a match has `wanted` (a normalised Urdu spelling) as a word or as its
/// start.
fn names_start_with(found: &Node, wanted: &str) -> bool {
    let Some(Node::Val(Val::Name(name))) = found.get(&["name"]) else {
        return false;
    };
    let ur = text::search_ur(&name.name_ur);
    !wanted.is_empty() && (ur.starts_with(wanted) || ur.split(' ').any(|w| w.starts_with(wanted)))
}

/// True when the word is typed in English letters, with at least one letter.
fn is_roman(query: &str) -> bool {
    query.chars().any(|c| c.is_ascii_alphabetic())
        && !query
            .chars()
            .any(mandi_services::domain::core::text::is_urdu_letter)
}

/// The source word of a match found through the assistant's own item aliases.
const ALIAS_SOURCE: &str = "alias";

/// Roman spellings of an item that the typing word list does not carry, each with its usual
/// spelling (plan 52 item 3, the lead's ruling of 03-11-2026: kept here and not in
/// `translit_seed.tsv`, whose measured accuracy must not drop).
const ITEM_ALIASES: [(&str, &str); 3] = [
    ("mausami", "mosambi"),
    ("mosami", "mosambi"),
    ("musambi", "mosambi"),
];

/// The usual spelling of an item typed in a spelling of the alias list, if it is one.
fn item_alias(query: &str) -> Option<&'static str> {
    let typed = query.trim().to_ascii_lowercase();
    ITEM_ALIASES
        .iter()
        .find(|(alias, _)| *alias == typed)
        .map(|(_, usual)| *usual)
}

/// The matches of an Urdu spelling: only a record whose Urdu name has the spelling as a word or a
/// start, because the search also returns near spellings ("کریم" finds اکرم), which are not what
/// was typed. `None` when there is none.
fn urdu_matches(
    env: &Env<'_>,
    role: Role,
    spelling: &str,
) -> Result<Option<(Vec<Node>, bool)>, Stop> {
    let (found, more) = matches_for(env, role, spelling)?;
    let wanted = text::search_ur(spelling);
    let found: Vec<Node> = found
        .into_iter()
        .filter(|m| names_start_with(m, &wanted))
        .collect();
    Ok(if found.is_empty() {
        None
    } else {
        Some((found, more))
    })
}

/// The tool.
pub fn run(env: &Env<'_>, _state: &super::State, args: &Value) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let query = parsed.text.trim();
    if query.is_empty() {
        return Err(ToolError::new("ERR_REQUIRED", "text", "nothing to find").into());
    }
    let role = parsed.role;
    let (mut matches, mut more) = matches_for(env, role, query)?;
    // Nothing found for a word in English letters: a Roman Urdu spelling of an Urdu name ("kela",
    // "saib", "alif") is tried in Urdu letters, by the same word list and learned pairs as the
    // typing help (plan 52 item 3). Raw hits always win; this is only for an empty answer.
    let mut via: Option<(&'static str, String)> = None;
    if matches.is_empty() && is_roman(query) {
        let suggestions = mandi_services::translit::suggest(
            env.ctx,
            query,
            mandi_services::core::translit::Direction::ToUrdu,
        )
        .unwrap_or_default();
        for suggestion in suggestions {
            if let Some((found, found_more)) = urdu_matches(env, role, &suggestion.text)? {
                matches = found;
                more = found_more;
                via = Some((suggestion.source.as_str(), suggestion.text));
                break;
            }
        }
        // An item the word list does not carry under that spelling: its usual spelling, by the
        // assistant's own short list (not the typing word list, whose accuracy is measured).
        if matches.is_empty()
            && role == Role::Item
            && let Some(usual) = item_alias(query)
        {
            let (found, found_more) = matches_for(env, role, usual)?;
            if found.is_empty() {
                let suggestions = mandi_services::translit::suggest(
                    env.ctx,
                    usual,
                    mandi_services::core::translit::Direction::ToUrdu,
                )
                .unwrap_or_default();
                for suggestion in suggestions {
                    if let Some((found, found_more)) = urdu_matches(env, role, &suggestion.text)? {
                        matches = found;
                        more = found_more;
                        via = Some((ALIAS_SOURCE, suggestion.text));
                        break;
                    }
                }
            } else {
                matches = found;
                more = found_more;
                via = Some((ALIAS_SOURCE, usual.to_owned()));
            }
        }
    }
    let mut fields = vec![
        ("text", Node::Str(query.to_owned())),
        ("role", Node::Str(role.word().to_owned())),
        ("matches", Node::List(matches)),
        ("more", Node::Bool(more)),
    ];
    if let Some((source, _)) = &via {
        fields.push(("via", Node::Str((*source).to_owned())));
        if *source == "rules" {
            fields.push((
                "note",
                Node::Str(
                    "found through the spelling rules only, not the word list: check that this \
                     is the one asked for"
                        .to_owned(),
                ),
            ));
        } else if *source == ALIAS_SOURCE {
            fields.push((
                "note",
                Node::Str(
                    "the word was not found as typed; it was found as its usual spelling"
                        .to_owned(),
                ),
            ));
        } else {
            fields.push((
                "note",
                Node::Str(
                    "the word was not found as typed; it was found as its Urdu spelling".to_owned(),
                ),
            ));
        }
    }
    Ok(Draft::new("find", args.clone(), Node::obj(fields)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_item_aliases_give_the_usual_spelling_in_any_case() {
        for typed in ["mausami", "Mausami", " MOSAMI ", "musambi"] {
            assert_eq!(item_alias(typed), Some("mosambi"), "{typed}");
        }
        // The usual spelling and other words are not aliases.
        for typed in ["mosambi", "kela", "", "mausam"] {
            assert_eq!(item_alias(typed), None, "{typed}");
        }
    }
}
