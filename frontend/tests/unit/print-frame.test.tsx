// The shared print header and footer, and the client's own layout of the buyer's bill (plan 17e,
// PRD FR-19.13, Section 21, R5, FR-12.12): the header with the firm, up to three contact persons
// and the trade mark in a box, none set = nothing drawn; the root that puts the print language,
// direction, paper and footer on a print; and R5, where the lines of تفصیل are shown exactly as the
// backend wrote them and the two amounts are the backend's own, never added up here. In Urdu and in
// English, with the canned print data of the PRD worked day.

import { render, within } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import type { Language } from '../../src/bindings/Language';
import type { PrintData } from '../../src/bindings/PrintData';
import { BikriPrintDocument } from '../../src/features/bikri/print/BikriPrintDocument';
import { LANGUAGES } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { printDataFor } from '../helpers/print-data';
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
function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}

describe.each(LANGUAGES)('the shared print header (%s)', (language: Language) => {
  const contacts =
    language === 'ur'
      ? ['منشی آصف بھٹی', 'ماہر عدنان', 'محمد اکرم']
      : ['Munshi Asif Bhatti', 'Maher Adnan', 'Muhammad Akram'];

  it('shows the firm, the three contacts with their phones, and the trade mark in a box', () => {
    const { root } = show(
      printDataFor('seller_bill', { language, contacts: true, tradeMark: 'RH' }),
    );
    const header = must(root.querySelector<HTMLElement>('.print-header'));
    const items = [...header.querySelectorAll('.print-contacts > li')];
    expect(items).toHaveLength(3);
    contacts.forEach((name, index) => {
      expect(textOf(must(items[index] as HTMLElement))).toContain(name);
    });
    expect(textOf(must(items[0] as HTMLElement))).toContain('0300-4058860');
    // A phone never flips inside the text: it sits in an isolated element.
    expect(
      within(header)
        .getAllByText('0300-4058860')
        .every((element) => element.closest('bdi')),
    ).toBe(true);
    const box = must(header.querySelector<HTMLElement>('.print-trade-mark'));
    expect(textOf(box)).toBe('RH');
    expect(box.getAttribute('aria-label')).toBe(message(language, 'print-header-trade-mark'));
    // The title, in the print language, and the firm are still there.
    expect(textOf(header)).toContain(message(language, 'print-seller-bill-title'));
    expect(textOf(header)).toContain(
      language === 'ur' ? 'ماہر عدنان اینڈ برادرز' : 'Maher Adnan and Brothers',
    );
  });

  it('draws no empty box and no empty line when no contact and no trade mark is set', () => {
    const { root } = show(printDataFor('seller_bill', { language }));
    expect(root.querySelector('.print-trade-mark')).toBeNull();
    expect(root.querySelector('.print-contacts')).toBeNull();
    // The rest of the header does not move: the firm, the mandi, the title and the facts.
    const header = must(root.querySelector<HTMLElement>('.print-header'));
    expect(textOf(header)).toContain(message(language, 'print-seller-bill-title'));
    expect(root.querySelectorAll('.print-fact')).toHaveLength(3);
  });

  it('a trade mark alone has its box; contacts alone have no box', () => {
    const mark = show(printDataFor('seller_bill', { language, tradeMark: 'RH' }));
    expect(mark.root.querySelector('.print-trade-mark')).not.toBeNull();
    expect(mark.root.querySelector('.print-contacts')).toBeNull();
    mark.unmount();
    const people = show(printDataFor('seller_bill', { language, contacts: true }));
    expect(people.root.querySelector('.print-trade-mark')).toBeNull();
    expect(people.root.querySelectorAll('.print-contacts > li')).toHaveLength(3);
  });

  it('the header of every sheet is the same, so print all has it on each buyer', () => {
    const { root } = show(
      printDataFor('buyer_slips', { language, contacts: true, tradeMark: 'RH' }),
    );
    const headers = [...root.querySelectorAll<HTMLElement>('.print-header')];
    expect(headers).toHaveLength(2);
    expect(textOf(must(headers[0]))).toBe(textOf(must(headers[1])));
  });
});

