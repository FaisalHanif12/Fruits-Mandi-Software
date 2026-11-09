//! The tool `find`: people and things by code, short code, Urdu or English name, tag or city,
//! forgiving small mistakes (PRD FR-7.77, FR-7.40). It uses the smart search of the pickers.
//!
//! Up to 5 matches come back, each with its key, code, name and, for an account, its checked
//! balance (party card against R13, read now, not from the search index: Code Standards Q7). Phones,
//! addresses and father or firm names are never sent (PRD FR-7.83): the search hit's second line is
//! left out. No match gives an empty list; the search already includes the close spellings, so an
//! empty list means nothing is close.
//!
//! Roles: buyer, seller, any_party, chukta, money_lent, separate_khata (accounts); item, city, adda,
//! collector, trip, garden (lists); partner (the partner section, so a locked section pauses the
//! question, PRD FR-7.82).

use mandi_services::assist;
use mandi_services::core::accounts::{
    AccountHit, AccountScope, AccountType, HitKind, SearchAccountsInput,
};
use mandi_services::core::chukta::ChuktaSearchInput;
use mandi_services::core::search::SearchScope;
use mandi_services::core::text;
use mandi_services::{chukta, coa, partners, search};
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
    /// A walk-in buyer of the چکتا کھاتہ (PRD FR-7.29, Section 13). He has no ledger and no balance.
    Chukta,
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
    /// The staff: a name, or all of one job by its word (munshi, collector, labour), with a count
    /// (plan 55 R5).
    Staff,
    /// An expense head, or all of them, with a count (plan 55 R5).
    ExpenseHead,
    /// The بکری of a truck number (plan 55 R6).
    Truck,
    /// The packings, with a count (plan 55 gap Q5).
    Packing,
    /// The marks, with a count: code and owner, never the mark text (plan 55 gap Q5).
    Marka,
    /// The stores, with a count (plan 55 gap Q5).
    Store,
    /// The standard narrations, with a count: never their text (plan 55 gap Q5).
    Narration,
}

impl Role {
    fn word(self) -> &'static str {
        match self {
            Role::Buyer => "buyer",
            Role::Seller => "seller",
            Role::AnyParty => "any_party",
            Role::Chukta => "chukta",
            Role::MoneyLent => "money_lent",
            Role::SeparateKhata => "separate_khata",
            Role::Partner => "partner",
            Role::Garden => "garden",
            Role::Trip => "trip",
            Role::Item => "item",
            Role::City => "city",
            Role::Adda => "adda",
            Role::Collector => "collector",
            Role::Staff => "staff",
            Role::ExpenseHead => "expense_head",
            Role::Truck => "truck",
            Role::Packing => "packing",
            Role::Marka => "marka",
            Role::Store => "store",
            Role::Narration => "narration",
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
        // A چکتا buyer rides in the buyer list of the pickers, but he is no account and has no
        // balance: he is found only by the role `chukta` (PRD FR-13.6).
        _ if hit.kind == HitKind::Chukta => false,
        Role::Buyer => hit.account_type == AccountType::Buyer,
        Role::Seller => hit.account_type == AccountType::Seller,
        Role::AnyParty => matches!(hit.account_type, AccountType::Buyer | AccountType::Seller),
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
            with_chukta: None,
        },
    )
    .map_err(service_stop)?;
    let hits: Vec<AccountHit> = hits.into_iter().filter(|h| keep(role, h)).collect();
    // The name typed IS an account's name: no close spelling beside it (E2E-52: the names that start
    // with it still stay, the Owner has to say which Akram).
    let exact = Exact::of(query);
    let hits = exact.without_close(
        hits,
        |h| exact.is(&h.name_ur, &h.name_en, h.tag.as_deref()),
        |h| exact.holds(&h.name_ur, &h.name_en, h.tag.as_deref()),
    );
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

/// The walk-in buyers whose name or phone hold the typed text, newest first. A match is his key
/// (`c` and his row), his type `chukta` and his name; he has no code, no city and no balance, and his
/// phone is never sent (PRD FR-7.83, FR-13.6).
fn chukta_matches(env: &Env<'_>, query: &str) -> Result<(Vec<Node>, bool), Stop> {
    let found = chukta::search(
        env.ctx,
        &ChuktaSearchInput {
            text: query.to_owned(),
        },
    )
    .map_err(service_stop)?;
    let more = found.len() > MAX_OPTIONS;
    let matches = found
        .into_iter()
        .take(MAX_OPTIONS)
        .map(|b| {
            Node::obj(vec![
                ("key", Node::Str(format!("c{}", b.id))),
                ("code", Node::Str(String::new())),
                ("type", Node::Str("chukta".to_owned())),
                (
                    "name",
                    Node::Val(Val::Name(NameVal {
                        name_ur: b.name_ur,
                        name_en: b.name_en,
                        tag: None,
                        code: None,
                    })),
                ),
                ("active", Node::Bool(true)),
            ])
        })
        .collect();
    Ok((matches, more))
}

