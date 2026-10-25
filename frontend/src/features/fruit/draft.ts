// The text side of the پھل screen (Code Standards F39): the form as text in the screen's state, the
// input the backend takes, and the field paths its problems point at. Nothing here works out money
// (CLAUDE.md rule 1): a quantity and a rate are the text the user typed, the rate is never filled
// from a hint (PRD FR-7.53), and the backend reads and checks everything again on save.

import type { RecordRef } from '../../bindings/RecordRef';
import { pickedOf } from '../../shared/accounts';
import { formatGrams, formatPaisa, isoToDisplayDate, type Grouping } from '../../shared/format';
import type { PickerValue } from '../../shared/ui';
import { textOrNull } from '../purchases';
import {
  FOR_GUESTS,
  type ChosenLot,
  type FruitDraft,
  type FruitInput,
  type FruitLotRow,
  type FruitRow,
  type FruitSaveRequest,
  type EditingFruit,
} from './types';

/** The fields the form shows next to their inputs, so the notice does not tell a problem twice. */
export const FRUIT_FIELD_PATHS: readonly string[] = [
  'partner_id',
  'date',
  'lot_id',
  'agent_account_id',
  'item_id',
  'packing_id',
  'qty',
  'weight',
  'rate',
  'note',
  'reason',
];

/** A record the form holds for a picker: both names, so a language switch needs no ask (F38). */
export function valueOfRef(ref: RecordRef): PickerValue {
  return pickedOf(ref);
}

/**
 * The lot a picker row stands for. The field shows the item and the packing in both scripts and
 * the code is the بکری number and the lot number ("1-2").
 */
export function chosenLotOf(row: FruitLotRow): ChosenLot {
  return {
    value: {
      id: row.lot_id,
      code: `${String(row.bikri_number)}-${String(row.lot_no)}`,
      name_ur: `${row.item.name_ur} ${row.packing.name_ur}`,
      name_en: `${row.item.name_en} ${row.packing.name_en}`,
    },
    row,
  };
}

/** A new form for the day (`YYYY-MM-DD`, or empty while the day is not known yet). */
export function newFruitDraft(today: string): FruitDraft {
  return {
    forWhom: '',
    date: today === '' ? '' : isoToDisplayDate(today),
    source: 'shop',
    lot: null,
    agent: null,
    item: null,
    packing: null,
    qty: '',
    weight: '',
    rate: '',
    note: '',
    reason: '',
  };
}

/**
 * The other source: what belongs to the first one goes (the lot, or the agent, item and packing,
 * and the weight, which belongs to the item). What is typed for the goods stays: quantity, rate and
 * note.
 */
export function changeSource(draft: FruitDraft, source: FruitDraft['source']): FruitDraft {
  if (source === draft.source) return draft;
  return { ...draft, source, lot: null, agent: null, item: null, packing: null, weight: '' };
}

/**
 * After a save the next entry starts with the same day, the same person and the same source: the
 * munshi often records several takes one after another. The rest is empty.
 */
export function nextFruitDraft(previous: FruitDraft): FruitDraft {
  return {
    ...newFruitDraft(''),
    date: previous.date,
    forWhom: previous.forWhom,
    source: previous.source,
  };
}

/**
 * The form of a saved entry, to edit it (PRD FR-16.12). The rate is the saved one, put into the
 * visible field like a typed value; the backend reads it again on save (F43). The reason starts
 * empty.
 */
export function draftFromRow(row: FruitRow, grouping: Grouping): FruitDraft {
  const lot: ChosenLot | null =
    row.lot === null
      ? null
      : {
          value: {
            id: row.lot.lot_id,
            code: `${String(row.lot.bikri_number)}-${String(row.lot.lot_no)}`,
            name_ur: `${row.item.name_ur} ${row.packing.name_ur}`,
            name_en: `${row.item.name_en} ${row.packing.name_en}`,
          },
          row: null,
        };
  const other = row.source === 'other_shop';
  return {
    forWhom: row.partner === null ? FOR_GUESTS : String(row.partner.id),
    date: isoToDisplayDate(row.entry_date),
    source: row.source,
    lot,
    agent: other ? valueOfRef(row.credit_account) : null,
    item: other ? valueOfRef(row.item) : null,
    packing: other ? valueOfRef(row.packing) : null,
    qty: String(row.qty),
    weight: row.weight_grams === null ? '' : formatGrams(row.weight_grams),
    rate: formatPaisa(row.rate_paisa, { grouping }),
    note: row.note ?? '',
    reason: '',
  };
}

/** Who the entry is for, as the backend takes it: a partner id, or the guests. */
function partnerOf(draft: FruitDraft): number | null {
  if (draft.forWhom === '' || draft.forWhom === FOR_GUESTS) return null;
  const id = Number(draft.forWhom);
  return Number.isSafeInteger(id) ? id : null;
}

/**
 * What goes to the backend. The item and packing of fruit from our shop come from the lot (the
 * backend ignores them), so only fruit from another shop sends them. While an entry is edited its id
 * goes too, so the preview gives the entry's quantity back to the lot and its part back to the
 * partner's total (`fruit_edit` takes the entry from its own id).
 */
export function buildFruit(draft: FruitDraft, editingId: number | null = null): FruitInput {
  const guests = draft.forWhom === FOR_GUESTS;
  const shop = draft.source === 'shop';
  return {
    for_guests: guests,
    partner_id: guests ? null : partnerOf(draft),
    date: draft.date.trim(),
    source: draft.source,
    lot_id: shop ? (draft.lot?.value.id ?? null) : null,
    agent_account_id: shop ? null : (draft.agent?.id ?? null),
    item_id: shop ? null : (draft.item?.id ?? null),
    packing_id: shop ? null : (draft.packing?.id ?? null),
    qty: draft.qty.trim(),
    weight: textOrNull(draft.weight),
    rate: draft.rate.trim(),
    note: textOrNull(draft.note),
    editing_id: editingId,
  };
}

/** The request of a save: a new entry, or the edit of a saved one with its reason. */
export function buildSaveRequest(
  draft: FruitDraft,
  editing: EditingFruit | null,
): FruitSaveRequest {
  return {
    entry: buildFruit(draft, editing === null ? null : editing.id),
    edit:
      editing === null
        ? null
        : { id: editing.id, version: editing.version, reason: draft.reason.trim() },
  };
}

/** Has anything been typed or chosen? An empty form is not "unsaved work" (PRD FR-7.9). */
export function fruitIsFilled(draft: FruitDraft): boolean {
  return (
    draft.lot !== null ||
    draft.agent !== null ||
    draft.item !== null ||
    draft.packing !== null ||
    [draft.qty, draft.weight, draft.rate, draft.note, draft.reason].some(
      (text) => text.trim() !== '',
    )
  );
}

/**
 * Is there enough to ask the backend? It answers with the numbers when the form is complete and
 * with the last rate as a hint as soon as the item is known (PRD FR-7.53), so the rate field can
 * show the hint before the rate is typed. A form with nothing chosen asks nothing.
 */
export function readyToAsk(draft: FruitDraft): boolean {
  return draft.source === 'shop'
    ? draft.lot !== null
    : draft.agent !== null || (draft.item !== null && draft.packing !== null);
}
