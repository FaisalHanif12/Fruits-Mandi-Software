// The real print data of the backend (plan 14, T4). The backend wrote it in its own `insta`
// snapshots (backend/crates/reports/tests/snapshots/*.snap), made from its real answer of
// `bikri_get` for the PRD worked day. The files are copied here, never edited. A snapshot is a four
// line header and then the JSON of the generated `PrintData` type, which is what is read here.

import fs from 'node:fs';
import path from 'node:path';
import type { Language } from '../../src/bindings/Language';
import type { PrintData } from '../../src/bindings/PrintData';
import type { PrintDoc } from '../../src/bindings/PrintDoc';

const FOLDER = path.resolve(import.meta.dirname, '../fixtures/print');

/** The snapshots there are: the document, which بکری of the worked day, the print language. */
export const REAL_SNAPSHOTS = [
  { doc: 'seller_bill', event: 'event1', language: 'ur' },
  { doc: 'seller_bill', event: 'event1', language: 'en' },
  { doc: 'seller_bill', event: 'event2', language: 'en' },
  { doc: 'buyer_slips', event: 'event1', language: 'ur' },
  { doc: 'buyer_slips', event: 'event1', language: 'en' },
  { doc: 'pukhta_slip', event: 'event2', language: 'ur' },
  { doc: 'pukhta_slip', event: 'event2', language: 'en' },
] as const satisfies readonly { doc: PrintDoc; event: string; language: Language }[];

export type RealSnapshot = (typeof REAL_SNAPSHOTS)[number];

const LANGUAGE_NAME: Record<Language, string> = { ur: 'urdu', en: 'english' };

/** The print data the backend made for one document of one بکری, in one language. */
export function realPrintData(snapshot: RealSnapshot): PrintData {
  const file = path.join(
    FOLDER,
    `print_data__snapshot_${snapshot.doc}_${snapshot.event}_${LANGUAGE_NAME[snapshot.language]}.snap`,
  );
  // The header of an insta snapshot is `---`, three lines, `---`.
  const body = fs.readFileSync(file, 'utf8').split('---\n').slice(2).join('---\n');
  return JSON.parse(body) as PrintData;
}

/** The first real snapshot of a document in a language, for tests that need "the real answer". */
export function realFor(doc: PrintDoc, language: Language): PrintData {
  const found = REAL_SNAPSHOTS.find((item) => item.doc === doc && item.language === language);
  if (found === undefined) throw new Error(`no real snapshot of ${doc} in ${language}`);
  return realPrintData(found);
}

/**
 * The shape of a value: its keys and the shape of each, with an array shown by its first element
 * and a null left open (a field can be null in one answer and text in another). Two answers of the
 * backend have the same shape when every field of one is a field of the other.
 */
export function shapeOf(value: unknown): unknown {
  if (value === null) return '?';
  if (Array.isArray(value)) return value.length === 0 ? [] : [shapeOf(value[0])];
  if (typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([key, item]) => [key, shapeOf(item)]),
    );
  }
  return typeof value;
}
