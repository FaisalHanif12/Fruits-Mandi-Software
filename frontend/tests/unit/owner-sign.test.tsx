// The Owner's sign (دستخط) when a closed day is reopened to fix an entry (PRD FR-21.4, FR-7.71, Section 21,
// v1.36, plan 73), in the Urdu and the English app, against a test double of the plan's contract (the backend's
// types and files follow): the reopen box has a second, required field below the reason; the sign is plain text;
// `day_fix_open` gets it as typed; ERR_SIGN_WRONG and ERR_SIGN_REQUIRED are told under the field and nothing
// opens; the stamp, its row mark and the edit history never show the sign (v1.58, plan 96 T5); Settings, Security changes
// the Owner's sign with his password and his current sign, and the new sign typed twice (plan 92). Keyboard only:
// Enter goes from the reason to the sign and opens.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Localized } from '@fluent/react';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import * as settingsApi from '../../src/features/settings/api';
import type { EditStamp } from '../../src/bindings/EditStamp';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as editApi from '../../src/shared/edit/api';
import { useFixDayFlow } from '../../src/shared/edit';
import { EditStampLine, EditedMark } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { named } from '../helpers/bikri-screen';
import { CommandError } from '../../src/shared/api';
import {
  fail,
  healthyBackend,
  leaveAppWindow,
  renderApp,
  resetClientState,
  sessionUser,
  signedIn,
} from '../helpers/app';
import {
  AUDIT_EDIT_HISTORY_BIKRI,
  AUDIT_EDIT_STAMP_BIKRI,
  DAY_FIX_OPEN_E2E_45,
  DAY_FIX_OPEN_SIGN_REQUIRED,
  DAY_FIX_OPEN_SIGN_WRONG,
  DAY_FIX_OPEN_SIGN_WRONG_REQUEST,
} from '../helpers/edit-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/auth/api');
vi.mock('../../src/features/settings/api');
vi.mock('../../src/shared/edit/api');
vi.mock('../../src/shared/urdu/api');

beforeEach(() => {
  vi.resetAllMocks();
  healthyBackend();
  vi.mocked(editApi.dayFixOpen).mockResolvedValue(DAY_FIX_OPEN_E2E_45);
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
});

afterEach(() => {
  leaveAppWindow();
});

type User = ReturnType<typeof userEvent.setup>;

/** A screen that does what every caller does: `start` with the day of the entry, and the box. */
function Caller({ then }: { then: () => void }) {
  const fix = useFixDayFlow();
  return (
    <>
      <Localized id="edit-action-fix-day">
        <button
          type="button"
          onClick={() => {
            fix.start({ date: '2026-10-10', ownerHref: '/bikri/book', then });
          }}
        />
      </Localized>
      {fix.dialog}
    </>
  );
}

