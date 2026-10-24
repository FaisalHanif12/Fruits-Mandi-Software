// The audit log in Settings (plan 22, PRD FR-21.3, NFR-SEC-5), in Urdu and in English, against a fake
// backend that answers with the backend's own fixtures. It lists the entries page by page with the
// backend's count of the whole filter; each filter box is sent as typed or chosen; a refusal shows
// under its own box and the rows stay; an entry opens with its changed fields, money through the
// formatter and a secret only as "changed"; the four states, a language switch in the middle of
// choosing filters, and the keyboard-only path are each proved. The screen adds up nothing and
// writes nothing.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AuditPage } from '../../src/bindings/AuditPage';
import type { AuditRow } from '../../src/bindings/AuditRow';
import * as auditApi from '../../src/features/audit/api';
import { useAuditStore } from '../../src/features/audit/store';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { AUDIT_FIRST_PAGE, AUDIT_OPTIONS, AUDIT_REFUSED, AUDIT_SETTINGS } from '../helpers/b5-real';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/audit/api');
vi.mock('../../src/shared/urdu/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

/** The backend's rows have the stand-in `<time>` and one id; a test shows real days and numbers. */
function dated(rows: readonly AuditRow[], first: number): AuditRow[] {
  return rows.map((row, index) => ({
    ...row,
    id: first + index,
    date: '2026-10-10',
    time: `14:05:0${String(index)}`,
    at: `2026-10-10T09:05:0${String(index)}Z`,
  }));
}

const ROWS = dated([...AUDIT_FIRST_PAGE.rows, ...AUDIT_SETTINGS.rows.slice(0, 1)], 10);

const FIRST_ROW = ((): AuditRow => {
  const row = ROWS[0];
  if (row === undefined) throw new Error('the fixture has no rows');
  return row;
})();

/** The element of a selector, or a plain failure that names it. */
function pick(selector: string): HTMLElement {
  const found = document.querySelector<HTMLElement>(selector);
  if (found === null) throw new Error(`nothing matches ${selector}`);
  return found;
}
const TOTAL = 230;

/** The page the backend would send for the rows `offset` on, out of `TOTAL` entries. */
function pageOf(offset: number, limit: number): AuditPage {
  return {
    ...AUDIT_FIRST_PAGE,
    rows: ROWS.map((row, index) => ({ ...row, id: 1000 - offset - index })).slice(
      0,
      Math.min(limit, 3),
    ),
    offset,
    limit,
    total_count: TOTAL,
  };
}

/** A row whose changes show a flag, an amount, a secret and a field that is new. */
const DETAIL_ROW: AuditRow = {
  ...FIRST_ROW,
  id: 4711,
  entity_id: 12,
  reason: 'گھر کا خرچ',
  changes: [
    {
      path: 'auto_day_close',
      old: { kind: 'bool', value: false },
      new: { kind: 'bool', value: true },
    },
    {
      path: 'limit_paisa',
      old: { kind: 'money', paisa: paisaForTest(100_000) },
      new: { kind: 'money', paisa: paisaForTest(150_000) },
    },
    { path: 'password', old: { kind: 'hidden' }, new: { kind: 'hidden' } },
    { path: 'firm.phone', old: { kind: 'null' }, new: { kind: 'text', text: '0300-0000000' } },
  ],
};

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitPairsList).mockResolvedValue([]);
  vi.mocked(auditApi.auditOptions).mockResolvedValue(AUDIT_OPTIONS);
  vi.mocked(auditApi.auditList).mockImplementation(({ offset, limit }) =>
    Promise.resolve(pageOf(offset, limit)),
  );
  act(() => {
    useAuditStore.getState().reset();
  });
});

afterEach(() => {
  leaveAppWindow();
});

function rowCells(rowId: string, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${rowId}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}

function drawnRows(): string[] {
  return Array.from(document.querySelectorAll('.ag-row[row-id]:not(.ag-row-pinned)')).map(
    (row) => row.getAttribute('row-id') ?? '',
  );
}

function lastRequest() {
  return vi.mocked(auditApi.auditList).mock.calls.at(-1)?.[0];
}

