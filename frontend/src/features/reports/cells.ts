// A cell of a report as the words and figures of the screen (PRD 19.1, FR-19.8, FR-19.13). Every
// amount, count and date is what the backend sent; this file only writes it: money through
// `formatPaisa` (the list context's `money`), names through `displayName` (Code Standards F38), and
// the words around the goods of a ledger line from Fluent. It adds up nothing (CLAUDE.md rule 1).

import type { ReportArgValue } from '../../bindings/ReportArgValue';
import type { ReportCell } from '../../bindings/ReportCell';
import type { ReportDetailPart } from '../../bindings/ReportDetailPart';
import type { ReportId } from '../../bindings/ReportId';
import type { ReportMessageArg } from '../../bindings/ReportMessageArg';
import type { ReportParty } from '../../bindings/ReportParty';
import type { Language } from '../../i18n';
import { formatBps, isoToDisplayDate } from '../../shared/format';
import type { ListContext } from '../../shared/grid';
import { displayName } from '../../shared/i18n';

/**
 * A share the backend sends in hundredths of a percent (0 to 10,000) as text with two decimals and
 * the sign: 7210 is "72.10%" (PRD 18.2: percentages to two decimals). Only the digits are put
 * together as text; nothing is worked out (CLAUDE.md rule 1).
 */
export function percentText(hundredths: number): string {
  const whole = Math.trunc(hundredths / 100);
  const part = String(hundredths % 100).padStart(2, '0');
  return `${String(whole)}.${part}%`;
}

/** The Fluent message of a month's name: `report-month-1` is January (PRD 19.2: R28 by month). */
export function monthMessageId(month: number): string {
  return `report-month-${String(month)}`;
}

/** A month as its name and the year ("October 2026", "اکتوبر 2026"), the way R28 by month shows it. */
export function monthText(text: (id: string) => string, year: number, month: number): string {
  return `${text(monthMessageId(month))} ${String(year)}`;
}

/**
 * The words for a name the backend left empty on purpose (PRD 19.2): the heading of the buyers with
 * no city (R21) or no collector (R22), and the sales with no اڈا (R8). Shown as an empty name, they
 * would look like a mistake. Null for any other empty cell.
 */
export function emptyNameMessageId(id: ReportId, rowKind: 'normal' | 'group'): string | null {
  if (id === 'R21' && rowKind === 'group') return 'report-group-no-city';
  if (id === 'R22' && rowKind === 'group') return 'report-group-no-collector';
  if (id === 'R8') return 'report-adda-none';
  return null;
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

function detailPartText(part: ReportDetailPart, context: ListContext, shortSale: boolean): string {
  const { language, text, money } = context;
  switch (part.kind) {
    case 'sale':
      // The buyers' list (R5) writes a line the way the client's own print does: "(item qty . rate)".
      if (shortSale) {
        return text('report-detail-sale-short', {
          item: displayName({ name_ur: part.item.ur, name_en: part.item.en }, language),
          qty: String(part.qty),
          rate: money(part.rate_paisa),
        });
      }
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

/**
 * One argument of a message cell as the words of the screen (plan 24, R31 and the partner lines).
 * Money is written with the app's grouping, a name in the app language, a percent as the reader
 * writes it (40, 33.33), and a message inside a message is worked out first. Nothing is added up.
 */
function argText(value: ReportArgValue, context: ListContext): string {
  switch (value.type) {
    case 'text':
      return value.text;
    case 'number':
      return String(value.value);
    case 'money':
      return context.money(value.paisa);
    case 'percent':
      return formatBps(value.hundredths);
    case 'names':
      return displayName({ name_ur: value.names.ur, name_en: value.names.en }, context.language);
    case 'message':
      return messageText(value.id, value.args, context);
  }
}

/** A Fluent message with the arguments the backend named (a `message` cell, PRD FR-17.10). */
export function messageText(
  id: string,
  args: readonly ReportMessageArg[],
  context: ListContext,
): string {
  const vars: Record<string, string> = {};
  for (const arg of args) vars[arg.name] = argText(arg.value, context);
  return context.text(id, vars);
}

/**
 * The text of one cell. A balance's side word is `sideText`, shown in its own slot. `shortSale`
 * writes each sale line of a detail cell as "(item qty . rate)" (R5).
 */
export function cellText(cell: ReportCell, context: ListContext, shortSale = false): string {
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
      return monthText(context.text, cell.year, cell.month);
    case 'percent':
      return percentText(cell.hundredths);
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
      return cell.parts.map((part) => detailPartText(part, context, shortSale)).join(' · ');
    case 'message':
      return messageText(cell.id, cell.args, context);
  }
}
