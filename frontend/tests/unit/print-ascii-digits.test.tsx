// Bug 3 of the client demo (D8, plan 63, decision D1, PRD FR-7.32, FR-19.6, NFR-URD-14): every number
// of an Urdu print is written with the digits 0 to 9 and grouped, like 2026 and 10,000. Never ۰ to ۹
// and never ٠ to ٩. There is no setting for it any more.
//
// The three prints of the acceptance (D8-A6) are drawn from the backend's real answers, in Urdu: the
// seller bill (R1), ادھاری تفصیل (R5) and the party day slip (R35). Each is drawn twice. Once as the
// backend answers now, and once as an OLD answer that still says "Urdu digits on" (the setting of an
// older version, kept in a computer's database). The page must not take any notice of that flag: the
// text, the footer counters, the digit style of the page and the fonts it loads stay 0-9 only.

import { render } from '@testing-library/react';
import fs from 'node:fs';
import path from 'node:path';
import type { ReactElement } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import type { BuyerBillsPrintData } from '../../src/bindings/BuyerBillsPrintData';
import type { PrintData } from '../../src/bindings/PrintData';
import type { ReportPrintData } from '../../src/bindings/ReportPrintData';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { BuyerBillsPrintDocument } from '../../src/features/reports/print/BuyerBillsPrintDocument';
import { ReportPrintDocument } from '../../src/features/reports/print/ReportPrintDocument';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { fontFamiliesOf } from '../../src/shared/print/wait-ready';
import { BUYER_BILLS_EN, BUYER_BILLS_UR } from '../helpers/reports-sales-real';
import { PRINT_SLIP_BUYER_EN, PRINT_SLIP_BUYER_UR } from '../helpers/day-slips-real';
import { PRINT_FRAME_EN, PRINT_FRAME_UR, printFrameFor } from '../helpers/print-frame';
import {
  REAL_COLLECTION_SNAPSHOTS,
  REAL_SNAPSHOTS,
  realCollectionData,
  realFor,
  realPrintData,
} from '../helpers/print-real';
import { visible } from '../helpers/render';

const SRC = path.resolve(import.meta.dirname, '../../src');

/** Every decimal digit that is not 0 to 9: Urdu (۰-۹), Arabic-Indic (٠-٩) and any other style. */
function oddDigits(text: string): string[] {
  return (text.match(/\p{Nd}/gu) ?? []).filter((char) => !/[0-9]/.test(char));
}

/** A number with its thousands commas, like 10,000 or 29,400. */
const GROUPED = /\d{1,3}(?:,\d{3})+/;

/**
 * The same answer as an old version of the backend made it when the Printing setting "Urdu digits"
 * was on: the flag is true. The text of the answer here is the 0-9 text of today, because this test
 * is about what the page does with the flag, not about what an old backend wrote.
 */
function withOldFlag<T extends object>(data: T): T {
  return { ...data, urdu_digits: true };
}

const footerOf = () => document.documentElement.style.getPropertyValue(FOOTER_PROPERTY);

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.removeAttribute('style');
});

function reportFrame(data: ReportPrintData | BuyerBillsPrintData, flag: boolean) {
  const frame = printFrameFor({
    language: data.language,
    title_id: data.title_id,
    facts: data.filters,
  });
  return { ...frame, dir: data.dir, ...(flag ? { urdu_digits: true } : {}) };
}

interface Print {
  name: string;
  /** What must be on the page, in 0-9 and grouped. */
  expected: readonly string[];
  draw: (oldFlag: boolean) => ReactElement;
}

const PRINTS: readonly Print[] = [
  {
    name: 'seller bill (R1)',
    expected: ['29,400', '19,316', '10-10-2026'],
    draw: (oldFlag) => {
      const real: PrintData = realFor('seller_bill', 'ur');
      return <BikriPrintDocument data={oldFlag ? withOldFlag(real) : real} applyRoot />;
    },
  },
  {
    name: 'ادھاری تفصیل (R5)',
    expected: ['12,000', '17,400'],
    draw: (oldFlag) => (
      <BuyerBillsPrintDocument
        data={oldFlag ? withOldFlag(BUYER_BILLS_UR) : BUYER_BILLS_UR}
        frame={reportFrame(BUYER_BILLS_UR, oldFlag)}
        applyRoot
      />
    ),
  },
  {
    name: 'party day slip (R35)',
    expected: ['10,000', '12,880', '10-10-2026'],
    draw: (oldFlag) => (
      <ReportPrintDocument
        data={oldFlag ? withOldFlag(PRINT_SLIP_BUYER_UR) : PRINT_SLIP_BUYER_UR}
        frame={reportFrame(PRINT_SLIP_BUYER_UR, oldFlag)}
        applyRoot
      />
    ),
  },
];