describe.each(LANGUAGES)('the audit log (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);

  async function openAudit() {
    const app = await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: m('settings-title') });
    const user = userEvent.setup();
    await user.click(screen.getByRole('tab', { name: named(language, 'settings-tab-audit') }));
    await screen.findByRole('heading', { level: 2, name: m('audit-title') });
    return { ...app, user };
  }

  it('asks for the first page with every filter sent empty, and shows the entries in words', async () => {
    await openAudit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(3);
    }, SLOW_MS);
    // Nothing is filtered, and the page size is the backend's page of a hundred.
    expect(lastRequest()).toEqual({
      filters: {
        user_id: null,
        action: '',
        entity: '',
        entity_id: null,
        from: '',
        to: '',
        text: '',
      },
      offset: 0,
      limit: 100,
    });
    // The newest first, as the backend sent it: the Pakistan day and time (without seconds), who,
    // what and to what.
    const first = drawnRows()[0] ?? '';
    expect(rowCells(first, 'when')).toBe('10-10-2026 14:05');
    expect(rowCells(first, 'user')).toBe('Owner');
    expect(rowCells(first, 'action')).toBe(m('audit-action-edit'));
    expect(rowCells(first, 'entity')).toBe(m('audit-entity-day-settings'));
    expect(rowCells(first, 'fields')).toBe('1');
    // The count is the backend's count of the whole filter, never the rows on screen.
    expect(
      screen.getByText(
        (_, element) =>
          visible(element?.textContent) === m('audit-count', { shown: 3, total: TOTAL }),
      ),
    ).toBeInTheDocument();
  });

  it('shows more entries on a press, from where the loaded ones end', async () => {
    const { user } = await openAudit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(3);
    }, SLOW_MS);
    await user.click(screen.getByRole('button', { name: named(language, 'audit-more') }));
    await waitFor(() => {
      expect(lastRequest()?.offset).toBe(3);
    }, SLOW_MS);
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(6);
    }, SLOW_MS);
  });

  it('sends the action, the area, the record, the days and the words as chosen and typed', async () => {
    const { user } = await openAudit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(3);
    }, SLOW_MS);
    await user.click(screen.getByRole('combobox', { name: m('audit-filter-action') }));
    await user.click(await screen.findByRole('option', { name: m('audit-action-edit') }));
    await user.click(screen.getByRole('combobox', { name: m('audit-filter-entity') }));
    await user.click(await screen.findByRole('option', { name: m('audit-entity-settings') }));
    await user.click(screen.getByRole('combobox', { name: m('audit-filter-user') }));
    await user.click(await screen.findByRole('option', { name: 'Owner' }));
    await user.type(screen.getByLabelText(named(language, 'audit-filter-record')), '12');
    await user.type(screen.getByLabelText(named(language, 'report-filter-from')), '01-10-2026');
    await user.type(screen.getByLabelText(named(language, 'report-filter-to')), '31-10-2026');
    await user.type(screen.getByLabelText(named(language, 'audit-filter-text')), 'lakh');
    await waitFor(() => {
      expect(lastRequest()?.filters).toEqual({
        user_id: 0,
        action: 'edit',
        entity: 'settings',
        entity_id: 12,
        from: '01-10-2026',
        to: '31-10-2026',
        text: 'lakh',
      });
    }, SLOW_MS);
    // The list asked for starts again from the first entry.
    expect(lastRequest()?.offset).toBe(0);
  });

  it('a code the app has no words for is shown as it is, never hidden', async () => {
    vi.mocked(auditApi.auditList).mockResolvedValue({
      ...pageOf(0, 100),
      rows: [{ ...FIRST_ROW, id: 1, action: 'reopen_day', entity: 'day_closes' }],
      total_count: 1,
    });
    await openAudit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(1);
    }, SLOW_MS);
    expect(rowCells('1', 'action')).toBe('reopen_day');
    expect(rowCells('1', 'entity')).toBe('day_closes');
  });

  it('a refusal shows under its own boxes and the entries stay where they were', async () => {
    const { user } = await openAudit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(3);
    }, SLOW_MS);
    vi.mocked(auditApi.auditList).mockRejectedValue(new CommandError(AUDIT_REFUSED));
    await user.type(screen.getByLabelText(named(language, 'report-filter-from')), '31-31-2026');
    const from = screen.getByLabelText(named(language, 'report-filter-from'));
    await waitFor(() => {
      expect(from).toHaveAttribute('aria-invalid', 'true');
    }, SLOW_MS);
    expect(drawnRows()).toHaveLength(3);
    // The unknown user is named under the user box: the log is not replaced by an error page.
    expect(
      screen.getAllByText((_, element) => visible(element?.textContent) === m('err-not-found'))
        .length,
    ).toBeGreaterThan(0);
  });

  describe('one entry opened', () => {
    async function openDetail() {
      vi.mocked(auditApi.auditList).mockResolvedValue({
        ...pageOf(0, 100),
        rows: [DETAIL_ROW],
        total_count: 1,
      });
      const app = await openAudit();
      await waitFor(() => {
        expect(drawnRows()).toEqual(['4711']);
      }, SLOW_MS);
      await app.user.click(pick('.ag-row[row-id="4711"] [col-id="user"]'));
      await app.user.keyboard('{Enter}');
      await screen.findByRole('heading', {
        level: 3,
        name: (name) => visible(name) === m('audit-detail-title', { number: '4711' }),
      });
      return app;
    }

    it('lists each changed field before and after: a flag in words, money formatted, a new field marked', async () => {
      await openDetail();
      // Only the grid of the changed fields is on screen now (the list is hidden behind it).
      const grid = document.body;
      const cells = (path: string, column: string) => {
        const rows = Array.from(grid.querySelectorAll('.ag-row'));
        const row = rows.find((candidate) => visible(candidate.textContent).includes(path));
        return visible(row?.querySelector(`[col-id="${column}"]`)?.textContent);
      };
      expect(cells('auto_day_close', 'old')).toBe(m('audit-value-no'));
      expect(cells('auto_day_close', 'new')).toBe(m('audit-value-yes'));
      // Amounts are the backend's, written by the money formatter (never worked out here).
      expect(cells('limit_paisa', 'old')).toBe('1,000');
      expect(cells('limit_paisa', 'new')).toBe('1,500');
      expect(cells('firm.phone', 'old')).toBe(m('audit-value-none'));
      expect(cells('firm.phone', 'new')).toBe('0300-0000000');
    });

    it('a secret is never shown, only that it changed', async () => {
      await openDetail();
      const rows = Array.from(document.querySelectorAll('.ag-row'));
      const row = rows.find((candidate) => visible(candidate.textContent).includes('password'));
      expect(visible(row?.querySelector('[col-id="old"]')?.textContent)).toBe(
        m('audit-value-hidden'),
      );
      expect(visible(row?.querySelector('[col-id="new"]')?.textContent)).toBe(
        m('audit-value-hidden'),
      );
    });

    it('shows who, when, what and the reason as typed, and Esc comes back to the list', async () => {
      const { user } = await openDetail();
      const facts = within(pick('dl'));
      expect(facts.getByText('Owner')).toBeInTheDocument();
      expect(facts.getByText(m('audit-action-edit'))).toBeInTheDocument();
      expect(facts.getByText('گھر کا خرچ')).toBeInTheDocument();
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(drawnRows()).toEqual(['4711']);
      }, SLOW_MS);
      expect(screen.queryByRole('heading', { level: 3 })).toBeNull();
    });

    it('says when more fields changed than the backend listed', async () => {
      vi.mocked(auditApi.auditList).mockResolvedValue({
        ...pageOf(0, 100),
        rows: [{ ...DETAIL_ROW, changes_truncated: true }],
        total_count: 1,
      });
      const { user } = await openAudit();
      await waitFor(() => {
        expect(drawnRows()).toEqual(['4711']);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="4711"] [col-id="user"]'));
      await user.keyboard('{Enter}');
      expect(
        await screen.findByText(
          (_, element) =>
            visible(element?.textContent) === m('audit-detail-truncated', { shown: 4 }),
        ),
      ).toBeInTheDocument();
    });

    it('an entry with no listed field (a sign-in) says so', async () => {
      vi.mocked(auditApi.auditList).mockResolvedValue({
        ...pageOf(0, 100),
        rows: [{ ...DETAIL_ROW, action: 'login', entity: 'user', changes: [] }],
        total_count: 1,
      });
      const { user } = await openAudit();
      await waitFor(() => {
        expect(drawnRows()).toEqual(['4711']);
      }, SLOW_MS);
      await user.click(pick('.ag-row[row-id="4711"] [col-id="user"]'));
      await user.keyboard('{Enter}');
      expect(await screen.findByText(m('audit-detail-none'))).toBeInTheDocument();
    });
  });

  describe('the four states', () => {
    it('shows the loading state while the backend answers', async () => {
      vi.mocked(auditApi.auditList).mockReturnValue(new Promise(() => undefined));
      await openAudit();
      expect(await screen.findByRole('status')).toBeInTheDocument();
      expect(drawnRows()).toHaveLength(0);
    });

    it('says nothing matches, and offers to clear a filter that is set', async () => {
      vi.mocked(auditApi.auditList).mockResolvedValue({
        ...pageOf(0, 100),
        rows: [],
        total_count: 0,
      });
      const { user } = await openAudit();
      expect(await screen.findByText(m('audit-empty'))).toBeInTheDocument();
      // Nothing is filtered, so there is nothing to clear.
      expect(
        screen.queryByRole('button', { name: named(language, 'report-clear-filters') }),
      ).toBeNull();
      await user.type(screen.getByLabelText(named(language, 'audit-filter-text')), 'zzz');
      await user.click(
        await screen.findByRole('button', { name: named(language, 'report-clear-filters') }),
      );
      await waitFor(() => {
        expect(
          screen.getByLabelText<HTMLInputElement>(named(language, 'audit-filter-text')).value,
        ).toBe('');
      });
    });

    it('says what happened and tries again when the log cannot be read', async () => {
      vi.mocked(auditApi.auditList).mockRejectedValueOnce(new Error('down'));
      const { user } = await openAudit();
      expect(await screen.findByText(m('audit-load-failed'))).toBeInTheDocument();
      await user.click(screen.getByRole('button', { name: named(language, 'common-retry') }));
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
    });

    it('keeps the log usable when the lists of the filter boxes cannot be read', async () => {
      vi.mocked(auditApi.auditOptions).mockRejectedValue(new Error('down'));
      await openAudit();
      expect(await screen.findByText(m('audit-options-failed'))).toBeInTheDocument();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
    });
  });

  describe('a language switch in the middle of choosing filters', () => {
    it('keeps the filters and the open entry, and writes the entries in the other language', async () => {
      const { user } = await openAudit();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await user.type(screen.getByLabelText(named(language, 'audit-filter-text')), 'lakh');
      await user.type(screen.getByLabelText(named(language, 'report-filter-from')), '01-10-2026');
      // The filters settle 250 ms after the last key (Code Standards F35): wait until the backend has
      // been asked with them, so the count below is the count after the last request, even on a
      // busy machine.
      await waitFor(() => {
        expect(lastRequest()?.filters).toMatchObject({ text: 'lakh', from: '01-10-2026' });
      }, SLOW_MS);
      const before = vi.mocked(auditApi.auditList).mock.calls.length;
      const other: Language = language === 'ur' ? 'en' : 'ur';
      act(() => {
        useLanguageStore.getState().setLanguage(other);
      });
      await waitFor(() => {
        expect(
          screen.getByLabelText<HTMLInputElement>(named(other, 'audit-filter-text')).value,
        ).toBe('lakh');
      }, SLOW_MS);
      expect(
        screen.getByLabelText<HTMLInputElement>(named(other, 'report-filter-from')).value,
      ).toBe('01-10-2026');
      await waitFor(() => {
        expect(rowCells(drawnRows()[0] ?? '', 'action')).toBe(message(other, 'audit-action-edit'));
      }, SLOW_MS);
      // The same filters are asked again by the same cache entry: nothing new went to the backend.
      expect(vi.mocked(auditApi.auditList).mock.calls.length).toBe(before);
    });
  });

  describe('keyboard only', () => {
    it('Tab into the list, a row, Enter for its fields, Esc back to the list', async () => {
      const { user } = await openAudit();
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      await user.click(pick(`.ag-row[row-id="${drawnRows()[1] ?? ''}"] [col-id="when"]`));
      await user.keyboard('{Enter}');
      await screen.findByRole('heading', { level: 3 });
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(drawnRows()).toHaveLength(3);
      }, SLOW_MS);
      expect(screen.queryByRole('heading', { level: 3 })).toBeNull();
      // The tab row stays reachable: Esc leaves the detail first, not the settings.
      expect(
        screen.getByRole('heading', { level: 1, name: m('settings-title') }),
      ).toBeInTheDocument();
    });
  });

  it('writes nothing: the only commands used are the two that read', async () => {
    await openAudit();
    await waitFor(() => {
      expect(drawnRows()).toHaveLength(3);
    }, SLOW_MS);
    expect(vi.mocked(settingsApi.settingsSetFirm)).not.toHaveBeenCalled();
    expect(vi.mocked(auditApi.auditList).mock.calls.length).toBeGreaterThan(0);
    expect(Object.keys(auditApi).toSorted()).toEqual(['auditList', 'auditOptions']);
  });
});
