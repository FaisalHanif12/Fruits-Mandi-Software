// The preview as text for the cells (PRD 12.4, 12.5). The backend sent every number; this only
// writes each as text with formatPaisa and hands the working next to it. Nothing is added up,
// multiplied or rounded here (CLAUDE.md rule 1, Code Standards M12). A row the backend could not
// work out yet (`calc` is null) has no number: its cells stay empty, never a guess.

import type { Working } from '../../bindings/Working';
import { formatBps, formatPaisa, isoToDisplayDate, type Grouping } from '../../shared/format';
import type { Language } from '../../i18n';
import { displayName, levelMessageId } from '../../shared/i18n';
import type { PickedRecord } from '../../shared/grid';
import { LOT, SALE, type LineCalc, type LinePreview, type LotPreview } from './types';
import type { LineCharge } from '../../bindings/LineCharge';

/** The text of one Fluent message, in the app language. */
export type Say = (id: string, vars?: Record<string, string>) => string;

type Cells = Record<string, string>;
type Workings = Record<string, Working>;

export interface RowView {
  calc: Cells;
  workings: Workings;
}

const NOTHING: RowView = { calc: {}, workings: {} };

/** The message of the working of نقد when the backend filled it for a one-time seller. */
const CASH_PUKHTA = 'working-cash-pukhta';

/**
 * The calculated cells of a lot row. The charges (مزدوری, منشیانہ) show the value the rules
 * gave while nothing is typed (NFR-USE-27); the net and the balance are read-only.
 */
export function lotView(lot: LotPreview | undefined, grouping: Grouping): RowView {
  if (lot === undefined) return NOTHING;
  const money = (paisa: Parameters<typeof formatPaisa>[0]) => formatPaisa(paisa, { grouping });
  const calc: Cells = {};
  const workings: Workings = {};
  if (lot.labour !== null) calc[LOT.labour] = money(lot.labour.value_paisa);
  if (lot.munshiana !== null) calc[LOT.munshiana] = money(lot.munshiana.value_paisa);
  if (lot.totals !== null) {
    calc[LOT.net] = money(lot.totals.net);
    workings[LOT.net] = lot.totals.workings.net;
    // A one-time seller is paid his whole net on the spot while نقد is empty (PRD FR-13.10): the
    // backend says so by the message of the working. The number is shown in the نقد cell like a
    // typed one, and a value typed there is kept as typed.
    if (lot.totals.workings.cash_paid.message_id === CASH_PUKHTA) {
      calc[LOT.cash] = money(lot.totals.cash_paid);
      workings[LOT.cash] = lot.totals.workings.cash_paid;
    }
  }
  if (lot.balance_qty !== null) calc[LOT.balance] = String(lot.balance_qty);
  return { calc, workings };
}

/** The words of a rate hint: where the rate came from (PRD FR-7.53). */
function hintWords(hint: NonNullable<LinePreview['rate_hint']>, say: Say, grouping: Grouping) {
  return say(`bikri-rate-hint-${hint.source.replaceAll('_', '-')}`, {
    rate: formatPaisa(hint.rate_paisa, { grouping }),
    date: isoToDisplayDate(hint.date),
  });
}

/**
 * The cells of a sale line. The rate hint is grey text in the empty rate cell and its words are
 * the tooltip; it is never typed into the cell (PRD FR-7.53, Code Standards F43).
 */
export function lineView(line: LinePreview | undefined, grouping: Grouping, say: Say): RowView {
  if (line === undefined) return NOTHING;
  const calc: Cells = {};
  const workings: Workings = {};
  if (line.rate_hint !== null) {
    calc[SALE.rate] = formatPaisa(line.rate_hint.rate_paisa, { grouping });
    calc[`${SALE.rate}Tip`] = hintWords(line.rate_hint, say, grouping);
  }
  const done = line.calc;
  if (done !== null) {
    const money = (paisa: LineCalc['buyer_total']) => formatPaisa(paisa, { grouping });
    calc[SALE.amount] = money(done.seller_amount);
    workings[SALE.amount] = done.workings.seller_amount;
    calc[SALE.sellerComm] = money(done.seller_comm.amount);
    workings[SALE.sellerComm] = done.seller_comm.working;
    calc[SALE.buyerComm] = money(done.buyer_comm.amount);
    workings[SALE.buyerComm] = done.buyer_comm.working;
    calc[SALE.laga] = money(done.laga.amount);
    workings[SALE.laga] = done.laga.working;
    calc[SALE.buyerTotal] = money(done.buyer_total);
    workings[SALE.buyerTotal] = done.workings.buyer_total;
    calc[SALE.payment] = done.payment_type;
  }
  return { calc, workings };
}

/** The number of a line charge as a person types it: "6" for 6%, "8" for 8 for each نگ. */
export function chargeNumber(charge: LineCharge, grouping: Grouping): string {
  if (charge.value_bps !== null) return formatBps(charge.value_bps);
  if (charge.value_paisa !== null) return formatPaisa(charge.value_paisa, { grouping });
  return '';
}

/** The value of a line charge as a person writes it: "6%", "8 / نگ". */
export function chargeValueText(charge: LineCharge, grouping: Grouping, say: Say): string {
  const value = chargeNumber(charge, grouping);
  if (value === '') return '';
  // A value typed by hand keeps what was typed: a percent, or an amount for each نگ.
  return say(charge.value_bps !== null ? 'bikri-tag-percent' : 'bikri-tag-per-unit', { value });
}

/**
 * The small tag of a charge (PRD FR-11.4): which rule decided and its value, for example
 * "رحیم + کیلا 5%". A value typed on the line says so (دستی). The backend sends only the level
 * of the rule; the names are the seller or buyer and the item of the line.
 */
export function ruleTag(
  charge: LineCharge,
  names: { party: PickedRecord | undefined; item: PickedRecord | undefined },
  language: Language,
  grouping: Grouping,
  say: Say,
): string {
  const value = chargeValueText(charge, grouping, say);
  if (charge.source === 'typed' || charge.method === 'manual') {
    return [say('bikri-tag-manual'), value].filter((part) => part !== '').join(' ');
  }
  const level = charge.rule_label?.level ?? 'default';
  const party = names.party === undefined ? '' : displayName(names.party, language);
  const item = names.item === undefined ? '' : displayName(names.item, language);
  const who =
    level === 'party_item'
      ? [party, item].filter((part) => part !== '').join(' + ')
      : level === 'party'
        ? party
        : level === 'item'
          ? item
          : say(levelMessageId(level));
  return [who === '' ? say(levelMessageId(level)) : who, value]
    .filter((part) => part !== '')
    .join(' ');
}
