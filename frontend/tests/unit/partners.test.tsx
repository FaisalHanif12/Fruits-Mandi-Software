// The partners (plan 11, PRD 17.1 to 17.3, FR-17.1, FR-17.2), in the Urdu and in the English app,
// by keyboard, against a fake backend: the overview with its four states, and the setup page with
// the share boxes, the running total from the backend, the day the shares start, adding a partner,
// and the details dialog. The screen adds no percent and no amount: every total here is the fake
// backend's (CLAUDE.md rule 1).

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as partnersApi from '../../src/features/partners/api';
import { usePartnerSetupStore } from '../../src/features/partners';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts, money } from '../helpers/accounts';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { fakePartners, overviewRow, partnerRow, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { at } from '../helpers/sample';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
/** The accessible name contains the words. Fluent isolates a glossary word, so compare visibly. */
const containing = (text: string) => (name: string) => visible(name).includes(text);

/** The name of partner A, B or C in the language of the app. */
const NAMES: Record<Language, readonly string[]> = {
  ur: ['الف', 'ب', 'ج'],
  en: ['Partner A', 'Partner B', 'Partner C'],
};

const shareBox = (language: Language, index: number) =>
  screen.findByLabelText<HTMLInputElement>(
    named(message(language, 'partners-share-of', { name: NAMES[language][index] ?? '' })),
  );

const field = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

const saveButton = (language: Language) =>
  screen.getByRole('button', { name: containing(message(language, 'partners-save')) });

/** jsdom has no layout: give the fields boxes one below the other, as on screen. */
function stackFields(language: Language) {
  const inputs = [0, 1, 2].map((index) =>
    screen.getByLabelText<HTMLInputElement>(
      named(message(language, 'partners-share-of', { name: NAMES[language][index] ?? '' })),
    ),
  );
  const date = screen.getByLabelText<HTMLInputElement>(
    named(message(language, 'partners-field-from')),
  );
  [...inputs, date].forEach((input, index) => {
    const top = index * 60;
    input.getBoundingClientRect = () => new DOMRect(0, top, 300, 44);
  });
}

/** The words of the running total box: its figure and its verdict. */
const totalBox = () => visible(screen.getByRole('status').textContent);

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  fakeAccounts([]);
  fakeAccountSearch([]);
  fakeMasters({});
  act(() => {
    usePartnerSetupStore.getState().clear();
  });
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the partner overview (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const store = fakePartners(THREE_PARTNERS);
    const [a, b, c] = THREE_PARTNERS;
    store.overview = [
      overviewRow(a ?? partnerRow(1, '', '', 0), {
        today_income_paisa: money(1580),
        month_income_paisa: money(2196),
        year_income_paisa: money(2196),
        month_expense_paisa: money(1200),
        month_profit_paisa: money(996),
        month_taken_paisa: money(2000),
        capital_balance_paisa: money(-4000),
      }),
      overviewRow(b ?? partnerRow(2, '', '', 0), { today_income_paisa: money(1184) }),
      overviewRow(c ?? partnerRow(3, '', '', 0), { today_income_paisa: money(1184) }),
    ];
    const app = await renderApp('/partners');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-title')),
    });
    return { ...app, store, user: userEvent.setup() };
  }

  it("shows one card per partner with his share and the backend's numbers, signs included", async () => {
    await open();
    const first = await screen.findByRole('heading', {
      level: 2,
      name: (name) => visible(name).startsWith(NAMES[language][0] ?? ''),
    });
    const card = visible(first.closest('section')?.textContent);
    expect(card).toContain(message(language, 'partners-metric-today'));
    expect(card).toContain('40%');
    expect(card).toContain('1,580');
    expect(card).toContain('996');
    expect(card).toContain('2,000');
    // A capital on the debit side is shown with its minus sign (PartnerOverviewRow).
    expect(card).toContain('-4,000');
    // The other partners have their own cards.
    expect(screen.getAllByRole('heading', { level: 2 })).toHaveLength(3);
  });

  it('shows the name of the language of the app first and the other script as a second line', async () => {
    await open();
    const first = at(await screen.findAllByRole('heading', { level: 2 }), 0);
    const [primary, secondary] = Array.from(first.querySelectorAll('span[lang]'));
    expect(primary?.textContent).toBe(NAMES[language][0]);
    expect(primary?.getAttribute('lang')).toBe(language);
    expect(secondary?.textContent).toBe(language === 'ur' ? 'Partner A' : 'الف');
  });

  it('with no partner says so and offers the setup, by button and by F2', async () => {
    const store = fakePartners([]);
    store.overview = [];
    const user = userEvent.setup();
    await renderApp('/partners');
    expect(
      await screen.findByRole('heading', {
        name: named(message(language, 'partners-overview-empty')),
      }),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', {
        name: containing(message(language, 'partners-overview-empty-action')),
      }),
    );
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'partners-setup-title')),
      }),
    ).toBeInTheDocument();
  });

  it('shows the reference number of a refusal and loads again on retry', async () => {
    const store = fakePartners(THREE_PARTNERS);
    store.overview = [];
    vi.mocked(partnersApi.partnersOverview).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const user = userEvent.setup();
    await renderApp('/partners');
    expect(
      await screen.findByText(named(message(language, 'partners-load-failed'))),
    ).toBeInTheDocument();
    expect(visible(document.body.textContent)).toContain(message(language, 'err-not-implemented'));
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'common-retry')) }),
    );
    expect(
      await screen.findByRole('heading', {
        name: named(message(language, 'partners-overview-empty')),
      }),
    ).toBeInTheDocument();
  });

  it('opens the setup page from its button', async () => {
    const { user } = await open();
    await user.click(
      screen.getByRole('button', { name: containing(message(language, 'partners-setup-open')) }),
    );
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'partners-setup-title')),
      }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('the partner setup (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open(rows = THREE_PARTNERS) {
    const store = fakePartners(rows);
    const app = await renderApp('/partners/setup');
    await screen.findByRole('heading', {
      level: 1,
      name: named(message(language, 'partners-setup-title')),
    });
    return { ...app, store, user: userEvent.setup() };
  }

  it('lists the active partners with their saved shares and a green total of exactly 100', async () => {
    await open();
    expect((await shareBox(language, 0)).value).toBe('40');
    expect((await shareBox(language, 1)).value).toBe('30');
    expect((await shareBox(language, 2)).value).toBe('30');
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-ok'));
    });
    expect(totalBox()).toContain('100%');
    expect(saveButton(language)).toBeEnabled();
    // The first field has focus, and today is the default start day.
    expect(await shareBox(language, 0)).toHaveFocus();
    expect((await field(language, 'partners-field-from')).value).toBe('10-10-2026');
  });

  it("sends each share as typed and shows the backend's total: red at 90, green again at 100", async () => {
    const { user } = await open();
    const first = await shareBox(language, 0);
    await user.clear(first);
    await user.type(first, '30');
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-bad'));
    });
    expect(totalBox()).toContain('90%');
    expect(saveButton(language)).toBeDisabled();
    expect(partnersApi.partnersCheckShares).toHaveBeenLastCalledWith([
      { partner_id: 1, share: '30' },
      { partner_id: 2, share: '30' },
      { partner_id: 3, share: '30' },
    ]);
    await user.clear(first);
    await user.type(first, '40');
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-ok'));
    });
    expect(saveButton(language)).toBeEnabled();
  });

  it('by keyboard only: arrows between the boxes, a new split, F10 saves, and the list shows what was saved', async () => {
    const { user, store } = await open();
    const first = await shareBox(language, 0);
    expect(first).toHaveFocus();
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-ok'));
    });
    stackFields(language);
    await user.keyboard('50{ArrowDown}30{ArrowDown}20');
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-ok'));
    });
    press('F10');
    await waitFor(() => {
      expect(partnersApi.partnersSetShares).toHaveBeenCalledTimes(1);
    });
    expect(store.setShares).toEqual([
      {
        effective_from: '10-10-2026',
        shares: [
          { partner_id: 1, share: '50' },
          { partner_id: 2, share: '30' },
          { partner_id: 3, share: '20' },
        ],
      },
    ]);
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'partners-saved', { date: '10-10-2026' }),
      );
    });
    // After the save the boxes show the saved shares again.
    await waitFor(() => {
      expect(usePartnerSetupStore.getState().shareEdits).toEqual({});
    });
    expect((await shareBox(language, 0)).value).toBe('50');
  });

  it('a share the backend cannot read is flagged at its own row, and a blank one is not flagged before it is typed', async () => {
    const { user } = await open();
    const second = await shareBox(language, 1);
    await user.clear(second);
    // Blank: the total is off, but no red message sits on a box nobody has typed in.
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-bad'));
    });
    expect(second).not.toHaveAttribute('aria-invalid', 'true');
    await user.type(second, 'abc');
    await waitFor(() => {
      expect(second).toHaveAttribute('aria-invalid', 'true');
    });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(message(language, 'err-not-a-number'));
    });

    expect(second.value).toBe('abc');
  });

  it('a back-dated start day is refused at the date field, with today in the message', async () => {
    const { user } = await open();
    const from = await field(language, 'partners-field-from');
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-ok'));
    });
    await user.click(from);
    await user.clear(from);
    await user.type(from, '09-10-2026');
    press('F10');
    await waitFor(() => {
      expect(from).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'err-share-backdated', { today: '10-10-2026' }),
    );
    // Nothing typed is lost.
    expect(from.value).toBe('09-10-2026');
  });

  it('a total that is not 100 on save shows the percent the backend counted, written from bps', async () => {
    await open();
    await waitFor(() => {
      expect(totalBox()).toContain(message(language, 'partners-total-ok'));
    });
    vi.mocked(partnersApi.partnersSetShares).mockRejectedValueOnce(
      fail({ code: 'ERR_SHARES_NOT_100', field: 'shares', values: { total_bps: 9050 } }),
    );
    press('F10');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'err-shares-not-100', { total: '90.5' }),
      );
    });
  });

  it('a language switch in the middle of an edit keeps what was typed and the total', async () => {
    const { user } = await open();
    const first = await shareBox(language, 0);
    await user.clear(first);
    await user.type(first, '25');
    await waitFor(() => {
      expect(totalBox()).toContain('85%');
    });
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    expect((await shareBox(other, 0)).value).toBe('25');
    expect(totalBox()).toContain('85%');
    expect(document.documentElement.dir).toBe(other === 'ur' ? 'rtl' : 'ltr');
    // Switch back: still there.
    act(() => {
      useLanguageStore.getState().setLanguage(language);
    });
    expect((await shareBox(language, 0)).value).toBe('25');
  });

  it('leaving with unsaved shares asks first', async () => {
    const { user, router } = await open();
    const first = await shareBox(language, 0);
    await user.clear(first);
    await user.type(first, '25');
    press('Escape');
    expect(
      await screen.findByRole('alertdialog', {
        name: named(message(language, 'layout-save-title')),
      }),
    ).toBeInTheDocument();
    expect(router.state.location.pathname).toBe('/partners/setup');
  });

  it('before any share was ever set the boxes are empty, not full of zeros', async () => {
    const { store } = await open([
      partnerRow(1, 'الف', 'Partner A', 0),
      partnerRow(2, 'ب', 'Partner B', 0),
    ]);
    expect(store.rows).toHaveLength(2);
    expect((await shareBox(language, 0)).value).toBe('');
    expect((await shareBox(language, 1)).value).toBe('');
    expect(saveButton(language)).toBeDisabled();
  });

  it('with no partner shows the empty state, and F2 opens the add dialog', async () => {
    const { user } = await open([]);
    expect(
      await screen.findByRole('heading', {
        name: named(message(language, 'partners-setup-empty')),
      }),
    ).toBeInTheDocument();
    press('F2');
    expect(
      await screen.findByRole('dialog', { name: named(message(language, 'partners-add-title')) }),
    ).toBeInTheDocument();
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });

  it('adds a partner with his names and opening capital as typed, then shows him with an empty box', async () => {
    const { user, store } = await open();
    press('F2');
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'partners-add-title')),
    });
    // The first field has focus; the other name is filled by itself or typed by hand.
    const urdu = await within(dialog).findByLabelText<HTMLInputElement>(
      named(message(language, 'partners-field-name-ur')),
    );
    const english = within(dialog).getByLabelText<HTMLInputElement>(
      named(message(language, 'partners-field-name-en')),
    );
    await user.click(urdu);
    await user.keyboard('د');
    await user.click(english);
    await user.clear(english);
    await user.keyboard('Partner D');
    await user.click(
      within(dialog).getByLabelText<HTMLInputElement>(
        named(message(language, 'partners-field-capital')),
      ),
    );
    await user.keyboard('50000');
    press('F10');
    await waitFor(() => {
      expect(partnersApi.partnersCreate).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(partnersApi.partnersCreate).mock.calls[0]?.[0]).toEqual({
      name_ur: 'د',
      name_en: 'Partner D',
      opening_capital: '50000',
      confirmed_rules_words: [],
    });
    expect(store.rows).toHaveLength(4);
    await waitFor(() => {
      expect(
        screen.queryByRole('dialog', { name: named(message(language, 'partners-add-title')) }),
      ).toBeNull();
    });
    // He is in the list. The others still total 100, so the total now counts his blank box too.
    const label = message(language, 'partners-share-of', {
      name: language === 'ur' ? 'د' : 'Partner D',
    });
    const box = await screen.findByLabelText<HTMLInputElement>(named(label));
    expect(box.value).toBe('0');
    // Focus is on the first share box, to type the shares next, not lost on the page.
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(
          named(message(language, 'partners-share-of', { name: NAMES[language][0] ?? '' })),
        ),
      ).toHaveFocus();
    });
  });

  it('Ctrl+Enter saves and keeps the dialog open and empty for the next partner', async () => {
    const { user, store } = await open([]);
    press('F2');
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'partners-add-title')),
    });
    const urdu = await within(dialog).findByLabelText<HTMLInputElement>(
      named(message(language, 'partners-field-name-ur')),
    );
    const english = within(dialog).getByLabelText<HTMLInputElement>(
      named(message(language, 'partners-field-name-en')),
    );
    await user.click(urdu);
    await user.keyboard('الف');
    await user.click(english);
    await user.keyboard('Partner A');
    press('Enter', { ctrlKey: true });
    await waitFor(() => {
      expect(store.rows).toHaveLength(1);
    });
    // Still open, and empty again, ready for the next name.
    const again = await screen.findByRole('dialog', {
      name: named(message(language, 'partners-add-title')),
    });
    await waitFor(() => {
      expect(
        within(again).getByLabelText<HTMLInputElement>(
          named(message(language, 'partners-field-name-ur')),
        ).value,
      ).toBe('');
    });
    // The capital was not typed, so none is sent.
    expect(vi.mocked(partnersApi.partnersCreate).mock.calls[0]?.[0].opening_capital).toBeNull();
  });

  it('shows what the backend refused in the add dialog, at the field', async () => {
    const { user } = await open();
    vi.mocked(partnersApi.partnersCreate).mockRejectedValueOnce(
      fail({ code: 'ERR_TOO_MANY_PARTNERS', values: { max: 10 } }),
    );
    press('F2');
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'partners-add-title')),
    });
    const urdu = await within(dialog).findByLabelText<HTMLInputElement>(
      named(message(language, 'partners-field-name-ur')),
    );
    await user.click(urdu);
    await user.keyboard('د');
    press('F10');
    await waitFor(() => {
      expect(visible(dialog.textContent)).toContain(
        message(language, 'err-too-many-partners', { max: '10' }),
      );
    });
    // The name typed is still there.
    expect(urdu.value).toBe('د');
  });

  it('the details dialog shows the share history, newest first, and refuses to deactivate a partner with a share', async () => {
    const { user, store } = await open();
    store.history[1] = [
      { share_bps: 3500, effective_from: '2026-09-01' },
      { share_bps: 4000, effective_from: '2026-10-10' },
    ];
    const details = await screen.findAllByRole('button', {
      name: containing(message(language, 'partners-edit')),
    });
    await user.click(at(details, 0));
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'partners-edit-title')),
    });
    const rows = await within(dialog).findAllByRole('listitem');
    expect(visible(rows[0]?.textContent)).toBe(
      message(language, 'partners-history-row', { date: '10-10-2026', share: '40' }),
    );
    expect(visible(rows[1]?.textContent)).toBe(
      message(language, 'partners-history-row', { date: '01-09-2026', share: '35' }),
    );
    await user.click(
      within(dialog).getByRole('checkbox', {
        name: named(message(language, 'partners-field-active')),
      }),
    );
    press('F10');
    await waitFor(() => {
      expect(visible(dialog.textContent)).toContain(message(language, 'err-partner-has-share'));
    });
    // Nothing changed.
    expect(store.rows[0]?.is_active).toBe(true);
  });

  it('makes a partner with no share inactive; he moves to the inactive list and out of the shares', async () => {
    const { user } = await open([
      partnerRow(1, 'الف', 'Partner A', 10000),
      partnerRow(2, 'ب', 'Partner B', 0),
    ]);
    const details = await screen.findAllByRole('button', {
      name: containing(message(language, 'partners-edit')),
    });
    await user.click(at(details, 1));
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'partners-edit-title')),
    });
    await user.click(
      await within(dialog).findByRole('checkbox', {
        name: named(message(language, 'partners-field-active')),
      }),
    );
    press('F10');
    await waitFor(() => {
      expect(
        screen.queryByRole('dialog', { name: named(message(language, 'partners-edit-title')) }),
      ).toBeNull();
    });
    expect(vi.mocked(partnersApi.partnersUpdate).mock.calls[0]?.[0]).toMatchObject({
      id: 2,
      version: 1,
      is_active: false,
    });
    expect(
      await screen.findByText(named(message(language, 'partners-inactive-title'))),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(partnersApi.partnersCheckShares).toHaveBeenLastCalledWith([
        { partner_id: 1, share: '100' },
      ]);
    });
  });

  it('shows the reference number of a refusal and loads again on retry', async () => {
    fakePartners(THREE_PARTNERS);
    vi.mocked(partnersApi.partnersList).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const user = userEvent.setup();
    await renderApp('/partners/setup');
    expect(
      await screen.findByText(named(message(language, 'partners-setup-failed'))),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'common-retry')) }),
    );
    expect((await shareBox(language, 0)).value).toBe('40');
  });
});
