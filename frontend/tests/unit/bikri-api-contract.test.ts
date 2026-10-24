// The screen and the backend agree on plan 13 (T4). The backend wrote, in its own tests, the
// request of each new command and its real answer (tests/fixtures/*.json, copied, never edited).
// This checks two things. First, each wrapper in api.ts sends the command name and the input of
// the backend's file. Second, the canned data the component tests use has the same shape as the
// real answers (the same keys at every level), so a test that passes on canned data cannot hide a
// field the real backend does not send, or one it sends and the canned data forgot.

import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import * as draftsApi from '../../src/shared/drafts/api';
import * as invoke from '../../src/shared/api/invoke';
import { searchGlobal } from '../../src/shared/accounts/api';
import { listRow, registerRow, savedEvent1, SHAHID } from '../helpers/bikri-saved';
import {
  DRAFT_ROW,
  GET_EVENT1,
  GET_EVENT2,
  GLOBAL_TRUCK,
  LIST_DAY,
  LIST_DAY_REQUEST,
  PEOPLE_SHAHID,
  REGISTER_DAY,
  REGISTER_DAY_REQUEST,
  SAVED_EVENT1,
  SAVED_EVENT2,
  SAVE_EVENT1_REQUEST,
  SAVE_EVENT2_REQUEST,
} from '../helpers/bikri-real';
import getEvent1Request from '../fixtures/bikri_get_event1_request.json';
import draftsGetRequest from '../fixtures/drafts_get_bikri_request.json';
import draftsSaveRequest from '../fixtures/drafts_save_bikri_request.json';
import draftsSaved from '../fixtures/drafts_save_bikri.json';
import globalTruckRequest from '../fixtures/search_global_bikri_truck_request.json';
import peopleRequest from '../fixtures/search_pukhta_people_shahid_request.json';

vi.mock('../../src/shared/api/invoke');

const call = vi.mocked(invoke.callCommand);

/** Answers every command with one answer, so a test can look at what was sent. */
function answers(answer: unknown) {
  call.mockResolvedValue(answer);
}

beforeEach(() => {
  vi.resetAllMocks();
});

describe('each wrapper sends the command and the input of the backend’s file', () => {
  it('bikri_save: the draft as typed and the cash confirmation, always sent', async () => {
    answers(SAVED_EVENT1);
    const saved = await bikriApi.bikriSave(
      SAVE_EVENT1_REQUEST.draft,
      SAVE_EVENT1_REQUEST.confirm_cash_below_zero,
    );
    expect(call).toHaveBeenCalledWith('bikri_save', { input: SAVE_EVENT1_REQUEST });
    expect(SAVE_EVENT1_REQUEST.confirm_cash_below_zero).toBe(false);
    expect(saved).toEqual(SAVED_EVENT1);
    // Event 2 carries the one-time person on the line.
    answers(SAVED_EVENT2);
    await bikriApi.bikriSave(SAVE_EVENT2_REQUEST.draft, true);
    expect(call).toHaveBeenLastCalledWith('bikri_save', {
      input: { draft: SAVE_EVENT2_REQUEST.draft, confirm_cash_below_zero: true },
    });
    expect(SAVE_EVENT2_REQUEST.draft.lots[0]?.sales[0]?.pukhta).toEqual({
      name_ur: 'شاہد',
      name_en: 'Shahid',
      phone: '0300-0000000',
    });
  });

  it('bikri_get: the id', async () => {
    answers(GET_EVENT1);
    await bikriApi.bikriGet(getEvent1Request.id);
    expect(call).toHaveBeenCalledWith('bikri_get', { input: getEvent1Request });
  });

  it('bikri_list: the day as typed, the other filters empty', async () => {
    answers(LIST_DAY);
    await bikriApi.bikriList({
      date: '10-10-2026',
      from: null,
      to: null,
      seller_id: null,
      item_id: null,
      status: null,
      offset: 0,
      limit: 0,
    });
    expect(call).toHaveBeenCalledTimes(1);
    expect(call.mock.calls[0]?.[0]).toBe('bikri_list');
    expect(call.mock.calls[0]?.[1]).toMatchObject({ input: LIST_DAY_REQUEST });
  });

  it('search_pukhta_people and pukhta_register', async () => {
    answers(PEOPLE_SHAHID);
    await pukhtaApi.searchPukhtaPeople(peopleRequest.text, peopleRequest.side as 'buyer');
    expect(call).toHaveBeenCalledWith('search_pukhta_people', { input: peopleRequest });
    answers(REGISTER_DAY);
    await pukhtaApi.pukhtaRegister({
      from: '10-10-2026',
      to: '10-10-2026',
      side: null,
      name: null,
      phone: null,
      state: null,
      offset: 0,
      limit: 0,
    });
    expect(call.mock.calls.at(-1)?.[0]).toBe('pukhta_register');
    expect(call.mock.calls.at(-1)?.[1]).toMatchObject({ input: REGISTER_DAY_REQUEST });
  });

  it('drafts_save, drafts_get: the screen key is "bikri" and the payload is sent as text', async () => {
    answers(draftsSaved);
    await draftsApi.draftsSave(draftsSaveRequest.screen, draftsSaveRequest.payload_json);
    expect(call).toHaveBeenCalledWith('drafts_save', { input: draftsSaveRequest });
    answers(DRAFT_ROW);
    expect(await draftsApi.draftsGet(draftsGetRequest.screen)).toEqual(DRAFT_ROW);
    expect(call).toHaveBeenLastCalledWith('drafts_get', { input: draftsGetRequest });
  });

  it('search_global: asks for a short list and keeps the بکری hit with its number and truck', async () => {
    answers(GLOBAL_TRUCK);
    const found = await searchGlobal(globalTruckRequest.text);
    expect(call.mock.calls[0]?.[0]).toBe('search_global');
    expect(call.mock.calls[0]?.[1]).toMatchObject({ input: { text: globalTruckRequest.text } });
    expect(found).toEqual([
      {
        kind: 'bikri',
        id: 1,
        number: 1,
        entry_date: '2026-10-10',
        truck_no: 'LES-1234',
        status: 'saved',
        matched: 'truck',
      },
    ]);
  });
});

