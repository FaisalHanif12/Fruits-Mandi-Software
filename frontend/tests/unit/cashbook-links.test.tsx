// The link of a الگ کھاتہ and of a person who is lent money, and the type of that person (plan 76 points 5 and
// 10, PRD FR-16.6, FR-16.13), in the Urdu and in the English app, by keyboard, against a fake backend:
// - a new khata asks its link (nobody, the shop or one partner) in its F4 form, and a refusal of the backend
//   shows at the field;
// - the khata page shows the link in words and changes it, except while the balance is not 0;
// - a new person asks his type and the link of his loans;
// - money lent has a "For" field that shows the person's link, is fixed while he owes, and sends the link only
//   when he owes nothing;
// - a salary return for a partner-linked person is refused at its field.
// The screens work out no money: the backend splits and posts.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as cashbookApi from '../../src/features/cashbook/api';
import * as vouchersApi from '../../src/features/vouchers/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  choose,
  dropdown,
  labelled,
  named,
  pageText,
  pickAccount,
  prepareCashbook,
  typeAmount,
} from '../helpers/cashbook-screen';
import { ASLAM_ROW, KHATA_ROW, type FakeCashbook } from '../helpers/cashbook';
import { fail, leaveAppWindow, press, renderApp } from '../helpers/app';
import { paisaForTest } from '../helpers/paisa';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/cashbook/api');
vi.mock('../../src/features/vouchers/api');
vi.mock('../../src/features/partners/api');
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

