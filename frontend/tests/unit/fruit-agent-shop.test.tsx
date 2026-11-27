// پھل from another shop (plan 97 item 1, PRD FR-16.11 v1.59, FR-16.7, FR-7.69), in the Urdu and the English app, against a fake
// backend:
// - the آڑھتی box lists only the accounts marked آڑھتی دکان (an ordinary seller is not there);
// - a shop that is not there is added on the spot by keyboard: the box types a name, Enter on the add row opens the short form
//   (names and an optional phone), F10 saves it and the box takes it;
// - the کل کٹوتی field is there only for the other source, after the rate, and the net shows from the backend's preview:
//   2 × 1,400 = 2,800, کل کٹوتی 200, net 2,600 (a home take: only A's total falls by 2,600; the guests: 868 / 866 / 866, C27);
// - a deduction that is not below the value is refused at its field, in words;
// - the save sends the deduction as typed, and sends none for fruit from our shop.
// Every figure on the screen is a figure the backend sent; the fake's sums are written the way the PRD has them.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as fruitApi from '../../src/features/fruit/api';
import { LANGUAGES } from '../../src/i18n';
import * as accountApi from '../../src/shared/accounts/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
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
import { fakeFruit, fakeFruitPickers } from '../helpers/fruit';
import { EMPTY_PAIRS } from '../helpers/paged';
import { fakePartners, THREE_PARTNERS } from '../helpers/partners';
import { settlePreview } from '../helpers/preview-settled';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/fruit/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/drafts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

/** Two shops marked آڑھتی دکان, and an ordinary seller who is not one. */
const HAJI = account(31, 'seller', '0012', 'حاجی نذیر', 'Haji Nazir', { is_agent_shop: true });
const SHAH = account(32, 'seller', '0013', 'شاہ مارکیٹ', 'Shah Market', { is_agent_shop: true });
const RAHIM = account(21, 'seller', '0010', 'رحیم', 'Rahim');

let shops = [HAJI, SHAH, RAHIM];

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  fakePartners(THREE_PARTNERS);
  fakeFruitPickers();
  shops = [HAJI, SHAH, RAHIM];
  fakeAccounts(shops);
  fakeAccountSearch(shops);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

type User = ReturnType<typeof userEvent.setup>;

