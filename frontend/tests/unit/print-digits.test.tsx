// The digits of every print (PRD FR-7.32, NFR-URD-14, Tech Stack 4.1, Umar 26-10-2026 and 09-11-2026):
// in BOTH languages every number of a print (amounts, dates, phones, short codes, serials, counts, the
// page counters) is written with the English digits 0-9, never with Urdu or Arabic-Indic digits. The
// Urdu digits setting that once existed is removed (plan 60 D1, plan 63), so there is nothing to turn
// on. See also print-ascii-digits.test.tsx, which draws the Urdu seller bill, R5 and R35.
//
// Every template is drawn here: the seller bill, the buyer slips, the receipt slip
// (receipt, payment, advance), the collection list, with the shared header (contacts and phones
// included) and footer, in Urdu and in English on A4, A5 and the 80 mm roll, and the backend's own
// real answers. The text of the sheet and the footer must hold no digit outside 0-9.

import { render } from '@testing-library/react';
import type { ReactElement } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import type { Language } from '../../src/bindings/Language';
import type { Paper } from '../../src/bindings/Paper';
import type { PrintData } from '../../src/bindings/PrintData';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { CollectionListTemplate } from '../../src/features/vouchers/print/CollectionListTemplate';
import { ReceiptSlipTemplate } from '../../src/features/vouchers/print/ReceiptSlipTemplate';
import { LANGUAGES } from '../../src/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import {
  collectionPrintData,
  printDataFor,
  voucherPrintData,
  type CannedOptions,
} from '../helpers/print-data';
import {
  REAL_COLLECTION_SNAPSHOTS,
  REAL_SNAPSHOTS,
  realCollectionData,
  realPrintData,
} from '../helpers/print-real';
import * as realVouchers from '../helpers/vouchers-real';
import { visible } from '../helpers/render';

const PAPERS: readonly Paper[] = ['a4', 'a5', 'mm80'];

/** Every character that is a decimal digit but not 0 to 9: Urdu, Arabic-Indic, and any other style. */
function oddDigits(text: string): string[] {
  return (text.match(/\p{Nd}/gu) ?? []).filter((char) => !/[0-9]/.test(char));
}
const ASCII_DIGIT = /[0-9]/;

interface Template {
  name: string;
  draw: (options: CannedOptions) => ReactElement;
}

/** Each template with the canned data of the PRD worked day, header contacts and trade mark set. */
const TEMPLATES: readonly Template[] = [
  ...(['seller_bill', 'buyer_slips'] as const).map((doc) => ({
    name: doc,
    draw: (options: CannedOptions) => (
      <BikriPrintDocument
        data={printDataFor(doc, { ...options, contacts: true, tradeMark: 'RH' })}
        applyRoot
      />
    ),
  })),
  ...(['receipt', 'payment', 'advance'] as const).map((kind) => ({
    name: `receipt slip (${kind})`,
    draw: (options: CannedOptions) => (
      <ReceiptSlipTemplate
        data={voucherPrintData(kind, { ...options, contacts: true, tradeMark: 'RH' })}
        applyRoot
      />
    ),
  })),
  {
    name: 'collection list',
    draw: (options: CannedOptions) => (
      <CollectionListTemplate
        data={collectionPrintData({ ...options, contacts: true, tradeMark: 'RH' })}
        applyRoot
      />
    ),
  },
];

function theSheet(container: HTMLElement): HTMLElement {
  const root = container.querySelector<HTMLElement>('.print-doc');
  if (root === null) throw new Error('no print document');
  return root;
}
const footerOf = () => document.documentElement.style.getPropertyValue(FOOTER_PROPERTY);

afterEach(() => {
  document.documentElement.removeAttribute('style');
});

describe.each(TEMPLATES)('the $name print', ({ draw }) => {
  describe.each(LANGUAGES)('in %s', (language: Language) => {
    it.each(PAPERS)(
      'writes every number with the digits 0-9 only, on %s (text and footer)',
      (paper) => {
        const { container } = render(draw({ language, paper }));
        const root = theSheet(container);
        const text = visible(root.textContent);
        // There are numbers on the sheet, and none of them has an Urdu or Arabic-Indic digit.
        expect(text).toMatch(ASCII_DIGIT);
        expect(oddDigits(text)).toEqual([]);
        expect(root.dataset['digits']).toBeUndefined();
        // The footer (print time and page counters) too: no Urdu digit style for the counters.
        expect(footerOf()).toMatch(ASCII_DIGIT);
        expect(oddDigits(footerOf())).toEqual([]);
        expect(footerOf()).not.toContain('persian');
        // Each number the template wraps is drawn with the number font, never with the font of the
        // words, so the digit shapes are Inter's (Code Standards F17).
        for (const number of root.querySelectorAll('.type-number')) {
          expect(oddDigits(number.textContent)).toEqual([]);
        }
      },
    );
  });
});

describe("the backend's real answers are written with the digits 0-9 only", () => {
  const draw = (data: PrintData): ReactElement => <BikriPrintDocument data={data} applyRoot />;

  it.each(REAL_SNAPSHOTS)(
    '$doc of $event in $language',
    (snapshot: (typeof REAL_SNAPSHOTS)[number]) => {
      const data = realPrintData(snapshot);
      const { container } = render(draw(data));
      const text = visible(theSheet(container).textContent);
      expect(text).toMatch(ASCII_DIGIT);
      expect(oddDigits(text)).toEqual([]);
      expect(oddDigits(footerOf())).toEqual([]);
    },
  );

  it.each([
    ['a receipt in Urdu', realVouchers.SLIP_RECEIPT_UR],
    ['a receipt in English', realVouchers.SLIP_RECEIPT_EN],
    ['a payment in English', realVouchers.SLIP_PAYMENT_EN],
    ['an advance in Urdu', realVouchers.SLIP_ADVANCE_UR],
  ] as const)('%s', (_name, data) => {
    const { container } = render(<ReceiptSlipTemplate data={data} applyRoot />);
    const text = visible(theSheet(container).textContent);
    expect(text).toMatch(ASCII_DIGIT);
    expect(oddDigits(text)).toEqual([]);
    expect(oddDigits(footerOf())).toEqual([]);
  });
});

describe('the real collection lists of the backend', () => {
  it.each(REAL_COLLECTION_SNAPSHOTS)(
    '$name',
    (snapshot: (typeof REAL_COLLECTION_SNAPSHOTS)[number]) => {
      const data = realCollectionData(snapshot);
      const { container } = render(<CollectionListTemplate data={data} applyRoot />);
      const root = theSheet(container);
      const text = visible(root.textContent);
      // 0-9 only, in the sheet and in the footer.
      expect(text).toMatch(ASCII_DIGIT);
      expect(oddDigits(text)).toEqual([]);
      expect(oddDigits(footerOf())).toEqual([]);
      expect(root.dataset['digits']).toBeUndefined();
    },
  );
});

describe('the canned data of the tests', () => {
  it('has no Urdu digit flag: a print has only one kind of digit', () => {
    for (const language of LANGUAGES) {
      expect(Object.keys(printDataFor('seller_bill', { language }))).not.toContain('urdu_digits');
      expect(Object.keys(voucherPrintData('receipt', { language }))).not.toContain('urdu_digits');
      expect(Object.keys(collectionPrintData({ language }))).not.toContain('urdu_digits');
    }
  });
});
