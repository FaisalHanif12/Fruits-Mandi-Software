//! The `find` roles for the smaller master lists: `packing`, `marka`, `store` and `narration`
//! (plan 55 §11.1 gap Q5). They answer a figure ("kitni packing hain"), a list ("kaun kaun se
//! store hain") and "no such" (`count` 0) the way `staff` and `expense_head` do, in `find_lists`.
//!
//! | Role | Key | Each match gives |
//! |---|---|---|
//! | `packing` | `pk5` | code, name, active |
//! | `marka` | `mk2` | code, the owner seller's name; **never the mark text**; found by code, by mark, and by the owner's name |
//! | `store` | `sr1` | code, name, active |
//! | `narration` | `nr3` | the voucher kinds it is used in; **never its text** |
//!
//! A mark's text and a standard narration's text are typed by hand and are on the list of what
//! never leaves the computer (PRD FR-7.83). So a marka is told by its code and owner, and a
//! narration only by its count and where it is used. Only active records are read. The word `all`
//! (and the role's own word, such as `packing` or `godam`) lists everything; any other word is
//! searched by the pickers' smart search, which also finds a mark by its text (what the Owner typed
//! is not given back). A marka is also found by its OWNER's name ("rahim ke marke": the markas of
//! Rahim), which the pickers' search does not look at (hidden round 4, H391).
//!
//! Example (worked day): `find{text:"all", role:"packing"}` gives `count` 1 (پیٹی) and its match.

use mandi_services::core::masters::{MasterKind, MasterRow};
use mandi_services::core::search::SearchScope;
use mandi_services::core::text;
use mandi_services::{masters, search};

use super::find::Role;
use super::find_lists::{list_node, name};
use super::{Env, Stop, service_stop};
use crate::results::{NameVal, Node, Val};

/// Most matches listed. `count` and `more` say when there are more.
const LIST_CAP: usize = 12;

/// What a role reads.
struct Spec {
    kind: MasterKind,
    scope: SearchScope,
    /// The word of the role in the result.
    role: &'static str,
    /// Words that mean every record, besides `all`.
    own_words: &'static [&'static str],
}

fn spec(role: Role) -> Option<Spec> {
    Some(match role {
        Role::Packing => Spec {
            kind: MasterKind::Packing,
            scope: SearchScope::Packing,
            role: "packing",
            own_words: &["packing", "packings", "pack", "peti", "پیکنگ"],
        },
        Role::Marka => Spec {
            kind: MasterKind::Marka,
            scope: SearchScope::Marka,
            role: "marka",
            own_words: &["marka", "markay", "marke", "mark", "marks", "مارکہ"],
        },
        Role::Store => Spec {
            kind: MasterKind::Store,
            scope: SearchScope::Store,
            role: "store",
            own_words: &["store", "stores", "godam", "godown", "سٹور", "گودام"],
        },
        Role::Narration => Spec {
            kind: MasterKind::Narration,
            scope: SearchScope::Narration,
            role: "narration",
            own_words: &["narration", "narrations", "tafseel", "tafsil", "تفصیل"],
        },
        _ => return None,
    })
}

/// One match, with nothing typed by hand.
fn row_node(row: &MasterRow) -> Option<Node> {
    Some(match row {
        MasterRow::Packing(r) => Node::obj(vec![
            ("key", Node::Str(format!("pk{}", r.id))),
            ("code", Node::Str(r.code.clone())),
            ("name", name(&r.name_ur, &r.name_en, &r.code)),
            ("active", Node::Bool(r.is_active)),
        ]),
        MasterRow::Store(r) => Node::obj(vec![
            ("key", Node::Str(format!("sr{}", r.id))),
            ("code", Node::Str(r.code.clone())),
            ("name", name(&r.name_ur, &r.name_en, &r.code)),
            ("active", Node::Bool(r.is_active)),
        ]),
        // The mark text is left out (FR-7.83): the code and the owner tell one mark from another.
        MasterRow::Marka(r) => {
            let owner = match (&r.owner_name_ur, &r.owner_name_en) {
                (Some(ur), Some(en)) => Node::Val(Val::Name(NameVal {
                    name_ur: ur.clone(),
                    name_en: en.clone(),
                    tag: None,
                    code: r.owner_account_code.clone(),
                })),
                _ => Node::Null,
            };
            Node::obj(vec![
                ("key", Node::Str(format!("mk{}", r.id))),
                ("code", Node::Str(r.code.clone())),
                ("owner", owner),
                ("active", Node::Bool(r.is_active)),
            ])
        }
        // The text is left out (FR-7.83): only the voucher kinds it is offered for.
        MasterRow::Narration(r) => Node::obj(vec![
            ("key", Node::Str(format!("nr{}", r.id))),
            (
                "used_in",
                Node::List(r.used_in.iter().map(|k| Node::Str(k.clone())).collect()),
            ),
            ("active", Node::Bool(r.is_active)),
        ]),
        _ => return None,
    })
}

