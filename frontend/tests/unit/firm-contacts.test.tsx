// The firm settings of plan 17e (PRD Section 21, FR-19.13): up to three contact persons and the
// trade mark, printed under the firm name on every print. In Urdu and in English, by keyboard,
// against a fake backend: adding, removing and the limit of three, an empty row is not a contact,
// what is sent on F10 (text as typed), and a refused field shows at its own field.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as settingsApi from '../../src/features/settings/api';
import * as urduApi from '../../src/shared/urdu/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
  settings,
  signedIn,
} from '../helpers/app';
import { message, visible } from '../helpers/render';
import { EMPTY_PAIRS } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/urdu/api');

function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}

const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

const CONTACTS = [
  { name_ur: 'منشی آصف بھٹی', name_en: 'Munshi Asif Bhatti', phone: '0300-4058860' },
  { name_ur: 'ماہر عدنان', name_en: 'Maher Adnan', phone: '0300-1112233' },
  { name_ur: 'محمد اکرم', name_en: 'Muhammad Akram', phone: '0321-5556677' },
];

function withFirm(extra: { firm_contacts?: typeof CONTACTS; trade_mark?: string }) {
  const base = settings();
  vi.mocked(settingsApi.settingsGet).mockResolvedValue({
    ...base,
    firm: { ...base.firm, firm_contacts: [], trade_mark: '', ...extra },
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(settingsApi.settingsSetFirm).mockImplementation((input) => Promise.resolve(input));
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the contacts and the trade mark of the firm (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const app = await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    await screen.findByLabelText(named(language, 'settings-firm-trade-mark'));
    return { ...app, user: userEvent.setup() };
  }

  /** The block of the contact persons, so the firm's own phone is not mixed with theirs. */
  const contacts = (): HTMLElement => {
    const block = screen
      .getByRole('heading', { name: named(language, 'settings-firm-contacts-title') })
      .closest('section');
    if (block === null) throw new Error('no contacts block');
    return block;
  };
  const phones = () =>
    within(contacts()).queryAllByLabelText<HTMLInputElement>(
      named(language, 'settings-firm-contact-phone'),
    );
  const saveButton = () =>
    screen.getByRole('button', { name: new RegExp(message(language, 'layout-save-save')) });
  const addButton = () =>
    screen.getByRole('button', { name: named(language, 'settings-firm-contact-add') });

  it('a firm with no contact says so, and has an empty trade mark', async () => {
    withFirm({});
    await open();
    expect(screen.getByText(message(language, 'settings-firm-contacts-empty'))).toBeInTheDocument();
    expect(
      screen.getByLabelText<HTMLInputElement>(named(language, 'settings-firm-trade-mark')).value,
    ).toBe('');
    expect(saveButton()).toBeDisabled();
  });

  it('shows the saved contacts and the trade mark', async () => {
    withFirm({ firm_contacts: CONTACTS.slice(0, 2), trade_mark: 'RH' });
    await open();
    expect(
      screen.getByLabelText<HTMLInputElement>(named(language, 'settings-firm-trade-mark')).value,
    ).toBe('RH');
    expect(phones().map((input) => input.value)).toEqual(['0300-4058860', '0300-1112233']);
    expect(
      screen.queryByText(message(language, 'settings-firm-contacts-empty')),
    ).not.toBeInTheDocument();
  });

  it('adds a contact and the trade mark, and F10 sends them as typed', async () => {
    withFirm({});
    const { user } = await open();
    await user.click(addButton());
    await user.type(
      await within(contacts()).findByLabelText(named(language, 'settings-firm-contact-name-en')),
      'Asif Bhatti',
    );
    await user.type(must(phones()[0]), '0300-4058860');
    await user.type(screen.getByLabelText(named(language, 'settings-firm-trade-mark')), 'RH');
    expect(saveButton()).toBeEnabled();
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(settingsApi.settingsSetFirm).mock.calls[0]?.[0]).toMatchObject({
      trade_mark: 'RH',
      firm_contacts: [{ name_en: 'Asif Bhatti', phone: '0300-4058860' }],
    });
  });

  it('three contacts are the most: the add button is off and the words say why', async () => {
    withFirm({ firm_contacts: CONTACTS });
    await open();
    expect(addButton()).toBeDisabled();
    expect(screen.getByText(message(language, 'settings-firm-contacts-full'))).toBeInTheDocument();
  });

  it('removing a contact takes only that one out of the save', async () => {
    withFirm({ firm_contacts: CONTACTS });
    const { user } = await open();
    await user.click(
      screen.getByRole('button', {
        name: (name) =>
          visible(name) ===
          message(language, 'settings-firm-contact-remove-label', { number: '2' }),
      }),
    );
    expect(addButton()).toBeEnabled();
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledTimes(1);
    });
    const sent = vi.mocked(settingsApi.settingsSetFirm).mock.calls[0]?.[0];
    expect(sent?.firm_contacts.map((contact) => contact.phone)).toEqual([
      '0300-4058860',
      '0321-5556677',
    ]);
  });

  it('a row left empty is not a contact: nothing changed, nothing to save', async () => {
    withFirm({});
    const { user } = await open();
    await user.click(addButton());
    await within(contacts()).findByLabelText(named(language, 'settings-firm-contact-phone'));
    expect(saveButton()).toBeDisabled();
  });

  it('a refused phone shows at the phone of that contact', async () => {
    withFirm({ firm_contacts: CONTACTS.slice(0, 2) });
    vi.mocked(settingsApi.settingsSetFirm).mockRejectedValue(
      fail({ code: 'ERR_PHONE_INVALID', field: 'firm_contacts[1].phone' }),
    );
    const { user } = await open();
    const [first, second] = phones();
    await user.type(must(second), '9');
    press('F10');
    await waitFor(() => {
      expect(second).toHaveAttribute('aria-invalid', 'true');
    });
    expect(first).not.toHaveAttribute('aria-invalid', 'true');
  });
});
