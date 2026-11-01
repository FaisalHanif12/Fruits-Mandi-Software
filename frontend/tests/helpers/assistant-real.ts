// The real answers and requests of the backend for plan 48 (ask the app, D6 Section 2). The backend wrote
// them in its own test (backend/crates/assistant/tests/fixtures/assistant_*.json); they are copied here,
// never edited, and read as the generated types. Money in them is paisa, as on the wire. A reader takes
// `unknown`, because the money in a binding is a branded type that a plain number is not.

import type { AskReply } from '../../src/bindings/AskReply';
import type { AssistantAskInput } from '../../src/bindings/AssistantAskInput';
import type { AssistantKeyRemoveInput } from '../../src/bindings/AssistantKeyRemoveInput';
import type { AssistantKeySaveInput } from '../../src/bindings/AssistantKeySaveInput';
import type { AssistantLogListInput } from '../../src/bindings/AssistantLogListInput';
import type { AssistantLogPage } from '../../src/bindings/AssistantLogPage';
import type { AssistantPickInput } from '../../src/bindings/AssistantPickInput';
import type { AssistantQuickInput } from '../../src/bindings/AssistantQuickInput';
import type { AssistantRerunInput } from '../../src/bindings/AssistantRerunInput';
import type { AssistantSettingsSetInput } from '../../src/bindings/AssistantSettingsSetInput';
import type { AssistantStatus } from '../../src/bindings/AssistantStatus';
import type { AssistantStep } from '../../src/bindings/AssistantStep';
import type { AssistantTestResult } from '../../src/bindings/AssistantTestResult';
import type { ConversationStarted } from '../../src/bindings/ConversationStarted';
import assistantKeyRemove from '../fixtures/assistant_key_remove.json';
import assistantKeyRemoveRequest from '../fixtures/assistant_key_remove_request.json';
import assistantKeySave from '../fixtures/assistant_key_save.json';
import assistantKeySaveRequest from '../fixtures/assistant_key_save_request.json';
import assistantLogListFirstPage from '../fixtures/assistant_log_list_first_page.json';
import assistantLogListFirstPageRequest from '../fixtures/assistant_log_list_first_page_request.json';
import assistantNewConversation from '../fixtures/assistant_new_conversation.json';
import assistantPickMatch from '../fixtures/assistant_pick_match.json';
import assistantPickMatchRequest from '../fixtures/assistant_pick_match_request.json';
import assistantPickPartnerCode from '../fixtures/assistant_pick_partner_code.json';
import assistantPickPartnerCodeRequest from '../fixtures/assistant_pick_partner_code_request.json';
import assistantQuickCashInHand from '../fixtures/assistant_quick_cash_in_hand.json';
import assistantQuickCashInHandRequest from '../fixtures/assistant_quick_cash_in_hand_request.json';
import assistantReplyAdvice from '../fixtures/assistant_reply_advice.json';
import assistantReplyAdviceRequest from '../fixtures/assistant_reply_advice_request.json';
import assistantReplyAnswered from '../fixtures/assistant_reply_answered.json';
import assistantReplyAnsweredChart from '../fixtures/assistant_reply_answered_chart.json';
import assistantReplyAnsweredChartRequest from '../fixtures/assistant_reply_answered_chart_request.json';
import assistantReplyAnsweredRequest from '../fixtures/assistant_reply_answered_request.json';
import assistantReplyAnsweredSum from '../fixtures/assistant_reply_answered_sum.json';
import assistantReplyAnsweredSumRequest from '../fixtures/assistant_reply_answered_sum_request.json';
import assistantReplyChangeRefused from '../fixtures/assistant_reply_change_refused.json';
import assistantReplyChangeRefusedRequest from '../fixtures/assistant_reply_change_refused_request.json';
import assistantReplyChecksFailed from '../fixtures/assistant_reply_checks_failed.json';
import assistantReplyChecksFailedRequest from '../fixtures/assistant_reply_checks_failed_request.json';
import assistantReplyError404 from '../fixtures/assistant_reply_error_404.json';
import assistantReplyError404Request from '../fixtures/assistant_reply_error_404_request.json';
import assistantReplyEstimate from '../fixtures/assistant_reply_estimate.json';
import assistantReplyEstimateRequest from '../fixtures/assistant_reply_estimate_request.json';
import assistantReplyEstimateTarget from '../fixtures/assistant_reply_estimate_target.json';
import assistantReplyEstimateTargetRequest from '../fixtures/assistant_reply_estimate_target_request.json';
import assistantReplyLimitReached from '../fixtures/assistant_reply_limit_reached.json';
import assistantReplyLimitReachedRequest from '../fixtures/assistant_reply_limit_reached_request.json';
import assistantReplyLocked from '../fixtures/assistant_reply_locked.json';
import assistantReplyLockedRequest from '../fixtures/assistant_reply_locked_request.json';
import assistantReplyNeedDates from '../fixtures/assistant_reply_need_dates.json';
import assistantReplyNeedDatesRequest from '../fixtures/assistant_reply_need_dates_request.json';
import assistantReplyNeedParty from '../fixtures/assistant_reply_need_party.json';
import assistantReplyNeedPartyRequest from '../fixtures/assistant_reply_need_party_request.json';
import assistantReplyNotFound from '../fixtures/assistant_reply_not_found.json';
import assistantReplyNotFoundRequest from '../fixtures/assistant_reply_not_found_request.json';
import assistantReplyNotUnderstood from '../fixtures/assistant_reply_not_understood.json';
import assistantReplyNotUnderstoodRequest from '../fixtures/assistant_reply_not_understood_request.json';
import assistantReplyOffline from '../fixtures/assistant_reply_offline.json';
import assistantReplyOfflineRequest from '../fixtures/assistant_reply_offline_request.json';
import assistantReplyOutsideBooks from '../fixtures/assistant_reply_outside_books.json';
import assistantReplyOutsideBooksRequest from '../fixtures/assistant_reply_outside_books_request.json';
import assistantReplyRecordsDiffer from '../fixtures/assistant_reply_records_differ.json';
import assistantReplyRecordsDifferRequest from '../fixtures/assistant_reply_records_differ_request.json';
import assistantReplyWhichOne from '../fixtures/assistant_reply_which_one.json';
import assistantReplyWhichOneRequest from '../fixtures/assistant_reply_which_one_request.json';
import assistantRerun from '../fixtures/assistant_rerun.json';
import assistantRerunRequest from '../fixtures/assistant_rerun_request.json';
import assistantSettingsSet from '../fixtures/assistant_settings_set.json';
import assistantSettingsSetRequest from '../fixtures/assistant_settings_set_request.json';
import assistantStatusOff from '../fixtures/assistant_status_off.json';
import assistantStatusOn from '../fixtures/assistant_status_on.json';
import assistantStepReadingReport from '../fixtures/assistant_step_reading_report.json';
import assistantTestNoEndpoint from '../fixtures/assistant_test_no_endpoint.json';
import assistantTestWorked from '../fixtures/assistant_test_worked.json';

