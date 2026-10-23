// The partner overview of plan 17 (F2, PRD 17.3, FR-17.9, C20, C22), in the Urdu and in the English
// app, against a fake backend: every card shows the returned, خالص, capital put in, the باغ، سفر
// اور پھل column and his کل رقم as the backend sent them (E2E-37: -1,004 / 50,746 / 746), and two
// buttons open his statement and his capital. The screen adds nothing up (CLAUDE.md rule 1).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { named } from '../helpers/cashbook-screen';
import { leaveAppWindow, renderApp } from '../helpers/app';
import { preparePartnerMoney } from '../helpers/partner-money';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the partner overview, plan 17 columns (%s)', (language) => {
  beforeEach(() => {
    preparePartnerMoney(language);
  });

  async function open() {
    const app = await renderApp('/partners');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-title')),
    });
    return { ...app, user: userEvent.setup() };
  }

  /** The card of a partner: the region that holds his name and his figures. */
  async function cardOf(name: string): Promise<HTMLElement> {
    const heading = await screen.findByText(name);
    const card = heading.closest('section');
    if (!(card instanceof HTMLElement)) throw new Error('no card');
    return card;
  }

  it('shows returned, خالص, capital put in, the trips column and کل رقم for each partner', async () => {
    await open();
    const a = await cardOf(nameOf(language, 'الف', 'Partner A'));
    const text = visible(a.textContent);
    for (const id of [
      'partners-metric-returned',
      'partners-metric-net',
      'partners-metric-capital-in',
      'partners-metric-ventures',
      'partners-metric-total',
    ]) {
      expect(text).toContain(message(language, id));
    }
    // E2E-9 and E2E-37: his profit share 996, taken 2,000, خالص and کل رقم -1,004.
    expect(text).toContain('996');
    expect(text).toContain('2,000');
    expect(text).toContain('-1,004');
    const b = await cardOf(nameOf(language, 'ب', 'Partner B'));
    expect(visible(b.textContent)).toContain('50,746');
    const c = await cardOf(nameOf(language, 'ج', 'Partner C'));
    expect(visible(c.textContent)).toContain('746');
  });

  it('a negative total reads in the due colour and with its minus sign', async () => {
    await open();
    const a = await cardOf(nameOf(language, 'الف', 'Partner A'));
    const total = within(a).getByText(named(message(language, 'partners-metric-total')));
    const value = total.closest('div')?.querySelector('dd');
    expect(visible(value?.textContent)).toBe('-1,004');
    expect(value?.querySelector('.text-due')).not.toBeNull();
  });

  it("the buttons open the partner's statement and his capital", async () => {
    const { user } = await open();
    const name = nameOf(language, 'ب', 'Partner B');
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'partners-open-statement-of', { name })),
      }),
    );
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-statement-title')),
    });
    await waitFor(() => {
      expect(partnersApi.partnersStatement).toHaveBeenCalledWith(
        expect.objectContaining({ partner_id: 2 }),
      );
    });
  });

  it('the capital button of a card opens the capital screen with that partner chosen', async () => {
    const { user } = await open();
    const name = nameOf(language, 'ب', 'Partner B');
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'partners-open-capital-of', { name })),
      }),
    );
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-capital-screen-title')),
    });
    await waitFor(() => {
      expect(partnersApi.capitalHistory).toHaveBeenCalledWith(
        expect.objectContaining({ partner_id: 2 }),
      );
    });
  });
});
