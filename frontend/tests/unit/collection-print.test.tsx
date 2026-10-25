// The collection list on paper (plan 17e, PRD FR-15.6, FR-7.61, FR-19.13): the template with the
// canned print data (groups by collector and city, each with its total, the total OF THIS LIST and
// the buyer count, every number as the backend sent it and nothing added up), the print box (language,
// printer or PDF, copies, preview, the refusal in words), and the print setting. In Urdu and in
// English, on A4, A5 and the 80 mm roll.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { CollectionPrintData } from '../../src/bindings/CollectionPrintData';
import * as reportsApi from '../../src/features/reports/api';
import * as settingsApi from '../../src/features/settings/api';
import { CollectionListTemplate } from '../../src/features/vouchers/print/CollectionListTemplate';
import { CollectionPrintBox } from '../../src/features/vouchers/print/CollectionPrintBox';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { FOOTER_PROPERTY } from '../../src/shared/print';
import { fail } from '../helpers/app';
import { collectionPrintData } from '../helpers/print-data';
import { printingSettings } from '../helpers/print-settings';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/settings/api');

const textOf = (element: HTMLElement) => visible(element.textContent);
function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}

function show(data: CollectionPrintData, applyRoot = false) {
  const result = render(<CollectionListTemplate data={data} applyRoot={applyRoot} />);
  const root = must(result.container.querySelector<HTMLElement>('.print-doc'));
  return { ...result, root };
}

afterEach(() => {
  useLanguageStore.setState({ language: 'ur' });
  document.documentElement.removeAttribute('style');
});

