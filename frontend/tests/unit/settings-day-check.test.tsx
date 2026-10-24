// Settings, plan 22 (D5 early track): the Day group (closing time and automatic close), the daily books
// check under it, and the number grouping in the Language group, in Urdu and in English, against a
// fake backend that answers with the backend's own fixtures. The time is sent as typed and the backend
// is the final word (a refusal shows under the box); the books check says each state in words; the
// grouping changes every amount at once and is kept.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as authApi from '../../src/features/auth/api';
import * as settingsApi from '../../src/features/settings/api';
import * as maintenance from '../../src/features/settings/maintenance-api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useGroupingStore } from '../../src/shared/format';
import * as urduApi from '../../src/shared/urdu/api';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  settings,
  signedIn,
} from '../helpers/app';
import {
  DAY_GET,
  DAY_SET,
  DAY_SET_REFUSED,
  DAY_SET_REQUEST,
  GROUPING_ANSWER,
  GROUPING_REQUEST,
  HEALTH_LAST,
  HEALTH_RUN,
} from '../helpers/b5-real';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/settings/maintenance-api');
vi.mock('../../src/shared/urdu/api');

const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

/** A button whose name is the message, followed by its key chip when it has one (F10). */
const keyed = (language: Language, id: string) => (name: string) =>
  visible(name).startsWith(message(language, id));

/** The element whose whole visible text is `expected`. Fluent wraps variables in isolation marks. */
const words = (expected: string) => (_content: string, element: Element | null) =>
  element?.tagName === 'SPAN' &&
  element.querySelector('span') === null &&
  visible(element.textContent) === expected;

/** The element that holds exactly this text, as one leaf (a field's problem, a notice). */
const leaf = (expected: string) => (_content: string, element: Element | null) =>
  element !== null && element.children.length === 0 && visible(element.textContent) === expected;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
  vi.mocked(maintenance.settingsDayGet).mockResolvedValue(DAY_GET);
  vi.mocked(maintenance.healthLast).mockResolvedValue(null);
  act(() => {
    useGroupingStore.getState().setGrouping('international');
  });
});

afterEach(() => {
  leaveAppWindow();
  act(() => {
    useGroupingStore.getState().setGrouping('international');
  });
});

