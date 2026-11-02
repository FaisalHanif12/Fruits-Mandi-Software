// Typed wrappers around the ask-the-app commands (plan 48 Section 4, PRD FR-7.72 to FR-7.88), and the
// one event the backend sends while a question runs. Components never call invoke() (Code Standards
// F4). The key is only ever an argument of `assistantKeySave`; it is not kept anywhere here.

import type { AskReply } from '../../bindings/AskReply';
import type { ConversationStarted } from '../../bindings/ConversationStarted';
import type { AssistantAskInput } from '../../bindings/AssistantAskInput';
import type { AssistantKeyRemoveInput } from '../../bindings/AssistantKeyRemoveInput';
import type { AssistantKeySaveInput } from '../../bindings/AssistantKeySaveInput';
import type { AssistantLogListInput } from '../../bindings/AssistantLogListInput';
import type { AssistantLogPage } from '../../bindings/AssistantLogPage';
import type { AssistantNewConversationInput } from '../../bindings/AssistantNewConversationInput';
import type { AssistantPickInput } from '../../bindings/AssistantPickInput';
import type { AssistantQuickInput } from '../../bindings/AssistantQuickInput';
import type { AssistantRerunInput } from '../../bindings/AssistantRerunInput';
import type { AssistantSettingsSetInput } from '../../bindings/AssistantSettingsSetInput';
import type { AssistantStatus } from '../../bindings/AssistantStatus';
import type { AssistantStep } from '../../bindings/AssistantStep';
import type { AssistantTestResult } from '../../bindings/AssistantTestResult';
import type { ReportMessageArg } from '../../bindings/ReportMessageArg';
import { callCommand, listenEvent } from '../../shared/api';

/** The event the backend sends for each step line of a running question (plan 48 `step.rs`). */
export const STEP_EVENT = 'assistant://step';

/** On or off, whether a key is saved (with its last 4 characters), the model, today's count. */
export function assistantStatus(): Promise<AssistantStatus> {
  return callCommand<AssistantStatus>('assistant_status');
}

/** A typed question to the model. One question, one log row; it counts once in the 20 a day. */
export function assistantAsk(input: AssistantAskInput): Promise<AskReply> {
  return callCommand<AskReply>('assistant_ask', { input });
}

/** A quick question (a chip): no model, no internet, not counted. */
export function assistantQuick(input: AssistantQuickInput): Promise<AskReply> {
  return callCommand<AskReply>('assistant_quick', { input });
}

/** Goes on with a question that waits: after "کون سا؟" or after the partner code. */
export function assistantPick(input: AssistantPickInput): Promise<AskReply> {
  return callCommand<AskReply>('assistant_pick', { input });
}

/** Reads an old answer again on today's books: plain tables, no model, not counted. */
export function assistantRerun(input: AssistantRerunInput): Promise<AskReply> {
  return callCommand<AskReply>('assistant_rerun', { input });
}

/**
 * "New question": the backend drops the memory of the conversation the panel leaves, at once (PRD
 * FR-7.78). Writes nothing. The next question is sent with no conversation, and the backend starts a
 * new one.
 */
export function assistantNewConversation(
  input: AssistantNewConversationInput,
): Promise<ConversationStarted> {
  return callCommand<ConversationStarted>('assistant_new_conversation', { input });
}

/** Turns the assistant on or off and sets the model name. */
export function assistantSettingsSet(input: AssistantSettingsSetInput): Promise<AssistantStatus> {
  return callCommand<AssistantStatus>('assistant_settings_set', { input });
}

/** Saves or replaces the key; the Owner's password proves it. The reply never holds the key. */
export function assistantKeySave(input: AssistantKeySaveInput): Promise<AssistantStatus> {
  return callCommand<AssistantStatus>('assistant_key_save', { input });
}

/** Removes the key (and switches the assistant off); the Owner's password proves it. */
export function assistantKeyRemove(input: AssistantKeyRemoveInput): Promise<AssistantStatus> {
  return callCommand<AssistantStatus>('assistant_key_remove', { input });
}

/** Sends one fixed test question; it counts as a question. A failed connection is a result. */
export function assistantTest(): Promise<AssistantTestResult> {
  return callCommand<AssistantTestResult>('assistant_test');
}

/** A page of the question log, newest first, with the count over the whole log. */
export function assistantLogList(input: AssistantLogListInput): Promise<AssistantLogPage> {
  return callCommand<AssistantLogPage>('assistant_log_list', { input });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

/** A step line from the event, or null when what came is not one. */
export function stepOf(payload: unknown): AssistantStep | null {
  if (!isRecord(payload)) return null;
  const { id, args } = payload;
  if (typeof id !== 'string' || id === '') return null;
  // The arguments are the report's own message arguments; each is checked to the extent that the
  // cell writer needs (a name and a value with a kind).
  const found: ReportMessageArg[] = [];
  if (Array.isArray(args)) {
    for (const arg of args) {
      if (isRecord(arg) && typeof arg['name'] === 'string' && isRecord(arg['value'])) {
        // The value's own shape is the backend's type; a value with no known kind is written as empty.
        found.push({ name: arg['name'], value: arg['value'] as ReportMessageArg['value'] });
      }
    }
  }
  return { id, args: found };
}

/**
 * Listens to the step lines of a running question. Returns the function that stops listening. In a
 * plain browser tab there is no window, so nothing is heard.
 */
export function listenSteps(handler: (step: AssistantStep) => void): Promise<() => void> {
  return listenEvent(STEP_EVENT, (payload) => {
    const step = stepOf(payload);
    if (step !== null) handler(step);
  });
}
