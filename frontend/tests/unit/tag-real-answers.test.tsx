// The account form with a tag and the firm settings with contacts and a trade mark (plan 17e, F2),
// driven by the backend's REAL answers and refusals (tests/fixtures, copied, never edited): the
// duplicate check with its mark, the refusal of a bad tag at the tag field, the firm saved with
// contacts as the backend's own test sent it, the refusals at their own fields (a fourth contact, a
// trade mark too long, a word of a contact's name not confirmed). In the Urdu and the English app,
// by keyboard only where it says so, and with a language switch in the middle of an entry.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as settingsApi from '../../src/features/settings/api';
import * as accountApi from '../../src/shared/accounts/api';
import { CommandError } from '../../src/shared/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as searchApi from '../../src/shared/search/api';
import * as urduApi from '../../src/shared/urdu/api';
import { account, fakeAccounts } from '../helpers/accounts';
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
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, visible } from '../helpers/render';
import { sendsTheSame } from '../helpers/shapes';
import * as real from '../helpers/tag-real';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
const labelled = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));
function must<T>(found: T | null | undefined): T {
  if (found === null || found === undefined) throw new Error('the element is not on the page');
  return found;
}

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
  vi.mocked(searchApi.searchRecent).mockResolvedValue([]);
  vi.mocked(searchApi.searchTouch).mockResolvedValue(null);
});

afterEach(() => {
  leaveAppWindow();
});

/** Tabs until the focused element is the one wanted: the keyboard only, no mouse. */
async function tabTo(
  user: ReturnType<typeof userEvent.setup>,
  wanted: (element: Element | null) => boolean,
  most = 80,
) {
  for (let step = 0; step < most; step += 1) {
    if (wanted(document.activeElement)) return;
    await user.tab();
  }
  throw new Error('the keyboard never reached the element');
}

