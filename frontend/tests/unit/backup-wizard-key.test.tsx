// The backup key step of the first-run wizard (plan 18, PRD NFR-INS-2, FR-20.10, NFR-BAK-7), in the
// Urdu and in the English app, against a fake backend: it comes after the recovery code and before
// the partners, makes the key once, shows it once, prints it (F9), asks the Owner to type it back
// from the paper, and goes on. A language switch loses nothing and asks for no second key. A key
// that is in use already is not shown again. Without a sign-in after the setup there is no key step.

import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as authApi from '../../src/features/auth/api';
import * as backupApi from '../../src/features/backup/api';
import { useBackupKeyStore } from '../../src/features/backup/key-store';
import { useWizardStore } from '../../src/features/auth/wizard-store';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeAccountSearch, fakeAccounts } from '../helpers/accounts';
import {
  authState,
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
} from '../helpers/app';
import { KEY_SETUP } from '../helpers/backup-real';
import { fakeMasters } from '../helpers/masters';
import { fakePartners } from '../helpers/partners';
import { message, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/features/partners/api');
vi.mock('../../src/features/backup/api');
vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const FIRM_NAMES = { en: 'Maher Adnan and Brothers', ur: 'مہر عدنان اینڈ برادرز' };
const MANDI_NAMES = { en: 'New Sabzi Mandi', ur: 'نئی سبزی منڈی' };
const OWNER_STEP = 'Adnan{Enter}owner{Enter}secret12{Enter}secret12{Enter}';
const printSpy = vi.fn();

/** The firm step typed in the order of its fields (the same as the wizard test). */
function firmStep(language: Language): string {
  const inOrder = (names: { en: string; ur: string }) =>
    language === 'ur' ? [names.ur, names.en] : [names.en, names.ur];
  return [...inOrder(FIRM_NAMES), ...inOrder(MANDI_NAMES), '0300-0000000', 'Shop 1']
    .map((text) => `${text}{Enter}`)
    .join('');
}

const named = (text: string) => (name: string) => visible(name) === text;

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(backupApi.backupKeySetup).mockResolvedValue(KEY_SETUP);
  vi.mocked(backupApi.backupKeyConfirm).mockResolvedValue(null);
  vi.mocked(authApi.setupFirstRun).mockResolvedValue({ recovery_code: '7KQ2M9XD4TPAHW3C' });
  fakeAccounts([]);
  fakeAccountSearch([]);
  fakeMasters({});
  fakePartners([]);
  window.print = printSpy;
  printSpy.mockClear();
});

afterEach(() => {
  leaveAppWindow();
});

