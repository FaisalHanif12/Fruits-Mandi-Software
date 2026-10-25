// The credit buyers' details (R5) and the sale book (R1) on the ONE report screen (round 12, PRD 19.2,
// FR-12.12, FR-19.12), in the Urdu and in the English app, against a fake backend that answers with the
// backend's own fixtures. R5 is one row for each credit buyer, each sale line written the way the
// client's own print writes it, "(item qty . rate)", with the backend's totals. R1 is the lots of one
// بکری: it cannot run until a بکری is picked (its number is typed and found), and its print is the
// seller's bill itself, not a page of the report. The four states, the print, a language switch and the
// keyboard are each proved. The screen adds up nothing: every figure is one the backend sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ReportCell } from '../../src/bindings/ReportCell';
import type { ReportPage } from '../../src/bindings/ReportPage';
import * as bikriApi from '../../src/features/bikri/api';
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
import { listRow } from '../helpers/bikri-saved';
import { paisaForTest } from '../helpers/paisa';
import { fakePartners } from '../helpers/partners';
import { printFirmName } from '../helpers/print-frame';
import { message, visible } from '../helpers/render';
import {
  BUYER_BILLS_EN,
  BUYER_BILLS_UR,
  CATALOG,
  PRINT_R5_EN,
  R1_BIKRI_1,
  R5_CREDIT_BUYERS,
  fakeReports,
} from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/bikri/api');
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

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakePartners([]);
  vi.mocked(bikriApi.bikriFind).mockResolvedValue([listRow({ id: 1, number: 1 })]);
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

