//! The `find` roles that read a short list and count it (plan 55 §11.1, R5 and R6).
//!
//! | Role | Finds | Key |
//! |---|---|---|
//! | `staff` | a staff member by name, or all of one job by its word (munshi, collector, labour), or all | `st5` |
//! | `expense_head` | an expense head by name, or all of them | `eh3` |
//! | `truck` | the بکری of a truck number, newest first | `bk12` |
//!
//! The result keeps the shape of every `find` result (`text`, `role`, `matches`, `more`) and adds
//! `count`: how many there are in all, which can be more than the matches listed. A job word
//! ("kitne munshi hain") gives all the staff of that job; `by_job` then counts each job. Only active
//! staff and heads are read. Phones, addresses, the driver and the notes are never given (PRD
//! FR-7.83). A truck number is shown as text the model can read but cannot cite: the answer names
//! the بکری by its number.
//!
//! Example (worked day): `find{text:"akram", role:"expense_head"}` finds nothing; `find{text:"all",
//! role:"expense_head"}` lists the heads with `count`; `find{text:"LES-1234", role:"truck"}` gives
//! بکری 1 with its date, seller, gross and net.

use mandi_services::core::bikri::{BikriFindInput, BikriListRow, BikriStatus};
use mandi_services::core::masters::{MasterKind, MasterRow};
use mandi_services::core::search::SearchScope;
use mandi_services::core::text;
use mandi_services::{bikri, masters, search};
use serde_json::Value;

use super::find::Role;
use super::{Draft, Env, Stop, service_stop};
use crate::results::{Money, NameVal, Node, Val};

/// Most matches listed. `count` and `more` say when there are more.
const LIST_CAP: usize = 12;

/// The jobs of the staff list (the master's stored words).
const JOBS: [&str; 3] = ["munshi", "collector", "labour"];

/// What a staff word asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaffWord {
    /// Everyone on the staff.
    All,
    /// One job.
    Job(&'static str),
}

/// Words that mean a job, or all of the staff, in Roman Urdu, Urdu letters and English.
const STAFF_WORDS: [(&str, StaffWord); 26] = [
    ("munshi", StaffWord::Job("munshi")),
    ("munshis", StaffWord::Job("munshi")),
    ("munshian", StaffWord::Job("munshi")),
    ("منشی", StaffWord::Job("munshi")),
    ("منشیوں", StaffWord::Job("munshi")),
    ("collector", StaffWord::Job("collector")),
    ("collectors", StaffWord::Job("collector")),
    ("ugrahi", StaffWord::Job("collector")),
    ("ugrahiwala", StaffWord::Job("collector")),
    ("wasooliwala", StaffWord::Job("collector")),
    ("اگراہی", StaffWord::Job("collector")),
    ("اگراہی والا", StaffWord::Job("collector")),
    ("labour", StaffWord::Job("labour")),
    ("labor", StaffWord::Job("labour")),
    ("mazdoor", StaffWord::Job("labour")),
    ("mazdur", StaffWord::Job("labour")),
    ("مزدور", StaffWord::Job("labour")),
    ("staff", StaffWord::All),
    ("workers", StaffWord::All),
    ("employees", StaffWord::All),
    ("mulazim", StaffWord::All),
    ("mulazmeen", StaffWord::All),
    ("amla", StaffWord::All),
    ("عملہ", StaffWord::All),
    ("ملازم", StaffWord::All),
    ("ملازمین", StaffWord::All),
];

/// Words that mean every expense head.
const HEAD_ALL_WORDS: [&str; 12] = [
    "all",
    "heads",
    "head",
    "expense",
    "expenses",
    "kharcha",
    "kharche",
    "kharchay",
    "kharch",
    "خرچہ",
    "اخراجات",
    "مد",
];

fn count(n: usize) -> Node {
    Node::Val(Val::Count(i64::try_from(n).unwrap_or(i64::MAX)))
}

/// The word typed, lower-cased and trimmed, for the lists above.
fn plain(query: &str) -> String {
    query.trim().to_lowercase()
}

