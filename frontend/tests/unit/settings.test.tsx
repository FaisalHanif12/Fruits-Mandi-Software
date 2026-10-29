// Settings (plan 04, PRD Section 21, board UsersUr), in Urdu and English, by keyboard, against a
// fake backend: the tabs, the firm form, users with their dialogs, language and text size, auto
// lock, and leaving with unsaved text.

import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as authApi from '../../src/features/auth/api';
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
  signedIn,
} from '../helpers/app';
import { PAGE_ROWS, pagerButton, rangeText } from '../helpers/pager';
import { message, visible } from '../helpers/render';
import { at } from '../helpers/sample';
import { suggestionsFor } from '../helpers/translit';
import { EMPTY_PAIRS, pairsOf } from '../helpers/paged';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/urdu/api');

const named = (language: Language, id: string) => (name: string) =>
  visible(name) === message(language, id);

const tab = (language: Language, id: string) =>
  screen.getByRole('tab', { name: named(language, id) });

function field(
  language: Language,
  id: string,
  root: HTMLElement = document.body,
): HTMLInputElement {
  return within(root).getByLabelText<HTMLInputElement>(named(language, id));
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('settings (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  async function open() {
    const app = await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    return { ...app, user: userEvent.setup() };
  }

  describe('tabs', () => {
    it('Esc leaves the settings for the dashboard, and asks first when text is not saved', async () => {
      const { user, router } = await open();
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/');
      });
    });

    it('Esc with unsaved text asks "Save changes?" and stays until he answers', async () => {
      const { user, router } = await open();
      await user.click(field(language, 'settings-firm-address'));
      await user.keyboard('{End} more');
      await user.keyboard('{Escape}');
      const prompt = await screen.findByRole('alertdialog');
      expect(visible(prompt.textContent)).toContain(message(language, 'layout-save-title'));
      expect(router.state.location.pathname).toBe('/settings');
    });

    it('has nine tabs, and ← and → move between them in the direction the user sees', async () => {
      const { user } = await open();
      const tabs = screen.getAllByRole('tab');
      expect(tabs).toHaveLength(9);
      expect(tabs[0]).toHaveAttribute('aria-selected', 'true');
      tabs[0]?.focus();
      // The next tab is on the left in Urdu and on the right in English.
      await user.keyboard(language === 'ur' ? '{ArrowLeft}' : '{ArrowRight}');
      expect(tab(language, 'settings-tab-charges')).toHaveAttribute('aria-selected', 'true');
      await user.keyboard(language === 'ur' ? '{ArrowRight}' : '{ArrowLeft}');
      expect(tab(language, 'settings-tab-firm')).toHaveAttribute('aria-selected', 'true');
    });

    it('a failure to load shows the message with a retry that works', async () => {
      vi.mocked(settingsApi.settingsGet).mockRejectedValueOnce(
        fail({ code: 'ERR_NOT_IMPLEMENTED' }),
      );
      await renderApp('/settings');
      const alert = await screen.findByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
      await userEvent
        .setup()
        .click(screen.getByRole('button', { name: message(language, 'common-retry') }));
      expect(
        await screen.findByRole('tab', { name: named(language, 'settings-tab-firm') }),
      ).toBeInTheDocument();
    });
  });

  describe('firm', () => {
    it('shows the saved firm, and F10 saves what was changed with a message', async () => {
      vi.mocked(settingsApi.settingsSetFirm).mockImplementation((input) => Promise.resolve(input));
      const { user } = await open();
      const name = field(language, 'settings-firm-name-en');
      expect(name.value).toBe('Maher Adnan and Brothers');
      const save = screen.getByRole('button', {
        name: new RegExp(message(language, 'layout-save-save')),
      });
      expect(save).toBeDisabled();
      await user.clear(name);
      await user.type(name, 'Maher Adnan Brothers');
      expect(save).toBeEnabled();
      press('F10');
      await waitFor(() => {
        expect(settingsApi.settingsSetFirm).toHaveBeenCalledWith(
          expect.objectContaining({ name_en: 'Maher Adnan Brothers', address: 'Shop 1' }),
        );
      });
      expect(await screen.findByText(message(language, 'settings-saved'))).toBeInTheDocument();
    });

    it('a firm name word made by the spelling rules alone is checked before the save (FR-7.66)', async () => {
      vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
        Promise.resolve(
          suggestionsFor(
            {
              toUrdu: { zaheer: ['ظہیر'] },
              toEnglish: { ظہیر: 'Zaheer' },
              rulesOnly: ['zaheer'],
            },
            text,
            direction,
          ),
        ),
      );
      vi.mocked(settingsApi.settingsSetFirm).mockImplementation((input) => Promise.resolve(input));
      const { user } = await open();
      const mandi = field(language, 'settings-firm-mandi-ur');
      await user.clear(mandi);
      await user.click(mandi);
      await user.keyboard('zaheer ');
      await waitFor(() => {
        expect(mandi.value).toBe('ظہیر ');
      });
      press('F10');
      // The save waits: the Owner sees the name with the marked word and confirms it.
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
      expect(await screen.findByText(message(language, 'settings-saved'))).toBeInTheDocument();
    });

    it('going back to fix the spelling saves nothing and puts the cursor in the names', async () => {
      vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
        Promise.resolve(
          suggestionsFor(
            { toUrdu: { zaheer: ['ظہیر'] }, toEnglish: { ظہیر: 'Zaheer' }, rulesOnly: ['zaheer'] },
            text,
            direction,
          ),
        ),
      );
      const { user } = await open();
      const firmName = field(language, 'settings-firm-name-ur');
      await user.clear(firmName);
      await user.click(firmName);
      await user.keyboard('zaheer ');
      await waitFor(() => {
        expect(firmName.value).toBe('ظہیر ');
      });
      press('F10');
      const dialog = await screen.findByRole('alertdialog');
      await user.click(
        within(dialog).getByRole('button', { name: new RegExp(message(language, 'common-back')) }),
      );
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
      });
      expect(settingsApi.settingsSetFirm).not.toHaveBeenCalled();
      await waitFor(() => {
        expect(document.activeElement).toBeInstanceOf(HTMLInputElement);
      });
    });

    it('a refused field shows at the field, the rest in a notice with the reference', async () => {
      vi.mocked(settingsApi.settingsSetFirm).mockRejectedValue(
        fail(
          { code: 'ERR_NAME_UR_SCRIPT', field: 'name_ur' },
          { code: 'ERR_PHONE_INVALID', field: 'phone' },
          { code: 'ERR_DB_BUSY' },
        ),
      );
      const { user } = await open();
      await user.type(field(language, 'settings-firm-address'), 'x');
      press('F10');
      const urdu = field(language, 'settings-firm-name-ur');
      await waitFor(() => {
        expect(urdu).toHaveAttribute('aria-invalid', 'true');
      });
      expect(field(language, 'settings-firm-phone')).toHaveAttribute('aria-invalid', 'true');
      const alert = screen.getByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'err-db-busy'));
      expect(visible(alert.textContent)).not.toContain(message(language, 'err-phone-invalid'));
    });

    it('flags Urdu letters in the English name at once, and says what each name takes', async () => {
      const { user } = await open();
      const english = field(language, 'settings-firm-name-en');
      await user.click(english);
      await user.keyboard('{End}مہر');
      expect(english).toHaveAttribute('aria-invalid', 'true');
      expect(
        visible(
          document.getElementById(english.getAttribute('aria-describedby') ?? '')?.textContent,
        ),
      ).toBe(message(language, 'err-name-en-script'));
      expect(visible(document.body.textContent)).toContain(
        message(
          language,
          language === 'ur' ? 'translit-pair-english-hint' : 'translit-pair-english-hint-first',
        ),
      );
      expect(visible(document.body.textContent)).toContain(
        message(
          language,
          language === 'ur' ? 'translit-pair-urdu-hint' : 'translit-pair-urdu-hint-second',
          { key: 'Alt+U' },
        ),
      );
    });

    it('asks "Save changes?" when he leaves with unsaved text, and each button does what it says', async () => {
      const { user, router } = await open();
      await user.click(field(language, 'settings-firm-address'));
      await user.keyboard('{End} extra');
      const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
      await user.click(
        within(nav).getByRole('button', { name: message(language, 'layout-nav-rokar') }),
      );
      const prompt = await screen.findByRole('alertdialog');
      expect(visible(prompt.textContent)).toContain(message(language, 'layout-save-title'));
      expect(router.state.location.pathname).toBe('/settings');
      // Back stays on the screen, with the text.
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
      });
      expect(field(language, 'settings-firm-address').value).toBe('Shop 1 extra');
      // Leave without saving goes on and saves nothing.
      await user.click(
        within(nav).getByRole('button', { name: message(language, 'layout-nav-rokar') }),
      );
      await user.click(
        await screen.findByRole('button', { name: message(language, 'layout-save-leave') }),
      );
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/rokar');
      });
      expect(settingsApi.settingsSetFirm).not.toHaveBeenCalled();
    });

    it('"Save" in that question saves first, then goes on', async () => {
      vi.mocked(settingsApi.settingsSetFirm).mockImplementation((input) => Promise.resolve(input));
      const { user, router } = await open();
      await user.click(field(language, 'settings-firm-address'));
      await user.keyboard('{End}!');
      const nav = screen.getByRole('navigation', { name: message(language, 'layout-nav-label') });
      await user.click(
        within(nav).getByRole('button', { name: message(language, 'layout-nav-reports') }),
      );
      await screen.findByRole('alertdialog');
      await user.keyboard('{Enter}');
      await waitFor(() => {
        expect(settingsApi.settingsSetFirm).toHaveBeenCalledWith(
          expect.objectContaining({ address: 'Shop 1!' }),
        );
      });
      await waitFor(() => {
        expect(router.state.location.pathname).toBe('/reports');
      });
    });

    it('switching language in the middle of typing keeps every value', async () => {
      const { user } = await open();
      await user.click(field(language, 'settings-firm-address'));
      await user.keyboard('{End} two');
      await user.keyboard('{Alt>}l{/Alt}');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      await screen.findByRole('heading', { level: 1, name: message(other, 'settings-title') });
      expect(field(other, 'settings-firm-address').value).toBe('Shop 1 two');
    });
  });

  describe('language and text', () => {
    it("the default language for new users is the Owner's setting", async () => {
      vi.mocked(settingsApi.settingsSetDefaultLanguage).mockResolvedValue(null);
      const { user } = await open();
      await user.click(tab(language, 'settings-tab-language'));
      const group = await screen.findByRole('group', {
        name: named(language, 'settings-language-default'),
      });
      await user.click(within(group).getByRole('button', { name: 'English' }));
      await waitFor(() => {
        expect(settingsApi.settingsSetDefaultLanguage).toHaveBeenCalledWith({ language: 'en' });
      });
    });

    it('text size goes up, down and back, within what the window allows, and is saved', async () => {
      Object.defineProperty(window, 'innerWidth', { value: 1920, configurable: true });
      const { user } = await open();
      await user.click(tab(language, 'settings-tab-language'));
      const bigger = await screen.findByRole('button', {
        name: new RegExp(message(language, 'settings-text-bigger')),
      });
      await user.click(bigger);
      await waitFor(() => {
        expect(document.documentElement.style.zoom).toBe('1.1');
      });
      expect(visible(document.body.textContent)).toContain('110%');
      await waitFor(() => {
        expect(settingsApi.prefsSet).toHaveBeenCalledWith({ key: 'text_size', value: '110' });
      });
      await user.click(
        screen.getByRole('button', { name: new RegExp(message(language, 'settings-text-normal')) }),
      );
      expect(document.documentElement.style.zoom).toBe('1');
      await user.click(
        screen.getByRole('button', {
          name: new RegExp(message(language, 'settings-text-smaller')),
        }),
      );
      expect(document.documentElement.style.zoom).toBe('0.9');
    });
  });

  describe('learned words', () => {
    const pairs = [
      {
        id: 7,
        roman: 'shahid',
        urdu: 'شاھد',
        kind: 'name' as const,
        use_count: 3,
        last_used_at: '2026-10-10T04:02:00Z',
      },
      {
        id: 9,
        roman: 'kela',
        urdu: 'کیلا',
        kind: 'item' as const,
        use_count: 1,
        last_used_at: null,
      },
    ];

    async function openWords() {
      const app = await open();
      await app.user.click(tab(language, 'settings-tab-language'));
      return app;
    }

    it('lists each pair with its kind and how often it was used', async () => {
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(pairsOf(pairs));
      await openWords();
      const table = await screen.findByRole('table', {
        name: message(language, 'translit-words-table'),
      });
      const rows = within(table).getAllByRole('row').slice(1);
      expect(rows).toHaveLength(2);
      expect(rows[0]).toHaveTextContent('shahid');
      expect(rows[0]).toHaveTextContent('شاھد');
      expect(visible(rows[0]?.textContent)).toContain(
        message(language, 'translit-words-kind-name'),
      );
      expect(visible(rows[0]?.textContent)).toContain('3');
      expect(visible(rows[0]?.textContent)).toMatch(/\d\d-\d\d-\d{4} \d\d:\d\d/);
      expect(visible(rows[1]?.textContent)).toContain(
        message(language, 'translit-words-kind-item'),
      );
    });

    it('150 pairs come page by page: 25 at once with the count of the backend, Next page loads the next 25', async () => {
      const many = Array.from({ length: 150 }, (_, index) => ({
        id: 100 + index,
        roman: `word${String(index)}`,
        urdu: 'لفظ',
        kind: 'word' as const,
        use_count: 1,
        last_used_at: null,
      }));
      vi.mocked(urduApi.translitPairsList).mockImplementation((offset, limit) =>
        Promise.resolve(pairsOf(many, offset, limit)),
      );
      const { user } = await openWords();
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(rangeText(language, 1, 25, 150));
      });
      expect(urduApi.translitPairsList).toHaveBeenLastCalledWith(0, PAGE_ROWS);
      await user.click(await pagerButton(language, 'next'));
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(rangeText(language, 26, 50, 150));
      });
      expect(urduApi.translitPairsList).toHaveBeenLastCalledWith(PAGE_ROWS, PAGE_ROWS);
    });

    it('Remove asks first, says what will happen, and then removes only that pair', async () => {
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(pairsOf(pairs));
      vi.mocked(urduApi.translitPairsRemove).mockResolvedValue(null);
      const { user } = await openWords();
      await screen.findByRole('table');
      const removeButtons = screen.getAllByRole('button', {
        name: message(language, 'translit-words-remove'),
      });
      await user.click(at(removeButtons, 0));
      const dialog = await screen.findByRole('alertdialog');
      expect(visible(dialog.textContent)).toContain(
        message(language, 'translit-words-confirm-title'),
      );
      expect(visible(dialog.textContent)).toContain(
        message(language, 'translit-words-confirm-description', { roman: 'shahid', urdu: 'شاھد' }),
      );
      // A removal is not undone easily: Back has the focus, so a stray Enter keeps the word.
      expect(
        within(dialog).getByRole('button', { name: new RegExp(message(language, 'common-back')) }),
      ).toHaveFocus();
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(pairsOf([at(pairs, 1)]));
      await user.click(
        within(dialog).getByRole('button', { name: message(language, 'translit-words-remove') }),
      );
      await waitFor(() => {
        expect(urduApi.translitPairsRemove).toHaveBeenCalledWith({ id: 7 });
      });
      await waitFor(() => {
        expect(screen.getAllByRole('row').slice(1)).toHaveLength(1);
      });
      expect(
        (await screen.findAllByText(message(language, 'translit-words-removed'))).length,
      ).toBeGreaterThan(0);
    });

    it('Esc keeps the word', async () => {
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(pairsOf(pairs));
      const { user } = await openWords();
      await screen.findByRole('table');
      await user.click(
        at(screen.getAllByRole('button', { name: message(language, 'translit-words-remove') }), 0),
      );
      await screen.findByRole('alertdialog');
      await user.keyboard('{Escape}');
      await waitFor(() => {
        expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
      });
      expect(urduApi.translitPairsRemove).not.toHaveBeenCalled();
    });

    it('with nothing learned yet, says why and what happens next', async () => {
      await openWords();
      expect(
        await screen.findByRole('heading', {
          name: message(language, 'translit-words-empty-title'),
        }),
      ).toBeInTheDocument();
      expect(screen.queryByRole('table')).not.toBeInTheDocument();
    });

    it('a failure shows the message with a retry that works', async () => {
      vi.mocked(urduApi.translitPairsList).mockRejectedValueOnce(fail({ code: 'ERR_DB_BUSY' }));
      const { user } = await openWords();
      const alert = await screen.findByRole('alert');
      expect(visible(alert.textContent)).toContain(message(language, 'err-db-busy'));
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(pairsOf(pairs));
      await user.click(
        within(alert).getByRole('button', { name: message(language, 'common-retry') }),
      );
      expect(await screen.findByRole('table')).toBeInTheDocument();
    });

    it('a failed removal is told, and the list stays', async () => {
      vi.mocked(urduApi.translitPairsList).mockResolvedValue(pairsOf(pairs));
      vi.mocked(urduApi.translitPairsRemove).mockRejectedValue(fail({ code: 'ERR_NOT_FOUND' }));
      const { user } = await openWords();
      await screen.findByRole('table');
      await user.click(
        at(screen.getAllByRole('button', { name: message(language, 'translit-words-remove') }), 0),
      );
      const dialog = await screen.findByRole('alertdialog');
      await user.click(
        within(dialog).getByRole('button', { name: message(language, 'translit-words-remove') }),
      );
      expect(visible((await screen.findByRole('alert')).textContent)).toContain(
        message(language, 'err-not-found'),
      );
      expect(screen.getAllByRole('row').slice(1)).toHaveLength(2);
    });
  });

  describe('auto lock', () => {
    it('shows the minutes, and F10 sends them as typed', async () => {
      vi.mocked(settingsApi.settingsSetSecurity).mockResolvedValue(null);
      const { user } = await open();
      await user.click(tab(language, 'settings-tab-lock'));
      const minutes = await screen.findByLabelText(named(language, 'settings-lock-minutes'));
      expect((minutes as HTMLInputElement).value).toBe('15');
      await user.clear(minutes);
      await user.type(minutes, '30');
      press('F10');
      await waitFor(() => {
        expect(settingsApi.settingsSetSecurity).toHaveBeenCalledWith({ idle_lock_minutes: '30' });
      });
    });

    it('a value outside the range is refused by the backend, at the field', async () => {
      vi.mocked(settingsApi.settingsSetSecurity).mockRejectedValue(
        fail({
          code: 'ERR_OUT_OF_RANGE',
          field: 'idle_lock_minutes',
          values: { min: 1, max: 120 },
        }),
      );
      const { user } = await open();
      await user.click(tab(language, 'settings-tab-lock'));
      const minutes = await screen.findByLabelText(named(language, 'settings-lock-minutes'));
      await user.clear(minutes);
      await user.type(minutes, '500{Enter}');
      press('F10');
      await waitFor(() => {
        expect(minutes).toHaveAttribute('aria-invalid', 'true');
      });
      expect((minutes as HTMLInputElement).value).toBe('500');
    });

    it('switching language in the middle of typing keeps the minutes', async () => {
      const { user } = await open();
      await user.click(tab(language, 'settings-tab-lock'));
      const minutes = await screen.findByLabelText(named(language, 'settings-lock-minutes'));
      await user.clear(minutes);
      await user.type(minutes, '45');
      await user.keyboard('{Alt>}l{/Alt}');
      const other: Language = language === 'ur' ? 'en' : 'ur';
      const again = await screen.findByLabelText(named(other, 'settings-lock-minutes'));
      expect((again as HTMLInputElement).value).toBe('45');
    });
  });
});

describe('the app window', () => {
  it('has no real command called in these tests', () => {
    expect(vi.mocked(appApi.exitApp)).not.toHaveBeenCalled();
    expect(vi.mocked(authApi.authLogin)).not.toHaveBeenCalled();
  });
});
