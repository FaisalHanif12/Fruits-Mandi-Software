// The open gardens and trips under a partner's card on the overview (plan 17c, PRD 17.3, FR-12.29,
// E2E-40), in the Urdu and the English app, against a fake backend: a partner in a garden shows it
// with his %, what he paid from the shop and from his pocket, his money back and the running
// balance, all as the backend sent them; a partner in none shows nothing; the garden opens from its
// name. The screen adds nothing up (CLAUDE.md rule 1).

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { PartnerVentureCard } from '../../src/bindings/PartnerVentureCard';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { paisaForTest } from '../helpers/paisa';
import { named } from '../helpers/bikri-screen';
import {
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardens } from '../helpers/gardens';
import * as real from '../helpers/gardens-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, overviewRow, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/gardens/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const money = (rupees: number) => paisaForTest(rupees * 100);

const GARDEN_CARD: PartnerVentureCard = {
  venture_id: 1,
  kind: 'garden',
  number: 1,
  code: '0080',
  name_ur: 'شاہ پور باغ',
  name_en: 'Shahpur Garden',
  share_bps: 6000,
  paid_shop_paisa: money(33_000),
  paid_pocket_paisa: money(0),
  money_back_paisa: money(0),
  balance_paisa: money(11_400),
};

const TRIP_CARD: PartnerVentureCard = {
  venture_id: 3,
  kind: 'trip',
  number: 3,
  code: '0072',
  name_ur: '0072 12-10-2026',
  name_en: '0072 12-10-2026',
  share_bps: 5000,
  paid_shop_paisa: money(0),
  paid_pocket_paisa: money(50_000),
  money_back_paisa: money(52_640),
  balance_paisa: money(5_280),
};

beforeEach(() => {
  // "More columns" is kept on this computer; a test starts with it off.
  window.localStorage.clear();
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('open gardens and trips under a partner card (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeGardens();
  });

  const nameOf = (language_: Language, ur: string, en: string) => (language_ === 'ur' ? ur : en);

  async function open() {
    const fake = fakePartners(THREE_PARTNERS);
    const [a, b, c] = THREE_PARTNERS;
    if (a === undefined || b === undefined || c === undefined) throw new Error('three partners');
    fake.overview = [
      overviewRow(a, { open_ventures: [GARDEN_CARD, TRIP_CARD] }),
      overviewRow(b),
      overviewRow(c),
    ];
    const view = await renderApp('/partners');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-title')),
    });
    return { ...view, user: userEvent.setup() };
  }

  /** The card of the open gardens and trips of a partner: his name is its title (plan 38). */
  async function cardOf(name: string): Promise<HTMLElement> {
    // The name is also the heading of his row in the table, so the card is the one that is a section.
    const found = await screen.findAllByText(name);
    const card = found.map((one) => one.closest('section')).find((one) => one !== null);
    if (!(card instanceof HTMLElement)) throw new Error('no card');
    return card;
  }

  /** Whether a partner has a card of open gardens and trips under the table. */
  async function hasCard(name: string): Promise<boolean> {
    // Wait for the table (his row), then look for a section with his name as the title.
    const found = await screen.findAllByText(name);
    return found.some((one) => one.closest('section') !== null);
  }

  it('shows the garden and the trip a partner is in, with his % and his money, as the backend sent them', async () => {
    await open();
    const card = await cardOf(nameOf(language, 'الف', 'Partner A'));
    const table = await within(card).findByRole('table', {
      name: named(
        message(language, 'partners-ventures-label', {
          name: nameOf(language, 'الف', 'Partner A'),
        }),
      ),
    });
    const text = visible(table.textContent);
    expect(text).toContain(language === 'ur' ? 'شاہ پور باغ' : 'Shahpur Garden');
    expect(text).toContain('0080');
    for (const amount of ['60%', '33,000', '11,400', '50%', '50,000', '52,640', '5,280']) {
      expect(text).toContain(amount);
    }
    expect(text).toContain(message(language, 'glossary-bagh'));
    expect(text).toContain(message(language, 'glossary-safar'));
  });

  it('a partner in no garden or trip has no list under his card', async () => {
    await open();
    expect(await hasCard(nameOf(language, 'ب', 'Partner B'))).toBe(false);
    expect(
      screen.queryByRole('table', {
        name: named(
          message(language, 'partners-ventures-label', {
            name: nameOf(language, 'ب', 'Partner B'),
          }),
        ),
      }),
    ).toBeNull();
  });

  it('the garden opens from its name', async () => {
    const { router, user } = await open();
    const card = await cardOf(nameOf(language, 'الف', 'Partner A'));
    await user.click(
      await within(card).findByRole('button', {
        name: (name) => visible(name).includes('0080'),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens/1');
    });
  });

  it('the trip opens from its name', async () => {
    const { router, user } = await open();
    const card = await cardOf(nameOf(language, 'الف', 'Partner A'));
    await user.click(
      await within(card).findByRole('button', {
        name: (name) => visible(name).includes('0072 12-10-2026'),
      }),
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/trips/3');
    });
  });
});

