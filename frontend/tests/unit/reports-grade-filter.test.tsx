// The quality box of the sales registers (R2, R3, R4; PRD FR-12.42, plan 59 Section 11 item 10): the
// backend's catalogue lists a `grade` filter on those three reports, so the screen has the box, sends
// the quality only when one is chosen, reads it from an address, keeps it for the Owner, and a button
// under an assistant answer opens the register with the quality the answer was read with. A report
// that does not take the box never sends a left-over quality.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { SourceReportFilters } from '../../src/bindings/SourceReportFilters';
import { reportFiltersOf, reportSearchFromFilters } from '../../src/features/assistant/addresses';
import * as reportsApi from '../../src/features/reports/api';
import {
  EMPTY_FILTERS,
  GRADE_CHOICES,
  filtersToWire,
  isUnfiltered,
} from '../../src/features/reports/filters';
import { filtersFromRemembered, rememberedFilters } from '../../src/features/reports/remember';
import { parseReportSearch, stateFromSearch } from '../../src/features/reports/search';
import { LANGUAGES } from '../../src/i18n';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { fakeAccountSearch } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { CATALOG, fakeReports } from '../helpers/reports';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

const SLOW_MS = { timeout: 8_000 };
const MONTH = { from: '01-10-2026', to: '31-10-2026' };
const REGISTERS = ['R2', 'R3', 'R4'] as const;

const entryOf = (id: string) => {
  const found = CATALOG.find((entry) => entry.id === id);
  if (found === undefined) throw new Error(`no ${id}`);
  return found;
};

function sourceFilters(filters: Partial<SourceReportFilters> = {}): SourceReportFilters {
  return {
    shortcut: null,
    from: '',
    to: '',
    as_of: '',
    party: null,
    seller: null,
    zamindar: null,
    item: null,
    city: null,
    adda: null,
    detail: false,
    group_by: null,
    status: null,
    partner: null,
    voucher_type: null,
    trip_view: null,
    garden: null,
    garden_view: null,
    bikri: null,
    guests_only: false,
    ...filters,
  };
}

describe('the catalogue and the filters', () => {
  it('lists the quality box on the three sales registers, and on no other report', () => {
    for (const id of REGISTERS) expect(entryOf(id).filters, id).toContain('grade');
    const others = CATALOG.filter((entry) => !(REGISTERS as readonly string[]).includes(entry.id));
    for (const entry of others) expect(entry.filters, entry.id).not.toContain('grade');
  });

  it('sends the quality only when one is chosen, and never "all"', () => {
    const takes = entryOf('R2').filters;
    expect(filtersToWire(EMPTY_FILTERS, takes)).not.toHaveProperty('grade');
    expect(filtersToWire({ ...EMPTY_FILTERS, grade: 'all' }, takes)).not.toHaveProperty('grade');
    for (const grade of ['first', 'second', 'third'] as const) {
      expect(filtersToWire({ ...EMPTY_FILTERS, grade }, takes)).toMatchObject({ grade });
    }
  });

  it('a report without the box never sends a quality left over from another report', () => {
    const wire = filtersToWire({ ...EMPTY_FILTERS, grade: 'second' }, entryOf('R7').filters);
    expect(wire).not.toHaveProperty('grade');
  });

  it('a chosen quality is a filter to clear, but only where the report takes the box', () => {
    const state = { ...EMPTY_FILTERS, grade: 'third' as const };
    expect(isUnfiltered(EMPTY_FILTERS, entryOf('R3').filters)).toBe(true);
    expect(isUnfiltered(state, entryOf('R3').filters)).toBe(false);
    expect(isUnfiltered(state, entryOf('R7').filters)).toBe(true);
  });

  it('has the three qualities in order after "all"', () => {
    expect(GRADE_CHOICES).toEqual(['all', 'first', 'second', 'third']);
  });
});

