// The printed documents (plan 14, PRD FR-12.12, FR-7.36, FR-19.3 to FR-19.7, NFR-URD-11 to
// NFR-URD-16, NFR-URD-28): the three templates with the canned print data of the PRD worked day, in
// Urdu and in English. Each value is shown exactly as the backend sent it, every number is
// isolated left to right, the print language is the print's own (not the app's), the paper class
// and the digit style are on the root, and print all gives one sheet for each buyer.

import { render, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { Language } from '../../src/bindings/Language';
import type { Paper } from '../../src/bindings/Paper';
import type { PrintData } from '../../src/bindings/PrintData';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { ReceiptSlipTemplate } from '../../src/features/vouchers/print/ReceiptSlipTemplate';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import {
  DEFAULT_PAPER,
  printDataFor,
  voucherPrintData,
  type SlipKind,
} from '../helpers/print-data';
import { message, visible } from '../helpers/render';

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
});

function show(data: PrintData, applyRoot = false) {
  const result = render(<BikriPrintDocument data={data} applyRoot={applyRoot} />);
  const root = result.container.querySelector<HTMLElement>('.print-doc');
  if (root === null) throw new Error('no print document');
  return { ...result, root };
}

const textOf = (element: HTMLElement) => visible(element.textContent);
/** The one element a query must find. */
function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}
const sheets = (root: HTMLElement) =>
  Array.from(root.querySelectorAll<HTMLElement>('.print-sheet'));

/** True when `value` is shown inside an element written left to right, so it never flips. */
function isolated(root: HTMLElement, value: string): boolean {
  const found = within(root)
    .getAllByText(
      (_, element) => element?.tagName === 'BDI' && textOf(element as HTMLElement) === value,
    )
    .map((element) => element.closest('bdi'));
  return found.length > 0 && found.every((bdi) => bdi?.getAttribute('dir') === 'ltr');
}

describe.each(LANGUAGES)('the seller bill (%s)', (language: Language) => {
  const data = printDataFor('seller_bill', { language });

  it('shows the PRD 4.4 numbers exactly as given, with the amount in words', () => {
    const { root } = show(data);
    const page = sheets(root);
    expect(page).toHaveLength(1);
    const text = textOf(root);
    for (const value of [
      '29,400',
      '3,000',
      '1,764',
      '250',
      '50',
      '5,000',
      '0',
      '20',
      '10,084',
      '19,316',
    ]) {
      expect(text).toContain(value);
    }
    const words =
      language === 'ur'
        ? 'انیس ہزار تین سو سولہ روپے'
        : 'Nineteen thousand three hundred sixteen rupees only';
    expect(text).toContain(words);
    // Every label is the Fluent message of the PRINT language.
    for (const id of [
      'print-seller-bill-title',
      'print-gross-sale',
      'print-freight',
      'print-commission',
      'print-labour',
      'print-munshiana',
      'print-cash-paid',
      'print-store-rent',
      'print-phone-charge',
      'print-total-deductions',
      'print-net-sale',
      'print-in-words',
      'print-fact-number',
      'print-fact-date',
      'print-fact-truck',
      'print-party-seller',
    ]) {
      expect(text).toContain(message(language, id));
    }
  });

  it('shows the firm, the facts, the seller, and the buyers on the lines', () => {
    const { root } = show(data);
    const text = textOf(root);
    expect(text).toContain(data.header.firm.name);
    expect(text).toContain(data.header.firm.mandi);
    expect(text).toContain('LES-1234');
    expect(text).toContain('10-10-2026');
    // The seller shows his short code (PRD FR-19.13), not the long one.
    expect(textOf(must(root.querySelector<HTMLElement>('.print-party')))).toContain(
      `${message(language, 'print-party-code')} 0`,
    );
    expect(text).toContain(language === 'ur' ? 'رحیم' : 'Rahim');
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(text).toContain(language === 'ur' ? 'ارشد ملتانی' : 'Arshad Multani');
    // The lines, a row each, with the heads of a goods table.
    const rows = root.querySelectorAll('tbody tr');
    expect(rows.length).toBeGreaterThanOrEqual(2);
    expect(textOf(root.querySelector('thead') as HTMLElement)).toContain(
      message(language, 'print-col-qty'),
    );
  });

  it('writes every number from the backend inside a left-to-right isolate, so none flips', () => {
    const { root } = show(data);
    for (const value of [
      '29,400',
      '10,084',
      '19,316',
      '600',
      '580',
      '12,000',
      '17,400',
      '10-10-2026',
      'LES-1234',
      '1',
    ]) {
      expect(isolated(root, value), `${value} is isolated`).toBe(true);
    }
  });

  it('has the language, direction and paper of the PRINT, whatever the app language is', () => {
    // The app is in the other language; the print keeps its own.
    useLanguageStore.setState({ language: language === 'ur' ? 'en' : 'ur' });
    const { root } = show(data);
    expect(root.lang).toBe(language);
    expect(root.dir).toBe(language === 'ur' ? 'rtl' : 'ltr');
    expect(root.dataset['paper']).toBe('a4');
    expect(useLanguageStore.getState().language).toBe(language === 'ur' ? 'en' : 'ur');
    expect(textOf(root)).toContain(message(language, 'print-seller-bill-title'));
  });

  it('puts the print time in the preview footer and the page footer on the root element', () => {
    const { root, unmount } = show(data, true);
    expect(textOf(must(root.querySelector('.print-screen-footer')))).toContain('10-10-2026 17:40');
    const footer = document.documentElement.style.getPropertyValue(FOOTER_PROPERTY);
    expect(footer).toContain(message(language, 'print-page-label'));
    expect(footer).toContain(message(language, 'print-page-of'));
    expect(footer).toContain('counter(page)');
    expect(footer).toContain('counter(pages)');
    expect(footer).toContain('10-10-2026 17:40');
    // The window's own language and direction are the print's while it is open.
    expect(document.documentElement.lang).toBe(language);
    unmount();
    expect(document.documentElement.style.getPropertyValue(FOOTER_PROPERTY)).toBe('');
  });
});

