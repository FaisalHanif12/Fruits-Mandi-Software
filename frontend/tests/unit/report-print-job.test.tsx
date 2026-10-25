// "Print" and "Save as PDF" in the preview of a report, of the slips of R5 and of an analytics view
// (round 14, 17e, PRD FR-19.3, FR-18.4, FR-7.36), in the Urdu and in the English app. The buttons send
// the page on screen to the backend's print job with the language and the paper chosen in the frame:
// the files of a PDF are shown, "sent to the printer" is said for a printer job, a refusal (printing
// works only on Windows) is shown in words and the preview stays, and changing the language or the
// paper forgets what the last job did. Nothing is sent for a page with nothing in it.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as analyticsApi from '../../src/features/analytics/api';
import { useAnalyticsStore } from '../../src/features/analytics/store';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeAnalytics } from '../helpers/analytics';
import { fakeGardens } from '../helpers/gardens';
import { fakePartners, partnerRow } from '../helpers/partners';
import { PRINT_WINDOWS_ONLY } from '../helpers/print-job';
import { message, visible } from '../helpers/render';
import { PRINT_R5_EN, R5_CREDIT_BUYERS, fakeReports } from '../helpers/reports';
import { fakeTrips } from '../helpers/trips';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/analytics/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');

const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners([partnerRow(1, 'الف', 'Partner A', 4000)]);
  fakeTrips();
  fakeGardens();
  useAnalyticsStore.getState().reset();
});

afterEach(() => {
  leaveAppWindow();
});

const KEY_CHIPS = new Set(['F9']);

