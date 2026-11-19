// The month close screen (round 7, PRD FR-17.6, FR-21.2, FR-7.8, FR-7.18), in the Urdu and in the
// English app, against a fake backend that answers with the backend's own fixtures. The list of
// months, the preview of the month that is open (the backend's own plan: income, shop expenses,
// profit and every line), the close with its confirm box, the reversal with its reason box, every
// refusal in the month screen's own words, the four states, a language switch in the middle of a
// month and the keyboard-only path are each proved. The screen adds up nothing: every figure is one
// the backend sent.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as monthApi from '../../src/features/month/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { useLayoutStep, type LayoutStep } from '../../src/shared/hooks/useLayoutStep';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  MONTH_LIST_READY,
  MONTH_PREVIEW,
  MONTH_PREVIEW_BAD_MONTH,
  MONTH_REVERSE_NOT_IN_FORCE,
  MONTH_REVERSE_NO_REASON,
  MONTH_SAVE_DAYS_OPEN,
  MONTH_SAVE_NOTHING_TO_CLOSE,
  MONTH_SAVE_TWICE,
  fakeMonth,
  withState,
} from '../helpers/month';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/month/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
// The window of a test has no layout, so the step is the middle one.
vi.mock('../../src/shared/hooks/useLayoutStep', async (importOriginal) => {
  const original = await importOriginal<typeof import('../../src/shared/hooks/useLayoutStep')>();
  return { ...original, useLayoutStep: vi.fn((): LayoutStep => 'normal') };
});

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(useLayoutStep).mockReturnValue('normal');
  healthyBackend();
  fakePartners([]);
});

afterEach(() => {
  leaveAppWindow();
});

const KEY_CHIPS = new Set(['F2', 'F9', 'F10', 'Enter', 'Esc']);

/** Matches a name that is exactly `text`, or `text` followed by the key chip of its button. */
const named = (text: string) => (name: string) => {
  const seen = visible(name);
  if (seen === text) return true;
  return seen.startsWith(text) && KEY_CHIPS.has(seen.slice(text.length).trim());
};