describe('the address, the memory and the answer button', () => {
  it('reads a quality from the address, and leaves out "all", an unknown word or none', () => {
    expect(parseReportSearch({ grade: 'first' })).toEqual({ grade: 'first' });
    expect(parseReportSearch({ grade: 'all' })).toEqual({});
    expect(parseReportSearch({ grade: 'fourth' })).toEqual({});
    expect(parseReportSearch({})).toEqual({});
    expect(stateFromSearch({ grade: 'second' }).grade).toBe('second');
    expect(stateFromSearch({}).grade).toBe('all');
  });

  it('keeps a chosen quality for the Owner and reads it back, and keeps nothing for "all"', () => {
    expect(rememberedFilters(EMPTY_FILTERS)).toBeNull();
    const kept = rememberedFilters({ ...EMPTY_FILTERS, grade: 'second' });
    expect(kept).toEqual({ grade: 'second' });
    expect(filtersFromRemembered(JSON.parse(JSON.stringify(kept)))?.grade).toBe('second');
    // A quality that cannot be read any more starts as "all".
    expect(filtersFromRemembered({ grade: 'fourth' })?.grade).toBe('all');
  });

  it('a button under an answer opens the register with the quality the answer was read with', () => {
    const source = sourceFilters({ grade: 'first', from: '2026-10-01', to: '2026-10-31' });
    expect(reportSearchFromFilters(source)).toEqual({ ...MONTH, grade: 'first' });
    expect(reportFiltersOf(source)).toMatchObject({ grade: 'first' });
    // No quality in the source: none in the address and none in the print.
    expect(reportSearchFromFilters(sourceFilters())).not.toHaveProperty('grade');
    expect(reportFiltersOf(sourceFilters())).not.toHaveProperty('grade');
  });
});

describe.each(LANGUAGES)('the quality box on a register (%s)', (language) => {
  beforeEach(() => {
    vi.resetAllMocks();
    vi.mocked(useLayoutStep).mockReturnValue('normal');
    healthyBackend();
    fakeAccountSearch([]);
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  afterEach(() => {
    leaveAppWindow();
  });

  const m = (id: string) => message(language, id);
  const lastRun = () => vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0];
  const gradeBox = () =>
    screen.findByRole('combobox', {
      name: (name) => visible(name).includes(m('common-grade')),
    });

  it('shows the quality the address names and asks the backend for exactly it', async () => {
    fakeReports();
    await renderApp(`/reports/R2?grade=second&from=${MONTH.from}&to=${MONTH.to}`);
    await waitFor(() => {
      expect(lastRun()?.filters).toMatchObject({ grade: 'second' });
    }, SLOW_MS);
    expect(visible((await gradeBox()).textContent)).toContain(m('glossary-darja-doyam'));
  });

  it.each(['grade=all', 'grade=fourth', ''])(
    'sends no quality for "%s", and the box reads "all qualities"',
    async (extra) => {
      fakeReports();
      await renderApp(`/reports/R3?${extra}&from=${MONTH.from}&to=${MONTH.to}`);
      await waitFor(() => {
        expect(lastRun()?.id).toBe('R3');
      }, SLOW_MS);
      expect(lastRun()?.filters).not.toHaveProperty('grade');
      expect(visible((await gradeBox()).textContent)).toContain(m('common-grade-all'));
    },
  );

  it('choosing a quality runs the register again with it, and "all" takes it away', async () => {
    fakeReports();
    const user = userEvent.setup();
    await renderApp(`/reports/R4?from=${MONTH.from}&to=${MONTH.to}`);
    await user.click(await gradeBox());
    // The list holds "all" and the three qualities, in the glossary's words.
    const options = await screen.findAllByRole('option');
    expect(options.map((option) => visible(option.textContent))).toEqual([
      m('common-grade-all'),
      m('glossary-darja-awwal'),
      m('glossary-darja-doyam'),
      m('glossary-darja-soyam'),
    ]);
    await user.click(options[3] ?? document.body);
    await waitFor(() => {
      expect(lastRun()?.filters).toMatchObject({ grade: 'third' });
    }, SLOW_MS);
    await user.click(await gradeBox());
    await user.click((await screen.findAllByRole('option'))[0] ?? document.body);
    await waitFor(() => {
      expect(lastRun()?.filters).not.toHaveProperty('grade');
    }, SLOW_MS);
  });

  it('the boxes sit in the shared filter grid, so no box is squeezed narrower than a filter (NFR-USE-16)', async () => {
    fakeReports();
    await renderApp(`/reports/R2?from=${MONTH.from}&to=${MONTH.to}`);
    const box = await gradeBox();
    expect(box.closest('.filter-grid')).not.toBeNull();
    expect(box.closest('[role="group"]')).toBe(document.querySelector('.filter-grid'));
  });

  it('a register that does not take the box shows none and sends none', async () => {
    fakeReports();
    await renderApp(`/reports/R7?grade=second&from=${MONTH.from}&to=${MONTH.to}`);
    await waitFor(() => {
      expect(lastRun()?.id).toBe('R7');
    }, SLOW_MS);
    expect(lastRun()?.filters).not.toHaveProperty('grade');
    expect(
      screen.queryByRole('combobox', {
        name: (name) => visible(name).includes(m('common-grade')),
      }),
    ).toBeNull();
  });
});