/** A button by its words, which may be followed by its key. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  return seen === text || (seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim()));
};

describe.each(LANGUAGES)('print and PDF from a preview (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string) => message(language, id);
  const other: Language = language === 'ur' ? 'en' : 'ur';

  async function openPreview(path: string, button = 'report-preview') {
    const user = userEvent.setup();
    await renderApp(path);
    await screen.findByRole('heading', { level: 1 }, SLOW_MS);
    await waitFor(() => {
      expect(document.querySelector('.ag-row')).not.toBeNull();
    }, SLOW_MS);
    await user.click(screen.getByRole('button', { name: named(m(button)) }));
    await screen.findByRole('heading', { level: 2, name: named(m('report-print-title')) }, SLOW_MS);
    await waitFor(() => {
      expect(document.querySelector('.print-sheet')).not.toBeNull();
    }, SLOW_MS);
    return user;
  }

  describe('a report', () => {
    it('Save as PDF sends the report with the language and paper of the frame, and shows the file', async () => {
      fakeReports();
      const user = await openPreview('/reports/R5');
      await user.click(screen.getByRole('button', { name: named(m('report-print-pdf')) }));
      await waitFor(() => {
        expect(reportsApi.reportsPrintReport).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(vi.mocked(reportsApi.reportsPrintReport).mock.calls[0]?.[0]).toMatchObject({
        report: { id: 'R5', language, paper: 'a4' },
        slips: false,
        to: 'pdf',
        copies: null,
      });
      const notice = await screen.findByRole('status', undefined, SLOW_MS);
      expect(visible(notice.textContent)).toContain(m('print-box-done-pdf'));
      expect(visible(notice.textContent)).toContain(`report-r5-20261010-${language}.pdf`);
    });

    it('Print sends it to the printer and says so, in words', async () => {
      fakeReports();
      const user = await openPreview('/reports/R18');
      await user.click(screen.getByRole('button', { name: named(m('report-print-print')) }));
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsPrintReport).mock.calls[0]?.[0]).toMatchObject({
          report: { id: 'R18' },
          to: 'printer',
        });
      }, SLOW_MS);
      const notice = await screen.findByRole('status', undefined, SLOW_MS);
      expect(visible(notice.textContent)).toBe(m('report-print-sent'));
    });

    it('asks for the paper and the language chosen in the frame, and forgets the last job when they change', async () => {
      fakeReports();
      const user = await openPreview('/reports/R18');
      await user.click(screen.getByRole('button', { name: named(m('report-print-print')) }));
      await screen.findByRole('status', undefined, SLOW_MS);
      // A5 paper: the notice was about the page before, so it goes.
      await user.click(
        within(screen.getByRole('main')).getByRole('button', { name: m('print-paper-a5') }),
      );
      await waitFor(() => {
        expect(screen.queryByRole('status')).toBeNull();
      }, SLOW_MS);
      const group = screen.getByRole('group', { name: m('print-box-language') });
      await user.click(
        within(group).getByRole('button', { name: other === 'ur' ? 'اردو' : 'English' }),
      );
      await screen.findAllByText(message(other, 'report-r18-title'), { exact: false }, SLOW_MS);
      await user.click(screen.getByRole('button', { name: named(m('report-print-pdf')) }));
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsPrintReport).mock.calls.at(-1)?.[0]).toMatchObject({
          report: { id: 'R18', language: other, paper: 'a5' },
          to: 'pdf',
        });
      }, SLOW_MS);
    });

    it('says in words that printing works only on Windows, and the preview stays', async () => {
      fakeReports({ printJob: () => PRINT_WINDOWS_ONLY });
      const user = await openPreview('/reports/R18');
      await user.click(screen.getByRole('button', { name: named(m('report-print-print')) }));
      const alert = await screen.findByRole('alert', undefined, SLOW_MS);
      expect(visible(alert.textContent)).toContain(m('err-print-windows-only'));
      expect(visible(alert.textContent)).not.toContain('ERR_');
      expect(document.querySelector('.print-sheet')).not.toBeNull();
    });

    it('sends nothing for a page with no rows: both buttons are off', async () => {
      fakeReports({
        pages: { R5: { ...R5_CREDIT_BUYERS, rows: [], total_count: 0 } },
        print: { ...PRINT_R5_EN, rows: [], summary: [] },
      });
      const user = userEvent.setup();
      await renderApp('/reports/R5');
      await screen.findByRole('heading', { level: 1 }, SLOW_MS);
      await screen.findByText(m('report-empty'), undefined, SLOW_MS);
      // The empty table has no print to preview: the preview button is on, and its page says so.
      await user.click(screen.getByRole('button', { name: named(m('report-preview')) }));
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('report-print-title')) },
        SLOW_MS,
      );
      await screen.findByText(m('report-print-empty'), undefined, SLOW_MS);
      expect(screen.getByRole('button', { name: named(m('report-print-print')) })).toBeDisabled();
      expect(screen.getByRole('button', { name: named(m('report-print-pdf')) })).toBeDisabled();
      expect(reportsApi.reportsPrintReport).not.toHaveBeenCalled();
    });
  });

  describe('the slips of R5', () => {
    it('Print and Save as PDF send the slips of all the credit buyers', async () => {
      fakeReports();
      const user = await openPreview('/reports/R5', 'report-print-all');
      await user.click(screen.getByRole('button', { name: named(m('report-print-pdf')) }));
      await waitFor(() => {
        expect(vi.mocked(reportsApi.reportsPrintReport).mock.calls[0]?.[0]).toMatchObject({
          report: { id: 'R5', language, paper: 'a4' },
          slips: true,
          to: 'pdf',
        });
      }, SLOW_MS);
      const notice = await screen.findByRole('status', undefined, SLOW_MS);
      expect(visible(notice.textContent)).toContain(`report-r5-slips-20261010-${language}.pdf`);
    });
  });

  describe('an analytics view', () => {
    it('Save as PDF and Print send the view with the language and paper of the frame', async () => {
      fakeReports();
      fakeAnalytics();
      const user = await openPreview('/analytics/K');
      await user.click(screen.getByRole('button', { name: named(m('report-print-pdf')) }));
      await waitFor(() => {
        expect(analyticsApi.analyticsPrintView).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(vi.mocked(analyticsApi.analyticsPrintView).mock.calls[0]?.[0]).toMatchObject({
        view: { view: 'K', language, paper: 'a4' },
        to: 'pdf',
        copies: null,
      });
      const notice = await screen.findByRole('status', undefined, SLOW_MS);
      expect(visible(notice.textContent)).toContain(`analytics-k-20261010-${language}.pdf`);
      await user.click(screen.getByRole('button', { name: named(m('report-print-print')) }));
      await waitFor(() => {
        expect(vi.mocked(analyticsApi.analyticsPrintView).mock.calls.at(-1)?.[0]).toMatchObject({
          to: 'printer',
        });
      }, SLOW_MS);
    });

    it('says in words that printing works only on Windows', async () => {
      fakeReports();
      fakeAnalytics({ printJob: () => PRINT_WINDOWS_ONLY });
      const user = await openPreview('/analytics/K');
      await user.click(screen.getByRole('button', { name: named(m('report-print-print')) }));
      const alert = await screen.findByRole('alert', undefined, SLOW_MS);
      expect(visible(alert.textContent)).toContain(m('err-print-windows-only'));
    });
  });
});
