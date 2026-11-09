// Umar's case of the demo of 08-11-2026 (D8, Bug 1, plan 61): a lot of 200 آم from احسان (freight
// 10,000, labour 1,000) sold in three lines, مانی 20 at 1,000, انس 50 at 900 and ہارون 130 at 900. The
// third line brings the lot to exactly 200 of 200, and its rate equals the grey hint the backend
// gives under the empty rate cell (the rate of the line before it).
//
// This file is a test double of `bikri_preview` for this one draft shape. It answers the way the
// backend does for blank rows, incomplete lines and complete lines (domain/src/bikri/process.rs:
// a row with no buyer and no rate is blank; a row with something missing has no `calc`; the hint is
// the rate of the nearest earlier line). The money in it is the PRD figures of D8 Section 3 written
// in paisa, copied, never worked out here (Code Standards M12). When the backend's own file for
// E2E-64 exists, `umarAnswer` is replaced by it. No test library is imported here, so the
// development page (`#/dev/bikri?offer&umar`) can use it too.

import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { LinePreview } from '../../src/bindings/LinePreview';
import type { LotPreview } from '../../src/bindings/LotPreview';
import type { Working } from '../../src/bindings/Working';
import type { BikriDraftStores } from '../../src/features/bikri/draft-store';
import { LOT, ON } from '../../src/features/bikri/types';
import { paisaForTest } from './paisa';
import { account } from './accounts';
import { EVENT1, IDS } from './bikri-data';

export const MANI = account(41, 'buyer', '0020', 'مانی', 'Mani');
export const ANAS = account(42, 'buyer', '0021', 'انس', 'Anas');
export const HAROON = account(43, 'buyer', '0022', 'ہارون', 'Haroon');
export const EHSAN = account(IDS.rahim, 'seller', '0010', 'احسان', 'Ehsan');
/** The seller as the draft holds a chosen record: the id, the code and both names. */
const EHSAN_PICKED = { id: EHSAN.id, code: EHSAN.code, name_ur: 'احسان', name_en: 'Ehsan' };
const MANGO = { id: 7, code: '0007', name_ur: 'آم', name_en: 'Mango' };
const BOX = { id: 1, code: '0001', name_ur: 'پیٹی', name_en: 'Box' };

export const UMAR_BUYERS = [MANI, ANAS, HAROON] as const;
export const LOT_QTY = 200;

/** Money as it arrives from the backend: whole rupees, as paisa. */
const rupees = (amount: number) => paisaForTest(amount * 100);

interface Figures {
  amount: number;
  sellerComm: number;
  buyerComm: number;
  laga: number;
  total: number;
}

/** The figures of D8 Section 3 for each line, by what was typed ("qty@rate"). */
const LINE_FIGURES: Readonly<Record<string, Figures>> = {
  '20@1000': { amount: 20_000, sellerComm: 1_200, buyerComm: 1_200, laga: 160, total: 21_360 },
  '50@900': { amount: 45_000, sellerComm: 2_700, buyerComm: 2_700, laga: 400, total: 48_100 },
  '130@900': { amount: 117_000, sellerComm: 7_020, buyerComm: 7_020, laga: 1_040, total: 125_060 },
};

interface Totals {
  gross: number;
  sellerComm: number;
  buyerComm: number;
  laga: number;
  deductions: number;
  net: number;
  buyerTotal: number;
  shared: number;
  split: readonly [number, number, number];
}

/** The lot's totals by how many lines stand (1, 2 or 3): D8 Section 3 and the image of the demo. */
const TOTALS: Readonly<Record<number, Totals>> = {
  1: {
    gross: 20_000,
    sellerComm: 1_200,
    buyerComm: 1_200,
    laga: 160,
    deductions: 12_200,
    net: 7_800,
    buyerTotal: 21_360,
    shared: 2_560,
    split: [1_024, 768, 768],
  },
  2: {
    gross: 65_000,
    sellerComm: 3_900,
    buyerComm: 3_900,
    laga: 560,
    deductions: 14_900,
    net: 50_100,
    buyerTotal: 69_460,
    shared: 8_360,
    split: [3_344, 2_508, 2_508],
  },
  3: {
    gross: 182_000,
    sellerComm: 10_920,
    buyerComm: 10_920,
    laga: 1_600,
    deductions: 21_920,
    net: 160_080,
    buyerTotal: 194_520,
    shared: 23_440,
    split: [9_376, 7_032, 7_032],
  },
};

type Sale = BikriDraft['lots'][number]['sales'][number];

