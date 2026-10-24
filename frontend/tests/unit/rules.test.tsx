// The commission rules (plan 09, PRD 11, FR-11.5, FR-11.8, FR-7.43): the list, the form that offers
// only the charges and methods of PRD 11.2, the overlap warning, "Test a rule" with its working,
// and the settings Charges and Overrides, in the Urdu and in the English app, by keyboard, against
// a fake backend. The frontend never works out an amount: every amount here is the backend's.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as rulesApi from '../../src/features/rules/api';
import * as settingsApi from '../../src/features/settings/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
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
import { fakeMasters, record } from '../helpers/masters';
import { message, visible } from '../helpers/render';
import { fakeRuleTest, fakeRules, pointer, rule, testResult } from '../helpers/rules';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/rules/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax');
const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim');
const BANANA = record('item', 5, '0005', 'کیلا', 'Banana');
const APPLE = record('item', 6, '0006', 'سیب', 'Apple');

const named = (text: string) => (name: string) => visible(name) === text;

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}
const rowIds = () =>
  Array.from(document.querySelectorAll('.ag-row[row-id]')).map((row) => row.getAttribute('row-id'));

const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

const dropdown = (language: Language, id: string) =>
  screen.findByRole('combobox', { name: named(message(language, id)) });

/** Opens a drop-down list and returns the words of its choices. */
async function choicesOf(user: ReturnType<typeof userEvent.setup>, box: HTMLElement) {
  await user.click(box);
  const options = await screen.findAllByRole('option');
  return options.map((option) => visible(option.textContent));
}

