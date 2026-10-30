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
  // "More columns" is kept on this computer; a test starts with it off.
  window.localStorage.clear();
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

  /**
   * The row of a partner in the table, with the row of his other figures under it when "More
   * columns" is ticked: what holds his name and his figures (plan 38).
   */
  async function cardOf(name: string): Promise<HTMLElement> {
    const found = await screen.findAllByText(name);
    const row = found.map((one) => one.closest('tr')).find((one) => one !== null);
    if (!(row instanceof HTMLElement)) throw new Error('no row');
    const wrapper = document.createElement('div');
    wrapper.append(row.cloneNode(true));
    const more = row.nextElementSibling;
    if (more instanceof HTMLElement && more.hasAttribute('data-partner-more')) {
      wrapper.append(more.cloneNode(true));
    }
    return wrapper;
  }

  /** Ticks "More columns": the other figures of every partner are drawn under his row. */
  async function showMore(user: ReturnType<typeof userEvent.setup>) {
    await user.click(
      await screen.findByRole('checkbox', {
        name: named(message(language, 'common-more-columns')),
      }),
    );
  }

  it('shows returned, خالص, capital put in, the trips column and کل رقم for each partner', async () => {
    const { user } = await open();
    await showMore(user);
    // The headings of the table and the labels of the other figures under each row.
    const table = await screen.findByRole('table', {
      name: named(message(language, 'partners-overview-label')),
    });
    for (const id of [
      'partners-metric-returned',
      'partners-metric-net',
      'partners-metric-capital-in',
      'partners-metric-ventures',
      'partners-metric-total',
    ]) {
      expect(visible(table.textContent)).toContain(message(language, id));
    }
    const a = await cardOf(nameOf(language, 'الف', 'Partner A'));
    const text = visible(a.textContent);
    // E2E-9 and E2E-37: his profit share 996, taken 2,000, خالص and کل رقم -1,004.
    expect(text).toContain('996');
    expect(text).toContain('2,000');
    expect(text).toContain('-1,004');
    const b = await cardOf(nameOf(language, 'ب', 'Partner B'));
    expect(visible(b.textContent)).toContain('50,746');
    const c = await cardOf(nameOf(language, 'ج', 'Partner C'));
    expect(visible(c.textContent)).toContain('746');
  });

  it('puts the کل رقم first among the figures, and a negative one reads in the due colour with its minus sign', async () => {
    await open();
    const table = await screen.findByRole('table', {
      name: named(message(language, 'partners-overview-label')),
    });
    const heads = within(table)
      .getAllByRole('columnheader')
      .map((head) => visible(head.textContent));
    expect(heads[1]).toBe(message(language, 'partners-metric-total'));
    const a = await cardOf(nameOf(language, 'الف', 'Partner A'));
    const total = a.querySelector('td');
    expect(visible(total?.textContent)).toBe('-1,004');
    expect(total?.querySelector('.text-due')).not.toBeNull();
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
