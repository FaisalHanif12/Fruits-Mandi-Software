// The first-run wizard (plan 04, PRD NFR-INS-2, boards WizardUr and WizardEn), in Urdu and
// English, by keyboard only, against a fake backend.

import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as authApi from '../../src/features/auth/api';
import * as partnersApi from '../../src/features/partners/api';
import * as settingsApi from '../../src/features/settings/api';
import * as urduApi from '../../src/shared/urdu/api';
import * as appApi from '../../src/app/api';
import { useWizardStore } from '../../src/features/auth/wizard-store';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  authState,
  fail,
  healthyBackend,
  leaveAppWindow,
  press,
  renderApp,
  resetClientState,
  sessionUser,
} from '../helpers/app';
import { fakeMasters } from '../helpers/masters';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const FIRM_NAMES = { en: 'Maher Adnan and Brothers', ur: 'مہر عدنان اینڈ برادرز' };
const MANDI_NAMES = { en: 'New Sabzi Mandi', ur: 'نئی سبزی منڈی' };

/**
 * The firm step typed in the order of its fields: the Urdu name comes first in the Urdu app and
 * the English name first in the English app (plan 05). Enter after each field moves on.
 */
function firmStep(language: Language): string {
  const inOrder = (names: { en: string; ur: string }) =>
    language === 'ur' ? [names.ur, names.en] : [names.en, names.ur];
  return [...inOrder(FIRM_NAMES), ...inOrder(MANDI_NAMES), '0300-0000000', 'Shop 1']
    .map((text) => `${text}{Enter}`)
    .join('');
}

const printSpy = vi.fn();
const OWNER_STEP = 'Adnan{Enter}owner{Enter}secret12{Enter}secret12{Enter}';

function field(language: Language, id: string): HTMLInputElement {
  const label = message(language, id);
  return screen.getByLabelText<HTMLInputElement>((text) => visible(text) === label);
}