describe.each(LANGUAGES)('the Owner’s sign in the reopen box (%s)', (language) => {
  const reasonBox = (dialog: HTMLElement) =>
    within(dialog).getByRole('combobox', { name: named(message(language, 'edit-fix-label')) });
  const signBox = (dialog: HTMLElement) =>
    within(dialog).getByRole('textbox', { name: named(message(language, 'edit-fix-sign-label')) });

  async function openBox(then: () => void = vi.fn()) {
    renderIn(language, <Caller then={then} />);
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('button', { name: named(message(language, 'edit-action-fix-day')) }),
    );
    return { user, dialog: await screen.findByRole('alertdialog') };
  }

  /** The reason typed and Enter, which goes on to the sign. */
  async function typeReason(user: User, dialog: HTMLElement) {
    await user.type(reasonBox(dialog), 'wrong rate{Enter}');
  }

  it('asks for the sign below the reason, and it is plain text: nothing is turned into Urdu letters', async () => {
    const { user, dialog } = await openBox();
    const reason = reasonBox(dialog);
    const sign = signBox(dialog);
    // The sign is the second field of the box, after the reason.
    expect(reason.compareDocumentPosition(sign) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    await user.type(sign, 'adnan');
    expect((sign as HTMLInputElement).value).toBe('adnan');
    // The Roman-to-Urdu service was never asked about the sign.
    expect(urduApi.translitSuggest).not.toHaveBeenCalledWith(
      expect.objectContaining({ text: 'adnan' }),
    );
  });

  it('keyboard only: Enter in the reason goes to the sign, Enter in the sign opens the day with the sign as typed', async () => {
    const then = vi.fn();
    const { user, dialog } = await openBox(then);
    await typeReason(user, dialog);
    await waitFor(() => {
      expect(signBox(dialog)).toHaveFocus();
    });
    await user.keyboard('adnan{Enter}');
    await waitFor(() => {
      expect(editApi.dayFixOpen).toHaveBeenCalledWith('10-10-2026', 'wrong rate', 'adnan');
    });
    await waitFor(() => {
      expect(then).toHaveBeenCalledTimes(1);
    });
  });

  it.each([
    ['capital letters and spaces at the ends', '  ADNAN ', 'ADNAN'],
    ['Urdu letters', 'عدنان', 'عدنان'],
  ])(
    'a sign with %s goes as typed, trimmed (the backend judges it)',
    async (_name, typed, sent) => {
      const { user, dialog } = await openBox();
      await typeReason(user, dialog);
      await user.keyboard(`${typed}{Enter}`);
      await waitFor(() => {
        expect(editApi.dayFixOpen).toHaveBeenCalledWith('10-10-2026', 'wrong rate', sent);
      });
    },
  );

  it('an empty sign is told at the field and nothing is sent', async () => {
    const { user, dialog } = await openBox();
    await typeReason(user, dialog);
    await user.keyboard('{Enter}');
    expect(await within(dialog).findByText(named(message(language, 'err-required')))).toBeTruthy();
    expect(editApi.dayFixOpen).not.toHaveBeenCalled();
  });

  // The backend's own refusals (backend/crates/services/tests/fixtures), with the request each one answers:
  // the wrong sign "munshi", and the sign that is missing.
  it.each([
    [
      'ERR_SIGN_WRONG',
      'err-sign-wrong',
      DAY_FIX_OPEN_SIGN_WRONG,
      DAY_FIX_OPEN_SIGN_WRONG_REQUEST.sign,
    ],
    ['ERR_SIGN_REQUIRED', 'err-sign-required', DAY_FIX_OPEN_SIGN_REQUIRED, 'x'],
  ])(
    '%s: the words are under the sign, nothing opens, and what was typed stays',
    async (code, id, answer, typed) => {
      expect(answer.errors[0]).toMatchObject({ code, field: 'sign' });
      vi.mocked(editApi.dayFixOpen).mockRejectedValue(new CommandError(answer));
      const then = vi.fn();
      const { user, dialog } = await openBox(then);
      await typeReason(user, dialog);
      await user.keyboard(`${typed}{Enter}`);
      const words = message(language, id);
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(words);
      });
      // Under the sign field, in its own words ("دستخط درست نہیں" in Urdu), not only in a general notice.
      const field = signBox(dialog).closest('div');
      expect(visible(field?.parentElement?.textContent)).toContain(words);
      expect(then).not.toHaveBeenCalled();
      expect(screen.getByRole('alertdialog')).toBeInTheDocument();
      expect((reasonBox(dialog) as HTMLInputElement).value).toBe('wrong rate');
      expect((signBox(dialog) as HTMLInputElement).value).toBe(typed);
    },
  );
});

function withQuery(ui: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{ui}</QueryClientProvider>;
}