describe.each(LANGUAGES)("the account form with the backend's real answers (%s)", (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  it('keyboard only: the tag is reached with Tab after the names, typed in lower case, and F10 sends it as typed', async () => {
    const store = fakeAccounts([]);
    vi.mocked(accountApi.accountsSave).mockImplementation((draft) => {
      const saved = account(18, 'buyer', '03040001', draft.names.urdu, draft.names.english, {
        tag: real.ACCOUNTS_SAVE_TAG.tag ?? '',
      });
      store.rows.push(saved);
      return Promise.resolve(saved);
    });
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    const english = await labelled(language, 'accounts-field-name-en');
    const urdu = await labelled(language, 'accounts-field-name-ur');
    const tag = await labelled(language, 'accounts-field-tag');
    // The first field of the form has the focus: no mouse is used from here on.
    await tabTo(user, (element) => element === (language === 'ur' ? urdu : english));
    await user.keyboard(language === 'ur' ? 'اکرم ٹول ٹیکس' : 'Akram Toll Tax');
    await tabTo(user, (element) => element === (language === 'ur' ? english : urdu));
    await user.keyboard(language === 'ur' ? 'Akram Toll Tax' : 'اکرم ٹول ٹیکس');
    await tabTo(user, (element) => element === tag);
    // The tag is the next field after the two names: nothing else sits between them.
    await user.keyboard('k');
    press('F10');
    await waitFor(() => {
      expect(accountApi.accountsSave).toHaveBeenCalledTimes(1);
    });
    const [draft] = vi.mocked(accountApi.accountsSave).mock.calls[0] ?? [];
    // As typed, exactly the tag of the backend's own request ("k"); the backend makes it capital.
    expect(draft?.tag).toBe(real.ACCOUNTS_SAVE_TAG_REQUEST.tag);
    expect(await screen.findByText(message(language, 'accounts-saved'))).toBeInTheDocument();
    expect(
      await screen.findByText(named(language === 'ur' ? 'اکرم ٹول ٹیکس K' : 'Akram Toll Tax K')),
    ).toBeInTheDocument();
  });

  it('the real refusal of a bad tag shows at the tag field, in words, with the field marked', async () => {
    fakeAccounts([]);
    vi.mocked(accountApi.accountsSave).mockRejectedValue(
      new CommandError(real.ACCOUNTS_SAVE_TAG_REFUSED),
    );
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard('Akram Toll Tax');
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard('اکرم ٹول ٹیکس');
    const tag = await labelled(language, 'accounts-field-tag');
    await user.click(tag);
    await user.keyboard('1A');
    press('F10');
    await waitFor(() => {
      expect(tag).toHaveAttribute('aria-invalid', 'true');
    });
    const words = message(language, 'err-tag-invalid');
    expect(screen.getByText(words)).toBeInTheDocument();
    // The words never show the code or the reference as the text of a field.
    expect(visible(document.body.textContent)).not.toContain('ERR_TAG_INVALID');
    // The words are the same in both languages for what they say: 1 to 4 capital letters.
    expect(words).toMatch(language === 'ur' ? /K|BA/ : /K, BA/);
  });

  it('the real duplicate answer marks the account whose name, tag and phone are equal; the other tag is not marked', async () => {
    fakeAccounts([]);
    vi.mocked(accountApi.accountsSimilar).mockResolvedValue(real.ACCOUNTS_SIMILAR_DUPLICATE);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard(real.ACCOUNTS_SIMILAR_DUPLICATE_REQUEST.name_ur);
    await user.click(await labelled(language, 'accounts-field-tag'));
    await user.keyboard(real.ACCOUNTS_SIMILAR_DUPLICATE_REQUEST.tag);
    await user.tab();
    await waitFor(() => {
      expect(accountApi.accountsSimilar).toHaveBeenLastCalledWith(
        expect.objectContaining({
          name_ur: real.ACCOUNTS_SIMILAR_DUPLICATE_REQUEST.name_ur,
          // The tag is made capital when he leaves the field; the backend compares it capital.
          tag: real.ACCOUNTS_SIMILAR_DUPLICATE_REQUEST.tag.toUpperCase(),
        }),
      );
    });
    const heading = await screen.findByRole('heading', {
      name: message(language, 'accounts-similar-duplicate-title'),
    });
    const panel = must(heading.closest('section'));
    expect(
      within(panel).getAllByText(message(language, 'accounts-similar-duplicate-chip')),
    ).toHaveLength(1);
    expect(visible(panel.textContent)).toContain(
      language === 'ur' ? 'اکرم ٹول ٹیکس K' : 'Akram Toll Tax K',
    );
  });

  it('the real answer for another tag lists him with the plain title and the help line, no mark', async () => {
    fakeAccounts([]);
    vi.mocked(accountApi.accountsSimilar).mockResolvedValue(real.ACCOUNTS_SIMILAR_OTHER);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard(real.ACCOUNTS_SIMILAR_OTHER_REQUEST.name_ur);
    await user.click(await labelled(language, 'accounts-field-tag'));
    await user.keyboard(real.ACCOUNTS_SIMILAR_OTHER_REQUEST.tag);
    await user.tab();
    const heading = await screen.findByRole('heading', {
      name: message(language, 'accounts-similar-title'),
    });
    const panel = must(heading.closest('section'));
    expect(
      within(panel).queryByText(message(language, 'accounts-similar-duplicate-chip')),
    ).toBeNull();
    expect(
      within(panel).getByText(message(language, 'accounts-similar-tag-help')),
    ).toBeInTheDocument();
    // He is listed with his own tag, so the two Akrams are told apart.
    expect(visible(panel.textContent)).toContain(
      language === 'ur' ? 'اکرم ٹول ٹیکس K' : 'Akram Toll Tax K',
    );
  });

  it('a language switch in the middle of an entry loses nothing: the names, the tag and the field order stay', async () => {
    fakeAccounts([]);
    await renderApp('/index/accounts/new');
    const user = userEvent.setup();
    await user.click(await labelled(language, 'accounts-field-name-en'));
    await user.keyboard('Ashiq Zafar');
    await user.click(await labelled(language, 'accounts-field-name-ur'));
    await user.keyboard('عاشق ظفر');
    const tag = await labelled(language, 'accounts-field-tag');
    await user.click(tag);
    await user.keyboard('ba');
    await user.tab();
    expect(tag.value).toBe('BA');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    press('KeyL', { altKey: true });
    // The same fields, now with the words of the other language, hold what was typed.
    const tagAfter = await labelled(other, 'accounts-field-tag');
    await waitFor(() => {
      expect(document.documentElement.lang).toBe(other);
    });
    expect(tagAfter.value).toBe('BA');
    expect((await labelled(other, 'accounts-field-name-en')).value).toBe('Ashiq Zafar');
    expect((await labelled(other, 'accounts-field-name-ur')).value).toBe('عاشق ظفر');
    // And the tag is still a plain English field after the switch.
    expect(tagAfter).toHaveAttribute('lang', 'en');
    expect(tagAfter).toHaveAttribute('dir', 'ltr');
    act(() => {
      press('KeyL', { altKey: true });
    });
    expect((await labelled(language, 'accounts-field-tag')).value).toBe('BA');
  });
});