describe.each(LANGUAGES)('the buyer slips, print all (%s)', (language: Language) => {
  const data = printDataFor('buyer_slips', { language });

  it('is one sheet for each buyer, each with its own header', () => {
    const { root } = show(data);
    const page = sheets(root);
    expect(page).toHaveLength(2);
    const title = message(language, 'print-buyer-slip-title');
    for (const sheet of page) expect(textOf(sheet)).toContain(title);
    const first = textOf(must(page[0]));
    const second = textOf(must(page[1]));
    // The client's own layout (R5): the buyer's name, then his line written "(item qty . rate)".
    const banana = language === 'ur' ? 'کیلا' : 'Banana';
    for (const value of [`(${banana} 20 . 600)`, '12,000', '720', '160', '12,880']) {
      expect(first).toContain(value);
    }
    for (const value of [`(${banana} 30 . 580)`, '17,400', '1,044', '240', '18,684']) {
      expect(second).toContain(value);
    }
    expect(first).not.toContain('18,684');
    expect(second).not.toContain('12,880');
  });

  it('shows how the buyer pays and his total in words', () => {
    const { root } = show(data);
    const first = textOf(must(sheets(root)[0]));
    expect(first).toContain(message(language, 'print-payment-credit'));
    expect(first).toContain(
      language === 'ur'
        ? 'بارہ ہزار آٹھ سو اسی روپے'
        : 'Twelve thousand eight hundred eighty rupees only',
    );
  });
});

describe.each(LANGUAGES)('the پختہ slip (%s)', (language: Language) => {
  it('shows the one-time buyer with his phone, the total 12,800, paid, and its words', () => {
    const data = printDataFor('pukhta_slip', { language });
    const { root } = show(data);
    expect(sheets(root)).toHaveLength(1);
    const text = textOf(root);
    expect(text).toContain(language === 'ur' ? 'شاہد' : 'Shahid');
    expect(text).toContain('0300-0000000');
    expect(text).toContain(message(language, 'print-party-onetime-buyer'));
    expect(text).toContain('12,000');
    expect(text).toContain('720');
    expect(text).toContain('80');
    expect(text).toContain('12,800');
    expect(text).toContain(message(language, 'print-payment-cash'));
    expect(text).toContain(
      language === 'ur' ? 'بارہ ہزار آٹھ سو روپے' : 'Twelve thousand eight hundred rupees only',
    );
    // A one-time buyer has no code.
    expect(root.querySelector('.print-party-extra')?.textContent).not.toContain('0304');
    expect(isolated(root, '0300-0000000')).toBe(true);
  });
});

