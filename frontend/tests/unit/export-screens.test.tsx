// The Excel export on the report screens and the analytics views (D5-Q1, backend round 10), in the
// Urdu and in the English app, against a fake backend that answers with the backend's own fixtures.
// One press opens the save box (a name from the report and the day), then asks the backend for EVERY
// row of the filters in the boxes now, in the app language (in the print preview: the print language
// chosen there); the progress shown is the one the backend sends for this job only; the result says
// where the file is and how many rows it holds; each refusal is told at the right place; Esc does
// nothing while the file is written and closes the notice after it; the action is reachable by the
// keyboard; a language switch in the middle loses nothing. The screens add up nothing.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ExcelExported } from '../../src/bindings/ExcelExported';
import * as analyticsApi from '../../src/features/analytics/api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as dialog from '../../src/shared/api/dialog';
import * as events from '../../src/shared/api/events';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeAnalytics } from '../helpers/analytics';
import {
  EXPORT_REFUSED_FILTER,
  EXPORT_REFUSED_JOB,
  EXPORT_REFUSED_PATH,
} from '../helpers/export-real';
import { message, visible } from '../helpers/render';
import { CATALOG, fakeReports } from '../helpers/reports';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/api/dialog');
vi.mock('../../src/shared/api/events');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);

/** Matches a name that is exactly `text`, or `text` followed by the key chip of its button. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

/** The element that holds exactly this text as one leaf. Fluent wraps variables in isolation marks. */
const leaf = (expected: string) => (_content: string, element: Element | null) =>
  element !== null && element.children.length === 0 && visible(element.textContent) === expected;

/** The events the screen listens to, so a test can send the backend's progress. */
let listeners: { name: string; handler: (payload: unknown) => void }[] = [];

function send(payload: unknown) {
  act(() => {
    for (const listener of listeners) {
      if (listener.name === 'export-progress') listener.handler(payload);
    }
  });
}

/** An export the test finishes (or refuses) when it wants to. */
function pendingExport() {
  let finish: (result: ExcelExported) => void = () => undefined;
  let refuse: (error: unknown) => void = () => undefined;
  const promise = new Promise<ExcelExported>((resolve, reject) => {
    finish = resolve;
    refuse = reject;
  });
  return { promise, finish, refuse };
}

/** The parent of an element, or a plain failure. */
function parentOf(element: HTMLElement): HTMLElement {
  const parent = element.parentElement;
  if (parent === null) throw new Error('the element has no parent');
  return parent;
}

