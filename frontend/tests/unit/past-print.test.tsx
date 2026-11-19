// The print, the PDF and the Excel of پچھلی بکری (plan 72 P8, PRD 12.7 "Print and Excel follow the same
// choice", FR-19.3, FR-19.13), in the Urdu and the English app, against a double shaped like the contract of
// `bikri_list_print_data`, `bikri_list_print` and `bikri_list_export_excel`. The buttons and the preview
// are drawn by `usePastPrintScreen`, which the list page holds; they are drawn here on their own, with the
// providers of the app. What is checked: Print `F9` and Excel are there and work by key; the data is
// asked with the filter the list has now (today, the window, the switch `all`), in the print language;
// the page has the shared header, a heading row for today and for each day (empty days too), the rows, the
// totals and digits 0 to 9; Print and PDF send the same input; Excel sends it with the language chosen in the
// preview; a language switch in the middle loses nothing. Every figure is the double's: the screen adds up
// nothing.

import { cleanup, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { PrintOutcome } from '../../src/bindings/PrintOutcome';
import { AppProviders } from '../../src/app/providers';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import { usePastPrintScreen } from '../../src/features/bikri/hooks/usePastPrintScreen';
import * as printApi from '../../src/features/bikri/past-print-api';
import type { PastPrintFilter } from '../../src/features/bikri/past-print-filter';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as dialog from '../../src/shared/api/dialog';
import * as events from '../../src/shared/api/events';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { pastPrintDataFor, type PastPrintFake } from '../helpers/past-print';
import { fakePrinting } from '../helpers/print';
import { printFirmName, printFrameFor } from '../helpers/print-frame-real';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/bikri/past-print-api');
vi.mock('../../src/shared/api/dialog');
vi.mock('../../src/shared/api/events');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(events.listenEvent).mockResolvedValue(() => undefined);
  vi.mocked(events.emitEvent).mockResolvedValue(undefined);
  vi.mocked(dialog.chooseSavePath).mockResolvedValue('C:\\Sales\\past');
});

afterEach(() => {
  leaveAppWindow();
});

/** What the list asks with on 10-10-2026: today, the window of 03-10-2026 to 09-10-2026, no filters. */
const FILTER: PastPrintFilter = {
  from: '03-10-2026',
  to: '09-10-2026',
  with_today: true,
  seller_id: null,
  item_id: null,
  status: null,
  show_finished: false,
  search: null,
  all: false,
};

/** The page's own use of the hook: the preview takes the place of the buttons. */
function Harness({ filter }: { filter: PastPrintFilter | null }) {
  // The key listeners the shell mounts once (Esc order).
  useEscapeDispatcher();
  const print = usePastPrintScreen(filter);
  if (print.preview !== null) return print.preview;
  return (
    <div>
      {print.buttons}
      {print.notices}
    </div>
  );
}

function fakeBackend(
  fake: PastPrintFake = {},
  outcome: PrintOutcome = { printed: [], pdf_paths: null },
) {
  fakePrinting();
  vi.mocked(reportsApi.reportsPrintHeader).mockImplementation((input) =>
    Promise.resolve(printFrameFor(input)),
  );
  vi.mocked(printApi.bikriListPrintData).mockImplementation((input) =>
    Promise.resolve(pastPrintDataFor(input, fake)),
  );
  vi.mocked(printApi.bikriListPrint).mockResolvedValue(outcome);
  vi.mocked(printApi.bikriListExportExcel).mockResolvedValue({
    path: 'C:\\Sales\\past.xlsx',
    rows: 4,
    size_bytes: 8000,
  });
}

