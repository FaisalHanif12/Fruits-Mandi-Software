// A saved جنرل entry open for edit, and the list of saved entries under the form (plan 44, PRD
// FR-7.70, FR-21.4), in the Urdu and the English app, against a fake backend: `?id=` fills the grid
// with the saved lines, the amber band shows, the save goes to `vouchers_edit`, an entry of an
// earlier day asks why, and the list under a new entry opens a saved one with its Edit button.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as vouchersApi from '../../src/features/vouchers/api';
import * as editApi from '../../src/shared/edit/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { cellOf, gridOf, textOf } from '../helpers/bikri-screen';
import { named, startsWith } from '../helpers/cashbook-screen';
import { leaveAppWindow, press, renderApp } from '../helpers/app';
import { AUDIT_EDIT_STAMP_BIKRI, noChangeReview } from '../helpers/edit-real';
import { backendReopens, confirmFixDay } from '../helpers/fix-day';
import { CLOSED_DAY_ENTRY } from '../helpers/entry-flags';
import { paisaForTest } from '../helpers/paisa';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message, visible } from '../helpers/render';
import { voucherRow } from '../helpers/voucher-books';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

beforeEach(() => {
  vi.clearAllMocks();
});
afterEach(() => {
  leaveAppWindow();
});

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

/** The correction of Event 5 as saved: electricity 3,000 on بنام, cash 3,000 on جمع. */
function journal(overrides: Parameters<typeof voucherRow>[0] = {}) {
  return {
    row: voucherRow({
      id: 120,
      number: 7,
      voucher_type: 'general',
      account: null,
      narration: 'electricity fix',
      amount_paisa: paisaForTest(300_000),
      ...overrides,
    }),
    entry: {
      type: 'general' as const,
      date: '10-10-2026',
      narration: 'electricity fix',
      lines: [
        { account_id: 501, dr: '3000', cr: '', narration: null },
        { account_id: 1, dr: '', cr: '3000', narration: null },
      ],
      confirm_cash_below_zero: false,
    },
  };
}

const EDITED = {
  voucher_id: 120,
  number: 7,
  version: 2,
  entry_id: 9,
  cash_after: paisaForTest(0),
} as never;

describe.each(LANGUAGES)('a journal entry open for edit (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(noChangeReview());
  });

  const grid = () => gridOf(language, 'cashbook-general-grid-label');

  async function open(id: number) {
    const app = await renderApp(`/rokar/general?id=${String(id)}`);
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'cashbook-general-screen-title')),
    });
    await waitFor(() => {
      expect(cellOf(grid(), 1, 'cr')).toBeInTheDocument();
    });
    return { ...app, user: userEvent.setup() };
  }

  it('fills the grid with the saved lines, shows the band, and has no list under it', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(journal());
    await open(120);
    expect(grid().querySelectorAll('.ag-row[row-index]')).toHaveLength(2);
    expect(textOf(grid(), 0, 'account')).toBe(nameOf(language, 'بجلی', 'Electricity'));
    expect(textOf(grid(), 0, 'dr')).toBe('3,000');
    expect(textOf(grid(), 1, 'account')).toBe(nameOf(language, 'روکڑ', 'Cash'));
    expect(textOf(grid(), 1, 'cr')).toBe('3,000');
    expect(
      (
        await screen.findByLabelText<HTMLInputElement>(
          named(message(language, 'cashbook-field-narration')),
        )
      ).value,
    ).toBe('electricity fix');
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-band', {
        what: message(language, 'glossary-general'),
        number: '7',
        date: '10-10-2026',
      }),
    );
    expect(visible(document.body.textContent)).not.toContain(
      message(language, 'cashbook-general-list-title'),
    );
  });

  it('saves with vouchers_edit: the id, the version, the lines as saved and the new note', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(journal());
    vi.mocked(vouchersApi.vouchersEdit).mockResolvedValue(EDITED);
    const { user, router } = await open(120);
    const note = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'cashbook-field-narration')),
    );
    await user.clear(note);
    await user.type(note, 'wrong head');
    await waitFor(() => {
      expect(vouchersApi.vouchersEditPreview).toHaveBeenCalled();
    });
    // The backend says the lines balance, so Save is on.
    const save = screen.getByRole('button', {
      name: startsWith(message(language, 'layout-save-save')),
    });
    await waitFor(() => {
      expect(save).toBeEnabled();
    });
    press('F10');
    await waitFor(() => {
      expect(vouchersApi.vouchersEdit).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersEdit).mock.calls[0]?.[0]).toMatchObject({
      id: 120,
      version: 1,
      reason: '',
      entry: {
        type: 'general',
        date: '10-10-2026',
        narration: 'wrong head',
        lines: [
          { account_id: 501, dr: '3000', cr: '' },
          { account_id: 1, dr: '', cr: '3000' },
        ],
      },
    });
    await waitFor(() => {
      expect(router.state.location.search).not.toHaveProperty('id');
    });
  });

  it('an entry of an earlier day asks why before it saves', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      journal({ entry_date: '2026-10-09' }),
    );
    vi.mocked(vouchersApi.vouchersEditPreview).mockResolvedValue(
      noChangeReview('2026-10-09', true),
    );
    vi.mocked(vouchersApi.vouchersEdit).mockResolvedValue(EDITED);
    const { user } = await open(120);
    const note = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'cashbook-field-narration')),
    );
    await user.type(note, ' again');
    await waitFor(() => {
      expect(vouchersApi.vouchersEditPreview).toHaveBeenCalled();
    });
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: startsWith(message(language, 'layout-save-save')) }),
      ).toBeEnabled();
    });
    press('F10');
    const reason = await screen.findByRole('combobox', {
      name: named(message(language, 'edit-reason-label')),
    });
    expect(vouchersApi.vouchersEdit).not.toHaveBeenCalled();
    await user.type(reason, 'typed wrong{Enter}');
    await waitFor(() => {
      expect(vouchersApi.vouchersEdit).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(vouchersApi.vouchersEdit).mock.calls[0]?.[0]).toMatchObject({
      id: 120,
      reason: 'typed wrong',
    });
  });

  it('the note and the line an earlier edit changed carry what they held before', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      journal({ edited: AUDIT_EDIT_STAMP_BIKRI[0]?.stamp ?? null }),
    );
    vi.mocked(editApi.auditEditHistory).mockResolvedValue({
      entity: 'voucher',
      id: 120,
      versions: [
        {
          audit_id: 3,
          at: '2026-10-11T09:00:00Z',
          local_date: '2026-10-11',
          local_time: '14:00',
          by: 'Owner',
          action: 'edit',
          reason: null,
          changes: [
            {
              path: 'narration',
              old: { kind: 'text', text: 'fix' },
              new: { kind: 'text', text: 'electricity fix' },
            },
            {
              path: 'lines[0].amount_paisa',
              old: { kind: 'money', paisa: paisaForTest(250_000) },
              new: { kind: 'money', paisa: paisaForTest(300_000) },
            },
          ],
          changes_truncated: false,
        },
      ],
    });
    await open(120);
    await waitFor(() => {
      expect(
        Array.from(document.querySelectorAll('[data-field-mark]')).map((mark) =>
          visible(mark.textContent),
        ),
      ).toContain(message(language, 'edit-mark-before', { before: 'fix' }));
    });
    // The line that was on بنام keeps its mark in the بنام cell, and the جمع cell has none.
    await waitFor(() => {
      expect(cellOf(grid(), 0, 'dr')).toHaveClass('cell-marked');
    });
    expect(cellOf(grid(), 0, 'cr')).not.toHaveClass('cell-marked');
  });

  it('a closed day shows the entry as stored and a button that reopens the day first', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      journal({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' }),
    );
    await renderApp('/rokar/general?id=120');
    expect(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      }),
    ).toBeEnabled();
    expect(
      screen.queryByLabelText(named(message(language, 'cashbook-field-narration'))),
    ).toBeNull();
  });

  it('a closed day: the box reopens the day of the entry as typed, DD-MM-YYYY (Umar, 13-11-2026)', async () => {
    vi.mocked(vouchersApi.vouchersOpenForEdit).mockResolvedValue(
      journal({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' }),
    );
    backendReopens();
    await renderApp('/rokar/general?id=120');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      }),
    );
    await confirmFixDay(language, user, '05-10-2026');
  });
});

