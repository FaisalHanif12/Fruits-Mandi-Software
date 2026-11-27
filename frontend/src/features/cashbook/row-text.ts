// What a row of the روکڑ بک says (PRD FR-16.14 v1.60, plan 99): every row is named for what it is, with its
// number and the records it is about, for example "وصولی 1، اکرم", "دکان خرچہ 1، بجلی", "بکری 2، موقع پر جمع،
// شاہد" and "باغ کی قسط 1، شاہ پور باغ". The backend says what the row is (`kind`) and sends the names (`refs`);
// this only puts them into words. Nothing here works out money.

import type { Language } from '../../i18n';
import { displayName } from '../../shared/i18n';
import type { CashbookRow, CashGroup } from './types';

type Say = (id: string, vars?: Record<string, string>) => string;

/** The Fluent message ID of a stored word such as `shop_expense`: hyphens, not underscores. */
export function kindId(kind: string): string {
  return kind.replaceAll('_', '-');
}

/** The word of a type of the filter and of the summary: وصولی, خرچہ, باغ ... */
export function groupMessageId(group: CashGroup): string {
  return `cashbook-group-${kindId(group)}`;
}

// The label is put together from finished pieces, never by passing one Fluent answer into another message as a
// variable: Fluent isolates every variable, and an isolate that holds only isolates has no letter of its own, so
// it turns left to right and an Urdu row reads backwards (found on the Urdu روکڑ). Only the separators and the
// short words between the pieces come from Fluent here. Names are isolated one by one (F25).

/** First strong isolate and its end: a name keeps its own direction inside the row (UBA). */
const FSI = '\u2068';
const PDI = '\u2069';

const isolated = (name: string) => `${FSI}${name}${PDI}`;

/**
 * The names a row is about. A row about one record (an expense head, a partner, a garden or a trip) is
 * named by it. Otherwise by its parties: the account of a voucher, the buyers or sellers of a sale. The
 * backend sends at most three of them with the count of all, so the rest are told as "and 2 more".
 */
function namesOf(row: CashbookRow, text: Say, language: Language): string {
  const { refs } = row;
  const one = refs.expense_head ?? refs.partner ?? refs.garden ?? refs.trip;
  if (one !== null) return isolated(displayName(one, language));
  const names = refs.parties
    .map((party) => isolated(displayName(party, language)))
    .join(text('cashbook-row-name-gap'));
  const more = refs.party_count - refs.parties.length;
  return more > 0 ? `${names} ${text('cashbook-row-more', { count: String(more) })}` : names;
}

/** The number a row is named with, as it is written elsewhere: a purchase's is خ-1, the others are plain. */
function numberOf(row: CashbookRow, text: Say): string | null {
  if (row.number === null) return null;
  const number = String(row.number);
  // The grid isolates every number run left to right (`NumberRunsCell`), so it is left as it is.
  return row.kind === 'purchase' ? text('purchases-number', { number }) : number;
}

/**
 * The words of a row. A sale's cash is named by its number first ("Sale 2, Paid on the Spot, Shahid"), the others
 * by what they are and their number ("Receipt 1, Akram"). An old posting and its reversing entry, shown only when
 * asked for, say so first.
 */
export function cashRowText(row: CashbookRow, text: Say, language: Language): string {
  const kind = text(`cashbook-row-kind-${kindId(row.kind)}`);
  const number = numberOf(row, text);
  let head: string[];
  if (number === null) head = [kind];
  else if (row.group === 'bikri') head = [text('cashbook-row-bikri', { number }), kind];
  else head = [`${kind} ${number}`];
  const names = namesOf(row, text, language);
  const label = (names === '' ? head : [...head, names]).join(text('cashbook-row-name-gap'));
  if (row.status === 'reversal') return `${text('cashbook-row-reversal')}${label}`;
  if (row.status === 'reversed') return `${text('cashbook-row-reversed')}${label}`;
  return label;
}
