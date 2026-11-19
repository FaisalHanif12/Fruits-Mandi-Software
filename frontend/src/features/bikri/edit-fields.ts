// The words for the cells of an edit (plan 16): which Fluent message names each cell that the user
// can change, so the list "What you changed" and the tooltip of a changed cell speak the same
// words as the screen's own labels. Pure; the messages are the book screen's.

import { LOT, SALE } from './types';

/** The label of a header field. */
export const HEADER_FIELD_LABEL: Readonly<Record<string, string>> = {
  entryDate: 'bikri-header-date',
};

/** The label of a cell of a lot row. */
export const LOT_FIELD_LABEL: Readonly<Record<string, string>> = {
  [LOT.seller]: 'glossary-beopari',
  [LOT.item]: 'glossary-qism-maal',
  [LOT.packing]: 'glossary-packing',
  [LOT.qty]: 'bikri-col-qty-received',
  [LOT.freight]: 'glossary-kiraya',
  [LOT.labour]: 'glossary-mazdoori',
  [LOT.munshiana]: 'glossary-munshiana',
  [LOT.cash]: 'glossary-naqd',
  [LOT.marka]: 'glossary-marka',
  [LOT.store]: 'bikri-header-store',
  [LOT.weight]: 'bikri-more-weight',
  [LOT.phone]: 'glossary-phone',
  [LOT.storeRent]: 'glossary-kiraya-store',
  [LOT.freightPaidBy]: 'bikri-more-freight-paid-by',
  [LOT.adjustAdvance]: 'bikri-more-adjust-advance',
  [LOT.labourMethod]: 'bikri-more-charge-method',
  [LOT.munshianaMethod]: 'bikri-more-charge-method',
  [LOT.phoneMethod]: 'bikri-more-charge-method',
};

/** The label of a cell of a sale line. */
export const SALE_FIELD_LABEL: Readonly<Record<string, string>> = {
  [SALE.buyer]: 'glossary-gahak',
  [SALE.qty]: 'bikri-col-qty',
  [SALE.rate]: 'bikri-col-rate',
  [SALE.grade]: 'glossary-darja',
  [SALE.paid]: 'bikri-col-paid-on-spot',
  [SALE.weight]: 'bikri-line-weight',
  [SALE.differs]: 'bikri-line-differs',
  [SALE.buyerRate]: 'bikri-line-buyer-rate',
  [SALE.sellerCommValue]: 'bikri-col-seller-comm',
  [SALE.sellerCommMethod]: 'bikri-more-charge-method',
  [SALE.buyerCommValue]: 'bikri-col-buyer-comm',
  [SALE.buyerCommMethod]: 'bikri-more-charge-method',
  [SALE.lagaValue]: 'glossary-laga',
  [SALE.lagaMethod]: 'bikri-more-charge-method',
  [SALE.note]: 'bikri-line-note',
};

/** The word of a value that is one of a few fixed words, or null when it is shown as typed. */
export function fixedWord(scope: 'lot' | 'line' | 'header', field: string, value: string) {
  if (scope === 'line' && field === SALE.grade) {
    if (value === 'first') return 'glossary-darja-awwal';
    if (value === 'second') return 'glossary-darja-doyam';
    if (value === 'third') return 'glossary-darja-soyam';
  }
  if (scope === 'line' && field === SALE.differs) return value === '1' ? 'common-yes' : 'common-no';
  if (scope === 'lot' && field === LOT.adjustAdvance)
    return value === '0' ? 'common-no' : 'common-yes';
  return null;
}
