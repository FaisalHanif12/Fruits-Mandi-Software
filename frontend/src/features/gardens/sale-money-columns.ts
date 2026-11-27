// The columns of "where the sale money is" (PRD FR-12.53): the day, the بکری and its lot; the buyer
// (or the other shop) with the quantity at its rate as R5 writes it, "(20 . 600)"; the value, the
// buyer's bill, what he paid on the spot (into روکڑ), what came in since by his receipts, what is
// still on his account; and what the garden was credited. A lot's charges have a row of their own, taken off the garden's credit. Every
// amount is the backend's and only shown (CLAUDE.md rule 1); a field a row does not have is empty.

import type { SaleMoneyRow } from '../../bindings/SaleMoneyRow';
import { formatGrams, isZeroPaisa, isoToDisplayDate, type Paisa } from '../../shared/format';
import type { ListColumn, ListContext } from '../../shared/grid';
import { displayName, lotItemWith } from '../../shared/i18n';

/** A row of the list: the backend's row with a key of its place, which is its identity in the page. */
export type SaleMoneyListRow = SaleMoneyRow & { key: string };

/** The day, the بکری and the lot, or the sale in another shop. */
export function saleText(row: SaleMoneyRow, { language, text }: ListContext): string {
  const date = isoToDisplayDate(row.date);
  const lot =
    row.lot === null ? null : lotItemWith(text, language, row.lot.item, row.lot.lot_counter);
  const parts: string[] = [date];
  if (row.kind === 'other_shop') {
    parts.push(text('gardens-sales-other-shop'));
    if (row.item !== null) parts.push(displayName(row.item, language));
  } else {
    if (row.bikri_number !== null) {
      parts.push(text('gardens-sales-bikri', { number: String(row.bikri_number) }));
    }
    if (lot !== null) parts.push(lot);
    if (row.kind === 'lot_charges') parts.push(text('gardens-sales-lot-charges'));
  }
  return parts.join(' · ');
}

/** The charges of a lot that are not 0, by name: "Freight 3,000 · Labour 250". */
function chargesText(row: SaleMoneyRow, { text, money }: ListContext): string {
  const charges = row.charges;
  if (charges === null) return '';
  const named: readonly [string, Paisa][] = [
    ['glossary-kiraya', charges.freight_paisa],
    ['glossary-mazdoori', charges.labour_paisa],
    ['glossary-munshiana', charges.munshiana_paisa],
    ['glossary-phone', charges.phone_paisa],
    ['glossary-kiraya-store', charges.store_rent_paisa],
  ];
  return named
    .filter(([, paisa]) => !isZeroPaisa(paisa))
    .map(([id, paisa]) => `${text(id)} ${money(paisa)}`)
    .join(' · ');
}

/** The buyer (or the other shop) and the goods, as R5 writes them: "Akram (20 . 600)". */
export function buyerText(row: SaleMoneyRow, context: ListContext): string {
  const { language, text, money } = context;
  if (row.kind === 'lot_charges') return chargesText(row, context);
  const who =
    row.kind === 'other_shop'
      ? (row.shop_name ?? '') === ''
        ? text('gardens-sales-shop-unnamed')
        : (row.shop_name ?? '')
      : row.buyer === null
        ? ''
        : displayName(row.buyer, language);
  const qty = String(row.qty ?? 0);
  if (row.rate_paisa === null) return `${who} ${text('gardens-sales-goods-count', { qty })}`;
  const rate = money(row.rate_paisa);
  const goods =
    row.weight_grams === null
      ? text('gardens-sales-goods', { qty, rate })
      : text('gardens-sales-goods-weight', { kg: formatGrams(row.weight_grams), rate });
  return `${who} ${goods}`;
}

const amount = (
  field: string,
  headerId: string,
  value: (row: SaleMoneyListRow) => Paisa | null,
  from?: ListColumn<SaleMoneyListRow>['from'],
): ListColumn<SaleMoneyListRow> => ({
  field,
  type: 'figure',
  headerId,
  ...(from === undefined ? {} : { from }),
  value: (row, { money }) => {
    const paisa = value(row);
    return paisa === null ? '' : money(paisa);
  },
});

export const SALE_MONEY_COLUMNS: readonly ListColumn<SaleMoneyListRow>[] = [
  {
    field: 'sale',
    type: 'text',
    headerId: 'gardens-sales-col-sale',
    grow: true,
    wrap: true,
    value: saleText,
  },
  {
    field: 'buyer',
    type: 'text',
    headerId: 'gardens-sales-col-buyer',
    grow: true,
    wrap: true,
    value: buyerText,
  },
  // At the narrowest step the value and the bill give way, so the list never scrolls sideways; the
  // line under the list shows both for the row that has focus.
  amount('value', 'gardens-sales-col-value', (row) => row.value_paisa, 'normal'),
  amount('bill', 'gardens-sales-col-bill', (row) => row.buyer_total_paisa, 'normal'),
  amount('paid', 'gardens-sales-col-paid', (row) => row.paid_paisa),
  amount('collected', 'gardens-sales-col-collected', (row) => row.collected_paisa),
  amount('still_due', 'gardens-sales-col-still-due', (row) => row.still_due_paisa),
  amount('credit', 'gardens-sales-col-credit', (row) => row.garden_credit_paisa),
];
