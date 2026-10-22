// The columns of the two grids (PRD 12.3, 12.4; plan 12). Enter stops are the cells of the PRD 7.2
// example: a sale line takes buyer, qty and rate, three fields and three Enters (NFR-USE-6, D3-Q1).
// The payment type is reached with the arrow keys, Space or a click.
//
// The windows are narrow: with the main menu and the side menu open, a 1024 px window leaves about
// 680 px for the screen (NFR-USE-15). So the grids hold the cells typed on most lots and lines,
// and the amounts the user checks; the rest sits in the "more" panel of the lot and in the details
// of the line (PRD FR-12.2, Code Standards F22). Wider windows add amounts: the net and the balance
// to the lots, the two commissions to the lines from 1366 px, and the لاگا from 1920 px.

import { FIT_MIN, type EntryColumn } from '../../shared/grid';
import { isFromStep, type LayoutStep } from '../../shared/hooks';
import { isOwnLot } from './draft';
import { LOT, SALE } from './types';

export function lotColumns(step: LayoutStep): readonly EntryColumn[] {
  return [
    {
      field: LOT.seller,
      type: 'picker',
      // The only account field that offers اپنا مال, the trips' goods (PRD FR-12.26): scope
      // `lot_seller` is `seller` plus that account, and the backend decides what is offered.
      picker: { kind: 'account', scope: 'lot_seller' },
      headerId: 'bikri-col-seller',
      minWidth: FIT_MIN.name,
    },
    // The item and packing of an own goods lot come from its purchase and follow it (PRD FR-12.26),
    // so on that lot they are shown and not typed.
    {
      field: LOT.item,
      type: 'picker',
      picker: { kind: 'master', scope: 'item' },
      headerId: 'glossary-qism-maal',
      editable: (row) => !isOwnLot(row),
    },
    {
      field: LOT.packing,
      type: 'picker',
      picker: { kind: 'master', scope: 'packing' },
      headerId: 'glossary-packing',
      editable: (row) => !isOwnLot(row),
    },
    { field: LOT.qty, type: 'qty', headerId: 'bikri-col-qty-received', minWidth: FIT_MIN.heading },
    { field: LOT.freight, type: 'money', headerId: 'glossary-kiraya' },
    { field: LOT.labour, type: 'money', headerId: 'glossary-mazdoori', showsCalc: true },
    {
      field: LOT.munshiana,
      type: 'money',
      headerId: 'glossary-munshiana',
      showsCalc: true,
      minWidth: FIT_MIN.longHeading,
    },
    // A one-time seller's نقد is filled by the backend while empty (FR-13.10), with its working.
    // An own goods lot has no نقد: the firm does not give cash to itself (PRD FR-12.26).
    {
      field: LOT.cash,
      type: 'money',
      headerId: 'glossary-naqd',
      showsCalc: true,
      explain: true,
      editable: (row) => !isOwnLot(row),
    },
    ...(isFromStep(step, 'normal')
      ? ([
          { field: LOT.net, type: 'calc', headerId: 'glossary-safi-bikri', explain: true },
          {
            field: LOT.balance,
            type: 'calc',
            headerId: 'bikri-col-balance',
            minWidth: FIT_MIN.figure,
          },
        ] satisfies readonly EntryColumn[])
      : []),
  ];
}

export function saleColumns(step: LayoutStep): readonly EntryColumn[] {
  return [
    {
      field: SALE.buyer,
      type: 'picker',
      picker: { kind: 'account', scope: 'buyer' },
      headerId: 'glossary-gahak',
      // A buyer's name needs room: a name cut short does not tell two buyers apart.
      minWidth: FIT_MIN.longName,
    },
    { field: SALE.qty, type: 'qty', headerId: 'bikri-col-qty' },
    {
      field: SALE.rate,
      type: 'rate',
      headerId: 'bikri-col-rate',
      // The last rate is grey text only; it is never typed into the cell (PRD FR-7.53).
      hintFromCalc: true,
      hintIsNumber: true,
      hintTooltipField: `${SALE.rate}Tip`,
    },
    { field: SALE.amount, type: 'calc', headerId: 'bikri-col-seller-amount', explain: true },
    ...(isFromStep(step, 'normal')
      ? ([
          {
            field: SALE.sellerComm,
            type: 'calc',
            headerId: 'bikri-col-seller-comm',
            explain: true,
          },
          { field: SALE.buyerComm, type: 'calc', headerId: 'bikri-col-buyer-comm', explain: true },
        ] satisfies readonly EntryColumn[])
      : []),
    ...(isFromStep(step, 'wide')
      ? ([
          { field: SALE.laga, type: 'calc', headerId: 'glossary-laga', explain: true },
        ] satisfies readonly EntryColumn[])
      : []),
    { field: SALE.buyerTotal, type: 'calc', headerId: 'bikri-col-buyer-total', explain: true },
    {
      field: SALE.payment,
      type: 'choice',
      headerId: 'bikri-col-payment',
      enterStop: false,
      choices: [
        { value: 'credit', labelId: 'glossary-udhari' },
        { value: 'cash', labelId: 'glossary-naqdi' },
      ],
    },
  ];
}

/**
 * The columns of the whole-sale view (PRD FR-12.1): the lot of each line first, then the columns
 * of the grid of one lot that fit beside it. The grid has the width of the grid of one lot, so the
 * lot column takes the room of the columns of the next step up (the commissions, the لاگا) and,
 * in the narrowest window, of the payment column: nothing may scroll sideways, not even inside the
 * grid (NFR-USE-15). The commissions and the payment type of the line that has focus are in the
 * line details under the grid, and every figure is in the summary. A seller's name is cut in the
 * lot column and read in full in the tooltip.
 */
export function allLinesColumns(step: LayoutStep): readonly EntryColumn[] {
  const lot: EntryColumn = {
    field: SALE.lot,
    type: 'label',
    headerId: 'bikri-col-lot',
    minWidth: isFromStep(step, 'wide') ? FIT_MIN.figure : FIT_MIN.name,
  };
  const beside = saleColumns(isFromStep(step, 'wide') ? 'normal' : 'compact');
  return [
    lot,
    ...(isFromStep(step, 'normal') ? beside : beside.filter((c) => c.field !== SALE.payment)),
  ];
}
