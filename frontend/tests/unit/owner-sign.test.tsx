// The Owner's sign (دستخط) when a closed day is reopened to fix an entry (PRD FR-21.4, FR-7.71, Section 21,
// v1.36, plan 73), in the Urdu and the English app, against a test double of the plan's contract (the backend's
// types and files follow): the reopen box has a second, required field below the reason; the sign is plain text;
// `day_fix_open` gets it as typed; ERR_SIGN_WRONG and ERR_SIGN_REQUIRED are told under the field and nothing
// opens; the stamp, its row mark and the edit history show the sign after the name; Settings, Security changes
// the Owner's sign with his password. Keyboard only: Enter goes from the reason to the sign and opens.

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

describe.each(LANGUAGES)('the sign in the stamp and the history (%s)', (language) => {
  const base = AUDIT_EDIT_STAMP_BIKRI[0]?.stamp;
  if (base === undefined || base === null) throw new Error('the backend file has a stamp');
  const stamp: EditStamp = base;
  const when = {
    date: '11-10-2026',
    time: message(language, 'edit-time', { hour: '2', minute: '00', part: 'noon', meridian: 'pm' }),
    by: 'Maher Adnan',
  };

  it('the backend’s own stamp of the edit made in the fix carries the sign, and its history version too', () => {
    expect(stamp.sign).toBe('adnan');
    expect(AUDIT_EDIT_HISTORY_BIKRI.versions[0]?.sign).toBe('adnan');
    // The edit of an open day in the same file has none.
    expect(AUDIT_EDIT_STAMP_BIKRI[1]?.stamp?.sign ?? null).toBeNull();
    renderIn(language, <EditStampLine stamp={stamp} />);
    expect(visible(document.body.textContent)).toContain(
      message(language, 'glossary-dastakhat') + ': adnan',
    );
  });

  it('an edit made through a reopened day shows the sign after the name; an edit of an open day does not', () => {
    const signed = { ...stamp, sign: 'adnan' };
    const first = renderIn(language, <EditStampLine stamp={signed} />);
    expect(visible(document.body.textContent)).toContain(
      message(language, 'edit-stamp-signed', { ...when, sign: 'adnan' }),
    );
    first.unmount();
    renderIn(language, <EditStampLine stamp={{ ...stamp, sign: null }} />);
    const text = visible(document.body.textContent);
    expect(text).toContain(message(language, 'edit-stamp', when));
    expect(text).not.toContain(message(language, 'glossary-dastakhat'));
  });

  it('the row mark carries the sign in its tip', () => {
    renderIn(language, <EditedMark stamp={{ ...stamp, sign: 'adnan' }} />);
    const mark = screen.getByText(message(language, 'edit-mark')).closest('[data-edited-mark]');
    expect(visible(mark?.getAttribute('title'))).toBe(
      message(language, 'edit-mark-tip-signed', { ...when, sign: 'adnan' }),
    );
  });

  it('the edit history lists the sign of the version that has one, and only that one', async () => {
    const [first, ...rest] = AUDIT_EDIT_HISTORY_BIKRI.versions;
    if (first === undefined) throw new Error('the backend file has a version');
    vi.mocked(editApi.auditEditHistory).mockResolvedValue({
      ...AUDIT_EDIT_HISTORY_BIKRI,
      versions: [{ ...first, sign: 'adnan' }, ...rest],
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
      message(language, 'glossary-dastakhat') + ': adnan',
    );
    for (const other of versions.slice(1)) {
      expect(visible(other.textContent)).not.toContain(message(language, 'glossary-dastakhat'));
    }
  });
});

describe.each(LANGUAGES)('Settings, Security: the Owner’s sign (%s)', (language: Language) => {
  beforeEach(() => {
    resetClientState(language);
    signedIn(sessionUser({ language }));
  });

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
    const sign = screen.getByLabelText<HTMLInputElement>(
      named(message(language, 'settings-sign-new')),
    );
    return { ...app, user, password, sign };
  }

  it('keyboard only: the password, Enter, the new sign, Enter: it is sent as typed, and both are cleared', async () => {
    vi.mocked(settingsApi.settingsSetOwnerSign).mockResolvedValue(null);
    const { user, password, sign } = await openCard();
    await user.click(password);
    await user.keyboard('owner-pass{Enter}');
    await waitFor(() => {
      expect(sign).toHaveFocus();
    });
    await user.keyboard('sana{Enter}');
    await waitFor(() => {
      expect(settingsApi.settingsSetOwnerSign).toHaveBeenCalledWith({
        password: 'owner-pass',
        sign: 'sana',
      });
    });
    await waitFor(() => {
      expect(password.value).toBe('');
      expect(sign.value).toBe('');
    });
    expect(visible(document.body.textContent)).toContain(message(language, 'settings-sign-saved'));
  });

  it('a wrong password is told under the password, and nothing is cleared', async () => {
    vi.mocked(settingsApi.settingsSetOwnerSign).mockRejectedValue(
      fail({ code: 'ERR_WRONG_PASSWORD', field: 'password', values: { tries_left: 9 } }),
    );
    const { user, password, sign } = await openCard();
    await user.type(password, 'bad');
    await user.type(sign, 'sana{Enter}');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(
        message(language, 'err-wrong-password', { tries_left: 9 }),
      );
    });
    expect(sign.value).toBe('sana');
  });

  it('a sign the backend refuses is told under the sign', async () => {
    vi.mocked(settingsApi.settingsSetOwnerSign).mockRejectedValue(
      fail({ code: 'ERR_REQUIRED', field: 'sign' }),
    );
    const { user, password, sign } = await openCard();
    await user.type(password, 'owner-pass');
    await user.type(sign, 'x{Enter}');
    await waitFor(() => {
      expect(visible(document.body.textContent)).toContain(message(language, 'err-required'));
    });
    expect(settingsApi.settingsSetOwnerSign).toHaveBeenCalledTimes(1);
  });
});