describe.each(LANGUAGES)(
  'the sign is never shown in a stamp, a tip or the history (%s)',
  (language) => {
    const base = AUDIT_EDIT_STAMP_BIKRI[0]?.stamp;
    if (base === undefined || base === null) throw new Error('the backend file has a stamp');
    const stamp: EditStamp = base;
    const when = {
      date: '11-10-2026',
      time: message(language, 'edit-time', {
        hour: '2',
        minute: '00',
        part: 'noon',
        meridian: 'pm',
      }),
      by: 'Maher Adnan',
    };
    // Everything a screen must never print: the sign as typed, and its word as a label.
    const secret = (text: string | null | undefined) => {
      const seen = visible(text);
      expect(seen).not.toContain('adnan');
      expect(seen).not.toContain(message(language, 'glossary-dastakhat'));
    };

    // An older backend sent the sign on the stamp and on each history version (plans 73 and 92). The types no longer
    // have it (plan 96 T4), but a screen must still never print it, so the tests build such data on purpose. A value
    // held in a variable is not an object literal, so it needs no cast.
    const legacy = { ...stamp, sign: 'adnan' };

    it('a stamp that still carries the sign shows the date, the time and who, and not the sign', () => {
      renderIn(language, <EditStampLine stamp={legacy} />);
      const text = visible(document.body.textContent);
      expect(text).toContain(message(language, 'edit-stamp', when));
      secret(text);
    });

    it('a stamp with a sign reads the same as a stamp without one', () => {
      const first = renderIn(language, <EditStampLine stamp={legacy} />);
      const signed = visible(document.querySelector('[data-edit-stamp]')?.textContent);
      first.unmount();
      renderIn(language, <EditStampLine stamp={stamp} />);
      expect(visible(document.querySelector('[data-edit-stamp]')?.textContent)).toBe(signed);
      secret(signed);
    });

    it('the row mark shows the date, the time and who on hover, with the reason after them, and never the sign', () => {
      const reasoned = { ...legacy, reason: 'wrong rate' };
      const first = renderIn(language, <EditedMark stamp={reasoned} />);
      const mark = screen.getByText(message(language, 'edit-mark')).closest('[data-edited-mark]');
      expect(visible(mark?.getAttribute('title'))).toBe(
        message(language, 'edit-mark-tip-reason', { ...when, reason: 'wrong rate' }),
      );
      secret(mark?.getAttribute('title'));
      first.unmount();
      // No reason typed: only the date, the time and who.
      renderIn(language, <EditedMark stamp={{ ...reasoned, reason: null }} />);
      const bare = screen.getByText(message(language, 'edit-mark')).closest('[data-edited-mark]');
      expect(visible(bare?.getAttribute('title'))).toBe(message(language, 'edit-mark-tip', when));
      secret(bare?.getAttribute('title'));
    });

    it('the edit history shows when, who and the reason of each version, and never the sign, not as a version word and not as a changed field', async () => {
      const [first, ...rest] = AUDIT_EDIT_HISTORY_BIKRI.versions;
      if (first === undefined) throw new Error('the backend file has a version');
      // A version of an older backend: the sign as a field of the version and as a changed field. Held in a
      // variable, so it is not an object literal and needs no cast.
      const legacyVersion = {
        ...first,
        sign: 'adnan',
        changes: [
          ...first.changes,
          {
            path: 'sign',
            old: { kind: 'null' as const },
            new: { kind: 'text' as const, text: 'adnan' },
          },
        ],
      };
      vi.mocked(editApi.auditEditHistory).mockResolvedValue({
        ...AUDIT_EDIT_HISTORY_BIKRI,
        versions: [legacyVersion, ...rest],
      });
      renderIn(
        language,
        withQuery(<EditStampLine stamp={stamp} history={{ entity: 'bikri', id: 1 }} />),
      );
      const user = userEvent.setup();
      await user.click(
        screen.getByRole('button', {
          name: (name) => visible(name).startsWith(message(language, 'edit-action-history')),
        }),
      );
      const dialog = await screen.findByRole('dialog');
      await waitFor(() => {
        expect(visible(dialog.textContent)).toContain(message(language, 'edit-history-title'));
        expect(dialog.querySelectorAll('[data-edit-version]').length).toBeGreaterThan(0);
      });
      const versions = Array.from(dialog.querySelectorAll('[data-edit-version]'));
      expect(visible(versions[0]?.textContent)).toContain(
        message(language, 'edit-history-version', {
          date: '11-10-2026',
          time: message(language, 'edit-time', {
            hour: '2',
            minute: '00',
            part: 'noon',
            meridian: 'pm',
          }),
          by: first.by,
        }),
      );
      for (const version of versions) secret(version.textContent);
      secret(dialog.textContent);
    });
  },
);

