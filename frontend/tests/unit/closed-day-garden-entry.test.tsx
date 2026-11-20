// A new garden entry on a closed day (plan 85, PRD FR-21.4 v1.49), in the Urdu and the English app: the garden's
// entry form shows the shared closed-day notice with "Reopen the day and save" when the backend refuses the date
// (`ERR_DATE_CLOSED` on `date`), the reason and the Owner's sign reopen the day, the save posts, and the days
// close again with the token of the reopen. The garden's entry form and the trip's entry form give the same shared
// piece the same refusals, so one stands for both here.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as editApi from '../../src/shared/edit/api';
import { useFixStore } from '../../src/shared/edit/fix-store';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { DAY_FIX_CLOSE_E2E_45, DAY_FIX_OPEN_E2E_45 } from '../helpers/edit-real';
import { confirmFixDay } from '../helpers/fix-day';
import { fakeGardens } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');
vi.mock('../../src/shared/edit/api');

const DAY = '10-10-2026';

/** True from the moment the double has opened the day. */
let opened = false;

const closedDay = () =>
  new CommandError(
    refusal([
      {
        code: 'ERR_DATE_CLOSED',
        field: 'date',
        values: { closed_up_to: '10-10-2026', today: '10-10-2026', today_open: 0 },
      },
    ]),
  );

beforeEach(() => {
  vi.resetAllMocks();
  opened = false;
  useFixStore.getState().end();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
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

const alertsText = () =>
  screen.queryAllByRole('alert').map((element) => visible(element.textContent));

describe.each(LANGUAGES)('a new garden entry on a closed day (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeGardens();
    const preview = vi.mocked(gardensApi.gardensEntryPreview).getMockImplementation();
    const save = vi.mocked(gardensApi.gardensEntrySave).getMockImplementation();
    if (preview === undefined || save === undefined) throw new Error('no garden fakes');
    vi.mocked(gardensApi.gardensEntryPreview).mockImplementation(async (...args) => {
      const reply = await preview(...args);
      return opened
        ? reply
        : { ...reply, errors: [{ code: 'ERR_DATE_CLOSED', field: 'date', values: {} }] };
    });
    vi.mocked(gardensApi.gardensEntrySave).mockImplementation((...args) =>
      opened ? save(...args) : Promise.reject(closedDay()),
    );
  });

  const closedWords = () => message(language, 'edit-closed-entry', { date: DAY });
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };

  it('shows the notice, reopens with the reason and the sign, and the save closes the days again', async () => {
    await renderApp('/gardens/1');
    await screen.findByText(
      (content) => visible(content) === message(language, 'gardens-detail-title', { code: '0080' }),
    );
    const user = userEvent.setup();
    await user.type(field('gardens-installment-amount'), '30000');
    await user.type(field('gardens-payment-reason'), 'First installment');
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(true);
    });
    await user.click(
      await screen.findByRole('button', { name: named(message(language, 'edit-action-fix-save')) }),
    );
    await confirmFixDay(language, user, DAY);
    await waitFor(() => {
      expect(alertsText().some((text) => text.includes(closedWords()))).toBe(false);
    });
    expect(gardensApi.gardensEntrySave).not.toHaveBeenCalled();
    press('F10');
    await waitFor(() => {
      expect(gardensApi.gardensEntrySave).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(editApi.dayFixClose).toHaveBeenCalledTimes(1);
    });
    expect(editApi.dayFixClose).toHaveBeenCalledWith(DAY_FIX_OPEN_E2E_45.token);
  });
});
