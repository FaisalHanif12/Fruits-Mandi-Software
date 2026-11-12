// The statement print box on پارٹی کھاتے (plan 74 T3, PRD FR-12.47, FR-7.19, FR-7.36): the button "اسٹیٹمنٹ پرنٹ"
// with its key F9, the box with the period (the last 10 days, the last 30 days by default, the last 6 months,
// own dates), the print language and the paper, the preview of the page that is printed, Print and Save as
// PDF. Only dates as typed are sent: the backend reads them and builds every figure. The backend's answers are
// doubles of the plan's contract (tests/helpers/statement-print.ts). In the Urdu and the English app.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import { StatementPrintAction } from '../../src/features/party_ledgers';
import * as statementApi from '../../src/features/party_ledgers/print/statement-api';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { DAY, fail, press, resetClientState } from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';
import { printFrameFor } from '../helpers/print-frame-real';
import { statementAnswer } from '../helpers/statement-print';

vi.mock('../../src/app/api');
vi.mock('../../src/features/party_ledgers/print/statement-api');
vi.mock('../../src/features/reports/api');

/** Today in the doubles is 12-11-2026, so the periods can be read off the calendar. */
const TODAY = '2026-11-12';
const named = (text: string) => (name: string) => visible(name) === text;

function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}

function host() {
  return (
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <StatementPrintAction accountId={20} />
    </QueryClientProvider>
  );
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(appApi.dayStatus).mockResolvedValue({ ...DAY, today: TODAY });
  vi.mocked(statementApi.partyLedgersStatementPrintData).mockImplementation((input) =>
    Promise.resolve(statementAnswer(input)),
  );
  vi.mocked(reportsApi.reportsPrintHeader).mockImplementation((input) =>
    Promise.resolve(printFrameFor(input)),
  );
  vi.mocked(statementApi.partyLedgersStatementPrint).mockResolvedValue({
    printed: [],
    pdf_paths: null,
  });
});

afterEach(() => {
  document.documentElement.removeAttribute('style');
});