describe.each(LANGUAGES)('the backup key step of the wizard (%s)', (language) => {
  beforeEach(() => {
    resetClientState(language);
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ first_run: true, login_language: language }),
    );
    vi.mocked(authApi.authLogin).mockResolvedValue(sessionUser({ language }));
  });

  /** The firm and Owner steps, then the recovery code step finished: the key step is next. */
  async function toKeyStep() {
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', {
      name: (name) => visible(name) === message(language, 'auth-wizard-firm-title'),
    });
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    await screen.findByLabelText('7KQ2M9XD4TPAHW3C');
    await user.click(screen.getByRole('checkbox'));
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'auth-wizard-next')) }),
    );
    await screen.findByRole('heading', {
      name: named(message(language, 'backup-wizard-key-title')),
    });
    return user;
  }

  const steps = () => screen.getByRole('list', { name: message(language, 'auth-wizard-steps') });
  const typedField = () =>
    screen.getByLabelText<HTMLInputElement>(named(message(language, 'backup-key-type-label')));
  const confirmButton = () =>
    screen.getByRole('button', { name: new RegExp(message(language, 'backup-key-confirm')) });

  it('comes after the recovery code and before the partners, and shows the key made by the backend', async () => {
    await toKeyStep();
    const items = steps().querySelectorAll('li');
    expect(items).toHaveLength(5);
    expect(items[3]).toHaveAttribute('aria-current', 'step');
    expect(visible(items[3]?.textContent)).toContain(message(language, 'backup-wizard-step-key'));
    expect(await screen.findByLabelText(KEY_SETUP.key)).toHaveTextContent(KEY_SETUP.key);
    expect(backupApi.backupKeySetup).toHaveBeenCalledTimes(1);
    // The recovery code was shown once and is gone.
    expect(screen.queryByLabelText('7KQ2M9XD4TPAHW3C')).toBeNull();
    // The sheet that is printed holds the key.
    expect(visible(document.querySelector('[data-key-sheet]')?.textContent)).toContain(
      KEY_SETUP.key,
    );
  });

  it('F9 prints, and confirming waits until the Owner says it is printed', async () => {
    const user = await toKeyStep();
    await screen.findByLabelText(KEY_SETUP.key);
    act(() => {
      fireEvent.keyDown(window, { code: 'F9' });
    });
    expect(printSpy).toHaveBeenCalledTimes(1);
    await user.type(typedField(), KEY_SETUP.key);
    expect(confirmButton()).toBeDisabled();
    await user.click(screen.getByRole('checkbox'));
    expect(confirmButton()).toBeEnabled();
  });

  it('a wrong key is shown at its field and the step stays; the right key goes on to the partners', async () => {
    const user = await toKeyStep();
    await screen.findByLabelText(KEY_SETUP.key);
    await user.click(screen.getByRole('checkbox'));
    vi.mocked(backupApi.backupKeyConfirm).mockRejectedValueOnce(
      fail({ code: 'ERR_BACKUP_KEY_WRONG', field: 'key' }),
    );
    await user.type(typedField(), 'ABCD-EFGH-JKLM-NPQR-STUV-WXYZ');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(typedField()).toHaveAttribute('aria-invalid', 'true');
    });
    expect(visible(document.body.textContent)).toContain(message(language, 'err-backup-key-wrong'));
    expect(steps().querySelectorAll('li')[3]).toHaveAttribute('aria-current', 'step');
    await user.clear(typedField());
    await user.type(typedField(), KEY_SETUP.key);
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(backupApi.backupKeyConfirm).toHaveBeenLastCalledWith({ key: KEY_SETUP.key });
    });
    await screen.findByRole('heading', {
      name: named(message(language, 'auth-wizard-partners-title')),
    });
    expect(steps().querySelectorAll('li')[4]).toHaveAttribute('aria-current', 'step');
    // The key is shown once: it is gone from the store after the step.
    expect(useBackupKeyStore.getState().key).toBeNull();
    expect(screen.queryByLabelText(KEY_SETUP.key)).toBeNull();
  });

  it('"do this later" goes on to the partners and drops the key', async () => {
    const user = await toKeyStep();
    await screen.findByLabelText(KEY_SETUP.key);
    await user.click(screen.getByRole('button', { name: message(language, 'backup-key-later') }));
    await screen.findByRole('heading', {
      name: named(message(language, 'auth-wizard-partners-title')),
    });
    expect(backupApi.backupKeyConfirm).not.toHaveBeenCalled();
    expect(useBackupKeyStore.getState().key).toBeNull();
  });

  it('a language switch in the middle keeps the key and the typing, and asks for no second key', async () => {
    const user = await toKeyStep();
    await screen.findByLabelText(KEY_SETUP.key);
    await user.click(screen.getByRole('checkbox'));
    await user.type(typedField(), 'ABCD-EF');
    const other: Language = language === 'ur' ? 'en' : 'ur';
    act(() => {
      useLanguageStore.setState({ language: other });
    });
    const moved = await screen.findByLabelText<HTMLInputElement>(
      named(message(other, 'backup-key-type-label')),
    );
    expect(moved.value).toBe('ABCD-EF');
    expect(screen.getByLabelText(KEY_SETUP.key)).toBeInTheDocument();
    expect(screen.getByRole<HTMLInputElement>('checkbox').checked).toBe(true);
    expect(backupApi.backupKeySetup).toHaveBeenCalledTimes(1);
  });

  it('a key that is in use already is not shown again, and the Owner goes on', async () => {
    vi.mocked(backupApi.backupKeySetup).mockRejectedValue(
      fail({ code: 'ERR_BACKUP_KEY_ALREADY_SET' }),
    );
    const user = await toKeyStep();
    expect(
      await screen.findByText(named(message(language, 'backup-key-already'))),
    ).toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: message(language, 'backup-key-continue') }),
    );
    await screen.findByRole('heading', {
      name: named(message(language, 'auth-wizard-partners-title')),
    });
  });

  it('a key that cannot be made says so with a retry that works', async () => {
    vi.mocked(backupApi.backupKeySetup).mockRejectedValueOnce(fail({ code: 'ERR_INTERNAL' }));
    const user = await toKeyStep();
    const alert = await screen.findByRole('alert');
    expect(visible(alert.textContent)).toContain(message(language, 'backup-key-failed'));
    expect(visible(alert.textContent)).toContain(message(language, 'err-internal'));
    await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
    expect(await screen.findByLabelText(KEY_SETUP.key)).toBeInTheDocument();
  });

  it('with no sign-in after the setup there is no key step: the wizard ends at the sign-in screen', async () => {
    vi.mocked(authApi.authLogin).mockRejectedValue(fail({ code: 'ERR_INTERNAL' }));
    await renderApp();
    const user = userEvent.setup();
    await screen.findByRole('heading', {
      name: (name) => visible(name) === message(language, 'auth-wizard-firm-title'),
    });
    await user.keyboard(firmStep(language));
    await screen.findByRole('heading', { name: message(language, 'auth-wizard-owner-title') });
    await user.keyboard(OWNER_STEP);
    await screen.findByLabelText('7KQ2M9XD4TPAHW3C');
    await user.click(screen.getByRole('checkbox'));
    vi.mocked(authApi.authStatus).mockResolvedValue(
      authState({ first_run: false, login_language: language }),
    );
    await user.click(
      screen.getByRole('button', { name: new RegExp(message(language, 'auth-wizard-finish')) }),
    );
    expect(
      await screen.findByRole('heading', { name: message(language, 'auth-login-title') }),
    ).toBeInTheDocument();
    expect(backupApi.backupKeySetup).not.toHaveBeenCalled();
    expect(useWizardStore.getState().signedIn).toBe(false);
  });
});
