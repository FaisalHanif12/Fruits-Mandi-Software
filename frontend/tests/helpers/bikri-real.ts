// The real answers of the backend for plan 13 (T4). The backend wrote them in its own tests
// (backend/crates/services/tests/fixtures/*.json). They are copied here, never edited, and read as
// the generated types. Money in them is paisa, as on the wire. The tests that use them show the
// screens the numbers of PRD 4.4 and 4.5 from the backend's own answers, not from canned data.

import saveEvent1 from '../fixtures/bikri_save_event1.json';
import saveEvent1Request from '../fixtures/bikri_save_event1_request.json';
import saveEvent2 from '../fixtures/bikri_save_event2.json';
import saveEvent2Request from '../fixtures/bikri_save_event2_request.json';
import getEvent1 from '../fixtures/bikri_get_event1.json';
import getEvent2 from '../fixtures/bikri_get_event2.json';
import listDay from '../fixtures/bikri_list_day.json';
import listDayRequest from '../fixtures/bikri_list_day_request.json';
import draftsGet from '../fixtures/drafts_get_bikri.json';
import searchGlobalTruck from '../fixtures/search_global_bikri_truck.json';
import chuktaSearchShahid from '../fixtures/chukta_search_shahid.json';
import chuktaSearchShahidRequest from '../fixtures/chukta_search_shahid_request.json';
import chuktaAddNasir from '../fixtures/chukta_add_nasir.json';
import chuktaAddNasirRequest from '../fixtures/chukta_add_nasir_request.json';
import chuktaAddRefused from '../fixtures/chukta_add_refused.json';
import chuktaAddRefusedRequest from '../fixtures/chukta_add_refused_request.json';
import searchChuktaBuyer from '../fixtures/search_accounts_chukta_buyer.json';
import searchChuktaBuyerRequest from '../fixtures/search_accounts_chukta_buyer_request.json';
import cardChuktaBuyer from '../fixtures/assist_party_card_chukta_buyer.json';
import cardChuktaBuyerRequest from '../fixtures/assist_party_card_chukta_buyer_request.json';
import type { BikriDetail } from '../../src/bindings/BikriDetail';
import type { BikriList } from '../../src/bindings/BikriList';
import type { BikriSaveInput } from '../../src/bindings/BikriSaveInput';
import type { BikriSaved } from '../../src/bindings/BikriSaved';
import type { DraftRow } from '../../src/bindings/DraftRow';
import type { GlobalHit } from '../../src/bindings/GlobalHit';
import type { AccountHit } from '../../src/bindings/AccountHit';
import type { ChuktaBuyer } from '../../src/bindings/ChuktaBuyer';
import type { PartyCard } from '../../src/bindings/PartyCard';
import type { UiError } from '../../src/bindings/UiError';

/** The fixture files are the backend's own JSON; their text is read as the generated types. */
function savedOf(value: unknown): BikriSaved {
  return value as BikriSaved;
}
function saveInputOf(value: unknown): BikriSaveInput {
  return value as BikriSaveInput;
}
function detailOf(value: unknown): BikriDetail {
  return value as BikriDetail;
}
function listOf(value: unknown): BikriList {
  return value as BikriList;
}
function draftOf(value: unknown): DraftRow {
  return value as DraftRow;
}
function hitsOf(value: unknown): GlobalHit[] {
  return value as GlobalHit[];
}
function chuktaListOf(value: unknown): ChuktaBuyer[] {
  return value as ChuktaBuyer[];
}
function chuktaOf(value: unknown): ChuktaBuyer {
  return value as ChuktaBuyer;
}
function hitListOf(value: unknown): AccountHit[] {
  return value as AccountHit[];
}
function cardOf(value: unknown): PartyCard {
  return value as PartyCard;
}
function refusalOf(value: unknown): UiError {
  return value as UiError;
}

export const SAVED_EVENT1: BikriSaved = savedOf(saveEvent1);
export const SAVED_EVENT2: BikriSaved = savedOf(saveEvent2);

/** What the screen sends to save Event 1 and Event 2, as the backend wrote it. */
export const SAVE_EVENT1_REQUEST: BikriSaveInput = saveInputOf(saveEvent1Request);
export const SAVE_EVENT2_REQUEST: BikriSaveInput = saveInputOf(saveEvent2Request);
export const GET_EVENT1: BikriDetail = detailOf(getEvent1);
export const GET_EVENT2: BikriDetail = detailOf(getEvent2);
/** The list of 10-10-2026: Event 2 first (number 2), then Event 1, newest first. */
export const LIST_DAY: BikriList = listOf(listDay);
export const LIST_DAY_REQUEST: Record<string, unknown> = listDayRequest;
export const DRAFT_ROW: DraftRow = draftOf(draftsGet);
export const GLOBAL_TRUCK: GlobalHit[] = hitsOf(searchGlobalTruck);
/** The چکتا buyers the buyer picker finds for "shahid" (plan 64), and what it sent. */
export const CHUKTA_SEARCH_SHAHID: ChuktaBuyer[] = chuktaListOf(chuktaSearchShahid);
export const CHUKTA_SEARCH_SHAHID_REQUEST: Record<string, unknown> = chuktaSearchShahidRequest;
/** A new چکتا buyer added from the picker, and the refusal with both errors together. */
export const CHUKTA_ADD_NASIR: ChuktaBuyer = chuktaOf(chuktaAddNasir);
export const CHUKTA_ADD_NASIR_REQUEST: Record<string, unknown> = chuktaAddNasirRequest;
export const CHUKTA_ADD_REFUSED: UiError = refusalOf(chuktaAddRefused);
export const CHUKTA_ADD_REFUSED_REQUEST: Record<string, unknown> = chuktaAddRefusedRequest;
/** The buyer search: a چکتا buyer is a hit marked `chukta` with the id of the account چکتا کھاتہ. */
export const SEARCH_CHUKTA_BUYER: AccountHit[] = hitListOf(searchChuktaBuyer);
export const SEARCH_CHUKTA_BUYER_REQUEST: Record<string, unknown> = searchChuktaBuyerRequest;
/** His own party card: name and phone, no balance. */
export const CARD_CHUKTA_BUYER: PartyCard = cardOf(cardChuktaBuyer);
export const CARD_CHUKTA_BUYER_REQUEST: Record<string, unknown> = cardChuktaBuyerRequest;