describe('the root of a print', () => {
  it('takes the print language, direction, paper and digit style, whatever the app language is', () => {
    useLanguageStore.setState({ language: 'en' });
    const { root } = show(printDataFor('seller_bill', { language: 'ur', paper: 'a5' }));
    expect(root.lang).toBe('ur');
    expect(root.dir).toBe('rtl');
    expect(root.dataset['paper']).toBe('a5');
    expect(root.dataset['digits']).toBe('latin');
    // Words are the print language's, not the app's.
    expect(textOf(root)).toContain(message('ur', 'print-seller-bill-title'));
  });

  it('in the hidden window the footer is the one the backend described, in the print language', () => {
    const window = show(printDataFor('seller_bill', { language: 'ur' }), true);
    const footer = document.documentElement.style.getPropertyValue(FOOTER_PROPERTY);
    expect(footer).toContain(message('ur', 'print-printed-at'));
    expect(footer).toContain('10-10-2026 17:40');
    expect(footer).toContain('counter(page)');
    expect(footer).toContain('counter(pages)');
    window.unmount();
    expect(document.documentElement.style.getPropertyValue(FOOTER_PROPERTY)).toBe('');
  });

  it('the preview line under a sheet says the print time with its label', () => {
    const { root } = show(printDataFor('seller_bill', { language: 'en' }));
    const line = must(root.querySelector<HTMLElement>('.print-screen-footer'));
    expect(textOf(line)).toBe(`${message('en', 'print-printed-at')} 10-10-2026 17:40`);
  });
});

