// A NEW بکری on a closed day (plan 85, PRD FR-21.4 v1.49, FR-12.6, FR-6.6), in the Urdu and the English app. The
// backend refuses a new بکری whose date is a closed day (`ERR_DATE_CLOSED` on `header.entry_date`), so every line
// has no figures. Before this plan the screen said so only in a small note under the date, with no way out. Now it
// shows the closed-day notice with "Reopen the day and save": the box asks the reason and the Owner's sign, the
// days reopen, the figures fill, the save posts, and the days close again. A wrong sign opens nothing.
// The real screen, the real grid, the real store; the backend is a double of `bikri_preview` and the two day-fix
// commands, with the files of the plan 44 and plan 73 contracts.

import { screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import { SALE } from '../../src/features/bikri/types';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as editApi from '../../src/shared/edit/api';
import { useFixStore } from '../../src/shared/edit/fix-store';
import * as urduApi from '../../src/shared/urdu/api';
import {
  healthyBackend,
  leaveAppWindow,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fillUmarsLot, umarAnswer, MANI } from '../helpers/bikri-umar-data';
import { openUmarsBook } from '../helpers/bikri-umar';
import { cellOf, named, salesGrid, textOf } from '../helpers/bikri-screen';
import {
  DAY_FIX_CLOSE_E2E_45,
  DAY_FIX_OPEN_E2E_45,
  DAY_FIX_OPEN_SIGN_WRONG,
} from '../helpers/edit-real';
import { OWNER_SIGN, confirmFixDay } from '../helpers/fix-day';
import { fakePartners } from '../helpers/partners';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

/** A wait that holds when the whole suite is running and the machine is busy. */
const SLOW_MS = { timeout: 8_000 };

/** The day of the sale book in these tests: the header date of `seedLine`, as typed. */
const DAY = '10-10-2026';

/** True from the moment the double has opened the day. */
let opened = false;

beforeEach(() => {
  vi.resetAllMocks();
  opened = false;
  useFixStore.getState().end();
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(editApi.auditEditStamp).mockResolvedValue([]);
  vi.mocked(editApi.dayFixOpen).mockImplementation(() => {
    opened = true;
    return Promise.resolve(DAY_FIX_OPEN_E2E_45);
  });
  vi.mocked(editApi.dayFixClose).mockResolvedValue(DAY_FIX_CLOSE_E2E_45);
});

afterEach(() => {
  leaveAppWindow();
  useFixStore.getState().end();
});

/** The next sale book opens with Umar's lot and one line: مانی, 130 at 900 (the figures are in `umarAnswer`). */
function seedLine(): void {
  const stores = createBikriDraft(DAY);
  fillUmarsLot(stores);
  const lot = stores.lots.getState().rows[0];
  if (lot === undefined) throw new Error('no lot row');
  const sales = stores.salesFor(lot.rowId);
  const first = sales.getState().rows[0];
  if (first === undefined) throw new Error('no line row');
  sales.getState().patchRow(first.rowId, {
    picked: { [SALE.buyer]: { id: MANI.id, code: MANI.code, name_ur: 'مانی', name_en: 'Mani' } },
    typed: { qty: '130', qtyFilled: '1', rate: '900' },
  });
  const snapshot = parseSnapshot(serialiseDraft(stores));
  if (snapshot === null) throw new Error('the seeded draft is not a snapshot');
  usePendingRestore.getState().offer(snapshot);
}

/**
 * The backend's answer while the day is closed: the one refusal of the date, no figures on any line and none
 * for the lot or the whole sale (domain/src/bikri/process.rs: with no date no lot is complete). Once the double
 * has opened the day it answers as for an open day.
 */
function answer(draft: BikriDraft): BikriPreview {
  const reply = umarAnswer(draft);
  if (opened) return reply;
  for (const lot of reply.lots) {
    lot.totals = null;
    for (const line of lot.lines) line.calc = null;
  }
  reply.errors = [
    {
      code: 'ERR_DATE_CLOSED',
      field: 'header.entry_date',
      values: { closed_up_to: '10-10-2026', today: '10-10-2026', today_open: 0 },
    },
  ];
  return reply;
}

const alertsText = () =>
  screen.queryAllByRole('alert').map((element) => visible(element.textContent));

describe.each(LANGUAGES)('a new بکری on a closed day (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const closedWords = () => message(language, 'edit-closed-entry', { date: DAY });
  const fixButton = () =>
    screen.findByRole(
      'button',
      { name: named(message(language, 'edit-action-fix-save')) },
      SLOW_MS,
    );

  it('a closed date shows the notice and the button "Reopen the day and save", and the line has no figures', async () => {
    seedLine();
    await openUmarsBook(language, answer);
    const sales = salesGrid(language);
    await waitFor(() => {
      expect(cellOf(sales, 0, 'buyer')).toBeInTheDocument();
    }, SLOW_MS);
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(true);
    }, SLOW_MS);
    expect(await fixButton()).toBeInTheDocument();
    // The line shows no figures while the day is closed.
    expect(textOf(sales, 0, 'buyerTotal')).toBe('');
    // Nothing was reopened by itself.
    expect(editApi.dayFixOpen).not.toHaveBeenCalled();
  });

  it('the reason and the Owner’s sign reopen the days (the date goes as typed), and the figures appear', async () => {
    seedLine();
    const { user } = await openUmarsBook(language, answer);
    const sales = salesGrid(language);
    await user.click(await fixButton());
    // The box asks the reason and the sign; the reopen is for the day of the entry.
    await confirmFixDay(language, user, DAY);
    // The day is open: the red notice is gone and the amber line says the days close again after the save.
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(false);
    }, SLOW_MS);
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
    expect(
      screen
        .getAllByRole('status')
        .some(
          (element) =>
            visible(element.textContent) === message(language, 'edit-band-days', { days: DAY }),
        ),
    ).toBe(true);
    // Only the reopen was asked so far: the days close again after the save.
    expect(editApi.dayFixClose).not.toHaveBeenCalled();
  });

  it('the save posts on that day and the days close again with the token of the reopen', async () => {
    seedLine();
    const { user, backend } = await openUmarsBook(language, answer);
    const sales = salesGrid(language);
    await user.click(await fixButton());
    await confirmFixDay(language, user, DAY);
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
    await user.click(
      screen.getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'bikri-save')),
      }),
    );
    await waitFor(() => {
      expect(backend.saves).toHaveLength(1);
    }, SLOW_MS);
    expect(backend.saves[0]?.draft.header.entry_date).toBe(DAY);
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    }, SLOW_MS);
    expect(editApi.dayFixClose).toHaveBeenCalledWith(DAY_FIX_OPEN_E2E_45.token);
    // The session is over: a second save would close nothing.
    expect(useFixStore.getState().session).toBeNull();
  });

  it('the day is open already (ERR_DAY_NOT_CLOSED): the box closes, the figures fill, and there is nothing to close again', async () => {
    seedLine();
    vi.mocked(editApi.dayFixOpen).mockImplementation(() => {
      opened = true;
      return Promise.reject(
        new CommandError(refusal([{ code: 'ERR_DAY_NOT_CLOSED', field: 'date', values: {} }])),
      );
    });
    const { user } = await openUmarsBook(language, answer);
    const sales = salesGrid(language);
    await user.click(await fixButton());
    await confirmFixDay(language, user, DAY);
    await waitFor(() => {
      expect(textOf(sales, 0, 'buyerTotal')).toBe('125,060');
    }, SLOW_MS);
    expect(screen.queryByRole('alertdialog')).toBeNull();
    // No token was given, so no days are waiting to be closed again.
    expect(useFixStore.getState().session).toBeNull();
    expect(editApi.dayFixClose).not.toHaveBeenCalled();
  });

  it('a wrong sign is told under the sign, nothing opens, and the notice stays', async () => {
    seedLine();
    vi.mocked(editApi.dayFixOpen).mockRejectedValue(new CommandError(DAY_FIX_OPEN_SIGN_WRONG));
    const { user } = await openUmarsBook(language, answer);
    const sales = salesGrid(language);
    await user.click(await fixButton());
    const dialog = await screen.findByRole('alertdialog');
    await user.type(
      within(dialog).getByRole('combobox', { name: named(message(language, 'edit-fix-label')) }),
      'wrong rate{Enter}',
    );
    await user.keyboard('munshi{Enter}');
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(editApi.dayFixOpen).mock.calls[0]?.[2]).toBe('munshi');
    expect(vi.mocked(editApi.dayFixOpen).mock.calls[0]?.[2]).not.toBe(OWNER_SIGN);
    // The words are in the box, the box is still open, and nothing was reopened.
    await waitFor(() => {
      expect(visible(dialog.textContent)).toContain(message(language, 'err-sign-wrong'));
    }, SLOW_MS);
    expect(useFixStore.getState().session).toBeNull();
    expect(textOf(sales, 0, 'buyerTotal')).toBe('');
    expect(editApi.dayFixClose).not.toHaveBeenCalled();
  });
});