/** Types in a picker and takes the first match with Enter. */
async function pick(
  user: ReturnType<typeof userEvent.setup>,
  language: Language,
  labelId: string,
  typed: string,
  shown: string,
) {
  await user.click(await dropdown(language, labelId));
  await user.keyboard(typed);
  await screen.findByRole('option', { name: (name) => visible(name).includes(shown) });
  await user.keyboard('{Enter}');
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  fakeMasters({ item: [BANANA, APPLE] });
  fakeAccounts([AKRAM, RAHIM]);
  fakeAccountSearch([AKRAM, RAHIM]);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the rule list (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the seeded defaults with side, charge, party, item or group, value and level', async () => {
    fakeRules();
    await renderApp('/index/rules');
    await screen.findByRole('heading', { level: 1, name: named(message(language, 'rules-title')) });
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2', '3']);
    });
    expect(cell(1, 'side')).toBe(message(language, 'rules-side-seller'));
    expect(cell(1, 'charge')).toBe(message(language, 'rules-charge-commission'));
    expect(cell(1, 'party')).toBe(message(language, 'rules-any-party'));
    expect(cell(1, 'item')).toBe(message(language, 'rules-any-item'));
    expect(cell(1, 'value')).toBe(message(language, 'rules-value-percent', { value: '6' }));
    // The three global defaults say so; they are changed in Settings.
    expect(cell(1, 'status')).toBe(message(language, 'rules-status-default'));
    // The لاگا default is a rule on the group fruit (PRD 4.2).
    expect(cell(3, 'charge')).toBe(message(language, 'rules-charge-laga'));
    expect(cell(3, 'item')).toBe(message(language, 'rules-group-fruit'));
    expect(cell(3, 'value')).toBe(message(language, 'rules-value-per-unit', { value: '8' }));
  });

  it('a rule of the user says active, and the three defaults are opened in Settings', async () => {
    fakeRules([rule(4, { level: 'party_item' })]);
    await renderApp('/index/rules');
    await waitFor(() => {
      expect(rowIds()).toEqual(['4']);
    });
    expect(cell(4, 'status')).toBe(message(language, 'rules-status-active'));
  });

  it('an old default that ended says until when, and the current one says only Default', async () => {
    fakeRules([
      rule(1, { is_default: true, to_date: '2026-09-30' }),
      rule(11, { is_default: true, from_date: '2026-10-01' }),
    ]);
    await renderApp('/index/rules');
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '11']);
    });
    expect(cell(1, 'status')).toBe(
      message(language, 'rules-status-default-ended', { to: '30-09-2026' }),
    );
    expect(cell(11, 'status')).toBe(message(language, 'rules-status-default'));
  });

  it('an ended default is read-only too, and its dates are told', async () => {
    fakeRules([rule(1, { is_default: true, from_date: '2026-08-31', to_date: '2026-09-30' })]);
    await renderApp('/index/rules/1');
    await screen.findByRole('heading', {
      level: 1,
      name: message(language, 'rules-default-title'),
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'rules-dates', { from: '31-08-2026', to: '30-09-2026' }),
    );
    expect(
      screen.queryByRole('button', { name: new RegExp(message(language, 'layout-save-save')) }),
    ).toBeNull();
  });

  it('a default rule has no form: it says it is changed in Settings, Charges, and the link opens that tab', async () => {
    fakeRules();
    const { router } = await renderApp('/index/rules/1');
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'rules-default-title'),
      }),
    ).toBeInTheDocument();
    expect(screen.getByText(message(language, 'rules-default-notice'))).toBeInTheDocument();
    // Nothing to save or switch off here.
    expect(
      screen.queryByRole('button', { name: new RegExp(message(language, 'layout-save-save')) }),
    ).toBeNull();
    expect(
      screen.queryByRole('button', { name: message(language, 'rules-make-inactive') }),
    ).toBeNull();
    await userEvent
      .setup()
      .click(
        screen.getByRole('button', { name: message(language, 'rules-default-open-settings') }),
      );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/settings');
    });
    expect(
      await screen.findByRole('tab', { name: named(message(language, 'settings-tab-charges')) }),
    ).toHaveAttribute('aria-selected', 'true');
  });

  it('the wide layout adds the level and the dates, so a list never scrolls sideways (NFR-USE-15)', async () => {
    fakeRules([rule(4, { level: 'party_item', to_date: '2026-12-31' })]);
    // A window of 1920 points: both layout steps match.
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: true,
      media: query,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    }));
    // The steps start where the design tokens say (tokens.css is not loaded in a test).
    document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
    document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
    try {
      await renderApp('/index/rules');
      await waitFor(() => {
        expect(rowIds()).toEqual(['4']);
      });
      expect(cell(4, 'level')).toBe(message(language, 'rules-level-party-item'));
      expect(cell(4, 'from_date')).toBe('31-08-2026');
      expect(cell(4, 'to_date')).toBe('31-12-2026');
    } finally {
      vi.unstubAllGlobals();
      document.documentElement.style.removeProperty('--breakpoint-normal');
      document.documentElement.style.removeProperty('--breakpoint-wide');
    }
  });

  it('150 rules come page by page: 100 at once with the count of the backend, Show more loads the rest', async () => {
    fakeRules(Array.from({ length: 150 }, (_, index) => rule(index + 1)));
    await renderApp('/index/rules');
    const showing = (shown: number) =>
      message(language, 'common-list-count', { shown: String(shown), total: '150' });
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(showing(100));
    });
    expect(rulesApi.rulesList).toHaveBeenLastCalledWith(expect.anything(), 0, 100);
    await userEvent.setup().click(
      await screen.findByRole('button', {
        name: (name) => visible(name) === message(language, 'common-list-more'),
      }),
    );
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(showing(150));
    });
    expect(rulesApi.rulesList).toHaveBeenLastCalledWith(expect.anything(), 100, 100);
  });

  it('the side, the charge and the inactive tick go to the backend filter', async () => {
    fakeRules([...fakeRulesRows(), rule(9, { is_active: false, value: '3' })]);
    await renderApp('/index/rules');
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2', '3']);
    });
    const user = userEvent.setup();
    await user.click(await dropdown(language, 'rules-filter-side'));
    await user.click(
      await screen.findByRole('option', {
        name: named(message(language, 'rules-side-buyer')),
      }),
    );
    await waitFor(() => {
      expect(rowIds()).toEqual(['2', '3']);
    });
    expect(rulesApi.rulesList).toHaveBeenLastCalledWith(
      {
        side: 'buyer',
        charge_type: null,
        account_id: null,
        item_id: null,
        include_inactive: false,
      },
      0,
      100,
    );
    await user.click(
      screen.getByRole('checkbox', { name: message(language, 'rules-show-inactive') }),
    );
    await waitFor(() => {
      expect(rulesApi.rulesList).toHaveBeenLastCalledWith(
        expect.objectContaining({ include_inactive: true }),
        0,
        100,
      );
    });
  });

  it('an empty list says why and has a button; a filter with no match offers to clear it', async () => {
    fakeRules([]);
    await renderApp('/index/rules');
    expect(await screen.findByText(message(language, 'rules-empty'))).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: new RegExp(message(language, 'rules-new')) }),
    ).toBeInTheDocument();
  });

  it('a filter with no match offers to clear it', async () => {
    fakeRules([rule(1)]);
    await renderApp('/index/rules');
    await waitFor(() => {
      expect(rowIds()).toEqual(['1']);
    });
    const user = userEvent.setup();
    await user.click(await dropdown(language, 'rules-filter-side'));
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'rules-side-buyer')) }),
    );
    expect(await screen.findByText(message(language, 'rules-no-match-title'))).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: message(language, 'rules-no-match-clear') }),
    );
    await waitFor(() => {
      expect(rowIds()).toEqual(['1']);
    });
  });

  it('a failed load shows the message with a retry that works', async () => {
    fakeRules();
    vi.mocked(rulesApi.rulesList).mockRejectedValueOnce(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    await renderApp('/index/rules');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2', '3']);
    });
  });

  it('F2 opens a new rule, Enter opens the row, and the test button opens the test panel', async () => {
    fakeRules();
    const { router } = await renderApp('/index/rules');
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2', '3']);
    });
    press('F2');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/rules/new');
    });
    act(() => {
      void router.navigate({ to: '/index/rules' });
    });
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '2', '3']);
    });
    await userEvent
      .setup()
      .click(screen.getByRole('button', { name: message(language, 'rules-test-open') }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/rules/test');
    });
  });
});