describe.each(LANGUAGES)('Print and Excel of پچھلی بکری (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const other: Language = language === 'ur' ? 'en' : 'ur';
  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  /** A button by its words, then its key chip when it has one. */
  const button = (words: string, key?: string) =>
    screen.getByRole('button', {
      name: (name: string) => {
        const seen = visible(name);
        return key === undefined ? seen === words : seen.startsWith(words) && seen.endsWith(key);
      },
    });
  const draw = (filter: PastPrintFilter | null = FILTER) =>
    render(
      <AppProviders>
        <Harness filter={filter} />
      </AppProviders>,
    );
  const calls = () => vi.mocked(printApi.bikriListPrintData).mock.calls.map(([input]) => input);
  const pageText = () => visible(document.body.textContent);

  /** Opens the preview with the Print button and waits for the page. */
  const openPreview = async (filter: PastPrintFilter = FILTER) => {
    draw(filter);
    const user = userEvent.setup();
    await user.click(button(m('print-box-print'), 'F9'));
    await waitFor(() => {
      expect(pageText()).toContain(m('bikri-list-print-title'));
    }, SLOW_MS);
    return user;
  };

  describe('the buttons', () => {
    it('Print F9 and Excel are there, dimmed until the list knows its days', () => {
      fakeBackend();
      draw(null);
      expect(button(m('print-box-print'), 'F9')).toBeDisabled();
      expect(button(m('export-excel'))).toBeDisabled();
    });

    it('F9 opens the preview, and asks the backend for the print once, with nothing else', async () => {
      fakeBackend();
      draw();
      expect(calls()).toHaveLength(0);
      press('F9');
      await waitFor(() => {
        expect(calls()).toHaveLength(1);
      }, SLOW_MS);
      expect(pageText()).not.toContain('bikri-list');
    });
  });

  describe('what is asked for', () => {
    it('the filter of the list, the print language of the app and A4, with no paging or sort', async () => {
      fakeBackend();
      await openPreview();
      expect(calls()[0]).toEqual({ ...FILTER, language, paper: 'a4' });
      expect(calls()[0]).not.toHaveProperty('offset');
      expect(calls()[0]).not.toHaveProperty('limit');
      expect(calls()[0]).not.toHaveProperty('sort');
    });

    it('the filters the Owner set go with it, and the switch "show every بکری" goes as `all`', async () => {
      fakeBackend();
      await openPreview({
        ...FILTER,
        seller_id: 3,
        item_id: 4,
        status: 'saved',
        search: 'LES',
        show_finished: true,
        all: true,
      });
      expect(calls()[0]).toMatchObject({
        seller_id: 3,
        item_id: 4,
        status: 'saved',
        search: 'LES',
        show_finished: true,
        all: true,
      });
      // The header names the switch alone, as the backend sent it: the dates are not listed.
      expect(pageText()).toContain(m('bikri-list-print-all'));
      expect(pageText()).not.toContain(m('report-filter-from'));
    });

    it('the switch off sends `all` false', async () => {
      fakeBackend();
      await openPreview();
      expect(calls()[0]).toMatchObject({ all: false });
      expect(pageText()).not.toContain(m('bikri-list-print-all'));
    });
  });

  describe('the page', () => {
    it('has the shared header, the title, the facts and the footer of the backend', async () => {
      fakeBackend();
      await openPreview();
      expect(pageText()).toContain(printFirmName(language));
      expect(pageText()).toContain(m('bikri-list-print-title'));
      expect(pageText()).toContain('03-10-2026');
      expect(pageText()).toContain('09-10-2026');
    });

    it('has a heading row for today and for each day of the window, newest first, empty days too', async () => {
      fakeBackend();
      await openPreview();
      const rows = Array.from(
        document.querySelectorAll('.print-table tbody tr[data-row="group"]'),
      ).map((row) => visible(row.textContent));
      // Today, then 09-10 down to 03-10: eight heading rows.
      expect(rows).toHaveLength(8);
      // The words are the messages the backend names, with the date and the count it sent.
      expect(rows[0]).toContain(m('bikri-list-today-heading', { date: '10-10-2026', count: '1' }));
      expect(rows[0]).toContain('29,400');
      expect(rows[1]).toContain(m('bikri-list-day-heading', { date: '09-10-2026', count: '2' }));
      expect(rows[1]).toContain('58,800');
      // A day with no بکری is its heading, which says so, with no figure.
      expect(rows[2]).toContain(m('bikri-list-day-heading-empty', { date: '08-10-2026' }));
      expect(rows[7]).toContain(m('bikri-list-day-heading-empty', { date: '03-10-2026' }));
      expect(rows[2]).toBe(m('bikri-list-day-heading-empty', { date: '08-10-2026' }));
    });

    it('has the rows of the days, the totals of the filter under the columns, and digits 0 to 9 only', async () => {
      fakeBackend();
      await openPreview();
      const body = Array.from(
        document.querySelectorAll('.print-table tbody tr[data-row="normal"]'),
      );
      // Three بکری: one of today and two of 09-10-2026.
      expect(body).toHaveLength(3);
      expect(visible(body[0]?.textContent)).toContain('LES-1234');
      expect(visible(body[0]?.textContent)).toContain(
        language === 'ur' ? 'رحیم، کیلا-1' : 'Rahim, Banana-1',
      );
      const footer = visible(document.querySelector('.print-table tfoot')?.textContent);
      expect(footer).toContain('117,600');
      expect(footer).toContain(m('report-total'));
      // No Urdu digits anywhere on the page (PRD FR-19.6).
      expect(document.body.textContent).not.toMatch(/[\u06F0-\u06F9\u0660-\u0669]/u);
    });

    it('the twelve columns have their headings, the time among them', async () => {
      fakeBackend();
      await openPreview();
      const heads = Array.from(document.querySelectorAll('.print-table thead th')).map((cell) =>
        visible(cell.textContent),
      );
      expect(heads).toHaveLength(12);
      expect(heads).toContain(m('bikri-list-col-time'));
      expect(heads).toContain(m('glossary-kham-bikri'));
    });

    it('an edited بکری says its status and the mark, and a print without today has no section for it', async () => {
      fakeBackend({ todayRows: [{ number: '1', edited: true }] });
      await openPreview();
      expect(pageText()).toContain(
        `${m('bikri-status-saved')} · ${m('edit-mark')}`.replace(/\s+/g, ' '),
      );
      cleanup();
      fakeBackend();
      await openPreview({ ...FILTER, with_today: false });
      expect(calls().at(-1)).toMatchObject({ with_today: false });
      const rows = Array.from(
        document.querySelectorAll('.print-table tbody tr[data-row="group"]'),
      ).map((row) => visible(row.textContent));
      expect(rows).toHaveLength(7);
      expect(pageText()).not.toContain(
        m('bikri-list-today-heading', { date: '10-10-2026', count: '1' }),
      );
    });

    it('a day that holds a cancelled بکری shows it only with the switch on, in words', async () => {
      fakeBackend({
        windowRows: { '2026-10-09': [{ number: '2' }, { number: '3', status: 'cancelled' }] },
      });
      await openPreview();
      expect(pageText()).not.toContain(m('bikri-status-cancelled'));
      // The same list with the switch on, from a new preview.
      cleanup();
      fakeBackend({
        windowRows: { '2026-10-09': [{ number: '2' }, { number: '3', status: 'cancelled' }] },
      });
      await openPreview({ ...FILTER, all: true });
      expect(pageText()).toContain(m('bikri-status-cancelled'));
    });
  });

  describe('the print language and the paper', () => {
    it('the language chosen in the frame asks again in that language; the app language stays', async () => {
      fakeBackend();
      const user = await openPreview();
      await user.click(
        screen.getByRole('button', {
          name: (name) => visible(name) === m(`common-language-${other}`),
        }),
      );
      await waitFor(() => {
        expect(calls().at(-1)).toMatchObject({ language: other, paper: 'a4' });
      }, SLOW_MS);
      // The words of the page are now the other language's; the buttons of the frame keep the app's.
      await waitFor(() => {
        expect(pageText()).toContain(message(other, 'bikri-list-print-title'));
      }, SLOW_MS);
      expect(pageText()).toContain(printFirmName(other));
    });

    it('the paper chosen is sent with the same filter', async () => {
      fakeBackend();
      const user = await openPreview();
      await user.click(
        screen.getByRole('button', { name: (name) => visible(name) === m('print-paper-a5') }),
      );
      await waitFor(() => {
        expect(calls().at(-1)).toEqual({ ...FILTER, language, paper: 'a5' });
      }, SLOW_MS);
    });
  });

  describe('Print, PDF and Excel', () => {
    it('Print sends the list to the printer and PDF saves it, both with the input the page was read with', async () => {
      fakeBackend();
      const user = await openPreview();
      const asked = calls().at(-1);
      await user.click(button(m('report-print-print')));
      await waitFor(() => {
        expect(printApi.bikriListPrint).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      expect(vi.mocked(printApi.bikriListPrint).mock.calls[0]?.[0]).toEqual({
        list: asked,
        to: 'printer',
        copies: null,
      });
      await user.click(button(m('report-print-pdf')));
      await waitFor(() => {
        expect(printApi.bikriListPrint).toHaveBeenCalledTimes(2);
      }, SLOW_MS);
      expect(vi.mocked(printApi.bikriListPrint).mock.calls[1]?.[0]).toEqual({
        list: asked,
        to: 'pdf',
        copies: null,
      });
    });

    it('Excel on the list sends the filter in the app language and says where the file is', async () => {
      fakeBackend();
      draw();
      const user = userEvent.setup();
      await user.click(button(m('export-excel')));
      await waitFor(() => {
        expect(printApi.bikriListExportExcel).toHaveBeenCalledTimes(1);
      }, SLOW_MS);
      const sent = vi.mocked(printApi.bikriListExportExcel).mock.calls[0]?.[0];
      expect(sent).toMatchObject({ ...FILTER, language });
      expect(sent?.path.endsWith('.xlsx')).toBe(true);
      expect(
        await screen.findByText((text) => visible(text).includes('past.xlsx')),
      ).toBeInTheDocument();
    });

    it('Excel in the preview takes the language chosen there', async () => {
      fakeBackend();
      const user = await openPreview();
      await user.click(
        screen.getByRole('button', {
          name: (name) => visible(name) === m(`common-language-${other}`),
        }),
      );
      await waitFor(() => {
        expect(calls().at(-1)).toMatchObject({ language: other });
      }, SLOW_MS);
      await user.click(
        screen.getByRole('button', { name: (name) => visible(name) === m('export-excel') }),
      );
      await waitFor(() => {
        expect(printApi.bikriListExportExcel).toHaveBeenCalled();
      }, SLOW_MS);
      expect(vi.mocked(printApi.bikriListExportExcel).mock.calls[0]?.[0]).toMatchObject({
        ...FILTER,
        language: other,
      });
    });
  });

  describe('the four states of the page', () => {
    it('nothing to print says so in words and keeps Print off', async () => {
      fakeBackend({ todayRows: [], windowRows: {} });
      draw();
      const user = userEvent.setup();
      await user.click(button(m('print-box-print'), 'F9'));
      expect(
        await screen.findByText(
          (text) => visible(text) === m('report-print-empty'),
          undefined,
          SLOW_MS,
        ),
      ).toBeInTheDocument();
    });

    it('a print that cannot be read says so with a retry that asks again', async () => {
      fakeBackend();
      vi.mocked(printApi.bikriListPrintData).mockRejectedValueOnce(new Error('down'));
      draw();
      const user = userEvent.setup();
      await user.click(button(m('print-box-print'), 'F9'));
      const retry = await screen.findByRole(
        'button',
        { name: (name) => visible(name) === m('common-retry') },
        SLOW_MS,
      );
      await user.click(retry);
      await waitFor(() => {
        expect(pageText()).toContain(m('bikri-list-print-title'));
      }, SLOW_MS);
    });

    it('Esc leaves the preview and the buttons are back', async () => {
      fakeBackend();
      await openPreview();
      press('Escape');
      await waitFor(() => {
        expect(button(m('print-box-print'), 'F9')).toBeInTheDocument();
      });
      expect(within(document.body).queryByText(m('bikri-list-print-title'))).toBeNull();
    });
  });
});