describe.each(LANGUAGES)('fruit from another shop (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const m = (id: string, vars: Record<string, string | number> = {}) => message(language, id, vars);
  const pageText = () => visible(document.body.textContent);
  const haji = language === 'ur' ? 'حاجی نذیر' : 'Haji Nazir';
  const shah = language === 'ur' ? 'شاہ مارکیٹ' : 'Shah Market';
  const rahim = language === 'ur' ? 'رحیم' : 'Rahim';
  const partnerA = language === 'ur' ? 'الف' : 'Partner A';

  const open = async () => {
    const view = await renderApp('/rokar/fruit');
    await screen.findByRole('heading', { level: 1, name: named(m('fruit-title')) });
    return { ...view, user: userEvent.setup() };
  };
  const field = (id: string) => {
    const label = named(m(id));
    return (
      screen.queryByRole('textbox', { name: label }) ??
      screen.getByRole('combobox', { name: label })
    );
  };
  const choose = async (user: User, id: string, optionText: string) => {
    await user.click(field(id));
    await user.click(await screen.findByRole('option', { name: named(optionText) }));
  };
  const otherShop = async (user: User) => {
    await user.click(
      within(screen.getByRole('group', { name: named(m('fruit-field-source')) })).getByRole(
        'button',
        { name: named(m('fruit-source-other')) },
      ),
    );
  };
  /** The item and the packing of the other source (banana and box are in the fake lists). */
  const pickGoods = async (user: User) => {
    const item = field('glossary-qism-maal');
    await user.click(item);
    await user.type(item, 'app');
    await user.click(
      await screen.findByRole('option', {
        name: (text) => visible(text).includes(language === 'ur' ? 'سیب' : 'Apple'),
      }),
    );
    const packing = field('glossary-packing');
    await user.click(packing);
    await user.type(packing, 'box');
    await user.click(await screen.findByRole('option'));
  };
  const pickAgent = async (user: User, text: string) => {
    const agent = field('fruit-field-agent');
    await user.click(agent);
    await user.type(agent, text);
    await user.click(await screen.findByRole('option'));
    await waitFor(() => {
      expect(agent).not.toHaveValue(text);
    });
  };
  async function previewSettled(deduction: string) {
    await waitFor(() => {
      expect(fruitApi.fruitPreview).toHaveBeenLastCalledWith(
        expect.objectContaining({ deduction }),
      );
    });
    await act(async () => {
      await vi.mocked(fruitApi.fruitPreview).mock.results.at(-1)?.value;
    });
    await settlePreview();
  }

  it('the آڑھتی box lists only the shops marked آڑھتی دکان, not an ordinary seller', async () => {
    fakeFruit();
    const { user } = await open();
    await otherShop(user);
    await user.click(field('fruit-field-agent'));
    const options = await screen.findAllByRole('option');
    const listed = options.map((option) => visible(option.textContent));
    expect(listed.some((text) => text.includes(haji))).toBe(true);
    expect(listed.some((text) => text.includes(shah))).toBe(true);
    expect(listed.some((text) => text.includes(rahim))).toBe(false);
    // An ordinary seller typed by name is not found either.
    await user.type(field('fruit-field-agent'), 'rahim');
    await screen.findByText(m('search-no-match'));
  });

  it('a shop that is not there is added by keyboard: Enter on the add row opens the short form, F10 saves, the box takes it', async () => {
    fakeFruit();
    const added = account(40, 'seller', '0014', 'نئی دکان', 'New Shop', { is_agent_shop: true });
    vi.mocked(accountApi.agentShopAdd).mockImplementation(() => {
      shops.push(added);
      return Promise.resolve(added);
    });
    const { user } = await open();
    await otherShop(user);
    const agent = field('fruit-field-agent');
    await user.click(agent);
    const typed = language === 'ur' ? 'نئی دکان' : 'zzzshop';
    await user.type(agent, typed);
    // The add row names the typed text and the new kind of record: "+ نئی آڑھتی دکان شامل کریں".
    const row = await screen.findByRole('button', {
      name: (name) =>
        visible(name).startsWith(m('search-add-row-named', { noun: 'agent-shop', text: typed })),
    });
    expect(row).toBeInTheDocument();
    await user.keyboard('{Enter}');
    const dialog = await screen.findByRole('dialog', { name: named(m('agent-shop-add-title')) });
    // The typed text starts the name; the other script is filled by itself; the phone is optional.
    const name = await within(dialog).findByLabelText<HTMLInputElement>(
      named(m(language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en')),
    );
    expect(name.value).toBe(typed);
    expect(within(dialog).getByLabelText(named(m('accounts-field-phone1')))).toBeInTheDocument();
    // It is the short form of a shop: no type list, no father or firm, no link to the account form.
    expect(
      within(dialog).queryByLabelText(named(m('accounts-field-father-en'))),
    ).not.toBeInTheDocument();
    if (language === 'en') {
      // The other script is filled in by itself from the typed text (FR-7.24); Urdu is typed in the Urdu app.
      await user.type(
        within(dialog).getByLabelText(named(m('accounts-field-name-ur'))),
        'نئی دکان',
      );
    } else {
      await user.type(
        within(dialog).getByLabelText(named(m('accounts-field-name-en'))),
        'New Shop',
      );
    }
    press('F10');
    await waitFor(() => {
      expect(accountApi.agentShopAdd).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.agentShopAdd).mock.calls[0]?.[0]).toMatchObject({
      phone: '',
      confirmed_rules_words: expect.any(Array) as string[],
    });
    // The box takes the new shop and the form is closed; nothing else was opened.
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: named(m('agent-shop-add-title')) })).toBeNull();
    });
    await waitFor(() => {
      expect(field('fruit-field-agent')).toHaveValue(language === 'ur' ? 'نئی دکان' : 'New Shop');
    });
  });

  it('Esc closes the short form of a shop and leaves the box as it was', async () => {
    fakeFruit();
    const { user } = await open();
    await otherShop(user);
    const agent = field('fruit-field-agent');
    await user.click(agent);
    const typed = language === 'ur' ? 'نئی دکان' : 'zzzshop';
    await user.type(agent, typed);
    await screen.findByRole('button', {
      name: (name) =>
        visible(name).startsWith(m('search-add-row-named', { noun: 'agent-shop', text: typed })),
    });
    await user.keyboard('{Enter}');
    await screen.findByRole('dialog', { name: named(m('agent-shop-add-title')) });
    press('Escape');
    await waitFor(() => {
      expect(screen.queryByRole('dialog', { name: named(m('agent-shop-add-title')) })).toBeNull();
    });
    expect(accountApi.agentShopAdd).not.toHaveBeenCalled();
  });

  it('کل کٹوتی is only for the other source: no field for our shop, and none is sent', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    expect(
      screen.queryByRole('textbox', { name: named(m('fruit-field-deduction')) }),
    ).not.toBeInTheDocument();
    await otherShop(user);
    // After the rate, only for fruit from another shop.
    expect(field('fruit-field-deduction')).toBeInTheDocument();
    expect(fake.saved.entries).toHaveLength(0);
  });

  it('home fruit with a کل کٹوتی: 2 × 1,400 = 2,800, deduction 200, net 2,600 from the backend, and A’s total falls by the net', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await otherShop(user);
    await pickAgent(user, 'nazir');
    await pickGoods(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '1400');
    await user.type(field('fruit-field-deduction'), '200');
    await previewSettled('200');
    await waitFor(() => {
      const text = pageText();
      for (const figure of ['2,800', '200', '2,600']) expect(text, figure).toContain(figure);
    });
    // The net is told with its own words, and A's part is the net: 1,424 becomes -1,176.
    expect(pageText()).toContain(m('fruit-numbers-net'));
    expect(pageText()).toContain('1,176');
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]).toMatchObject({
      source: 'other_shop',
      agent_account_id: 31,
      qty: '2',
      rate: '1400',
      deduction: '200',
    });
    // The list shows the entry with the net the backend posted.
    await waitFor(() => {
      expect(pageText()).toContain('2,600');
    });
  });

  it('guest fruit with a کل کٹوتی is shared over the net: 2,600 as 868 / 866 / 866 (C27)', async () => {
    fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', m('fruit-for-guests'));
    await otherShop(user);
    await pickAgent(user, 'nazir');
    await pickGoods(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '1400');
    await user.type(field('fruit-field-deduction'), '200');
    await previewSettled('200');
    await waitFor(() => {
      expect(pageText()).toContain(m('fruit-parts-title-guests'));
    });
    const table = screen.getAllByRole('table')[0];
    if (table === undefined) throw new Error('no parts table');
    const rows = within(table).getAllByRole('row').slice(1);
    expect(rows.map((row) => visible(within(row).getAllByRole('cell')[1]?.textContent))).toEqual([
      '868',
      '866',
      '866',
    ]);
  });

  it('a deduction that is not below the value is refused at its field, in words', async () => {
    fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await otherShop(user);
    await pickAgent(user, 'nazir');
    await pickGoods(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '1400');
    await user.type(field('fruit-field-deduction'), '2800');
    await previewSettled('2800');
    await waitFor(() => {
      expect(pageText()).toContain(
        m('err-deduction-not-below-value', { max: '2,799', value: '2,800' }),
      );
    });
  });

  it('with no deduction the card shows no deduction or net line and the saved entry sends none', async () => {
    const fake = fakeFruit();
    const { user } = await open();
    await choose(user, 'fruit-field-for', partnerA);
    await otherShop(user);
    await pickAgent(user, 'nazir');
    await pickGoods(user);
    await user.type(field('fruit-field-qty'), '2');
    await user.type(field('fruit-field-rate'), '1400');
    await waitFor(() => {
      expect(pageText()).toContain('2,800');
    });
    expect(pageText()).not.toContain(m('fruit-numbers-net'));
    press('F10');
    await waitFor(() => {
      expect(fake.saved.entries).toHaveLength(1);
    });
    expect(fake.saved.entries[0]?.deduction).toBeNull();
  });
});