describe('paper classes', () => {
  const PAPERS_UNDER_TEST: Paper[] = ['a4', 'a5', 'mm80'];

  it.each(PAPERS_UNDER_TEST)('puts the %s class on the root', (paper) => {
    const { root } = show(printDataFor('buyer_slips', { paper }));
    expect(root.dataset['paper']).toBe(paper);
  });

  it('starts every document on the paper Settings gave it (A4 bill, A5 slips)', () => {
    expect(DEFAULT_PAPER).toEqual({
      seller_bill: 'a4',
      buyer_slips: 'a5',
      pukhta_slip: 'a5',
      receipt_slip: 'a5',
      collection_list: 'a4',
    });
    for (const doc of ['seller_bill', 'buyer_slips', 'pukhta_slip'] as const) {
      const { root, unmount } = show(printDataFor(doc));
      expect(root.dataset['paper']).toBe(DEFAULT_PAPER[doc]);
      unmount();
    }
  });

  it('the 80 mm roll has no columns: the pieces of the buyer are written one under the other', () => {
    const { root } = show(printDataFor('buyer_slips', { paper: 'mm80', language: 'ur' }));
    expect(root.querySelector('table.print-table')).toBeNull();
    expect(root.querySelectorAll('.print-r5-compact')).toHaveLength(2);
    // One line of تفصیل for each buyer, exactly as the backend wrote it.
    const details = [...root.querySelectorAll('.print-r5-detail')].map((item) => item.textContent);
    expect(details).toEqual(['(کیلا 20 . 600)', '(کیلا 30 . 580)']);
  });

  it('the goods of the seller bill on the roll are two rows of text, with "20 @ 600" kept in order', () => {
    const { root } = show(printDataFor('seller_bill', { paper: 'mm80', language: 'ur' }));
    expect(root.querySelector('table.print-table')).toBeNull();
    expect(root.querySelectorAll('.print-compact-line')).toHaveLength(2);
    expect(isolated(root, '20 @ 600')).toBe(true);
  });

  it('A4 and A5 use a table with a header row that repeats', () => {
    const { root } = show(printDataFor('seller_bill'));
    expect(root.querySelector('table.print-table > thead')).not.toBeNull();
  });
});

describe('the hidden window and the roll', () => {
  it('marks the document of the hidden window, and not the preview', () => {
    const data = printDataFor('seller_bill');
    const window = show(data, true);
    expect(window.root.dataset['window']).toBe('true');
    window.unmount();
    expect(show(data, false).root.dataset['window']).toBeUndefined();
  });

  it('puts only the print time in the footer of the 80 mm roll, and the page count on A4 and A5', () => {
    const roll = show(printDataFor('buyer_slips', { paper: 'mm80', language: 'en' }), true);
    const rollFooter = document.documentElement.style.getPropertyValue(FOOTER_PROPERTY);
    expect(rollFooter).toContain('10-10-2026 17:40');
    expect(rollFooter).not.toContain('counter(');
    roll.unmount();
    const sheet = show(printDataFor('buyer_slips', { paper: 'a5', language: 'en' }), true);
    expect(document.documentElement.style.getPropertyValue(FOOTER_PROPERTY)).toContain(
      'counter(pages)',
    );
    sheet.unmount();
  });
});

describe('what the backend sends is what is shown', () => {
  it('does not change a number: a figure in any grouping is shown as it came', () => {
    const data = printDataFor('seller_bill');
    const page = data.pages[0];
    if (page === undefined) throw new Error('no page');
    const odd: PrintData = {
      ...data,
      pages: [
        {
          ...page,
          amounts: [{ label_id: 'print-net-sale', amount: '13,16,040', role: 'total' }],
          words: null,
        },
      ],
    };
    const { root } = show(odd);
    expect(textOf(root)).toContain('13,16,040');
  });

  it('a seller who owes the firm gets the "owes" label and the amount without a minus', () => {
    const data = printDataFor('seller_bill', { language: 'en' });
    const page = data.pages[0];
    if (page === undefined) throw new Error('no page');
    const owes: PrintData = {
      ...data,
      pages: [
        { ...page, amounts: [{ label_id: 'print-seller-owes', amount: '1,050', role: 'total' }] },
      ],
    };
    const { root } = show(owes);
    const text = textOf(root);
    expect(text).toContain(message('en', 'print-seller-owes'));
    expect(text).toContain('1,050');
    expect(text).not.toContain('-1,050');
  });

  it('a cancelled بکری is stamped', () => {
    const { root } = show(printDataFor('seller_bill', { language: 'en', cancelled: true }));
    expect(textOf(must(root.querySelector('.print-stamp')))).toBe(
      message('en', 'print-cancelled-stamp'),
    );
  });

  it('the weight and date columns come only when a line has a weight or another day', () => {
    const base = printDataFor('seller_bill', { language: 'en' });
    const page = base.pages[0];
    const line = page?.lines[0];
    if (page === undefined || line === undefined) throw new Error('no page');
    const plain = show(base);
    const heads = textOf(plain.root.querySelector('thead') as HTMLElement);
    expect(heads).not.toContain(message('en', 'print-col-weight'));
    expect(heads).not.toContain(message('en', 'print-col-date'));
    plain.unmount();
    const later: PrintData = {
      ...base,
      pages: [
        {
          ...page,
          lines: [{ ...line, weight_kg: '12.345', date: '11-10-2026', note: 'second sale' }],
        },
      ],
    };
    const shown = show(later);
    const laterHeads = textOf(shown.root.querySelector('thead') as HTMLElement);
    expect(laterHeads).toContain(message('en', 'print-col-weight'));
    expect(laterHeads).toContain(message('en', 'print-col-date'));
    expect(textOf(shown.root)).toContain('12.345');
    expect(textOf(shown.root)).toContain('11-10-2026');
    expect(textOf(shown.root)).toContain('second sale');
  });

  it('says so when there is nothing to print', () => {
    const data = printDataFor('pukhta_slip', { language: 'en' });
    const { root } = show({ ...data, pages: [] });
    expect(textOf(root)).toContain(message('en', 'print-preview-empty'));
    expect(sheets(root)).toHaveLength(0);
  });
});

