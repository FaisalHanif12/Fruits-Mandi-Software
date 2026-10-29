// The lots of our shop that have goods left, as a source for the shared picker (PRD FR-16.11):
// typing searches them, an empty field lists them. Trip goods and the lots of a closed garden are
// never in the answer; the backend decides, the screen filters nothing.

import { PICKER_RESULT_LIMIT, type PickerSource } from '../../shared/search';
import type { PickerHit } from '../../shared/ui';
import { fruitLots } from './api';
import type { FruitLotRow } from './types';

/** A lot in the picker: the hit the picker needs, with the row the screen needs. */
export interface LotHit extends PickerHit {
  row: FruitLotRow;
}

/** The code of a lot is the بکری number and the lot number: "1-2". */
export function lotCode(row: Pick<FruitLotRow, 'bikri_number' | 'lot_no'>): string {
  return `${String(row.bikri_number)}-${String(row.lot_no)}`;
}

function hitOf(row: FruitLotRow): LotHit {
  // The one-time seller of a پختہ کھاتہ lot is told by his own name.
  const seller = row.pukhta ?? row.seller;
  return {
    id: row.lot_id,
    code: lotCode(row),
    name_ur: `${row.item.name_ur} ${row.packing.name_ur}`,
    name_en: `${row.item.name_en} ${row.packing.name_en}`,
    extra_ur: seller.name_ur,
    extra_en: seller.name_en,
    is_active: true,
    row,
  };
}

/** The picker source of the lots with goods left. A lot is not "remembered" when chosen. */
export const LOT_SOURCE: PickerSource<LotHit> = {
  key: ['fruit', 'lots'],
  search: (text, limit) =>
    fruitLots({ search: text, offset: 0, limit }).then((answer) => answer.rows.map(hitOf)),
  recent: () =>
    fruitLots({ search: null, offset: 0, limit: PICKER_RESULT_LIMIT }).then((answer) =>
      answer.rows.map(hitOf),
    ),
  touch: () => Promise.resolve(null),
};
