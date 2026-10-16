// Closing and reopening days (plan 10, PRD FR-21.1, FR-21.2, FR-7.8), in the Urdu and in the
// English app, against a fake backend: the preview of what a close does, the confirm box that says
// what will happen, the refusals of the backend at their fields, the reopen with its required
// reason, and the status bar following the day. The screen works out no money and no date.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as dayApi from '../../src/features/day/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { balance } from '../helpers/accounts';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { fakeDay } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/day/api');

const named = (text: string) => (name: string) => visible(name) === text;

const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

const toastWith = (text: string) => (content: string) => visible(content) === text;

/** The words the status bar shows for the last closed day (the number markup of the message removed). */
const closedUpTo = (language: Language, date: string) =>
  message(language, 'layout-status-closed-up-to', { date }).replace(/<\/?num>/g, '');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the day screen (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open(options: Parameters<typeof fakeDay>[0] = {}) {
    const day = fakeDay(options);
    const app = await renderApp('/day');
    await screen.findByRole('heading', { level: 1, name: message(language, 'day-title') });
    return { ...app, day, user: userEvent.setup() };
  }

  const statusBar = () => visible(document.querySelector('footer')?.textContent);

  it('shows today, the last closed day and what closing would do, as the backend says it', async () => {
    await open();
    expect(visible(document.body.textContent)).toContain('30-09-2026');
    const list = (await screen.findByText(named(message(language, 'day-close-days')))).closest(
      'dl',
    );
    await waitFor(() => {
      expect(visible(list?.textContent)).toContain('58,484');
    });
    const words = visible(list?.textContent);
    expect(words).toContain('10');
    expect(words).toContain(message(language, 'day-close-trial'));
    expect(words).toContain(message(language, 'day-close-cash'));
    expect(words).toContain(message(language, 'common-side-dr'));
    expect(dayApi.dayClosePreview).toHaveBeenCalledWith('');
  });

  it('the preview follows the date typed, as the text typed', async () => {
    const { user } = await open();
    const upTo = await labelled(language, 'day-close-up-to');
    await user.clear(upTo);
    await user.type(upTo, '05-10-2026');
    await waitFor(() => {
      expect(dayApi.dayClosePreview).toHaveBeenLastCalledWith('05-10-2026');
    });
  });

  it('the date starts as the oldest open day the backend names, and an emptied field closes nothing', async () => {
    const { user } = await open();
    const upTo = await labelled(language, 'day-close-up-to');
    await waitFor(() => {
      expect(upTo.value).toBe('01-10-2026');
    });
    await user.clear(upTo);
    await user.type(upTo, '05-10-2026');
    expect(upTo.value).toBe('05-10-2026');
    await user.clear(upTo);
    expect(upTo.value).toBe('');
    expect(
      screen.getByRole('button', { name: named(message(language, 'day-close-button')) }),
    ).toBeDisabled();
  });

  it('shows the first open day, the days, the entries, the trial balance totals and the cash', async () => {
    await open();
    const list = (await screen.findByText(named(message(language, 'day-close-from')))).closest(
      'dl',
    );
    await waitFor(() => {
      expect(visible(list?.textContent)).toContain('85,048');
    });
    const words = visible(list?.textContent);
    expect(words).toContain('01-10-2026');
    expect(words).toContain(message(language, 'day-close-entries'));
    expect(words).toContain(message(language, 'day-close-trial-dr'));
    expect(words).toContain(message(language, 'day-close-trial-cr'));
    // The entries of the days, as the backend counts them.
    expect(words).toContain('12');
  });

  it('a backend that finds no open day says so at the date field and the button stays off', async () => {
    fakeDay();
    vi.mocked(dayApi.dayClosePreview).mockRejectedValueOnce(
      fail({ code: 'ERR_NOTHING_TO_CLOSE', field: 'up_to' }),
    );
    await renderApp('/day');
    const upTo = await labelled(language, 'day-close-up-to');
    await waitFor(() => {
      expect(upTo).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(document.body.textContent)).toContain(message(language, 'err-nothing-to-close'));
    expect(
      screen.getByRole('button', { name: named(message(language, 'day-close-button')) }),
    ).toBeDisabled();
  });

  it('with every day up to today closed there is nothing to ask or close', async () => {
    await open({ status: { today: '2026-10-10', closed_up_to: '2026-10-10', today_open: false } });
    expect(await screen.findByText(message(language, 'day-close-nothing'))).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: named(message(language, 'day-close-button')) }),
    ).toBeNull();
    expect(dayApi.dayClosePreview).not.toHaveBeenCalled();
  });

  it('a date the preview refuses shows at the date field with the date from the backend', async () => {
    const { user } = await open();
    vi.mocked(dayApi.dayClosePreview).mockRejectedValueOnce(
      fail({ code: 'ERR_DATE_CLOSED', field: 'up_to', values: { closed_up_to: '30-09-2026' } }),
    );
    const upTo = await labelled(language, 'day-close-up-to');
    await user.clear(upTo);
    await user.keyboard('15-09-2026');
    await waitFor(() => {
      expect(upTo).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(document.body.textContent)).toContain('30-09-2026');
    expect(
      screen.getByRole('button', { name: named(message(language, 'day-close-button')) }),
    ).toBeDisabled();
  });

  it('a close asks first, says exactly what will happen, then closes and the status bar follows', async () => {
    const { user } = await open();
    expect(statusBar()).toContain(message(language, 'layout-status-day-open'));
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'day-close-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain(
      message(language, 'day-close-confirm-description', { date: '01-10-2026', days: 10 }),
    );
    expect(dayApi.dayClose).not.toHaveBeenCalled();
    await user.click(
      within(dialog).getByRole('button', { name: named(message(language, 'day-close-button')) }),
    );
    await waitFor(() => {
      expect(dayApi.dayClose).toHaveBeenCalledWith('01-10-2026');
    });
    expect(
      (
        await screen.findAllByText(
          toastWith(message(language, 'day-closed', { date: '01-10-2026' })),
        )
      ).length,
    ).toBeGreaterThan(0);
    // The status bar says up to when. Today is still open: only the days before it are closed.
    await waitFor(() => {
      expect(statusBar()).toContain(closedUpTo(language, '01-10-2026'));
    });
    expect(statusBar()).toContain(message(language, 'layout-status-day-open'));
    expect(statusBar()).toContain(closedUpTo(language, '01-10-2026'));
  });

  it('by keyboard only: Enter on the close button asks, Esc backs out, Esc again leaves to the dashboard', async () => {
    const { user, router } = await open();
    const button = await screen.findByRole('button', {
      name: named(message(language, 'day-close-button')),
    });
    button.focus();
    await user.keyboard('{Enter}');
    await screen.findByRole('alertdialog');
    // Esc closes the question first and nothing is closed.
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(dayApi.dayClose).not.toHaveBeenCalled();
    expect(router.state.location.pathname).toBe('/day');
    // With nothing open, Esc leaves the screen (PRD FR-7.18).
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/');
    });
  });

  it('cancelling the confirm box closes nothing', async () => {
    const { user } = await open();
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'day-close-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.click(
      within(dialog).getByRole('button', { name: new RegExp(message(language, 'common-back')) }),
    );
    expect(dayApi.dayClose).not.toHaveBeenCalled();
  });

  it('a refused close shows the reason in the app language, and the date stays', async () => {
    const { user } = await open({ trial: balance(2, 'dr') });
    vi.mocked(dayApi.dayClose).mockRejectedValueOnce(fail({ code: 'ERR_JOURNAL_UNBALANCED' }));
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'day-close-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.click(
      within(dialog).getByRole('button', { name: named(message(language, 'day-close-button')) }),
    );
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-journal-unbalanced'));
    expect((await labelled(language, 'day-close-up-to')).value).toBe('01-10-2026');
    expect(statusBar()).toContain(message(language, 'layout-status-day-open'));
  });

  it('a date the backend refuses shows at the date field', async () => {
    const { user } = await open();
    vi.mocked(dayApi.dayClose).mockRejectedValueOnce(
      fail({ code: 'ERR_DATE_IN_FUTURE', field: 'up_to', values: { today: '10-10-2026' } }),
    );
    const upTo = await labelled(language, 'day-close-up-to');
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'day-close-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.click(
      within(dialog).getByRole('button', { name: named(message(language, 'day-close-button')) }),
    );
    await waitFor(() => {
      expect(upTo).toHaveAttribute('aria-invalid', 'true');
    });
  });

  it('nothing to close says so and the button is off', async () => {
    await open({ days: 0 });
    expect(await screen.findByText(message(language, 'day-close-nothing'))).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: named(message(language, 'day-close-button')) }),
    ).toBeDisabled();
  });

  it('a preview that cannot be read shows the message with a retry that works', async () => {
    fakeDay();
    vi.mocked(dayApi.dayClosePreview).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp('/day');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(await screen.findByText(named(message(language, 'day-close-days')))).toBeInTheDocument();
  });

  it('a reopen needs a reason, asks first, and the status bar follows', async () => {
    const { user } = await open();
    // The date starts as the last closed day.
    expect((await labelled(language, 'day-reopen-date')).value).toBe('30-09-2026');
    await user.click(await labelled(language, 'day-reopen-reason'));
    await user.keyboard('Wrong rate');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'day-reopen-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain(
      message(language, 'day-reopen-confirm-description', { date: '30-09-2026' }),
    );
    await user.click(
      within(dialog).getByRole('button', { name: named(message(language, 'day-reopen-button')) }),
    );
    await waitFor(() => {
      expect(dayApi.dayReopen).toHaveBeenCalledWith('30-09-2026', 'Wrong rate');
    });
    expect(
      (
        await screen.findAllByText(
          toastWith(message(language, 'day-reopened', { date: '30-09-2026' })),
        )
      ).length,
    ).toBeGreaterThan(0);
    await waitFor(() => {
      expect(statusBar()).toContain(closedUpTo(language, '29-09-2026'));
    });
  });

  it('a reopen without a reason is refused at the reason field, and nothing typed is lost', async () => {
    const { user } = await open();
    vi.mocked(dayApi.dayReopen).mockRejectedValueOnce(
      fail({ code: 'ERR_REQUIRED', field: 'reason' }),
    );
    const date = await labelled(language, 'day-reopen-date');
    await user.clear(date);
    await user.type(date, '28-09-2026');
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'day-reopen-button')) }),
    );
    const dialog = await screen.findByRole('alertdialog');
    await user.click(
      within(dialog).getByRole('button', { name: named(message(language, 'day-reopen-button')) }),
    );
    const reason = await labelled(language, 'day-reopen-reason');
    await waitFor(() => {
      expect(reason).toHaveAttribute('aria-invalid', 'true');
    });
    expect(date.value).toBe('28-09-2026');
    expect(statusBar()).toContain(closedUpTo(language, '30-09-2026'));
  });

  it('with no closed day there is nothing to reopen, and the status bar shows no closed-up-to date', async () => {
    fakeDay({ status: { today: '2026-10-10', closed_up_to: null, today_open: true } });
    await renderApp('/day');
    // The words are the same on the top line and on the reopen card: both are there.
    expect((await screen.findAllByText(message(language, 'day-reopen-none-title'))).length).toBe(2);
    expect(visible(document.querySelector('footer')?.textContent)).not.toContain(
      closedUpTo(language, '').trim(),
    );
    // The label and its value are two elements of the line, so the gap between them shows
    // ("Closed up to" and "No day is closed yet" never run together).
    const label = await screen.findByText(message(language, 'day-closed-up-to'));
    expect(visible(label.nextElementSibling?.textContent)).toBe(
      message(language, 'day-none-closed'),
    );
  });

  it('switching the language in the middle keeps the reason and the date typed', async () => {
    const { user } = await open();
    await user.click(await labelled(language, 'day-reopen-reason'));
    await user.keyboard('abc');
    const upTo = await labelled(language, 'day-close-up-to');
    await user.clear(upTo);
    await user.type(upTo, '05-10-2026');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'day-reopen-reason')).value).toBe('abc');
    expect((await labelled(other, 'day-close-up-to')).value).toBe('05-10-2026');
  });

  it('a failed read of the day shows the message with a retry that works', async () => {
    fakeDay();
    vi.mocked(appApi.dayStatus).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp('/day');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(
      await screen.findByRole('heading', { level: 1, name: message(language, 'day-title') }),
    ).toBeInTheDocument();
  });

  it('the day state in the status bar opens this screen', async () => {
    fakeDay();
    const { router } = await renderApp('/');
    await waitFor(() => {
      expect(visible(document.querySelector('footer')?.textContent)).toContain(
        message(language, 'layout-status-day-open'),
      );
    });
    await userEvent.setup().click(
      within(screen.getByRole('contentinfo')).getByRole('button', {
        name: named(message(language, 'layout-status-day-open')),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/day');
    });
  });
});
