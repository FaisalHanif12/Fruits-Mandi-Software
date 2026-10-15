// The accounts and the chart of accounts (plan 07, PRD 10.1, 10.2, FR-10.5, FR-7.63, E2E-23): the
// list, the form with every field, the duplicate warning, the opening balance, the chart tree and
// its keys, in the Urdu and in the English app, by keyboard, against a fake backend.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as accountApi from '../../src/shared/accounts/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, balance, fakeAccounts, money, ref } from '../helpers/accounts';
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
import { message, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const AKRAM = account(21, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  city: ref(1, '0001', 'کوٹ رادھا کشن', 'Kot Radha Kishan'),
  balance: balance(2880, 'dr'),
  credit_limit_paisa: money(50000),
});
const ARSHAD = account(22, 'buyer', '03040002', 'ارشد ملتانی', 'Arshad Multani', {
  balance: balance(18684, 'dr'),
});
const RAHIM = account(31, 'seller', '030110001', 'رحیم', 'Rahim', {
  opening_dr_paisa: money(15000),
  opening_is_advance: true,
});
const OLD = account(41, 'buyer', '03040009', 'پرانا', 'Old One', { is_active: false });
const CASH = account(51, 'other', '01010001', 'روکڑ', 'Cash in Hand', { is_system: true });

const named = (text: string) => (name: string) => visible(name) === text;

function fakeTranslit() {
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: { tariq: ['طارق'], mehmood: ['محمود'], zaheer: ['ظہیر'], ahmad: ['احمد'] },
          toEnglish: { طارق: 'Tariq', محمود: 'Mehmood', ظہیر: 'Zaheer', احمد: 'Ahmad' },
          rulesOnly: ['zaheer'],
        },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
}

function cell(rowId: number, colId: string): string {
  const row = document.querySelector(`.ag-row[row-id="${String(rowId)}"]`);
  return visible(row?.querySelector(`[col-id="${colId}"]`)?.textContent);
}
const rowIds = () =>
  Array.from(document.querySelectorAll('.ag-row[row-id]')).map((row) => row.getAttribute('row-id'));