fn staff_word(query: &str) -> Option<StaffWord> {
    let typed = plain(query);
    let urdu = text::search_ur(query);
    STAFF_WORDS
        .iter()
        .find(|(word, _)| *word == typed || *word == urdu)
        .map(|(_, what)| *what)
}

/// The runs of the new roles; `None` for any other role.
///
/// # Errors
/// As the services it reads: a stop for the model.
pub(super) fn run(
    env: &Env<'_>,
    role: Role,
    query: &str,
    args: &Value,
) -> Result<Option<Draft>, Stop> {
    let node = match role {
        Role::Staff => staff(env, query)?,
        Role::ExpenseHead => heads(env, query)?,
        Role::Truck => trucks(env, query)?,
        Role::Packing | Role::Marka | Role::Store | Role::Narration => {
            super::find_masters::list(env, role, query)?
        }
        _ => return Ok(None),
    };
    Ok(Some(Draft::new("find", args.clone(), node)))
}

pub(super) fn name(name_ur: &str, name_en: &str, code: &str) -> Node {
    Node::Val(Val::Name(NameVal {
        name_ur: name_ur.to_owned(),
        name_en: name_en.to_owned(),
        tag: None,
        code: Some(code.to_owned()),
    }))
}

/// The common shape of a list result.
pub(super) fn list_node(
    role: &str,
    query: &str,
    matches: Vec<Node>,
    total: usize,
    mut extra: Vec<(&str, Node)>,
) -> Node {
    let more = total > matches.len();
    let mut fields = vec![
        ("text", Node::Str(query.to_owned())),
        ("role", Node::Str(role.to_owned())),
        ("matches", Node::List(matches)),
        ("more", Node::Bool(more)),
        ("count", count(total)),
    ];
    fields.append(&mut extra);
    Node::obj(fields)
}

// ---- staff ------------------------------------------------------------------------------------

fn staff(env: &Env<'_>, query: &str) -> Result<Node, Stop> {
    let rows: Vec<_> = masters::list(env.ctx, MasterKind::Staff, false)
        .map_err(service_stop)?
        .into_iter()
        .filter_map(|row| match row {
            MasterRow::Staff(staff) => Some(staff),
            _ => None,
        })
        .collect();
    let by_word = staff_word(query);
    let found: Vec<_> = match by_word {
        Some(StaffWord::All) => rows.iter().collect(),
        Some(StaffWord::Job(job)) => rows.iter().filter(|r| r.job == job).collect(),
        None => {
            // A name: the smart search of the pickers, then the job of each hit.
            let hits = search::search(env.ctx, env.actor, SearchScope::Staff, query, 50)
                .map_err(service_stop)?;
            hits.iter()
                .filter_map(|hit| rows.iter().find(|r| r.id == hit.id))
                .collect()
        }
    };
    let total = found.len();
    let matches = found
        .iter()
        .take(LIST_CAP)
        .map(|r| {
            Node::obj(vec![
                ("key", Node::Str(format!("st{}", r.id))),
                ("code", Node::Str(r.code.clone())),
                ("name", name(&r.name_ur, &r.name_en, &r.code)),
                ("job", Node::Str(r.job.clone())),
                ("active", Node::Bool(r.is_active)),
            ])
        })
        .collect();
    let mut extra = Vec::new();
    // For a job word or "all staff", the count of each job over the people found.
    if by_word.is_some() {
        let each = JOBS
            .iter()
            .map(|job| (*job, count(found.iter().filter(|r| r.job == *job).count())))
            .collect::<Vec<_>>();
        extra.push(("by_job", Node::obj(each)));
    }
    Ok(list_node("staff", query, matches, total, extra))
}

// ---- expense heads ----------------------------------------------------------------------------