describe.each(LANGUAGES)('the overview with the backend’s real answers (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    fakeGardens();
  });

  const nameOf = (ur: string, en: string) => (language === 'ur' ? ur : en);

  async function openWith(rows: typeof real.PARTNERS_OVERVIEW_WITH_GARDEN_CARDS) {
    const fake = fakePartners(THREE_PARTNERS);
    fake.overview = rows;
    await renderApp('/partners');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-title')),
    });
  }

  /** The card of the open gardens and trips of a partner: his name is its title (plan 38). */
  async function cardOf(name: string): Promise<HTMLElement> {
    // The name is also the heading of his row in the table, so the card is the one that is a section.
    const found = await screen.findAllByText(name);
    const card = found.map((one) => one.closest('section')).find((one) => one !== null);
    if (!(card instanceof HTMLElement)) throw new Error('no card');
    return card;
  }

  /** Whether a partner has a card of open gardens and trips under the table. */
  async function hasCard(name: string): Promise<boolean> {
    // Wait for the table (his row), then look for a section with his name as the title.
    const found = await screen.findAllByText(name);
    return found.some((one) => one.closest('section') !== null);
  }

  /** The row of a partner in the table: what holds his name and his figures. */
  async function rowOf(name: string): Promise<HTMLElement> {
    const found = await screen.findAllByText(name);
    const row = found.map((one) => one.closest('tr')).find((one) => one !== null);
    if (!(row instanceof HTMLElement)) throw new Error('no row');
    return row;
  }

  it('E2E-40 before the closing: A is in the garden with 60%, 33,000 from the shop and a balance of 11,400', async () => {
    await openWith(real.PARTNERS_OVERVIEW_WITH_GARDEN_CARDS);
    const a = await cardOf(nameOf('الف', 'Partner A'));
    const table = await within(a).findByRole('table');
    const text = visible(table.textContent);
    for (const figure of ['60%', '33,000', '11,400']) expect(text).toContain(figure);
    // His کل رقم is the backend's, −29,800 with the commission share of the sale in it.
    expect(visible((await rowOf(nameOf('الف', 'Partner A'))).textContent)).toContain('-29,800');
    const b = await cardOf(nameOf('ب', 'Partner B'));
    expect(visible((await within(b).findByRole('table')).textContent)).toContain('22,000');
    expect(await hasCard(nameOf('ج', 'Partner C'))).toBe(false);
  });

  it('after the closing no card shows a garden any more, and A’s total is 10,040', async () => {
    await openWith(real.PARTNERS_OVERVIEW_AFTER_GARDEN_CLOSE);
    expect(await hasCard(nameOf('الف', 'Partner A'))).toBe(false);
    const user = userEvent.setup();
    // The month's garden figure is among the other figures: tick "More columns" to see it.
    await user.click(
      await screen.findByRole('checkbox', {
        name: named(message(language, 'common-more-columns')),
      }),
    );
    const row = await rowOf(nameOf('الف', 'Partner A'));
    const figures = visible(`${row.textContent}${row.nextElementSibling?.textContent ?? ''}`);
    expect(figures).toContain('10,040');
    expect(figures).toContain('6,840');
  });
});