describe.each(LANGUAGES)('the statement print box (%s)', (language: Language) => {
  const m = (id: string) => message(language, id);

  beforeEach(() => {
    resetClientState(language);
  });

  async function open() {
    const view = renderIn(language, host());
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(m('statement-print-button')),
      }),
    );
    const box = await screen.findByRole('dialog');
    return { ...view, user, box };
  }
  const period = (box: HTMLElement, id: string) =>
    within(box).getByRole('button', { name: (name) => visible(name) === m(id) });
  const lastRequest = () =>
    vi.mocked(statementApi.partyLedgersStatementPrintData).mock.calls.at(-1)?.[0];

  it('the button shows its key F9, and the box starts on the last 30 days with the dates written out', async () => {
    renderIn(language, host());
    const button = screen.getByRole('button', {
      name: (name) => visible(name).startsWith(m('statement-print-button')),
    });
    expect(visible(button.textContent)).toContain('F9');
    const user = userEvent.setup();
    await user.click(button);
    const box = await screen.findByRole('dialog');
    expect(visible(box.textContent)).toContain(m('statement-print-title'));
    expect(period(box, 'statement-print-last-30')).toHaveAttribute('aria-pressed', 'true');
    // 30 days ending today, today included: 14-10-2026 to 12-11-2026.
    expect(visible(box.textContent)).toContain('14-10-2026');
    expect(visible(box.textContent)).toContain('12-11-2026');
    await waitFor(() => {
      expect(lastRequest()).toEqual({
        account_id: 20,
        from: '14-10-2026',
        to: '12-11-2026',
        language,
      });
    });
    // The page that will be printed is the preview: the party, the closing balance and the words.
    const page = await waitFor(() => must(box.querySelector<HTMLElement>('.print-doc')));
    expect(visible(page.textContent)).toContain('252,216');
  });

  it('keyboard only: F9 opens the box, Tab reaches Print, Enter prints, Esc closes', async () => {
    renderIn(language, host());
    const user = userEvent.setup();
    press('F9');
    const box = await screen.findByRole('dialog');
    // The box asks for today's date first; Print is there once the page is.
    const print = await within(box).findByRole('button', { name: named(m('report-print-print')) });
    await waitFor(() => {
      expect(print).toBeEnabled();
    });
    for (let step = 0; step < 40 && document.activeElement !== print; step += 1) await user.tab();
    expect(document.activeElement).toBe(print);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(statementApi.partyLedgersStatementPrint).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(statementApi.partyLedgersStatementPrint).mock.calls[0]?.[0]).toEqual({
      statement: { account_id: 20, from: '14-10-2026', to: '12-11-2026', language, paper: 'a4' },
      to: 'printer',
      copies: null,
    });
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it('the last 10 days and the last 6 months send their own dates', async () => {
    const { user, box } = await open();
    await user.click(period(box, 'statement-print-last-10'));
    await waitFor(() => {
      expect(lastRequest()).toMatchObject({ from: '03-11-2026', to: '12-11-2026' });
    });
    expect(period(box, 'statement-print-last-10')).toHaveAttribute('aria-pressed', 'true');
    await user.click(period(box, 'statement-print-last-6-months'));
    await waitFor(() => {
      expect(lastRequest()).toMatchObject({ from: '12-05-2026', to: '12-11-2026' });
    });
    await user.click(period(box, 'statement-print-last-30'));
    await waitFor(() => {
      expect(lastRequest()).toMatchObject({ from: '14-10-2026', to: '12-11-2026' });
    });
  });

  it('own dates start from the last 30 days, are sent as typed (shortcuts included) and wait until both are there', async () => {
    const { user, box } = await open();
    await user.click(period(box, 'statement-print-custom'));
    const from = within(box).getByRole('textbox', { name: named(m('statement-print-from')) });
    const to = within(box).getByRole('textbox', { name: named(m('statement-print-to')) });
    expect(from).toHaveValue('14-10-2026');
    expect(to).toHaveValue('12-11-2026');
    // Typing "10" and "10-9" is passed on as typed: the backend reads the shortcut (FR-7.19).
    await user.clear(from);
    await user.type(from, '10');
    await user.clear(to);
    await user.type(to, '10-9');
    await waitFor(() => {
      expect(lastRequest()).toMatchObject({ from: '10', to: '10-9' });
    });
    // One date missing: nothing is asked and no page is drawn.
    vi.mocked(statementApi.partyLedgersStatementPrintData).mockClear();
    await user.clear(to);
    await waitFor(() => {
      expect(box.querySelector('.print-doc')).toBeNull();
    });
    expect(statementApi.partyLedgersStatementPrintData).not.toHaveBeenCalled();
  });

  it('a date the backend refuses is told in words, with a retry', async () => {
    vi.mocked(statementApi.partyLedgersStatementPrintData).mockRejectedValueOnce(
      fail({ code: 'ERR_DATE_INVALID', field: 'from' }),
    );
    const { user, box } = await open();
    expect(await within(box).findByText(m('report-print-failed'))).toBeInTheDocument();
    expect(visible(box.textContent)).not.toContain('ERR_');
    await user.click(within(box).getByRole('button', { name: named(m('common-retry')) }));
    await waitFor(() => {
      expect(box.querySelector('.print-doc')).not.toBeNull();
    });
  });

  it('a period with no entry says so, and Print and Save as PDF are dimmed', async () => {
    const { user, box } = await open();
    await user.click(period(box, 'statement-print-custom'));
    const from = within(box).getByRole('textbox', { name: named(m('statement-print-from')) });
    await user.clear(from);
    await user.type(from, '01-01-2000');
    const to = within(box).getByRole('textbox', { name: named(m('statement-print-to')) });
    await user.clear(to);
    await user.type(to, '31-12-2000');
    expect(await within(box).findByText(m('report-print-empty'))).toBeInTheDocument();
    expect(box.querySelector('.print-doc')).toBeNull();
    expect(
      within(box).getByRole('button', { name: named(m('report-print-print')) }),
    ).toBeDisabled();
    expect(within(box).getByRole('button', { name: named(m('report-print-pdf')) })).toBeDisabled();
  });

  it('the page is loading while the answer has not come', async () => {
    vi.mocked(statementApi.partyLedgersStatementPrintData).mockReturnValue(
      new Promise(() => undefined),
    );
    const { box } = await open();
    expect(await within(box).findByRole('status')).toBeInTheDocument();
    expect(box.querySelector('.print-doc')).toBeNull();
    expect(
      within(box).getByRole('button', { name: named(m('report-print-print')) }),
    ).toBeDisabled();
  });

  it('the print language can differ from the app: the page is asked for again in it, Print and PDF send it with the paper', async () => {
    const other: Language = language === 'ur' ? 'en' : 'ur';
    vi.mocked(statementApi.partyLedgersStatementPrint).mockResolvedValue({
      printed: [],
      pdf_paths: [`statement-0020-${other}.pdf`],
    });
    const { user, box } = await open();
    await waitFor(() => {
      expect(box.querySelector('.print-doc')).not.toBeNull();
    });
    await user.click(
      within(box).getByRole('button', {
        name: named(message(language, `common-language-${other}`)),
      }),
    );
    await waitFor(() => {
      expect(lastRequest()).toMatchObject({ language: other });
    });
    await waitFor(() => {
      expect(box.querySelector('.print-doc')?.getAttribute('lang') ?? other).toBe(other);
    });
    await user.click(within(box).getByRole('button', { name: named(m('print-paper-a5')) }));
    await user.click(within(box).getByRole('button', { name: named(m('report-print-pdf')) }));
    await waitFor(() => {
      expect(statementApi.partyLedgersStatementPrint).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(statementApi.partyLedgersStatementPrint).mock.calls[0]?.[0]).toEqual({
      statement: {
        account_id: 20,
        from: '14-10-2026',
        to: '12-11-2026',
        language: other,
        paper: 'a5',
      },
      to: 'pdf',
      copies: null,
    });
    await waitFor(() => {
      expect(visible(box.textContent)).toContain(`statement-0020-${other}.pdf`);
    });
    // The app did not change language.
    expect(document.documentElement.lang).toBe('');
    expect(screen.getByTestId('language-root')).toHaveAttribute('lang', language);
  });

  it('a refusal of the print is told in words and the preview stays', async () => {
    vi.mocked(statementApi.partyLedgersStatementPrint).mockRejectedValue(
      fail({ code: 'ERR_PRINT_WINDOWS_ONLY' }),
    );
    const { user, box } = await open();
    await waitFor(() => {
      expect(
        within(box).getByRole('button', { name: named(m('report-print-print')) }),
      ).toBeEnabled();
    });
    await user.click(within(box).getByRole('button', { name: named(m('report-print-print')) }));
    const alert = await within(box).findByRole('alert');
    expect(visible(alert.textContent)).toContain(m('err-print-windows-only'));
    expect(visible(alert.textContent)).not.toContain('ERR_');
    expect(box.querySelector('.print-doc')).not.toBeNull();
  });

  it('Back closes the box, and the next opening starts again from the last 30 days', async () => {
    const { user, box } = await open();
    await user.click(period(box, 'statement-print-last-10'));
    await user.click(within(box).getByRole('button', { name: named(m('common-back')) }));
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(m('statement-print-button')),
      }),
    );
    const again = await screen.findByRole('dialog');
    expect(period(again, 'statement-print-last-30')).toHaveAttribute('aria-pressed', 'true');
  });
});