fn heads(env: &Env<'_>, query: &str) -> Result<Node, Stop> {
    let rows: Vec<_> = masters::list(env.ctx, MasterKind::ExpenseHead, false)
        .map_err(service_stop)?
        .into_iter()
        .filter_map(|row| match row {
            MasterRow::ExpenseHead(head) => Some(head),
            _ => None,
        })
        .collect();
    let typed = plain(query);
    let all = HEAD_ALL_WORDS.contains(&typed.as_str()) || HEAD_ALL_WORDS.contains(&query.trim());
    let found: Vec<_> = if all {
        rows.iter().collect()
    } else {
        let hits = search::search(env.ctx, env.actor, SearchScope::ExpenseHead, query, 50)
            .map_err(service_stop)?;
        hits.iter()
            .filter_map(|hit| rows.iter().find(|r| r.id == hit.id))
            .collect()
    };
    let total = found.len();
    let matches = found
        .iter()
        .take(LIST_CAP)
        .map(|r| {
            Node::obj(vec![
                ("key", Node::Str(format!("eh{}", r.id))),
                ("code", Node::Str(r.code.clone())),
                ("name", name(&r.name_ur, &r.name_en, &r.code)),
                ("active", Node::Bool(r.is_active)),
            ])
        })
        .collect();
    Ok(list_node("expense_head", query, matches, total, Vec::new()))
}

// ---- trucks -----------------------------------------------------------------------------------

fn money(paisa: mandi_services::core::money::Paisa) -> Node {
    Node::Val(Val::Money(Money::plain(paisa)))
}

fn truck_row(row: &BikriListRow) -> Node {
    let seller = row.sellers.first().map_or(Node::Null, |s| {
        Node::Val(Val::Name(NameVal {
            name_ur: s.name_ur.clone(),
            name_en: s.name_en.clone(),
            tag: s.tag.clone(),
            code: Some(s.code.clone()),
        }))
    });
    let status = match row.status {
        BikriStatus::Saved => "saved",
        BikriStatus::Cancelled => "cancelled",
    };
    Node::obj(vec![
        ("key", Node::Str(format!("bk{}", row.id))),
        (
            "bikri",
            Node::Val(Val::Doc {
                type_id: "report-doc-bikri-first".to_owned(),
                number: Some(row.number),
            }),
        ),
        ("number", Node::Val(Val::Count(row.number))),
        ("date", Node::Val(Val::Date(row.entry_date.clone()))),
        ("truck", Node::Str(row.truck_no.clone().unwrap_or_default())),
        ("seller", seller),
        ("status", Node::Str(status.to_owned())),
        ("qty_received", Node::Val(Val::Count(row.qty_received))),
        ("qty_sold", Node::Val(Val::Count(row.qty_sold))),
        ("gross", money(row.gross)),
        ("net", money(row.net)),
    ])
}

/// The بکری of a truck number, newest first. A number typed with spaces is tried with dashes too.
fn trucks(env: &Env<'_>, query: &str) -> Result<Node, Stop> {
    let read = |truck: &str| {
        bikri::find(
            env.ctx,
            &BikriFindInput {
                truck_no: Some(truck.to_owned()),
                ..BikriFindInput::default()
            },
        )
        .map_err(service_stop)
    };
    let typed = query.trim();
    let mut rows = read(typed)?;
    if rows.is_empty() && typed.contains(' ') {
        rows = read(&typed.split_whitespace().collect::<Vec<_>>().join("-"))?;
    }
    let total = rows.len();
    let matches = rows
        .iter()
        .take(crate::limits::MAX_OPTIONS)
        .map(truck_row)
        .collect();
    Ok(list_node("truck", query, matches, total, Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_word_or_an_all_word_is_known_in_every_script() {
        assert_eq!(staff_word("munshi"), Some(StaffWord::Job("munshi")));
        assert_eq!(staff_word(" Munshi "), Some(StaffWord::Job("munshi")));
        assert_eq!(staff_word("منشی"), Some(StaffWord::Job("munshi")));
        assert_eq!(staff_word("ugrahi"), Some(StaffWord::Job("collector")));
        assert_eq!(staff_word("mazdoor"), Some(StaffWord::Job("labour")));
        assert_eq!(staff_word("staff"), Some(StaffWord::All));
        assert_eq!(staff_word("asif"), None);
    }

    #[test]
    fn the_jobs_are_the_masters_stored_words() {
        for (_, what) in STAFF_WORDS {
            if let StaffWord::Job(job) = what {
                assert!(JOBS.contains(&job), "{job}");
            }
        }
    }
}
