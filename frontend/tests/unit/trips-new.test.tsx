// A new trip (plan 16b, PRD 12.10, FR-12.29, A56), in the Urdu and the English app, against a fake
// backend: the shares first (filled from the defaults, with the backend's running total), then the
// source of each partner, then the details; the refusals at their fields; the keyboard-only path;
// and a language switch in the middle of the entry that loses nothing (Code Standards F30, F39).

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as tripsApi from '../../src/features/trips/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { useLanguageStore } from '../../src/shared/i18n';
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
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { fakeTrips, tripPartners } from '../helpers/trips';
import { refusal } from '../helpers/ui-error';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/trips/api');
vi.mock('../../src/features/purchases/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;

describe.each(LANGUAGES)('a new trip (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const names = {
    a: language === 'ur' ? 'الف' : 'Partner A',
    b: language === 'ur' ? 'ب' : 'Partner B',
    c: language === 'ur' ? 'ج' : 'Partner C',
  };
  const shareBox = (name: string) =>
    screen.getByRole('textbox', {
      name: named(message(language, 'ventures-share-of', { name })),
    });
  const sourceGroup = (name: string) =>
    screen.getByRole('group', { name: named(message(language, 'ventures-source-of', { name })) });

  const open = async () => {
    const view = await renderApp('/trips/new');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'trips-new-title')),
    });
    // The form waits for the defaults and the partners.
    await screen.findByText(said(message(language, 'trips-new-firm-shares')));
    return { ...view, user: userEvent.setup() };
  };

  const saveButton = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).includes(message(language, 'trips-new-save')),
    });

  it('starts filled from the defaults with the shares first, the total from the backend', async () => {
    fakeTrips();
    await open();
    expect(shareBox(names.a)).toHaveValue('40');
    expect(shareBox(names.b)).toHaveValue('30');
    expect(shareBox(names.c)).toHaveValue('30');
    // Each partner starts from the shop's cash.
    for (const name of Object.values(names)) {
      const shop = within(sourceGroup(name)).getByRole('button', {
        name: message(language, 'ventures-source-shop'),
      });
      expect(shop).toHaveAttribute('aria-pressed', 'true');
    }
    expect(
      await screen.findByText(said(message(language, 'partners-total-ok'))),
    ).toBeInTheDocument();
    expect(saveButton()).toBeEnabled();
    // The order of the page: the shares, then the details.
    const headings = screen
      .getAllByRole('heading', { level: 2 })
      .map((h) => visible(h.textContent));
    expect(headings.indexOf(message(language, 'trips-new-shares-title'))).toBeLessThan(
      headings.indexOf(message(language, 'trips-new-details-title')),
    );
  });

  it('says where the form came from: the last trip, or the firm shares', async () => {
    fakeTrips({
      defaults: { partners: tripPartners(['shop', 'pocket', 'shop']), from_last_trip: true },
    });
    await renderApp('/trips/new');
    expect(
      await screen.findByText(said(message(language, 'trips-new-from-last'))),
    ).toBeInTheDocument();
    // The source of the last trip is filled in.
    const pocket = within(sourceGroup(names.b)).getByRole('button', {
      name: message(language, 'ventures-source-pocket'),
    });
    expect(pocket).toHaveAttribute('aria-pressed', 'true');
  });

  it('shares that do not total 100% show the total in red with words and block the save', async () => {
    fakeTrips();
    const { user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    await user.clear(shareBox(names.c));
    await user.type(shareBox(names.c), '20');
    expect(
      await screen.findByText(said(message(language, 'partners-total-bad'))),
    ).toBeInTheDocument();
    expect(saveButton()).toBeDisabled();
  });

  it('a partner with no share is not in the trip: his source choice goes and 0 is sent', async () => {
    const fake = fakeTrips();
    const { user } = await open();
    await user.clear(shareBox(names.c));
    await user.clear(shareBox(names.b));
    await user.type(shareBox(names.b), '60');
    expect(screen.getAllByText(said(message(language, 'ventures-not-in')))).toHaveLength(1);
    await waitFor(() => {
      expect(saveButton()).toBeEnabled();
    });
    await user.click(saveButton());
    await waitFor(() => {
      expect(fake.saved.trips).toHaveLength(1);
    });
    expect(fake.saved.trips[0]?.partners).toEqual([
      { partner_id: 1, share: '40', source: 'shop' },
      { partner_id: 2, share: '60', source: 'shop' },
      { partner_id: 3, share: '0', source: 'shop' },
    ]);
  });

  it('saves the shares, the sources and the details as typed and opens the trip', async () => {
    const fake = fakeTrips();
    const { router, user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    await user.click(
      within(sourceGroup(names.b)).getByRole('button', {
        name: message(language, 'ventures-source-pocket'),
      }),
    );
    await user.type(
      screen.getByRole('combobox', { name: named(message(language, 'trips-field-market')) }),
      'Sargodha',
    );
    await user.type(
      screen.getByRole('combobox', { name: named(message(language, 'trips-field-sent')) }),
      'Aslam',
    );
    await user.click(saveButton());
    await waitFor(() => {
      expect(fake.saved.trips).toHaveLength(1);
    });
    expect(fake.saved.trips[0]).toEqual({
      date: '10-10-2026',
      market: 'Sargodha',
      sent_person: 'Aslam',
      note: null,
      partners: [
        { partner_id: 1, share: '40', source: 'shop' },
        { partner_id: 2, share: '30', source: 'pocket' },
        { partner_id: 3, share: '30', source: 'shop' },
      ],
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toMatch(/^\/trips\/\d+$/);
    });
  });

  it('the keyboard alone does it: type in the market, F10 saves', async () => {
    const fake = fakeTrips();
    const { user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    const market = screen.getByRole('combobox', {
      name: named(message(language, 'trips-field-market')),
    });
    market.focus();
    await user.keyboard('Sahiwal');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.trips).toHaveLength(1);
    });
    expect(fake.saved.trips[0]?.market).toBe('Sahiwal');
  });

  it('a refusal shows at the field and in words, and nothing typed is lost', async () => {
    const fake = fakeTrips();
    fake.refuse.trip = [
      { code: 'ERR_SHARES_NOT_100', field: 'partners', values: { total_bps: 9000 } },
      { code: 'ERR_DATE_CLOSED', field: 'date', values: { closed_up_to: '09-10-2026' } },
    ];
    const { user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    await user.type(
      screen.getByRole('combobox', { name: named(message(language, 'trips-field-market')) }),
      'Okara',
    );
    await user.click(saveButton());
    expect(
      await screen.findAllByText(/./, { selector: '[role="alert"], [role="alert"] *' }),
    ).not.toHaveLength(0);
    expect(
      screen.getByRole('combobox', { name: named(message(language, 'trips-field-market')) }),
    ).toHaveValue('Okara');
    expect(shareBox(names.a)).toHaveValue('40');
  });

  it('a language switch in the middle of the entry keeps every value and the choice of source', async () => {
    fakeTrips();
    const { user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    await user.click(
      within(sourceGroup(names.c)).getByRole('button', {
        name: message(language, 'ventures-source-pocket'),
      }),
    );
    await user.type(
      screen.getByRole('combobox', { name: named(message(language, 'trips-field-market')) }),
      'Multan',
    );
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const otherNames = {
      a: other === 'ur' ? 'الف' : 'Partner A',
      c: other === 'ur' ? 'ج' : 'Partner C',
    };
    await waitFor(() => {
      expect(
        screen.getByRole('textbox', {
          name: named(message(other, 'ventures-share-of', { name: otherNames.a })),
        }),
      ).toHaveValue('40');
    });
    expect(
      screen.getByRole('combobox', { name: named(message(other, 'trips-field-market')) }),
    ).toHaveValue('Multan');
    const pocket = within(
      screen.getByRole('group', {
        name: named(message(other, 'ventures-source-of', { name: otherNames.c })),
      }),
    ).getByRole('button', { name: message(other, 'ventures-source-pocket') });
    expect(pocket).toHaveAttribute('aria-pressed', 'true');
  });

  it('while the defaults load there is a loading state, and a failure has a retry', async () => {
    fakeTrips();
    vi.mocked(tripsApi.tripsDefaults).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    const { user } = await (async () => {
      const view = await renderApp('/trips/new');
      return { ...view, user: userEvent.setup() };
    })();
    expect(
      await screen.findByText(said(message(language, 'trips-load-failed'))),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(
      await screen.findByText(said(message(language, 'trips-new-firm-shares'))),
    ).toBeInTheDocument();
  });

  it('with no partner yet it says so and links to the partner setup', async () => {
    fakeTrips({ defaults: { partners: [], from_last_trip: false } });
    await renderApp('/trips/new');
    expect(
      await screen.findByText(said(message(language, 'trips-new-no-partners'))),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: named(message(language, 'trips-new-no-partners-link')) }),
    ).toBeInTheDocument();
  });
});
