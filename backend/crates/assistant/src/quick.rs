//! The quick questions (chips) of PRD FR-7.80: fixed tool calls and fixed words, with no model and
//! no internet. They are not counted in the daily 20 and they work when the assistant is off or no
//! key is saved.
//!
//! Each chip reads one tile (or, for a party, his card and ledger) with the tools of the model, so
//! a quick figure is the very figure the model would cite, with the same ✓ where it has a second
//! place. The words are the panel's own Fluent words (`assistant-chip-*`).
//!
//! Example (worked day): the chip "گاہکوں کے ذمے" answers `buyers_dues` 21,564 ✓.

use mandi_services::core::error::ErrorCode;
use mandi_services::{ServiceError, assist, figures};
use serde_json::{Value, json};

use crate::answer::{Answer, AnswerPart};
use crate::ask::{AskOutcome, AskReply, AssistantQuickInput, QuickChip};
use crate::conversation::{Conversation, Turn};
use crate::fill::part_of_val;
use crate::results::{NameVal, Node, ToolResult, Val};
use crate::run::{Deps, Finished, Run, Source};
use crate::tools::{self, State};

/// The Fluent word of a chip and where its figure is in the tiles: (`word`, `tile`, `field`).
fn chip_spec(chip: QuickChip) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match chip {
        QuickChip::TodaySales => ("assistant-chip-sales", "today_sales", "amount"),
        QuickChip::TodayCommission => ("assistant-chip-commission", "today_commission", "amount"),
        QuickChip::CashInHand => ("assistant-chip-cash", "cash_in_hand", "amount"),
        QuickChip::BuyersDues => ("assistant-chip-dues", "buyers_dues", "amount"),
        QuickChip::TotalDues => ("assistant-chip-total-dues", "total_dues", "amount"),
        QuickChip::DeadAmount => ("assistant-chip-dead", "dead_amount", "amount"),
        QuickChip::SellersPayable => ("assistant-chip-payable", "sellers_payable", "amount"),
        QuickChip::AdvancesOpen => ("assistant-chip-advances", "advances_open", "amount"),
        QuickChip::StockLeft => ("assistant-chip-stock", "stock", "qty"),
        QuickChip::PartyLedger => return None,
    })
}

fn chip_name(chip: QuickChip) -> &'static str {
    match chip {
        QuickChip::TodaySales => "today_sales",
        QuickChip::TodayCommission => "today_commission",
        QuickChip::CashInHand => "cash_in_hand",
        QuickChip::BuyersDues => "buyers_dues",
        QuickChip::TotalDues => "total_dues",
        QuickChip::DeadAmount => "dead_amount",
        QuickChip::SellersPayable => "sellers_payable",
        QuickChip::AdvancesOpen => "advances_open",
        QuickChip::StockLeft => "stock_left",
        QuickChip::PartyLedger => "party_ledger",
    }
}

fn result_of(state: &mut State, draft: tools::Draft, run: &Run<'_>) -> ToolResult {
    let id = state.next_id('r');
    let mut result = ToolResult {
        id,
        tool: draft.tool,
        input: draft.input,
        node: draft.node,
        shown: draft.shown,
        source: draft.source,
        partner_parts_locked: draft.partner_parts_locked,
        partner_data: draft.partner_data,
        turn: 0,
        working: draft.working,
    };
    crate::results::fit(&mut result, &run.words);
    result
}

fn answer_of(state: &State, parts: Vec<AnswerPart>, read_at: &str) -> Answer {
    let mut answer = crate::fill::plain_tables(state, read_at);
    // A chip shows its figure and the table of its source; the tables stay for the ledger only.
    answer.parts = parts;
    answer
}