describe.each(LANGUAGES)('the collection list template (%s)', (language: Language) => {
  const ur = language === 'ur';

  it('has the shared header with its title and facts, and one block for each collector and city', () => {
    const { root } = show(collectionPrintData({ language, contacts: true, tradeMark: 'RH' }));
    const header = must(root.querySelector<HTMLElement>('.print-header'));
    expect(textOf(header)).toContain(message(language, 'print-collection-list-title'));
    expect(root.querySelector('.print-trade-mark')).not.toBeNull();
    expect(root.querySelectorAll('.print-contacts > li')).toHaveLength(3);
    // The facts: the day, the collector and the city, each with its label.
    const facts = [...root.querySelectorAll('.print-fact')].map((fact) =>
      textOf(fact as HTMLElement),
    );
    expect(facts).toEqual([
      `${message(language, 'print-fact-date')}10-10-2026`,
      `${message(language, 'print-fact-collector')}${ur ? 'اسلم' : 'Aslam'}`,
      `${message(language, 'print-fact-city')}${ur ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan'}`,
    ]);
    const titles = [...root.querySelectorAll('.print-collection-group-title')].map((item) =>
      textOf(item as HTMLElement),
    );
    expect(titles).toEqual([
      `${ur ? 'اسلم' : 'Aslam'} · ${ur ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan'}`,
      `${message(language, 'print-collection-no-collector')} · ${message(language, 'print-collection-no-city')}`,
    ]);
  });

  it('labels every column, and each row has its serial, short code, name with tag, phone, dues, oldest bill and days', () => {
    const { root } = show(collectionPrintData({ language, tagged: true }));
    const first = must(root.querySelector<HTMLTableElement>('table.print-collection'));
    const heads = [...first.querySelectorAll('thead th')].map((cell) =>
      textOf(cell as HTMLElement),
    );
    expect(heads).toEqual(
      ['serial', 'code', 'party', 'dues', 'oldest', 'days', 'note'].map((part) =>
        message(language, `print-collection-col-${part}`),
      ),
    );
    const rows = [...first.querySelectorAll('tbody tr')].filter(
      (row) => !row.classList.contains('print-amount-subtotal'),
    );
    expect(rows).toHaveLength(2);
    const cells = [...must(rows[0]).querySelectorAll('td')].map((cell) => textOf(cell));
    // The phone is under the name, in the same cell, so a narrow sheet keeps room for the name.
    expect(cells).toEqual([
      '1',
      '1',
      `${ur ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax'} K0300-000001`,
      '2,880',
      '10-10-2026',
      '0',
      '',
    ]);
    expect(must(must(rows[0]).querySelector('.print-collection-phone')).textContent).toBe(
      '0300-000001',
    );
  });

  it('leaves the note column empty for the collector to write in, on every row', () => {
    const { root } = show(collectionPrintData({ language }));
    const notes = [...root.querySelectorAll('td.print-col-note')];
    expect(notes).toHaveLength(3);
    expect(notes.every((cell) => cell.textContent === '')).toBe(true);
  });

  it('a buyer with no phone and no open bill date has empty cells, not a dash or a zero', () => {
    const { root } = show(collectionPrintData({ language }));
    const second = [...root.querySelectorAll('table.print-collection')][1];
    const row = must(must(second).querySelector('tbody tr'));
    const cells = [...row.querySelectorAll('td')].map((cell) => textOf(cell));
    expect(cells).toEqual(['3', '9', ur ? 'ناصر' : 'Nasir', '2,000', '', '', '']);
    expect(row.querySelector('.print-collection-phone')).toBeNull();
  });

  it('shows the totals as the backend gave them: the total of each group, then the total OF THIS LIST and the buyers', () => {
    const { root } = show(collectionPrintData({ language }));
    const groupTotals = [...root.querySelectorAll('tr.print-amount-subtotal')].map((row) =>
      textOf(row as HTMLElement),
    );
    expect(groupTotals).toEqual([
      `${message(language, 'print-collection-group-total')}21,564`,
      `${message(language, 'print-collection-group-total')}2,000`,
    ]);
    const totals = must(root.querySelector<HTMLElement>('table.print-collection-totals'));
    const text = textOf(totals);
    expect(text).toContain(`${message(language, 'print-collection-buyers')}3`);
    expect(text).toContain(`${message(language, 'print-collection-total')}23,564`);
    // The words say it is the total of this list, not the shop's total.
    expect(message(language, 'print-collection-total')).toBe(
      ur ? 'اس فہرست کا میزان' : 'Total of this list',
    );
  });

  it('adds nothing up: a group total and a list total that are not the sum are shown as sent', () => {
    const data = collectionPrintData({ language });
    const odd: CollectionPrintData = {
      ...data,
      groups: data.groups.map((group) => ({ ...group, total: '1' })),
      total: { ...data.total, amount: '5' },
      buyer_count: '42',
    };
    const { root } = show(odd);
    const text = textOf(root);
    expect(text).toContain(`${message(language, 'print-collection-group-total')}1`);
    expect(text).toContain(`${message(language, 'print-collection-total')}5`);
    expect(text).toContain(`${message(language, 'print-collection-buyers')}42`);
    expect(text).not.toContain('21,564');
    expect(text).not.toContain('23,564');
  });

  it('numbers stay on the right edge and isolated: dues, serial and code are number cells, the phone sits in an isolated element', () => {
    const { root } = show(collectionPrintData({ language }));
    const row = must(root.querySelector<HTMLElement>('table.print-collection tbody tr'));
    const numberCells = [...row.querySelectorAll('td.print-num')].map((cell) =>
      textOf(cell as HTMLElement),
    );
    expect(numberCells).toEqual(['1', '1', '2,880', '10-10-2026', '0']);
    expect(within(row).getByText('0300-000001').closest('bdi')).not.toBeNull();
  });

  it('takes the paper, language and direction from the print, and the footer in the hidden window', () => {
    useLanguageStore.setState({ language: language === 'ur' ? 'en' : 'ur' });
    const window = show(collectionPrintData({ language, paper: 'a5' }), true);
    expect(window.root.lang).toBe(language);
    expect(window.root.dir).toBe(ur ? 'rtl' : 'ltr');
    expect(window.root.dataset['paper']).toBe('a5');
    const footer = document.documentElement.style.getPropertyValue(FOOTER_PROPERTY);
    expect(footer).toContain(message(language, 'print-printed-at'));
    expect(footer).toContain('counter(pages)');
    window.unmount();
    expect(document.documentElement.style.getPropertyValue(FOOTER_PROPERTY)).toBe('');
  });

  it('the roll has no columns: a buyer is two lines, each group has its total, and the list its total', () => {
    const { root } = show(collectionPrintData({ language, paper: 'mm80' }));
    expect(root.querySelector('table.print-collection')).toBeNull();
    const lines = root.querySelectorAll('li.print-compact-line:not(.print-amount-subtotal)');
    expect(lines).toHaveLength(3);
    const first = textOf(lines[0] as HTMLElement);
    for (const value of ['1', ur ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax', '2,880', '0300-000001']) {
      expect(first).toContain(value);
    }
    expect(textOf(root)).toContain('21,564');
    expect(textOf(root)).toContain(`${message(language, 'print-collection-total')}23,564`);
  });

  it('a list with nobody on it says so', () => {
    const data = collectionPrintData({ language });
    const { root } = show({ ...data, groups: [], buyer_count: '0' });
    expect(textOf(root)).toContain(message(language, 'print-preview-empty'));
    expect(root.querySelector('table')).toBeNull();
  });
});

const TARGET = { date: '10-10-2026', collectorStaffId: 7, cityId: null };

describe.each(LANGUAGES)('the print box of the collection list (%s)', (language: Language) => {
  beforeEach(() => {
    vi.resetAllMocks();
    useLanguageStore.setState({ language });
    vi.mocked(settingsApi.settingsGetPrinting).mockResolvedValue(printingSettings({ copies: 2 }));
    vi.mocked(reportsApi.reportsCollectionPrintData).mockImplementation((input) =>
      Promise.resolve(collectionPrintData({ language: input.language })),
    );
    vi.mocked(reportsApi.reportsPrintCollection).mockResolvedValue({
      printed: ['collection_list'],
      pdf_paths: null,
    });
  });

  function open() {
    renderIn(
      language,
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <CollectionPrintBox open onOpenChange={() => undefined} target={TARGET} />
      </QueryClientProvider>,
    );
    return userEvent.setup();
  }

  const button = (id: string) =>
    screen.getByRole('button', { name: (name) => visible(name) === message(language, id) });

  it('says which list it prints, in the print language of the app to start', async () => {
    open();
    const box = await screen.findByRole('dialog');
    expect(textOf(box)).toContain(message(language, 'vouchers-collection-print-title'));
    expect(textOf(box)).toContain(
      message(language, 'vouchers-collection-print-help', { date: '10-10-2026' }),
    );
    const choice = within(box).getByRole('button', {
      name: (name) => visible(name) === message(language, `common-language-${language}`),
    });
    expect(choice).toHaveAttribute('aria-pressed', 'true');
  });

  it('prints the list of the screen: its day and filters, the language, the copies as typed text', async () => {
    const user = open();
    const box = await screen.findByRole('dialog');
    await user.type(
      within(box).getByLabelText((name) => visible(name) === message(language, 'print-box-copies')),
      '3',
    );
    await user.click(button('print-box-print'));
    await waitFor(() => {
      expect(reportsApi.reportsPrintCollection).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.reportsPrintCollection).mock.calls[0]?.[0]).toEqual({
      date: '10-10-2026',
      collector_staff_id: 7,
      city_id: null,
      language,
      copies: '3',
      to: 'printer',
    });
  });

  it('the language of the print and PDF can be chosen apart from the app', async () => {
    const other: Language = language === 'ur' ? 'en' : 'ur';
    const user = open();
    const box = await screen.findByRole('dialog');
    await user.click(
      within(box).getByRole('button', {
        name: (name) => visible(name) === message(language, `common-language-${other}`),
      }),
    );
    await user.click(
      within(box).getByRole('button', {
        name: (name) => visible(name) === message(language, 'print-box-to-pdf'),
      }),
    );
    await user.click(button('print-box-save-pdf'));
    await waitFor(() => {
      expect(reportsApi.reportsPrintCollection).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(reportsApi.reportsPrintCollection).mock.calls[0]?.[0]).toMatchObject({
      language: other,
      to: 'pdf',
      copies: null,
    });
    // The app itself did not change language.
    expect(useLanguageStore.getState().language).toBe(language);
  });

  it('the preview shows the list laid out as it will print, in the language chosen', async () => {
    const user = open();
    await screen.findByRole('dialog');
    await user.click(button('print-box-preview'));
    const sheet = await waitFor(() => {
      const found = document.querySelector<HTMLElement>('.print-doc');
      if (found === null) throw new Error('no preview yet');
      return found;
    });
    expect(reportsApi.reportsCollectionPrintData).toHaveBeenCalledWith({
      date: '10-10-2026',
      collector_staff_id: 7,
      city_id: null,
      language,
    });
    expect(textOf(sheet)).toContain('23,564');
    expect(textOf(sheet)).toContain(message(language, 'print-collection-total'));
    await user.click(button('common-back'));
    expect(
      await screen.findByRole('button', {
        name: (name) => visible(name) === message(language, 'print-box-print'),
      }),
    ).toBeInTheDocument();
  });

  it('a refusal is told in words at the box, not as a code', async () => {
    vi.mocked(reportsApi.reportsPrintCollection).mockRejectedValue(
      fail({ code: 'ERR_PRINT_WINDOWS_ONLY' }),
    );
    const user = open();
    await screen.findByRole('dialog');
    await user.click(button('print-box-print'));
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-print-windows-only'));
    expect(visible(alert.textContent)).not.toContain('ERR_');
  });

  it('a list that cannot be read shows the plain failure in the preview, with a retry', async () => {
    vi.mocked(reportsApi.reportsCollectionPrintData).mockRejectedValue(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const user = open();
    await screen.findByRole('dialog');
    await user.click(button('print-box-preview'));
    expect(await screen.findByText(message(language, 'print-preview-failed'))).toBeInTheDocument();
  });
});