const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  fakeTranslit();
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the account list (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('shows the columns of FR-10.5 with the balance and its side as a word, and the totals', async () => {
    fakeAccounts([AKRAM, ARSHAD, OLD]);
    await renderApp('/index/accounts');
    await screen.findByRole('heading', { level: 1, name: message(language, 'accounts-title') });
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '22']);
    });
    expect(cell(21, 'code')).toBe('03040001');
    expect(cell(21, 'name')).toBe(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(cell(21, 'city')).toBe(language === 'ur' ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan');
    expect(cell(21, 'opening_date')).toBe('31-08-2026');
    expect(cell(21, 'credit_allowed')).toBe(message(language, 'common-yes'));
    expect(cell(21, 'balance')).toBe('2,880');
    expect(cell(21, 'side')).toBe(message(language, 'common-side-dr'));
    expect(cell(22, 'balance')).toBe('18,684');
    // The footer totals come from the backend.
    const footer = visible(document.body.textContent);
    expect(footer).toContain(message(language, 'accounts-total-dr'));
    expect(footer).toContain('65,000');
  });

  it('the wide layout puts the totals under the opening columns, in a pinned row, with no second line', async () => {
    fakeAccounts([AKRAM, RAHIM]);
    vi.stubGlobal('matchMedia', (query: string) => ({
      matches: true,
      media: query,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
    }));
    document.documentElement.style.setProperty('--breakpoint-normal', '1366px');
    document.documentElement.style.setProperty('--breakpoint-wide', '1920px');
    try {
      await renderApp('/index/accounts');
      await waitFor(() => {
        expect(rowIds().slice(0, 2)).toEqual(['21', '31']);
      });
      const pinned = document.querySelector('.ag-row-pinned');
      expect(pinned).not.toBeNull();
      const under = (column: string) =>
        visible(pinned?.querySelector(`[col-id="${column}"]`)?.textContent);
      expect(under('name')).toBe(message(language, 'glossary-meezan'));
      // The backend's totals (the fake sends 65,000 on each side), each under its own column.
      expect(under('opening_dr')).toBe('65,000');
      expect(under('opening_cr')).toBe('65,000');
      expect(
        screen.queryByText(named(message(language, 'accounts-total-dr'))),
      ).not.toBeInTheDocument();
    } finally {
      vi.unstubAllGlobals();
      document.documentElement.style.removeProperty('--breakpoint-normal');
      document.documentElement.style.removeProperty('--breakpoint-wide');
    }
  });

  it('the type, the search and the inactive tick go to the backend filter', async () => {
    fakeAccounts([AKRAM, RAHIM, OLD]);
    await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31']);
    });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('checkbox', { name: message(language, 'accounts-show-inactive') }),
    );
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '31', '41']);
    });
    expect(accountApi.accountsList).toHaveBeenLastCalledWith({
      account_type: null,
      text: '',
      include_inactive: true,
    });
    expect(document.querySelector('[row-id="41"]')).toHaveClass('row-inactive');
    // Dimmed is not enough (NFR-USE-12): the name says "inactive" in words, in the app language.
    expect(cell(41, 'name')).toContain(message(language, 'masters-status-inactive'));
    expect(cell(21, 'name')).not.toContain(message(language, 'masters-status-inactive'));
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'accounts-filter-type')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'accounts-type-seller')) }),
    );
    await waitFor(() => {
      expect(rowIds()).toEqual(['31']);
    });
    await user.type(await labelled(language, 'accounts-filter-label'), 'zzz');
    expect(
      await screen.findByRole('heading', { name: message(language, 'accounts-no-match-title') }),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: message(language, 'accounts-no-match-clear') }),
    );
    await waitFor(() => {
      expect(rowIds().length).toBeGreaterThan(1);
    });
  });

  it('with nothing in it, says why, and F2 opens the form for a new one', async () => {
    fakeAccounts([]);
    const { router } = await renderApp('/index/accounts');
    expect(
      await screen.findByRole('heading', { name: message(language, 'accounts-empty') }),
    ).toBeInTheDocument();
    press('F2');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/accounts/new');
    });
  });

  it('a failure shows the message with a reference and a retry that works', async () => {
    fakeAccounts([AKRAM]);
    vi.mocked(accountApi.accountsList).mockRejectedValueOnce(fail({ code: 'ERR_DB_BUSY' }));
    await renderApp('/index/accounts');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'accounts-load-failed'));
    await userEvent
      .setup()
      .click(within(alert).getByRole('button', { name: message(language, 'common-retry') }));
    await waitFor(() => {
      expect(rowIds()).toEqual(['21']);
    });
  });

  it('Enter on a row opens its form', async () => {
    fakeAccounts([AKRAM, ARSHAD]);
    const { router } = await renderApp('/index/accounts');
    await waitFor(() => {
      expect(rowIds()).toEqual(['21', '22']);
    });
    const user = userEvent.setup();
    const code = document.querySelector<HTMLElement>('.ag-row[row-id="22"] [col-id="code"]');
    if (code === null) throw new Error('no cell');
    await user.click(code);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/accounts/22');
    });
  });
});