/// A quick question (`assistant_quick`).
///
/// # Errors
/// `ERR_REQUIRED` on `party_id` for the party chip; `ERR_NOT_FOUND` on `party_id`.
pub fn quick(d: &Deps<'_>, input: &AssistantQuickInput) -> Result<AskReply, ServiceError> {
    let run = Run::new(d)?;
    let id = input
        .conversation_id
        .clone()
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| d.runtime.new_conversation_id(d.ctx));
    let conv = d
        .runtime
        .take(&id)
        .unwrap_or_else(|| Conversation::new(&id));
    let mut turn = Turn {
        question: chip_name(input.chip).to_owned(),
        ..Turn::default()
    };
    let numbers = figures::question_numbers("");
    let mut state = State {
        results: Vec::new(),
        turn: 0,
    };
    let env = run.env(&numbers);
    let mut parts = Vec::new();
    let finished = match chip_spec(input.chip) {
        Some((word, tile, field)) => {
            let draft = tools::call(&env, &state, "tiles", "{}")
                .map_err(|_| ServiceError::reject(ErrorCode::ErrInternal))?;
            let result = result_of(&mut state, draft, &run);
            parts.push(AnswerPart::Text {
                text: format!("{}: ", run.words.text(run.language, word)),
            });
            if let Some(Node::Val(val)) = result.node.get(&[tile, field]) {
                parts.push(part_of_val(val, None));
            }
            turn.tool_log
                .push(json!({"id": result.id, "tool": "tiles", "input": {}}));
            state.results.push(result);
            let mut f = Finished::plain(AskOutcome::Answered);
            f.answer = Some(answer_of(&state, parts, &run.read_at()));
            f
        }
        None => {
            let party_id = input
                .party_id
                .ok_or_else(|| ServiceError::reject_field(ErrorCode::ErrRequired, "party_id"))?;
            let card = assist::party_card(&run.snap, party_id)
                .map_err(|_| ServiceError::reject_field(ErrorCode::ErrNotFound, "party_id"))?;
            let key = format!("a{}", card.id);
            // A find result that holds the party, so the tools may name him by his key.
            let found = ToolResult {
                id: state.next_id('r'),
                tool: "find",
                input: Value::Null,
                node: Node::obj(vec![(
                    "matches",
                    Node::List(vec![Node::obj(vec![
                        ("key", Node::Str(key.clone())),
                        (
                            "name",
                            Node::Val(Val::Name(NameVal {
                                name_ur: card.name_ur.clone(),
                                name_en: card.name_en.clone(),
                                tag: card.tag.clone(),
                                code: Some(card.code.clone()),
                            })),
                        ),
                    ])]),
                )]),
                shown: None,
                source: None,
                partner_parts_locked: false,
                partner_data: false,
                turn: 0,
                working: None,
            };
            state.results.push(found);
            let args = json!({
                "id": "R13", "from": null, "to": null, "as_of": null, "party": key,
                "item": null, "city": null, "adda": null, "collector": null, "partner": null,
                "trip": null, "garden": null, "bikri": null, "group_by": null, "where": null,
                "sort": null, "rows": null,
            });
            let draft = tools::call(&env, &state, "report", &args.to_string())
                .map_err(|_| ServiceError::reject(ErrorCode::ErrInternal))?;
            let result = result_of(&mut state, draft, &run);
            parts.push(AnswerPart::Text {
                text: format!("{}: ", run.words.text(run.language, "assistant-chip-party")),
            });
            parts.push(AnswerPart::Name {
                name: NameVal {
                    name_ur: card.name_ur.clone(),
                    name_en: card.name_en.clone(),
                    tag: card.tag.clone(),
                    code: Some(card.code.clone()),
                }
                .answer_name(),
            });
            parts.push(AnswerPart::Text {
                text: " ".to_owned(),
            });
            if let Some(Node::Val(val)) = result.node.get(&["summary", "closing"]) {
                parts.push(part_of_val(val, None));
            }
            turn.tool_log
                .push(json!({"id": result.id, "tool": "report", "input": args}));
            state.results.push(result);
            let mut f = Finished::plain(AskOutcome::Answered);
            f.answer = Some(answer_of(&state, parts, &run.read_at()));
            f
        }
    };
    run.conclude(conv, turn, finished, Source::Chip, None)
}