describe.each(LANGUAGES)('the receipt slip template (%s)', (language: Language) => {
  const show = (kind: SlipKind, options: Parameters<typeof voucherPrintData>[1] = {}) => {
    const data = voucherPrintData(kind, { language, ...options });
    const result = render(<ReceiptSlipTemplate data={data} />);
    const root = result.container.querySelector<HTMLElement>('.print-doc');
    if (root === null) throw new Error('no print document');
    return { ...result, root, data };
  };

  it('a receipt: the title, the number, the party, one total and the amount in words', () => {
    const { root } = show('receipt');
    const text = textOf(root);
    expect(text).toContain(message(language, 'print-receipt-slip-title'));
    expect(text).toContain(language === 'ur' ? 'و-1' : 'R-1');
    expect(text).toContain(message(language, 'print-party-buyer'));
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    // The buyer shows his short code.
    expect(text).toContain(`${message(language, 'print-party-code')} 0`);
    expect(text).toContain(message(language, 'print-slip-amount-received'));
    expect(text).toContain('10,000');
    expect(text).toContain(language === 'ur' ? 'دس ہزار روپے' : 'Ten thousand rupees only');
    // A slip has no goods table.
    expect(root.querySelector('table.print-goods')).toBeNull();
    expect(root.querySelectorAll('.print-sheet')).toHaveLength(1);
  });

  it('has no line about how it was paid: all money is روکڑ (PRD 15.1)', () => {
    for (const kind of ['receipt', 'payment', 'advance'] as const) {
      const { root, unmount } = show(kind);
      expect(root.querySelector('.print-payment')).toBeNull();
      unmount();
    }
  });

  it('a payment and an advance have their own title and total label', () => {
    for (const [kind, title, total, amount] of [
      ['payment', 'print-payment-slip-title', 'print-slip-amount-paid', '4,316'],
      ['advance', 'print-advance-slip-title', 'print-slip-amount-advance', '5,000'],
    ] as const) {
      const { root, unmount } = show(kind);
      const text = textOf(root);
      expect(text).toContain(message(language, title));
      expect(text).toContain(message(language, total));
      expect(text).toContain(amount);
      expect(text).toContain(message(language, 'print-party-seller'));
      unmount();
    }
  });

  it('the extras show reason, collector and narration', () => {
    const { root } = show('receipt', { extras: true });
    const text = textOf(root);
    expect(text).toContain(message(language, 'print-fact-collector'));
    expect(text).toContain(message(language, 'print-fact-reason'));
    expect(text).toContain(message(language, 'print-fact-narration'));
  });

  it('the paper class and the direction are on the root; the roll is a slip too', () => {
    const { root } = show('receipt', { paper: 'mm80' });
    expect(root.getAttribute('data-paper')).toBe('mm80');
    expect(root.getAttribute('dir')).toBe(language === 'ur' ? 'rtl' : 'ltr');
    expect(root.getAttribute('lang')).toBe(language);
  });

  it('a cancelled voucher is stamped', () => {
    const { root } = show('receipt', { cancelled: true });
    expect(textOf(root)).toContain(message(language, 'print-cancelled-stamp'));
  });

  it('the numbers keep their order inside the sentence (isolated left to right)', () => {
    const { root } = show('receipt');
    const amount = within(root)
      .getAllByText(
        (_, element) => element?.tagName === 'BDI' && textOf(element as HTMLElement) === '10,000',
      )
      .map((element) => element.closest('bdi'));
    expect(amount.length).toBeGreaterThan(0);
    expect(amount.every((bdi) => bdi?.getAttribute('dir') === 'ltr')).toBe(true);
  });
});