describe.each(LANGUAGES)('Settings, Security: the Owner’s sign (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

  const field = (id: string) =>
    screen.getByLabelText<HTMLInputElement>(named(message(language, id)));

  async function openCard() {
    const app = await renderApp('/settings');
    await screen.findByRole('heading', { level: 1, name: message(language, 'settings-title') });
    const user = userEvent.setup();
    await user.click(
      screen.getByRole('tab', { name: named(message(language, 'settings-tab-lock')) }),
    );
    const password = await screen.findByLabelText<HTMLInputElement>(
      named(message(language, 'settings-sign-password')),
    );
    return {
      ...app,
      user,
      password,
      current: field('settings-sign-current'),
      sign: field('settings-sign-new'),
      again: field('settings-sign-new-again'),
    };
  }

  it('keyboard only: the password, the current sign, the new sign twice, each with Enter: all are sent as typed, and all are cleared', async () => {
    vi.mocked(settingsApi.settingsSetOwnerSign).mockResolvedValue(null);
    const { user, password, current, sign, again } = await openCard();
    await user.click(password);
    await user.keyboard('owner-pass{Enter}');
    await waitFor(() => {
      expect(current).toHaveFocus();
    });
    await user.keyboard('adnan{Enter}');
    await waitFor(() => {
      expect(sign).toHaveFocus();
    });
    await user.keyboard('sana{Enter}');
    await waitFor(() => {
      expect(again).toHaveFocus();
    });
    await user.keyboard('sana{Enter}');
    await waitFor(() => {
      expect(settingsApi.settingsSetOwnerSign).toHaveBeenCalledWith({
        password: 'owner-pass',
        current_sign: 'adnan',
        sign: 'sana',
      });
    });
    await waitFor(() => {
      expect(password.value).toBe('');
      expect(current.value).toBe('');
      expect(sign.value).toBe('');
      expect(again.value).toBe('');
    });
    expect(visible(document.body.textContent)).toContain(message(language, 'settings-sign-saved'));
  });

  it('two new signs that are not the same are told under the second one, and nothing is sent', async () => {
    const { user, password, current, sign, again } = await openCard();
    await user.type(password, 'owner-pass');
    await user.type(current, 'adnan');
    await user.type(sign, 'sana');
    await user.type(again, 'sanaa{Enter}');
    expect(
      await screen.findByText(named(message(language, 'settings-sign-mismatch'))),
    ).toBeTruthy();
    expect(settingsApi.settingsSetOwnerSign).not.toHaveBeenCalled();
    // What was typed stays, and typing again takes the words away.
    expect(sign.value).toBe('sana');
    await user.type(again, '{Backspace}');
    await waitFor(() => {
      expect(screen.queryByText(named(message(language, 'settings-sign-mismatch')))).toBeNull();
    });
  });

  it('the button waits until all four are filled', async () => {
    const { user, password, current, sign } = await openCard();
    const button = screen.getByRole('button', {
      name: named(message(language, 'settings-sign-button')),
    });
    expect(button).toBeDisabled();
    await user.type(password, 'owner-pass');
    await user.type(current, 'adnan');
    await user.type(sign, 'sana');
    expect(button).toBeDisabled();
  });

  it('a wrong password is told under the password, and nothing is cleared', async () => {
    vi.mocked(settingsApi.settingsSetOwnerSign).mockRejectedValue(
      fail({ code: 'ERR_WRONG_PASSWORD', field: 'password', values: { tries_left: 9 } }),
    );
    const { user, password, current, sign, again } = await openCard();
    await user.type(password, 'bad');
    await user.type(current, 'adnan');
    await user.type(sign, 'sana');
    await user.type(again, 'sana{Enter}');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'err-wrong-password', { tries_left: 9 }),
      );
    });
    expect(sign.value).toBe('sana');
    expect(current.value).toBe('adnan');
  });

  it.each([
    ['ERR_SIGN_WRONG', 'err-sign-wrong'],
    ['ERR_SIGN_REQUIRED', 'err-sign-required'],
  ])(
    'a current sign the backend refuses (%s) is told under the current sign, and nothing is cleared',
    async (code, id) => {
      vi.mocked(settingsApi.settingsSetOwnerSign).mockRejectedValue(
        fail({ code, field: 'current_sign' }),
      );
      const { user, password, current, sign, again } = await openCard();
      await user.type(password, 'owner-pass');
      await user.type(current, 'munshi');
      await user.type(sign, 'sana');
      await user.type(again, 'sana{Enter}');
      await waitFor(() => {
        expect(visible(current.closest('div')?.parentElement?.textContent)).toContain(
          message(language, id),
        );
      });
      expect(settingsApi.settingsSetOwnerSign).toHaveBeenCalledWith({
        password: 'owner-pass',
        current_sign: 'munshi',
        sign: 'sana',
      });
      expect(current.value).toBe('munshi');
      expect(sign.value).toBe('sana');
      expect(again.value).toBe('sana');
    },
  );

  it('a new sign the backend refuses is told under the new sign', async () => {
    vi.mocked(settingsApi.settingsSetOwnerSign).mockRejectedValue(
      fail({ code: 'ERR_REQUIRED', field: 'sign' }),
    );
    const { user, password, current, sign, again } = await openCard();
    await user.type(password, 'owner-pass');
    await user.type(current, 'adnan');
    await user.type(sign, 'x');
    await user.type(again, 'x{Enter}');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(message(language, 'err-required'));
    });
    expect(settingsApi.settingsSetOwnerSign).toHaveBeenCalledTimes(1);
  });
});
