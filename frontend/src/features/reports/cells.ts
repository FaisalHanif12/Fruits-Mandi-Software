// A cell of a report as the words and figures of the screen (PRD 19.1, FR-19.8, FR-19.13). Every
// amount, count and date is what the backend sent; this file only writes it: money through
// `formatPaisa` (the list context's `money`), names through `displayName` (Code Standards F38), and
// the words around the goods of a ledger line from Fluent. It adds up nothing (CLAUDE.md rule 1).

import type { ReportCell } from '../../bindings/ReportCell';
import type { ReportDetailPart } from '../../bindings/ReportDetailPart';
import type { ReportParty } from '../../bindings/ReportParty';
import type { Language } from '../../i18n';
import { isoToDisplayDate } from '../../shared/format';
import type { ListContext } from '../../shared/grid';
import { displayName } from '../../shared/i18n';

/** A month as MM-YYYY (R28 by month). The month names of a print come with the report screens of step F3. */
export function monthText(year: number, month: number): string {
  return `${String(month).padStart(2, '0')}-${String(year)}`;
}

/** The tag follows the name after a space ("عاشق ظفر K", PRD 10.2, Code Standards F38). */
export function nameWithTag(
  names: { name_ur: string; name_en: string },
  tag: string | null,
  language: Language,
): string {
  const name = displayName(names, language);
  return tag === null || tag === '' ? name : `${name} ${tag}`;
}

/**
 * A party on a screen: the full code, then the name with its tag. The full code is used on a
 * screen because a short code alone can mean a seller or a buyer; a print uses the short code
 * where there is one (FR-19.13).
 */
export function partyText(party: ReportParty, language: Language): string {
  return `${party.code} ${nameWithTag(party, party.tag, language)}`;
}

/** The side word of a balance: بنام or جمع. A zero balance has none. */
export function sideText(cell: ReportCell, context: ListContext): string {
  if (cell.type !== 'balance' || cell.side === 'zero') return '';
  return context.text(cell.side === 'dr' ? 'report-side-dr' : 'report-side-cr');
}

function detailPartText(part: ReportDetailPart, context: ListContext): string {
  const { language, text, money } = context;
  switch (part.kind) {
    case 'sale':
      return text('report-detail-sale', {
        number: String(part.bikri_number),
        item: displayName({ name_ur: part.item.ur, name_en: part.item.en }, language),
        qty: String(part.qty),
        packing: displayName({ name_ur: part.packing.ur, name_en: part.packing.en }, language),
        rate: money(part.rate_paisa),
        value: money(part.value_paisa),
        commission: money(part.commission_paisa),
        laga: money(part.laga_paisa),
      });
    case 'lot':
      return text('report-detail-lot', {
        number: String(part.bikri_number),
        item: displayName({ name_ur: part.item.ur, name_en: part.item.en }, language),
        qty: String(part.qty),
        packing: displayName({ name_ur: part.packing.ur, name_en: part.packing.en }, language),
        gross: money(part.gross_paisa),
        deductions: money(part.deductions_paisa),
      });
    case 'cash_lent':
      return text('report-detail-cash-lent', { reason: part.reason });
    case 'text':
      return part.text;
  }
}

/** The text of one cell. A balance's side word is `sideText`, shown in its own slot. */
export function cellText(cell: ReportCell, context: ListContext): string {
  switch (cell.type) {
    case 'empty':
      return '';
    case 'text':
      return cell.text;
    case 'label':
      return context.text(cell.id);
    case 'number':
      return String(cell.value);
    case 'money':
    case 'balance':
      return context.money(cell.paisa);
    case 'date':
      return isoToDisplayDate(cell.iso);
    case 'month':
      return monthText(cell.year, cell.month);
    case 'party':
      return partyText(cell.party, context.language);
    case 'names':
      return displayName({ name_ur: cell.names.ur, name_en: cell.names.en }, context.language);
    case 'doc': {
      const kind = context.text(cell.type_id);
      return cell.number === null
        ? kind
        : context.text('report-doc-number', { kind, number: String(cell.number) });
    }
    case 'detail':
      return cell.parts.map((part) => detailPartText(part, context)).join(' · ');
  }
}
