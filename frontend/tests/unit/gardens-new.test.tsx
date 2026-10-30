// A new garden (plan 17c, PRD 12.11, FR-12.29, FR-12.34, A56), in the Urdu and the English app,
// against a fake backend: the shares first (filled from the defaults, with the backend's running
// total), then the source of each partner with the words that a pocket part posts nothing, then the
// details with both names; each partner's part of the first installment before the save, the cash
// question, the refusals at their fields, the keyboard-only path, and a language switch in the
// middle of the entry that loses nothing (Code Standards F30, F39).

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as gardensApi from '../../src/features/gardens/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import { openInfo } from '../helpers/info';
import {
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeGardenItemPicker, fakeGardens } from '../helpers/gardens';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { refusal } from '../helpers/ui-error';
import { settlePreview } from '../helpers/preview-settled';

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

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeGardenItemPicker();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

const said = (text: string) => (content: string) => visible(content) === text;
type User = ReturnType<typeof userEvent.setup>;

describe.each(LANGUAGES)('a new garden (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const names = {
    a: language === 'ur' ? 'الف' : 'Partner A',
    b: language === 'ur' ? 'ب' : 'Partner B',
    c: language === 'ur' ? 'ج' : 'Partner C',
  };
  const URDU_NAME = 'شاہ پور باغ';
  const ENGLISH_NAME = 'Shahpur Garden';
  const shareBox = (name: string) =>
    screen.getByRole('textbox', {
      name: named(message(language, 'ventures-share-of', { name })),
    });
  const sourceGroup = (name: string) =>
    screen.getByRole('group', { name: named(message(language, 'ventures-source-of', { name })) });
  const choose = async (user: User, name: string, source: 'shop' | 'pocket') => {
    await user.click(
      within(sourceGroup(name)).getByRole('button', {
        name: message(language, `ventures-source-${source}`),
      }),
    );
  };
  /** A text field by its label: a plain box, or the Urdu text field (a combobox). */
  const field = (id: string) => {
    const label = named(message(language, id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };

  const open = async () => {
    const view = await renderApp('/gardens/new');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'gardens-new-title')),
    });
    await screen.findByText(said(message(language, 'gardens-new-shares-title')));
    return { ...view, user: userEvent.setup() };
  };

  const saveButton = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).includes(message(language, 'gardens-new-save')),
    });

  /** Types both names, the Urdu one in its field and the English one in its own. */
  async function typeNames(user: User) {
    const urdu = screen.getByLabelText<HTMLInputElement>(
      named(message(language, 'gardens-field-name-ur')),
    );
    const english = screen.getByLabelText<HTMLInputElement>(
      named(message(language, 'gardens-field-name-en')),
    );
    await user.click(urdu);
    await user.keyboard(URDU_NAME);
    await user.click(english);
    await user.clear(english);
    await user.keyboard(ENGLISH_NAME);
  }

  /** The garden of E2E-40: A 60% from the shop, B 40% from his pocket, C not in it. */
  async function fillE40(user: User) {
    await user.clear(shareBox(names.a));
    await user.type(shareBox(names.a), '60');
    await user.clear(shareBox(names.b));
    await user.type(shareBox(names.b), '40');
    await user.clear(shareBox(names.c));
    await choose(user, names.b, 'pocket');
    await typeNames(user);
    await user.type(field('gardens-field-rent'), '50000');
  }

  it('starts filled from the defaults with the shares first, the total from the backend', async () => {
    fakeGardens();
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
      .map((heading) => visible(heading.textContent));
    expect(headings.indexOf(message(language, 'gardens-new-shares-title'))).toBeLessThan(
      headings.indexOf(message(language, 'gardens-new-details-title')),
    );
  });

  it('says in words that a part paid from a pocket posts nothing', async () => {
    fakeGardens();
    const { user } = await open();
    // The words about the sources are behind the ⓘ of the shares card (plan 38 G5).
    await openInfo(user, language, message(language, 'gardens-new-shares-title'));
    expect(
      await screen.findByText(said(message(language, 'gardens-new-sources-help'))),
    ).toBeInTheDocument();
  });

  it('shares that do not total 100% show the total in red with words and block the save', async () => {
    fakeGardens();
    const { user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    await user.clear(shareBox(names.c));
    await user.type(shareBox(names.c), '20');
    expect(
      await screen.findByText(said(message(language, 'partners-total-bad'))),
    ).toBeInTheDocument();
    expect(saveButton()).toBeDisabled();
  });

  it('a partner with no share is not in the garden: his source choice goes and 0 is sent', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await user.clear(shareBox(names.a));
    await user.type(shareBox(names.a), '60');
    await user.clear(shareBox(names.b));
    await user.type(shareBox(names.b), '40');
    await user.clear(shareBox(names.c));
    expect(screen.getAllByText(said(message(language, 'ventures-not-in')))).toHaveLength(1);
    await waitFor(() => {
      expect(saveButton()).toBeEnabled();
    });
    await user.click(saveButton());
    await waitFor(() => {
      expect(fake.saved.gardens).toHaveLength(1);
    });
    expect(fake.saved.gardens[0]?.partners).toEqual([
      { partner_id: 1, share: '60', source: 'shop' },
      { partner_id: 2, share: '40', source: 'shop' },
      { partner_id: 3, share: '0', source: 'shop' },
    ]);
  });

  it('saves the shares, the sources and the details as typed, tells the number and opens the garden', async () => {
    const fake = fakeGardens();
    const { router, user } = await open();
    await fillE40(user);
    await user.type(field('gardens-field-paid-now'), '30000');
    // The item is picked from the master picker.
    const item = field('glossary-qism-maal');
    await user.click(item);
    await user.type(item, language === 'ur' ? 'ک' : 'Ban');
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(saveButton()).toBeEnabled();
    });
    await user.click(saveButton());
    await waitFor(() => {
      expect(fake.saved.gardens).toHaveLength(1);
    });
    expect(fake.saved.gardens[0]).toEqual({
      name_ur: URDU_NAME,
      name_en: ENGLISH_NAME,
      confirmed_rules_words: [],
      date: '10-10-2026',
      item_id: 62,
      rent: '50000',
      season_end: null,
      note: null,
      partners: [
        { partner_id: 1, share: '60', source: 'shop' },
        { partner_id: 2, share: '40', source: 'pocket' },
        { partner_id: 3, share: '0', source: 'shop' },
      ],
      paid_now: '30000',
      // Filled in with the standard words when the amount was typed (F43), in the app language.
      paid_now_reason: message(language, 'gardens-first-installment-reason'),
      parts: [],
      confirm_cash_below_zero: false,
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/gardens/2');
    });
  });

  it('shows each partner’s part of the first installment before the save, from the shop or a pocket', async () => {
    fakeGardens();
    const { user } = await open();
    await fillE40(user);
    await user.type(field('gardens-field-paid-now'), '30000');
    await waitFor(() => {
      expect(gardensApi.gardensCreatePreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ rent: '50000', paid_now: '30000' }),
      );
    });
    const parts = await screen.findByRole('list', {
      name: named(message(language, 'gardens-first-parts-title')),
    });
    const text = visible(parts.textContent);
    expect(text).toContain('18,000');
    expect(text).toContain('12,000');
    expect(within(parts).getAllByText(said(message(language, 'ventures-part-shop')))).toHaveLength(
      1,
    );
    expect(
      within(parts).getAllByText(said(message(language, 'ventures-part-pocket'))),
    ).toHaveLength(1);
  });

  it('an installment that takes cash below zero asks first; No saves nothing, Yes saves', async () => {
    const fake = fakeGardens({ cashBefore: 5_000 });
    const { user } = await open();
    await fillE40(user);
    await user.type(field('gardens-field-paid-now'), '30000');
    expect(
      await screen.findByText(said(message(language, 'common-cash-confirm-title'))),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(gardensApi.gardensCreatePreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ paid_now: '30000' }),
      );
    });
    await act(async () => {
      await vi.mocked(gardensApi.gardensCreatePreview).mock.results.at(-1)?.value;
    });
    await settlePreview();
    press('F10');
    const startsWith = (id: string) => (name: string) =>
      visible(name).startsWith(message(language, id));
    const no = await screen.findByRole('button', { name: startsWith('common-cash-confirm-no') });
    // Focus starts on "No", so Enter out of habit never saves a mistake.
    expect(no).toHaveFocus();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(
        screen.queryByRole('button', { name: startsWith('common-cash-confirm-no') }),
      ).toBeNull();
    });
    expect(gardensApi.gardensCreate).not.toHaveBeenCalled();
    press('F10');
    await user.click(
      await screen.findByRole('button', { name: startsWith('common-cash-confirm-yes') }),
    );
    await waitFor(() => {
      expect(fake.saved.gardens).toHaveLength(1);
    });
    expect(fake.saved.gardens[0]?.confirm_cash_below_zero).toBe(true);
  });

  it('typing the first installment fills its reason with the standard words, visible and changeable', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await typeNames(user);
    await user.type(field('gardens-field-rent'), '50000');
    // No amount, no reason field.
    expect(
      screen.queryByRole('combobox', {
        name: named(message(language, 'gardens-field-paid-now-reason')),
      }),
    ).toBeNull();
    await user.type(field('gardens-field-paid-now'), '30000');
    const reason = field('gardens-field-paid-now-reason');
    expect(reason).toHaveValue(message(language, 'gardens-first-installment-reason'));
    // The Owner can change it, and what he typed is what is sent.
    await user.clear(reason);
    await user.type(reason, 'Advance to the owner');
    await waitFor(() => {
      expect(saveButton()).toBeEnabled();
    });
    await user.click(saveButton());
    await waitFor(() => {
      expect(fake.saved.gardens).toHaveLength(1);
    });
    expect(fake.saved.gardens[0]).toMatchObject({
      paid_now: '30000',
      paid_now_reason: 'Advance to the owner',
    });
  });

  it('an emptied reason is told at its field only when the save is tried, and nothing is sent', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await typeNames(user);
    await user.type(field('gardens-field-rent'), '50000');
    await user.type(field('gardens-field-paid-now'), '30000');
    await user.clear(field('gardens-field-paid-now-reason'));
    // Nothing is told while the field is only empty.
    expect(screen.queryByText(said(message(language, 'err-reason-required')))).toBeNull();
    await user.click(saveButton());
    expect(
      (await screen.findAllByText(said(message(language, 'err-reason-required')))).length,
    ).toBeGreaterThan(0);
    expect(fake.saved.gardens).toHaveLength(0);
  });

  it('the keyboard alone does it: type the rent and F10 saves', async () => {
    const fake = fakeGardens();
    const { user } = await open();
    await typeNames(user);
    field('gardens-field-rent').focus();
    await user.keyboard('50000');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.gardens).toHaveLength(1);
    });
    expect(fake.saved.gardens[0]?.rent).toBe('50000');
    expect(fake.saved.gardens[0]?.paid_now).toBeNull();
  });

  it('a refusal shows at the field and in words, and nothing typed is lost', async () => {
    const fake = fakeGardens();
    fake.refuse.garden = [
      { code: 'ERR_SHARES_NOT_100', field: 'partners', values: { total_bps: 9000 } },
      { code: 'ERR_DATE_CLOSED', field: 'date', values: { closed_up_to: '09-10-2026' } },
    ];
    const { user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    await user.type(field('gardens-field-rent'), '50000');
    await user.click(saveButton());
    expect(
      await screen.findAllByText(/./, { selector: '[role="alert"], [role="alert"] *' }),
    ).not.toHaveLength(0);
    expect(field('gardens-field-rent')).toHaveValue('50,000');
    expect(shareBox(names.a)).toHaveValue('40');
  });

  it('a language switch in the middle of the entry keeps every value and the choice of source', async () => {
    fakeGardens();
    const { user } = await open();
    await screen.findByText(said(message(language, 'partners-total-ok')));
    await choose(user, names.c, 'pocket');
    await typeNames(user);
    await user.type(field('gardens-field-rent'), '50000');
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
    const rent = named(message(other, 'gardens-field-rent'));
    expect(
      screen.queryByRole('textbox', { name: rent }) ?? screen.getByRole('combobox', { name: rent }),
    ).toHaveValue('50,000');
    expect(
      screen.getByLabelText<HTMLInputElement>(named(message(other, 'gardens-field-name-ur'))).value,
    ).toBe(URDU_NAME);
    expect(
      screen.getByLabelText<HTMLInputElement>(named(message(other, 'gardens-field-name-en'))).value,
    ).toBe(ENGLISH_NAME);
    const pocket = within(
      screen.getByRole('group', {
        name: named(message(other, 'ventures-source-of', { name: otherNames.c })),
      }),
    ).getByRole('button', { name: message(other, 'ventures-source-pocket') });
    expect(pocket).toHaveAttribute('aria-pressed', 'true');
  });

  it('while the defaults load there is a loading state, and a failure has a retry', async () => {
    fakeGardens();
    vi.mocked(gardensApi.gardensDefaults).mockRejectedValueOnce(
      new CommandError(refusal([{ code: 'ERR_INTERNAL' }])),
    );
    const view = await renderApp('/gardens/new');
    const user = userEvent.setup();
    expect(view.router.state.location.pathname).toBe('/gardens/new');
    expect(
      await screen.findByText(said(message(language, 'gardens-load-failed'))),
    ).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(
      await screen.findByText(said(message(language, 'gardens-new-shares-title'))),
    ).toBeInTheDocument();
  });

  it('with no partner yet it says so and links to the partner setup', async () => {
    fakeGardens({ defaults: { partners: [] } });
    await renderApp('/gardens/new');
    expect(
      await screen.findByText(said(message(language, 'gardens-new-no-partners'))),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', {
        name: named(message(language, 'gardens-new-no-partners-link')),
      }),
    ).toBeInTheDocument();
  });
});