describe.each(LANGUAGES)('the account form (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('E2E-23: a new buyer typed `tariq mehmood` gets code 03040003 and both names', async () => {
    const store = fakeAccounts([AKRAM, ARSHAD]);
    const { router } = await renderApp('/index/accounts/new');
    const code = await labelled(language, 'accounts-field-code');
    await waitFor(() => {
      expect(code.value).toBe('03040003');
    });
    const user = userEvent.setup();
    if (language === 'ur') {
      await user.click(await labelled(language, 'accounts-field-name-ur'));
      await user.keyboard('tariq mehmood{Enter}');
    } else {
      await user.click(await labelled(language, 'accounts-field-name-en'));
      await user.keyboard('Tariq Mehmood');
    }
    await waitFor(() => {
      expect(
        screen.getByLabelText<HTMLInputElement>(named(message(language, 'accounts-field-name-en')))
          .value,
      ).toBe('Tariq Mehmood');
    });
    expect(
      screen.getByLabelText<HTMLInputElement>(named(message(language, 'accounts-field-name-ur')))
        .value,
    ).toBe('طارق محمود');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    const [draft, confirmed] = vi.mocked(accountApi.accountsSave).mock.calls[0] ?? [];
    expect(draft).toMatchObject({
      id: null,
      account_type: 'buyer',
      code: '03040003',
      names: { urdu: 'طارق محمود', english: 'Tariq Mehmood' },
      credit_allowed: true,
      opening_date: '10-10-2026',
    });
    expect(confirmed).toEqual([]);
    expect(store.rows).toHaveLength(3);
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/accounts');
    });
    expect(await screen.findByText(message(language, 'accounts-saved'))).toBeInTheDocument();
  });

  it('changing the type to seller gives the seller code, and typing a code of his own keeps it', async () => {
    fakeAccounts([AKRAM, RAHIM]);
    await renderApp('/index/accounts/new');
    const code = await labelled(language, 'accounts-field-code');
    await waitFor(() => {
      expect(code.value).toBe('03040002');
    });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'accounts-field-type')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'accounts-type-seller')) }),
    );
    await waitFor(() => {
      expect(code.value).toBe('030110002');
    });
    await user.click(code);
    await user.keyboard('{Control>}a{/Control}030110077');
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'accounts-field-type')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'accounts-type-buyer')) }),
    );
    // A new type asks for a new suggestion: the code of the old type is not kept.
    await waitFor(() => {
      expect(code.value).toBe('03040002');
    });
  });

  it('the advance tick shows only for a seller with an opening debit', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    const tick = () =>
      screen.queryByLabelText(named(message(language, 'accounts-field-opening-advance')));
    await user.click(
      await screen.findByRole('combobox', {
        name: named(message(language, 'accounts-field-type')),
      }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'accounts-type-seller')) }),
    );
    expect(tick()).not.toBeInTheDocument();
    await user.click(await labelled(language, 'accounts-field-opening-dr'));
    await user.keyboard('15000');
    expect(tick()).toBeInTheDocument();
    await user.clear(await labelled(language, 'accounts-field-opening-dr'));
    expect(tick()).not.toBeInTheDocument();
  });

  it('credit and collection are for a buyer; the default mark is for a seller', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const credit = () => screen.queryByText(named(message(language, 'accounts-section-credit')));
    expect(
      await screen.findByText(message(language, 'accounts-section-account')),
    ).toBeInTheDocument();
    expect(credit()).toBeInTheDocument();
    expect(
      screen.queryByText(named(message(language, 'accounts-field-marka'))),
    ).not.toBeInTheDocument();
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('combobox', { name: named(message(language, 'accounts-field-type')) }),
    );
    await user.click(
      await screen.findByRole('option', { name: named(message(language, 'accounts-type-seller')) }),
    );
    expect(credit()).not.toBeInTheDocument();
    expect(
      await screen.findByRole('combobox', {
        name: named(message(language, 'accounts-field-marka')),
      }),
    ).toBeInTheDocument();
  });

  it('a new buyer starts with credit allowed ticked (PRD 10.2)', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    expect(
      await screen.findByRole('checkbox', {
        name: named(message(language, 'accounts-field-credit-allowed')),
      }),
    ).toBeChecked();
  });

  it('accounts that look like the new one show below the name, with city and phone', async () => {
    fakeAccounts([AKRAM]);
    vi.mocked(accountApi.accountsSimilar).mockResolvedValue([
      {
        id: 21,
        code: '03040001',
        name_ur: 'اکرم ٹول ٹیکس',
        name_en: 'Akram Toll Tax',
        father_or_firm_ur: 'ٹول ٹیکس',
        father_or_firm_en: 'Toll Tax',
        city_ur: 'کوٹ رادھا کشن',
        city_en: 'Kot Radha Kishan',
        phone1: '03001234567',
        account_type: 'buyer',
      },
    ]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(
      await labelled(
        language,
        language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en',
      ),
    );
    await user.keyboard(language === 'ur' ? 'ahmad ' : 'Ahmad');
    const heading = await screen.findByRole('heading', {
      name: message(language, 'accounts-similar-title'),
    });
    const panel = heading.closest('section');
    expect(panel).not.toBeNull();
    const text = visible(panel?.textContent);
    expect(text).toContain('03040001');
    expect(text).toContain(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    expect(text).toContain(language === 'ur' ? 'کوٹ رادھا کشن' : 'Kot Radha Kishan');
    expect(text).toContain('03001234567');
    expect(accountApi.accountsSimilar).toHaveBeenCalledWith(expect.objectContaining({ phone: '' }));
  });

  it('an account being edited does not warn about itself', async () => {
    fakeAccounts([AKRAM]);
    vi.mocked(accountApi.accountsSimilar).mockResolvedValue([
      {
        id: 21,
        code: '03040001',
        name_ur: 'اکرم ٹول ٹیکس',
        name_en: 'Akram Toll Tax',
        father_or_firm_ur: '',
        father_or_firm_en: '',
        city_ur: '',
        city_en: '',
        phone1: '',
        account_type: 'buyer',
      },
    ]);
    await renderApp('/index/accounts/21');
    await labelled(language, 'accounts-field-code');
    await waitFor(() => {
      expect(accountApi.accountsSimilar).toHaveBeenCalledWith(
        expect.objectContaining({ exclude_id: 21 }),
      );
    });
    expect(
      screen.queryByRole('heading', { name: message(language, 'accounts-similar-title') }),
    ).not.toBeInTheDocument();
  });

  it('an account being edited shows what was saved, the opening money as the text the backend reads', async () => {
    fakeAccounts([RAHIM]);
    await renderApp('/index/accounts/31');
    expect((await labelled(language, 'accounts-field-code')).value).toBe('030110001');
    expect((await labelled(language, 'accounts-field-opening-dr')).value).toBe('15,000');
    expect((await labelled(language, 'accounts-field-opening-cr')).value).toBe('');
    expect((await labelled(language, 'accounts-field-opening-date')).value).toBe('31-08-2026');
    expect(
      await screen.findByRole('checkbox', {
        name: named(message(language, 'accounts-field-opening-advance')),
      }),
    ).toBeChecked();
    press('F10');
    // Nothing changed: it saves what it read, with the id and the version.
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      id: 31,
      version: 1,
      opening_dr: '15,000',
      opening_cr: '',
      opening_is_advance: true,
    });
  });

  // The backend clears an opening when its fields come back empty (plan 07), so a form that opened
  // an account with an opening must send that opening back unchanged when only another field
  // is edited.
  it('editing only the phone sends the stored opening back unchanged: date, debit and the advance mark', async () => {
    fakeAccounts([RAHIM]);
    await renderApp('/index/accounts/31');
    const phone = await labelled(language, 'accounts-field-phone1');
    const user = userEvent.setup();
    await user.click(phone);
    await user.keyboard('03001234567');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0];
    expect(sent).toMatchObject({
      id: 31,
      phone1: '03001234567',
      opening_date: '31-08-2026',
      opening_dr: '15,000',
      opening_cr: '',
      opening_is_advance: true,
    });
  });

  it('editing only the note sends an opening credit and a credit limit back unchanged', async () => {
    const buyer = account(23, 'buyer', '03040003', 'ناصر', 'Nasir', {
      opening_date: '2026-08-31',
      opening_cr_paisa: money(2500),
      credit_limit_paisa: money(50000),
    });
    fakeAccounts([buyer]);
    await renderApp('/index/accounts/23');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-note'));
    await user.keyboard('x');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      note: 'x',
      opening_date: '31-08-2026',
      opening_dr: '',
      opening_cr: '2,500',
      credit_limit: '50,000',
    });
  });

  it('an opening is sent empty only when the user cleared it', async () => {
    fakeAccounts([RAHIM]);
    await renderApp('/index/accounts/31');
    const debit = await labelled(language, 'accounts-field-opening-dr');
    const user = userEvent.setup();
    await user.clear(debit);
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      opening_dr: '',
      opening_cr: '',
    });
  });

  it('an account with no opening sends none when another field is edited', async () => {
    const plain = account(24, 'buyer', '03040004', 'بغیر', 'Plain', { opening_date: null });
    fakeAccounts([plain]);
    await renderApp('/index/accounts/24');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-note'));
    await user.keyboard('x');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[0]).toMatchObject({
      opening_date: '',
      opening_dr: '',
      opening_cr: '',
      opening_is_advance: false,
    });
  });

  it('a refusal at the opening date sits under that field, and what was typed stays', async () => {
    fakeAccounts([]);
    vi.mocked(accountApi.accountsSave).mockRejectedValueOnce(
      fail({
        code: 'ERR_DATE_CLOSED',
        field: 'opening_date',
        values: { closed_up_to: '09-10-2026' },
      }),
    );
    await renderApp('/index/accounts/new');
    const date = await labelled(language, 'accounts-field-opening-date');
    const user = userEvent.setup();
    await user.click(
      await labelled(
        language,
        language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en',
      ),
    );
    await user.keyboard(language === 'ur' ? 'ahmad ' : 'Ahmad');
    press('F10');
    await waitFor(() => {
      expect(date).toHaveAttribute('aria-invalid', 'true');
    });
    expect(
      visible(document.getElementById(date.getAttribute('aria-describedby') ?? '')?.textContent),
    ).toBe(message(language, 'err-date-closed', { closed_up_to: '09-10-2026' }));
    expect(date.value).toBe('10-10-2026');
  });

  it('a system account cannot be made inactive', async () => {
    fakeAccounts([CASH, AKRAM]);
    await renderApp('/index/accounts/51');
    await labelled(language, 'accounts-field-code');
    // The button is there but off, and a line says why (PRD FR-10.4).
    expect(
      screen.getByRole('button', { name: message(language, 'accounts-make-inactive') }),
    ).toBeDisabled();
    expect(
      screen.getByText(named(message(language, 'accounts-system-stays-active'))),
    ).toBeInTheDocument();
  });

  it('if the backend refuses to make a system account inactive, the screen says so in words', async () => {
    // The screen thought the account an ordinary one (an old copy of it): the backend is the final word.
    fakeAccounts([CASH, AKRAM]);
    vi.mocked(accountApi.accountsSetActive).mockRejectedValueOnce(
      fail({ code: 'ERR_SYSTEM_ACCOUNT', field: 'id' }),
    );
    await renderApp('/index/accounts/21');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'accounts-make-inactive') }),
    );
    const box = await screen.findByRole('alertdialog');
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'accounts-make-inactive')) }),
    );
    expect(
      (await screen.findAllByText(named(message(language, 'err-system-account')))).length,
    ).toBeGreaterThan(0);
  });

  it('making an account inactive asks first, says what happens, and nothing is deleted', async () => {
    const store = fakeAccounts([CASH, AKRAM]);
    await renderApp('/index/accounts/21');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('button', { name: message(language, 'accounts-make-inactive') }),
    );
    const box = await screen.findByRole('alertdialog');
    expect(accountApi.accountsSetActive).not.toHaveBeenCalled();
    await user.click(
      within(box).getByRole('button', { name: named(message(language, 'accounts-make-inactive')) }),
    );
    await waitFor(() => {
      expect(accountApi.accountsSetActive).toHaveBeenCalledWith(21, false);
    });
    expect(store.rows).toHaveLength(2);
  });

  it('a rules-only word in the father or firm name asks to confirm, after the name is checked', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(
      await labelled(
        language,
        language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en',
      ),
    );
    await user.keyboard(language === 'ur' ? 'ahmad ' : 'Ahmad');
    await user.click(
      await labelled(
        language,
        language === 'ur' ? 'accounts-field-father-ur' : 'accounts-field-father-en',
      ),
    );
    await user.keyboard(language === 'ur' ? 'zaheer ' : 'Zaheer');
    await waitFor(() => {
      expect(screen.getByRole('status').textContent).toContain('ظہیر');
    });
    press('F10');
    const box = await screen.findByRole('alertdialog');
    expect(box.querySelector('mark')?.textContent).toBe('ظہیر');
    expect(accountApi.accountsSave).not.toHaveBeenCalled();
    await user.click(
      within(box).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.accountsSave).mock.calls[0]?.[1]).toEqual(['ظہیر']);
  });

  it('a language switch in the middle of the form keeps every value', async () => {
    fakeAccounts([AKRAM]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-phone1'));
    await user.keyboard('03001234567');
    await user.click(await labelled(language, 'accounts-field-opening-dr'));
    await user.keyboard('65000');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.getState().setLanguage(other);
    });
    expect((await labelled(other, 'accounts-field-phone1')).value).toBe('03001234567');
    expect((await labelled(other, 'accounts-field-opening-dr')).value).toBe('65,000');
    expect((await labelled(other, 'accounts-field-code')).value).toBe('03040002');
  });

  it('F4 on the city picker adds a city without losing what is typed (PRD FR-7.64)', async () => {
    fakeAccounts([]);
    fakeMasters({ city: [] });
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-phone1'));
    await user.keyboard('03001234567');
    const city = await screen.findByRole('combobox', {
      name: named(message(language, 'accounts-field-city')),
    });
    await user.click(city);
    await user.keyboard('Ahmad');
    await waitFor(() => {
      expect(screen.getByText(message(language, 'search-no-match'))).toBeInTheDocument();
    });
    act(() => {
      city.dispatchEvent(new KeyboardEvent('keydown', { code: 'F4', key: 'F4', bubbles: true }));
    });
    const dialog = await screen.findByRole('dialog');
    expect(
      within(dialog).getByLabelText<HTMLInputElement>(
        named(message(language, 'masters-field-name-en')),
      ).value,
    ).toBe('Ahmad');
    await user.click(
      within(dialog).getByRole('button', {
        name: (name) => visible(name).startsWith(message(language, 'layout-save-save')),
      }),
    );
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
    // The new city is in the field, and the phone typed before is still there.
    expect((city as HTMLInputElement).value).toBe(language === 'ur' ? 'احمد' : 'Ahmad');
    expect((await labelled(language, 'accounts-field-phone1')).value).toBe('03001234567');
  });

  it('leaving with unsaved changes asks first', async () => {
    fakeAccounts([AKRAM]);
    const { router } = await renderApp('/index/accounts/21');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-phone1'));
    await user.keyboard('0300');
    press('Escape');
    const prompt = await screen.findByRole('alertdialog');
    expect(visible(prompt.textContent)).toContain(message(language, 'layout-save-title'));
    expect(router.state.location.pathname).toBe('/index/accounts/21');
  });
});