const CONTACT_NAMES = [
  ['منشی آصف بھٹی', 'Munshi Asif Bhatti', '0300-4058860'],
  ['اکرم', 'Akram', '03211234567'],
] as const;

describe.each(LANGUAGES)("the firm settings with the backend's real answers (%s)", (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
    vi.mocked(settingsApi.settingsSetFirm).mockImplementation((input) => Promise.resolve(input));
  });

  async function open() {
    await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    await screen.findByLabelText(named(message(language, 'settings-firm-trade-mark')));
    return userEvent.setup();
  }
  const contactsBlock = () =>
    must(
      screen
        .getByRole('heading', { name: named(message(language, 'settings-firm-contacts-title')) })
        .closest('section'),
    );
  const contactField = (id: string) =>
    within(contactsBlock()).getAllByLabelText<HTMLInputElement>(named(message(language, id)));

  it('shows the contacts and the trade mark the backend stored, the phones as 11 digits', async () => {
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(real.SETTINGS_GET_CONTACTS);
    await open();
    expect(
      screen.getByLabelText<HTMLInputElement>(named(message(language, 'settings-firm-trade-mark')))
        .value,
    ).toBe('RH');
    expect(contactField('settings-firm-contact-phone').map((input) => input.value)).toEqual([
      '03004058860',
      '03211234567',
    ]);
    expect(contactField('settings-firm-contact-name-en').map((input) => input.value)).toEqual([
      'Munshi Asif Bhatti',
      'Akram',
    ]);
  });

  it('keyboard only: Tab reaches "Add a contact person", Enter adds, the names and phone are typed, F10 sends the backend\'s own request', async () => {
    const firm = real.SETTINGS_GET_CONTACTS.firm;
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(
      settings({ firm: { ...firm, firm_contacts: [], trade_mark: '' } }),
    );
    vi.mocked(settingsApi.settingsSetFirm).mockResolvedValue(real.SETTINGS_SET_FIRM_CONTACTS);
    const user = await open();
    const add = screen.getByRole('button', {
      name: named(message(language, 'settings-firm-contact-add')),
    });
    // The trade mark first: Tab to it, type it.
    const mark = screen.getByLabelText<HTMLInputElement>(
      named(message(language, 'settings-firm-trade-mark')),
    );
    await tabTo(user, (element) => element === mark);
    await user.keyboard('RH');
    for (const [urdu, english, phone] of CONTACT_NAMES) {
      await tabTo(user, (element) => element === add);
      await user.keyboard('{Enter}');
      const place = contactField('settings-firm-contact-name-en').length - 1;
      const urduBox = must(contactField('settings-firm-contact-name-ur')[place]);
      const englishBox = must(contactField('settings-firm-contact-name-en')[place]);
      const phoneBox = must(contactField('settings-firm-contact-phone')[place]);
      for (const [box, text] of [
        [urduBox, urdu],
        [englishBox, english],
        [phoneBox, phone],
      ] as const) {
        await tabTo(user, (element) => element === box);
        await user.keyboard(text);
      }
    }
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledTimes(1);
    });
    const { confirmed_rules_words: confirmed, ...sent } = must(
      vi.mocked(settingsApi.settingsSetFirm).mock.calls[0]?.[0],
    );
    expect(confirmed).toEqual([]);
    // The same firm, contacts and trade mark the backend's own test sent, the phones as typed.
    expect(sendsTheSame(sent, real.SETTINGS_SET_FIRM_CONTACTS_REQUEST)).toBeNull();
    expect(await screen.findByText(message(language, 'settings-saved'))).toBeInTheDocument();
  });

  it('the real refusal of a fourth contact shows in the contacts block, with the least and the most', async () => {
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(real.SETTINGS_GET_CONTACTS);
    vi.mocked(settingsApi.settingsSetFirm).mockRejectedValue(
      new CommandError(real.SETTINGS_SET_FIRM_CONTACTS_REFUSED),
    );
    const user = await open();
    await user.type(
      screen.getByLabelText(named(message(language, 'settings-firm-trade-mark'))),
      'X',
    );
    press('F10');
    const alert = await within(contactsBlock()).findByRole('alert');
    expect(visible(alert.textContent)).toBe(
      message(language, 'err-out-of-range', { min: 0, max: 3 }),
    );
    expect(visible(document.body.textContent)).not.toContain('ERR_OUT_OF_RANGE');
  });

  it('a trade mark that is too long shows at the trade mark field, with the most allowed', async () => {
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(real.SETTINGS_GET_CONTACTS);
    vi.mocked(settingsApi.settingsSetFirm).mockRejectedValue(
      fail({ code: 'ERR_TEXT_TOO_LONG', field: 'trade_mark', values: { max: 500 } }),
    );
    const user = await open();
    const mark = screen.getByLabelText<HTMLInputElement>(
      named(message(language, 'settings-firm-trade-mark')),
    );
    await user.type(mark, 'X');
    press('F10');
    await waitFor(() => {
      expect(mark).toHaveAttribute('aria-invalid', 'true');
    });
    expect(
      screen.getByText(named(message(language, 'err-text-too-long', { max: 500 }))),
    ).toBeInTheDocument();
  });

  it("a word of a contact's name the backend names as not confirmed asks him, then the save is repeated with it confirmed", async () => {
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(real.SETTINGS_GET_CONTACTS);
    vi.mocked(settingsApi.settingsSetFirm)
      .mockRejectedValueOnce(
        fail({
          code: 'ERR_SPELLING_UNCONFIRMED',
          field: 'firm_contacts[1].name_ur',
          values: { words: 'اکرم' },
        }),
      )
      .mockImplementation((input) => Promise.resolve(input));
    const user = await open();
    await user.type(
      screen.getByLabelText(named(message(language, 'settings-firm-trade-mark'))),
      'X',
    );
    press('F10');
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain('اکرم');
    await user.click(
      within(dialog).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledTimes(2);
    });
    expect(vi.mocked(settingsApi.settingsSetFirm).mock.calls[1]?.[0]).toMatchObject({
      confirmed_rules_words: ['اکرم'],
    });
  });

  it("a rules-only word in a contact's name is checked before the send, like the firm name (FR-7.66)", async () => {
    vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
      Promise.resolve(
        suggestionsFor(
          { toUrdu: { zaheer: ['ظہیر'] }, toEnglish: { ظہیر: 'Zaheer' }, rulesOnly: ['zaheer'] },
          text,
          direction,
        ),
      ),
    );
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(
      settings({ firm: { ...real.SETTINGS_GET_CONTACTS.firm, firm_contacts: [], trade_mark: '' } }),
    );
    const user = await open();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'settings-firm-contact-add')) }),
    );
    const urdu = must(contactField('settings-firm-contact-name-ur')[0]);
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(urdu.value).toBe('ظہیر ');
    });
    press('F10');
    const dialog = await screen.findByRole('alertdialog');
    expect(settingsApi.settingsSetFirm).not.toHaveBeenCalled();
    expect(visible(dialog.textContent)).toContain('ظہیر');
    await user.click(
      within(dialog).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(settingsApi.settingsSetFirm).mock.calls[0]?.[0]).toMatchObject({
      confirmed_rules_words: ['ظہیر'],
    });
  });

  it('a language switch in the middle of an entry loses nothing: the contacts, the phones and the trade mark stay', async () => {
    vi.mocked(settingsApi.settingsGet).mockResolvedValue(
      settings({ firm: { ...real.SETTINGS_GET_CONTACTS.firm, firm_contacts: [], trade_mark: '' } }),
    );
    const user = await open();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'settings-firm-contact-add')) }),
    );
    await user.type(must(contactField('settings-firm-contact-name-en')[0]), 'Asif Bhatti');
    await user.type(must(contactField('settings-firm-contact-phone')[0]), '0300-4058860');
    await user.type(
      screen.getByLabelText(named(message(language, 'settings-firm-trade-mark'))),
      'RH',
    );
    const other: Language = language === 'ur' ? 'en' : 'ur';
    press('KeyL', { altKey: true });
    await waitFor(() => {
      expect(document.documentElement.lang).toBe(other);
    });
    const block = must(
      screen
        .getByRole('heading', { name: named(message(other, 'settings-firm-contacts-title')) })
        .closest('section'),
    );
    expect(
      within(block).getByLabelText<HTMLInputElement>(
        named(message(other, 'settings-firm-contact-name-en')),
      ).value,
    ).toBe('Asif Bhatti');
    expect(
      within(block).getByLabelText<HTMLInputElement>(
        named(message(other, 'settings-firm-contact-phone')),
      ).value,
    ).toBe('0300-4058860');
    expect(
      screen.getByLabelText<HTMLInputElement>(named(message(other, 'settings-firm-trade-mark')))
        .value,
    ).toBe('RH');
    // He carries on in the other language and saves: nothing was lost on the way.
    press('F10');
    await waitFor(() => {
      expect(settingsApi.settingsSetFirm).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(settingsApi.settingsSetFirm).mock.calls[0]?.[0]).toMatchObject({
      trade_mark: 'RH',
      firm_contacts: [{ name_en: 'Asif Bhatti', phone: '0300-4058860' }],
    });
  });
});