/** A fresh copy of the seeded rows, for a test that adds one more. */
function fakeRulesRows() {
  return [
    rule(1),
    rule(2, { side: 'buyer' }),
    rule(3, {
      side: 'buyer',
      charge_type: 'laga',
      method: 'per_unit',
      value: '8',
      item_group: 'fruit',
      level: 'item_group',
    }),
  ];
}

describe.each(LANGUAGES)('the rule form (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('a new rule starts as a seller commission in percent, from today', async () => {
    fakeRules();
    await renderApp('/index/rules/new');
    await screen.findByRole('heading', { level: 1, name: message(language, 'rules-new-title') });
    expect((await dropdown(language, 'rules-field-side')).textContent).toContain(
      message(language, 'rules-side-seller'),
    );
    expect((await dropdown(language, 'rules-field-charge')).textContent).toContain(
      message(language, 'rules-charge-commission'),
    );
    expect((await dropdown(language, 'rules-field-method')).textContent).toContain(
      message(language, 'rules-method-percent'),
    );
    expect((await labelled(language, 'rules-field-from')).value).toBe('10-10-2026');
    expect((await labelled(language, 'rules-field-value')).value).toBe('');
    expect((await labelled(language, 'rules-field-to')).value).toBe('');
  });

  it('offers only the charges and methods of PRD 11.2 for the side and the charge', async () => {
    fakeRules();
    await renderApp('/index/rules/new');
    const user = userEvent.setup();
    const charge = await dropdown(language, 'rules-field-charge');
    const word = (id: string) => message(language, id);
    // Seller side: commission, labour, munshiana, phone.
    expect(await choicesOf(user, charge)).toEqual([
      word('rules-charge-commission'),
      word('rules-charge-labour'),
      word('rules-charge-munshiana'),
      word('rules-charge-phone'),
    ]);
    // Labour has no percent method.
    await user.click(screen.getByRole('option', { name: named(word('rules-charge-labour')) }));
    expect(await choicesOf(user, await dropdown(language, 'rules-field-method'))).toEqual([
      word('rules-method-per_unit'),
      word('rules-method-per_lot'),
    ]);
    await user.keyboard('{Escape}');
    // Buyer side: commission and لاگا only.
    await user.click(await dropdown(language, 'rules-field-side'));
    await user.click(await screen.findByRole('option', { name: named(word('rules-side-buyer')) }));
    expect(await choicesOf(user, await dropdown(language, 'rules-field-charge'))).toEqual([
      word('rules-charge-commission'),
      word('rules-charge-laga'),
    ]);
  });

  it('choosing an item locks the group, and an item group is sent as the group', async () => {
    const store = fakeRules();
    await renderApp('/index/rules/new');
    const user = userEvent.setup();
    await pick(
      user,
      language,
      'rules-field-item',
      language === 'ur' ? 'کیلا' : 'Banana',
      language === 'ur' ? 'کیلا' : 'Banana',
    );
    await waitFor(() => {
      expect(
        screen.getByRole('combobox', { name: named(message(language, 'rules-field-group')) }),
      ).toBeDisabled();
    });
    await user.type(await labelled(language, 'rules-field-value'), '5');
    press('F10');
    await waitFor(() => {
      expect(rulesApi.rulesSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(rulesApi.rulesSave).mock.calls[0]?.[0];
    expect(sent?.item?.id).toBe(5);
    expect(sent?.item_group).toBe('');
    expect(store.rows.at(-1)?.item?.id).toBe(5);
  });

  it('choosing a party locks the group too: a group is only for a rule with no party and no item', async () => {
    fakeRules();
    await renderApp('/index/rules/new');
    const user = userEvent.setup();
    const group = await dropdown(language, 'rules-field-group');
    expect(group).toBeEnabled();
    await pick(user, language, 'rules-field-party', 'rahim', '030110001');
    await waitFor(() => {
      expect(group).toBeDisabled();
    });
  });

  it('making an inactive rule active shows the overlap warning and waits for Done', async () => {
    const store = fakeRules([rule(8, { is_active: false, value: '4' })]);
    // The backend answers the switch with the rules that overlap it.
    vi.mocked(rulesApi.rulesSetActive).mockResolvedValueOnce({
      row: rule(8, { value: '4' }),
      overlaps: [{ id: 1, from_date: '2026-08-31', to_date: null }],
    });
    const { router } = await renderApp('/index/rules/8');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'rules-make-active') }),
    );
    expect(
      await screen.findByRole('heading', { name: message(language, 'rules-overlap-title') }),
    ).toBeInTheDocument();
    expect(router.state.location.pathname).toBe('/index/rules/8');
    await user.click(screen.getByRole('button', { name: message(language, 'rules-overlap-done') }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/rules');
    });
    expect(store.rows).toHaveLength(1);
  });

  it('a value of 0 per نگ is sent as typed: the form does not refuse it', async () => {
    fakeRules();
    await renderApp('/index/rules/new');
    const user = userEvent.setup();
    await pick(user, language, 'rules-field-party', 'rahim', 'Rahim');
    await user.click(await dropdown(language, 'rules-field-method'));
    await user.click(
      await screen.findByRole('option', {
        name: named(message(language, 'rules-method-per_unit')),
      }),
    );
    await user.click(await labelled(language, 'rules-field-value'));
    await user.keyboard('0');
    press('F10');
    await waitFor(() => {
      expect(rulesApi.rulesSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(rulesApi.rulesSave).mock.calls[0]?.[0];
    expect(sent).toEqual(expect.objectContaining({ method: 'per_unit', value: '0' }));
  });

  it('F10 sends the value and the dates as typed, says it is saved and goes back to the list', async () => {
    const store = fakeRules();
    const { router } = await renderApp('/index/rules/new');
    const user = userEvent.setup();
    await pick(user, language, 'rules-field-party', 'rahim', 'Rahim');
    await user.click(await labelled(language, 'rules-field-value'));
    await user.keyboard('2.5');
    press('F10');
    await waitFor(() => {
      expect(rulesApi.rulesSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(rulesApi.rulesSave).mock.calls[0]?.[0];
    expect(sent?.account?.id).toBe(31);
    expect(sent).toEqual(
      expect.objectContaining({
        id: null,
        side: 'seller',
        charge_type: 'commission',
        method: 'percent',
        value: '2.5',
        from_date: '10-10-2026',
        to_date: '',
        is_active: true,
      }),
    );
    expect((await screen.findAllByText(message(language, 'rules-saved'))).length).toBeGreaterThan(
      0,
    );
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/rules');
    });
    expect(store.rows).toHaveLength(4);
  });

  it('a rule that overlaps another shows the warning, and stays until Done', async () => {
    const store = fakeRules();
    store.overlapsOnNextSave = [{ id: 1, from_date: '2026-08-31', to_date: null }];
    const { router } = await renderApp('/index/rules/new');
    const user = userEvent.setup();
    await user.type(await labelled(language, 'rules-field-value'), '5');
    press('F10');
    const title = await screen.findByRole('heading', {
      name: message(language, 'rules-overlap-title'),
    });
    const warning = title.closest('section');
    expect(warning).not.toBeNull();
    expect(visible(warning?.textContent)).toContain(message(language, 'rules-overlap-title'));
    expect(visible(warning?.textContent)).toContain(message(language, 'rules-overlap-help'));
    // The older rule is named and its dates are told, so the user can see which one it is.
    await waitFor(() => {
      expect(visible(warning?.textContent)).toContain(message(language, 'rules-charge-commission'));
    });
    expect(visible(warning?.textContent)).toContain(
      message(language, 'rules-dates-open', { from: '31-08-2026' }),
    );
    expect(router.state.location.pathname).toBe('/index/rules/new');
    // The rule is saved already: F10 again only finishes.
    expect(rulesApi.rulesSave).toHaveBeenCalledTimes(1);
    await user.click(screen.getByRole('button', { name: message(language, 'rules-overlap-done') }));
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/rules');
    });
    expect(rulesApi.rulesSave).toHaveBeenCalledTimes(1);
  });

  it('a refused value shows at the field, the rest in a notice', async () => {
    fakeRules();
    vi.mocked(rulesApi.rulesSave).mockRejectedValue(
      fail({ code: 'ERR_PERCENT_OVER_100', field: 'value' }, { code: 'ERR_DB_BUSY' }),
    );
    await renderApp('/index/rules/new');
    const user = userEvent.setup();
    const value = await labelled(language, 'rules-field-value');
    await user.type(value, '101');
    press('F10');
    await waitFor(() => {
      expect(value).toHaveAttribute('aria-invalid', 'true');
    });
    const alert = screen.getByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-db-busy'));
    expect(visible(alert.textContent)).not.toContain(message(language, 'err-percent-over-100'));
    // What was typed stays.
    expect(value.value).toBe('101');
  });

  it('an edit shows what was saved, and making the rule inactive asks first', async () => {
    fakeRules([
      rule(7, {
        side: 'buyer',
        account: pointer(21, '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax'),
        value: '7',
        level: 'party',
      }),
    ]);
    const { router } = await renderApp('/index/rules/7');
    await screen.findByRole('heading', { level: 1, name: message(language, 'rules-edit-title') });
    expect((await labelled(language, 'rules-field-value')).value).toBe('7');
    expect((await dropdown(language, 'rules-field-side')).textContent).toContain(
      message(language, 'rules-side-buyer'),
    );
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', { name: message(language, 'rules-make-inactive') }),
    );
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain(message(language, 'rules-inactive-description'));
    await user.click(
      within(dialog).getByRole('button', { name: message(language, 'rules-make-inactive') }),
    );
    await waitFor(() => {
      expect(rulesApi.rulesSetActive).toHaveBeenCalledWith(7, false);
    });
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/rules');
    });
  });

  it('a rule that is not there says so, with the way back', async () => {
    fakeRules();
    await renderApp('/index/rules/abc');
    expect(await screen.findByText(message(language, 'rules-not-found-title'))).toBeInTheDocument();
  });

  it('switching the language in the middle of an entry keeps what was typed (F39)', async () => {
    fakeRules();
    await renderApp('/index/rules/new');
    const user = userEvent.setup();
    const value = await labelled(language, 'rules-field-value');
    await user.type(value, '4');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    expect((await labelled(other, 'rules-field-value')).value).toBe('4');
    expect(
      await screen.findByRole('heading', { level: 1, name: message(other, 'rules-new-title') }),
    ).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('Test a rule (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function fill(user: ReturnType<typeof userEvent.setup>) {
    await pick(user, language, 'rules-test-seller', 'rahim', 'Rahim');
    await pick(user, language, 'rules-test-buyer', 'akram', 'Akram Toll Tax');
    await pick(
      user,
      language,
      'rules-test-item',
      language === 'ur' ? 'کیلا' : 'Banana',
      language === 'ur' ? 'کیلا' : 'Banana',
    );
    await user.type(await labelled(language, 'rules-test-qty'), '20');
    await user.type(await labelled(language, 'rules-test-rate'), '600');
  }

  it('sends the quantity and the rate as typed and shows, for each charge, the winner, the level, the skipped levels and the amount', async () => {
    fakeRules();
    const asked = fakeRuleTest();
    await renderApp('/index/rules/test');
    await screen.findByRole('heading', { level: 1, name: message(language, 'rules-test-title') });
    const user = userEvent.setup();
    await fill(user);
    await user.click(screen.getByRole('button', { name: message(language, 'rules-test-run') }));
    await waitFor(() => {
      expect(asked).toHaveLength(1);
    });
    expect(asked[0]).toEqual({
      seller_id: 31,
      buyer_id: 21,
      item_id: 5,
      date: '10-10-2026',
      qty: '20',
      rate: '600',
    });
    const table = await screen.findByRole('table');
    const rows = within(table).getAllByRole('row').slice(1);
    expect(rows).toHaveLength(4);
    const seller = visible(rows[0]?.textContent);
    expect(seller).toContain(message(language, 'rules-level-default'));
    expect(seller).toContain(message(language, 'rules-value-percent', { value: '6' }));
    expect(seller).toContain('720');
    // The value of the line is the backend's.
    expect(visible(table.closest('section')?.textContent)).toContain('12,000');
    // The skipped levels are listed.
    expect(seller).toContain(message(language, 'rules-level-party-item'));
    expect(seller).toContain(message(language, 'rules-level-item-group'));
    const laga = visible(rows[2]?.textContent);
    expect(laga).toContain(message(language, 'rules-level-item-group'));
    expect(laga).toContain(message(language, 'rules-value-per-unit', { value: '8' }));
    expect(laga).toContain('160');
    // A charge with no rule says the amount is typed on the lot.
    expect(visible(rows[3]?.textContent)).toContain(message(language, 'rules-test-no-rule'));
  });

  it('the small ⓘ shows the working the backend sent, with the real numbers', async () => {
    fakeRules();
    fakeRuleTest();
    await renderApp('/index/rules/test');
    const user = userEvent.setup();
    await fill(user);
    press('Enter');
    await user.click(screen.getByRole('button', { name: message(language, 'rules-test-run') }));
    const table = await screen.findByRole('table');
    const first = within(table).getAllByRole('row')[1];
    const info = within(first ?? table).getByRole('button', {
      name: message(language, 'calc-info-label'),
    });
    await user.click(info);
    expect(await screen.findByText(message(language, 'calc-info-title'))).toBeInTheDocument();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'working-percent', { base: '12,000', bps: '6', result: '720' }),
    );
  });

  it('a seller, a buyer or an item left empty is told at the field, and the backend is not asked', async () => {
    fakeRules();
    const asked = fakeRuleTest();
    await renderApp('/index/rules/test');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'rules-test-run') }),
    );
    for (const id of ['rules-test-seller', 'rules-test-buyer', 'rules-test-item']) {
      expect(await dropdown(language, id)).toHaveAttribute('aria-invalid', 'true');
    }
    expect(asked).toHaveLength(0);
    expect(visible(document.body.textContent)).toContain(message(language, 'err-required'));
  });

  it('a refused field shows at the field', async () => {
    fakeRules();
    vi.mocked(rulesApi.rulesTest).mockRejectedValue(
      fail({ code: 'ERR_QTY_NOT_NUMBER', field: 'qty' }),
    );
    await renderApp('/index/rules/test');
    const user = userEvent.setup();
    await fill(user);
    const qty = await labelled(language, 'rules-test-qty');
    await user.type(qty, 'x');
    await user.click(screen.getByRole('button', { name: message(language, 'rules-test-run') }));
    await waitFor(() => {
      expect(qty).toHaveAttribute('aria-invalid', 'true');
    });
  });

  it('before the first test it says what to do', async () => {
    fakeRules();
    fakeRuleTest(testResult());
    await renderApp('/index/rules/test');
    expect(await screen.findByText(message(language, 'rules-test-nothing'))).toBeInTheDocument();
  });
});