describe.each(LANGUAGES)('the chart of accounts (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const rows = () => screen.getAllByRole('treeitem');
  const ROW_TEXT = (index: number) => visible(rows()[index]?.textContent);

  it('shows heads and controls at first, each with its balance and the side as a word', async () => {
    fakeAccounts();
    await renderApp('/index/chart');
    await screen.findByRole('tree');
    expect(rows().map((row) => row.getAttribute('aria-level'))).toEqual(['1', '2', '1', '2', '2']);
    expect(ROW_TEXT(0)).toContain('58,484');
    expect(ROW_TEXT(0)).toContain(message(language, 'common-side-dr'));
    expect(ROW_TEXT(0)).toContain('01');
    // A control with nothing in it shows 0 and no side word.
    expect(ROW_TEXT(4)).toContain('03011');
    expect(ROW_TEXT(4)).not.toContain(message(language, 'common-side-dr'));
  });

  it('the arrow at the children opens a row, the other closes it; ↑ ↓ move; Enter on an account opens it', async () => {
    fakeAccounts();
    const { router } = await renderApp('/index/chart');
    const tree = await screen.findByRole('tree');
    const user = userEvent.setup();
    tree.focus();
    // In right-to-left the left arrow points at the children.
    const toChildren = language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}';
    const toParent = language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}';
    await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}');
    expect(tree).toHaveAttribute('aria-activedescendant', rows()[3]?.id);
    await user.keyboard(toChildren);
    await waitFor(() => {
      expect(rows()).toHaveLength(7);
    });
    expect(rows().map((row) => row.getAttribute('aria-level'))).toEqual([
      '1',
      '2',
      '1',
      '2',
      '3',
      '3',
      '2',
    ]);
    expect(rows()[3]).toHaveAttribute('aria-expanded', 'true');
    // On an open row the same arrow goes to its first account.
    await user.keyboard(toChildren);
    expect(tree).toHaveAttribute('aria-activedescendant', rows()[4]?.id);
    expect(ROW_TEXT(4)).toContain('03040001');
    expect(ROW_TEXT(4)).toContain('2,880');
    // The other arrow goes back to the control, then closes it.
    await user.keyboard(toParent);
    expect(tree).toHaveAttribute('aria-activedescendant', rows()[3]?.id);
    await user.keyboard(toParent);
    await waitFor(() => {
      expect(rows()).toHaveLength(5);
    });
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rows()).toHaveLength(7);
    });
    await user.keyboard('{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(router.state.location.pathname).toBe('/index/accounts/21');
    });
  });

  it('Home and End jump to the first and the last row', async () => {
    fakeAccounts();
    await renderApp('/index/chart');
    const tree = await screen.findByRole('tree');
    tree.focus();
    await userEvent.setup().keyboard('{End}');
    expect(tree).toHaveAttribute('aria-activedescendant', rows()[4]?.id);
    await userEvent.setup().keyboard('{Home}');
    expect(tree).toHaveAttribute('aria-activedescendant', rows()[0]?.id);
  });

  it('the head list and the control list are their own tabs', async () => {
    fakeAccounts();
    await renderApp('/index/chart');
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('tab', { name: message(language, 'chart-tab-heads') }),
    );
    await waitFor(() => {
      expect(rowIds()).toEqual(['1', '3']);
    });
    expect(cell(1, 'balance')).toBe('58,484');
    expect(cell(1, 'side')).toBe(message(language, 'common-side-dr'));
    await user.click(screen.getByRole('tab', { name: message(language, 'chart-tab-controls') }));
    await waitFor(() => {
      expect(rowIds()).toEqual(['101', '4', '11']);
    });
  });

  it('a failure shows the message with a retry', async () => {
    fakeAccounts();
    vi.mocked(accountApi.coaTree).mockRejectedValueOnce(fail({ code: 'ERR_DB_BUSY' }));
    await renderApp('/index/chart');
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'chart-load-failed'));
    await userEvent
      .setup()
      .click(within(alert).getByRole('button', { name: message(language, 'common-retry') }));
    expect(await screen.findByRole('tree')).toBeInTheDocument();
  });
});