describe.each(LANGUAGES)("the buyer slip in the client's layout, R5 (%s)", (language: Language) => {
  const banana = language === 'ur' ? 'کیلا' : 'Banana';
  const akram = language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax';

  function slips(change?: (data: PrintData) => PrintData) {
    const base = printDataFor('buyer_slips', { language });
    return show(change === undefined ? base : change(base));
  }

  it('is one row for the buyer: serial, party, تفصیل, رقم and رقم بعد لاگا, with the words of the print', () => {
    const { root } = slips();
    const table = must(root.querySelector<HTMLTableElement>('table.print-r5'));
    const heads = [...table.querySelectorAll('thead th')].map((cell) =>
      textOf(cell as HTMLElement),
    );
    expect(heads).toEqual(
      ['serial', 'party', 'details', 'value', 'total'].map((part) =>
        message(language, `print-r5-col-${part}`),
      ),
    );
    const rows = table.querySelectorAll('tbody tr');
    expect(rows).toHaveLength(1);
    const cells = [...must(rows[0]).querySelectorAll('td')].map((cell) => textOf(cell));
    expect(cells).toEqual(['1', akram, `(${banana} 20 . 600)`, '12,000', '12,880']);
  });

  it('writes each line as the backend wrote it: the spaces round the dot stay, nothing wraps, nothing is isolated as a whole', () => {
    const { root } = slips();
    const detail = must(root.querySelector<HTMLElement>('.print-r5-detail'));
    expect(detail.textContent).toBe(`(${banana} 20 . 600)`);
    // The class that keeps one line in one piece, and no <bdi> round the whole text.
    expect(detail.className).toContain('print-r5-detail');
    expect(detail.querySelector('bdi')).toBeNull();
    expect(detail.closest('ul')?.className).toContain('print-r5-details');
  });

  it('shows one line of تفصیل for each line of the buyer, in order', () => {
    const { root } = slips((data) => {
      const first = must(data.pages[0]);
      const line = must(first.lines[0]);
      return {
        ...data,
        pages: [
          {
            ...first,
            lines: [line, { ...line, qty: '5', rate: '560', detail: `(${banana} 5 . 560)` }],
          },
          ...data.pages.slice(1),
        ],
      };
    });
    const details = [
      ...must(root.querySelector('.print-sheet')).querySelectorAll('.print-r5-detail'),
    ];
    expect(details.map((item) => item.textContent)).toEqual([
      `(${banana} 20 . 600)`,
      `(${banana} 5 . 560)`,
    ]);
  });

  it("the two amounts are the backend's own lines, picked by their labels, and nothing is added up", () => {
    const { root } = slips((data) => ({
      ...data,
      pages: data.pages.map((page) => ({
        ...page,
        amounts: page.amounts.map((item) =>
          item.label_id === 'print-items-value'
            ? { ...item, amount: '777' }
            : item.label_id === 'print-buyer-pays'
              ? { ...item, amount: '999' }
              : item,
        ),
      })),
    }));
    const row = must(root.querySelector<HTMLElement>('table.print-r5 tbody tr'));
    const cells = [...row.querySelectorAll('td')].map((cell) => textOf(cell));
    // 777 + 720 + 160 is not 999: the template shows what it was given.
    expect(cells.slice(3)).toEqual(['777', '999']);
  });

  it('a line of an item sold by weight shows the weight in kilograms, never the نگ count next to the rate', () => {
    const { root } = slips((data) => {
      const first = must(data.pages[0]);
      const line = must(first.lines[0]);
      return {
        ...data,
        pages: [
          {
            ...first,
            lines: [
              {
                ...line,
                qty: '4',
                weight_kg: '12.345',
                rate: '590',
                detail: `(${banana} 4 . 590)`,
              },
            ],
          },
          ...data.pages.slice(1),
        ],
      };
    });
    const detail = must(root.querySelector<HTMLElement>('.print-r5-detail'));
    expect(textOf(detail)).toBe(`(${banana} 12.345 ${language === 'ur' ? 'کلو' : 'kg'} . 590)`);
    expect(textOf(detail)).not.toContain(' 4 ');
  });

  it("shows the tag after the buyer's name", () => {
    const { root } = slips((data) => ({
      ...data,
      pages: data.pages.map((page, index) =>
        index === 0 ? { ...page, party: { ...page.party, tag: 'K' } } : page,
      ),
    }));
    const row = must(root.querySelector<HTMLElement>('table.print-r5 tbody tr'));
    expect(textOf(must(row.querySelectorAll('td')[1] as HTMLElement))).toBe(`${akram} K`);
  });

  it('the roll has no columns: the same pieces one under the other, each with its words', () => {
    const { root } = show(printDataFor('buyer_slips', { language, paper: 'mm80' }));
    expect(root.querySelector('table.print-r5')).toBeNull();
    const first = must(root.querySelector<HTMLElement>('.print-r5-compact'));
    const text = textOf(first);
    for (const value of ['1', akram, `(${banana} 20 . 600)`, '12,000', '12,880']) {
      expect(text).toContain(value);
    }
    expect(text).toContain(message(language, 'print-r5-col-value'));
    expect(text).toContain(message(language, 'print-r5-col-total'));
  });

  it('the پختہ slip keeps its goods layout, with no R5 table', () => {
    const { root } = show(printDataFor('pukhta_slip', { language }));
    expect(root.querySelector('table.print-r5')).toBeNull();
    expect(root.querySelector('table.print-table')).not.toBeNull();
  });
});

describe.each(LANGUAGES)('the party of a print shows the short code (%s)', (language: Language) => {
  it('a seller or buyer shows the short code the backend sent, an account without one its full code, a one-time person none', () => {
    const data = printDataFor('seller_bill', { language });
    const page = must(data.pages[0]);
    const withShort = show(data);
    const block = must(withShort.root.querySelector<HTMLElement>('.print-party'));
    expect(textOf(block)).toContain(`${message(language, 'print-party-code')} 1`);
    expect(textOf(block)).not.toContain('030110001');
    withShort.unmount();
    // An account that has no short code shows its full code.
    const bare = { ...page.party };
    delete bare.short_code;
    const full = show({ ...data, pages: [{ ...page, party: bare }] });
    expect(textOf(must(full.root.querySelector<HTMLElement>('.print-party')))).toContain(
      '030110001',
    );
    full.unmount();
    const oneTime = show(printDataFor('pukhta_slip', { language }));
    expect(oneTime.root.querySelector('.print-party-extra')?.textContent).not.toContain('0304');
  });
});
