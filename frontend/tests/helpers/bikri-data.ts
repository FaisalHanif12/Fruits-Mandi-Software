// Test data for the بکری screen (plan 12): the reply of `bikri_preview` for Event 1 of PRD 4.4,
// written by the backend's own test (tests/fixtures/bikri_preview_event1.json), and small
// functions that give that reply for a draft the screen sent. Nothing here works out money: the
// numbers are the backend's, read from the file. Used by the component tests and by the
// development page (/dev/bikri), so it imports nothing from Vitest.

import e2e26 from '../fixtures/bikri_preview_e2e26.json';
import e2e26Request from '../fixtures/bikri_preview_e2e26_request.json';
import event1 from '../fixtures/bikri_preview_event1.json';
import event1Request from '../fixtures/bikri_preview_event1_request.json';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { BikriPreviewInput } from '../../src/bindings/BikriPreviewInput';
import type { LinePreview } from '../../src/bindings/LinePreview';
import type { LotPreview } from '../../src/bindings/LotPreview';
import type { SaleInput } from '../../src/bindings/SaleInput';
import type { SellerMemory } from '../../src/bindings/SellerMemory';
import type { TruckMemory } from '../../src/bindings/TruckMemory';

/** The fixture files are the backend's own JSON; their text is read as the generated types. */
function previewOf(value: unknown): BikriPreview {
  return value as BikriPreview;
}
function requestOf(value: unknown): BikriPreviewInput {
  return value as BikriPreviewInput;
}

/** The reply for Event 1 with both lines and a blank third row, as the backend wrote it. */
export const EVENT1: BikriPreview = previewOf(event1);
/** What the screen sends for Event 1, as the backend wrote it (keys lot-1, line-1 to line-3). */
export const EVENT1_REQUEST: BikriPreviewInput = requestOf(event1Request);
/** E2E-26: line 1 sold (20 at 600) and line 2 a blank row that starts at the 30 left. */
export const E2E26: BikriPreview = previewOf(e2e26);
export const E2E26_REQUEST: BikriPreviewInput = requestOf(e2e26Request);

/** The ids of Event 1 in the backend's test (the request files above). */
export const IDS = {
  rahim: 17,
  karim: 18,
  akram: 19,
  arshad: 20,
  pukhtaBuyer: 21,
  pukhtaSeller: 22,
  banana: 1,
  apple: 2,
  box: 1,
  swat: 1,
} as const;

function clone<T>(value: T): T {
  return structuredClone(value);
}

/** A row the backend ignores: no buyer and no rate. It still gets the quantity left and the hint. */
export function blankLine(key: string, lineNo: number, remaining: number): LinePreview {
  return {
    key,
    line_no: lineNo,
    is_blank: true,
    remaining_qty: remaining,
    rate_hint: null,
    calc: null,
    warnings: [],
  };
}

/** The line a row gets: a finished line of the file if it has a buyer, else what a blank row gets. */
function lineFor(base: LotPreview, index: number, sale: SaleInput): LinePreview {
  const at = base.lines[index];
  if (sale.buyer_account_id !== null) {
    const done = at !== undefined && !at.is_blank ? at : base.lines.find((line) => !line.is_blank);
    return done === undefined
      ? blankLine(sale.key, index + 1, 0)
      : { ...clone(done), key: sale.key, line_no: index + 1 };
  }
  if (at?.is_blank === true) return { ...clone(at), key: sale.key, line_no: index + 1 };
  // A row that is finished in the file but empty here: what the backend says for a blank row.
  const source = at ?? base.lines.at(-1);
  return {
    ...blankLine(sale.key, index + 1, source?.remaining_qty ?? 0),
    rate_hint: source?.rate_hint ?? null,
  };
}

/**
 * The reply for a draft, built from the backend's own files: with one finished line it is the
 * E2E-26 reply, with two or more it is the Event 1 reply. Every key is the key the screen sent.
 * It is a test double: it answers like the backend for Event 1 and for nothing else, which is all
 * the screen tests need to see the screen draw what comes back.
 */
export function previewFor(draft: BikriDraft): BikriPreview {
  const done = draft.lots[0]?.sales.filter((sale) => sale.buyer_account_id !== null).length ?? 0;
  const source = done <= 1 ? E2E26 : EVENT1;
  const base = clone(source);
  const template = base.lots[0];
  if (template === undefined) return base;
  const lots: LotPreview[] = draft.lots.map((lot, lotIndex) => {
    const made = clone(template);
    made.key = lot.key;
    made.lot_no = lotIndex + 1;
    made.lines = lot.sales.map((sale, index) => lineFor(template, index, sale));
    return made;
  });
  return { ...base, lots };
}

/** What the backend remembers about the truck LES-1234 (PRD FR-7.49, FR-7.50). */
export const TRUCK_MEMORY: TruckMemory = {
  last: {
    bikri_number: 1,
    date: '2026-10-09',
    adda_id: IDS.swat,
    zamindar_account_id: IDS.rahim,
    driver_name: 'Aslam',
    driver_phone: '03001234567',
  },
  same_day: { bikri_number: 1 },
  suggestions: [{ truck_no: 'LES-1234', date: '2026-10-09' }],
};

/** What the backend remembers about Rahim's last lot (PRD FR-7.51). */
export const SELLER_MEMORY: SellerMemory = {
  item_id: IDS.banana,
  packing_id: IDS.box,
  marka_id: null,
  from_last_lot: true,
};