function readAskReply(value: unknown): AskReply {
  return value as AskReply;
}
function readAssistantAskInput(value: unknown): AssistantAskInput {
  return value as AssistantAskInput;
}
function readAssistantKeyRemoveInput(value: unknown): AssistantKeyRemoveInput {
  return value as AssistantKeyRemoveInput;
}
function readAssistantKeySaveInput(value: unknown): AssistantKeySaveInput {
  return value as AssistantKeySaveInput;
}
function readAssistantLogListInput(value: unknown): AssistantLogListInput {
  return value as AssistantLogListInput;
}
function readAssistantLogPage(value: unknown): AssistantLogPage {
  return value as AssistantLogPage;
}
function readAssistantPickInput(value: unknown): AssistantPickInput {
  return value as AssistantPickInput;
}
function readAssistantQuickInput(value: unknown): AssistantQuickInput {
  return value as AssistantQuickInput;
}
function readAssistantRerunInput(value: unknown): AssistantRerunInput {
  return value as AssistantRerunInput;
}
function readAssistantSettingsSetInput(value: unknown): AssistantSettingsSetInput {
  return value as AssistantSettingsSetInput;
}
function readAssistantStatus(value: unknown): AssistantStatus {
  return value as AssistantStatus;
}
function readAssistantStep(value: unknown): AssistantStep {
  return value as AssistantStep;
}
function readAssistantTestResult(value: unknown): AssistantTestResult {
  return value as AssistantTestResult;
}
function readConversationStarted(value: unknown): ConversationStarted {
  return value as ConversationStarted;
}

