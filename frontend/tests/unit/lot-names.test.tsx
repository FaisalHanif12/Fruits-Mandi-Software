// The name of a lot (PRD FR-12.45, plan 71 L1 to L4, L8): the seller, the item and the counter that
// belongs to that seller and item, "Ehsan, Mango-2" / "احسان، آم-2". One place makes it for every
// screen: the shared `lotName` words, the item with its counter for a list that has the seller in a
// column of its own, the place of a change in an edit, and the lists of بکری. Urdu and English.

import { ReactLocalization } from '@fluent/react';
import { describe, expect, it } from 'vitest';
import type { BikriListRow } from '../../src/bindings/BikriListRow';
import type { EditPlace } from '../../src/bindings/EditPlace';
import { lotItemNames, lotNamesOf } from '../../src/features/bikri/list-columns';
import { LANGUAGES, bundles, type Language } from '../../src/i18n';
import { placeText, type ValueContext } from '../../src/shared/edit/value-text';
import { lotItemText, lotNameText } from '../../src/shared/i18n';
import { listRow } from '../helpers/bikri-saved';
import { visible } from '../helpers/render';

const EHSAN = { id: 1, code: '0010', name_ur: 'احسان', name_en: 'Ehsan' };
const MANGO = { id: 2, code: '0040', name_ur: 'آم', name_en: 'Mango' };
const BANANA = { id: 3, code: '0041', name_ur: 'کیلا', name_en: 'Banana' };

describe.each(LANGUAGES)('the name of a lot (%s)', (language: Language) => {
  const l10n = new ReactLocalization([bundles[language]]);
  const say = (id: string, vars?: Record<string, string>) => l10n.getString(id, vars);
  const joined = language === 'ur' ? '، ' : ', ';
  const ehsan = language === 'ur' ? 'احسان' : 'Ehsan';
  const mango = language === 'ur' ? 'آم' : 'Mango';
  const banana = language === 'ur' ? 'کیلا' : 'Banana';

  it('is the seller, the item and the counter, with no isolation marks inside it', () => {
    const text = lotNameText(l10n, language, EHSAN, MANGO, 2);
    expect(text).toBe(`${ehsan}${joined}${mango}-2`);
    expect(text).not.toMatch(/[⁦-⁩]/);
  });

  it('keeps the first lot numbered too: -1', () => {
    expect(lotNameText(l10n, language, EHSAN, BANANA, 1)).toBe(`${ehsan}${joined}${banana}-1`);
  });

  it('shows the names alone while the backend has not given a counter', () => {
    expect(lotNameText(l10n, language, EHSAN, MANGO, null)).toBe(`${ehsan}${joined}${mango}`);
    expect(lotNameText(l10n, language, EHSAN, MANGO, undefined)).toBe(`${ehsan}${joined}${mango}`);
  });

  it('shows the one name that is chosen, and nothing while neither is', () => {
    expect(lotNameText(l10n, language, EHSAN, null, null)).toBe(ehsan);
    expect(lotNameText(l10n, language, null, MANGO, null)).toBe(mango);
    expect(lotNameText(l10n, language, null, null, null)).toBe('');
  });

  it('puts the tag after the seller, as every name does', () => {
    const text = lotNameText(l10n, language, { ...EHSAN, tag: 'K' }, MANGO, 3);
    expect(text).toBe(`${ehsan} K${joined}${mango}-3`);
  });

  it('is the item with its counter alone for a list with the seller in a column of its own', () => {
    expect(lotItemText(l10n, language, MANGO, 2)).toBe(`${mango}-2`);
    expect(lotItemText(l10n, language, MANGO, null)).toBe(mango);
  });

  it('starts the sentence of a change of a lot in an edit with its number and its name', () => {
    const ctx: ValueContext = { l10n, language, grouping: 'international' };
    const place: EditPlace = {
      kind: 'lot',
      lot_no: 2,
      lot_counter: 2,
      item_ur: 'آم',
      item_en: 'Mango',
      line_no: null,
      label_ur: 'احسان',
      label_en: 'Ehsan',
    };
    const words = visible(placeText(place, ctx));
    expect(words).toContain(`${ehsan}${joined}${mango}-2`);
    expect(words).toContain('2');
    // Without a counter or without the item the number stands alone, as before.
    const bare = visible(placeText({ ...place, lot_counter: null }, ctx));
    expect(bare).not.toContain(mango);
  });

  // A بکری of three lots: Ehsan's mango (his 2nd), Ali's banana (his 1st) and Ehsan's mango again (his 3rd).
  const row: BikriListRow = {
    ...listRow(),
    lots: [
      { lot_id: 1, seller: EHSAN, item: MANGO, counter: 2 },
      {
        lot_id: 2,
        seller: { ...EHSAN, id: 9, name_ur: 'علی', name_en: 'Ali' },
        item: BANANA,
        counter: 1,
      },
      { lot_id: 3, seller: EHSAN, item: MANGO, counter: 3 },
    ],
  };
  const ali = language === 'ur' ? 'علی' : 'Ali';
  const between = language === 'ur' ? '، ' : ', ';

  it('lists the items of a بکری one for each lot with its counter, in lot order', () => {
    expect(lotItemNames(row, language, say)).toBe(
      [`${mango}-2`, `${banana}-1`, `${mango}-3`].join(between),
    );
  });

  it('lists the lot names of a بکری in lot order', () => {
    expect(visible(lotNamesOf(row, language, say))).toContain(`${ehsan}${joined}${mango}-2`);
    expect(visible(lotNamesOf(row, language, say))).toContain(`${ali}${joined}${banana}-1`);
    expect(visible(lotNamesOf(row, language, say))).toContain(`${ehsan}${joined}${mango}-3`);
  });
});
