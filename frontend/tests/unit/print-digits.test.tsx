// The digits of every print (PRD FR-7.32, NFR-URD-14, Tech Stack 4.1, Umar 26-10-2026): in BOTH
// languages every number of a print (amounts, dates, phones, short codes, serials, counts, the page
// counters) is written with the English digits 0-9, never with Urdu or Arabic-Indic digits. The
// Urdu digits setting exists for Urdu prints and is OFF by default. This file is the ONE place that
// turns it on; no fixture, helper, screenshot or development page has it on by default.
//
// Every template is drawn here: the seller bill, the buyer slips, the پختہ slip, the receipt slip
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
import { REAL_SNAPSHOTS, realPrintData } from '../helpers/print-real';
import * as realVouchers from '../helpers/vouchers-real';
import { visible } from '../helpers/render';

const PAPERS: readonly Paper[] = ['a4', 'a5', 'mm80'];

/** Every character that is a decimal digit but not 0 to 9: Urdu, Arabic-Indic, and any other style. */
function oddDigits(text: string): string[] {
  return (text.match(/\p{Nd}/gu) ?? []).filter((char) => !/[0-9]/.test(char));
}
const URDU_DIGIT = /[۰-۹]/;
const ASCII_DIGIT = /[0-9]/;
/** The shapes of an amount or a count ("2,880", "۲,۸۸۰", "20") and of a date ("10-10-2026"). */
const AMOUNT_OR_DATE = /^[0-9۰-۹]{1,3}(,[0-9۰-۹]{3})*$|^[0-9۰-۹]{2}-[0-9۰-۹]{2}-[0-9۰-۹]{4}$/;
/** A phone number ("0300-4058860"): never converted. */
const PHONE = /^0[0-9۰-۹]{3}-[0-9۰-۹]{6,7}$/;

interface Template {
  name: string;
  draw: (options: CannedOptions) => ReactElement;
}

/** Each template with the canned data of the PRD worked day, header contacts and trade mark set. */
const TEMPLATES: readonly Template[] = [
  ...(['seller_bill', 'buyer_slips', 'pukhta_slip'] as const).map((doc) => ({
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
      'by default writes every number with the digits 0-9 only, on %s (text and footer)',
      (paper) => {
        const { container } = render(draw({ language, paper }));
        const root = theSheet(container);
        const text = visible(root.textContent);
        // There are numbers on the sheet, and none of them has an Urdu or Arabic-Indic digit.
        expect(text).toMatch(ASCII_DIGIT);
        expect(oddDigits(text)).toEqual([]);
        expect(root.dataset['digits']).toBe('latin');
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

  it('with the Urdu digits setting on, an Urdu print writes its numbers with ۰-۹, in the number cells and the footer', () => {
    for (const paper of PAPERS) {
      const { container, unmount } = render(draw({ language: 'ur', paper, urduDigits: true }));
      const root = theSheet(container);
      expect(root.dataset['digits']).toBe('urdu');
      expect(visible(root.textContent)).toMatch(URDU_DIGIT);
      const cells = [...root.querySelectorAll('.type-number')];
      const numbers = cells.map((item) => item.textContent);
      expect(numbers.length).toBeGreaterThan(0);
      // An amount, a count and a date have Urdu digits only: no 0-9 is left in them. The short code
      // column of the collection list is not one of them: a code is never converted.
      const figures = cells
        .filter((item) => item.closest('.print-col-code') === null)
        .map((item) => item.textContent)
        .filter((text) => AMOUNT_OR_DATE.test(text));
      expect(figures.length).toBeGreaterThan(0);
      for (const text of figures) expect(text).not.toMatch(ASCII_DIGIT);
      // A phone is never converted, even with the setting on (backend, 26-10-2026).
      for (const text of numbers.filter((value) => PHONE.test(value))) {
        expect(oddDigits(text)).toEqual([]);
      }
      expect(footerOf()).toMatch(URDU_DIGIT);
      if (paper !== 'mm80') expect(footerOf()).toContain('persian');
      unmount();
      document.documentElement.removeAttribute('style');
    }
  });
});

describe("the backend's real answers are written with the digits 0-9 only", () => {
  const draw = (data: PrintData): ReactElement => <BikriPrintDocument data={data} applyRoot />;

  it.each(REAL_SNAPSHOTS)(
    '$doc of $event in $language',
    (snapshot: (typeof REAL_SNAPSHOTS)[number]) => {
      const data = realPrintData(snapshot);
      expect(data.urdu_digits).toBe(false);
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
    expect(data.urdu_digits).toBe(false);
    const { container } = render(<ReceiptSlipTemplate data={data} applyRoot />);
    const text = visible(theSheet(container).textContent);
    expect(text).toMatch(ASCII_DIGIT);
    expect(oddDigits(text)).toEqual([]);
    expect(oddDigits(footerOf())).toEqual([]);
  });
});

describe('the default of the canned data is the English digits', () => {
  it('no canned print, in either language, has the Urdu digits setting on unless a test asks for it', () => {
    for (const language of LANGUAGES) {
      expect(printDataFor('seller_bill', { language }).urdu_digits).toBe(false);
      expect(voucherPrintData('receipt', { language }).urdu_digits).toBe(false);
      expect(collectionPrintData({ language }).urdu_digits).toBe(false);
    }
  });
});
