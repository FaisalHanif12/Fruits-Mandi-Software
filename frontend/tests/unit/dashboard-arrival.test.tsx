// The garden goods that wait, in the attention bell (plan 33 version 1.1, PRD FR-7.44, FR-12.40): the
// list of what to do names a goods arrived entry as the most urgent record of the waiting stock, and
// its button opens the garden screen of that entry. The backend gives the id of the entry only; the
// screen asks which garden it is in. Urdu and English, against a fake backend.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AttentionList } from '../../src/bindings/AttentionList';
import * as dashboardApi from '../../src/features/dashboard/api';
import { destinationOfFirst } from '../../src/features/dashboard/destinations';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES } from '../../src/i18n';
import * as arrivals from '../helpers/arrivals-real';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { ATTENTION_18_10, fakeDashboard } from '../helpers/dashboard';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/dashboard/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/settings/maintenance-api');

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
});

afterEach(() => {
  leaveAppWindow();
});

/**
 * The attention list of the fixture with one more item: the stock that waits, whose most urgent
 * record is a garden's goods arrived entry.
 */
function withWaitingArrival(): AttentionList {
  return {
    ...ATTENTION_18_10,
    total_count: ATTENTION_18_10.total_count + 1,
    items: [
      ...ATTENTION_18_10.items,
      {
        kind: 'stock_waiting',
        count: 1,
        amount_paisa: null,
        qty: 100,
        days: 3,
        first: {
          target: 'arrival',
          id: 12,
          person_key: null,
          name_ur: 'کیلا',
          name_en: 'Banana',
        },
      },
    ],
  };
}

describe('the destination of the most urgent record', () => {
  it('a goods arrived entry is opened by its id', () => {
    expect(
      destinationOfFirst({
        target: 'arrival',
        id: 12,
        person_key: null,
        name_ur: 'کیلا',
        name_en: 'Banana',
      }),
    ).toEqual({ kind: 'arrival', id: 12 });
  });
});

describe.each(LANGUAGES)('the bell with garden goods waiting (%s)', (language) => {
  async function openBell() {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeDashboard({ attention: withWaitingArrival() });
    const view = await renderApp('/bikri/today');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', {
        name: (name) =>
          visible(name) ===
          message(language, 'attention-bell-label', { count: ATTENTION_18_10.total_count + 1 }),
      }),
    );
    return { ...view, user };
  }

  /** The "most urgent" button of the waiting stock: the one that names the item. */
  const firstButton = async () => {
    const item = language === 'ur' ? 'کیلا' : 'Banana';
    const buttons = await screen.findAllByRole('button', {
      name: (name) =>
        visible(name).startsWith(message(language, 'attention-open-first', { name: '' }).trim()),
    });
    const found = buttons.find((button) => visible(button.textContent).includes(item));
    if (found === undefined) throw new Error('the waiting stock has no first record');
    return found;
  };

  it('the record is a button with the item, and it opens the garden the entry is in', async () => {
    vi.mocked(gardensApi.gardensArrivalGet).mockResolvedValue(arrivals.ARRIVAL_AFTER_A_SALE);
    const { router, user } = await openBell();
    const first = await firstButton();
    await user.click(first);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens/1');
    });
    expect(gardensApi.gardensArrivalGet).toHaveBeenCalledWith(12);
    expect(dashboardApi.assistAttention).toHaveBeenCalled();
  });

  it('if the entry cannot be read, the list of goods left opens instead', async () => {
    vi.mocked(gardensApi.gardensArrivalGet).mockRejectedValue(new Error('gone'));
    const { router, user } = await openBell();
    await user.click(await firstButton());
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/bikri/stock');
    });
  });
});