describe.each(LANGUAGES)('the link of a khata and of a person (%s)', (language) => {
  let books: FakeCashbook;
  const family = nameOf(language, 'خاندانی کھاتہ', 'Family Khata');
  const aslam = nameOf(language, 'اسلم', 'Aslam');
  const partnerB = nameOf(language, 'ب', 'Partner B');
  const m = (id: string) => message(language, id);
  /** A matcher for an element whose whole text is `text`, without the isolation marks of a glossary word. */
  const said = (text: string) => (content: string) => visible(content) === text;

  async function openKhata() {
    const app = await renderApp('/rokar/khata?type=khata_out');
    await screen.findByRole('heading', { level: 1, name: m('cashbook-khata-screen-title') });
    return { ...app, user: userEvent.setup() };
  }

  async function openLent(type = 'money_lent') {
    const app = await renderApp(`/rokar/lent?type=${type}`);
    await screen.findByRole('heading', { level: 1, name: m('cashbook-lent-screen-title') });
    return { ...app, user: userEvent.setup() };
  }

  beforeEach(() => {
    books = prepareCashbook(language, [KHATA_ROW, ASLAM_ROW]);
  });

  describe('a new khata', () => {
    async function openAddForm(user: ReturnType<typeof userEvent.setup>) {
      await user.click(await dropdown(language, 'cashbook-field-khata'));
      await user.keyboard('Garden');
      await screen.findByText(m('search-no-match'));
      press('F4');
      await screen.findByRole('heading', { name: named(m('cashbook-add-khata-title')) });
      const urdu = await labelled(language, 'cashbook-add-name-ur');
      await user.click(urdu);
      await user.keyboard('باغ');
    }

    it('starts with no link, and sends the shop when the shop is chosen', async () => {
      const { user } = await openKhata();
      await openAddForm(user);
      // The form shows the choice and starts with nobody: the khata is the firm's, as before.
      expect(await dropdown(language, 'cashbook-link-label')).toHaveTextContent(
        m('cashbook-link-kind-none'),
      );
      await choose(user, language, 'cashbook-link-label', m('cashbook-link-kind-shop'));
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.khataCreate).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(cashbookApi.khataCreate).mock.calls[0]?.[0]).toMatchObject({
        name_en: 'Garden',
        link: { kind: 'shop', partner_id: null },
      });
    });

    it('sends the partner of a partner link, and a refusal of the backend shows at its field', async () => {
      const { user } = await openKhata();
      await openAddForm(user);
      await choose(user, language, 'cashbook-link-label', m('cashbook-link-kind-partner'));
      vi.mocked(cashbookApi.khataCreate).mockRejectedValueOnce(
        fail({ code: 'ERR_LINK_PARTNER_REQUIRED', field: 'link.partner_id' }),
      );
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.khataCreate).toHaveBeenCalledTimes(1);
      });
      // No partner was chosen, so none is sent; the backend's words show under the partner field.
      expect(vi.mocked(cashbookApi.khataCreate).mock.calls[0]?.[0]).toMatchObject({
        link: { kind: 'partner', partner_id: null },
      });
      expect(await screen.findByText(said(m('err-link-partner-required')))).toBeInTheDocument();
      await choose(user, language, 'cashbook-link-partner', partnerB);
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.khataCreate).toHaveBeenCalledTimes(2);
      });
      expect(vi.mocked(cashbookApi.khataCreate).mock.calls[1]?.[0]).toMatchObject({
        link: { kind: 'partner', partner_id: 2 },
      });
    });
  });

  describe('the link of a khata that exists', () => {
    it('is shown in words, and changed to one partner while the balance is 0', async () => {
      const { user } = await openKhata();
      vi.mocked(cashbookApi.khataSetLink).mockImplementation((input) =>
        Promise.resolve({
          account: books.store.khatas[0]?.account ?? { id: 0, code: '', name_ur: '', name_en: '' },
          note: '',
          link: input.link,
          balance_paisa: paisaForTest(0),
        }),
      );
      await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
      const change = await screen.findByRole('button', {
        name: (name) => visible(name) === m('cashbook-link-change'),
      });
      expect(pageText()).toContain(m('cashbook-link-word-none'));
      expect(change).toBeEnabled();
      await user.click(change);
      await choose(user, language, 'cashbook-link-label', m('cashbook-link-kind-partner'));
      await choose(user, language, 'cashbook-link-partner', partnerB);
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.khataSetLink).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(cashbookApi.khataSetLink).mock.calls[0]?.[0]).toEqual({
        khata_account_id: 71,
        link: { kind: 'partner', partner_id: 2 },
      });
    });

    it('cannot be changed while the balance is not 0: the button is off and says why', async () => {
      const { user } = await openKhata();
      // The khata has taken money out: its balance is minus.
      vi.mocked(cashbookApi.khataStatement).mockImplementation((input) => {
        const statement = books.khataStatement(input);
        return Promise.resolve({
          ...statement,
          khata: {
            ...statement.khata,
            link: { kind: 'shop', partner_id: null },
            balance_paisa: paisaForTest(-1_000_000),
          },
        });
      });
      await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
      const change = await screen.findByRole('button', {
        name: (name) => visible(name) === m('cashbook-link-change'),
      });
      expect(change).toBeDisabled();
      expect(pageText()).toContain(m('cashbook-link-locked'));
      // The link is shown in words: the shop.
      expect(pageText()).toContain(m('cashbook-link-word-shop'));
    });

    it("shows the backend's refusal when it says the link is locked", async () => {
      const { user } = await openKhata();
      vi.mocked(cashbookApi.khataSetLink).mockRejectedValue(
        fail({ code: 'ERR_LINK_LOCKED', field: 'link' }),
      );
      await pickAccount(user, language, 'cashbook-field-khata', 'family', family);
      await user.click(
        await screen.findByRole('button', {
          name: (name) => visible(name) === m('cashbook-link-change'),
        }),
      );
      await choose(user, language, 'cashbook-link-label', m('cashbook-link-kind-shop'));
      press('F10');
      expect(await screen.findByText(said(m('err-link-locked')))).toBeInTheDocument();
    });
  });

  describe('a new person who is lent money', () => {
    it('asks his type and the link of his loans, and sends both', async () => {
      const { user } = await openLent();
      await user.click(await dropdown(language, 'cashbook-field-person'));
      await user.keyboard('Nawaz');
      await screen.findByText(m('search-no-match'));
      press('F4');
      await screen.findByRole('heading', { name: m('cashbook-add-person-title') });
      const urdu = await labelled(language, 'cashbook-add-name-ur');
      await user.click(urdu);
      await user.keyboard('نواز');
      // A person starts as "other", his loans the shop's.
      expect(await dropdown(language, 'cashbook-person-type')).toHaveTextContent(
        m('cashbook-person-type-other'),
      );
      expect(await dropdown(language, 'cashbook-link-for')).toHaveTextContent(
        m('cashbook-link-kind-shop'),
      );
      await choose(user, language, 'cashbook-person-type', m('cashbook-person-type-garden-owner'));
      await choose(user, language, 'cashbook-link-for', m('cashbook-link-kind-partner'));
      await choose(user, language, 'cashbook-link-partner', nameOf(language, 'الف', 'Partner A'));
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.lentPersonCreate).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(cashbookApi.lentPersonCreate).mock.calls[0]?.[0]).toMatchObject({
        name_en: 'Nawaz',
        person_type: 'garden_owner',
        link: { kind: 'partner', partner_id: 1 },
      });
    });
  });

  describe('the "For" field of money lent', () => {
    it('shows the shop for a person, and sends the partner chosen while he owes nothing', async () => {
      const { user } = await openLent();
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      expect(await dropdown(language, 'cashbook-link-for')).toHaveTextContent(
        m('cashbook-link-kind-shop'),
      );
      await choose(user, language, 'cashbook-link-for', m('cashbook-link-kind-partner'));
      await choose(user, language, 'cashbook-link-partner', partnerB);
      await typeAmount(user, language, '20000');
      await user.click(await labelled(language, 'cashbook-field-reason'));
      await user.keyboard('next season');
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.saveMoneyLent).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(cashbookApi.saveMoneyLent).mock.calls[0]?.[0]).toMatchObject({
        person_account_id: 61,
        amount: '20000',
        link: { kind: 'partner', partner_id: 2 },
      });
    });

    it('asks the preview of a new loan with the link typed on the form, so it shows what the save will post', async () => {
      const { user } = await openLent();
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      await choose(user, language, 'cashbook-link-for', m('cashbook-link-kind-partner'));
      await choose(user, language, 'cashbook-link-partner', partnerB);
      await typeAmount(user, language, '20000');
      await waitFor(() => {
        expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
          expect.objectContaining({ type: 'money_lent', amount: '20000' }),
          null,
          { kind: 'partner', partner_id: 2 },
        );
      });
    });

    it('sends no link with the preview of a new loan when none was chosen', async () => {
      const { user } = await openLent();
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      await typeAmount(user, language, '5000');
      await waitFor(() => {
        expect(vouchersApi.vouchersPreview).toHaveBeenLastCalledWith(
          expect.objectContaining({ type: 'money_lent', amount: '5000' }),
          null,
          null,
        );
      });
    });

    it('shows what the save will do to each partner, as the backend worked it out', async () => {
      const ask = vi.mocked(vouchersApi.vouchersPreview).getMockImplementation();
      if (ask === undefined) throw new Error('the preview is not faked');
      vi.mocked(vouchersApi.vouchersPreview).mockImplementation(async (entry, replacing, link) => ({
        ...(await ask(entry, replacing, link)),
        linked_parts: [
          { partner_id: 1, amount_paisa: paisaForTest(-800_000) },
          { partner_id: 2, amount_paisa: paisaForTest(-600_000) },
        ],
      }));
      const { user } = await openLent();
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      await typeAmount(user, language, '20000');
      const list = await screen.findByRole('list', {
        name: said(m('cashbook-linked-parts-title')),
      });
      const text = visible(list.textContent);
      expect(text).toContain(nameOf(language, 'الف', 'Partner A'));
      expect(text).toContain(partnerB);
      expect(text).toContain('-8,000');
      expect(text).toContain('-6,000');
    });

    it('sends no link when the user did not choose one, so the backend keeps his own', async () => {
      const { user } = await openLent();
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      await typeAmount(user, language, '5000');
      await user.click(await labelled(language, 'cashbook-field-reason'));
      await user.keyboard('home');
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.saveMoneyLent).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(cashbookApi.saveMoneyLent).mock.calls[0]?.[0]).toMatchObject({
        link: null,
      });
    });

    it('is fixed while he owes: the field cannot be changed and no link is sent', async () => {
      const { user } = await openLent();
      // He owes 5,000 already, linked to partner A.
      vi.mocked(cashbookApi.lentList).mockImplementation((input) => {
        const list = books.lentList(input);
        return Promise.resolve({
          ...list,
          person: { ...list.person, link: { kind: 'partner', partner_id: 1 } },
          balance: { amount_paisa: paisaForTest(500_000), side: 'dr' },
        });
      });
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      const field = await dropdown(language, 'cashbook-link-for');
      await waitFor(() => {
        expect(field).toBeDisabled();
      });
      expect(pageText()).toContain(m('cashbook-link-locked'));
      await typeAmount(user, language, '1000');
      await user.click(await labelled(language, 'cashbook-field-reason'));
      await user.keyboard('more');
      press('F10');
      await waitFor(() => {
        expect(cashbookApi.saveMoneyLent).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(cashbookApi.saveMoneyLent).mock.calls[0]?.[0]).toMatchObject({ link: null });
    });

    it('shows a refusal of the preview at the link field: a different link while he owes', async () => {
      const ask = vi.mocked(vouchersApi.vouchersPreview).getMockImplementation();
      if (ask === undefined) throw new Error('the preview is not faked');
      vi.mocked(vouchersApi.vouchersPreview).mockImplementation(async (entry, replacing, link) => ({
        ...(await ask(entry, replacing, link)),
        errors: [{ code: 'ERR_LINK_FIXED_WHILE_OWED', field: 'link', values: {} }],
      }));
      const { user } = await openLent();
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      await choose(user, language, 'cashbook-link-for', m('cashbook-link-kind-partner'));
      await choose(user, language, 'cashbook-link-partner', partnerB);
      await typeAmount(user, language, '20000');
      expect(await screen.findByText(said(m('err-link-fixed-while-owed')))).toBeInTheDocument();
    });

    it('shows the backend refusal of a salary return for a partner-linked person at its field', async () => {
      const { user } = await openLent('money_lent_return');
      vi.mocked(cashbookApi.saveMoneyLentReturn).mockRejectedValue(
        fail({ code: 'ERR_SALARY_NOT_ALLOWED_FOR_LINKED', field: 'mode' }),
      );
      await pickAccount(user, language, 'cashbook-field-person', 'aslam', aslam);
      await typeAmount(user, language, '1000');
      await choose(user, language, 'cashbook-field-mode', m('cashbook-mode-salary'));
      press('F10');
      expect(
        await screen.findByText(said(m('err-salary-not-allowed-for-linked'))),
      ).toBeInTheDocument();
    });
  });
});