const RESULT: ExcelExported = { path: 'C:\\Reports\\trial.xlsx', rows: 1200, size_bytes: 9000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  listeners = [];
  vi.mocked(events.listenEvent).mockImplementation((name, handler) => {
    const entry = { name, handler };
    listeners.push(entry);
    return Promise.resolve(() => {
      listeners = listeners.filter((candidate) => candidate !== entry);
    });
  });
  vi.mocked(events.emitEvent).mockResolvedValue(undefined);
  vi.mocked(dialog.chooseSavePath).mockResolvedValue('C:\\Reports\\trial');
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('Excel export (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const other: Language = language === 'ur' ? 'en' : 'ur';
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const exportName = named(message(language, 'export-excel'));

  async function openReport(id = 'R18') {
    fakeReports();
    const entry = CATALOG.find((candidate) => candidate.id === id);
    const view = await renderApp(`/reports/${id}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(entry?.title_id ?? 'report-r18-title')),
    });
    return { ...view, user: userEvent.setup() };
  }

  async function openView(letter = 'A') {
    fakeAnalytics();
    const view = await renderApp(`/analytics/${letter}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(m(`analytics-view-${letter.toLowerCase()}-title`)),
    });
    return { ...view, user: userEvent.setup() };
  }

  /** The language button of the print preview's own box (the top bar has the same two words). */
  const printLanguage = (choice: Language) =>
    within(screen.getByRole('group', { name: named(m('print-box-language')) })).getByRole(
      'button',
      {
        name: named(m(`common-language-${choice}`)),
      },
    );
  const excelButton = () => screen.getByRole('button', { name: exportName });
  const lastReportRequest = () => vi.mocked(reportsApi.reportsExportExcel).mock.calls.at(-1)?.[0];

  describe('a report', () => {
    it('asks where to save, then writes every row with the filters on screen and shows its progress and its result', async () => {
      const job = pendingExport();
      vi.mocked(reportsApi.reportsExportExcel).mockReturnValue(job.promise);
      const { user } = await openReport();
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun)).toHaveBeenCalled();
      }, SLOW_MS);
      await user.click(excelButton());
      await waitFor(() => {
        expect(reportsApi.reportsExportExcel).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      // The save box starts with a name from the report and the day, and asks for Excel files.
      const asked = vi.mocked(dialog.chooseSavePath).mock.calls[0]?.[0];
      expect(asked?.defaultName).toMatch(/^R18-\d{8}\.xlsx$/);
      expect(asked?.extension).toBe('xlsx');
      expect(asked?.filterName).toBe(m('export-file-type'));
      // The command gets the chosen path with .xlsx, this job's id, the app language, and exactly the
      // filters the report was asked for: every row, never only the page.
      const request = lastReportRequest();
      expect(request?.path).toBe('C:\\Reports\\trial.xlsx');
      expect(request?.id).toBe('R18');
      expect(request?.language).toBe(language);
      expect(request?.job_id).toMatch(/^[A-Za-z0-9_-]{1,64}$/);
      expect(request?.filters).toEqual(
        vi.mocked(reportsApi.reportsRun).mock.calls.at(-1)?.[0].filters,
      );
      const jobId = request?.job_id ?? '';

      // While it works: reading, then the rows written so far; another job's progress is ignored.
      expect(await screen.findByText(leaf(m('export-reading')))).toBeInTheDocument();
      expect(excelButton()).toBeDisabled();
      send({ job_id: jobId, stage: 'reading', done: 0, total: 0 });
      send({ job_id: jobId, stage: 'writing', done: 500, total: 1200 });
      expect(
        await screen.findByText(leaf(m('export-writing', { done: '500', total: '1,200' }))),
      ).toBeInTheDocument();
      const bar = screen.getByRole('progressbar');
      expect(bar).toHaveAttribute('value', '500');
      expect(bar).toHaveAttribute('max', '1200');
      send({ job_id: 'someone-else', stage: 'writing', done: 900, total: 1200 });
      expect(screen.getByRole('progressbar')).toHaveAttribute('value', '500');
      send({ job_id: jobId, stage: 'done', done: 1200, total: 1200 });
      expect(await screen.findByText(leaf(m('export-finishing')))).toBeInTheDocument();

      // The answer says where the file is and how many rows it holds, and listening has stopped.
      act(() => {
        job.finish(RESULT);
      });
      expect(
        await screen.findByText(
          leaf(m('export-done', { path: 'C:\\Reports\\trial.xlsx', rows: '1,200' })),
        ),
      ).toBeInTheDocument();
      expect(screen.queryByRole('progressbar')).toBeNull();
      // Listening to the progress has stopped (the exit overlay's own listener stays).
      expect(listeners.filter((entry) => entry.name === 'export-progress')).toHaveLength(0);
      expect(excelButton()).toBeEnabled();
      await user.click(screen.getByRole('button', { name: named(m('export-close')) }));
      expect(
        screen.queryByText(leaf(m('export-done', { path: RESULT.path, rows: '1,200' }))),
      ).toBeNull();
    });

    it('uses the filters in the boxes now, even before the screen has asked for them', async () => {
      vi.mocked(reportsApi.reportsExportExcel).mockResolvedValue(RESULT);
      const { user } = await openReport('R11');
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsRun)).toHaveBeenCalled();
      }, SLOW_MS);
      await user.type(screen.getByLabelText(named(m('report-filter-from'))), '01-10-2026');
      await user.click(excelButton());
      await waitFor(() => {
        expect(reportsApi.reportsExportExcel).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(lastReportRequest()?.filters).toMatchObject({ from: '01-10-2026', to: '' });
      expect(lastReportRequest()?.id).toBe('R11');
      expect(vi.mocked(dialog.chooseSavePath).mock.calls[0]?.[0].defaultName).toMatch(
        /^R11-\d{8}\.xlsx$/,
      );
    });

    it('closing the save box without choosing writes nothing and shows nothing', async () => {
      vi.mocked(dialog.chooseSavePath).mockResolvedValue(null);
      const { user } = await openReport();
      await user.click(excelButton());
      await waitFor(() => {
        expect(dialog.chooseSavePath).toHaveBeenCalledTimes(1);
      });
      expect(reportsApi.reportsExportExcel).not.toHaveBeenCalled();
      await waitFor(() => {
        expect(excelButton()).toBeEnabled();
      });
      expect(screen.queryByText(leaf(m('export-reading')))).toBeNull();
      expect(screen.queryByRole('button', { name: named(m('export-close')) })).toBeNull();
    });

    it('a second press while the box is open or the file is written does nothing', async () => {
      const job = pendingExport();
      vi.mocked(reportsApi.reportsExportExcel).mockReturnValue(job.promise);
      const { user } = await openReport();
      await user.click(excelButton());
      await waitFor(() => {
        expect(reportsApi.reportsExportExcel).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      await user.click(excelButton());
      expect(dialog.chooseSavePath).toHaveBeenCalledTimes(1);
      expect(reportsApi.reportsExportExcel).toHaveBeenCalledTimes(1);
      act(() => {
        job.finish(RESULT);
      });
    });

    const REFUSALS: readonly [string, () => unknown, string | null][] = [
      [
        'a path the backend refuses',
        () => new CommandError(EXPORT_REFUSED_PATH),
        'export-path-refused',
      ],
      [
        'too many rows for one sheet',
        () => fail({ code: 'ERR_OUT_OF_RANGE', field: 'filters' }),
        'export-too-many-rows',
      ],
      [
        'a job id the backend refuses (our own slip)',
        () => new CommandError(EXPORT_REFUSED_JOB),
        'err-internal',
      ],
      ['a full disk', () => fail({ code: 'ERR_DISK_FULL' }), 'err-disk-full'],
      ['an overflow', () => fail({ code: 'ERR_OVERFLOW' }), 'err-internal'],
      [
        'a filter the backend refuses, on its own field',
        () => new CommandError(EXPORT_REFUSED_FILTER),
        'err-date-invalid',
      ],
    ];

    it.each(REFUSALS)(
      '%s is told in words, never with an empty range',
      async (_name, refusal, id) => {
        vi.mocked(reportsApi.reportsExportExcel).mockRejectedValue(refusal());
        const { user } = await openReport();
        await user.click(excelButton());
        expect(await screen.findByText(leaf(m(id ?? 'err-internal')))).toBeInTheDocument();
        expect(document.body.textContent).not.toMatch(/\{|between\s*and/);
        // He can close it, and press again.
        await user.click(screen.getByRole('button', { name: named(m('export-close')) }));
        await waitFor(() => {
          expect(screen.queryByText(leaf(m(id ?? 'err-internal')))).toBeNull();
        });
        expect(excelButton()).toBeEnabled();
      },
    );

    it('a save box that cannot open says something went wrong, and nothing is written', async () => {
      vi.mocked(dialog.chooseSavePath).mockRejectedValue(new Error('no window'));
      const { user } = await openReport();
      await user.click(excelButton());
      expect(await screen.findByText(leaf(m('err-internal')))).toBeInTheDocument();
      expect(reportsApi.reportsExportExcel).not.toHaveBeenCalled();
    });

    it('Esc does nothing while the file is written, closes the notice after it, and only then leaves the report', async () => {
      const job = pendingExport();
      vi.mocked(reportsApi.reportsExportExcel).mockReturnValue(job.promise);
      const { user, router } = await openReport();
      await user.click(excelButton());
      await screen.findByText(leaf(m('export-reading')), undefined, SLOW_MS);
      await user.keyboard('{Escape}');
      expect(router.state.location.pathname).toBe('/reports/R18');
      expect(screen.getByText(leaf(m('export-reading')))).toBeInTheDocument();
      act(() => {
        job.finish(RESULT);
      });
      await screen.findByText(leaf(m('export-done', { path: RESULT.path, rows: '1,200' })));
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(
          screen.queryByText(leaf(m('export-done', { path: RESULT.path, rows: '1,200' }))),
        ).toBeNull();
      });
      expect(router.state.location.pathname).toBe('/reports/R18');
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      });
    });

    it('keyboard only: Tab goes from Print preview to Excel, and Enter starts it', async () => {
      vi.mocked(reportsApi.reportsExportExcel).mockResolvedValue(RESULT);
      const { user } = await openReport();
      const preview = screen.getByRole('button', { name: named(m('report-preview')) });
      preview.focus();
      await user.tab();
      expect(excelButton()).toHaveFocus();
      await user.keyboard('{Enter}');
      expect(
        await screen.findByText(
          leaf(m('export-done', { path: RESULT.path, rows: '1,200' })),
          undefined,
          SLOW_MS,
        ),
      ).toBeInTheDocument();
    });

    it('a report that needs a party has its Excel button off until one is chosen', async () => {
      await openReport('R13');
      expect(excelButton()).toBeDisabled();
      expect(dialog.chooseSavePath).not.toHaveBeenCalled();
    });

    it('in the print preview the button sits with PDF and Print, and the sheet takes the print language chosen there', async () => {
      vi.mocked(reportsApi.reportsExportExcel).mockResolvedValue(RESULT);
      const { user } = await openReport();
      // The page of the print is held back until the test lets it go, so "off while it loads" is
      // seen at a moment the test chooses and does not depend on how fast the fake answers.
      const answer = vi.mocked(reportsApi.reportsPrintData).getMockImplementation();
      if (answer === undefined) throw new Error('the fake has no print data');
      let release: () => void = () => undefined;
      const held = new Promise<void>((resolve) => {
        release = resolve;
      });
      vi.mocked(reportsApi.reportsPrintData).mockImplementation(async (input) => {
        await held;
        return answer(input);
      });
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      const pdf = await screen.findByRole('button', { name: named(m('report-print-pdf')) });
      expect(pdf).toBeDisabled();
      act(() => {
        release();
      });
      await waitFor(() => {
        expect(pdf).toBeEnabled();
      }, SLOW_MS);
      // The frame has its own Excel button; the header's is not drawn twice.
      expect(screen.getAllByRole('button', { name: exportName })).toHaveLength(1);
      const row = parentOf(pdf);
      const inFrame = within(row).getByRole('button', { name: exportName });
      // The language box starts at the app language; choose the other one.
      await user.click(printLanguage(other));
      await user.click(inFrame);
      await waitFor(() => {
        expect(reportsApi.reportsExportExcel).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(lastReportRequest()?.language).toBe(other);
      expect(
        await screen.findByText(leaf(m('export-done', { path: RESULT.path, rows: '1,200' }))),
      ).toBeInTheDocument();
    });

    it('a language switch in the middle keeps the progress, and the result is written in the other language', async () => {
      const job = pendingExport();
      vi.mocked(reportsApi.reportsExportExcel).mockReturnValue(job.promise);
      const { user } = await openReport();
      await user.click(excelButton());
      await screen.findByText(leaf(m('export-reading')), undefined, SLOW_MS);
      const jobId = lastReportRequest()?.job_id ?? '';
      send({ job_id: jobId, stage: 'writing', done: 500, total: 1200 });
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      expect(
        await screen.findByText(
          leaf(message(other, 'export-writing', { done: '500', total: '1,200' })),
          undefined,
          SLOW_MS,
        ),
      ).toBeInTheDocument();
      // The file was asked for in the language of the press, whatever the screen shows now.
      expect(lastReportRequest()?.language).toBe(language);
      act(() => {
        job.finish(RESULT);
      });
      expect(
        await screen.findByText(
          leaf(message(other, 'export-done', { path: RESULT.path, rows: '1,200' })),
        ),
      ).toBeInTheDocument();
    });
  });

  describe('an analytics view', () => {
    const lastViewRequest = () =>
      vi.mocked(analyticsApi.analyticsExportExcel).mock.calls.at(-1)?.[0];

    it('sends the view, every filter as the boxes hold it and the app language, and shows the result', async () => {
      const job = pendingExport();
      vi.mocked(analyticsApi.analyticsExportExcel).mockReturnValue(job.promise);
      const { user } = await openView('B');
      await waitFor(() => {
        expect(vi.mocked(analyticsApi.analyticsRun)).toHaveBeenCalled();
      }, SLOW_MS);
      await user.click(excelButton());
      await waitFor(() => {
        expect(analyticsApi.analyticsExportExcel).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(vi.mocked(dialog.chooseSavePath).mock.calls[0]?.[0].defaultName).toMatch(
        /^analytics-B-\d{8}\.xlsx$/,
      );
      const request = lastViewRequest();
      expect(request?.view).toBe('B');
      expect(request?.language).toBe(language);
      expect(request?.path).toBe('C:\\Reports\\trial.xlsx');
      expect(request?.filters).toEqual(
        vi.mocked(analyticsApi.analyticsRun).mock.calls.at(-1)?.[0].filters,
      );
      send({ job_id: request?.job_id ?? '', stage: 'writing', done: 5, total: 20 });
      expect(
        await screen.findByText(leaf(m('export-writing', { done: '5', total: '20' }))),
      ).toBeInTheDocument();
      act(() => {
        job.finish({ ...RESULT, rows: 20 });
      });
      expect(
        await screen.findByText(leaf(m('export-done', { path: RESULT.path, rows: '20' }))),
      ).toBeInTheDocument();
    });

    it('too many rows for one sheet is told with its own words', async () => {
      vi.mocked(analyticsApi.analyticsExportExcel).mockRejectedValue(
        fail({ code: 'ERR_OUT_OF_RANGE', field: 'filters' }),
      );
      const { user } = await openView('A');
      await user.click(excelButton());
      expect(await screen.findByText(leaf(m('export-too-many-rows')))).toBeInTheDocument();
    });

    it('keyboard only: Tab from Print preview reaches Excel, and the print preview has its own button with the print language', async () => {
      vi.mocked(analyticsApi.analyticsExportExcel).mockResolvedValue(RESULT);
      const { user } = await openView('A');
      const preview = screen.getByRole('button', { name: named(m('report-preview')) });
      preview.focus();
      await user.tab();
      expect(excelButton()).toHaveFocus();
      press('F9');
      const pdf = await screen.findByRole('button', { name: named(m('report-print-pdf')) });
      await user.click(printLanguage(other));
      await user.click(within(parentOf(pdf)).getByRole('button', { name: exportName }));
      await waitFor(() => {
        expect(analyticsApi.analyticsExportExcel).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(lastViewRequest()?.language).toBe(other);
    });
  });
});