beforeEach(() => {
  vi.clearAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  fakeAccounts([]);
  fakeAccountSearch([]);
  fakeMasters({});
  window.print = printSpy;
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('first-run wizard (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ first_run: true, login_language: language }),
    );
  });

  async function start() {
    await renderApp();
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-firm-title') });
    return userEvent.setup();
  }

  it('shows four steps, the first current, and the first field has focus', async () => {
    await start();
    const steps = screen.getByRole('list', { name: message(language, 'auth-wizard-steps') });
    const items = steps.querySelectorAll('li');
    expect(items).toHaveLength(4);
    expect(items[0]).toHaveAttribute('aria-current', 'step');
    // The name to type comes first: Urdu in the Urdu app, English in the English app.
    expect(
      field(language, language === 'ur' ? 'auth-wizard-firm-name-ur' : 'auth-wizard-firm-name-en'),
    ).toHaveFocus();
    expect(document.documentElement.dir).toBe(language === 'ur' ? 'rtl' : 'ltr');
  });

  it('by keyboard only: firm, Owner, then the recovery code, printed and finished (no sign-in after the setup: it ends at the sign-in screen)', async () => {
    vi.mocked(authApi.setupFirstRun).mockResolvedValue({ recovery_code: '7KQ2M9XD4TPAHW3C' });
    // The sign-in right after the setup fails, so there is no partners step (plan 11).
    vi.mocked(authApi.authLogin).mockRejectedValue(fail({ code: 'ERR_INTERNAL' }));
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    expect(field(language, 'auth-wizard-owner-name')).toHaveFocus();
    await user.keyboard(OWNER_STEP);
    await waitFor(() => {
      expect(authApi.setupFirstRun).toHaveBeenCalledWith({
        firm_name_ur: 'مہر عدنان اینڈ برادرز',
        firm_name_en: 'Maher Adnan and Brothers',
        mandi_ur: 'نئی سبزی منڈی',
        mandi_en: 'New Sabzi Mandi',
        address: 'Shop 1',
        phone: '0300-0000000',
        owner_username: 'owner',
        owner_display_name: 'Adnan',
        password: 'secret12',
        password_repeat: 'secret12',
        confirmed_rules_words: [],
      });
    });
    expect(authApi.authLogin).toHaveBeenCalledWith({ username: 'owner', password: 'secret12' });
    // The code is shown once, in groups of four, left to right. The passwords are gone, also
    // when the sign-in failed (plan 11, the lead's condition 1).
    expect(await screen.findByLabelText('7KQ2M9XD4TPAHW3C')).toHaveTextContent(
      '7KQ2-M9XD-4TPA-HW3C',
    );
    expect(useWizardStore.getState().values.password).toBe('');
    expect(useWizardStore.getState().values.password_repeat).toBe('');
    const finish = screen.getByRole('button', {
      name: new RegExp(message(language, 'auth-wizard-finish')),
    });
    expect(finish).toBeDisabled();
    // F9 prints.
    act(() => {
      fireEvent.keyDown(window, { code: 'F9' });
    });
    expect(printSpy).toHaveBeenCalledTimes(1);
    await user.click(screen.getByRole('checkbox'));
    expect(finish).toBeEnabled();
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ first_run: false, login_language: language }),
    );
    await user.click(finish);
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
    expect(useWizardStore.getState().recoveryCode).toBeNull();
    // No partners step when the sign-in failed: nothing asked for the partners, and the
    // wizard ended at the sign-in screen (the partners are set up later at /partners/setup).
    expect(partnersApi.partnersList).not.toHaveBeenCalled();
    expect(useWizardStore.getState().signedIn).toBe(false);
  });

  it('Esc and Back return to the firm step with everything still typed', async () => {
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard('Adnan');
    await user.keyboard('{Escape}');
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-firm-title') });
    expect(field(language, 'auth-wizard-firm-name-en').value).toBe('Maher Adnan and Brothers');
    expect(field(language, 'auth-wizard-address').value).toBe('Shop 1');
    await user.keyboard('{Enter}');
    expect(useWizardStore.getState().values.owner_display_name).toBe('Adnan');
  });

  it('a refused field of the firm step takes him back to it, with the message at the field', async () => {
    vi.mocked(authApi.setupFirstRun).mockRejectedValue(
      fail({ code: 'ERR_NAME_UR_SCRIPT', field: 'firm_name_ur' }),
    );
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-firm-title') });
    const urdu = field(language, 'auth-wizard-firm-name-ur');
    expect(urdu).toHaveAttribute('aria-invalid', 'true');
    const note = document.getElementById(urdu.getAttribute('aria-describedby') ?? '');
    expect(visible(note?.textContent)).toBe(message(language, 'err-name-ur-script'));
  });

  it('refused fields on both steps take him back to the earliest one, and each message sits at its field', async () => {
    vi.mocked(authApi.setupFirstRun).mockRejectedValue(
      fail(
        { code: 'ERR_PHONE_INVALID', field: 'phone' },
        { code: 'ERR_USERNAME_INVALID', field: 'owner_username' },
        { code: 'ERR_TEXT_TOO_LONG', field: 'address', values: { max: 500 } },
      ),
    );
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    // The backend refused a firm field and an Owner field: he is taken back to the firm step.
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-firm-title') });
    expect(field(language, 'auth-wizard-phone')).toHaveAttribute('aria-invalid', 'true');
    expect(field(language, 'auth-wizard-address')).toHaveAttribute('aria-invalid', 'true');
    expect(field(language, 'auth-wizard-firm-name-en')).not.toHaveAttribute('aria-invalid', 'true');
    // Nobody is told twice as a general message.
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    // On to the Owner step: its message is at its own field, and everything typed is still there.
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'auth-wizard-next')) }),
    );
    const username = field(language, 'auth-login-username');
    expect(username).toHaveAttribute('aria-invalid', 'true');
    expect(username.value).toBe('owner');
  });

  it('an empty step is held back with a hint at each empty required field, and nothing is sent', async () => {
    const user = await start();
    // Enter through the six firm fields with nothing typed.
    await user.keyboard('{Enter}{Enter}{Enter}{Enter}{Enter}{Enter}');
    expect(
      screen.getByRole('heading', { name: message(language, 'auth-wizard-firm-title') }),
    ).toBeInTheDocument();
    for (const id of [
      'auth-wizard-firm-name-en',
      'auth-wizard-firm-name-ur',
      'auth-wizard-mandi-en',
      'auth-wizard-mandi-ur',
    ]) {
      const input = field(language, id);
      expect(input).toHaveAttribute('aria-invalid', 'true');
      const note = document.getElementById(input.getAttribute('aria-describedby') ?? '');
      expect(visible(note?.textContent)).toBe(message(language, 'err-required'));
    }
    // Phone and address may stay empty.
    expect(field(language, 'auth-wizard-phone')).not.toHaveAttribute('aria-invalid', 'true');
    // Focus goes to the first field that needs a look, in the order on screen.
    expect(
      field(language, language === 'ur' ? 'auth-wizard-firm-name-ur' : 'auth-wizard-firm-name-en'),
    ).toHaveFocus();
    expect(authApi.setupFirstRun).not.toHaveBeenCalled();
  });

  it('the Owner step is held back the same way, and the backend is not called', async () => {
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard('{Enter}{Enter}{Enter}{Enter}');
    expect(field(language, 'auth-wizard-owner-name')).toHaveAttribute('aria-invalid', 'true');
    expect(field(language, 'auth-wizard-password')).toHaveAttribute('aria-invalid', 'true');
    expect(authApi.setupFirstRun).not.toHaveBeenCalled();
  });

  it('Urdu letters in an English name are flagged at once; English letters in an Urdu name are flagged only once they stay English', async () => {
    const user = await start();
    const english = field(language, 'auth-wizard-firm-name-en');
    const urdu = field(language, 'auth-wizard-firm-name-ur');
    const noteOf = (input: HTMLElement) =>
      visible(document.getElementById(input.getAttribute('aria-describedby') ?? '')?.textContent);
    await user.click(english);
    await user.keyboard('مہر');
    expect(english).toHaveAttribute('aria-invalid', 'true');
    expect(noteOf(english)).toBe(message(language, 'err-name-en-script'));
    // In the Urdu field, English letters are a word waiting to become Urdu: not a mistake.
    await user.click(urdu);
    await user.keyboard('Maher');
    expect(urdu).not.toHaveAttribute('aria-invalid', 'true');
    // With the typing mode on EN (no conversion) English letters stay English: a mistake.
    await user.keyboard('{Alt>}u{/Alt}{Alt>}u{/Alt}');
    expect(urdu).toHaveAttribute('aria-invalid', 'true');
    expect(noteOf(urdu)).toBe(message(language, 'err-name-ur-script'));
    // Digits and punctuation are allowed in both, and a name in the right script is not flagged.
    await user.click(english);
    await user.keyboard('Maher & Sons (1)');
    expect(english).not.toHaveAttribute('aria-invalid', 'true');
  });

  it('the tariq mehmood flow: English letters become Urdu, the English name fills itself, and both reach the backend', async () => {
    const dictionary: Record<string, string[]> = {
      tariq: ['طارق', 'تارق'],
      mehmood: ['محمود', 'مہمود'],
    };
    vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
      Promise.resolve(suggestionsFor({ toUrdu: dictionary }, text, direction)),
    );
    vi.mocked(authApi.setupFirstRun).mockResolvedValue({ recovery_code: '7KQ2M9XD4TPAHW3C' });
    const user = await start();
    const urdu = field(language, 'auth-wizard-firm-name-ur');
    const english = field(language, 'auth-wizard-firm-name-en');
    await user.click(urdu);
    await user.keyboard('tariq ');
    await user.keyboard('mehmood');
    await waitFor(() => {
      expect(screen.getAllByRole('option')).toHaveLength(2);
    });
    await user.keyboard('{Enter}');
    expect(urdu.value).toBe('طارق محمود');
    await waitFor(() => {
      expect(english.value).toBe('Tariq Mehmood');
    });
    // The mandi name the same way, then the rest of the step.
    await user.click(field(language, 'auth-wizard-mandi-en'));
    await user.keyboard(MANDI_NAMES.en);
    await user.click(field(language, 'auth-wizard-mandi-ur'));
    await user.keyboard(MANDI_NAMES.ur);
    await user.click(field(language, 'auth-wizard-phone'));
    await user.keyboard('0300-0000000{Enter}Shop 1{Enter}');
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    await waitFor(() => {
      expect(authApi.setupFirstRun).toHaveBeenCalledWith(
        expect.objectContaining({ firm_name_ur: 'طارق محمود', firm_name_en: 'Tariq Mehmood' }),
      );
    });
  });

  it('a firm name word made by the spelling rules alone is checked on the first step, then sent as confirmed (FR-7.66)', async () => {
    vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
      Promise.resolve(
        suggestionsFor(
          { toUrdu: { zaheer: ['ظہیر'] }, toEnglish: { ظہیر: 'Zaheer' }, rulesOnly: ['zaheer'] },
          text,
          direction,
        ),
      ),
    );
    vi.mocked(authApi.setupFirstRun).mockResolvedValue({ recovery_code: '7KQ2M9XD4TPAHW3C' });
    const user = await start();
    await user.click(field(language, 'auth-wizard-firm-name-ur'));
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(field(language, 'auth-wizard-firm-name-en').value).toBe('Zaheer');
    });
    await user.click(field(language, 'auth-wizard-mandi-en'));
    await user.keyboard(MANDI_NAMES.en);
    await user.click(field(language, 'auth-wizard-mandi-ur'));
    await user.keyboard(MANDI_NAMES.ur);
    await user.click(field(language, 'auth-wizard-phone'));
    await user.keyboard('0300-0000000{Enter}Shop 1{Enter}');
    // Going on from the names asks first, and nothing is sent.
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain('ظہیر');
    expect(authApi.setupFirstRun).not.toHaveBeenCalled();
    await user.click(
      within(dialog).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    await waitFor(() => {
      expect(authApi.setupFirstRun).toHaveBeenCalledWith(
        expect.objectContaining({ confirmed_rules_words: ['ظہیر'] }),
      );
    });
  });

  it('going back to fix the spelling stays on the first step and sends nothing', async () => {
    vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
      Promise.resolve(
        suggestionsFor(
          { toUrdu: { zaheer: ['ظہیر'] }, toEnglish: { ظہیر: 'Zaheer' }, rulesOnly: ['zaheer'] },
          text,
          direction,
        ),
      ),
    );
    const user = await start();
    await user.click(field(language, 'auth-wizard-firm-name-ur'));
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(field(language, 'auth-wizard-firm-name-en').value).toBe('Zaheer');
    });
    await user.click(field(language, 'auth-wizard-mandi-en'));
    await user.keyboard(MANDI_NAMES.en);
    await user.click(field(language, 'auth-wizard-mandi-ur'));
    await user.keyboard(MANDI_NAMES.ur);
    await user.click(field(language, 'auth-wizard-phone'));
    await user.keyboard('0300-0000000{Enter}Shop 1{Enter}');
    const dialog = await screen.findByRole('alertdialog');
    await user.click(
      within(dialog).getByRole('button', { name: new RegExp(message(language, 'common-back')) }),
    );
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole('heading', { name: message(language, 'auth-wizard-firm-title') }),
    ).toBeInTheDocument();
    expect(authApi.setupFirstRun).not.toHaveBeenCalled();
  });

  it('switching language while a word is being typed keeps the letters and the typing mode', async () => {
    const user = await start();
    const urdu = field(language, 'auth-wizard-firm-name-ur');
    await user.click(urdu);
    await user.keyboard('{Alt>}u{/Alt}');
    await user.keyboard('ab');
    expect(urdu.value).toBe('اب');
    await user.keyboard('{Alt>}l{/Alt}');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await screen.findByRole('heading', { name: message(other, 'auth-wizard-firm-title') });
    const again = field(other, 'auth-wizard-firm-name-ur');
    expect(again.value).toBe('اب');
    // The mode is the one he chose for this name, in the words of the new language; the mandi
    // name beside it keeps its own.
    const badges = screen.getAllByRole('button', {
      name: (name) => visible(name).startsWith(message(other, 'translit-mode-label')),
    });
    expect(badges[0]).toHaveTextContent(message(other, 'translit-mode-urdu'));
    expect(badges[1]).toHaveTextContent(message(other, 'translit-mode-roman'));
  });

  it('Back and Next keep everything typed on both steps', async () => {
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard('Adnan{Enter}owner{Enter}secret12{Enter}secret');
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'common-back')) }),
    );
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-firm-title') });
    expect(field(language, 'auth-wizard-firm-name-en').value).toBe('Maher Adnan and Brothers');
    expect(field(language, 'auth-wizard-mandi-ur').value).toBe('نئی سبزی منڈی');
    expect(field(language, 'auth-wizard-phone').value).toBe('0300-0000000');
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'auth-wizard-next')) }),
    );
    expect(field(language, 'auth-wizard-owner-name').value).toBe('Adnan');
    expect(field(language, 'auth-login-username').value).toBe('owner');
    expect(field(language, 'auth-wizard-password').value).toBe('secret12');
    expect(field(language, 'auth-wizard-password-repeat').value).toBe('secret');
  });

  it('a password problem stays on the Owner step, at its field, with the limit in the words', async () => {
    vi.mocked(authApi.setupFirstRun).mockRejectedValue(
      fail({ code: 'ERR_PASSWORD_TOO_SHORT', field: 'password', values: { min: 6 } }),
    );
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard('Adnan{Enter}owner{Enter}abc{Enter}abc{Enter}');
    const password = field(language, 'auth-wizard-password');
    await waitFor(() => {
      expect(password).toHaveAttribute('aria-invalid', 'true');
    });
    const note = document.getElementById(password.getAttribute('aria-describedby') ?? '');
    expect(visible(note?.textContent)).toBe(
      message(language, 'err-password-too-short', { min: '6' }),
    );
    expect(
      screen.getByRole('heading', { name: message(language, 'auth-wizard-owner-title') }),
    ).toBeInTheDocument();
  });

  /** The firm and Owner steps, signed in after the setup, up to the partners step. */
  async function toPartnersStep() {
    vi.mocked(authApi.setupFirstRun).mockResolvedValue({ recovery_code: '7KQ2M9XD4TPAHW3C' });
    vi.mocked(authApi.authLogin).mockResolvedValue(sessionUser({ language }));
    const store = fakePartners([]);
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    // He is signed in with what he just typed, before the passwords are cleared (plan 11).
    await waitFor(() => {
      expect(authApi.authLogin).toHaveBeenCalledWith({ username: 'owner', password: 'secret12' });
    });
    await screen.findByLabelText('7KQ2M9XD4TPAHW3C');
    expect(useWizardStore.getState().values.password).toBe('');
    // The screen stays on the wizard: the sign-in did not refresh who is signed in.
    expect(authApi.authStatus).toHaveBeenCalledTimes(1);
    await user.click(screen.getByRole('checkbox'));
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'auth-wizard-next')) }),
    );
    await screen.findByRole('heading', {
      name: (name) => visible(name) === message(language, 'auth-wizard-partners-title'),
    });
    // The recovery code was shown once: it is gone from the screen and from the store.
    expect(screen.queryByLabelText('7KQ2M9XD4TPAHW3C')).toBeNull();
    expect(useWizardStore.getState().recoveryCode).toBeNull();
    return { user, store };
  }

  const named = (text: string) => (name: string) => visible(name) === text;

  it('signed in after the setup, the code step goes on to the partners step, which can be left for later', async () => {
    const { user } = await toPartnersStep();
    const steps = screen.getByRole('list', { name: message(language, 'auth-wizard-steps') });
    expect(steps.querySelectorAll('li')[3]).toHaveAttribute('aria-current', 'step');
    expect(
      await screen.findByRole('heading', {
        name: named(message(language, 'partners-setup-empty')),
      }),
    ).toBeInTheDocument();
    // The Owner is signed in: the app opens when the wizard ends, with no sign-in screen between.
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ first_run: false, session: sessionUser({ language }), login_language: language }),
    );
    await user.click(
      screen.getByRole('button', {
        name: new RegExp(message(language, 'auth-wizard-partners-later')),
      }),
    );
    await waitFor(() => {
      expect(
        screen.queryByRole('list', { name: message(language, 'auth-wizard-steps') }),
      ).toBeNull();
    });
    expect(partnersApi.partnersSetShares).not.toHaveBeenCalled();
    expect(useWizardStore.getState().step).toBe(0);
    expect(useWizardStore.getState().signedIn).toBe(false);
  });

  it('adds three partners and their shares in the wizard, by keyboard, and ends signed in', async () => {
    const { user, store } = await toPartnersStep();
    const names = language === 'ur' ? ['الف', 'ب', 'ج'] : ['Partner A', 'Partner B', 'Partner C'];
    const urdu = ['الف', 'ب', 'ج'];
    const english = ['Partner A', 'Partner B', 'Partner C'];
    for (const index of names.keys()) {
      press('F2');
      const dialog = await screen.findByRole('dialog', {
        name: named(message(language, 'partners-add-title')),
      });
      await user.click(
        await within(dialog).findByLabelText(named(message(language, 'partners-field-name-ur'))),
      );
      await user.keyboard(urdu[index] ?? '');
      await user.click(
        within(dialog).getByLabelText(named(message(language, 'partners-field-name-en'))),
      );
      await user.keyboard(english[index] ?? '');
      // The last one is saved with F10, which closes the dialog; the others with Ctrl+Enter.
      press(index === names.length - 1 ? 'F10' : 'Enter', index === 2 ? {} : { ctrlKey: true });
      await waitFor(() => {
        expect(store.rows).toHaveLength(index + 1);
      });
    }
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).toBeNull();
    });
    const box = (name: string) =>
      screen.findByLabelText<HTMLInputElement>(
        named(message(language, 'partners-share-of', { name })),
      );
    expect((await box(names[0] ?? '')).value).toBe('');
    await user.click(await box(names[0] ?? ''));
    await user.keyboard('40{Enter}30{Enter}30');
    const save = screen.getByRole('button', {
      name: (name) => visible(name).includes(message(language, 'auth-wizard-partners-save')),
    });
    await waitFor(() => {
      expect(save).toBeEnabled();
    });
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ first_run: false, session: sessionUser({ language }), login_language: language }),
    );
    await user.click(save);
    await waitFor(() => {
      expect(store.setShares).toEqual([
        {
          effective_from: '10-10-2026',
          shares: [
            { partner_id: 1, share: '40' },
            { partner_id: 2, share: '30' },
            { partner_id: 3, share: '30' },
          ],
        },
      ]);
    });
    await waitFor(() => {
      expect(
        screen.queryByRole('list', { name: message(language, 'auth-wizard-steps') }),
      ).toBeNull();
    });
    expect(useWizardStore.getState().step).toBe(0);
  });

  it('a command that is not ready shows its message, and nothing is made up', async () => {
    vi.mocked(authApi.setupFirstRun).mockRejectedValue(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    const user = await start();
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'err-not-implemented'));
    expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
  });

  it('switching language in the middle of a step keeps every value and the typing mode', async () => {
    const user = await start();
    // Firm names in the order of the fields, then the first mandi field halfway.
    const urduFirst = language === 'ur';
    await user.keyboard(
      `${urduFirst ? 'فرم' : 'Maher'}{Enter}${urduFirst ? 'Maher' : 'فرم'}{Enter}`,
    );
    await user.keyboard(urduFirst ? 'منڈی' : 'New');
    await user.keyboard('{Alt>}l{/Alt}');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    await screen.findByRole('heading', { name: message(other, 'auth-wizard-firm-title') });
    expect(useLanguageStore.getState().language).toBe(other);
    expect(field(other, 'auth-wizard-firm-name-en').value).toBe('Maher');
    expect(field(other, 'auth-wizard-firm-name-ur').value).toBe('فرم');
    expect(field(other, urduFirst ? 'auth-wizard-mandi-ur' : 'auth-wizard-mandi-en').value).toBe(
      urduFirst ? 'منڈی' : 'New',
    );
    // The order of the pair follows the app: the name to type comes first.
    const english = field(other, 'auth-wizard-firm-name-en');
    const urdu = field(other, 'auth-wizard-firm-name-ur');
    const first = other === 'ur' ? urdu : english;
    const second = other === 'ur' ? english : urdu;
    expect(first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING).not.toBe(0);
    expect(settingsApi.prefsSet).not.toHaveBeenCalled();
    expect(appApi.exitApp).not.toHaveBeenCalled();
  });
});