/** The money of a line paid on the spot, in rupees (PRD FR-12.44): what the backend decides for it. */
interface PaidFigures {
  paid: number;
  left: number;
  /** The buyer must pay in full: the backend sets the amount and the cell is closed. */
  fixed: boolean;
  /** What was typed is more than the buyer total: counted as 0 and refused. */
  over: boolean;
}

/**
 * What the backend says about the paid amount of a line (plan 65): a چکتا buyer pays in full and the
 * cell is closed; any other buyer pays what was typed, empty is 0, and more than the buyer total is
 * counted as 0 and refused. The numbers are the PRD's D8 figures, copied (Code Standards M12).
 */
function paidFigures(sale: Sale, total: number): PaidFigures {
  if (sale.chukta !== null) return { paid: total, left: 0, fixed: true, over: false };
  const typed = Number((sale.paid ?? '').replaceAll(',', '').trim() || '0');
  if (typed > total) return { paid: 0, left: total, fixed: false, over: true };
  return { paid: typed, left: total - typed, fixed: false, over: false };
}

/** A working with money arguments, as the backend sends it (`working-paid-*`, `working-left`). */
function workingOf(messageId: string, args: Record<string, number>): Working {
  return {
    message_id: messageId,
    args: Object.entries(args).map(([name, amount]) => ({
      name,
      value: { kind: 'money', paisa: rupees(amount) },
    })),
    rule_id: null,
  };
}

/** Event 1's finished line, with the figures of this case put in. Only the numbers are changed. */
function doneLine(
  sale: Sale,
  index: number,
  remaining: number,
  figures: Figures,
): { line: LinePreview; paid: PaidFigures } {
  const template = EVENT1.lots[0]?.lines[0];
  if (template?.calc == null) throw new Error('Event 1 has no finished line');
  const line = structuredClone(template);
  const calc = line.calc;
  if (calc === null) throw new Error('unreachable');
  line.key = sale.key;
  line.line_no = index + 1;
  line.remaining_qty = remaining;
  calc.qty = Number(sale.qty);
  calc.seller_amount = rupees(figures.amount);
  calc.buyer_value = rupees(figures.amount);
  calc.seller_comm.amount = rupees(figures.sellerComm);
  calc.buyer_comm.amount = rupees(figures.buyerComm);
  calc.laga.amount = rupees(figures.laga);
  calc.buyer_total = rupees(figures.total);
  const paid = paidFigures(sale, figures.total);
  calc.paid_paisa = rupees(paid.paid);
  calc.left_paisa = rupees(paid.left);
  calc.paid_fixed = paid.fixed;
  calc.workings.paid = paid.over
    ? workingOf('working-paid-over', {
        total: figures.total,
        typed: Number((sale.paid ?? '').replaceAll(',', '')),
        result: paid.paid,
      })
    : paid.fixed
      ? workingOf('working-paid-full', { total: figures.total, result: paid.paid })
      : workingOf('working-paid-typed', { result: paid.paid });
  calc.workings.left = workingOf('working-left', {
    total: figures.total,
    paid: paid.paid,
    result: paid.left,
  });
  return { line, paid };
}

/**
 * What the backend says for this draft. A row with no buyer and no rate is blank (it only gets the
 * quantity left and the hint); a row with something typed but not all of it has no figures and says
 * what is missing; a complete row has its figures. The hint is the rate of the nearest earlier
 * line that has one (PRD FR-7.53).
 */
