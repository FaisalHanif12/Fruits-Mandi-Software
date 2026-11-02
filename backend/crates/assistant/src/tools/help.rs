//! The tool `app_help`: the written guide of the app (plan 51 P51-B3, PRD FR-7.89).
//!
//! The Owner asks "bikri kaise add karun", "wasooli kaise likhun" or "market safar kya hai". The
//! books have no figure for that, so the model asks for a topic and the app gives back its guide:
//! what the screen is for, when to use it, the steps, the keys and one worked example, in the app
//! language, as the app itself wrote it (`prompt/guide.json`). The model cites those texts by
//! reference (`{{r1.steps.0}}`); the app puts the same words into the answer, so a key such as
//! F10 or a step such as "press Save" is never retyped by the model and never wrong.
//!
//! The guide holds no figure of the books and no name of a person: it reads nothing, writes
//! nothing and needs no partner code.
//!
//! Example: `app_help{topic:"bikri_book"}` gives `title`, `summary`, `steps` (a list), `keys` and
//! `example`, each a text of the guide.

use mandi_services::core::auth::Language;
use serde::Deserialize;
use serde_json::Value as Json;

use super::{Draft, Env, State, Stop, ToolError, input};
use crate::prompt::guide;
use crate::results::{Node, Val};

/// The input of `app_help` (D6-system-prompt Section 3).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The topic: one of the names in the enum of the tool.
    pub topic: String,
}

fn text(s: &str) -> Node {
    Node::Val(Val::Guide(s.to_owned()))
}

/// The tool.
pub fn run(env: &Env<'_>, _state: &State, args: &Json) -> Result<Draft, Stop> {
    let parsed: Input = input(args)?;
    let guide = guide();
    let Some(topic) = guide.topics.get(&parsed.topic) else {
        return Err(ToolError::new(
            "ERR_OUT_OF_RANGE",
            "topic",
            "no such topic in the guide; use one of the listed topics",
        )
        .allowed(guide.topics.keys().cloned().collect())
        .into());
    };
    let guide_text = match env.language {
        Language::Ur => &topic.ur,
        Language::En => &topic.en,
    };
    let node = Node::obj(vec![
        ("topic", Node::Str(parsed.topic.clone())),
        ("title", text(&guide_text.title)),
        ("summary", text(&guide_text.summary)),
        (
            "steps",
            Node::List(guide_text.steps.iter().map(|s| text(s)).collect()),
        ),
        ("keys", text(&guide_text.keys)),
        ("example", text(&guide_text.example)),
    ]);
    Ok(Draft::new("app_help", args.clone(), node))
}