fn list_matches(
    env: &Env<'_>,
    scope: SearchScope,
    prefix: &str,
    query: &str,
) -> Result<(Vec<Node>, bool), Stop> {
    let hits = search::search(env.ctx, env.actor, scope, query, ASK_LISTS).map_err(service_stop)?;
    // "tomato" is Tomato, not Tomato and Potato: no close spelling beside an exact name.
    let exact = Exact::of(query);
    let hits = exact.without_close(
        hits,
        |h| exact.is(&h.name_ur, &h.name_en, None),
        |h| exact.holds(&h.name_ur, &h.name_en, None),
    );
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

/// Several accounts came back, and ONE is clear: its name is the typed name, or its name has the
/// typed phonetic key ("kareem" is "krm", as Karim), while every other is only a close spelling
/// (neither its name nor a word of it is the typed text). Then that one is the answer and the
/// Owner is not asked which (live run 2, F5: "karim" and "kareem" each asked "which one" beside a
/// close spelling). Two that are equally clear, or a name that starts with the typed text, still
/// ask. It works on the merged matches, so a buyer and a seller of `any_party` are weighed
/// together (each scope was cut apart before: a close buyer stayed beside the exact seller).
///
/// Example: "kareem" finds Karim (key `krm`) and Kareen (a close spelling): Karim alone.
fn prefer_the_clear_one(matches: Vec<Node>, query: &str) -> Vec<Node> {
    if matches.len() < 2 {
        return matches;
    }
    let exact = Exact::of(query);
    let typed_key = text::search_phonetic(query);
    let latin_key = !typed_key.is_empty() && typed_key.is_ascii();
    let name_of = |m: &Node| match m.get(&["name"]) {
        Some(Node::Val(Val::Name(n))) => Some(n.clone()),
        _ => None,
    };
    let clear = |m: &Node| {
        name_of(m).is_some_and(|n| {
            exact.is(&n.name_ur, &n.name_en, n.tag.as_deref())
                || (latin_key && text::search_phonetic(&n.name_en) == typed_key)
        })
    };
    let only_close = |m: &Node| {
        !clear(m)
            && name_of(m).is_some_and(|n| !exact.holds(&n.name_ur, &n.name_en, n.tag.as_deref()))
    };
    let clear_count = matches.iter().filter(|m| clear(m)).count();
    if clear_count == 1 && matches.iter().all(|m| clear(m) || only_close(m)) {
        return matches.into_iter().filter(|m| clear(m)).collect();
    }
    matches
}

/// True when one of the matches (as `find` writes them) is named exactly the typed text.
fn names_the_typed_text(matches: &[Node], exact: &Exact) -> bool {
    matches.iter().any(|m| {
        matches!(
            m.get(&["name"]),
            Some(Node::Val(Val::Name(n))) if exact.is(&n.name_ur, &n.name_en, n.tag.as_deref())
        )
    })
}

/// True when one of the matches names the typed text, as a whole or as words of it.
fn holds_the_typed_text(matches: &[Node], exact: &Exact) -> bool {
    matches.iter().any(|m| {
        matches!(
            m.get(&["name"]),
            Some(Node::Val(Val::Name(n))) if exact.holds(&n.name_ur, &n.name_en, n.tag.as_deref())
        )
    })
}

/// The text typed, as the two keys a name is compared with (`core::text` normalisation: look-alike
/// Urdu letters, case, spaces). A name is exact when its whole key is the whole typed key; it holds
/// the typed text when the typed words start it or are whole words of it.
///
/// Example: "Tomato" and "tomato" are exact for the item Tomato, not for Potato; "Akram Toll Tax"
/// holds "akram" without being exact; "tmatar" is exact for nothing, so its close spellings stay.
struct Exact {
    en: String,
    ur: String,
}

impl Exact {
    fn of(query: &str) -> Self {
        Self {
            en: text::search_en(query),
            ur: text::search_ur(query),
        }
    }

    /// The keys of a name, with and without its tag, each with the typed key of its script.
    fn keys(&self, name_ur: &str, name_en: &str, tag: Option<&str>) -> Vec<(String, String)> {
        let mut out = vec![
            (self.en.clone(), text::search_en(name_en)),
            (self.ur.clone(), text::search_ur(name_ur)),
        ];
        if tag.is_some() {
            out.push((self.en.clone(), text::search_en_tagged(name_en, tag)));
            out.push((self.ur.clone(), text::search_ur_tagged(name_ur, tag)));
        }
        out.retain(|(typed, _)| !typed.is_empty());
        out
    }

    /// True when the typed text IS this name, with or without its tag.
    fn is(&self, name_ur: &str, name_en: &str, tag: Option<&str>) -> bool {
        self.keys(name_ur, name_en, tag)
            .iter()
            .any(|(typed, key)| typed == key)
    }

    /// True when the typed words start this name or are whole words of it ("akram" in "akram toll
    /// tax"): a name the Owner may mean, unlike a close spelling ("potato" for "tomato").
    fn holds(&self, name_ur: &str, name_en: &str, tag: Option<&str>) -> bool {
        self.keys(name_ur, name_en, tag).iter().any(|(typed, key)| {
            key.starts_with(typed.as_str()) || format!(" {key} ").contains(&format!(" {typed} "))
        })
    }

    /// When any hit is exact, the exact hits and those that hold the typed text, all of them (two
    /// accounts with the same name both stay); the close spellings go. When none is exact, every hit.
    fn without_close<T>(
        &self,
        hits: Vec<T>,
        is_exact: impl Fn(&T) -> bool,
        holds: impl Fn(&T) -> bool,
    ) -> Vec<T> {
        if hits.iter().any(&is_exact) {
            hits.into_iter()
                .filter(|h| is_exact(h) || holds(h))
                .collect()
        } else {
            hits
        }
    }
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
        Role::Chukta => chukta_matches(env, query)?,
        Role::MoneyLent => account_matches(env, role, AccountScope::Any, None, query)?,
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
        // The lists with a count are read by `find_lists`, before this function is reached.
        Role::Staff
        | Role::ExpenseHead
        | Role::Truck
        | Role::Packing
        | Role::Marka
        | Role::Store
        | Role::Narration => (Vec::new(), false),
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
pub(super) fn item_alias(query: &str) -> Option<&'static str> {
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

/// What one search of one role found: the matches, whether there are more, and how a word typed in
/// English letters was found as an Urdu spelling or an item alias.
struct Found {
    matches: Vec<Node>,
    more: bool,
    via: Option<(&'static str, String)>,
}

/// The matches of a word in one role: as typed, then, for an empty answer, as the Urdu spelling of
/// a Roman Urdu word (the same word list and learned pairs as the typing help), then, for an item,
/// by the assistant's own aliases.
fn search_role(env: &Env<'_>, role: Role, query: &str) -> Result<Found, Stop> {
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
    Ok(Found { matches, more, via })
}

/// The words that only say what a garden or a trip is, in Roman Urdu, Urdu letters and English:
/// "shahpur bag" is the garden "Shahpur" (Q209 of the live run of 03-11-2026).
fn role_words(role: Role) -> &'static [&'static str] {
    match role {
        Role::Garden => &[
            "bag",
            "bagh",
            "baag",
            "baagh",
            "garden",
            "gardens",
            "باغ",
            "باغات",
            "بیگ",
        ],
        Role::Trip => &["safar", "trip", "trips", "سفر"],
        _ => &[],
    }
}

/// The query without the words that only name the kind of record (`role_words`). Empty when the
/// query is nothing but such words.
fn without_role_words(role: Role, query: &str) -> String {
    let words = role_words(role);
    query
        .split_whitespace()
        .filter(|w| !words.contains(&w.to_lowercase().as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The other roles asked when a buyer, a seller or any party is not found: a buyer typed as a
/// seller and the reverse (Q070: two buyers asked for as sellers), and a partner typed as a party
/// (Q055: "alif ka khalis"), also when the model asked for him as a buyer or a seller (live run 2,
/// Q3827: "Alif ne kal zaati kharche ke liye kitna liya" ended as "no such buyer", and the partner
/// had taken 0). Their matches say their `type`, and the result says it was widened.
fn widened_roles(role: Role) -> &'static [Role] {
    match role {
        Role::Buyer => &[Role::Seller, Role::Partner],
        Role::Seller => &[Role::Buyer, Role::Partner],
        Role::AnyParty => &[Role::Partner],
        _ => &[],
    }
}

/// The tool.
pub fn run(env: &Env<'_>, _state: &super::State, args: &Value) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let query = parsed.text.trim();
    if query.is_empty() {
        return Err(ToolError::new("ERR_REQUIRED", "text", "nothing to find").into());
    }
    let role = parsed.role;
    // Staff, expense heads and trucks are short lists with a count (plan 55 R5, R6).
    if let Some(draft) = super::find_lists::run(env, role, query, args)? {
        return Ok(draft);
    }
    let mut found = search_role(env, role, query)?;
    if matches!(role, Role::Buyer | Role::Seller | Role::AnyParty) {
        found.matches = prefer_the_clear_one(std::mem::take(&mut found.matches), query);
    }
    let mut notes: Vec<String> = Vec::new();
    // "shahpur bag" / "safar 2": a garden or a trip is found by the rest of the words.
    if found.matches.is_empty() {
        let rest = without_role_words(role, query);
        if !rest.is_empty() && rest != query {
            let again = search_role(env, role, &rest)?;
            if !again.matches.is_empty() {
                found = again;
                notes.push(format!(
                    "found as \"{rest}\", without the word for the kind"
                ));
            }
        }
    }
    let mut widened: Option<Role> = None;
    // The text is exactly a partner's name and the buyers or sellers it found are only close
    // spellings of it (live run 2, Q3849: "Alif" as a buyer found Kashif Ali): it is the partner.
    if matches!(role, Role::Buyer | Role::Seller | Role::AnyParty) && !found.matches.is_empty() {
        let exact = Exact::of(query);
        if !holds_the_typed_text(&found.matches, &exact) {
            match search_role(env, Role::Partner, query) {
                Ok(partners) if names_the_typed_text(&partners.matches, &exact) => {
                    found = partners;
                    widened = Some(Role::Partner);
                    notes.push(format!(
                        "this name is a partner's; no {} has it, only close spellings, which are \
                         left out",
                        role.word()
                    ));
                }
                Ok(_) | Err(Stop::Locked) => {}
                Err(other_error) => return Err(other_error),
            }
        }
    }
    // Nothing as that role: the other role of a party, or a partner, is asked. A locked partner
    // section is not a reason to stop the question here: the widening is left out.
    if found.matches.is_empty() {
        for other in widened_roles(role) {
            match search_role(env, *other, query) {
                Ok(again) if !again.matches.is_empty() => {
                    found = again;
                    widened = Some(*other);
                    notes.push(format!(
                        "no {} has this name; these are found as {}, see each match's type",
                        role.word(),
                        other.word()
                    ));
                    break;
                }
                Ok(_) | Err(Stop::Locked) => {}
                Err(other_error) => return Err(other_error),
            }
        }
    }
    let Found { matches, more, via } = found;
    let mut fields = vec![
        ("text", Node::Str(query.to_owned())),
        ("role", Node::Str(role.word().to_owned())),
        ("matches", Node::List(matches)),
        ("more", Node::Bool(more)),
    ];
    if let Some(other) = widened {
        fields.push(("widened", Node::Str(other.word().to_owned())));
    }
    if let Some((source, _)) = &via {
        fields.push(("via", Node::Str((*source).to_owned())));
        notes.push(
            if *source == "rules" {
                "found through the spelling rules only, not the word list: check that this is the \
                 one asked for"
            } else if *source == ALIAS_SOURCE {
                "the word was not found as typed; it was found as its usual spelling"
            } else {
                "the word was not found as typed; it was found as its Urdu spelling"
            }
            .to_owned(),
        );
    }
    if !notes.is_empty() {
        fields.push(("note", Node::Str(notes.join("; "))));
    }
    Ok(Draft::new("find", args.clone(), Node::obj(fields)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_words_for_the_kind_of_record_are_left_out_of_a_garden_or_trip_name() {
        assert_eq!(without_role_words(Role::Garden, "shahpur bag"), "shahpur");
        assert_eq!(
            without_role_words(Role::Garden, "Shahpur Garden"),
            "Shahpur"
        );
        assert_eq!(without_role_words(Role::Garden, "شاہ پور باغ"), "شاہ پور");
        assert_eq!(without_role_words(Role::Trip, "safar 2"), "2");
        // Nothing but the kind word: nothing is left.
        assert_eq!(without_role_words(Role::Garden, "bagh"), "");
        // Other roles keep their words.
        assert_eq!(without_role_words(Role::Item, "bag"), "bag");
        assert_eq!(without_role_words(Role::Buyer, "akram bagh"), "akram bagh");
    }

    #[test]
    fn a_buyer_or_seller_not_found_widens_to_the_other_role_and_any_party_to_a_partner() {
        assert_eq!(widened_roles(Role::Buyer), [Role::Seller, Role::Partner]);
        assert_eq!(widened_roles(Role::Seller), [Role::Buyer, Role::Partner]);
        assert_eq!(widened_roles(Role::AnyParty), [Role::Partner]);
        assert!(widened_roles(Role::Item).is_empty());
        assert!(widened_roles(Role::Partner).is_empty());
        assert!(widened_roles(Role::Garden).is_empty());
    }

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