describe.each(LANGUAGES)('the list of saved journal entries (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
  });

  /** The list answers with these rows, whatever the filter. */
  function listWith(...rows: ReturnType<typeof voucherRow>[]) {
    vi.mocked(vouchersApi.vouchersList).mockResolvedValue({
      rows,
      total_count: rows.length,
      totals: { cancelled_count: 0, total_paisa: paisaForTest(300_000) },
      months: [],
    });
  }

  const numberText = (number: number) => message(language, 'vouchers-number-general', { number });

  it('lists the entries under a new one, and Edit opens the one that has focus', async () => {
    listWith(journal().row);
    const { router } = await renderApp('/rokar/general');
    const user = userEvent.setup();
    expect(
      await screen.findByText(named(message(language, 'cashbook-general-list-title'))),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(vouchersApi.vouchersList).toHaveBeenCalled();
    });
    expect(vi.mocked(vouchersApi.vouchersList).mock.calls[0]?.[0]).toMatchObject({
      voucher_type: 'general',
    });
    // Edit is off until a row has focus.
    const edit = () =>
      screen.getByRole('button', { name: startsWith(message(language, 'edit-action-edit')) });
    expect(edit()).toBeDisabled();
    await user.click(await screen.findByText(named(numberText(7))));
    await waitFor(() => {
      expect(edit()).toBeEnabled();
    });
    await user.click(edit());
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar/general');
      expect(router.state.location.search).toMatchObject({ id: 120 });
    });
  });

  it('a row of a closed day says it reopens the day first', async () => {
    listWith(journal({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' }).row);
    await renderApp('/rokar/general');
    const user = userEvent.setup();
    await user.click(await screen.findByText(named(numberText(7))));
    expect(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      }),
    ).toBeEnabled();
  });

  it('a row of a closed day: the box reopens its day as typed, DD-MM-YYYY (Umar, 13-11-2026)', async () => {
    listWith(journal({ ...CLOSED_DAY_ENTRY, entry_date: '2026-10-05' }).row);
    backendReopens();
    await renderApp('/rokar/general');
    const user = userEvent.setup();
    await user.click(await screen.findByText(named(numberText(7))));
    await user.click(
      await screen.findByRole('button', {
        name: startsWith(message(language, 'edit-action-fix-day')),
      }),
    );
    await confirmFixDay(language, user, '05-10-2026');
  });

  it('an empty list says so and what to do', async () => {
    listWith();
    await renderApp('/rokar/general');
    expect(
      await screen.findByText(named(message(language, 'cashbook-general-list-empty'))),
    ).toBeInTheDocument();
  });
});
