// Print and Excel on the page of پچھلی بکری itself (plan 72 P8, PRD FR-12.46): the whole screen, not the hook on
// its own (past-print.test.tsx draws the hook and the preview alone). What is checked here is the
// joining: the Print `F9` and Excel buttons are in the actions of the screen; the print is asked with the
// window, the filters and the switch "show every بکری" the lists have now; the preview takes the place of
// the page, and Esc brings the page back with its filters as they were; Esc on the page leaves it as
// before. Today in these tests is 10-10-2026. The print data is a double: the screen adds up nothing.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as printApi from '../../src/features/bikri/past-print-api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as dialog from '../../src/shared/api/dialog';
import * as events from '../../src/shared/api/events';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { pastPrintDataFor } from '../helpers/past-print';
import { fakePrinting } from '../helpers/print';
import { printFrameFor } from '../helpers/print-frame-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/bikri/past-print-api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/api/dialog');
vi.mock('../../src/shared/api/events');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(events.listenEvent).mockResolvedValue(() => undefined);
  vi.mocked(events.emitEvent).mockResolvedValue(undefined);
  vi.mocked(dialog.chooseSavePath).mockResolvedValue('C:\\Sales\\past');
});

afterEach(() => {
  vi.unstubAllGlobals();
  leaveAppWindow();
});

/** The width of the window as the three steps see it: media queries that match up to a step. */
function stubStep(step: 'compact' | 'normal' | 'wide') {
  const matchesFor = (query: string) => {
    if (query.includes('1920')) return step === 'wide';
    if (query.includes('1366')) return step !== 'compact';
    return false;
  };
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: matchesFor(query),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  }));
  document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
  document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
}

describe.each(LANGUAGES)('Print and Excel on پچھلی بکری (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    fakePrinting();
    vi.mocked(reportsApi.reportsPrintHeader).mockImplementation((input) =>
      Promise.resolve(printFrameFor(input)),
    );
    vi.mocked(printApi.bikriListPrintData).mockImplementation((input) =>
      Promise.resolve(pastPrintDataFor(input, {})),
    );
    vi.mocked(printApi.bikriListPrint).mockResolvedValue({ printed: [], pdf_paths: null });
    vi.mocked(printApi.bikriListExportExcel).mockResolvedValue({
      path: 'C:\\Sales\\past.xlsx',
      rows: 4,
      size_bytes: 8000,
    });
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const pageText = () => visible(document.body.textContent);
  const printCalls = () =>
    vi.mocked(printApi.bikriListPrintData).mock.calls.map(([input]) => input);
  /** The page with its days known: the divider is drawn and the buttons are on. */
  const open = async () => {
    const view = await renderApp('/bikri/today');
    await screen.findByRole('group', { name: named(m('bikri-past-nav-label')) });
    return { ...view, user: userEvent.setup() };
  };
  const printButton = () =>
    screen.getByRole('button', {
      name: (name) =>
        visible(name).startsWith(m('print-box-print')) && visible(name).endsWith('F9'),
    });
  const excelButton = () => screen.getByRole('button', { name: named(m('export-excel')) });

  it('the Print F9 and Excel buttons are in the actions, beside New, and are on once the days are known', async () => {
    await open();
    await waitFor(() => {
      expect(printButton()).toBeEnabled();
    });
    expect(excelButton()).toBeEnabled();
    expect(printCalls()).toHaveLength(0);
  });

  it('at the compact step (1024 px) Print is its icon and its key and Excel its icon; the words are their names', async () => {
    stubStep('compact');
    await open();
    await waitFor(() => {
      expect(screen.getByRole('button', { name: named(m('print-box-print')) })).toBeEnabled();
    });
    const print = screen.getByRole('button', { name: named(m('print-box-print')) });
    const excel = screen.getByRole('button', { name: named(m('export-excel')) });
    // No words are drawn: the key chip is all the Print button says, and Excel says nothing but its icon.
    expect(visible(print.textContent)).toBe('F9');
    expect(visible(excel.textContent)).toBe('');
    expect(excel).toBeEnabled();
    // The key still works, and the button still opens the preview.
    press('F9');
    await waitFor(() => {
      expect(printCalls()).toHaveLength(1);
    }, SLOW_MS);
  });

  it('F9 asks for the print with the window of the screen, today’s section and no filters', async () => {
    await open();
    await waitFor(() => {
      expect(printButton()).toBeEnabled();
    });
    press('F9');
    await waitFor(() => {
      expect(printCalls()).toHaveLength(1);
    }, SLOW_MS);
    expect(printCalls()[0]).toMatchObject({
      from: '03-10-2026',
      to: '09-10-2026',
      seller_id: null,
      item_id: null,
      status: null,
      show_finished: false,
      search: null,
      all: false,
      language,
    });
    // The preview takes the place of the page.
    await waitFor(() => {
      expect(pageText()).toContain(m('bikri-list-print-title'));
    }, SLOW_MS);
    expect(screen.queryByRole('group', { name: named(m('bikri-past-nav-label')) })).toBeNull();
  });

  it('the window the Owner moved to, the filters and the switch go with the print, exactly as the lists have them', async () => {
    const { user } = await open();
    await user.type(
      await screen.findByRole('textbox', { name: named(m('common-search-label')) }),
      'LES',
    );
    await user.click(
      await screen.findByRole('checkbox', { name: named(m('bikri-past-show-all')) }),
    );
    await waitFor(() => {
      expect(
        vi
          .mocked(bikriApi.bikriList)
          .mock.calls.map(([input]) => input)
          .at(-1),
      ).toMatchObject({ all: true });
    });
    press('F9');
    await waitFor(() => {
      expect(printCalls()).toHaveLength(1);
    }, SLOW_MS);
    // With the switch on the filters are set aside, so they do not go (they keep their values).
    expect(printCalls()[0]).toMatchObject({
      from: '03-10-2026',
      to: '09-10-2026',
      all: true,
      search: null,
    });
  });

  it('a filter and an earlier window go with the print when the switch is off', async () => {
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(m('bikri-past-back', { days: 7 })),
      }),
    );
    await user.type(
      await screen.findByRole('textbox', { name: named(m('common-search-label')) }),
      'LES',
    );
    await waitFor(() => {
      expect(
        vi
          .mocked(bikriApi.bikriList)
          .mock.calls.map(([input]) => input)
          .at(-1),
      ).toMatchObject({ from: '26-09-2026', to: '02-10-2026', search: 'LES' });
    });
    press('F9');
    await waitFor(() => {
      expect(printCalls()).toHaveLength(1);
    }, SLOW_MS);
    expect(printCalls()[0]).toMatchObject({
      from: '26-09-2026',
      to: '02-10-2026',
      search: 'LES',
      all: false,
    });
  });

  it('Esc closes the preview and the page is back with its filters; Esc on the page leaves it', async () => {
    const { router, user } = await open();
    const search = await screen.findByRole('textbox', { name: named(m('common-search-label')) });
    await user.type(search, 'LES');
    await waitFor(() => {
      expect(printButton()).toBeEnabled();
    });
    press('F9');
    await waitFor(() => {
      expect(pageText()).toContain(m('bikri-list-print-title'));
    }, SLOW_MS);
    // The first Esc closes the preview only.
    press('Escape', { key: 'Escape' });
    await screen.findByRole('group', { name: named(m('bikri-past-nav-label')) });
    expect(router.state.location.pathname).toBe('/bikri/today');
    expect(
      await screen.findByRole('textbox', { name: named(m('common-search-label')) }),
    ).toHaveValue('LES');
    // The next one leaves the screen, as before.
    press('Escape', { key: 'Escape' });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });
});
