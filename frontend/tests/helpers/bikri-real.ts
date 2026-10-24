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
import registerDay from '../fixtures/pukhta_register_day.json';
import registerDayBuyers from '../fixtures/pukhta_register_day_buyers.json';
import registerDayBuyersRequest from '../fixtures/pukhta_register_day_buyers_request.json';
import registerDayRequest from '../fixtures/pukhta_register_day_request.json';
import peopleShahid from '../fixtures/search_pukhta_people_shahid.json';
import type { BikriDetail } from '../../src/bindings/BikriDetail';
import type { BikriList } from '../../src/bindings/BikriList';
import type { BikriSaveInput } from '../../src/bindings/BikriSaveInput';
import type { BikriSaved } from '../../src/bindings/BikriSaved';
import type { DraftRow } from '../../src/bindings/DraftRow';
import type { GlobalHit } from '../../src/bindings/GlobalHit';
import type { PukhtaPerson } from '../../src/bindings/PukhtaPerson';
import type { PukhtaRegister } from '../../src/bindings/PukhtaRegister';

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
function registerOf(value: unknown): PukhtaRegister {
  return value as PukhtaRegister;
}
function peopleOf(value: unknown): PukhtaPerson[] {
  return value as PukhtaPerson[];
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
export const REGISTER_DAY: PukhtaRegister = registerOf(registerDay);
export const REGISTER_DAY_REQUEST: Record<string, unknown> = registerDayRequest;
/** The register of the buyers only: with a side chosen the totals are given (PRD FR-19.11). */
export const REGISTER_DAY_BUYERS: PukhtaRegister = registerOf(registerDayBuyers);
export const REGISTER_DAY_BUYERS_REQUEST: Record<string, unknown> = registerDayBuyersRequest;
export const PEOPLE_SHAHID: PukhtaPerson[] = peopleOf(peopleShahid);
