//! The harness of the loop tests: the worked day with the assistant on, a key in memory, and a
//! recorded model. Shared by `golden_questions.rs` and `loop_limits.rs`.
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use mandi_assistant::answer::{Answer, AnswerPart};
use mandi_assistant::ask::{
    AskReply, AssistantAskInput, AssistantPickInput, AssistantQuickInput, PickChoice, QuickChip,
};
use mandi_assistant::client::{ModelClient, RecordedClient};
use mandi_assistant::keys::MemoryKeyStore;
use mandi_assistant::request::provider;
use mandi_assistant::run::{Deps, ask, pick};
use mandi_assistant::runtime::Runtime;
use mandi_assistant::step::AssistantStep;
use mandi_reports::dashboard::BackupFacts;
use mandi_reports::reports::Cell;
use mandi_services::assistant;
use mandi_services::core::accounts::BalanceSide;
use serde_json::Value;

use crate::{worked, world};

/// A key that looks like one and is not (a placeholder for the tests; the real one is never in
/// code, fixtures or logs).
pub const TEST_KEY: &str = "sk-or-v1-0000000000000000000000000000000000000000000000000000";

pub struct Harness {
    pub world: world::World,
    pub runtime: Runtime,
    pub steps: Arc<Mutex<Vec<AssistantStep>>>,
    /// Every request body sent, for the privacy scan.
    pub seen: Mutex<Vec<Value>>,
}

impl Harness {
    /// The worked day after Events 1 to 5, with the assistant on and a key saved.
    pub fn day() -> Self {
        let world = worked::worked_day();
        worked::events_1_to_5(&world);
        Self::over(world)
    }

    pub fn over(world: world::World) -> Self {
        let runtime = Runtime::new(Arc::new(MemoryKeyStore::default()));
        runtime.keys().set(TEST_KEY).unwrap();
        assistant::set_config(
            &world.env.ctx,
            &world.owner,
            true,
            "openai/gpt-6-luna",
            true,
        )
        .unwrap();
        Self {
            world,
            runtime,
            steps: Arc::new(Mutex::new(Vec::new())),
            seen: Mutex::new(Vec::new()),
        }
    }

    pub fn deps<'a>(
        &'a self,
        client: &'a dyn ModelClient,
        emit: &'a dyn Fn(AssistantStep),
    ) -> Deps<'a> {
        Deps {
            ctx: &self.world.env.ctx,
            actor: &self.world.owner,
            runtime: &self.runtime,
            client,
            emit,
            backup: BackupFacts {
                cloud_on: false,
                cloud_age_hours: None,
            },
        }
    }

    /// Asks one question with a client, and gives the reply.
    pub fn ask_with(
        &self,
        text: &str,
        conversation: Option<&str>,
        client: &RecordedClient,
    ) -> AskReply {
        let steps = self.steps.clone();
        let emit = move |s: AssistantStep| steps.lock().unwrap().push(s);
        let deps = self.deps(client, &emit);
        let reply = ask(
            &deps,
            &AssistantAskInput {
                text: text.to_owned(),
                conversation_id: conversation.map(str::to_owned),
            },
        )
        .unwrap();
        self.seen.lock().unwrap().extend(client.requests());
        reply
    }

    /// Asks one question with recorded replies, and gives the reply and the requests it sent.
    pub fn ask(
        &self,
        text: &str,
        conversation: Option<&str>,
        replies: Vec<Value>,
    ) -> (AskReply, Vec<Value>) {
        let client = RecordedClient::new(replies);
        let reply = self.ask_with(text, conversation, &client);
        (reply, client.requests())
    }

    pub fn pick(
        &self,
        question_id: i64,
        choice: PickChoice,
        replies: Vec<Value>,
    ) -> (AskReply, Vec<Value>) {
        let client = RecordedClient::new(replies);
        let steps = self.steps.clone();
        let emit = move |s: AssistantStep| steps.lock().unwrap().push(s);
        let deps = self.deps(&client, &emit);
        let reply = pick(
            &deps,
            &AssistantPickInput {
                question_id,
                choice,
            },
        )
        .unwrap();
        self.seen.lock().unwrap().extend(client.requests());
        (reply, client.requests())
    }

    pub fn chip(&self, chip: QuickChip) -> AskReply {
        let client = RecordedClient::new(Vec::new());
        let steps = self.steps.clone();
        let emit = move |s: AssistantStep| steps.lock().unwrap().push(s);
        let deps = self.deps(&client, &emit);
        let reply = mandi_assistant::quick::quick(
            &deps,
            &AssistantQuickInput {
                chip,
                party_id: None,
                conversation_id: None,
            },
        )
        .unwrap();
        assert!(client.requests().is_empty(), "a chip never calls the model");
        reply
    }

    /// E2E-54: nothing private in any request, and always the provider block.
    pub fn scan_requests(&self) {
        for body in self.seen.lock().unwrap().iter() {
            assert_eq!(body["provider"], provider(), "the provider block");
            assert_eq!(body["stream"], false);
            let text = body.to_string();
            for forbidden in [
                worked::PRIVATE_PHONE,
                "03001111111",
                worked::PRIVATE_ADDRESS,
                worked::PRIVATE_NOTE,
                "حاجی صاحب",
                "Haji Sahib",
                "0300-0000000",
                "گھر کا خرچ",
                TEST_KEY,
            ] {
                assert!(!text.contains(forbidden), "a request holds {forbidden:?}");
            }
        }
    }
}

pub fn transcript(text: &str) -> Vec<Value> {
    serde_json::from_str(text).unwrap()
}

pub fn answer_of(reply: &AskReply) -> &Answer {
    reply.answer.as_ref().expect("an answer")
}

/// The figures of an answer in order, with whether each carries ✓.
pub fn figures(answer: &Answer) -> Vec<(&Cell, bool)> {
    answer
        .parts
        .iter()
        .filter_map(|p| match p {
            AnswerPart::Figure { cell, checked, .. } => Some((cell, *checked)),
            _ => None,
        })
        .collect()
}

pub fn balance(cell: &Cell) -> (i64, BalanceSide) {
    match cell {
        Cell::Balance { paisa, side } => (paisa.rupees(), *side),
        other => panic!("not a balance: {other:?}"),
    }
}

pub fn money(cell: &Cell) -> i64 {
    match cell {
        Cell::Money { paisa } => paisa.rupees(),
        other => panic!("not money: {other:?}"),
    }
}

pub fn count(cell: &Cell) -> i64 {
    match cell {
        Cell::Number { value } => *value,
        other => panic!("not a count: {other:?}"),
    }
}
