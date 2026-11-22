// Plan 95 items 15 and 16 (PRD FR-5.3 v1.57, FR-7.17, FR-15.10): وصولی, ادائیگی and پیشگی are three tabs of the روکڑ
// area that open the one voucher screen with their type already chosen, and that screen shows the روکڑ tabs with the
// right one open; the combined "وصولی / ادائیگی / پیشگی کھاتہ" tab is gone. The بکری tabs are بکری بک, پارٹی کھاتے,
// پچھلی بکری and باقی مال. The keys: ← → move between the tabs in the reading direction, Enter opens, ↓ goes into the
// screen; Alt+V, Alt+A and Alt+P still open the voucher screen. In the Urdu and in the English app.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as reportsApi from '../../src/features/reports/api';
import { LANGUAGES } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakeMasters } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { at } from '../helpers/sample';
import { AKRAM, ARSHAD, CHUKTA_KHATA, KARIM, RAHIM, fakeVouchers } from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(reportsApi.reportsListPrinters).mockResolvedValue([]);
  const rows = [AKRAM, ARSHAD, CHUKTA_KHATA, RAHIM, KARIM];
  fakeAccounts(rows);
  fakeAccountSearch(rows);
  fakeMasters({});
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the voucher tabs of روکڑ (%s)', (language) => {
  const m = (id: string) => message(language, id);
  // In Urdu the left arrow reads on; in English the right arrow does.
  const next = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
  const back = language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}';

  beforeEach(() => {
    window.localStorage.clear();
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeVouchers();
  });

  async function open(type: 'receipt' | 'payment' | 'advance') {
    const app = await renderApp(`/bikri/voucher?type=${type}`);
    await screen.findByRole('navigation', { name: named(m('layout-rokar-label')) });
    return { ...app, user: userEvent.setup() };
  }

  const rokarNav = () => screen.getByRole('navigation', { name: named(m('layout-rokar-label')) });
  const tabs = () => within(rokarNav()).getAllByRole('button');
  const openTab = () => tabs().find((tab) => tab.getAttribute('aria-current') === 'page');

  it('the voucher screen shows the روکڑ tabs, with the combined tab gone and the type of the address open', async () => {
    await open('payment');
    expect(tabs().map((tab) => visible(tab.textContent))).toEqual([
      m('layout-rokar-cashbook'),
      m('layout-rokar-receipt'),
      m('layout-rokar-payment'),
      m('layout-rokar-advance'),
      m('layout-rokar-expense'),
      m('layout-rokar-lent'),
      m('layout-rokar-khata'),
      m('layout-rokar-general'),
    ]);
    expect(visible(openTab()?.textContent)).toBe(m('layout-rokar-payment'));
    // The three tabs say the type only; there is no "وصولی / ادائیگی / پیشگی کھاتہ" tab.
    for (const tab of tabs()) expect(visible(tab.textContent)).not.toContain('/');
    // The بکری tabs are not on this screen, and the main menu marks روکڑ, not بکری.
    expect(screen.queryByRole('navigation', { name: named(m('layout-module-label')) })).toBeNull();
    const main = screen.getByRole('navigation', { name: named(m('layout-nav-label')) });
    const current = within(main)
      .getAllByRole('button')
      .find((item) => item.getAttribute('aria-current') === 'page');
    expect(visible(current?.textContent)).toBe(m('layout-nav-rokar'));
  });

  it.each([
    ['receipt', 1],
    ['payment', 2],
    ['advance', 3],
  ] as const)(
    'the %s tab opens the voucher screen with that type, and it is the open tab',
    async (type, index) => {
      const { user, router } = await open(type === 'receipt' ? 'payment' : 'receipt');
      await user.click(at(tabs(), index));
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/bikri/voucher');
        expect(router.state.location.search).toMatchObject({ type });
      });
      await waitFor(() => {
        expect(visible(openTab()?.textContent)).toBe(m(`layout-rokar-${type}`));
      });
      // The voucher screen's own title says the same type.
      expect(
        await screen.findByRole('heading', { level: 1, name: named(m(`vouchers-${type}-title`)) }),
      ).toBeInTheDocument();
    },
  );

  it('← and → move between the tabs, Enter opens the one with focus, and ↓ goes into the screen', async () => {
    const { user, router } = await open('receipt');
    at(tabs(), 1).focus();
    await user.keyboard(next);
    expect(at(tabs(), 2)).toHaveFocus();
    await user.keyboard(next);
    expect(at(tabs(), 3)).toHaveFocus();
    await user.keyboard(back);
    expect(at(tabs(), 2)).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ type: 'payment' });
    });
    await waitFor(() => {
      expect(visible(openTab()?.textContent)).toBe(m('layout-rokar-payment'));
    });
    // ↓ leaves the tabs for the first field of the screen, never for another tab.
    at(tabs(), 2).focus();
    await user.keyboard('{ArrowDown}');
    expect(rokarNav().contains(document.activeElement)).toBe(false);
  });

  it.each([
    ['KeyV', 'receipt'],
    ['KeyA', 'payment'],
    ['KeyP', 'advance'],
  ] as const)('%s with Alt still opens the %s voucher with its tab open', async (_code, type) => {
    const { user } = await open(type === 'advance' ? 'receipt' : 'advance');
    const key = { KeyV: 'v', KeyA: 'a', KeyP: 'p' }[_code];
    await user.keyboard(`{Alt>}${key}{/Alt}`);
    await waitFor(() => {
      expect(visible(openTab()?.textContent)).toBe(m(`layout-rokar-${type}`));
    });
  });

  it('the cash book tab leads back to روکڑ, and the voucher tabs sit between it and خرچہ', async () => {
    const { user, router } = await open('advance');
    const order = tabs().map((tab) => visible(tab.textContent));
    expect(order.indexOf(m('layout-rokar-cashbook'))).toBe(0);
    expect(order.indexOf(m('layout-rokar-expense'))).toBe(4);
    await user.click(at(tabs(), 0));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/rokar');
    });
  });
});
