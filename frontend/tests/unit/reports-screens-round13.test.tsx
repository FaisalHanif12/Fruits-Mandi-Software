// The fruit register (R37) on the ONE report screen (round 13, PRD 16.7, 19.2 R37, FR-16.12), in the
// Urdu and in the English app, against a fake backend that answers with the backend's own fixtures. It
// is the fruit partners took home and the fruit served to the shop's guests: the date, the document,
// who it was for, the item, the quantity, the rate, the value and where it came from, with the
// backend's totals. Its "Partner or guests" box takes a partner OR the guests, never both. The four
// states, the print, the layout steps, a language switch and the way into the fruit screen are each
// proved. The screen adds up nothing: every figure is one the backend sent.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportCell } from '../../src/bindings/ReportCell';
import type { ReportPage } from '../../src/bindings/ReportPage';
import type { ReportRow } from '../../src/bindings/ReportRow';
import * as gardensApi from '../../src/features/gardens/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import { paisaForTest } from '../helpers/paisa';
import { fakePartners, partnerRow } from '../helpers/partners';
import { printFirmName } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';
import {
  CATALOG,
  PRINT_R37_EN,
  R37_FRUIT,
  R31_BAY,
  R37_GUESTS_REQUEST,
  R37_FRUIT_REQUEST,
  fakeReports,
} from '../helpers/reports';
import { PAGE_ROWS } from '../helpers/pager';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
// The window of a test has no layout, so the step is the middle one.
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

const PARTNERS = [
  partnerRow(1, 'الف', 'Partner A', 4000),
  partnerRow(2, 'ب', 'Partner B', 3000),
  partnerRow(3, 'ج', 'Partner C', 3000),
];

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakePartners(PARTNERS);
  fakeGardens();
});

afterEach(() => {
  leaveAppWindow();
});

const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);

