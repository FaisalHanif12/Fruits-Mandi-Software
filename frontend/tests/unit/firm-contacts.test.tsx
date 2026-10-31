// The firm settings of plan 17e and plan 42 (PRD Section 21, FR-19.13, v1.12): up to six people, each
// with a role (owner or munshi) and an optional phone, and the trade mark, printed under the firm name
// on every print. In Urdu and in English, against a fake backend: adding, removing and the limit of
// six, a new person starts as an owner and the role can be changed, a phone can be left empty, an
// empty row is not a person, what is sent on F10 (text as typed), and a refused field shows at its
// own field.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { FirmContact } from '../../src/bindings/FirmContact';
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

const CONTACTS: FirmContact[] = [
  { name_ur: 'ماہر عدنان', name_en: 'Maher Adnan', role: 'owner', phone: '0300-1112233' },
  { name_ur: 'محمد اکرم', name_en: 'Muhammad Akram', role: 'owner', phone: '0321-5556677' },
  { name_ur: 'ابراہیم', name_en: 'Ibrahim', role: 'owner', phone: '0322-1110000' },
  { name_ur: 'عمران', name_en: 'Imran', role: 'owner', phone: '' },
  { name_ur: 'سلمان', name_en: 'Salman', role: 'owner', phone: '0333-4445555' },
  { name_ur: 'آصف بھٹی', name_en: 'Asif Bhatti', role: 'munshi', phone: '0300-4058860' },
];

function withFirm(extra: { firm_contacts?: FirmContact[]; trade_mark?: string }) {
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
  const roles = () =>
    within(contacts()).queryAllByRole('combobox', {
      name: named(language, 'settings-firm-contact-role'),
    });
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
    withFirm({
      firm_contacts: [CONTACTS[0], CONTACTS[5]].filter(Boolean) as FirmContact[],
      trade_mark: 'RH',
    });
    await open();
    expect(
      screen.getByLabelText<HTMLInputElement>(named(language, 'settings-firm-trade-mark')).value,
    ).toBe('RH');
    expect(phones().map((input) => input.value)).toEqual(['0300-1112233', '0300-4058860']);
    // Each person shows his role: the owner first, then the munshi.
    expect(roles().map((role) => visible(role.textContent))).toEqual([
      message(language, 'settings-firm-role-owner'),
      message(language, 'settings-firm-role-munshi'),
    ]);
    expect(
      screen.queryByText(message(language, 'settings-firm-contacts-empty')),
    ).not.toBeInTheDocument();
  });

  it('adds a person (an owner at first) and the trade mark, and F10 sends them as typed', async () => {
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
      firm_contacts: [{ name_en: 'Asif Bhatti', role: 'owner', phone: '0300-4058860' }],
    });
  });

  it('the role can be changed to munshi, and a person can have no phone', async () => {
    withFirm({});
    const { user } = await open();
    await user.click(addButton());
    await user.type(
      await within(contacts()).findByLabelText(named(language, 'settings-firm-contact-name-en')),
      'Asif Bhatti',
    );
    const [role] = roles();
    await user.click(must(role));
    await user.click(
      await screen.findByRole('option', { name: named(language, 'settings-firm-role-munshi') }),
    );
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(settingsApi.settingsSetFirm).mock.calls[0]?.[0].firm_contacts).toEqual([
      expect.objectContaining({ name_en: 'Asif Bhatti', role: 'munshi', phone: '' }),
    ]);
  });

  it('six people are the most: the add button is off and the words say why', async () => {
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
    expect(sent?.firm_contacts.map((contact) => contact.name_en)).toEqual([
      'Maher Adnan',
      'Ibrahim',
      'Imran',
      'Salman',
      'Asif Bhatti',
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