/// The result of `find` for one of the four roles.
///
/// # Errors
/// As the services it reads: a stop for the model.
pub(super) fn list(env: &Env<'_>, role: Role, query: &str) -> Result<Node, Stop> {
    let Some(spec) = spec(role) else {
        return Ok(list_node(role_word(role), query, Vec::new(), 0, Vec::new()));
    };
    let rows = masters::list(env.ctx, spec.kind, false).map_err(service_stop)?;
    let typed = query.trim().to_lowercase();
    let all = typed == "all"
        || spec.own_words.contains(&typed.as_str())
        || spec.own_words.contains(&query.trim());
    let found: Vec<&MasterRow> = if all {
        rows.iter().collect()
    } else {
        let hits =
            search::search(env.ctx, env.actor, spec.scope, query, 50).map_err(service_stop)?;
        let mut found: Vec<&MasterRow> = hits
            .iter()
            .filter_map(|hit| rows.iter().find(|r| r.id() == hit.id))
            .collect();
        // The markas of an owner: the search keys of a marka are its code and its mark, not the
        // name of the seller it belongs to.
        for row in rows.iter().filter(|r| owner_matches(r, query)) {
            if !found.iter().any(|f| f.id() == row.id()) {
                found.push(row);
            }
        }
        found
    };
    let total = found.len();
    let matches = found
        .iter()
        .take(LIST_CAP)
        .filter_map(|row| row_node(row))
        .collect();
    Ok(list_node(spec.role, query, matches, total, Vec::new()))
}

/// True when the row is a marka whose owner's name holds every word of the query, in either script
/// (look-alike Urdu letters and capitals do not matter). Words of one letter are left out.
///
/// Example: the query "rahim" matches the marka of "Rahim"; "rahim khan" does not match "Rahim".
fn owner_matches(row: &MasterRow, query: &str) -> bool {
    let MasterRow::Marka(r) = row else {
        return false;
    };
    let words = |key: String| -> Vec<String> {
        key.split(' ')
            .filter(|w| w.chars().count() > 1)
            .map(str::to_owned)
            .collect()
    };
    let (en, ur) = (words(text::search_en(query)), words(text::search_ur(query)));
    let holds = |owner: &Option<String>, wanted: &[String], key: &dyn Fn(&str) -> String| {
        let Some(owner) = owner else {
            return false;
        };
        let owner = key(owner);
        !wanted.is_empty() && wanted.iter().all(|w| owner.contains(w.as_str()))
    };
    holds(&r.owner_name_en, &en, &text::search_en) || holds(&r.owner_name_ur, &ur, &text::search_ur)
}

fn role_word(role: Role) -> &'static str {
    match role {
        Role::Packing => "packing",
        Role::Marka => "marka",
        Role::Store => "store",
        Role::Narration => "narration",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_of_the_four_roles_has_its_own_spec_and_no_other_role_has() {
        for role in [Role::Packing, Role::Marka, Role::Store, Role::Narration] {
            let s = spec(role).unwrap();
            assert_eq!(s.role, role_word(role));
            assert!(!s.own_words.is_empty());
        }
        assert!(spec(Role::Staff).is_none());
        assert!(spec(Role::Item).is_none());
    }
}