export function umarAnswer(draft: BikriDraft): BikriPreview {
  const lot = draft.lots[0];
  const template = EVENT1.lots[0];
  if (lot === undefined || template === undefined) throw new Error('no lot');
  const errors: BikriPreview['errors'] = [];
  const lines: LinePreview[] = [];
  let sold = 0;
  let stand = 0;
  // What the buyers paid on the spot and what stays on their accounts (PRD FR-12.44).
  let paidSum = 0;
  let leftSum = 0;
  let lastRate: { rate: number; date: string } | null = null;
  lot.sales.forEach((sale, index) => {
    const remaining = Math.max(LOT_QTY - sold, 0);
    const hint =
      lastRate === null
        ? null
        : {
            rate_paisa: rupees(lastRate.rate),
            source: 'this_bikri' as const,
            date: draft.header.entry_date,
          };
    const noBuyer = sale.buyer_account_id === null;
    if (noBuyer && sale.seller_rate === '') {
      lines.push({
        key: sale.key,
        line_no: index + 1,
        is_blank: true,
        remaining_qty: remaining,
        rate_hint: hint,
        calc: null,
        warnings: [],
      });
      return;
    }
    const path = (sub: string) => `lots[0].sales[${String(index)}].${sub}`;
    const qty = sale.qty.trim() === '' ? null : Number(sale.qty);
    const standing = (calc: LinePreview['calc'] = null): LinePreview => ({
      key: sale.key,
      line_no: index + 1,
      is_blank: false,
      remaining_qty: remaining,
      rate_hint: hint,
      calc,
      warnings: [],
    });
    // A line that takes the lot over what it has left is refused: its error, no figures, and it
    // takes no quantity (PRD FR-12.6; domain/src/bikri/process.rs).
    if (qty !== null && qty > remaining) {
      errors.push({
        code: 'ERR_QTY_EXCEEDS_LOT',
        field: path('qty'),
        values: { available: remaining },
      });
      lines.push(standing());
      return;
    }
    // Any other row that is not blank takes its quantity, even when it is not complete yet: the
    // backend counts it as sold, so the lot's balance goes down (the demo screenshot shows باقی 0).
    if (qty !== null) sold += qty;
    if (noBuyer)
      errors.push({ code: 'ERR_BUYER_REQUIRED', field: path('buyer_account_id'), values: {} });
    if (qty === null) errors.push({ code: 'ERR_REQUIRED', field: path('qty'), values: {} });
    if (sale.seller_rate === '')
      errors.push({ code: 'ERR_REQUIRED', field: path('seller_rate'), values: {} });
    if (noBuyer || qty === null || sale.seller_rate === '') {
      lines.push(standing());
      return;
    }
    const figures = LINE_FIGURES[`${sale.qty}@${sale.seller_rate}`];
    if (figures === undefined) throw new Error(`no figures for ${sale.qty}@${sale.seller_rate}`);
    const done = doneLine(sale, index, remaining, figures);
    lines.push({ ...done.line, rate_hint: hint });
    // More than the buyer total is refused on the paid cell, with the most allowed (PRD FR-12.44).
    if (done.paid.over) {
      errors.push({
        code: 'ERR_PAID_EXCEEDS_TOTAL',
        field: path('paid'),
        values: { max: figures.total },
      });
    }
    paidSum += done.paid.paid;
    leftSum += done.paid.left;
    stand += 1;
    lastRate = { rate: Number(sale.seller_rate), date: draft.header.entry_date };
  });
  const lotPreview: LotPreview = structuredClone(template);
  lotPreview.key = lot.key;
  lotPreview.lot_no = 1;
  lotPreview.lines = lines;
  lotPreview.sold_qty = sold;
  lotPreview.balance_qty = LOT_QTY - sold;
  const totals = TOTALS[stand];
  if (totals === undefined || lotPreview.totals === null) {
    lotPreview.totals = null;
    return { ...structuredClone(EVENT1), lots: [lotPreview], errors, warnings: [] };
  }
  const summary = structuredClone(EVENT1.summary);
  for (const target of [lotPreview.totals, summary]) {
    target.gross = rupees(totals.gross);
    target.seller_comm = rupees(totals.sellerComm);
    target.buyer_comm = rupees(totals.buyerComm);
    target.laga = rupees(totals.laga);
    target.deductions = rupees(totals.deductions);
    target.net = rupees(totals.net);
    target.buyer_total = rupees(totals.buyerTotal);
    target.buyer_credit = rupees(leftSum);
    target.buyer_cash = rupees(paidSum);
    target.shared_income = rupees(totals.shared);
  }
  const split = structuredClone(EVENT1.split);
  split.forEach((part, at) => {
    part.income_paisa = rupees(totals.split[at] ?? 0);
  });
  return {
    ...structuredClone(EVENT1),
    lots: [lotPreview],
    summary,
    split,
    errors,
    warnings: [],
  };
}

/**
 * Puts Umar's lot into a draft, with no sale line typed yet: احسان's آم in پیٹی, 200 received,
 * freight 10,000, labour 1,000 typed as an amount.
 */
export function fillUmarsLot(stores: BikriDraftStores): void {
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  stores.header.getState().patch({ truckNo: 'LES-1234', zamindar: EHSAN_PICKED });
  stores.lots.getState().patchRow(lot.rowId, {
    typed: {
      [LOT.qty]: String(LOT_QTY),
      [LOT.freight]: '10000',
      [LOT.labour]: '1000',
      [LOT.sellerFilled]: ON,
      [LOT.memoryFilledFor]: String(EHSAN.id),
    },
    picked: { [LOT.seller]: EHSAN_PICKED, [LOT.item]: MANGO, [LOT.packing]: BOX },
  });
}