describe.each(LANGUAGES)('R5, the credit buyers’ details (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const other: Language = language === 'ur' ? 'en' : 'ur';

  async function openCredit(options: Parameters<typeof fakeReports>[0] = {}) {
    const fake = fakeReports(options);
    const view = await renderApp('/reports/R5');
    await screen.findByRole('heading', { level: 1, name: named(m('report-r5-title')) });
    return { ...view, fake, user: userEvent.setup() };
  }

  it('is in the catalogue in the sales group with the boxes of R4', () => {
    expect(CATALOG.find((entry) => entry.id === 'R5')).toMatchObject({
      group: 'sales',
      title_id: 'report-r5-title',
      filters: ['date_range', 'party', 'seller', 'zamindar', 'item', 'city', 'adda'],
      party_required: false,
    });
  });

  it('asks with no بکری and no garden, and draws a row for each credit buyer with the backend’s figures', async () => {
    await openCredit();
    await waitFor(() => {
      expect(drawnRows()).toEqual(['0', '1']);
    }, SLOW_MS);
    expect(lastRun()).toMatchObject({
      id: 'R5',
      filters: { bikri_id: null, garden_id: null },
      offset: 0,
      limit: 100,
    });
    expect(drawnColumns()).toEqual(['serial', 'buyer', 'detail', 'value', 'buyer_total']);
    expect(cell('0', 'serial')).toBe('1');
    expect(cell('0', 'buyer')).toBe(`03040002 ${names('ارشد ملتانی', 'Arshad Multani')}`);
    expect(cell('0', 'value')).toBe('17,400');
    expect(cell('0', 'buyer_total')).toBe('18,684');
    expect(cell('1', 'serial')).toBe('2');
    expect(cell('1', 'buyer_total')).toBe('12,880');
    // The totals are the backend's, the same as the credit sales report R4.
    expect(totalsCell('value')).toBe('29,400');
    expect(totalsCell('buyer_total')).toBe('31,564');
  });

  it('writes each sale line the way the client’s own print does: (item qty . rate)', async () => {
    await openCredit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    expect(cell('0', 'detail')).toBe(`(${names('کیلا', 'Banana')} 30 . 580)`);
    expect(cell('1', 'detail')).toBe(`(${names('کیلا', 'Banana')} 20 . 600)`);
  });

  it('writes the rate plain, 2600, while the value columns keep the app’s grouping', async () => {
    // The client's own print reads "(سیب 8 . 2600)": no separator inside the line of words.
    const base = R5_CREDIT_BUYERS.rows[0];
    if (base === undefined) throw new Error('The R5 fixture has no row.');
    const page: ReportPage = {
      ...R5_CREDIT_BUYERS,
      rows: [
        {
          ...base,
          cells: base.cells.map((found): ReportCell => {
            if (found.type === 'detail') {
              return {
                ...found,
                parts: found.parts.map((part) =>
                  part.kind === 'sale' ? { ...part, rate_paisa: paisaForTest(260_000) } : part,
                ),
              };
            }
            if (found.type === 'money') return { ...found, paisa: paisaForTest(1_560_000) };
            return found;
          }),
        },
      ],
    };
    await openCredit({ pages: { R5: page } });
    await waitFor(() => {
      expect(drawnRows()).toEqual(['0']);
    }, SLOW_MS);
    expect(cell('0', 'detail')).toBe(`(${names('کیلا', 'Banana')} 30 . 2600)`);
    expect(cell('0', 'value')).toBe('15,600');
  });

  it('prints the same lines, with the backend’s figures and the totals row', async () => {
    const { user } = await openCredit({ print: PRINT_R5_EN });
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
    await screen.findByText(printFirmName(language), undefined, SLOW_MS);
    expect(vi.mocked(reportsApi.reportsPrintData).mock.calls[0]?.[0]).toMatchObject({
      id: 'R5',
      language,
      paper: 'a4',
    });
    const body = visible(document.querySelector('.print-report tbody')?.textContent);
    expect(body).toContain('(Banana 30 . 580)');
    expect(body).toContain('18,684');
    expect(visible(document.querySelector('.print-report tfoot')?.textContent)).toContain('31,564');
  });

  describe('Print all slips: one sheet for each credit buyer (PRD 19.2 R5)', () => {
    // The slips come in the print language, which starts as the app's.
    const bills = language === 'ur' ? BUYER_BILLS_UR : BUYER_BILLS_EN;
    const banana = names('کیلا', 'Banana');

    async function openSlips(options: Parameters<typeof fakeReports>[0] = {}) {
      const opened = await openCredit(options);
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await opened.user.click(screen.getByRole('button', { name: named(m('report-print-all')) }));
      await screen.findAllByText(printFirmName(language), undefined, SLOW_MS);
      return opened;
    }

    it('asks the backend for every buyer of the filter, in the print language and on A4, with the filters of the screen', async () => {
      await openSlips();
      expect(vi.mocked(reportsApi.reportsBuyerBillsPrintData)).toHaveBeenCalledTimes(1);
      expect(vi.mocked(reportsApi.reportsBuyerBillsPrintData).mock.calls[0]?.[0]).toMatchObject({
        id: 'R5',
        language,
        paper: 'a4',
        filters: { bikri_id: null, garden_id: null },
      });
      // It is not the table print: that one is not asked for.
      expect(vi.mocked(reportsApi.reportsPrintData)).not.toHaveBeenCalled();
    });

    it('draws one sheet for each buyer, in the backend’s order, each with his own lines and totals', async () => {
      await openSlips();
      const sheets = Array.from(document.querySelectorAll('.print-report .print-sheet'));
      expect(sheets).toHaveLength(2);
      const first = visible(sheets[0]?.textContent);
      const second = visible(sheets[1]?.textContent);
      // ارشد first, as the backend orders the names; each sheet holds only his line.
      expect(first).toContain(names('ارشد ملتانی', 'Arshad Multani'));
      expect(first).toContain(`(${banana} 30 . 580)`);
      expect(first).not.toContain(`(${banana} 20 . 600)`);
      expect(second).toContain(names('اکرم ٹول ٹیکس', 'Akram Toll Tax'));
      expect(second).toContain(`(${banana} 20 . 600)`);
      // His totals row is the backend's, never added up here.
      expect(visible(sheets[0]?.querySelector('tfoot')?.textContent)).toContain('18,684');
      expect(visible(sheets[1]?.querySelector('tfoot')?.textContent)).toContain('12,880');
      // His total نگ, with the backend's own message and figure.
      expect(first).toContain(message(language, bills.qty_label_id));
    });

    it('shows what all the slips come to in the preview only, from the backend’s totals', async () => {
      await openSlips();
      const total = visible(document.querySelector('[data-slips-total]')?.textContent);
      expect(total).toContain('2');
      expect(total).toContain('50');
      expect(total).toContain('29,400');
      expect(total).toContain('31,564');
      // It is not inside a slip, so it is not on a buyer's paper.
      expect(document.querySelector('.print-sheet [data-slips-total]')).toBeNull();
    });

    it('takes another print language in the frame and asks again', async () => {
      const { user } = await openSlips();
      // The frame's own choice, not the language switch of the top bar.
      await user.click(
        within(screen.getByRole('main')).getByRole('button', {
          name: message(language, `common-language-${other}`),
        }),
      );
      await waitFor(() => {
        expect(
          vi.mocked(reportsApi.reportsBuyerBillsPrintData).mock.calls.at(-1)?.[0],
        ).toMatchObject({ language: other });
      }, SLOW_MS);
    });

    it('goes back to the table with the filters as they were, and Esc does the same', async () => {
      const { user } = await openSlips();
      await user.click(screen.getByRole('button', { name: named(m('common-back')) }));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-print-all')) }));
      await screen.findAllByText(printFirmName(language), undefined, SLOW_MS);
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
    });

    it('says there is nothing to print when no buyer has a slip', async () => {
      const opened = await openCredit({
        slips: { ...bills, slips: [], slip_count: '0', qty: '0' },
      });
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(2);
      }, SLOW_MS);
      await opened.user.click(screen.getByRole('button', { name: named(m('report-print-all')) }));
      await screen.findByText(named(m('report-print-empty')), undefined, SLOW_MS);
    });

    it('is offered on R5 only', async () => {
      fakeReports();
      await renderApp('/reports/R4');
      await screen.findByRole('heading', { level: 1, name: named(m('report-r4-title')) });
      expect(screen.queryByRole('button', { name: named(m('report-print-all')) })).toBeNull();
    });
  });

  it('Enter on a buyer does nothing here (the lines have no record of their own) and nothing breaks', async () => {
    const { user } = await openCredit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    const target = document.querySelector('.ag-row[row-id="0"] [col-id="buyer"]');
    if (target === null) throw new Error('no buyer cell');
    await user.click(target);
    await user.keyboard('{Enter}');
    expect(
      screen.getByRole('heading', { level: 1, name: named(m('report-r5-title')) }),
    ).toBeVisible();
  });

  it('says there is nothing, and what happened when the backend fails, with a retry', async () => {
    let failing = true;
    const { user } = await openCredit({
      pages: { R5: { ...R5_CREDIT_BUYERS, rows: [], total_count: 0 } },
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

  it('keeps the lines across a language switch, writing the item in the other language', async () => {
    await openCredit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(2);
    }, SLOW_MS);
    const before = runCount();
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    await waitFor(() => {
      expect(cell('0', 'detail')).toBe(`(${other === 'ur' ? 'کیلا' : 'Banana'} 30 . 580)`);
    }, SLOW_MS);
    expect(runCount()).toBe(before);
  });
});

