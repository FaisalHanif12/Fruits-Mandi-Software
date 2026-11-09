// 1,000 sale lines in one lot (D8, plan 61 T4, NFR-SCL-3 v1.17): the draft, and a stand-in for
// `bikri_preview` that answers for it the way the backend does in size and shape. Used by the
// development page (`#/dev/bikri?thousand`) to time typing, moving and the live summary in a real
// browser. No test library is imported, so the page can use it.
//
// The answer is built once as text and parsed again on every call, like an answer that comes over the
// Tauri bridge (the webview parses the JSON of every command). The figures are not real: the gross is
// the sum of the typed quantities, so a test can tell when the summary has followed a change. Nothing
// here is shown as money by the app except through the usual cells.

import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import type { LinePreview } from '../../src/bindings/LinePreview';
import type { BikriDraftStores } from '../../src/features/bikri/draft-store';
import { LOT, ON, SALE } from '../../src/features/bikri/types';
import { EVENT1, IDS } from './bikri-data';
import { paisaForTest } from './paisa';

export const THOUSAND = 1_000;
const QTY_EACH = 10;
const RATE = '600';
const AKRAM = { id: IDS.akram, code: '0020', name_ur: 'اکرم ٹول ٹیکس', name_en: 'Akram Toll Tax' };
const RAHIM = { id: IDS.rahim, code: '0010', name_ur: 'رحیم', name_en: 'Rahim' };
const BANANA = { id: IDS.banana, code: '0001', name_ur: 'کیلا', name_en: 'Banana' };
const BOX = { id: IDS.box, code: '0001', name_ur: 'پیٹی', name_en: 'Box' };

/** One lot of Rahim's bananas, 10,000 received, and `count` lines of 10 at 600 to Akram. */
export function fillThousandLines(stores: BikriDraftStores, count: number = THOUSAND): void {
  stores.header.getState().patch({ truckNo: 'LES-1234', zamindar: RAHIM });
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) return;
  stores.lots.getState().patchRow(lot.rowId, {
    picked: { [LOT.seller]: RAHIM, [LOT.item]: BANANA, [LOT.packing]: BOX },
    typed: {
      [LOT.qty]: String(count * QTY_EACH),
      [LOT.sellerFilled]: ON,
      [LOT.memoryFilledFor]: String(IDS.rahim),
    },
  });
  const sales = stores.salesFor(lot.rowId);
  const first = sales.getState().rows[0];
  if (first === undefined) return;
  const line = { qty: String(QTY_EACH), rate: RATE, qtyFilled: ON };
  sales.getState().patchRow(first.rowId, { picked: { [SALE.buyer]: AKRAM }, typed: line });
  for (let at = 1; at < count; at += 1) sales.getState().addRow({ ...line }, { buyer: AKRAM });
}

/** The gross in the text of an answer is this number until the answer is given for a draft. */
const GROSS_MARK = 123_456_789_012_300;

/** The answer as text for these keys (the lot and its lines), made once and kept. */
const cache = new Map<string, string>();

function answerText(draft: BikriDraft): string {
  const lot = draft.lots[0];
  const template = EVENT1.lots[0];
  const done = template?.lines[0];
  if (lot === undefined || template === undefined || done?.calc == null) throw new Error('no lot');
  const sum = lot.sales.reduce((total, sale) => total + Number(sale.qty || '0'), 0);
  const signature = `${lot.key}|${String(lot.sales.length)}|${lot.sales[0]?.key ?? ''}|${lot.sales.at(-1)?.key ?? ''}`;
  let base = cache.get(signature);
  if (base === undefined) {
    const lines: LinePreview[] = lot.sales.map((sale, index) => {
      const line = structuredClone(done);
      line.key = sale.key;
      line.line_no = index + 1;
      return line;
    });
    const preview = structuredClone(EVENT1);
    const made = structuredClone(template);
    made.key = lot.key;
    made.lines = lines;
    preview.lots = [made];
    for (const target of [made.totals, preview.summary]) {
      if (target !== null) target.gross = paisaForTest(GROSS_MARK);
    }
    // Held as plain data (no money type), as it is on the wire, so it can be written out as text.
    const plain: unknown = preview;
    base = JSON.stringify(plain);
    cache.clear();
    cache.set(signature, base);
  }
  return base.replaceAll(String(GROSS_MARK), String(sum * 100));
}

/** The answer for the draft: parsed from text on every call, like an answer over the bridge. */
export function thousandAnswer(draft: BikriDraft): BikriPreview {
  return JSON.parse(answerText(draft)) as BikriPreview;
}

/** How large the answer is, in characters, for the report. */
export function thousandAnswerSize(draft: BikriDraft): number {
  return answerText(draft).length;
}
