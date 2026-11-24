// The sale book (R1) on the ONE report screen (round 12, PRD 19.2, FR-12.12, FR-19.12), in the Urdu
// and in the English app, against a fake backend that answers with the backend's own fixtures. (R5 is
// the اگراہی sheet since plan 76: its own screen, tests/unit/collection-sheet-page.test.tsx.) R1 is the lots of one
// بکری: it cannot run until a بکری is picked (its number is typed and found), and its print is the
// seller's bill itself, not a page of the report. The four states, the print, a language switch and the
// keyboard are each proved. The screen adds up nothing: every figure is one the backend sent.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { listRow } from '../helpers/bikri-saved';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { CATALOG, R1_BIKRI_1, fakeReports } from '../helpers/reports';

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
      buyer_account_id: null,
      buyer_text: null,
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
    expect(cell('0', 'seller')).toBe(`0010 ${names('رحیم', 'Rahim')}`);
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
    // The button names the word of the print and its key. jsdom loads no CSS, so the short word that gives way to
    // the key in a short row (plan 95) is in its name too: the name starts with the print word, and there is one.
    const printButtons = screen
      .getAllByRole('button')
      .filter((button) => visible(button.textContent).startsWith(m('print-box-print')));
    expect(printButtons).toHaveLength(1);
    expect(printButtons[0]).toBeVisible();
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