describe.each(LANGUAGES)('R1, the sale book (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);

  async function openBook(options: Parameters<typeof fakeReports>[0] = {}) {
    const fake = fakeReports(options);
    const view = await renderApp('/reports/R1');
    await screen.findByRole('heading', { level: 1, name: named(m('report-r1-title')) });
    return { ...view, fake, user: userEvent.setup() };
  }

  /** Types the number into the box and takes the first match with Enter. */
  async function pickBikri(user: ReturnType<typeof userEvent.setup>) {
    const box = screen.getByRole('combobox', { name: named(m('report-filter-bikri')) });
    await user.click(box);
    await user.keyboard('1');
    await screen.findByRole(
      'option',
      { name: (name) => visible(name).includes('10-10-2026') },
      SLOW_MS,
    );
    await user.keyboard('{Enter}');
  }

  it('is in the catalogue with one box, the بکری, and cannot run without it', () => {
    expect(CATALOG.find((entry) => entry.id === 'R1')).toMatchObject({
      group: 'sales',
      title_id: 'report-r1-title',
      filters: ['bikri'],
      party_required: true,
    });
  });

  it('starts by asking for a بکری, and asks the backend for nothing until one is picked', async () => {
    await openBook();
    await screen.findByText(named(m('report-pick-bikri')), undefined, SLOW_MS);
    expect(screen.getByText(named(m('report-pick-bikri-help')))).toBeVisible();
    expect(runCount()).toBe(0);
    // The generic preview waits too: the bill is printed from the بکری, once one is picked.
    expect(screen.getByRole('button', { name: named(m('report-preview')) })).toBeDisabled();
  });

  it('finds the بکری by its number, takes it with Enter and shows its lots with the backend’s totals', async () => {
    const { user } = await openBook();
    await pickBikri(user);
    await waitFor(() => {
      expect(lastRun()?.filters.bikri_id).toBe(1);
    }, SLOW_MS);
    expect(bikriApi.bikriFind).toHaveBeenCalledWith({
      number: 1,
      truck_no: null,
      seller_id: null,
      date: null,
      item_id: null,
    });
    await waitFor(() => {
      expect(drawnRows()).toEqual(['0']);
    }, SLOW_MS);
    // The lots of the bill that fit the normal step: the rest wait for the wide step or the print.
    expect(drawnColumns()).toEqual([
      'date',
      'seller',
      'item',
      'qty',
      'gross',
      'commission',
      'deductions',
      'net',
    ]);
    expect(cell('0', 'seller')).toBe(`030110001 ${names('رحیم', 'Rahim')}`);
    expect(cell('0', 'qty')).toBe('50');
    expect(cell('0', 'gross')).toBe('29,400');
    expect(totalsCell('gross')).toBe('29,400');
    expect(totalsCell('net')).toBe('19,316');
    // The number picked is in the box.
    expect(screen.getByRole('combobox', { name: named(m('report-filter-bikri')) })).toHaveValue(
      '1',
    );
  });

  it('prints the seller’s bill itself: the print button is the bill’s, not a preview of the report', async () => {
    const { user } = await openBook();
    await pickBikri(user);
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(1);
    }, SLOW_MS);
    expect(screen.getByRole('button', { name: named(m('print-box-print')) })).toBeVisible();
    expect(screen.queryByRole('button', { name: named(m('report-preview')) })).toBeNull();
  });

  it('typing again after a pick lets the بکری go, so the report waits for another', async () => {
    const { user } = await openBook();
    await pickBikri(user);
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(1);
    }, SLOW_MS);
    const box = screen.getByRole('combobox', { name: named(m('report-filter-bikri')) });
    await user.clear(box);
    await screen.findByText(named(m('report-pick-bikri')), undefined, SLOW_MS);
    expect(R1_BIKRI_1.rows).toHaveLength(1);
  });
});
