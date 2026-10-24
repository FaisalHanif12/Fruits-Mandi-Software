// The masters commands' own row and input types against the screens' shape (plan 06 T4): each of
// the nine kinds in and out, and what the form does not edit travels back unchanged.

import { describe, expect, it } from 'vitest';
import type { MasterRow } from '../../src/bindings/MasterRow';
import { draftToInput, rowToRecord } from '../../src/features/masters/wire';
import { configOfKind } from '../../src/features/masters/configs';
import { valuesOf } from '../../src/features/masters/draft';
import { at } from '../helpers/sample';

const names = { name_ur: 'کوٹ', name_en: 'Kot' };

const ROWS: MasterRow[] = [
  { kind: 'city', id: 1, version: 3, code: '0001', is_active: true, ...names },
  {
    kind: 'item',
    id: 2,
    version: 1,
    code: '0002',
    is_active: true,
    ...names,
    item_group: 'fruit',
    default_packing_id: 12,
    default_packing_name_ur: 'کریٹ',
    default_packing_name_en: 'Crate',
    sold_by: 'unit',
  },
  { kind: 'packing', id: 3, version: 1, code: '0003', is_active: true, ...names, approx_kg: '20' },
  {
    kind: 'adda',
    id: 4,
    version: 1,
    code: '0004',
    is_active: true,
    ...names,
    city_id: 1,
    city_name_ur: 'لاہور',
    city_name_en: 'Lahore',
  },
  {
    kind: 'marka',
    id: 5,
    version: 1,
    code: '0005',
    is_active: true,
    mark_text: 'AR',
    owner_account_id: 31,
    owner_account_code: '030110001',
    owner_name_ur: 'رحیم',
    owner_name_en: 'Rahim',
  },
  { kind: 'store', id: 6, version: 1, code: '0006', is_active: true, ...names },
  {
    kind: 'narration',
    id: 7,
    version: 1,
    is_active: true,
    text_ur: 'بجلی کا بل',
    text_en: 'Electricity bill',
    used_in: ['shop_expense', 'payment'],
  },
  {
    kind: 'expense_head',
    id: 8,
    version: 1,
    code: '0008',
    is_active: true,
    ...names,
    account_id: 80,
    account_code: '05010008',
  },
  {
    kind: 'staff',
    id: 9,
    version: 1,
    is_active: true,
    ...names,
    job: 'munshi',
    phone: null,
    account_id: null,
    account_code: null,
  },
];

describe('rowToRecord', () => {
  it('keeps the id, version, code and state of every kind', () => {
    for (const row of ROWS) {
      const record = rowToRecord(row);
      expect(record).toMatchObject({
        kind: row.kind,
        id: row.id,
        version: row.version,
        is_active: true,
      });
    }
  });

  it('puts the names of a master in one place, and a mark has none', () => {
    expect(rowToRecord(at(ROWS, 0)).names).toEqual(names);
    expect(rowToRecord(at(ROWS, 4)).names).toBeNull();
    // A narration is two texts: they are its two names for the screen, and it has no code.
    expect(rowToRecord(at(ROWS, 6))).toMatchObject({
      code: '',
      names: { name_ur: 'بجلی کا بل', name_en: 'Electricity bill' },
    });
  });

  it('turns the flat fields of a kind into text, picked records and read-only facts', () => {
    expect(rowToRecord(at(ROWS, 1))).toMatchObject({
      text: { item_group: 'fruit', sold_by: 'unit' },
      refs: { default_packing: { id: 12, name_ur: 'کریٹ', name_en: 'Crate' } },
    });
    expect(rowToRecord(at(ROWS, 2)).text).toEqual({ approx_kg: '20' });
    expect(rowToRecord(at(ROWS, 3)).refs['city']).toMatchObject({
      id: 1,
      name_en: 'Lahore',
    });
    expect(rowToRecord(at(ROWS, 7)).shown).toEqual({ account_code: '05010008' });
    expect(rowToRecord(at(ROWS, 8))).toMatchObject({
      text: { job: 'munshi', phone: '' },
      shown: { account_code: '' },
    });
  });

  it('a packing with no weight and an item with no packing come out empty', () => {
    expect(
      rowToRecord({
        kind: 'packing',
        id: 1,
        version: 1,
        code: '1',
        is_active: true,
        ...names,
        approx_kg: null,
      }).text['approx_kg'],
    ).toBe('');
    expect(
      rowToRecord({
        kind: 'item',
        id: 1,
        version: 1,
        code: '1',
        is_active: true,
        ...names,
        item_group: 'fruit',
        default_packing_id: null,
        default_packing_name_ur: null,
        default_packing_name_en: null,
        sold_by: 'unit',
      }).refs['default_packing'],
    ).toBeNull();
  });
});

describe('draftToInput', () => {
  const draftOf = (row: MasterRow) => valuesOf(configOfKind(row.kind), rowToRecord(row));

  it('sends back what a save of every kind needs, with the id and the version that were read', () => {
    for (const row of ROWS) {
      const input = draftToInput(row.kind, draftOf(row), ['x']);
      expect(input).toMatchObject({ kind: row.kind, id: row.id, version: row.version });
    }
  });

  it('a city sends its two names and the confirmed words', () => {
    expect(draftToInput('city', draftOf(at(ROWS, 0)), ['ظہیر'])).toEqual({
      kind: 'city',
      id: 1,
      version: 3,
      code: '0001',
      name_ur: 'کوٹ',
      name_en: 'Kot',
      confirmed_rules_words: ['ظہیر'],
    });
  });

  it('an item sends its group, how it is sold and the id of its default packing', () => {
    expect(draftToInput('item', draftOf(at(ROWS, 1)), [])).toMatchObject({
      item_group: 'fruit',
      sold_by: 'unit',
      default_packing_id: 12,
    });
  });

  it('a mark keeps its owner although the form has no field for him, and sends no names', () => {
    const input = draftToInput('marka', draftOf(at(ROWS, 4)), []);
    expect(input).toEqual({
      kind: 'marka',
      id: 5,
      version: 1,
      code: '0005',
      mark_text: 'AR',
      owner_account_id: 31,
    });
  });

  it('a narration keeps its "used in" list and sends no spelling words', () => {
    const input = draftToInput('narration', draftOf(at(ROWS, 6)), []);
    expect(input).toEqual({
      kind: 'narration',
      id: 7,
      version: 1,
      text_ur: 'بجلی کا بل',
      text_en: 'Electricity bill',
      used_in: ['shop_expense', 'payment'],
    });
  });

  it('a staff member sends his job and phone and his names', () => {
    expect(draftToInput('staff', draftOf(at(ROWS, 8)), [])).toMatchObject({
      job: 'munshi',
      phone: '',
    });
    expect(draftToInput('staff', draftOf(at(ROWS, 8)), [])).toMatchObject({ name_ur: 'کوٹ' });
  });

  it('a packing sends its weight as the text typed', () => {
    const draft = draftOf(at(ROWS, 2));
    expect(draftToInput('packing', { ...draft, text: { approx_kg: '25' } }, [])).toMatchObject({
      approx_kg: '25',
    });
  });

  it('a new master sends no id and no version', () => {
    const draft = draftOf(at(ROWS, 0));
    expect(draftToInput('city', { ...draft, id: null, version: null }, [])).toMatchObject({
      id: null,
      version: null,
    });
  });
});
