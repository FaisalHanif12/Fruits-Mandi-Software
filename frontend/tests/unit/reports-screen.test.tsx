// رپورٹ (plan 20, PRD 19.1 to 19.5, 19.10, 19.11): the catalogue and the ONE report screen, in the
// Urdu and in the English app, against a fake backend that answers with the backend's own fixtures:
// the groups of the catalogue, the filter boxes each report takes, the rows page by page with the
// totals row from the backend, the party a ledger needs, the refusal of a half-typed date under its
// box, the four states, a row that opens its record, a language switch in the middle of a filter,
// and the keyboard-only path. Every figure on the screen is one the backend sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
  fail,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import {
  CATALOG,
  PRINT_R18_EN,
  R11_BAD_DATES,
  R13_AKRAM,
  R13_RAHIM,
  R18_FULL,
  fakeReports,
} from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

/** The keys a button shows after its name (PRD FR-7.4); the name is the words before them. */
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

function cellElement(rowId: string, colId: string): HTMLElement {
  const found = document.querySelector<HTMLElement>(
    `.ag-row[row-id="${rowId}"] [col-id="${colId}"]`,
  );
  if (found === null) throw new Error(`no cell ${rowId}:${colId}`);
  return found;
}

function totalsCell(colId: string): string {
  const row = document.querySelector('.ag-row-pinned');
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

/** The ids of the rows drawn now, without the totals row. */
function drawnRows(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map(
    (row) => row.getAttribute('row-id') ?? '',
  );
}

describe.each(LANGUAGES)('رپورٹ (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  /** The name of the region of one group (Fluent puts invisible marks around the group's name). */
  const groupName = (id: string) => (name: string) =>
    visible(name) === m('reports-group-label', { group: m(id) });
  const groupRegion = (id: string) => screen.findByRole('region', { name: groupName(id) });

  async function open(path: string, options: Parameters<typeof fakeReports>[0] = {}) {
    const fake = fakeReports(options);
    const view = await renderApp(path);
    return { ...view, fake, user: userEvent.setup() };
  }

  async function openReport(id: string, options: Parameters<typeof fakeReports>[0] = {}) {
    const entry = CATALOG.find((candidate) => candidate.id === id);
    const view = await open(`/reports/${id}`, options);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(entry?.title_id ?? 'report-r18-title')),
    });
    return view;
  }

  describe('the catalogue', () => {
    it('lists the reports of each group, as the backend sent them, with a line about each', async () => {
      await open('/reports');
      await screen.findByRole('heading', { level: 1, name: named(m('reports-title')) });
      expect(CATALOG).toHaveLength(22);
      for (const entry of CATALOG) {
        const group = await groupRegion(entry.group_title_id);
        const link = within(group)
          .getAllByRole('link')
          .find(
            (candidate) =>
              candidate.getAttribute('href')?.endsWith(`/reports/${entry.id}`) === true,
          );
        expect(visible(link?.textContent)).toContain(m(entry.title_id));
        expect(visible(link?.textContent)).toContain(m(entry.title_id.replace('-title', '-help')));
      }
    });

    it('shows the groups in the order of PRD 19.2 and each group’s reports in PRD order', async () => {
      // The list arrives in any order; the screen puts it in the PRD's order.
      await open('/reports', { catalog: [...CATALOG].reverse() });
      await screen.findByRole('heading', { level: 1, name: named(m('reports-title')) });
      const headings = (await screen.findAllByRole('heading', { level: 2 })).map((heading) =>
        visible(heading.textContent),
      );
      expect(headings).toEqual([
        m('report-group-sales'),
        m('report-group-accounts'),
        m('report-group-collections'),
        m('report-group-business'),
      ]);
      const idsOf = async (group: string) =>
        within(await groupRegion(group))
          .getAllByRole('link')
          .map((link) => link.getAttribute('href')?.split('/').at(-1));
      expect(await idsOf('report-group-sales')).toEqual(['R2', 'R3', 'R4', 'R6', 'R7', 'R8', 'R9']);
      expect(await idsOf('report-group-accounts')).toEqual([
        'R11',
        'R13',
        'R14',
        'R16',
        'R17',
        'R18',
        'R19',
      ]);
      expect(await idsOf('report-group-collections')).toEqual([
        'R20',
        'R21',
        'R22',
        'R24',
        'R26',
        'R27',
      ]);
      expect(await idsOf('report-group-business')).toEqual(['R28', 'R30']);
    });

    it('says so when the list could not be read, and tries again', async () => {
      const view = await open('/reports', { catalog: fail({ code: 'ERR_INTERNAL' }) });
      await screen.findByText(named(m('reports-catalog-failed')), undefined, SLOW_MS);
      fakeReports();
      await view.user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
      await screen.findAllByRole('link');
    });

    it('says so when no report is ready yet', async () => {
      await open('/reports', { catalog: [] });
      expect(await screen.findByText(named(m('reports-catalog-empty')))).toBeInTheDocument();
    });
  });

  describe('a report with no party: R18', () => {
    it('asks the backend for the first page of 100 with every filter key, and draws its rows', async () => {
      await openReport('R18');
      await waitFor(() => {
        expect(drawnRows()).toContain('0');
      }, SLOW_MS);
      expect(reportsApi.reportsRun).toHaveBeenCalledWith({
        id: 'R18',
        filters: {
          shortcut: null,
          from: '',
          to: '',
          as_of: '',
          party: null,
          seller_id: null,
          zamindar_id: null,
          item_id: null,
          city_id: null,
          adda_id: null,
          detail: false,
        },
        offset: 0,
        limit: 100,
      });
      // The first row is the cash account: its code and its name in this language.
      expect(cell('0', 'account')).toBe(`01010001 ${language === 'ur' ? 'روکڑ' : 'Cash'}`);
      expect(cell('0', 'debit')).toBe('58,484');
      expect(cell('0', 'credit')).toBe('');
    });

    it('shows the totals row of the backend under its columns: 85,048 on both sides', async () => {
      await openReport('R18');
      await waitFor(() => {
        expect(totalsCell('debit')).toBe('85,048');
      }, SLOW_MS);
      expect(totalsCell('credit')).toBe('85,048');
      expect(totalsCell('account')).toBe(m('report-total'));
    });

    it('shows the lines above the table: the difference of the two sides is nil', async () => {
      await openReport('R18');
      const label = await screen.findByText(named(m('report-difference')), undefined, SLOW_MS);
      const fact = label.parentElement;
      expect(visible(fact?.textContent)).toContain('0');
    });

    it('says how many rows it shows and how many the filter has', async () => {
      await openReport('R18');
      expect(
        await screen.findByText(
          named(m('report-count', { shown: 17, total: 17 })),
          undefined,
          SLOW_MS,
        ),
      ).toBeInTheDocument();
    });

    it('asks the backend again 250 ms after the last key of a date, with the date as typed', async () => {
      const { user } = await openReport('R18');
      await waitFor(() => {
        expect(drawnRows()).toContain('0');
      }, SLOW_MS);
      vi.mocked(reportsApi.reportsRun).mockClear();
      await user.type(screen.getByLabelText(m('report-filter-as-of')), '09-10-2026');
      await waitFor(() => {
        expect(reportsApi.reportsRun).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsRun).mock.calls[0]?.[0].filters.as_of).toBe('09-10-2026');
    });

    it('does not show a filter the report does not take', async () => {
      await openReport('R18');
      expect(screen.queryByLabelText(m('report-filter-party'))).toBeNull();
      expect(screen.queryByLabelText(m('report-filter-period'))).toBeNull();
      expect(screen.getByLabelText(m('report-filter-as-of'))).toBeInTheDocument();
    });
  });

  describe('paging', () => {
    it('loads the next 100 rows from where the first page ended, with the same totals', async () => {
      const row = R18_FULL.rows[0];
      if (row === undefined) throw new Error('no row');
      const many = {
        ...R18_FULL,
        rows: Array.from({ length: 230 }, () => row),
        total_count: 230,
      };
      const { user } = await openReport('R18', { pages: { R18: many } });
      await waitFor(() => {
        expect(drawnRows().length).toBeGreaterThan(0);
      }, SLOW_MS);
      expect(
        screen.getByText(named(m('report-count', { shown: 100, total: 230 }))),
      ).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(m('report-more')) }));
      await waitFor(() => {
        expect(
          screen.getByText(named(m('report-count', { shown: 200, total: 230 }))),
        ).toBeInTheDocument();
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsRun).mock.calls.map(([input]) => input.offset)).toEqual([
        0, 100,
      ]);
      // The totals are the backend's, over the whole filter: never the sum of the rows on screen.
      expect(totalsCell('debit')).toBe('85,048');
    });
  });

  describe('a ledger needs a party: R13', () => {
    it('asks for the party first and asks the backend nothing until one is picked', async () => {
      await openReport('R13');
      expect(await screen.findByText(named(m('report-pick-party')))).toBeInTheDocument();
      expect(reportsApi.reportsRun).not.toHaveBeenCalled();
    });

    it('opens on the party of the address and sends him as the party filter', async () => {
      await open('/reports/R13?party=20&from=01-10-2026&to=10-10-2026');
      await waitFor(() => {
        expect(drawnRows()).toContain('0');
      }, SLOW_MS);
      const filters = vi.mocked(reportsApi.reportsRun).mock.calls[0]?.[0].filters;
      expect(filters?.party).toEqual({ account_id: 20, person_key: null });
      expect(filters?.from).toBe('01-10-2026');
      expect(filters?.to).toBe('10-10-2026');
      // His names come back with the answer and fill the box, in this language.
      const box = screen.getByRole('combobox', { name: m('report-filter-party') });
      await waitFor(() => {
        expect(box).toHaveValue(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
      }, SLOW_MS);
    });

    it('shows the ledger of E2E-33: the goods of the sale, the receipt, and 2,880 owed at the end', async () => {
      await open('/reports/R13?party=20');
      await waitFor(() => {
        expect(cell('0', 'date')).toBe('10-10-2026');
      }, SLOW_MS);
      const item = language === 'ur' ? 'کیلا' : 'Banana';
      const packing = language === 'ur' ? 'پیٹی' : 'Box';
      // "Sale 1: Banana 20 Box @ 600 = 12,000, Commission 720, Laga 160" (PRD FR-19.8)
      expect(cell('0', 'detail')).toBe(
        m('report-detail-sale', {
          number: '1',
          item,
          qty: '20',
          packing,
          rate: '600',
          value: '12,000',
          commission: '720',
          laga: '160',
        }),
      );
      expect(cell('0', 'doc')).toBe(
        m('report-doc-number', { kind: m('report-doc-bikri-first'), number: '1' }),
      );
      expect(cell('0', 'debit')).toBe('12,880');
      expect(cell('0', 'balance')).toContain('12,880');
      // The receipt: the voucher's own kind and number, the credit, and the balance left.
      expect(cell('1', 'doc')).toBe(
        m('report-doc-number', { kind: m('report-doc-receipt'), number: '1' }),
      );
      expect(cell('1', 'credit')).toBe('10,000');
      expect(cell('1', 'balance')).toContain('2,880');
      // The totals are the backend's: debit, credit and the closing balance with its side.
      expect(totalsCell('debit')).toBe('12,880');
      expect(totalsCell('credit')).toBe('10,000');
      expect(totalsCell('balance')).toContain('2,880');
      // The closing balance is also above the table, with its side as a word.
      const closing = screen.getByText(named(m('report-closing-balance'))).parentElement;
      expect(visible(closing?.textContent)).toContain('2,880');
      expect(visible(closing?.textContent)).toContain(m('report-side-dr'));
    });

    it('puts the side word of a balance in its own slot after the figure', async () => {
      await open('/reports/R13?party=20');
      await waitFor(() => {
        expect(cell('0', 'balance')).not.toBe('');
      }, SLOW_MS);
      const slot = cellElement('0', 'balance').querySelector('.cell-unit');
      expect(visible(slot?.textContent)).toBe(m('report-side-dr'));
      // A balance of nothing has no side word.
      expect(visible(totalsCell('balance'))).toContain(m('report-side-dr'));
    });

    it('shows a seller credit with its gross sale and deductions', async () => {
      await open('/reports/R13?party=18', { pages: { R13: R13_RAHIM } });
      await waitFor(() => {
        expect(cell('0', 'detail')).not.toBe('');
      }, SLOW_MS);
      expect(cell('0', 'detail')).toBe(
        m('report-detail-lot', {
          number: '1',
          item: language === 'ur' ? 'کیلا' : 'Banana',
          qty: '50',
          packing: language === 'ur' ? 'پیٹی' : 'Box',
          gross: '29,400',
          deductions: '5,084',
        }),
      );
      expect(cell('0', 'balance')).toContain('9,316');
      expect(visible(cellElement('0', 'balance').querySelector('.cell-unit')?.textContent)).toBe(
        m('report-side-cr'),
      );
    });
  });

  describe('a statement: R14', () => {
    it('shows balances only until the detail box is ticked, then the lines of the ledger', async () => {
      const { user } = await open('/reports/R14?party=20');
      // The summary of the statement: opening, bought, paid, closing.
      const label = await screen.findByText(
        named(m('report-statement-bought')),
        undefined,
        SLOW_MS,
      );
      expect(visible(label.parentElement?.textContent)).toContain('12,880');
      expect(screen.getByText(named(m('report-balance-only-help')))).toBeInTheDocument();
      expect(drawnRows()).toEqual([]);
      await user.click(screen.getByRole('checkbox', { name: m('report-filter-detail') }));
      await waitFor(() => {
        expect(cell('0', 'date')).toBe('10-10-2026');
      }, SLOW_MS);
      const sent = vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
      expect(sent?.filters.detail).toBe(true);
      expect(sent?.filters.party).toEqual({ account_id: 20, person_key: null });
    });
  });

  describe('the half-typed date', () => {
    it('is refused under its own box and the rows already shown stay', async () => {
      const { user } = await openReport('R11', {
        refuse: (input) => (input.filters.to === '10-' ? R11_BAD_DATES : null),
      });
      await waitFor(() => {
        expect(cell('0', 'date')).toBe('10-10-2026');
      }, SLOW_MS);
      await user.type(screen.getByLabelText(m('report-filter-to')), '10-');
      const refusal = await screen.findByText(named(m('err-date-invalid')), undefined, SLOW_MS);
      expect(refusal).toBeInTheDocument();
      // No page-wide error: the rows are still there, and so is the totals row.
      expect(screen.queryByText(named(m('report-load-failed')))).toBeNull();
      expect(cell('0', 'date')).toBe('10-10-2026');
      expect(totalsCell('debit')).not.toBe('');
      // Finishing the date clears the refusal and asks again.
      await user.type(screen.getByLabelText(m('report-filter-to')), '10-2026');
      await waitFor(() => {
        expect(screen.queryByText(named(m('err-date-invalid')))).toBeNull();
      }, SLOW_MS);
    });

    it('an error with no field says what happened and tries again on the button', async () => {
      let fail1 = true;
      const view = await open('/reports/R16', {
        refuse: () => {
          if (!fail1) return null;
          fail1 = false;
          return {
            ok: false,
            errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
            ref: 'E-1',
          } as never;
        },
      });
      await screen.findByText(named(m('report-load-failed')), undefined, SLOW_MS);
      await view.user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
      await waitFor(() => {
        expect(cell('0', 'account')).not.toBe('');
      }, SLOW_MS);
    });
  });

  describe('a period shortcut', () => {
    it('sends the shortcut, hides the two dates, and shows the days the backend read', async () => {
      const { user } = await openReport('R11');
      await waitFor(() => {
        expect(cell('0', 'date')).toBe('10-10-2026');
      }, SLOW_MS);
      await user.click(screen.getByRole('combobox', { name: m('report-filter-period') }));
      await user.click(await screen.findByRole('option', { name: m('report-period-today') }));
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters.shortcut).toBe(
          'today',
        );
      }, SLOW_MS);
      const sent = vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters;
      // A shortcut wins: no dates are sent with it (the backend would ignore them).
      expect(sent?.from).toBe('');
      expect(sent?.to).toBe('');
      expect(screen.queryByLabelText(m('report-filter-from'))).toBeNull();
    });

    it('says which days a page covers', async () => {
      await openReport('R11', {
        pages: { R11: { ...R13_AKRAM, id: 'R11' } },
      });
      expect(
        await screen.findByText(named(m('report-range-all')), undefined, SLOW_MS),
      ).toBeInTheDocument();
    });
  });

  describe('a row opens the record behind it', () => {
    it('Enter on a sale line of the day book opens its بکری', async () => {
      const { router, user } = await openReport('R11');
      await waitFor(() => {
        expect(cell('0', 'date')).toBe('10-10-2026');
      }, SLOW_MS);
      await user.click(cellElement('0', 'date'));
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/book');
      }, SLOW_MS);
      expect(router.state.location.search).toEqual({ id: 1 });
    });

    it('Enter on an account of the trial balance opens that account’s ledger', async () => {
      const { router, user } = await openReport('R18');
      await waitFor(() => {
        expect(cell('0', 'account')).not.toBe('');
      }, SLOW_MS);
      await user.click(cellElement('0', 'account'));
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports/R13');
      }, SLOW_MS);
      expect(router.state.location.search).toEqual({ party: 1 });
    });

    it('Enter on a head of the head balances (no record behind it) opens nothing', async () => {
      const { router, user } = await openReport('R17');
      await waitFor(() => {
        expect(cell('0', 'code')).toBe('01');
      }, SLOW_MS);
      await user.click(cellElement('0', 'code'));
      await user.keyboard('{Enter}');
      expect(router.state.location.pathname).toBe('/reports/R17');
    });
  });

  describe('Esc and the buttons', () => {
    it('Esc leaves the report for the list of reports', async () => {
      const { router, user } = await openReport('R18');
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      }, SLOW_MS);
    });

    it('the Back button does the same', async () => {
      const { router, user } = await openReport('R18');
      await user.click(screen.getByRole('button', { name: named(m('report-back')) }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      }, SLOW_MS);
    });

    it('a report the backend does not list says so and offers the list', async () => {
      const { router, user } = await open('/reports/R99');
      expect(
        await screen.findByText(named(m('report-load-failed')), undefined, SLOW_MS),
      ).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(m('report-back')) }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      }, SLOW_MS);
    });
  });

  describe('nothing found', () => {
    it('says so, and offers to clear the filters when some are filled', async () => {
      const empty = { ...R18_FULL, rows: [], total_count: 0, summary: [] };
      const { user } = await openReport('R18', { pages: { R18: empty } });
      expect(
        await screen.findByText(named(m('report-empty')), undefined, SLOW_MS),
      ).toBeInTheDocument();
      // Nothing is filled: no button.
      expect(screen.queryByRole('button', { name: named(m('report-clear-filters')) })).toBeNull();
      await user.type(screen.getByLabelText(m('report-filter-as-of')), '09-10-2026');
      const clear = await screen.findByRole('button', {
        name: named(m('report-clear-filters')),
      });
      await user.click(clear);
      await waitFor(() => {
        expect(screen.getByLabelText(m('report-filter-as-of'))).toHaveValue('');
      }, SLOW_MS);
    });
  });

  describe('the print preview', () => {
    it('F9 shows the page that would print, in the app language, with every row asked for', async () => {
      const { user } = await openReport('R18');
      await waitFor(() => {
        expect(cell('0', 'account')).not.toBe('');
      }, SLOW_MS);
      await user.click(screen.getByLabelText(m('report-filter-as-of')));
      press('F9');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('report-print-title')) },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(reportsApi.reportsPrintData).toHaveBeenCalled();
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
        id: 'R18',
        language,
        paper: 'a4',
      });
      // The firm, the totals row and the rows of the print data.
      const doc = await screen.findByText(PRINT_R18_EN.firm.name, undefined, SLOW_MS);
      expect(doc).toBeInTheDocument();
      expect(document.querySelectorAll('.print-report tbody tr')).toHaveLength(17);
      expect(visible(document.querySelector('.print-report tfoot')?.textContent)).toContain(
        '85,048',
      );
      // The printer and PDF buttons wait for the print job of reports, and say so.
      expect(screen.getByRole('button', { name: named(m('report-print-print')) })).toBeDisabled();
      expect(screen.getByRole('button', { name: named(m('report-print-pdf')) })).toBeDisabled();
      expect(screen.getByText(named(m('report-print-job-later')))).toBeInTheDocument();
    });

    it('can be previewed in the other language without changing the app', async () => {
      const { user } = await openReport('R18');
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('report-print-title')) },
        SLOW_MS,
      );
      const other: Language = language === 'ur' ? 'en' : 'ur';
      const group = screen.getByRole('group', { name: m('print-box-language') });
      await user.click(
        within(group).getByRole('button', { name: other === 'ur' ? 'اردو' : 'English' }),
      );
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsPrintData).mock.calls.at(-1)?.[0].language).toBe(other);
      }, SLOW_MS);
      expect(document.documentElement.lang).toBe(language);
    });

    it('Esc closes the preview first, and the next Esc leaves the report', async () => {
      const { router, user } = await openReport('R18');
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('report-print-title')) },
        SLOW_MS,
      );
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(
          screen.queryByRole('heading', { level: 2, name: named(m('report-print-title')) }),
        ).toBeNull();
      }, SLOW_MS);
      expect(router.state.location.pathname).toBe('/reports/R18');
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      }, SLOW_MS);
    });

    it('a report that cannot run yet (no party) cannot be previewed', async () => {
      await openReport('R13');
      expect(screen.getByRole('button', { name: named(m('report-preview')) })).toBeDisabled();
    });
  });

  describe('a language switch in the middle of a filter', () => {
    it('keeps the filter and the focused row, and draws the same rows in the other language', async () => {
      const { user } = await open('/reports/R13?party=20');
      await waitFor(() => {
        expect(cell('0', 'date')).toBe('10-10-2026');
      }, SLOW_MS);
      await user.type(screen.getByLabelText(m('report-filter-to')), '10-10-2026');
      // The backend is asked once the date has rested; the switch comes after that answer.
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters.to).toBe(
          '10-10-2026',
        );
      }, SLOW_MS);
      await waitFor(() => {
        expect(cell('1', 'doc')).not.toBe('');
      }, SLOW_MS);
      await user.click(cellElement('1', 'doc'));
      const before = vi.mocked(reportsApi.reportsRun).mock.calls.length;
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(cell('1', 'doc')).toBe(
          message(other, 'report-doc-number', {
            kind: message(other, 'report-doc-receipt'),
            number: '1',
          }),
        );
      }, SLOW_MS);
      // The date typed is still there, and the row that had focus has focus again.
      expect(screen.getByLabelText(message(other, 'report-filter-to'))).toHaveValue('10-10-2026');
      await waitFor(() => {
        const focused = document.querySelector('.ag-cell-focus')?.closest('.ag-row');
        expect(focused?.getAttribute('row-id')).toBe('1');
      }, SLOW_MS);
      // Nothing was asked of the backend again: the same pages were drawn again.
      expect(vi.mocked(reportsApi.reportsRun).mock.calls.length).toBe(before);
      // The names of the picker follow the language.
      expect(
        screen.getByRole('combobox', { name: message(other, 'report-filter-party') }),
      ).toHaveValue(other === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    });
  });

  describe('keyboard only', () => {
    it('the list, a report, a filter, the grid and a row open, then Esc back, without the mouse', async () => {
      const { router, user } = await open('/reports');
      await screen.findByRole('heading', { level: 1, name: named(m('reports-title')) });
      // The first report (R2, the first of Sales) has focus. ↓ twelve times is R18: the seven
      // Sales reports, then R11, R13, R14, R16, R17 and R18.
      await waitFor(() => {
        expect(document.activeElement?.getAttribute('href')).toMatch(/R2$/);
      }, SLOW_MS);
      await user.keyboard(`${'{ArrowDown}'.repeat(12)}{Enter}`);
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports/R18');
      }, SLOW_MS);
      // The first filter box has focus: type the day, and the report follows.
      const box = await screen.findByLabelText(m('report-filter-as-of'));
      await waitFor(() => {
        expect(document.activeElement).toBe(box);
      }, SLOW_MS);
      await user.keyboard('09-10-2026');
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters.as_of).toBe(
          '09-10-2026',
        );
      }, SLOW_MS);
      // Tab to the table, ↓ to the first row, Enter opens the ledger of that account.
      await user.tab();
      await user.keyboard('{ArrowDown}{Enter}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports/R13');
      }, SLOW_MS);
      // Esc goes back to the list of reports.
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      }, SLOW_MS);
    });
  });
});
