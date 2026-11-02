// A stand-in for the backend for the development page /dev/assistant (plan 49): the question panel and
// the Settings group "سوال پوچھیں" with the backend's own fixtures (tests/fixtures/assistant_*.json), so
// every state can be opened in a plain browser tab and photographed in both languages and at every
// window size. It answers the assistant commands and what the app shell asks; every other command is
// refused with ERR_NOT_IMPLEMENTED. These are the contract's shapes with the worked numbers, not the
// backend's own answers. Never part of a build: only the development route imports it.
//
// Flags in the hash, for example `#/dev/assistant?open&ask=ikram&reply=answered`:
//   open            the panel is open        tall      the tall height        screen=settings  the Settings group
//   ask=<text>      one question is asked at once       reply=<key>  which reply the model gives (see REPLIES)
//   chip=<chip>     one quick question is asked at once   slow  the reply takes 6 seconds
//   status=off|nokey   questions are off / no key is saved       down   every command fails
//   steps           two step lines are shown while the question works (use with slow)

import type { AskReply } from '../../src/bindings/AskReply';
import { dashboardHandlers, inputOf, refusal, type Input } from './dev-bridge-dashboard';
import { devValue } from './dev-bridge';
import * as real from './assistant-real';

/** The replies a page can ask for with `reply=<key>`. */
export const REPLIES: Readonly<Record<string, AskReply>> = {
  answered: real.REPLY_ANSWERED,
  sum: real.REPLY_ANSWERED_SUM,
  chart: real.REPLY_ANSWERED_CHART,
  estimate: real.REPLY_ESTIMATE,
  target: real.REPLY_ESTIMATE_TARGET,
  // A what-if: the same-pace figure scaled by a factor the Owner asked for (plan 52 item 5).
  whatif: real.REPLY_ESTIMATE_WHAT_IF,
  // To the end of the year: the books so far, the months left and the whole year (PRD FR-7.86, v1.20).
  yearend: real.REPLY_ESTIMATE_YEAR_END,
  advice: real.REPLY_ADVICE,
  // The books had too little: short general advice with its own band (PRD FR-7.88, v1.19).
  general: real.REPLY_GENERAL_ADVICE,
  // An answer or advice whose question also asked for a change carries the route of the entry screen.
  advice_route: real.REPLY_ADVICE_ROUTE,
  answered_route: real.REPLY_ANSWERED_ROUTE,
  which_one: real.REPLY_WHICH_ONE,
  not_found: real.REPLY_NOT_FOUND,
  need_dates: real.REPLY_NEED_DATES,
  need_party: real.REPLY_NEED_PARTY,
  not_understood: real.REPLY_NOT_UNDERSTOOD,
  outside: real.REPLY_OUTSIDE_BOOKS,
  change: real.REPLY_CHANGE_REFUSED,
  checks_failed: real.REPLY_CHECKS_FAILED,
  differ: real.REPLY_RECORDS_DIFFER,
  locked: real.REPLY_LOCKED,
  limit: real.REPLY_LIMIT_REACHED,
  offline: real.REPLY_OFFLINE,
  error: real.REPLY_ERROR_404,
};

/** The reply for a question: the one the flag names, else by a word of the question. */
function replyFor(text: string): AskReply {
  const named = devValue('reply');
  const found = named === null ? undefined : REPLIES[named];
  if (found !== undefined) return found;
  const lower = text.toLowerCase();
  if (lower.includes('aakhir')) return real.REPLY_ESTIMATE_YEAR_END;
  if (lower.includes('double')) return real.REPLY_ESTIMATE_WHAT_IF;
  if (lower.includes('barhayen')) return real.REPLY_GENERAL_ADVICE;
  if (lower.includes('agar') || lower.includes('estimate')) return real.REPLY_ESTIMATE;
  if (lower.includes('kis se') || lower.includes('pehle')) return real.REPLY_ADVICE;
  if (lower.includes('mila kar') || lower.includes('dono')) return real.REPLY_ANSWERED_SUM;
  if (lower.includes('partner')) return real.REPLY_LOCKED;
  if (lower.includes('likh')) return real.REPLY_CHANGE_REFUSED;
  if (lower.includes('akram') || lower.includes('akrm')) return real.REPLY_WHICH_ONE;
  if (lower.includes('ikram')) return real.REPLY_ANSWERED;
  return real.REPLY_NOT_UNDERSTOOD;
}

function status() {
  const mode = devValue('status');
  if (mode === 'off') return real.STATUS_OFF;
  if (mode === 'nokey') return { ...real.STATUS_OFF, key_saved: false, key_last4: null };
  return real.STATUS_ON;
}

const handlers: Record<string, (input: Input) => unknown> = {
  ...dashboardHandlers,
  assistant_status: () => status(),
  assistant_ask: (input) => {
    const text = input['text'];
    return replyFor(typeof text === 'string' ? text : '');
  },
  assistant_quick: () => real.QUICK_CASH_IN_HAND,
  assistant_pick: () => real.PICK_MATCH,
  assistant_rerun: () => real.RERUN,
  assistant_new_conversation: () => real.NEW_CONVERSATION,
  assistant_settings_set: () => real.SETTINGS_SET,
  assistant_key_save: () => real.KEY_SAVE,
  assistant_key_remove: () => real.KEY_REMOVE,
  assistant_test: () => (devValue('test') === 'bad' ? real.TEST_NO_ENDPOINT : real.TEST_WORKED),
  assistant_log_list: () => real.LOG_LIST_FIRST_PAGE,
  partners_code_status: () => ({ is_set: true, unlocked: false, wait_seconds: 0 }),
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installAssistantDevBridge(): void {
  const delay = devValue('slow') === null ? 20 : 6000;
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      transformCallback: () => 0,
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(
            () => {
              const handler = handlers[command];
              if (handler === undefined || devValue('down') !== null) {
                reject(refusal(handler === undefined ? 'ERR_NOT_IMPLEMENTED' : 'ERR_INTERNAL'));
                return;
              }
              try {
                resolve(handler(inputOf(args)));
              } catch (error) {
                reject(error instanceof Error ? error : refusal('ERR_INTERNAL'));
              }
            },
            command.startsWith('assistant_ask') ? delay : 20,
          );
        }),
    },
  });
}
