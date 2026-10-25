// The words of a partner statement line (PRD FR-17.9, FR-17.10). The backend sends the kind of the
// line (`heading`, `detail`), the numbers it needs (`number`, `secondary_number`, `values`) and the
// record it is about (`subject`); the words come from Fluent here, so every sentence is the same
// in both languages and never built by hand in code (Code Standards F31). Money in `values` is
// formatted with the app's grouping and never worked out (CLAUDE.md rule 1).
//
// Examples (PRD FR-17.10, ending at -296 in D4-A33):
//   "10-10-26 سفر-1: خریداری خ-1، دکان سے" -4,160
//   "10-10-26 سفر-1، بکری 1: مالٹا 15 پیٹی، فروخت 8,460 ... آپ کا 40%" +3,384

import { messageVars } from '../../shared/api';
import type { Grouping } from '../../shared/format';
import { displayName } from '../../shared/i18n';
import type { Language } from '../../i18n';
import type { StatementLine } from './types';

export interface StatementSay {
  language: Language;
  grouping: Grouping;
  /** A Fluent message as words, with its variables. */
  text: (id: string, vars?: Record<string, string>) => string;
  /** True when this build has words for the message. */
  exists: (id: string) => boolean;
}

/** The Fluent message ID part of a stored kind such as `income_share`: hyphens, not underscores. */
function idPart(kind: string): string {
  return kind.replaceAll('_', '-');
}

/**
 * The number series of the voucher a line came from, by its detail. The backend names the exact
 * series in `values.kind` when the line did not come from the usual one: a salary deduction is an
 * LR- voucher and a جنرل is a G- voucher (statement.rs, plan 17 T2b); see `seriesOf`.
 */
const VOUCHER_SERIES: Readonly<Partial<Record<StatementLine['detail'], string>>> = {
  expense_share: 'shop-expense',
  capital_in: 'capital-in',
  capital_out: 'capital-out',
  taken: 'personal-expense',
  returned: 'personal-return',
};

/** The `values.kind` words that name a series of their own, whatever the detail is. */
const KIND_SERIES: Readonly<Record<string, string>> = {
  salary: 'money-lent-return',
  general: 'general',
  shop_expense: 'shop-expense',
  capital_in: 'capital-in',
  capital_out: 'capital-out',
  personal_expense: 'personal-expense',
  personal_return: 'personal-return',
};

/** The series of a line: its kind first (PRD FR-17.10), else the usual one for its detail. */
function seriesOf(line: StatementLine): string | undefined {
  const kind = line.values['kind'];
  if (line.heading !== 'trip' && typeof kind === 'string') {
    const named = KIND_SERIES[kind];
    if (named !== undefined) return named;
  }
  return VOUCHER_SERIES[line.detail];
}

/** The word of the group a line belongs to (the heading). */
export function headingText(line: StatementLine, say: StatementSay): string {
  const id = `partners-statement-heading-${idPart(line.heading)}`;
  return say.exists(id) ? say.text(id) : say.text('partners-statement-heading-other');
}

/** What the secondary number of a trip line is: the backend names it in `values.kind`. */
function secondaryText(line: StatementLine, say: StatementSay): string {
  if (line.secondary_number === null) return '';
  const number = String(line.secondary_number);
  switch (line.values['kind']) {
    case 'purchase':
      return say.text('purchases-number', { number });
    case 'cost':
      return say.text('trips-cost-number', { number });
    default:
      return number;
  }
}

/**
 * A fruit line of an entry that was changed since: the backend gives the details (item, packing,
 * values) only for the posting in force, so an older posting and its reversal have the number only
 * (PRD FR-17.10). A cancelled entry keeps its details. They read as that, never as a sentence
 * with holes.
 */
function isOldFruitLine(line: StatementLine): boolean {
  return line.detail === 'fruit_taken' && line.values['qty'] === undefined;
}

/** The sentence of a line, from its detail and the numbers the backend sent. */
export function detailText(line: StatementLine, say: StatementSay): string {
  if (isOldFruitLine(line)) {
    const number = line.number === null ? '' : String(line.number);
    return say.text('partners-statement-detail-fruit-earlier', {
      fruit: number === '' ? '' : say.text('fruit-number', { number }),
    });
  }
  const id = `partners-statement-detail-${idPart(line.detail)}`;
  if (!say.exists(id)) {
    return say.text('partners-statement-detail-other', { reason: line.reason });
  }
  const vars: Record<string, string> = messageVars(line.values, say.grouping);
  const number = line.number === null ? '' : String(line.number);
  vars['number'] = number;
  vars['reason'] = line.reason;
  vars['subject'] = line.subject === null ? '' : displayName(line.subject, say.language);
  // The packing of a fruit line (PRD FR-17.10): "2 پیٹی مالٹا @ 1,400". Null on every other line.
  vars['packing'] = line.packing === null ? '' : displayName(line.packing, say.language);
  const series = seriesOf(line);
  vars['voucher'] =
    series === undefined || line.number === null
      ? number
      : say.text(`vouchers-number-${series}`, { number });
  if (line.heading === 'garden') {
    // `number` is the garden's number in its own series (PRD FR-12.11).
    vars['garden'] = line.number === null ? '' : say.text('gardens-number', { number });
  }
  if (line.heading === 'trip') {
    vars['trip'] = line.number === null ? '' : say.text('trips-number', { number });
    vars['secondary'] = secondaryText(line, say);
    vars['kind'] = typeof line.values['kind'] === 'string' ? line.values['kind'] : 'other';
    // A sale in another shop has no بکری number: the line says so (PRD FR-12.32).
    vars['source'] =
      line.secondary_number !== null && line.secondary_number > 0 ? 'bikri' : 'other';
  }
  return say.text(id, vars);
}