describe.each(PRINTS)('the Urdu $name', ({ expected, draw }) => {
  describe.each([
    ['as the backend answers now', false],
    ['as an old answer with the Urdu digits flag on', true],
  ] as const)('%s', (_label, oldFlag) => {
    it('has only the digits 0-9, grouped, in its text', () => {
      const { container } = render(draw(oldFlag));
      const root = container.querySelector<HTMLElement>('.print-doc');
      if (root === null) throw new Error('no print document');
      expect(root.getAttribute('lang')).toBe('ur');
      const text = visible(root.textContent);
      expect(oddDigits(text)).toEqual([]);
      expect(text).toMatch(GROUPED);
      for (const figure of expected) expect(text).toContain(figure);
      // The year is four plain digits (the date of the print is DD-MM-YYYY).
      expect(text).toMatch(/\b\d{2}-\d{2}-2026\b/);
    });

    it('has no digit style on the page: a number is never given the digits of another script', () => {
      const { container } = render(draw(oldFlag));
      expect(container.querySelector('.print-doc')?.getAttribute('data-digits')).toBeNull();
    });

    it('has the page counters and the print time of the footer in 0-9, with no Urdu digit style', () => {
      render(draw(oldFlag));
      expect(footerOf()).toContain('counter(page)');
      expect(footerOf()).not.toContain('persian');
      expect(oddDigits(footerOf())).toEqual([]);
      expect(footerOf()).toMatch(/\d{2}-\d{2}-\d{4}/);
    });

    it('draws every number with the number font, which has no Urdu digits to give', () => {
      const { container } = render(draw(oldFlag));
      const numbers = [...container.querySelectorAll('.type-number')];
      expect(numbers.length).toBeGreaterThan(0);
      for (const number of numbers) expect(oddDigits(number.textContent)).toEqual([]);
    });
  });
});

describe('the fonts a print loads', () => {
  it('are Inter and Gulzar in Urdu, and Inter alone in English, whatever an old answer says about digits', () => {
    const oldUrdu = { language: 'ur' as const, urdu_digits: true };
    const oldEnglish = { language: 'en' as const, urdu_digits: true };
    expect(fontFamiliesOf(oldUrdu)).toEqual(['Inter', 'Gulzar']);
    expect(fontFamiliesOf(oldEnglish)).toEqual(['Inter']);
  });
});

describe('the style sheets of a print', () => {
  const read = (file: string) => fs.readFileSync(path.join(SRC, file), 'utf8');

  it('have no Urdu digit font and no Urdu digit style of the page', () => {
    // The ten digits are Inter, in every language (`Mandi Digits`, fonts.css). Nothing puts a Naskh
    // face in front of a number any more.
    expect(read('shared/print/print.css')).not.toMatch(/data-digits|font-urdu-digits/);
    expect(read('styles/tokens.css')).not.toContain('font-urdu-digits');
  });
});

describe("the backend's real answers of a print", () => {
  // The backend half of plan 63 removes the field from every print answer and regenerates the files
  // these tests read (T3). Until it has, this one fails; the page above never looks at the flag.
  it('no longer say anything about Urdu digits', () => {
    const answers: readonly object[] = [
      ...REAL_SNAPSHOTS.map((snapshot) => realPrintData(snapshot)),
      ...REAL_COLLECTION_SNAPSHOTS.map((snapshot) => realCollectionData(snapshot)),
      BUYER_BILLS_UR,
      BUYER_BILLS_EN,
      PRINT_SLIP_BUYER_UR,
      PRINT_SLIP_BUYER_EN,
      PRINT_FRAME_UR,
      PRINT_FRAME_EN,
    ];
    for (const answer of answers) expect(Object.keys(answer)).not.toContain('urdu_digits');
  });
});
