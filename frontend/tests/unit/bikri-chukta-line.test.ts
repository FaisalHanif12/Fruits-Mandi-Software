// Plan 64, PRD FR-12.5, FR-13.1, FR-13.5: a sale line to a چکتا buyer. The buyer picker gives the
// line one chosen record, so the line has no name or phone cells of its own (and no extra Enter
// stop). The draft sends the line to the account چکتا کھاتہ (`buyer_account_id`, the id every
// چکتا record shares) with his names and phone in `chukta`; an ordinary buyer sends `chukta: null`.
// Read back from a saved line, he is the same chosen record, drawn from the names and phone the line
// holds. Against the backend's own Event 2 request (Shahid, PRD 4.5).

import { describe, expect, it } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import { buildDraft, isChuktaBuyer } from '../../src/features/bikri/draft';
import { applyBackendDraft, type DraftRecords } from '../../src/features/bikri/draft-load';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { captureBaseline, diffEdit } from '../../src/features/bikri/edit-changes';
import { SALE } from '../../src/features/bikri/types';
import { cardChuktaOf, pickedOf } from '../../src/shared/accounts';
import { LINE_GRID_FIELD, saysEmpty } from '../../src/features/bikri/problems';
import { SAVE_EVENT2_REQUEST } from '../helpers/bikri-real';
import { withDateOnlyHeader } from '../helpers/bikri-old-draft';

/** Every id of the draft found as a record named after its id. */
const RECORDS: DraftRecords = {
  account: (id) =>
    id === null
      ? null
      : { id, code: `c${String(id)}`, name_ur: `ur${String(id)}`, name_en: `en${String(id)}` },
  master: (scope, id) =>
    id === null
      ? null
      : {
          id,
          code: `${scope}${String(id)}`,
          name_ur: `ur-${scope}-${String(id)}`,
          name_en: `en-${scope}-${String(id)}`,
        },
};

function loaded(draft: BikriDraft = SAVE_EVENT2_REQUEST.draft) {
  const stores = createBikriDraft('10-10-2026');
  applyBackendDraft(stores, structuredClone(draft), RECORDS);
  return stores;
}

const SHAHID_LINE = SAVE_EVENT2_REQUEST.draft.lots[0]?.sales[0];

describe('a sale line to a چکتا buyer', () => {
  it('the real Event 2 request goes in and builds back the same draft', () => {
    expect(buildDraft(loaded())).toEqual(withDateOnlyHeader(SAVE_EVENT2_REQUEST.draft));
  });

  it('is the account چکتا کھاتہ with his names and phone, as the backend wrote it', () => {
    expect(SHAHID_LINE?.buyer_account_id).toBe(9);
    expect(SHAHID_LINE?.chukta).toEqual({
      name_en: 'Shahid',
      name_ur: 'شاہد',
      phone: '0300-0000000',
    });
  });

  it('read back, the buyer is one chosen record with his own names, so the line has no extra cells', () => {
    const stores = loaded();
    const sales = stores.salesFor(stores.lots.getState().rows[0]?.rowId ?? '');
    const row = sales.getState().rows[0];
    expect(row).toBeDefined();
    if (row === undefined) return;
    expect(isChuktaBuyer(row)).toBe(true);
    expect(row.picked?.[SALE.buyer]).toMatchObject({
      id: 9,
      name_ur: 'شاہد',
      name_en: 'Shahid',
      chukta: { id: null, phone: '0300-0000000' },
    });
    // The party card of a saved line is drawn from the line (no record id of him is kept).
    expect(cardChuktaOf(row.picked?.[SALE.buyer])).toEqual({
      id: null,
      name_ur: 'شاہد',
      name_en: 'Shahid',
      phone: '0300-0000000',
    });
    // His names and phone are not typed cells: nothing but the buyer, qty and rate are on the line.
    expect(Object.keys(row.typed).filter((key) => /name|phone|chukta/i.test(key))).toEqual([]);
  });

  it('a buyer picked from the list (a چکتا hit) builds the line the same way', () => {
    const stores = loaded();
    const sales = stores.salesFor(stores.lots.getState().rows[0]?.rowId ?? '');
    const row = sales.getState().rows[0];
    if (row === undefined) return;
    // The picker's hit: the shared account 0030 (id 9) and the person's own record (chukta_id 1).
    const picked = pickedOf({
      id: 9,
      code: '0030',
      name_ur: 'ناصر',
      name_en: 'Nasir',
      kind: 'chukta',
      chukta_id: 2,
      phone: '03001111111',
    });
    sales.getState().updatePicked(row.rowId, SALE.buyer, picked);
    const line = buildDraft(stores).lots[0]?.sales[0];
    expect(line?.buyer_account_id).toBe(9);
    expect(line?.chukta).toEqual({ name_ur: 'ناصر', name_en: 'Nasir', phone: '03001111111' });
    expect(cardChuktaOf(picked)).toEqual({
      id: 2,
      name_ur: 'ناصر',
      name_en: 'Nasir',
      phone: '03001111111',
    });
  });

  it('an ordinary buyer sends no chukta part, and is not a چکتا buyer', () => {
    const stores = loaded();
    const sales = stores.salesFor(stores.lots.getState().rows[0]?.rowId ?? '');
    const row = sales.getState().rows[0];
    if (row === undefined) return;
    sales
      .getState()
      .updatePicked(
        row.rowId,
        SALE.buyer,
        pickedOf({ id: 20, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' }),
      );
    const line = buildDraft(stores).lots[0]?.sales[0];
    expect(line?.buyer_account_id).toBe(20);
    expect(line?.chukta).toBeNull();
    expect(isChuktaBuyer(sales.getState().rows[0] ?? row)).toBe(false);
  });

  it('is no change when an edit opens it and nothing is touched', () => {
    const stores = loaded();
    expect(diffEdit(stores, captureBaseline(stores)).unchanged).toBe(true);
  });

  it('another چکتا buyer on the line is a change of buyer, told by his name', () => {
    const stores = loaded();
    const baseline = captureBaseline(stores);
    const sales = stores.salesFor(stores.lots.getState().rows[0]?.rowId ?? '');
    const row = sales.getState().rows[0];
    if (row === undefined) return;
    sales.getState().updatePicked(
      row.rowId,
      SALE.buyer,
      pickedOf({
        id: 9,
        code: '0030',
        name_ur: 'ناصر',
        name_en: 'Nasir',
        kind: 'chukta',
        chukta_id: 2,
        phone: null,
      }),
    );
    const diff = diffEdit(stores, baseline);
    expect(diff.unchanged).toBe(false);
    expect(diff.items.some((item) => item.field === SALE.buyer)).toBe(true);
  });
});

describe('a refusal about a چکتا buyer of a line', () => {
  it('is shown on the buyer cell, because the line has no cells for his names and phone', () => {
    expect(LINE_GRID_FIELD['chukta.name_ur']).toBe('buyer');
    expect(LINE_GRID_FIELD['chukta.name_en']).toBe('buyer');
    expect(LINE_GRID_FIELD['chukta.phone']).toBe('buyer');
  });

  it('an empty name is held back until the user saves, like any empty cell; the phone is optional', () => {
    expect(saysEmpty('ERR_CHUKTA_NAME_REQUIRED')).toBe(true);
    expect(saysEmpty('ERR_PHONE_INVALID')).toBe(false);
  });
});
