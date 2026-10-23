// The day close screen shows what the REAL backend answers (plan 17, F5, PRD FR-21.1, 4.9). The
// backend wrote the answer in its own test (tests/fixtures/day_close_preview_summary.json, copied,
// never edited). The screen is driven by that answer, in the Urdu and in the English app: the days,
// the entries, the trial balance and the cash, then the summary with the number of بکری, خام
// بکری, the commission, the shared income and each partner's income, expense and profit share. The
// screen works out no money: every figure on it is one the backend sent.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as dayApi from '../../src/features/day/api';
import { CommandError } from '../../src/shared/api';
import { LANGUAGES } from '../../src/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import * as real from '../helpers/day-close-real';
import { message, visible } from '../helpers/render';
import { fakeDay } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/day/api');

const named = (text: string) => (name: string) => visible(name) === text;

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the day close screen with the real answer (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open(answer = real.DAY_CLOSE_PREVIEW) {
    fakeDay({ status: { today: '2026-10-10', closed_up_to: '2026-10-09', today_open: true } });
    vi.mocked(dayApi.dayClosePreview).mockResolvedValue(answer);
    const app = await renderApp('/day');
    await screen.findByRole('heading', { level: 1, name: message(language, 'day-title') });
    const heading = await screen.findByRole('heading', {
      name: named(message(language, 'day-summary-title')),
    });
    const section = heading.closest('section');
    if (!(section instanceof HTMLElement)) throw new Error('no summary');
    return { ...app, section, user: userEvent.setup() };
  }

  it('asks for the oldest open day, with the empty date of the backend’s own request', async () => {
    await open();
    expect(dayApi.dayClosePreview).toHaveBeenCalledWith(real.REQUEST_DAY_CLOSE_PREVIEW.up_to);
  });

  it('shows the figures of the days: the days, the entries, the trial balance and the cash', async () => {
    await open();
    const list = (await screen.findByText(named(message(language, 'day-close-from')))).closest(
      'dl',
    );
    await waitFor(() => {
      expect(visible(list?.textContent)).toContain('76,564');
    });
    const words = visible(list?.textContent);
    // From 10-10-2026, 1 day, 4 entries, both trial totals the same, cash 42,000 (بنام).
    expect(words).toContain('10-10-2026');
    expect(words).toContain(message(language, 'day-close-entries'));
    expect(words.match(/76,564/g)).toHaveLength(2);
    expect(words).toContain('42,000');
    expect(words).toContain(message(language, 'common-side-dr'));
    // The books balance: the difference is 0, with no side word beside it.
    const difference = (
      await screen.findByText(named(message(language, 'day-close-trial')))
    ).closest('div');
    expect(visible(difference?.querySelector('dd')?.textContent)).toBe('0');
  });

  it('shows the summary the backend sent: one بکری, خام 29,400, commission 3,928, shared income 3,948', async () => {
    const { section } = await open();
    const facts = within(section).getAllByRole('definition');
    const text = facts.map((fact) => visible(fact.textContent));
    // The six facts, in the order of the summary, each as the backend's number.
    expect(text).toEqual(['1', '29,400', '3,928', '3,948', '0', '0']);
    const terms = within(section)
      .getAllByRole('term')
      .map((term) => visible(term.textContent));
    expect(terms).toEqual(
      [
        'day-summary-bikri-count',
        'day-summary-gross',
        'day-summary-commission',
        'day-summary-shared-income',
        'day-summary-open-lots',
        'day-summary-open-qty',
      ].map((id) => message(language, id)),
    );
  });

  it('shows each partner’s income, expense and profit share, which add up for the eye', async () => {
    const { section } = await open();
    const table = within(section).getByRole('table', {
      name: named(message(language, 'day-summary-partners-label')),
    });
    const rows = within(table).getAllByRole('row');
    expect(rows).toHaveLength(4);
    const cells = (row: HTMLElement | undefined) =>
      within(row ?? table)
        .getAllByRole('cell')
        .map((cell) => visible(cell.textContent));
    // The name is shown in the language of the app, then income, expense and profit.
    const names = language === 'ur' ? ['الف', 'ب', 'ج'] : ['Partner A', 'Partner B', 'Partner C'];
    expect(cells(rows[1])).toEqual([names[0], '1,580', '1,200', '380']);
    expect(cells(rows[2])).toEqual([names[1], '1,184', '900', '284']);
    expect(cells(rows[3])).toEqual([names[2], '1,184', '900', '284']);
  });

  it('the close button is on, and its confirm box names the date and the one day', async () => {
    const { user } = await open();
    const button = await screen.findByRole('button', {
      name: named(message(language, 'day-close-button')),
    });
    await waitFor(() => {
      expect(button).toBeEnabled();
    });
    await user.click(button);
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain('10-10-2026');
  });

  it('shows the whole worked day of PRD 4.9: 2 بکری, 41,400, 5,448, 5,488, shares 996 / 746 / 746, cash 58,484', async () => {
    const { section } = await open(real.DAY_CLOSE_PREVIEW_WORKED_DAY);
    const facts = within(section)
      .getAllByRole('definition')
      .map((fact) => visible(fact.textContent));
    expect(facts).toEqual(['2', '41,400', '5,448', '5,488', '0', '0']);
    const table = within(section).getByRole('table', {
      name: named(message(language, 'day-summary-partners-label')),
    });
    const rows = within(table).getAllByRole('row');
    const cells = (row: HTMLElement | undefined) =>
      within(row ?? table)
        .getAllByRole('cell')
        .map((cell) => visible(cell.textContent));
    // Income, expense and profit of each partner: they add up for the eye (PRD C12).
    expect(cells(rows[1]).slice(1)).toEqual(['2,196', '1,200', '996']);
    expect(cells(rows[2]).slice(1)).toEqual(['1,646', '900', '746']);
    expect(cells(rows[3]).slice(1)).toEqual(['1,646', '900', '746']);
    // The figures of the days: 85,048 on both sides of the trial balance, difference 0, cash 58,484.
    const list = (await screen.findByText(named(message(language, 'day-close-from')))).closest(
      'dl',
    );
    await waitFor(() => {
      expect(visible(list?.textContent)).toContain('85,048');
    });
    const words = visible(list?.textContent);
    expect(words.match(/85,048/g)).toHaveLength(2);
    expect(words).toContain('58,484');
    const difference = (
      await screen.findByText(named(message(language, 'day-close-trial')))
    ).closest('div');
    expect(visible(difference?.querySelector('dd')?.textContent)).toBe('0');
    // No stock left, so no reminder.
    expect(within(section).queryByRole('status')).toBeNull();
  });

  it('with 5 نگ left (E2E-16) the summary shows the lots, the نگ and the reminder, and the day still closes', async () => {
    const { section, user } = await open(real.DAY_CLOSE_PREVIEW_STOCK_LEFT);
    // The date typed goes to the backend as the text typed, as in the backend's own request.
    const upTo = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'day-close-up-to')),
    );
    await user.clear(upTo);
    await user.type(upTo, real.REQUEST_DAY_CLOSE_PREVIEW_STOCK_LEFT.up_to);
    await waitFor(() => {
      expect(dayApi.dayClosePreview).toHaveBeenLastCalledWith(
        real.REQUEST_DAY_CLOSE_PREVIEW_STOCK_LEFT.up_to,
      );
    });
    const facts = within(section)
      .getAllByRole('definition')
      .map((fact) => visible(fact.textContent));
    // 1 بکری, خام 26,500, commission 3,540, shared income 3,560, 1 lot and 5 نگ left.
    expect(facts).toEqual(['1', '26,500', '3,540', '3,560', '1', '5']);
    // The reminder says it with the backend's numbers, as a note and not as an error.
    const note = within(section).getByRole('status');
    expect(visible(note.textContent)).toBe(
      message(language, 'day-summary-stock-left-note', { lots: 1, qty: 5 }),
    );
    // The partners' shares are the backend's: 1,424 / 1,068 / 1,068 with no expense.
    const table = within(section).getByRole('table', {
      name: named(message(language, 'day-summary-partners-label')),
    });
    const rows = within(table).getAllByRole('row');
    expect(visible(rows[1]?.textContent)).toContain('1,424');
    expect(visible(rows[2]?.textContent)).toContain('1,068');
    // Stock left never blocks the close.
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: named(message(language, 'day-close-button')) }),
      ).toBeEnabled();
    });
  });

  it('with no stock left there is no reminder', async () => {
    const { section } = await open();
    expect(within(section).queryByRole('status')).toBeNull();
  });

  it('a day that is closed already is told at the date field with the backend’s words, and nothing is closed', async () => {
    fakeDay();
    vi.mocked(dayApi.dayClosePreview).mockRejectedValue(new CommandError(real.REFUSED_DAY_CLOSED));
    await renderApp('/day');
    const upTo = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'day-close-up-to')),
    );
    await waitFor(() => {
      expect(upTo).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'err-date-closed', { closed_up_to: '10-10-2026' }),
    );
    expect(
      screen.getByRole('button', { name: named(message(language, 'day-close-button')) }),
    ).toBeDisabled();
    expect(dayApi.dayClose).not.toHaveBeenCalled();
  });
});