/** Matches a name that is exactly `text`, or `text` followed by the key chip of its button. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

function cell(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function totalsCell(colId: string): string {
  const row = document.querySelector('.ag-row-pinned');
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function drawnRows(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map(
    (row) => row.getAttribute('row-id') ?? '',
  );
}

function drawnColumns(): string[] {
  return Array.from(document.querySelectorAll('.ag-header-cell[col-id]')).map(
    (header) => header.getAttribute('col-id') ?? '',
  );
}

function lastRun() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
}

function runCount() {
  return vi.mocked(reportsApi.reportsRun).mock.calls.length;
}

describe.each(LANGUAGES)('the fruit register R37 (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const other: Language = language === 'ur' ? 'en' : 'ur';

  async function openRegister(options: Parameters<typeof fakeReports>[0] = {}) {
    const fake = fakeReports(options);
    const view = await renderApp('/reports/R37');
    await screen.findByRole('heading', { level: 1, name: named(m('report-r37-title')) });
    return { ...view, fake, user: userEvent.setup() };
  }

  async function choose(user: ReturnType<typeof userEvent.setup>, box: string, option: string) {
    await user.click(screen.getByRole('combobox', { name: named(m(box)) }));
    await user.click(await screen.findByRole('option', { name: option }, SLOW_MS));
  }

  it('is in the business group with the boxes of the PRD: the days, the partner or guests, the item and the garden', () => {
    expect(CATALOG.find((entry) => entry.id === 'R37')).toMatchObject({
      group: 'business',
      title_id: 'report-r37-title',
      filters: ['date_range', 'partner', 'guests', 'item', 'garden'],
      party_required: false,
    });
  });

  it('asks for every line with no partner and no guests choice, and draws them with the backend’s figures', async () => {
    await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toEqual(['0', '1']);
    }, SLOW_MS);
    expect(lastRun()).toMatchObject({ id: 'R37', offset: 0, limit: PAGE_ROWS });
    // The guests choice is sent only when it is set (the backend reads a missing key as every line).
    expect(lastRun()?.filters.guests_only).toBeUndefined();
    expect(lastRun()?.filters.partner_id).toBeNull();
    expect(cell('0', 'for')).toBe(names('الف', 'Partner A'));
    expect(cell('0', 'item')).toBe(names('کیلا', 'Banana'));
    expect(cell('0', 'qty')).toBe('2');
    expect(cell('0', 'rate')).toBe('580');
    expect(cell('0', 'value')).toBe('1,160');
    expect(cell('0', 'source')).toBe(m('fruit-source-own'));
    expect(cell('0', 'doc')).toBe(
      m('report-doc-number', { kind: m('report-doc-fruit-taken'), number: '1' }),
    );
    // The guests' line says "guests", and came from another shop's agent.
    expect(cell('1', 'for')).toBe(m('fruit-for-guests'));
    expect(cell('1', 'qty')).toBe('3');
    expect(cell('1', 'value')).toBe('3,000');
    expect(cell('1', 'source')).toBe(m('fruit-source-other'));
    // The totals are the backend's: the quantity and the value of the whole filter.
    expect(totalsCell('qty')).toBe('5');
    expect(totalsCell('value')).toBe('4,160');
  });

  it('draws the columns that fit: the seller, the packing and the weight wait for the wide step', async () => {
    await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    expect(drawnColumns()).toEqual([
      'date',
      'doc',
      'for',
      'item',
      'qty',
      'rate',
      'value',
      'source',
    ]);
  });

  it.each([
    ['compact', ['date', 'for', 'item', 'qty', 'value']],
    [
      'wide',
      [
        'date',
        'doc',
        'for',
        'seller',
        'item',
        'packing',
        'weight',
        'qty',
        'rate',
        'value',
        'source',
      ],
    ],
  ] as const)('draws the columns of the %s step', async (step, columns) => {
    vi.mocked(useLayoutStep).mockReturnValue(step);
    await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    expect(drawnColumns()).toEqual([...columns]);
  });

  it('labels its box "Partner or guests" and offers all, the guests and each partner', async () => {
    const { user } = await openRegister();
    await user.click(screen.getByRole('combobox', { name: named(m('fruit-filter-for')) }));
    const options = (await screen.findAllByRole('option', undefined, SLOW_MS)).map((option) =>
      visible(option.textContent),
    );
    expect(options).toEqual([
      m('report-partner-all'),
      m('fruit-for-guests'),
      names('الف', 'Partner A'),
      names('ب', 'Partner B'),
      names('ج', 'Partner C'),
    ]);
  });

  it('the guests choice asks for their lines only, with no partner, and a partner after it clears it', async () => {
    const { user } = await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    await choose(user, 'fruit-filter-for', m('fruit-for-guests'));
    await waitFor(() => {
      expect(lastRun()?.filters.guests_only).toBe(true);
    }, SLOW_MS);
    expect(lastRun()?.filters.partner_id).toBeNull();
    await waitFor(() => {
      expect(drawnRows()).toEqual(['0']);
    }, SLOW_MS);
    expect(cell('0', 'for')).toBe(m('fruit-for-guests'));
    expect(R37_GUESTS_REQUEST.filters.guests_only).toBe(true);
    // A partner after the guests: his lines, and the guests choice is no longer sent.
    await choose(user, 'fruit-filter-for', names('الف', 'Partner A'));
    await waitFor(() => {
      expect(lastRun()?.filters.partner_id).toBe(1);
    }, SLOW_MS);
    expect(lastRun()?.filters.guests_only).toBeUndefined();
    expect(R37_FRUIT_REQUEST.filters.guests_only).toBeUndefined();
  });

  it('keeps the guests choice across a language switch and asks again for nothing', async () => {
    const { user } = await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    await choose(user, 'fruit-filter-for', m('fruit-for-guests'));
    await waitFor(() => {
      expect(drawnRows()).toEqual(['0']);
    }, SLOW_MS);
    const before = runCount();
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    await waitFor(() => {
      expect(cell('0', 'for')).toBe(message(other, 'fruit-for-guests'));
    }, SLOW_MS);
    expect(runCount()).toBe(before);
    expect(lastRun()?.filters.guests_only).toBe(true);
  });

  it('writes the partner’s name in the other language after a switch, with the same rows', async () => {
    await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    const before = runCount();
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    await waitFor(() => {
      expect(cell('0', 'for')).toBe(other === 'ur' ? 'الف' : 'Partner A');
    }, SLOW_MS);
    expect(runCount()).toBe(before);
  });

  it('Enter on a line opens that entry on the fruit screen (?entry=)', async () => {
    const { router, user } = await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    const target = document.querySelector('.ag-row[row-id="0"] [col-id="for"]');
    if (target === null) throw new Error('no cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/fruit');
    }, SLOW_MS);
    // The line's own id (the first line of E2E-42 is entry 1), read by the fruit screen.
    expect(router.state.location.search).toEqual({ entry: 1 });
  });

  it('starts from the address the fruit screen sends: the days, the guests and the item', async () => {
    fakeReports();
    const item = { id: 2, name_ur: 'سیب', name_en: 'Apple', code: '2' };
    await renderApp(
      `/reports/R37?from=10-10-2026&to=10-10-2026&guests=true&item=${encodeURIComponent(JSON.stringify(item))}`,
    );
    await screen.findByRole('heading', { level: 1, name: named(m('report-r37-title')) });
    await waitFor(() => {
      expect(lastRun()?.filters).toMatchObject({
        from: '10-10-2026',
        to: '10-10-2026',
        guests_only: true,
        partner_id: null,
        item_id: 2,
      });
    }, SLOW_MS);
  });

  it('starts with a partner from the address and no guests, even when both were sent', async () => {
    fakeReports();
    await renderApp('/reports/R37?partner=1&guests=true');
    await screen.findByRole('heading', { level: 1, name: named(m('report-r37-title')) });
    await waitFor(() => {
      expect(lastRun()?.filters.partner_id).toBe(1);
    }, SLOW_MS);
    expect(lastRun()?.filters.guests_only).toBeUndefined();
  });

  it('shows the print with every line and the totals row', async () => {
    const { user } = await openRegister({ print: PRINT_R37_EN });
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
    await screen.findByText(printFirmName(language), undefined, SLOW_MS);
    expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
      id: 'R37',
      language,
      paper: 'a4',
    });
    expect(document.querySelectorAll('.print-report tbody tr')).toHaveLength(2);
    expect(visible(document.querySelector('.print-report tfoot')?.textContent)).toContain('4,160');
  });

  it('says there is nothing, and what happened when the backend fails, with a retry', async () => {
    let failing = true;
    const empty: ReportPage = { ...R37_FRUIT, rows: [], total_count: 0 };
    const { user } = await openRegister({
      pages: { R37: empty },
      refuse: () => {
        if (!failing) return null;
        failing = false;
        return {
          ok: false,
          errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
          ref: 'E-1',
        } as never;
      },
    });
    await screen.findByText(named(m('report-load-failed')), undefined, SLOW_MS);
    await user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
    await screen.findByText(named(m('report-empty')), undefined, SLOW_MS);
  });

  it('asks for the garden picked, and the gardens come from the gardens module', async () => {
    const { user } = await openRegister();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    await choose(user, 'report-filter-garden', `0080 ${names('شاہ پور باغ', 'Shahpur Garden')}`);
    await waitFor(() => {
      expect(lastRun()?.filters.garden_id).toBe(1);
    }, SLOW_MS);
    expect(vi.mocked(gardensApi.gardensList)).toHaveBeenCalled();
  });
});

describe.each(LANGUAGES)(
  'R31, the partner statement, with fruit lines (%s)',
  (language: Language) => {
    beforeEach(() => {
      resetClientState(language);
      signedIn(sessionUser({ language }));
    });

    const m = (id: string, vars: Record<string, string | number> = {}) =>
      message(language, id, vars);
    const names = (ur: string, en: string) => (language === 'ur' ? ur : en);

    /** A message cell with its arguments written as text, like the backend sends them. */
    const sentence = (id: string, args: Record<string, string>): ReportCell => ({
      type: 'message',
      id,
      args: Object.entries(args).map(([name, text]) => ({
        name,
        value: { type: 'text', text },
      })),
    });

    /** The statement of ب with two fruit lines made from the backend's first line: his equal part of the guests' fruit, and fruit he took home. */
    function statementWithFruit(): ReportPage {
      const base = R31_BAY.rows[0];
      if (base === undefined) throw new Error('The R31 fixture has no row.');
      const line = (detail: ReportCell, paisa: number, running: number): ReportRow => ({
        ...base,
        cells: base.cells.map((found, index): ReportCell => {
          if (index === 1) return { type: 'label', id: 'partners-statement-heading-fruit' };
          if (index === 2) return detail;
          if (index === 3) return { type: 'money', paisa: paisaForTest(paisa) };
          if (index === 4) return { type: 'money', paisa: paisaForTest(running) };
          return found;
        }),
      });
      return {
        ...R31_BAY,
        rows: [
          line(
            sentence('partners-statement-detail-fruit-taken', {
              for: 'guests',
              subject: names('سیب', 'Apple'),
              qty: '3',
              packing: names('پیٹی', 'Box'),
              rate: '1,000',
              value: '3,000',
            }),
            -100_000,
            -100_000,
          ),
          line(
            sentence('partners-statement-detail-fruit-taken', {
              for: 'home',
              subject: names('کیلا', 'Banana'),
              qty: '2',
              packing: names('پیٹی', 'Box'),
              rate: '580',
              value: '1,160',
            }),
            -116_000,
            -216_000,
          ),
        ],
        total_count: 2,
      };
    }

    it('draws the heading and the sentence of each fruit line, with his part and the running total, and no hole in a sentence', async () => {
      fakeReports({ pages: { R31: statementWithFruit() } });
      await renderApp('/reports/R31');
      await screen.findByRole('heading', { level: 1, name: named(m('report-r31-title')) });
      const user = userEvent.setup();
      await user.click(screen.getByRole('combobox', { name: m('report-filter-partner') }));
      await user.click(
        await screen.findByRole('option', { name: names('ب', 'Partner B') }, SLOW_MS),
      );
      await waitFor(() => {
        expect(drawnRows()).toEqual(['0', '1']);
      }, SLOW_MS);
      expect(cell('0', 'heading')).toBe(m('partners-statement-heading-fruit'));
      expect(cell('0', 'detail')).toBe(
        m('partners-statement-detail-fruit-taken', {
          for: 'guests',
          subject: names('سیب', 'Apple'),
          qty: '3',
          packing: names('پیٹی', 'Box'),
          rate: '1,000',
          value: '3,000',
        }),
      );
      expect(cell('1', 'detail')).toBe(
        m('partners-statement-detail-fruit-taken', {
          for: 'home',
          subject: names('کیلا', 'Banana'),
          qty: '2',
          packing: names('پیٹی', 'Box'),
          rate: '580',
          value: '1,160',
        }),
      );
      // His part is a minus, and the running total follows it: the backend worked both out.
      expect(cell('0', 'amount')).toBe('-1,000');
      expect(cell('1', 'amount')).toBe('-1,160');
      expect(cell('1', 'running')).toBe('-2,160');
      const text = visible(document.body.textContent);
      expect(text).not.toMatch(/\$\{|partners-statement-|\{\s*\$/);
    });
  },
);