type Shape = string | null | Shape[] | { [key: string]: Shape };

/** The keys and kinds of a value at every level; a list is shown by its first element. */
function shapeOf(value: unknown): Shape {
  if (value === null || value === undefined) return null;
  if (Array.isArray(value)) return value.length === 0 ? [] : [shapeOf(value[0])];
  if (typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, item]) => [key, shapeOf(item)]),
    );
  }
  return typeof value;
}

/** The first difference between two shapes, or null. A null or an empty list can hold anything. */
function firstDifference(real: Shape, canned: Shape, path = ''): string | null {
  if (real === null || canned === null) return null;
  if (Array.isArray(real) || Array.isArray(canned)) {
    if (!Array.isArray(real) || !Array.isArray(canned)) return `${path}: list against not a list`;
    const [a, b] = [real[0], canned[0]];
    return a === undefined || b === undefined ? null : firstDifference(a, b, `${path}[]`);
  }
  if (typeof real === 'string' || typeof canned === 'string') {
    return real === canned
      ? null
      : `${path}: ${JSON.stringify(real)} against ${JSON.stringify(canned)}`;
  }
  const keys = new Set([...Object.keys(real), ...Object.keys(canned)]);
  for (const key of [...keys].sort()) {
    const [a, b] = [real[key], canned[key]];
    if (a === undefined) return `${path}.${key}: the real answer has no such field`;
    if (b === undefined) return `${path}.${key}: the canned data has no such field`;
    const inner = firstDifference(a, b, `${path}.${key}`);
    if (inner !== null) return inner;
  }
  return null;
}

describe('the canned data of the component tests has the real shape', () => {
  // The comparison itself: it must see a missing field, an extra field and a changed kind.
  it('the comparison sees a missing field, an extra field and a different kind', () => {
    expect(firstDifference(shapeOf({ a: 1, b: 2 }), shapeOf({ a: 1 }))).toContain('.b');
    expect(firstDifference(shapeOf({ a: 1 }), shapeOf({ a: 1, c: 2 }))).toContain('.c');
    expect(firstDifference(shapeOf({ a: 1 }), shapeOf({ a: 'x' }))).toContain('.a');
    expect(firstDifference(shapeOf({ a: [{ k: 1 }] }), shapeOf({ a: [{ j: 1 }] }))).not.toBeNull();
    expect(firstDifference(shapeOf({ a: null }), shapeOf({ a: 1 }))).toBeNull();
  });

  it('a saved بکری (Event 1): every field, lot, sale, charge, posting, split and totals', () => {
    expect(firstDifference(shapeOf(GET_EVENT1), shapeOf(savedEvent1()))).toBeNull();
  });

  it('a saved بکری (Event 2) has the same shape as Event 1, with the one-time buyer on the line', () => {
    expect(firstDifference(shapeOf(GET_EVENT2), shapeOf(GET_EVENT1))).toBeNull();
    expect(GET_EVENT2.lots[0]?.sales[0]?.pukhta).toMatchObject({ name_en: 'Shahid' });
  });

  it('a list row', () => {
    expect(firstDifference(shapeOf(LIST_DAY.rows[0]), shapeOf(listRow()))).toBeNull();
    // A page: the rows, the count of the whole filter, and the totals over all of it.
    expect(Object.keys(LIST_DAY).sort()).toEqual(['rows', 'total_count', 'totals']);
    expect(Object.keys(LIST_DAY.totals).sort()).toEqual([
      'buyer_total',
      'gross',
      'net',
      'qty_received',
      'qty_sold',
    ]);
  });

  it('a register row and a one-time person', () => {
    expect(firstDifference(shapeOf(REGISTER_DAY.rows[0]), shapeOf(registerRow()))).toBeNull();
    expect(Object.keys(REGISTER_DAY.totals).sort()).toEqual([
      'amount',
      'commission',
      'open',
      'qty',
    ]);
    expect(firstDifference(shapeOf(PEOPLE_SHAHID[0]), shapeOf(SHAHID))).toBeNull();
  });

  it('the result of a save has the same keys, and the preview inside it has the preview’s keys', () => {
    expect(Object.keys(SAVED_EVENT1).sort()).toEqual(['bikri_id', 'entry_id', 'number', 'preview']);
    expect(
      firstDifference(shapeOf(SAVED_EVENT1.preview), shapeOf(SAVED_EVENT2.preview)),
    ).toBeNull();
  });
});

describe('what the real answers say, read as the screen reads them', () => {
  it('Event 1 is saved as number 1 and Event 2 as number 2', () => {
    expect([SAVED_EVENT1.number, SAVED_EVENT2.number]).toEqual([1, 2]);
    expect(GET_EVENT1.totals.net).toBe(1_931_600);
    expect(GET_EVENT2.totals.net).toBe(1_016_000);
  });

  it('the list is newest first: number 2, then number 1', () => {
    expect(LIST_DAY.rows.map((row) => row.number)).toEqual([2, 1]);
    expect(LIST_DAY.total_count).toBe(2);
  });
});
