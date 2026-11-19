// The words of one value in a list of changes (plan 44, PRD FR-7.70): "600", "5 kg", "12.5%",
// "10-10-2026", the name in the language of the app, "Yes". Display only. A money value goes through
// `formatPaisa` and nothing else; no figure is added or rounded here (CLAUDE.md rule 1).

import type { ReactLocalization } from '@fluent/react';
import type { AuditValue } from '../../bindings/AuditValue';
import type { ChangeValue } from '../../bindings/ChangeValue';
import type { EditPlace } from '../../bindings/EditPlace';
import { formatBps, formatGrams, formatPaisa, isoToDisplayDate, type Grouping } from '../format';
import type { Language } from '../../i18n';
import { displayName, lotNameText } from '../i18n';

/** What the words of a value need: the messages, the language and the number grouping. */
export interface ValueContext {
  l10n: ReactLocalization;
  language: Language;
  grouping: Grouping;
}

/** The Fluent message of a field of an edit: the backend's key with dashes. */
export function fieldMessageId(key: string): string {
  return `edit-field-${key.replaceAll('_', '-')}`;
}

/** The name of a field in words, or a plain "other field" for a key the app has no words for. */
export function fieldText(key: string, ctx: ValueContext): string {
  const id = fieldMessageId(key);
  return ctx.l10n.getString(id, null, ctx.l10n.getString('edit-field-other'));
}

/** One value of a change as words. */
export function changeValueText(value: ChangeValue, ctx: ValueContext): string {
  switch (value.type) {
    case 'empty':
      return ctx.l10n.getString('edit-empty');
    case 'text':
      return value.text.trim() === '' ? ctx.l10n.getString('edit-empty') : value.text;
    case 'number':
      return String(value.value);
    case 'money':
      return formatPaisa(value.paisa, { grouping: ctx.grouping });
    case 'weight':
      return formatGrams(value.grams);
    case 'bps':
      return `${formatBps(value.bps)}${ctx.l10n.getString('common-percent-sign')}`;
    case 'date':
      return isoToDisplayDate(value.iso);
    case 'names':
      return displayName({ name_ur: value.ur, name_en: value.en }, ctx.language);
    case 'flag':
      return ctx.l10n.getString(value.on ? 'common-yes' : 'common-no');
    case 'choice':
      // A fixed word the backend names (for example "credit" or "cash"): its own message when the
      // app has one, else the word as it is.
      return ctx.l10n.getString(`edit-choice-${value.word}`, null, value.word);
  }
}

/** One value of the audit log (the history of an entry) as words. */
export function auditValueText(value: AuditValue, ctx: ValueContext): string {
  switch (value.kind) {
    case 'null':
      return ctx.l10n.getString('edit-empty');
    case 'bool':
      return ctx.l10n.getString(value.value ? 'common-yes' : 'common-no');
    case 'number':
      return String(value.value);
    case 'money':
      return formatPaisa(value.paisa, { grouping: ctx.grouping });
    case 'text':
      return value.text.trim() === '' ? ctx.l10n.getString('edit-empty') : value.text;
    case 'hidden':
      return '••••';
  }
}

/** The name of the lot of a place: the seller (its label), the item and the counter, or ''. */
function lotNameOfPlace(place: EditPlace, ctx: ValueContext): string {
  if (place.lot_counter === null || place.item_ur === null || place.item_en === null) return '';
  if (place.label_ur === null || place.label_en === null) return '';
  return lotNameText(
    ctx.l10n,
    ctx.language,
    { name_ur: place.label_ur, name_en: place.label_en },
    { name_ur: place.item_ur, name_en: place.item_en },
    place.lot_counter,
  );
}

/** The place of a change as the start of a sentence: "Lot 1, line 2 (Akram Toll Tax):" or ''. */
export function placeText(place: EditPlace, ctx: ValueContext): string {
  const named =
    place.label_ur === null && place.label_en === null
      ? ''
      : displayName({ name_ur: place.label_ur ?? '', name_en: place.label_en ?? '' }, ctx.language);
  const label = named === '' ? '' : ` (${named})`;
  const lot = String(place.lot_no ?? 0);
  switch (place.kind) {
    case 'entry':
      return '';
    case 'header':
      return `${ctx.l10n.getString('edit-review-where-header')} `;
    case 'lot': {
      // The lot's name with its counter, "Ehsan, Mango-2" (PRD FR-12.45), when the backend gave the
      // seller, the item and the counter; else the number alone.
      const name = lotNameOfPlace(place, ctx);
      return `${ctx.l10n.getString('edit-review-where-lot', { lot, name: name === '' ? '' : ` (${name})` })} `;
    }
    case 'line':
      return `${ctx.l10n.getString('edit-review-where-line', {
        lot,
        line: String(place.line_no ?? 0),
        label,
      })} `;
    case 'charge':
      return `${ctx.l10n.getString('edit-review-where-charge', { label })} `;
    case 'part':
      return `${ctx.l10n.getString('edit-review-where-part', { label })} `;
  }
}

/**
 * The fields the app works out again after an edit (commission, totals): the backend lists them with
 * the changes, and the review shows them under the changes the user typed. Display only.
 */
const WORKED_OUT = new Set([
  'seller_comm',
  'buyer_comm',
  'laga',
  'seller_amount',
  'buyer_value',
  'buyer_total',
  'gross',
  'net',
  'deductions',
  'value',
]);

export function isWorkedOut(field: string): boolean {
  return WORKED_OUT.has(field);
}
