// The columns of the two grids (PRD 12.3, 12.4; plan 12). Enter stops of a sale line are buyer, qty,
// rate and paid on the spot: four fields and four Enters when the buyer may take credit, three when he
// must pay in full and the backend closes the paid cell (NFR-USE-6, D8 Q2, PRD FR-12.44, plan 65).
// The money paid on the spot (موقع پر جمع) is the last money cell of the line, after the buyer total.
// What is left on the buyer's account is in the details of the line under the grid.
//
// The windows are narrow: with the main menu and the side menu open, a 1024 px window leaves about
// 680 px for the screen (NFR-USE-15). So the grids hold the cells typed on most lots and lines,
// and the amounts the user checks; the rest sits in the "more" panel of the lot and in the details
// of the line (PRD FR-12.2, Code Standards F22). Wider windows add amounts: the net and the balance
// to the lots; to the lines the seller amount and the seller commission from 1366 px, and the buyer
// commission from 1920 px. The paid cell is one more column, so in every window the seller amount, the
// commissions and the لاگا that do not fit are in the details of the line instead (plan 65, plan 58
// option C). bikri-column-widths.test.ts keeps the sums inside the room of each window.

import { FIT_MIN, type EntryColumn } from '../../shared/grid';
import { isFromStep, type LayoutStep } from '../../shared/hooks';
import { isArrivalLot, isGardenLot, isOwnLot } from './draft';
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
      editable: (row) => !isOwnLot(row) && !isArrivalLot(row),
    },
    {
      field: LOT.packing,
      type: 'picker',
      picker: { kind: 'master', scope: 'packing' },
      headerId: 'glossary-packing',
      editable: (row) => !isOwnLot(row) && !isArrivalLot(row),
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
    // An own goods lot has no نقد: the firm does not give cash to itself (PRD FR-12.26). A garden's
    // lot has none either (PRD FR-12.36).
    {
      field: LOT.cash,
      type: 'money',
      headerId: 'glossary-naqd',
      showsCalc: true,
      explain: true,
      editable: (row) => !isOwnLot(row) && !isGardenLot(row),
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
      // The buyer box lists چکتا buyers and adds a new one (PRD FR-7.29, FR-13.3).
      picker: { kind: 'account', scope: 'buyer', withChukta: true },
      headerId: 'glossary-gahak',
      // A buyer's name needs room: a name cut short does not tell two buyers apart.
      minWidth: FIT_MIN.longName,
    },
    { field: SALE.qty, type: 'qty', headerId: 'bikri-col-qty' },
    {
      field: SALE.rate,
      type: 'rate',
      headerId: 'bikri-col-rate',
      // The last rate is grey, italic words ("Last 900"), laid out as text and not as a figure, so it
      // is never taken for a typed rate. It is never typed into the cell (PRD FR-7.53, D8 Bug 1).
      hintFromCalc: true,
      hintTooltipField: `${SALE.rate}Tip`,
    },
    {
      field: SALE.grade,
      type: 'select',
      headerId: 'glossary-darja',
      // Optional and never an Enter stop: a line stays buyer, qty, rate (PRD FR-12.42, NFR-USE-6). It is
      // reached with the arrow keys or a click, and Space or Alt+↓ opens its list.
      enterStop: false,
      choices: [
        { value: '', labelId: 'bikri-grade-none' },
        { value: 'first', labelId: 'glossary-darja-awwal' },
        { value: 'second', labelId: 'glossary-darja-doyam' },
        { value: 'third', labelId: 'glossary-darja-soyam' },
      ],
    },
    // The amounts the user checks, by room (plan 65): the paid cell is one more column, so the least used
    // amounts moved to the details of the line, where every amount of the line stands with its working.
    // The seller amount and the seller commission come from 1366 px, the buyer commission from 1920 px;
    // the لاگا is in the details in every window.
    ...(isFromStep(step, 'normal')
      ? ([
          { field: SALE.amount, type: 'calc', headerId: 'bikri-col-seller-amount', explain: true },
          {
            field: SALE.sellerComm,
            type: 'calc',
            headerId: 'bikri-col-seller-comm',
            explain: true,
          },
        ] satisfies readonly EntryColumn[])
      : []),
    ...(isFromStep(step, 'wide')
      ? ([
          { field: SALE.buyerComm, type: 'calc', headerId: 'bikri-col-buyer-comm', explain: true },
        ] satisfies readonly EntryColumn[])
      : []),
    { field: SALE.buyerTotal, type: 'calc', headerId: 'bikri-col-buyer-total', explain: true },
    {
      field: SALE.paid,
      type: 'money',
      headerId: 'bikri-col-paid-on-spot',
      // The backend decides the amount of a buyer who must pay in full: the cell then shows it, with
      // its working, and is closed (PRD FR-12.44). The screen closes it with `editable`.
      fixedFromCalc: SALE.paidFixed,
      explain: true,
      // The heading is two words ("موقع پر جمع"); it wraps to a second line.
      minWidth: FIT_MIN.longHeading,
    },
  ];
}

/**
 * The columns of the whole-sale view (PRD FR-12.1): the lot of each line first, then the columns
 * of the grid of one lot that fit beside it. The grid has the width of the grid of one lot, so the
 * lot column takes the room of the columns of the next step up (the commissions, the لاگا) and, in the
 * narrowest window, of the quality column: nothing
 * may scroll sideways, not even inside the grid (NFR-USE-15). The commissions and what is left on the
 * account of the line that has focus are in the line details under the grid, and every figure is in
 * the summary. A seller's name is cut in the lot column and read in full in the tooltip.
 */
export function allLinesColumns(step: LayoutStep): readonly EntryColumn[] {
  const lot: EntryColumn = {
    field: SALE.lot,
    type: 'label',
    headerId: 'bikri-col-lot',
    minWidth: isFromStep(step, 'wide') ? FIT_MIN.figure : FIT_MIN.name,
  };
  const beside = saleColumns(isFromStep(step, 'wide') ? 'normal' : 'compact');
  // In the narrowest window the quality (درجہ) is left to the grid of one lot, as the room is short;
  // the lot column and the buyer, qty and rate stops come first (plan 58, option C).
  return [
    lot,
    ...(isFromStep(step, 'normal') ? beside : beside.filter((c) => c.field !== SALE.grade)),
  ];
}
