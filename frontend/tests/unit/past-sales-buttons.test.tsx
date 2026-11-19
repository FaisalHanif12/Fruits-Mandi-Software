// The buttons of print and Excel on the page of پچھلی بکری by mouse (plan 72 P8, PRD FR-12.46, FR-19.3):
// past-sales-print.test.tsx joins the key `F9` and the filters to the print; this file clicks the two
// buttons on the whole screen. Print opens the preview with the window of the screen. Excel asks for the
// place of the file, then sends the same window, filters and switch "show every بکری" as the list, and
// the notice under the actions says where the file is, or why it was refused, and goes when it is closed.
// A closed save box sends nothing. The page stays in place while Excel works. Today is 10-10-2026. The
// backend is a double: the screen adds up nothing.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { UiError } from '../../src/bindings/UiError';
import * as printApi from '../../src/features/bikri/past-print-api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as dialog from '../../src/shared/api/dialog';
import * as events from '../../src/shared/api/events';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeBikri } from '../helpers/bikri';
import { named } from '../helpers/bikri-screen';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners } from '../helpers/partners';
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
const CHOSEN = 'C:\\Sales\\past';
const WRITTEN = 'C:\\Sales\\past.xlsx';

/** The refusal of a path the backend will not write to (the field `path` of the export command). */
const PATH_REFUSED: UiError = {
  ok: false,
  ref: '',
  errors: [{ code: 'ERR_OUT_OF_RANGE', field: 'path', values: {} }],
};

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(events.listenEvent).mockResolvedValue(() => undefined);
  vi.mocked(events.emitEvent).mockResolvedValue(undefined);
  vi.mocked(dialog.chooseSavePath).mockResolvedValue(CHOSEN);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the Print and Excel buttons of پچھلی بکری by mouse (%s)', (language) => {
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
      path: WRITTEN,
      rows: 4,
      size_bytes: 8000,
    });
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const sent = () => vi.mocked(printApi.bikriListExportExcel).mock.calls.map(([input]) => input);
  const navGroup = () => screen.queryByRole('group', { name: named(m('bikri-past-nav-label')) });
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
  const closeButton = () => screen.findByRole('button', { name: named(m('export-close')) });
  const searchBox = () => screen.findByRole('textbox', { name: named(m('common-search-label')) });

  it('a click on Print opens the preview with the window of the screen and today’s section', async () => {
    const { user } = await open();
    await waitFor(() => {
      expect(printButton()).toBeEnabled();
    });
    await user.click(printButton());
    await waitFor(() => {
      expect(printApi.bikriListPrintData).toHaveBeenCalledTimes(1);
    }, SLOW_MS);
    expect(vi.mocked(printApi.bikriListPrintData).mock.calls[0]?.[0]).toMatchObject({
      from: '03-10-2026',
      to: '09-10-2026',
      with_today: true,
      all: false,
      language,
    });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(m('bikri-list-print-title'));
    }, SLOW_MS);
    expect(navGroup()).toBeNull();
  });

  it('a click on Excel asks where to save, sends the window of the screen and says where the file is', async () => {
    const { user } = await open();
    await waitFor(() => {
      expect(excelButton()).toBeEnabled();
    });
    await user.click(excelButton());
    await waitFor(() => {
      expect(sent()).toHaveLength(1);
    }, SLOW_MS);
    const box = vi.mocked(dialog.chooseSavePath).mock.calls[0]?.[0];
    expect(box?.extension).toBe('xlsx');
    expect(box?.defaultName.startsWith('past-sales')).toBe(true);
    expect(sent()[0]).toMatchObject({
      from: '03-10-2026',
      to: '09-10-2026',
      with_today: true,
      seller_id: null,
      item_id: null,
      status: null,
      show_finished: false,
      search: null,
      all: false,
      language,
      path: WRITTEN,
    });
    expect(sent()[0]?.job_id).toMatch(/\S/);
    // The notice says where the file is and how many rows are in it, under the actions of the page.
    expect(
      await screen.findByText(
        (text) => visible(text) === m('export-done', { path: WRITTEN, rows: 4 }),
      ),
    ).toBeInTheDocument();
    // The page stays: Excel never takes its place the way the preview does.
    expect(navGroup()).not.toBeNull();
    await user.click(await closeButton());
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: named(m('export-close')) })).toBeNull();
    });
  });

  it('Excel sends the earlier window and the filter the page has, and the switch sets the filter aside', async () => {
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(m('bikri-past-back', { days: 7 })),
      }),
    );
    await user.type(await searchBox(), 'LES');
    await waitFor(() => {
      expect(excelButton()).toBeEnabled();
    });
    await user.click(excelButton());
    await waitFor(() => {
      expect(sent()).toHaveLength(1);
    }, SLOW_MS);
    expect(sent()[0]).toMatchObject({
      from: '26-09-2026',
      to: '02-10-2026',
      search: 'LES',
      all: false,
    });
    await user.click(await closeButton());
    // With the switch on, the filter is set aside and the whole window goes.
    await user.click(
      await screen.findByRole('checkbox', { name: named(m('bikri-past-show-all')) }),
    );
    await user.click(excelButton());
    await waitFor(() => {
      expect(sent()).toHaveLength(2);
    }, SLOW_MS);
    expect(sent()[1]).toMatchObject({
      from: '26-09-2026',
      to: '02-10-2026',
      search: null,
      all: true,
    });
  });

  it('a save box that was closed sends nothing and leaves no notice', async () => {
    vi.mocked(dialog.chooseSavePath).mockResolvedValue(null);
    const { user } = await open();
    await waitFor(() => {
      expect(excelButton()).toBeEnabled();
    });
    await user.click(excelButton());
    await waitFor(() => {
      expect(dialog.chooseSavePath).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(excelButton()).toBeEnabled();
    });
    expect(sent()).toHaveLength(0);
    expect(screen.queryByRole('button', { name: named(m('export-close')) })).toBeNull();
    expect(navGroup()).not.toBeNull();
  });

  it('a refused path is told in words under the actions, and the page is still there', async () => {
    vi.mocked(printApi.bikriListExportExcel).mockRejectedValue(PATH_REFUSED);
    const { user } = await open();
    await waitFor(() => {
      expect(excelButton()).toBeEnabled();
    });
    await user.click(excelButton());
    expect(
      await screen.findByText((text) => visible(text) === m('export-path-refused')),
    ).toBeInTheDocument();
    expect(navGroup()).not.toBeNull();
    await user.click(await closeButton());
    await waitFor(() => {
      expect(screen.queryByText((text) => visible(text) === m('export-path-refused'))).toBeNull();
    });
  });
});