function cell(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function drawnRows(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map(
    (row) => row.getAttribute('row-id') ?? '',
  );
}

/** The value written under a label of the fact lists (`dt` then `dd`). */
function fact(label: string): string {
  const term = Array.from(document.querySelectorAll('dt')).find(
    (candidate) => visible(candidate.textContent) === label,
  );
  return visible(term?.nextElementSibling?.textContent);
}

/** The cells of the rows of the table of lines, and of its total row under them. */
function cellsOf(selector: string): string[][] {
  return Array.from(document.querySelectorAll(selector)).map((row) =>
    Array.from(row.querySelectorAll('td')).map((td) => visible(td.textContent)),
  );
}
const lines = () => cellsOf('table tbody tr');
const totalRow = () => cellsOf('table tfoot tr')[0];

function refusedWith(refusal: typeof MONTH_SAVE_TWICE) {
  return () => refusal;
}

describe.each(LANGUAGES)('the month close screen (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const names = (ur: string, en: string) => (language === 'ur' ? ur : en);
  const other: Language = language === 'ur' ? 'en' : 'ur';
  /** "October 2026" in the app language. */
  const label = (key: string, inLanguage: Language = language) => {
    const [year, month] = key.split('-');
    return `${message(inLanguage, `report-month-${String(Number(month))}`)} ${String(year)}`;
  };
  const OCTOBER = label('2026-10');
  const SEPTEMBER = label('2026-09');
  const NOVEMBER = label('2026-11');

  async function openScreen(options: Parameters<typeof fakeMonth>[0] = {}, path = '/month') {
    const fake = fakeMonth(options);
    const view = await renderApp(path);
    await screen.findByRole('heading', { level: 1, name: named(m('month-title')) });
    return { ...view, fake, user: userEvent.setup() };
  }

  /** Opens a month of the list: a click on its row and Enter, as the keyboard does. */
  async function openMonth(user: ReturnType<typeof userEvent.setup>, key: string) {
    await waitFor(() => {
      expect(document.querySelector(`.ag-row[row-id="${key}"]`)).not.toBeNull();
    }, SLOW_MS);
    const target = document.querySelector(`.ag-row[row-id="${key}"] [col-id="state"]`);
    if (target === null) throw new Error(`no row ${key}`);
    await user.click(target);
    await user.keyboard('{Enter}');
  }

  describe('the list of months', () => {
    it('lists the months newest first, with where each stands', async () => {
      await openScreen();
      await waitFor(() => {
        expect(drawnRows()).toEqual(['2026-11', '2026-10', '2026-09', '2026-08']);
      }, SLOW_MS);
      expect(cell('2026-11', 'month')).toBe(NOVEMBER);
      expect(cell('2026-11', 'state')).toBe(m('month-state-not-finished'));
      expect(cell('2026-10', 'state')).toBe(m('month-state-ready'));
      expect(cell('2026-10', 'closed_on')).toBe('');
    });

    it('shows the day a closed month was closed', async () => {
      await openScreen({ closed: true });
      await waitFor(() => {
        expect(cell('2026-10', 'state')).toBe(m('month-state-closed'));
      }, SLOW_MS);
      expect(cell('2026-10', 'closed_on')).toBe('02-11-2026');
    });
  });

  describe('a month that can be closed', () => {
    it('opens on the newest month that is ready, with what closing it posts', async () => {
      await openScreen();
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: OCTOBER })) },
        SLOW_MS,
      );
      expect(monthApi.monthClosePreview).toHaveBeenCalledWith('2026-10');
      // The income, the shop expenses and the profit are the backend's figures (E2E-10).
      await waitFor(() => {
        expect(fact(m('month-fact-income'))).toBe('5,488');
      }, SLOW_MS);
      expect(fact(m('month-fact-expense'))).toBe('3,000');
      expect(fact(m('month-fact-profit'))).toBe('2,488');
      // The entry is dated today, the day the backend says.
      expect(fact(m('month-fact-date'))).toBe('02-11-2026');
    });

    it('draws every line of the entry with its debit and credit, and the backend’s total', async () => {
      await openScreen();
      await waitFor(() => {
        expect(lines()).toHaveLength(MONTH_PREVIEW.lines.length);
      }, SLOW_MS);
      const rows = lines();
      // Four income accounts are debited, the shop expense and the three partners are credited.
      expect(rows[0]).toEqual([
        expect.stringContaining('04010001'),
        m('month-kind-income'),
        '2,484',
        '',
      ]);
      const expense = rows.find((row) => row[1] === m('month-kind-expense'));
      expect(expense?.[0]).toContain('05010001');
      expect(expense?.slice(2)).toEqual(['', '3,000']);
      const capital = rows.filter((row) => row[1] === m('month-kind-partner-capital'));
      expect(capital.map((row) => row[3])).toEqual(['996', '746', '746']);
      expect(capital[0]?.[0]).toContain(names('الف', 'Partner A'));
      // The row under the lines is the backend's total, the same under the debit and the credit.
      expect(totalRow()).toEqual([m('month-total'), '', '5,488', '5,488']);
    });

    it('opens another month with Enter on its row and asks the backend for that month', async () => {
      const { user } = await openScreen();
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: OCTOBER })) },
        SLOW_MS,
      );
      await openMonth(user, '2026-09');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: SEPTEMBER })) },
        SLOW_MS,
      );
      expect(vi.mocked(monthApi.monthClosePreview).mock.calls.at(-1)?.[0]).toBe('2026-09');
    });

    it('asks first, with Back in focus, and posts nothing when the box is left with Esc', async () => {
      const { user } = await openScreen();
      await user.click(
        await screen.findByRole('button', { name: named(m('month-close-button')) }, SLOW_MS),
      );
      const dialog = await screen.findByRole('alertdialog');
      expect(visible(dialog.textContent)).toContain(
        m('month-close-confirm-description', { month: OCTOBER, date: '02-11-2026' }),
      );
      // A stray Enter must not close a month: the focus starts on Back.
      expect(within(dialog).getByRole('button', { name: named(m('common-back')) })).toHaveFocus();
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      }, SLOW_MS);
      expect(monthApi.monthCloseSave).not.toHaveBeenCalled();
    });

    it('closes the month when the box is confirmed, says so, and shows the month as closed', async () => {
      const { user } = await openScreen();
      await user.click(
        await screen.findByRole('button', { name: named(m('month-close-button')) }, SLOW_MS),
      );
      const dialog = await screen.findByRole('alertdialog');
      await user.click(
        within(dialog).getByRole('button', { name: named(m('month-close-button')) }),
      );
      await waitFor(() => {
        expect(monthApi.monthCloseSave).toHaveBeenCalledWith('2026-10');
      }, SLOW_MS);
      expect(
        await screen.findByText(named(m('month-closed-toast', { month: OCTOBER }))),
      ).toBeVisible();
      // The list is read again: October is closed and its card is the closed one.
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-closed-title', { month: OCTOBER })) },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(cell('2026-10', 'state')).toBe(m('month-state-closed'));
      }, SLOW_MS);
      expect(monthApi.monthCloseSave).toHaveBeenCalledTimes(1);
    });
  });

  describe('a month that cannot be closed yet', () => {
    it('says a month that is not finished is not finished, in words, with no Close button', async () => {
      const { user } = await openScreen();
      await openMonth(user, '2026-11');
      expect(
        await screen.findByText(
          named(m('month-error-not-finished', { month: NOVEMBER })),
          undefined,
          SLOW_MS,
        ),
      ).toBeVisible();
      expect(screen.queryByRole('button', { name: named(m('month-close-button')) })).toBeNull();
      expect(visible(document.body.textContent)).not.toMatch(/ERR_/);
    });

    it('says the days are not all closed, up to which day, and opens the Day screen from a link', async () => {
      const { router, user } = await openScreen({
        list: withState(MONTH_LIST_READY, '2026-09', 'days_open'),
      });
      await openMonth(user, '2026-09');
      expect(
        await screen.findByText(
          named(m('month-error-days-open', { month: SEPTEMBER, closed_up_to: '09-10-2026' })),
          undefined,
          SLOW_MS,
        ),
      ).toBeVisible();
      expect(cell('2026-09', 'state')).toBe(m('month-state-days-open'));
      await user.click(screen.getByRole('button', { name: named(m('month-open-day-close')) }));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/day');
      }, SLOW_MS);
    });

    it('tells a month in the wrong form in the general words', async () => {
      await openScreen({ refusePreview: refusedWith(MONTH_PREVIEW_BAD_MONTH) });
      expect(
        await screen.findByText(named(m('err-format-invalid')), undefined, SLOW_MS),
      ).toBeVisible();
    });

    it('opens on the month of the address', async () => {
      await openScreen({}, '/month?month=2026-09');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: SEPTEMBER })) },
        SLOW_MS,
      );
      expect(vi.mocked(monthApi.monthClosePreview).mock.calls.at(-1)?.[0]).toBe('2026-09');
    });
  });

  describe('a close the backend refuses', () => {
    async function pressClose(user: ReturnType<typeof userEvent.setup>) {
      await user.click(
        await screen.findByRole('button', { name: named(m('month-close-button')) }, SLOW_MS),
      );
      const dialog = await screen.findByRole('alertdialog');
      await user.click(
        within(dialog).getByRole('button', { name: named(m('month-close-button')) }),
      );
    }

    it.each([
      ['a month closed meanwhile', MONTH_SAVE_TWICE, 'month-error-already-closed'],
      [
        'a month with nothing to close',
        MONTH_SAVE_NOTHING_TO_CLOSE,
        'month-error-nothing-to-close',
      ],
    ] as const)(
      '%s is told in the month screen’s words, and nothing breaks',
      async (_name, refusal, id) => {
        const { user } = await openScreen({ refuseSave: refusedWith(refusal) });
        await pressClose(user);
        expect(
          await screen.findByText(named(m(id, { month: OCTOBER })), undefined, SLOW_MS),
        ).toBeVisible();
        // No “closed” message, and the month can still be tried again.
        expect(screen.queryByText(named(m('month-closed-toast', { month: OCTOBER })))).toBeNull();
        expect(screen.getByRole('button', { name: named(m('month-close-button')) })).toBeEnabled();
        expect(visible(document.body.textContent)).not.toMatch(/ERR_/);
      },
    );

    it('tells the days that are still open, with the day they are closed up to', async () => {
      const { user } = await openScreen({ refuseSave: refusedWith(MONTH_SAVE_DAYS_OPEN) });
      await pressClose(user);
      expect(
        await screen.findByText(
          named(m('month-error-days-open', { month: OCTOBER, closed_up_to: '09-10-2026' })),
          undefined,
          SLOW_MS,
        ),
      ).toBeVisible();
    });
  });

  describe('a month that is closed', () => {
    async function openClosedOctober() {
      const view = await openScreen({ closed: true });
      await openMonth(view.user, '2026-10');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-closed-title', { month: OCTOBER })) },
        SLOW_MS,
      );
      return view;
    }

    it('shows the day it was closed, and does not ask the backend for a preview of it', async () => {
      await openClosedOctober();
      expect(fact(m('month-closed-on'))).toBe('02-11-2026');
      expect(
        vi.mocked(monthApi.monthClosePreview).mock.calls.some(([month]) => month === '2026-10'),
      ).toBe(false);
      expect(screen.queryByRole('button', { name: named(m('month-close-button')) })).toBeNull();
    });

    it('asks for a reason before it reverses, and sends nothing without one', async () => {
      const { user } = await openClosedOctober();
      await user.click(screen.getByRole('button', { name: named(m('month-reverse-button')) }));
      const dialog = await screen.findByRole('alertdialog');
      await user.click(
        within(dialog).getByRole('button', { name: named(m('month-reverse-confirm')) }),
      );
      expect(await within(dialog).findByText(named(m('err-required')))).toBeVisible();
      expect(monthApi.monthCloseReverse).not.toHaveBeenCalled();
    });

    it('reverses with the reason typed, says so, and the month can be closed again', async () => {
      const { user } = await openClosedOctober();
      await user.click(screen.getByRole('button', { name: named(m('month-reverse-button')) }));
      const dialog = await screen.findByRole('alertdialog');
      await user.type(within(dialog).getByLabelText(m('month-reverse-reason')), 'wrong month');
      await user.click(
        within(dialog).getByRole('button', { name: named(m('month-reverse-confirm')) }),
      );
      await waitFor(() => {
        expect(monthApi.monthCloseReverse).toHaveBeenCalledWith('2026-10', 'wrong month');
      }, SLOW_MS);
      expect(
        await screen.findByText(named(m('month-reversed-toast', { month: OCTOBER }))),
      ).toBeVisible();
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: OCTOBER })) },
        SLOW_MS,
      );
      await waitFor(() => {
        expect(cell('2026-10', 'state')).toBe(m('month-state-ready'));
      }, SLOW_MS);
    });

    it('keeps the box open with what was typed when the backend has no close to reverse', async () => {
      const { user } = await (async () => {
        const view = await openScreen({
          closed: true,
          refuseReverse: refusedWith(MONTH_REVERSE_NOT_IN_FORCE),
        });
        await openMonth(view.user, '2026-10');
        await screen.findByRole(
          'heading',
          { level: 2, name: named(m('month-closed-title', { month: OCTOBER })) },
          SLOW_MS,
        );
        return view;
      })();
      await user.click(screen.getByRole('button', { name: named(m('month-reverse-button')) }));
      const dialog = await screen.findByRole('alertdialog');
      const reason = within(dialog).getByLabelText(m('month-reverse-reason'));
      await user.type(reason, 'wrong month');
      await user.click(
        within(dialog).getByRole('button', { name: named(m('month-reverse-confirm')) }),
      );
      expect(await within(dialog).findByText(named(m('err-not-found')))).toBeVisible();
      expect(reason).toHaveValue('wrong month');
      expect(visible(dialog.textContent)).not.toMatch(/ERR_/);
    });

    it('shows the backend’s refusal of the reason under the reason box', async () => {
      const view = await openScreen({
        closed: true,
        refuseReverse: refusedWith(MONTH_REVERSE_NO_REASON),
      });
      await openMonth(view.user, '2026-10');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-closed-title', { month: OCTOBER })) },
        SLOW_MS,
      );
      await view.user.click(screen.getByRole('button', { name: named(m('month-reverse-button')) }));
      const dialog = await screen.findByRole('alertdialog');
      await view.user.type(within(dialog).getByLabelText(m('month-reverse-reason')), 'x');
      await view.user.click(
        within(dialog).getByRole('button', { name: named(m('month-reverse-confirm')) }),
      );
      expect(
        await within(dialog).findByText(named(m('err-reason-required')), undefined, SLOW_MS),
      ).toBeVisible();
    });
  });

  describe('the four states', () => {
    it('shows the loading state while the backend answers', async () => {
      fakeMonth();
      vi.mocked(monthApi.monthList).mockImplementation(() => new Promise(() => undefined));
      await renderApp('/month');
      // The screen has asked and is waiting: its own loading state is there, and no title yet.
      await waitFor(() => {
        expect(monthApi.monthList).toHaveBeenCalled();
      }, SLOW_MS);
      expect(await screen.findByRole('status')).toBeInTheDocument();
      expect(screen.queryByRole('heading', { level: 1, name: named(m('month-title')) })).toBeNull();
    });

    it('says what happened and tries again when the months cannot be read', async () => {
      const failing = new CommandError({
        ok: false,
        errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
        ref: 'E-1',
      });
      const view = await (async () => {
        fakeMonth({ list: failing });
        await renderApp('/month');
        return { user: userEvent.setup() };
      })();
      await screen.findByText(named(m('month-load-failed')), undefined, SLOW_MS);
      fakeMonth();
      await view.user.click(screen.getByRole('button', { name: named(m('common-retry')) }));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(4);
      }, SLOW_MS);
    });

    it('says there is no month yet, and what to do, when there is no entry', async () => {
      await openScreen({ list: { rows: [] } });
      expect(await screen.findByText(named(m('month-empty')))).toBeVisible();
      expect(screen.getByText(named(m('month-empty-help')))).toBeVisible();
    });
  });

  describe('a language switch in the middle of a month', () => {
    it('keeps the month that is open, words it in the other language, and asks nothing again', async () => {
      const { user } = await openScreen();
      await openMonth(user, '2026-09');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: SEPTEMBER })) },
        SLOW_MS,
      );
      const before = vi.mocked(monthApi.monthClosePreview).mock.calls.length;
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await screen.findByRole(
        'heading',
        {
          level: 2,
          name: named(message(other, 'month-preview-title', { month: label('2026-09', other) })),
        },
        SLOW_MS,
      );
      // The list is drawn again, in the other language, with the same months.
      await waitFor(() => {
        expect(cell('2026-09', 'month')).toBe(label('2026-09', other));
      }, SLOW_MS);
      expect(vi.mocked(monthApi.monthClosePreview).mock.calls.length).toBe(before);
    });
  });

  describe('keyboard only', () => {
    it('opens a month from the list, reaches Close, asks, leaves the box with Esc, and then closes', async () => {
      const { user } = await openScreen();
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: OCTOBER })) },
        SLOW_MS,
      );
      // Tab to the list, ↓ to September, Enter opens it.
      for (let tabs = 0; tabs < 30; tabs += 1) {
        if (document.activeElement?.closest('.ag-root-wrapper') != null) break;
        await user.tab();
      }
      expect(document.activeElement?.closest('.ag-root-wrapper')).not.toBeNull();
      for (let downs = 0; downs < 8; downs += 1) {
        const focused = document.querySelector('.ag-cell-focus')?.closest('.ag-row');
        if (focused?.getAttribute('row-id') === '2026-09') break;
        await user.keyboard('{ArrowDown}');
      }
      await user.keyboard('{Enter}');
      await screen.findByRole(
        'heading',
        { level: 2, name: named(m('month-preview-title', { month: SEPTEMBER })) },
        SLOW_MS,
      );
      // Tab to the Close button and press it: the box opens with Back in focus, Esc leaves it.
      const close = await screen.findByRole(
        'button',
        { name: named(m('month-close-button')) },
        SLOW_MS,
      );
      // (Leaving the grid with Tab is the grid's own business, checked on the real app in the Windows
      // suite: jsdom does not move focus out of AG Grid's tab guards.)
      await act(async () => {
        close.focus();
        await Promise.resolve();
      });
      expect(document.activeElement).toBe(close);
      await user.keyboard('{Enter}');
      const dialog = await screen.findByRole('alertdialog');
      expect(within(dialog).getByRole('button', { name: named(m('common-back')) })).toHaveFocus();
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).toBeNull();
      }, SLOW_MS);
      expect(monthApi.monthCloseSave).not.toHaveBeenCalled();
      // Again, and this time Tab to the action and Enter closes September.
      await user.keyboard('{Enter}');
      const again = await screen.findByRole('alertdialog');
      await user.tab();
      expect(
        within(again).getByRole('button', { name: named(m('month-close-button')) }),
      ).toHaveFocus();
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(monthApi.monthCloseSave).toHaveBeenCalledWith('2026-09');
      }, SLOW_MS);
    });
  });

  describe('where the screen is reached from', () => {
    it('has a Month Close button on the partners overview that opens it', async () => {
      fakeMonth();
      const { router } = await renderApp('/partners');
      const user = userEvent.setup();
      await user.click(
        await screen.findByRole('button', { name: named(m('month-open')) }, SLOW_MS),
      );
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/month');
      }, SLOW_MS);
      await screen.findByRole('heading', { level: 1, name: named(m('month-title')) }, SLOW_MS);
    });
  });
});