export const KEY_REMOVE = readAssistantStatus(assistantKeyRemove);
export const KEY_REMOVE_REQUEST = readAssistantKeyRemoveInput(assistantKeyRemoveRequest);
export const KEY_SAVE = readAssistantStatus(assistantKeySave);
export const KEY_SAVE_REQUEST = readAssistantKeySaveInput(assistantKeySaveRequest);
export const LOG_LIST_FIRST_PAGE = readAssistantLogPage(assistantLogListFirstPage);
export const LOG_LIST_FIRST_PAGE_REQUEST = readAssistantLogListInput(
  assistantLogListFirstPageRequest,
);
export const NEW_CONVERSATION = readConversationStarted(assistantNewConversation);
export const PICK_MATCH = readAskReply(assistantPickMatch);
export const PICK_MATCH_REQUEST = readAssistantPickInput(assistantPickMatchRequest);
export const PICK_PARTNER_CODE = readAskReply(assistantPickPartnerCode);
export const PICK_PARTNER_CODE_REQUEST = readAssistantPickInput(assistantPickPartnerCodeRequest);
export const QUICK_CASH_IN_HAND = readAskReply(assistantQuickCashInHand);
export const QUICK_CASH_IN_HAND_REQUEST = readAssistantQuickInput(assistantQuickCashInHandRequest);
export const REPLY_ADVICE = readAskReply(assistantReplyAdvice);
export const REPLY_ADVICE_REQUEST = readAssistantAskInput(assistantReplyAdviceRequest);
export const REPLY_ANSWERED = readAskReply(assistantReplyAnswered);
export const REPLY_ANSWERED_CHART = readAskReply(assistantReplyAnsweredChart);
export const REPLY_ANSWERED_CHART_REQUEST = readAssistantAskInput(
  assistantReplyAnsweredChartRequest,
);
export const REPLY_ANSWERED_REQUEST = readAssistantAskInput(assistantReplyAnsweredRequest);
export const REPLY_ANSWERED_SUM = readAskReply(assistantReplyAnsweredSum);
export const REPLY_ANSWERED_SUM_REQUEST = readAssistantAskInput(assistantReplyAnsweredSumRequest);
export const REPLY_CHANGE_REFUSED = readAskReply(assistantReplyChangeRefused);
export const REPLY_CHANGE_REFUSED_REQUEST = readAssistantAskInput(
  assistantReplyChangeRefusedRequest,
);
export const REPLY_CHECKS_FAILED = readAskReply(assistantReplyChecksFailed);
export const REPLY_CHECKS_FAILED_REQUEST = readAssistantAskInput(assistantReplyChecksFailedRequest);
export const REPLY_ERROR_404 = readAskReply(assistantReplyError404);
export const REPLY_ERROR_404_REQUEST = readAssistantAskInput(assistantReplyError404Request);
export const REPLY_ESTIMATE = readAskReply(assistantReplyEstimate);
export const REPLY_ESTIMATE_REQUEST = readAssistantAskInput(assistantReplyEstimateRequest);
export const REPLY_ESTIMATE_TARGET = readAskReply(assistantReplyEstimateTarget);
export const REPLY_ESTIMATE_TARGET_REQUEST = readAssistantAskInput(
  assistantReplyEstimateTargetRequest,
);
export const REPLY_LIMIT_REACHED = readAskReply(assistantReplyLimitReached);
export const REPLY_LIMIT_REACHED_REQUEST = readAssistantAskInput(assistantReplyLimitReachedRequest);
export const REPLY_LOCKED = readAskReply(assistantReplyLocked);
export const REPLY_LOCKED_REQUEST = readAssistantAskInput(assistantReplyLockedRequest);
export const REPLY_NEED_DATES = readAskReply(assistantReplyNeedDates);
export const REPLY_NEED_DATES_REQUEST = readAssistantAskInput(assistantReplyNeedDatesRequest);
export const REPLY_NEED_PARTY = readAskReply(assistantReplyNeedParty);
export const REPLY_NEED_PARTY_REQUEST = readAssistantAskInput(assistantReplyNeedPartyRequest);
export const REPLY_NOT_FOUND = readAskReply(assistantReplyNotFound);
export const REPLY_NOT_FOUND_REQUEST = readAssistantAskInput(assistantReplyNotFoundRequest);
export const REPLY_NOT_UNDERSTOOD = readAskReply(assistantReplyNotUnderstood);
export const REPLY_NOT_UNDERSTOOD_REQUEST = readAssistantAskInput(
  assistantReplyNotUnderstoodRequest,
);
export const REPLY_OFFLINE = readAskReply(assistantReplyOffline);
export const REPLY_OFFLINE_REQUEST = readAssistantAskInput(assistantReplyOfflineRequest);
export const REPLY_OUTSIDE_BOOKS = readAskReply(assistantReplyOutsideBooks);
export const REPLY_OUTSIDE_BOOKS_REQUEST = readAssistantAskInput(assistantReplyOutsideBooksRequest);
export const REPLY_RECORDS_DIFFER = readAskReply(assistantReplyRecordsDiffer);
export const REPLY_RECORDS_DIFFER_REQUEST = readAssistantAskInput(
  assistantReplyRecordsDifferRequest,
);
export const REPLY_WHICH_ONE = readAskReply(assistantReplyWhichOne);
export const REPLY_WHICH_ONE_REQUEST = readAssistantAskInput(assistantReplyWhichOneRequest);
export const RERUN = readAskReply(assistantRerun);
export const RERUN_REQUEST = readAssistantRerunInput(assistantRerunRequest);
export const SETTINGS_SET = readAssistantStatus(assistantSettingsSet);
export const SETTINGS_SET_REQUEST = readAssistantSettingsSetInput(assistantSettingsSetRequest);
export const STATUS_OFF = readAssistantStatus(assistantStatusOff);
export const STATUS_ON = readAssistantStatus(assistantStatusOn);
export const STEP_READING_REPORT = readAssistantStep(assistantStepReadingReport);
export const TEST_NO_ENDPOINT = readAssistantTestResult(assistantTestNoEndpoint);
export const TEST_WORKED = readAssistantTestResult(assistantTestWorked);
