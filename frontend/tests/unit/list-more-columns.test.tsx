// The "More columns" tick of the list of بکری and of the پختہ کھاتہ register (plan 13, PRD FR-12.7,
// FR-13.6, Code Standards F22). The نگ column fits beside the others at no layout step, so the tick
// adds it, a less needed column steps aside where the room is short, the choice is kept for next
// time, and the tick is reached and changed with the keyboard. In the Urdu and the English app.
// The widths are added up in list-widths.test.ts.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as bikriApi from '../../src/features/bikri/api';
import * as pukhtaApi from '../../src/features/pukhta_khata/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeBikri } from '../helpers/bikri';
import { listRow, registerRow } from '../helpers/bikri-saved';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS, bikriListOf, pukhtaRegisterOf } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/bikri/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const LIST_KEY = 'mandi.filters.bikri.list_more_columns';
const REGISTER_KEY = 'mandi.filters.pukhta.more_columns';

/** The width of the window as the three steps see it: media queries that match up to a step. */
function stubStep(step: 'compact' | 'normal' | 'wide') {
  const matchesFor = (query: string) => {
    if (query.includes('1920')) return step === 'wide';
    if (query.includes('1366')) return step !== 'compact';
    return false;
  };
  vi.stubGlobal('matchMedia', (query: string) => ({
    matches: matchesFor(query),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  }));
  document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
  document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
}

const hasColumn = (field: string) =>
  document.querySelector(`.ag-header-cell[col-id="${field}"]`) !== null;
const cell = (rowId: string, field: string) =>
  visible(
    document.querySelector(`.ag-row[row-id="${rowId}"]`)?.querySelector(`[col-id="${field}"]`)
      ?.textContent,
  );
const tickOf = (language: Language) =>
  screen.getByRole('checkbox', { name: message(language, 'common-more-columns') });

beforeEach(() => {
  // Reset, not only clear: a rejection queued by a test that failed early must not reach the next.
  vi.resetAllMocks();
  window.localStorage.removeItem(LIST_KEY);
  window.localStorage.removeItem(REGISTER_KEY);
  healthyBackend();
  fakePartners();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.documentElement.style.removeProperty('--breakpoint-normal');
  document.documentElement.style.removeProperty('--breakpoint-wide');
  window.localStorage.removeItem(LIST_KEY);
  window.localStorage.removeItem(REGISTER_KEY);
  leaveAppWindow();
});

describe.each(LANGUAGES)('the list of بکری, More columns (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    // 45 of 50 sold: the cell says both, from the numbers the backend sent.
    vi.mocked(bikriApi.bikriList).mockResolvedValue(
      bikriListOf([listRow({ qty_sold: 45, qty_received: 50 })]),
    );
  });

  const open = async (step: 'compact' | 'normal' | 'wide') => {
    stubStep(step);
    await renderApp('/bikri/today');
    await waitFor(() => {
      expect(cell('41', 'number')).toBe('1');
    });
  };

  it('has no نگ column until the tick is on; then it shows sold and received', async () => {
    await open('normal');
    expect(hasColumn('qty')).toBe(false);
    const tick = tickOf(language);
    expect(tick).not.toBeChecked();
    await userEvent.setup().click(tick);
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(cell('41', 'qty')).toBe(message(language, 'bikri-list-qty', { sold: 45, received: 50 }));
    // The day and time step aside at the normal step; the amounts and the status stay.
    expect(hasColumn('when')).toBe(false);
    expect(hasColumn('gross')).toBe(true);
    expect(hasColumn('net')).toBe(true);
    expect(hasColumn('status')).toBe(true);
  });

  it('at the compact step خام steps aside and صافی stays', async () => {
    await open('compact');
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(hasColumn('gross')).toBe(false);
    expect(hasColumn('net')).toBe(true);
    expect(hasColumn('status')).toBe(true);
  });

  it('at the wide step it fits beside the truck, items and buyer total', async () => {
    await open('wide');
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    for (const field of ['truck', 'items', 'buyer_total', 'gross', 'net', 'status']) {
      expect(hasColumn(field), field).toBe(true);
    }
    expect(hasColumn('when')).toBe(false);
  });

  it('is reached with Tab, changed with Space, and unticking brings the other columns back', async () => {
    await open('normal');
    const user = userEvent.setup();
    const tick = tickOf(language);
    expect(tick.tabIndex).toBeGreaterThanOrEqual(0);
    tick.focus();
    await user.keyboard(' ');
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(tick).toHaveFocus();
    await user.keyboard(' ');
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(false);
    });
    expect(hasColumn('when')).toBe(true);
  });

  it('is remembered for the next time the list is opened', async () => {
    stubStep('normal');
    const first = await renderApp('/bikri/today');
    await waitFor(() => {
      expect(cell('41', 'number')).toBe('1');
    });
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(window.localStorage.getItem(LIST_KEY)).toBe('1');
    first.unmount();

    await renderApp('/bikri/today');
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(tickOf(language)).toBeChecked();
  });
});

describe.each(LANGUAGES)('the پختہ کھاتہ register, More columns (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeBikri();
    vi.mocked(pukhtaApi.pukhtaRegister).mockResolvedValue(
      pukhtaRegisterOf([registerRow({ qty: 10 })]),
    );
    vi.mocked(pukhtaApi.searchPukhtaPeople).mockResolvedValue([]);
  });

  const open = async (step: 'compact' | 'normal' | 'wide') => {
    stubStep(step);
    await renderApp('/bikri/pukhta');
    await waitFor(() => {
      expect(cell('0', 'bikri')).toBe('2');
    });
  };

  it('has no نگ column until the tick is on; then it shows the نگ of the line', async () => {
    await open('normal');
    expect(hasColumn('qty')).toBe(false);
    expect(hasColumn('phone')).toBe(true);
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(cell('0', 'qty')).toBe('10');
    // The phone steps aside at the normal step; the name, the amount and paid or due stay.
    expect(hasColumn('phone')).toBe(false);
    for (const field of ['date', 'bikri', 'name', 'amount', 'state']) {
      expect(hasColumn(field), field).toBe(true);
    }
  });

  it('at the compact step the date steps aside', async () => {
    await open('compact');
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(hasColumn('date')).toBe(false);
    for (const field of ['bikri', 'name', 'amount', 'state']) {
      expect(hasColumn(field), field).toBe(true);
    }
  });

  it('at the wide step only the commission steps aside', async () => {
    await open('wide');
    expect(hasColumn('commission')).toBe(true);
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(hasColumn('commission')).toBe(false);
    for (const field of ['date', 'bikri', 'side', 'name', 'phone', 'item', 'amount', 'state']) {
      expect(hasColumn(field), field).toBe(true);
    }
  });

  it('is remembered for the next time the register is opened', async () => {
    stubStep('normal');
    const first = await renderApp('/bikri/pukhta');
    await waitFor(() => {
      expect(cell('0', 'bikri')).toBe('2');
    });
    await userEvent.setup().click(tickOf(language));
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(window.localStorage.getItem(REGISTER_KEY)).toBe('1');
    first.unmount();

    await renderApp('/bikri/pukhta');
    await waitFor(() => {
      expect(hasColumn('qty')).toBe(true);
    });
    expect(tickOf(language)).toBeChecked();
  });
});
