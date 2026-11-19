// What the partner statement remembers for the Owner (PRD FR-7.45): the partner and the days he last
// looked at, kept as text under the screen name `list.partner-statement`. An address with a partner in
// it (the dashboard's partner card) wins. A partner that is gone is simply not found by the screen.
// Nothing about money is kept.

import { idOrNull, isRecord, textOr } from '../../shared/screen-state';

export const STATEMENT_SCREEN = 'list.partner-statement';

export interface StatementChoices {
  partnerId: string;
  from: string;
  to: string;
}

export const NO_STATEMENT_CHOICES: StatementChoices = { partnerId: '', from: '', to: '' };

function short(value: unknown): string {
  const text = textOr(value, '');
  return text.length <= 20 ? text : '';
}

export function statementChoicesFrom(raw: unknown): StatementChoices {
  if (!isRecord(raw)) return NO_STATEMENT_CHOICES;
  const partner = idOrNull(raw['partner']);
  return {
    partnerId: partner === null ? '' : String(partner),
    from: short(raw['from']),
    to: short(raw['to']),
  };
}

export function statementChoicesText(choices: StatementChoices): Record<string, unknown> | null {
  const kept: Record<string, unknown> = {};
  if (choices.partnerId !== '' && Number.isSafeInteger(Number(choices.partnerId))) {
    kept['partner'] = Number(choices.partnerId);
  }
  if (choices.from !== '') kept['from'] = choices.from;
  if (choices.to !== '') kept['to'] = choices.to;
  return Object.keys(kept).length === 0 ? null : kept;
}
