// The optional `party` address parameter of the entry screens (plan 49, PRD FR-7.79): the button under
// an answer of the question panel opens an entry screen with the party chosen. The party is a whole
// picker value (id, names, code, tag) in the address; the screen puts him in its field, every amount
// stays empty, and nothing is saved by it. With no parameter each screen is as it was, and a bare id
// or anything that is not a whole record is left out. One screen of each kind, in the Urdu and in the
// English app: the voucher, the expense, the money lent, the khata, the capital, the fruit and the
// journal.

import { screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as fruitApi from '../../src/features/fruit/api';
import * as partnersApi from '../../src/features/partners/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import {
  parseExpenseSearch,
  parseGeneralSearch,
  parseKhataSearch,
  parseLentSearch,
} from '../../src/features/cashbook/screen-type';
import { parseFruitSearch } from '../../src/features/fruit/screen-search';
import { parseCapitalSearch } from '../../src/features/partners/screen-type';
import { parseVoucherSearch } from '../../src/features/vouchers/screen-type';
import { LANGUAGES, type Language } from '../../src/i18n';
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
import { gridOf, textOf } from '../helpers/bikri-screen';
import { ASLAM, FAMILY_KHATA } from '../helpers/cashbook';
import { dropdown, labelled, named } from '../helpers/cashbook-screen';
import { fakeFruitPickers } from '../helpers/fruit';
import { fakeMasters, record } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { preparePartnerMoney } from '../helpers/partner-money';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import {
  AKRAM,
  ARSHAD,
  KARIM,
  PUKHTA_BUYER,
  RAHIM,
  TRANSPORT,
  fakeVouchers,
} from '../helpers/vouchers';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/pukhta_khata/api');
vi.mock('../../src/features/reports/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/features/fruit/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** A whole record, the way the question panel puts it in an address. */
function whole(id: number, code: string, ur: string, en: string, tag: string | null = null) {
  return { id, code, name_ur: ur, name_en: en, tag };
}

/** The address of a screen with the party in it, as a link would carry it. */
function addressOf(path: string, party: unknown, more: Record<string, string> = {}): string {
  const query = new URLSearchParams({ ...more, party: JSON.stringify(party) });
  return `${path}?${query.toString()}`;
}

const nameOf = (language: Language, ur: string, en: string) => (language === 'ur' ? ur : en);

describe('the address parsers read a whole party, and only that', () => {
  const party = { id: 7, name_ur: 'اسلم', name_en: 'Aslam', code: '0001', tag: 'BA' };

  it('with no party every screen reads its address exactly as before', () => {
    expect(parseVoucherSearch({ type: 'receipt' })).toEqual({ type: 'receipt' });
    expect(parseExpenseSearch({ type: 'shop_expense', id: '4' })).toEqual({
      type: 'shop_expense',
      id: 4,
    });
    expect(parseLentSearch({})).toEqual({});
    expect(parseKhataSearch({ type: 'khata_in' })).toEqual({ type: 'khata_in' });
    expect(parseGeneralSearch({ id: 3 })).toEqual({ id: 3 });
    expect(parseFruitSearch({})).toEqual({});
    expect(parseFruitSearch({ entry: '7' })).toEqual({ entry: 7 });
    expect(parseFruitSearch({ entry: 'x' })).toEqual({ entry: 'bad' });
    expect(parseCapitalSearch({ type: 'capital_out', partner: 2 })).toEqual({
      type: 'capital_out',
      partner: 2,
    });
  });

  it('a whole party is kept with its names, code and tag; the empty tag is left out', () => {
    const kept = { id: 7, name_ur: 'اسلم', name_en: 'Aslam', code: '0001', tag: 'BA' };
    expect(parseVoucherSearch({ type: 'advance', party })).toEqual({
      type: 'advance',
      party: kept,
    });
    expect(parseLentSearch({ party }).party).toEqual(kept);
    expect(parseKhataSearch({ party }).party).toEqual(kept);
    expect(parseExpenseSearch({ party }).party).toEqual(kept);
    expect(parseGeneralSearch({ party }).party).toEqual(kept);
    expect(parseFruitSearch({ party }).party).toEqual(kept);
    const noTag = { ...party, tag: null };
    expect(parseLentSearch({ party: noTag }).party).toEqual({
      id: 7,
      name_ur: 'اسلم',
      name_en: 'Aslam',
      code: '0001',
    });
  });

  it('a bare id, text or a record without an id is not a party', () => {
    for (const bad of [7, '7', 'aslam', null, {}, { name_en: 'Aslam' }, { id: 0 }, { id: -3 }]) {
      expect(parseVoucherSearch({ party: bad })).toEqual({});
      expect(parseLentSearch({ party: bad })).toEqual({});
      expect(parseFruitSearch({ party: bad })).toEqual({});
    }
  });

  it('the capital screen knows a partner by his id: a whole party is the same as `partner`, which wins', () => {
    expect(parseCapitalSearch({ party })).toEqual({ partner: 7 });
    expect(parseCapitalSearch({ party, partner: 2 })).toEqual({ partner: 2 });
    expect(parseCapitalSearch({ party: 7 })).toEqual({});
  });
});

describe.each(LANGUAGES)('the entry screens opened with a party (%s)', (language) => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  afterEach(() => {
    leaveAppWindow();
  });

  const aslam = nameOf(language, 'اسلم', 'Aslam');
  const partnerB = nameOf(language, 'ب', 'Partner B');

  /** Nothing was saved, whatever the screen: no save command was called. */
  const nothingSaved = () => {
    for (const save of [
      cashbookApi.saveShopExpense,
      cashbookApi.savePersonalExpense,
      cashbookApi.savePersonalReturn,
      cashbookApi.saveMoneyLent,
      cashbookApi.saveMoneyLentReturn,
      cashbookApi.saveKhataOut,
      cashbookApi.saveKhataIn,
      cashbookApi.generalSave,
      partnersApi.saveCapitalIn,
      partnersApi.saveCapitalOut,
      fruitApi.fruitSave,
      vouchersApi.vouchersSave,
    ]) {
      expect(save).not.toHaveBeenCalled();
    }
  };

  describe('the cash book screens', () => {
    beforeEach(() => {
      preparePartnerMoney(language);
    });

    it('the ادھار رقم screen has the person chosen, with his tag, and the amount empty', async () => {
      await renderApp(
        addressOf('/rokar/lent', whole(ASLAM.id, ASLAM.code, 'اسلم', 'Aslam', 'BA'), {
          type: 'money_lent',
        }),
      );
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'cashbook-lent-screen-title'),
      });
      expect(await dropdown(language, 'cashbook-field-person')).toHaveValue(`${aslam} BA`);
      expect((await labelled(language, 'cashbook-field-amount')).value).toBe('');
      expect((await labelled(language, 'cashbook-field-reason')).value).toBe('');
      nothingSaved();
    });

    it('the الگ کھاتہ screen has the khata chosen and the amount empty', async () => {
      await renderApp(
        addressOf(
          '/rokar/khata',
          whole(FAMILY_KHATA.id, FAMILY_KHATA.code, 'خاندانی کھاتہ', 'Family Khata'),
        ),
      );
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'cashbook-khata-screen-title'),
      });
      expect(await dropdown(language, 'cashbook-field-khata')).toHaveValue(
        nameOf(language, 'خاندانی کھاتہ', 'Family Khata'),
      );
      expect((await labelled(language, 'cashbook-field-amount')).value).toBe('');
      nothingSaved();
    });

    it('the خرچہ screen starts as the partner’s own spending with him chosen, and the amount empty', async () => {
      await renderApp(addressOf('/rokar/expense', whole(2, '', 'ب', 'Partner B')));
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'cashbook-expense-title'),
      });
      expect(await dropdown(language, 'cashbook-field-partner')).toHaveTextContent(partnerB);
      expect((await labelled(language, 'cashbook-field-amount')).value).toBe('');
      nothingSaved();
    });

    it('the خرچہ screen keeps the shop expense when the address names it: no partner there', async () => {
      await renderApp(
        addressOf('/rokar/expense', whole(2, '', 'ب', 'Partner B'), { type: 'shop_expense' }),
      );
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'cashbook-expense-title'),
      });
      expect(
        screen.queryByRole('combobox', {
          name: named(message(language, 'cashbook-field-partner')),
        }),
      ).toBeNull();
      expect((await labelled(language, 'cashbook-field-amount')).value).toBe('');
    });

    it('the سرمایہ screen has the partner chosen from a whole party', async () => {
      await renderApp(
        addressOf('/partners/capital', whole(2, '', 'ب', 'Partner B'), { type: 'capital_in' }),
      );
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'partners-capital-screen-title'),
      });
      expect(await dropdown(language, 'cashbook-field-partner')).toHaveTextContent(partnerB);
      expect((await labelled(language, 'cashbook-field-amount')).value).toBe('');
      nothingSaved();
    });

    it('the جنرل screen has the account on its first line, with no amount on either side', async () => {
      await renderApp(
        addressOf('/rokar/general', whole(21, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax')),
      );
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'cashbook-general-screen-title')),
      });
      const grid = gridOf(language, 'cashbook-general-grid-label');
      await waitFor(() => {
        expect(textOf(grid, 0, 'account')).toContain(
          nameOf(language, 'اکرم ٹول ٹیکس', 'Akram Toll Tax'),
        );
      });
      expect(textOf(grid, 0, 'dr')).toBe('');
      expect(textOf(grid, 0, 'cr')).toBe('');
      // The second line is as empty as it always was.
      expect(textOf(grid, 1, 'account')).toBe('');
      nothingSaved();
    });

    it('with no party the screens open as they always did', async () => {
      await renderApp('/rokar/lent');
      await screen.findByRole('heading', {
        level: 1,
        name: message(language, 'cashbook-lent-screen-title'),
      });
      expect(await dropdown(language, 'cashbook-field-person')).toHaveValue('');
    });
  });

  describe('the voucher screen', () => {
    beforeEach(() => {
      healthyBackend();
      resetClientState(language);
      signedIn(sessionUser({ language }));
      vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
      const rows = [AKRAM, ARSHAD, PUKHTA_BUYER, RAHIM, KARIM, TRANSPORT];
      fakeAccounts(rows);
      fakeAccountSearch(rows);
      fakeMasters({ staff: [record('staff', 81, '0001', 'اسلم', 'Aslam')] });
      fakeVouchers();
    });

    it('a وصولی opens with the buyer chosen and the amount empty', async () => {
      await renderApp(
        addressOf('/bikri/voucher', whole(AKRAM.id, '0020', 'اکرم ٹول ٹیکس', 'Akram Toll Tax'), {
          type: 'receipt',
        }),
      );
      const buyer = await dropdown(language, 'vouchers-field-buyer');
      await waitFor(() => {
        expect(visible((buyer as HTMLInputElement).value)).toContain(
          nameOf(language, 'اکرم ٹول ٹیکس', 'Akram Toll Tax'),
        );
      });
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('');
      nothingSaved();
    });

    it('an ادائیگی opens with the seller chosen and the amount empty', async () => {
      await renderApp(
        addressOf('/bikri/voucher', whole(RAHIM.id, '0010', 'رحیم', 'Rahim'), { type: 'payment' }),
      );
      const seller = await dropdown(language, 'vouchers-field-paid-to');
      await waitFor(() => {
        expect(visible((seller as HTMLInputElement).value)).toContain(
          nameOf(language, 'رحیم', 'Rahim'),
        );
      });
      expect((await labelled(language, 'vouchers-field-amount')).value).toBe('');
      nothingSaved();
    });

    it('with no party the voucher opens empty, as it always did', async () => {
      await renderApp('/bikri/voucher?type=receipt');
      expect((await dropdown(language, 'vouchers-field-buyer')) as HTMLInputElement).toHaveValue(
        '',
      );
    });
  });

  describe('the پھل screen', () => {
    beforeEach(() => {
      healthyBackend();
      resetClientState(language);
      signedIn(sessionUser({ language }));
      fakePartners(THREE_PARTNERS);
      fakeFruitPickers();
      vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
    });

    it('opens for the partner, with the quantity and the rate empty', async () => {
      await renderApp(addressOf('/rokar/fruit', whole(1, '', 'الف', 'Partner A')));
      await screen.findByRole('heading', {
        level: 1,
        name: named(message(language, 'fruit-title')),
      });
      expect(
        await screen.findByRole('combobox', { name: named(message(language, 'fruit-field-for')) }),
      ).toHaveTextContent(nameOf(language, 'الف', 'Partner A'));
      expect(
        screen.getByRole('textbox', { name: named(message(language, 'fruit-field-qty')) }),
      ).toHaveValue('');
      expect(
        screen.getByRole('textbox', { name: named(message(language, 'fruit-field-rate')) }),
      ).toHaveValue('');
      nothingSaved();
    });
  });
});