describe.each(LANGUAGES)('the settings Charges and Overrides (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function openTab(id: string) {
    await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    const user = userEvent.setup();
    await user.click(screen.getByRole('tab', { name: named(message(language, id)) }));
    return user;
  }

  /** The block of one default on the Charges tab: its heading, with its method and value fields. */
  const block = async (id: string) => {
    const heading = await screen.findByRole('heading', { name: named(message(language, id)) });
    const section = heading.closest('section');
    if (section === null) throw new Error('no section');
    return {
      method: within(section).getByRole('combobox', {
        name: named(message(language, 'rules-field-method')),
      }),
      value: within(section).getByLabelText<HTMLInputElement>(
        named(message(language, 'rules-field-value')),
      ),
    };
  };

  it('Charges shows each default with its method and value, and F10 sends them as typed with the version read', async () => {
    const store = fakeRules();
    const user = await openTab('settings-tab-charges');
    const seller = await block('settings-charges-seller');
    expect(seller.value.value).toBe('6');
    expect(seller.method.textContent).toContain(message(language, 'rules-method-percent'));
    const laga = await block('settings-charges-laga');
    expect(laga.value.value).toBe('8');
    expect(visible(laga.method.textContent)).toContain(message(language, 'rules-method-per_unit'));
    const save = screen.getByRole('button', {
      name: new RegExp(message(language, 'layout-save-save')),
    });
    expect(save).toBeDisabled();
    await user.clear(seller.value);
    await user.type(seller.value, '7');
    expect(save).toBeEnabled();
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetCharges).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(settingsApi.settingsSetCharges).mock.calls[0]?.[0]).toEqual({
      seller_commission: { version: 1, method: 'percent', value: '7' },
      buyer_commission: { version: 1, method: 'percent', value: '6' },
      laga_fruit: { version: 1, method: 'per_unit', value: '8' },
    });
    expect(
      (await screen.findAllByText(message(language, 'settings-charges-saved'))).length,
    ).toBeGreaterThan(0);
    expect(store.charges.seller_commission.value_text).toBe('7');
    // The form starts again from what was saved, so nothing is left to save.
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) }),
      ).toBeDisabled();
    });
  });

  it('a default offers only the methods PRD 11.2 allows for its charge', async () => {
    fakeRules();
    const user = await openTab('settings-tab-charges');
    const word = (id: string) => message(language, id);
    const seller = await block('settings-charges-seller');
    expect(await choicesOf(user, seller.method)).toEqual([
      word('rules-method-percent'),
      word('rules-method-per_unit'),
      word('rules-method-per_lot'),
    ]);
    await user.keyboard('{Escape}');
    const laga = await block('settings-charges-laga');
    expect(await choicesOf(user, laga.method)).toEqual([
      word('rules-method-per_unit'),
      word('rules-method-percent'),
    ]);
  });

  it('each default says the day it applies from, and a change starts today and leaves the old rate in the list', async () => {
    fakeRules();
    const user = await openTab('settings-tab-charges');
    const seller = await block('settings-charges-seller');
    const from = (id: string) =>
      visible(
        screen.getByRole('heading', { name: named(message(language, id)) }).closest('section')
          ?.textContent,
      );
    expect(from('settings-charges-seller')).toContain(
      message(language, 'settings-charges-from', { date: '31-08-2026' }),
    );
    await user.clear(seller.value);
    await user.type(seller.value, '7');
    press('F10');
    // The seller default now applies from today; the buyer one keeps its own start.
    await waitFor(() => {
      expect(from('settings-charges-seller')).toContain(
        message(language, 'settings-charges-from', { date: '10-10-2026' }),
      );
    });
    expect(from('settings-charges-buyer')).toContain(
      message(language, 'settings-charges-from', { date: '31-08-2026' }),
    );
  });

  it('Charges shows a refused rate at its field', async () => {
    fakeRules();
    vi.mocked(settingsApi.settingsSetCharges).mockRejectedValue(
      fail({ code: 'ERR_PERCENT_OVER_100', field: 'buyer_commission.value' }),
    );
    const user = await openTab('settings-tab-charges');
    const buyer = await block('settings-charges-buyer');
    await user.clear(buyer.value);
    await user.type(buyer.value, '120');
    press('F10');
    await waitFor(() => {
      expect(buyer.value).toHaveAttribute('aria-invalid', 'true');
    });
  });

  it('Charges says that labour, munshiana and phone have no default, and links to the rules', async () => {
    fakeRules();
    const user = await openTab('settings-tab-charges');
    expect(
      await screen.findByText(
        (content) => visible(content) === message(language, 'settings-charges-note'),
      ),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', {
        name: named(message(language, 'settings-charges-open-rules')),
      }),
    );
    expect(
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'rules-title')),
      }),
    ).toBeInTheDocument();
  });

  it('Overrides sends the tick and the credit limit choice', async () => {
    const store = fakeRules();
    const user = await openTab('settings-tab-overrides');
    const allow = await screen.findByRole('checkbox', {
      name: named(message(language, 'settings-overrides-allow')),
    });
    expect(allow).not.toBeChecked();
    await user.click(allow);
    await user.click(await dropdown(language, 'settings-overrides-credit'));
    await user.click(
      await screen.findByRole('option', {
        name: named(message(language, 'settings-overrides-credit-block')),
      }),
    );
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetOverrides).toHaveBeenCalledWith({
        allow_line_override: true,
        credit_limit_mode: 'block',
        rate_check_enabled: true,
        rate_check_percent: 30,
      });
    });
    expect(
      (await screen.findAllByText(message(language, 'settings-overrides-saved'))).length,
    ).toBeGreaterThan(0);
    expect(store.overrides.credit_limit_mode).toBe('block');
  });

  it('a failed load of the charges shows the message with a retry', async () => {
    fakeRules();
    vi.mocked(settingsApi.settingsGetCharges).mockRejectedValueOnce(
      fail({ code: 'ERR_NOT_IMPLEMENTED' }),
    );
    const user = await openTab('settings-tab-charges');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect((await block('settings-charges-seller')).value.value).toBe('6');
  });
});