describe.each(LANGUAGES)('Settings, Day and check (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  async function openTab(tabId: string) {
    const app = await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: m('settings-title') });
    const user = userEvent.setup();
    await user.click(screen.getByRole('tab', { name: named(language, tabId) }));
    return { ...app, user };
  }

  describe('the closing time and the automatic close', () => {
    it('shows what is saved, and says what the automatic close does and what off means', async () => {
      await openTab('settings-tab-day');
      const time = await screen.findByLabelText<HTMLInputElement>(
        named(language, 'settings-day-closing-time'),
      );
      expect(time.value).toBe('21:00');
      expect(
        screen.getByRole('checkbox', { name: m('settings-day-auto-close') }),
      ).not.toBeChecked();
      // It says what the switch does and, while it is off, what off means. It never says the
      // switch is only stored.
      expect(screen.getByText(m('settings-day-auto-close-help'))).toBeInTheDocument();
      expect(screen.getByText(m('settings-day-auto-close-off-note'))).toBeInTheDocument();
      // Nothing is changed yet, so there is nothing to save.
      expect(
        screen.getByRole('button', { name: keyed(language, 'layout-save-save') }),
      ).toBeDisabled();
    });

    it('says the day is closed by itself when the switch is on, and drops the off note', async () => {
      const { user } = await openTab('settings-tab-day');
      await user.click(await screen.findByRole('checkbox', { name: m('settings-day-auto-close') }));
      expect(screen.getByText(m('settings-day-auto-close-help'))).toBeInTheDocument();
      expect(screen.queryByText(m('settings-day-auto-close-off-note'))).not.toBeInTheDocument();
    });

    it('sends the time as typed and the tick, exactly as the backend wrote its request', async () => {
      vi.mocked(maintenance.settingsDaySet).mockResolvedValue(DAY_SET);
      const { user } = await openTab('settings-tab-day');
      const time = await screen.findByLabelText(named(language, 'settings-day-closing-time'));
      await user.clear(time);
      await user.type(time, '22:30');
      await user.click(screen.getByRole('checkbox', { name: m('settings-day-auto-close') }));
      press('F10');
      await waitFor(() => {
        expect(maintenance.settingsDaySet).toHaveBeenCalledWith(DAY_SET_REQUEST);
      });
      expect(await screen.findByText(m('settings-saved'))).toBeInTheDocument();
      // What the backend kept is what the form shows now, and it is saved: the button is off again.
      await waitFor(() => {
        expect(screen.getByRole('checkbox', { name: m('settings-day-auto-close') })).toBeChecked();
      });
      expect(
        screen.getByRole('button', { name: keyed(language, 'layout-save-save') }),
      ).toBeDisabled();
    });

    it('a time the backend refuses shows under the box with its range, and the typing stays', async () => {
      vi.mocked(maintenance.settingsDaySet).mockRejectedValue(fail(...DAY_SET_REFUSED.errors));
      const { user } = await openTab('settings-tab-day');
      const time = await screen.findByLabelText<HTMLInputElement>(
        named(language, 'settings-day-closing-time'),
      );
      await user.clear(time);
      await user.type(time, '25:00');
      press('F10');
      await waitFor(() => {
        expect(time).toHaveAttribute('aria-invalid', 'true');
      });
      expect(time.value).toBe('25:00');
      expect(
        screen.getByText(leaf(m('err-out-of-range', { min: '00:00', max: '23:59' }))),
      ).toBeInTheDocument();
    });

    it('the day settings fail to load: an error with a retry', async () => {
      vi.mocked(maintenance.settingsDayGet).mockRejectedValueOnce(fail({ code: 'ERR_INTERNAL' }));
      const { user } = await openTab('settings-tab-day');
      expect(await screen.findByText(m('settings-load-failed'))).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(language, 'common-retry') }));
      expect(
        await screen.findByLabelText(named(language, 'settings-day-closing-time')),
      ).toBeInTheDocument();
    });

    it('switching language in the middle of typing keeps the time and the tick', async () => {
      const { user } = await openTab('settings-tab-day');
      const time = await screen.findByLabelText(named(language, 'settings-day-closing-time'));
      await user.clear(time);
      await user.type(time, '20:45');
      await user.click(screen.getByRole('checkbox', { name: m('settings-day-auto-close') }));
      await user.keyboard('{Alt>}l{/Alt}');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      const again = await screen.findByLabelText<HTMLInputElement>(
        named(other, 'settings-day-closing-time'),
      );
      expect(again.value).toBe('20:45');
      expect(
        screen.getByRole('checkbox', { name: message(other, 'settings-day-auto-close') }),
      ).toBeChecked();
    });
  });

  describe('the books check', () => {
    it('before any check says so, and runs one on a press', async () => {
      vi.mocked(maintenance.healthRun).mockResolvedValue(HEALTH_RUN);
      const { user } = await openTab('settings-tab-day');
      expect(await screen.findByText(m('settings-check-none'))).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(language, 'settings-check-run') }));
      expect(await screen.findByText(m('settings-check-ok'))).toBeInTheDocument();
      expect(maintenance.healthRun).toHaveBeenCalledTimes(1);
      // The result of the run replaces "nothing yet".
      expect(screen.queryByText(m('settings-check-none'))).toBeNull();
    });

    it('the worked day is in order: balanced, nothing changed outside the app, data and backup', async () => {
      vi.mocked(maintenance.healthLast).mockResolvedValue({
        ...HEALTH_LAST,
        checked_at: '2026-10-23T09:05:00Z',
      });
      await openTab('settings-tab-day');
      expect(await screen.findByText(m('settings-check-ok'))).toBeInTheDocument();
      expect(
        screen.getByText(words(m('settings-check-books-balanced', { entries: 0 }))),
      ).toBeInTheDocument();
      expect(screen.getByText(words(m('settings-check-journal-ok')))).toBeInTheDocument();
      expect(screen.getByText(words(m('settings-check-audit-ok')))).toBeInTheDocument();
      expect(screen.getByText(words(m('settings-check-data-ok')))).toBeInTheDocument();
      // Cloud backup is not set up in the backend's fixture: said in words, not as a failure.
      expect(screen.getByText(words(m('settings-check-backup-off')))).toBeInTheDocument();
      expect(screen.getByText(/23-10-2026/)).toBeInTheDocument();
    });

    it('says each failure in words: the difference, the chains, the data file, the old backup', async () => {
      vi.mocked(maintenance.healthLast).mockResolvedValue({
        ...HEALTH_RUN,
        ok: false,
        books: {
          ...HEALTH_RUN.books,
          balanced: false,
          difference_paisa: paisaForTest(200_000),
          journal_chain: { ok: false, at: '12' },
          audit_chain: { ok: false, at: null },
        },
        data: { state: 'failed', reference: 'E-20261023-0007' },
        backup: {
          last_local_at: null,
          last_usb_at: null,
          last_cloud_at: '2026-10-20T09:00:00Z',
          cloud_age_hours: 60,
          cloud_level: 'alert',
          queue_len: 2,
          errors: 1,
        },
      });
      await openTab('settings-tab-day');
      expect(await screen.findByText(m('settings-check-bad'))).toBeInTheDocument();
      // The difference is the backend's amount, written by the money formatter.
      expect(
        screen.getByText(words(m('settings-check-books-difference', { difference: '2,000' }))),
      ).toBeInTheDocument();
      expect(
        screen.getByText(words(m('settings-check-journal-bad', { entry: '12' }))),
      ).toBeInTheDocument();
      expect(screen.getByText(words(m('settings-check-audit-bad-unknown')))).toBeInTheDocument();
      expect(screen.getByText(words(m('settings-check-data-failed')))).toBeInTheDocument();
      expect(screen.getByText(leaf('E-20261023-0007'))).toBeInTheDocument();
      expect(document.body.textContent).not.toMatch(/<(num|code)>/);
      expect(
        screen.getByText(words(m('settings-check-backup-alert', { hours: 60 }))),
      ).toBeInTheDocument();
      expect(
        screen.getByText(words(m('settings-check-backup-queue', { count: 2 }))),
      ).toBeInTheDocument();
      expect(
        screen.getByText(words(m('settings-check-backup-errors', { count: 1 }))),
      ).toBeInTheDocument();
    });

    it('a check that could not run is a failure in words, never a pass', async () => {
      vi.mocked(maintenance.healthLast).mockResolvedValue({
        ...HEALTH_RUN,
        ok: false,
        books: { ...HEALTH_RUN.books, ran: false, balanced: false },
        data: { state: 'checking', reference: null },
        backup: null,
      });
      await openTab('settings-tab-day');
      expect(await screen.findByText(m('settings-check-books-not-run'))).toBeInTheDocument();
      expect(screen.getByText(words(m('settings-check-data-checking')))).toBeInTheDocument();
      expect(screen.getByText(words(m('settings-check-backup-unknown')))).toBeInTheDocument();
      expect(screen.getByText(m('settings-check-bad'))).toBeInTheDocument();
    });

    it('an answer that cannot be read is an error with a retry, and a failed run says why', async () => {
      vi.mocked(maintenance.healthLast).mockRejectedValueOnce(fail({ code: 'ERR_INTERNAL' }));
      vi.mocked(maintenance.healthRun).mockRejectedValue(fail({ code: 'ERR_DATA_CHECK_FAILED' }));
      const { user } = await openTab('settings-tab-day');
      expect(await screen.findByText(m('settings-check-failed'))).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(language, 'common-retry') }));
      expect(await screen.findByText(m('settings-check-none'))).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(language, 'settings-check-run') }));
      expect(await screen.findByText(m('err-data-check-failed'))).toBeInTheDocument();
    });
  });

  describe('the number grouping', () => {
    it('shows the saved grouping, and a press changes every amount and is sent as the backend wrote it', async () => {
      vi.mocked(maintenance.settingsSetNumberGrouping).mockResolvedValue(GROUPING_ANSWER);
      const { user } = await openTab('settings-tab-language');
      const international = await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(m('settings-grouping-international').slice(0, 6)),
      });
      expect(international).toHaveAttribute('aria-pressed', 'true');
      // The two examples are numbers set apart, never raw markup.
      expect(document.body.textContent).not.toContain('<num>');
      expect(visible(international.textContent)).toContain('1,316,040');
      const lakh = screen.getByRole('button', {
        name: (name) => visible(name).startsWith(m('settings-grouping-lakh').slice(0, 4)),
      });
      expect(lakh).toHaveAttribute('aria-pressed', 'false');
      await user.click(lakh);
      await waitFor(() => {
        expect(maintenance.settingsSetNumberGrouping).toHaveBeenCalledWith(GROUPING_REQUEST);
      });
      // Every amount in the app follows at once, and the saved settings keep it.
      await waitFor(() => {
        expect(useGroupingStore.getState().grouping).toBe('lakh');
      });
      await waitFor(() => {
        expect(lakh).toHaveAttribute('aria-pressed', 'true');
      });
    });

    it('the saved grouping is the one in use from the first screen after sign-in', async () => {
      vi.mocked(settingsApi.settingsGet).mockResolvedValue(settings({ number_grouping: 'lakh' }));
      await renderApp('/');
      await waitFor(() => {
        expect(useGroupingStore.getState().grouping).toBe('lakh');
      });
      expect(appApi.dayStatus).toHaveBeenCalled();
      expect(authApi.authStatus).toHaveBeenCalled();
    });

    it('a grouping the backend refuses is told in words and nothing changes', async () => {
      vi.mocked(maintenance.settingsSetNumberGrouping).mockRejectedValue(
        fail({ code: 'ERR_INTERNAL' }),
      );
      const { user } = await openTab('settings-tab-language');
      const lakh = await screen.findByRole('button', {
        name: (name) => visible(name).startsWith(m('settings-grouping-lakh').slice(0, 4)),
      });
      await user.click(lakh);
      expect(await screen.findByText(m('err-internal'))).toBeInTheDocument();
      expect(useGroupingStore.getState().grouping).toBe('international');
    });
  });
});
